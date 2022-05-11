use std::convert::TryInto;

use cosmwasm_std::{Uint128, Uint256};
use schemars::JsonSchema;

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Decimal(#[schemars(with = "String")] Uint128);

impl Decimal {
    // const DECIMAL_PLACES: usize = 18;
    const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);

    /// Creates a Decimal(value)
    /// This is equivalent to `Decimal::from_atomics(value, 18)` but usable in a const context.
    pub fn new(value: Uint128) -> Self {
        Self(value * Self::DECIMAL_FRACTIONAL)
    }

    /// Create a 1.0 Decimal
    pub const fn one() -> Self {
        Self(Self::DECIMAL_FRACTIONAL)
    }

    /// Create a 0.0 Decimal
    pub const fn zero() -> Self {
        Self(Uint128::zero())
    }

    pub fn get_uint128(&self) -> Uint128 {
        self.0 / Self::DECIMAL_FRACTIONAL
    }

    /// Convert x% into Decimal
    pub fn percent(x: u64) -> Self {
        Self(((x as u128) * 10_000_000_000_000_000).into())
    }

    pub fn from_ratio(numerator: Uint128, denominator: Uint128) -> Decimal {
        match numerator.full_mul(Self::DECIMAL_FRACTIONAL).checked_div(Uint256::from(denominator)) .unwrap().try_into() {
            Ok(result) => Decimal(result),
            Err(_) => panic!("from ratio error occured"),
        }
    }

    pub fn add(self, x: Decimal) -> Decimal {
        Decimal(self.0.checked_add(x.0).unwrap())
    }

    pub fn sub(self, x: Decimal) -> Decimal {
        Decimal(self.0.checked_sub(x.0).unwrap())
    }

    pub fn mult(self, x: Decimal) -> Decimal {
        match (self.0.full_mul(x.0) / Uint256::from_uint128(Self::DECIMAL_FRACTIONAL)).try_into() {
            Ok(result) => Decimal(result),
            Err(_) => panic!("mult overflow occured"),
        }
    }

    pub fn div(self, x: Decimal) -> Decimal {
        Decimal(self.0 / x.0)
    }
}

#[cfg(test)]
mod state_tests {
    use super::*;

    #[test]
    fn basic_test() {
        let x = Decimal::new(Uint128::from(1_u128));
        let y = Decimal::one();
        let z = x.add(y);
        assert_eq!(
            z.get_uint128(),
            Uint128::from(2_u128)
        )
    }

    #[test]
    fn percent_test() {
        let x = Decimal::new(Uint128::from(1000_u128));
        let y = Decimal::one().add(Decimal::percent(3));
        assert_eq!(
            x.mult(y).get_uint128(),
            Uint128::from(1030_u128)
        )
    }

    #[test]
    fn from_ratio_test() {
        let x = Decimal::new(Uint128::from(1000_u128));
        let y = Decimal::from_ratio(Uint128::from(1_u128), Uint128::from(2_u128));
        assert_eq!(
            x.mult(y).get_uint128(),
            Uint128::from(500_u128)
        )
    }
}