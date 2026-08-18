//! `gateway-teams-connector` — Microsoft Teams enterprise connector.
//!
//! # Coverage
//!
//! Microsoft Graph API surface for Teams:
//! * `/teams/{id}/channels`
//! * `/teams/{id}/channels/{id}/messages`
//! * Adaptive cards for action buttons (encoded in the `message` `attachments` field).
//!
//! # Auth
//!
//! OAuth 2.0 (Microsoft identity platform). SecretReference resolves a
//! tenant-scoped service-principal client_secret stored in OpenBao.
//!
//! See `README.md` for sandbox setup and `specs/openapi.snapshot.yaml`
//! for the upstream Graph surface this adapter covers.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_shared_connector_kernel::{
    AuthScheme, Connector, ConnectorCapabilities, ConnectorCtx, ConnectorError, Cursor, EntityDoc,
    EntityValue, Event, EventStream, HealthReport, IdempotencyKey, OntologyProjection, Page,
    PatchOp, RateLimitDescriptor, btree_keyset_page, connector_operation_audit_digest,
    entity_doc_payload_digest, is_canonical_sha256_hex, patch_op_payload_digest,
};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Mutex;

type Result<T> = std::result::Result<T, ConnectorError>;
type Store = HashMap<String, HashMap<String, BTreeMap<String, EntityDoc>>>;
type IdemRecord = (String, String, EntityDoc);
type IdemMap = HashMap<String, HashMap<String, HashMap<String, IdemRecord>>>;
type EventQueues = HashMap<String, VecDeque<Event>>;

const PROVIDER_ID: &str = "teams";

/// Microsoft Teams connector adapter.
pub struct TeamsConnector {
    store: Mutex<Store>,
    idem: Mutex<IdemMap>,
    events: Mutex<EventQueues>,
    next_id: Mutex<u64>,
}

impl Default for TeamsConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl TeamsConnector {
    pub fn new() -> Self {
        let s = Self {
            store: Mutex::new(HashMap::new()),
            idem: Mutex::new(HashMap::new()),
            events: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        };
        s.seed_sandbox();
        s
    }

    fn seed_sandbox(&self) {
        for tenant in ["sandbox", "t-1"] {
            for (cid, name) in [
                ("19:abc1@thread.tacv2", "general"),
                ("19:abc2@thread.tacv2", "random"),
            ] {
                let mut d = EntityDoc::new();
                d.insert("id", EntityValue::Str(cid.to_owned()));
                d.insert("displayName", EntityValue::Str(name.to_owned()));
                self.put(tenant, "channel", cid, d);
            }
            let mut m = EntityDoc::new();
            m.insert("id", EntityValue::Str("1700000001".into()));
            m.insert("channelId", EntityValue::Str("19:abc1@thread.tacv2".into()));
            m.insert("body", EntityValue::Str("seed teams message".into()));
            self.put(tenant, "message", "1700000001", m);
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
    fn next_seq_candidate(&self) -> u64 {
        let g = match self.next_id.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        *g
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
            "channel" | "message" | "adaptive-card" => Ok(()),
            other => Err(ConnectorError::Unsupported(format!(
                "teams entity_kind={other}"
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

    fn push_event(&self, ctx: &ConnectorCtx, event: Event) {
        self.lock_events()
            .entry(ctx.tenant_id().as_str().to_owned())
            .or_default()
            .push_back(event);
    }

    fn matching_events_for_tenant(
        &self,
        ctx: &ConnectorCtx,
        entity_kinds: &[String],
    ) -> VecDeque<Event> {
        let mut queues = self.lock_events();
        let queue = queues
            .entry(ctx.tenant_id().as_str().to_owned())
            .or_default();
        if entity_kinds.is_empty() {
            return std::mem::take(queue);
        }

        let mut matched = VecDeque::new();
        let mut unmatched = VecDeque::new();
        while let Some(event) = queue.pop_front() {
            if entity_kinds.iter().any(|kind| kind == &event.entity_kind) {
                matched.push_back(event);
            } else {
                unmatched.push_back(event);
            }
        }
        *queue = unmatched;
        matched
    }

    fn update_idempotency_scope(entity_kind: &str, id: &str) -> String {
        format!("update:{entity_kind}:{id}")
    }
}

impl Connector for TeamsConnector {
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
            .ok_or_else(|| ConnectorError::NotFound(format!("teams {entity_kind}/{id}")))
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
        if let Some((p, previous_digest, previous_result)) = prev {
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
            "channel" => format!("19:gen{v:04}@thread.tacv2"),
            "message" => format!("1700001{v:04}"),
            "adaptive-card" => format!("card-{v:08}"),
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
                (id.clone(), submitted_digest.clone(), doc.clone()),
            );
        self.push_event(
            ctx,
            Event {
                entity_kind: entity_kind.to_owned(),
                kind: "created".to_owned(),
                doc: doc.clone(),
            },
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
        let idem_scope = Self::update_idempotency_scope(entity_kind, id);
        let prev = self
            .lock_idem()
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(&idem_scope))
            .and_then(|m| m.get(idempotency_key.as_str()))
            .cloned();
        if let Some((prev_id, previous_digest, previous_result)) = prev {
            if previous_digest != patch_digest {
                let conflict_digest = self.operation_digest(
                    ctx,
                    "connector.update",
                    [
                        ("entity_kind", entity_kind),
                        ("id", id),
                        ("idempotency_key", idempotency_key.as_str()),
                        ("outcome", "idempotency_conflict"),
                        ("patch_digest", patch_digest.as_str()),
                        ("stored_patch_digest", previous_digest.as_str()),
                    ],
                );
                self.seal(ctx, "connector.update", &conflict_digest)?;
                return Err(ConnectorError::IdempotencyConflict(format!(
                    "{PROVIDER_ID} {} entity_kind={} {} {}",
                    Self::redacted("tenant", ctx.tenant_id().as_str()),
                    entity_kind,
                    Self::redacted("entity_id", id),
                    Self::redacted("idempotency_key", idempotency_key.as_str())
                )));
            }
            let doc_digest = entity_doc_payload_digest(&previous_result);
            let replay_digest = self.operation_digest(
                ctx,
                "connector.update",
                [
                    ("entity_kind", entity_kind),
                    ("id", prev_id.as_str()),
                    ("idempotency_key", idempotency_key.as_str()),
                    ("outcome", "idempotent_replay"),
                    ("patch_digest", patch_digest.as_str()),
                    ("doc_digest", doc_digest.as_str()),
                ],
            );
            self.seal(ctx, "connector.update", &replay_digest)?;
            return Ok(previous_result);
        }

        let mut doc = self.get(ctx, entity_kind, id)?;
        let v = patch.value.unwrap_or(EntityValue::Null);
        doc.insert(patch.field.clone(), v);
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
        self.lock_idem()
            .entry(ctx.tenant_id().as_str().to_owned())
            .or_default()
            .entry(idem_scope)
            .or_default()
            .insert(
                idempotency_key.as_str().to_owned(),
                (id.to_owned(), patch_digest.clone(), doc.clone()),
            );
        self.push_event(
            ctx,
            Event {
                entity_kind: entity_kind.to_owned(),
                kind: "updated".to_owned(),
                doc: doc.clone(),
            },
        );
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
        self.push_event(
            ctx,
            Event {
                entity_kind: entity_kind.to_owned(),
                kind: "deleted".to_owned(),
                doc: EntityDoc::new(),
            },
        );
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
        let audit_digest = self.operation_digest(
            ctx,
            "connector.subscribe",
            [("entity_kinds", joined.as_str())],
        );
        self.seal(ctx, "connector.subscribe", &audit_digest)?;
        let q = self.matching_events_for_tenant(ctx, entity_kinds);
        Ok(Box::new(VecStream { q }))
    }

    fn health(&self) -> Result<HealthReport> {
        Ok(HealthReport {
            reachable: true,
            last_latency_ms: 55,
            upstream_status: "ok".to_owned(),
        })
    }

    fn rate_limits(&self) -> RateLimitDescriptor {
        // Graph API: 10k requests per 10 minutes per app per tenant.
        RateLimitDescriptor {
            requests_per_second: 16,
            burst_capacity: 100,
            daily_quota: Some(1_000_000),
            note: "Graph API: 10k req / 10min / app / tenant".to_owned(),
        }
    }

    fn auth_scheme(&self) -> AuthScheme {
        AuthScheme::OAuth2
    }

    fn ontology_projections(&self) -> Vec<OntologyProjection> {
        vec![
            OntologyProjection::new("Channel", "teams:channel")
                .map_field("id", "id")
                .map_field("displayName", "displayName"),
            OntologyProjection::new("Message", "teams:message")
                .map_field("id", "id")
                .map_field("channelId", "channelId")
                .map_field("body", "body"),
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
            PrincipalId::new("svc-teams").unwrap(),
            SecretReference::new(format!("sref://{tenant}/teams/client-secret")).unwrap(),
            TraceContext::new("00-trace").unwrap(),
            AuditSealHandle::new("chain-1").unwrap(),
        )
    }
    fn ik(s: &str) -> IdempotencyKey {
        IdempotencyKey::new(format!("{s:0>16}")).unwrap()
    }

    #[test]
    fn provider_id_is_teams() {
        assert_eq!(TeamsConnector::new().provider_id(), "teams");
    }
    #[test]
    fn capabilities_full() {
        let c = TeamsConnector::new().capabilities();
        assert!(c.list && c.get && c.create && c.update && c.delete && c.subscribe);
    }
    #[test]
    fn list_channels_returns_seeded() {
        let s = TeamsConnector::new();
        assert!(!s.list(&ctx(), "channel", None).unwrap().items.is_empty());
    }
    #[test]
    fn list_unsupported_errors() {
        let s = TeamsConnector::new();
        assert!(matches!(
            s.list(&ctx(), "no-such", None),
            Err(ConnectorError::Unsupported(_))
        ));
    }
    #[test]
    fn get_seeded_returns_doc() {
        let s = TeamsConnector::new();
        assert!(s.get(&ctx(), "message", "1700000001").is_ok());
    }
    #[test]
    fn get_unknown_returns_not_found() {
        let s = TeamsConnector::new();
        assert!(matches!(
            s.get(&ctx(), "message", "missing"),
            Err(ConnectorError::NotFound(_))
        ));
    }
    #[test]
    fn create_message_idempotent() {
        let s = TeamsConnector::new();
        let mut d = EntityDoc::new();
        d.insert("body", EntityValue::Str("hi".into()));
        let k = ik("k1");
        let a = s.create(&ctx(), "message", d.clone(), k.clone()).unwrap();
        let b = s.create(&ctx(), "message", d, k).unwrap();
        assert_eq!(a.get("id"), b.get("id"));
    }

    #[test]
    fn idempotency_conflict_redacts_tenant_and_key() {
        let s = TeamsConnector::new();
        let key = ik("conflict");
        let mut first = EntityDoc::new();
        first.insert("body", EntityValue::Str("first".into()));
        let _ = s.create(&ctx(), "message", first, key.clone()).unwrap();

        let mut second = EntityDoc::new();
        second.insert("body", EntityValue::Str("second".into()));
        let err = s.create(&ctx(), "message", second, key).unwrap_err();

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
    fn update_then_read() {
        let s = TeamsConnector::new();
        s.update(
            &ctx(),
            "message",
            "1700000001",
            PatchOp::set("body", EntityValue::Str("edited".into())),
            ik("u1"),
        )
        .unwrap();
        let d = s.get(&ctx(), "message", "1700000001").unwrap();
        assert_eq!(d.get("body"), Some(&EntityValue::Str("edited".into())));
    }

    #[test]
    fn update_idempotency_replays_prior_result_and_conflicts_on_mismatch() {
        let s = TeamsConnector::new();
        let replay_key = ik("updreplay");
        let replay_patch = PatchOp::set("body", EntityValue::Str("first-idempotent".into()));

        let first = s
            .update(
                &ctx(),
                "message",
                "1700000001",
                replay_patch.clone(),
                replay_key.clone(),
            )
            .unwrap();
        let _ = s
            .update(
                &ctx(),
                "message",
                "1700000001",
                PatchOp::set("body", EntityValue::Str("later-change".into())),
                ik("updother"),
            )
            .unwrap();
        let replay = s
            .update(&ctx(), "message", "1700000001", replay_patch, replay_key)
            .unwrap();

        assert_eq!(first, replay);
        assert_eq!(
            replay.get("body"),
            Some(&EntityValue::Str("first-idempotent".into()))
        );
        assert_eq!(
            s.get(&ctx(), "message", "1700000001").unwrap().get("body"),
            Some(&EntityValue::Str("later-change".into()))
        );

        let conflict_key = ik("updconflict");
        let _ = s
            .update(
                &ctx(),
                "message",
                "1700000001",
                PatchOp::set("body", EntityValue::Str("conflict-a".into())),
                conflict_key.clone(),
            )
            .unwrap();
        let err = s
            .update(
                &ctx(),
                "message",
                "1700000001",
                PatchOp::set("body", EntityValue::Str("conflict-b".into())),
                conflict_key,
            )
            .unwrap_err();

        match err {
            ConnectorError::IdempotencyConflict(message) => {
                assert!(message.contains("[REDACTED"));
                assert!(!message.contains("t-1"));
                assert!(!message.contains("1700000001"));
                assert!(!message.contains("updconflict"));
            }
            other => panic!("expected idempotency conflict, got {other:?}"),
        }
    }
    #[test]
    fn delete_then_get_not_found() {
        let s = TeamsConnector::new();
        s.delete(&ctx(), "channel", "19:abc1@thread.tacv2").unwrap();
        assert!(s.get(&ctx(), "channel", "19:abc1@thread.tacv2").is_err());
    }
    #[test]
    fn subscribe_drains_events() {
        let s = TeamsConnector::new();
        let mut d = EntityDoc::new();
        d.insert("body", EntityValue::Str("x".into()));
        s.create(&ctx(), "message", d, ik("sub1")).unwrap();
        let mut st = s.subscribe(&ctx(), &["message".into()]).unwrap();
        assert!(st.next().is_some());
    }

    #[test]
    fn subscribe_preserves_unmatched_entity_kind_events() {
        let s = TeamsConnector::new();

        let mut message = EntityDoc::new();
        message.insert("body", EntityValue::Str("mixed-message".into()));
        let _ = s
            .create(&ctx(), "message", message, ik("mixmessage"))
            .unwrap();

        let mut channel = EntityDoc::new();
        channel.insert("displayName", EntityValue::Str("mixed-channel".into()));
        let _ = s
            .create(&ctx(), "channel", channel, ik("mixchannel"))
            .unwrap();

        let mut messages = s.subscribe(&ctx(), &["message".to_owned()]).unwrap();
        assert_eq!(
            messages.next().map(|event| event.entity_kind),
            Some("message".to_owned())
        );
        assert!(messages.next().is_none());

        let mut channels = s.subscribe(&ctx(), &["channel".to_owned()]).unwrap();
        assert_eq!(
            channels.next().map(|event| event.entity_kind),
            Some("channel".to_owned())
        );
    }

    #[test]
    fn subscribe_is_tenant_bound() {
        let s = TeamsConnector::new();
        let mut d = EntityDoc::new();
        d.insert("body", EntityValue::Str("tenant-two-only".into()));

        let _ = s
            .create(&ctx_for("t-2"), "message", d, ik("tenant2event"))
            .unwrap();

        let mut t1_stream = s.subscribe(&ctx(), &["message".to_owned()]).unwrap();
        assert!(t1_stream.next().is_none());

        let mut t2_stream = s
            .subscribe(&ctx_for("t-2"), &["message".to_owned()])
            .unwrap();
        assert_eq!(t2_stream.next().map(|e| e.kind), Some("created".to_owned()));
    }

    #[test]
    fn audit_seal_rejects_raw_payload_inputs() {
        let s = TeamsConnector::new();
        assert!(matches!(
            s.seal(&ctx(), "connector.get", "message"),
            Err(ConnectorError::AuditSealFailed(_))
        ));

        let digest = s.operation_digest(
            &ctx(),
            "connector.get",
            [("entity_kind", "message"), ("id", "m-1")],
        );
        assert!(s.seal(&ctx(), "connector.get", &digest).is_ok());
    }

    #[test]
    fn list_uses_keyset_page_boundaries() {
        let s = TeamsConnector::new();
        for i in 1..=101 {
            let mut d = EntityDoc::new();
            let id = format!("card-{i:08}");
            d.insert("id", EntityValue::Str(id.clone()));
            s.put("t-1", "adaptive-card", &id, d);
        }

        let first = s.list(&ctx(), "adaptive-card", None).unwrap();
        assert_eq!(first.items.len(), 100);
        assert_eq!(
            first.next_cursor.as_ref().map(Cursor::as_str),
            Some("card-00000100")
        );

        let second = s.list(&ctx(), "adaptive-card", first.next_cursor).unwrap();
        assert_eq!(second.items.len(), 1);
        assert!(second.next_cursor.is_none());

        let exact = TeamsConnector::new();
        for i in 1..=100 {
            let mut d = EntityDoc::new();
            let id = format!("card-{i:08}");
            d.insert("id", EntityValue::Str(id.clone()));
            exact.put("t-1", "adaptive-card", &id, d);
        }
        let exact_page = exact.list(&ctx(), "adaptive-card", None).unwrap();
        assert_eq!(exact_page.items.len(), 100);
        assert!(exact_page.next_cursor.is_none());
    }
    #[test]
    fn auth_scheme_oauth2() {
        assert_eq!(TeamsConnector::new().auth_scheme(), AuthScheme::OAuth2);
    }
    #[test]
    fn ontology_projections_present() {
        assert!(TeamsConnector::new().ontology_projections().len() >= 2);
    }
    #[test]
    fn rate_limits_reasonable() {
        let r = TeamsConnector::new().rate_limits();
        assert!(r.daily_quota.is_some());
    }
}
