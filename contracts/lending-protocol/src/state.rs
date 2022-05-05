use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub generic_token: Addr,
    pub lending_token: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserData {
    pub generic_token_deposited: Uint128,
    pub total_loan_taken: Uint128,
    // TODO: find way to store each loan taken
}

pub const CONFIG: Item<Config> = Item::new("Config");
pub const USER_INFO: Map<&Addr, UserData> = Map::new("User");
