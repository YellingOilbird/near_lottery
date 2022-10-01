use crate::*;

uint::construct_uint!(
    pub struct U256(4);
);

pub(crate) fn u128_ratio(a: u128, num: u128, denom: u128) -> u128 {
    (U256::from(a) * U256::from(num) / U256::from(denom)).as_u128()
}

pub(crate) fn ratio(balance: u128, r: u32) -> u128 {
    assert!(r <= MAX_RATIO);
    u128_ratio(balance, u128::from(r), u128::from(MAX_RATIO))
}