//! Deterministic moderation-queue triage usecase.
//!
//! Accepts moderation outcomes produced by [`super::moderate_post`] /
//! [`oya_community_post_store_domain::moderation_case`], assigns a
//! severity-ordered priority, and exposes a pure ordered-drain function.
//!
//! # Ordering contract (documented tiebreak)
//! 1. `severity` descending: `Remove > Hide > Allow`
//! 2. `evidence_strength` descending: `Strong > None`
//! 3. `report_count` descending: higher count first
//! 4. `idempotency_key` ascending: lexicographic (stable, deterministic final tiebreak)

use oya_community_post_store_api::{AuthorizedCommunityContext, ModerationVerb};
use oya_community_post_store_domain::CommunityError;

use crate::CommunityUsecaseError;

/// Severity derived from [`ModerationVerb`]. `Remove` is highest priority.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum QueueSeverity {
    Allow,
    Hide,
    Remove,
}

/// Evidence strength signal attached to a queued moderation case.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EvidenceStrength {
    None,
    Strong,
}

/// A single entry in the moderation queue.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModerationQueueEntry {
    pub post_id: String,
    pub severity: QueueSeverity,
    pub evidence_strength: EvidenceStrength,
    pub report_count: u32,
    /// Dedup key. Copied verbatim from [`AuthorizedCommunityContext`]; never mutated.
    pub idempotency_key: String,
    /// Audit correlation. Copied verbatim; never mutated by sort or drain.
    pub audit_correlation_id: String,
    pub policy_decision_ref: String,
    pub tenant_scope_ref: String,
    pub principal_ref: String,
}

/// Pure in-memory moderation queue. No I/O, no async.
#[derive(Clone, Debug, Default)]
pub struct ModerationQueue {
    entries: Vec<ModerationQueueEntry>,
}

impl ModerationQueue {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Enqueue a moderation case produced by the `moderate_post` path.
///
/// # Errors
/// - `CommunityUsecaseError::Api` — `ctx.validate()` fails.
/// - `CommunityUsecaseError::Domain(CommunityError::Invalid)` — `post_id` is empty.
/// - `CommunityUsecaseError::Domain(CommunityError::ModerationNeedsEvidence)` — `Hide`/`Remove`
///   with empty `evidence_ref`.
///
/// # Idempotency
/// If an entry with the same `idempotency_key` already exists the call is a
/// no-op and returns `Ok(())`.
pub fn enqueue(
    queue: &mut ModerationQueue,
    ctx: &AuthorizedCommunityContext,
    post_id: String,
    verb: ModerationVerb,
    evidence_ref: &str,
    report_count: u32,
) -> Result<(), CommunityUsecaseError> {
    ctx.validate().map_err(CommunityUsecaseError::Api)?;

    if post_id.trim().is_empty() {
        return Err(CommunityUsecaseError::Domain(CommunityError::Invalid));
    }

    let severity = verb_to_severity(verb);

    // Evidence gate mirrors moderation_case in the domain crate.
    let evidence_strength = match verb {
        ModerationVerb::Hide | ModerationVerb::Remove => {
            if evidence_ref.trim().is_empty() {
                return Err(CommunityUsecaseError::Domain(
                    CommunityError::ModerationNeedsEvidence,
                ));
            }
            EvidenceStrength::Strong
        }
        ModerationVerb::Allow => EvidenceStrength::None,
    };

    // Idempotency: duplicate key is a no-op.
    if queue
        .entries
        .iter()
        .any(|e| e.idempotency_key == ctx.idempotency_key)
    {
        return Ok(());
    }

    queue.entries.push(ModerationQueueEntry {
        post_id,
        severity,
        evidence_strength,
        report_count,
        idempotency_key: ctx.idempotency_key.clone(),
        audit_correlation_id: ctx.audit_correlation_id.clone(),
        policy_decision_ref: ctx.policy_decision_ref.clone(),
        tenant_scope_ref: ctx.tenant_scope_ref.clone(),
        principal_ref: ctx.principal_ref.clone(),
    });

    Ok(())
}

/// Returns a reference to the highest-priority entry without mutating the queue.
/// Returns `None` if the queue is empty.
pub fn next_case(queue: &ModerationQueue) -> Option<&ModerationQueueEntry> {
    drain_ordered(queue).into_iter().next()
}

/// Returns all entries sorted highest-priority-first using the documented
/// tiebreak: severity desc → evidence_strength desc → report_count desc →
/// idempotency_key asc.
///
/// Non-mutating. `audit_correlation_id` and `idempotency_key` are never modified.
pub fn drain_ordered(queue: &ModerationQueue) -> Vec<&ModerationQueueEntry> {
    let mut refs: Vec<&ModerationQueueEntry> = queue.entries.iter().collect();
    refs.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| b.evidence_strength.cmp(&a.evidence_strength))
            .then_with(|| b.report_count.cmp(&a.report_count))
            .then_with(|| a.idempotency_key.cmp(&b.idempotency_key))
    });
    refs
}

fn verb_to_severity(verb: ModerationVerb) -> QueueSeverity {
    match verb {
        ModerationVerb::Allow => QueueSeverity::Allow,
        ModerationVerb::Hide => QueueSeverity::Hide,
        ModerationVerb::Remove => QueueSeverity::Remove,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(idem: &str) -> AuthorizedCommunityContext {
        AuthorizedCommunityContext {
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:mod".into(),
            idempotency_key: idem.into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit-1".into(),
        }
    }

    // ── ST1: evidence gate ────────────────────────────────────────────────

    #[test]
    fn enqueue_rejects_hide_without_evidence() {
        let mut q = ModerationQueue::new();
        assert_eq!(
            enqueue(
                &mut q,
                &ctx("k1"),
                "post:1".into(),
                ModerationVerb::Hide,
                "",
                1
            ),
            Err(CommunityUsecaseError::Domain(
                CommunityError::ModerationNeedsEvidence
            ))
        );
        assert!(drain_ordered(&q).is_empty());
    }

    #[test]
    fn enqueue_rejects_remove_without_evidence() {
        let mut q = ModerationQueue::new();
        assert_eq!(
            enqueue(
                &mut q,
                &ctx("k1"),
                "post:1".into(),
                ModerationVerb::Remove,
                "",
                0
            ),
            Err(CommunityUsecaseError::Domain(
                CommunityError::ModerationNeedsEvidence
            ))
        );
    }

    #[test]
    fn enqueue_allow_succeeds_without_evidence() {
        let mut q = ModerationQueue::new();
        enqueue(
            &mut q,
            &ctx("k1"),
            "post:1".into(),
            ModerationVerb::Allow,
            "",
            0,
        )
        .unwrap();
        assert_eq!(drain_ordered(&q).len(), 1);
    }

    // ── ST2: ordering ─────────────────────────────────────────────────────

    #[test]
    fn drain_ordered_remove_before_hide_before_allow() {
        let mut q = ModerationQueue::new();
        enqueue(
            &mut q,
            &ctx("k-allow"),
            "post:a".into(),
            ModerationVerb::Allow,
            "",
            0,
        )
        .unwrap();
        enqueue(
            &mut q,
            &ctx("k-hide"),
            "post:b".into(),
            ModerationVerb::Hide,
            "ev",
            1,
        )
        .unwrap();
        enqueue(
            &mut q,
            &ctx("k-remove"),
            "post:c".into(),
            ModerationVerb::Remove,
            "ev",
            1,
        )
        .unwrap();

        let ordered = drain_ordered(&q);
        assert_eq!(ordered[0].severity, QueueSeverity::Remove);
        assert_eq!(ordered[1].severity, QueueSeverity::Hide);
        assert_eq!(ordered[2].severity, QueueSeverity::Allow);
    }

    #[test]
    fn drain_ordered_stable_tiebreak_on_equal_severity() {
        let mut q = ModerationQueue::new();
        // Same verb/evidence/report_count — tiebreak on idempotency_key ascending.
        enqueue(
            &mut q,
            &ctx("k-zzz"),
            "post:z".into(),
            ModerationVerb::Hide,
            "ev",
            5,
        )
        .unwrap();
        enqueue(
            &mut q,
            &ctx("k-aaa"),
            "post:a".into(),
            ModerationVerb::Hide,
            "ev",
            5,
        )
        .unwrap();
        enqueue(
            &mut q,
            &ctx("k-mmm"),
            "post:m".into(),
            ModerationVerb::Hide,
            "ev",
            5,
        )
        .unwrap();

        let ordered = drain_ordered(&q);
        assert_eq!(ordered[0].idempotency_key, "k-aaa");
        assert_eq!(ordered[1].idempotency_key, "k-mmm");
        assert_eq!(ordered[2].idempotency_key, "k-zzz");
    }

    #[test]
    fn audit_fields_pass_through_unchanged() {
        let mut q = ModerationQueue::new();
        let c = AuthorizedCommunityContext {
            tenant_scope_ref: "tenant:t".into(),
            principal_ref: "user:mod".into(),
            idempotency_key: "idem-42".into(),
            policy_decision_ref: "pdp-ref".into(),
            audit_correlation_id: "audit-xyz".into(),
        };
        enqueue(
            &mut q,
            &c,
            "post:1".into(),
            ModerationVerb::Remove,
            "evidence",
            3,
        )
        .unwrap();

        let entry = next_case(&q).unwrap();
        assert_eq!(entry.idempotency_key, "idem-42");
        assert_eq!(entry.audit_correlation_id, "audit-xyz");
        assert_eq!(entry.policy_decision_ref, "pdp-ref");
    }

    // ── ST3: edge cases ───────────────────────────────────────────────────

    #[test]
    fn next_case_empty_queue_returns_none() {
        let q = ModerationQueue::new();
        assert!(next_case(&q).is_none());
    }

    #[test]
    fn drain_ordered_empty_queue_returns_empty_vec() {
        let q = ModerationQueue::new();
        assert!(drain_ordered(&q).is_empty());
    }

    #[test]
    fn enqueue_duplicate_idempotency_key_is_noop() {
        let mut q = ModerationQueue::new();
        enqueue(
            &mut q,
            &ctx("k1"),
            "post:1".into(),
            ModerationVerb::Remove,
            "ev",
            1,
        )
        .unwrap();
        // Second call with same idempotency_key must be a no-op.
        enqueue(
            &mut q,
            &ctx("k1"),
            "post:2".into(),
            ModerationVerb::Remove,
            "ev",
            99,
        )
        .unwrap();

        let ordered = drain_ordered(&q);
        assert_eq!(ordered.len(), 1, "duplicate must not be inserted");
        assert_eq!(ordered[0].post_id, "post:1", "original entry preserved");
    }

    #[test]
    fn enqueue_tenant_mismatch_caught_by_ctx_validate() {
        let bad_ctx = AuthorizedCommunityContext {
            // missing "tenant:" prefix — validate() returns MissingTenantScope
            tenant_scope_ref: "org:acme".into(),
            principal_ref: "user:mod".into(),
            idempotency_key: "k1".into(),
            policy_decision_ref: "policy".into(),
            audit_correlation_id: "audit".into(),
        };
        let mut q = ModerationQueue::new();
        assert!(matches!(
            enqueue(
                &mut q,
                &bad_ctx,
                "post:1".into(),
                ModerationVerb::Allow,
                "",
                0
            ),
            Err(CommunityUsecaseError::Api(_))
        ));
    }

    #[test]
    fn enqueue_report_count_tiebreak_higher_first() {
        let mut q = ModerationQueue::new();
        enqueue(
            &mut q,
            &ctx("k-low"),
            "post:a".into(),
            ModerationVerb::Hide,
            "ev",
            1,
        )
        .unwrap();
        enqueue(
            &mut q,
            &ctx("k-high"),
            "post:b".into(),
            ModerationVerb::Hide,
            "ev",
            99,
        )
        .unwrap();

        let ordered = drain_ordered(&q);
        assert_eq!(ordered[0].report_count, 99);
        assert_eq!(ordered[1].report_count, 1);
    }
}
