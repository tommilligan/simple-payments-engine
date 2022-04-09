use crate::types::action;

pub mod client;
pub mod error;
pub mod transfer;

use error::Error;

#[derive(Debug, Clone, Default)]
pub struct Store {
    client: client::Store,
    transfer: transfer::Store,
}

impl Store {
    pub fn into_client_store(self) -> client::Store {
        self.client
    }

    pub fn apply(&mut self, action: action::Action) -> Result<(), Error> {
        let action::Action {
            client_id,
            transfer_id,
            kind,
        } = action;

        // Ensure the client has sufficient funds (initialising if not found).
        // It's okay the client gets stored with a default state, even if the
        // transfer fails.
        let client = self.client.entry(client_id).or_default();
        if client.is_locked() {
            return Err(Error::ClientLocked { client_id });
        }

        match kind {
            action::ActionKind::Transfer(payload) => {
                // If this is a withdrawal, ensure the client has enough funds.
                //
                // A client should never have negative available funds without
                // having their account frozen (due to a chargeback), but it's
                // cheap to check.
                if payload.value < 0.0 && client.available() + payload.value < 0.0 {
                    return Err(Error::InsufficientFunds {
                        client_id,
                        kind: "withdrawal would result in negative available funds",
                    });
                }

                // Then, store the transfer.
                use std::collections::hash_map::Entry;
                match self.transfer.entry(transfer_id) {
                    Entry::Occupied(_) => {
                        return Err(Error::TransferConflict {
                            transfer_id,
                            kind: "transfer exists",
                            description: "a transfer already exists with this id".to_owned(),
                        })
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(transfer::State {
                            value: payload.value,
                            status: transfer::Status::Transferred,
                        });
                    }
                };

                // And update the client.
                client.total += payload.value
            }
            action::ActionKind::Dispute => {
                let transfer = self
                    .transfer
                    .get_mut(&transfer_id)
                    .ok_or_else(|| Error::TransferNotFound { transfer_id })?;
                match transfer.status {
                    transfer::Status::Transferred => transfer.status = transfer::Status::Disputed,
                    _ => {
                        return Err(Error::TransferConflict {
                            transfer_id,
                            kind: "disputed non-transferred transfer",
                            description: format!(
                                "transfer should be transferred, found: {:?}",
                                transfer.status
                            ),
                        })
                    }
                }

                // Here we assume that the client has sufficient funds to hold.
                //
                // This is by construction (assuming no chargebacks have occurred).
                client.held += transfer.value
            }
            action::ActionKind::Settle(payload) => {
                let transfer = self
                    .transfer
                    .get_mut(&transfer_id)
                    .ok_or_else(|| Error::TransferNotFound { transfer_id })?;
                if transfer.status != transfer::Status::Disputed {
                    return Err(Error::TransferConflict {
                        transfer_id,
                        kind: "settled non-disputed transfer",
                        description: format!(
                            "transfer should be disputed, found: {:?}",
                            transfer.status
                        ),
                    });
                };
                // Here, we assume the client id on the action is correct.
                // I would very much not trust this in production code, and
                // would store the client id of the original transfer, and
                // check it here before taking action.
                match payload.action {
                    action::SettleAction::Resolve => {
                        transfer.status = transfer::Status::Transferred;
                        client.held -= transfer.value;
                    }
                    action::SettleAction::Chargeback => {
                        transfer.status = transfer::Status::Chargebacked;
                        client.access = client::Access::Locked;
                        client.held -= transfer.value;
                        client.total -= transfer.value;
                    }
                }
            }
        }
        Ok(())
    }
}
