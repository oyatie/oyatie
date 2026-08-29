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
            return Err(self.deny_principal_mismatch(&request));
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
            return Err(self.deny_unlicensed_capability(
                &request,
                &tenant,
                &capability,
                invocation_span.as_ref(),
            )?);
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
            return Err(self.deny_unauthorized_invocation(
                &request,
                &tenant,
                &capability,
                authorization_decision,
                authorization_audit_hash,
                invocation_span.as_ref(),
            )?);
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
            return Err(self.deny_autonomy_ceiling(
                &request,
                &tenant,
                &capability,
                &autonomy_decision,
                &autonomy_audit_hash,
                invocation_span.as_ref(),
            )?);
        }
        if let Err(denial) = evaluate_invocation_data_use(
            &capability,
            &request,
            self.consent_scopes.get(&request.tenant_id),
        ) {
            return Err(self.deny_data_use(
                &request,
                &tenant,
                &capability,
                denial,
                invocation_span.as_ref(),
            )?);
        }
        if !capability.allows_projected_invocation_cost(request.projected_cost_micros) {
            return Err(self.deny_cost_profile(
                &request,
                &tenant,
                &capability,
                invocation_span.as_ref(),
            )?);
        }

        let ReservedCapacity {
            cost_budget_warning,
            provider_route,
            provider_id,
            provider_route_audit_hash,
            reservation,
        } = self.reserve_invocation_capacity(
            &request,
            &tenant,
            &capability,
            invocation_span.as_ref(),
        )?;

        let StartedInvocation {
            run,
            completed_step,
        } = self.start_invocation(InvocationStart {
            request: &request,
            tenant: &tenant,
            capability: &capability,
            reservation: &reservation,
            autonomy_decision: &autonomy_decision,
            pre_break_glass_autonomy_decision: &pre_break_glass_autonomy_decision,
            privacy_data_classes: &privacy_data_classes,
            provider_id: &provider_id,
            autonomy_break_glass: &autonomy_break_glass,
            autonomy_audit_hash: &autonomy_audit_hash,
            break_glass_invoke_audit_hash: &break_glass_invoke_audit_hash,
        })?;
        self.complete_invocation(InvocationCompletion {
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
        })
    }
}

mod complete;
mod denial;
mod gate;
mod reserve;
mod start;

use complete::InvocationCompletion;
use gate::InvocationDenial;
use reserve::ReservedCapacity;
use start::{InvocationStart, StartedInvocation};
