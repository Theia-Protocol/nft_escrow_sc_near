use near_sdk::{ext_contract, AccountId};

/// Fee divisor, allowing to provide fee in bps.
pub const FEE_DIVISOR: u32 = 10_000;

#[ext_contract(ext_self)]
pub trait SelfCallbacks {
    fn active_project_callback(&mut self, token_id: AccountId);
}

/// Newton's method of integer square root.
pub fn integer_sqrt(value: u128) -> u128 {
    let mut guess: u128 = (value + 1) >> 1;
    let mut res = value;
    while guess < res {
        res = guess;
        guess = (value / guess + guess) >> 1;
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt() {
        assert_eq!(integer_sqrt(0u128), 0.into());
        assert_eq!(integer_sqrt(4u128), 2.into());
        assert_eq!(
            integer_sqrt(1_516_156_330_329u128),
            1_231_323u128
        );
    }
}