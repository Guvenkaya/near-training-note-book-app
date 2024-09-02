mod exchange;
mod ft_receiver;
mod migration;
mod ownership;

use std::u32;

use near_sdk::{
    collections::{LookupMap as LookUpMapCollections, UnorderedSet},
    env,
    json_types::{U128, U64},
    log, near, require,
    store::{IterableMap, IterableSet, LookupMap, LookupSet},
    AccountId, BorshStorageKey, NearToken, PanicOnDefault, Promise,
};

const MIN_NOTE_DEPOSIT: u128 = NearToken::from_near(1).as_yoctonear();
#[near(serializers = [borsh, json])]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PostedNote {
    pub id: Option<U64>,
    pub title: String,
    pub body: String,
    //pub author: AccountId, // MIGRATION HOMEWORK: UNCOMMENT
}

impl PostedNote {
    pub fn new(
        title: String,
        body: String,
        id: Option<U64>,
        // author: AccountId,
    ) -> Self {
        Self {
            title,
            body,
            id,
            // author,
        }
    }
}

#[near]
#[derive(BorshStorageKey)]
pub enum StorageKey {
    NotesPerUser,
    Notes(AccountId),
    Managers,
    UserPoints,
}

// Define the contract structure
#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    greeting: String,
    note_book: IterableMap<AccountId, IterableSet<PostedNote>>,
    // note_book_collections:
    //     LookUpMapCollections<AccountId, UnorderedSet<PostedNote>>,
    pub ft_id: AccountId,
    pub w_near_id: AccountId,
    next_entry_id: Option<u64>,
    managers: LookupSet<AccountId>,
    user_points: LookupMap<AccountId, U128>,
    version: U64,
}

// Implement the contract structure
#[near]
impl Contract {
    #[init]
    pub fn new(
        ft_id: AccountId,
        w_near_id: AccountId,
        managers: Vec<AccountId>,
    ) -> Self {
        let mut managers_set = LookupSet::new(StorageKey::Managers);

        managers.into_iter().for_each(|manager| {
            managers_set.insert(manager);
        });

        Self {
            greeting: "Hello".to_string(),
            note_book: IterableMap::new(StorageKey::NotesPerUser),
            //note_book_collections: LookUpMapCollections::new(b"mm".to_vec()),
            ft_id,
            w_near_id,
            managers: managers_set,
            next_entry_id: None,
            user_points: LookupMap::new(StorageKey::UserPoints),
            version: U64(1),
        }
    }

    // Public method - returns the greeting saved, defaulting to
    pub fn get_greeting(&self) -> String {
        self.greeting.clone()
    }

    // Public method - accepts a greeting, such as "howdy", and records it
    pub fn set_greeting(&mut self, greeting: String) {
        log!("Saving greeting: {greeting}");
        self.greeting = greeting;
    }

    pub fn get_version(&self) -> U64 {
        self.version.clone()
    }

    // MIGRATION HOMEWORK: Modify TO ALSO PASS AUTHOR TO NOTE author for
    // Migration homework
    #[payable]
    pub fn add_note(&mut self, title: String, body: String) {
        let account_id = env::predecessor_account_id();

        let next_entry_id = self.next_entry_id.unwrap_or(0);

        let note = PostedNote::new(
            title.clone(),
            body,
            Some(next_entry_id.into()),
            // account_id.clone(),
        );

        self.internal_add_note(
            account_id.clone(),
            &note,
            Some(env::attached_deposit().as_yoctonear()),
            next_entry_id,
        );
    }

    // pub fn add_note_collection(&mut self, title: String, body: String) {
    //     let account_id = env::predecessor_account_id();

    //     let next_entry_id = self.next_entry_id.unwrap_or(0);

    //     let note = PostedNote::new(title.clone(), body, next_entry_id);

    //     self.internal_add_note_collection(&account_id, &note);

    //     self.next_entry_id = Some(next_entry_id + 1);

    //     log!("Added note to the note book: {}", note.title);
    // }

    pub fn get_note(&self, account_id: AccountId, id: U64) -> &PostedNote {
        let id = id.0;

        require!(id <= self.next_entry_id.unwrap_or(0), "Note does not exist");

        let notes = self
            .note_book
            .get(&account_id)
            .unwrap_or_else(|| env::panic_str("no entry"));

        notes
            .iter()
            .find(|note| note.id.unwrap() == id.into())
            .unwrap_or_else(|| env::panic_str("no entry"))
    }

    pub fn get_notes(
        &self,
        account_id: AccountId,
        from_index: Option<u32>,
        limit: Option<u32>,
    ) -> Vec<&PostedNote> {
        let notes = self
            .note_book
            .get(&account_id)
            .unwrap_or_else(|| env::panic_str("no entry"));

        notes
            .iter()
            .skip(from_index.unwrap_or(0) as usize)
            .take(limit.unwrap_or(u32::MAX) as usize)
            .collect()
    }

    pub fn get_user_points(&self, account_id: AccountId) -> &U128 {
        self.user_points.get(&account_id).expect("no entry")
    }

    fn internal_add_note(
        &mut self,
        account_id: AccountId,
        note: &PostedNote,
        deposit: Option<u128>,
        next_entry_id: u64,
    ) {
        let storage_usage = env::storage_usage();

        if let Some(notes) = self.note_book.get_mut(&account_id) {
            notes.insert(note.clone());
        } else {
            self.note_book.insert(
                account_id.clone(),
                IterableSet::new(StorageKey::Notes(account_id.clone())),
            );

            let notes = self.note_book.get_mut(&account_id).unwrap();
            notes.insert(note.clone());
        }

        self.next_entry_id = Some(next_entry_id + 1);

        let storage_cost = env::storage_byte_cost().as_yoctonear()
            * (env::storage_usage() - storage_usage) as u128;

        let to_refund = deposit
            .unwrap_or(0)
            .checked_sub(storage_cost)
            .expect("not enough attached deposit");

        if to_refund != 0 {
            Promise::new(account_id)
                .transfer(NearToken::from_yoctonear(to_refund));
        }

        log!("Added note to the note book: {}", note.title);
    }

    // fn internal_add_note_collection(
    //     &mut self,
    //     account_id: &AccountId,
    //     note: &PostedNote,
    // ) {
    //     if let Some(mut notes) = self.note_book_collections.get(&account_id)
    // {         notes.insert(note);
    //     } else {
    //         self.note_book_collections.insert(
    //             account_id,
    //             &UnorderedSet::new(StorageKey::Notes(account_id.clone())),
    //         );

    //         let mut notes =
    //             self.note_book_collections.get(&account_id).unwrap();
    //         notes.insert(note);
    //     }
    // }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use near_sdk::{test_utils::VMContextBuilder, testing_env, NearToken};

    use super::*;

    #[test]
    fn get_default_greeting() {
        let contract = Contract::new(
            "some_acc.near".parse().unwrap(),
            "some_acc.near".parse().unwrap(),
            vec!["some_acc.near".parse().unwrap()],
        );
        // this test did not call set_greeting so should return the default
        // "Hello" greeting
        assert_eq!(contract.get_greeting(), "Hello");
    }

    #[test]
    fn set_then_get_greeting() {
        let mut contract = Contract::new(
            "some_acc.near".parse().unwrap(),
            "some_acc.near".parse().unwrap(),
            vec!["some_acc.near".parse().unwrap()],
        );
        contract.set_greeting("howdy".to_string());
        assert_eq!(contract.get_greeting(), "howdy");
    }

    #[test]
    fn add_note() {
        let mut contract = Contract::new(
            "some_acc.near".parse().unwrap(),
            "some_acc.near".parse().unwrap(),
            vec!["some_acc.near".parse().unwrap()],
        );

        let account_id = "account_id1";
        set_context(account_id, NearToken::from_near(1));

        let posted_note = PostedNote::new(
            "title".into(),
            "body".into(),
            Some(contract.next_entry_id.unwrap_or(0).into()),
            // account_id.parse().unwrap(),
        );

        contract.add_note(posted_note.title.clone(), posted_note.body.clone());

        let notes = contract
            .note_book
            .get(&account_id.to_string().parse::<AccountId>().unwrap())
            .unwrap();

        assert_eq!(notes.len(), 1);
        assert!(notes.contains(&posted_note));

        // add another note for the same account
        let posted_note_2 = PostedNote::new(
            "title2".into(),
            "body2".into(),
            Some(contract.next_entry_id.unwrap().into()),
            // account_id.parse().unwrap(),
        );

        contract
            .add_note(posted_note_2.title.clone(), posted_note_2.body.clone());

        // add another note for a different account
        let account_id_2 = "account_id2";
        set_context(account_id_2, NearToken::from_near(1));

        let posted_note_3 = PostedNote::new(
            "title3".into(),
            "body3".into(),
            Some(contract.next_entry_id.unwrap().into()),
            // account_id.parse().unwrap(),
        );

        contract
            .add_note(posted_note_3.title.clone(), posted_note_3.body.clone());

        let notes = contract
            .note_book
            .get(&account_id.to_string().parse::<AccountId>().unwrap())
            .unwrap();

        let notes_2 = contract
            .note_book
            .get(&account_id_2.to_string().parse::<AccountId>().unwrap())
            .unwrap();

        assert_eq!(notes.len(), 2);
        assert!(notes.contains(&posted_note));
        assert!(notes.contains(&posted_note_2));

        assert!(notes_2.contains(&posted_note_3));
    }

    // #[test]
    // fn add_note_collection() {
    //     let mut contract = Contract::init(
    //         "some_acc.near".parse().unwrap(),
    //         "some_acc.near".parse().unwrap(),
    //         "some_acc.near".parse().unwrap(),
    //     );

    //     let account_id = "account_id1";
    //     set_context(account_id, NearToken::from_near(0));

    //     let posted_note = PostedNote::new(
    //         "title".into(),
    //         "body".into(),
    //         contract.next_entry_id.unwrap_or(0),
    //     );

    //     contract.add_note_collection(
    //         posted_note.title.clone(),
    //         posted_note.body.clone(),
    //     );

    //     let notes = contract
    //         .note_book_collections
    //         .get(&account_id.to_string().parse::<AccountId>().unwrap())
    //         .unwrap();

    //     println!("{}", notes.is_empty());
    //     println!("{}", notes.contains(&posted_note));

    //     // Len is zero but contains the posted note
    //     assert_eq!(notes.len(), 0);
    //     assert!(notes.contains(&posted_note));
    // }

    fn set_context(predecessor: &str, amount: NearToken) {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor.parse().unwrap());
        builder.attached_deposit(amount);

        testing_env!(builder.build());
    }
}
