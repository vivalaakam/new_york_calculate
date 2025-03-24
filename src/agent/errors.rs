use crate::Symbol;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum CalculateAgentError {
    #[error("Insufficient balance: available {available}, required {required}")]
    InsufficientBalance { available: f32, required: f32 },

    #[error("Insufficient asset balance for {symbol}: available {available}, required {required}")]
    InsufficientAssetBalance {
        symbol: Symbol,
        available: f32,
        required: f32,
    },

    #[error("Unknown command")]
    UnknownCommand,
}
