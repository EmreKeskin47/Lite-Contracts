use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Contract balance is too small to execute")]
    InsufficientBalance {},

    #[error("Send single native token")]
    SendSingleNativeToken {},

    #[error("Given denom {denom} not allowed ")]
    DenomNotAllowed { denom: String },

    #[error("Sent funds do not match input")]
    InputMismatch {},
}
