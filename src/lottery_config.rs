use std::collections::HashMap;

use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Copy, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum LotteryType {
    SimpleLottery,
    BigLottery
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StoredCashback {
    pub amount: Balance, 
    pub accounts: Vec<AccountId>
}

impl From<String> for LotteryType {
    fn from(s: String) -> Self {
        if s == *SIMPLE_LOTTERY {
            LotteryType::SimpleLottery
        } else if s == *BIG_LOTTERY {
            LotteryType::BigLottery
        } else {
            panic!("Unknown lottery type")
        }
    }
}

///Options for NEAR lottery
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct LotteryConfig {
    pub entry_fees: HashMap<AccountId, Vec<U128>>,
    pub num_participants: Vec<u32>,
    pub big_lottery_num_participants: Vec<u32>
}

impl LotteryConfig {
    pub fn new(
        entry_fees: Vec<(AccountId, Vec<U128>)>,
        num_participants: Vec<u32>,
        big_lottery_num_participants: Vec<u32>
    ) -> Self {
        Self {
            entry_fees: entry_fees.iter().cloned().collect(),
            num_participants,
            big_lottery_num_participants,
        }
    }
    pub fn assert_valid(&self) {
        assert!(!self.entry_fees.is_empty());
        assert!(!self.num_participants.is_empty());
        assert!(!self.big_lottery_num_participants.is_empty());
    }

    pub fn add_num_participants(&mut self, num: u32) {
        self.num_participants.push(num);
    }

    pub fn remove_num_participants(&mut self, num: u32) {
        assert!(!self.num_participants.is_empty(), "Cannot remove last lottery num_participants");
        let index = self.num_participants.iter().position(|x| x == &num).expect("invalid num to remove");
        self.num_participants.remove(index);
    }

    pub fn add_big_lottery_num_participants(&mut self, num: u32) {
        self.big_lottery_num_participants.push(num)
    }

    pub fn remove_big_lottery_num_participants(&mut self, num: u32) {
        assert!(self.big_lottery_num_participants.len() > 1, "Cannot remove last lottery num_participants");
        let index = self.big_lottery_num_participants.iter().position(|x| x == &num).expect("invalid num to remove");
        self.big_lottery_num_participants.remove(index);
    }

    pub fn add_entry_fee(&mut self, token_id: Option<AccountId>, fee: U128) {
        if let Some(token_id) = token_id {
            self
                .entry_fees
                .entry(token_id)
                .or_default()
                .push(fee)
        } else {
            self
                .entry_fees
                .entry(near())
                .or_default()
                .push(fee)
        }
    }

    pub fn remove_entry_fee(&mut self, token_id: Option<AccountId>, fee: U128) {
        let (index, token) = if let Some(token_id) = token_id {
            let position = self
                .entry_fees
                .entry(token_id.clone())
                .or_default()
                .iter()
                .position(|x| x == &fee).expect("invalid fee to remove");
            (position, token_id)
        } else {
            let position = self
                .entry_fees
                .entry(near())
                .or_default()
                .iter()
                .position(|x| x == &fee).expect("invalid fee to remove");
            (position, near())
        };
        
        self
            .entry_fees
            .entry(token)
            .or_default()
            .remove(index);
    }
}

impl Contract {
    pub (crate) fn internal_lottery_config(&self) -> LotteryConfig {
        self.lotteries_config.get().unwrap()
    }

    pub (crate) fn assert_required_num_participants(&self, num: u32, lottery_type: LotteryType) {
        let required_num_participants = match lottery_type {
            LotteryType::SimpleLottery => {
                self
                    .internal_lottery_config()
                    .num_participants
            },
            LotteryType::BigLottery => {
                self
                    .internal_lottery_config()
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
        let lottery_config = self.internal_lottery_config();
        let required_entry_fees = match lottery_type {
            LotteryType::SimpleLottery => {
                lottery_config
                    .entry_fees
                    .get(token_id)
                    .expect("No required fees for token")
            },
            LotteryType::BigLottery => {
                lottery_config
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
}

#[near_bindgen]
impl Contract {
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

        let mut config = self.internal_lottery_config();

        if lottery_type == *SIMPLE_LOTTERY {
            config.add_num_participants(num);
        } else if lottery_type == *BIG_LOTTERY {
            config.add_big_lottery_num_participants(num);
        }

        config.assert_valid();
        self.lotteries_config.set(&config);
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

        let mut config = self.internal_lottery_config();

        match lottery_type {
            LotteryType::SimpleLottery => {
                config.remove_num_participants(num);
            },
            LotteryType::BigLottery => {
                config.remove_big_lottery_num_participants(num);
            },
        }
        
        config.assert_valid();
        self.lotteries_config.set(&config);
    }
    /// Added the lottery config new entry_fee required.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    pub fn add_entry_fee(&mut self, token_id: Option<AccountId>, entry_fee: U128) {
        assert_one_yocto();
        self.assert_owner();

        let mut config = self.internal_lottery_config();
        config.add_entry_fee(token_id, entry_fee);
        config.assert_valid();

        self.lotteries_config.set(&config);
    }
    /// Removes the lottery config given entry_fee.
    /// - Requires one yoctoNEAR.
    /// - Requires to be called by the contract owner.
    #[payable]
    pub fn remove_entry_fee(&mut self, token_id: Option<AccountId>, entry_fee: U128) {
        assert_one_yocto();
        self.assert_owner();

        let mut config = self.internal_lottery_config();
        config.remove_entry_fee(token_id, entry_fee);
        config.assert_valid();

        self.lotteries_config.set(&config);
    }
}