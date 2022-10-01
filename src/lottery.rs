use crate::*;

pub const SIMPLE_LOTTERY:&str = "SIMPLE_LOTTERY";
pub const BIG_LOTTERY:&str = "BIG_LOTTERY";

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum Lottery {
    SimpleLottery(SimpleLottery),
    Lottery(BigLottery)
}  

#[derive(BorshSerialize, BorshDeserialize, Serialize, PartialEq, Copy, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum LotteryStatus {
    Active,
    Finished
}

impl Lottery {
    pub fn kind(&self) -> String {
        match self {
            Lottery::SimpleLottery(_) => SIMPLE_LOTTERY.into(),
            Lottery::Lottery(_) => BIG_LOTTERY.into(),
        }
    }

    pub fn get_id(&self) -> LotteryId {
        match self {
            Lottery::SimpleLottery(lottery) => lottery.id,
            Lottery::Lottery(lottery) => lottery.id,
        }
    }

    pub fn num_participants(&self) -> u32 {
        match self {
            Lottery::SimpleLottery(lottery) => (lottery.required_pool / lottery.entry_fee) as _,
            Lottery::Lottery(lottery) => (lottery.required_pool / lottery.entry_fee) as _,
        }
    }

    pub fn entry_fee(&self) -> Balance {
        match self {
            Lottery::SimpleLottery(lottery) => lottery.entry_fee,
            Lottery::Lottery(lottery) => lottery.entry_fee,
        }
    }

    pub fn update(&mut self) -> LotteryStatus {
        match self {
            Lottery::SimpleLottery(lottery) => {
                lottery.update()
            },
            Lottery::Lottery(lottery) => {
                lottery.update()
            },
        }
    }
}

impl Contract {
    pub (crate) fn internal_get_lottery_by_parameters(
        &self,
        num_participants: u32,
        entry_fee: Balance
    ) -> Option<Lottery> {
        self.lotteries
            .values()
            .find(|lottery| lottery.entry_fee() == entry_fee && lottery.num_participants() == num_participants)
    }

    pub (crate) fn internal_get_lottery(&self, lottery_id: LotteryId) -> Option<Lottery> {
        self.lotteries
            .get(&lottery_id)
            .map(|mut lottery| {
                //always updated status
                lottery.update();
                lottery
            })
    }

    // pub (crate) fn internal_unwrap_lottery(&self, lottery_id: LotteryId) -> Lottery {
    //     self.internal_get_lottery(lottery_id).expect("Lottery was not found")
    // }

    pub (crate) fn internal_set_lottery(&mut self, lottery_id: &LotteryId, lottery: Lottery) {
        self.lotteries.insert(lottery_id, &lottery);
    }

    pub fn distribute(&mut self, lottery: Lottery) -> bool {
        match lottery {
            Lottery::SimpleLottery(lottery) => {
                lottery.assert_is_finished();

                let winner_id = lottery.get_winner_unwrap();
                let reward = lottery.current_pool;
                let mut contract_fees = ratio(reward, self.get_contract_fee_ratio());
                let treasury_fees = self.get_treasury_taken_amount(contract_fees);
                let investor_fees = self.get_investor_taken_amount(contract_fees);
        
                // take contract fees from reward
                let reward_fees_taken = reward - contract_fees;
                assert!(reward_fees_taken > contract_fees, "Reward cannot be less than contract fees");
                assert!(
                    contract_fees > treasury_fees + investor_fees, 
                    "Contract fees cannot be less than treasury & investor fees"
                );
                contract_fees -= treasury_fees + investor_fees;

                // transfer all fees & reward
                Promise::new(winner_id.clone()).transfer(reward_fees_taken);
                log!("Winner is @{} reward is {} yoctoNEAR", winner_id, reward_fees_taken);

                if treasury_fees > 0 {
                    Promise::new(self.treasury()).transfer(treasury_fees);
                    log!("Treasury transfered {} yoctoNEAR", treasury_fees);
                }

                if investor_fees > 0 {
                    Promise::new(self.investor()).transfer(investor_fees);
                    log!("Investor transfered {} yoctoNEAR", investor_fees);
                }      

                self.fees += contract_fees;
                true
            },
            Lottery::Lottery(lottery) => {
                lottery.assert_is_finished();

                let winners = lottery.get_winners();

                let reward_fifty_percents_up = lottery.entry_fee + lottery.entry_fee / 2;
                let reward_ten_percents_up = lottery.entry_fee + lottery.entry_fee / 10;
                let cashback = lottery.entry_fee / 2;

                let exact_reward = 
                    reward_fifty_percents_up * lottery.fifty_percent_winners_num as u128
                        + reward_ten_percents_up * lottery.ten_percent_winners_num as u128
                            + cashback * lottery.cashbacked_num as u128;
                println!(
                    "lottery.current_pool: {}, exact_reward: {} ",
                    lottery.current_pool, exact_reward
                );

                assert!(lottery.current_pool > exact_reward, "Current pool amount must be greater than exact transfered reward");

                let mut contract_fees = lottery.current_pool - exact_reward;
                
                let treasury_fees = self.get_treasury_taken_amount(contract_fees);
                let investor_fees = self.get_investor_taken_amount(contract_fees);

                assert!(exact_reward > contract_fees, "Exact Reward cannot be less than contract fees");
                assert!(
                    contract_fees > treasury_fees + investor_fees, 
                    "Contract fees cannot be less than treasury & investor fees"
                );

                contract_fees -= treasury_fees + investor_fees;

                //transfers
                for (winner_type, account_id) in winners {
                    let amount = match winner_type {
                        WinnerType::UpToFiftyPercent => reward_fifty_percents_up,
                        WinnerType::UpToTenPercent => reward_ten_percents_up,
                        WinnerType::Cashback => cashback,
                    };
                    log!("Transfered {} yoctoNEAR to @{}", amount, &account_id);
                    Promise::new(account_id).transfer(amount);
                }

                if treasury_fees > 0 {
                    Promise::new(self.treasury()).transfer(treasury_fees);
                    log!("Treasury transfered {} yoctoNEAR", treasury_fees);
                }

                if investor_fees > 0 {
                    Promise::new(self.investor()).transfer(investor_fees);
                    log!("Investor transfered {} yoctoNEAR", investor_fees);
                }  
                
                self.fees += contract_fees;
                true
            },
        }
    }

    pub fn add_new_lottery(
        &mut self,
        lottery_type: LotteryType,
        num_participants: u32,
        entry_fee: U128
    ) -> Lottery {
        self.assert_required_entry_fees(entry_fee.0, lottery_type);
        self.assert_required_num_participants(num_participants, lottery_type);

        let lottery_id = self.next_lottery_id;
        let lottery = match lottery_type {
            LotteryType::SimpleLottery => {
                Lottery::SimpleLottery(
                    SimpleLottery::new(
                        lottery_id,
                        num_participants,
                        entry_fee.0
                    )
                )
            },
            LotteryType::BigLottery => {
                Lottery::Lottery(
                    BigLottery::new(
                        lottery_id,
                        num_participants,
                        entry_fee.0
                    )
                )
            },
        };
        self.next_lottery_id += 1;
        lottery
    }
}
    
#[near_bindgen]
impl Contract {
    #[payable]
    pub fn draw_near_enter(
        &mut self, 
        lottery_type: String,
        num_participants: u32,
        entry_fee: U128
    ) -> LotteryId {
        let account_id = env::predecessor_account_id();
        let attached_deposit = env::attached_deposit();
        let lottery_type = LotteryType::from(lottery_type);

        let lottery = match self.internal_get_lottery_by_parameters(num_participants, entry_fee.0) {
            Some(lottery) => lottery,
            None => {
                self.add_new_lottery(lottery_type, num_participants, entry_fee)
            },
        };

        // overchecking (acknowleged)
        assert_eq!(attached_deposit, entry_fee.0, "Attached amount need to be equal to entry fee");

        let lottery_id = lottery.get_id();
        match lottery {
            Lottery::SimpleLottery(mut simple_lottery) => {
                let (lottery_status, unused_amount) = simple_lottery.draw_near_enter(&account_id, attached_deposit);
                if unused_amount > 0 {
                    // user was last + 1 for that simple_lottery. Need to create new one
                    panic!("Unexpected behaviour");
                } else {
                    match lottery_status {
                        // user was last for that lottery. Need to distribute reward                   
                        LotteryStatus::Finished => {
                            let success_distribute = self.distribute(Lottery::SimpleLottery(simple_lottery));
                            if success_distribute {
                                self.lotteries.remove(&lottery_id);
                            }
                        },
                        // user just created entry for that lottery
                        LotteryStatus::Active => {
                            self.internal_set_lottery(&lottery_id, Lottery::SimpleLottery(simple_lottery))
                        }
                    }
                }; 
                lottery_id
            },
            Lottery::Lottery(mut big_lottery) => {
                let (lottery_status, unused_amount) = big_lottery.draw_near_enter(&account_id, attached_deposit);
                if unused_amount > 0 {
                    // user was last + 1 for that big_lottery. Need to create new one
                    panic!("Unexpected behaviour");
                } else {
                    match lottery_status {
                        // user was last for that lottery. Need to distribute reward                   
                        LotteryStatus::Finished => {
                            let success_distribute = self.distribute(Lottery::Lottery(big_lottery));
                            if success_distribute {
                                self.lotteries.remove(&lottery_id);
                            }
                        },
                        // user just created entry for that lottery
                        LotteryStatus::Active => {
                            self.internal_set_lottery(&lottery_id, Lottery::Lottery(big_lottery))
                        }
                    }
                }; 
                lottery_id
            },
        }
    }
}