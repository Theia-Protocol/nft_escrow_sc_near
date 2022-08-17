// Escrow errors.
pub const ERR00_INVALID_NAME: &str = "E00: Invalid name";
pub const ERR01_INVALID_SYMBOL: &str = "E01: Invalid symbol";
pub const ERR02_INVALID_COLLECTION_BASE_URI: &str = "E02: Invalid collection base uri";
pub const ERR03_INVALID_BLANK_URI: &str = "E03: Invalid blank uri";
pub const ERR04_INVALID_MAX_SUPPLY: &str = "E04: Invalid max supply";
pub const ERR05_INVALID_FUNDING_TARGET: &str = "E05: Invalid funding target";
pub const ERR06_INVALID_CONVERSION_PERIOD: &str = "E06: Invalid conversion period (min: 1 day)";
pub const ERR07_INSUFFICIENT_FUND: &str = "E07: Insufficient fund";
pub const ERR08_ALREADY_INITIALIZED: &str = "E08: Already initialized";
pub const ERR09_INVALID_ACTION: &str = "E09: Invalid action";
pub const ERR010_INVALID_AMOUNT: &str = "E10: Invalid amount";
pub const ERR011_NOT_AVAILABLE_TO_CLOSE: &str = "E11: Invalid amount";
pub const ERR012_CLOSE_PROJECT_FAILED: &str = "E12: Closing project failed";
pub const ERR013_ALREADY_CLOSED: &str = "E13: Project was already closed";
pub const ERR014_CONVERT_FAILED: &str = "E14: Convert failed";

// Validate errors
pub const ERR10_NOT_ACTIVATED: &str = "E10: Escrow is not activated";
pub const ERR11_NOT_ONGOING: &str = "E11: Escrow is not in ongoing state";
pub const ERR12_NOT_OVER_FUNDING_TARGET: &str = "E12: Escrow is not reached to funding threshold";
pub const ERR13_IN_BUFFER_PERIOD: &str = "E13: Escrow is in buffer period";
pub const ERR14_OVER_CONVERSION_PERIOD: &str = "E14: Escrow is outside of conversion period";
pub const ERR15_NOT_OVER_CONVERSION_PERIOD: &str = "E15: Escrow is not over conversion period";

// Owner errors
pub const ERR20_NOT_ALLOW: &str = "E20: The action is allowed by only owner";

// Pause errors
pub const ERR30_PAUSED: &str = "E20: Escrow was paused";
