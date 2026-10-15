use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    ActionInvocationRequest, ActionTypeDefinition, ActionTypeId, AutonomyTier,
    EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, PropertyTier,
};
use foundry_edits::{EditSet, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue};
use foundry_ontology_app::{AppState, Caller, Surface};
use foundry_projection_draft::{
    AppliedEntry, ApplyReceipt, KeyDesignations, Page, PageRequest, ProjectedLink, ProjectedObject,
    ProjectionStore, ProjectionStoreError, PropertyPredicate,
};
use foundry_spine::{ActionSubmission, ApplyOutcome, submit_through};

use crate::facade_support::TENANT;

pub fn declare(state: &mut AppState, kind: &str, action: &str, property: &str) {
    let tenant = state.tenants.get_mut(TENANT).unwrap().get_mut();
    let projection = tenant.projection_for_injection();
    for registry in [&mut projection.engine, &mut projection.registry_input] {
        let entity_type = EntityTypeId::new(kind).unwrap();
        registry
            .register_entity_type(
                EntityTypeDefinition::new(
                    TENANT,
                    entity_type.clone(),
                    "Other",
                    vec![
                        EntityTypePropertyDefinition::new(
                            property,
                            PropertyTier::Scalar,
                            PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
                            true,
                        )
                        .unwrap(),
                    ],
                    1,
                )
                .unwrap(),
            )
            .unwrap();
        registry
            .register_action_type(
                ActionTypeDefinition::new(
                    TENANT,
                    ActionTypeId::new(action).unwrap(),
                    entity_type,
                    "ops-console",
                    AutonomyTier::T1Assist,
                    "other.created",
                )
                .unwrap(),
            )
            .unwrap();
    }
}

pub async fn send(
    state: &AppState,
    kind: &str,
    action: &str,
    property: &str,
    value: &str,
    key: &str,
) {
    let caller = Caller {
        tenant_id: TENANT.into(),
        principal_id: "prn_alice".into(),
        roles: vec!["foundry-operator".into()],
    };
    let decision = state
        .pep
        .decide(&caller, Surface::Invoke, "ent_alpha", TENANT)
        .unwrap();
    let submission = ActionSubmission {
        request: ActionInvocationRequest {
            tenant_id: TENANT.into(),
            principal_id: "prn_alice".into(),
            action_id: ActionTypeId::new(action).unwrap(),
            entity_id: "ent_alpha".into(),
            idempotency_key: key.into(),
            requested_at_epoch_seconds: 1_700_000_000,
        },
        decision,
        parameters: vec![],
        edits: EditSet::new(vec![
            OntologyEdit::create_object(
                kind,
                vec![
                    WireProperty::new(
                        property,
                        WireTier::Scalar,
                        WireDataClass::InternalOnly,
                        WireValue::String(value.into()),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        ])
        .unwrap(),
    };
    let mut tenant = state.tenants[TENANT].lock().await;
    let (log, denials, projection, store) = tenant.injected_write_handles().unwrap();
    let accepted = submit_through(submission, log, denials, projection, store).unwrap();
    assert!(matches!(accepted.outcome, ApplyOutcome::Applied { .. }));
    accepted.mirror.unwrap();
}

pub fn point(kind: &str, object: &str, revision: u32) -> String {
    let mut url = reqwest::Url::parse("https://point.invalid/").unwrap();
    url.query_pairs_mut()
        .append_pair("type", kind)
        .append_pair("revision", &revision.to_string());
    format!(
        "/v1/tenants/ten_acme/objects/{object}?{}",
        url.query().unwrap()
    )
}

pub struct ChangingStore {
    inner: Box<dyn ProjectionStore + Send>,
    first: Option<ProjectedObject>,
    reads: Arc<AtomicUsize>,
    delay: Duration,
}

impl ChangingStore {
    pub fn install(
        state: &mut AppState,
        first: Option<ProjectedObject>,
        delay: Duration,
    ) -> Arc<AtomicUsize> {
        let tenant = state.tenants.get_mut(TENANT).unwrap().get_mut();
        let inner = tenant.replace_projection_store(Box::new(
            foundry_projection_draft::MemoryProjectionStore::default(),
        ));
        let reads = Arc::new(AtomicUsize::new(0));
        tenant.replace_projection_store(Box::new(Self {
            inner,
            first,
            reads: reads.clone(),
            delay,
        }));
        reads
    }
}

impl ProjectionStore for ChangingStore {
    fn apply(
        &mut self,
        entry: AppliedEntry,
        keys: &KeyDesignations,
    ) -> Result<ApplyReceipt, ProjectionStoreError> {
        self.inner.apply(entry, keys)
    }
    fn applied_head(&self, tenant: &str) -> Result<u64, ProjectionStoreError> {
        self.inner.applied_head(tenant)
    }
    fn get(
        &self,
        tenant: &str,
        object: &str,
    ) -> Result<Option<ProjectedObject>, ProjectionStoreError> {
        let read = self.reads.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(self.delay);
        if read == 0 && self.first.is_some() {
            return Ok(self.first.clone());
        }
        self.inner.get(tenant, object)
    }
    fn objects_of_type(
        &self,
        tenant: &str,
        kind: &str,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        self.inner.objects_of_type(tenant, kind, page)
    }
    fn filter(
        &self,
        tenant: &str,
        kind: &str,
        predicate: &PropertyPredicate,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        self.inner.filter(tenant, kind, predicate, page)
    }
    fn links_from(
        &self,
        tenant: &str,
        object: &str,
    ) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        self.inner.links_from(tenant, object)
    }
    fn links_to(
        &self,
        tenant: &str,
        object: &str,
    ) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        self.inner.links_to(tenant, object)
    }
    fn poisoned(&self, tenant: &str) -> Result<Vec<(u64, String)>, ProjectionStoreError> {
        self.inner.poisoned(tenant)
    }
}
