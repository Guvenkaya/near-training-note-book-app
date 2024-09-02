use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{
    env, json_types::U128, near, require, AccountId, NearToken, PromiseOrValue,
};

use crate::{Contract, ContractExt, PostedNote, MIN_NOTE_DEPOSIT};

#[near(serializers = [json])]
pub enum FtMessage {
    AddNote(PostedNote),
    RemoveNote(u64),
    Exchange,
}

#[near(serializers = [json])]
#[derive(Clone, PartialEq, Eq)]
pub enum Token {
    Near,
    FungibleToken,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Token::Near => write!(f, "WNEAR"),
            Token::FungibleToken => write!(f, "FungibleToken"),
        }
    }
}

impl From<Token> for AccountId {
    fn from(token: Token) -> Self {
        match token {
            Token::Near => {
                let contract: Contract = env::state_read().unwrap_or_default();
                contract.w_near_id
            }
            Token::FungibleToken => {
                let contract: Contract = env::state_read().unwrap_or_default();
                contract.ft_id
            }
        }
    }
}

impl FromStr for Token {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let contract: Contract = env::state_read().unwrap_or_default();

        if s == contract.w_near_id {
            Ok(Token::Near)
        } else if s == contract.ft_id {
            Ok(Token::FungibleToken)
        } else {
            Err("Token not supported".to_string())
        }
    }
}

#[near]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let ft_message = near_sdk::serde_json::from_str::<FtMessage>(&msg)
            .expect("Unable to deserialize msg");

        let token = env::predecessor_account_id()
            .as_str()
            .parse::<Token>()
            .expect("Token not supported");

        match ft_message {
            FtMessage::AddNote(note) => {
                require!(
                    amount.0 >= MIN_NOTE_DEPOSIT,
                    "Minimum deposit is 1 FT"
                );

                let next_entry_id = self.next_entry_id.unwrap_or(0);

                let note = PostedNote {
                    id: Some(note.id.unwrap_or(next_entry_id.into()).into()),
                    ..note
                };

                self.internal_add_note(
                    sender_id.clone(),
                    &note,
                    None,
                    next_entry_id,
                );

                self.user_points
                    .entry(sender_id)
                    .and_modify(|points| points.0 += amount.0)
                    .or_insert(amount.0.into());
            }

            FtMessage::RemoveNote(note) => {
                todo!("Remove note")
            }

            FtMessage::Exchange => {
                let (token_in, token_out, amount_out) = if token == Token::Near
                {
                    require!(
                        amount.0 >= NearToken::from_near(1).as_yoctonear(),
                        "ERR_MIN_AMOUNT"
                    );
                    let amount_out = amount.0 * 4;

                    (Token::Near, Token::FungibleToken, amount_out)
                } else {
                    require!(
                        amount.0 >= NearToken::from_near(4).as_yoctonear(),
                        "ERR_MIN_AMOUNT"
                    );

                    let amount_out = amount.0 / 4;

                    (Token::FungibleToken, Token::Near, amount_out)
                };

                self.internal_exchange(
                    sender_id.clone(),
                    token_in.clone(),
                    token_out,
                    amount_out,
                )
                .then(
                    Self::ext(env::current_account_id()).resolve_exchange(
                        sender_id,
                        token_in,
                        amount_out.into(),
                    ),
                );
            }
        }

        PromiseOrValue::Value(0.into())
    }
}
