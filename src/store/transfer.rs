use crate::types::TransferId;

#[derive(Debug, Clone, PartialEq, Eq)]
// Make sure this is small, we need to store one per transfer.
#[repr(u8)]
pub enum Status {
    Transferred,
    Disputed,
    Chargebacked,
}

#[derive(Debug, Clone)]
pub struct State {
    pub value: f64,
    pub status: Status,
}

pub type Store = std::collections::HashMap<TransferId, State>;
