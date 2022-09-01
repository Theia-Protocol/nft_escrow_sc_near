mod owner;

use near_contract_standards::fungible_token::events::{FtBurn, FtMint};
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{
    env, log, near_bindgen, require, AccountId, Balance, BorshStorageKey, PanicOnDefault,
    PromiseOrValue,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    owner_id: AccountId,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    FungibleToken,
    Metadata,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(owner_id: AccountId, name: String, symbol: String) -> Self {
        require!(!env::state_exists(), "Already initialized");

        let metadata = FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name,
            symbol,
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: 24,
        };
        metadata.assert_valid();

        Self {
            token: FungibleToken::new(StorageKey::FungibleToken),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            owner_id: owner_id.clone(),
        }
    }

    pub fn ft_mint(&mut self, receiver_id: AccountId, amount: U128) {
        require!(env::predecessor_account_id() == self.owner_id, "UnAuthorized");

        self.token.internal_deposit(&receiver_id, amount.0);
        FtMint {
            owner_id: &(receiver_id),
            amount: &amount,
            memo: None
        }.emit();
    }

    pub fn ft_burn(&mut self, from_id: AccountId, amount: U128) {
        require!(env::predecessor_account_id() == self.owner_id, "UnAuthorized");

        self.token.internal_withdraw(&from_id, amount.0);
        FtBurn {
            owner_id: &(from_id),
            amount: &amount,
            memo: None
        }.emit();
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[allow(dead_code, unused)]
#[cfg(test)]
mod tests {
    use near_sdk::{test_utils::*, testing_env, AccountId, ONE_NEAR};
    use super::*;

    fn contract_account() -> AccountId {
        "contract".parse::<AccountId>().unwrap()
    }

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(contract_account())
            .account_balance(15 * ONE_NEAR)
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test() {
        let owner_id = accounts(0);
        let alice_id = accounts(1);

        // deploy
        testing_env!(get_context(owner_id.clone()).build());
        let mut contract = Contract::new(owner_id.clone(), String::from("Test FT"), String::from("TFT"));

        // mint
        testing_env!(
            get_context(owner_id.clone())
                .attached_deposit(125 * env::storage_byte_cost())
                .build()
        );
        contract.storage_deposit(Some(accounts(0)), None);
        contract.ft_mint(accounts(0), 1_000_000u128.into());
        assert_eq!(contract.ft_balance_of(accounts(0)), 1_000_000u128.into());

        // transfer
        testing_env!(
            get_context(owner_id.clone())
                .attached_deposit(125 * env::storage_byte_cost())
                .build()
        );
        contract.storage_deposit(Some(alice_id.clone()), None);
        testing_env!(
             get_context(owner_id.clone())
                .attached_deposit(1)
                .build());
        contract.ft_transfer(alice_id.clone(), 1_000u128.into(), None);
        assert_eq!(contract.ft_balance_of(accounts(1)), 1_000u128.into());

        // burn
        contract.ft_burn(alice_id.clone(), 500u128.into());
        assert_eq!(contract.ft_balance_of(accounts(1)), 500u128.into());
    }
}
