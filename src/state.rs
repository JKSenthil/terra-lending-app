use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserData {
    pub collateral_amount: Uint128,
    pub lending_amount: Uint128,
}

pub const USER_INFO: Map<&Addr, UserData> = Map::new("User");
