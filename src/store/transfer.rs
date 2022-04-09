use crate::types::TransferId;
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
