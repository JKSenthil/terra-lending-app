#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, StdError, Uint128, from_binary, Addr};
use cw2::set_contract_version;
use cw20::{Cw20ReceiveMsg,};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, QueryMsg, LoanInfoResponse, InstantiateMsg, Cw20HookMsg};
use crate::state::{UserData, USER_INFO, Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lending-app";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        admin: msg.admin,
        generic_token: msg.generic_token,
    };
    CONFIG.save(deps.storage, &config)?;

    // TODO change
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", info.sender)
        .add_attribute("generic token", config.generic_token))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(_msg) => receive_cw20(deps, _env, info, _msg),
        ExecuteMsg::Withdraw { amount } => try_withdraw(deps, info, amount), 
        ExecuteMsg::Borrow { amount } => try_borrow(deps, info, amount),
        ExecuteMsg::Repay { amount } => try_repay(deps, info, amount),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let contract_addr = info.sender;
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Deposit {}) => {
            // only asset contract can execute this message
            let config: Config = CONFIG.load(deps.storage)?;
            if contract_addr != config.generic_token {
                return Err(ContractError::Unauthorized {});
            }

            let cw20_sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
            try_deposit(deps, cw20_sender_addr, cw20_msg.amount)
        }
        _ => Err(ContractError::MissingDepositHook {}),
    }
}

pub fn try_deposit(deps: DepsMut, user_addr: Addr, amount: Uint128) -> Result<Response, ContractError> {
    USER_INFO.update(
        deps.storage,
        &user_addr,
        |balance: Option<UserData>| -> StdResult<_> { 
            match balance {
                Some(user_data) => Ok(
                    UserData {
                        generic_token_deposited: user_data.generic_token_deposited + amount,
                    }
                ),
                None => Ok (UserData {
                    generic_token_deposited: amount,
                })
            }
        },
    )?;
    Ok(Response::default())
}

pub fn try_withdraw(deps: DepsMut, info: MessageInfo, amount: Uint128) -> Result<Response, ContractError>{
    Ok(Response::new().add_attribute("not", "yet implemented"))
}

pub fn try_borrow(deps: DepsMut, info: MessageInfo, amount: Uint128) -> Result<Response, ContractError>{
    Ok(Response::new().add_attribute("not", "yet implemented"))
}

pub fn try_repay(deps: DepsMut, info: MessageInfo, amount: Uint128) -> Result<Response, ContractError>{
    Ok(Response::new().add_attribute("not", "yet implemented"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUserInfo {} => to_binary(&get_user_info(deps, _env)?),
    }
}

pub fn get_user_info(deps: Deps, _env: Env) -> StdResult<LoanInfoResponse> {
    Ok(LoanInfoResponse { count: 1 })
}

// REFERENCE CODE
// fn query_count(deps: Deps) -> StdResult<CountResponse> {
//     let state = STATE.load(deps.storage)?;
//     Ok(CountResponse { count: state.count })
// }

// TODO write tests later
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
//     use cosmwasm_std::{coins, from_binary};

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(1000, "earth"));

//         // we can just call .unwrap() to assert this was a success
//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // it worked, let's query the state
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(17, value.count);
//     }

//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Increment {};
//         let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // should increase counter by 1
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }

//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let unauth_info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
//         match res {
//             Err(ContractError::Unauthorized {}) => {}
//             _ => panic!("Must return unauthorized error"),
//         }

//         // only the original creator can reset the counter
//         let auth_info = mock_info("creator", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

//         // should now be 5
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(5, value.count);
//     }
// }
