use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Copy, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum LotteryType {
    SimpleLottery,
    BigLottery
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
    pub entry_fees: Vec<Balance>,
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
        self.num_participants.push(num)
    }

    pub fn remove_num_participants(&mut self, num: u32) {
        let index = self.num_participants.iter().position(|x| x == &num).expect("invalid num to remove");
        self.num_participants.remove(index);
    }

    pub fn add_big_lottery_num_participants(&mut self, num: u32) {
        self.big_lottery_num_participants.push(num)
    }

    pub fn remove_big_lottery_num_participants(&mut self, num: u32) {
        let index = self.big_lottery_num_participants.iter().position(|x| x == &num).expect("invalid num to remove");
        self.big_lottery_num_participants.remove(index);
    }

    pub fn add_entry_fee(&mut self, fee: Balance) {
        self.entry_fees.push(fee)
    }

    pub fn remove_entry_fee(&mut self, fee: Balance) {
        let index = self.entry_fees.iter().position(|x| x == &fee).expect("invalid fee to remove");
        self.entry_fees.remove(index);
    }
}

impl Contract {

}