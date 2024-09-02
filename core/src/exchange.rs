use near_sdk::{
    ext_contract, is_promise_success, json_types::U128, log, near, AccountId,
    NearToken, Promise, PromiseOrValue,
};

use crate::{ft_receiver::Token, Contract, ContractExt};

#[ext_contract(ext_ft)]
pub trait ExtFt {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128);
}

// Simple exchange rate
// 1 Near = 4 FungibleToken
#[near]
impl Contract {
    pub(crate) fn internal_exchange(
        &mut self,
        sender_id: AccountId,
        token_in: Token,
        token_out: Token,
        amount_out: u128,
    ) -> Promise {
        let destination = AccountId::from(token_out);

        if token_in == Token::Near {
            self.user_points
                .entry(sender_id.clone())
                .and_modify(|points| points.0 += amount_out)
                .or_insert(amount_out.into());
        } else {
            self.user_points
                .entry(sender_id.clone())
                .and_modify(|points| points.0 -= amount_out)
                .or_insert(0.into());
        }

        ext_ft::ext(destination.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_transfer(sender_id, amount_out.into())
    }

    #[private]
    pub fn resolve_exchange(
        &mut self,
        sender_id: AccountId,
        token_in: Token,
        amount: U128,
    ) -> PromiseOrValue<U128> {
        if is_promise_success() {
            log!("Exchange successful");
            PromiseOrValue::Value(amount)
        } else {
            log!("Exchange failed");

            if token_in == Token::Near {
                self.user_points
                    .entry(sender_id)
                    .and_modify(|points| points.0 -= amount.0)
                    .or_insert(0.into());
            } else {
                self.user_points
                    .entry(sender_id.clone())
                    .and_modify(|points| points.0 += amount.0)
                    .or_insert(amount.0.into());
            }

            PromiseOrValue::Value(amount)
        }
    }
}
