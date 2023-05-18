use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub members: Vec<(String,Uint128)>,
    pub cw20: String,
    pub allowed_operators: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Claim {},
    EditState {
        cw20: String,
        allowed_operators: Vec<String>,
    },
    AddMembers {
        members: Vec<(String,Uint128)>,
    },
    RemoveMembers {
        members: Vec<(String,Uint128)>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StateResponse)]
    GetState {},
    #[returns(MembersResponse)]
    Members {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(IsMemberResponse)]
    IsMember { address: String },
}

#[cw_serde]
pub struct StateResponse {
    pub cw20: String,
    pub allowed_operators: Vec<String>,
    pub size: u32,
    pub claimed: u32,
}

#[cw_serde]
pub struct MembersResponse {
    pub members: Vec<String>,
}
#[cw_serde]
pub struct IsMemberResponse {
    pub is_member: bool,
}

#[cw_serde]
pub struct BalanceResponse {
    pub balance: Uint128,
}
