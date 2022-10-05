use crate::*;

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractParams {
    pub fees_collected: Vec<(AccountId, U128)>,
    pub config: ConfigView,
    pub cashback_accounts_num: Vec<(AccountId, u32)>,
    pub whitelisted_tokens: Vec<AccountId>
}

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct BigLotteryParams {
    pub cashbacked_num: u32,
    pub ten_percent_winners_num: u32,
    pub fifty_percent_winners_num: u32,
}

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ConfigView {
    /// contract owner
    pub owner_id: AccountId,
    /// fees taken from prize pool to contract
    pub contract_fee_ratio: u32,
    /// lotteries config
    pub entry_fees_required: Vec<(AccountId, Vec<U128>)>,
    pub num_participants_required: Vec<(LotteryType, Vec<u32>)>
}

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LotteryView {
    pub lottery_token_id: AccountId,
    pub lottery_status: LotteryStatus,
    /// A list of lottery_ids in this lottery
    pub entries: Vec<AccountId>,
    /// Amount to participate a lottery
    pub entry_fee: U128,
    /// Current amount deposited
    pub current_pool: U128,
    /// Required total amount for lottery to start
    pub required_pool: U128,
    pub big_lottery_params: Option<BigLotteryParams>
}

impl Contract {
    pub fn get_contract_view(&self) -> ContractParams {
        let config_internal = self.internal_config();
        let lottery_config_internal = self.internal_lottery_config();
        let config = ConfigView {
            owner_id: config_internal.owner_id,
            contract_fee_ratio: config_internal.contract_fee_ratio,
            entry_fees_required: lottery_config_internal
                .entry_fees
                .into_iter()
                .collect(),
            num_participants_required: vec![
                (LotteryType::SimpleLottery, lottery_config_internal.num_participants),
                (LotteryType::BigLottery, lottery_config_internal.big_lottery_num_participants),
            ],
        };

        ContractParams { 
            fees_collected: self.fees.to_vec().iter().map(|(acc, fee)| (acc.clone(), U128(*fee))).collect::<Vec<_>>(), 
            config,
            cashback_accounts_num: self
                .cashback_accounts
                .iter()
                .map(|(token_id, stored_cashback)| {
                    (token_id, stored_cashback.accounts.len() as u32)
                }) 
                .collect(),
            whitelisted_tokens: self.whitelisted_tokens.to_vec()
        }
    }
    pub fn get_lottery_view(&self, lottery: Lottery) -> LotteryView {
        match lottery {
            Lottery::Lottery(lottery) => {
                LotteryView { 
                    lottery_token_id: lottery.lottery_token_id,
                    lottery_status: lottery.lottery_status, 
                    entries: lottery.entries, 
                    entry_fee: lottery.entry_fee.into(), 
                    current_pool: lottery.current_pool.into(), 
                    required_pool: lottery.required_pool.into(), 
                    big_lottery_params: Some(BigLotteryParams {
                        cashbacked_num: lottery.cashbacked_num,
                        ten_percent_winners_num: lottery.ten_percent_winners_num,
                        fifty_percent_winners_num: lottery.fifty_percent_winners_num,
                    })
                }
            },
            Lottery::SimpleLottery(lottery) => {
                LotteryView { 
                    lottery_token_id: lottery.lottery_token_id,
                    lottery_status: lottery.lottery_status, 
                    entries: lottery.entries, 
                    entry_fee: lottery.entry_fee.into(), 
                    current_pool: lottery.current_pool.into(), 
                    required_pool: lottery.required_pool.into(), 
                    big_lottery_params: None
                }
            }
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn get_lotteries_num(&self) -> u64 {
        self.lotteries.keys_as_vector().len()
    }

    pub fn get_contract_params(&self) -> ContractParams {
        self.get_contract_view()
    }

    /// Returns detailed information about an lottery for a given lottery_id.
    pub fn get_lottery(&self, lottery_id: LotteryId) -> Option<LotteryView> {
        self.internal_get_lottery(lottery_id)
            .map(|lottery| self.get_lottery_view(lottery))
    }

    /// Returns limited lottery information for lotteriess from a given index up to a given limit.
    pub fn get_lotteries_paged(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<Lottery> {
        let values = self.lotteries.values_as_vector();
        let from_index = from_index.unwrap_or(0);
        let limit = limit.unwrap_or(values.len());
        (from_index..std::cmp::min(values.len(), from_index + limit))
            .map(|index| values.get(index).unwrap().into())
            .collect()
    }
}