pub enum TransferStatus {
    Transferred,
    Disputed,
    // either resolved or chargebacked
    Closed,
}

mod action {
    use crate::{ClientId, TransferId};

    pub struct Action {
        pub transfer_id: TransferId,
        pub kind: ActionKind,
    }

    pub enum ActionKind {
        Transfer(Transfer),
        Dispute(Dispute),
        Close(Close),
    }

    pub struct Transfer {
        pub client: ClientId,
        pub value: f64,
    }

    pub struct Dispute {
        pub client: ClientId,
    }

    pub struct Close {
        pub client: ClientId,
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
        use crate::action;
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

        #[derive(Debug, Clone, Default)]
        pub struct Store {
            pub inner: HashMap<TransferId, State>,
        }

        use thiserror::Error;

        #[derive(Error, Debug)]
        pub enum Error {
            #[error("not found: {transfer_id:?}")]
            NotFound { transfer_id: TransferId },
            #[error("conflict:  {transfer_id:?}, {kind}: {description}")]
            Conflict {
                transfer_id: TransferId,
                kind: &'static str,
                description: String,
            },
        }

        pub type Result<T> = std::result::Result<T, Error>;

        impl Store {
            pub fn apply(&mut self, action: action::Action) -> Result<()> {
                let action::Action { transfer_id, kind } = action;
                match kind {
                    action::ActionKind::Transfer(transfer) => {
                        use std::collections::hash_map::Entry;
                        match self.inner.entry(transfer_id) {
                            Entry::Occupied(_) => {
                                return Err(Error::Conflict {
                                    transfer_id,
                                    kind: "transfer exists",
                                    description: "a transfer already exists with this id"
                                        .to_owned(),
                                })
                            }
                            Entry::Vacant(entry) => {
                                entry.insert(State {
                                    value: transfer.value,
                                    status: Status::Transferred,
                                });
                            }
                        }
                    }
                    action::ActionKind::Dispute(_) => {
                        let transfer = self
                            .inner
                            .get_mut(&transfer_id)
                            .ok_or_else(|| Error::NotFound { transfer_id })?;
                        match transfer.status {
                            Status::Transferred => transfer.status = Status::Disputed,
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
                            .inner
                            .get_mut(&transfer_id)
                            .ok_or_else(|| Error::NotFound { transfer_id })?;
                        match transfer.status {
                            Status::Disputed => transfer.status = Status::Closed,
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
    }

    pub mod client {
        use crate::ClientId;
        use std::collections::HashMap;

        // Make sure this is small, we need to store one per client.
        #[repr(u8)]
        pub enum Access {
            Active,
            Frozen,
        }

        pub struct State {
            pub available: f64,
            pub held: f64,
            pub access: Access,
        }

        impl State {
            pub fn total(&self) -> f64 {
                self.available + self.held
            }
        }

        pub type Store = HashMap<ClientId, State>;
    }
}

use state::client::Store as ClientStore;
use state::transfer::Store as TransferStore;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
