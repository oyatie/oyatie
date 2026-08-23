//! `gateway-epic-fhir-connector` — Epic FHIR R4 connector.
//!
//! # Coverage
//!
//! Epic FHIR R4 + USCDI surfaces:
//! * `Patient` — demographics
//! * `Encounter` — visits
//! * `MedicationRequest` — orders
//! * `Observation` — labs/vitals
//!
//! HL7v2 fallback is provided via a sibling `gateway-hl7v2-connector`
//! tracked as a future migration IP.
//!
//! # Auth
//!
//! SMART-on-FHIR (OAuth2 with backend services profile + signed JWT
//! client assertion). SecretReference resolves the JWT signing key.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use shared_connector_kernel::{
    AuthScheme, Connector, ConnectorCapabilities, ConnectorCtx, ConnectorError, Cursor, EntityDoc,
    EntityValue, EventStream, HealthReport, IdempotencyKey, OntologyProjection, Page, PatchOp,
    RateLimitDescriptor, btree_keyset_page, connector_operation_audit_digest,
    entity_doc_payload_digest, is_canonical_sha256_hex, patch_op_payload_digest,
};
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

type Result<T> = std::result::Result<T, ConnectorError>;
type Store = HashMap<String, HashMap<String, BTreeMap<String, EntityDoc>>>;
type IdemMap = HashMap<String, HashMap<String, HashMap<String, (String, String)>>>;
const PROVIDER_ID: &str = "epic-fhir";

pub struct EpicFhirConnector {
    store: Mutex<Store>,
    idem: Mutex<IdemMap>,
    next_id: Mutex<u64>,
}

impl Default for EpicFhirConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl EpicFhirConnector {
    pub fn new() -> Self {
        let s = Self {
            store: Mutex::new(HashMap::new()),
            idem: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        };
        s.seed();
        s
    }
    fn seed(&self) {
        for tenant in ["sandbox", "t-1"] {
            for i in 1..=5 {
                let id = format!("Patient-{i:08}");
                let mut d = EntityDoc::new();
                d.insert("id", EntityValue::Str(id.clone()));
                d.insert("resourceType", EntityValue::Str("Patient".into()));
                d.insert("family", EntityValue::Str("DOE".into()));
                d.insert("given", EntityValue::Str(format!("JOHN{i}")));
                self.put(tenant, "Patient", &id, d);
            }
            for i in 1..=3 {
                let id = format!("Encounter-{i:08}");
                let mut d = EntityDoc::new();
                d.insert("id", EntityValue::Str(id.clone()));
                d.insert("resourceType", EntityValue::Str("Encounter".into()));
                d.insert("subject", EntityValue::Str("Patient-00000001".into()));
                d.insert("status", EntityValue::Str("finished".into()));
                self.put(tenant, "Encounter", &id, d);
            }
            for i in 1..=2 {
                let id = format!("Observation-{i:08}");
                let mut d = EntityDoc::new();
                d.insert("id", EntityValue::Str(id.clone()));
                d.insert("resourceType", EntityValue::Str("Observation".into()));
                d.insert("code", EntityValue::Str("8480-6".into())); // LOINC: BP systolic
                d.insert("valueQuantity", EntityValue::Int(120));
                self.put(tenant, "Observation", &id, d);
            }
        }
    }
    fn put(&self, tenant: &str, kind: &str, id: &str, doc: EntityDoc) {
        self.lock_store()
            .entry(tenant.to_owned())
            .or_default()
            .entry(kind.to_owned())
            .or_default()
            .insert(id.to_owned(), doc);
    }
    fn lock_store(&self) -> std::sync::MutexGuard<'_, Store> {
        match self.store.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        }
    }
    fn lock_idem(&self) -> std::sync::MutexGuard<'_, IdemMap> {
        match self.idem.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        }
    }
    fn next_seq_candidate(&self) -> u64 {
        let g = match self.next_id.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        *g + 7000
    }
    fn commit_seq(&self) {
        let mut g = match self.next_id.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        *g += 1;
    }
    fn check_kind(&self, kind: &str) -> Result<()> {
        match kind {
            "Patient" | "Encounter" | "MedicationRequest" | "Observation" => Ok(()),
            other => Err(ConnectorError::Unsupported(format!(
                "epic-fhir resourceType={other}"
            ))),
        }
    }
    fn seal(&self, ctx: &ConnectorCtx, op: &str, payload_digest: &str) -> Result<()> {
        if !is_canonical_sha256_hex(payload_digest) {
            return Err(ConnectorError::AuditSealFailed(format!(
                "{op} payload digest must be canonical sha256"
            )));
        }
        let receipt = ctx.audit_handle().seal(op, payload_digest)?;
        if receipt.chain_id.is_empty()
            || receipt.kind != op
            || receipt.payload_digest != payload_digest
        {
            return Err(ConnectorError::AuditSealFailed(format!(
                "{op} seal receipt mismatch"
            )));
        }
        Ok(())
    }

    fn operation_digest<I, K, V>(&self, ctx: &ConnectorCtx, op: &str, fields: I) -> String
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        connector_operation_audit_digest(
            PROVIDER_ID,
            ctx.tenant_id().as_str(),
            ctx.principal_id().as_str(),
            op,
            fields,
        )
    }

    fn redacted(label: &str, value: &str) -> String {
        format!("{label}=[REDACTED:{} chars]", value.len())
    }
}

impl Connector for EpicFhirConnector {
    fn provider_id(&self) -> &str {
        PROVIDER_ID
    }
    fn capabilities(&self) -> ConnectorCapabilities {
        // Epic FHIR is read-heavy; create/update/delete depend on profile.
        ConnectorCapabilities {
            list: true,
            get: true,
            create: true,     // e.g. POST MedicationRequest
            update: true,     // PATCH/PUT
            delete: false,    // Epic generally forbids hard-delete of clinical records
            subscribe: false, // FHIR Subscriptions are tenant-config dependent
        }
    }
    fn list(&self, ctx: &ConnectorCtx, entity_kind: &str, cursor: Option<Cursor>) -> Result<Page> {
        self.check_kind(entity_kind)?;
        let cursor_value = cursor.as_ref().map(Cursor::as_str).unwrap_or("");
        let audit_digest = self.operation_digest(
            ctx,
            "connector.list",
            [("entity_kind", entity_kind), ("cursor", cursor_value)],
        );
        self.seal(ctx, "connector.list", &audit_digest)?;
        let store = self.lock_store();
        let items = store
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(entity_kind));
        const PAGE: usize = 100;
        btree_keyset_page(items, cursor.as_ref(), PAGE)
    }
    fn get(&self, ctx: &ConnectorCtx, entity_kind: &str, id: &str) -> Result<EntityDoc> {
        self.check_kind(entity_kind)?;
        let audit_digest = self.operation_digest(
            ctx,
            "connector.get",
            [("entity_kind", entity_kind), ("id", id)],
        );
        self.seal(ctx, "connector.get", &audit_digest)?;
        self.lock_store()
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(entity_kind))
            .and_then(|m| m.get(id))
            .cloned()
            .ok_or_else(|| ConnectorError::NotFound(format!("fhir {entity_kind}/{id}")))
    }
    fn create(
        &self,
        ctx: &ConnectorCtx,
        entity_kind: &str,
        payload: EntityDoc,
        idempotency_key: IdempotencyKey,
    ) -> Result<EntityDoc> {
        self.check_kind(entity_kind)?;
        let submitted_payload = payload.clone();
        let submitted_digest = entity_doc_payload_digest(&submitted_payload);
        let prev = self
            .lock_idem()
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(entity_kind))
            .and_then(|m| m.get(idempotency_key.as_str()))
            .cloned();
        if let Some((p, previous_digest)) = prev {
            if previous_digest != submitted_digest {
                let conflict_digest = self.operation_digest(
                    ctx,
                    "connector.create",
                    [
                        ("entity_kind", entity_kind),
                        ("id", p.as_str()),
                        ("idempotency_key", idempotency_key.as_str()),
                        ("outcome", "idempotency_conflict"),
                        ("submitted_digest", submitted_digest.as_str()),
                        ("stored_digest", previous_digest.as_str()),
                    ],
                );
                self.seal(ctx, "connector.create", &conflict_digest)?;
                return Err(ConnectorError::IdempotencyConflict(format!(
                    "{PROVIDER_ID} {} entity_kind={} {}",
                    Self::redacted("tenant", ctx.tenant_id().as_str()),
                    entity_kind,
                    Self::redacted("idempotency_key", idempotency_key.as_str())
                )));
            }
            let previous_result = self
                .lock_store()
                .get(ctx.tenant_id().as_str())
                .and_then(|m| m.get(entity_kind))
                .and_then(|m| m.get(&p))
                .cloned()
                .ok_or_else(|| {
                    ConnectorError::NotFound(format!("{PROVIDER_ID} {entity_kind}/{p}"))
                })?;
            let doc_digest = entity_doc_payload_digest(&previous_result);
            let replay_digest = self.operation_digest(
                ctx,
                "connector.create",
                [
                    ("entity_kind", entity_kind),
                    ("id", p.as_str()),
                    ("idempotency_key", idempotency_key.as_str()),
                    ("outcome", "idempotent_replay"),
                    ("submitted_digest", submitted_digest.as_str()),
                    ("doc_digest", doc_digest.as_str()),
                ],
            );
            self.seal(ctx, "connector.create", &replay_digest)?;
            return Ok(previous_result);
        }
        let v = self.next_seq_candidate();
        let id = format!("{entity_kind}-{v:08}");
        let mut doc = payload;
        doc.insert("id", EntityValue::Str(id.clone()));
        doc.insert("resourceType", EntityValue::Str(entity_kind.to_owned()));
        let doc_digest = entity_doc_payload_digest(&doc);
        let audit_digest = self.operation_digest(
            ctx,
            "connector.create",
            [
                ("entity_kind", entity_kind),
                ("id", id.as_str()),
                ("idempotency_key", idempotency_key.as_str()),
                ("submitted_digest", submitted_digest.as_str()),
                ("doc_digest", doc_digest.as_str()),
            ],
        );
        self.seal(ctx, "connector.create", &audit_digest)?;
        self.commit_seq();
        self.put(ctx.tenant_id().as_str(), entity_kind, &id, doc.clone());
        self.lock_idem()
            .entry(ctx.tenant_id().as_str().to_owned())
            .or_default()
            .entry(entity_kind.to_owned())
            .or_default()
            .insert(
                idempotency_key.as_str().to_owned(),
                (id.clone(), submitted_digest.clone()),
            );
        Ok(doc)
    }
    fn update(
        &self,
        ctx: &ConnectorCtx,
        entity_kind: &str,
        id: &str,
        patch: PatchOp,
        idempotency_key: IdempotencyKey,
    ) -> Result<EntityDoc> {
        self.check_kind(entity_kind)?;
        let patch_digest = patch_op_payload_digest(&patch);
        let mut doc = self.get(ctx, entity_kind, id)?;
        doc.insert(
            patch.field.clone(),
            patch.value.unwrap_or(EntityValue::Null),
        );
        let doc_digest = entity_doc_payload_digest(&doc);
        let audit_digest = self.operation_digest(
            ctx,
            "connector.update",
            [
                ("entity_kind", entity_kind),
                ("id", id),
                ("idempotency_key", idempotency_key.as_str()),
                ("patch_digest", patch_digest.as_str()),
                ("doc_digest", doc_digest.as_str()),
            ],
        );
        self.seal(ctx, "connector.update", &audit_digest)?;
        self.put(ctx.tenant_id().as_str(), entity_kind, id, doc.clone());
        Ok(doc)
    }
    fn delete(&self, _ctx: &ConnectorCtx, _entity_kind: &str, _id: &str) -> Result<()> {
        Err(ConnectorError::Unsupported(
            "epic-fhir forbids hard-delete of clinical resources".into(),
        ))
    }
    fn subscribe(
        &self,
        _ctx: &ConnectorCtx,
        _entity_kinds: &[String],
    ) -> Result<Box<dyn EventStream>> {
        Err(ConnectorError::Unsupported(
            "FHIR Subscription requires tenant-side configuration; deferred".into(),
        ))
    }
    fn health(&self) -> Result<HealthReport> {
        Ok(HealthReport {
            reachable: true,
            last_latency_ms: 200,
            upstream_status: "ok".to_owned(),
        })
    }
    fn rate_limits(&self) -> RateLimitDescriptor {
        // Epic FHIR: tenant-configurable; published default ~5 req/sec.
        RateLimitDescriptor {
            requests_per_second: 5,
            burst_capacity: 20,
            daily_quota: None,
            note: "Epic FHIR tenant-configurable; conservative default".to_owned(),
        }
    }
    fn auth_scheme(&self) -> AuthScheme {
        AuthScheme::SignedJwt
    }
    fn ontology_projections(&self) -> Vec<OntologyProjection> {
        vec![
            OntologyProjection::new("Patient", "fhir:Patient")
                .map_field("givenName", "given")
                .map_field("familyName", "family"),
            OntologyProjection::new("Encounter", "fhir:Encounter")
                .map_field("subject", "subject")
                .map_field("status", "status"),
            OntologyProjection::new("Observation", "fhir:Observation")
                .map_field("code", "code")
                .map_field("value", "valueQuantity"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_connector_kernel::{
        AuditSealHandle, PrincipalId, SecretReference, TenantId, TraceContext,
    };
    fn ctx() -> ConnectorCtx {
        ConnectorCtx::new(
            TenantId::new("t-1").unwrap(),
            PrincipalId::new("svc-fhir").unwrap(),
            SecretReference::new("sref://t-1/epic-fhir/jwt").unwrap(),
            TraceContext::new("00-trace").unwrap(),
            AuditSealHandle::new("chain-1").unwrap(),
        )
    }
    fn ik(s: &str) -> IdempotencyKey {
        IdempotencyKey::new(format!("{s:0>16}")).unwrap()
    }
    #[test]
    fn provider() {
        assert_eq!(EpicFhirConnector::new().provider_id(), "epic-fhir");
    }
    #[test]
    fn list_patients() {
        assert_eq!(
            EpicFhirConnector::new()
                .list(&ctx(), "Patient", None)
                .unwrap()
                .items
                .len(),
            5
        );
    }
    #[test]
    fn list_uses_keyset_page_boundaries() {
        let s = EpicFhirConnector::new();
        for i in 1..=101 {
            let mut d = EntityDoc::new();
            let id = format!("MedicationRequest-{i:08}");
            d.insert("id", EntityValue::Str(id.clone()));
            d.insert("resourceType", EntityValue::Str("MedicationRequest".into()));
            s.put("t-1", "MedicationRequest", &id, d);
        }

        let first = s.list(&ctx(), "MedicationRequest", None).unwrap();
        assert_eq!(first.items.len(), 100);
        assert_eq!(
            first.next_cursor.as_ref().map(Cursor::as_str),
            Some("MedicationRequest-00000100")
        );

        let second = s
            .list(&ctx(), "MedicationRequest", first.next_cursor)
            .unwrap();
        assert_eq!(second.items.len(), 1);
        assert!(second.next_cursor.is_none());

        let exact = EpicFhirConnector::new();
        for i in 1..=100 {
            let mut d = EntityDoc::new();
            let id = format!("MedicationRequest-{i:08}");
            d.insert("id", EntityValue::Str(id.clone()));
            d.insert("resourceType", EntityValue::Str("MedicationRequest".into()));
            exact.put("t-1", "MedicationRequest", &id, d);
        }
        let exact_page = exact.list(&ctx(), "MedicationRequest", None).unwrap();
        assert_eq!(exact_page.items.len(), 100);
        assert!(exact_page.next_cursor.is_none());
    }
    #[test]
    fn get_patient_ok() {
        assert!(
            EpicFhirConnector::new()
                .get(&ctx(), "Patient", "Patient-00000001")
                .is_ok()
        );
    }
    #[test]
    fn unknown_not_found() {
        assert!(
            EpicFhirConnector::new()
                .get(&ctx(), "Patient", "missing")
                .is_err()
        );
    }
    #[test]
    fn unsupported_resource() {
        assert!(matches!(
            EpicFhirConnector::new().list(&ctx(), "FooResource", None),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn create_observation_stamps_resource_type() {
        let s = EpicFhirConnector::new();
        let mut d = EntityDoc::new();
        d.insert("code", EntityValue::Str("8302-2".into())); // LOINC: body height
        let r = s.create(&ctx(), "Observation", d, ik("k")).unwrap();
        assert_eq!(
            r.get("resourceType"),
            Some(&EntityValue::Str("Observation".into()))
        );
    }
    #[test]
    fn update_then_get() {
        let s = EpicFhirConnector::new();
        s.update(
            &ctx(),
            "Encounter",
            "Encounter-00000001",
            PatchOp::set("status", EntityValue::Str("in-progress".into())),
            ik("u"),
        )
        .unwrap();
        assert_eq!(
            s.get(&ctx(), "Encounter", "Encounter-00000001")
                .unwrap()
                .get("status"),
            Some(&EntityValue::Str("in-progress".into()))
        );
    }
    #[test]
    fn delete_forbidden() {
        assert!(matches!(
            EpicFhirConnector::new().delete(&ctx(), "Patient", "Patient-00000001"),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn subscribe_deferred() {
        assert!(matches!(
            EpicFhirConnector::new().subscribe(&ctx(), &["Patient".into()]),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn auth_signed_jwt() {
        assert_eq!(
            EpicFhirConnector::new().auth_scheme(),
            AuthScheme::SignedJwt
        );
    }
    #[test]
    fn ontology_projections_cover_clinical() {
        let p = EpicFhirConnector::new().ontology_projections();
        assert!(p.iter().any(|x| x.object_type == "Patient"));
        assert!(p.iter().any(|x| x.object_type == "Encounter"));
        assert!(p.iter().any(|x| x.object_type == "Observation"));
    }
    #[test]
    fn audit_seal_rejects_raw_payload_inputs() {
        let s = EpicFhirConnector::new();
        assert!(matches!(
            s.seal(&ctx(), "connector.get", "Patient"),
            Err(ConnectorError::AuditSealFailed(_))
        ));

        let digest = s.operation_digest(
            &ctx(),
            "connector.get",
            [("entity_kind", "Patient"), ("id", "Patient-00000001")],
        );
        assert!(s.seal(&ctx(), "connector.get", &digest).is_ok());
    }

    #[test]
    fn idempotency_conflict_redacts_tenant_and_key() {
        let s = EpicFhirConnector::new();
        let key = ik("conflict");
        let mut first = EntityDoc::new();
        first.insert("family", EntityValue::Str("Y".into()));
        let _ = s.create(&ctx(), "Patient", first, key.clone()).unwrap();

        let mut second = EntityDoc::new();
        second.insert("family", EntityValue::Str("Z".into()));
        let err = s.create(&ctx(), "Patient", second, key).unwrap_err();

        match err {
            ConnectorError::IdempotencyConflict(message) => {
                assert!(message.contains("[REDACTED"));
                assert!(!message.contains("t-1"));
                assert!(!message.contains("conflict"));
            }
            other => panic!("expected idempotency conflict, got {other:?}"),
        }
    }
}
