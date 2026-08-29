//! Starting a granted invocation: the run, its autonomy evidence, and the
//! step that carries the provider call.

use crate::*;

/// Inputs the gates resolved, borrowed for the duration of the start phase.
pub(crate) struct InvocationStart<'a> {
    pub request: &'a CapabilityInvocationRequest,
    pub tenant: &'a Tenant,
    pub capability: &'a Capability,
    pub privacy_data_classes: &'a [PrivacyDataClass],
    pub provider_id: &'a str,
    pub reservation: &'a check_cost_budget::BudgetReservation,
    pub autonomy_decision: &'a AutonomyDecision,
    pub pre_break_glass_autonomy_decision: &'a AutonomyDecision,
    pub autonomy_break_glass: &'a Option<AutonomyBreakGlass>,
    pub autonomy_audit_hash: &'a str,
    pub break_glass_invoke_audit_hash: &'a Option<String>,
}

/// The run and step a started invocation is carried by.
pub(crate) struct StartedInvocation {
    pub run: Run,
    pub completed_step: Step,
}

impl Foundation {
    pub(crate) fn start_invocation(
        &mut self,
        start: InvocationStart<'_>,
    ) -> Result<StartedInvocation, FoundationError> {
        let InvocationStart {
            request,
            tenant,
            capability,
            privacy_data_classes,
            provider_id,
            reservation,
            autonomy_decision,
            pre_break_glass_autonomy_decision,
            autonomy_break_glass,
            autonomy_audit_hash,
            break_glass_invoke_audit_hash,
        } = start;
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
                privacy_data_classes.to_vec(),
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
            privacy_data_classes.to_vec(),
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
                provider_id.to_string(),
                Some(FOUNDATION_LOCAL_MODEL_REF.into()),
                None,
                None,
                privacy_data_classes.to_vec(),
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
        Ok(StartedInvocation {
            run,
            completed_step,
        })
    }
}
