use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{AccountId, env, PromiseOrValue};
use near_sdk::json_types::U128;

use crate::*;

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    #[allow(unreachable_code)]
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128> {
        self.assert_not_paused();
        self.assert_is_ongoing();

        let token_in = env::predecessor_account_id();
        let args = msg.split(":").collect::<Vec<&str>>();

        if args.len() == 2 && args[0] == "buy" && token_in == self.stable_coin_id {
            self.buy(sender_id, U128(args[1].parse::<u128>().unwrap()), amount);
        }

        PromiseOrValue::Value(U128(0))
    }
}