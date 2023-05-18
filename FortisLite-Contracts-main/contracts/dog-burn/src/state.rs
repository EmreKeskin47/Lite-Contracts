use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Owner If None set, contract is frozen.
    pub owner: Option<Addr>,
    pub dog_token_address: Addr,
    pub bdog_token_address: Addr,
    pub dog_burn_amount: Uint128,
    pub bdog_sent_amount: Uint128,
    pub bdog_current_amount: Uint128,
    pub ratio: Uint128,
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);
