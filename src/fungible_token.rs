use crate::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::{Gas, ext_contract, is_promise_success, serde_json, PromiseOrValue, ONE_YOCTO};

const GAS_FOR_FT_TRANSFER: Gas = Gas(Gas::ONE_TERA.0 * 10);
const GAS_FOR_AFTER_FT_TRANSFER: Gas = Gas(Gas::ONE_TERA.0 * 20);

#[ext_contract(ext_ft)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) -> Promise;
}

/// Draw FT Enter with required num participants
/// This is `msg` from fungible token transfer
#[derive(Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, Serialize))]
#[serde(crate = "near_sdk::serde")]
pub enum TokenReceiverMsg {
    DrawEnter {
        num_participants: u32,
        lottery_type: String,
        referrer_id: Option<AccountId>
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Receives the transfer from the fungible token and executes a list of actions given in the
    /// message on behalf of the sender. The actions that can be executed should be limited to a set
    /// that doesn't require pricing.
    /// - Requires to be called by the fungible token account.
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_id = env::predecessor_account_id();
        assert!(self.whitelisted_tokens.contains(&token_id), "Token is not whitelisted");

        let token_receiver_msg: TokenReceiverMsg =
            serde_json::from_str(&msg).expect("Can't parse TokenReceiverMsg");

        match token_receiver_msg {
            TokenReceiverMsg::DrawEnter { 
                num_participants, 
                lottery_type,
                referrer_id
            } => {
                let lottery_id = self.draw_enter(
                    &sender_id,
                    token_id,
                    LotteryType::from(lottery_type),
                    num_participants,
                    amount.0,
                    referrer_id
                );
                log!("Draw enter. Lottery ID: {}, account: @{}", lottery_id, sender_id);
            },
        }

        PromiseOrValue::Value(U128(0))
    }
}

impl Contract {
    pub fn internal_ft_transfer(
        &mut self,
        account_id: &AccountId,
        token_id: &AccountId,
        amount: Balance,
    ) -> Promise {
        ext_ft::ext(token_id.clone())
            .with_attached_deposit(ONE_YOCTO)
            .with_static_gas(GAS_FOR_FT_TRANSFER)
        .ft_transfer(
            account_id.clone(), 
            amount.into(), 
            None
        )
    }

    pub fn internal_ft_transfer_checked(
        &mut self,
        account_id: &AccountId,
        token_id: &AccountId,
        amount: Balance,
    ) -> Promise {
        ext_ft::ext(token_id.clone())
            .with_attached_deposit(ONE_YOCTO)
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer(
                account_id.clone(), 
                amount.into(), 
                Some("withdrawal from vault supplied".into())
            )
        .then(Self::ext(env::current_account_id())
            .with_static_gas(GAS_FOR_AFTER_FT_TRANSFER)
            .after_ft_transfer(account_id.clone(), token_id.clone(), amount)
        )
    }
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn after_ft_transfer(&mut self, account_id: AccountId, token_id: AccountId, amount: Balance)
        -> bool;
}

#[near_bindgen]
impl ExtSelf for Contract {
    #[private]
    fn after_ft_transfer(
        &mut self,
        account_id: AccountId,
        token_id: AccountId,
        amount: Balance,
    ) -> bool {
        let promise_success = is_promise_success();
        if !promise_success {
            //revert to store and do stuff again
        }
        promise_success
    }
}
