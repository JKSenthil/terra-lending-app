use cosmwasm_std::{Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw20::Cw20ReceiveMsg;


#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct InstantiateMsg {
    pub admin: String,
    pub generic_token: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub enum Cw20HookMsg {
    /// Deposit generic token
    Deposit {},

    /// Payoff loan
    Payoff {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// CW20 token receiver
    Receive (Cw20ReceiveMsg),

    ////////////////////
    /// User operations
    ////////////////////
    Withdraw {amount: Uint128},
    Borrow {amount: Uint128},

    ////////////////////
    /// Admin operations
    ////////////////////
    SetLendingTokenAddress {address: String}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetUserInfo { address: String },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UserInfoResponse {
    pub generic_token_deposited: Uint128,
    pub lending_token_withdrawed: Uint128,
    pub total_loan_owed: Uint128,
}
