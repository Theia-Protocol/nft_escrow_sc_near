/// Newton's method of integer square root.
pub fn integer_sqrt(value: U256) -> U256 {
    let mut guess: U256 = (value + U256::one()) >> 1;
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
        assert_eq!(integer_sqrt(U256::from(0)), 0.into());
        assert_eq!(integer_sqrt(U256::from(4)), 2.into());
        assert_eq!(
            integer_sqrt(U256::from(1_516_156_330_329u128)),
            U256::from(1_231_323)
        );
    }
}