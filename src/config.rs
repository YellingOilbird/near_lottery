use crate::*;

pub const MAX_RATIO: u32 = 10000;

/// Contract config
/// Fees ratio represented in Basis Points
/// - 100% = `MAX_RATIO` (10000), E.g 1% equals to 100
/// - Contract fee ratio takes fees from lottery reward
/// - Treasury ratio takes amount from contract fees
/// - Investor ratio takes amount from contract fees
/// - Example: 
/// 
/// Reward is 100 NEAR, `contract_fee_ratio = 100`, `treasury_ratio` = 6000, `investor_ratio` = 1000:
/// - `contract_fees = 100 * 0.5%` = 0.5N
/// - `investor_amount = contract_fees * 10% = 0.5 * 0.1 = 0.05N`
/// - `treasury_amount = contract_fees * 60% = 0.5 * 0.6 = 0.3N`
/// - Keeping 0.5N - 0.05N - 0.3N = 0.15N on contract
/// - Transfer 0.05N to investor account
/// - Transfer 0.3N to treasury
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    /// contract owner
    pub owner_id: AccountId,
    /// fees taken from prize pool to contract
    pub contract_fee_ratio: u32,
    /// fees taken from `contract_fee` to treasury
    pub treasury_ratio: u32,
    /// fees taken from `contract_fee` to investor
    pub investor_ratio: u32,
    /// treasury account
    pub treasury: AccountId,
    /// investor account
    pub investor: AccountId,
    /// lotteries config
    pub lotteries_config: LotteryConfig
}

impl Config {
    pub fn assert_valid(&self) {
        assert!(self.contract_fee_ratio <= MAX_RATIO, "fees cannot be more than 100% in Basis Points");
        assert!(self.treasury_ratio <= MAX_RATIO, "treasury ratio cannot be more than 100% from contract fees");
        assert!(
            self.investor_ratio + self.treasury_ratio < MAX_RATIO,
            "Incorrect ratio setup, contract_fee_ratio must be less than ( investor_ratio + treasury_ratio ) "
        );
        self.lotteries_config.assert_valid();
    }
}

impl Contract {
    fn assert_owner(&self) {
        assert_eq!(
            &env::predecessor_account_id(),
            &self.internal_config().owner_id,
            "Not an owner"
        );
    }

    pub (crate) fn internal_config(&self) -> Config {
        self.config.get().unwrap()
    }

    pub (crate) fn assert_required_num_participants(&self, num: u32, lottery_type: LotteryType) {
        let required_num_participants = match lottery_type {
            LotteryType::SimpleLottery => {
                self
                    .internal_config()
                    .lotteries_config
                    .num_participants
            },
            LotteryType::BigLottery => {
                self
                    .internal_config()
                    .lotteries_config
                    .big_lottery_num_participants
            },
        };
        assert!(
            required_num_participants.contains(&num),
            "Lottery expected one from that number of participants  {:?} ",
            required_num_participants
        );
    }

    pub (crate) fn assert_required_entry_fees(&self, token_id: &AccountId, amount: Balance, lottery_type: LotteryType) {
        let config = self.internal_config();
        let required_entry_fees = match lottery_type {
            LotteryType::SimpleLottery => {
                config
                    .lotteries_config
                    .entry_fees
                    .get(token_id)
                    .expect("No required fees for token")
            },
            LotteryType::BigLottery => {
                config
                    .lotteries_config
                    .entry_fees
                    .get(token_id)
                    .expect("No required fees for token")
            }
        };
        assert!(
            required_entry_fees.contains(&amount.into()),
            "Lottery expected one from that entry fees in yoctoNEAR : {:?} ",
            required_entry_fees
        );
    }

    pub (crate) fn treasury(&self) -> AccountId {
        self.internal_config().treasury
    }

    pub (crate) fn investor(&self) -> AccountId {
        self.internal_config().investor
    }

    /// Contract fees ratio in basis points
    pub (crate) fn get_contract_fee_ratio(&self) -> u32 {
        self.internal_config().contract_fee_ratio
    }

    /// Crop from `contract_fee_ratio` in basis points (b.p)
    pub (crate) fn get_treasury_ratio_from_contract_fees(&self) -> u32 {
        self.internal_config().treasury_ratio
    }

    /// Crop from `contract_fee_ratio` in basis points (b.p)
    pub (crate) fn get_investor_ratio_from_contract_fees(&self) -> u32 {
        self.internal_config().investor_ratio
    }

    /// Treasury ratio in basis points
    pub (crate) fn get_treasury_taken_amount(&self, contract_fees: Balance) -> Balance {
        compute_internal_fee_ratio(contract_fees, self.get_treasury_ratio_from_contract_fees())
    }

    /// Investor ratio in basis points
    pub (crate) fn get_investor_taken_amount(&self, contract_fees: Balance) -> Balance {
        compute_internal_fee_ratio(contract_fees, self.get_investor_ratio_from_contract_fees())
    }
}

fn compute_internal_fee_ratio(contract_fees: Balance, ratio_from_contract_fees: u32) -> Balance {
    ratio(contract_fees, ratio_from_contract_fees)
}

#[near_bindgen]
impl Contract {
    /// Updates the current config.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    pub fn update_config(&mut self, config: Config) {
        assert_one_yocto();
        self.assert_owner();

        config.assert_valid();
        config.lotteries_config.assert_valid();

        self.config.set(&config);
    }

    /// Add FT to the whitelist.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    /// - Requires this token not being already whitelisted.
    #[payable]
    pub fn whitelist_token(&mut self, token_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();

        assert!(!self.whitelisted_tokens.contains(&token_id), "Already whitelisted");
        self.whitelisted_tokens.insert(&token_id);
    }

    /// Removes FT to the whitelist.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    /// - Requires this token being whitelisted.
    #[payable]
    pub fn remove_whitelist_token(&mut self, token_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();

        assert!(self.whitelisted_tokens.contains(&token_id), "Not fount in whitelisted list");
        self.whitelisted_tokens.remove(&token_id);
    }

    /// Added the lottery config new num_participants required.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    pub fn add_num_participants(
        &mut self, 
        num: u32,
        lottery_type: String
    ) {
        assert_one_yocto();
        self.assert_owner();

        let mut config = self.internal_config();

        if lottery_type == SIMPLE_LOTTERY.to_string() {
            config.lotteries_config.add_num_participants(num);
        } else if lottery_type == BIG_LOTTERY.to_string() {
            config.lotteries_config.add_big_lottery_num_participants(num);
        }

        config.lotteries_config.assert_valid();
        self.config.set(&config);
    }
    /// Removes the lottery config given num_participants.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    pub fn remove_num_participants (      
        &mut self, 
        num: u32,
        lottery_type: String
    ) {
        assert_one_yocto();
        self.assert_owner();

        let lottery_type = LotteryType::from(lottery_type);

        let mut config = self.internal_config();

        match lottery_type {
            LotteryType::SimpleLottery => {
                config.lotteries_config.remove_num_participants(num);
            },
            LotteryType::BigLottery => {
                config.lotteries_config.remove_big_lottery_num_participants(num);
            },
        }
        
        config.lotteries_config.assert_valid();
        self.config.set(&config);
    }
    /// Added the lottery config new entry_fee required.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    pub fn add_entry_fee(&mut self, token_id: Option<AccountId>, entry_fee: U128) {
        assert_one_yocto();
        self.assert_owner();

        let mut config = self.internal_config();
        config.lotteries_config.add_entry_fee(token_id, entry_fee);
        config.lotteries_config.assert_valid();

        self.config.set(&config);
    }
    /// Removes the lottery config given entry_fee.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    pub fn remove_entry_fee(&mut self, token_id: Option<AccountId>, entry_fee: U128) {
        assert_one_yocto();
        self.assert_owner();

        let mut config = self.internal_config();
        config.lotteries_config.remove_entry_fee(token_id, entry_fee);
        config.lotteries_config.assert_valid();

        self.config.set(&config);
    }
}