use crate::types::ClientId;
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
