use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub allowed_operators: Vec<String>,
    pub allowed_native: Vec<String>,
    pub allowed_cw20: Vec<String>,
    pub ash_cw20: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct DepositInfo {
    pub addr: String,
    pub amount: Uint128,
}

pub const STATE: Item<State> = Item::new("state");

pub const CW20_DEPOSITED: Item<Uint128> = Item::new("huahua");
pub const HUAHUA_DEPOSITED: Item<Uint128> = Item::new("huahua");
pub const NATIVE_DEPOSITED: Item<Uint128> = Item::new("native");

pub const DEPOSIT_LIST: Map<String, DepositInfo> = Map::new("deposit_list");

pub const ASH_BALANCE: Item<Uint128> = Item::new("ash");
pub const ASH_MINTED: Item<Uint128> = Item::new("ash");
