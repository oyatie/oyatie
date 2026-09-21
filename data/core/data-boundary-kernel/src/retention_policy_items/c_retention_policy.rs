/// Purge action that must be taken when the retention window expires.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PurgeAction {
    /// Cryptographically shred the encryption key; the ciphertext becomes
    /// irrecoverable. Required for [`ClassificationLevel::Critical`] data.
    CryptoShred,
    /// Overwrite the storage block with zeroes before deallocation.
    SecureErase,
    /// Standard logical deletion (sufficient for unrestricted data).
    LogicalDelete,
}

/// Declared retention window and mandatory purge action for a classified data
/// object.
///
/// [`RetentionPolicy`] does not enforce the purge itself — that is the
/// responsibility of the data-boundary purge executor. It is a value type
/// that carries the contract established at classification time so that the
/// executor can audit-log the correct action and duration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionPolicy {
    /// Maximum wall-clock duration for which a data object at this
    /// classification level may be retained after its last legitimate use.
    // data_class: INTERNAL_ONLY
    pub retention_window: Duration,
    /// The purge action that must be taken when `retention_window` elapses.
    // data_class: INTERNAL_ONLY
    pub purge_action: PurgeAction,
    /// The classification level that governs this policy.
    // data_class: INTERNAL_ONLY
    pub level: ClassificationLevel,
}

impl RetentionPolicy {
    /// Construct a [`RetentionPolicy`] from a raw [`DataClass`].
    ///
    /// The defaults mirror the regulatory minimums codified in ADR-0008:
    ///
    /// | Level | Window | Action |
    /// |---|---|---|
    /// | Critical | 30 days | CryptoShred |
    /// | Sensitive | 90 days | SecureErase |
    /// | Restricted | 365 days | LogicalDelete |
    /// | Unrestricted | 730 days | LogicalDelete |
    pub fn from_data_class(data_class: DataClass) -> Self {
        let level = ClassificationLevel::from_data_class(data_class);
        Self::from_level(level)
    }

    /// Construct a [`RetentionPolicy`] directly from a [`ClassificationLevel`].
    pub fn from_level(level: ClassificationLevel) -> Self {
        let (retention_window, purge_action) = match level {
            ClassificationLevel::Critical => (
                Duration::from_secs(30 * 24 * 3600),
                PurgeAction::CryptoShred,
            ),
            ClassificationLevel::Sensitive => (
                Duration::from_secs(90 * 24 * 3600),
                PurgeAction::SecureErase,
            ),
            ClassificationLevel::Restricted => (
                Duration::from_secs(365 * 24 * 3600),
                PurgeAction::LogicalDelete,
            ),
            ClassificationLevel::Unrestricted => (
                Duration::from_secs(730 * 24 * 3600),
                PurgeAction::LogicalDelete,
            ),
        };
        Self {
            retention_window,
            purge_action,
            level,
        }
    }
}
