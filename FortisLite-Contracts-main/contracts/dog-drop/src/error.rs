use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("cw20 error : {method}")]
    Cw20ContractError { method: String },

    #[error("DuplicateMember: {0}")]
    DuplicateMember(String),

    #[error("AlreadyClaimed: {0}")]
    AlreadyClaimed(String),

    #[error("NoMemberFound: {0}")]
    NoMemberFound(String),

    #[error("InsufficientBalance")]
    InsufficientBalance(),
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
