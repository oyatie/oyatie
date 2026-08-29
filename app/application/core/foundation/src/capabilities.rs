//! Capability registration, eval sets, and tenant grants.

use crate::*;

use crate::Foundation;

impl Foundation {
    pub fn register_capability(
        &mut self,
        registration: CapabilityRegistration,
    ) -> Result<Capability, FoundationError> {
        self.register_capability_with_cost_profile(
            registration,
            CapabilityCostProfile::foundation_local_default(),
        )
    }

    pub fn register_capability_with_cost_profile(
        &mut self,
        registration: CapabilityRegistration,
        cost_profile: CapabilityCostProfile,
    ) -> Result<Capability, FoundationError> {
        let mcp_contract = CapabilityMcpContract::default_for(
            &registration.capability_id,
            &registration.namespace,
        )
        .map_err(map_capability_error)?;
        self.register_capability_with_cost_profile_and_mcp_contract(
            registration,
            cost_profile,
            mcp_contract,
        )
    }

    pub fn register_capability_with_mcp_contract(
        &mut self,
        registration: CapabilityRegistration,
        mcp_contract: CapabilityMcpContract,
    ) -> Result<Capability, FoundationError> {
        self.register_capability_with_cost_profile_and_mcp_contract(
            registration,
            CapabilityCostProfile::foundation_local_default(),
            mcp_contract,
        )
    }

    pub fn register_capability_with_cost_profile_and_mcp_contract(
        &mut self,
        registration: CapabilityRegistration,
        cost_profile: CapabilityCostProfile,
        mcp_contract: CapabilityMcpContract,
    ) -> Result<Capability, FoundationError> {
        self.eval_gate
            .assert_publish_ready(&registration.capability_id)
            .map_err(map_eval_error)?;
        let mut capability = Capability::new_with_action_and_cost_profile(
            registration.capability_id,
            registration.namespace,
            registration.action,
            registration.required_tier,
            registration.touched_privacy_data_classes,
            registration.evidence_topic,
            cost_profile,
        )
        .map_err(map_capability_error)?;
        capability.mcp_contract = mcp_contract;
        self.capabilities
            .publish(capability.clone())
            .map_err(map_capability_error)?;
        self.audit_chain.append_classifications(
            "ten_system",
            "foundry.capability.publish",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(capability)
    }

    pub fn register_capability_eval_set(
        &mut self,
        eval_set: EvalSetInput,
    ) -> Result<(), FoundationError> {
        let capability_id = eval_set.capability_id.clone();
        self.eval_gate
            .register_eval_set(eval_set)
            .map_err(map_eval_error)?;
        self.audit_chain.append_classifications(
            "ten_system",
            "foundry.eval-set.register",
            Plane::Control,
            Purpose::CoreService,
            audit_classifications(),
            "ALLOW",
        )?;
        self.audit_chain.append_classifications(
            "ten_system",
            format!("foundry.eval-set.ready:{capability_id}"),
            Plane::Control,
            Purpose::CoreService,
            audit_classifications(),
            "ALLOW",
        )?;
        Ok(())
    }

    pub fn record_capability_eval_run(
        &mut self,
        eval_run: EvalRunInput,
    ) -> Result<(), FoundationError> {
        let capability_id = eval_run.capability_id.clone();
        self.eval_gate
            .record_run(eval_run)
            .map_err(map_eval_error)?;
        self.audit_chain.append_classifications(
            "ten_system",
            format!("foundry.eval-run.pass:{capability_id}"),
            Plane::Analytics,
            Purpose::Analytics,
            behavioral_audit_classifications(),
            "ALLOW",
        )?;
        Ok(())
    }

    pub fn grant_capability_to_tenant(
        &mut self,
        grant: TenantCapabilityGrant,
    ) -> Result<(), FoundationError> {
        self.require_tenant(&grant.tenant_id)?;
        self.capabilities
            .grant_to_tenant(
                grant.tenant_id.clone(),
                grant.capability_id.clone(),
                grant.mcp_visible,
            )
            .map_err(map_capability_error)?;
        self.audit_chain.append_classifications(
            grant.tenant_id,
            "foundry.capability.license",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(())
    }

    pub fn discover_tenant_capabilities(
        &self,
        tenant_id: &str,
    ) -> Result<Vec<Capability>, FoundationError> {
        let policy = self
            .tenant_policies
            .get(tenant_id)
            .ok_or(FoundationError::TenantNotFound)?;
        self.capabilities
            .discover_for_tenant(tenant_id, policy.autonomy_ceiling)
            .map_err(map_capability_error)
    }
}
