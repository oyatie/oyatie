//! Foundry supervisor security adapter — Cedar + OpenBao integration.

use oya_foundry_account_adapter_openbao::OpenBaoAdapter;
use oya_foundry_account_domain::{SecretMaterial, SecretReference, SecretStorePort};
use oya_foundry_autonomy_ceiling_app as ceiling_app;
use oya_foundry_autonomy_ceiling_domain::{CeilingPolicy, TenantId};
use oya_foundry_supervisor_kernel::{
    AccountId, AutonomyCeilingPort, AutonomyTier, SupervisorError,
};

pub struct CedarAutonomyCeilingAdapter {
    policy: CeilingPolicy,
}

impl CedarAutonomyCeilingAdapter {
    pub fn new(policy: CeilingPolicy) -> Self {
        Self { policy }
    }
}

impl AutonomyCeilingPort for CedarAutonomyCeilingAdapter {
    fn enforce(&self, account_id: &AccountId, tier: AutonomyTier) -> Result<(), SupervisorError> {
        // Bridges AccountId to TenantId (Simplified for Wave 4: assumes 1:1 or extract)
        let tenant_id = TenantId::new(&account_id.0);

        // Construct a dummy capability for the check
        let cap_id =
            oya_foundry_capability_registry_kernel::CapabilityId::new("foundry.supervisor.spawn");
        let cap = oya_foundry_capability_registry_kernel::Capability::new(
            cap_id,
            "spawn",
            match tier {
                AutonomyTier::T1Read => {
                    oya_foundry_capability_registry_kernel::AutonomyTier::T1Read
                }
                AutonomyTier::T2Suggest => {
                    oya_foundry_capability_registry_kernel::AutonomyTier::T2Suggest
                }
                AutonomyTier::T3PropAct => {
                    oya_foundry_capability_registry_kernel::AutonomyTier::T3PropAct
                }
                AutonomyTier::T4Actuate => {
                    oya_foundry_capability_registry_kernel::AutonomyTier::T4Actuate
                }
            },
            true,
        );

        match ceiling_app::enforce_for_tenant(&cap, &tenant_id, &self.policy) {
            oya_foundry_autonomy_ceiling_kernel::CeilingVerdict::Allow => Ok(()),
            oya_foundry_autonomy_ceiling_kernel::CeilingVerdict::Block {
                capability_tier,
                ceiling,
            } => Err(SupervisorError::Quarantined(format!(
                "autonomy ceiling block: capability tier {} exceeds ceiling {}",
                capability_tier, ceiling
            ))),
        }
    }
}

pub struct OpenBaoSecretResolver {
    inner: OpenBaoAdapter,
}

impl OpenBaoSecretResolver {
    pub fn new(inner: OpenBaoAdapter) -> Self {
        Self { inner }
    }

    pub fn resolve(&self, sref: &SecretReference) -> Result<SecretMaterial, SupervisorError> {
        self.inner
            .get(sref)
            .map_err(|e| SupervisorError::DriverError(format!("secret resolution failed: {:?}", e)))
    }
}
