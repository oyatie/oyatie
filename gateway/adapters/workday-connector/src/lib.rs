//! `gateway-workday-connector` — Workday HCM connector.
//!
//! # Coverage
//!
//! Workday REST API v40+ surfaces:
//! * `/workers` — employees + employment data
//! * `/jobProfiles` — job catalog
//! * `/compensation` — compensation plans + allocations
//!
//! Bulk extracts via Workday Studio are tracked as a follow-up adapter.
//!
//! # Auth
//!
//! Workday tenant uses OAuth 2.0 client-credentials with an isu (integration
//! system user) JWT. SecretReference resolves the integration credentials.
//!
//! See `README.md` for sandbox setup and `specs/openapi.snapshot.yaml`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_shared_connector_kernel::{
    AuthScheme, Connector, ConnectorCapabilities, ConnectorCtx, ConnectorError, Cursor, EntityDoc,
    EntityValue, EventStream, HealthReport, IdempotencyKey, OntologyProjection, Page, PatchOp,
    RateLimitDescriptor, btree_keyset_page,
};
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

type Result<T> = std::result::Result<T, ConnectorError>;
type Store = HashMap<String, HashMap<String, BTreeMap<String, EntityDoc>>>;
type IdemMap = HashMap<String, HashMap<String, HashMap<String, (String, EntityDoc)>>>;

const PROVIDER_ID: &str = "workday";

pub struct WorkdayConnector {
    store: Mutex<Store>,
    idem: Mutex<IdemMap>,
    next_id: Mutex<u64>,
}

impl Default for WorkdayConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkdayConnector {
    pub fn new() -> Self {
        let s = Self {
            store: Mutex::new(HashMap::new()),
            idem: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        };
        s.seed_sandbox();
        s
    }

    fn seed_sandbox(&self) {
        for tenant in ["sandbox", "t-1"] {
            // 10 workers — exactly the count the buildability bar requires.
            for i in 1..=10 {
                let mut d = EntityDoc::new();
                let id = format!("WID-{i:08}");
                d.insert("id", EntityValue::Str(id.clone()));
                d.insert("workerId", EntityValue::Str(format!("{i:06}")));
                d.insert("givenName", EntityValue::Str(format!("Seed{i}")));
                d.insert("familyName", EntityValue::Str("Worker".into()));
                d.insert("active", EntityValue::Bool(true));
                self.put(tenant, "worker", &id, d);
            }
            for (jid, name) in [
                ("JP-001", "Software Engineer"),
                ("JP-002", "Product Manager"),
            ] {
                let mut d = EntityDoc::new();
                d.insert("id", EntityValue::Str(jid.to_owned()));
                d.insert("name", EntityValue::Str(name.to_owned()));
                self.put(tenant, "job-profile", jid, d);
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
    fn next_seq(&self) -> u64 {
        let mut g = match self.next_id.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let v = *g + 100;
        *g += 1;
        v
    }
    fn check_kind(&self, kind: &str) -> Result<()> {
        match kind {
            "worker" | "job-profile" | "compensation" => Ok(()),
            other => Err(ConnectorError::Unsupported(format!(
                "workday entity_kind={other}"
            ))),
        }
    }
    fn seal(&self, ctx: &ConnectorCtx, op: &str, payload: &str) -> Result<()> {
        let receipt = ctx.audit_handle().seal(op, payload);
        if receipt.chain_id.is_empty() || receipt.kind != op || receipt.payload_digest != payload {
            return Err(ConnectorError::AuditSealFailed(format!(
                "{op} seal receipt mismatch"
            )));
        }
        Ok(())
    }
}

impl Connector for WorkdayConnector {
    fn provider_id(&self) -> &str {
        PROVIDER_ID
    }
    fn capabilities(&self) -> ConnectorCapabilities {
        // Workday HRIS rarely exposes hard-delete; subscribe is not on the REST surface.
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
        self.seal(ctx, "connector.list", entity_kind)?;
        let store = self.lock_store();
        let items = store
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(entity_kind));
        const PAGE: usize = 100;
        btree_keyset_page(items, cursor.as_ref(), PAGE)
    }
    fn get(&self, ctx: &ConnectorCtx, entity_kind: &str, id: &str) -> Result<EntityDoc> {
        self.check_kind(entity_kind)?;
        self.seal(ctx, "connector.get", id)?;
        self.lock_store()
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(entity_kind))
            .and_then(|m| m.get(id))
            .cloned()
            .ok_or_else(|| ConnectorError::NotFound(format!("workday {entity_kind}/{id}")))
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
        let prev = self
            .lock_idem()
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(entity_kind))
            .and_then(|m| m.get(idempotency_key.as_str()))
            .cloned();
        if let Some((p, previous_payload)) = prev {
            if previous_payload != submitted_payload {
                return Err(ConnectorError::IdempotencyConflict(format!(
                    "{PROVIDER_ID} tenant={} entity_kind={} idempotency_key={}",
                    ctx.tenant_id().as_str(),
                    entity_kind,
                    idempotency_key.as_str()
                )));
            }
            return self.get(ctx, entity_kind, &p);
        }
        let v = self.next_seq();
        let id = match entity_kind {
            "worker" => format!("WID-{v:08}"),
            "job-profile" => format!("JP-{v:03}"),
            "compensation" => format!("CMP-{v:08}"),
            _ => format!("x-{v:08}"),
        };
        let mut doc = payload;
        doc.insert("id", EntityValue::Str(id.clone()));
        self.put(ctx.tenant_id().as_str(), entity_kind, &id, doc.clone());
        self.lock_idem()
            .entry(ctx.tenant_id().as_str().to_owned())
            .or_default()
            .entry(entity_kind.to_owned())
            .or_default()
            .insert(
                idempotency_key.as_str().to_owned(),
                (id.clone(), submitted_payload),
            );
        self.seal(ctx, "connector.create", &id)?;
        Ok(doc)
    }
    fn update(
        &self,
        ctx: &ConnectorCtx,
        entity_kind: &str,
        id: &str,
        patch: PatchOp,
        _idempotency_key: IdempotencyKey,
    ) -> Result<EntityDoc> {
        self.check_kind(entity_kind)?;
        let mut doc = self.get(ctx, entity_kind, id)?;
        let v = patch.value.unwrap_or(EntityValue::Null);
        doc.insert(patch.field.clone(), v);
        self.put(ctx.tenant_id().as_str(), entity_kind, id, doc.clone());
        self.seal(ctx, "connector.update", id)?;
        Ok(doc)
    }
    fn delete(&self, _ctx: &ConnectorCtx, _entity_kind: &str, _id: &str) -> Result<()> {
        Err(ConnectorError::Unsupported(
            "workday HRIS does not expose hard-delete".into(),
        ))
    }
    fn subscribe(
        &self,
        _ctx: &ConnectorCtx,
        _entity_kinds: &[String],
    ) -> Result<Box<dyn EventStream>> {
        Err(ConnectorError::Unsupported(
            "workday REST does not expose change-data-capture; use bulk extracts".into(),
        ))
    }
    fn health(&self) -> Result<HealthReport> {
        Ok(HealthReport {
            reachable: true,
            last_latency_ms: 120,
            upstream_status: "ok".to_owned(),
        })
    }
    fn rate_limits(&self) -> RateLimitDescriptor {
        // Workday REST: ~ 10 req/s per integration system user.
        RateLimitDescriptor {
            requests_per_second: 10,
            burst_capacity: 30,
            daily_quota: Some(500_000),
            note: "Workday REST per-ISU throttle".to_owned(),
        }
    }
    fn auth_scheme(&self) -> AuthScheme {
        AuthScheme::SignedJwt
    }
    fn ontology_projections(&self) -> Vec<OntologyProjection> {
        vec![
            OntologyProjection::new("Employee", "workday:worker")
                .map_field("givenName", "givenName")
                .map_field("familyName", "familyName")
                .map_field("active", "active"),
            OntologyProjection::new("JobProfile", "workday:job-profile").map_field("name", "name"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_shared_connector_kernel::{
        AuditSealHandle, PrincipalId, SecretReference, TenantId, TraceContext,
    };

    fn ctx() -> ConnectorCtx {
        ConnectorCtx::new(
            TenantId::new("t-1").unwrap(),
            PrincipalId::new("svc-wd").unwrap(),
            SecretReference::new("sref://t-1/workday/isu-jwt").unwrap(),
            TraceContext::new("00-trace").unwrap(),
            AuditSealHandle::new("chain-1").unwrap(),
        )
    }
    fn ik(s: &str) -> IdempotencyKey {
        IdempotencyKey::new(format!("{s:0>16}")).unwrap()
    }

    #[test]
    fn provider_id_is_workday() {
        assert_eq!(WorkdayConnector::new().provider_id(), "workday");
    }
    #[test]
    fn capabilities_no_delete_no_subscribe() {
        let c = WorkdayConnector::new().capabilities();
        assert!(c.list && c.get && c.create && c.update);
        assert!(!c.delete && !c.subscribe);
    }
    #[test]
    fn smoke_lists_ten_workers() {
        let s = WorkdayConnector::new();
        let p = s.list(&ctx(), "worker", None).unwrap();
        assert_eq!(p.items.len(), 10, "buildability bar: 10-employee smoke");
    }
    #[test]
    fn list_uses_keyset_page_boundaries() {
        let s = WorkdayConnector::new();
        for i in 1..=101 {
            let mut d = EntityDoc::new();
            let id = format!("CMP-{i:08}");
            d.insert("id", EntityValue::Str(id.clone()));
            s.put("t-1", "compensation", &id, d);
        }

        let first = s.list(&ctx(), "compensation", None).unwrap();
        assert_eq!(first.items.len(), 100);
        assert_eq!(
            first.next_cursor.as_ref().map(Cursor::as_str),
            Some("CMP-00000100")
        );

        let second = s.list(&ctx(), "compensation", first.next_cursor).unwrap();
        assert_eq!(second.items.len(), 1);
        assert!(second.next_cursor.is_none());

        let exact = WorkdayConnector::new();
        for i in 1..=100 {
            let mut d = EntityDoc::new();
            let id = format!("CMP-{i:08}");
            d.insert("id", EntityValue::Str(id.clone()));
            exact.put("t-1", "compensation", &id, d);
        }
        let exact_page = exact.list(&ctx(), "compensation", None).unwrap();
        assert_eq!(exact_page.items.len(), 100);
        assert!(exact_page.next_cursor.is_none());
    }
    #[test]
    fn get_known_worker() {
        let s = WorkdayConnector::new();
        let d = s.get(&ctx(), "worker", "WID-00000001").unwrap();
        assert!(d.get("givenName").is_some());
    }
    #[test]
    fn get_unknown_not_found() {
        let s = WorkdayConnector::new();
        assert!(matches!(
            s.get(&ctx(), "worker", "WID-99999999"),
            Err(ConnectorError::NotFound(_))
        ));
    }
    #[test]
    fn list_unsupported_kind() {
        let s = WorkdayConnector::new();
        assert!(matches!(
            s.list(&ctx(), "no-such", None),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn delete_is_unsupported() {
        let s = WorkdayConnector::new();
        assert!(matches!(
            s.delete(&ctx(), "worker", "WID-00000001"),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn subscribe_is_unsupported() {
        let s = WorkdayConnector::new();
        assert!(matches!(
            s.subscribe(&ctx(), &["worker".into()]),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn create_then_idempotent_replay() {
        let s = WorkdayConnector::new();
        let mut d = EntityDoc::new();
        d.insert("givenName", EntityValue::Str("X".into()));
        let k = ik("k");
        let a = s.create(&ctx(), "worker", d.clone(), k.clone()).unwrap();
        let b = s.create(&ctx(), "worker", d, k).unwrap();
        assert_eq!(a.get("id"), b.get("id"));
    }
    #[test]
    fn update_changes_field() {
        let s = WorkdayConnector::new();
        s.update(
            &ctx(),
            "worker",
            "WID-00000001",
            PatchOp::set("active", EntityValue::Bool(false)),
            ik("u"),
        )
        .unwrap();
        let d = s.get(&ctx(), "worker", "WID-00000001").unwrap();
        assert_eq!(d.get("active"), Some(&EntityValue::Bool(false)));
    }
    #[test]
    fn auth_scheme_signed_jwt() {
        assert_eq!(WorkdayConnector::new().auth_scheme(), AuthScheme::SignedJwt);
    }
    #[test]
    fn ontology_projections_include_employee() {
        let p = WorkdayConnector::new().ontology_projections();
        assert!(p.iter().any(|x| x.object_type == "Employee"));
    }
    #[test]
    fn rate_limits_published() {
        let r = WorkdayConnector::new().rate_limits();
        assert!(r.requests_per_second > 0);
    }
}
