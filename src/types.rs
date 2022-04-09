pub mod action {
    use crate::types::{ClientId, TransferId};

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
