#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_slice, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg};
use crate::state::{State, ASH_MINTED, CW20_DEPOSITED, HUAHUA_DEPOSITED, NATIVE_DEPOSITED, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:ash-house";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        allowed_operators: msg.allowed_operators,
        allowed_native: msg.allowed_native,
        allowed_cw20: msg.allowed_cw20,
        ash_cw20: msg.ash_cw20,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    CW20_DEPOSITED.save(deps.storage, &Uint128::new(0))?;
    HUAHUA_DEPOSITED.save(deps.storage, &Uint128::new(0))?;
    NATIVE_DEPOSITED.save(deps.storage, &Uint128::new(0))?;
    ASH_MINTED.save(deps.storage, &Uint128::new(0))?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MintAsh { native, amount } => {
            execute::mint_ash(deps, env, info, native, amount)
        }
        ExecuteMsg::EditState {
            allowed_operators,
            allowed_native,
            allowed_cw20,
            ash_cw20,
        } => execute::edit_state(
            deps,
            info,
            allowed_operators,
            allowed_native,
            allowed_cw20,
            ash_cw20,
        ),
        ExecuteMsg::Receive(cw20_receive_msg) => execute_receive(deps, info, cw20_receive_msg),
        ExecuteMsg::MintFromHuahua {} => execute::mint_from_huahua(deps, env, info),
    }
}
pub fn execute_receive(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveMsg = from_slice(&wrapper.msg)?;
    let amount = wrapper.amount;
    let sender = wrapper.sender;
    match msg {
        ReceiveMsg::MintCw20 { owner } => execute::mint_ash_cw20(deps, sender, info, owner, amount),
    }
}

pub mod execute {
    use super::*;
    use crate::state::{DepositInfo, ASH_MINTED, DEPOSIT_LIST};
    use cosmwasm_std::WasmMsg::Execute;
    use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, StdError, Uint128};
    use cw20::Cw20ExecuteMsg;
    use std::ops::Add;

    pub fn mint_ash(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        native: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        let mut msgs: Vec<CosmosMsg> = vec![];

        if !state.allowed_operators.contains(&info.sender.to_string()) {
            if info.funds.len() != 1 {
                return Err(ContractError::SendSingleNativeToken {});
            }

            let sent_fund = info.funds.get(0).unwrap();

            if !state.allowed_native.contains(&sent_fund.denom) {
                return Err(ContractError::DenomNotAllowed {
                    denom: sent_fund.clone().denom,
                });
            }

            if sent_fund.amount != amount && sent_fund.denom != native {
                return Err(ContractError::InputMismatch {});
            }

            let contract_balances = deps.querier.query_all_balances(&env.contract.address)?;
            let coin = contract_balances
                .iter()
                .find(|coin| state.allowed_native.contains(&coin.denom) && !coin.amount.is_zero());

            let coin = match coin {
                Some(coin) => match coin.amount >= amount {
                    true => Coin {
                        amount,
                        denom: coin.clone().denom,
                    },
                    false => coin.clone(),
                },
                None => {
                    return Err(ContractError::InsufficientBalance {});
                }
            };

            // we can now proceed to burning the coins
            // create a burn message
            let amount = [coin].to_vec();
            let burn_msg = BankMsg::Burn { amount };

            msgs.push(burn_msg.into())
        }

        //Transfer ash based on amount
        let cw20_transfer = Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount,
        };

        let transfer_msg = CosmosMsg::Wasm(Execute {
            contract_addr: state.ash_cw20.to_string(),
            msg: to_binary(&cw20_transfer)?,
            funds: vec![],
        });

        msgs.push(transfer_msg);
        NATIVE_DEPOSITED.update::<_, StdError>(deps.storage, |id| Ok(id.add(amount)))?;
        ASH_MINTED.update::<_, StdError>(deps.storage, |id| Ok(id.add(amount)))?;

        //check if exists
        let dep_info = DEPOSIT_LIST.load(deps.storage, info.sender.to_string());
        let deposit_info;
        if dep_info.is_err() {
            deposit_info = DepositInfo {
                addr: info.sender.to_string(),
                amount,
            };
        } else {
            let deposit_data = dep_info?;
            deposit_info = DepositInfo {
                addr: deposit_data.addr,
                amount: deposit_data.amount + amount,
            }
        }
        DEPOSIT_LIST.save(deps.storage, info.sender.to_string(), &deposit_info)?;

        // Build response
        let res = Response::new().add_messages(msgs);

        // return response
        Ok(res)
    }

    pub fn mint_from_huahua(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        let mut msgs: Vec<CosmosMsg> = vec![];

        if info.funds.len() != 1 {
            return Err(ContractError::SendSingleNativeToken {});
        }

        let sent_fund = info.funds.get(0).unwrap();

        if sent_fund.denom != "uhuahua" {
            return Err(ContractError::DenomNotAllowed {
                denom: sent_fund.clone().denom,
            });
        }
        let sent_amount = sent_fund.amount;
        let contract_balances = deps.querier.query_all_balances(&env.contract.address)?;
        let coin = contract_balances
            .iter()
            .find(|coin| state.allowed_native.contains(&coin.denom) && !coin.amount.is_zero());

        let coin = match coin {
            Some(coin) => match coin.amount >= sent_amount {
                true => Coin {
                    amount: sent_amount,
                    denom: coin.clone().denom,
                },
                false => coin.clone(),
            },
            None => {
                return Err(ContractError::InsufficientBalance {});
            }
        };

        // we can now proceed to burning the coins
        // create a burn message
        let amount = [coin].to_vec();
        let burn_msg = BankMsg::Burn { amount };

        msgs.push(burn_msg.into());

        //Transfer ash based on amount
        let cw20_transfer = Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount: sent_amount,
        };

        let transfer_msg = CosmosMsg::Wasm(Execute {
            contract_addr: state.ash_cw20.to_string(),
            msg: to_binary(&cw20_transfer)?,
            funds: vec![],
        });

        msgs.push(transfer_msg);
        HUAHUA_DEPOSITED.update::<_, StdError>(deps.storage, |id| Ok(id.add(sent_amount)))?;
        ASH_MINTED.update::<_, StdError>(deps.storage, |id| Ok(id.add(sent_amount)))?;

        // Build response
        let res = Response::new()
            .add_attribute("method", "execute_burn_daily_quota")
            .add_messages(msgs);

        // return response
        Ok(res)
    }

    pub fn mint_ash_cw20(
        deps: DepsMut,
        sender: String,
        info: MessageInfo,
        owner: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        let mut msgs: Vec<CosmosMsg> = vec![];

        if !state.allowed_operators.contains(&sender) {
            if !state.allowed_cw20.contains(&info.sender.to_string()) {
                return Err(ContractError::DenomNotAllowed { denom: sender });
            }

            // we can now proceed to burning the coins
            // create a burn message
            let burn_msg = Cw20ExecuteMsg::Burn { amount };
            let execute_burn = CosmosMsg::Wasm(Execute {
                contract_addr: info.sender.to_string(),
                msg: to_binary(&burn_msg)?,
                funds: vec![],
            });

            msgs.push(execute_burn);
        }

        //Transfer ash based on amount
        let cw20_transfer = Cw20ExecuteMsg::Transfer {
            recipient: owner,
            amount,
        };

        let transfer_msg = CosmosMsg::Wasm(Execute {
            contract_addr: state.ash_cw20.to_string(),
            msg: to_binary(&cw20_transfer)?,
            funds: vec![],
        });

        msgs.push(transfer_msg);
        CW20_DEPOSITED.update::<_, StdError>(deps.storage, |id| Ok(id.add(amount)))?;
        ASH_MINTED.update::<_, StdError>(deps.storage, |id| Ok(id.add(amount)))?;

        //check if exists
        let dep_info = DEPOSIT_LIST.load(deps.storage, info.sender.to_string().clone());
        let deposit_info;
        if dep_info.is_err() {
            deposit_info = DepositInfo {
                addr: info.sender.to_string().clone(),
                amount,
            };
        } else {
            let deposit_data = dep_info?;
            deposit_info = DepositInfo {
                addr: deposit_data.addr,
                amount: deposit_data.amount + amount,
            }
        }
        DEPOSIT_LIST.save(deps.storage, info.sender.to_string(), &deposit_info)?;

        // Build response
        let res = Response::new()
            .add_attribute("method", "execute_burn_daily_quota")
            .add_messages(msgs);

        // return response
        Ok(res)
    }

    pub fn edit_state(
        deps: DepsMut,
        info: MessageInfo,
        allowed_operators: Vec<String>,
        allowed_native: Vec<String>,
        allowed_cw20: Vec<String>,
        ash_cw20: Addr,
    ) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            if !state.allowed_operators.contains(&info.sender.to_string()) {
                return Err(ContractError::Unauthorized {});
            }

            state.allowed_operators = allowed_operators;
            state.allowed_native = allowed_native;
            state.allowed_cw20 = allowed_cw20;
            state.ash_cw20 = ash_cw20;
            Ok(state)
        })?;
        Ok(Response::new().add_attribute("action", "reset"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => to_binary(&query::state(deps)?),
        QueryMsg::GetContractBalance {} => to_binary(&query::balance(deps, env)?),
        QueryMsg::GetDepositDetails { start_after, limit } => {
            to_binary(&query::details_list(deps, start_after, limit)?)
        }
    }
}

pub mod query {
    use super::*;
    use crate::msg::{BalanceResponse, DepositDetailedInfoResponse, GetStateResponse};
    use crate::state::DEPOSIT_LIST;
    use cosmwasm_std::Order;
    use cw_storage_plus::Bound;

    const DEFAULT_LIMIT: u32 = 10;
    const MAX_LIMIT: u32 = 30;

    pub fn state(deps: Deps) -> StdResult<GetStateResponse> {
        let state = STATE.load(deps.storage)?;
        let ash = ASH_MINTED.load(deps.storage)?;
        let huahua = HUAHUA_DEPOSITED.load(deps.storage)?;
        let cw20 = CW20_DEPOSITED.load(deps.storage)?;
        let native = NATIVE_DEPOSITED.load(deps.storage)?;

        Ok(GetStateResponse {
            allowed_operators: state.allowed_operators,
            allowed_native: state.allowed_native,
            allowed_cw20: state.allowed_cw20,
            ash_cw20: state.ash_cw20,
            ash,
            cw20,
            huahua,
            native,
        })
    }

    pub fn balance(deps: Deps, env: Env) -> StdResult<BalanceResponse> {
        let state = STATE.load(deps.storage)?;
        let ash_addr = state.ash_cw20;
        let contract_balance = deps.querier.query_balance(env.contract.address, ash_addr)?;
        Ok(BalanceResponse {
            balance: contract_balance.amount,
        })
    }

    pub fn details_list(
        deps: Deps,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<DepositDetailedInfoResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(Bound::exclusive);

        let records: StdResult<Vec<_>> = DEPOSIT_LIST
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .collect();

        let rec = records?.into_iter().map(|r| r.1).collect();

        Ok(DepositDetailedInfoResponse { details: rec })
    }
}
