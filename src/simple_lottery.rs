use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone))]
#[serde(crate = "near_sdk::serde")]
pub struct SimpleLottery {
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
    pub winner: Option<AccountId>
}

impl SimpleLottery {
    pub fn new(
        id: LotteryId,
        num_participants: u32,
        entry_fee: Balance 
    ) -> Self {
        let required_pool:Balance = match entry_fee.checked_mul(num_participants as u128) {
            Some(amount) => amount,
            None => panic!("Incorrect lottery setup, math overflow through  `entry_fee * num_participants`"),
        };
        let lottery = Self {
            id,
            lottery_status: LotteryStatus::Active,
            entries: vec![],
            entry_fee,
            current_pool: 0,
            required_pool,
            winner: None
        };
        lottery.assert_valid();
        lottery
    }

    fn assert_valid(&self) {
        assert!(self.entry_fee > 0, "entry_fee cannot be zero");
        assert!(self.required_pool > 0, "num_participants cannot be zero");
    }
    
    fn get_accounts_num(&self) -> u32 {
        self.entries.len() as _
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
        if self.lottery_status == LotteryStatus::Finished && self.is_pools_equal() && self.winner.is_some() {
            true
        } else {
            false
        }
    }

    pub fn assert_is_finished(&self) {
        self.assert_equals_pool();
        assert!(self.winner.is_some());
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
        let random_seed = env::random_seed_array();
        let total_entries = self.get_accounts_num();
        let random_seed_sum = random_seed.iter().sum::<u8>() as u32;
        let winner = &self.entries[(random_seed_sum%total_entries) as usize];
       
        self.winner = Some(winner.clone());
    }

    pub fn get_winner_unwrap(&self) -> AccountId {
        match &self.winner {
            Some(winner) => winner.clone(),
            None => panic!("Lottery has no winner"),
        }
    }
}
  