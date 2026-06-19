//! Message delivery-class and channel-kind domain types for Professional Messenger.
//!
//! Introduced as merge-variant delta-1 for M03-P05-connect-pro-messenger per
//! user-directive-option-2 (execution_variant=merge-into-existing-crates).
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Delivery semantics for a message within the Professional Messenger surface.
///
/// The class is set at send time and is immutable thereafter — it cannot be
/// downgraded once written (ADR-0208: Professional context InternalAuditable
/// mode; ADR-0083 Tier 1 error-handling).
///
/// `InternalAuditOnly` implies that the message body is stored encrypted under
/// the tenant DEK and is only decryptable via the four-eyes audit pathway
/// (ADR-0208 §5).  It is the mandatory class for all Professional-context
/// channels at M03.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MessageDeliveryClass {
    /// Message is delivered in real-time over the WebSocket fan-out bus
    /// (p99 ≤ 200 ms at 5 k concurrent sessions).  Standard Professional
    /// delivery for interactive threads.
    Realtime,
    /// Message is queued for at-most-once async delivery (e.g. when the
    /// recipient is offline).  Used for broadcast / announcement channels
    /// where head-of-line latency is acceptable.
    Deferred,
    /// Message body is written exclusively to the InternalAuditable store
    /// (tenant-DEK-encrypted) and never delivered over the real-time bus.
    /// Surfaced only through the four-eyes audit pathway.  Mandatory for all
    /// compliance-scoped channels (ADR-0208 §5).
    InternalAuditOnly,
}

impl MessageDeliveryClass {
    /// Returns `true` when the delivery class pushes messages over the
    /// real-time WebSocket bus.
    pub fn is_realtime(&self) -> bool {
        matches!(self, Self::Realtime)
    }

    /// Returns `true` when this class mandates that the message body is stored
    /// under the tenant DEK and is only decryptable via the four-eyes audit
    /// pathway.
    pub fn is_audit_scoped(&self) -> bool {
        matches!(self, Self::InternalAuditOnly)
    }

    /// Returns `true` when the delivery class is Generally Available for
    /// Professional-context channels at the current milestone (M03).
    ///
    /// At M03, only `InternalAuditOnly` is GA for Professional channels
    /// (ADR-0208 §5).  `Realtime` and `Deferred` are pre-GA and must not be
    /// admitted by any policy check that relies on this predicate.
    pub fn is_ga(&self) -> bool {
        matches!(self, Self::InternalAuditOnly)
    }
}

/// Channel surface kind within the Professional Messenger product.
///
/// Mirrors the SQL `CHECK (kind IN ('messaging','broadcast','discussion'))`
/// constraint from `migrations/connect/002_messenger_schema.sql`.
/// The variant is fixed at channel-creation time; changing it requires a
/// full channel re-provision.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MessengerChannelKind {
    /// Bi-directional interactive messaging.  Default class: `InternalAuditOnly` (M03).
    Messaging,
    /// One-way broadcast from owner to subscribers.  Default class: `InternalAuditOnly` (M03).
    Broadcast,
    /// Threaded discussion board.  Default class: `InternalAuditOnly` (M03).
    Discussion,
}

impl MessengerChannelKind {
    /// Returns the default `MessageDeliveryClass` for this channel kind.
    ///
    /// At M03 all Professional-context channels default to `InternalAuditOnly`
    /// (tenant-DEK + four-eyes pathway) regardless of interactivity (ADR-0208 §5).
    /// Callers that previously relied on `Realtime` or `Deferred` defaults must
    /// explicitly opt in via channel-provisioning overrides if a future milestone
    /// re-opens those classes.
    pub fn default_delivery_class(&self) -> MessageDeliveryClass {
        match self {
            Self::Messaging | Self::Discussion | Self::Broadcast => {
                MessageDeliveryClass::InternalAuditOnly
            }
        }
    }

    /// Returns `true` when the channel kind supports interactive (non-broadcast)
    /// participation by any member.
    pub fn is_interactive(&self) -> bool {
        !matches!(self, Self::Broadcast)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_class_realtime_flags() {
        assert!(MessageDeliveryClass::Realtime.is_realtime());
        assert!(!MessageDeliveryClass::Deferred.is_realtime());
        assert!(!MessageDeliveryClass::InternalAuditOnly.is_realtime());
    }

    #[test]
    fn delivery_class_audit_scope() {
        assert!(!MessageDeliveryClass::Realtime.is_audit_scoped());
        assert!(!MessageDeliveryClass::Deferred.is_audit_scoped());
        assert!(MessageDeliveryClass::InternalAuditOnly.is_audit_scoped());
    }

    #[test]
    fn delivery_class_ga_flag() {
        // At M03 only InternalAuditOnly is GA for Professional channels (ADR-0208 §5).
        assert!(!MessageDeliveryClass::Realtime.is_ga());
        assert!(!MessageDeliveryClass::Deferred.is_ga());
        assert!(MessageDeliveryClass::InternalAuditOnly.is_ga());
    }

    // --- synthetic violation regression tests (Codex P1 fix) ---

    /// Ensures that `is_ga` does NOT admit Realtime as a GA delivery class at
    /// M03.  A regression here would allow policy checks to treat real-time
    /// delivery as permitted in Professional channels, bypassing the
    /// InternalAuditOnly mandate (ADR-0208 §5).
    #[test]
    fn realtime_is_not_ga_at_m03_professional() {
        assert!(
            !MessageDeliveryClass::Realtime.is_ga(),
            "Realtime must not be GA for Professional channels at M03"
        );
    }

    /// Ensures that `is_ga` does NOT admit Deferred as a GA delivery class at
    /// M03.
    #[test]
    fn deferred_is_not_ga_at_m03_professional() {
        assert!(
            !MessageDeliveryClass::Deferred.is_ga(),
            "Deferred must not be GA for Professional channels at M03"
        );
    }

    #[test]
    fn delivery_class_ordering() {
        assert!(MessageDeliveryClass::Realtime < MessageDeliveryClass::Deferred);
        assert!(MessageDeliveryClass::Deferred < MessageDeliveryClass::InternalAuditOnly);
    }

    #[test]
    fn channel_kind_default_delivery() {
        // At M03 all Professional-context channel kinds default to InternalAuditOnly
        // (ADR-0208 §5 — tenant-DEK + four-eyes pathway).
        assert_eq!(
            MessengerChannelKind::Messaging.default_delivery_class(),
            MessageDeliveryClass::InternalAuditOnly
        );
        assert_eq!(
            MessengerChannelKind::Broadcast.default_delivery_class(),
            MessageDeliveryClass::InternalAuditOnly
        );
        assert_eq!(
            MessengerChannelKind::Discussion.default_delivery_class(),
            MessageDeliveryClass::InternalAuditOnly
        );
    }

    /// Synthetic-violation regression: no channel kind must ever silently
    /// default to Realtime or Deferred at M03.  A regression here would cause
    /// newly provisioned channels to route message bodies outside the required
    /// audit-only path (Codex P1, ADR-0208 §5).
    #[test]
    fn no_channel_kind_defaults_to_non_audit_delivery_at_m03() {
        let kinds = [
            MessengerChannelKind::Messaging,
            MessengerChannelKind::Broadcast,
            MessengerChannelKind::Discussion,
        ];
        for kind in kinds {
            let class = kind.default_delivery_class();
            assert!(
                class.is_audit_scoped(),
                "{kind:?} defaulted to {class:?} — must be InternalAuditOnly at M03"
            );
        }
    }

    #[test]
    fn channel_kind_interactive() {
        assert!(MessengerChannelKind::Messaging.is_interactive());
        assert!(!MessengerChannelKind::Broadcast.is_interactive());
        assert!(MessengerChannelKind::Discussion.is_interactive());
    }

    #[test]
    fn channel_kind_ordering() {
        assert!(MessengerChannelKind::Messaging < MessengerChannelKind::Broadcast);
        assert!(MessengerChannelKind::Broadcast < MessengerChannelKind::Discussion);
    }
}
