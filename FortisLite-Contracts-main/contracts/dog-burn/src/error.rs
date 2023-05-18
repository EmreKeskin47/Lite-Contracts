use cosmwasm_std::StdError;
use cw_utils::{Expiration, Scheduled};
use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Hex(#[from] FromHexError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("InvalidInput")]
    InvalidInput {},

    #[error("Not Dog or bDog token")]
    UnacceptableToken {},

    #[error("Not enough Dog")]
    NotEnoughDog {},

    #[error("Not enough bDog")]
    NotEnoughbDog {},

    #[error("Already claimed")]
    Claimed {},

    #[error("Wrong length")]
    WrongLength {},

    #[error("Verification failed")]
    VerificationFailed {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("Airdrop stage {stage} expired at {expiration}")]
    StageExpired { stage: u8, expiration: Expiration },

    #[error("Airdrop stage {stage} begins at {start}")]
    StageNotBegun { stage: u8, start: Scheduled },
}
