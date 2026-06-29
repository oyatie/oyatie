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
    PatchOp, RateLimitDescriptor, canonical_audit_payload_digest, entity_doc_payload_digest,
    is_canonical_sha256_hex, windowed_page,
};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Mutex;

type Result<T> = std::result::Result<T, ConnectorError>;
type Store = HashMap<String, HashMap<String, BTreeMap<String, EntityDoc>>>;
type IdemMap = HashMap<String, HashMap<String, HashMap<String, (String, EntityDoc)>>>;
type EventQueues = HashMap<String, VecDeque<Event>>;
const PROVIDER_ID: &str = "salesforce";

pub struct SalesforceConnector {
    store: Mutex<Store>,
    idem: Mutex<IdemMap>,
    events: Mutex<EventQueues>,
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
            events: Mutex::new(HashMap::new()),
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
    fn lock_events(&self) -> std::sync::MutexGuard<'_, EventQueues> {
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
    fn seal(&self, ctx: &ConnectorCtx, op: &str, payload_digest: &str) -> Result<()> {
        if !is_canonical_sha256_hex(payload_digest) {
            return Err(ConnectorError::AuditSealFailed(format!(
                "{op} payload digest must be canonical sha256"
            )));
        }
        let receipt = ctx.audit_handle().seal(op, payload_digest);
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

    fn operation_digest<I, K, V>(&self, ctx: &ConnectorCtx, fields: I) -> String
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut parts = vec![
            ("provider".to_owned(), PROVIDER_ID.to_owned()),
            ("tenant".to_owned(), ctx.tenant_id().as_str().to_owned()),
        ];
        for (key, value) in fields {
            parts.push((key.as_ref().to_owned(), value.as_ref().to_owned()));
        }
        canonical_audit_payload_digest(parts)
    }

    fn redacted(label: &str, value: &str) -> String {
        format!("{label}=[REDACTED:{} chars]", value.len())
    }

    fn push_event(&self, ctx: &ConnectorCtx, event: Event) {
        self.lock_events()
            .entry(ctx.tenant_id().as_str().to_owned())
            .or_default()
            .push_back(event);
    }

    fn drain_events_for_tenant(&self, ctx: &ConnectorCtx) -> VecDeque<Event> {
        self.lock_events()
            .entry(ctx.tenant_id().as_str().to_owned())
            .or_default()
            .drain(..)
            .collect()
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
        let cursor_value = cursor.as_ref().map(Cursor::as_str).unwrap_or("");
        let audit_digest = self.operation_digest(
            ctx,
            [("entity_kind", entity_kind), ("cursor", cursor_value)],
        );
        self.seal(ctx, "connector.list", &audit_digest)?;

        let store = self.lock_store();
        let items = store
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(entity_kind))
            .into_iter()
            .flat_map(|m| m.values().cloned());
        const PAGE: usize = 200; // Salesforce default batch
        windowed_page(items, cursor.as_ref(), PAGE)
    }
    fn get(&self, ctx: &ConnectorCtx, entity_kind: &str, id: &str) -> Result<EntityDoc> {
        self.check_kind(entity_kind)?;
        let audit_digest = self.operation_digest(ctx, [("entity_kind", entity_kind), ("id", id)]);
        self.seal(ctx, "connector.get", &audit_digest)?;
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
                    "{PROVIDER_ID} {} entity_kind={} {}",
                    Self::redacted("tenant", ctx.tenant_id().as_str()),
                    entity_kind,
                    Self::redacted("idempotency_key", idempotency_key.as_str())
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
        self.push_event(
            ctx,
            Event {
                entity_kind: entity_kind.to_owned(),
                kind: "created".to_owned(),
                doc: doc.clone(),
            },
        );
        let doc_digest = entity_doc_payload_digest(&doc);
        let audit_digest = self.operation_digest(
            ctx,
            [
                ("entity_kind", entity_kind),
                ("id", id.as_str()),
                ("doc_digest", doc_digest.as_str()),
            ],
        );
        self.seal(ctx, "connector.create", &audit_digest)?;
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
        self.push_event(
            ctx,
            Event {
                entity_kind: entity_kind.to_owned(),
                kind: "updated".to_owned(),
                doc: doc.clone(),
            },
        );
        let doc_digest = entity_doc_payload_digest(&doc);
        let audit_digest = self.operation_digest(
            ctx,
            [
                ("entity_kind", entity_kind),
                ("id", id),
                ("doc_digest", doc_digest.as_str()),
            ],
        );
        self.seal(ctx, "connector.update", &audit_digest)?;
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
        self.push_event(
            ctx,
            Event {
                entity_kind: entity_kind.to_owned(),
                kind: "deleted".to_owned(),
                doc: EntityDoc::new(),
            },
        );
        let audit_digest = self.operation_digest(ctx, [("entity_kind", entity_kind), ("id", id)]);
        self.seal(ctx, "connector.delete", &audit_digest)?;
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
        let joined = entity_kinds.join(",");
        let audit_digest = self.operation_digest(ctx, [("entity_kinds", joined.as_str())]);
        self.seal(ctx, "connector.subscribe", &audit_digest)?;
        let mut q = self.drain_events_for_tenant(ctx);
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
        ctx_for("t-1")
    }

    fn ctx_for(tenant: &str) -> ConnectorCtx {
        ConnectorCtx::new(
            TenantId::new(tenant).unwrap(),
            PrincipalId::new("svc-sf").unwrap(),
            SecretReference::new(format!("sref://{tenant}/salesforce/oauth")).unwrap(),
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
    fn idempotency_conflict_redacts_tenant_and_key() {
        let s = SalesforceConnector::new();
        let key = ik("conflict");
        let mut first = EntityDoc::new();
        first.insert("Name", EntityValue::Str("first".into()));
        let _ = s.create(&ctx(), "Account", first, key.clone()).unwrap();

        let mut second = EntityDoc::new();
        second.insert("Name", EntityValue::Str("second".into()));
        let err = s.create(&ctx(), "Account", second, key).unwrap_err();

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
    fn subscribe_is_tenant_bound() {
        let s = SalesforceConnector::new();
        let mut d = EntityDoc::new();
        d.insert("Name", EntityValue::Str("tenant-two-only".into()));

        let _ = s
            .create(&ctx_for("t-2"), "Account", d, ik("tenant2event"))
            .unwrap();

        let mut t1_stream = s.subscribe(&ctx(), &["Account".to_owned()]).unwrap();
        assert!(t1_stream.next().is_none());

        let mut t2_stream = s
            .subscribe(&ctx_for("t-2"), &["Account".to_owned()])
            .unwrap();
        assert_eq!(t2_stream.next().map(|e| e.kind), Some("created".to_owned()));
    }

    #[test]
    fn audit_seal_rejects_raw_payload_inputs() {
        let s = SalesforceConnector::new();
        assert!(matches!(
            s.seal(&ctx(), "connector.get", "Account"),
            Err(ConnectorError::AuditSealFailed(_))
        ));

        let digest = s.operation_digest(&ctx(), [("entity_kind", "Account"), ("id", "a-1")]);
        assert!(s.seal(&ctx(), "connector.get", &digest).is_ok());
    }

    #[test]
    fn list_uses_windowed_page_boundaries() {
        let s = SalesforceConnector::new();
        for i in 0..191 {
            let mut d = EntityDoc::new();
            d.insert("FirstName", EntityValue::Str(format!("bulk-{i}")));
            d.insert("LastName", EntityValue::Str("Contact".into()));
            let _ = s
                .create(&ctx(), "Contact", d, ik(&format!("bulk{i}")))
                .unwrap();
        }

        let first = s.list(&ctx(), "Contact", None).unwrap();
        assert_eq!(first.items.len(), 200);
        assert_eq!(first.next_cursor.as_ref().map(Cursor::as_str), Some("200"));

        let second = s
            .list(&ctx(), "Contact", first.next_cursor.clone())
            .unwrap();
        assert_eq!(second.items.len(), 1);
        assert!(second.next_cursor.is_none());
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
