use cosmwasm_std::{DivideByZeroError, OverflowError, StdError, Uint128};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized {msg}")]
    Unauthorized { msg: String },

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Min unstaking time required")]
    MinUnstakingTimeRequired {},

    #[error("Unsupported  CW20")]
    InvalidToken {},

    #[error("Unstaking process must be started first")]
    UnstakingProcessIsNotStarted {},

    #[error("Requested unstake amount is larger than staked amount")]
    MoreThanStakeAmount {},

    #[error("Unstaking process is ongoing")]
    OngoingUnstakingProcess {},

    #[error("Claim amount greater than unstaked unstaked: {unstaked} and ask: {amount}")]
    AmountLargerThanUnstaked { unstaked: Uint128, amount: Uint128 },

    #[error("Claim amount greater than reward: {reward} and ask: {amount}")]
    AmountLargerThanReward { reward: Uint128, amount: Uint128 },

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("{0}")]
    DivideByZeroError(#[from] DivideByZeroError),
}
