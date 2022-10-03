use rand::seq::SliceRandom;
use crate::*;

uint::construct_uint!(
    pub struct U256(4);
);

pub const NEAR:&str = "near";

pub(crate) fn near() -> AccountId {
    AccountId::new_unchecked(NEAR.to_string())
}

pub(crate) fn u128_ratio(a: u128, num: u128, denom: u128) -> u128 {
    (U256::from(a) * U256::from(num) / U256::from(denom)).as_u128()
}

pub(crate) fn ratio(balance: u128, r: u32) -> u128 {
    assert!(r <= MAX_RATIO);
    u128_ratio(balance, u128::from(r), u128::from(MAX_RATIO))
}

pub (crate) fn get_range_random_number(range_start: u32, range_end: u32) -> usize {
    let random_seed = env::random_seed_array();
    let mut rng:StdRng = SeedableRng::from_seed(random_seed);
    rng.gen_range(range_start, range_end) as _
}

pub (crate) fn shuffle(mut list: Vec<AccountId>) -> Vec<AccountId> {
    let random_seed = env::random_seed_array();
    let mut rng:StdRng = SeedableRng::from_seed(random_seed);
    list.shuffle(&mut rng);
    list
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn random_smoke_test() {
    let vec_to_shuffle = vec![
        "acc1.near".parse::<AccountId>().unwrap(),
        "acc2.near".parse::<AccountId>().unwrap(),
        "acc3.near".parse::<AccountId>().unwrap(),
        "acc4.near".parse::<AccountId>().unwrap(),
        "acc5.near".parse::<AccountId>().unwrap()
    ];

    near_sdk::testing_env!(near_sdk::test_utils::VMContextBuilder::new()
        .random_seed([8; 32])
        .build());

    assert_eq!(near_sdk::env::random_seed(), [8; 32]);
    let shuffled = shuffle(vec_to_shuffle);
    dbg!("shuffled: {:?}", shuffled);
}