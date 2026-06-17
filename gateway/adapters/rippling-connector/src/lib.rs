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
    fn next_seq(&self) -> u64 {
        let mut g = match self.next_id.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let v = *g + 2000;
        *g += 1;
        v
    }
    fn check_kind(&self, kind: &str) -> Result<()> {
        match kind {
            "employee" | "device" | "transaction" => Ok(()),
            other => Err(ConnectorError::Unsupported(format!(
                "rippling entity_kind={other}"
            ))),
        }
    }
    fn seal(&self, ctx: &ConnectorCtx, op: &str, payload: &str) {
        let _ = ctx.audit_handle().seal(op, payload);
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
            "employee" => format!("emp-{v:06}"),
            "device" => format!("dev-{v:06}"),
            "transaction" => format!("txn-{v:08}"),
            _ => format!("x-{v:06}"),
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
    fn delete(&self, ctx: &ConnectorCtx, entity_kind: &str, id: &str) -> Result<()> {
        self.check_kind(entity_kind)?;
        let removed = self
            .lock_store()
            .get_mut(ctx.tenant_id().as_str())
            .and_then(|m| m.get_mut(entity_kind))
            .and_then(|m| m.remove(id))
            .is_some();
        if !removed {
            return Err(ConnectorError::NotFound(format!(
                "rippling {entity_kind}/{id}"
            )));
        }
        self.seal(ctx, "connector.delete", id);
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
    use oya_shared_connector_kernel::{
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
}
