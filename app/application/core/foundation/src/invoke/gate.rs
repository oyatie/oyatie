//! The shared shape of a gate denial.
//!
//! Every gate that refuses an invocation writes the same trail: one `DENY`
//! audit on the gate's own surface, the pair of invocation-denial audits, a
//! denied-invocation record carrying both hashes, and a `denied` trace. The
//! gates differ only in which surface, disposition, reason and error they
//! name, so those are data here rather than repeated code.

use crate::*;

/// What distinguishes one gate's denial from another's.
pub(crate) struct InvocationDenial {
    /// Audit surface the gate's own `DENY` event is appended to.
    pub audit_surface: &'static str,
    /// Key the gate's audit hash is recorded under on the denial record.
    pub audit_hash_field: &'static str,
    pub disposition: RunDisposition,
    pub evidence_kind: EvidenceKind,
    pub reason: &'static str,
    pub trace_label: &'static str,
    /// The error the caller returns once the trail is written.
    pub error: FoundationError,
}

impl Foundation {
    /// Writes a gate denial trail and yields the error to return.
    ///
    /// `?` still surfaces an audit-write failure ahead of the denial, exactly
    /// as the inline blocks did.
    pub(crate) fn deny_invocation(
        &mut self,
        request: &CapabilityInvocationRequest,
        tenant: &Tenant,
        capability: &Capability,
        invocation_span: &dyn CapabilityInvocationTraceSpan,
        denial: InvocationDenial,
    ) -> Result<FoundationError, FoundationError> {
        let gate_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                denial.audit_surface,
                Plane::Control,
                request.purpose,
                vec![DataClass::InternalOnly],
                "DENY",
            )?
            .hash
            .clone();
        let (capability_invoke_audit_hash, topic_audit_hash) =
            self.append_invocation_denial_audits(request, capability)?;
        self.record_denied_invocation(DeniedInvocationRecord {
            request,
            tenant,
            capability,
            disposition: denial.disposition,
            evidence_kind: denial.evidence_kind,
            reason: denial.reason,
            audit_event_hash: topic_audit_hash,
            extra_fields: BTreeMap::from([
                (denial.audit_hash_field.to_string(), gate_audit_hash),
                (
                    "capability_invoke_audit_event_hash".to_string(),
                    capability_invoke_audit_hash,
                ),
            ]),
        })?;
        emit_invocation_trace(invocation_span, "denied", Some(denial.trace_label));
        Ok(denial.error)
    }
}
