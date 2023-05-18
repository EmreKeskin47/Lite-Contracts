#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, AIRDROP, STATE, SIZE, CLAIMED};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:dog-drop";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use cw20::Cw20ExecuteMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        cw20: msg.cw20,
        allowed_operators: msg.allowed_operators,
    };

    let mut filtered_members = msg.members;

    // remove duplicate members
    filtered_members.sort_unstable();
    filtered_members.dedup();

    //True is set for all members, they can all claim their dragon
    for (member,amount) in filtered_members.into_iter() {
        let addr = deps.api.addr_validate(&member.clone())?;
        AIRDROP.save(deps.storage, addr, &amount)?;
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    SIZE.save(deps.storage, &0)?;
    CLAIMED.save(deps.storage, &0)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Claim {} => execute::execute_claim(deps, env, info),
        ExecuteMsg::AddMembers { members } => execute::add_members(deps, env, info, members),
        ExecuteMsg::RemoveMembers { members } => execute::remove_members(deps, env, info, members),
        ExecuteMsg::EditState {
            cw20,
            allowed_operators,
        } => execute::edit_state(
            deps,
            info,
            cw20,
            allowed_operators,
        ),
    }
}

pub mod execute {
    use std::ops::{Add, Sub};
    use super::*;
    use cosmwasm_std::WasmMsg::Execute;
    use cosmwasm_std::{CosmosMsg, StdError, Uint128};

    pub fn execute_claim(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;

        // if address do not exists in whitelist
        let addr = deps.api.addr_validate(info.sender.as_str())?;
        if !AIRDROP.has(deps.storage, addr.clone()) {
            return Err(ContractError::NoMemberFound(addr.to_string()));
        }

        // if dog token is already claimed
        let can_claim = AIRDROP.load(deps.storage, addr.clone())?;
        if can_claim == Uint128::new(0) {
            return Err(ContractError::AlreadyClaimed(addr.to_string()));
        }

        let transfer_msg = Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount: can_claim,
        };
        let transfer = CosmosMsg::Wasm(Execute {
            contract_addr: state.clone().cw20,
            msg: to_binary(&transfer_msg)?,
            funds: vec![],
        });

        AIRDROP.save(deps.storage, addr.clone(), &Uint128::new(0))?;
        CLAIMED.update::<_, StdError>(deps.storage, |id| Ok(id.add(1)))?;

        Ok(Response::new()
            .add_attribute("action", "increment")
            .add_message(transfer))
    }

    pub fn edit_state(
        deps: DepsMut,
        info: MessageInfo,
        cw20: String,
        allowed_operators: Vec<String>,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        if !state.allowed_operators.contains(&info.sender.to_string()) {
            return Err(ContractError::Unauthorized {});
        }

        let new = State {
            allowed_operators,
            cw20,
        };

        STATE.save(deps.storage, &new)?;
        Ok(Response::new().add_attribute("action", "edit-state "))
    }

    pub fn add_members(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        mut members: Vec<(String,Uint128)>,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;

        if !state.allowed_operators.contains(&info.sender.to_string()) {
            return Err(ContractError::Unauthorized {});
        }

        // remove duplicate members
        members.sort_unstable();
        members.dedup();

        for (add, amount) in members.into_iter() {
            let addr = deps.api.addr_validate(&add)?;
            if AIRDROP.has(deps.storage, addr.clone()) {
                return Err(ContractError::DuplicateMember(addr.to_string()));
            }
            AIRDROP.save(deps.storage, addr, &amount)?;
            SIZE.update::<_, StdError>(deps.storage, |id| Ok(id.add(1)))?;

        }

        Ok(Response::new()
            .add_attribute("action", "add_members")
            .add_attribute("sender", info.sender))
    }

    pub fn remove_members(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        members: Vec<(String,Uint128)>,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;

        if !state.allowed_operators.contains(&info.sender.to_string()) {
            return Err(ContractError::Unauthorized {});
        }

        for (remove,_bal) in members.into_iter() {
            let addr = deps.api.addr_validate(&remove)?;
            if !AIRDROP.has(deps.storage, addr.clone()) {
                return Err(ContractError::NoMemberFound(addr.to_string()));
            }
            AIRDROP.remove(deps.storage, addr);
            SIZE.update::<_, StdError>(deps.storage, |id| Ok(id.sub(1)))?;
        }

        Ok(Response::new()
            .add_attribute("action", "remove_members")
            .add_attribute("sender", info.sender))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => to_binary(&query::state(deps)?),
        QueryMsg::Members { start_after, limit } => {
            to_binary(&query::query_members(deps, start_after, limit)?)
        }
        QueryMsg::IsMember { address } => to_binary(&query::is_member(deps, address)?),
    }
}

pub mod query {
    use super::*;
    use crate::msg::{IsMemberResponse, MembersResponse, StateResponse};
    use cosmwasm_std::Order;
    use cw_storage_plus::Bound;
    use cw_utils::maybe_addr;

    const PAGINATION_DEFAULT_LIMIT: u32 = 25;
    const PAGINATION_MAX_LIMIT: u32 = 100;

    pub fn state(deps: Deps) -> StdResult<StateResponse> {
        let state = STATE.load(deps.storage)?;
        let size = SIZE.load(deps.storage)?;
        let claimed = CLAIMED.load(deps.storage)?;
        Ok(StateResponse {
            cw20: state.cw20,
            allowed_operators: state.allowed_operators,
            size,
            claimed,
        })
    }

    pub fn query_members(
        deps: Deps,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<MembersResponse> {
        let limit = limit
            .unwrap_or(PAGINATION_DEFAULT_LIMIT)
            .min(PAGINATION_MAX_LIMIT) as usize;
        let start_addr = maybe_addr(deps.api, start_after)?;
        let start = start_addr.map(Bound::exclusive);
        let members = AIRDROP
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|addr| addr.unwrap().0.to_string())
            .collect::<Vec<String>>();

        Ok(MembersResponse { members })
    }

    pub fn is_member(deps: Deps, address: String) -> StdResult<IsMemberResponse> {
        let addr = deps.api.addr_validate(&address)?;
        let res = AIRDROP.has(deps.storage, addr);
        Ok(IsMemberResponse { is_member: res })

    }
}
