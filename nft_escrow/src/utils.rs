use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::{ext_contract, AccountId, Gas};
use near_sdk::json_types::{U128};

/// Fee divisor, allowing to provide fee in bps.
pub const FEE_DIVISOR: u32 = 10_000;
/// Amount of gas for fungible token transfers.
pub const GAS_FOR_FT_TRANSFER: Gas = Gas::ONE_TERA;

#[ext_contract(ext_self)]
pub trait SelfCallbacks {
    fn close_project_callback_after_get_owner(&mut self);
}

#[ext_contract(ext_nft_collection)]
pub trait NonFungibleToken {
    fn new(&mut self, name: String, symbol: String, blank_uri: String, max_supply: u128);
    fn nft_mint(&mut self, receiver_id: AccountId, amount: u128);
    fn nft_transfer(&mut self, receiver_id: AccountId, token_id: TokenId, approval_id: Option<u64>, memo: Option<String>);
    fn get_owner(&self) -> AccountId;
    fn set_owner(&mut self, owner_id: AccountId);
}

#[ext_contract(ext_fungible_token)]
pub trait FungibleToken {
    fn new(&mut self, name: String, symbol: String);
    fn ft_mint(&mut self, receiver_id: AccountId, amount: u128);
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn get_owner(&self) -> AccountId;
    fn set_owner(&mut self, owner_id: AccountId);
}

#[ext_contract(ext_proxy_token)]
pub trait ProxyToken {
    fn new(&mut self, name: String, symbol: String, blank_uri: String, max_supply: u128);
    fn mt_mint(&mut self, receiver_id: AccountId, amount: u128);
    fn mt_burn(&mut self, from_id: AccountId, token_ids: Vec<TokenId>);
    fn mt_all_total_supply(&self);
}

/// Newton's method of integer square root.
pub fn integer_sqrt(value: u128) -> u128 {
    let mut guess: u128 = (value + 1) >> 1;
    let mut res = value;
    while guess < res {
        res = guess;
        guess = (value / guess + guess) >> 1;
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt() {
        assert_eq!(integer_sqrt(0u128), 0.into());
        assert_eq!(integer_sqrt(4u128), 2.into());
        assert_eq!(
            integer_sqrt(1_516_156_330_329u128),
            1_231_323u128
        );
    }
}