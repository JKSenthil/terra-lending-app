#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Order};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, StdError, Uint128, from_binary, Addr, attr, Timestamp, Decimal, Storage};
use cw2::set_contract_version;
use cw20::{Cw20ReceiveMsg, Cw20Contract, Cw20ExecuteMsg,};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, QueryMsg, UserInfoResponse, InstantiateMsg, Cw20HookMsg};
use crate::state::{UserData, USER_INFO, Config, CONFIG, LoanInfo, LOANS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lending-app";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        admin: deps.api.addr_validate(&msg.admin)?,
        generic_token: deps.api.addr_validate(&msg.generic_token)?,
        lending_token: None,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("admin", msg.admin)
        .add_attribute("generic token", config.generic_token))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(_msg) => receive_cw20(deps, info, env, _msg),
        ExecuteMsg::Withdraw { amount } => try_withdraw(deps, info, env, amount), 
        ExecuteMsg::Borrow { amount } => try_borrow(deps, info, env, amount),
        ExecuteMsg::SetLendingTokenAddress { address } => set_lending_token_addr(deps, info, address),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Deposit {}) => {
            // only asset contract can execute this message
            let contract_addr = info.sender;
            let config: Config = CONFIG.load(deps.storage)?;
            if contract_addr != config.generic_token {
                return Err(ContractError::Unauthorized {});
            }

            let cw20_sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
            try_deposit(deps, cw20_sender_addr, cw20_msg.amount)
        },
        Ok(Cw20HookMsg::Payoff { }) => {
            // only lending token contract can execute this message
            let contract_addr = info.sender;
            let config: Config = CONFIG.load(deps.storage)?;
            if contract_addr != config.lending_token.unwrap() {
                return Err(ContractError::Unauthorized {});
            }
            let cw20_sender_addr = deps.api.addr_validate(&cw20_msg.sender)?;
            try_payoff(deps, cw20_sender_addr, env, cw20_msg.amount)
        } 
        _ => Err(ContractError::MissingDepositHook {}),
    }
}

pub fn try_deposit(deps: DepsMut, user_addr: Addr, amount: Uint128) -> Result<Response, ContractError> {
    USER_INFO.update(
        deps.storage,
        &user_addr,
        |ud: Option<UserData>| -> StdResult<_> { 
            match ud {
                Some(user_data) => Ok(user_data.deposit_amount(amount)),
                None => Ok (UserData::new().deposit_amount(amount))
            }
        },
    )?;
    Ok(Response::default())
}

pub fn try_payoff(deps: DepsMut, user_addr: Addr, env: Env, amount: Uint128) -> Result<Response, ContractError> {
    let mut payoff_amount = amount;
    let loans: StdResult<Vec<_>> = LOANS.prefix(&user_addr).range(deps.storage, None, None, Order::Ascending).collect();
    for (loan_id, loan_info) in loans.unwrap() {
        let updated_loan_info = loan_info.update_loan(env.block.time);
        let mut principal = updated_loan_info.principal.atomics();
        let mut amount_owed = updated_loan_info.amount_owed.atomics();
        if payoff_amount <= amount_owed {
            principal = amount_owed - payoff_amount;
            amount_owed -= payoff_amount;
            if amount_owed == Uint128::zero() {
                LOANS.remove(deps.storage, (&user_addr, loan_id));
            } else {
                LOANS.save(deps.storage, (&user_addr, loan_id), &LoanInfo{ 
                    start_time: updated_loan_info.start_time, 
                    last_update_time: env.block.time, 
                    principal: Decimal::new(principal), 
                    amount_owed: Decimal::new(amount_owed) 
                })?;
            }
            break;
        } else {
            payoff_amount -= amount_owed;
            LOANS.remove(deps.storage, (&user_addr, loan_id));
        }
    }

    // if leftovers exist, return to user
    let config = CONFIG.load(deps.storage)?;
    if payoff_amount > Uint128::zero() {
        let transfer_response = Cw20Contract(config.lending_token.clone().unwrap()).call(
            Cw20ExecuteMsg::Transfer { recipient: user_addr.to_string(), amount: payoff_amount }
        )?;
        let burn_msg = Cw20Contract(config.lending_token.clone().unwrap()).call(
            Cw20ExecuteMsg::Burn { amount: amount.checked_sub(payoff_amount).unwrap() }
        )?;
        return Ok(Response::new().add_messages(vec![transfer_response, burn_msg]));
    }
    let burn_msg = Cw20Contract(config.lending_token.clone().unwrap()).call(
        Cw20ExecuteMsg::Burn { amount: amount }
    )?;
    Ok(Response::new().add_message(burn_msg))
}

/// Ensure user exists, and subtract from deposit
/// 
/// TODO (do after borrowing is implemented):
///     Must ensure that the user is still liquid after withdrawal 
///     ie Borrow amount must not exceed deposited amount
pub fn try_withdraw(deps: DepsMut, info: MessageInfo, env: Env, withdraw_amount: Uint128) -> Result<Response, ContractError>{
    let value = USER_INFO.may_load(deps.storage, &info.sender).unwrap();
    match value {
        Some(user_data) => {
            let deposit_amount = user_data.generic_token_deposited;
            let amount_owed = get_total_owed(deps.storage, env, info.clone().sender);
            if withdraw_amount > deposit_amount {
                return Err(ContractError::InsufficientFunds {  });
            } else if amount_owed < deposit_amount && withdraw_amount > (deposit_amount - amount_owed) {
                return Err(ContractError::InsufficientFunds {  });
            }
            USER_INFO.save(deps.storage, &info.sender, &user_data.withdraw_amount(withdraw_amount))?;
            let config = CONFIG.load(deps.storage)?;
            let transfer_response = Cw20Contract(config.generic_token).call(
                Cw20ExecuteMsg::Transfer { recipient: info.sender.to_string(), amount: withdraw_amount }
            )?;
            return Ok(Response::new().add_message(transfer_response).add_attributes(vec![
                attr("action", "withdraw"),
                attr("withdrawer", info.sender.to_string()),
                attr("amount", withdraw_amount.to_string()),
            ]))
        },
        None => return Err(ContractError::UserDNE { })
    };
}

pub fn try_borrow(deps: DepsMut, info: MessageInfo, env: Env, borrow_amount: Uint128) -> Result<Response, ContractError>{
    let mint_response;
    let value = USER_INFO.may_load(deps.storage, &info.sender).unwrap();
    match value {
        Some(user_data) => {
            if borrow_amount > (user_data.generic_token_deposited - user_data.borrow_amt) {
                return Err(ContractError::InsufficientFunds {  });
            }
            // mint lending token and send to borrower
            let config = CONFIG.load(deps.storage)?;
            mint_response = Cw20Contract(config.lending_token.unwrap()).call(
                Cw20ExecuteMsg::Mint { 
                    recipient: info.sender.to_string(), 
                    amount: borrow_amount
                }
            )?;
            
            // create and save loan
            let loan_id = user_data.curr_loan_id;
            let loan_info = LoanInfo::new(env.block.time, borrow_amount);
            LOANS.save(deps.storage, (&info.sender, loan_id.u128()), &loan_info)?;
            USER_INFO.save(deps.storage, &info.sender, &user_data.borrow_amount(borrow_amount))?;
        },
        None => return Err(ContractError::UserDNE { })
    }
    Ok(Response::new()
        .add_message(mint_response)
        .add_attributes(vec![
            attr("action", "borrow"),
            attr("borrower", info.sender.to_string()),
            attr("amount", borrow_amount.to_string()),
        ])
    )
}

pub fn set_lending_token_addr(deps: DepsMut, info: MessageInfo, address: String) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage).unwrap();
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {  });
    }
    let contract_addr = deps.api.addr_validate(&address)?;
    CONFIG.save(deps.storage, 
        &Config{ 
            admin: config.admin, 
            generic_token: config.generic_token, 
            lending_token: Some(contract_addr)
        }
    )?;
    Ok(Response::default())
}

pub fn get_total_owed(storage: &mut dyn Storage, env: Env, addr: Addr) -> Uint128 {
    let loans: StdResult<Vec<_>> = LOANS.prefix(&addr).range(storage, None, None, Order::Ascending).collect();
    let mut total_loan = Decimal::zero();
    for (_, loan_info) in loans.unwrap() {
        let updated_loan_info = loan_info.update_loan(env.block.time);
        total_loan += updated_loan_info.amount_owed;
    }
    total_loan.atomics()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUserInfo {address} => to_binary(&get_user_info(deps, env,address)?),
    }
}

pub fn get_user_info(deps: Deps, env: Env, address: String) -> StdResult<Option<UserInfoResponse>> {
    let address = deps.api.addr_validate(&address)?;
    // accumulate loans and add totals
    let loans: StdResult<Vec<_>> = LOANS.prefix(&address).range(deps.storage, None, None, Order::Ascending).collect();
    let mut total_loan = Decimal::zero();
    for (_, loan_info) in loans.unwrap() {
        let updated_loan_info = loan_info.update_loan(env.block.time);
        total_loan += updated_loan_info.amount_owed;
    }
    let res = match USER_INFO.may_load(deps.storage, &address) {
        Ok(Some(user_info)) => Some(
            UserInfoResponse { 
                generic_token_deposited: user_info.generic_token_deposited,
                lending_token_withdrawed: user_info.borrow_amt,
                total_loan_owed: total_loan.atomics(), 
            }
        ),
        Ok(None) => None,
        Err(_) => None,
    };
    Ok(res)
}

// TODO write tests
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_env, mock_info, mock_dependencies};   
    use cosmwasm_std::{to_binary, Uint128};

    #[test]
    fn basic_test() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg{ 
            admin: "admin".to_string(), 
            generic_token: "token".to_string(), 
        };
        let info = mock_info("creator", &[]);
        let env = mock_env();
        let res = instantiate(deps.as_mut(), env.clone(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let lend_token_addr_msg = ExecuteMsg::SetLendingTokenAddress { address: "token".to_string() };
        let info = mock_info("admin", &[]);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), lend_token_addr_msg.clone());
        match res {
            Ok(_) => {}
            Err(_) => panic!("Should not have received an error"),
        }

        let recv_msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "user1".to_string(),
            amount: Uint128::from(100u128),
            msg: to_binary(&Cw20HookMsg::Deposit {}).unwrap(),
        });
        let info = mock_info("token", &[]);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), recv_msg.clone());
        match res {
            Ok(_) => {}
            Err(_) => panic!("Should not have received an error"),
        }
        
        let response : StdResult<Option<UserInfoResponse>> = get_user_info(deps.as_ref(), env.clone(), "user1".to_string());
        assert_eq!(
            response,
            Ok(Some(UserInfoResponse {generic_token_deposited: Uint128::from(100u128), lending_token_withdrawed: Uint128::zero(), total_loan_owed: Uint128::zero() }))
        );

        // test non-existent user
        let response : StdResult<Option<UserInfoResponse>> = get_user_info(deps.as_ref(), env.clone(), "user2".to_string());
        assert_eq!(
            response,
            Ok(None)
        );

        // withdrawal test
        let withdraw_msg = ExecuteMsg::Withdraw { amount: Uint128::from(99u128) };
        let info = mock_info("user1", &[]);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), withdraw_msg.clone());
        match res {
            Ok(_) => {}
            Err(_) => panic!("Should not have received an error"),
        }
        
        let response : StdResult<Option<UserInfoResponse>> = get_user_info(deps.as_ref(), env.clone(), "user1".to_string());
        assert_eq!(
            response,
            Ok(Some(UserInfoResponse { generic_token_deposited: Uint128::from(1u128), lending_token_withdrawed: Uint128::zero(), total_loan_owed: Uint128::zero() }))
        );

        // borrow test (insufficient funds)
        let borrow_msg = ExecuteMsg::Borrow { amount: Uint128::from(2u128) };
        let info = mock_info("user1", &[]);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), borrow_msg.clone());
        match res {
            Ok(_) => panic!("Should have received an error"),
            _ => {},
        }

        // borrow test (sufficient funds)
        let borrow_msg = ExecuteMsg::Borrow { amount: Uint128::from(1u128) };
        let info = mock_info("user1", &[]);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), borrow_msg.clone());
        match res {
            Ok(_) => {},
            _ => panic!("Should not have received an error"),
        }
        
        // check borrow amount is updated
        let user_info = get_user_info(deps.as_ref(), env.clone(), "user1".to_string()).unwrap();
        match user_info {
            Some(ui) => {
                assert_eq!(ui.generic_token_deposited, Uint128::from(1u128));
                assert_eq!(ui.lending_token_withdrawed, Uint128::from(1u128));
                assert_eq!(ui.total_loan_owed, Uint128::from(1u128));
            },
            None => panic!("Should not be none!"),
        }

        // borrow test (insufficient funds)
        let borrow_msg = ExecuteMsg::Borrow { amount: Uint128::from(2u128) };
        let info = mock_info("user1", &[]);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), borrow_msg.clone());
        match res {
            Ok(_) => panic!("Should have received an error"),
            _ => {},
        }

        // deposit more $$
        let recv_msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "user1".to_string(),
            amount: Uint128::from(100u128),
            msg: to_binary(&Cw20HookMsg::Deposit {}).unwrap(),
        });
        let info = mock_info("token", &[]);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), recv_msg.clone());
        match res {
            Ok(_) => {}
            Err(_) => panic!("Should not have received an error"),
        }

        // borrow test, see if total borrow amount is correct
        let borrow_msg = ExecuteMsg::Borrow { amount: Uint128::from(50u128) };
        let info = mock_info("user1", &[]);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), borrow_msg.clone());
        match res {
            Ok(_) => {},
            _ => panic!("Should not have received an error"),
        }
        
        let user_info = get_user_info(deps.as_ref(), env.clone(), "user1".to_string()).unwrap();
        match user_info {
            Some(ui) => {
                assert_eq!(ui.generic_token_deposited, Uint128::from(101u128));
                assert_eq!(ui.lending_token_withdrawed, Uint128::from(51u128));
                assert_eq!(ui.total_loan_owed, Uint128::from(51u128));
            },
            None => panic!("Should not be none!"),
        }
    }
}
