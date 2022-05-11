// #![cfg(test)]

// use cosmwasm_std::{to_binary, Addr, Empty, Uint128, BlockInfo, coin};
// use cw20::{Cw20Coin, Cw20Contract, Cw20ExecuteMsg, MinterResponse};
// use cw_multi_test::{App, Contract, ContractWrapper, AppBuilder, Executor};

// use crate::{msg::{ExecuteMsg, InstantiateMsg, QueryMsg, Cw20HookMsg, UserInfoResponse}};

// pub fn contract_lending_protocol() -> Box<dyn Contract<Empty>> {
//     let contract = ContractWrapper::new(
//         crate::contract::execute,
//         crate::contract::instantiate,
//         crate::contract::query,
//     );
//     Box::new(contract)
// }

// pub fn contract_generic_cw20() -> Box<dyn Contract<Empty>> {
//     let contract = ContractWrapper::new(
//         cw20_base::contract::execute,
//         cw20_base::contract::instantiate,
//         cw20_base::contract::query,
//     );
//     Box::new(contract)
// }

// // NOTE: (MAYBE) need to call cargo build in lending-token folder for imports to work
// pub fn contract_lending_token() -> Box<dyn Contract<Empty>> {
//     let contract = ContractWrapper::new(
//         lending_token::contract::execute,
//         lending_token::contract::instantiate,
//         lending_token::contract::query
//     );
//     Box::new(contract)
// }

// fn mock_app() -> App<Empty> {
//     AppBuilder::new().build()
// }

// #[test]
// fn integration() {
//     // init vars
//     let admin = Addr::unchecked("admin");

//     // set personal balance
//     let user1 = Addr::unchecked("user1");
//     let init_funds = vec![coin(20, "btc"), coin(100, "eth")];

//     let mut router = mock_app();

//     // set money
//     router.init_bank_balance(&admin, init_funds).unwrap();

//     // setup generic token contract
//     let generic_id = router.store_code(contract_generic_cw20());
//     let msg = cw20_base::msg::InstantiateMsg {
//         name: "generic".to_string(),
//         symbol: "GEN".to_string(),
//         decimals: 6,
//         initial_balances: vec![Cw20Coin {
//             address: user1.to_string(),
//             amount: Uint128::new(5000_u128.pow(6)),
//         }],
//         mint: None,
//         marketing: None,
//     };
//     let generic_addr = router.instantiate_contract(
//         generic_id,
//         user1.clone(), 
//         &msg, 
//         &[], 
//         "GENERIC", 
//         None
//     ).unwrap();
//     let generic_token_contract = Cw20Contract(generic_addr.clone());

//     // setup lending protocol contract
//     let lending_protocol_id = router.store_code(contract_lending_protocol());
//     let msg = InstantiateMsg { 
//         admin: admin.clone().into_string(), 
//         generic_token: generic_addr.clone().into_string()
//     };
//     let lending_protocol_addr = router.instantiate_contract(
//         lending_protocol_id,
//         admin.clone(), 
//         &msg, 
//         &[], 
//         "LENDING_PROTOCOL", 
//         None
//     ).unwrap();

//     // setup lending token contract
//     let lending_id = router.store_code(contract_lending_token());
//     let msg = lending_token::msg::InstantiateMsg {
//         name: "lending".to_string(),
//         symbol: "LEN".to_string(),
//         decimals: 6,
//         initial_balances: vec![],
//         mint: Some(
//             MinterResponse{ 
//                 minter: lending_protocol_addr.clone().into_string() , 
//                 cap: None 
//             } 
//         ),
//         marketing: None,
//     };
//     let lending_addr = router.instantiate_contract(
//         lending_id,
//         user1.clone(), 
//         &msg, 
//         &[], 
//         "GENERIC", 
//         None
//     ).unwrap();
//     let lending_token_contract = Cw20Contract(lending_addr.clone());

//     // setup lending token address in lending protocol
//     let msg = ExecuteMsg::SetLendingTokenAddress { address: lending_addr.clone().into_string() };
//     router.execute_contract(admin.clone(), lending_protocol_addr.clone(), &msg, &[]).unwrap();

//     /*
//      * user1 deposit generic token into lending protocol
//      */
//     let send_msg = Cw20ExecuteMsg::Send { 
//         contract: lending_protocol_addr.clone().to_string(), 
//         amount: Uint128::new(4000_u128.pow(6)), 
//         msg: to_binary(&Cw20HookMsg::Deposit {}).unwrap()
//     };
//     router.execute_contract(user1.clone(), generic_addr.clone(), &send_msg, &[]).unwrap();
//     // check generic tokens have been routed successfully to lending protocol 
//     let balance = generic_token_contract.balance::<_, _>(&router, lending_protocol_addr.clone()).unwrap();
//     assert_eq!(
//         balance.u128(),
//         4000_u128.pow(6)
//     );

//     /*
//      * user1 requests to borrow 1 lending token
//      */
//     let borrow_amt = 1000_u128.pow(6);
//     let borrow_msg = ExecuteMsg::Borrow { amount: Uint128::from(borrow_amt) };
//     router.execute_contract(user1.clone(), lending_protocol_addr.clone(), &borrow_msg, &[]).unwrap();
//     // check lending tokens have been minted to user1
//     let balance = lending_token_contract.balance::<_, _>(&router, user1.clone()).unwrap();
//     assert_eq!(
//         balance.u128(),
//         borrow_amt
//     );

//     /*
//      * user1 trys to withdraw more that allowed, due to collateral on loan
//      */
//     let withdraw_amt = 4000_u128.pow(6);
//     let withdraw_msg = ExecuteMsg::Withdraw { amount: Uint128::from(withdraw_amt) };
//     // expect error to be unwrapped
//     router.execute_contract(user1.clone(), lending_protocol_addr.clone(), &withdraw_msg, &[]).unwrap_err();
//     // check contract still has 4000_u128 in account
//     let balance = generic_token_contract.balance::<_, _>(&router, lending_protocol_addr.clone()).unwrap();
//     assert_eq!(
//         balance.u128(),
//         4000_u128.pow(6)
//     );

//     /*
//      * user1 trys allowed withdrawal
//      */
//     let withdraw_amt = 3000_u128.pow(6);
//     let withdraw_msg = ExecuteMsg::Withdraw { amount: Uint128::from(withdraw_amt) };
//     router.execute_contract(user1.clone(), lending_protocol_addr.clone(), &withdraw_msg, &[]).unwrap();
//     // check contract has right amount left in its account
//     let balance = generic_token_contract.balance::<_, _>(&router, lending_protocol_addr.clone()).unwrap();
//     assert_eq!(
//         balance.u128(),
//         4000_u128.pow(6) - 3000_u128.pow(6)
//     );

//     /*
//      * user1 wants to see current loan status after half a year
//      */
//     let query_msg = QueryMsg::GetUserInfo { address: user1.clone().to_string() };
//     router.set_block(BlockInfo {
//         height: router.block_info().height,
//         time: router.block_info().time.plus_seconds(86400 / 2 * 365),
//         chain_id: router.block_info().chain_id,
//     });
//     let user_info: Option<UserInfoResponse> = router.wrap().query_wasm_smart(lending_protocol_addr.clone(), &query_msg).unwrap();
//     match user_info {
//         Some(ui) => {
//             assert_eq!(ui.total_loan_owed.u128(), 1014958904109589041);
//         },
//         None => panic!("User should exist!")
//     };

//     /*
//      * user1 wants to see current loan status after a year
//      */
//     let query_msg = QueryMsg::GetUserInfo { address: user1.clone().to_string() };
//     router.set_block(BlockInfo {
//         height: router.block_info().height,
//         time: router.block_info().time.plus_seconds(86400 / 2 * 365),
//         chain_id: router.block_info().chain_id,
//     });
//     let user_info: Option<UserInfoResponse> = router.wrap().query_wasm_smart(lending_protocol_addr.clone(), &query_msg).unwrap();
//     match user_info {
//         Some(ui) => {
//             assert_eq!(ui.total_loan_owed.u128(), 1030000000000000000);
//         },
//         None => panic!("User should exist!")
//     };
// }
