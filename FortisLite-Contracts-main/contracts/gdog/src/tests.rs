// use cosmwasm_std::{Addr, Coin, Empty, Uint128};
// use cosmwasm_std::OverflowOperation::Add;
// use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
// use cw_multi_test::{App, BankKeeper, Contract, ContractWrapper, Executor};
// use crate::contract::{execute, instantiate, query};
// use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse};

// fn mock_app() -> App {
//     let env = mock_env();
//     let api = Box::new(MockApi::default());
//     let bank = BankKeeper::new();

//     // App::new(FnOnce(api, env.block, bank, Box::new(MockStorage::new()))
//     App::default()
// }

// pub fn contract_gdog() -> Box<dyn Contract<Empty>>{
//     let contract = ContractWrapper::new_with_empty(
//         execute,
//         instantiate,
//         query,
//     );
//     Box::new(contract)
// }

// pub fn gdog_instantiate_msg(owner: Option<String>) -> InstantiateMsg {
//     InstantiateMsg {
//         owner,
//         bdog_token_address: Addr::unchecked(MOCK_CONTRACT_ADDR)
//     }
// }

// #[test]
// fn gdog_contract_multi_test() {
//     //////////////////////////////////////// INSTANTIATE  ////////////////////////////////////////
//     // Create the owner account
//     let owner = Addr::unchecked("simon");
//     let mut router = mock_app();

//     let gdog_contract_code_id = router.store_code(contract_gdog());
//     // Setup the counter contract with an initial count of zero
//     let init_msg = InstantiateMsg {
//         owner: Option::from("simon".to_string()),
//         bdog_token_address: Addr::unchecked(MOCK_CONTRACT_ADDR)
//     };
//     // Instantiate the counter contract using its newly stored code id
//     let mocked_contract_addr = router
//         .instantiate_contract(gdog_contract_code_id, owner.clone(), &init_msg, &[], "gdog", None)
//         .unwrap();

//     println!("{:?}", mocked_contract_addr);

//     //////////////////////////////////////// EXECUTE::UpdateConfig  ////////////////////////////////////////
//     //Generate message
//     let msg = ExecuteMsg::UpdateConfig {
//         new_owner: Option::from("arda".to_string()),
//         gdog_token_address: Option::from(Addr::unchecked("new gdog address")),
//         bdog_token_address: Option::from(Addr::unchecked("simon"))
//     };

//     //Execute function
//     let _ = router.execute_contract(
//         owner.clone(),
//         mocked_contract_addr.clone(),
//         &msg,
//         &[],
//     )
//         .unwrap();

//     //Query new config
//     let msg = QueryMsg::Config {};
//     let res: ConfigResponse = router
//         .wrap()
//         .query_wasm_smart(mocked_contract_addr.clone(), &msg)
//         .unwrap();

//     println!("{:?}", res);

//     //Receive function
//     let msg = ExecuteMsg::ReceiveTest { sender: "simon".to_string(), amount: Uint128::new(100) };
//     let res = router.execute_contract(
//         owner.clone(),
//         mocked_contract_addr.clone(),
//         &msg,
//         // &[Coin{denom: "gdog".to_string(), amount: Uint128::new(1)}],
//         &[]
//     )
//         .unwrap();
//     println!("{:?}", res);
// }
