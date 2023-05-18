#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
    WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw20::Cw20ReceiveMsg;

use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

use cw20_base::msg::ExecuteMsg as Cw20ExecuteMsg;

// Version info, for migration info
const CONTRACT_NAME: &str = "bdogburn";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg
        .owner
        .map_or(Ok(info.sender), |o| deps.api.addr_validate(&o))?;

    let config = Config {
        owner: Some(owner),
        bdog_token_address: msg.bdog_token_address.clone(),
        gdog_token_address: msg.gdog_token_address.clone(),
        bdog_burn_amount: Uint128::zero(),
        gdog_sent_amount: Uint128::zero(),
        ratio: Uint128::from(Uint128::new(1000u128)),
        step: Uint128::from(1u128),
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
        ExecuteMsg::UpdateConfig {
            new_owner,
            gdog_token_address,
            bdog_token_address,
        } => execute_update_config(
            deps,
            info,
            new_owner,
            gdog_token_address,
            bdog_token_address,
        ),
        ExecuteMsg::Receive(msg) => try_receive(deps, info, msg),
    }
}

fn calculate_step(total_amount: Uint128, current_step: Uint128) -> Uint128 {
    let mut step_count: Uint128 = Uint128::new(1u128);
    let mut current_amount: Uint128 = Uint128::zero();
    let no_decimal: Uint128 = total_amount.checked_div(Uint128::new(1000000u128)).unwrap();
    loop {
        current_amount = current_amount
            .checked_add(Uint128::new(1000u128))
            .unwrap()
            .checked_add(step_count)
            .unwrap()
            .checked_sub(Uint128::new(1u128))
            .unwrap_or(current_step);
        step_count = step_count.checked_add(Uint128::new(1u128)).unwrap();

        if current_amount >= no_decimal {
            break;
        }
    }
    return step_count
        .checked_sub(Uint128::new(1u128))
        .unwrap_or(current_step);
}

pub fn try_receive(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let mut cfg = CONFIG.load(deps.storage)?;
    let user_addr = &deps.api.addr_validate(&wrapper.sender)?;

    if info.sender == cfg.bdog_token_address {
        //Get the gdog amount
        let bdog_received_amount = wrapper.amount;
        let mut gdog_send_amount = Uint128::zero();

        if bdog_received_amount > Uint128::zero() {
            //Calculate ratio
            cfg.step = calculate_step(
                cfg.bdog_burn_amount
                    .checked_add(bdog_received_amount)
                    .unwrap_or(cfg.bdog_burn_amount),
                cfg.step,
            );
            cfg.ratio = Uint128::new(1000u128)
                .checked_add(cfg.step)
                .unwrap()
                .checked_sub(Uint128::new(1u128))
                .unwrap_or(cfg.ratio);
            gdog_send_amount = bdog_received_amount.checked_div(cfg.ratio).unwrap();
        }

        //Update the number of burnt bdog tokens
        cfg.bdog_burn_amount = cfg
            .bdog_burn_amount
            .checked_add(bdog_received_amount)
            .unwrap_or(cfg.bdog_burn_amount);
        //Update the gdog sent amount
        cfg.gdog_sent_amount = cfg
            .gdog_sent_amount
            .checked_add(gdog_send_amount)
            .unwrap_or(cfg.gdog_sent_amount);

        CONFIG.save(deps.storage, &cfg)?;

        //send gdog and burn bdog
        return Ok(Response::new()
            .add_message(WasmMsg::Execute {
                contract_addr: cfg.gdog_token_address.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: user_addr.to_string(),
                    amount: gdog_send_amount,
                })?,
            })
            .add_message(WasmMsg::Execute {
                contract_addr: cfg.bdog_token_address.into(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: bdog_received_amount,
                })?,
            })
            .add_attributes(vec![
                attr("action", "send_bdog_burn_dog"),
                attr("address", user_addr),
                attr("bdog_amount", bdog_received_amount),
                attr("gdog_amount", gdog_send_amount),
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
    gdog_token_address: Option<Addr>,
    bdog_token_address: Option<Addr>,
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

    if let Some(new_gdog_address) = gdog_token_address {
        CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
            exists.gdog_token_address = new_gdog_address;
            Ok(exists)
        })?;
    }

    if let Some(new_bdog_address) = bdog_token_address {
        CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
            exists.bdog_token_address = new_bdog_address;
            Ok(exists)
        })?;
    }

    Ok(Response::new().add_attribute("action", "update_config"))
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
        bdog_token_address: cfg.bdog_token_address.into(),
        gdog_token_address: cfg.gdog_token_address.into(),
        bdog_burn_amount: cfg.bdog_burn_amount,
        gdog_sent_amount: cfg.gdog_sent_amount,
        ratio: cfg.ratio,
        step: cfg.step,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}
