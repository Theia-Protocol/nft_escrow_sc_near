use near_sdk::AccountId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use crate::metadata::TokenMetadata;

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
