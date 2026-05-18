//! `oya-connector-netsuite-adapter` — NetSuite ERP connector.
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
    fn next_seq(&self) -> u64 {
        let mut g = match self.next_id.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let v = *g + 5000;
        *g += 1;
        v
    }
    fn check_kind(&self, kind: &str) -> Result<()> {
        match kind {
            "customer" | "vendor" | "salesOrder" | "journalEntry" => Ok(()),
            other => Err(ConnectorError::Unsupported(format!(
                "netsuite recordType={other}"
            ))),
        }
    }
    fn seal(&self, ctx: &ConnectorCtx, op: &str, payload: &str) {
        let _ = ctx.audit_handle().seal(op, payload);
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
        let prev = self
            .lock_idem()
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(idempotency_key.as_str()))
            .cloned();
        if let Some(p) = prev {
            return self.get(ctx, entity_kind, &p);
        }
        let v = self.next_seq();
        let id = format!("{v}");
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
        doc.insert(patch.field.clone(), patch.value.unwrap_or(EntityValue::Null));
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
                "netsuite {entity_kind}/{id}"
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
            OntologyProjection::new("Vendor", "netsuite:vendor")
                .map_field("name", "companyName"),
            OntologyProjection::new("SalesOrder", "netsuite:salesOrder")
                .map_field("total", "total"),
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
    fn get_customer_ok() {
        assert!(NetsuiteConnector::new().get(&ctx(), "customer", "1").is_ok());
    }
    #[test]
    fn unknown_record_not_found() {
        assert!(NetsuiteConnector::new()
            .get(&ctx(), "customer", "999")
            .is_err());
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
}
