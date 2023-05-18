use crate::state::StakeInfo;
use cosmwasm_std::Uint128;

// Output is amount of reward multiplied with 10000
pub fn calculate_stake_reward(apr: Uint128, stake_info: StakeInfo, seconds: u128) -> Uint128 {
    let stake_start_time = stake_info.stake_start_time;
    let staked_amount = stake_info.stake_amount;

    let each_second_reward = get_reward_for_each_second(apr.to_string());
    let now = Uint128::new(seconds);
    let mut reward = Uint128::zero();

    if staked_amount > Uint128::zero() {
        let second_difference_from_start = now - stake_start_time;
        let one_token_reward = second_difference_from_start * each_second_reward;
        reward = one_token_reward * staked_amount;
    }
    reward
}

pub fn get_reward_for_each_second(apr: String) -> Uint128 {
    // find reward for each passing second
    let decimal = Uint128::new(100000);
    let for_decimal_values = Uint128::new(100);
    let mut annual = Uint128::new(apr.parse::<u128>().unwrap());
    annual = annual * decimal * for_decimal_values;

    let days = Uint128::new(365);
    let daily_reward = annual / days;

    let seconds = Uint128::new(86400);
    let second_reward = daily_reward / seconds;

    second_reward
}
