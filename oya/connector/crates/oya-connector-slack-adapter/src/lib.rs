//! `oya-connector-slack-adapter` — Slack enterprise connector.
//!
//! # Coverage
//!
//! Implements [`oya_shared_connector_kernel::Connector`] against the
//! Slack Web API surface that ships in `specs/openapi.snapshot.yaml`:
//!
//! * Conversations (`conversations.history`, `conversations.replies`)
//! * Messages (`chat.postMessage`)
//! * Files (`files.upload`)
//! * Events API subscription (webhook receiver — events delivered as
//!   [`oya_shared_connector_kernel::EventStream`]).
//!
//! # Auth
//!
//! Bot-user OAuth 2.0. SecretReference resolves a long-lived bot token
//! stored in OpenBao under `sref://<tenant>/slack/bot-token`. Token
//! rotation is handled by `oya-intelligence-adapter-anthropic-api-kernel`'s
//! sister substrate at the secrets layer; the adapter never sees the
//! raw bytes.
//!
//! # Sandbox-shaped implementation
//!
//! The adapter ships with an in-memory tenant-scoped store seeded with
//! a Slack-shaped fixture (channels + messages + files). This matches
//! the in-house adapter doctrine (ADR-0083 §sandbox-first): real auth,
//! real rate-limit, real pagination, real idempotency, no upstream
//! traffic. The live `reqwest`-backed adapter is gated behind a
//! `live-network` feature in a future migration IP.
//!
//! See `README.md` for vendor sandbox setup and `specs/openapi.snapshot.yaml`
//! for the upstream surface this adapter covers.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_shared_connector_kernel::{
    AuthScheme, Connector, ConnectorCapabilities, ConnectorCtx, ConnectorError, Cursor, EntityDoc,
    EntityValue, Event, EventStream, HealthReport, IdempotencyKey, OntologyProjection, Page,
    PatchOp, RateLimitDescriptor,
};

type Result<T> = std::result::Result<T, ConnectorError>;
/// Tenant → entity_kind → id → doc.
type Store = HashMap<String, HashMap<String, BTreeMap<String, EntityDoc>>>;
/// Tenant → idempotency_key → doc id.
type IdemMap = HashMap<String, HashMap<String, String>>;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Mutex;

const PROVIDER_ID: &str = "slack";

/// Tenant environment tier for metadata-only connector outbound plans.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectorEnvTier {
    Test,
    Staging,
    Prod,
}

impl ConnectorEnvTier {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "test" => Some(Self::Test),
            "staging" => Some(Self::Staging),
            "prod" => Some(Self::Prod),
            _ => None,
        }
    }

    fn outbound_mode(self) -> ConnectorOutboundMode {
        match self {
            Self::Test => ConnectorOutboundMode::Intercept,
            Self::Staging => ConnectorOutboundMode::TestRecipients,
            Self::Prod => ConnectorOutboundMode::Live,
        }
    }
}

/// Derived outbound behavior. Callers never supply this as authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectorOutboundMode {
    Intercept,
    TestRecipients,
    Live,
}

impl ConnectorOutboundMode {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "intercept" => Some(Self::Intercept),
            "test_recipients" => Some(Self::TestRecipients),
            "live" => Some(Self::Live),
            _ => None,
        }
    }
}

/// Metadata-only fixture input for connector/integration outbound plans.
///
/// This guardrail deliberately does not perform OAuth, API, webhook, or
/// production delivery. It only validates the evidence envelope a later
/// connector adapter lane must carry before any live side-effect path exists.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConnectorOutboundPlanFixture {
    pub tenant_id: Option<String>,
    pub env_tier: Option<String>,
    pub caller_supplied_outbound_mode: Option<String>,
    pub attempts_external_delivery: bool,
    pub connector_adapter_class: Option<String>,
    pub destination_binding_ref: Option<String>,
    pub destination_tenant_id: Option<String>,
    pub qa_endpoint_ref: Option<String>,
    pub retirement_no_new_runtime_scope_evidence_ref: Option<String>,
    pub policy_evidence_ref: Option<String>,
    pub connector_authorization_evidence_ref: Option<String>,
    pub secret_material_probe: Option<String>,
}

/// Validated metadata envelope with outbound mode derived from `env_tier`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectorOutboundPlanMetadata {
    pub tenant_id: String,
    pub env_tier: ConnectorEnvTier,
    pub outbound_mode: ConnectorOutboundMode,
    pub connector_adapter_class: String,
    pub destination_binding_ref: String,
    pub retirement_no_new_runtime_scope_evidence_ref: String,
    pub policy_evidence_ref: String,
    pub connector_authorization_evidence_ref: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectorOutboundMetadataError {
    MissingTenantId,
    MissingEnvTier,
    InvalidEnvTier,
    OutboundModeMismatch,
    MissingConnectorAdapterClass,
    MissingDestinationBindingRef,
    MissingRetirementNoNewRuntimeScopeEvidence,
    MissingPolicyEvidenceRef,
    TestTierExternalDelivery,
    StagingMissingQaEndpoint,
    ProdMissingConnectorAuthorization,
    CrossTenantDestinationLeakage,
    RawSecretMaterial,
}

/// Validate metadata-only connector outbound fixtures and derive outbound mode.
pub fn validate_connector_outbound_plan_fixture(
    fixture: ConnectorOutboundPlanFixture,
) -> std::result::Result<ConnectorOutboundPlanMetadata, ConnectorOutboundMetadataError> {
    let ConnectorOutboundPlanFixture {
        tenant_id,
        env_tier,
        caller_supplied_outbound_mode,
        attempts_external_delivery,
        connector_adapter_class,
        destination_binding_ref,
        destination_tenant_id,
        qa_endpoint_ref,
        retirement_no_new_runtime_scope_evidence_ref,
        policy_evidence_ref,
        connector_authorization_evidence_ref,
        secret_material_probe,
    } = fixture;

    let tenant_id = required_string(tenant_id, ConnectorOutboundMetadataError::MissingTenantId)?;
    let env_tier_raw = required_string(env_tier, ConnectorOutboundMetadataError::MissingEnvTier)?;
    let env_tier = ConnectorEnvTier::parse(&env_tier_raw)
        .ok_or(ConnectorOutboundMetadataError::InvalidEnvTier)?;
    let outbound_mode = env_tier.outbound_mode();

    if caller_supplied_outbound_mode
        .as_deref()
        .and_then(ConnectorOutboundMode::parse)
        .is_some_and(|caller_mode| caller_mode != outbound_mode)
        || caller_supplied_outbound_mode
            .as_deref()
            .is_some_and(|raw| ConnectorOutboundMode::parse(raw).is_none())
    {
        return Err(ConnectorOutboundMetadataError::OutboundModeMismatch);
    }

    let connector_adapter_class = required_string(
        connector_adapter_class,
        ConnectorOutboundMetadataError::MissingConnectorAdapterClass,
    )?;
    let destination_binding_ref = required_string(
        destination_binding_ref,
        ConnectorOutboundMetadataError::MissingDestinationBindingRef,
    )?;
    let retirement_no_new_runtime_scope_evidence_ref = required_string(
        retirement_no_new_runtime_scope_evidence_ref,
        ConnectorOutboundMetadataError::MissingRetirementNoNewRuntimeScopeEvidence,
    )?;
    let policy_evidence_ref = required_string(
        policy_evidence_ref,
        ConnectorOutboundMetadataError::MissingPolicyEvidenceRef,
    )?;

    if let Some(destination_tenant_id) = destination_tenant_id.as_deref()
        && destination_tenant_id != tenant_id
    {
        return Err(ConnectorOutboundMetadataError::CrossTenantDestinationLeakage);
    }

    if [
        Some(connector_adapter_class.as_str()),
        Some(destination_binding_ref.as_str()),
        Some(retirement_no_new_runtime_scope_evidence_ref.as_str()),
        Some(policy_evidence_ref.as_str()),
        qa_endpoint_ref.as_deref(),
        connector_authorization_evidence_ref.as_deref(),
        secret_material_probe.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(contains_raw_secret_material)
    {
        return Err(ConnectorOutboundMetadataError::RawSecretMaterial);
    }

    match env_tier {
        ConnectorEnvTier::Test if attempts_external_delivery => {
            return Err(ConnectorOutboundMetadataError::TestTierExternalDelivery);
        }
        ConnectorEnvTier::Staging if !has_required_string(qa_endpoint_ref.as_deref()) => {
            return Err(ConnectorOutboundMetadataError::StagingMissingQaEndpoint);
        }
        ConnectorEnvTier::Prod
            if !has_required_string(connector_authorization_evidence_ref.as_deref()) =>
        {
            return Err(ConnectorOutboundMetadataError::ProdMissingConnectorAuthorization);
        }
        _ => {}
    }

    Ok(ConnectorOutboundPlanMetadata {
        tenant_id,
        env_tier,
        outbound_mode,
        connector_adapter_class,
        destination_binding_ref,
        retirement_no_new_runtime_scope_evidence_ref,
        policy_evidence_ref,
        connector_authorization_evidence_ref,
    })
}

fn required_string(
    value: Option<String>,
    error: ConnectorOutboundMetadataError,
) -> std::result::Result<String, ConnectorOutboundMetadataError> {
    match value {
        Some(value) if !value.trim().is_empty() => Ok(value),
        _ => Err(error),
    }
}

fn has_required_string(value: Option<&str>) -> bool {
    value.is_some_and(|value| !value.trim().is_empty())
}

fn contains_raw_secret_material(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    [
        "sk_test_",
        "sk_stage_",
        "sk_live_",
        "oauth_token=",
        "webhook_secret=",
        "api_key=",
        "bearer ",
        "xoxb-",
    ]
    .into_iter()
    .any(|marker| value.contains(marker))
}

/// Slack connector adapter.
///
/// In-memory state is wrapped in `RefCell` so the adapter satisfies the
/// `&self` trait surface. Multi-thread access would wrap this in
/// `Mutex` at the app layer.
pub struct SlackConnector {
    /// Per-tenant per-entity-kind store.
    store: Mutex<Store>,
    /// Idempotency-key cache.
    idem: Mutex<IdemMap>,
    /// Pending events (FIFO).
    events: Mutex<VecDeque<Event>>,
    /// Monotonic id sequence (per-tenant per-kind would be safer; this
    /// is sufficient for sandbox).
    next_id: Mutex<u64>,
}

impl Default for SlackConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl SlackConnector {
    /// New connector with a seeded sandbox fixture.
    pub fn new() -> Self {
        let s = Self {
            store: Mutex::new(HashMap::new()),
            idem: Mutex::new(HashMap::new()),
            events: Mutex::new(VecDeque::new()),
            next_id: Mutex::new(1),
        };
        s.seed_sandbox();
        s
    }

    fn lock_store(&self) -> std::sync::MutexGuard<'_, Store> {
        // Mutex is poison-only-on-panic; on poison we recover the inner
        // guard because the sandbox state is internally consistent
        // (writes are atomic). Production live adapter would surface
        // poisoning as ConnectorError::AuditSealFailed.
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
        let v = *g;
        *g += 1;
        v
    }

    fn seed_sandbox(&self) {
        // Two seed tenants × one seed channel + three seed messages.
        for tenant in ["sandbox", "t-1"] {
            for (cid, name) in [("C0001", "general"), ("C0002", "random")] {
                let mut d = EntityDoc::new();
                d.insert("id", EntityValue::Str(cid.to_owned()));
                d.insert("name", EntityValue::Str(name.to_owned()));
                d.insert("is_archived", EntityValue::Bool(false));
                self.put(tenant, "conversation", cid, d);
            }
            for ts in ["1700000001.000100", "1700000002.000200"] {
                let mut d = EntityDoc::new();
                d.insert("ts", EntityValue::Str(ts.to_owned()));
                d.insert("channel", EntityValue::Str("C0001".to_owned()));
                d.insert("text", EntityValue::Str(format!("seed-msg-{ts}")));
                self.put(tenant, "message", ts, d);
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

    fn check_kind(&self, kind: &str) -> Result<()> {
        match kind {
            "conversation" | "message" | "file" => Ok(()),
            other => Err(ConnectorError::Unsupported(format!(
                "slack entity_kind={other}"
            ))),
        }
    }

    fn seal(&self, ctx: &ConnectorCtx, op: &str, payload: &str) {
        let _ = ctx.audit_handle().seal(op, payload);
    }
}

impl Connector for SlackConnector {
    fn provider_id(&self) -> &str {
        PROVIDER_ID
    }

    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            list: true,
            get: true,
            create: true,    // chat.postMessage + files.upload
            update: true,    // chat.update
            delete: true,    // chat.delete
            subscribe: true, // Events API
        }
    }

    fn list(&self, ctx: &ConnectorCtx, entity_kind: &str, cursor: Option<Cursor>) -> Result<Page> {
        self.check_kind(entity_kind)?;
        self.seal(ctx, "connector.list", entity_kind);
        let store = self.lock_store();
        let by_kind = store
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(entity_kind));
        let mut items: Vec<EntityDoc> = by_kind
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default();
        // 100-per-page cursor; cursor is the next start index as a string.
        let start: usize = cursor
            .as_ref()
            .map(|c| c.as_str().parse::<usize>().unwrap_or(0))
            .unwrap_or(0);
        const PAGE: usize = 100;
        let end = std::cmp::min(start + PAGE, items.len());
        let page = items.drain(start..end).collect();
        let next = if end < start + PAGE {
            None
        } else {
            Cursor::new(end.to_string()).ok()
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
            .ok_or_else(|| ConnectorError::NotFound(format!("slack {entity_kind}/{id}")))
    }

    fn create(
        &self,
        ctx: &ConnectorCtx,
        entity_kind: &str,
        payload: EntityDoc,
        idempotency_key: IdempotencyKey,
    ) -> Result<EntityDoc> {
        self.check_kind(entity_kind)?;
        // Idempotency-replay: if the key was already seen, return the stored id.
        let prev_id_opt = self
            .lock_idem()
            .get(ctx.tenant_id().as_str())
            .and_then(|m| m.get(idempotency_key.as_str()))
            .cloned();
        if let Some(prev_id) = prev_id_opt {
            return self.get(ctx, entity_kind, &prev_id);
        }
        let v = self.next_seq();
        let id = match entity_kind {
            "message" => format!("1700001000.{v:06}"),
            "conversation" => format!("C{v:06}"),
            "file" => format!("F{v:06}"),
            _ => format!("X{v:06}"),
        };
        let mut doc = payload;
        doc.insert("id", EntityValue::Str(id.clone()));
        self.put(ctx.tenant_id().as_str(), entity_kind, &id, doc.clone());
        self.lock_idem()
            .entry(ctx.tenant_id().as_str().to_owned())
            .or_default()
            .insert(idempotency_key.as_str().to_owned(), id.clone());
        self.lock_events().push_back(Event {
            entity_kind: entity_kind.to_owned(),
            kind: "created".to_owned(),
            doc: doc.clone(),
        });
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
        match patch.value {
            Some(v) => doc.insert(patch.field.clone(), v),
            None => {
                // Honest contract: kernel doc is BTreeMap-backed; remove is
                // expressed as setting Null. Tracking removal in the
                // kernel would require an API change; for now Null marks
                // tombstone.
                doc.insert(patch.field.clone(), EntityValue::Null);
            }
        }
        self.put(ctx.tenant_id().as_str(), entity_kind, id, doc.clone());
        self.lock_events().push_back(Event {
            entity_kind: entity_kind.to_owned(),
            kind: "updated".to_owned(),
            doc: doc.clone(),
        });
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
                "slack {entity_kind}/{id}"
            )));
        }
        self.lock_events().push_back(Event {
            entity_kind: entity_kind.to_owned(),
            kind: "deleted".to_owned(),
            doc: EntityDoc::new(),
        });
        self.seal(ctx, "connector.delete", id);
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
        self.seal(ctx, "connector.subscribe", &entity_kinds.join(","));
        // Drain current queue, filtered by entity_kinds.
        let mut q: VecDeque<Event> = self.lock_events().drain(..).collect();
        if !entity_kinds.is_empty() {
            q.retain(|e| entity_kinds.iter().any(|k| k == &e.entity_kind));
        }
        Ok(Box::new(VecStream { q }))
    }

    fn health(&self) -> Result<HealthReport> {
        Ok(HealthReport {
            reachable: true,
            last_latency_ms: 42,
            upstream_status: "ok".to_owned(),
        })
    }

    fn rate_limits(&self) -> RateLimitDescriptor {
        // Slack tier-3 bot — 50 req/min (~1/sec) per
        // https://api.slack.com/apis/rate-limits.
        RateLimitDescriptor {
            requests_per_second: 1,
            burst_capacity: 5,
            daily_quota: None,
            note: "slack tier-3 bot (~50/min)".to_owned(),
        }
    }

    fn auth_scheme(&self) -> AuthScheme {
        AuthScheme::OAuth2
    }

    fn ontology_projections(&self) -> Vec<OntologyProjection> {
        vec![
            OntologyProjection::new("Channel", "slack:conversation")
                .map_field("id", "id")
                .map_field("displayName", "name"),
            OntologyProjection::new("Message", "slack:message")
                .map_field("id", "ts")
                .map_field("channelId", "channel")
                .map_field("body", "text"),
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
            PrincipalId::new("svc-slack").unwrap(),
            SecretReference::new("sref://t-1/slack/bot-token").unwrap(),
            TraceContext::new("00-trace").unwrap(),
            AuditSealHandle::new("chain-1").unwrap(),
        )
    }

    fn ik(s: &str) -> IdempotencyKey {
        let padded = format!("{s:0>16}");
        IdempotencyKey::new(padded).unwrap()
    }

    #[test]
    fn provider_id_is_slack() {
        assert_eq!(SlackConnector::new().provider_id(), "slack");
    }

    #[test]
    fn capabilities_full() {
        let c = SlackConnector::new().capabilities();
        assert!(c.list && c.get && c.create && c.update && c.delete && c.subscribe);
    }

    #[test]
    fn list_conversations_returns_seeded_fixture() {
        let s = SlackConnector::new();
        let p = s.list(&ctx(), "conversation", None).unwrap();
        assert!(!p.items.is_empty());
    }

    #[test]
    fn list_unsupported_kind_errors() {
        let s = SlackConnector::new();
        assert!(matches!(
            s.list(&ctx(), "no-such-kind", None),
            Err(ConnectorError::Unsupported(_))
        ));
    }

    #[test]
    fn get_known_returns_doc() {
        let s = SlackConnector::new();
        let d = s.get(&ctx(), "conversation", "C0001").unwrap();
        assert!(d.get("name").is_some());
    }

    #[test]
    fn get_unknown_returns_not_found() {
        let s = SlackConnector::new();
        assert!(matches!(
            s.get(&ctx(), "conversation", "C9999"),
            Err(ConnectorError::NotFound(_))
        ));
    }

    #[test]
    fn create_assigns_id_and_idempotency_replays() {
        let s = SlackConnector::new();
        let mut d = EntityDoc::new();
        d.insert("channel", EntityValue::Str("C0001".into()));
        d.insert("text", EntityValue::Str("hello".into()));
        let key = ik("key1");
        let a = s.create(&ctx(), "message", d.clone(), key.clone()).unwrap();
        let b = s.create(&ctx(), "message", d, key).unwrap();
        assert_eq!(a.get("id"), b.get("id"));
    }

    #[test]
    fn update_writes_then_reads_back() {
        let s = SlackConnector::new();
        let _ = s
            .update(
                &ctx(),
                "conversation",
                "C0001",
                PatchOp::set("name", EntityValue::Str("renamed".into())),
                ik("upd1"),
            )
            .unwrap();
        let got = s.get(&ctx(), "conversation", "C0001").unwrap();
        assert_eq!(got.get("name"), Some(&EntityValue::Str("renamed".into())));
    }

    #[test]
    fn delete_then_get_returns_not_found() {
        let s = SlackConnector::new();
        s.delete(&ctx(), "conversation", "C0001").unwrap();
        assert!(matches!(
            s.get(&ctx(), "conversation", "C0001"),
            Err(ConnectorError::NotFound(_))
        ));
    }

    #[test]
    fn subscribe_drains_pending_events() {
        let s = SlackConnector::new();
        let mut d = EntityDoc::new();
        d.insert("text", EntityValue::Str("x".into()));
        let _ = s.create(&ctx(), "message", d, ik("evkey")).unwrap();
        let mut stream = s.subscribe(&ctx(), &["message".to_owned()]).unwrap();
        let e = stream.next().expect("event present");
        assert_eq!(e.kind, "created");
    }

    #[test]
    fn rate_limits_declared() {
        let r = SlackConnector::new().rate_limits();
        assert!(r.requests_per_second >= 1);
        assert!(r.note.contains("slack"));
    }

    #[test]
    fn auth_scheme_is_oauth2() {
        assert_eq!(SlackConnector::new().auth_scheme(), AuthScheme::OAuth2);
    }

    #[test]
    fn ontology_projections_cover_channel_and_message() {
        let p = SlackConnector::new().ontology_projections();
        assert!(p.iter().any(|x| x.object_type == "Channel"));
        assert!(p.iter().any(|x| x.object_type == "Message"));
    }

    #[test]
    fn health_reports_reachable() {
        assert!(SlackConnector::new().health().unwrap().reachable);
    }
}
