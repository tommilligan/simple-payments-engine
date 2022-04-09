pub enum TransferStatus {
    Transferred,
    Disputed,
    // either resolved or chargebacked
    Closed,
}

mod action {
    use crate::{ClientId, TransferId};

    pub struct Action {
        pub client_id: ClientId,
        pub transfer_id: TransferId,
        pub kind: ActionKind,
    }

    pub enum ActionKind {
        Transfer(Transfer),
        Dispute,
        Close(Close),
    }

    pub struct Transfer {
        pub value: f64,
    }

    pub struct Close {
        pub action: CloseAction,
    }

    pub enum CloseAction {
        Resolve,
        Chargeback,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(pub u16);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransferId(pub u32);

pub mod state {
    pub mod transfer {
        use crate::TransferId;
        use std::collections::HashMap;

        #[derive(Debug, Clone)]
        // Make sure this is small, we need to store one per transfer.
        #[repr(u8)]
        pub enum Status {
            Transferred,
            Disputed,
            // either resolved or chargebacked
            Closed,
        }

        #[derive(Debug, Clone)]
        pub struct State {
            pub value: f64,
            pub status: Status,
        }

        pub type Store = HashMap<TransferId, State>;
    }

    pub mod client {
        use crate::ClientId;
        use std::collections::HashMap;

        #[derive(Debug, Clone)]
        // Make sure this is small, we need to store one per client.
        #[repr(u8)]
        pub enum Access {
            Active,
            Frozen,
        }

        impl Default for Access {
            fn default() -> Self {
                Self::Active
            }
        }

        #[derive(Debug, Clone, Default)]
        pub struct State {
            pub total: f64,
            pub held: f64,
            pub access: Access,
        }

        impl State {
            pub fn available(&self) -> f64 {
                self.total - self.held
            }
        }

        pub type Store = HashMap<ClientId, State>;
    }
}

use state::client;
use state::transfer;
use thiserror::Error;

#[derive(Debug, Clone, Default)]
pub struct Store {
    client: client::Store,
    transfer: transfer::Store,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("not found: {transfer_id:?}")]
    NotFound { transfer_id: TransferId },
    #[error("conflict: {transfer_id:?}, {kind}: {description}")]
    Conflict {
        transfer_id: TransferId,
        kind: &'static str,
        description: String,
    },
    #[error("insufficient funds: {client_id:?}, {kind}")]
    InsufficientFunds {
        client_id: ClientId,
        kind: &'static str,
    },
}

impl Store {
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
                        return Err(Error::Conflict {
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
                    .ok_or_else(|| Error::NotFound { transfer_id })?;
                match transfer.status {
                    transfer::Status::Transferred => transfer.status = transfer::Status::Disputed,
                    _ => {
                        return Err(Error::Conflict {
                            transfer_id,
                            kind: "disputed non-transferred transfer",
                            description: format!(
                                "transfer should be transferred, found: {:?}",
                                transfer.status
                            ),
                        })
                    }
                }
            }
            action::ActionKind::Close(_) => {
                let transfer = self
                    .transfer
                    .get_mut(&transfer_id)
                    .ok_or_else(|| Error::NotFound { transfer_id })?;
                match transfer.status {
                    transfer::Status::Disputed => transfer.status = transfer::Status::Closed,
                    _ => {
                        return Err(Error::Conflict {
                            transfer_id,
                            kind: "closed non-disputed transfer",
                            description: format!(
                                "transfer should be disputed, found: {:?}",
                                transfer.status
                            ),
                        })
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
