//! Thread-state and mailbox-kind domain types for Connect Professional Mail.
//!
//! Introduced as merge-variant delta-1 for M03-P04-connect-pro-mail per
//! user-directive-option-2 (execution_variant=merge-into-existing-crates).
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Which context owns a mailbox within the dual-context isolation model
/// (ADR-0208 / ADR-0215: Professional context unreachable from Personal).
///
/// The variant is determined at account-provisioning time and is immutable
/// thereafter — a mailbox cannot migrate between contexts without a full
/// re-provision.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MailboxKind {
    /// Corporate mailbox provisioned by the tenant for an employee identity.
    /// Governed by the organisation's retention, legal-hold, and eDiscovery
    /// policies.
    Professional,
    /// Personal mailbox owned by the end-user (not the tenant).
    /// Scaffold is present; surface is NOT GA (M03-P04 scope note).
    PersonalNotGa,
    /// Shared mailbox (e.g. `support@example.com`) owned by the tenant but
    /// not tied to a single identity.
    SharedInbox,
}

impl MailboxKind {
    /// Returns `true` when this kind falls under tenant-governed compliance
    /// controls (retention, legal-hold, eDiscovery).
    pub fn is_tenant_governed(&self) -> bool {
        matches!(self, Self::Professional | Self::SharedInbox)
    }

    /// Returns `true` only when the kind is Generally Available.
    pub fn is_ga(&self) -> bool {
        !matches!(self, Self::PersonalNotGa)
    }
}

/// Lifecycle state of an email thread visible to the owning mailbox.
///
/// Transitions flow in one direction: `Active` → `Muted` or `Active` →
/// `Archived`, and `Archived` / `Muted` → `Deleted` (soft-delete pending
/// retention expiry). Hard-delete is not represented here; it occurs only
/// after the retention window expires in `RetentionPolicy`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ThreadStatus {
    /// Thread is in the inbox and may receive new messages.
    Active,
    /// Notifications suppressed by the user; thread still receives messages.
    Muted,
    /// Moved out of inbox; searchable but not prominently surfaced.
    Archived,
    /// Soft-deleted; hidden from the user but retained until the retention
    /// window expires.
    Deleted,
}

impl ThreadStatus {
    /// Returns `true` when the thread is still eligible to receive inbound
    /// messages (i.e. not soft-deleted).
    pub fn can_receive(&self) -> bool {
        !matches!(self, Self::Deleted)
    }

    /// Returns `true` when the thread is visible in the default inbox view.
    pub fn is_inbox_visible(&self) -> bool {
        matches!(self, Self::Active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mailbox_kind_tenant_governance() {
        assert!(MailboxKind::Professional.is_tenant_governed());
        assert!(MailboxKind::SharedInbox.is_tenant_governed());
        assert!(!MailboxKind::PersonalNotGa.is_tenant_governed());
    }

    #[test]
    fn mailbox_kind_ga_flag() {
        assert!(MailboxKind::Professional.is_ga());
        assert!(MailboxKind::SharedInbox.is_ga());
        assert!(!MailboxKind::PersonalNotGa.is_ga());
    }

    #[test]
    fn mailbox_kind_ordering() {
        assert!(MailboxKind::Professional < MailboxKind::PersonalNotGa);
        assert!(MailboxKind::PersonalNotGa < MailboxKind::SharedInbox);
    }

    #[test]
    fn thread_status_can_receive() {
        assert!(ThreadStatus::Active.can_receive());
        assert!(ThreadStatus::Muted.can_receive());
        assert!(ThreadStatus::Archived.can_receive());
        assert!(!ThreadStatus::Deleted.can_receive());
    }

    #[test]
    fn thread_status_inbox_visibility() {
        assert!(ThreadStatus::Active.is_inbox_visible());
        assert!(!ThreadStatus::Muted.is_inbox_visible());
        assert!(!ThreadStatus::Archived.is_inbox_visible());
        assert!(!ThreadStatus::Deleted.is_inbox_visible());
    }

    #[test]
    fn thread_status_ordering() {
        assert!(ThreadStatus::Active < ThreadStatus::Muted);
        assert!(ThreadStatus::Muted < ThreadStatus::Archived);
        assert!(ThreadStatus::Archived < ThreadStatus::Deleted);
    }
}
