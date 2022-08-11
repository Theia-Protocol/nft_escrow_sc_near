use crate::multi_token::metadata::{MtContractMetadata, TokenMetadata, MT_METADATA_SPEC, MultiTokenMetadataProvider};
use crate::multi_token::token::{Token, TokenId};
use crate::multi_token::core::MultiToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ProxyMt {
    tokens: MultiToken,
    metadata: LazyOption<MtContractMetadata>,
    max_supply: u128,
    blank_media_uri: String,
    description: String,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    MultiToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

#[near_bindgen]
impl ProxyMt {
    /// Initializes the contract owned by `owner_id` with nft metadata
    #[init]
    pub fn new(owner_id: AccountId, name: String, symbol: String, blank_media_uri: String, max_supply: u128, description: String ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let metadata = MtContractMetadata {
            spec: MT_METADATA_SPEC.to_string(),
            name,
            symbol,
            icon: None,
            base_uri: Some(media_base_uri),
            reference: None,
            reference_hash: None,
        };

        Self {
            tokens: MultiToken::new(
                StorageKey::MultiToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            max_supply,
            blank_media_uri,
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
        assert!(self.current_index + amount < self.max_supply, "OverMaxSupply");
        assert_eq!(env::predecessor_account_id(), self.tokens.owner_id, "Unauthorized");

        let mut tokens: Vec<Token> = vec![];
        let mut i = 0;

        while i < amount {
            let mut title = "#".to_string();
            title.push_str(&i.to_string());

            let token_metadata = TokenMetadata {
                title: Some(title),
                description: Some(self.description.clone()),
                media: Some(self.blank_media_uri.clone()),
                media_hash: None,
                issued_at: None,
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: None,
                reference_hash: None
            };
            let token: Token = self.tokens.internal_mint(receiver_id.clone(), Some(1), Some(token_metadata), None);
            tokens.push(token);
            i += 1;
        }
        self.current_index += amount;
        tokens
    }
}

crate::multi_token::impl_multi_token_core!(ProxyMt, tokens);
crate::multi_token::impl_multi_token_approval!(ProxyMt, tokens);
crate::multi_token::impl_multi_token_enumeration!(ProxyMt, tokens);

#[near_bindgen]
impl MultiTokenMetadataProvider for ProxyMt {
    fn mt_metadata(&self) -> MtContractMetadata {
        self.metadata.get().unwrap()
    }
}