//! Cost bounds for one expansion.
//!
//! A relationship graph is attacker-influenced: a tenant can author tuples and
//! a config can nest rewrites. Both walks are therefore bounded, and exceeding
//! a bound is a typed refusal rather than a truncated answer — a truncated
//! authorization result is a wrong one, not a partial one.

/// Bounds applied to a single check or expand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpansionBounds {
    /// Maximum rewrite nesting depth before refusing.
    pub max_depth: u32,
    /// Maximum tuples read across the whole walk before refusing.
    pub max_tuples_read: usize,
    /// Maximum pages read for any one tupleset before refusing.
    pub max_pages_per_tupleset: usize,
}

impl ExpansionBounds {
    /// Bounds that comfortably admit ordinary object hierarchies while still
    /// refusing a pathological config or tuple set.
    pub const DEFAULT: Self = Self {
        max_depth: 32,
        max_tuples_read: 10_000,
        max_pages_per_tupleset: 64,
    };
}

impl Default for ExpansionBounds {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Tracks spend against [`ExpansionBounds`] for one walk.
#[derive(Debug)]
pub(crate) struct Budget {
    tuples_read: usize,
    limit: usize,
}

impl Budget {
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            tuples_read: 0,
            limit,
        }
    }

    /// Charge `count` tuples, refusing once the bound is passed.
    pub(crate) fn charge(&mut self, count: usize) -> Result<(), crate::ExpansionError> {
        self.tuples_read = self.tuples_read.saturating_add(count);
        if self.tuples_read > self.limit {
            return Err(crate::ExpansionError::TupleBudgetExceeded { limit: self.limit });
        }
        Ok(())
    }
}
