use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{ext_contract, AccountId, Gas, Balance, PromiseOrValue, env, require, Promise};
use near_sdk::json_types::{U128};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use uint::construct_uint;


/// Fee divisor, allowing to provide fee in bps.
pub const FEE_DIVISOR: u32 = 10_000;
/// Amount of gas for fungible token transfers.
pub const GAS_FOR_FT_TRANSFER: Gas = Gas::ONE_TERA;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum RunningState {
    Running,
    Paused,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ProjectTokenType {
    NonFungible,
    Fungible,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum CurveType {
    Horizontal,
    Linear,
    Sigmoidal,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Clone, Debug)]
pub enum ClosedStep {
    None = 0,
    PreMint = 1,
    RemainProxy = 2,
    TransferOwnership = 3
}

impl ClosedStep {
    pub fn increase(&self) -> ClosedStep {
        return match self {
            ClosedStep::None => {
                ClosedStep::PreMint
            }
            ClosedStep::PreMint => {
                ClosedStep::RemainProxy
            }
            _ => {
                ClosedStep::TransferOwnership
            }
        }
    }

    pub fn decrease(&self) -> ClosedStep {
        return match self {
            ClosedStep::TransferOwnership => {
                ClosedStep::RemainProxy
            }
            ClosedStep::RemainProxy => {
                ClosedStep::PreMint
            }
            _ => {
                ClosedStep::None
            }
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CurveArgs {
    pub arg_a: Option<u128>,
    pub arg_b: Option<u128>,
    pub arg_c: Option<u128>,
    pub arg_d: Option<u128>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenArgs {
    pub owner_id: AccountId,
    pub name: String,
    pub symbol: String
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NonFungibleTokenArgs {
    pub owner_id: AccountId,
    pub name: String,
    pub symbol: String,
    pub base_uri: String,
    pub max_supply: U128
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ProxyTokenArgs {
    pub owner_id: AccountId,
    pub name: String,
    pub symbol: String,
    pub blank_media_uri: String,
    pub max_supply: U128
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ProxyTokenMintArgs {
    pub receiver_id: AccountId,
    pub amount: U128
}
#[ext_contract(ext_self)]
pub trait SelfCallbacks {
    fn on_activate(
        &mut self,
        project_token_id: AccountId
    ) -> PromiseOrValue<bool>;
    fn on_buy(&mut self, from: AccountId, amount: U128, deposit: U128, reserve: U128) -> bool;
    fn on_sell(&mut self, from: AccountId, refund: U128, token_ids: Vec<TokenId>) -> bool;
    fn on_convert(&mut self, from: AccountId, token_ids: Vec<TokenId>) -> bool;
    fn on_claim_fund(&mut self, amount: U128);
    fn on_claim_finder_fee(&mut self, amount: U128);
    fn on_close_project(&mut self);
    fn pt_mint(&mut self, receiver_id: AccountId, amount: U128);
}

#[ext_contract(ext_nft_collection)]
pub trait NonFungibleToken {
    fn new(&mut self, name: String, symbol: String, blank_uri: String, max_supply: U128);
    fn nft_mint(&mut self, receiver_id: AccountId, amount: U128);
    fn nft_transfer(&mut self, receiver_id: AccountId, token_id: TokenId, approval_id: Option<u64>, memo: Option<String>);
    fn nft_batch_transfer(&mut self, to: AccountId, token_ids: Vec<TokenId>, memo: Option<String>);
    fn get_owner(&self) -> AccountId;
    fn set_owner(&mut self, owner_id: AccountId);
}

#[ext_contract(ext_fungible_token)]
pub trait FungibleToken {
    fn new(&mut self, name: String, symbol: String);
    fn ft_mint(&mut self, receiver_id: AccountId, amount: U128);
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn get_owner(&self) -> AccountId;
    fn set_owner(&mut self, owner_id: AccountId);
}

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

/// Newton's method of integer square root.
pub fn integer_sqrt(value: U256) -> U256 {
    let mut guess: U256 = (value + 1) >> 1;
    let mut res = value;
    while guess < res {
        res = guess;
        guess = (value / guess + guess) >> 1;
    }
    res
}

pub fn refund_deposit_to_account(storage_used: u64, account_id: AccountId) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit();

    require!(
        required_cost <= attached_deposit,
        format!("Must attach {} yoctoNEAR to cover storage, attached {}", required_cost, attached_deposit)
    );

    let refund = attached_deposit - required_cost;
    if refund > 1 {
        Promise::new(account_id).transfer(refund);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt() {
        assert_eq!(integer_sqrt(U256::from(0u128)), 0u128.into());
        assert_eq!(integer_sqrt(U256::from(4u128)), 2u128.into());
        assert_eq!(
            integer_sqrt(U256::from(1_516_156_330_329u128)),
            1_231_323u128
        );
    }
}