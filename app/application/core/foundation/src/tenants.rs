//! Tenant onboarding and cell binding.

use crate::*;

use crate::Foundation;

impl Foundation {
    pub fn with_invocation_trace_observer(
        mut self,
        observer: impl CapabilityInvocationTraceObserver + 'static,
    ) -> Self {
        self.observability = FoundationObservability::new(observer);
        self
    }

    pub fn register_autonomy_break_glass(
        &mut self,
        input: AutonomyBreakGlassInput,
    ) -> Result<AutonomyBreakGlass, FoundationError> {
        self.require_tenant(&input.tenant_id)?;
        if self.capabilities.get(&input.capability_id).is_none() {
            return Err(FoundationError::CapabilityNotFound);
        }
        let record = input.build().map_err(map_bypass_error)?;
        let mut candidate = self.foundation_bypass_ledger.clone();
        candidate
            .insert_record(BypassLedgerRecord::from(record.clone()))
            .map_err(map_bypass_error)?;
        candidate
            .validate_windows(record.created_at_epoch_days.value)
            .map_err(map_bypass_error)?;
        self.foundation_bypass_ledger = candidate;
        self.audit_chain.append_classifications(
            record.tenant_id.value.clone(),
            "foundry.autonomy.break_glass.approve",
            Plane::Control,
            Purpose::CoreService,
            internal_audit_classifications(),
            "ALLOW",
        )?;
        Ok(record)
    }

    pub fn foundation_bypass_ledger(&self) -> &BypassLedger {
        &self.foundation_bypass_ledger
    }

    pub fn onboard_tenant(
        &mut self,
        registration: TenantRegistration,
    ) -> Result<Tenant, FoundationError> {
        if self.tenants.contains_key(&registration.tenant_id) {
            return Err(FoundationError::TenantAlreadyExists);
        }
        let residency_class = parse_residency_class_label(&registration.residency_class)
            .ok_or(FoundationError::InvalidInput)?;
        let tenant = Tenant::new(
            registration.tenant_id.clone(),
            registration.legal_name,
            registration.home_region,
            residency_class,
            registration.regulatory_packs,
        )
        .map_err(map_tenant_error)?;
        self.tenant_policies.insert(
            tenant.id.clone(),
            TenantPolicy::new(tenant.id.clone(), registration.autonomy_ceiling),
        );
        self.tenants.insert(tenant.id.clone(), tenant.clone());
        self.audit_chain.append_classifications(
            tenant.id.clone(),
            "tenant.create",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )?;
        Ok(tenant)
    }

    pub fn bind_cell(
        &mut self,
        tenant_id: &str,
        az: impl Into<String>,
        cell_id: impl Into<String>,
    ) -> Result<CellBinding, FoundationError> {
        let tenant = self.require_tenant(tenant_id)?.clone();
        let region = tenant.home_region.value.clone();
        let region_ref = RegionRef::new(RegionRefCreate {
            region_id: region.clone(),
            jurisdiction: infer_region_jurisdiction_label(&region),
            cell_group_ref: format!("cells/{region}"),
        })
        .map_err(|_| FoundationError::InvalidInput)?;
        let cell_id = cell_id.into();
        let binding_input = CellBindingCreate {
            tenant_id: tenant_id.to_string(),
            region: region_ref,
            residency_class: tenant.residency_class.value,
            az: az.into(),
            hsm_partition_ref: format!("hsm/{region}/{cell_id}"),
            cell_id,
            tier: CellTier::Pooled,
        };
        match self.cells.bind(binding_input) {
            Ok(binding) => {
                self.audit_chain.append_classifications(
                    tenant_id,
                    "cloud.cell.bind",
                    Plane::Control,
                    Purpose::CoreService,
                    vec![DataClass::InternalOnly],
                    "ALLOW",
                )?;
                Ok(binding)
            }
            Err(CellError::AlreadyBound) => {
                self.audit_chain.append_classifications(
                    tenant_id,
                    "cloud.cell.bind",
                    Plane::Control,
                    Purpose::CoreService,
                    vec![DataClass::InternalOnly],
                    "DENY",
                )?;
                Err(FoundationError::CellBindingImmutable)
            }
            Err(
                CellError::InvalidTenantId
                | CellError::EmptyAz
                | CellError::EmptyCell
                | CellError::EmptyHsmPartition
                | CellError::AzRegionMismatch
                | CellError::ResidencyRegionMismatch,
            ) => Err(FoundationError::InvalidInput),
        }
    }
}
