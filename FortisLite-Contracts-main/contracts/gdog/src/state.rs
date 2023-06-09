use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Owner If None set, contract is frozen.
    pub owner: Option<Addr>,
    pub bdog_token_address: Addr,
    pub gdog_token_address: Addr,
    pub bdog_burn_amount: Uint128,
    pub gdog_sent_amount: Uint128,
    pub ratio: Uint128,
    pub step: Uint128,
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);
