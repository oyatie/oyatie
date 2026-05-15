//! Per-PR retry budget — IP-006 §"Bounded retry per PR".
//!
//! Mirrors IP-005's shared-pool budget so the kernel side has the same
//! N=5 cap. On exhaustion the kernel emits an `EvictWithEscalation`
//! decision; the integration crate
//! (`tools/oya-foundry-vcs-merge-queue-fix-loop-app`) translates that to
//! "remove PR from queue + open stuck-PR issue with `human-escalation`
//! label" (the same escalation issue created by IP-005's escalation path —
//! the integration crate deduplicates by PR number).

use std::collections::BTreeMap;
use std::fmt;

pub const MAX_ATTEMPTS_PER_PR: u32 = 5;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PrBudget {
    entries: BTreeMap<u64, u32>,
    evicted: BTreeMap<u64, u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BudgetVerdict {
    /// Within budget — proceed with re-admission (speculative rebase + re-CI).
    Proceed { attempts_used: u32 },
    /// Budget exhausted — evict and escalate.
    EvictWithEscalation { attempts_used: u32 },
    /// PR was already evicted; idempotent no-op.
    AlreadyEvicted { attempts_used: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrBudgetError {
    InvalidPrNumber,
}

impl fmt::Display for PrBudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PrBudgetError {}

impl PrBudget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn attempts_used(&self, pr_number: u64) -> u32 {
        if let Some(used) = self.evicted.get(&pr_number) {
            return *used;
        }
        self.entries.get(&pr_number).copied().unwrap_or(0)
    }

    pub fn is_evicted(&self, pr_number: u64) -> bool {
        self.evicted.contains_key(&pr_number)
    }

    /// Register a new revalidation attempt for `pr_number`.
    ///
    /// Returns:
    /// - `Proceed { attempts_used }` while under cap.
    /// - `EvictWithEscalation { attempts_used }` on the 5th attempt that
    ///   exhausts the budget. (Eviction fires on the attempt that takes
    ///   us to MAX_ATTEMPTS_PER_PR, not on attempt MAX+1, because the
    ///   merge-queue admission is what triggers the budget tick — IP-006
    ///   §"retry counter decrements once per cycle, not once per failure".)
    /// - `AlreadyEvicted` for any subsequent invocation.
    pub fn register_revalidation(
        &mut self,
        pr_number: u64,
    ) -> Result<BudgetVerdict, PrBudgetError> {
        if pr_number == 0 {
            return Err(PrBudgetError::InvalidPrNumber);
        }
        if let Some(&used) = self.evicted.get(&pr_number) {
            return Ok(BudgetVerdict::AlreadyEvicted {
                attempts_used: used,
            });
        }
        let counter = self.entries.entry(pr_number).or_insert(0);
        *counter += 1;
        let now = *counter;
        if now >= MAX_ATTEMPTS_PER_PR {
            self.evicted.insert(pr_number, now);
            self.entries.remove(&pr_number);
            return Ok(BudgetVerdict::EvictWithEscalation { attempts_used: now });
        }
        Ok(BudgetVerdict::Proceed { attempts_used: now })
    }

    /// Acknowledge a successful merge (clears the counter).
    pub fn mark_merged(&mut self, pr_number: u64) {
        self.entries.remove(&pr_number);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_proceeds_under_cap_and_evicts_at_cap() {
        let mut budget = PrBudget::new();
        for i in 1..MAX_ATTEMPTS_PER_PR {
            assert_eq!(
                budget.register_revalidation(7).unwrap(),
                BudgetVerdict::Proceed { attempts_used: i }
            );
        }
        assert_eq!(
            budget.register_revalidation(7).unwrap(),
            BudgetVerdict::EvictWithEscalation {
                attempts_used: MAX_ATTEMPTS_PER_PR
            }
        );
        // subsequent invocations are idempotent
        assert_eq!(
            budget.register_revalidation(7).unwrap(),
            BudgetVerdict::AlreadyEvicted {
                attempts_used: MAX_ATTEMPTS_PER_PR
            }
        );
        assert!(budget.is_evicted(7));
    }

    #[test]
    fn budget_rejects_pr_zero() {
        let mut budget = PrBudget::new();
        assert_eq!(
            budget.register_revalidation(0),
            Err(PrBudgetError::InvalidPrNumber)
        );
    }

    #[test]
    fn mark_merged_clears_counter() {
        let mut budget = PrBudget::new();
        budget.register_revalidation(7).unwrap();
        budget.mark_merged(7);
        assert_eq!(budget.attempts_used(7), 0);
    }
}
