use near_sdk::{env, near, require, AccountId};

use crate::{Contract, ContractExt};

#[near]
impl Contract {
    pub fn add_manager(&mut self, manager: AccountId) {
        self.assert_manager();
        self.managers.insert(manager);
    }

    pub fn remove_manager(&mut self, manager: AccountId) {
        todo!()
    }

    pub(crate) fn assert_manager(&self) {
        require!(
            self.managers.contains(&env::predecessor_account_id()),
            "ERR_NOT_MANAGER"
        );
    }
}
