//! `gateway-rippling-connector` — Rippling unified HRIS/IT/Finance connector.
//!
//! # Coverage
//!
//! Rippling REST API surfaces:
//! * `/employees` — HRIS
//! * `/devices` — IT (device fleet)
//! * `/transactions` — Finance (expense / ledger transactions)
//!
//! # Auth
//!
//! Long-lived API key (header bearer). SecretReference resolves the API
//! key in OpenBao.
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
const PROVIDER_ID: &str = "rippling";

pub struct RipplingConnector {
    store: Mutex<Store>,
    idem: Mutex<IdemMap>,
    next_id: Mutex<u64>,
}

impl Default for RipplingConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl RipplingConnector {
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
            for i in 1..=10 {
                let mut d = EntityDoc::new();
                let id = format!("emp-{i:06}");
                d.insert("id", EntityValue::Str(id.clone()));
                d.insert("firstName", EntityValue::Str(format!("Rip{i}")));
                d.insert("lastName", EntityValue::Str("Employee".into()));
                d.insert("status", EntityValue::Str("ACTIVE".into()));
                self.put(tenant, "employee", &id, d);
            }
            for i in 1..=2 {
                let mut d = EntityDoc::new();
                d.insert("id", EntityValue::Str(format!("dev-{i:06}")));
                d.insert("employeeId", EntityValue::Str(format!("emp-{i:06}")));
                d.insert("model", EntityValue::Str("MacBook Pro".into()));
                self.put(tenant, "device", &format!("dev-{i:06}"), d);
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
        *g + 2000
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
            "employee" | "device" | "transaction" => Ok(()),
            other => Err(ConnectorError::Unsupported(format!(
                "rippling entity_kind={other}"
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

impl Connector for RipplingConnector {
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
            subscribe: false,
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
            .ok_or_else(|| ConnectorError::NotFound(format!("rippling {entity_kind}/{id}")))
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
        let id = match entity_kind {
            "employee" => format!("emp-{v:06}"),
            "device" => format!("dev-{v:06}"),
            "transaction" => format!("txn-{v:08}"),
            _ => format!("x-{v:06}"),
        };
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
            "rippling webhook subscription deferred".into(),
        ))
    }
    fn health(&self) -> Result<HealthReport> {
        Ok(HealthReport {
            reachable: true,
            last_latency_ms: 95,
            upstream_status: "ok".to_owned(),
        })
    }
    fn rate_limits(&self) -> RateLimitDescriptor {
        RateLimitDescriptor {
            requests_per_second: 10,
            burst_capacity: 50,
            daily_quota: Some(250_000),
            note: "Rippling REST default app rate-limit".to_owned(),
        }
    }
    fn auth_scheme(&self) -> AuthScheme {
        AuthScheme::ApiKey
    }
    fn ontology_projections(&self) -> Vec<OntologyProjection> {
        vec![
            OntologyProjection::new("Employee", "rippling:employee")
                .map_field("givenName", "firstName")
                .map_field("familyName", "lastName"),
            OntologyProjection::new("Device", "rippling:device").map_field("model", "model"),
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
            PrincipalId::new("svc-rip").unwrap(),
            SecretReference::new("sref://t-1/rippling/key").unwrap(),
            TraceContext::new("00-trace").unwrap(),
            AuditSealHandle::new("chain-1").unwrap(),
        )
    }
    fn ik(s: &str) -> IdempotencyKey {
        IdempotencyKey::new(format!("{s:0>16}")).unwrap()
    }
    #[test]
    fn provider_id() {
        assert_eq!(RipplingConnector::new().provider_id(), "rippling");
    }
    #[test]
    fn smoke_ten_employees() {
        assert_eq!(
            RipplingConnector::new()
                .list(&ctx(), "employee", None)
                .unwrap()
                .items
                .len(),
            10
        );
    }
    #[test]
    fn list_uses_keyset_page_boundaries() {
        let s = RipplingConnector::new();
        for i in 1..=101 {
            let mut d = EntityDoc::new();
            let id = format!("txn-{i:08}");
            d.insert("id", EntityValue::Str(id.clone()));
            s.put("t-1", "transaction", &id, d);
        }

        let first = s.list(&ctx(), "transaction", None).unwrap();
        assert_eq!(first.items.len(), 100);
        assert_eq!(
            first.next_cursor.as_ref().map(Cursor::as_str),
            Some("txn-00000100")
        );

        let second = s.list(&ctx(), "transaction", first.next_cursor).unwrap();
        assert_eq!(second.items.len(), 1);
        assert!(second.next_cursor.is_none());

        let exact = RipplingConnector::new();
        for i in 1..=100 {
            let mut d = EntityDoc::new();
            let id = format!("txn-{i:08}");
            d.insert("id", EntityValue::Str(id.clone()));
            exact.put("t-1", "transaction", &id, d);
        }
        let exact_page = exact.list(&ctx(), "transaction", None).unwrap();
        assert_eq!(exact_page.items.len(), 100);
        assert!(exact_page.next_cursor.is_none());
    }
    #[test]
    fn get_employee_ok() {
        assert!(
            RipplingConnector::new()
                .get(&ctx(), "employee", "emp-000001")
                .is_ok()
        );
    }
    #[test]
    fn get_unknown_not_found() {
        assert!(
            RipplingConnector::new()
                .get(&ctx(), "employee", "missing")
                .is_err()
        );
    }
    #[test]
    fn unsupported_kind() {
        assert!(matches!(
            RipplingConnector::new().list(&ctx(), "x", None),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn idempotent() {
        let s = RipplingConnector::new();
        let mut d = EntityDoc::new();
        d.insert("firstName", EntityValue::Str("Z".into()));
        let k = ik("k");
        let a = s.create(&ctx(), "employee", d.clone(), k.clone()).unwrap();
        let b = s.create(&ctx(), "employee", d, k).unwrap();
        assert_eq!(a.get("id"), b.get("id"));
    }
    #[test]
    fn update_then_get() {
        let s = RipplingConnector::new();
        s.update(
            &ctx(),
            "employee",
            "emp-000001",
            PatchOp::set("status", EntityValue::Str("TERMINATED".into())),
            ik("u"),
        )
        .unwrap();
        assert_eq!(
            s.get(&ctx(), "employee", "emp-000001")
                .unwrap()
                .get("status"),
            Some(&EntityValue::Str("TERMINATED".into()))
        );
    }
    #[test]
    fn delete_then_get_not_found() {
        let s = RipplingConnector::new();
        s.delete(&ctx(), "device", "dev-000001").unwrap();
        assert!(s.get(&ctx(), "device", "dev-000001").is_err());
    }
    #[test]
    fn auth_api_key() {
        assert_eq!(RipplingConnector::new().auth_scheme(), AuthScheme::ApiKey);
    }
    #[test]
    fn subscribe_unsupported() {
        assert!(matches!(
            RipplingConnector::new().subscribe(&ctx(), &["employee".into()]),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn ontology_projection_present() {
        assert!(!RipplingConnector::new().ontology_projections().is_empty());
    }
    #[test]
    fn audit_seal_rejects_raw_payload_inputs() {
        let s = RipplingConnector::new();
        assert!(matches!(
            s.seal(&ctx(), "connector.get", "employee"),
            Err(ConnectorError::AuditSealFailed(_))
        ));

        let digest = s.operation_digest(
            &ctx(),
            "connector.get",
            [("entity_kind", "employee"), ("id", "emp-000001")],
        );
        assert!(s.seal(&ctx(), "connector.get", &digest).is_ok());
    }

    #[test]
    fn idempotency_conflict_redacts_tenant_and_key() {
        let s = RipplingConnector::new();
        let key = ik("conflict");
        let mut first = EntityDoc::new();
        first.insert("firstName", EntityValue::Str("Y".into()));
        let _ = s.create(&ctx(), "employee", first, key.clone()).unwrap();

        let mut second = EntityDoc::new();
        second.insert("firstName", EntityValue::Str("Z".into()));
        let err = s.create(&ctx(), "employee", second, key).unwrap_err();

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
