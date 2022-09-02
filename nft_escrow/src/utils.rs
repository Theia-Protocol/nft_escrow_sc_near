use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{ext_contract, AccountId, Gas, Balance, PromiseOrValue, PromiseResult, env};
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
        project_token_id: AccountId,
        proxy_token_id: AccountId
    ) -> PromiseOrValue<bool>;
    fn on_buy(&mut self, from: AccountId, remain: U128) -> bool;
    fn on_sell(&mut self);
    fn on_convert(&mut self, amount: Balance);
    fn on_claim_fund(&mut self, amount: U128);
    fn on_close_project(&mut self);
}

#[ext_contract(ext_nft_collection)]
pub trait NonFungibleToken {
    fn new(&mut self, name: String, symbol: String, blank_uri: String, max_supply: U128);
    fn nft_mint(&mut self, receiver_id: AccountId, amount: U128);
    fn nft_transfer(&mut self, receiver_id: AccountId, token_id: TokenId, approval_id: Option<u64>, memo: Option<String>);
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

#[ext_contract(ext_proxy_token)]
pub trait ProxyToken {
    fn new(&mut self, name: String, symbol: String, blank_uri: String, max_supply: U128);
    fn mt_mint(&mut self, receiver_id: AccountId, amount: U128);
    fn mt_burn(&mut self, from_id: AccountId, token_ids: Vec<TokenId>);
    fn mt_burn_with_amount(&mut self, from_id: AccountId, start_id: TokenId, amount: U128);
    fn mt_all_total_supply(&self);
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

pub fn is_promise_ok(result: PromiseResult) -> bool {
    match result {
        PromiseResult::NotReady => unreachable!(),
        PromiseResult::Successful(_) => true,
        PromiseResult::Failed => env::panic_str("ERR_CALL_FAILED"),
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