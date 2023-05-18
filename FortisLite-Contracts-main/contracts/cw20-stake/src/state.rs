use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub allowed_operators: Vec<Addr>,
    pub unstaking_duration: Uint128,
    pub apr: String,
    pub bdog_ratio: Uint128,
    pub gdog_ratio: Uint128,
    pub token_address: Addr,
    pub token_source: Addr,
    pub reward_token_address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakeInfo {
    pub owner: Addr,
    pub stake_amount: Uint128,
    pub unstaking_amount: Uint128,
    pub reward_amount: Uint128,
    pub apr: String,
    pub stake_start_time: Uint128,
    pub reward_start_time: Uint128,
    pub unstaking_start_time: Uint128,
    pub reward_end_time: Uint128,
    pub unstaking_process: bool,
    pub unstake_end_time: Uint128,
}

pub const STATE: Item<State> = Item::new("state");
pub const STAKE_LIST: Map<Addr, StakeInfo> = Map::new("stake_list");
pub const STAKED_TOTAL: Item<Uint128> = Item::new("total_staked_amount");
pub const REWARD_TOTAL: Item<Uint128> = Item::new("reward_total");
