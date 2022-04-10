use crate::types::{action, ClientId, TransferId};
use serde::{Deserialize, Serialize};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("bad action kind: {kind}")]
    BadActionKind { kind: String },
    #[error("missing amount for action {kind}")]
    MissingAmount { kind: String },
    #[error("invalid amount {value}")]
    InvalidAmount { value: f64 },
}

#[derive(Debug, Deserialize)]
pub struct InputRow {
    // type is a rust keyword, so rename here (better than r#... everywhere)
    #[serde(rename = "type")]
    pub kind: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
}

impl TryFrom<InputRow> for action::Action {
    type Error = Error;
    fn try_from(other: InputRow) -> Result<Self, Error> {
        let InputRow {
            kind,
            client,
            tx,
            amount,
        } = other;
        let client_id = ClientId(client);
        let transfer_id = TransferId(tx);
        let kind = match kind.as_str() {
            "deposit" | "withdrawal" => {
                let value = amount.ok_or_else(|| Error::MissingAmount { kind: kind.clone() })?;
                if !value.is_finite() || value < 0.0 {
                    return Err(Error::InvalidAmount { value });
                }
                let value = if &kind == "withdrawal" { -value } else { value };
                action::ActionKind::Transfer(action::Transfer { value, client_id })
            }
            "dispute" => action::ActionKind::Dispute,
            "resolve" => action::ActionKind::Settle(action::Settle {
                action: action::SettleAction::Resolve,
            }),
            "chargeback" => action::ActionKind::Settle(action::Settle {
                action: action::SettleAction::Chargeback,
            }),
            _ => return Err(Error::BadActionKind { kind }),
        };
        Ok(Self { transfer_id, kind })
    }
}

#[derive(Debug, Serialize)]
pub struct OutputRow {
    pub client: u16,
    pub available: String,
    pub held: String,
    pub total: String,
    pub locked: bool,
}
