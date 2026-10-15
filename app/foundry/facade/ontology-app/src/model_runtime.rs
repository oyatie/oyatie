use crate::{Config, model_control::ModelMutationOrigin};
use foundry_model_api::ModelBinding;
use serde::Deserialize;
use std::path::PathBuf;

mod installed;
mod paths;
pub(crate) use installed::RuntimeInstallation;
mod lifecycle;
mod publication;
mod restore;
pub(crate) use lifecycle::lifecycle;
pub(crate) use publication::{StagedPublication, StagedRecovery};

pub use restore::initialize_model;
pub(crate) use restore::{open, projection};

pub const MODELS_ENV: &str = "OYATIE_FOUNDRY_ONTOLOGY_MODELS";
pub const ORIGIN_ENV: &str = "OYATIE_FOUNDRY_ONTOLOGY_MODEL_ORIGIN";
pub const LOOPBACK_ENV: &str = "OYATIE_FOUNDRY_ONTOLOGY_MODEL_ALLOW_LOOPBACK_HTTP";

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelStoreBinding {
    pub path: PathBuf,         // data_class: INTERNAL_ONLY
    pub binding: ModelBinding, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug)]
pub struct ModelRuntimeConfig {
    pub stores: Vec<ModelStoreBinding>, // data_class: INTERNAL_ONLY
    pub origin: ModelMutationOrigin,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelRuntimeConfigError(pub &'static str);

impl std::fmt::Display for ModelRuntimeConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for ModelRuntimeConfigError {}

impl ModelRuntimeConfig {
    pub fn from_env() -> Result<Option<Self>, ModelRuntimeConfigError> {
        let read = |name| match std::env::var(name) {
            Ok(value) => Ok(Some(value)),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(_) => Err(ModelRuntimeConfigError("model configuration is not UTF-8")),
        };
        Self::from_values(
            read(MODELS_ENV)?.as_deref(),
            read(ORIGIN_ENV)?.as_deref(),
            read(LOOPBACK_ENV)?.as_deref(),
        )
    }

    pub fn from_values(
        models: Option<&str>,
        origin: Option<&str>,
        loopback: Option<&str>,
    ) -> Result<Option<Self>, ModelRuntimeConfigError> {
        let local = match loopback {
            None | Some("false") => false,
            Some("true") => true,
            _ => {
                return Err(ModelRuntimeConfigError(
                    "model loopback mode must be true or false",
                ));
            }
        };
        if models.is_none() && origin.is_none() && !local {
            return Ok(None);
        }
        let models = models.ok_or(ModelRuntimeConfigError("model store bindings are required"))?;
        let origin = origin.ok_or(ModelRuntimeConfigError("model mutation origin is required"))?;
        let stores: Vec<ModelStoreBinding> = serde_json::from_str(models).map_err(|_| {
            ModelRuntimeConfigError("model store bindings must be a closed JSON array")
        })?;
        if stores.is_empty() {
            return Err(ModelRuntimeConfigError(
                "model store bindings must not be empty",
            ));
        }
        let origin = if local {
            ModelMutationOrigin::loopback_http(origin)
        } else {
            ModelMutationOrigin::https(origin)
        }
        .map_err(|_| ModelRuntimeConfigError("model mutation origin must be exact and trusted"))?;
        Ok(Some(Self { stores, origin }))
    }

    pub fn validate(&self, config: &Config) -> Result<(), ModelRuntimeConfigError> {
        if self.stores.is_empty() {
            return Err(ModelRuntimeConfigError(
                "model store bindings must not be empty",
            ));
        }
        let mut tenants = std::collections::BTreeSet::new();
        if config
            .tenants
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != config.tenants.len()
        {
            return Err(ModelRuntimeConfigError(
                "the served tenant roster is ambiguous",
            ));
        }
        for (index, model) in self.stores.iter().enumerate() {
            model
                .binding
                .validate()
                .map_err(|_| ModelRuntimeConfigError("invalid model binding"))?;
            if !config.tenants.contains(&model.binding.tenant_id) {
                return Err(ModelRuntimeConfigError(
                    "model tenant is not in the served roster",
                ));
            }
            if !tenants.insert(&model.binding.tenant_id) {
                return Err(ModelRuntimeConfigError("duplicate model tenant binding"));
            }
            if model.path.as_os_str().is_empty() {
                return Err(ModelRuntimeConfigError("model store path is required"));
            }
            let other_paths = [
                &config.action_log,
                &config.denial_log,
                &config.projection_store,
            ];
            if other_paths
                .into_iter()
                .any(|path| paths::alias(&model.path, path))
                || self.stores[..index]
                    .iter()
                    .any(|other| paths::alias(&model.path, &other.path))
            {
                return Err(ModelRuntimeConfigError(
                    "model storage must be distinct from all other stores",
                ));
            }
        }
        Ok(())
    }
}
