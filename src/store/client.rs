use crate::types::ClientId;

#[derive(Debug, Clone, PartialEq, Eq)]
// Make sure this is small, we need to store one per client.
#[repr(u8)]
pub enum Access {
    Active,
    Locked,
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

    pub fn is_locked(&self) -> bool {
        self.access == Access::Locked
    }
}

pub type Store = indexmap::IndexMap<ClientId, State>;
