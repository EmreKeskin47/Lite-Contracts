#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;

use crate::error::ContractError;
use crate::helper::calculate_stake_reward;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg};
use crate::state::{StakeInfo, State, REWARD_TOTAL, STAKED_TOTAL, STAKE_LIST, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-stake";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        allowed_operators: msg.allowed_operators,
        unstaking_duration: msg.unstaking_duration,
        apr: msg.apr,
        bdog_ratio: msg.bdog_ratio,
        gdog_ratio: msg.gdog_ratio,
        token_address: msg.token_address,
        token_source: msg.token_source,
        reward_token_address: msg.reward_token_address,
    };
    STATE.save(deps.storage, &state)?;
    STAKED_TOTAL.save(deps.storage, &Uint128::new(0))?;
    REWARD_TOTAL.save(deps.storage, &Uint128::new(0))?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

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
        ExecuteMsg::StartUnstake { amount } => execute::start_unstake(deps, env, info, amount),
        ExecuteMsg::ClaimReward { amount } => execute::claim_reward(deps, env, info, amount),
        ExecuteMsg::ClaimUnstaked { amount } => execute::claim_unstaked(deps, info, env, amount),
        ExecuteMsg::EditState {
            allowed_operators,
            token_address,
            unstaking_duration,
            apr,
            bdog_ratio,
            gdog_ratio,
            token_source,
            reward_token_address,
        } => execute::edit_state(
            deps,
            env,
            info,
            allowed_operators,
            token_address,
            unstaking_duration,
            apr,
            bdog_ratio,
            gdog_ratio,
            token_source,
            reward_token_address,
        ),
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
    }
}

pub fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.token_address {
        return Err(ContractError::InvalidToken {});
    }
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    let sender = deps.api.addr_validate(&wrapper.sender)?;
    match msg {
        ReceiveMsg::Stake {} => execute::stake(deps, env, sender, wrapper.amount),
    }
}

pub mod execute {
    use super::*;
    use cosmwasm_std::{Addr, Order, SubMsg};
    use cw20::{Cw20Contract, Cw20ExecuteMsg};

    pub fn stake(
        deps: DepsMut,
        env: Env,
        sender: Addr,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let state = STATE.load(deps.storage)?;
        let now = env.block.time.seconds() as u128;
        let time = Uint128::new(now);

        let stake_info = STAKE_LIST.load(deps.storage, sender.clone());
        let stake;
        let new_apr =
            state.gdog_ratio / state.bdog_ratio * Uint128::new(state.apr.parse::<u128>().unwrap());
        if stake_info.is_err() {
            //No previous stake data exists
            stake = StakeInfo {
                owner: sender.clone(),
                stake_amount: amount,
                apr: new_apr.to_string(),
                unstaking_amount: Uint128::zero(),
                reward_amount: Uint128::zero(),
                stake_start_time: time,
                reward_start_time: time,
                unstaking_start_time: Uint128::zero(),
                unstaking_process: false,
                reward_end_time: Uint128::zero(),
                unstake_end_time: Uint128::zero(),
            };
        } else {
            let stake_data = stake_info?;
            let curr_reward = calculate_stake_reward(new_apr, stake_data.clone(), now);
            stake = StakeInfo {
                owner: sender.clone(),
                stake_amount: amount + stake_data.stake_amount,
                apr: new_apr.to_string(),
                stake_start_time: time,
                reward_start_time: time,
                unstaking_start_time: Uint128::zero(),
                unstaking_process: false,
                reward_end_time: Uint128::zero(),
                unstaking_amount: stake_data.unstaking_amount,
                reward_amount: curr_reward + stake_data.reward_amount,
                unstake_end_time: stake_data.unstake_end_time,
            };
        }
        let total = STAKED_TOTAL.load(deps.storage)?;
        STAKE_LIST.save(deps.storage, sender, &stake)?;
        STAKED_TOTAL.save(deps.storage, &(total + amount))?;
        Ok(Response::new())
    }

    pub fn start_unstake(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let mut stake_info = STAKE_LIST.load(deps.storage, info.sender.clone())?;
        let state = STATE.load(deps.storage)?;
        if stake_info.stake_amount < amount {
            return Err(ContractError::MoreThanStakeAmount {});
        }
        if stake_info.owner != info.sender.to_string() {
            return Err(ContractError::Unauthorized {
                msg: "only owner can unstake".to_string(),
            });
        }
        let now = env.block.time.seconds() as u128;
        let time = Uint128::new(now);
        let apr =
            state.gdog_ratio / state.bdog_ratio * Uint128::new(state.apr.parse::<u128>().unwrap());
        let curr_reward = calculate_stake_reward(apr, stake_info.clone(), now);

        let remaining_stake_balance = stake_info.stake_amount - amount;
        stake_info.unstaking_process = true;
        stake_info.unstaking_start_time = time;
        stake_info.reward_end_time = time;
        stake_info.unstaking_amount = stake_info.unstaking_amount + amount;
        stake_info.unstake_end_time = time + state.unstaking_duration;
        stake_info.stake_amount = remaining_stake_balance;

        let total = STAKED_TOTAL.load(deps.storage)?;
        STAKED_TOTAL.save(deps.storage, &(total - amount))?;

        stake_info.reward_amount = stake_info.reward_amount + curr_reward;
        STAKE_LIST.save(deps.storage, info.sender, &stake_info)?;
        Ok(Response::new().add_attribute("action", "start_unstake"))
    }

    pub fn claim_unstaked(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        let mut stake_info = STAKE_LIST.load(deps.storage, info.sender.clone())?;
        let state = STATE.load(deps.storage)?;

        if !stake_info.unstaking_process {
            return Err(ContractError::UnstakingProcessIsNotStarted {});
        }
        let now = env.block.time.seconds() as u128;
        let time = Uint128::new(now);
        if time - stake_info.unstaking_start_time < state.unstaking_duration
            || stake_info.unstaking_start_time == Uint128::zero()
        {
            return Err(ContractError::MinUnstakingTimeRequired {});
        }

        if stake_info.unstaking_amount < amount {
            return Err(ContractError::AmountLargerThanUnstaked {
                unstaked: stake_info.unstaking_amount,
                amount,
            });
        }

        stake_info.unstaking_amount = stake_info.unstaking_amount - amount;
        if stake_info.unstaking_amount == Uint128::zero() {
            stake_info.unstaking_start_time = Uint128::zero();
            stake_info.unstaking_process = false;
            if stake_info.stake_amount == Uint128::zero() {
                stake_info.stake_start_time = Uint128::zero();
                stake_info.reward_start_time = Uint128::zero();
            }
        }

        let cw20_execute_send = Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount,
        };
        let reward_send_msg = Cw20Contract(state.token_address)
            .call(cw20_execute_send)
            .map_err(ContractError::Std)?;

        STAKE_LIST.save(deps.storage, info.sender.clone(), &stake_info)?;

        Ok(Response::new()
            .add_submessages(vec![SubMsg::new(reward_send_msg)])
            .add_attribute("action", "claim_unstaked"))
    }

    pub fn claim_reward(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        _amount: Uint128,
    ) -> Result<Response, ContractError> {
        let mut stake_info = STAKE_LIST.load(deps.storage, info.sender.clone())?;
        let state = STATE.load(deps.storage)?;

        let now = env.block.time.seconds() as u128;

        let apr =
            state.gdog_ratio / state.bdog_ratio * Uint128::new(state.apr.parse::<u128>().unwrap());
        let reward_with_decimals = calculate_stake_reward(apr, stake_info.clone(), now);
        let curr_reward = reward_with_decimals / (Uint128::new(100000));
        let total_reward = stake_info.reward_amount + curr_reward;

        let cw20_execute_msg_fp = Cw20ExecuteMsg::Transfer {
            recipient: info.sender.to_string(),
            amount: total_reward,
        };
        let fee_payout_msg = Cw20Contract(state.reward_token_address)
            .call(cw20_execute_msg_fp)
            .map_err(ContractError::Std)?;

        stake_info.reward_amount = Uint128::zero();
        stake_info.stake_start_time = Uint128::new(now);
        STAKE_LIST.save(deps.storage, info.sender, &stake_info)?;
        Ok(Response::new()
            .add_submessages(vec![SubMsg::new(fee_payout_msg)])
            .add_attribute("method", "distribute_reward"))
    }

    pub fn edit_state(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        allowed_operators: Option<Vec<Addr>>,
        token_address: Option<Addr>,
        unstaking_duration: Option<Uint128>,
        apr: Option<String>,
        bdog_ratio: Option<Uint128>,
        gdog_ratio: Option<Uint128>,
        token_source: Option<Addr>,
        reward_token_address: Option<Addr>,
    ) -> Result<Response, ContractError> {
        let mut state = STATE.load(deps.storage)?;

        if state.allowed_operators.contains(&info.sender) == false {
            return Err(ContractError::Unauthorized {
                msg: "Only allowed operators can execute this message".to_string(),
            });
        }

        if let Some(allowed_operators) = allowed_operators {
            state.allowed_operators = allowed_operators
        }
        if let Some(token_address) = token_address {
            state.token_address = token_address
        }
        if let Some(unstaking_duration) = unstaking_duration {
            state.unstaking_duration = unstaking_duration
        }
        if let Some(apr) = apr {
            let now = env.block.time.seconds() as u128;

            let curr_apr = state.gdog_ratio / state.bdog_ratio
                * Uint128::new(state.apr.parse::<u128>().unwrap());

            let records: StdResult<Vec<_>> = STAKE_LIST
                .range(deps.storage, None, None, Order::Ascending)
                .collect();

            state.apr = apr;

            for stake_item in records.unwrap() {
                let addr = stake_item.0;
                let stake_data = stake_item.1;

                let reward_with_decimals =
                    calculate_stake_reward(curr_apr, stake_data.clone(), now);
                let new_stake_data = StakeInfo {
                    owner: stake_data.owner,
                    stake_amount: stake_data.stake_amount,
                    unstaking_amount: stake_data.unstaking_amount,
                    reward_amount: stake_data.reward_amount + reward_with_decimals,
                    apr: state.apr.clone(),
                    stake_start_time: Uint128::new(now),
                    reward_start_time: Uint128::new(now),
                    unstaking_start_time: stake_data.unstaking_start_time,
                    reward_end_time: stake_data.reward_end_time,
                    unstaking_process: false,
                    unstake_end_time: stake_data.unstake_end_time,
                };

                STAKE_LIST.save(deps.storage, addr, &new_stake_data)?;
            }
        }
        if let Some(token_source) = token_source {
            state.token_source = token_source
        }
        if let Some(reward_token_address) = reward_token_address {
            state.reward_token_address = reward_token_address
        }
        if let Some(bdog_ratio) = bdog_ratio {
            state.bdog_ratio = bdog_ratio
        }
        if let Some(gdog_ratio) = gdog_ratio {
            state.gdog_ratio = gdog_ratio
        }
        STATE.save(deps.storage, &state)?;
        Ok(Response::new().add_attribute("action", "increment"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUserReward { addr } => to_binary(&query::user_reward(deps, env, addr)?),
        QueryMsg::GetUserStakeInfo { addr } => to_binary(&query::user_stake_info(deps, addr)?),
        QueryMsg::GetState {} => to_binary(&query::state(deps)?),
        QueryMsg::RangeStakeList { start_after, limit } => {
            to_binary(&query::list(deps, start_after, limit)?)
        }
    }
}

pub mod query {
    use super::*;
    use crate::msg::{
        GetStakeResponse, GetStateResponse, GetUserRewardResponse, StakeListResponse,
    };
    use cosmwasm_std::{Addr, Order};
    use cw_storage_plus::Bound;

    const DEFAULT_LIMIT: u32 = 10;
    const MAX_LIMIT: u32 = 30;

    pub fn user_reward(deps: Deps, env: Env, addr: Addr) -> StdResult<GetUserRewardResponse> {
        let stake_info = STAKE_LIST.load(deps.storage, addr)?;
        let state = STATE.load(deps.storage)?;

        let now = env.block.time.seconds() as u128;
        let mut total = stake_info.reward_amount;
        let apr =
            state.gdog_ratio / state.bdog_ratio * Uint128::new(state.apr.parse::<u128>().unwrap());
        if stake_info.stake_amount > Uint128::zero() {
            let curr_reward = calculate_stake_reward(apr, stake_info, now);
            total = total + curr_reward;
        }

        Ok(GetUserRewardResponse { amount: total })
    }

    pub fn user_stake_info(deps: Deps, addr: Addr) -> StdResult<GetStakeResponse> {
        let info = STAKE_LIST.load(deps.storage, addr)?;
        Ok(GetStakeResponse { info })
    }

    pub fn state(deps: Deps) -> StdResult<GetStateResponse> {
        let state = STATE.load(deps.storage)?;
        let total_staked = STAKED_TOTAL.load(deps.storage)?;
        let total_reward = REWARD_TOTAL.load(deps.storage)?;
        Ok(GetStateResponse {
            allowed_operators: state.allowed_operators,
            token_address: state.token_address,
            unstaking_duration: state.unstaking_duration,
            apr: state.apr,
            token_source: state.token_source,
            total_staked,
            total_reward,
            reward_token_address: state.reward_token_address,
        })
    }

    pub fn list(
        deps: Deps,
        start_after: Option<Addr>,
        limit: Option<u32>,
    ) -> StdResult<StakeListResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(Bound::exclusive);

        let records: StdResult<Vec<_>> = STAKE_LIST
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .collect();

        let rec = records?.into_iter().map(|r| r.1).collect();

        Ok(StakeListResponse { stake_list: rec })
    }
}
