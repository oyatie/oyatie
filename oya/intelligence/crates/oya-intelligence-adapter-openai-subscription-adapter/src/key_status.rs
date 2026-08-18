//! Key lifecycle state for the OpenAI API-key pool.
// data_class: INTERNAL_ONLY throughout this module.

/// The lifecycle state of a single API key in the pool.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KeyStatus {
    /// Key is eligible for selection.
    Active,
    /// Key experienced transient failures; skip until `until_epoch_secs`.
    Cooling {
        /// Epoch-seconds after which the key may be re-selected.
        until_epoch_secs: u64,
        /// Number of consecutive transient failures that triggered cooling.
        failure_count: u32,
    },
    /// Key had a terminal error (invalid / quota exhausted); never selected again.
    Blacklisted,
}

impl KeyStatus {
    /// Returns `true` if the key is eligible for selection at the given epoch.
    pub fn is_eligible(&self, now_epoch_secs: u64) -> bool {
        match self {
            Self::Active => true,
            Self::Cooling {
                until_epoch_secs, ..
            } => now_epoch_secs >= *until_epoch_secs,
            Self::Blacklisted => false,
        }
    }
}

/// A single entry in the pool: the secret-reference path and its status.
#[derive(Clone, Debug)]
pub struct KeyEntry {
    /// The secret reference path (e.g. `sref://openai-key-0`).
    /// SECURITY: This is a reference path, not raw key material.
    pub sref_path: String,
    /// Current lifecycle status of the key.
    pub status: KeyStatus,
}

impl KeyEntry {
    /// Create a new `KeyEntry` in `Active` state.
    pub fn new(sref_path: impl Into<String>) -> Self {
        Self {
            sref_path: sref_path.into(),
            status: KeyStatus::Active,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_is_eligible_always() {
        let e = KeyEntry::new("sref://k0");
        assert!(e.status.is_eligible(0));
        assert!(e.status.is_eligible(u64::MAX));
    }

    #[test]
    fn cooling_not_eligible_before_expiry() {
        let s = KeyStatus::Cooling {
            until_epoch_secs: 1000,
            failure_count: 3,
        };
        assert!(!s.is_eligible(999));
    }

    #[test]
    fn cooling_eligible_at_expiry() {
        let s = KeyStatus::Cooling {
            until_epoch_secs: 1000,
            failure_count: 3,
        };
        assert!(s.is_eligible(1000));
        assert!(s.is_eligible(1001));
    }

    #[test]
    fn blacklisted_never_eligible() {
        assert!(!KeyStatus::Blacklisted.is_eligible(0));
        assert!(!KeyStatus::Blacklisted.is_eligible(u64::MAX));
    }

    #[test]
    fn key_entry_new_starts_active() {
        let e = KeyEntry::new("sref://openai-key-1");
        assert_eq!(e.status, KeyStatus::Active);
        assert_eq!(e.sref_path, "sref://openai-key-1");
    }
}
