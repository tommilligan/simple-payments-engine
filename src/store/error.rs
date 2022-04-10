use crate::{
    store::client,
    types::{ClientId, TransferId},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("not found: {transfer_id:?}")]
    TransferNotFound { transfer_id: TransferId },
    #[error("conflict: {transfer_id:?}, {kind}: {description}")]
    TransferConflict {
        transfer_id: TransferId,
        kind: &'static str,
        description: String,
    },
    #[error("insufficient funds: {client_id:?}, {kind}")]
    InsufficientFunds {
        client_id: ClientId,
        kind: &'static str,
    },
    #[error("client error: {0}")]
    Client(#[from] client::Error),
}
