use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::{env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, StorageUsage};
use near_contract_standards::non_fungible_token::refund_deposit_to_account;
use crate::event::{MtBurn, MtMint};
use crate::metadata::{MT_METADATA_SPEC, MtContractMetadata, TokenMetadata};
use crate::token::{Token, TokenId};

mod event;
mod token;
mod metadata;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Owner of contract
    owner_id: AccountId,

    /// How much storage takes every token
    extra_storage_in_bytes_per_emission: StorageUsage,

    /// Total supply for each token
    total_supply: LookupMap<TokenId, Balance>,

    /// Balance of user for given token
    balances_per_token: UnorderedMap<TokenId, LookupMap<AccountId, Balance>>,

    /// Next id for token
    all_total_supply: Balance,

    metadata: LazyOption<MtContractMetadata>,

    max_supply: u128,

    blank_media_uri: String,

    description: String,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Metadata,
    TotalSupply { supply: u128 },
    Balances,
    BalancesInner { token_id: Vec<u8> },
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with nft metadata
    #[init]
    pub fn new(owner_id: AccountId, name: String, symbol: String, blank_media_uri: String, max_supply: u128, description: String ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let metadata = MtContractMetadata {
            spec: MT_METADATA_SPEC.to_string(),
            name,
            symbol,
            icon: None,
            base_uri: None,
            reference: None,
            reference_hash: None,
        };

        Self {
            owner_id,
            extra_storage_in_bytes_per_emission: 0,
            total_supply: LookupMap::new(StorageKey::TotalSupply { supply: u128::MAX }),
            balances_per_token: UnorderedMap::new(StorageKey::Balances),
            all_total_supply: 0,
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            max_supply,
            blank_media_uri,
            description,
        }
    }

    /// Mint nft tokens with amount belonging to `receiver_id`.
    /// caller should be owner
    #[payable]
    pub fn mt_mint(
        &mut self,
        receiver_id: AccountId,
        amount: u128,
    ) -> Vec<TokenId> {
        assert!(amount > 0, "Invalid amount");
        assert!(self.all_total_supply.checked_add(amount).unwrap() < self.max_supply, "OverMaxSupply");
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Unauthorized");

        let mut token_ids: Vec<TokenId> = vec![];
        let mut i = 0;
        let refund_id = Some(env::predecessor_account_id());

        while i < amount {

            // Remember current storage usage if refund_id is Some
            let initial_storage_usage = refund_id.as_ref().map(|account_id| (account_id, env::storage_usage()));

            let token_id: TokenId = self.all_total_supply.checked_add(i).unwrap().to_string();

            // Insert new supply
            self.total_supply.insert(
                &token_id,
                &self
                    .total_supply
                    .get(&token_id).unwrap_or(0)
                    .checked_add(1)
                    .unwrap_or_else(|| env::panic_str("Total supply overflow")));

            // Insert new balance
            if self.balances_per_token.get(&token_id).is_none() {
                let mut new_set: LookupMap<AccountId, u128> = LookupMap::new(StorageKey::BalancesInner {
                    token_id: env::sha256(token_id.as_bytes()),
                });
                new_set.insert(&receiver_id, &1u128);
                self.balances_per_token.insert(&token_id, &new_set);
            } else {
                let new = self.balances_per_token.get(&token_id).unwrap().get(&receiver_id).unwrap_or(0).checked_add(1).unwrap();
                let mut balances = self.balances_per_token.get(&token_id).unwrap();
                balances.insert(&receiver_id, &new);
            }

            if let Some((id, usage)) = initial_storage_usage {
                refund_deposit_to_account(env::storage_usage() - usage, id.clone());
            }

            token_ids.push(token_id.clone());
            i += 1;

            MtMint {
                owner_id: &receiver_id,
                token_ids: &[&token_id],
                amounts: &["1"],
                memo: None,
            }
                .emit();
        }
        self.all_total_supply = self.all_total_supply.checked_add(amount).unwrap();

        token_ids
    }

    /// Burn nft tokens from `from_id`.
    /// caller should be owner
    pub fn mt_burn(
        &mut self,
        from_id: AccountId,
        token_ids: Vec<TokenId>,
    ) -> bool {
        assert!(token_ids.len() > 0, "Invalid param");
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Unauthorized");

        token_ids.iter().enumerate().for_each(|(_, token_id)| {
            let balance = self.internal_unwrap_balance_of(token_id, &from_id);
            if let Some(new) = balance.checked_sub(1) {
                let mut balances = self.balances_per_token.get(token_id).unwrap();
                balances.insert(&from_id, &new);
                self.total_supply.insert(
                    token_id,
                    &self
                        .total_supply
                        .get(token_id)
                        .unwrap()
                        .checked_sub(1)
                        .unwrap_or_else(|| env::panic_str("Total supply overflow")),
                );
            } else {
                env::panic_str("The account doesn't have enough balance");
            }

            MtBurn {
                owner_id: &from_id,
                authorized_id: Some(&self.owner_id),
                token_ids: &[&token_id],
                amounts: &["1"],
                memo: None,
            }
                .emit();
        });

        self.all_total_supply = self.all_total_supply.checked_sub(token_ids.len().try_into().unwrap()).unwrap();

        true
    }

    pub fn mt_token(&self, token_id: TokenId) -> Option<Token> {
        let metadata = TokenMetadata {
            title: Some(token_id.clone()),
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
        let supply = self.total_supply.get(&token_id)?;

        Some(Token {
            token_id,
            owner_id: None,
            supply,
            metadata
        })
    }

    /// Used to get balance of specified account in specified token
    pub fn internal_unwrap_balance_of(
        &self,
        token_id: &TokenId,
        account_id: &AccountId,
    ) -> Balance {
        match self
            .balances_per_token
            .get(token_id)
            .expect("This token does not exist")
            .get(account_id)
        {
            Some(balance) => balance,
            None => {
                env::panic_str(format!("The account {} is not registered", account_id).as_str())
            }
        }
    }

    pub fn mt_balance_of(&self, owner: AccountId, id: Vec<TokenId>) -> Vec<u128> {
        self.balances_per_token
            .iter()
            .filter(|(token_id, _)| id.contains(token_id))
            .map(|(_, balances)| {
                balances
                    .get(&owner)
                    .expect("User does not have account in of the tokens")
            })
            .collect()
    }

    pub fn mt_metadata(&self) -> MtContractMetadata {
        self.metadata.get().unwrap()
    }

    pub fn mt_all_total_supply(&self) -> Balance { self.all_total_supply.clone() }
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
