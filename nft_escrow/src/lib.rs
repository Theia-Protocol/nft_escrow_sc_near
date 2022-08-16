use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise, Gas};
use crate::utils::{FEE_DIVISOR, GAS_FOR_FT_TRANSFER, ext_fungible_token, ext_nft_collection, ext_proxy_token};

mod utils;
mod errors;
mod views;
mod curves;
mod owner;

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
    Horizontal,
    Linear,
    Sigmoidal
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
    /// Protocol fee percent
    protocol_fee: u32,
    /// Finder account
    finder_id: Option<AccountId>,
    /// Finder fee percent
    finder_fee: u32,
    /// Project token type
    project_token_type: ProjectTokenType,
    /// Project token id
    project_token_id: Option<AccountId>,
    /// Proxy token id
    proxy_token_id: Option<AccountId>,
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

const PROXY_TOKEN_CODE: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/proxy_token.wasm");
const NFT_COLLECTION_CODE: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/nft_collection.wasm");
const FUNGIBLE_TOKEN_CODE: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/ft_token.wasm");

#[near_bindgen]
impl Contract {
    /// Initialize the contract
    #[init]
    pub fn new(owner_id: AccountId, stable_coin_id: AccountId, stable_coin_decimals: u8, curve_type: CurveType, curve_args: CurveArgs, treasury_id: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");

        Self {
            owner_id,
            treasury_id,
            protocol_fee: 100,  // 1%
            finder_id: None,
            finder_fee: 100,    // 1%
            project_token_type: ProjectTokenType::NonFungible,
            project_token_id: None,
            proxy_token_id: None,
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

    /// Active NFT project
    pub fn active_nft_project(&mut self, name: String, symbol: String, uri_prefix: String, blank_uri: String, max_supply: Balance, finder_id: AccountId, pre_mint_amount: Balance, fund_threshold: Balance, buffer_period: u64, conversion_period: u64) -> Promise {
        assert!(name.len() < 13 && name.len() > 2, "Invalid collection name");
        assert!(symbol.len() < 13 && symbol.len() > 2, "Invalid collection symbol");
        assert!(uri_prefix.len() > 0, "Invalid collection base uri");
        assert!(blank_uri.len() > 0, "Invalid blank uri");
        assert!(max_supply > 0, "Invalid max supply");
        assert!(fund_threshold > 0, "Invalid funding target");
        assert!(conversion_period >= 86400, "Invalid conversion period");

        self.finder_id = Some(finder_id);
        self.fund_threshold = fund_threshold;
        self.pre_mint_amount = pre_mint_amount;
        self.buffer_period = buffer_period;
        self.conversion_period = conversion_period;
        self.start_timestamp = env::block_timestamp();
        self.project_token_type = ProjectTokenType::NonFungible;

        let project_token_id = name.clone() + "." + &env::current_account_id().to_string();
        let proxy_token_id = "P".to_owned() + &name + "." + &env::current_account_id().to_string();
        self.project_token_id = Some(project_token_id.parse().unwrap());
        self.proxy_token_id = Some(proxy_token_id.parse().unwrap());

        Promise::new(self.project_token_id.unwrap())
            .create_account()
            .transfer(MIN_STORAGE)
            .deploy_contract(NFT_COLLECTION_CODE.to_vec())
            .then(
                ext_nft_collection::new(
                    self.project_token_id.unwrap(),
                    name.clone(),
                    symbol.clone(),
                    &uri_prefix,
                    &max_supply,
                    1,  // one yocto near
                    Gas::ONE_TERA,
                )
                    .then(
                        Promise::new(self.proxy_token_id.unwrap())
                            .create_account()
                            .transfer(MIN_STORAGE)
                            .deploy_contract(PROXY_TOKEN_CODE.to_vec())
                            .then(
                                ext_proxy_token::new(
                                    self.proxy_token_id.unwrap(),
                                    &name,
                                    &symbol,
                                    blank_uri,
                                    max_supply,
                                    1,  // one yocto near
                                    Gas::ONE_TERA,
                                )
                                .then(
                                    ext_proxy_token::nft_mint(
                                        self.proxy_token_id.unwrap(),
                                        self.owner_id.clone(),
                                        pre_mint_amount,
                                        1,  // one yocto near
                                        Gas::ONE_TERA,
                                    )
                                )
                            )
                    )
            )
    }

    /// Active FT project
    pub fn active_ft_project(&mut self, name: String, symbol: String, finder_id: AccountId, pre_mint_amount: Balance, fund_threshold: Balance, buffer_period: u64, conversion_period: u64) -> Promise {
        assert!(name.len() < 13 && name.len() > 2, "Invalid collection name");
        assert!(symbol.len() < 13 && symbol.len() > 2, "Invalid collection symbol");
        assert!(fund_threshold > 0, "Invalid funding target");
        assert!(conversion_period >= 86400, "Invalid conversion period");

        self.finder_id = Some(finder_id);
        self.fund_threshold = fund_threshold;
        self.pre_mint_amount = pre_mint_amount;
        self.buffer_period = buffer_period;
        self.conversion_period = conversion_period;
        self.start_timestamp = env::block_timestamp();
        self.project_token_type = ProjectTokenType::Fungible;

        Promise::new(self.project_token_id.unwrap())
            .create_account()
            .transfer(MIN_STORAGE)
            .deploy_contract(FUNGIBLE_TOKEN_CODE.to_vec())
            .then(
                ext_fungible_token::new(
                    self.project_token_id.unwrap(),
                    name.clone(),
                    symbol.clone(),
                    1,  // one yocto near
                    Gas::ONE_TERA,
                )
                    .then(
                        Promise::new(self.proxy_token_id.unwrap())
                            .create_account()
                            .transfer(MIN_STORAGE)
                            .deploy_contract(PROXY_TOKEN_CODE.to_vec())
                            .then(
                                ext_proxy_token::new(
                                    self.proxy_token_id.unwrap(),
                                    &name,
                                    &symbol,
                                    blank_uri,
                                    max_supply,
                                    1,  // one yocto near
                                    Gas::ONE_TERA,
                                )
                                    .then(
                                        ext_proxy_token::nft_mint(
                                            self.proxy_token_id.unwrap(),
                                            self.owner_id.clone(),
                                            pre_mint_amount,
                                            1,  // one yocto near
                                            Gas::ONE_TERA,
                                        )
                                    )
                            )
                    )
            )
    }

    pub fn buy(&mut self, amount: Balance, coin_amount: Balance) -> Promise {
        let cal_coin_amount = self.calculate_buy_proxy_token(amount);

        assert!(coin_amount >= cal_coin_amount, "Insufficient fund");

        let protocol_fee_amount = cal_coin_amount
            .checked_mul(self.protocol_fee as u128)
            .unwrap()
            .checked_div(FEE_DIVISOR as u128)
            .unwrap();

        let reserve_fund_amount = cal_coin_amount.checked_sub(protocol_fee_amount).unwrap();

        self.total_fund_amount = self.total_fund_amount
            .checked_add(reserve_fund_amount)
            .unwrap();

        if self.tp_timestamp == 0 && self.total_fund_amount >= self.fund_threshold {
            self.tp_timestamp = env::block_timestamp();
        }

        ext_fungible_token::ft_transfer(
            &(self.stable_coin_id),
            env::predecessor_account_id(),
            reserve_fund_amount.into(),
            None,
            1,  // one yocto near
            GAS_FOR_FT_TRANSFER,
        )
        .then(
            ext_fungible_token::ft_transfer(
                &(self.stable_coin_id),
                env::predecessor_account_id(),
                protocol_fee_amount,
                None,
                1,  // one yocto near
                GAS_FOR_FT_TRANSFER,
            )
        ).then(
            ext_proxy_token::nft_mint(
                self.proxy_token_id.clone(),
                env::predecessor_account_id(),
                amount,
                1,  // one yocto near
                GAS_FOR_FT_TRANSFER
            )
        )
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
