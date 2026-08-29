//! Completing a granted invocation: the provider call, budget commit,
//! run completion, evidence, and the receipt handed back to the caller.

use crate::*;

/// Everything the gates and reservation resolved, handed to completion as
/// one value so the phase boundary is explicit rather than positional.
pub(crate) struct InvocationCompletion {
    pub request: CapabilityInvocationRequest,
    pub tenant: Tenant,
    pub capability: Capability,
    pub data_classifications: Vec<DataClassification>,
    pub privacy_data_classes: Vec<PrivacyDataClass>,
    pub cost_budget_warning: Option<BudgetWarning>,
    pub provider_route: ProviderRoute,
    pub provider_id: String,
    pub provider_route_audit_hash: String,
    pub reservation: check_cost_budget::BudgetReservation,
    pub run: Run,
    pub completed_step: Step,
    pub invocation_span: Box<dyn CapabilityInvocationTraceSpan>,
}

impl Foundation {
    pub(crate) fn complete_invocation(
        &mut self,
        completion: InvocationCompletion,
    ) -> Result<InvocationReceipt, FoundationError> {
        let InvocationCompletion {
            request,
            tenant,
            capability,
            data_classifications,
            privacy_data_classes,
            cost_budget_warning,
            provider_route,
            provider_id,
            provider_route_audit_hash,
            reservation,
            run,
            completed_step,
            invocation_span,
        } = completion;
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
