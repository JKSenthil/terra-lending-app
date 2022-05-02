#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Decimal, DepsMut, Env,
    MessageInfo, Response, StakingMsg, StdError, StdResult, Uint128, WasmMsg, to_binary, Deps, Binary,
};

use cw2::set_contract_version;
use cw20_base::allowances::{
    execute_burn_from, execute_decrease_allowance, execute_increase_allowance, execute_send_from,
    execute_transfer_from, query_allowance,
};
use cw20_base::contract::{
    execute_burn, execute_mint, execute_send, execute_transfer, query_balance, query_token_info, query_minter,
};
use cw20_base::enumerable::query_all_allowances;
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // store token info using cw20-base format
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: Uint128::zero(),
        // set self as minter, so we can properly execute mint and burn
        mint: Some(MinterData {
            minter: env.contract.address,
            cap: None,
        }),
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // these all come from cw20-base to implement the cw20 standard
        ExecuteMsg::Transfer { recipient, amount } => {
            Ok(execute_transfer(deps, env, info, recipient, amount)?)
        }
        ExecuteMsg::Burn { amount } => Ok(execute_burn(deps, env, info, amount)?),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => Ok(execute_send(deps, env, info, contract, amount, msg)?),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_increase_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_decrease_allowance(
            deps, env, info, spender, amount, expires,
        )?),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => Ok(execute_transfer_from(
            deps, env, info, owner, recipient, amount,
        )?),
        ExecuteMsg::BurnFrom { owner, amount } => {
            Ok(execute_burn_from(deps, env, info, owner, amount)?)
        }
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => Ok(execute_send_from(
            deps, env, info, owner, contract, amount, msg,
        )?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::Minter {} => to_binary(&query_minter(deps)?),
        QueryMsg::Allowance { owner, spender } => {
            to_binary(&query_allowance(deps, owner, spender)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info,
    };

    fn default_instantiate() -> InstantiateMsg {
        InstantiateMsg {
            name: "Cool Derivative".to_string(),
            symbol: "DRV".to_string(),
            decimals: 9,
        }
    }

    fn get_balance<U: Into<String>>(deps: Deps, addr: U) -> Uint128 {
        query_balance(deps, addr.into()).unwrap().balance
    }

    /*
    * TODO write test for this token
    */
    #[test]
    fn cw20_imports_work() {
        let mut deps = mock_dependencies();

        // set the actors... bob stakes, sends coins to carl, and gives allowance to alice
        let bob = String::from("bob");
        let alice = String::from("alice");
        let carl = String::from("carl");

        // create the contract
        let creator = String::from("creator");
        let instantiate_msg = default_instantiate();
        let info = mock_info(&creator, &[]);
        instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();

        // bob got 1000 DRV for 1000 stake at a 1.0 ratio
        assert_eq!(get_balance(deps.as_ref(), &bob), Uint128::new(1000));

        // send coins to carl
        let bob_info = mock_info(&bob, &[]);
        let transfer = ExecuteMsg::Transfer {
            recipient: carl.clone(),
            amount: Uint128::new(200),
        };
        execute(deps.as_mut(), mock_env(), bob_info.clone(), transfer).unwrap();
        assert_eq!(get_balance(deps.as_ref(), &bob), Uint128::new(800));
        assert_eq!(get_balance(deps.as_ref(), &carl), Uint128::new(200));

        // allow alice
        let allow = ExecuteMsg::IncreaseAllowance {
            spender: alice.clone(),
            amount: Uint128::new(350),
            expires: None,
        };
        execute(deps.as_mut(), mock_env(), bob_info.clone(), allow).unwrap();
        assert_eq!(get_balance(deps.as_ref(), &bob), Uint128::new(800));
        assert_eq!(get_balance(deps.as_ref(), &alice), Uint128::zero());
        assert_eq!(
            query_allowance(deps.as_ref(), bob.clone(), alice.clone())
                .unwrap()
                .allowance,
            Uint128::new(350)
        );

        // alice takes some for herself
        let self_pay = ExecuteMsg::TransferFrom {
            owner: bob.clone(),
            recipient: alice.clone(),
            amount: Uint128::new(250),
        };
        let alice_info = mock_info(&alice, &[]);
        execute(deps.as_mut(), mock_env(), alice_info, self_pay).unwrap();
        assert_eq!(get_balance(deps.as_ref(), &bob), Uint128::new(550));
        assert_eq!(get_balance(deps.as_ref(), &alice), Uint128::new(250));
        assert_eq!(
            query_allowance(deps.as_ref(), bob.clone(), alice)
                .unwrap()
                .allowance,
            Uint128::new(100)
        );

        // burn some, but not too much
        let burn_too_much = ExecuteMsg::Burn {
            amount: Uint128::new(1000),
        };
        let failed = execute(deps.as_mut(), mock_env(), bob_info.clone(), burn_too_much);
        assert!(failed.is_err());
        assert_eq!(get_balance(deps.as_ref(), &bob), Uint128::new(550));
        let burn = ExecuteMsg::Burn {
            amount: Uint128::new(130),
        };
        execute(deps.as_mut(), mock_env(), bob_info, burn).unwrap();
        assert_eq!(get_balance(deps.as_ref(), &bob), Uint128::new(420));
    }
}
