mod utils;
mod errors;
mod views;
mod curves;
mod owner;
mod validates;
mod pause;
mod token_receiver;
mod pt_metadata;
mod proxy_token;

use near_contract_standards::non_fungible_token::TokenId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::STORAGE_PRICE_PER_BYTE;
use near_sdk::{env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, Promise, Gas, log, is_promise_success, PromiseOrValue};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use crate::pt_metadata::*;
use crate::errors::*;
use crate::utils::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Owner of contract
    owner_id: AccountId,
    /// Project token name
    name: String,
    /// Project token symbol
    symbol: String,
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
    /// Total claimed fund amount
    claimed_fund_amount: Balance,
    /// Total claimed finder fee amount
    claimed_finder_fee: Balance,
    /// Pre-mint amount
    pre_mint_amount: Balance,
    /// Amount of converted proxy token
    converted_amount: Balance,
    /// Circulating supply of proxy token
    circulating_supply: Balance,
    /// Auction curve type
    curve_type: CurveType,
    /// Auction curve args
    curve_args: CurveArgs,
    /// Running state
    state: RunningState,
    /// Closed
    closed_step: ClosedStep,
    /// Proxy token media uri
    pt_media_uri: String,
    /// Proxy token max supply
    pt_max_supply: u128,
    /// Proxy token all total supply
    pt_all_total_supply: Balance,
    /// Proxy token total supply by token id
    pt_total_supply: LookupMap<TokenId, Balance>,
    /// Proxy token balance by token id and account id
    pt_balances_per_token: UnorderedMap<TokenId, LookupMap<AccountId, Balance>>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    TotalSupply { supply: u128 },
    Balances,
    BalancesInner { token_id: Vec<u8> },
}

const MIN_STORAGE_NON_FUNGIBLE_TOKEN: Balance = 600_000 * STORAGE_PRICE_PER_BYTE;
const MIN_STORAGE_FUNGIBLE_TOKEN: Balance = 600_000 * STORAGE_PRICE_PER_BYTE;
const DEPOSIT_ONE_NFT_MINT: Balance = 638 * STORAGE_PRICE_PER_BYTE;
const DEPOSIT_ONE_PT_MINT: Balance = 415 * STORAGE_PRICE_PER_BYTE;
const NO_DEPOSIT: Balance = 0u128;
const ONE_YOCTO: Balance = 1u128;
const TGAS: u64 = 1_000_000_000_000;
const GAS_FOR_PT_MINT: Gas = Gas(100 * TGAS);

const NFT_COLLECTION_CODE: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/nft_collection.wasm");
const FUNGIBLE_TOKEN_CODE: &[u8] = include_bytes!("../../target/wasm32-unknown-unknown/release/ft_token.wasm");

#[near_bindgen]
impl Contract {
    /// Initialize the contract
    #[init]
    pub fn new(owner_id: AccountId, name: String, symbol: String, pt_media_uri: String, stable_coin_id: AccountId, stable_coin_decimals: u8, curve_type: CurveType, curve_args: CurveArgs, treasury_id: AccountId) -> Self {
        assert!(!env::state_exists(), "{}", ERR08_ALREADY_INITIALIZED);
        assert!(name.len() > 2, "{}", ERR00_INVALID_NAME);
        assert!(symbol.len() < 13 && symbol.len() > 2, "{}", ERR01_INVALID_SYMBOL);
        assert!(pt_media_uri.len() > 0, "{}", ERR03_INVALID_PT_MEDIA_URI);

        Self {
            owner_id,
            name,
            symbol,
            treasury_id,
            treasury_fee: 100,  // 1%
            finder_id: None,
            finder_fee: 100,    // 1%
            project_token_type: ProjectTokenType::NonFungible,
            project_token_id: None,
            fund_threshold: 0,
            start_timestamp: 0,
            tp_timestamp: 0,
            buffer_period: 0,
            conversion_period: 0,
            stable_coin_id,
            stable_coin_decimals,
            total_fund_amount: 0,
            claimed_fund_amount: 0,
            claimed_finder_fee: 0,
            pre_mint_amount: 0,
            converted_amount: 0,
            circulating_supply: 0,
            curve_type,
            curve_args,
            state: RunningState::Running,
            closed_step: ClosedStep::None,
            pt_media_uri,
            pt_total_supply: LookupMap::new(StorageKey::TotalSupply { supply: u128::MAX }),
            pt_balances_per_token: UnorderedMap::new(StorageKey::Balances),
            pt_max_supply: 0,
            pt_all_total_supply: 0
        }
    }

    /// Pre-mint
    #[payable]
    pub fn pre_mint(&mut self, amount: U128) {
        self.assert_owner();
        assert!(amount.0 > 0, "{}", ERR010_INVALID_AMOUNT);
        assert_eq!(self.start_timestamp, 0, "{}", ERR15_ALREADY_ACTIVATED);

        self.pre_mint_amount = self.pre_mint_amount.checked_add(amount.0).unwrap();

        self.pt_mint(self.owner_id.clone(), amount);

        log!("Pre-mint {}", amount.0);
    }

    /// Active NFT project
    pub fn active_nft_project(&mut self, base_uri: String, max_supply: U128, finder_id: AccountId, fund_threshold: U128, buffer_period: u64, conversion_period: u64) -> Promise {
        self.assert_owner();
        assert!(self.closed_step == ClosedStep::None, "{}", ERR012_ALREADY_CLOSED);
        assert!(base_uri.len() > 0, "{}", ERR02_INVALID_COLLECTION_BASE_URI);
        assert!(max_supply.0 > 0 && self.pre_mint_amount < max_supply.0, "{}", ERR04_INVALID_MAX_SUPPLY);
        assert!(fund_threshold.0 > 0, "{}", ERR05_INVALID_FUNDING_TARGET);
        assert!(conversion_period >= 86400, "{}", ERR06_INVALID_CONVERSION_PERIOD);

        self.finder_id = Some(finder_id);
        self.fund_threshold = fund_threshold.0;
        self.buffer_period = buffer_period;
        self.conversion_period = conversion_period;
        self.project_token_type = ProjectTokenType::NonFungible;
        self.pt_max_supply = max_supply.0;

        let mut token_suffix = self.name.clone().to_lowercase();
        token_suffix.retain(|c| !c.is_whitespace());
        let project_token_id = AccountId::new_unchecked(format!("{}.{}", token_suffix, env::current_account_id()));

        // deploy non-fungible token
        let project_token_promise = Promise::new(project_token_id.clone())
            .create_account()
            .transfer(MIN_STORAGE_NON_FUNGIBLE_TOKEN)
            .deploy_contract(NFT_COLLECTION_CODE.to_vec())
            .function_call(
                "new".to_string(),
                json!({
                    "owner_id": env::current_account_id(),
                    "name": self.name.clone(),
                    "symbol": self.symbol.clone(),
                    "base_uri": base_uri,
                    "max_supply": max_supply
                }).to_string().as_bytes().to_vec(),
                NO_DEPOSIT,
                Gas(5 * TGAS)
            );

        project_token_promise
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * TGAS))
                    .on_activate(project_token_id)
            )
    }

    /// Active FT project
    pub fn active_ft_project(&mut self, max_supply: U128, finder_id: AccountId, fund_threshold: U128, buffer_period: u64, conversion_period: u64) -> Promise {
        self.assert_owner();
        assert!(self.closed_step == ClosedStep::None, "{}", ERR012_ALREADY_CLOSED);
        assert!(max_supply.0 > 0 && self.pre_mint_amount < max_supply.0, "{}", ERR04_INVALID_MAX_SUPPLY);
        assert!(fund_threshold.0 > 0, "{}", ERR05_INVALID_FUNDING_TARGET);
        assert!(conversion_period >= 86400, "{}", ERR06_INVALID_CONVERSION_PERIOD);

        self.finder_id = Some(finder_id);
        self.fund_threshold = fund_threshold.0;
        self.buffer_period = buffer_period;
        self.conversion_period = conversion_period;
        self.project_token_type = ProjectTokenType::Fungible;
        self.pt_max_supply = max_supply.0;

        let mut token_suffix = self.name.clone().to_lowercase();
        token_suffix.retain(|c| !c.is_whitespace());
        let project_token_id = AccountId::new_unchecked(format!("{}.{}", token_suffix, env::current_account_id()));

        // deploy fungible token
        let project_token_promise = Promise::new(project_token_id.clone())
            .create_account()
            .transfer(MIN_STORAGE_FUNGIBLE_TOKEN)
            .deploy_contract(FUNGIBLE_TOKEN_CODE.to_vec())
            .function_call(
                "new".to_string(),
                json!({
                    "owner_id": env::current_account_id(),
                    "name": self.name.clone(),
                    "symbol": self.symbol.clone(),
                    "decimals": 1u8
                }).to_string().as_bytes().to_vec(),
                NO_DEPOSIT,
                Gas(5 * TGAS)
            );

        project_token_promise.then(
            ext_self::ext(env::current_account_id())
                .with_static_gas(Gas(5 * TGAS))
                .on_activate(project_token_id)
        )
    }

    /// Callback after project token was created
    #[private]
    pub fn on_activate(
        &mut self,
        project_token_id: AccountId
    ) -> bool {
        if is_promise_success() {
            self.project_token_id = Some(project_token_id.clone());
            self.start_timestamp = env::block_timestamp();

            log!("Activated {} {}", project_token_id.to_string(), self.start_timestamp);
        } else {
            return false;
        }

        true
    }

    /// buy proxy token
    pub(crate) fn buy(&mut self, from: AccountId, amount: U128, deposit: U128) -> Promise {
        self.assert_not_paused();
        self.assert_is_ongoing();
        assert!(amount.0 > 0, "Invalid amount");
        assert!(self.pt_all_total_supply + amount.0 < self.pt_max_supply, "OverMaxSupply");

        let cal_coin_amount = self.calculate_buy_proxy_token(amount);
        assert!(deposit.0 >= cal_coin_amount, "{}", ERR07_INSUFFICIENT_FUND);

        // Mint proxy token to customer
        let mint_promise = ext_self::ext(env::current_account_id())
            .with_static_gas(GAS_FOR_PT_MINT)
            .with_attached_deposit(amount.0 * DEPOSIT_ONE_PT_MINT)
            .pt_mint(from.clone(), amount);

        mint_promise.then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * TGAS))
                    .on_buy(from, amount, deposit, U128(cal_coin_amount))
            )
    }

    #[private]
    #[payable]
    pub fn on_buy(&mut self, from: AccountId, amount: U128, deposit: U128, reserve: U128) -> bool {
        if is_promise_success() {
            let treasury_fee_amount = reserve.0
                .checked_mul(self.treasury_fee as u128)
                .unwrap()
                .checked_div(FEE_DIVISOR as u128)
                .unwrap();

            let reserve_fund_amount = reserve.0.checked_sub(treasury_fee_amount).unwrap();

            self.total_fund_amount = self.total_fund_amount
                .checked_add(reserve_fund_amount)
                .unwrap();

            if self.tp_timestamp == 0 && self.total_fund_amount >= self.fund_threshold {
                self.tp_timestamp = env::block_timestamp();
            }
            // update circulating supply
            self.circulating_supply += amount.0;

            // Transfer stable coin to treasury
            ext_fungible_token::ext(self.stable_coin_id.clone())
                .with_static_gas(GAS_FOR_FT_TRANSFER)
                .with_attached_deposit(ONE_YOCTO)
                .ft_transfer(
                    self.treasury_id.clone(),
                    U128::from(treasury_fee_amount),
                    None,
                );

            let remain = deposit.0 - reserve.0;
            if remain > 0 {
                ext_fungible_token::ext(self.stable_coin_id.clone())
                    .with_static_gas(GAS_FOR_FT_TRANSFER)
                    .with_attached_deposit(ONE_YOCTO)
                    .ft_transfer(
                        from.clone(),
                        U128(remain),
                        None,
                    );
            }

            log!("Buy {} {} {}", from, amount.0, reserve.0);
            true
        } else {
            ext_fungible_token::ext(self.stable_coin_id.clone())
                .with_static_gas(GAS_FOR_FT_TRANSFER)
                .with_attached_deposit(ONE_YOCTO)
                .ft_transfer(
                    from.clone(),
                    deposit,
                    None,
                );
            false
        }
    }

    /// sell proxy token
    pub fn sell(&mut self, token_ids: Vec<TokenId>) -> Promise {
        self.assert_not_paused();
        self.assert_is_ongoing();

        let cal_coin_amount = self.calculate_sell_proxy_token(token_ids.clone());
        assert!(cal_coin_amount > 0, "{}", ERR09_INVALID_ACTION);

        // Burn Proxy Token
        self.pt_burn(
            env::predecessor_account_id(),
            token_ids.clone(),
        );

        // Transfer stable coin to customer
        ext_fungible_token::ext(self.stable_coin_id.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(ONE_YOCTO)
            .ft_transfer(
                env::predecessor_account_id(),
                U128(cal_coin_amount),
                None,
            )
            .then(
                ext_self::ext(env::current_account_id())
                .with_static_gas(Gas(5 * TGAS))
                .on_sell(env::predecessor_account_id(), U128(cal_coin_amount), token_ids)
            )
    }

    #[private]
    pub fn on_sell(&mut self, from: AccountId, refund: U128, token_ids: Vec<TokenId>) -> bool {
        if is_promise_success() {
            self.total_fund_amount = self.total_fund_amount.checked_sub(refund.0).unwrap();
            // update circulating supply
            self.circulating_supply -= token_ids.len() as u128;

            log!("Sell {} [{}] {}", from, token_ids.join(","), refund.0);
            true
        } else {
            self.revert_pt_burn(from, token_ids);

            false
        }
    }

    /// convert proxy token to real token
    pub fn convert(&mut self, token_ids: Vec<TokenId>) -> Promise {
        self.assert_not_paused();
        self.assert_is_after_buffer_period();

        let convert_project_token;
        if self.closed_step >= ClosedStep::RemainProxy {
            convert_project_token = self.internal_convert_transfer(env::predecessor_account_id(), token_ids.len() as u128)
        } else {
            convert_project_token = self.internal_project_token_mint(env::predecessor_account_id(), U128::from(token_ids.len() as u128))
        };

        self.pt_burn(env::predecessor_account_id(), token_ids.clone());

        convert_project_token
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * TGAS))
                    .on_convert(env::predecessor_account_id(), token_ids)
            )
    }

    #[private]
    pub fn on_convert(&mut self, from: AccountId, token_ids: Vec<TokenId>) -> bool {
        if !is_promise_success() {
            self.revert_pt_burn(from.clone(), token_ids.clone());
            return false;
        }
        
        self.converted_amount = self.converted_amount.checked_add(token_ids.len() as u128).unwrap();

        log!("Convert {} {}", from, token_ids.join(","));
        true
    }

    /// claim fund
    pub fn claim_fund(&mut self, to: AccountId, amount: U128) -> Promise {
        self.assert_owner();
        self.assert_is_after_conversion_period();

        let total_finder_fee = self.total_fund_amount.checked_mul(self.finder_fee as u128).unwrap().checked_div(FEE_DIVISOR as u128).unwrap();
        let total_claimable_fund = self.total_fund_amount.checked_sub(total_finder_fee).unwrap();
        assert!(amount.0 > 0 && (total_claimable_fund - self.claimed_fund_amount) >= amount.0, "{}", ERR010_INVALID_AMOUNT);

        ext_fungible_token::ext(self.stable_coin_id.clone())
            .with_static_gas(Gas(5 * TGAS))
            .with_attached_deposit(ONE_YOCTO)
            .ft_transfer(
                to,
                U128::from(amount.0),
                None,
            )
            .then(
                ext_self::ext(env::current_account_id())
                        .with_static_gas(Gas(5 * TGAS))
                        .on_claim_fund(amount)
            )
    }

    #[private]
    pub fn on_claim_fund(&mut self, amount: U128) -> bool {
        if is_promise_success() {
            self.claimed_fund_amount = self.claimed_fund_amount + amount.0;
            return true;
        }

        false
    }

    /// claim finder fee
    pub fn claim_finder_fee(&mut self, amount: U128) -> Promise {
        self.assert_owner();
        self.assert_is_after_conversion_period();

        let total_finder_fee = self.total_fund_amount.checked_mul(self.finder_fee as u128).unwrap().checked_div(FEE_DIVISOR as u128).unwrap();
        assert!(amount.0 > 0 && (total_finder_fee - self.claimed_finder_fee) >= amount.0, "{}", ERR010_INVALID_AMOUNT);

       ext_fungible_token::ext(self.stable_coin_id.clone())
            .with_static_gas(Gas(5 * TGAS))
            .with_attached_deposit(ONE_YOCTO)
            .ft_transfer(
                self.finder_id.clone().unwrap(),
                U128::from(amount.0),
                None,
            )
           .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * TGAS))
                    .on_claim_finder_fee(amount)
            )
    }

    #[private]
    pub fn on_claim_finder_fee(&mut self, amount: U128) -> bool {
        if is_promise_success() {
            self.claimed_finder_fee = self.claimed_finder_fee + amount.0;
            return true;
        }

        false
    }

    /// close project 1-step pre-mint
    pub fn close_project(&mut self) -> PromiseOrValue<bool> {
        self.assert_owner();
        assert!(
            self.start_timestamp == 0 ||
                (self.tp_timestamp > 0 &&
                    env::block_timestamp() > self.tp_timestamp.checked_add(self.buffer_period).unwrap().checked_add(self.conversion_period).unwrap()),
            "{}",
            ERR011_NOT_AVAILABLE_TO_CLOSE
        );

        let close_promise: Option<Promise> = match self.closed_step {
            ClosedStep::None => {
                if self.pre_mint_amount > 0 {
                    let token_ids = (0..self.pre_mint_amount - 1).enumerate().map(|(_, token_id)| { token_id.to_string() }).collect();
                    self.pt_burn(self.owner_id.clone(), token_ids);

                    Some(self.internal_project_token_mint(self.owner_id.clone(), U128::from(self.pre_mint_amount)))
                } else {
                    None
                }
            }
            ClosedStep::PreMint => {
                let remain_proxys = self.circulating_supply.checked_sub(self.converted_amount).unwrap();
                if remain_proxys > 0 {
                    Some(self.internal_project_token_mint(env::current_account_id(), U128::from(remain_proxys)))
                } else {
                    None
                }
            }
            ClosedStep::RemainProxy => {
                Some(match self.project_token_type {
                        ProjectTokenType::Fungible =>
                            ext_fungible_token::ext(self.project_token_id.clone().unwrap())
                                .with_static_gas(Gas(5 * TGAS))
                                .set_owner(self.owner_id.clone()),
                        ProjectTokenType::NonFungible => ext_nft_collection::ext(self.project_token_id.clone().unwrap())
                            .with_static_gas(Gas(5 * TGAS))
                            .set_owner(self.owner_id.clone())
                })
            }
            ClosedStep::TransferOwnership => {
                env::panic_str(ERR012_ALREADY_CLOSED);
            }
        };

        return if close_promise.is_none() {
            self.closed_step = self.closed_step.increase();
            PromiseOrValue::Value(true)
        } else {
            PromiseOrValue::Promise(
                close_promise.unwrap()
                    .then(
                        ext_self::ext(env::current_account_id())
                            .with_static_gas(Gas(5 * TGAS))
                            .on_close_project()
                    )
            )
        }
    }

    pub fn on_close_project(&mut self) -> bool {
        if is_promise_success() {
            self.closed_step = self.closed_step.increase();
            return true;
        }

        if self.closed_step == ClosedStep::None {
            let token_ids = (0..self.pre_mint_amount - 1).enumerate().map(|(_, token_id)| { token_id.to_string() }).collect();
            self.revert_pt_burn(self.owner_id.clone(), token_ids);
        }

        return false;
    }

    pub fn internal_project_token_mint(&mut self, to: AccountId, amount: U128) -> Promise {
        match self.project_token_type {
            ProjectTokenType::NonFungible => ext_nft_collection::ext(self.project_token_id.clone().unwrap())
                .with_static_gas(Gas(100 * TGAS))
                .with_attached_deposit(DEPOSIT_ONE_NFT_MINT * amount.0)
                .nft_mint(
                    to,
                    amount,
                ),
            ProjectTokenType::Fungible => ext_fungible_token::ext(self.project_token_id.clone().unwrap())
                .with_static_gas(Gas(100 * TGAS))
                .ft_mint(
                    to,
                    amount,
                ),
        }
    }

    pub fn internal_convert_transfer(&mut self, to: AccountId, amount: u128) -> Promise {
        match self.project_token_type {
            ProjectTokenType::NonFungible => {
                let token_ids: Vec<TokenId> = (0..amount-1).enumerate().map(|(_, id)| {
                    return (self.pre_mint_amount + self.converted_amount + id).to_string();
                }).collect();
                ext_nft_collection::ext(self.project_token_id.clone().unwrap())
                    .with_static_gas(Gas(5 * TGAS))
                    .with_attached_deposit(ONE_YOCTO)
                    .nft_batch_transfer(
                        to.clone(),
                        token_ids,
                        Some("".to_string()),
                    )
            }
            ProjectTokenType::Fungible => ext_fungible_token::ext(self.project_token_id.clone().unwrap())
                .with_static_gas(Gas(5 * TGAS))
                .with_attached_deposit(ONE_YOCTO)
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