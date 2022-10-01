use crate::*;

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractParams {
    pub fees_collected: U128,
    pub config: ConfigView
}

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ConfigView {
    /// contract owner
    pub owner_id: AccountId,
    /// fees taken from prize pool to contract
    pub contract_fee_ratio: u32,
    /// lotteries config
    pub entry_fees_required: Vec<U128>,
    pub num_participants_required: Vec<(LotteryType, Vec<u32>)>
}

#[derive(Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LotteryView {
    pub lottery_status: LotteryStatus,
    /// A list of lottery_ids in this lottery
    pub entries: Vec<AccountId>,
    /// Amount to participate a lottery
    pub entry_fee: U128,
    /// Current amount deposited
    pub current_pool: U128,
    /// Required total amount for lottery to start
    pub required_pool: U128,
    pub winners: Option<Vec<AccountId>>
}

impl Contract {
    pub fn get_contract_params(&self) -> ContractParams {
        let config_internal = self.internal_config();
        let config = ConfigView {
            owner_id: config_internal.owner_id,
            contract_fee_ratio: config_internal.contract_fee_ratio,
            entry_fees_required: config_internal
                .lotteries_config
                .entry_fees
                .iter()
                .map(|fee_amount| U128(*fee_amount))
                .collect::<Vec<_>>(),
            num_participants_required: vec![
                (LotteryType::SimpleLottery, config_internal.lotteries_config.num_participants),
                (LotteryType::BigLottery, config_internal.lotteries_config.big_lottery_num_participants),
            ],
        };

        ContractParams { 
            fees_collected: self.fees.into(), 
            config
        }
    }
    pub fn get_lottery_view(&self, lottery: Lottery) -> LotteryView {
        match lottery {
            Lottery::Lottery(lottery) => {
                LotteryView { 
                    lottery_status: lottery.lottery_status, 
                    entries: lottery.entries, 
                    entry_fee: lottery.entry_fee.into(), 
                    current_pool: lottery.current_pool.into(), 
                    required_pool: lottery.required_pool.into(), 
                    winners: None //unimplemented for view
                }
            },
            Lottery::SimpleLottery(lottery) => {
                let winners = match lottery.winner {
                    Some(winner) => Some(vec![winner.clone()]),
                    None => None,
                };
                LotteryView { 
                    lottery_status: lottery.lottery_status, 
                    entries: lottery.entries, 
                    entry_fee: lottery.entry_fee.into(), 
                    current_pool: lottery.current_pool.into(), 
                    required_pool: lottery.required_pool.into(), 
                    winners
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