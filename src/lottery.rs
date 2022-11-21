use near_sdk::{require, json_types::U64};

use crate::{*, views::{LotteryResult, SimpleLotteryResult, BigLotteryResult}};

pub const ONE_PERCENT_RATIO:u32 = MAX_RATIO / 100;

pub const SIMPLE_LOTTERY:&str = "SIMPLE_LOTTERY";
pub const BIG_LOTTERY:&str = "BIG_LOTTERY";

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum Lottery {
    SimpleLottery(SimpleLottery),
    Lottery(BigLottery)
}  

// #[derive(BorshSerialize, BorshDeserialize, Serialize, Clone)]
// //#[cfg_attr(not(target_arch = "wasm32"), derive(Clone))]
// #[serde(crate = "near_sdk::serde")]
// pub struct Entry{
//     pub account_id: AccountId,
//     pub referrer_id: Option<AccountId>
// }

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

    pub fn status(&self) -> LotteryStatus {
        match self {
            Lottery::SimpleLottery(lottery) => lottery.lottery_status,
            Lottery::Lottery(lottery) => lottery.lottery_status,
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

    pub fn lottery_token_id(&self) -> &AccountId {
        match self {
            Lottery::SimpleLottery(lottery) => &lottery.lottery_token_id,
            Lottery::Lottery(lottery) => &lottery.lottery_token_id,
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
    pub (crate) fn check_accepted_subs(&self, account_id: &AccountId) {
        let accepted_subs = self.accepted_subs();
        let accepted_sub = accepted_subs
            .split('.')
            .collect::<Vec<_>>();
        let stringified_account = account_id
            .to_string();
        let splitted_acc = stringified_account
            .split('.')
            .collect::<Vec<_>>();
        require!(splitted_acc.len() == 3, "Expected len for subaccounts");
        require!(
            accepted_sub[0] == splitted_acc[1] && accepted_sub[1] == splitted_acc[2], 
            format!("Incorrect subaccount. Accepted subs is {}", accepted_subs)
        );
    }
    pub (crate) fn internal_get_lottery_by_parameters(
        &self,
        token_id: &AccountId,
        num_participants: u32,
        entry_fee: Balance
    ) -> Option<Lottery> {
        self.lotteries
            .values()
            .find(|lottery| {
                lottery.entry_fee() == entry_fee 
                    && lottery.num_participants() == num_participants
                        && lottery.lottery_token_id() == token_id
            })
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

    pub (crate) fn update_cashback_storage(&mut self) {
        let cashback_accounts = self.cashback_accounts.to_vec();
        for (token_id, stored_cashback) in cashback_accounts.iter() {
            if token_id == &near() {
                for account in &stored_cashback.accounts {
                    //log!("Cashback transfered ( {} yoctoNEAR ) to @{}", stored_cashback.amount, account);
                    Promise::new(account.clone()).transfer(stored_cashback.amount);
                }
            } else {
                for account in &stored_cashback.accounts {
                    //log!("Cashback transfered ( {} yocto {} ) to @{}", token_id, stored_cashback.amount, account);
                    self.internal_ft_transfer(account, token_id, stored_cashback.amount);
                }
            }
        }
    }

    pub (crate) fn internal_set_lottery(&mut self, lottery_id: &LotteryId, lottery: Lottery) {
        if !self.cashback_accounts.is_empty() {
            self.update_cashback_storage();
        }    
        self.lotteries.insert(lottery_id, &lottery);
    }

    pub fn draw_enter(
        &mut self,
        entry_account_id: &AccountId,
        lottery_token_id: AccountId,
        lottery_type: LotteryType,
        num_participants: u32,
        entry_fee: Balance,
        referrer_id: Option<AccountId>
    ) -> LotteryId {

        let lottery = match self.internal_get_lottery_by_parameters(&lottery_token_id, num_participants, entry_fee) {
            Some(lottery) => lottery,
            None => {
                self.add_new_lottery(
                    lottery_token_id.clone(),
                    lottery_type, 
                    num_participants, 
                    entry_fee
                )
            },
        };

        let lottery_id = lottery.get_id();
        match lottery {
            Lottery::SimpleLottery(mut simple_lottery) => {
                let lottery_status = simple_lottery.draw_enter(entry_account_id, entry_fee);
                
                if let Some(refferer) = referrer_id {
                    let referrer_reward = ratio(entry_fee, ONE_PERCENT_RATIO);
                    if lottery_token_id == near() {
                        Promise::new(refferer).transfer(referrer_reward);
                    } else {
                        self.internal_ft_transfer(&refferer, &lottery_token_id, referrer_reward);
                    }
                    simple_lottery.add_refferal_transfered(referrer_reward);
                }

                match lottery_status {
                    // user was last for that lottery. Need to distribute reward                   
                    LotteryStatus::Finished => {
                        let lottery_result = self.distribute(Lottery::SimpleLottery(simple_lottery));
                        log!("{:#?}", lottery_result);
                        self.lotteries.remove(&lottery_id);
                    },
                    // user just created entry for that lottery
                    LotteryStatus::Active => {
                        self.internal_set_lottery(&lottery_id, Lottery::SimpleLottery(simple_lottery))
                    }
                } 
                lottery_id
            },
            Lottery::Lottery(mut big_lottery) => {
                let lottery_status = big_lottery.draw_enter(entry_account_id, entry_fee);
                
                if let Some(refferer) = referrer_id {
                    let referrer_reward = ratio(entry_fee, ONE_PERCENT_RATIO);
                    if lottery_token_id == near() {
                        Promise::new(refferer).transfer(referrer_reward);
                    } else {
                        self.internal_ft_transfer(&refferer, &lottery_token_id, referrer_reward);
                    }
                    big_lottery.add_refferal_transfered(referrer_reward);
                }

                match lottery_status {
                    // user was last for that lottery. Need to distribute reward                   
                    LotteryStatus::Finished => {
                        let lottery_result = self.distribute(Lottery::Lottery(big_lottery));
                        log!("{:#?}", lottery_result);
                        self.lotteries.remove(&lottery_id);
                    },
                    // user just created entry for that lottery
                    LotteryStatus::Active => {
                        self.internal_set_lottery(&lottery_id, Lottery::Lottery(big_lottery))
                    }
                } 
                lottery_id
            },
        }
    }

    pub fn deposit_fees(&mut self, token_id: &AccountId, amount: Balance) {
        let mut fee_amount = self.fees.get(token_id).unwrap_or_default();
        fee_amount += amount;
        self.fees.insert(token_id, &fee_amount);
    }

    pub fn distribute(&mut self, lottery: Lottery) -> LotteryResult {
        match lottery {
            Lottery::SimpleLottery(lottery) => {
                lottery.assert_is_finished();
                let lottery_token_id = lottery.lottery_token_id.clone();

                let winner_id = lottery.get_winner_unwrap();

                let reward = lottery.current_pool;
                let mut contract_fees = ratio(reward, self.get_contract_fee_ratio());
                assert!(reward > contract_fees, "Reward cannot be less than contract fees");
                // take contract fees from reward
                let reward_fees_taken = reward - contract_fees;
                if lottery.refferal_transfered > 0 {
                    assert!(contract_fees > lottery.refferal_transfered, "Refferal's reward cannot be greater than contract fees");
                    contract_fees -= lottery.refferal_transfered;
                }
                let treasury_fees = self.get_treasury_taken_amount(contract_fees);
                let investor_fees = self.get_investor_taken_amount(contract_fees);
                assert!(
                    contract_fees >= treasury_fees + investor_fees, 
                    "Contract fees cannot be less than treasury & investor fees"
                );
                contract_fees -= treasury_fees + investor_fees;

                // transfer all fees & reward
                if lottery_token_id == near() {
                    //todo - add callback here
                    Promise::new(winner_id.clone()).transfer(reward_fees_taken);

                    if treasury_fees > 0 {
                        Promise::new(self.treasury()).transfer(treasury_fees);
                    }
    
                    if investor_fees > 0 {
                        Promise::new(self.investor()).transfer(investor_fees);
                    } 
                } else {
                    //todo - add callback here
                    self.internal_ft_transfer(&winner_id, &lottery_token_id, reward_fees_taken);

                    if treasury_fees > 0 {
                        self.internal_ft_transfer(&winner_id, &lottery_token_id, treasury_fees);
                    }
    
                    if investor_fees > 0 {
                        self.internal_ft_transfer(&winner_id, &lottery_token_id, investor_fees);
                    } 
                }  

                if contract_fees > 0 {
                    self.deposit_fees(&lottery_token_id, contract_fees);
                }
                
                LotteryResult::SimpleLotteryResult( 
                    SimpleLotteryResult {
                        lottery_id: U64(lottery.id),
                        lottery_token_id,
                        participants: lottery.entries,
                        winner: winner_id,
                        winning_amount: U128(reward_fees_taken),
                        contract_fee: U128(contract_fees),
                    }
                )
            },
            Lottery::Lottery(lottery) => {
                lottery.assert_is_finished();
                let lottery_token_id = lottery.lottery_token_id.clone();

                let reward_fifty_percents_up = lottery.entry_fee + lottery.entry_fee / 2;
                let reward_ten_percents_up = lottery.entry_fee + lottery.entry_fee / 10;
                let cashback = lottery.entry_fee / 2;

                let exact_reward = 
                    reward_fifty_percents_up * lottery.fifty_percent_winners_num as u128
                        + reward_ten_percents_up * lottery.ten_percent_winners_num as u128
                            + cashback * lottery.cashbacked_num as u128;

                assert!(lottery.current_pool > exact_reward, "Current pool amount must be greater than exact transfered reward");

                let mut contract_fees = lottery.current_pool - exact_reward;

                if lottery.refferal_transfered > 0 {
                    assert!(contract_fees > lottery.refferal_transfered, "Refferal's reward cannot be greater than contract fees");
                    contract_fees -= lottery.refferal_transfered;
                }
                
                let treasury_fees = self.get_treasury_taken_amount(contract_fees);
                let investor_fees = self.get_investor_taken_amount(contract_fees);

                assert!(exact_reward > contract_fees, "Exact Reward cannot be less than contract fees");
                assert!(
                    contract_fees > treasury_fees + investor_fees, 
                    "Contract fees cannot be less than treasury & investor fees"
                );

                contract_fees -= treasury_fees + investor_fees;

                let cashbacked_accounts = lottery.get_winners(WinnerType::Cashback);
                log!("total cashbacked accounts: {}", cashbacked_accounts.len());
                self.cashback_accounts.insert(
                    &lottery_token_id, 
                    &StoredCashback { 
                        amount: cashback, 
                        accounts: cashbacked_accounts.to_vec() 
                    });
                
                let up_to_fifty_winners = lottery.get_winners(WinnerType::UpToFiftyPercent);
                let up_to_ten_winners = lottery.get_winners(WinnerType::UpToTenPercent);

                if lottery_token_id == near() {
                    //transfers NEAR
                    for account in up_to_fifty_winners {
                        Promise::new(account.clone()).transfer(reward_fifty_percents_up);
                        log!("Reward up to 50% transfered ( {} yoctoNEAR ) to @{} ", reward_fifty_percents_up, account);
                    }

                    for account in up_to_ten_winners {
                        Promise::new(account.clone()).transfer(reward_ten_percents_up);
                        log!("Reward up to 10% transfered ( {} yoctoNEAR ) to @{} ", reward_ten_percents_up, account);
                    }

                    if treasury_fees > 0 {
                        Promise::new(self.treasury()).transfer(treasury_fees);
                    }
    
                    if investor_fees > 0 {
                        Promise::new(self.investor()).transfer(investor_fees);
                    }
                } else {
                    //transfers FT
                    for account in up_to_fifty_winners {
                        self.internal_ft_transfer(account, &lottery_token_id, reward_fifty_percents_up);
                        log!("Reward up to 50% transfered ( {} yocto{}) to @{} ", reward_fifty_percents_up, match_token_id(&lottery_token_id), account);
                    }

                    for account in up_to_ten_winners {
                        self.internal_ft_transfer(account, &lottery_token_id, reward_ten_percents_up);
                        log!("Reward up to 10% transfered ( {} yocto{}) to @{} ", reward_ten_percents_up, match_token_id(&lottery_token_id), account);
                    }

                    if treasury_fees > 0 {
                        self.internal_ft_transfer(&self.treasury(), &lottery_token_id, treasury_fees);
                    }
    
                    if investor_fees > 0 {
                        self.internal_ft_transfer(&self.investor(), &lottery_token_id, investor_fees);
                    }
                }  
                
                self.deposit_fees(&lottery_token_id, contract_fees);

                LotteryResult::BigLotteryResult( 
                    BigLotteryResult {
                        lottery_id: U64(lottery.id),
                        lottery_token_id,
                        participants: lottery.entries.clone(),
                        winners_up_to_50: up_to_fifty_winners.to_vec(),
                        winners_up_to_10: up_to_ten_winners.to_vec(),
                        total_winning_amount: U128(lottery.current_pool),
                        contract_fee: U128(contract_fees),
                    }
                )
            },
        }
    }

    pub fn add_new_lottery(
        &mut self,
        lottery_token_id: AccountId,
        lottery_type: LotteryType,
        num_participants: u32,
        entry_fee: Balance
    ) -> Lottery {
        self.assert_required_entry_fees(&lottery_token_id, entry_fee, lottery_type);
        self.assert_required_num_participants(num_participants, lottery_type);

        let lottery_id = self.next_lottery_id;
        let lottery = match lottery_type {
            LotteryType::SimpleLottery => {
                Lottery::SimpleLottery(
                    SimpleLottery::new(
                        lottery_id,
                        lottery_token_id,
                        num_participants,
                        entry_fee
                    )
                )
            },
            LotteryType::BigLottery => {
                Lottery::Lottery(
                    BigLottery::new(
                        lottery_id,
                        lottery_token_id,
                        num_participants,
                        entry_fee
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
        referrer_id: Option<AccountId>
    ) -> LotteryId {
        let account_id = env::predecessor_account_id();
        let attached_deposit = env::attached_deposit();

        self.check_accepted_subs(&account_id);
        
        self.draw_enter(
            &account_id, 
            near(), 
            LotteryType::from(lottery_type), 
            num_participants, 
            attached_deposit,
            referrer_id
        )
    }
}