use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_units::parse_near;

pub const NAME: &str = "Theia Collection 1";
pub const SYMBOL: &str = "TCN";
pub const NFT_BASE_URI: &str = "https://ipfs.io/ipfs/QmUDqczgXxZ7exQ9znjZRB1CCvEmQ5FZchatueZXWnIkly/";
pub const NFT_BLANK_URI: &str = "https://ipfs.io/ipfs/QmZRBnIklexQCvEmQxZ1CDqczgXcy7hatu9eZXW5FZUznj";
pub const NFT_MAX_SUPPLY: U128 = U128(20_000u128);
pub const FT_MAX_SUPPLY: U128 = U128(1000_000u128);
pub const PRE_MINT_AMOUNT: U128 = U128(2u128);
pub const FUND_THRESHOLD: U128 = U128(parse_near!("200 N"));
pub const PROTOCOL_FEE: u16 = 1u16; // 1%
pub const FINDER_FEE: u16 = 1u16; // 1%

pub const FIVE_MINUTES: u128 = 300u128 * 1_000_000_000;    // 5 min (nanosecond)
pub const TEN_MINUTES: u128 = 600u128 * 1_000_000_000;   // 10 min (nanosecond)

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
