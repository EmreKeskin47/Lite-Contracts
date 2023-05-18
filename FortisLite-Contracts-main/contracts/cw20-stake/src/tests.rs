#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_env;
    use cosmwasm_std::Uint128;
    use std::str::FromStr;

    #[test]
    fn stake_test() {
        let apr = "20";

        let decimal = Uint128::new(100000);
        let for_decimal_values = Uint128::new(1000);
        let mut annual = Uint128::new(apr.parse::<u128>().unwrap());
        annual = annual * decimal * for_decimal_values;
        println!("annual reward is: {}", annual);

        let days = Uint128::new(365);
        let daily_reward = annual / days;
        println!("daily reward is: {}", daily_reward);

        let seconds = Uint128::new(86400);
        let second_reward = daily_reward / seconds;
        println!("each second token reward is: {}", second_reward);

        let seconds = mock_env().block.time.seconds() as u128;
        println!("time of now seconds : {}", seconds);
        let now = Uint128::new(seconds);
        println!("time of now : {}", now);

        let reward_start_time = Uint128::new(1571797408);
        let staked_amount = Uint128::new(10000000);

        let reward;

        let second_difference_from_start = now - reward_start_time;
        println!("seconds pass from start : {}", second_difference_from_start);
        reward = second_difference_from_start * second_reward;
        println!("reward for 1 token : {}", reward);
        let output = reward * staked_amount;
        println!("output : {}", output);
    }
}
