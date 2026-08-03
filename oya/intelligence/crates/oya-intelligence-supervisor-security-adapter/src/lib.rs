//! Foundry supervisor security adapter — Cedar autonomy ceiling + secret-store
//! resolution port integration. Production OpenBao backend is deferred per
//! ADR-0043 + M02-P06; callers inject the concrete `SecretStorePort`.

use oya_intelligence_account_domain::{SecretMaterial, SecretReference, SecretStorePort};
use oya_intelligence_autonomy_ceiling_domain::{CeilingPolicy, TenantId};
use intelligence_autonomy_ceiling_kernel::{AutonomyTier as CeilingTier, check_tier};
use intelligence_capability_registry_kernel::AutonomyTier as CapabilityTier;
use intelligence_supervisor_kernel::{
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
        let cap_id = intelligence_capability_registry_kernel::CapabilityId::new(
            "foundry.supervisor.spawn",
        );
        let cap = intelligence_capability_registry_kernel::Capability::new(
            cap_id,
            "spawn",
            match tier {
                AutonomyTier::T1Read => {
                    intelligence_capability_registry_kernel::AutonomyTier::T1Read
                }
                AutonomyTier::T2Suggest => {
                    intelligence_capability_registry_kernel::AutonomyTier::T2Suggest
                }
                AutonomyTier::T3PropAct => {
                    intelligence_capability_registry_kernel::AutonomyTier::T3PropAct
                }
                AutonomyTier::T4Actuate => {
                    intelligence_capability_registry_kernel::AutonomyTier::T4Actuate
                }
            },
            true,
        );

        match check_tier(
            bridge_capability_tier(cap.autonomy_tier),
            self.policy.ceiling_for(&tenant_id),
        ) {
            intelligence_autonomy_ceiling_kernel::CeilingVerdict::Allow => Ok(()),
            intelligence_autonomy_ceiling_kernel::CeilingVerdict::Block {
                capability_tier,
                ceiling,
            } => Err(SupervisorError::Quarantined(format!(
                "autonomy ceiling block: capability tier {} exceeds ceiling {}",
                capability_tier, ceiling
            ))),
        }
    }
}

fn bridge_capability_tier(tier: CapabilityTier) -> CeilingTier {
    match tier {
        CapabilityTier::T1Read => CeilingTier::T1Read,
        CapabilityTier::T2Suggest => CeilingTier::T2Suggest,
        CapabilityTier::T3PropAct => CeilingTier::T3PropAct,
        CapabilityTier::T4Actuate => CeilingTier::T4Actuate,
    }
}

pub struct SecretStoreResolver<S> {
    inner: S,
}

impl<S> SecretStoreResolver<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S: SecretStorePort> SecretStoreResolver<S> {
    pub fn resolve(&self, sref: &SecretReference) -> Result<SecretMaterial, SupervisorError> {
        self.inner
            .get(sref)
            .map_err(|e| SupervisorError::DriverError(format!("secret resolution failed: {:?}", e)))
    }
}
