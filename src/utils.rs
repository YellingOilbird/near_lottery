use rand::seq::SliceRandom;
use crate::*;

uint::construct_uint!(
    pub struct U256(4);
);

pub const NEAR:&str = "near";

pub(crate) fn match_token_id(account_id: &AccountId) -> String {
    let binding = account_id.to_string();
    let stringify_account = binding.as_str();
    match stringify_account {
        "usn" => "USN".to_string(),
        "near" => "NEAR".to_string(),
        "dac17f958d2ee523a2206206994597c13d831ec7.factory.bridge.near" => "USDT".to_string(),
        "6b175474e89094c44da98b954eedeac495271d0f.factory.bridge.near" => "DAI".to_string(),
        "a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48.factory.bridge.near" => "USDC".to_string(),
        "wrap.near" => "WNEAR".to_string(),
        "c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2.factory.bridge.near" => "WETH".to_string(),
        // testnet
        "usdn.testnet" => "USN".to_string(),
        "guacharo.testnet" => "GUA".to_string(),
        "usdt.fakes.testnet" => "USDT".to_string(),
        "dai.fakes.testnet" => "DAI".to_string(),
        "usdc.fakes.testnet" => "USDC".to_string(),
        "wrap.testnet" => "WNEAR".to_string(),
        "weth.fakes.testnet" => "WETH".to_string(),
        _ => unimplemented!()
    }
}

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