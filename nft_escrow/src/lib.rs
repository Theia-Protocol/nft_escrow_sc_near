mod utils;
mod errors;
mod views;
mod curves;
mod owner;
mod validates;
mod pause;

use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise, Gas, PromiseError, PromiseOrValue, is_promise_success};
use near_sdk::json_types::U128;
use near_units::parse_gas;
use serde_json::to_vec;
use crate::errors::*;
use crate::utils::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Owner of contract
    owner_id: AccountId,
    /// Protocol account
    treasury_id: AccountId,
    /// Protocol fee percent
    treasury_fee: u32,
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
    /// Stable coin decimals
    stable_coin_decimals: u8,
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
    /// Closed
    is_closed: bool,
}

//1.1Ⓝ
const MIN_STORAGE: Balance = 1_100_000_000_000_000_000_000_000;
const TGAS: u64 = 1_000_000_000_000;
const GAS_NON_FUNGIBLE_TOKEN_NEW: Gas = Gas(parse_gas!("20 Tgas") as u64);
const GAS_FUNGIBLE_TOKEN_NEW: Gas = Gas(parse_gas!("20 Tgas") as u64);
const GAS_PROXY_TOKEN_NEW: Gas = Gas(parse_gas!("50 Tgas") as u64);
const GAS_PROXY_TOKEN_MINT: Gas = Gas(parse_gas!("50 Tgas") as u64);
const NO_DEPOSIT: Balance = 0u128;

const PROXY_TOKEN_CODE: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/proxy_token.wasm");
const NFT_COLLECTION_CODE: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/nft_collection.wasm");
const FUNGIBLE_TOKEN_CODE: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/ft_token.wasm");

#[near_bindgen]
impl Contract {
    /// Initialize the contract
    #[init]
    pub fn new(owner_id: AccountId, stable_coin_id: AccountId, stable_coin_decimals: u8, curve_type: CurveType, curve_args: CurveArgs, treasury_id: AccountId) -> Self {
        assert!(!env::state_exists(), "{}", ERR08_ALREADY_INITIALIZED);

        Self {
            owner_id,
            treasury_id,
            treasury_fee: 100,  // 1%
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
            stable_coin_decimals,
            total_fund_amount: 0,
            pre_mint_amount: 0,
            converted_proxy_token_amount: 0,
            curve_type,
            curve_args,
            state: RunningState::Running,
            is_closed: false,
        }
    }

    /// Active NFT project
    pub fn active_nft_project(&mut self, name: String, symbol: String, base_uri: String, blank_media_uri: String, max_supply: Balance, finder_id: AccountId, pre_mint_amount: Balance, fund_threshold: Balance, buffer_period: u64, conversion_period: u64) -> Promise {
        self.assert_owner();
        assert_eq!(self.is_closed, false, "{}", ERR013_ALREADY_CLOSED);
        assert!(name.len() > 2, "{}", ERR00_INVALID_NAME);
        assert!(symbol.len() < 13 && symbol.len() > 2, "{}", ERR01_INVALID_SYMBOL);
        assert!(base_uri.len() > 0, "{}", ERR02_INVALID_COLLECTION_BASE_URI);
        assert!(blank_media_uri.len() > 0, "{}", ERR03_INVALID_BLANK_URI);
        assert!(max_supply > 0, "{}", ERR04_INVALID_MAX_SUPPLY);
        assert!(fund_threshold > 0, "{}", ERR05_INVALID_FUNDING_TARGET);
        assert!(conversion_period >= 86400, "{}", ERR06_INVALID_CONVERSION_PERIOD);

        self.finder_id = Some(finder_id);
        self.fund_threshold = fund_threshold;
        self.pre_mint_amount = pre_mint_amount;
        self.buffer_period = buffer_period;
        self.conversion_period = conversion_period;
        self.start_timestamp = env::block_timestamp();
        self.project_token_type = ProjectTokenType::NonFungible;

        let project_token_id = AccountId::try_from(name.clone() + "." + &env::current_account_id().to_string()).expect("The project token account ID is invalid");
        let proxy_token_id = AccountId::try_from("P".to_owned() + &name + "." + &env::current_account_id().to_string()).expect("The proxy token account ID is invalid");

        // deploy non-fungible token
        let project_token_promise = Promise::new(project_token_id.clone())
            .create_account()
            .transfer(MIN_STORAGE)
            .deploy_contract(NFT_COLLECTION_CODE.to_vec())
            .function_call("new".to_string(), to_vec(&NonFungibleTokenArgs { owner_id: env::current_account_id(), name: name.clone(), symbol: symbol.clone(), base_uri, max_supply }).unwrap(), NO_DEPOSIT, GAS_NON_FUNGIBLE_TOKEN_NEW);

        // deploy proxy token
        let proxy_token_promise = Promise::new(proxy_token_id.clone())
            .create_account()
            .transfer(MIN_STORAGE)
            .deploy_contract(PROXY_TOKEN_CODE.to_vec())
            .function_call("new".to_string(), to_vec(&ProxyTokenArgs { owner_id: env::current_account_id(), name, symbol, blank_media_uri, max_supply }).unwrap(), NO_DEPOSIT, GAS_PROXY_TOKEN_NEW)
            .function_call("mt_mint".to_string(), to_vec(&ProxyTokenMintArgs { receiver_id: self.owner_id.clone(), amount: pre_mint_amount }).unwrap(), NO_DEPOSIT, GAS_PROXY_TOKEN_MINT);

        project_token_promise.and(proxy_token_promise).then(
            ext_self::ext(env::current_account_id()).on_activate(project_token_id, proxy_token_id, env::attached_deposit(), env::predecessor_account_id())
        )
    }

    /// Active FT project
    pub fn active_ft_project(&mut self, name: String, symbol: String, blank_uri: String, max_supply: u128, finder_id: AccountId, pre_mint_amount: Balance, fund_threshold: Balance, buffer_period: u64, conversion_period: u64) -> Promise {
        self.assert_owner();
        assert_eq!(self.is_closed, false, "{}", ERR013_ALREADY_CLOSED);
        assert!(name.len() < 13 && name.len() > 2, "{}", ERR00_INVALID_NAME);
        assert!(symbol.len() < 13 && symbol.len() > 2, "{}", ERR01_INVALID_SYMBOL);
        assert!(blank_uri.len() > 0, "{}", ERR03_INVALID_BLANK_URI);
        assert!(max_supply > 0, "{}", ERR04_INVALID_MAX_SUPPLY);
        assert!(fund_threshold > 0, "{}", ERR05_INVALID_FUNDING_TARGET);
        assert!(conversion_period >= 86400, "{}", ERR06_INVALID_CONVERSION_PERIOD);

        self.finder_id = Some(finder_id);
        self.fund_threshold = fund_threshold;
        self.pre_mint_amount = pre_mint_amount;
        self.buffer_period = buffer_period;
        self.conversion_period = conversion_period;
        self.start_timestamp = env::block_timestamp();
        self.project_token_type = ProjectTokenType::Fungible;

        let project_token_id = AccountId::try_from(name.clone() + "." + &env::current_account_id().to_string()).expect("The project token account ID is invalid");
        let proxy_token_id = AccountId::try_from("P".to_owned() + &name + "." + &env::current_account_id().to_string()).expect("The proxy token account ID is invalid");

        // deploy fungible token
        let project_token_promise = Promise::new(project_token_id.clone())
            .create_account()
            .transfer(MIN_STORAGE)
            .deploy_contract(FUNGIBLE_TOKEN_CODE.to_vec())
            .function_call("new".to_string(), to_vec(&FungibleTokenArgs { owner_id: env::current_account_id(), name: name.clone(), symbol: symbol.clone() }).unwrap(), NO_DEPOSIT, GAS_FUNGIBLE_TOKEN_NEW);

        // deploy proxy token
        let proxy_token_promise = Promise::new(proxy_token_id.clone())
            .create_account()
            .transfer(MIN_STORAGE)
            .deploy_contract(PROXY_TOKEN_CODE.to_vec())
            .function_call("new".to_string(), to_vec(&ProxyTokenArgs { owner_id: env::current_account_id(), name, symbol, blank_media_uri: blank_uri, max_supply }).unwrap(), NO_DEPOSIT, GAS_PROXY_TOKEN_NEW)
            .function_call("mt_mint".to_string(), to_vec(&ProxyTokenMintArgs { receiver_id: self.owner_id.clone(), amount: pre_mint_amount }).unwrap(), NO_DEPOSIT, GAS_PROXY_TOKEN_MINT);

        project_token_promise.and(proxy_token_promise).then(
            ext_self::ext(env::current_account_id()).on_activate(project_token_id, proxy_token_id, env::attached_deposit(), env::predecessor_account_id())
        )
    }

    /// Callback after project token was created
    #[private]
    pub fn on_activate(
        &mut self,
        project_token_id: AccountId,
        proxy_token_id: AccountId,
        attached_deposit: U128,
        predecessor_account_id: AccountId,
    ) -> PromiseOrValue<bool> {
        if is_promise_success() {
            self.project_token_id = Some(project_token_id);
            self.proxy_token_id = Some(proxy_token_id);
            PromiseOrValue::Value(true)
        } else {
            Promise::new(predecessor_account_id).transfer(attached_deposit.0);
            PromiseOrValue::Value(false)
        }
    }

    /// buy proxy token
    pub fn buy(&mut self, amount: Balance, coin_amount: Balance) -> Promise {
        self.assert_not_paused();
        self.assert_is_ongoing();

        let cal_coin_amount = self.calculate_buy_proxy_token(amount);
        assert!(coin_amount >= cal_coin_amount, "{}", ERR07_INSUFFICIENT_FUND);

        let treasury_fee_amount = cal_coin_amount
            .checked_mul(self.treasury_fee as u128)
            .unwrap()
            .checked_div(FEE_DIVISOR as u128)
            .unwrap();

        let reserve_fund_amount = cal_coin_amount.checked_sub(treasury_fee_amount).unwrap();

        self.total_fund_amount = self.total_fund_amount
            .checked_add(reserve_fund_amount)
            .unwrap();

        if self.tp_timestamp == 0 && self.total_fund_amount >= self.fund_threshold {
            self.tp_timestamp = env::block_timestamp();
        }

        // Transfer stable coin to customer
        ext_fungible_token::ext(self.stable_coin_id.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer(
                env::current_account_id(),
                U128::from(reserve_fund_amount),
                None,
            )
            .then(
                // Transfer stable coin to customer
                ext_fungible_token::ext(self.stable_coin_id.clone())
                    .with_static_gas(GAS_FOR_FT_TRANSFER)
                    .ft_transfer(
                        self.treasury_id.clone(),
                        U128::from(treasury_fee_amount),
                        None,
                    )
                    .then(
                        // Mint proxy token to customer
                        ext_proxy_token::ext(self.proxy_token_id.clone().unwrap())
                            .with_static_gas(Gas(5 * TGAS))
                            .mt_mint(
                                env::predecessor_account_id(),
                                amount,
                            )
                    )
            )
    }

    /// sell proxy token
    pub fn sell(&mut self, token_ids: Vec<TokenId>) -> Promise {
        self.assert_not_paused();
        self.assert_is_ongoing();

        let cal_coin_amount = self.calculate_sell_proxy_token(token_ids.clone());
        assert!(cal_coin_amount > 0, "{}", ERR09_INVALID_ACTION);

        self.total_fund_amount = self.total_fund_amount.checked_sub(cal_coin_amount).unwrap();

        // Transfer stable coin to customer
        ext_fungible_token::ext(self.stable_coin_id.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer(
                env::predecessor_account_id(),
                U128::from(cal_coin_amount),
                None,
            )
            .then(
                // Burn Proxy Token
                ext_proxy_token::ext(self.proxy_token_id.clone().unwrap())
                    .with_static_gas(Gas(5 * TGAS))
                    .mt_burn(
                        env::predecessor_account_id(),
                        token_ids,
                    )
            )
    }

    /// convert proxy token to real token
    pub fn convert(&mut self, token_ids: Vec<TokenId>) -> Promise {
        self.assert_not_paused();
        self.assert_is_after_buffer_period();

        let burn_proxy = ext_proxy_token::ext(self.proxy_token_id.clone().unwrap())
            .with_static_gas(Gas(5 * TGAS))
            .mt_burn(
                env::predecessor_account_id(),
                token_ids.clone(),
            );

        let convert_project_token = match self.is_closed {
            true => {
                self.internal_project_token_mint(env::predecessor_account_id(), token_ids.len() as u128)
            }
            false => {
                // owner
                self.internal_convert_transfer(env::predecessor_account_id(), token_ids.len() as u128)
            }
        };

        let convert_callback =
            ext_self::ext(env::current_account_id())
                .with_static_gas(Gas(5 * TGAS))
                .on_convert(token_ids.len() as Balance);

        convert_project_token.then(burn_proxy).then(convert_callback)
    }

    #[private]
    pub fn on_convert(&mut self, converted_amount: Balance, #[callback_result] call_result: Result<(), PromiseError>) {
        if call_result.is_err() {
            env::panic_str(ERR014_CONVERT_FAILED);
        }
        self.converted_proxy_token_amount = self.converted_proxy_token_amount.checked_add(converted_amount).unwrap();
    }

    /// claim fund
    pub fn claim_fund(&mut self, to: AccountId, amount: u128) -> Promise {
        self.assert_owner();
        self.assert_is_after_conversion_period();

        assert!(amount > 0 && self.total_fund_amount >= amount, "{}", ERR010_INVALID_AMOUNT);

        let finder_fee_amount = amount.checked_mul(self.finder_fee as u128).unwrap().checked_div(FEE_DIVISOR as u128).unwrap();

        let fund_transfer = ext_fungible_token::ext(self.stable_coin_id.clone())
            .with_static_gas(Gas(5 * TGAS))
            .ft_transfer(
                to,
                U128::from(amount - finder_fee_amount),
                None,
            );

        let finder_fee_transfer = ext_fungible_token::ext(self.stable_coin_id.clone())
            .with_static_gas(Gas(5 * TGAS))
            .ft_transfer(
                self.finder_id.clone().unwrap(),
                U128::from(finder_fee_amount),
                None,
            );

        fund_transfer.then(finder_fee_transfer)
    }

    /// close project
    pub fn close_project(&mut self) -> Promise {
        self.assert_owner();
        assert!(
            self.start_timestamp == 0 ||
                (self.tp_timestamp > 0 &&
                    env::block_timestamp() > self.tp_timestamp.checked_add(self.buffer_period).unwrap().checked_add(self.conversion_period).unwrap()),
            "{}",
            ERR011_NOT_AVAILABLE_TO_CLOSE
        );

        assert_eq!(self.is_closed, false, "{}", ERR013_ALREADY_CLOSED);

        let transfer_owner = match self.project_token_type {
            ProjectTokenType::Fungible => ext_fungible_token::ext(self.project_token_id.clone().unwrap())
                .with_static_gas(Gas(5 * TGAS))
                .set_owner(self.owner_id.clone()),
            ProjectTokenType::NonFungible => ext_nft_collection::ext(self.project_token_id.clone().unwrap())
                .with_static_gas(Gas(5 * TGAS))
                .set_owner(self.owner_id.clone())
        };

        let close_project_callback =
            ext_self::ext(env::current_account_id())
                .with_static_gas(Gas(5 * TGAS))
                .on_close_project();

        if self.pre_mint_amount > 0 {
            let pre_mint = self.internal_project_token_mint(self.owner_id.clone(), self.pre_mint_amount);
            let burn_batch = ext_proxy_token::ext(self.proxy_token_id.clone().unwrap())
                .with_static_gas(Gas(5 * TGAS))
                .mt_burn_with_amount(
                    self.owner_id.clone(),
                    "0".to_string(),
                    self.pre_mint_amount,
                );
            burn_batch.then(pre_mint).then(transfer_owner).then(close_project_callback)
        } else {
            transfer_owner.then(close_project_callback)
        }
    }

    #[private]
    pub fn on_close_project(&mut self, #[callback_result] call_result: Result<(), PromiseError>) {
        if call_result.is_err() {
            env::panic_str(ERR012_CLOSE_PROJECT_FAILED);
        }
        self.is_closed = true;
    }

    pub fn internal_project_token_mint(&mut self, to: AccountId, amount: u128) -> Promise {
        match self.project_token_type {
            ProjectTokenType::NonFungible => ext_nft_collection::ext(self.project_token_id.clone().unwrap())
                .with_static_gas(Gas(5 * TGAS))
                .nft_mint(
                    to,
                    amount,
                ),
            ProjectTokenType::Fungible => ext_fungible_token::ext(self.project_token_id.clone().unwrap())
                .with_static_gas(Gas(5 * TGAS))
                .ft_mint(
                    to,
                    amount,
                ),
        }
    }

    pub fn internal_convert_transfer(&mut self, to: AccountId, amount: u128) -> Promise {
        match self.project_token_type {
            ProjectTokenType::NonFungible => {
                let mut promise = ext_nft_collection::ext(self.project_token_id.clone().unwrap())
                    .with_static_gas(Gas(5 * TGAS))
                    .nft_transfer(
                        to.clone(),
                        (self.pre_mint_amount + self.converted_proxy_token_amount).to_string(),
                        None,
                        None,
                    );
                let mut id = 1;
                while id < amount {
                    let token_id: TokenId = (self.pre_mint_amount + self.converted_proxy_token_amount + id).to_string();
                    promise = promise.and(ext_nft_collection::ext(self.project_token_id.clone().unwrap())
                        .with_static_gas(Gas(5 * TGAS))
                        .nft_transfer(
                            to.clone(),
                            token_id,
                            None,
                            None,
                        ));
                    id += 1;
                }
                promise
            }
            ProjectTokenType::Fungible => ext_fungible_token::ext(self.project_token_id.clone().unwrap())
                .with_static_gas(Gas(5 * TGAS))
                .ft_transfer(
                    to,
                    U128::from(amount),
                    None,
                ),
        }
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
    fn test() {}
}
