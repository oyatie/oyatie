//! Capability invocation as a principal.

use crate::*;

use crate::Foundation;

impl Foundation {
    pub fn invoke_capability_as_principal(
        &mut self,
        principal: CapabilityInvocationPrincipal,
        request: CapabilityInvocationRequest,
    ) -> Result<InvocationReceipt, FoundationError> {
        if principal.tenant_id != request.tenant_id || principal.user_id != request.user_id {
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
            return Err(FoundationError::CapabilityInvocationUnauthorized);
        }
        let user = self
            .require_user(&request.tenant_id, &request.user_id)?
            .clone();
        let tenant = self.require_tenant(&request.tenant_id)?.clone();
        let capability = self
            .capabilities
            .get(&request.capability_id)
            .ok_or(FoundationError::CapabilityNotFound)?
            .clone();
        let data_classifications = capability_record_classifications(&capability);
        let privacy_data_classes = capability.touched_privacy_data_classes().to_vec();
        let policy = self
            .tenant_policies
            .get(&request.tenant_id)
            .ok_or(FoundationError::TenantNotFound)?;
        let touched_data_classes = telemetry_data_classifications_label(&data_classifications);
        let cell_id = self
            .cells
            .get(&request.tenant_id)
            .map(|cell_binding| cell_binding.cell_id.value.clone());
        let invocation_span =
            self.observability
                .start_capability_invocation(&CapabilityInvocationTraceContext {
                    service_name: "foundation-app".to_string(),
                    tenant_id: request.tenant_id.clone(),
                    tenant_region: tenant.home_region.value.clone(),
                    cell_id,
                    capability_id: request.capability_id.clone(),
                    data_classes_touched: touched_data_classes,
                    operation_name: CAPABILITY_INVOCATION_OPERATION_NAME.to_string(),
                    provider_name: FOUNDRY_PROVIDER_NAME.to_string(),
                });
        emit_invocation_trace(invocation_span.as_ref(), "started", None);
        if !self
            .capabilities
            .is_licensed_for_tenant(&request.tenant_id, &request.capability_id)
        {
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
                self.append_invocation_denial_audits(&request, &capability)?;
            self.record_denied_invocation(DeniedInvocationRecord {
                request: &request,
                tenant: &tenant,
                capability: &capability,
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
            emit_invocation_trace(invocation_span.as_ref(), "denied", Some("license"));
            return Err(FoundationError::CapabilityNotLicensed);
        }

        let mut autonomy_decision = policy.evaluate_with_context(
            &capability,
            principal.autonomy_ceiling,
            &tenant.regulatory_packs.value,
            request.subject_class,
        );
        let pre_break_glass_autonomy_decision = autonomy_decision.clone();
        let autonomy_break_glass = if autonomy_decision.allowed() {
            None
        } else {
            self.foundation_bypass_ledger
                .active_autonomy_break_glass_for(
                    &request.tenant_id,
                    &request.capability_id,
                    capability.required_tier,
                    epoch_seconds_to_epoch_days(request.started_at_epoch_seconds),
                )
                .cloned()
        };
        if let Some(break_glass) = &autonomy_break_glass {
            apply_autonomy_break_glass(&mut autonomy_decision, break_glass);
        }
        invocation_span
            .record_autonomy_tier(autonomy_tier_label(autonomy_decision.effective_ceiling));
        let authorization_decision = self.policies.authorize(&AuthorizationQuery {
            subject: AuthorizationSubject {
                tenant_id: request.tenant_id.clone(),
                roles: user.roles.value.clone(),
            },
            action: "foundry.capability.invoke".to_string(),
            resource: format!("capability:{}", request.capability_id),
            attributes: invocation_authorization_attributes(
                &request,
                &capability,
                principal.autonomy_ceiling,
                &autonomy_decision,
                autonomy_break_glass.as_ref(),
                if autonomy_break_glass.is_some() {
                    Some(&pre_break_glass_autonomy_decision)
                } else {
                    None
                },
            ),
        });
        let authorization_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "cedar.policy.authorize",
                Plane::Control,
                request.purpose,
                vec![DataClass::InternalOnly],
                if authorization_decision.allowed {
                    "ALLOW"
                } else {
                    "DENY"
                },
            )?
            .hash
            .clone();
        if !authorization_decision.allowed {
            let (capability_invoke_audit_hash, topic_audit_hash) =
                self.append_invocation_denial_audits(&request, &capability)?;
            self.record_denied_invocation(DeniedInvocationRecord {
                request: &request,
                tenant: &tenant,
                capability: &capability,
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
            emit_invocation_trace(invocation_span.as_ref(), "denied", Some("authorization"));
            return Err(FoundationError::CapabilityInvocationUnauthorized);
        }

        let autonomy_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "foundry.autonomy.decision",
                Plane::Control,
                request.purpose,
                internal_audit_classifications(),
                if autonomy_decision.allowed() {
                    "ALLOW"
                } else {
                    "DENY"
                },
            )?
            .hash
            .clone();
        let break_glass_invoke_audit_hash = match autonomy_break_glass.as_ref() {
            Some(break_glass) => Some(
                self.audit_chain
                    .append_classifications(
                        break_glass.tenant_id.value.clone(),
                        "foundry.autonomy.break_glass.invoke",
                        Plane::Control,
                        request.purpose,
                        internal_audit_classifications(),
                        "ALLOW",
                    )?
                    .hash
                    .clone(),
            ),
            None => None,
        };
        if !autonomy_decision.allowed() {
            let (capability_invoke_audit_hash, topic_audit_hash) =
                self.append_invocation_denial_audits(&request, &capability)?;
            let mut autonomy_fields =
                autonomy_decision_fields(&autonomy_decision, &autonomy_audit_hash);
            autonomy_fields.insert(
                "capability_invoke_audit_event_hash".to_string(),
                capability_invoke_audit_hash,
            );
            self.record_denied_invocation(DeniedInvocationRecord {
                request: &request,
                tenant: &tenant,
                capability: &capability,
                disposition: RunDisposition::FailureAutonomy,
                evidence_kind: EvidenceKind::AutonomyDecision,
                reason: "autonomy",
                audit_event_hash: topic_audit_hash,
                extra_fields: autonomy_fields,
            })?;
            emit_invocation_trace(invocation_span.as_ref(), "denied", Some("autonomy"));
            return Err(FoundationError::AutonomyCeilingExceeded);
        }
        if let Err(denial) = evaluate_invocation_data_use(
            &capability,
            &request,
            self.consent_scopes.get(&request.tenant_id),
        ) {
            let data_use_audit_hash = self
                .audit_chain
                .append_classifications(
                    request.tenant_id.clone(),
                    "privacy.data-use.evaluate",
                    Plane::Control,
                    denial.effective_purpose,
                    capability_record_classifications(&capability),
                    "DENY",
                )?
                .hash
                .clone();
            let (capability_invoke_audit_hash, topic_audit_hash) =
                self.append_invocation_denial_audits(&request, &capability)?;
            let mut data_use_fields = data_use_denial_fields(
                &request,
                &capability,
                &denial,
                capability_invoke_audit_hash,
            );
            data_use_fields.insert("data_use_audit_event_hash".to_string(), data_use_audit_hash);
            self.record_denied_invocation(DeniedInvocationRecord {
                request: &request,
                tenant: &tenant,
                capability: &capability,
                disposition: RunDisposition::FailureClass,
                evidence_kind: EvidenceKind::ConsentCheck,
                reason: "data_boundary",
                audit_event_hash: topic_audit_hash,
                extra_fields: data_use_fields,
            })?;
            emit_invocation_trace(invocation_span.as_ref(), "denied", Some("data_boundary"));
            return Err(FoundationError::DataUseNotAllowed);
        }
        if !capability.allows_projected_invocation_cost(request.projected_cost_micros) {
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
                self.append_invocation_denial_audits(&request, &capability)?;
            self.record_denied_invocation(DeniedInvocationRecord {
                request: &request,
                tenant: &tenant,
                capability: &capability,
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
            emit_invocation_trace(
                invocation_span.as_ref(),
                "denied",
                Some("capability_cost_profile"),
            );
            return Err(FoundationError::CostBudgetExceeded);
        }

        let budget_scope = BudgetScope::new(
            request.tenant_id.clone(),
            request.capability_id.clone(),
            request.budget_window_id.clone(),
        )
        .map_err(map_budget_error)?;
        let cost_budget_warning = match self
            .cost_budgets
            .evaluate(&budget_scope, request.projected_cost_micros)
        {
            Ok(decision) => decision.warning.value,
            Err(error) => {
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
                    self.append_invocation_denial_audits(&request, &capability)?;
                self.record_denied_invocation(DeniedInvocationRecord {
                    request: &request,
                    tenant: &tenant,
                    capability: &capability,
                    disposition: RunDisposition::FailureBudget,
                    evidence_kind: EvidenceKind::CapabilityInvocation,
                    reason: "budget",
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
                    ]),
                })?;
                emit_invocation_trace(invocation_span.as_ref(), "denied", Some("budget"));
                return Err(map_budget_error(error));
            }
        };
        let budget_snapshot = match self.cost_budgets.snapshot(&budget_scope) {
            Ok(snapshot) => snapshot,
            Err(error) => {
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
                    self.append_invocation_denial_audits(&request, &capability)?;
                self.record_denied_invocation(DeniedInvocationRecord {
                    request: &request,
                    tenant: &tenant,
                    capability: &capability,
                    disposition: RunDisposition::FailureBudget,
                    evidence_kind: EvidenceKind::CapabilityInvocation,
                    reason: "budget_snapshot",
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
                    ]),
                })?;
                emit_invocation_trace(invocation_span.as_ref(), "denied", Some("budget_snapshot"));
                return Err(map_budget_error(error));
            }
        };
        let provider_route = match Self::resolve_foundation_local_provider_route(
            &tenant,
            &capability,
            &budget_snapshot,
            &request,
            capability.touched_privacy_data_classes(),
        ) {
            Ok(route) => route,
            Err(error) => {
                let provider_route_audit_hash = self
                    .audit_chain
                    .append_classifications(
                        request.tenant_id.clone(),
                        "foundry.provider.route",
                        Plane::Data,
                        request.purpose,
                        internal_audit_classifications(),
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
                    disposition: RunDisposition::FailureProvider,
                    evidence_kind: EvidenceKind::CapabilityInvocation,
                    reason: "provider_route",
                    audit_event_hash: topic_audit_hash,
                    extra_fields: BTreeMap::from([
                        (
                            "provider_route_audit_event_hash".to_string(),
                            provider_route_audit_hash,
                        ),
                        (
                            "capability_invoke_audit_event_hash".to_string(),
                            capability_invoke_audit_hash,
                        ),
                    ]),
                })?;
                emit_invocation_trace(invocation_span.as_ref(), "denied", Some("provider_route"));
                return Err(map_adapter_error(error));
            }
        };
        let provider_id = provider_route
            .primary()
            .map_err(map_adapter_error)?
            .id
            .value
            .value
            .clone();
        let provider_route_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "foundry.provider.route",
                Plane::Data,
                request.purpose,
                internal_audit_classifications(),
                "ALLOW",
            )?
            .hash
            .clone();
        let reservation = match self
            .cost_budgets
            .reserve(&budget_scope, request.projected_cost_micros)
        {
            Ok(reservation) => reservation,
            Err(error) => {
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
                    self.append_invocation_denial_audits(&request, &capability)?;
                self.record_denied_invocation(DeniedInvocationRecord {
                    request: &request,
                    tenant: &tenant,
                    capability: &capability,
                    disposition: RunDisposition::FailureBudget,
                    evidence_kind: EvidenceKind::CapabilityInvocation,
                    reason: "budget",
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
                    ]),
                })?;
                emit_invocation_trace(invocation_span.as_ref(), "denied", Some("budget_reserve"));
                return Err(map_budget_error(error));
            }
        };

        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.cost-budget.reserve",
            Plane::Control,
            request.purpose,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        let run = match self.foundry_runs.start(
            RunStart::new(
                request.tenant_id.clone(),
                request.capability_id.clone(),
                request.user_id.clone(),
                autonomy_decision.effective_ceiling,
                privacy_data_classes.clone(),
                tenant.home_region.value.clone(),
                reservation.reservation_id.value.clone(),
                request.started_at_epoch_seconds,
            )
            .map_err(map_run_error)?,
        ) {
            Ok(run) => run,
            Err(error) => {
                let primary_error = map_run_error(error);
                return Err(self.settle_failed_invocation(
                    &request,
                    Some(&reservation.reservation_id.value),
                    None,
                    RunDisposition::FailureProvider,
                    primary_error,
                )?);
            }
        };
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.run.start",
            Plane::Data,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        let mut autonomy_evidence_fields =
            autonomy_decision_fields(&autonomy_decision, &autonomy_audit_hash);
        append_break_glass_evidence_fields(
            &mut autonomy_evidence_fields,
            autonomy_break_glass.as_ref(),
            if autonomy_break_glass.is_some() {
                Some(&pre_break_glass_autonomy_decision)
            } else {
                None
            },
            break_glass_invoke_audit_hash.as_deref(),
        );
        autonomy_evidence_fields.insert("run_id".to_string(), run.run_id.value.clone());
        autonomy_evidence_fields.insert(
            "evidence_topic".to_string(),
            capability.evidence_topic.value.clone(),
        );
        let autonomy_evidence = match self.foundry_evidence.append(
            request.tenant_id.clone(),
            run.run_id.value.clone(),
            None,
            request.capability_id.clone(),
            EvidenceKind::AutonomyDecision,
            autonomy_evidence_fields,
            privacy_data_classes.clone(),
            request.started_at_epoch_seconds,
        ) {
            Ok(evidence) => evidence,
            Err(error) => {
                let primary_error = map_evidence_error(error);
                return Err(self.settle_failed_invocation(
                    &request,
                    Some(&reservation.reservation_id.value),
                    Some(&run.run_id.value),
                    RunDisposition::FailureProvider,
                    primary_error,
                )?);
            }
        };
        if let Err(error) = self.outbox.publish(
            request.tenant_id.clone(),
            capability.evidence_topic.value.clone(),
            autonomy_evidence.evidence_id.value.clone(),
            format!("foundry-evidence:{}", autonomy_evidence.evidence_id.value),
        ) {
            let primary_error = map_eventing_error(error);
            return Err(self.settle_failed_invocation(
                &request,
                Some(&reservation.reservation_id.value),
                Some(&run.run_id.value),
                RunDisposition::FailureProvider,
                primary_error,
            )?);
        }
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.evidence.topic.emit",
            Plane::Audit,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.evidence.emit",
            Plane::Audit,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        let step = match self.foundry_steps.start(
            StepStart::new(
                run.run_id.value.clone(),
                StepKind::ProviderCall,
                provider_id.clone(),
                Some(FOUNDATION_LOCAL_MODEL_REF.into()),
                None,
                None,
                privacy_data_classes.clone(),
                request.started_at_epoch_seconds,
            )
            .map_err(map_step_error)?,
        ) {
            Ok(step) => step,
            Err(error) => {
                let primary_error = map_step_error(error);
                return Err(self.settle_failed_invocation(
                    &request,
                    Some(&reservation.reservation_id.value),
                    Some(&run.run_id.value),
                    RunDisposition::FailureProvider,
                    primary_error,
                )?);
            }
        };
        let completed_step = match self.foundry_steps.complete(
            &step.step_id.value,
            StepDisposition::Succeeded,
            1,
            request.started_at_epoch_seconds.saturating_add(1),
        ) {
            Ok(step) => step,
            Err(error) => {
                let primary_error = map_step_error(error);
                return Err(self.settle_failed_invocation(
                    &request,
                    Some(&reservation.reservation_id.value),
                    Some(&run.run_id.value),
                    RunDisposition::FailureProvider,
                    primary_error,
                )?);
            }
        };
        let provider_call_idempotency_key = format!(
            "provider-call:{}:{}:{}:{:03}",
            run.run_id.value,
            completed_step.step_id.value,
            provider_id,
            FOUNDATION_LOCAL_PROVIDER_ATTEMPT
        );
        let provider_call_receipt = match ProviderCallReceipt::from_route(
            &provider_route,
            provider_call_idempotency_key,
            FOUNDATION_LOCAL_PROVIDER_ATTEMPT,
            FOUNDATION_LOCAL_MODEL_REF.into(),
            tenant.home_region.value.clone(),
        ) {
            Ok(receipt) => receipt,
            Err(error) => {
                self.audit_chain.append_classifications(
                    request.tenant_id.clone(),
                    "foundry.provider.call",
                    Plane::Data,
                    request.purpose,
                    internal_audit_classifications(),
                    "DENY",
                )?;
                let primary_error = map_adapter_error(error);
                return Err(self.settle_failed_invocation(
                    &request,
                    Some(&reservation.reservation_id.value),
                    Some(&run.run_id.value),
                    RunDisposition::FailureProvider,
                    primary_error,
                )?);
            }
        };
        let provider_call_audit_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "foundry.provider.call",
                Plane::Data,
                request.purpose,
                data_classifications.clone(),
                "ALLOW",
            )?
            .hash
            .clone();
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.step.emit",
            Plane::Data,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        let committed = match self.cost_budgets.commit(&reservation.reservation_id.value) {
            Ok(committed) => committed,
            Err(error) => {
                let primary_error = map_budget_error(error);
                return Err(self.settle_failed_invocation(
                    &request,
                    Some(&reservation.reservation_id.value),
                    Some(&run.run_id.value),
                    RunDisposition::FailureBudget,
                    primary_error,
                )?);
            }
        };
        let capability_invoke_event_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                "foundry.capability.invoke",
                Plane::Data,
                request.purpose,
                data_classifications.clone(),
                "ALLOW",
            )?
            .hash
            .clone();
        if let Err(error) = self.foundry_runs.complete(
            &run.run_id.value,
            RunDisposition::Success,
            request.started_at_epoch_seconds.saturating_add(1),
        ) {
            let primary_error = map_run_error(error);
            return Err(self.settle_failed_invocation(
                &request,
                None,
                Some(&run.run_id.value),
                RunDisposition::FailureProvider,
                primary_error,
            )?);
        }
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.run.complete",
            Plane::Data,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        let evidence_event_hash = self
            .audit_chain
            .append_classifications(
                request.tenant_id.clone(),
                capability.evidence_topic.value.clone(),
                Plane::Audit,
                request.purpose,
                data_classifications.clone(),
                "ALLOW",
            )?
            .hash
            .clone();
        let mut evidence_fields = BTreeMap::new();
        evidence_fields.insert("audit_event_hash".to_string(), evidence_event_hash.clone());
        evidence_fields.insert(
            "capability_invoke_audit_event_hash".to_string(),
            capability_invoke_event_hash,
        );
        evidence_fields.insert(
            "provider_route_audit_event_hash".to_string(),
            provider_route_audit_hash,
        );
        evidence_fields.insert(
            "provider_call_audit_event_hash".to_string(),
            provider_call_audit_hash,
        );
        evidence_fields.insert(
            "cost_reservation_id".to_string(),
            reservation.reservation_id.value.clone(),
        );
        evidence_fields.insert(
            "evidence_topic".to_string(),
            capability.evidence_topic.value.clone(),
        );
        evidence_fields.insert("run_id".to_string(), run.run_id.value.clone());
        evidence_fields.insert("step_id".to_string(), completed_step.step_id.value.clone());
        evidence_fields.insert(
            "provider_id".to_string(),
            provider_call_receipt.provider_id.value.value.clone(),
        );
        evidence_fields.insert(
            "provider_mode".to_string(),
            format!("{:?}", provider_call_receipt.provider_mode.value),
        );
        evidence_fields.insert(
            "provider_call_receipt_id".to_string(),
            provider_call_receipt.receipt_id.value.clone(),
        );
        evidence_fields.insert(
            "provider_call_idempotency_key".to_string(),
            provider_call_receipt.idempotency_key.value.clone(),
        );
        evidence_fields.insert(
            "provider_call_attempt".to_string(),
            provider_call_receipt.attempt.value.to_string(),
        );
        evidence_fields.insert(
            "provider_region".to_string(),
            provider_call_receipt.provider_region.value.clone(),
        );
        evidence_fields.insert(
            "provider_model_ref".to_string(),
            provider_call_receipt.model_ref.value.clone(),
        );
        evidence_fields.insert(
            "provider_projected_cost_micros".to_string(),
            provider_call_receipt
                .projected_cost_micros
                .value
                .to_string(),
        );
        evidence_fields.insert(
            "provider_p95_latency_ms".to_string(),
            provider_call_receipt.p95_latency_ms.value.to_string(),
        );
        let evidence = match self.foundry_evidence.append(
            request.tenant_id.clone(),
            run.run_id.value.clone(),
            Some(completed_step.step_id.value.clone()),
            request.capability_id.clone(),
            EvidenceKind::CapabilityInvocation,
            evidence_fields,
            privacy_data_classes,
            request.started_at_epoch_seconds.saturating_add(1),
        ) {
            Ok(evidence) => evidence,
            Err(error) => {
                let primary_error = map_evidence_error(error);
                return Err(self.settle_failed_invocation(
                    &request,
                    None,
                    Some(&run.run_id.value),
                    RunDisposition::FailureProvider,
                    primary_error,
                )?);
            }
        };
        if let Err(error) = self.outbox.publish(
            request.tenant_id.clone(),
            capability.evidence_topic.value.clone(),
            evidence.evidence_id.value.clone(),
            format!("foundry-evidence:{}", evidence.evidence_id.value),
        ) {
            let primary_error = map_eventing_error(error);
            return Err(self.settle_failed_invocation(
                &request,
                None,
                Some(&run.run_id.value),
                RunDisposition::FailureProvider,
                primary_error,
            )?);
        }
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.evidence.topic.emit",
            Plane::Audit,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        self.audit_chain.append_classifications(
            request.tenant_id.clone(),
            "foundry.evidence.emit",
            Plane::Audit,
            request.purpose,
            audit_classifications(),
            "ALLOW",
        )?;
        emit_invocation_trace(invocation_span.as_ref(), "succeeded", None);
        Ok(InvocationReceipt {
            tenant_id: request.tenant_id,
            user_id: request.user_id,
            capability_id: request.capability_id,
            evidence_event_hash,
            cost_reservation_id: Some(committed.reservation_id.value),
            cost_budget_warning,
            run_id: Some(run.run_id.value),
            foundry_step_id: Some(completed_step.step_id.value),
            foundry_evidence_id: Some(evidence.evidence_id.value),
        })
    }
}
