use crate::composition::TenantState;
use data_ontology_kernel::OntologyEngine;
use foundry_model_api::ModelBinding;
use foundry_store_local_draft::LocalStoreGuard;

pub(crate) enum RuntimeInstallation {
    Unobserved,
    Invalidated,
    Installed(Box<FencedRuntime>),
}

#[derive(Clone)]
pub(crate) struct FencedRuntime {
    pub(super) binding: ModelBinding,
    pub(super) baseline: OntologyEngine,
    pub(super) registry: OntologyEngine,
    pub(super) generation: u64,
    guards: [LocalStoreGuard; 4],
}

impl RuntimeInstallation {
    pub(crate) fn verified(
        tenant: &TenantState,
        baseline: OntologyEngine,
        generation: u64,
        guards: [LocalStoreGuard; 4],
    ) -> Result<Self, &'static str> {
        let binding = tenant
            .model_store
            .as_ref()
            .ok_or("established model is absent")?
            .binding
            .clone();
        if generation == 0 || !guards.iter().all(|guard| guard.access().exclusive()) {
            return Err("exclusive installed ownership is absent");
        }
        let proof = FencedRuntime {
            binding,
            baseline,
            registry: tenant.projection.registry_input.clone(),
            generation,
            guards,
        };
        if !proof.matches(tenant) {
            return Err("installed stores do not share the admitted ownership");
        }
        Ok(Self::Installed(Box::new(proof)))
    }
    pub(crate) fn invalidate(&mut self) {
        if !matches!(self, Self::Unobserved) {
            *self = Self::Invalidated;
        }
    }
    pub(crate) fn generation(&self, tenant: &TenantState) -> Option<u64> {
        match self {
            Self::Installed(proof) if proof.matches(tenant) => Some(proof.generation),
            _ => None,
        }
    }
    pub(crate) fn requires_observation(&self) -> bool {
        !matches!(self, Self::Unobserved)
    }
}

impl FencedRuntime {
    pub(super) fn replacement(
        &self,
        registry: OntologyEngine,
        generation: u64,
    ) -> RuntimeInstallation {
        RuntimeInstallation::Installed(Box::new(Self {
            registry,
            generation,
            ..self.clone()
        }))
    }
    pub(super) fn matches(&self, tenant: &TenantState) -> bool {
        let Some(model) = tenant.model_store.as_ref() else {
            return false;
        };
        if model.binding != self.binding
            || tenant.projection.tenant_id != self.binding.tenant_id
            || tenant.projection.registry_input != self.registry
        {
            return false;
        }
        let actual = [
            tenant.action_log.store_access(),
            tenant.denial_log.store_access(),
            tenant.projection_store.store_access(),
            model.store.store_access(),
        ];
        self.guards.iter().zip(actual).all(|(guard, access)| {
            guard.validate(guard.path()).is_ok()
                && access.is_some_and(|access| guard.access().same_owner(access))
        })
    }
}
