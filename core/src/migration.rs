use near_sdk::{
    env,
    json_types::{U128, U64},
    near,
    store::{IterableMap, IterableSet, LookupMap, LookupSet},
    AccountId, Gas, NearToken, Promise,
};

use crate::{Contract, ContractExt, PostedNote};

const CALL_GAS: Gas = Gas::from_tgas(200);

//UNCOMMENT FOR MIGRATION HOMEWORK
#[near(serializers = [borsh, json])]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OldPostedNote {
    pub id: Option<U64>,
    pub title: String,
    pub body: String,
}

#[near(serializers = [borsh])]
struct OldState {
    greeting: String,
    note_book: IterableMap<AccountId, IterableSet<PostedNote>>,
    // note_book_collections:
    //     LookUpMapCollections<AccountId, UnorderedSet<PostedNote>>,
    ft_id: AccountId,
    w_near_id: AccountId,
    next_entry_id: Option<u64>,
    managers: LookupSet<AccountId>,
    user_points: LookupMap<AccountId, U128>,
    // version: U64, // MIGRATION HOMEWORK: UNCOMMENT
}

#[near]
impl Contract {
    pub fn update_contract(&self) -> Promise {
        // Check the caller is authorized to update the code
        self.assert_manager();

        // Receive the code directly from the input to avoid the
        // GAS overhead of deserializing parameters
        let code = env::input().expect("Error: No input").to_vec();

        // Deploy the contract on self
        Promise::new(env::current_account_id())
            .deploy_contract(code)
            .function_call(
                "migrate".to_string(),
                vec![],
                NearToken::from_near(0),
                CALL_GAS,
            )
    }

    // MIGRATION HOMEWORK: GO OVER OLD POSTED NOTES, INCLUDE AUTHOR AND ADD TO A
    // NEW STATE_VAR, AND MIGRATE
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let old_state: OldState = env::state_read().expect("failed");

        Self {
            greeting: old_state.greeting,
            note_book: old_state.note_book,
            // note_book_collections: old_state.note_book_collections,
            ft_id: old_state.ft_id,
            w_near_id: old_state.w_near_id,
            next_entry_id: old_state.next_entry_id,
            managers: old_state.managers,
            user_points: old_state.user_points,
            version: U64(2),
        }
    }
}
