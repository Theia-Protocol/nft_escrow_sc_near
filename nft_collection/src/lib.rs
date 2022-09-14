mod owner;

use std::collections::HashMap;

use near_contract_standards::non_fungible_token::events::NftMint;
use near_contract_standards::non_fungible_token::metadata::{NFTContractMetadata, TokenMetadata, NFT_METADATA_SPEC, NonFungibleTokenMetadataProvider};
use near_contract_standards::non_fungible_token::{Token, TokenId, refund_deposit_to_account};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedSet};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, CryptoHash, assert_one_yocto};
use near_sdk::json_types::U128;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    current_index: u128,
    max_supply: u128
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum NonFungibleTokenStorageKey {
    TokensPerOwner { account_hash: Vec<u8> },
    TokenPerOwnerInner { account_id_hash: CryptoHash },
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with nft metadata
    #[init]
    pub fn new(owner_id: AccountId, name: String, symbol: String, base_uri: String, max_supply: U128) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let metadata = NFTContractMetadata {
            spec: NFT_METADATA_SPEC.to_string(),
            name,
            symbol,
            icon: None,
            base_uri: Some(base_uri),
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
            max_supply: max_supply.0
        }
    }

    /// Mint nft tokens with amount belonging to `receiver_id`.
    /// caller should be owner
    #[payable]
    pub fn nft_mint(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
    ) -> Vec<Token> {
        assert!(amount.0 > 0, "Invalid amount");
        assert!(self.current_index.checked_add(amount.0).unwrap() < self.max_supply, "OverMaxSupply");
        assert_eq!(env::predecessor_account_id(), self.tokens.owner_id, "Unauthorized");

        let mut tokens: Vec<Token> = vec![];
        let mut i = 0;
        let mut all_token_ids: Vec<TokenId> = vec![];

        // Remember current storage usage if refund_id is Some
        let initial_storage_usage = env::storage_usage();

        while i < amount.0 {
            let title = Some(format!("{} #{}", self.metadata.get().unwrap().name, i.to_string()));
            let media = Some(i.to_string());
            let reference = Some(format!("{}.json", i.to_string()));

            let token_id: TokenId = (self.current_index + i).to_string();
            let token_metadata = Some(TokenMetadata {
                title,
                description: None,
                media,
                media_hash: None,
                copies: Some(1u64),
                issued_at: Some(env::block_timestamp().to_string()),
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference,
                reference_hash: None
            });
            
            //let token: Token = self.tokens.internal_mint(token_id.to_string(), receiver_id.clone(), token_metadata);
                
            if self.tokens.owner_by_id.get(&token_id).is_some() {
                env::panic_str("token_id must be unique");
            }

            let owner_id: AccountId = receiver_id.clone();

            // Core behavior: every token must have an owner
            self.tokens.owner_by_id.insert(&token_id, &owner_id);

            // Metadata extension: Save metadata, keep variable around to return later.
            // Note that check above already panicked if metadata extension in use but no metadata
            // provided to call.
            self.tokens.token_metadata_by_id
                .as_mut()
                .and_then(|by_id| by_id.insert(&token_id, token_metadata.as_ref().unwrap()));

            // Enumeration extension: Record tokens_per_owner for use with enumeration view methods.
            if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
                let mut token_ids = tokens_per_owner.get(&owner_id).unwrap_or_else(|| {
                    UnorderedSet::new(NonFungibleTokenStorageKey::TokensPerOwner {
                        account_hash: env::sha256(&owner_id.as_bytes()),
                    })
                });
                token_ids.insert(&token_id);
                tokens_per_owner.insert(&owner_id, &token_ids);
            }

            // Approval Management extension: return empty HashMap as part of Token
            let approved_account_ids =
                if self.tokens.approvals_by_id.is_some() { Some(HashMap::new()) } else { None };

            all_token_ids.push(owner_id.to_string());

            let token = Token { token_id, owner_id: owner_id.clone(), metadata: token_metadata, approved_account_ids };
            
            tokens.push(token.clone());
            i += 1;
        }
        
        self.current_index += amount.0;

        let str_token_ids: Vec<&str> = all_token_ids.iter().map(AsRef::as_ref).collect();
        NftMint { owner_id: &receiver_id, token_ids: &str_token_ids, memo: None }.emit();

        // Return any extra attached deposit not used for storage
        refund_deposit_to_account(env::storage_usage() - initial_storage_usage, receiver_id);

        tokens
    }

    pub fn nft_batch_transfer(&mut self, to: AccountId, token_ids: Vec<TokenId>, approval_id: Option<u64>, memo: Option<String>) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
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
        let mut contract = Contract::new(
            owner_id.clone(),
            String::from("Test FT"),
            String::from("TFT"),
            String::from("https://ipfs.io/ipfs/QmXa5nrfaqrvvcYFeEvs8E9W7AAeCZeUAuN6jophN9y8Ds/"),
            U128::from(100)
        );

        // mint
        testing_env!(
            get_context(owner_id.clone())
                .attached_deposit(553 * env::storage_byte_cost())
                .build()
        );
        contract.nft_mint(accounts(0), 1u128.into());
        assert_eq!(contract.nft_supply_for_owner(accounts(0)), 1u128.into());

        // transfer
        testing_env!(
             get_context(owner_id.clone())
                .attached_deposit(1)
                .build());
        contract.nft_transfer(alice_id.clone(), "0".to_string(), None, None);
        assert_eq!(contract.nft_supply_for_owner(accounts(0)), 0u128.into());
        assert_eq!(contract.nft_supply_for_owner(accounts(1)), 1u128.into());
        assert_eq!(contract.nft_total_supply(), 1u128.into());
    }
}