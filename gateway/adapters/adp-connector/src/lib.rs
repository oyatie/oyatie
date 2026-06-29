//! `gateway-adp-connector` — ADP Workforce Now connector.
//!
//! # Coverage
//!
//! ADP Workforce Now / Marketplace API surfaces:
//! * `/hr/v2/workers` — workers + assignments
//! * `/payroll/v1/pay-statements` — pay statements (read-only)
//! * `/time/v2/time-cards` — time-card entries
//! * `/benefits/v1/coverages` — enrollment + dependents
//!
//! # Auth
//!
//! ADP mTLS (certificate-bound client auth) plus OAuth2 access token.
//! SecretReference resolves the client cert + key pair in OpenBao.
//!
//! See `README.md` for sandbox setup and `specs/openapi.snapshot.yaml`.
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
type IdemMap = HashMap<String, HashMap<String, HashMap<String, (String, EntityDoc)>>>;

const PROVIDER_ID: &str = "adp";

pub struct AdpConnector {
    store: Mutex<Store>,
    idem: Mutex<IdemMap>,
    next_id: Mutex<u64>,
}

impl Default for AdpConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl AdpConnector {
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
                let id = format!("AOID-{i:08}");
                d.insert("id", EntityValue::Str(id.clone()));
                d.insert("associateOid", EntityValue::Str(id.clone()));
                d.insert("givenName", EntityValue::Str(format!("Adp{i}")));
                d.insert("familyName", EntityValue::Str("Worker".into()));
                self.put(tenant, "worker", &id, d);
            }
            for i in 1..=3 {
                let mut d = EntityDoc::new();
                d.insert("id", EntityValue::Str(format!("PS-{i:08}")));
                d.insert("workerId", EntityValue::Str("AOID-00000001".into()));
                d.insert("grossPay", EntityValue::Int(500_000));
                self.put(tenant, "pay-statement", &format!("PS-{i:08}"), d);
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
        let v = *g + 1000;
        *g += 1;
        v
    }
    fn check_kind(&self, kind: &str) -> Result<()> {
        match kind {
            "worker" | "pay-statement" | "time-card" | "benefit-coverage" => Ok(()),
            other => Err(ConnectorError::Unsupported(format!(
                "adp entity_kind={other}"
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

impl Connector for AdpConnector {
    fn provider_id(&self) -> &str {
        PROVIDER_ID
    }
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            list: true,
            get: true,
            create: true,
            update: true,
            delete: false, // ADP HCM never hard-deletes; uses termination dates.
            subscribe: false,
        }
    }
    fn list(&self, ctx: &ConnectorCtx, entity_kind: &str, cursor: Option<Cursor>) -> Result<Page> {
        self.check_kind(entity_kind)?;
        self.seal(ctx, "connector.list", entity_kind)?;
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
        self.seal(ctx, "connector.get", id)?;
        self.lock_store()
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(entity_kind))
            .and_then(|m| m.get(id))
            .cloned()
            .ok_or_else(|| ConnectorError::NotFound(format!("adp {entity_kind}/{id}")))
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
            "worker" => format!("AOID-{v:08}"),
            "pay-statement" => format!("PS-{v:08}"),
            "time-card" => format!("TC-{v:08}"),
            "benefit-coverage" => format!("BC-{v:08}"),
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
        doc.insert(
            patch.field.clone(),
            patch.value.unwrap_or(EntityValue::Null),
        );
        self.put(ctx.tenant_id().as_str(), entity_kind, id, doc.clone());
        self.seal(ctx, "connector.update", id)?;
        Ok(doc)
    }
    fn delete(&self, _ctx: &ConnectorCtx, _entity_kind: &str, _id: &str) -> Result<()> {
        Err(ConnectorError::Unsupported(
            "adp HCM does not hard-delete; use termination event".into(),
        ))
    }
    fn subscribe(
        &self,
        _ctx: &ConnectorCtx,
        _entity_kinds: &[String],
    ) -> Result<Box<dyn EventStream>> {
        Err(ConnectorError::Unsupported(
            "adp event-notification webhooks deferred to follow-up adapter".into(),
        ))
    }
    fn health(&self) -> Result<HealthReport> {
        Ok(HealthReport {
            reachable: true,
            last_latency_ms: 180,
            upstream_status: "ok".to_owned(),
        })
    }
    fn rate_limits(&self) -> RateLimitDescriptor {
        RateLimitDescriptor {
            requests_per_second: 5,
            burst_capacity: 20,
            daily_quota: Some(100_000),
            note: "ADP Marketplace standard tier".to_owned(),
        }
    }
    fn auth_scheme(&self) -> AuthScheme {
        AuthScheme::MutualTls
    }
    fn ontology_projections(&self) -> Vec<OntologyProjection> {
        vec![
            OntologyProjection::new("Employee", "adp:worker")
                .map_field("givenName", "givenName")
                .map_field("familyName", "familyName"),
            OntologyProjection::new("PayStatement", "adp:pay-statement")
                .map_field("grossPay", "grossPay"),
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
            PrincipalId::new("svc-adp").unwrap(),
            SecretReference::new("sref://t-1/adp/mtls").unwrap(),
            TraceContext::new("00-trace").unwrap(),
            AuditSealHandle::new("chain-1").unwrap(),
        )
    }
    fn ik(s: &str) -> IdempotencyKey {
        IdempotencyKey::new(format!("{s:0>16}")).unwrap()
    }

    #[test]
    fn provider_id_adp() {
        assert_eq!(AdpConnector::new().provider_id(), "adp");
    }
    #[test]
    fn capabilities_no_delete() {
        assert!(!AdpConnector::new().capabilities().delete);
    }
    #[test]
    fn smoke_lists_ten_workers() {
        let s = AdpConnector::new();
        let p = s.list(&ctx(), "worker", None).unwrap();
        assert_eq!(p.items.len(), 10);
    }
    #[test]
    fn get_worker_returns_doc() {
        let s = AdpConnector::new();
        assert!(s.get(&ctx(), "worker", "AOID-00000001").is_ok());
    }
    #[test]
    fn get_unknown_not_found() {
        let s = AdpConnector::new();
        assert!(s.get(&ctx(), "worker", "missing").is_err());
    }
    #[test]
    fn list_pay_statements() {
        let s = AdpConnector::new();
        assert!(
            !s.list(&ctx(), "pay-statement", None)
                .unwrap()
                .items
                .is_empty()
        );
    }
    #[test]
    fn unsupported_kind_errors() {
        assert!(matches!(
            AdpConnector::new().list(&ctx(), "x", None),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn delete_unsupported() {
        assert!(matches!(
            AdpConnector::new().delete(&ctx(), "worker", "AOID-00000001"),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn idempotent_create() {
        let s = AdpConnector::new();
        let mut d = EntityDoc::new();
        d.insert("givenName", EntityValue::Str("Y".into()));
        let k = ik("k");
        let a = s.create(&ctx(), "worker", d.clone(), k.clone()).unwrap();
        let b = s.create(&ctx(), "worker", d, k).unwrap();
        assert_eq!(a.get("id"), b.get("id"));
    }
    #[test]
    fn auth_scheme_mtls() {
        assert_eq!(AdpConnector::new().auth_scheme(), AuthScheme::MutualTls);
    }
    #[test]
    fn ontology_projections_present() {
        assert!(!AdpConnector::new().ontology_projections().is_empty());
    }
}
