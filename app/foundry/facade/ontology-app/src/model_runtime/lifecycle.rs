use crate::composition::TenantState;
use foundry_model_api::{ModelCaller, ModelError, control::*};

pub(crate) fn observed(tenant: &TenantState, desired: u64) -> Result<ModelLifecycle, ModelError> {
    let desired_generation = ModelNumber::new(desired)?;
    let observed_generation = tenant
        .observed_model_generation()
        .map(ModelNumber::new)
        .transpose()?;
    let phase = if tenant.publication_unknown.is_some() {
        ModelLifecyclePhase::CommitUnknown
    } else {
        match observed_generation {
            None => ModelLifecyclePhase::Unobserved,
            Some(value) if value == desired_generation => ModelLifecyclePhase::Current,
            Some(_) => ModelLifecyclePhase::Pending,
        }
    };
    let ready = phase == ModelLifecyclePhase::Current
        && tenant
            .sync_status(&tenant.projection.tenant_id)
            .is_ok_and(|status| {
                status.head == status.applied_ordinal
                    && status.head == tenant.projection.applied_ordinal
            });
    Ok(ModelLifecycle {
        desired_generation,
        observed_generation,
        phase,
        ready,
        scope: ModelObservationScope::RespondingInstance,
    })
}

pub(crate) fn lifecycle(
    tenant: &TenantState,
    caller: &ModelCaller,
) -> Result<ModelLifecycle, ModelError> {
    let model = tenant
        .model_store
        .as_ref()
        .ok_or(ModelError::Uninitialized)?;
    model.binding.check_caller(caller)?;
    let head = model.store.head(caller)?;
    head.document.check_bounds()?;
    observed(tenant, head.generation)
}

pub(super) fn current(generation: u64) -> Result<ModelLifecycle, ModelError> {
    let number = ModelNumber::new(generation)?;
    Ok(ModelLifecycle {
        desired_generation: number,
        observed_generation: Some(number),
        phase: ModelLifecyclePhase::Current,
        ready: true,
        scope: ModelObservationScope::RespondingInstance,
    })
}
