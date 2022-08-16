use near_contract_standards::non_fungible_token::TokenId;
use crate::*;
use crate::utils::{FEE_DIVISOR, integer_sqrt};

#[near_bindgen]
impl Contract {
    #[private]
    pub fn get_sum_price(&self, to_token_id: u128) -> u128 {
        let one_coin = 10u128.pow(self.stable_coin_decimals as u32);
        let mut price;
        match &self.curve_type {
            CurveType::Horizontal => {
                // p = A * x
                price = (&self).curve_args.arg_a
                    .unwrap()
                    .checked_mul(to_token_id)
                    .unwrap()
                    .checked_mul(one_coin)
                    .unwrap();
            },
            CurveType::Linear=> {
                // p = A * x^2 / 2 + B * x
                price = (&self).curve_args.arg_a
                    .unwrap()
                    .checked_mul(to_token_id * to_token_id)
                    .unwrap()
                    .checked_div(2)
                    .unwrap();
                price = price
                    .checked_add((&self).curve_args.arg_b
                        .unwrap()
                        .checked_mul(to_token_id).unwrap())
                        .unwrap();
                price = price
                    .checked_mul(one_coin)
                    .unwrap();
            },
            CurveType::Sigmoidal => {
                // p = A * sqr(C + (x + B)^2) + x * (D + A)
                let aa = integer_sqrt(
                    (
                        (&self).curve_args.arg_c.unwrap()
                            .checked_add(
                                to_token_id
                                    .checked_add(
                                        (&self).curve_args.arg_b.unwrap()
                                    )
                                    .unwrap()
                                    .checked_pow(2)
                                    .unwrap()
                            )
                            .unwrap()
                    )
                    .checked_mul(
                        one_coin.checked_pow(2).unwrap()
                    )
                    .unwrap()
                );
                price = (&self).curve_args.arg_a
                    .unwrap()
                    .checked_mul(aa)
                    .unwrap();
                price = price
                    .checked_add(
                    to_token_id.checked_mul(
                        (&self).curve_args.arg_d.unwrap()
                                .checked_add(
                                    (&self).curve_args.arg_a.unwrap()
                                )
                                .unwrap()
                        )
                        .unwrap()
                    )
                    .unwrap();
            }
        }
        price
    }

    pub fn get_token_price(&self, token_id: u128) -> u128 {
        self.get_sum_price(token_id.checked_add(1).unwrap())
            .checked_sub(self.get_sum_price(token_id))
            .unwrap()
    }

    pub fn calculate_buy_proxy_token(&self, amount: Balance) -> u128 {
        let circulating_supply = 0u128;

        self.get_sum_price(circulating_supply.checked_add(amount).unwrap())
            .checked_sub(self.get_sum_price(circulating_supply))
            .unwrap()
    }

    pub fn calculate_sell_proxy_token(&self, token_ids: Vec<TokenId>) -> u128 {
        let mut total_price = 0u128;
        token_ids.iter().enumerate().for_each(|(_, token_id) | {
            total_price = total_price.checked_add(self.get_token_price(token_id.parse().unwrap())).unwrap();
        });

        total_price
            .checked_mul(FEE_DIVISOR.checked_sub(self.treasury_fee).unwrap() as u128)
            .unwrap()
            .checked_div(FEE_DIVISOR as u128).unwrap()
    }
}