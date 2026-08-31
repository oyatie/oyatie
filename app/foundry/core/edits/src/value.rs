//! The spine-owned typed value mirror: what a property value is on the
//! wire, independent of any kernel crate's in-memory carrier.

use std::collections::BTreeMap;

/// Why a wire value was refused at construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WireValueError {
    NonFiniteDouble,
    InvalidDate,
}

/// A finite IEEE double stored as its monotone order-preserving u64 key
/// (negatives: `!bits`; non-negatives: `bits ^ (1 << 63)`), so the DERIVED
/// `Eq`/`Ord`/`Hash` agree with IEEE numeric order and the key doubles as
/// a sort key. `NaN` and the infinities are rejected at construction;
/// `-0.0` folds to `0.0`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WireDouble(u64);

impl WireDouble {
    pub fn new(value: f64) -> Result<Self, WireValueError> {
        if !value.is_finite() {
            return Err(WireValueError::NonFiniteDouble);
        }
        let canonical = if value == 0.0 { 0.0 } else { value };
        let bits = canonical.to_bits();
        let key = if canonical.is_sign_negative() {
            !bits
        } else {
            bits ^ (1 << 63)
        };
        Ok(Self(key))
    }

    pub fn get(self) -> f64 {
        if self.0 & (1 << 63) == 0 {
            f64::from_bits(!self.0)
        } else {
            f64::from_bits(self.0 ^ (1 << 63))
        }
    }

    /// The monotone key itself — the future index sort key.
    pub const fn sort_key(self) -> u64 {
        self.0
    }
}

/// A validated proleptic-Gregorian calendar date.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WireDate {
    year: i32,
    month: u8,
    day: u8,
}

impl WireDate {
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, WireValueError> {
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
            return Err(WireValueError::InvalidDate);
        }
        Ok(Self { year, month, day })
    }

    pub const fn year(self) -> i32 {
        self.year
    }

    pub const fn month(self) -> u8 {
        self.month
    }

    pub const fn day(self) -> u8 {
        self.day
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// A typed property value as it appears on the wire. Mirrors the kernel's
/// typed carrier shape by construction, never by dependency.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WireValue {
    String(String),
    Integer(i64),
    Double(WireDouble),
    Boolean(bool),
    Date(WireDate),
    Timestamp { epoch_millis: i64 },
    Array(Vec<WireValue>),
    Struct(BTreeMap<String, WireValue>),
}
