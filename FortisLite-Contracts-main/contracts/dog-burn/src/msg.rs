use cosmwasm_schema::{cw_serde, QueryResponses};

use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    /// Owner if none set to info.sender.
    pub owner: Option<String>,
    pub dog_token_address: String,
    pub bdog_token_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        /// NewOwner if non sent, contract gets locked. Recipients can receive airdrops
        /// but owner cannot register new stages.
        new_owner: Option<String>,
    },
    Receive(Cw20ReceiveMsg),
    WithdrawAll {},
}

#[cw_serde]
pub enum ReceiveMsg {
    Dog {},
    Bdog {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub owner: Option<String>,
    pub dog_token_address: String,
    pub bdog_token_address: String,
    pub dog_burn_amount: Uint128,
    pub bdog_sent_amount: Uint128,
    pub bdog_current_amount: Uint128,
    pub ratio: Uint128,
}
