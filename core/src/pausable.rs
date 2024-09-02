use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{
    env, json_types::U128, near, require, AccountId, NearToken, PromiseOrValue,
};

use crate::{Contract, ContractExt, PostedNote, MIN_NOTE_DEPOSIT};

pub trait Pausable {
    fn toggle_pause(&mut self);
    fn is_paused(&self) -> bool;
    fn assert_not_paused(&self);
}

#[near]
impl Pausable for Contract {
    fn toggle_pause(&mut self) {
        todo!("Implement toggle_paus")
    }

    fn is_paused(&self) -> bool {
        todo!("Implement is_paused")
    }

    fn assert_not_paused(&self) {
        todo!("Implement assert_not_paused")
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::{test_utils::VMContextBuilder, testing_env, NearToken};

    use super::*;

    #[test]
    fn paused() {
        todo!("Implement paused test")
    }

    // more tests here

    fn set_context(predecessor: &str, amount: NearToken) {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor.parse().unwrap());
        builder.attached_deposit(amount);

        testing_env!(builder.build());
    }
}
