#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use crate::state::{Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:dog-burn";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg
        .owner
        .map_or(Ok(info.sender), |o| deps.api.addr_validate(&o))?;

    let config = Config {
        owner: Some(owner),
        dog_token_address: deps.api.addr_validate(&msg.dog_token_address)?,
        bdog_token_address: deps.api.addr_validate(&msg.bdog_token_address)?,
        dog_burn_amount: Uint128::zero(),
        bdog_sent_amount: Uint128::zero(),
        bdog_current_amount: Uint128::zero(),
        ratio: Uint128::new(10u128),
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig { new_owner } => execute_update_config(deps, info, new_owner),
        ExecuteMsg::Receive(msg) => try_receive(deps, info, msg),
        ExecuteMsg::WithdrawAll {} => try_withdraw_all(deps, info),
    }
}

fn calculate_ratio(burn_amount: Uint128, current_ratio: Uint128) -> Uint128 {
    let no_decimal: Uint128 = burn_amount
        .checked_div(Uint128::new(1_000_000u128))
        .unwrap();
    if no_decimal < Uint128::new(1_000_000u128) {
        current_ratio
    } else {
        Uint128::new(10u128)
            .checked_add(no_decimal.checked_div(Uint128::new(1_000_000u128)).unwrap())
            .unwrap_or(current_ratio)
    }
}

pub fn try_receive(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let mut cfg = CONFIG.load(deps.storage)?;
    let user_addr = &deps.api.addr_validate(&wrapper.sender)?;

    if info.sender == cfg.dog_token_address {
        //Get the dog amount
        let dog_received_amount = wrapper.amount;
        let mut bdog_send_amount = Uint128::zero();

        if dog_received_amount > Uint128::zero() {
            //Calculate ratio
            cfg.ratio = calculate_ratio(
                cfg.dog_burn_amount
                    .checked_add(dog_received_amount)
                    .unwrap_or(cfg.dog_burn_amount),
                cfg.ratio,
            );
            bdog_send_amount = cfg.ratio.checked_mul(dog_received_amount).unwrap();
        }

        //Update the number of burnt dog tokens
        cfg.dog_burn_amount = cfg
            .dog_burn_amount
            .checked_add(dog_received_amount)
            .unwrap_or(cfg.dog_burn_amount);
        //Update the bdog sent amount
        cfg.bdog_sent_amount = cfg
            .bdog_sent_amount
            .checked_add(bdog_send_amount)
            .unwrap_or(cfg.bdog_sent_amount);

        CONFIG.save(deps.storage, &cfg)?;

        //send bdog_send_amount, burn dog_received_amount
        return Ok(Response::new()
            .add_message(WasmMsg::Execute {
                contract_addr: cfg.bdog_token_address.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_addr.to_string(),
                    amount: bdog_send_amount,
                })?,
            })
            .add_message(WasmMsg::Execute {
                contract_addr: cfg.dog_token_address.into(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: dog_received_amount,
                })?,
            })
            .add_attributes(vec![
                attr("action", "send_bdog_burn_dog"),
                attr("address", user_addr),
                attr("bdog_amount", bdog_send_amount),
                attr("dog_amount", dog_received_amount),
            ]));
    } else if info.sender == cfg.bdog_token_address {
        //Just receive in contract cache and update config
        cfg.bdog_current_amount = cfg.bdog_current_amount + wrapper.amount;
        CONFIG.save(deps.storage, &cfg)?;

        return Ok(Response::new().add_attributes(vec![
            attr("action", "receive_bdog"),
            attr("address", user_addr),
            attr("bdog_amount", wrapper.amount),
        ]));
    } else {
        return Err(ContractError::UnacceptableToken {});
    }
}

pub fn check_owner(deps: &DepsMut, info: &MessageInfo) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let owner = cfg.owner.ok_or(ContractError::Unauthorized {})?;
    if info.sender != owner {
        return Err(ContractError::Unauthorized {});
    }
    Ok(Response::new().add_attribute("action", "check_owner"))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Option<String>,
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(&deps, &info)?;

    //test code for checking if check_owner works well
    // return Err(ContractError::InvalidInput {});
    // if owner some validated to addr, otherwise set to none
    let mut tmp_owner = None;
    if let Some(addr) = new_owner {
        tmp_owner = Some(deps.api.addr_validate(&addr)?)
    }

    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.owner = tmp_owner;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn try_withdraw_all(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    check_owner(&deps, &info)?;
    let mut cfg = CONFIG.load(deps.storage)?;

    let bdog_current_amount = cfg.bdog_current_amount;
    let bdog_token_address = cfg.bdog_token_address.clone();
    cfg.bdog_current_amount = Uint128::zero();

    CONFIG.save(deps.storage, &cfg)?;

    // create transfer cw20 msg
    let exec_cw20_transfer = WasmMsg::Execute {
        contract_addr: bdog_token_address.into(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: info.sender.clone().into(),
            amount: bdog_current_amount,
        })?,
        funds: vec![],
    };

    // return Ok(Response::new());
    return Ok(Response::new()
        .add_message(exec_cw20_transfer)
        .add_attributes(vec![
            attr("action", "bdog_withdraw_all"),
            attr("address", info.sender.clone()),
            attr("bdog_amount", bdog_current_amount),
        ]));
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: cfg.owner.map(|o| o.into()),
        dog_token_address: cfg.dog_token_address.into(),
        bdog_token_address: cfg.bdog_token_address.into(),
        dog_burn_amount: cfg.dog_burn_amount,
        bdog_sent_amount: cfg.bdog_sent_amount,
        bdog_current_amount: cfg.bdog_current_amount,
        ratio: cfg.ratio,
    })
}
