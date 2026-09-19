//! Typed option and enum codes from `fdb.options` (release 7.3.79). Each
//! discriminant is the wire value the C API takes.

/// Resolves to a key relative to `key` (FDB key selectors).
#[derive(Clone, Copy, Debug)]
pub struct KeySelector<'a> {
    pub key: &'a [u8],
    pub or_equal: bool,
    pub offset: i32,
}

impl<'a> KeySelector<'a> {
    pub const fn new(key: &'a [u8], or_equal: bool, offset: i32) -> Self {
        Self {
            key,
            or_equal,
            offset,
        }
    }
    pub const fn first_greater_or_equal(key: &'a [u8]) -> Self {
        Self::new(key, false, 1)
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StreamingMode {
    #[default]
    WantAll = -2,
    Iterator = -1,
    Exact = 0,
    Serial = 4,
}

/// Paging knobs for `get_range`; zero `limit`/`target_bytes` mean unlimited.
#[derive(Clone, Copy, Debug, Default)]
pub struct RangeOptions {
    pub limit: i32,
    pub target_bytes: i32,
    pub mode: StreamingMode,
    pub iteration: i32,
    pub snapshot: bool,
    pub reverse: bool,
}

/// `fdb_error_predicate` tests.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorPredicate {
    Retryable = 50000,
    MaybeCommitted = 50001,
    RetryableNotCommitted = 50002,
}
