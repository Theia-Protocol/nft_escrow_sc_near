use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::{env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, StorageUsage, Promise, Gas, PromiseError};

mod utils;
mod errors;
mod views;
mod curves;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum RunningState {
    Running,
    Paused
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum ProjectTokenType {
    NonFungible,
    Fungible
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum CurveType {
    NonFungible,
    Fungible
}


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CurveArgs {
    pub arg_a: Option<u128>,
    pub arg_b: Option<u128>,
    pub arg_c: Option<u128>,
    pub arg_d: Option<u128>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Owner of contract
    owner_id: AccountId,
    /// Protocol account
    treasury_id: AccountId,
    /// Finder account
    finder_id: AccountId,
    /// Project token id
    project_token_id: AccountId,
    /// Proxy token id
    proxy_token_id: AccountId,
    /// Funding target amount
    fund_threshold: Balance,
    /// Start timestamp
    start_timestamp: u64,
    /// Threshold timestamp
    tp_timestamp: u64,
    /// Buffer period
    buffer_period: u64,
    /// Conversion period
    conversion_period: u64,
    /// Stable coin
    stable_coin_id: AccountId,
    /// Total fund amount
    total_fund_amount: Balance,
    /// Pre-mint amount
    pre_mint_amount: Balance,
    /// Amount of converted proxy token
    converted_proxy_token_amount: Balance,
    /// Auction curve type
    curve_type: CurveType,
    /// Auction curve args
    curve_args: CurveArgs,
    /// Running state
    state: RunningState,
}

const MIN_STORAGE: Balance = 1_100_000_000_000_000_000_000_000; //1.1Ⓝ
const TGAS: u64 = 1_000_000_000_000;

const PROXY_TOKEN_CODE: &[u8] = include_bytes!("./proxy_token/target/wasm32-unknown-unknown/release/proxy_token.wasm");
const NFT_COLLECTION_CODE: &[u8] = include_bytes!("./nft_collection/target/wasm32-unknown-unknown/release/nft_collection.wasm");
const FUNGIBLE_TOKEN_CODE: &[u8] = include_bytes!("./ft_token/target/wasm32-unknown-unknown/release/ft_token.wasm");

#[near_bindgen]
impl Contract {
    /// Initialize the contract
    #[init]
    pub fn new(owner_id: AccountId, name: String, symbol: String, uri_prefix: String, blank_uri: String, max_supply: Balance, finder_id: AccountId, stable_coin_id: AccountId, curve_type: CurveType, curve_args: CurveArgs, treasury_id: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        assert!(name.len() < 13 && name.len() > 2, "Invalid collection name");
        assert!(symbol.len() < 13 && symbol.len() > 2, "Invalid collection symbol");
        assert!(uri_prefix.len() > 0, "Invalid collection base uri");
        assert!(blank_uri.len() > 0, "Invalid blank uri");
        assert!(max_supply > 0, "Invalid max supply");

        let project_token_id = &name + "." + &env::current_account_id().to_string();
        Promise::new(project_token_id.parse().unwrap())
            .create_account()
            .transfer(MIN_STORAGE)
            .deploy_contract(NFT_COLLECTION_CODE.to_vec());

        nft_collection::ext(project_token_id)
            .with_static_gas(Gas(5*TGAS))
            .new(&name, &symbol, &uri_prefix, &max_supply);

        let proxy_token_id = "P" + name + "." + &env::current_account_id().to_string();
        Promise::new(proxy_token_id.parse().unwrap())
            .create_account()
            .transfer(MIN_STORAGE)
            .deploy_contract(PROXY_TOKEN_CODE.to_vec());

        proxy_token::ext(project_token_id)
            .with_static_gas(Gas(5*TGAS))
            .new(&name, &symbol, blank_uri, max_supply);

        Self {
            owner_id,
            treasury_id,
            finder_id,
            project_token_id,
            proxy_token_id: proxy_token,
            fund_threshold: 0,
            start_timestamp: 0,
            tp_timestamp: 0,
            buffer_period: 0,
            conversion_period: 0,
            stable_coin_id,
            total_fund_amount: 0,
            pre_mint_amount: 0,
            converted_proxy_token_amount: 0,
            curve_type,
            curve_args,
            state: RunningState::Running
        }
    }

    /// Active project
    pub fn active_project(&mut self, pre_mint_amount: Balance, fund_threshold: Balance, buffer_period: u64, conversion_period: u64) -> Promise {
        assert!(fund_threshold > 0, "Invalid funding target");
        assert!(conversion_period >= 86400, "Invalid conversion period");

        self.fund_threshold = fund_threshold;
        self.pre_mint_amount = pre_mint_amount;
        self.buffer_period = buffer_period;
        self.conversion_period = conversion_period;
        self.start_timestamp = env::block_timestamp();

        proxy_token::ext(self.proxy_token_id.clone())
            .with_static_gas(Gas(5*TGAS))
            .nft_mint(self.owner_id.clone(), pre_mint_amount)
            .then(self_ext::active_project_callback(&env::current_account_id(), 0, 5*TGAS))
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn active_project_callback(&self) {
        if call_result.is_err() {
            env::panic_str("There was an error contacting Hello NEAR");
        }
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
