use super::faults::{Fault, FaultModel, WriteAttempts};
use crate::{
    AppState, Config, compose_with_models,
    model_control::ModelMutationOrigin,
    model_runtime::{ModelRuntimeConfig, ModelStoreBinding},
};
use foundry_model_api::{control::*, *};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};
pub(super) const TENANT: &str = "ten_publication";
static NEXT: AtomicU64 = AtomicU64::new(0);
pub(super) struct Files {
    pub directory: PathBuf,
    pub config: Config,
    pub models: ModelRuntimeConfig,
    pub writes: Arc<Mutex<WriteAttempts>>,
    cleanup: bool,
}
impl Files {
    pub fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "foundry-publication-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir(&directory).unwrap();
        let mut files = Self::existing(directory);
        files.cleanup = true;
        crate::model_runtime::initialize_model(&files.config, &files.models, TENANT).unwrap();
        files
    }
    pub fn existing(directory: PathBuf) -> Self {
        let config = Config {
            listen_addr: "127.0.0.1:0".into(),
            action_log: directory.join("actions.sqlite"),
            denial_log: directory.join("denials.sqlite"),
            projection_store: directory.join("projection.sqlite"),
            tenants: vec![TENANT.into()],
            operators: vec![crate::OperatorCredential {
                token: "publication-operator".into(),
                tenant_id: TENANT.into(),
                principal_id: "prn_operator".into(),
                roles: vec!["foundry-operator".into()],
            }],
        };
        let models = ModelRuntimeConfig {
            stores: vec![ModelStoreBinding {
                path: directory.join("model.sqlite"),
                binding: ModelBinding {
                    tenant_id: TENANT.into(),
                    store_id: "publication-model".into(),
                    baseline_id: "trusted-record".into(),
                },
            }],
            origin: ModelMutationOrigin::https("https://shell.example").unwrap(),
        };
        Self {
            directory,
            config,
            models,
            writes: Arc::new(Mutex::new(WriteAttempts::default())),
            cleanup: false,
        }
    }
    pub fn compose(&self, fault: Fault) -> (AppState, Arc<Mutex<Fault>>) {
        let mut app = compose_with_models(&self.config, Some(&self.models)).unwrap();
        let model = app
            .tenants
            .get_mut(TENANT)
            .unwrap()
            .get_mut()
            .model_store
            .as_mut()
            .unwrap();
        let baseline =
            ModelBaseline::capture(&crate::seed::registry_for(TENANT).unwrap(), TENANT).unwrap();
        let placeholder =
            Box::new(ModelLedger::initialize(model.binding.clone(), baseline).unwrap());
        let inner = std::mem::replace(&mut model.store, placeholder);
        let fault = Arc::new(Mutex::new(fault));
        model.store = Box::new(FaultModel {
            inner,
            fault: fault.clone(),
            attempted: AtomicBool::new(false),
            writes: self.writes.clone(),
        });
        (app, fault)
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        if self.cleanup {
            let _ = std::fs::remove_dir_all(&self.directory);
        }
    }
}
pub(super) fn caller() -> ModelCaller {
    ModelCaller {
        tenant_id: TENANT.into(),
        owner: "usr_designer".into(),
    }
}
pub(super) fn draft(tenant: &mut crate::composition::TenantState) -> ModelDraft {
    let store = &mut tenant.model_store.as_mut().unwrap().store;
    let head = store.head(&caller()).unwrap();
    let mut document = head.document;
    document.entities[0].revision += 1;
    document.entities[0].display_name = "A published record".into();
    store
        .create_draft(&caller(), head.generation, document)
        .unwrap()
}
pub(super) fn input(draft: &ModelDraft) -> PrepareModelDraft {
    PrepareModelDraft {
        expected_version: ModelNumber::new(draft.version).unwrap(),
        expected_base: ModelNumber::new(draft.base_generation).unwrap(),
    }
}
