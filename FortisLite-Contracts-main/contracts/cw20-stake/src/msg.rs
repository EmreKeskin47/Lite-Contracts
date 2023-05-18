use crate::state::StakeInfo;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub allowed_operators: Vec<Addr>,
    pub unstaking_duration: Uint128,
    pub apr: String,
    pub bdog_ratio: Uint128,
    pub gdog_ratio: Uint128,
    pub token_address: Addr,
    pub token_source: Addr,
    pub reward_token_address: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    StartUnstake {
        amount: Uint128,
    },
    ClaimUnstaked {
        amount: Uint128,
    },
    ClaimReward {
        amount: Uint128,
    },
    EditState {
        allowed_operators: Option<Vec<Addr>>,
        token_address: Option<Addr>,
        unstaking_duration: Option<Uint128>,
        apr: Option<String>,
        bdog_ratio: Option<Uint128>,
        gdog_ratio: Option<Uint128>,
        token_source: Option<Addr>,
        reward_token_address: Option<Addr>,
    },
}

#[cw_serde]
pub enum ReceiveMsg {
    Stake {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetUserRewardResponse)]
    GetUserReward { addr: Addr },
    #[returns(GetStakeResponse)]
    GetUserStakeInfo { addr: Addr },
    #[returns(GetStateResponse)]
    GetState {},
    #[returns(StakeListResponse)]
    RangeStakeList {
        start_after: Option<Addr>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct GetUserRewardResponse {
    pub amount: Uint128,
}

#[cw_serde]
pub struct GetStateResponse {
    pub allowed_operators: Vec<Addr>,
    pub token_address: Addr,
    pub unstaking_duration: Uint128,
    pub apr: String,
    pub token_source: Addr,
    pub total_staked: Uint128,
    pub total_reward: Uint128,
    pub reward_token_address: Addr,
}

#[cw_serde]
pub struct StakeListResponse {
    pub stake_list: Vec<StakeInfo>,
}

#[cw_serde]
pub struct GetStakeResponse {
    pub info: StakeInfo,
}
