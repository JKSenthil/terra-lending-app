use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128, Timestamp, Decimal};
use cw_storage_plus::{Item, Map};

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
    pub last_update_time: Timestamp,
    pub principal: Decimal,
    pub amount_owed: Decimal,
}

impl LoanInfo {
    pub fn new(ts: Timestamp, principal: Uint128) -> LoanInfo {
        LoanInfo {
            start_time: ts, 
            last_update_time: ts, 
            principal: Decimal::new(principal),
            amount_owed: Decimal::new(principal),
        }
    }

    /// Computes current loan values on granularity of days
    pub fn update_loan(self, ts: Timestamp) -> LoanInfo {
        let mut principal = self.principal;
        let mut amount_owed = self.amount_owed;

        let prev_days_elapsed = (self.last_update_time.seconds() - self.start_time.seconds()) / 86400;
        let new_days_elapsed = (ts.seconds() - self.start_time.seconds()) / 86400;
        let mut inbetween_days = new_days_elapsed - prev_days_elapsed;

        let prev_year = prev_days_elapsed / 365;
        let curr_year = new_days_elapsed / 365;
        if prev_year != curr_year {
            // find number of days to next year
            let next_year_in_days = (prev_year + 1) * 365;
            let apply_days = next_year_in_days - prev_days_elapsed;

            // update principal
            let p1 = Decimal::one() + Decimal::percent(3);
            amount_owed = principal * p1;
            principal = amount_owed;

            // update number of days left to update;
            inbetween_days -= apply_days;
            while inbetween_days >= 365 {  
                amount_owed = principal * p1;
                principal = amount_owed;
                inbetween_days -= 365;
            }
        }
        if inbetween_days > 0 {
            let p = Decimal::one() + (Decimal::percent(3) * Decimal::from_ratio(inbetween_days, 365_u128)); 
            let amt = principal * p;
            return LoanInfo {
                start_time: self.start_time,
                last_update_time: ts,
                principal: principal,
                amount_owed: amt,
            }
        }
        return LoanInfo {
            start_time: self.start_time,
            last_update_time: ts,
            principal: principal,
            amount_owed: amount_owed
        }
    }
}

pub const CONFIG: Item<Config> = Item::new("Config");
pub const USER_INFO: Map<&Addr, UserData> = Map::new("User");
pub const LOANS: Map<(&Addr, u128), LoanInfo> = Map::new("Loan");

#[cfg(test)]
mod state_tests {
    use cosmwasm_std::{Timestamp, Uint128};

    use super::LoanInfo;

    #[test]
    fn basic_loan_test() {
        let ts = Timestamp::from_seconds(0);
        let mut loan_info = LoanInfo::new(ts, Uint128::from(1000_u128));
        let ts2 = Timestamp::from_seconds(86400 * 365);
        let loan_info2 = loan_info.update_loan(ts2);
        assert_eq!(
            loan_info2.principal.atomics(),
            Uint128::from(1030_u128)
        )
    }

    #[test]
    fn multi_year_loan_test() {
        let ts = Timestamp::from_seconds(0);
        let mut loan_info = LoanInfo::new(ts, Uint128::from(1000_u128));
        let ts2 = Timestamp::from_seconds(86400 * 1); // 1 day
        let mut loan_info2 = loan_info.update_loan(ts2);
        let ts3 = Timestamp::from_seconds(86400 * 365);
        let mut loan_info3 = loan_info2.update_loan(ts3);
        assert_eq!(
            loan_info3.principal.atomics(),
            Uint128::from(1030_u128)
        );
        let ts4 = Timestamp::from_seconds(86400 * 365 * 3);
        let loan_info4 = loan_info3.update_loan(ts4);
        assert_eq!(
            loan_info4.principal.atomics(),
            Uint128::from(1091_u128)
        )
    }
}