use crate::*;

#[near_bindgen]
impl Contract {
    pub(crate) fn assert_is_ongoing(&self) {
        assert!(self.start_timestamp > 0, "{}", ERR10_NOT_ACTIVATED);
        assert!(
            self.tp_timestamp == 0 ||
                env::block_timestamp() < self.tp_timestamp.checked_add(self.buffer_period).unwrap().checked_add(self.conversion_period).unwrap(),
            "{}",
            ERR11_NOT_ONGOING
        );
    }

    pub(crate) fn assert_is_after_buffer_period(&self) {
        assert!(self.tp_timestamp > 0, "{}", ERR12_NOT_OVER_FUNDING_TARGET);
        assert!(env::block_timestamp() >= self.tp_timestamp.checked_add(self.buffer_period).unwrap(), "{}", ERR13_IN_BUFFER_PERIOD);
    }

    pub(crate) fn assert_is_after_conversion_period(&self) {
        assert!(self.tp_timestamp > 0, "{}", ERR12_NOT_OVER_FUNDING_TARGET);
        assert!(env::block_timestamp() > self.tp_timestamp.checked_add(self.buffer_period).unwrap().checked_add(self.conversion_period).unwrap(), "{}", ERR14_NOT_OVER_CONVERSION_PERIOD);
    }
}