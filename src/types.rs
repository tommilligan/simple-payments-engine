pub mod action {
    use crate::types::{ClientId, TransferId};

    #[derive(Debug, Clone, PartialEq)]
    pub struct Action {
        pub client_id: ClientId,
        pub transfer_id: TransferId,
        pub kind: ActionKind,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum ActionKind {
        Transfer(Transfer),
        Dispute,
        Close(Close),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Transfer {
        pub value: f64,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Close {
        pub action: CloseAction,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum CloseAction {
        Resolve,
        Chargeback,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(pub u16);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransferId(pub u32);
