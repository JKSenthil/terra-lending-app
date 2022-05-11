use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::math::Decimal;
use cosmwasm_std::{Addr, Uint128, Timestamp};
use cw_storage_plus::{Item, Map, U128Key};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub generic_token: Addr,
    pub lending_token: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserData {
    pub generic_token_deposited: Uint128,
    pub borrow_amt: Uint128,
    pub curr_loan_id: Uint128,
}

impl UserData {
    pub fn new() -> UserData {
        UserData { 
            generic_token_deposited: Uint128::from(0_u128),
            borrow_amt: Uint128::from(0_u128),
            curr_loan_id: Uint128::from(0_u128),
        }
    }

    pub fn deposit_amount(&self, amount: Uint128) -> UserData {
        UserData { 
            generic_token_deposited: self.generic_token_deposited + amount, 
            borrow_amt: self.borrow_amt,
            curr_loan_id: self.curr_loan_id,
        }
    }

    pub fn withdraw_amount(&self, amount: Uint128) -> UserData {
        UserData { 
            generic_token_deposited: self.generic_token_deposited - amount, 
            borrow_amt: self.borrow_amt,
            curr_loan_id: self.curr_loan_id,
        }
    }

    /// update borrow amount & increment loan id
    pub fn borrow_amount(&self, amount: Uint128) -> UserData {
        UserData { 
            generic_token_deposited: self.generic_token_deposited, 
            borrow_amt: self.borrow_amt + amount, 
            curr_loan_id: self.curr_loan_id + Uint128::from(1_u128),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LoanInfo {
    pub start_time: Timestamp,
    pub principal: Uint128,
}

impl LoanInfo {
    pub fn new(ts: Timestamp, principal: Uint128) -> LoanInfo {
        LoanInfo {
            start_time: ts, 
            principal: principal,
        }
    }

    /// Computes current loan values on granularity of days
    pub fn amount_owed(self, ts: Timestamp) -> Uint128 {
        let p1 = Decimal::one().add(Decimal::percent(3));
        let mut owed = Decimal::new(self.principal);
        let mut days_elapsed = (ts.seconds() - self.start_time.seconds()) / 86400;

        // account for years
        while days_elapsed >= 365 {
            owed = owed.mult(p1);
            days_elapsed -= 365;
        }

        // account for remaining days
        if days_elapsed > 0 {
            let mut multiplier = Decimal::from_ratio(Uint128::from(days_elapsed), Uint128::from(365_u128));
            multiplier = multiplier.mult(Decimal::percent(3));
            multiplier = multiplier.add(Decimal::one());
            owed = owed.mult(multiplier);
        }
        return owed.get_uint128()
    }
}

pub const CONFIG: Item<Config> = Item::new("Config");
pub const USER_INFO: Map<&Addr, UserData> = Map::new("User");
pub const LOANS: Map<(&Addr, U128Key), LoanInfo> = Map::new("Loan");

#[cfg(test)]
mod state_tests {
    use cosmwasm_std::{Timestamp, Uint128};

    use super::LoanInfo;

    #[test]
    fn basic_loan_test() {
        let ts = Timestamp::from_seconds(0);
        let loan_info = LoanInfo::new(ts, Uint128::from(1000_u128));
        let ts2 = Timestamp::from_seconds(86400 * 365);
        assert_eq!(
            loan_info.amount_owed(ts2),
            Uint128::from(1030_u128)
        )
    }

    #[test]
    fn multi_year_loan_test() {
        let ts = Timestamp::from_seconds(0);
        let loan_info = LoanInfo::new(ts, Uint128::from(1000_u128));
        let ts2 = Timestamp::from_seconds(86400 / 2 * 365);
        assert_eq!(
            loan_info.clone().amount_owed(ts2),
            Uint128::from(1014_u128)
        );
        let ts3 = Timestamp::from_seconds(86400 * 365 * 3);
        assert_eq!(
            loan_info.amount_owed(ts3),
            Uint128::from(1092_u128)
        )
    }
}