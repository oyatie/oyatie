use crate::{BootError, Config, model_runtime::ModelRuntimeConfig};
use foundry_projection_draft::ProjectionStoreError;
use foundry_projection_sqlite_draft::SqliteProjectionStore;
use foundry_records_draft::RecordsLogError;
use foundry_records_sqlite_draft::SqliteRecordsLog;
use foundry_store_local_draft::{LocalStoreGuard, LocalStoreSet, StoreOpen};
use std::path::Path;

pub(super) struct NativeStores {
    ownership: Option<LocalStoreSet>,
}
impl NativeStores {
    pub(super) fn acquire(
        config: &Config,
        models: Option<&ModelRuntimeConfig>,
    ) -> Result<Self, BootError> {
        let Some(models) = models else {
            return Ok(Self { ownership: None });
        };
        for model in &models.stores {
            if !model.path.is_file() {
                return Err(BootError::ModelStoreRefused {
                    tenant_id: model.binding.tenant_id.clone(),
                    detail: "established model file is unavailable".into(),
                });
            }
        }
        let mut paths = vec![
            (config.action_log.clone(), StoreOpen::Create),
            (config.denial_log.clone(), StoreOpen::Create),
            (config.projection_store.clone(), StoreOpen::Create),
        ];
        paths.extend(
            models
                .stores
                .iter()
                .map(|model| (model.path.clone(), StoreOpen::Existing)),
        );
        let ownership = LocalStoreSet::exclusive(&paths).map_err(refused)?;
        Ok(Self {
            ownership: Some(ownership),
        })
    }
    pub(super) fn guard(&self, path: &Path) -> Result<Option<LocalStoreGuard>, BootError> {
        self.ownership
            .as_ref()
            .map(|set| set.guard(path).map_err(refused))
            .transpose()
    }
    pub(super) fn model_guards(
        &self,
        config: &Config,
        model_path: &Path,
    ) -> Result<[LocalStoreGuard; 4], BootError> {
        let guard = |path| {
            self.guard(path)?
                .ok_or_else(|| refused("exclusive model ownership is absent"))
        };
        Ok([
            guard(&config.action_log)?,
            guard(&config.denial_log)?,
            guard(&config.projection_store)?,
            guard(model_path)?,
        ])
    }
    pub(super) fn records(&self, path: &Path) -> Result<SqliteRecordsLog, RecordsLogError> {
        match &self.ownership {
            Some(set) => SqliteRecordsLog::open_with_access(
                path,
                set.guard(path).map_err(|error| RecordsLogError::Storage {
                    detail: error.to_string(),
                })?,
            ),
            None => SqliteRecordsLog::open(path),
        }
    }
    pub(super) fn projection(
        &self,
        path: &Path,
    ) -> Result<SqliteProjectionStore, ProjectionStoreError> {
        match &self.ownership {
            Some(set) => SqliteProjectionStore::open_with_access(
                path,
                set.guard(path)
                    .map_err(|error| ProjectionStoreError::Storage {
                        detail: error.to_string(),
                    })?,
            ),
            None => SqliteProjectionStore::open(path),
        }
    }
}
fn refused(error: impl std::fmt::Display) -> BootError {
    BootError::StoreOwnershipRefused {
        detail: error.to_string(),
    }
}
