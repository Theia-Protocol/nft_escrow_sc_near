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
                    U256::from(
                        (&self).curve_args.arg_c.unwrap() + (to_token_id + (&self).curve_args.arg_b.unwrap()).pow(2)
                    )
                    * U256::from(one_coin)
                    * U256::from(one_coin)
                );
                price = (&self).curve_args.arg_a
                    .unwrap()
                    .checked_mul(aa.as_u128())
                    .unwrap();
                price = price
                    .checked_add(
                    to_token_id.checked_mul(
                        (&self).curve_args.arg_d.unwrap() + (&self).curve_args.arg_a.unwrap()
                        )
                        .unwrap()
                        * one_coin
                    )
                    .unwrap();
            }
        }
        price
    }

    pub fn get_curve_type(&self) -> CurveType { self.curve_type.clone() }

    pub fn get_curve_args(&self) -> CurveArgs { self.curve_args.clone() }

    pub fn get_token_price(&self, token_id: U128) -> u128 {
        if token_id.0 < self.pre_mint_amount {
            return 0u128;
        }
        let token_index = token_id.0 - self.pre_mint_amount;
        self.get_sum_price(token_index + 1)
            .checked_sub(self.get_sum_price(token_index))
            .unwrap()
    }

    pub fn calculate_buy_proxy_token(&self, amount: U128) -> u128 {
        self.get_sum_price(self.circulating_supply.checked_add(amount.0).unwrap())
            .checked_sub(self.get_sum_price(self.circulating_supply))
            .unwrap()
    }

    pub fn calculate_sell_proxy_token(&self, token_ids: Vec<TokenId>) -> u128 {
        let mut total_price = 0u128;
        token_ids.iter().enumerate().for_each(|(_, token_id) | {
            total_price = total_price.checked_add(self.get_token_price(U128::from(token_id.parse::<u128>().unwrap()))).unwrap();
        });

        total_price
            .checked_mul(FEE_DIVISOR.checked_sub(self.treasury_fee).unwrap() as u128)
            .unwrap()
            .checked_div(FEE_DIVISOR as u128).unwrap()
    }
}