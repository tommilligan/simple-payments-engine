use crate::types::TransferId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Make sure this is small, we need to store one per transfer.
#[repr(u8)]
pub enum Status {
    Transferred,
    Disputed,
    Chargebacked,
}

#[derive(Debug, Clone, Copy)]
// Pack the struct smaller, so we can store more of them in memory.
// Tradeoff - we make the type Copy, but gain 43% memory.
#[repr(packed(1))]
pub struct State {
    pub value: f64,
    pub status: Status,
}

pub type Store = std::collections::HashMap<TransferId, State>;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn size_of() {
        assert_eq!(std::mem::size_of::<Status>(), 1);
        assert_eq!(std::mem::size_of::<f64>(), 8);
        // verify we pack the state as small as possible
        // without custom packing, this would default to 16
        assert_eq!(std::mem::size_of::<State>(), 9);
    }
}
