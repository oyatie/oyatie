//! Cost bounds for one expansion.
//!
//! A relationship graph is attacker-influenced: a tenant can author tuples and
//! a config can nest rewrites. Both walks are therefore bounded, and exceeding
//! a bound is a typed refusal rather than a truncated answer — a truncated
//! authorization result is a wrong one, not a partial one.

/// Bounds applied to one expansion session. A standalone check creates a
/// fresh one-candidate session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpansionBounds {
    /// Maximum candidate memberships checked in one decision scope.
    pub max_candidates: usize,
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
        max_candidates: 256,
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

/// Tracks spend against [`ExpansionBounds`] for one expansion session.
#[derive(Debug)]
pub(crate) struct Budget {
    candidates_checked: usize,
    tuples_read: usize,
    bounds: ExpansionBounds,
}

impl Budget {
    pub(crate) fn new(bounds: ExpansionBounds) -> Self {
        Self {
            candidates_checked: 0,
            tuples_read: 0,
            bounds,
        }
    }

    /// Charge one candidate before starting its subwalk.
    pub(crate) fn charge_candidate(&mut self) -> Result<(), crate::ExpansionError> {
        self.candidates_checked = self.candidates_checked.saturating_add(1);
        if self.candidates_checked > self.bounds.max_candidates {
            return Err(crate::ExpansionError::CandidateBudgetExceeded {
                limit: self.bounds.max_candidates,
            });
        }
        Ok(())
    }

    /// Charge `count` tuples, refusing once the bound is passed.
    pub(crate) fn charge_tuples(&mut self, count: usize) -> Result<(), crate::ExpansionError> {
        self.tuples_read = self.tuples_read.saturating_add(count);
        if self.tuples_read > self.bounds.max_tuples_read {
            return Err(crate::ExpansionError::TupleBudgetExceeded {
                limit: self.bounds.max_tuples_read,
            });
        }
        Ok(())
    }
}
