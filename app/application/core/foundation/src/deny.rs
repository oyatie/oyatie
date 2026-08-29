//! Denied-invocation audit trails.

use crate::*;

use crate::Foundation;

impl Foundation {
    pub(crate) fn append_invocation_denial_audits(
        &mut self,
        request: &CapabilityInvocationRequest,
        capability: &Capability,
    ) -> Result<(String, String), FoundationError> {
        // ADR-0083 amendment 2026-05-15: `append_classifications` is Tier 1
        // fallible; this helper propagates `AuditChainError` to the caller
        // via `FoundationError::AuditChainAppendFailed`.
        let capability_invoke_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "foundry.capability.invoke",
                Plane::Data,
                request.purpose,
                capability_record_classifications(capability),
                "DENY",
            )?
            .hash
            .clone();
        let topic_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                capability.evidence_topic.value.clone(),
                Plane::Audit,
                request.purpose,
                capability_record_classifications(capability),
                "DENY",
            )?
            .hash
            .clone();
        Ok((capability_invoke_audit_hash, topic_audit_hash))
    }

    pub(crate) fn record_denied_invocation(
        &mut self,
        denial: DeniedInvocationRecord<'_>,
    ) -> Result<EvidenceRecord, FoundationError> {
        let privacy_data_classes = denial.capability.touched_privacy_data_classes().to_vec();
        let run = self
            .foundry_runs
            .reject(
                RunStart::new(
                    denial.request.tenant_id.clone(),
                    denial.request.capability_id.clone(),
                    denial.request.user_id.clone(),
                    denial.capability.required_tier,
                    privacy_data_classes.clone(),
                    denial.tenant.home_region.value.clone(),
                    format!(
                        "deny:{}:{}:{}",
                        denial.reason,
                        denial.request.capability_id,
                        denial.request.started_at_epoch_seconds
                    ),
                    denial.request.started_at_epoch_seconds,
                )
                .map_err(map_run_error)?,
                denial.disposition,
            )
            .map_err(map_run_error)?;
        let run_reject_audit_hash = self
            .audit_chain
            .append_classifications(
                denial.request.tenant_id.clone(),
                "foundry.run.reject",
                Plane::Data,
                denial.request.purpose,
                audit_classifications(),
                "ALLOW",
            )?
            .hash
            .clone();
        let mut evidence_fields = BTreeMap::from([
            (
                "audit_event_hash".to_string(),
                denial.audit_event_hash.clone(),
            ),
            ("decision".to_string(), "DENY".to_string()),
            (
                "evidence_topic".to_string(),
                denial.capability.evidence_topic.value.clone(),
            ),
            ("reason".to_string(), denial.reason.to_string()),
            ("run_id".to_string(), run.run_id.value.clone()),
            (
                "run_reject_audit_event_hash".to_string(),
                run_reject_audit_hash,
            ),
        ]);
        evidence_fields.extend(denial.extra_fields);
        let evidence = self
            .foundry_evidence
            .append(
                denial.request.tenant_id.clone(),
                run.run_id.value,
                None,
                denial.request.capability_id.clone(),
                denial.evidence_kind,
                evidence_fields,
                privacy_data_classes,
                denial.request.started_at_epoch_seconds,
            )
            .map_err(map_evidence_error)?;
        self.outbox
            .publish(
                denial.request.tenant_id.clone(),
                denial.capability.evidence_topic.value.clone(),
                evidence.evidence_id.value.clone(),
                format!("foundry-evidence:{}", evidence.evidence_id.value),
            )
            .map_err(map_eventing_error)?;
        self.audit_chain.append_classifications(
            denial.request.tenant_id.clone(),
            "foundry.evidence.topic.emit",
            Plane::Audit,
            denial.request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        self.audit_chain.append_classifications(
            denial.request.tenant_id.clone(),
            "foundry.evidence.emit",
            Plane::Audit,
            denial.request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        Ok(evidence)
    }
}
