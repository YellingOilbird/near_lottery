use std::collections::HashMap;
use crate::*;

/// for a big lottery only
#[derive(
    BorshSerialize, 
    BorshDeserialize, 
    Serialize,
    PartialEq,
    PartialOrd, 
    Eq,
    Clone,
    Debug,
    Hash
)]
#[serde(crate = "near_sdk::serde")]
pub enum WinnerType {
    UpToFiftyPercent,
    UpToTenPercent,
    Cashback
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BigLottery {
    pub id: LotteryId,
    pub lottery_token_id: AccountId,
    pub lottery_status: LotteryStatus,
    /// A list of account_ids in this lottery
    pub entries: Vec<AccountId>,
    /// Amount to participate a lottery
    pub entry_fee: Balance,
    /// Current amount deposited
    pub current_pool: Balance,
    /// Required total amount for lottery to start
    pub required_pool: Balance,
    pub winners: HashMap<WinnerType, Vec<AccountId>>,
    pub cashbacked_num: u32,
    pub ten_percent_winners_num: u32,
    pub fifty_percent_winners_num: u32,
    pub refferal_transfered: Balance
}

impl BigLottery {
    pub fn new(
        id: LotteryId,
        lottery_token_id: AccountId,
        num_participants: u32,
        entry_fee: Balance 
    ) -> Self {
        //for cashback count
        assert!(num_participants % 2 == 0, "Number of participants must be divisible by two");
        assert!(num_participants % 5 == 0, "Number of participants must be divisible by five");
        assert!(num_participants <= 64, "Unexpected random result for more than 64 participants");
        let required_pool:Balance = match entry_fee.checked_mul(num_participants as u128) {
            Some(amount) => amount,
            None => panic!("Incorrect lottery setup, math overflow through  `entry_fee * num_participants`"),
        };
        assert!(required_pool % 2 == 0, "required_pool must be divisible by two");
        let lottery = Self {
            id,
            lottery_token_id,
            lottery_status: LotteryStatus::Active,
            entries: vec![],
            entry_fee,
            current_pool: 0,
            required_pool,
            winners: HashMap::new(),
            cashbacked_num: num_participants / 2,
            ten_percent_winners_num: num_participants / 2 - num_participants / 5,
            fifty_percent_winners_num: num_participants / 5,
            refferal_transfered: 0
        };
        lottery.assert_valid();
        lottery
    }

    fn assert_valid(&self) {
        assert!(self.entry_fee > 0, "entry_fee cannot be zero");
        assert!(self.required_pool > 0, "num_participants cannot be zero");
    }

    fn assert_equals_pool(&self) {
        assert_eq!(
            self.current_pool, self.required_pool,
            "Current pool must be equal to Required. Current: {} Required: {} ",
            self.current_pool, self.required_pool,
        );
    }

    fn is_pools_equal(&self) -> bool {
        if self.current_pool == self.required_pool {
            true
        } else {
            false
        }
    }

    fn is_finished(&self) -> bool {
        if self.lottery_status == LotteryStatus::Finished && self.is_pools_equal() && !self.winners.is_empty() {
            true
        } else {
            false
        }
    }

    // fn contains_entry(&self, account_id: &AccountId) -> bool {
    //     self.entries
    //         .iter()
    //         .any(|entry| &entry.account_id == account_id)
    // }

    pub fn assert_is_finished(&self) {
        self.assert_equals_pool();
        assert!(!self.winners.is_empty());
    }

    pub fn add_refferal_transfered(&mut self, amount: Balance) {
        self.refferal_transfered += amount
    }

    pub fn update(&mut self) -> LotteryStatus {
        if self.is_pools_equal() {
            self.lottery_status = LotteryStatus::Finished;
            self.set_winner();
        }
        self.lottery_status
    }

    /// Draw lottery entry
    pub fn draw_enter(&mut self, account_id: &AccountId, amount: Balance) -> LotteryStatus {
        if !self.is_finished() {
            assert_eq!(
                amount, self.entry_fee,
                "Supplied: {}, but Required amount to paticipate is: {}",
                self.entry_fee, amount
            );
            assert!(!self.entries.contains(account_id), "Already entered");
            self.entries.push(account_id.clone());
            self.current_pool += amount;
        }

        // check is required pool filled now and always return a lottery status
        self.update()
    }

    fn set_winner(&mut self) {
        // 50 accounts
        let total_entries = self.entries.len();
        let shuffled_entries = shuffle(self.entries.clone());
        check_account_duplicates(&shuffled_entries);

        let up_to_fifty_num = self.fifty_percent_winners_num as usize;
        let up_to_ten_num = self.ten_percent_winners_num as usize;

        let up_to_fifty_vec = &shuffled_entries[..up_to_fifty_num];
        let up_to_ten_vec = &shuffled_entries[up_to_fifty_num..up_to_ten_num + up_to_fifty_num];
        let cashback_vec = &shuffled_entries[up_to_ten_num + up_to_fifty_num..];

        assert_eq!(cashback_vec.len(), total_entries / 2, "Incorrect cashback accounts num");
        assert_eq!(up_to_fifty_vec.len(), up_to_fifty_num, "Incorrect winners +50% accounts num");
        assert_eq!(up_to_ten_vec.len(), up_to_ten_num, "Incorrect winners +10% accounts num");

        assert!(
            up_to_fifty_num + up_to_ten_num == total_entries / 2,
            "Mismatched len of rewarded and cashback accounts: Rewarded {} Cashback {}",
            up_to_fifty_num + up_to_ten_num, total_entries / 2,
        );

        self.winners.insert(WinnerType::UpToFiftyPercent, up_to_fifty_vec.to_vec());
        self.winners.insert(WinnerType::UpToTenPercent, up_to_ten_vec.to_vec());
        self.winners.insert(WinnerType::Cashback, cashback_vec.to_vec());
    }

    pub fn get_winners(&self, winner_type: WinnerType) -> &[AccountId] {
        match winner_type {
            WinnerType::UpToFiftyPercent => {
                self.winners.get(&WinnerType::UpToFiftyPercent).expect("No required winners found")
            },
            WinnerType::UpToTenPercent => {
                self.winners.get(&WinnerType::UpToTenPercent).expect("No required winners found")
            },
            WinnerType::Cashback => {
                self.winners.get(&WinnerType::Cashback).expect("No required winners found")
            },
        }
    }
}

fn check_account_duplicates(checked_vec: &Vec<AccountId>) {
    let mut dedup = checked_vec.clone();
    dedup.dedup();
    assert_eq!(
        &dedup, checked_vec,
        "Duplicated accounts found"
    )
}