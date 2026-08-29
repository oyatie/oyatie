//! Reserving capacity for a granted invocation: the cost-budget checks,
//! the provider route, and the budget reservation itself.

use crate::*;

use super::gate::InvocationDenial;

/// Capacity resolved for one invocation, ready for the run to start.
pub(crate) struct ReservedCapacity {
    pub cost_budget_warning: Option<BudgetWarning>,
    pub provider_route: ProviderRoute,
    pub provider_id: String,
    pub provider_route_audit_hash: String,
    pub reservation: check_cost_budget::BudgetReservation,
}

impl Foundation {
    pub(crate) fn reserve_invocation_capacity(
        &mut self,
        request: &CapabilityInvocationRequest,
        tenant: &Tenant,
        capability: &Capability,
        invocation_span: &dyn CapabilityInvocationTraceSpan,
    ) -> Result<ReservedCapacity, FoundationError> {
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
                return Err(self.deny_invocation(
                    request,
                    tenant,
                    capability,
                    invocation_span,
                    InvocationDenial {
                        audit_surface: "foundry.cost-budget.reserve",
                        audit_hash_field: "cost_budget_audit_event_hash",
                        disposition: RunDisposition::FailureBudget,
                        evidence_kind: EvidenceKind::CapabilityInvocation,
                        reason: "budget",
                        trace_label: "budget",
                        error: map_budget_error(error),
                    },
                )?);
            }
        };
        let budget_snapshot = match self.cost_budgets.snapshot(&budget_scope) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                return Err(self.deny_invocation(
                    request,
                    tenant,
                    capability,
                    invocation_span,
                    InvocationDenial {
                        audit_surface: "foundry.cost-budget.reserve",
                        audit_hash_field: "cost_budget_audit_event_hash",
                        disposition: RunDisposition::FailureBudget,
                        evidence_kind: EvidenceKind::CapabilityInvocation,
                        reason: "budget_snapshot",
                        trace_label: "budget_snapshot",
                        error: map_budget_error(error),
                    },
                )?);
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
                return Err(self.deny_invocation(
                    request,
                    tenant,
                    capability,
                    invocation_span,
                    InvocationDenial {
                        audit_surface: "foundry.provider.route",
                        audit_hash_field: "provider_route_audit_event_hash",
                        disposition: RunDisposition::FailureProvider,
                        evidence_kind: EvidenceKind::CapabilityInvocation,
                        reason: "provider_route",
                        trace_label: "provider_route",
                        error: map_adapter_error(error),
                    },
                )?);
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
                return Err(self.deny_invocation(
                    request,
                    tenant,
                    capability,
                    invocation_span,
                    InvocationDenial {
                        audit_surface: "foundry.cost-budget.reserve",
                        audit_hash_field: "cost_budget_audit_event_hash",
                        disposition: RunDisposition::FailureBudget,
                        evidence_kind: EvidenceKind::CapabilityInvocation,
                        reason: "budget",
                        trace_label: "budget_reserve",
                        error: map_budget_error(error),
                    },
                )?);
            }
        };
        Ok(ReservedCapacity {
            cost_budget_warning,
            provider_route,
            provider_id,
            provider_route_audit_hash,
            reservation,
        })
    }
}
