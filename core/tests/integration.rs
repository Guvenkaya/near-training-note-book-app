use near_sdk::{
    json_types::{U128, U64},
    near,
    serde_json::json,
    AccountId,
};
use near_workspaces::{
    result::ValueOrReceiptId, types::NearToken, Account, Contract,
};

#[near(serializers = [borsh, json])]
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PostedNote {
    pub id: Option<U64>,
    pub title: String,
    pub body: String,
}

#[near(serializers = [borsh, json])]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PostedNoteNew {
    pub id: Option<U64>,
    pub title: String,
    pub body: String,
    pub author: AccountId,
}

#[near(serializers = [json])]
pub enum FtMessage {
    AddNote(PostedNote),
    RemoveNote(U64),
    Exchange,
}

pub struct Env {
    note_book_contract: Contract,
    note_book_contract_old: Contract,
    ft_contract: Contract,
    w_near: Contract,
    manager: Account,
    user: Account,
}

const NOTE_BOOK_CONTRACT: &[u8] =
    include_bytes!("../../res/core_contract.wasm");

const NOTE_BOOK_CONTRACT_OLD: &[u8] =
    include_bytes!("../../res/core_contract_old.wasm");

const NOTE_BOOK_CONTRACT_VERSIONED: &[u8] =
    include_bytes!("../../res/core_contract_old_version.wasm");

const FT_CONTRACT: &[u8] = include_bytes!("../../res/fungible_token.wasm");

async fn prepare() -> color_eyre::Result<Env> {
    let sandbox = near_workspaces::sandbox().await?;
    let mainnet = near_workspaces::mainnet_archival().await?;

    let note_book_contract = sandbox.dev_deploy(NOTE_BOOK_CONTRACT).await?;
    let note_book_contract_old =
        sandbox.dev_deploy(NOTE_BOOK_CONTRACT_OLD).await?;
    let ft_contract = sandbox.dev_deploy(FT_CONTRACT).await?;
    let manager = sandbox.dev_create_account().await?;
    let user = sandbox.dev_create_account().await?;

    let w_near = sandbox
        .import_contract(&"wrap.near".parse().unwrap(), &mainnet)
        .transact()
        .await?;

    manager
        .call(w_near.id(), "new")
        .transact()
        .await?
        .into_result()?;

    println!("w_near contract deployed: {}\n", w_near.id());

    manager
        .call(w_near.id(), "near_deposit")
        .deposit(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?;

    user.call(w_near.id(), "near_deposit")
        .deposit(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?;

    note_book_contract
        .as_account()
        .call(w_near.id(), "near_deposit")
        .deposit(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?;

    ft_contract
        .call("new_default_meta")
        .args_json(json!({
            "owner_id": manager.id(),
            "total_supply": U128(NearToken::from_near(1_000_000_000).as_yoctonear()),
        }))
        .transact()
        .await?
        .into_result()?;

    user.call(ft_contract.id(), "storage_deposit")
        .args_json(json!({}))
        .deposit(NearToken::from_millinear(30))
        .transact()
        .await?
        .into_result()?;

    note_book_contract
        .as_account()
        .call(ft_contract.id(), "storage_deposit")
        .args_json(json!({}))
        .deposit(NearToken::from_millinear(30))
        .transact()
        .await?
        .into_result()?;

    println!("fungible token deployed: {}\n", ft_contract.id());

    note_book_contract.call("new")
        .args_json(serde_json::json!({"managers": vec![manager.id()], "ft_id": ft_contract.id(), "w_near_id": w_near.id()}))
        .transact().await?.into_result()?;

    println!("note book contract deployed: {}\n", note_book_contract.id());

    note_book_contract_old.call("new")
        .args_json(serde_json::json!({"managers": vec![manager.id()], "ft_id": ft_contract.id(), "w_near_id": w_near.id()}))
        .transact().await?.into_result()?;

    println!(
        "old note book contract deployed: {}\n",
        note_book_contract.id()
    );

    Ok(Env {
        note_book_contract,
        note_book_contract_old,
        ft_contract,
        w_near,
        manager,
        user,
    })
}

#[tokio::test]
async fn contract_is_operational() -> color_eyre::Result<()> {
    let Env {
        note_book_contract,
        user,
        ..
    } = prepare().await?;

    let outcome = user
        .call(note_book_contract.id(), "set_greeting")
        .args_json(json!({"greeting": "Hello World!"}))
        .transact()
        .await?;

    println!("OUTCOME: {outcome:#?}");
    assert!(outcome.is_success());

    let user_message_outcome = note_book_contract
        .view("get_greeting")
        .args_json(json!({}))
        .await?;
    assert_eq!(user_message_outcome.json::<String>()?, "Hello World!");

    Ok(())
}

#[tokio::test]
async fn migration_works() -> color_eyre::Result<()> {
    let Env {
        note_book_contract_old,
        manager,
        ..
    } = prepare().await?;

    manager
        .call(note_book_contract_old.id(), "update_contract")
        .args(NOTE_BOOK_CONTRACT.to_vec())
        .max_gas()
        .transact()
        .await?
        .into_result()?;

    let version = note_book_contract_old
        .view("get_version")
        .args_json(json!({}))
        .await?
        .json::<U64>()?;

    assert_eq!(version.0, 2);

    Ok(())
}

#[tokio::test]
async fn migration_works_author_field() -> color_eyre::Result<()> {
    todo!("HOMEWORK");

    Ok(())
}

#[tokio::test]
async fn add_note() -> color_eyre::Result<()> {
    let Env {
        note_book_contract,
        user,
        ..
    } = prepare().await?;

    let res = user
        .call(note_book_contract.id(), "add_note")
        .deposit(NearToken::from_near(1))
        .args_json(json!(
        {
            "title": "Hello",
            "body": "World"
        }))
        .transact()
        .await?
        .into_result()?;

    assert!(res
        .logs()
        .iter()
        .find(|log| log.contains("Added note to the note book"))
        .is_some());

    let notes = note_book_contract
        .view("get_notes")
        .args_json(json!({"account_id": user.id()}))
        .await?
        .json::<Vec<PostedNote>>()?;

    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "Hello");
    assert_eq!(notes[0].body, "World");
    Ok(())
}

#[tokio::test]
async fn add_note_ft() -> color_eyre::Result<()> {
    let Env {
        note_book_contract,
        ft_contract,
        user,
        manager,
        ..
    } = prepare().await?;

    manager
        .call(ft_contract.id(), "ft_transfer")
        .args_json(json!({
            "receiver_id": user.id(),
            "amount": U128(NearToken::from_near(2).as_yoctonear()),
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?
        .into_result()?;

    let res = user
        .call(ft_contract.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": note_book_contract.id(),
            "amount": U128(NearToken::from_near(1).as_yoctonear()),
            "msg": serde_json::to_string(&FtMessage::AddNote(PostedNote {
                id: None,
                title: "Hello".to_string(),
                body: "World".to_string(),
            }))?,
        }))
        .max_gas()
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?
        .into_result()?;

    assert!(res
        .logs()
        .iter()
        .find(|log| log.contains("Added note to the note book"))
        .is_some());

    let notes = note_book_contract
        .view("get_notes")
        .args_json(json!({"account_id": user.id()}))
        .await?
        .json::<Vec<PostedNote>>()?;

    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "Hello");
    assert_eq!(notes[0].body, "World");

    let points = note_book_contract
        .view("get_user_points")
        .args_json(json!({"account_id": user.id()}))
        .await?
        .json::<U128>()?;

    assert_eq!(points.0, NearToken::from_near(1).as_yoctonear());

    Ok(())
}

async fn remove_note() -> color_eyre::Result<()> {
    todo!("HOMEWORK");
}

async fn remove_note_ft() -> color_eyre::Result<()> {
    todo!("HOMEWORK");
}

#[tokio::test]
async fn add_manager() -> color_eyre::Result<()> {
    todo!("HOMEWORK");
}

#[tokio::test]
async fn remove_manager() -> color_eyre::Result<()> {
    todo!("HOMEWORK");
}

#[tokio::test]
async fn exchange_ft_wnear() -> color_eyre::Result<()> {
    let Env {
        note_book_contract,
        ft_contract,
        w_near,
        user,
        manager,
        ..
    } = prepare().await?;
    manager
        .call(ft_contract.id(), "ft_transfer")
        .args_json(json!({
            "receiver_id": user.id(),
            "amount": U128(NearToken::from_near(10).as_yoctonear()),
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?
        .into_result()?;

    let w_near_balance_before = w_near
        .view("ft_balance_of")
        .args_json(json!({"account_id": user.id()}))
        .await?
        .json::<U128>()?;

    let res = user
        .call(ft_contract.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": note_book_contract.id(),
            "amount": U128(NearToken::from_near(8).as_yoctonear()),
            "msg": serde_json::to_string(&FtMessage::Exchange)?,
        }))
        .max_gas()
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?
        .into_result()?;

    assert!(res
        .logs()
        .iter()
        .find(|log| log.contains("Exchange successful"))
        .is_some());

    let points = note_book_contract
        .view("get_user_points")
        .args_json(json!({"account_id": user.id()}))
        .await?
        .json::<U128>()?;

    assert_eq!(points.0, 0);

    let w_near_balance_after = w_near
        .view("ft_balance_of")
        .args_json(json!({"account_id": user.id()}))
        .await?
        .json::<U128>()?;

    let amount = res
        .receipt_outcomes()
        .into_iter()
        .find_map(|receipt| {
            if receipt
                .logs
                .iter()
                .any(|log| log.contains("Exchange successful"))
            {
                let val = receipt.clone().into_result().unwrap();

                if let ValueOrReceiptId::Value(amount) = val {
                    return Some(amount);
                } else {
                    return None;
                }
            } else {
                None
            }
        })
        .unwrap()
        .json::<U128>()?;

    assert_eq!(w_near_balance_after.0 - w_near_balance_before.0, amount.0);

    Ok(())
}

#[tokio::test]
async fn exchange_near_ft() -> color_eyre::Result<()> {
    todo!("HOMEWORK");
}

// #[tokio::test]
// async fn test_contract_is_operational() -> Result<(), Box<dyn
// std::error::Error>> {
//     let sandbox = near_workspaces::sandbox().await?;
//     let contract_wasm = near_workspaces::compile_project("../").await?;
//     let contract_ft =
// near_workspaces::compile_project("../../FT/ft/").await?;

//     let contract = sandbox.dev_deploy(&contract_wasm).await?;

//     let user_account = sandbox.dev_create_account().await?;

//     let outcome = user_account
//         .call(contract.id(), "set_greeting")
//         .args_json(json!({"greeting": "Hello World!"}))
//         .transact()
//         .await?;
//     assert!(outcome.is_success());

//     let user_message_outcome =
//         contract.view("get_greeting").args_json(json!({})).await?;
//     assert_eq!(user_message_outcome.json::<String>()?, "Hello World!");

//     Ok(())
// }
