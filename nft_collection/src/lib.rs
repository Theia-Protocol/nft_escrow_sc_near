use near_contract_standards::non_fungible_token::metadata::{NFTContractMetadata, TokenMetadata, NFT_METADATA_SPEC, NonFungibleTokenMetadataProvider};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    current_index: u128,
    max_supply: u128,
    json_base_uri: String,
    description: String,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with nft metadata
    #[init]
    pub fn new(owner_id: AccountId, name: String, symbol: String, media_base_uri: String, max_supply: u128, json_base_uri: String, description: String ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let metadata = NFTContractMetadata {
            spec: NFT_METADATA_SPEC.to_string(),
            name,
            symbol,
            icon: None,
            base_uri: Some(media_base_uri),
            reference: None,
            reference_hash: None,
        };

        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            current_index: 0u128,
            max_supply,
            json_base_uri,
            description,
        }
    }

    /// Mint nft tokens with amount belonging to `receiver_id`.
    /// caller should be owner
    #[payable]
    pub fn nft_mint(
        &mut self,
        receiver_id: AccountId,
        amount: u128,
    ) -> Vec<Token> {
        assert!(amount > 0, "Invalid amount");
        assert!(self.current_index.checked_add(amount).unwrap() < self.max_supply, "OverMaxSupply");
        assert_eq!(env::predecessor_account_id(), self.tokens.owner_id, "Unauthorized");

        let mut tokens: Vec<Token> = vec![];
        let mut i = 0;

        while i < amount {
            let mut title = "#".to_string();
            title.push_str(&i.to_string());

            let mut media_uri: String = self.metadata.get().unwrap().base_uri.unwrap().clone();
            media_uri.push_str(&i.to_string());
            media_uri.push_str(".json");

            let mut json_uri = self.json_base_uri.clone();
            json_uri.push_str(&i.to_string());
            json_uri.push_str(".json");

            let token_id: TokenId = (self.current_index + i).to_string();
            let token_metadata = TokenMetadata {
                title: Some(title),
                description: Some(self.description.clone()),
                media: Some(media_uri),
                media_hash: None,
                copies: None,
                issued_at: None,
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: Some(json_uri),
                reference_hash: None
            };
            let token: Token = self.tokens.internal_mint(token_id.to_string(), receiver_id.clone(), Some(token_metadata));
            tokens.push(token);
            i += 1;
        }
        self.current_index += amount;
        tokens
    }
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[allow(dead_code, unused)]
#[cfg(test)]
mod tests {
    use near_sdk::{test_utils::*, testing_env, AccountId, ONE_NEAR};

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
    fn test() {}
}