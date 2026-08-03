//! M02-P02-IP-006 — ToS-acknowledgment policy + pool-routing audit-chain kernel.
//!
//! Pure-Rust kernel that gates pool-membership >1 on a non-revoked ToS-ack
//! row per (tenant, provider). Linus good-taste: ToS-ack is binary (present
//! or absent). One predicate; one verdict; no branching.
//!
//! Cedar evaluation is wired in the app layer; this kernel only owns the
//! data shape + the pure decision function so the cedar policy file and the
//! Rust check can be cross-property-tested for equivalence.
//!
//! Audit-chain emission of `EVT-PROVIDER-POOL-ROUTING` is produced as a
//! deterministic record value that the audit-chain adapter consumes.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use intelligence_account_kernel::ProviderFamily;
use intelligence_provider_pool_kernel::{
    PoolId, PoolRoutingDecision, PoolRoutingReason, ProviderAccountId, TenantId, TosAckId,
};
use std::fmt;

/// data_class: INTERNAL_ONLY — SHA-256 hex hash of the signed-acceptance bundle.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Sha256Hash(pub String); // data_class: INTERNAL_ONLY

/// data_class: INTERNAL_ONLY — opaque actor id (kernel never sees PII).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ActorId(pub String); // data_class: INTERNAL_ONLY

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToSAcknowledgment {
    pub id: TosAckId,                      // data_class: INTERNAL_ONLY
    pub tenant_id: TenantId,               // data_class: TENANT_SCOPED
    pub provider: ProviderFamily,          // data_class: INTERNAL_ONLY
    pub upstream_tos_version: String,      // data_class: INTERNAL_ONLY
    pub accepted_at_unix_secs: u64,        // data_class: INTERNAL_ONLY
    pub accepted_by: ActorId,              // data_class: INTERNAL_ONLY
    pub evidence_hash: Sha256Hash,         // data_class: INTERNAL_ONLY
    pub revoked_at_unix_secs: Option<u64>, // data_class: INTERNAL_ONLY
}

impl ToSAcknowledgment {
    pub fn is_active(&self) -> bool {
        self.revoked_at_unix_secs.is_none()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AntiCorrelationRule {
    DistinctSourceIp,
    DistinctOAuthIdentity,
    MinRotationIntervalMs(u64),
    BlocklistedDualUse,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantPoolingPolicy {
    pub tenant_id: TenantId,            // data_class: TENANT_SCOPED
    pub max_pool_size_per_provider: u8, // data_class: INTERNAL_ONLY
    pub anti_correlation_rules: Vec<AntiCorrelationRule>, // data_class: INTERNAL_ONLY
    /// One ack per provider family. Vec instead of BTreeMap because
    /// `ProviderFamily` (defined in `oya-intelligence-account-kernel`) does not
    /// derive `Ord`/`Hash`. Lookup is O(N) but N == # provider families
    /// (≤10), so cost is negligible.
    pub tos_acks: Vec<ToSAcknowledgment>, // data_class: INTERNAL_ONLY
}

impl TenantPoolingPolicy {
    /// Pure lookup — find an ack for the given provider, if present.
    pub fn ack_for(&self, provider: ProviderFamily) -> Option<&ToSAcknowledgment> {
        self.tos_acks.iter().find(|a| a.provider == provider)
    }

    /// Pure mutable lookup — used to revoke an ack in tests / fixtures.
    pub fn ack_for_mut(&mut self, provider: ProviderFamily) -> Option<&mut ToSAcknowledgment> {
        self.tos_acks.iter_mut().find(|a| a.provider == provider)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PoolingPolicyVerdict {
    Allow,
    Deny(PoolingPolicyDenial),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PoolingPolicyDenial {
    ToSAckRequired,
    ToSAckRevoked,
    PoolSizeExceeded,
    BlocklistedDualUse,
}

impl PoolingPolicyDenial {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ToSAckRequired => "tos_ack_required",
            Self::ToSAckRevoked => "tos_ack_revoked",
            Self::PoolSizeExceeded => "pool_size_exceeded",
            Self::BlocklistedDualUse => "blocklisted_dual_use",
        }
    }
}

impl PoolingPolicyVerdict {
    pub fn allowed(&self) -> bool {
        matches!(self, Self::Allow)
    }
}

/// Pure decision function. One predicate, one verdict — no branching beyond
/// the explicit denial enum (Linus good-taste row).
pub fn pooling_policy_check(
    policy: &TenantPoolingPolicy,
    provider: ProviderFamily,
    pool_size: u8,
) -> PoolingPolicyVerdict {
    // pool_size <= 1 is always allowed (single-account == no pool, see IP-001
    // Linus row).
    if pool_size <= 1 {
        return PoolingPolicyVerdict::Allow;
    }
    if pool_size > policy.max_pool_size_per_provider {
        return PoolingPolicyVerdict::Deny(PoolingPolicyDenial::PoolSizeExceeded);
    }
    if policy
        .anti_correlation_rules
        .iter()
        .any(|r| matches!(r, AntiCorrelationRule::BlocklistedDualUse))
    {
        return PoolingPolicyVerdict::Deny(PoolingPolicyDenial::BlocklistedDualUse);
    }
    match policy.ack_for(provider) {
        None => PoolingPolicyVerdict::Deny(PoolingPolicyDenial::ToSAckRequired),
        Some(ack) if !ack.is_active() => {
            PoolingPolicyVerdict::Deny(PoolingPolicyDenial::ToSAckRevoked)
        }
        Some(_) => PoolingPolicyVerdict::Allow,
    }
}

/// data_class: INTERNAL_ONLY — `EVT-PROVIDER-POOL-ROUTING` audit-chain entry.
/// Adapter-layer audit-chain writer is the consumer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolRoutingEvent {
    pub pool_id: PoolId,                        // data_class: INTERNAL_ONLY
    pub tenant_id: TenantId,                    // data_class: TENANT_SCOPED
    pub provider: ProviderFamily,               // data_class: INTERNAL_ONLY
    pub chosen_account_redacted_suffix: String, // data_class: INTERNAL_ONLY (last 4 of id)
    pub routing_reason: String,                 // data_class: INTERNAL_ONLY
    pub fallback_chain_redacted: Vec<String>,   // data_class: INTERNAL_ONLY (suffixes)
    pub tos_ack_ref: Option<TosAckId>,          // data_class: INTERNAL_ONLY
    pub trace_id: String,                       // data_class: INTERNAL_ONLY
    pub decided_at_unix_ms: u64,                // data_class: INTERNAL_ONLY
    pub autonomy_tier: String,                  // data_class: INTERNAL_ONLY
    pub event_kind: &'static str,               // data_class: INTERNAL_ONLY
}

pub const EVT_PROVIDER_POOL_ROUTING: &str = "EVT-PROVIDER-POOL-ROUTING";

pub fn emit_pool_routing_event(
    pool_id: &PoolId,
    tenant_id: &TenantId,
    provider: ProviderFamily,
    decision: &PoolRoutingDecision,
    tos_ack_ref: Option<&TosAckId>,
    trace_id: &str,
    autonomy_tier: &str,
) -> PoolRoutingEvent {
    PoolRoutingEvent {
        pool_id: pool_id.clone(),
        tenant_id: tenant_id.clone(),
        provider,
        chosen_account_redacted_suffix: redact_suffix(&decision.account_id),
        routing_reason: routing_reason_label(&decision.reason),
        fallback_chain_redacted: decision.fallback_chain.iter().map(redact_suffix).collect(),
        tos_ack_ref: tos_ack_ref.cloned(),
        trace_id: trace_id.to_owned(),
        decided_at_unix_ms: decision.decided_at_unix_ms.0,
        autonomy_tier: autonomy_tier.to_owned(),
        event_kind: EVT_PROVIDER_POOL_ROUTING,
    }
}

fn redact_suffix(id: &ProviderAccountId) -> String {
    let s = &id.0;
    if s.len() <= 4 {
        format!("****{s}")
    } else {
        let suffix = &s[s.len() - 4..];
        format!("****{suffix}")
    }
}

fn routing_reason_label(r: &PoolRoutingReason) -> String {
    match r {
        PoolRoutingReason::Healthy => "healthy".into(),
        PoolRoutingReason::FailoverFrom(prev) => format!("failover_from:{}", redact_suffix(prev)),
        PoolRoutingReason::Sticky => "sticky".into(),
        PoolRoutingReason::QuotaPreserve => "quota_preserve".into(),
        PoolRoutingReason::LeastUsedTieBreak => "least_used_tie_break".into(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToSPolicyError {
    EvidenceHashEmpty,
    UpstreamTosVersionEmpty,
}

impl fmt::Display for ToSPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EvidenceHashEmpty => write!(f, "evidence_hash is empty"),
            Self::UpstreamTosVersionEmpty => write!(f, "upstream_tos_version is empty"),
        }
    }
}

/// Pure builder that validates required fields and emits a ToSAcknowledgment.
pub fn build_acknowledgment(
    id: TosAckId,
    tenant_id: TenantId,
    provider: ProviderFamily,
    upstream_tos_version: String,
    accepted_at_unix_secs: u64,
    accepted_by: ActorId,
    evidence_hash: Sha256Hash,
) -> Result<ToSAcknowledgment, ToSPolicyError> {
    if evidence_hash.0.is_empty() {
        return Err(ToSPolicyError::EvidenceHashEmpty);
    }
    if upstream_tos_version.is_empty() {
        return Err(ToSPolicyError::UpstreamTosVersionEmpty);
    }
    Ok(ToSAcknowledgment {
        id,
        tenant_id,
        provider,
        upstream_tos_version,
        accepted_at_unix_secs,
        accepted_by,
        evidence_hash,
        revoked_at_unix_secs: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use intelligence_provider_pool_kernel::UnixMillis;

    fn tenant() -> TenantId {
        TenantId("t-1".into())
    }

    fn ack(provider: ProviderFamily) -> ToSAcknowledgment {
        build_acknowledgment(
            TosAckId("ack-1".into()),
            tenant(),
            provider,
            "anthropic-aup-2024-09".into(),
            1_700_000_000,
            ActorId("operator-001".into()),
            Sha256Hash("deadbeef".into()),
        )
        .unwrap()
    }

    fn policy_with(provider: Option<ProviderFamily>, max: u8) -> TenantPoolingPolicy {
        let mut acks: Vec<ToSAcknowledgment> = Vec::new();
        if let Some(p) = provider {
            acks.push(ack(p));
        }
        TenantPoolingPolicy {
            tenant_id: tenant(),
            max_pool_size_per_provider: max,
            anti_correlation_rules: Vec::new(),
            tos_acks: acks,
        }
    }

    #[test]
    fn pool_size_1_always_allowed() {
        let p = policy_with(None, 5);
        assert_eq!(
            pooling_policy_check(&p, ProviderFamily::Claude, 1),
            PoolingPolicyVerdict::Allow
        );
    }

    #[test]
    fn no_ack_pool_size_2_denied() {
        let p = policy_with(None, 5);
        assert_eq!(
            pooling_policy_check(&p, ProviderFamily::Claude, 2),
            PoolingPolicyVerdict::Deny(PoolingPolicyDenial::ToSAckRequired)
        );
    }

    #[test]
    fn revoked_ack_pool_size_2_denied() {
        let mut p = policy_with(Some(ProviderFamily::Claude), 5);
        let entry = p.ack_for_mut(ProviderFamily::Claude).unwrap();
        entry.revoked_at_unix_secs = Some(1_700_000_100);
        assert_eq!(
            pooling_policy_check(&p, ProviderFamily::Claude, 2),
            PoolingPolicyVerdict::Deny(PoolingPolicyDenial::ToSAckRevoked)
        );
    }

    #[test]
    fn valid_ack_pool_size_within_max_allowed() {
        let p = policy_with(Some(ProviderFamily::Claude), 5);
        assert_eq!(
            pooling_policy_check(&p, ProviderFamily::Claude, 3),
            PoolingPolicyVerdict::Allow
        );
    }

    #[test]
    fn pool_size_exceeds_max_denied() {
        let p = policy_with(Some(ProviderFamily::Claude), 3);
        assert_eq!(
            pooling_policy_check(&p, ProviderFamily::Claude, 5),
            PoolingPolicyVerdict::Deny(PoolingPolicyDenial::PoolSizeExceeded)
        );
    }

    #[test]
    fn blocklisted_dual_use_denies_regardless_of_ack() {
        let mut p = policy_with(Some(ProviderFamily::Claude), 5);
        p.anti_correlation_rules
            .push(AntiCorrelationRule::BlocklistedDualUse);
        let _ = p.ack_for(ProviderFamily::Claude); // exercise lookup helper
        assert_eq!(
            pooling_policy_check(&p, ProviderFamily::Claude, 2),
            PoolingPolicyVerdict::Deny(PoolingPolicyDenial::BlocklistedDualUse)
        );
    }

    #[test]
    fn verdict_allowed_helper() {
        assert!(PoolingPolicyVerdict::Allow.allowed());
        assert!(!PoolingPolicyVerdict::Deny(PoolingPolicyDenial::ToSAckRequired).allowed());
    }

    #[test]
    fn build_ack_rejects_empty_evidence_hash() {
        let r = build_acknowledgment(
            TosAckId("a".into()),
            tenant(),
            ProviderFamily::Claude,
            "v".into(),
            0,
            ActorId("x".into()),
            Sha256Hash(String::new()),
        );
        assert_eq!(r, Err(ToSPolicyError::EvidenceHashEmpty));
    }

    #[test]
    fn build_ack_rejects_empty_version() {
        let r = build_acknowledgment(
            TosAckId("a".into()),
            tenant(),
            ProviderFamily::Claude,
            String::new(),
            0,
            ActorId("x".into()),
            Sha256Hash("h".into()),
        );
        assert_eq!(r, Err(ToSPolicyError::UpstreamTosVersionEmpty));
    }

    #[test]
    fn emit_event_redacts_account_to_last_four() {
        let decision = PoolRoutingDecision {
            account_id: ProviderAccountId("acct-very-long-id-1234".into()),
            reason: PoolRoutingReason::Healthy,
            fallback_chain: vec![ProviderAccountId("acct-fallback-5678".into())],
            decided_at_unix_ms: UnixMillis(42),
        };
        let ev = emit_pool_routing_event(
            &PoolId("p".into()),
            &tenant(),
            ProviderFamily::Claude,
            &decision,
            Some(&TosAckId("ack-1".into())),
            "trace-x",
            "T4_AUTO_EXECUTE",
        );
        assert_eq!(ev.event_kind, EVT_PROVIDER_POOL_ROUTING);
        assert_eq!(ev.chosen_account_redacted_suffix, "****1234");
        assert_eq!(ev.fallback_chain_redacted, vec!["****5678".to_owned()]);
        // Routing reason is a plain label here.
        assert_eq!(ev.routing_reason, "healthy");
    }

    #[test]
    fn emit_event_failover_redacts_previous() {
        let decision = PoolRoutingDecision {
            account_id: ProviderAccountId("aaaaABCD".into()),
            reason: PoolRoutingReason::FailoverFrom(ProviderAccountId("oldoldoldEFGH".into())),
            fallback_chain: Vec::new(),
            decided_at_unix_ms: UnixMillis(0),
        };
        let ev = emit_pool_routing_event(
            &PoolId("p".into()),
            &tenant(),
            ProviderFamily::Claude,
            &decision,
            None,
            "t",
            "T3",
        );
        assert!(ev.routing_reason.starts_with("failover_from:****"));
        assert!(ev.routing_reason.contains("EFGH"));
    }

    #[test]
    fn redact_short_id_padded() {
        let r = redact_suffix(&ProviderAccountId("ab".into()));
        assert_eq!(r, "****ab");
    }

    #[test]
    fn denial_names_distinct() {
        let s: std::collections::HashSet<&str> = [
            PoolingPolicyDenial::ToSAckRequired,
            PoolingPolicyDenial::ToSAckRevoked,
            PoolingPolicyDenial::PoolSizeExceeded,
            PoolingPolicyDenial::BlocklistedDualUse,
        ]
        .iter()
        .map(|d| d.name())
        .collect();
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn tos_error_display_distinct() {
        let m: Vec<String> = vec![
            format!("{}", ToSPolicyError::EvidenceHashEmpty),
            format!("{}", ToSPolicyError::UpstreamTosVersionEmpty),
        ];
        let uniq: std::collections::HashSet<_> = m.iter().collect();
        assert_eq!(uniq.len(), m.len());
    }

    #[test]
    fn ack_is_active_only_when_not_revoked() {
        let mut a = ack(ProviderFamily::Claude);
        assert!(a.is_active());
        a.revoked_at_unix_secs = Some(1);
        assert!(!a.is_active());
    }

    #[test]
    fn anti_correlation_rule_variants_distinct() {
        let s: Vec<AntiCorrelationRule> = vec![
            AntiCorrelationRule::DistinctSourceIp,
            AntiCorrelationRule::DistinctOAuthIdentity,
            AntiCorrelationRule::MinRotationIntervalMs(60_000),
            AntiCorrelationRule::BlocklistedDualUse,
        ];
        // Sanity: each variant has a unique Debug repr.
        let debugs: std::collections::HashSet<String> =
            s.iter().map(|r| format!("{r:?}")).collect();
        assert_eq!(debugs.len(), 4);
    }
}
