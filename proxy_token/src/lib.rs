use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::{env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, StorageUsage, require, serde_json};
use near_contract_standards::non_fungible_token::refund_deposit_to_account;

/// Type alias for convenience
pub type TokenId = String;

/// Info on individual token
#[derive(Debug, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Token {
    pub token_id: TokenId,
    pub owner_id: Option<AccountId>,
    /// Total amount generated
    pub supply: u128,
    pub metadata: TokenMetadata,
}

/// Version of standard
pub const MT_METADATA_SPEC: &str = "mt-0.0.1";

/// Metadata that will be permanently set at the contract init
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct MtContractMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub base_uri: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<String>,
}

/// Metadata for each token
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: Option<String>,
    /// Free-form description
    pub description: Option<String>,
    /// URL to associated media, preferably to decentralized, content-addressed storage
    pub media: Option<String>,
    /// Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
    pub media_hash: Option<String>,
    /// When token was issued or minted, Unix epoch in milliseconds
    pub issued_at: Option<String>,
    /// When token expires, Unix epoch in milliseconds
    pub expires_at: Option<String>,
    /// When token starts being valid, Unix epoch in milliseconds
    pub starts_at: Option<String>,
    /// When token was last updated, Unix epoch in milliseconds
    pub updated_at: Option<String>,
    /// Anything extra the MT wants to store on-chain. Can be stringified JSON.
    pub extra: Option<String>,
    /// URL to an off-chain JSON file with more info.
    pub reference: Option<String>,
    /// Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
    pub reference_hash: Option<String>,
}


impl MtContractMetadata {
    pub fn assert_valid(&self) {
        require!(self.spec == MT_METADATA_SPEC, "Spec is not NFT metadata");
        require!(
            self.reference.is_some() == self.reference_hash.is_some(),
            "Reference and reference hash must be present"
        );
        if let Some(reference_hash) = &self.reference_hash {
            require!(reference_hash.len() == 32, "Hash has to be 32 bytes");
        }
    }
}

impl TokenMetadata {
    pub fn assert_valid(&self) {
        require!(self.media.is_some() == self.media_hash.is_some());
        if let Some(media_hash) = &self.media_hash {
            require!(media_hash.len() == 32, "Media hash has to be 32 bytes");
        }

        require!(self.reference.is_some() == self.reference_hash.is_some());
        if let Some(reference_hash) = &self.reference_hash {
            require!(reference_hash.len() == 32, "Reference hash has to be 32 bytes");
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "standard")]
#[must_use = "don't forget to `.emit()` this event"]
#[serde(rename_all = "snake_case")]
pub(crate) enum NearEvent<'a> {
    Nep245(Nep245Event<'a>),
}

impl<'a> NearEvent<'a> {
    fn to_json_string(&self) -> String {
        // Events cannot fail to serialize so fine to panic on error
        #[allow(clippy::redundant_closure)]
        serde_json::to_string(self).ok().unwrap_or_else(|| env::abort())
    }

    fn to_json_event_string(&self) -> String {
        format!("EVENT_JSON:{}", self.to_json_string())
    }

    /// Logs the event to the host. This is required to ensure that the event is triggered
    /// and to consume the event.
    pub(crate) fn emit(self) {
        near_sdk::env::log_str(&self.to_json_event_string());
    }
}

#[must_use]
#[derive(Serialize, Debug, Clone)]
pub struct MtMint<'a> {
    pub owner_id: &'a AccountId,
    pub token_ids: &'a [&'a str],
    pub amounts: &'a [&'a str],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<&'a str>
}

impl MtMint<'_> {
    pub fn emit(self) {
        Self::emit_many(&[self])
    }

    pub fn emit_many(data: &[MtMint<'_>]) {
        new_245_v1(Nep245EventKind::MtMint(data)).emit()
    }
}

#[must_use]
#[derive(Serialize, Debug, Clone)]
pub struct MtTransfer<'a> {
    pub old_owner_id: &'a AccountId,
    pub new_owner_id: &'a AccountId,
    pub token_ids: &'a [&'a str],
    pub amounts: &'a [&'a str],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_id: Option<&'a AccountId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<&'a str>
}

impl MtTransfer<'_> {
    pub fn emit(self) {
        Self::emit_many(&[self])
    }

    pub fn emit_many(data: &[MtTransfer<'_>]) {
        new_245_v1(Nep245EventKind::MtTransfer(data)).emit()
    }
}

#[must_use]
#[derive(Serialize, Debug, Clone)]
pub struct MtBurn<'a> {
    pub owner_id: &'a AccountId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_id: Option<&'a AccountId>,
    pub token_ids: &'a [&'a str],
    pub amounts: &'a [&'a str],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<&'a str>
}

impl MtBurn<'_> {
    pub fn emit(self) {
        Self::emit_many(&[self])
    }

    pub fn emit_many(data: &[MtBurn<'_>]) {
        new_245_v1(Nep245EventKind::MtBurn(data)).emit()
    }
}

#[derive(Serialize, Debug)]
pub struct Nep245Event<'a> {
    version:  &'static str,
    #[serde(flatten)]
    event_kind: Nep245EventKind<'a>
}

#[derive(Serialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
enum Nep245EventKind<'a> {
    MtMint(&'a [MtMint<'a>]),
    MtTransfer(&'a [MtTransfer<'a>]),
    MtBurn(&'a [MtBurn<'a>]),
}

fn new_245<'a>(version: &'static str, event_kind: Nep245EventKind<'a>) -> NearEvent<'a> {
    NearEvent::Nep245(Nep245Event { version, event_kind })
}

fn new_245_v1(event_kind: Nep245EventKind) -> NearEvent {
    new_245("1.0.0", event_kind)
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ProxyMt {
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
}