use near_units::parse_near;

pub const NAME: &str = "Theia Collection 1";
pub const SYMBOL: &str = "TCN";
pub const NFT_BASE_URI: &str = "https://ipfs.io/ipfs/QmUDqczgXxZ7exQ9znjZRB1CCvEmQ5FZchatueZXWnIkly/";
pub const NFT_BLANK_URI: &str = "https://ipfs.io/ipfs/QmZRBnIklexQCvEmQxZ1CDqczgXcy7hatu9eZXW5FZUznj";
pub const NFT_MAX_SUPPLY: u128 = 1000u128;
pub const PRE_MINT_AMOUNT: u128 = 2u128;
pub const FUND_THRESHOLD: u128 = parse_near!("200 N");
pub const PROTOCOL_FEE: u16 = 1u16; // 1%
pub const FINDER_FEE: u16 = 1u16; // 1%

pub const ONE_DAY: u128 = 3600u128 * 24u128;
pub const TWO_DAYS: u128 = 3600u128 * 24u128 * 2u128;
pub const ONE_WEEK: u128 = 3600u128 * 24u128 * 7u128;