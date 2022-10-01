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
    pub lottery_status: LotteryStatus,
    /// A list of account_ids in this lottery
    pub entries: Vec<AccountId>,
    /// Amount to participate a lottery
    pub entry_fee: Balance,
    /// Current amount deposited
    pub current_pool: Balance,
    /// Required total amount for lottery to start
    pub required_pool: Balance,
    pub winners: HashMap<WinnerType, AccountId>,
    pub cashbacked_num: u32,
    pub ten_percent_winners_num: u32,
    pub fifty_percent_winners_num: u32,
}

impl BigLottery {
    pub fn new(
        id: LotteryId,
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
            lottery_status: LotteryStatus::Active,
            entries: vec![],
            entry_fee,
            current_pool: 0,
            required_pool,
            winners: HashMap::new(),
            cashbacked_num: num_participants / 2,
            ten_percent_winners_num: num_participants / 2 - num_participants / 5,
            fifty_percent_winners_num: num_participants / 5,
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

    pub fn assert_is_finished(&self) {
        self.assert_equals_pool();
        assert!(!self.winners.is_empty());
    }

    pub fn update(&mut self) -> LotteryStatus {
        if self.is_pools_equal() {
            self.lottery_status = LotteryStatus::Finished;
            self.set_winner();
        }
        self.lottery_status
    }

    /// Draw lottery entry
    pub fn draw_near_enter(&mut self, account_id: &AccountId, amount: Balance) -> (LotteryStatus, Balance) {
        if !self.is_finished() {
            assert_eq!(
                amount, self.entry_fee,
                "Supplied: {}, but Required amount to paticipate is: {}",
                self.entry_fee, amount
            );
            assert!(!self.entries.contains(account_id), "Already entered");
            self.entries.push(account_id.clone());
            self.current_pool += amount;
            // check is required pool filled now and always return a lottery status
            (self.update(), 0)
        } else {
            (self.update(), amount)
        }
    }

    fn set_winner(&mut self) {
        // 50 accounts
        let total_entries = self.entries.len();
        assert!(total_entries <= 64, "Unexpected random result for more than 64 participants");
        // 32 random bytes
        let random_seed = env::random_seed_array().to_vec();
        let random_seed_taken:Vec<_> = random_seed
            .iter()
            .take(total_entries / 2)
            .collect();

        let mut random_order_vector:Vec<AccountId> = vec![];

        for seed_byte in random_seed_taken {
            let seed_byte = *seed_byte as usize;
            let account = self.entries[seed_byte%total_entries].clone();
            self.entries.remove(seed_byte%total_entries);
            random_order_vector.push(account);
        }

        for index in 0..self.fifty_percent_winners_num {
            self.winners.insert(WinnerType::UpToFiftyPercent, random_order_vector[index as usize].clone());
        }

        for index in self.fifty_percent_winners_num..self.cashbacked_num {
            self.winners.insert(WinnerType::UpToTenPercent, random_order_vector[index as usize].clone());
        }

        for account_id in self.entries.iter() {
            self.winners.insert(WinnerType::Cashback, account_id.clone());
        }
        //TODO - add
        //assert_eq!(self.winners.len(), total_entries, "Mismatched winners/entries for a big lottery");
        //assert!(self.entries.is_empty(), "All Entries must migrate to winners");
        check_account_duplicates(self.winners.values().cloned().collect());
    }

    pub fn get_winners(&self) -> HashMap<WinnerType, AccountId> {
        self.winners.clone()
    }
}

fn check_account_duplicates(checked_vec: Vec<AccountId>) {
    let mut dedup = checked_vec.clone();
    dedup.dedup();
    assert_eq!(
        dedup, checked_vec,
        "Duplicated accounts found"
    )
}