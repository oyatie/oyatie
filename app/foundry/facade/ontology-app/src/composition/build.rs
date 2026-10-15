use super::{AppState, BootError, TenantState, stores::NativeStores};
use crate::{Config, authz::PolicyEnforcementPoint, seed::registry_for};
use foundry_caller_draft::RosterVerifier;
use foundry_ontology_query_usecase::OntologyQueryExecutionUsecase;
use foundry_projection_sqlite_draft::SqliteProjectionStore;
use foundry_records_draft::RecordsLog;
use foundry_records_sqlite_draft::SqliteRecordsLog;
use foundry_spine::{catch_up, fold_from_scratch};
use std::collections::BTreeMap;
use std::path::Path;
use tokio::sync::Mutex;

pub fn compose(config: &Config) -> Result<AppState, BootError> {
    compose_with_models(config, None)
}

pub fn compose_with_models(
    config: &Config,
    models: Option<&crate::model_runtime::ModelRuntimeConfig>,
) -> Result<AppState, BootError> {
    if let Some(models) = models {
        models
            .validate(config)
            .map_err(|error| BootError::ModelConfigurationRefused { detail: error.0 })?;
    }
    if config.tenants.is_empty() {
        return Err(BootError::NoTenantsConfigured);
    }
    if paths_alias(&config.action_log, &config.denial_log) {
        return Err(BootError::LogPathsAliased);
    }
    if paths_alias(&config.projection_store, &config.action_log)
        || paths_alias(&config.projection_store, &config.denial_log)
    {
        return Err(BootError::StorePathAliased);
    }
    let stores = NativeStores::acquire(config, models)?;
    let action_log =
        stores
            .records(&config.action_log)
            .map_err(|error| BootError::ActionLogUnopenable {
                detail: format!("{error:?}"),
            })?;
    stores
        .records(&config.denial_log)
        .map(drop)
        .map_err(|error| BootError::DenialLogUnopenable {
            detail: format!("{error:?}"),
        })?;

    let pep = PolicyEnforcementPoint::load(POLICY_VERSION).map_err(|error| {
        BootError::PolicyRejected {
            detail: error.to_string(),
        }
    })?;

    let mut tenants = BTreeMap::new();
    for tenant_id in &config.tenants {
        let registry = registry_for(tenant_id).map_err(|error| BootError::SeedRefused {
            tenant_id: tenant_id.clone(),
            detail: error.to_string(),
        })?;
        let baseline = registry.clone();
        let model_config = models.and_then(|models| {
            models
                .stores
                .iter()
                .find(|model| model.binding.tenant_id == *tenant_id)
        });
        let guard = model_config
            .map(|model| stores.guard(&model.path))
            .transpose()?
            .flatten();
        let (model_store, registry, generation) =
            match crate::model_runtime::open(models, tenant_id, &registry, guard)? {
                Some((model, registry, generation)) => (Some(model), registry, Some(generation)),
                None => (None, registry, None),
            };
        let entries = action_log
            .replay(tenant_id, 1)
            .map_err(|error| BootError::ReplayFailed {
                tenant_id: tenant_id.clone(),
                detail: format!("{error:?}"),
            })?;
        let mut store = stores
            .projection(&config.projection_store)
            .map_err(|error| BootError::ProjectionStoreUnopenable {
                detail: format!("{error:?}"),
            })?;
        let projection = if model_store.is_some() {
            let log_head = action_log
                .head(tenant_id)
                .map_err(|error| BootError::ReplayFailed {
                    tenant_id: tenant_id.clone(),
                    detail: format!("{error:?}"),
                })?;
            crate::model_runtime::projection(tenant_id, &registry, &mut store, &entries, log_head)?
        } else {
            catch_up(tenant_id, &registry, &mut store, &entries).map_err(|error| {
                BootError::CatchUpRefused {
                    tenant_id: tenant_id.clone(),
                    detail: format!("{error:?}"),
                }
            })?;
            fold_from_scratch(tenant_id, &registry, entries.iter())
        };
        drop(entries);
        let action_log =
            stores
                .records(&config.action_log)
                .map_err(|error| BootError::ActionLogUnopenable {
                    detail: format!("{error:?}"),
                })?;
        let denial_log =
            stores
                .records(&config.denial_log)
                .map_err(|error| BootError::DenialLogUnopenable {
                    detail: format!("{error:?}"),
                })?;
        let mut tenant = TenantState {
            projection,
            queries: OntologyQueryExecutionUsecase::default(),
            query_decisions: BTreeMap::new(),
            human_query_intents: BTreeMap::new(),
            action_log: Box::new(action_log),
            denial_log: Box::new(denial_log),
            projection_store: Box::new(store),
            model_store,
            model_installation: crate::model_runtime::RuntimeInstallation::Unobserved,
            publication_unknown: None,
        };
        if let (Some(model), Some(generation)) = (model_config, generation) {
            tenant.model_installation = crate::model_runtime::RuntimeInstallation::verified(
                &tenant,
                baseline,
                generation,
                stores.model_guards(config, &model.path)?,
            )
            .map_err(|detail| BootError::ModelRestoreRefused {
                tenant_id: tenant_id.clone(),
                detail: detail.into(),
            })?;
        }
        tenants.insert(tenant_id.clone(), Mutex::new(tenant));
    }
    Ok(AppState {
        tenants,
        pep,
        metrics: crate::metrics::Metrics::default(),
        verifier: Box::new(RosterVerifier::new(config.operators.clone())),
    })
}

const POLICY_VERSION: &str = "psv-000001";

/// Two configured paths name one store. Canonicalization is best-effort —
/// the paths need not exist yet — so a literal match is the fallback.
fn paths_alias(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}
