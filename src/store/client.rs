use crate::types::ClientId;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("client is locked: {client_id:?}")]
    ClientLocked { client_id: ClientId },
}

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

#[derive(Debug, Clone, Default)]
pub struct Store {
    inner: indexmap::IndexMap<ClientId, State>,
}

impl Store {
    /// Get a client state, creating and returning a default entry if it does not exist.
    pub fn get_or_default_mut(&mut self, client_id: ClientId) -> Result<&mut State, Error> {
        let client = self.inner.entry(client_id).or_default();
        if client.is_locked() {
            return Err(Error::ClientLocked { client_id });
        };
        Ok(client)
    }

    pub fn into_inner(self) -> indexmap::IndexMap<ClientId, State> {
        self.inner
    }
}
