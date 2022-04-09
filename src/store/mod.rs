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
        match kind {
            action::ActionKind::Transfer(payload) => {
                // Ensure the client has sufficient funds (initialising if not found).
                // It's okay the client gets stored with a default state, even if the
                // transfer fails.
                let client = self.client.entry(client_id).or_default();

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
                let client = self.client.entry(client_id).or_default();
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
            action::ActionKind::Close(payload) => {
                let client = self.client.entry(client_id).or_default();
                let transfer = self
                    .transfer
                    .get_mut(&transfer_id)
                    .ok_or_else(|| Error::TransferNotFound { transfer_id })?;
                match transfer.status {
                    transfer::Status::Disputed => transfer.status = transfer::Status::Closed,
                    _ => {
                        return Err(Error::TransferConflict {
                            transfer_id,
                            kind: "closed non-disputed transfer",
                            description: format!(
                                "transfer should be disputed, found: {:?}",
                                transfer.status
                            ),
                        })
                    }
                };
                match payload.action {
                    action::CloseAction::Resolve => client.held -= transfer.value,
                    action::CloseAction::Chargeback => {
                        client.access = client::Access::Frozen;
                        client.held -= transfer.value;
                        client.total -= transfer.value;
                    }
                }
            }
        }
        Ok(())
    }
}
