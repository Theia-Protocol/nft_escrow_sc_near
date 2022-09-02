use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_project_token_type(&self) -> ProjectTokenType { self.project_token_type.clone() }

    pub fn get_project_token_id(&self) -> AccountId { self.project_token_id.clone().unwrap() }

    pub fn get_proxy_token_id(&self) -> AccountId { self.proxy_token_id.clone().unwrap() }

    pub fn get_finder_fee(&self) -> u32 { self.finder_fee }

    pub fn get_finder_id(&self) -> AccountId { self.finder_id.clone().unwrap() }

    pub fn get_treasury_fee(&self) -> u32 { self.treasury_fee }

    pub fn get_treasury_id(&self) -> AccountId { self.treasury_id.clone() }

    pub fn get_fund_threshold(&self) -> Balance { self.fund_threshold }

    pub fn get_total_fund_amount(&self) -> Balance { self.total_fund_amount }

    pub fn get_pre_mint_amount(&self) -> Balance { self.pre_mint_amount }

    pub fn get_start_timestamp(&self) -> u64 { self.start_timestamp }

    pub fn get_tp_timestamp(&self) -> u64 { self.tp_timestamp }

    pub fn get_buffer_period(&self) -> u64 { self.buffer_period }

    pub fn get_conversion_period(&self) -> u64 { self.conversion_period }

    pub fn get_stable_coin_id(&self) -> AccountId { self.stable_coin_id.clone() }

    pub fn get_running_state(&self) -> RunningState { self.state.clone() }

    pub fn get_is_closed(&self) -> bool { self.is_closed }

    pub fn get_converted_amount(&self) -> Balance { self.converted_amount }

    pub fn get_circulating_supply(&self) -> Balance { self.circulating_supply }
}