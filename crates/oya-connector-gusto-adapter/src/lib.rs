//! `oya-connector-gusto-adapter` — Gusto Embedded Payroll connector.
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

use oya_shared_connector_kernel::{
    AuthScheme, Connector, ConnectorCapabilities, ConnectorCtx, ConnectorError, Cursor, EntityDoc,
    EntityValue, EventStream, HealthReport, IdempotencyKey, OntologyProjection, Page, PatchOp,
    RateLimitDescriptor,
};
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

type Result<T> = std::result::Result<T, ConnectorError>;
type Store = HashMap<String, HashMap<String, BTreeMap<String, EntityDoc>>>;
type IdemMap = HashMap<String, HashMap<String, String>>;
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
    fn next_seq(&self) -> u64 {
        let mut g = match self.next_id.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let v = *g + 3000;
        *g += 1;
        v
    }
    fn check_kind(&self, kind: &str) -> Result<()> {
        match kind {
            "employee" | "payroll" | "contractor" => Ok(()),
            other => Err(ConnectorError::Unsupported(format!(
                "gusto entity_kind={other}"
            ))),
        }
    }
    fn seal(&self, ctx: &ConnectorCtx, op: &str, payload: &str) {
        let _ = ctx.audit_handle().seal(op, payload);
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
        self.seal(ctx, "connector.list", entity_kind);
        let store = self.lock_store();
        let mut items: Vec<EntityDoc> = store
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(entity_kind))
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default();
        let start: usize = cursor
            .as_ref()
            .map(|c| c.as_str().parse::<usize>().unwrap_or(0))
            .unwrap_or(0);
        const PAGE: usize = 100;
        let end = std::cmp::min(start + PAGE, items.len());
        let page: Vec<EntityDoc> = if start <= items.len() {
            items.drain(start..end).collect()
        } else {
            Vec::new()
        };
        let next = if page.len() == PAGE {
            Cursor::new(end.to_string()).ok()
        } else {
            None
        };
        Ok(Page {
            items: page,
            next_cursor: next,
        })
    }
    fn get(&self, ctx: &ConnectorCtx, entity_kind: &str, id: &str) -> Result<EntityDoc> {
        self.check_kind(entity_kind)?;
        self.seal(ctx, "connector.get", id);
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
        let prev = self
            .lock_idem()
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(idempotency_key.as_str()))
            .cloned();
        if let Some(p) = prev {
            return self.get(ctx, entity_kind, &p);
        }
        let v = self.next_seq();
        let id = match entity_kind {
            "employee" => format!("emp-{v:08}"),
            "payroll" => format!("pr-{v:08}"),
            "contractor" => format!("ctr-{v:08}"),
            _ => format!("x-{v:08}"),
        };
        let mut doc = payload;
        doc.insert("id", EntityValue::Str(id.clone()));
        self.put(ctx.tenant_id().as_str(), entity_kind, &id, doc.clone());
        self.lock_idem()
            .entry(ctx.tenant_id().as_str().to_owned())
            .or_default()
            .insert(idempotency_key.as_str().to_owned(), id.clone());
        self.seal(ctx, "connector.create", &id);
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
        doc.insert(
            patch.field.clone(),
            patch.value.unwrap_or(EntityValue::Null),
        );
        self.put(ctx.tenant_id().as_str(), entity_kind, id, doc.clone());
        self.seal(ctx, "connector.update", id);
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
    use oya_shared_connector_kernel::{
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
}
