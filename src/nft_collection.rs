use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};
use crate::nft_collection::StorageKey::TokenMetadata;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct NftCollection {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    current_index: u128,
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
impl NftCollection {
    /// Initializes the contract owned by `owner_id` with nft metadata
    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
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
        }
    }

    /// Mint a new token with ID=`token_id` belonging to `receiver_id`.
    ///
    /// Since this example implements metadata, it also requires per-token metadata to be provided
    /// in this call. `self.tokens.mint` will also require it to be Some, since
    /// `StorageKey::TokenMetadata` was provided at initialization.
    ///
    /// `self.tokens.mint` will enforce `predecessor_account_id` to equal the `owner_id` given in
    /// initialization call to `new`.
    #[payable]
    pub fn nft_mint(
        &mut self,
        receiver_id: AccountId,
        amount: u128,
    ) -> Vec<Token> {
        tokens: Vec<Token>;
        assert!(amount > 0, "Invalid amount");
        for i in 1..amount {
            let mut token_uri: &str = self.metadata.base_uri.into();
            token_uri.push_str(i.to_string());

            let token_id: u128 = self.current_index.into() + i.into();
            let token_metadata = TokenMetadata {
                title: None,
                description: None,
                media: None,
                media_hash: None,
                copies: None,
                issued_at: None,
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: token_uri.into(),
                reference_hash: None
            };
            let token: Token = self.tokens.mint(token_id.into(), receiver_id.clone(), Some(token_metadata));
            tokens.push(token.into());
        }
        tokens
    }
}

near_contract_standards::impl_non_fungible_token_core!(NftCollection, tokens);
near_contract_standards::impl_non_fungible_token_approval!(NftCollection, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(NftCollection, tokens);