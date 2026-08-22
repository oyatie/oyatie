//! Decision-audit emission onto the tracing pipeline.
//!
//! [`TracingDecisionAuditSink`] bridges the kernel [`DecisionAuditSink`]
//! port onto structured tracing JSON so every decision — allow or deny,
//! cached or evaluated — is on the log stream from first boot (the
//! identity `TracingAuditSink` precedent). The audit-chain bridge
//! (CloudEvents envelope + signed digest chain) lands behind this SAME port
//! in a follow-up slice.

use iam_cloud_pdp_kernel::DecisionAuditSink;
use shared_pdp_kernel::DecisionAuditRecord;
use shared_platform_contracts_kernel::pdp::Decision;

/// [`DecisionAuditSink`] that emits each sealed record as one structured
/// tracing event. Emission cannot fail (tracing is fire-and-forget), so the
/// port contract — never fail the decision path — holds trivially.
#[derive(Clone, Copy, Debug, Default)]
pub struct TracingDecisionAuditSink;

impl TracingDecisionAuditSink {
    /// Build the sink.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl DecisionAuditSink for TracingDecisionAuditSink {
    fn record(&self, record: &DecisionAuditRecord) {
        let decision = match record.decision {
            Decision::Allow => "allow",
            Decision::Deny => "deny",
        };
        tracing::info!(
            target: "cloud_iam_pdp::audit",
            decision_id = %record.decision_id,
            request_id = %record.request_id,
            tenant_id = %record.tenant_id,
            principal_type = %record.principal.entity_type,
            principal_id = %record.principal.entity_id,
            action = %record.action,
            resource_type = %record.resource.entity_type,
            resource_id = %record.resource.entity_id,
            decision,
            policy_version = %record.policy_version.as_str(),
            determining_policy_ids = %record.determining_policy_ids.join(","),
            cache_hit = record.cache_hit,
            "decision-audit-record",
        );
    }
}
