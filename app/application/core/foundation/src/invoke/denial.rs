//! Denial paths for capability invocation.

use crate::*;

impl Foundation {
    /// Always denies. Records the denial trail first; if that trail cannot be
    /// written, its error is returned instead, exactly as the inline `?` did.
    pub(crate) fn deny_principal_mismatch(
        &mut self,
        request: &CapabilityInvocationRequest,
    ) -> FoundationError {
        if let Err(error) = self.record_principal_mismatch_denial(request) {
            return error;
        }
        FoundationError::CapabilityInvocationUnauthorized
    }

    fn record_principal_mismatch_denial(
        &mut self,
        request: &CapabilityInvocationRequest,
    ) -> Result<(), FoundationError> {
        if let (Some(tenant), Some(capability)) = (
            self.tenants.get(&request.tenant_id).cloned(),
            self.capabilities.get(&request.capability_id).cloned(),
        ) {
            let authorization_audit_hash = self
                .audit_chain
                .append_classifications(
                    request.tenant_id.clone(),
                    "cedar.policy.authorize",
                    Plane::Control,
                    request.purpose,
                    vec![DataClass::InternalOnly],
                    "DENY",
                )?
                .hash
                .clone();
            let (capability_invoke_audit_hash, topic_audit_hash) =
                self.append_invocation_denial_audits(&request, &capability)?;
            self.record_denied_invocation(DeniedInvocationRecord {
                request: &request,
                tenant: &tenant,
                capability: &capability,
                disposition: RunDisposition::FailureAuthorization,
                evidence_kind: EvidenceKind::CapabilityInvocation,
                reason: "principal_mismatch",
                audit_event_hash: topic_audit_hash,
                extra_fields: BTreeMap::from([
                    (
                        "authorization_audit_event_hash".to_string(),
                        authorization_audit_hash,
                    ),
                    (
                        "capability_invoke_audit_event_hash".to_string(),
                        capability_invoke_audit_hash,
                    ),
                ]),
            })?;
        }
        Ok(())
    }

    /// Denies an unlicensed capability. `?` still surfaces an audit-write failure ahead of
    /// the denial, exactly as the inline block did.
    pub(crate) fn deny_unlicensed_capability(
        &mut self,
        request: &CapabilityInvocationRequest,
        tenant: &Tenant,
        capability: &Capability,
        invocation_span: &dyn CapabilityInvocationTraceSpan,
    ) -> Result<FoundationError, FoundationError> {
        let license_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "foundry.capability.license",
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
            disposition: RunDisposition::FailureLicense,
            evidence_kind: EvidenceKind::CapabilityInvocation,
            reason: "license",
            audit_event_hash: topic_audit_hash,
            extra_fields: BTreeMap::from([
                (
                    "capability_invoke_audit_event_hash".to_string(),
                    capability_invoke_audit_hash,
                ),
                ("license_audit_event_hash".to_string(), license_audit_hash),
            ]),
        })?;
        emit_invocation_trace(invocation_span, "denied", Some("license"));
        Ok(FoundationError::CapabilityNotLicensed)
    }

    /// Denies a request the policy engine refused. `?` still surfaces an audit-write failure ahead of
    /// the denial, exactly as the inline block did.
    pub(crate) fn deny_unauthorized_invocation(
        &mut self,
        request: &CapabilityInvocationRequest,
        tenant: &Tenant,
        capability: &Capability,
        authorization_decision: AuthorizationDecision,
        authorization_audit_hash: String,
        invocation_span: &dyn CapabilityInvocationTraceSpan,
    ) -> Result<FoundationError, FoundationError> {
        let (capability_invoke_audit_hash, topic_audit_hash) =
            self.append_invocation_denial_audits(request, capability)?;
        self.record_denied_invocation(DeniedInvocationRecord {
            request,
            tenant,
            capability,
            disposition: RunDisposition::FailureAuthorization,
            evidence_kind: EvidenceKind::CapabilityInvocation,
            reason: "authorization",
            audit_event_hash: topic_audit_hash,
            extra_fields: BTreeMap::from([
                (
                    "authorization_audit_event_hash".to_string(),
                    authorization_audit_hash,
                ),
                (
                    "authorization_reason".to_string(),
                    authorization_decision.reason,
                ),
                (
                    "capability_invoke_audit_event_hash".to_string(),
                    capability_invoke_audit_hash,
                ),
            ]),
        })?;
        emit_invocation_trace(invocation_span, "denied", Some("authorization"));
        Ok(FoundationError::CapabilityInvocationUnauthorized)
    }

    /// Denies a request above the tenant's autonomy ceiling. `?` still surfaces an audit-write failure ahead of
    /// the denial, exactly as the inline block did.
    pub(crate) fn deny_autonomy_ceiling(
        &mut self,
        request: &CapabilityInvocationRequest,
        tenant: &Tenant,
        capability: &Capability,
        autonomy_decision: &AutonomyDecision,
        autonomy_audit_hash: &str,
        invocation_span: &dyn CapabilityInvocationTraceSpan,
    ) -> Result<FoundationError, FoundationError> {
        let (capability_invoke_audit_hash, topic_audit_hash) =
            self.append_invocation_denial_audits(request, capability)?;
        let mut autonomy_fields = autonomy_decision_fields(autonomy_decision, autonomy_audit_hash);
        autonomy_fields.insert(
            "capability_invoke_audit_event_hash".to_string(),
            capability_invoke_audit_hash,
        );
        self.record_denied_invocation(DeniedInvocationRecord {
            request,
            tenant,
            capability,
            disposition: RunDisposition::FailureAutonomy,
            evidence_kind: EvidenceKind::AutonomyDecision,
            reason: "autonomy",
            audit_event_hash: topic_audit_hash,
            extra_fields: autonomy_fields,
        })?;
        emit_invocation_trace(invocation_span, "denied", Some("autonomy"));
        Ok(FoundationError::AutonomyCeilingExceeded)
    }

    /// Denies a request that fails the data-use boundary. `?` still surfaces an audit-write failure ahead of
    /// the denial, exactly as the inline block did.
    pub(crate) fn deny_data_use(
        &mut self,
        request: &CapabilityInvocationRequest,
        tenant: &Tenant,
        capability: &Capability,
        denial: InvocationDataUseDenial,
        invocation_span: &dyn CapabilityInvocationTraceSpan,
    ) -> Result<FoundationError, FoundationError> {
        let data_use_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "privacy.data-use.evaluate",
                Plane::Control,
                denial.effective_purpose,
                capability_record_classifications(capability),
                "DENY",
            )?
            .hash
            .clone();
        let (capability_invoke_audit_hash, topic_audit_hash) =
            self.append_invocation_denial_audits(request, capability)?;
        let mut data_use_fields =
            data_use_denial_fields(&request, &capability, &denial, capability_invoke_audit_hash);
        data_use_fields.insert("data_use_audit_event_hash".to_string(), data_use_audit_hash);
        self.record_denied_invocation(DeniedInvocationRecord {
            request,
            tenant,
            capability,
            disposition: RunDisposition::FailureClass,
            evidence_kind: EvidenceKind::ConsentCheck,
            reason: "data_boundary",
            audit_event_hash: topic_audit_hash,
            extra_fields: data_use_fields,
        })?;
        emit_invocation_trace(invocation_span, "denied", Some("data_boundary"));
        Ok(FoundationError::DataUseNotAllowed)
    }

    /// Denies a request above the capability's projected-cost profile. `?` still surfaces an audit-write failure ahead of
    /// the denial, exactly as the inline block did.
    pub(crate) fn deny_cost_profile(
        &mut self,
        request: &CapabilityInvocationRequest,
        tenant: &Tenant,
        capability: &Capability,
        invocation_span: &dyn CapabilityInvocationTraceSpan,
    ) -> Result<FoundationError, FoundationError> {
        let cost_budget_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "foundry.cost-budget.reserve",
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
            disposition: RunDisposition::FailureBudget,
            evidence_kind: EvidenceKind::CapabilityInvocation,
            reason: "capability_cost_profile",
            audit_event_hash: topic_audit_hash,
            extra_fields: BTreeMap::from([
                (
                    "cost_budget_audit_event_hash".to_string(),
                    cost_budget_audit_hash,
                ),
                (
                    "capability_invoke_audit_event_hash".to_string(),
                    capability_invoke_audit_hash,
                ),
                (
                    "capability_per_invocation_limit_micros".to_string(),
                    capability
                        .cost_profile()
                        .per_invocation_limit_micros
                        .value
                        .to_string(),
                ),
                (
                    "projected_cost_micros".to_string(),
                    request.projected_cost_micros.to_string(),
                ),
            ]),
        })?;
        emit_invocation_trace(invocation_span, "denied", Some("capability_cost_profile"));
        Ok(FoundationError::CostBudgetExceeded)
    }
}
