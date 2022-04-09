pub enum TransferStatus {
    Transferred,
    Disputed,
    // either resolved or chargebacked
    Closed,
}

mod action {
    use crate::{ClientId, TransferId};

    pub enum Action {
        Transfer(Transfer),
        Dispute(Dispute),
        Close(Close),
    }

    pub struct Transfer {
        pub client: ClientId,
        pub id: TransferId,
        pub value: f64,
    }

    pub struct Dispute {
        pub client: ClientId,
        pub id: TransferId,
    }

    pub struct Close {
        pub client: ClientId,
        pub id: TransferId,
        pub action: CloseAction,
    }

    pub enum CloseAction {
        Resolve,
        Chargeback,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ClientId(pub u16);
#[derive(Debug, Clone, Copy)]
pub struct TransferId(pub u32);

mod state {
    mod transfer {
        use crate::TransferId;
        use std::collections::HashMap;

        // Make sure this is small, we need to store one per transfer.
        #[repr(u8)]
        pub enum Status {
            Transferred,
            Disputed,
            // either resolved or chargebacked
            Closed,
        }

        pub struct State {
            pub value: f64,
            pub status: Status,
        }

        pub type Store = HashMap<TransferId, State>;
    }

    mod client {
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
