use super::{ModelRuntimeConfig, ModelRuntimeConfigError};
use crate::{BootError, Config, model_control::EstablishedModelStore};
use data_ontology_kernel::OntologyEngine;
use foundry_model_api::{ModelBaseline, ModelCaller, ModelStore};
use foundry_model_sqlite_draft::SqliteModelStore;
use foundry_projection_draft::ProjectionStore;
use foundry_records_draft::SealedEnvelope;
use foundry_spine::{ProjectionState, project_through};

pub(crate) fn configuration(error: ModelRuntimeConfigError) -> BootError {
    BootError::ModelConfigurationRefused { detail: error.0 }
}

fn refused(tenant: &str, error: impl std::fmt::Debug) -> BootError {
    BootError::ModelStoreRefused {
        tenant_id: tenant.into(),
        detail: format!("{error:?}"),
    }
}

/// Explicit offline establishment; no IAM call, listener, log creation or recovery fallback.
pub fn initialize_model(
    config: &Config,
    models: &ModelRuntimeConfig,
    tenant: &str,
) -> Result<(), BootError> {
    models.validate(config).map_err(configuration)?;
    let model = models
        .stores
        .iter()
        .find(|model| model.binding.tenant_id == tenant)
        .ok_or_else(|| {
            configuration(ModelRuntimeConfigError(
                "tenant has no configured model binding",
            ))
        })?;
    let baseline = crate::seed::registry_for(tenant).map_err(|error| refused(tenant, error))?;
    let baseline =
        ModelBaseline::capture(&baseline, tenant).map_err(|error| refused(tenant, error))?;
    SqliteModelStore::initialize(&model.path, model.binding.clone(), baseline)
        .map(drop)
        .map_err(|error| refused(tenant, error))
}

pub(crate) fn open(
    models: Option<&ModelRuntimeConfig>,
    tenant: &str,
    baseline: &OntologyEngine,
    access: Option<foundry_store_local_draft::LocalStoreGuard>,
) -> Result<Option<(EstablishedModelStore, OntologyEngine, u64)>, BootError> {
    let Some(model) =
        models.and_then(|models| models.stores.iter().find(|m| m.binding.tenant_id == tenant))
    else {
        return Ok(None);
    };
    let expected =
        ModelBaseline::capture(baseline, tenant).map_err(|error| refused(tenant, error))?;
    let access = access.ok_or_else(|| refused(tenant, "exclusive model ownership is absent"))?;
    let store = SqliteModelStore::open_existing_with_access(
        &model.path,
        model.binding.clone(),
        expected,
        access,
    )
    .map_err(|error| refused(tenant, error))?;
    let history = store
        .history(&ModelCaller {
            tenant_id: tenant.into(),
            owner: "model-composition".into(),
        })
        .map_err(|error| refused(tenant, error))?;
    let registry = history
        .registry_with_runtime_baseline(baseline)
        .map_err(|error| refused(tenant, error))?;
    Ok(Some((
        EstablishedModelStore {
            binding: model.binding.clone(),
            store: Box::new(store),
        },
        registry,
        history
            .head()
            .map_err(|error| refused(tenant, error))?
            .generation,
    )))
}

pub(crate) fn projection(
    tenant: &str,
    registry: &OntologyEngine,
    store: &mut dyn ProjectionStore,
    log: &[SealedEnvelope],
    log_head: u64,
) -> Result<ProjectionState, BootError> {
    let error = |detail| BootError::ModelRestoreRefused {
        tenant_id: tenant.into(),
        detail,
    };
    let head = store
        .applied_head(tenant)
        .map_err(|e| error(format!("{e:?}")))?;
    if log.iter().enumerate().any(|(index, entry)| {
        entry.envelope.tenant_id != tenant || entry.receipt.ordinal != index as u64 + 1
    }) {
        return Err(error(
            "model restore requires the complete dense tenant log".into(),
        ));
    }
    if log.last().map_or(0, |entry| entry.receipt.ordinal) != log_head {
        return Err(error(
            "model restore log does not reach its durable head".into(),
        ));
    }
    if head > log_head {
        return Err(error(
            "the durable projection is ahead of its action log".into(),
        ));
    }
    let mut projection = ProjectionState::new(tenant, registry);
    // Every existing ordinal is byte-compared before a genuinely missing suffix is mirrored.
    project_through(&mut projection, store, log).map_err(|e| error(format!("{e:?}")))?;
    if store
        .applied_head(tenant)
        .map_err(|e| error(format!("{e:?}")))?
        != projection.applied_ordinal
    {
        return Err(error(
            "model restore did not reach its durable log head".into(),
        ));
    }
    Ok(projection)
}
