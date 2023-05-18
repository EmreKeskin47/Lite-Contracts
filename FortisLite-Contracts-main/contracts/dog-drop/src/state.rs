use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub allowed_operators: Vec<String>,
    pub cw20: String,
}

pub const STATE: Item<State> = Item::new("state");
pub const AIRDROP: Map<Addr, Uint128> = Map::new("whitelist");
pub const SIZE:Item<u32> = Item::new("size");
pub const CLAIMED:Item<u32> = Item::new("claimed");
