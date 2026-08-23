//! `gateway-gusto-connector` — Gusto Embedded Payroll connector.
//!
//! # Coverage
//!
//! Gusto Embedded Payroll API:
//! * `/companies/{id}/employees` — employees + compensations
//! * `/companies/{id}/payrolls` — pay runs
//! * `/companies/{id}/contractors` — contractor pay
//!
//! # Auth
//!
//! OAuth 2.0 with company-scoped access tokens.
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
const PROVIDER_ID: &str = "gusto";

pub struct GustoConnector {
    store: Mutex<Store>,
    idem: Mutex<IdemMap>,
    next_id: Mutex<u64>,
}

impl Default for GustoConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl GustoConnector {
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
                let id = format!("emp-{i:08}");
                d.insert("id", EntityValue::Str(id.clone()));
                d.insert("uuid", EntityValue::Str(id.clone()));
                d.insert("first_name", EntityValue::Str(format!("Gus{i}")));
                d.insert("last_name", EntityValue::Str("SMB".into()));
                self.put(tenant, "employee", &id, d);
            }
            for i in 1..=2 {
                let mut d = EntityDoc::new();
                let id = format!("pr-{i:08}");
                d.insert("id", EntityValue::Str(id.clone()));
                d.insert("check_date", EntityValue::Str("2026-05-15".into()));
                self.put(tenant, "payroll", &id, d);
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
        *g + 3000
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
            "employee" | "payroll" | "contractor" => Ok(()),
            other => Err(ConnectorError::Unsupported(format!(
                "gusto entity_kind={other}"
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

impl Connector for GustoConnector {
    fn provider_id(&self) -> &str {
        PROVIDER_ID
    }
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            list: true,
            get: true,
            create: true,
            update: true,
            delete: false,
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
            .ok_or_else(|| ConnectorError::NotFound(format!("gusto {entity_kind}/{id}")))
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
            "employee" => format!("emp-{v:08}"),
            "payroll" => format!("pr-{v:08}"),
            "contractor" => format!("ctr-{v:08}"),
            _ => format!("x-{v:08}"),
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
    fn delete(&self, _ctx: &ConnectorCtx, _entity_kind: &str, _id: &str) -> Result<()> {
        Err(ConnectorError::Unsupported(
            "gusto does not hard-delete payroll records".into(),
        ))
    }
    fn subscribe(
        &self,
        _ctx: &ConnectorCtx,
        _entity_kinds: &[String],
    ) -> Result<Box<dyn EventStream>> {
        Err(ConnectorError::Unsupported(
            "gusto event webhooks deferred".into(),
        ))
    }
    fn health(&self) -> Result<HealthReport> {
        Ok(HealthReport {
            reachable: true,
            last_latency_ms: 80,
            upstream_status: "ok".to_owned(),
        })
    }
    fn rate_limits(&self) -> RateLimitDescriptor {
        RateLimitDescriptor {
            requests_per_second: 4,
            burst_capacity: 20,
            daily_quota: Some(100_000),
            note: "Gusto Embedded standard tier".to_owned(),
        }
    }
    fn auth_scheme(&self) -> AuthScheme {
        AuthScheme::OAuth2
    }
    fn ontology_projections(&self) -> Vec<OntologyProjection> {
        vec![
            OntologyProjection::new("Employee", "gusto:employee")
                .map_field("givenName", "first_name")
                .map_field("familyName", "last_name"),
            OntologyProjection::new("Payroll", "gusto:payroll")
                .map_field("checkDate", "check_date"),
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
            PrincipalId::new("svc-gusto").unwrap(),
            SecretReference::new("sref://t-1/gusto/oauth").unwrap(),
            TraceContext::new("00-trace").unwrap(),
            AuditSealHandle::new("chain-1").unwrap(),
        )
    }
    fn ik(s: &str) -> IdempotencyKey {
        IdempotencyKey::new(format!("{s:0>16}")).unwrap()
    }
    #[test]
    fn provider() {
        assert_eq!(GustoConnector::new().provider_id(), "gusto");
    }
    #[test]
    fn ten_employees_smoke() {
        assert_eq!(
            GustoConnector::new()
                .list(&ctx(), "employee", None)
                .unwrap()
                .items
                .len(),
            10
        );
    }
    #[test]
    fn list_uses_keyset_page_boundaries() {
        let s = GustoConnector::new();
        for i in 1..=101 {
            let mut d = EntityDoc::new();
            let id = format!("ctr-{i:08}");
            d.insert("id", EntityValue::Str(id.clone()));
            s.put("t-1", "contractor", &id, d);
        }

        let first = s.list(&ctx(), "contractor", None).unwrap();
        assert_eq!(first.items.len(), 100);
        assert_eq!(
            first.next_cursor.as_ref().map(Cursor::as_str),
            Some("ctr-00000100")
        );

        let second = s.list(&ctx(), "contractor", first.next_cursor).unwrap();
        assert_eq!(second.items.len(), 1);
        assert!(second.next_cursor.is_none());

        let exact = GustoConnector::new();
        for i in 1..=100 {
            let mut d = EntityDoc::new();
            let id = format!("ctr-{i:08}");
            d.insert("id", EntityValue::Str(id.clone()));
            exact.put("t-1", "contractor", &id, d);
        }
        let exact_page = exact.list(&ctx(), "contractor", None).unwrap();
        assert_eq!(exact_page.items.len(), 100);
        assert!(exact_page.next_cursor.is_none());
    }
    #[test]
    fn get_employee_ok() {
        assert!(
            GustoConnector::new()
                .get(&ctx(), "employee", "emp-00000001")
                .is_ok()
        );
    }
    #[test]
    fn unknown_employee_not_found() {
        assert!(
            GustoConnector::new()
                .get(&ctx(), "employee", "missing")
                .is_err()
        );
    }
    #[test]
    fn list_payrolls() {
        assert!(
            !GustoConnector::new()
                .list(&ctx(), "payroll", None)
                .unwrap()
                .items
                .is_empty()
        );
    }
    #[test]
    fn unsupported_kind() {
        assert!(matches!(
            GustoConnector::new().list(&ctx(), "x", None),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn idempotent_create() {
        let s = GustoConnector::new();
        let mut d = EntityDoc::new();
        d.insert("first_name", EntityValue::Str("Q".into()));
        let k = ik("k");
        let a = s.create(&ctx(), "employee", d.clone(), k.clone()).unwrap();
        let b = s.create(&ctx(), "employee", d, k).unwrap();
        assert_eq!(a.get("id"), b.get("id"));
    }
    #[test]
    fn delete_unsupported() {
        assert!(matches!(
            GustoConnector::new().delete(&ctx(), "employee", "emp-00000001"),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn auth_oauth2() {
        assert_eq!(GustoConnector::new().auth_scheme(), AuthScheme::OAuth2);
    }
    #[test]
    fn ontology_projections_present() {
        assert!(!GustoConnector::new().ontology_projections().is_empty());
    }
    #[test]
    fn rate_limits_published() {
        assert!(GustoConnector::new().rate_limits().requests_per_second > 0);
    }
    #[test]
    fn audit_seal_rejects_raw_payload_inputs() {
        let s = GustoConnector::new();
        assert!(matches!(
            s.seal(&ctx(), "connector.get", "employee"),
            Err(ConnectorError::AuditSealFailed(_))
        ));

        let digest = s.operation_digest(
            &ctx(),
            "connector.get",
            [("entity_kind", "employee"), ("id", "emp-00000001")],
        );
        assert!(s.seal(&ctx(), "connector.get", &digest).is_ok());
    }

    #[test]
    fn idempotency_conflict_redacts_tenant_and_key() {
        let s = GustoConnector::new();
        let key = ik("conflict");
        let mut first = EntityDoc::new();
        first.insert("first_name", EntityValue::Str("Y".into()));
        let _ = s.create(&ctx(), "employee", first, key.clone()).unwrap();

        let mut second = EntityDoc::new();
        second.insert("first_name", EntityValue::Str("Z".into()));
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

    #[test]
    fn update_employee_then_get() {
        let s = GustoConnector::new();
        s.update(
            &ctx(),
            "employee",
            "emp-00000001",
            PatchOp::set("first_name", EntityValue::Str("Updated".into())),
            ik("update"),
        )
        .unwrap();
        assert_eq!(
            s.get(&ctx(), "employee", "emp-00000001")
                .unwrap()
                .get("first_name"),
            Some(&EntityValue::Str("Updated".into()))
        );
    }
}
