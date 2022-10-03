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
        if s == SIMPLE_LOTTERY.to_string() {
            LotteryType::SimpleLottery
        } else if s == BIG_LOTTERY.to_string() {
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
    pub fn assert_valid(&self) {
        assert!(self.entry_fees.len() > 0);
        assert!(self.num_participants.len() > 0);
        assert!(self.big_lottery_num_participants.len() > 0);
    }

    pub fn add_num_participants(&mut self, num: u32) {
        self.num_participants.push(num);
    }

    pub fn remove_num_participants(&mut self, num: u32) {
        assert!(self.num_participants.len() >= 1, "Cannot remove last lottery num_participants");
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
                .or_insert(vec![])
                .push(fee)
        } else {
            self
                .entry_fees
                .entry(near())
                .or_insert(vec![])
                .push(fee)
        }
    }

    pub fn remove_entry_fee(&mut self, token_id: Option<AccountId>, fee: U128) {
        let (index, token) = if let Some(token_id) = token_id {
            let position = self
                .entry_fees
                .entry(token_id.clone())
                .or_insert(vec![])
                .iter()
                .position(|x| x == &fee).expect("invalid fee to remove");
            (position, token_id)
        } else {
            let position = self
                .entry_fees
                .entry(near())
                .or_insert(vec![])
                .iter()
                .position(|x| x == &fee).expect("invalid fee to remove");
            (position, near())
        };
        
        self
            .entry_fees
            .entry(token)
            .or_insert(vec![])
            .remove(index);
    }
}

impl Contract {

}