//! `gateway-netsuite-connector` — NetSuite ERP connector.
//!
//! # Coverage
//!
//! NetSuite SuiteTalk REST surface (SOAP fallback follows the same shape):
//! * `customer` — companies
//! * `vendor` — vendors
//! * `salesOrder` — sales orders (multi-subsidiary)
//! * `journalEntry` — accounting journals
//!
//! # Auth
//!
//! Token-based authentication (TBA): account-id-scoped, OAuth1-style signed.
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
const PROVIDER_ID: &str = "netsuite";

pub struct NetsuiteConnector {
    store: Mutex<Store>,
    idem: Mutex<IdemMap>,
    next_id: Mutex<u64>,
}

impl Default for NetsuiteConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl NetsuiteConnector {
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
                let id = format!("{i}");
                let mut d = EntityDoc::new();
                d.insert("id", EntityValue::Str(id.clone()));
                d.insert("companyName", EntityValue::Str(format!("NS Co {i}")));
                d.insert("subsidiary", EntityValue::Int(1));
                self.put(tenant, "customer", &id, d);
            }
            for i in 1..=3 {
                let id = format!("v-{i}");
                let mut d = EntityDoc::new();
                d.insert("id", EntityValue::Str(id.clone()));
                d.insert("companyName", EntityValue::Str(format!("Vendor {i}")));
                self.put(tenant, "vendor", &id, d);
            }
            for i in 1..=2 {
                let id = format!("so-{i}");
                let mut d = EntityDoc::new();
                d.insert("id", EntityValue::Str(id.clone()));
                d.insert("entity", EntityValue::Str(format!("{i}")));
                d.insert("total", EntityValue::Int(1_500_000));
                self.put(tenant, "salesOrder", &id, d);
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
        *g + 5000
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
            "customer" | "vendor" | "salesOrder" | "journalEntry" => Ok(()),
            other => Err(ConnectorError::Unsupported(format!(
                "netsuite recordType={other}"
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

impl Connector for NetsuiteConnector {
    fn provider_id(&self) -> &str {
        PROVIDER_ID
    }
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            list: true,
            get: true,
            create: true,
            update: true,
            delete: true,
            subscribe: false, // SuiteScript SDF deploy required for events; out of scope here
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
            .ok_or_else(|| ConnectorError::NotFound(format!("netsuite {entity_kind}/{id}")))
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
        let id = format!("{v}");
        let mut doc = payload;
        doc.insert("id", EntityValue::Str(id.clone()));
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
    fn delete(&self, ctx: &ConnectorCtx, entity_kind: &str, id: &str) -> Result<()> {
        self.check_kind(entity_kind)?;
        let deleted_doc = self
            .lock_store()
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(entity_kind))
            .and_then(|m| m.get(id))
            .cloned()
            .ok_or_else(|| ConnectorError::NotFound(format!("{PROVIDER_ID} {entity_kind}/{id}")))?;
        let doc_digest = entity_doc_payload_digest(&deleted_doc);
        let audit_digest = self.operation_digest(
            ctx,
            "connector.delete",
            [
                ("entity_kind", entity_kind),
                ("id", id),
                ("doc_digest", doc_digest.as_str()),
            ],
        );
        self.seal(ctx, "connector.delete", &audit_digest)?;
        let _ = self
            .lock_store()
            .get_mut(ctx.tenant_id().as_str())
            .and_then(|m| m.get_mut(entity_kind))
            .and_then(|m| m.remove(id));
        Ok(())
    }
    fn subscribe(
        &self,
        _ctx: &ConnectorCtx,
        _entity_kinds: &[String],
    ) -> Result<Box<dyn EventStream>> {
        Err(ConnectorError::Unsupported(
            "netsuite event subscription requires SuiteScript SDF deploy".into(),
        ))
    }
    fn health(&self) -> Result<HealthReport> {
        Ok(HealthReport {
            reachable: true,
            last_latency_ms: 250,
            upstream_status: "ok".to_owned(),
        })
    }
    fn rate_limits(&self) -> RateLimitDescriptor {
        RateLimitDescriptor {
            requests_per_second: 4,
            burst_capacity: 10,
            daily_quota: None,
            note: "NetSuite SuiteTalk concurrency-gov 10 (Enterprise)".to_owned(),
        }
    }
    fn auth_scheme(&self) -> AuthScheme {
        AuthScheme::SignedJwt
    }
    fn ontology_projections(&self) -> Vec<OntologyProjection> {
        vec![
            OntologyProjection::new("Customer", "netsuite:customer")
                .map_field("name", "companyName"),
            OntologyProjection::new("Vendor", "netsuite:vendor").map_field("name", "companyName"),
            OntologyProjection::new("SalesOrder", "netsuite:salesOrder")
                .map_field("total", "total"),
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
            PrincipalId::new("svc-ns").unwrap(),
            SecretReference::new("sref://t-1/netsuite/tba").unwrap(),
            TraceContext::new("00-trace").unwrap(),
            AuditSealHandle::new("chain-1").unwrap(),
        )
    }
    fn ik(s: &str) -> IdempotencyKey {
        IdempotencyKey::new(format!("{s:0>16}")).unwrap()
    }
    #[test]
    fn provider() {
        assert_eq!(NetsuiteConnector::new().provider_id(), "netsuite");
    }
    #[test]
    fn capabilities_no_subscribe() {
        assert!(!NetsuiteConnector::new().capabilities().subscribe);
    }
    #[test]
    fn list_customers() {
        let p = NetsuiteConnector::new()
            .list(&ctx(), "customer", None)
            .unwrap();
        assert_eq!(p.items.len(), 5);
    }
    #[test]
    fn list_uses_keyset_page_boundaries() {
        let s = NetsuiteConnector::new();
        for i in 1..=101 {
            let mut d = EntityDoc::new();
            let id = format!("je-{i:08}");
            d.insert("id", EntityValue::Str(id.clone()));
            s.put("t-1", "journalEntry", &id, d);
        }

        let first = s.list(&ctx(), "journalEntry", None).unwrap();
        assert_eq!(first.items.len(), 100);
        assert_eq!(
            first.next_cursor.as_ref().map(Cursor::as_str),
            Some("je-00000100")
        );

        let second = s.list(&ctx(), "journalEntry", first.next_cursor).unwrap();
        assert_eq!(second.items.len(), 1);
        assert!(second.next_cursor.is_none());

        let exact = NetsuiteConnector::new();
        for i in 1..=100 {
            let mut d = EntityDoc::new();
            let id = format!("je-{i:08}");
            d.insert("id", EntityValue::Str(id.clone()));
            exact.put("t-1", "journalEntry", &id, d);
        }
        let exact_page = exact.list(&ctx(), "journalEntry", None).unwrap();
        assert_eq!(exact_page.items.len(), 100);
        assert!(exact_page.next_cursor.is_none());
    }
    #[test]
    fn get_customer_ok() {
        assert!(
            NetsuiteConnector::new()
                .get(&ctx(), "customer", "1")
                .is_ok()
        );
    }
    #[test]
    fn unknown_record_not_found() {
        assert!(
            NetsuiteConnector::new()
                .get(&ctx(), "customer", "999")
                .is_err()
        );
    }
    #[test]
    fn unsupported_record_type() {
        assert!(matches!(
            NetsuiteConnector::new().list(&ctx(), "no-such", None),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn idempotent_create() {
        let s = NetsuiteConnector::new();
        let mut d = EntityDoc::new();
        d.insert("companyName", EntityValue::Str("New".into()));
        let k = ik("k");
        let a = s.create(&ctx(), "customer", d.clone(), k.clone()).unwrap();
        let b = s.create(&ctx(), "customer", d, k).unwrap();
        assert_eq!(a.get("id"), b.get("id"));
    }
    #[test]
    fn update_changes_field() {
        let s = NetsuiteConnector::new();
        s.update(
            &ctx(),
            "customer",
            "1",
            PatchOp::set("companyName", EntityValue::Str("Renamed".into())),
            ik("u"),
        )
        .unwrap();
        assert_eq!(
            s.get(&ctx(), "customer", "1").unwrap().get("companyName"),
            Some(&EntityValue::Str("Renamed".into()))
        );
    }
    #[test]
    fn delete_then_not_found() {
        let s = NetsuiteConnector::new();
        s.delete(&ctx(), "customer", "1").unwrap();
        assert!(s.get(&ctx(), "customer", "1").is_err());
    }
    #[test]
    fn subscribe_unsupported() {
        assert!(matches!(
            NetsuiteConnector::new().subscribe(&ctx(), &["customer".into()]),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn auth_signed_jwt() {
        assert_eq!(
            NetsuiteConnector::new().auth_scheme(),
            AuthScheme::SignedJwt
        );
    }
    #[test]
    fn ontology_projections_present() {
        assert!(NetsuiteConnector::new().ontology_projections().len() >= 3);
    }
    #[test]
    fn audit_seal_rejects_raw_payload_inputs() {
        let s = NetsuiteConnector::new();
        assert!(matches!(
            s.seal(&ctx(), "connector.get", "customer"),
            Err(ConnectorError::AuditSealFailed(_))
        ));

        let digest = s.operation_digest(
            &ctx(),
            "connector.get",
            [("entity_kind", "customer"), ("id", "1")],
        );
        assert!(s.seal(&ctx(), "connector.get", &digest).is_ok());
    }

    #[test]
    fn idempotency_conflict_redacts_tenant_and_key() {
        let s = NetsuiteConnector::new();
        let key = ik("conflict");
        let mut first = EntityDoc::new();
        first.insert("companyName", EntityValue::Str("Y".into()));
        let _ = s.create(&ctx(), "customer", first, key.clone()).unwrap();

        let mut second = EntityDoc::new();
        second.insert("companyName", EntityValue::Str("Z".into()));
        let err = s.create(&ctx(), "customer", second, key).unwrap_err();

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
