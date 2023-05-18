use crate::state::DepositInfo;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub allowed_operators: Vec<String>,
    pub allowed_native: Vec<String>,
    pub allowed_cw20: Vec<String>,
    pub ash_cw20: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    MintAsh {
        native: String,
        amount: Uint128,
    },
    MintFromHuahua {},
    Receive(Cw20ReceiveMsg),
    EditState {
        allowed_operators: Vec<String>,
        allowed_native: Vec<String>,
        allowed_cw20: Vec<String>,
        ash_cw20: Addr,
    },
}

#[cw_serde]
pub enum ReceiveMsg {
    MintCw20 { owner: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetStateResponse)]
    GetState {},
    #[returns(BalanceResponse)]
    GetContractBalance {},
    #[returns(DepositDetailedInfoResponse)]
    GetDepositDetails {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct GetStateResponse {
    pub allowed_operators: Vec<String>,
    pub allowed_native: Vec<String>,
    pub allowed_cw20: Vec<String>,
    pub ash_cw20: Addr,
    pub ash: Uint128,
    pub cw20: Uint128,
    pub huahua: Uint128,
    pub native: Uint128,
}

#[cw_serde]
pub struct DepositDetailedInfoResponse {
    pub details: Vec<DepositInfo>,
}

#[cw_serde]
pub struct BalanceResponse {
    pub balance: Uint128,
}
