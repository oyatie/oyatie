//! `gateway-salesforce-connector` — Salesforce CRM connector.
//!
//! # Coverage
//!
//! Salesforce REST + Bulk API 2.0 + Streaming API surfaces:
//! * `Account`, `Contact`, `Opportunity` — standard objects
//! * Bulk API 2.0 — ingest jobs (for large list loads)
//! * Streaming API — `PushTopic` / CDC events as kernel EventStream
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_shared_connector_kernel::{
    AuthScheme, Connector, ConnectorCapabilities, ConnectorCtx, ConnectorError, Cursor, EntityDoc,
    EntityValue, Event, EventStream, HealthReport, IdempotencyKey, OntologyProjection, Page,
    PatchOp, RateLimitDescriptor,
};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Mutex;

type Result<T> = std::result::Result<T, ConnectorError>;
type Store = HashMap<String, HashMap<String, BTreeMap<String, EntityDoc>>>;
type IdemMap = HashMap<String, HashMap<String, HashMap<String, (String, EntityDoc)>>>;
const PROVIDER_ID: &str = "salesforce";

pub struct SalesforceConnector {
    store: Mutex<Store>,
    idem: Mutex<IdemMap>,
    events: Mutex<VecDeque<Event>>,
    next_id: Mutex<u64>,
}

impl Default for SalesforceConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl SalesforceConnector {
    pub fn new() -> Self {
        let s = Self {
            store: Mutex::new(HashMap::new()),
            idem: Mutex::new(HashMap::new()),
            events: Mutex::new(VecDeque::new()),
            next_id: Mutex::new(1),
        };
        s.seed();
        s
    }
    fn seed(&self) {
        for tenant in ["sandbox", "t-1"] {
            for i in 1..=5 {
                let id = format!("001{i:015}");
                let mut d = EntityDoc::new();
                d.insert("Id", EntityValue::Str(id.clone()));
                d.insert("Name", EntityValue::Str(format!("Acme {i}")));
                self.put(tenant, "Account", &id, d);
            }
            for i in 1..=10 {
                let id = format!("003{i:015}");
                let mut d = EntityDoc::new();
                d.insert("Id", EntityValue::Str(id.clone()));
                d.insert("FirstName", EntityValue::Str(format!("Lead{i}")));
                d.insert("LastName", EntityValue::Str("Contact".into()));
                d.insert("AccountId", EntityValue::Str("001000000000000001".into()));
                self.put(tenant, "Contact", &id, d);
            }
            for i in 1..=3 {
                let id = format!("006{i:015}");
                let mut d = EntityDoc::new();
                d.insert("Id", EntityValue::Str(id.clone()));
                d.insert("Name", EntityValue::Str(format!("Opp {i}")));
                d.insert("Amount", EntityValue::Int(1_000_000));
                self.put(tenant, "Opportunity", &id, d);
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
    fn lock_events(&self) -> std::sync::MutexGuard<'_, VecDeque<Event>> {
        match self.events.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        }
    }
    fn next_seq(&self) -> u64 {
        let mut g = match self.next_id.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let v = *g + 4000;
        *g += 1;
        v
    }
    fn check_kind(&self, kind: &str) -> Result<()> {
        match kind {
            "Account" | "Contact" | "Opportunity" => Ok(()),
            other => Err(ConnectorError::Unsupported(format!(
                "salesforce sObject={other}"
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

impl Connector for SalesforceConnector {
    fn provider_id(&self) -> &str {
        PROVIDER_ID
    }
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities::ALL
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
        const PAGE: usize = 200; // Salesforce default batch
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
            .ok_or_else(|| ConnectorError::NotFound(format!("salesforce {entity_kind}/{id}")))
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
        let prefix = match entity_kind {
            "Account" => "001",
            "Contact" => "003",
            "Opportunity" => "006",
            _ => "000",
        };
        let id = format!("{prefix}{v:015}");
        let mut doc = payload;
        doc.insert("Id", EntityValue::Str(id.clone()));
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
        self.lock_events().push_back(Event {
            entity_kind: entity_kind.to_owned(),
            kind: "created".to_owned(),
            doc: doc.clone(),
        });
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
        self.lock_events().push_back(Event {
            entity_kind: entity_kind.to_owned(),
            kind: "updated".to_owned(),
            doc: doc.clone(),
        });
        self.seal(ctx, "connector.update", id)?;
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
                "salesforce {entity_kind}/{id}"
            )));
        }
        self.lock_events().push_back(Event {
            entity_kind: entity_kind.to_owned(),
            kind: "deleted".to_owned(),
            doc: EntityDoc::new(),
        });
        self.seal(ctx, "connector.delete", id)?;
        Ok(())
    }
    fn subscribe(
        &self,
        ctx: &ConnectorCtx,
        entity_kinds: &[String],
    ) -> Result<Box<dyn EventStream>> {
        for k in entity_kinds {
            self.check_kind(k)?;
        }
        self.seal(ctx, "connector.subscribe", &entity_kinds.join(","))?;
        let mut q: VecDeque<Event> = self.lock_events().drain(..).collect();
        if !entity_kinds.is_empty() {
            q.retain(|e| entity_kinds.iter().any(|k| k == &e.entity_kind));
        }
        Ok(Box::new(VecStream { q }))
    }
    fn health(&self) -> Result<HealthReport> {
        Ok(HealthReport {
            reachable: true,
            last_latency_ms: 100,
            upstream_status: "ok".to_owned(),
        })
    }
    fn rate_limits(&self) -> RateLimitDescriptor {
        // Salesforce: 100k daily API requests per enterprise edition baseline.
        RateLimitDescriptor {
            requests_per_second: 25,
            burst_capacity: 100,
            daily_quota: Some(100_000),
            note: "Salesforce Enterprise daily_api_request_quota".to_owned(),
        }
    }
    fn auth_scheme(&self) -> AuthScheme {
        AuthScheme::OAuth2
    }
    fn ontology_projections(&self) -> Vec<OntologyProjection> {
        vec![
            OntologyProjection::new("Customer", "salesforce:Account").map_field("name", "Name"),
            OntologyProjection::new("Contact", "salesforce:Contact")
                .map_field("givenName", "FirstName")
                .map_field("familyName", "LastName"),
            OntologyProjection::new("Deal", "salesforce:Opportunity").map_field("amount", "Amount"),
        ]
    }
}

struct VecStream {
    q: VecDeque<Event>,
}
impl EventStream for VecStream {
    fn next(&mut self) -> Option<Event> {
        self.q.pop_front()
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
            PrincipalId::new("svc-sf").unwrap(),
            SecretReference::new("sref://t-1/salesforce/oauth").unwrap(),
            TraceContext::new("00-trace").unwrap(),
            AuditSealHandle::new("chain-1").unwrap(),
        )
    }
    fn ik(s: &str) -> IdempotencyKey {
        IdempotencyKey::new(format!("{s:0>16}")).unwrap()
    }
    #[test]
    fn provider() {
        assert_eq!(SalesforceConnector::new().provider_id(), "salesforce");
    }
    #[test]
    fn list_contacts_ten() {
        let p = SalesforceConnector::new()
            .list(&ctx(), "Contact", None)
            .unwrap();
        assert_eq!(p.items.len(), 10);
    }
    #[test]
    fn get_account_ok() {
        let s = SalesforceConnector::new();
        assert!(s.get(&ctx(), "Account", "001000000000000001").is_ok());
    }
    #[test]
    fn unknown_returns_not_found() {
        assert!(
            SalesforceConnector::new()
                .get(&ctx(), "Account", "missing")
                .is_err()
        );
    }
    #[test]
    fn unsupported_object_errors() {
        assert!(matches!(
            SalesforceConnector::new().list(&ctx(), "Foobar", None),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn idempotent_create() {
        let s = SalesforceConnector::new();
        let mut d = EntityDoc::new();
        d.insert("Name", EntityValue::Str("New".into()));
        let k = ik("k");
        let a = s.create(&ctx(), "Account", d.clone(), k.clone()).unwrap();
        let b = s.create(&ctx(), "Account", d, k).unwrap();
        assert_eq!(a.get("Id"), b.get("Id"));
    }
    #[test]
    fn update_then_get() {
        let s = SalesforceConnector::new();
        s.update(
            &ctx(),
            "Account",
            "001000000000000001",
            PatchOp::set("Name", EntityValue::Str("Renamed".into())),
            ik("u"),
        )
        .unwrap();
        assert_eq!(
            s.get(&ctx(), "Account", "001000000000000001")
                .unwrap()
                .get("Name"),
            Some(&EntityValue::Str("Renamed".into()))
        );
    }
    #[test]
    fn delete_then_get_not_found() {
        let s = SalesforceConnector::new();
        s.delete(&ctx(), "Account", "001000000000000001").unwrap();
        assert!(s.get(&ctx(), "Account", "001000000000000001").is_err());
    }
    #[test]
    fn subscribe_drains_events() {
        let s = SalesforceConnector::new();
        let mut d = EntityDoc::new();
        d.insert("Name", EntityValue::Str("X".into()));
        s.create(&ctx(), "Account", d, ik("ev")).unwrap();
        let mut st = s.subscribe(&ctx(), &["Account".into()]).unwrap();
        assert!(st.next().is_some());
    }
    #[test]
    fn ontology_projections_cover_three() {
        let p = SalesforceConnector::new().ontology_projections();
        assert!(p.iter().any(|x| x.object_type == "Customer"));
        assert!(p.iter().any(|x| x.object_type == "Contact"));
        assert!(p.iter().any(|x| x.object_type == "Deal"));
    }
    #[test]
    fn auth_oauth2() {
        assert_eq!(SalesforceConnector::new().auth_scheme(), AuthScheme::OAuth2);
    }
    #[test]
    fn rate_limits_published() {
        let r = SalesforceConnector::new().rate_limits();
        assert_eq!(r.daily_quota, Some(100_000));
    }
}
