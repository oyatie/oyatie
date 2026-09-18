//! Mutation budgets and the node-local per-account mutation semaphore.
//! Sleeping between attempts is the caller's: it drops its worker first.
use mail_kernel::Error;
use std::collections::HashMap;
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Fixed per-command budget for IMAP/POP after the literal completes and per
/// JMAP method call; queue waits and re-attempts all count against it.
pub const MUTATION_DEADLINE: Duration = Duration::from_secs(10);
/// Concurrent mutations one node admits per account.
pub const PER_ACCOUNT_MUTATIONS: u32 = 4;

#[derive(Clone, Copy, Debug)]
pub struct Budget {
    deadline: Instant,
}
impl Budget {
    pub fn fixed() -> Self {
        Self::until(Instant::now() + MUTATION_DEADLINE)
    }
    pub fn until(deadline: Instant) -> Self {
        Self { deadline }
    }
    pub fn deadline(&self) -> Instant {
        self.deadline
    }
    pub fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }
    pub fn expired(&self) -> bool {
        self.remaining().is_zero()
    }
}

/// Jittered exponential backoff for attempt `n` (0-based), capped so the
/// budget is spent in a handful of attempts rather than a tight loop.
pub fn backoff(attempt: u32) -> Duration {
    let base = 20u64 << attempt.min(6);
    let nanos = Instant::now().elapsed().subsec_nanos() as u64 ^ (attempt as u64 * 0x9E37_79B9);
    Duration::from_millis(base + nanos % base.max(1))
}

#[derive(Default)]
pub struct Admission {
    slots: Mutex<HashMap<String, u32>>,
    released: Condvar,
}

pub struct Slot<'a> {
    admission: &'a Admission,
    account: String,
}
impl Drop for Slot<'_> {
    fn drop(&mut self) {
        if let Ok(mut slots) = self.admission.slots.lock() {
            match slots.get_mut(&self.account) {
                Some(n) if *n > 1 => *n -= 1,
                _ => {
                    slots.remove(&self.account);
                }
            }
        }
        self.admission.released.notify_all();
    }
}

impl Admission {
    pub fn node() -> &'static Admission {
        static NODE: OnceLock<Admission> = OnceLock::new();
        NODE.get_or_init(Admission::default)
    }

    /// Wait for a slot until the budget's deadline; `Busy` when it passes.
    pub fn acquire(&self, account: &str, budget: &Budget) -> Result<Slot<'_>, Error> {
        let mut slots = self.slots.lock().map_err(|_| Error::Unavailable)?;
        loop {
            let held = slots.get(account).copied().unwrap_or(0);
            if held < PER_ACCOUNT_MUTATIONS {
                slots.insert(account.to_owned(), held + 1);
                return Ok(Slot {
                    admission: self,
                    account: account.to_owned(),
                });
            }
            let remaining = budget.remaining();
            if remaining.is_zero() {
                return Err(Error::Busy);
            }
            slots = self
                .released
                .wait_timeout(slots, remaining)
                .map_err(|_| Error::Unavailable)?
                .0;
        }
    }

    pub fn held(&self, account: &str) -> u32 {
        self.slots
            .lock()
            .map(|s| s.get(account).copied().unwrap_or(0))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_are_bounded_per_account_and_released_on_drop() {
        let admission = Admission::default();
        let budget = Budget::until(Instant::now() + Duration::from_millis(50));
        let held: Vec<_> = (0..PER_ACCOUNT_MUTATIONS)
            .map(|_| admission.acquire("a", &budget).unwrap())
            .collect();
        assert!(admission.acquire("b", &budget).is_ok());
        assert_eq!(admission.acquire("a", &budget).err(), Some(Error::Busy));
        drop(held);
        assert_eq!(admission.held("a"), 0);
        assert!(admission.acquire("a", &Budget::fixed()).is_ok());
    }

    #[test]
    fn backoff_grows_and_stays_bounded() {
        assert!(backoff(0) < Duration::from_millis(41));
        assert!(backoff(6) >= Duration::from_millis(1280));
        assert!(backoff(20) < Duration::from_millis(2561));
    }
}
