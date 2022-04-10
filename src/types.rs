pub mod action {
    use crate::types::{ClientId, TransferId};

    #[derive(Debug, Clone, PartialEq)]
    pub struct Action {
        pub transfer_id: TransferId,
        pub kind: ActionKind,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum ActionKind {
        Transfer(Transfer),
        Dispute,
        Settle(Settle),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Transfer {
        pub client_id: ClientId,
        pub value: f64,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Settle {
        pub action: SettleAction,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum SettleAction {
        Resolve,
        Chargeback,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(pub u16);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransferId(pub u32);
