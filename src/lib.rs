use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, log, near_bindgen, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, Promise,
};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

mod config;
mod fungible_token;
mod lottery;
mod lottery_config;
mod big_lottery;
mod simple_lottery;
mod views;
mod utils;

use crate::config::*;
use crate::lottery::*;
use crate::lottery_config::*;
use crate::big_lottery::*;
use crate::simple_lottery::*;
use crate::utils::*;

pub type LotteryId = u64;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Config,
    LotteryConfig,
    Fees,
    Lotteries,
    Cashbacks,
    WhitelistedTokens
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub config: LazyOption<Config>,
    pub lotteries: UnorderedMap<LotteryId, Lottery>,
    /// a list of allowed fungible tokens
    pub whitelisted_tokens: UnorderedSet<AccountId>,
    /// contract fees balance
    pub fees: UnorderedMap<AccountId, Balance>,
    /// counter for lotteries
    pub next_lottery_id: LotteryId,
    /// buffer for accounts to cashback transfer
    pub cashback_accounts: UnorderedMap<AccountId, StoredCashback>,
    /// lotteries config
    pub lotteries_config: LazyOption<LotteryConfig>
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        config: Config,
        entry_fees: Vec<(AccountId, Vec<U128>)>,
        num_participants: Vec<u32>,
        big_lottery_num_participants: Vec<u32>
    ) -> Self {
        config.assert_valid();
        let lottery_config = LotteryConfig::new(
            entry_fees,
            num_participants,
            big_lottery_num_participants
        );
        lottery_config.assert_valid();
        Self {
            config: LazyOption::new(StorageKey::Config, Some(&config)),
            whitelisted_tokens: UnorderedSet::new(StorageKey::WhitelistedTokens),
            lotteries: UnorderedMap::new(StorageKey::Lotteries),
            fees: UnorderedMap::new(StorageKey::Fees),
            next_lottery_id: 0,
            cashback_accounts: UnorderedMap::new(StorageKey::Cashbacks),
            lotteries_config: LazyOption::new(StorageKey::LotteryConfig, Some(&lottery_config))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fungible_token::TokenReceiverMsg;
    use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
    use near_sdk::ONE_NEAR;
    const ONE_USN:Balance = 1_000_000_000_000_000_000;
    // use near_contract_standards::{
    //     storage_management::StorageManagement,
    //     non_fungible_token::core::NonFungibleTokenReceiver,
    //     fungible_token::receiver::FungibleTokenReceiver,
    // };
    use near_sdk::test_utils::{VMContextBuilder};
    use near_sdk::{testing_env, ONE_YOCTO};

    fn user(user: &str) -> AccountId {
        user.parse().unwrap()
    }

    fn token(token: &str) -> AccountId {
        token.parse().unwrap()
    }

    fn owner() -> AccountId {
        "owner.near".parse().unwrap()
    }

    fn get_owner(contract: &mut Contract) -> AccountId {
        contract.internal_config().owner_id
    }

    // pub fn amount_to_yocto(value: Balance, decimals: u8) -> Balance {
    //     value * 10u128.pow(decimals as _)
    // }

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    fn create_config() -> Config {
        let config = Config {
            owner_id: owner(),
            contract_fee_ratio: 1000, //10%
            treasury_ratio: 0, //0% from contract_fee_ratio
            investor_ratio: 4000, //40% from contract_fee_ratio
            treasury: user("treasury.near"),
            investor: user("investor.near"),
        };
        config.assert_valid();
        config
    }

    fn setup_contract() -> Contract {
        let mut entry_fees:Vec<(AccountId, Vec<U128>)> = Vec::new();
        let near_entry_fees:Vec<U128> = vec![ONE_NEAR.into(), (3 * ONE_NEAR).into(), (5 * ONE_NEAR).into()];
        entry_fees.push((near(), near_entry_fees));

        let mut context = get_context(owner());
        testing_env!(context
            .attached_deposit(ONE_YOCTO)
            .build()
        );
        Contract::new(
            create_config(),
            entry_fees,
            vec![5,6,7,8,9,10],
            vec![50]
        )
    }

    fn contract_context() -> (Contract, VMContextBuilder) {
        let mut contract = setup_contract();
        assert_eq!(get_owner(&mut contract), owner(), "owner.near must be a contract owner");
        (contract , get_context(owner()))
    }

    fn owner_env(
        context: &mut VMContextBuilder
    ) {
        testing_env!(context
            .predecessor_account_id(owner())
            .attached_deposit(ONE_YOCTO)
            .build()
        )
    }

    fn enter_lottery_ft(
        token_id: &str,
        contract: &mut Contract,
        context: &mut VMContextBuilder,
        user: &AccountId,
        lottery_type: String,
        entry_fee: U128,
        lottery_num_participants: u32,
        is_first: bool,
        is_last: bool,
        referrer_id: Option<AccountId>
    ) {
        let token_receiver_msg = near_sdk::serde_json::to_string(
            &TokenReceiverMsg::DrawEnter {
                num_participants: lottery_num_participants,
                lottery_type,
                referrer_id
            }
        )
        .expect("Parse repay FT error");

        let token_id = token(token_id);
        testing_env!(context
            .predecessor_account_id(token_id)
            .attached_deposit(ONE_YOCTO)
            .build()
        );
        let prev_lotteries_num = contract.get_lotteries_num();
        contract
            .ft_on_transfer(
                user.clone(),
                entry_fee,
                token_receiver_msg
            );

        if is_last {
            assert!(contract.get_lottery(0).is_none());
            return;
        }

        let lottery = contract.get_lottery(0).unwrap();
        let entries_num = lottery.entries.len();
        let current_pool = lottery.current_pool.0;
        if is_first {
            assert!(contract.get_lotteries_num() ==  prev_lotteries_num + 1, "contract expected added new lottery instance");
            assert_eq!(entries_num, 1);
            assert_eq!(current_pool, entry_fee.0);
        }

        assert_eq!(lottery.entry_fee.0, entry_fee.0);
        assert_eq!(lottery.required_pool.0, entry_fee.0 * lottery_num_participants as u128);
        println!(
            "{:#?}", lottery
        );
    }

    fn enter_lottery(
        contract: &mut Contract,
        context: &mut VMContextBuilder,
        user: &AccountId,
        lottery_type: String,
        entry_fee: U128,
        lottery_num_participants: u32,
        is_first: bool,
        is_last: bool,
        referrer_id: Option<AccountId>
    ) -> (usize, Balance) {
        testing_env!(context
            .predecessor_account_id(user.clone())
            .attached_deposit(entry_fee.0)
            .build()
        );
        let prev_lotteries_num = contract.get_lotteries_num();
        let lottery_id = contract.draw_near_enter(lottery_type, lottery_num_participants, referrer_id);

        if is_last {
            assert!(contract.get_lottery(lottery_id).is_none());
            return (0,0);
        }

        let lottery = contract.get_lottery(lottery_id).unwrap();
        let entries_num = lottery.entries.len();
        let current_pool = lottery.current_pool.0;
        if is_first {
            assert!(contract.get_lotteries_num() ==  prev_lotteries_num + 1, "contract expected added new lottery instance");
            assert_eq!(entries_num, 1);
            assert_eq!(current_pool, entry_fee.0);
        }

        assert_eq!(lottery.entry_fee.0, entry_fee.0);
        assert_eq!(lottery.required_pool.0, entry_fee.0 * lottery_num_participants as u128);
        println!(
            "{:#?}", lottery
        );
        (entries_num, current_pool)
    }

    #[test]
    fn test_basics() {
        let (mut contract, mut context) = contract_context();
        enter_lottery(
            &mut contract, 
            &mut context, 
            &user("user.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(3 * ONE_NEAR), 
            6u32,
            true,
            false,
            None
        );
    } 

    #[test]
    #[should_panic(expected = "Already entered")]
    fn test_double_enter() {
        let (mut contract, mut context) = contract_context();
        enter_lottery(
            &mut contract, 
            &mut context, 
            &user("user.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_NEAR), 
            5u32,
            true,
            false,
            None
        );

        enter_lottery(
            &mut contract, 
            &mut context, 
            &user("user.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_NEAR), 
            5u32,
            true,
            false,
            None
        );
    } 

    #[test]
    #[should_panic]
    fn test_incorrect_entry_fee() {
        let (mut contract, mut context) = contract_context();
        enter_lottery(
            &mut contract, 
            &mut context, 
            &user("user.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_NEAR * 4), 
            5u32,
            true,
            false,
            None
        );
    }

    #[test]
    #[should_panic]
    fn test_incorrect_num_participants() {
        let (mut contract, mut context) = contract_context();
        enter_lottery(
            &mut contract, 
            &mut context, 
            &user("user.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_NEAR), 
            11u32,
            true,
            false,
            None
        );
    } 

    #[test]
    fn test_modify_lottery_config() {
        let (mut contract, mut context) = contract_context();
        owner_env(&mut context);

        contract.add_entry_fee(Some(near()), U128(20 * ONE_NEAR));
        assert!(
            contract.get_contract_params()
                .config
                .entry_fees_required
                .iter()
                .filter(|(acc, _)| acc == &near())
                .map(|(_, vec_fees)| vec_fees)
                .any(|v| v.contains(&U128(20 * ONE_NEAR)))
        );
        contract.remove_entry_fee(Some(near()), U128(20 * ONE_NEAR));
        assert!(
            !contract.get_contract_params()
                .config
                .entry_fees_required
                .iter()
                .filter(|(acc, _)| acc == &near())
                .map(|(_, vec_fees)| vec_fees)
                .any(|v| v.contains(&U128(20 * ONE_NEAR)))
        );
        contract.add_num_participants(25, SIMPLE_LOTTERY.to_string());
        contract.add_num_participants(25, BIG_LOTTERY.to_string());
        assert!(
            contract.get_contract_params()
                .config
                .num_participants_required
                .iter()
                .all(|(_, nums_vector)|{
                    nums_vector.contains(&25)
                })
        );
        contract.remove_num_participants(25, BIG_LOTTERY.to_string());
        contract.remove_num_participants(25, SIMPLE_LOTTERY.to_string());
        println!("{:?}", contract.get_contract_params()
            .config
            .num_participants_required 
        );
        contract.add_entry_fee(Some(token("usdn.near")), U128(20 * ONE_USN));
        println!("{:?}", contract.get_contract_params()
            .config
            .entry_fees_required 
        );
        assert!(
            !contract.get_contract_params()
                .config
                .num_participants_required
                .iter()
                .any(|(_, nums_vector)|{
                    nums_vector.contains(&25)
                })
        );
    }

    #[test]
    fn test_finished_lottery() {
        let (mut contract, mut context) = contract_context();
        enter_lottery(
            &mut contract, 
            &mut context, 
            &user("user1.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_NEAR * 3), 
            6u32,
            true,
            false,
            None
        );
        enter_lottery(
            &mut contract, 
            &mut context, 
            &user("user2.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_NEAR * 3), 
            6u32,
            false,
            false,
            None
        );
        enter_lottery(
            &mut contract, 
            &mut context, 
            &user("user3.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_NEAR * 3), 
            6u32,
            false,
            false,
            None
        );
        enter_lottery(
            &mut contract, 
            &mut context, 
            &user("user4.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_NEAR * 3), 
            6u32,
            false,
            false,
            None
        );
        enter_lottery(
            &mut contract, 
            &mut context, 
            &user("user5.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_NEAR * 3), 
            6u32,
            false,
            false,
            None
        );
        enter_lottery(
            &mut contract, 
            &mut context, 
            &user("user6.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_NEAR * 3), 
            6u32,
            false,
            true,
            None
        );

        let params = contract.get_contract_params();
        let contract_fees = ratio(18 * ONE_NEAR, params.config.contract_fee_ratio);
        // 0% and 40% takes from contract fees to investor and treasury
        let keeped_fees = ratio(contract_fees, 6000);
        assert_eq!(
            params.fees_collected[0], (near(), U128(keeped_fees)),
            "Mismatched fees collected"
        );
        println!(
            "{:#?}", params
        );
        //4950000000000000000000000
        //0.05 -> 0.03 to treasury, 0.005 to investor, 0.15 keeped
        //30000000000000000000000 + 5000000000000000000000 + 15000000000000000000000
    }
    
    #[test]
    fn test_finished_lottery_usdt() {
        let (mut contract, mut context) = contract_context();
        owner_env(&mut context);
        contract.whitelist_token(token("usdt.near"));
        assert!(contract.get_contract_params().whitelisted_tokens.contains(&(token("usdt.near"))));
        contract.add_entry_fee(Some(token("usdt.near")), U128(3 * ONE_USN));

        enter_lottery_ft(
            "usdt.near",
            &mut contract, 
            &mut context, 
            &user("user1.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_USN * 3), 
            6u32,
            true,
            false,
            Some(user("referrer.near"))
        );
        enter_lottery_ft(
            "usdt.near",
            &mut contract, 
            &mut context, 
            &user("user2.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_USN * 3), 
            6u32,
            false,
            false,
            Some(user("referrer.near"))
        );
        enter_lottery_ft(
            "usdt.near",
            &mut contract, 
            &mut context, 
            &user("user3.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_USN * 3), 
            6u32,
            false,
            false,
            Some(user("referrer.near"))
        );
        enter_lottery_ft(
            "usdt.near",
            &mut contract, 
            &mut context, 
            &user("user4.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_USN * 3), 
            6u32,
            false,
            false,
            Some(user("referrer.near"))
        );
        enter_lottery_ft(
            "usdt.near",
            &mut contract, 
            &mut context, 
            &user("user5.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_USN * 3), 
            6u32,
            false,
            false,
            Some(user("referrer.near"))
        );
        enter_lottery_ft(
            "usdt.near",
            &mut contract, 
            &mut context, 
            &user("user6.near"), 
            SIMPLE_LOTTERY.to_string(), 
            U128(ONE_USN * 3), 
            6u32,
            false,
            true,
            Some(user("referrer.near"))
        );

        let params = contract.get_contract_params();
        let contract_fees = ratio(18 * ONE_USN, params.config.contract_fee_ratio);
        // 0% and 40% takes from contract fees to investor and treasury
        let keeped_fees = ratio(contract_fees, 6000);
        assert_eq!(
            params.fees_collected[0], (token("usdt.near"), U128(keeped_fees)),
            "Mismatched fees collected"
        );
        println!(
            "{:#?}", params
        );
        //4950000000000000000000000
        //0.05 -> 0.03 to treasury, 0.005 to investor, 0.15 keeped
        //30000000000000000000000 + 5000000000000000000000 + 15000000000000000000000
    }

    #[test]
    fn test_finished_big_lottery() {
        let entry_fee = 3 * ONE_NEAR;
        let (mut contract, mut context) = contract_context();
        enter_lottery(
            &mut contract, 
            &mut context, 
            &user("user0.near"), 
            BIG_LOTTERY.to_string(), 
            U128(entry_fee), 
            50u32,
            true,
            false,
            Some(user("referrer.near"))
        );
        for index in 1u8..49u8 {
            let account = user(&("user".to_string().to_owned() + &index.to_string()));
            enter_lottery(
                &mut contract, 
                &mut context, 
                &account, 
                BIG_LOTTERY.to_string(), 
                U128(entry_fee), 
                50u32,
                false,
                false,
                Some(user("referrer.near"))
            );
        }
        enter_lottery(
            &mut contract, 
            &mut context, 
            &user("user50.near"), 
            BIG_LOTTERY.to_string(), 
            U128(entry_fee), 
            50u32,
            false,
            true,
            Some(user("referrer.near"))
        );
        let params = contract.get_contract_params();
        let contract_fees = 50 * entry_fee - ( (entry_fee / 2 * 25) + ( ( entry_fee / 10 + entry_fee ) * 15) + ( ( entry_fee / 2 + entry_fee ) * 10) );
        // 0% and 40% takes from contract fees to investor and treasury
        let keeped_fees = ratio(contract_fees, 6000);
        assert_eq!(
            params.fees_collected[0], (near(), U128(keeped_fees)),
            "Mismatched fees collected"
        );
        println!(
            "{:#?}", params
        );
        enter_lottery(
            &mut contract, 
            &mut context, 
            &user("user0.near"), 
            BIG_LOTTERY.to_string(), 
            U128(entry_fee), 
            50u32,
            true,
            false,
            Some(user("referrer.near"))
        );
        //7200000000000000000000000 - 40%
        //10800000000000000000000000 - 60%
    }
    // TESTS HERE
}
