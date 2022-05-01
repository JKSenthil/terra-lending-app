use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive (Cw20ReceiveMsg),
    // Deposit {amount: Uint128}, (TODO: Receive is equivalent to Deposit?)
    Withdraw {amount: Uint128},
    Borrow {amount: Uint128},
    Repay {amount: Uint128},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetLoanInfo {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LoanInfoResponse {
    // TODO populate fields
    pub count: i32,
}
