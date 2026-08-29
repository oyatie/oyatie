//! MCP gateway discovery, invocation, and rate/cost configuration.

use crate::*;

use crate::Foundation;

impl Foundation {
    pub fn discover_mcp_gateway(
        &mut self,
        request: McpDiscoveryRequest,
    ) -> Result<McpGatewayDescriptor, FoundationError> {
        let tenant = self.require_tenant(&request.tenant_id)?.clone();
        let endpoint = McpTenantEndpoint::new(
            tenant.id.clone(),
            tenant.home_region.value.clone(),
            request.tld,
            request.authorization_server,
        )
        .map_err(map_mcp_error)?;
        let principal =
            self.mcp_principal(&endpoint, request.access_token, request.now_epoch_seconds)?;

        if let Err(error) = McpGatewayDescriptor::new(endpoint.clone(), &principal, &[]) {
            self.audit_chain.append_classifications(
                tenant.id,
                "foundry.mcp.tools.list",
                Plane::Control,
                Purpose::CapabilityInvocation,
                vec![DataClass::InternalOnly],
                "DENY",
            )?;
            return Err(map_mcp_error(error));
        }

        let capabilities = self
            .capabilities
            .discover_for_tenant(&endpoint.tenant_id.value, principal.autonomy_ceiling)
            .map_err(map_capability_error)?;
        let descriptor = McpGatewayDescriptor::new(endpoint, &principal, &capabilities)
            .map_err(map_mcp_error)?;
        self.audit_chain.append_classifications(
            descriptor.endpoint.tenant_id.value.clone(),
            "foundry.mcp.tools.list",
            Plane::Control,
            Purpose::CapabilityInvocation,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(descriptor)
    }

    pub fn invoke_capability_via_mcp(
        &mut self,
        request: McpToolCallRequest,
    ) -> Result<InvocationReceipt, FoundationError> {
        self.require_user(&request.tenant_id, &request.user_id)?;
        let tenant = self.require_tenant(&request.tenant_id)?.clone();
        let endpoint = McpTenantEndpoint::new(
            tenant.id.clone(),
            tenant.home_region.value.clone(),
            request.tld,
            request.authorization_server,
        )
        .map_err(map_mcp_error)?;
        let principal = self.mcp_principal(
            &endpoint,
            request.access_token,
            request.started_at_epoch_seconds,
        )?;
        if endpoint.tenant_id.value != principal.tenant_id.value {
            self.audit_chain.append_classifications(
                tenant.id,
                "foundry.mcp.tool.call",
                Plane::Data,
                request.purpose,
                vec![DataClass::InternalOnly],
                "DENY",
            )?;
            return Err(FoundationError::McpAccessDenied);
        }
        if principal.subject_id.value != request.user_id {
            self.audit_chain.append_classifications(
                tenant.id,
                "foundry.mcp.tool.call",
                Plane::Data,
                request.purpose,
                vec![DataClass::InternalOnly],
                "DENY",
            )?;
            return Err(FoundationError::McpAccessDenied);
        }

        let visible_capability_opt = self
            .capabilities
            .discover_for_tenant(&endpoint.tenant_id.value, principal.autonomy_ceiling)
            .map_err(map_capability_error)?
            .into_iter()
            .find(|capability| capability.id == request.tool_name);
        let visible_capability = match visible_capability_opt {
            Some(capability) => capability,
            None => {
                self.audit_chain.append_classifications(
                    tenant.id.clone(),
                    "foundry.mcp.tool.call",
                    Plane::Data,
                    request.purpose,
                    vec![DataClass::InternalOnly],
                    "DENY",
                )?;
                return Err(FoundationError::McpAccessDenied);
            }
        };
        let tool = project_capability_tool(&visible_capability).map_err(map_mcp_error)?;
        if let Err(error) = authorize_tool_call(&endpoint, &principal, &tool) {
            self.audit_chain.append_classifications(
                tenant.id,
                "foundry.mcp.tool.call",
                Plane::Data,
                request.purpose,
                vec![DataClass::InternalOnly],
                "DENY",
            )?;
            return Err(map_mcp_error(error));
        }

        if let Err(error) = self.mcp_rate_limiter.check_and_record(
            &endpoint.tenant_id.value,
            &tool.name.value,
            request.started_at_epoch_seconds,
        ) {
            self.audit_chain.append_classifications(
                tenant.id,
                "foundry.mcp.rate-limit",
                Plane::Data,
                request.purpose,
                vec![DataClass::InternalOnly, DataClass::BehavioralTenantProduct],
                "DENY",
            )?;
            return Err(map_mcp_error(error));
        }

        self.audit_chain.append_classifications(
            endpoint.tenant_id.value.clone(),
            "foundry.mcp.tool.call",
            Plane::Data,
            request.purpose,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        self.invoke_capability_as_principal(
            CapabilityInvocationPrincipal {
                tenant_id: endpoint.tenant_id.value.clone(),
                user_id: principal.subject_id.value,
                autonomy_ceiling: principal.autonomy_ceiling,
            },
            CapabilityInvocationRequest {
                tenant_id: endpoint.tenant_id.value,
                user_id: request.user_id,
                capability_id: request.tool_name,
                purpose: request.purpose,
                subject_class: request.subject_class,
                budget_window_id: request.budget_window_id,
                projected_cost_micros: request.projected_cost_micros,
                started_at_epoch_seconds: request.started_at_epoch_seconds,
            },
        )
    }

    pub fn configure_mcp_rate_limit(
        &mut self,
        policy: McpRateLimitPolicy,
    ) -> Result<(), FoundationError> {
        self.mcp_rate_limiter.set_policy(policy);
        self.audit_chain.append_classifications(
            "ten_system",
            "foundry.mcp.rate-limit.configure",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(())
    }

    pub fn configure_tenant_cost_budget(
        &mut self,
        registration: CostBudgetRegistration,
    ) -> Result<(), FoundationError> {
        self.require_tenant(&registration.tenant_id)?;
        let ceiling = BudgetCeiling::new(
            registration.monthly_limit_micros,
            registration.per_invocation_limit_micros,
            registration.warning_threshold_percent,
        )
        .map_err(map_budget_error)?;

        if let Some(capability_id) = registration.capability_id {
            if self.capabilities.get(&capability_id).is_none() {
                return Err(FoundationError::CapabilityNotFound);
            }
            let scope = BudgetScope::new(
                registration.tenant_id.clone(),
                capability_id,
                registration.window_id.clone(),
            )
            .map_err(map_budget_error)?;
            self.cost_budgets
                .configure_capability_ceiling(scope, ceiling)
                .map_err(map_budget_error)?;
        } else {
            self.cost_budgets
                .configure_tenant_ceiling(
                    registration.tenant_id.clone(),
                    registration.window_id.clone(),
                    ceiling,
                )
                .map_err(map_budget_error)?;
        }

        self.audit_chain.append_classifications(
            registration.tenant_id,
            "foundry.cost-budget.configure",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(())
    }
}
