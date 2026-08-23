//! `shared-connector-kernel` — the enterprise integration substrate.
//!
//! # Purpose
//!
//! Every external SaaS integration (Workday, Salesforce, Slack, Epic, …)
//! sits behind the [`Connector`] trait defined in this crate. Adapters
//! live in sibling crates named `connector-<vendor>-adapter` and
//! implement [`Connector`] against their specific provider.
//!
//! # Layer / placement
//!
//! Layer 1 — kernel — per ADR-0148 layered architecture discipline.
//! No I/O, no network, no codegen, no async runtime. The kernel is a
//! pure port-and-types crate so it can be linked into any layer.
//!
//! # Why a kernel pattern
//!
//! * **Adapter swap.** If Workday becomes unaffordable we swap to an
//!   in-house adapter — the contract here is stable, callers do not move.
//! * **Multi-tenant by construction.** Each call carries a [`ConnectorCtx`]
//!   carrying [`TenantId`], [`PrincipalId`], a [`SecretReference`] into
//!   OpenBao, a [`TraceContext`] handle, and an [`AuditSealHandle`] (ADR-0145).
//! * **Ontology projection.** Adapters emit projections into Ontology
//!   entities (Employee from Workday, Customer from Salesforce, …) using
//!   the canonical [`OntologyProjection`] envelope. The check
//!   `check-ontology-projection-coverage` enforces coverage.
//! * **Audit chain.** Every Connector call must seal an event through
//!   [`AuditSealHandle`] (ADR-0145). The check
//!   `check-audit-chain-seal-coverage` enforces coverage.
//! * **Rate limits as contracts.** Each adapter declares its
//!   [`RateLimitDescriptor`] up front so callers can budget. The kernel
//!   never queries a remote system — adapters do.
//!
//! # Tier-3 test exemption
//!
//! Per ADR-0083 tests legitimately use `.unwrap()` / `.expect()` /
//! `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;

// =====================================================================
// IDs and references
// =====================================================================

/// Tenant id under which the Connector call is performed.
/// Opaque, RLS-relevant per ADR-0056.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TenantId(String); // data_class: INTERNAL_ONLY

impl TenantId {
    /// Construct a tenant id. Empty strings are rejected.
    pub fn new(s: impl Into<String>) -> Result<Self, ConnectorError> {
        let s = s.into();
        if s.is_empty() {
            return Err(ConnectorError::InvalidArgument("tenant_id empty".into()));
        }
        Ok(Self(s))
    }
    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Principal acting through the Connector — service-account or user.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct PrincipalId(String); // data_class: INTERNAL_ONLY

impl PrincipalId {
    /// Construct a principal id. Empty strings are rejected.
    pub fn new(s: impl Into<String>) -> Result<Self, ConnectorError> {
        let s = s.into();
        if s.is_empty() {
            return Err(ConnectorError::InvalidArgument("principal_id empty".into()));
        }
        Ok(Self(s))
    }
    /// Borrow the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// SecretReference — opaque pointer into OpenBao (KV-v2 path).
/// Raw secret bytes never live in process memory.
/// `Debug` is redacted; there is no `Display`.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct SecretReference {
    path: String, // data_class: INTERNAL_ONLY
}

impl SecretReference {
    /// Construct a secret reference. Path must begin with `sref://`.
    pub fn new(path: impl Into<String>) -> Result<Self, ConnectorError> {
        let path = path.into();
        if !path.starts_with("sref://") {
            return Err(ConnectorError::InvalidArgument(
                "SecretReference must begin with sref://".into(),
            ));
        }
        Ok(Self { path })
    }
    /// Borrow the canonical path (`sref://...`).
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl fmt::Debug for SecretReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show path tag only; never the resolved secret.
        write!(
            f,
            "SecretReference(path=[REDACTED:{} chars])",
            self.path.len()
        )
    }
}

/// Idempotency key — at-most-once guarantee for `create` / `update` / `delete`.
/// Per ADR-0149 idempotency-key discipline.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct IdempotencyKey(String); // data_class: INTERNAL_ONLY

impl IdempotencyKey {
    /// Construct an idempotency key. Must be 8–128 chars.
    pub fn new(s: impl Into<String>) -> Result<Self, ConnectorError> {
        let s = s.into();
        if s.len() < 8 || s.len() > 128 {
            return Err(ConnectorError::InvalidArgument(
                "idempotency_key must be 8..=128 chars".into(),
            ));
        }
        Ok(Self(s))
    }
    /// Borrow the key.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Cursor — opaque pagination handle per ADR-0150 cursor-pagination.
/// Adapters define their own representation; the kernel only sees bytes.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Cursor(String); // data_class: INTERNAL_ONLY

impl Cursor {
    /// Construct a cursor. Empty cursors are rejected (use `None` instead).
    pub fn new(s: impl Into<String>) -> Result<Self, ConnectorError> {
        let s = s.into();
        if s.is_empty() {
            return Err(ConnectorError::InvalidArgument("cursor empty".into()));
        }
        Ok(Self(s))
    }
    /// Borrow the cursor bytes.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// TraceContext — W3C-tracecontext propagation handle (ADR-0182).
/// The kernel stores the serialized `traceparent` header; the adapter
/// is responsible for propagating it on the wire.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceContext {
    traceparent: String, // data_class: INTERNAL_ONLY
}

impl TraceContext {
    /// Construct from a W3C `traceparent` header value.
    /// Rejects empty values; full RFC validation is the tracing adapter's job.
    pub fn new(traceparent: impl Into<String>) -> Result<Self, ConnectorError> {
        let traceparent = traceparent.into();
        if traceparent.is_empty() {
            return Err(ConnectorError::InvalidArgument("traceparent empty".into()));
        }
        Ok(Self { traceparent })
    }
    /// Borrow the traceparent header value.
    pub fn traceparent(&self) -> &str {
        &self.traceparent
    }
}

/// AuditSealHandle — opaque handle into the audit-chain ADR-0145 seal stream.
/// Adapters call [`AuditSealHandle::seal`] after a Connector call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditSealHandle {
    chain_id: String, // data_class: INTERNAL_ONLY
}

impl AuditSealHandle {
    /// Construct from the audit-chain id this call attaches to.
    pub fn new(chain_id: impl Into<String>) -> Result<Self, ConnectorError> {
        let chain_id = chain_id.into();
        if chain_id.is_empty() {
            return Err(ConnectorError::InvalidArgument(
                "audit chain_id empty".into(),
            ));
        }
        Ok(Self { chain_id })
    }
    /// The chain id this handle is bound to.
    pub fn chain_id(&self) -> &str {
        &self.chain_id
    }
    /// Seal a record. Returns the seal receipt (chain_id + monotonic seq).
    ///
    /// Live impls (`shared-audit-chain-client-kernel`) emit a network
    /// call; tests rely on the deterministic stub here so the kernel
    /// stays I/O-free and adapter unit tests can run hermetically.
    pub fn seal(
        &self,
        kind: &str,
        payload_digest: &str,
    ) -> Result<AuditSealReceipt, ConnectorError> {
        if !is_canonical_sha256_hex(payload_digest) {
            return Err(ConnectorError::AuditSealFailed(format!(
                "{kind} payload digest must be canonical sha256"
            )));
        }
        Ok(AuditSealReceipt {
            chain_id: self.chain_id.clone(),
            kind: kind.to_owned(),
            payload_digest: payload_digest.to_owned(),
        })
    }
}

/// Receipt returned by [`AuditSealHandle::seal`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditSealReceipt {
    /// Chain id the seal lives on.
    pub chain_id: String, // data_class: INTERNAL_ONLY
    /// Event kind (`"connector.list"`, `"connector.create"`, …).
    pub kind: String, // data_class: INTERNAL_ONLY
    /// Digest of the payload (sha256 hex) — adapters compute this.
    pub payload_digest: String, // data_class: INTERNAL_ONLY
}

/// Return `true` when `value` is a canonical lower-case SHA-256 hex digest.
pub fn is_canonical_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Build the canonical SHA-256 payload digest adapters pass to audit seals.
///
/// The input is normalized by field name before hashing, and every field is
/// length-delimited to prevent ambiguous concatenation. Values may already be
/// digests (for example [`entity_doc_payload_digest`]); this function still
/// hashes the canonical payload envelope so audit-chain inputs never receive
/// raw tenant ids, entity ids, or entity kinds.
pub fn canonical_audit_payload_digest<I, K, V>(parts: I) -> String
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut normalized = BTreeMap::new();
    for (key, value) in parts {
        normalized.insert(key.as_ref().to_owned(), value.as_ref().to_owned());
    }

    let mut hasher = Sha256::new();
    for (key, value) in normalized {
        update_hash_field(&mut hasher, &key, value.as_bytes());
    }
    hex_lower(hasher.finalize().as_ref())
}
/// Build the canonical SHA-256 payload digest for a connector audit operation.
///
/// The envelope always binds provider, tenant, principal, and operation before
/// adding operation-specific fields. Callers pass document or patch content as
/// already-canonical digests so raw tenant ids, entity ids, idempotency keys,
/// or payload bodies never enter the audit seal as the seal payload itself.
pub fn connector_operation_audit_digest<I, K, V>(
    provider: &str,
    tenant: &str,
    principal: &str,
    operation: &str,
    fields: I,
) -> String
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut parts = vec![
        ("provider".to_owned(), provider.to_owned()),
        ("tenant".to_owned(), tenant.to_owned()),
        ("principal".to_owned(), principal.to_owned()),
        ("operation".to_owned(), operation.to_owned()),
    ];
    for (key, value) in fields {
        parts.push((key.as_ref().to_owned(), value.as_ref().to_owned()));
    }
    canonical_audit_payload_digest(parts)
}

/// Build a canonical SHA-256 digest for an [`EntityDoc`].
pub fn entity_doc_payload_digest(doc: &EntityDoc) -> String {
    let mut hasher = Sha256::new();
    for (key, value) in doc.iter() {
        let rendered = value.canonical_audit_value();
        update_hash_field(&mut hasher, key, rendered.as_bytes());
    }
    hex_lower(hasher.finalize().as_ref())
}

fn update_hash_field(hasher: &mut Sha256, key: &str, value: &[u8]) {
    hasher.update((key.len() as u64).to_be_bytes());
    hasher.update(key.as_bytes());
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value);
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

// =====================================================================
// ConnectorCtx
// =====================================================================

/// Per-call context carried into every Connector method.
///
/// Adapters must NOT cache this across calls — each call is its own
/// audit + tracing event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectorCtx {
    tenant_id: TenantId,           // data_class: INTERNAL_ONLY
    principal_id: PrincipalId,     // data_class: INTERNAL_ONLY
    secret_ref: SecretReference,   // data_class: INTERNAL_ONLY
    trace_ctx: TraceContext,       // data_class: INTERNAL_ONLY
    audit_handle: AuditSealHandle, // data_class: INTERNAL_ONLY
}

impl ConnectorCtx {
    /// Construct a context. All fields are mandatory.
    pub fn new(
        tenant_id: TenantId,
        principal_id: PrincipalId,
        secret_ref: SecretReference,
        trace_ctx: TraceContext,
        audit_handle: AuditSealHandle,
    ) -> Self {
        Self {
            tenant_id,
            principal_id,
            secret_ref,
            trace_ctx,
            audit_handle,
        }
    }
    /// Tenant id for this call (RLS partition key).
    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
    /// Acting principal.
    pub fn principal_id(&self) -> &PrincipalId {
        &self.principal_id
    }
    /// Secret reference (OpenBao path).
    pub fn secret_ref(&self) -> &SecretReference {
        &self.secret_ref
    }
    /// Trace context for propagation.
    pub fn trace_ctx(&self) -> &TraceContext {
        &self.trace_ctx
    }
    /// Audit-chain seal handle.
    pub fn audit_handle(&self) -> &AuditSealHandle {
        &self.audit_handle
    }
}

// =====================================================================
// EntityDoc, PatchOp, Page
// =====================================================================

/// Generic entity document — a sorted map of field name to scalar value.
///
/// The kernel deliberately stays string-typed so the surface is
/// `serde_json`-free (kernel is layer 1). Adapters convert provider
/// types to/from this shape; Ontology projections downstream re-type
/// fields according to the projection schema.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EntityDoc {
    fields: BTreeMap<String, EntityValue>, // data_class: INTERNAL_ONLY
}

impl EntityDoc {
    /// New empty document.
    pub fn new() -> Self {
        Self::default()
    }
    /// Insert a field.
    pub fn insert(&mut self, key: impl Into<String>, value: EntityValue) {
        self.fields.insert(key.into(), value);
    }
    /// Borrow a field.
    pub fn get(&self, key: &str) -> Option<&EntityValue> {
        self.fields.get(key)
    }
    /// Iterate fields in canonical (sorted) order.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &EntityValue)> {
        self.fields.iter()
    }
    /// Field count.
    pub fn len(&self) -> usize {
        self.fields.len()
    }
    /// True if the document is empty.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// Scalar value inside an [`EntityDoc`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EntityValue {
    /// String value (ids, names, codes).
    Str(String),
    /// 64-bit signed integer (counts, ordinals).
    Int(i64),
    /// Boolean.
    Bool(bool),
    /// Null / absent.
    Null,
}

impl EntityValue {
    fn canonical_audit_value(&self) -> String {
        match self {
            Self::Str(value) => format!("str:{value}"),
            Self::Int(value) => format!("int:{value}"),
            Self::Bool(value) => format!("bool:{value}"),
            Self::Null => "null:".to_owned(),
        }
    }
}

/// A single patch operation applied to an entity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatchOp {
    /// Field name to update.
    pub field: String, // data_class: INTERNAL_ONLY
    /// New value (None = remove).
    pub value: Option<EntityValue>, // data_class: INTERNAL_ONLY
}

impl PatchOp {
    /// Set a field to a value.
    pub fn set(field: impl Into<String>, value: EntityValue) -> Self {
        Self {
            field: field.into(),
            value: Some(value),
        }
    }
    /// Remove a field.
    pub fn remove(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            value: None,
        }
    }
}

/// Build a canonical SHA-256 digest for a [`PatchOp`].
pub fn patch_op_payload_digest(patch: &PatchOp) -> String {
    let value = patch
        .value
        .as_ref()
        .map(EntityValue::canonical_audit_value)
        .unwrap_or_else(|| "remove:".to_owned());
    canonical_audit_payload_digest([("field", patch.field.as_str()), ("value", value.as_str())])
}

/// Page of entities returned by [`Connector::list`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Page {
    /// Entities in this page.
    pub items: Vec<EntityDoc>, // data_class: INTERNAL_ONLY
    /// Cursor to the next page, or `None` if this is the last page.
    pub next_cursor: Option<Cursor>, // data_class: INTERNAL_ONLY
}

/// Build an offset-cursor page from an iterator without materializing the whole result set.
///
/// The iterator is advanced to the numeric `cursor` offset, then consumed for
/// at most `page_size + 1` records after that offset. The extra record is used
/// only to determine whether a next cursor exists. Use [`btree_keyset_page`]
/// when ordered map keys are available and non-initial pages must avoid
/// walking skipped records.
pub fn windowed_page<I>(
    items: I,
    cursor: Option<&Cursor>,
    page_size: usize,
) -> Result<Page, ConnectorError>
where
    I: IntoIterator<Item = EntityDoc>,
{
    if page_size == 0 {
        return Err(ConnectorError::InvalidArgument(
            "page_size must be greater than zero".into(),
        ));
    }

    let start = cursor
        .and_then(|c| c.as_str().parse::<usize>().ok())
        .unwrap_or(0);
    let mut iter = items.into_iter().skip(start);
    let mut page_items = Vec::with_capacity(page_size);

    for _ in 0..page_size {
        match iter.next() {
            Some(item) => page_items.push(item),
            None => break,
        }
    }

    let next_cursor = if iter.next().is_some() {
        Cursor::new(start.saturating_add(page_size).to_string()).ok()
    } else {
        None
    };

    Ok(Page {
        items: page_items,
        next_cursor,
    })
}

/// Build a keyset-cursor page from a [`BTreeMap`] without materializing the whole result set.
///
/// The cursor is the last key returned by the previous page. The map range is
/// pre-positioned after that key, so every page consumes at most `page_size + 1`
/// records regardless of cursor depth. The extra record is used only to
/// determine whether a next cursor exists.
pub fn btree_keyset_page(
    items: Option<&BTreeMap<String, EntityDoc>>,
    cursor: Option<&Cursor>,
    page_size: usize,
) -> Result<Page, ConnectorError> {
    use std::ops::Bound::{Excluded, Unbounded};

    if page_size == 0 {
        return Err(ConnectorError::InvalidArgument(
            "page_size must be greater than zero".into(),
        ));
    }

    let Some(items) = items else {
        return Ok(Page {
            items: Vec::new(),
            next_cursor: None,
        });
    };

    let mut iter = match cursor {
        Some(cursor) => items.range::<str, _>((Excluded(cursor.as_str()), Unbounded)),
        None => items.range::<str, _>((Unbounded, Unbounded)),
    };
    let mut page_items = Vec::with_capacity(page_size);
    let mut last_key = None;

    for _ in 0..page_size {
        match iter.next() {
            Some((key, item)) => {
                last_key = Some(key.clone());
                page_items.push(item.clone());
            }
            None => break,
        }
    }

    let next_cursor = if iter.next().is_some() {
        last_key.and_then(|key| Cursor::new(key).ok())
    } else {
        None
    };

    Ok(Page {
        items: page_items,
        next_cursor,
    })
}

// =====================================================================
// Capabilities, rate limits, auth, health
// =====================================================================

/// Which CRUD-S verbs an adapter supports.
///
/// Adapters MUST declare honestly. Calls into unsupported verbs fail
/// fast with [`ConnectorError::Unsupported`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnectorCapabilities {
    /// Adapter supports [`Connector::list`].
    pub list: bool, // data_class: INTERNAL_ONLY
    /// Adapter supports [`Connector::get`].
    pub get: bool, // data_class: INTERNAL_ONLY
    /// Adapter supports [`Connector::create`].
    pub create: bool, // data_class: INTERNAL_ONLY
    /// Adapter supports [`Connector::update`].
    pub update: bool, // data_class: INTERNAL_ONLY
    /// Adapter supports [`Connector::delete`].
    pub delete: bool, // data_class: INTERNAL_ONLY
    /// Adapter supports [`Connector::subscribe`] (event stream).
    pub subscribe: bool, // data_class: INTERNAL_ONLY
}

impl ConnectorCapabilities {
    /// Adapter supports everything (rare — most providers don't expose
    /// `delete` for HRIS records, for instance).
    pub const ALL: Self = Self {
        list: true,
        get: true,
        create: true,
        update: true,
        delete: true,
        subscribe: true,
    };
    /// Read-only adapter (`list` + `get` only).
    pub const READ_ONLY: Self = Self {
        list: true,
        get: true,
        create: false,
        update: false,
        delete: false,
        subscribe: false,
    };
}

/// Rate-limit declaration so callers can budget per-tenant requests.
///
/// Adapters declare what the upstream provider exposes (e.g. Salesforce
/// `daily_api_request_quota`, Slack tier-3 50rpm). The kernel does not
/// query upstream — it relies on adapters to publish.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateLimitDescriptor {
    /// Requests per second the adapter promises not to exceed.
    pub requests_per_second: u32, // data_class: INTERNAL_ONLY
    /// Burst capacity the adapter will absorb without throttling.
    pub burst_capacity: u32, // data_class: INTERNAL_ONLY
    /// Per-day request quota, if the provider enforces one.
    pub daily_quota: Option<u64>, // data_class: INTERNAL_ONLY
    /// Free-form human note (e.g. `"tier-3 Slack bot"`).
    pub note: String, // data_class: INTERNAL_ONLY
}

/// Auth scheme an adapter uses to authenticate to its provider.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthScheme {
    /// OAuth 2.0 (authorization code, refresh-token).
    OAuth2,
    /// Long-lived API key in a header.
    ApiKey,
    /// Mutual TLS with client cert from OpenBao.
    MutualTls,
    /// Signed JWT (e.g. service account assertion).
    SignedJwt,
}

/// Health-report values returned by [`Connector::health`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthReport {
    /// Is the upstream provider currently reachable?
    pub reachable: bool, // data_class: INTERNAL_ONLY
    /// Round-trip latency in milliseconds (last observed).
    pub last_latency_ms: u64, // data_class: INTERNAL_ONLY
    /// Provider-side status string (`"ok"`, `"degraded"`, `"down"`).
    pub upstream_status: String, // data_class: INTERNAL_ONLY
}

// =====================================================================
// Event streams (subscribe)
// =====================================================================

/// Event emitted on the [`Connector::subscribe`] stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    /// Entity kind this event concerns.
    pub entity_kind: String, // data_class: INTERNAL_ONLY
    /// Event kind (`"created"`, `"updated"`, `"deleted"`).
    pub kind: String, // data_class: INTERNAL_ONLY
    /// Entity snapshot at event time.
    pub doc: EntityDoc, // data_class: INTERNAL_ONLY
}

/// Polling event stream — `next()` returns the next event or `None`
/// when the stream is exhausted.
///
/// The kernel deliberately uses a poll-shape (not `Stream`) so it does
/// not pull in `futures`. Adapters that wrap an async source build an
/// internal queue and drain it through this poll.
pub trait EventStream: Send + Sync {
    /// Return the next event, or `None` if exhausted.
    fn next(&mut self) -> Option<Event>;
}

// =====================================================================
// Ontology projection
// =====================================================================

/// Ontology projection emitted alongside Connector results.
///
/// The Ontology µservice consumes these via the inter-µservice eventing
/// substrate (ADR-0145). Every adapter that returns entities MUST emit
/// a projection; `check-ontology-projection-coverage` enforces this.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OntologyProjection {
    /// Object type name in the Ontology (e.g. `"Employee"`).
    pub object_type: String, // data_class: INTERNAL_ONLY
    /// Stable Ontology id (vendor id namespaced by provider).
    pub object_id: String, // data_class: INTERNAL_ONLY
    /// Field projections (ontology field → entity field name).
    pub field_map: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

impl OntologyProjection {
    /// New projection.
    pub fn new(object_type: impl Into<String>, object_id: impl Into<String>) -> Self {
        Self {
            object_type: object_type.into(),
            object_id: object_id.into(),
            field_map: BTreeMap::new(),
        }
    }
    /// Map an ontology field to an entity field.
    pub fn map_field(
        mut self,
        ontology_field: impl Into<String>,
        entity_field: impl Into<String>,
    ) -> Self {
        self.field_map
            .insert(ontology_field.into(), entity_field.into());
        self
    }
}

// =====================================================================
// Errors
// =====================================================================

/// All Connector failures funnel through this enum so adapters and
/// callers share one error vocabulary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectorError {
    /// Caller passed a malformed argument (id, cursor, payload, …).
    InvalidArgument(String),
    /// Adapter does not support the requested verb / entity kind.
    Unsupported(String),
    /// Entity not found.
    NotFound(String),
    /// Idempotency-key replay — the operation already succeeded with a
    /// different payload; per ADR-0149 we surface the conflict.
    IdempotencyConflict(String),
    /// Upstream provider returned an error.
    UpstreamRejected(String),
    /// Rate limit / quota exceeded.
    RateLimited(String),
    /// Upstream network unreachable; retryable.
    Unreachable(String),
    /// Authentication failed (bad SecretReference / expired token).
    AuthFailed(String),
    /// Audit-chain seal could not be emitted; per ADR-0145 the call
    /// must NOT silently succeed if its seal failed.
    AuditSealFailed(String),
}

impl fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(s) => write!(f, "invalid argument: {s}"),
            Self::Unsupported(s) => write!(f, "unsupported: {s}"),
            Self::NotFound(s) => write!(f, "not found: {s}"),
            Self::IdempotencyConflict(s) => write!(f, "idempotency conflict: {s}"),
            Self::UpstreamRejected(s) => write!(f, "upstream rejected: {s}"),
            Self::RateLimited(s) => write!(f, "rate limited: {s}"),
            Self::Unreachable(s) => write!(f, "upstream unreachable: {s}"),
            Self::AuthFailed(s) => write!(f, "auth failed: {s}"),
            Self::AuditSealFailed(s) => write!(f, "audit seal failed: {s}"),
        }
    }
}

impl std::error::Error for ConnectorError {}

// =====================================================================
// The Connector trait
// =====================================================================

/// The enterprise integration substrate contract.
///
/// Every external SaaS integration implements this trait. Adapters live
/// in `connector-<vendor>-adapter` crates.
///
/// # Synchronous shape
///
/// Methods are synchronous so the kernel stays runtime-agnostic.
/// Adapters that wrap async clients drive their runtime internally
/// (typically `tokio::runtime::Handle::block_on` inside the adapter
/// crate, which is layer 5+). See ADR-0148 layered architecture.
pub trait Connector: Send + Sync {
    /// Stable provider id (`"slack"`, `"workday"`, `"epic-fhir"`).
    fn provider_id(&self) -> &str;

    /// Which CRUD-S verbs this adapter supports.
    fn capabilities(&self) -> ConnectorCapabilities;

    /// List entities of `entity_kind`. Pagination via `cursor`.
    fn list(
        &self,
        ctx: &ConnectorCtx,
        entity_kind: &str,
        cursor: Option<Cursor>,
    ) -> Result<Page, ConnectorError>;

    /// Read a single entity by id.
    fn get(
        &self,
        ctx: &ConnectorCtx,
        entity_kind: &str,
        id: &str,
    ) -> Result<EntityDoc, ConnectorError>;

    /// Create a new entity. `idempotency_key` enforces at-most-once.
    fn create(
        &self,
        ctx: &ConnectorCtx,
        entity_kind: &str,
        payload: EntityDoc,
        idempotency_key: IdempotencyKey,
    ) -> Result<EntityDoc, ConnectorError>;

    /// Apply a patch to an existing entity.
    fn update(
        &self,
        ctx: &ConnectorCtx,
        entity_kind: &str,
        id: &str,
        patch: PatchOp,
        idempotency_key: IdempotencyKey,
    ) -> Result<EntityDoc, ConnectorError>;

    /// Delete an entity.
    fn delete(&self, ctx: &ConnectorCtx, entity_kind: &str, id: &str)
    -> Result<(), ConnectorError>;

    /// Subscribe to change events on the given entity kinds.
    fn subscribe(
        &self,
        ctx: &ConnectorCtx,
        entity_kinds: &[String],
    ) -> Result<Box<dyn EventStream>, ConnectorError>;

    /// Adapter-local health probe.
    fn health(&self) -> Result<HealthReport, ConnectorError>;

    /// Declared per-provider rate limits.
    fn rate_limits(&self) -> RateLimitDescriptor;

    /// Auth scheme used to talk to the provider.
    fn auth_scheme(&self) -> AuthScheme;

    /// Ontology projection map for entities this adapter emits.
    ///
    /// Default impl returns empty — adapters override.
    fn ontology_projections(&self) -> Vec<OntologyProjection> {
        Vec::new()
    }
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_ctx() -> ConnectorCtx {
        ConnectorCtx::new(
            TenantId::new("t-1").unwrap(),
            PrincipalId::new("svc-1").unwrap(),
            SecretReference::new("sref://t-1/workday").unwrap(),
            TraceContext::new("00-trace-span-01").unwrap(),
            AuditSealHandle::new("chain-1").unwrap(),
        )
    }

    #[test]
    fn tenant_id_rejects_empty() {
        assert!(TenantId::new("").is_err());
    }

    #[test]
    fn principal_id_rejects_empty() {
        assert!(PrincipalId::new("").is_err());
    }

    #[test]
    fn secret_reference_requires_sref_scheme() {
        assert!(SecretReference::new("https://bad").is_err());
        assert!(SecretReference::new("sref://t/x").is_ok());
    }

    #[test]
    fn secret_reference_debug_is_redacted() {
        let s = SecretReference::new("sref://very-private-token").unwrap();
        let dbg = format!("{s:?}");
        assert!(dbg.contains("[REDACTED"));
        assert!(!dbg.contains("very-private-token"));
    }

    #[test]
    fn idempotency_key_bounds_enforced() {
        assert!(IdempotencyKey::new("short").is_err());
        assert!(IdempotencyKey::new("a".repeat(8)).is_ok());
        assert!(IdempotencyKey::new("a".repeat(128)).is_ok());
        assert!(IdempotencyKey::new("a".repeat(129)).is_err());
    }

    #[test]
    fn cursor_rejects_empty() {
        assert!(Cursor::new("").is_err());
        assert!(Cursor::new("page-2").is_ok());
    }

    #[test]
    fn trace_context_rejects_empty() {
        assert!(TraceContext::new("").is_err());
    }

    #[test]
    fn audit_seal_emits_receipt() {
        let h = AuditSealHandle::new("chain-1").unwrap();
        let digest = canonical_audit_payload_digest([
            ("entity_kind", "message"),
            ("id", "1700000001.000100"),
        ]);
        let r = h.seal("connector.list", &digest).unwrap();
        assert_eq!(r.chain_id, "chain-1");
        assert_eq!(r.kind, "connector.list");
        assert_eq!(r.payload_digest, digest);

        assert!(matches!(
            h.seal("connector.list", "deadbeef"),
            Err(ConnectorError::AuditSealFailed(_))
        ));
    }

    #[test]
    fn canonical_audit_payload_digest_is_sha256_hex_not_raw() {
        let digest = canonical_audit_payload_digest([
            ("entity_kind", "message"),
            ("id", "1700000001.000100"),
        ]);
        let reordered = canonical_audit_payload_digest([
            ("id", "1700000001.000100"),
            ("entity_kind", "message"),
        ]);

        assert!(is_canonical_sha256_hex(&digest));
        assert_eq!(digest, reordered);
        assert_ne!(digest, "message");
        assert_ne!(digest, "1700000001.000100");
    }

    #[test]
    fn connector_operation_audit_digest_binds_operation_and_redacts_inputs() {
        let digest = connector_operation_audit_digest(
            "slack",
            "tenant-secret",
            "principal-secret",
            "connector.create",
            [
                ("entity_kind", "message"),
                ("id", "raw-id"),
                ("idempotency_key", "raw-idempotency-key"),
                (
                    "doc_digest",
                    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                ),
            ],
        );
        let different_operation = connector_operation_audit_digest(
            "slack",
            "tenant-secret",
            "principal-secret",
            "connector.update",
            [
                ("entity_kind", "message"),
                ("id", "raw-id"),
                ("idempotency_key", "raw-idempotency-key"),
                (
                    "doc_digest",
                    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                ),
            ],
        );
        let different_principal = connector_operation_audit_digest(
            "slack",
            "tenant-secret",
            "other-principal",
            "connector.create",
            [
                ("entity_kind", "message"),
                ("id", "raw-id"),
                ("idempotency_key", "raw-idempotency-key"),
                (
                    "doc_digest",
                    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                ),
            ],
        );

        assert!(is_canonical_sha256_hex(&digest));
        assert_ne!(digest, different_operation);
        assert_ne!(digest, different_principal);
        assert_ne!(digest, "tenant-secret");
        assert_ne!(digest, "principal-secret");
        assert_ne!(digest, "raw-id");
        assert_ne!(digest, "raw-idempotency-key");
    }

    #[test]
    fn entity_doc_payload_digest_is_canonical() {
        let mut first = EntityDoc::new();
        first.insert("b", EntityValue::Int(2));
        first.insert("a", EntityValue::Str("1".into()));

        let mut second = EntityDoc::new();
        second.insert("a", EntityValue::Str("1".into()));
        second.insert("b", EntityValue::Int(2));

        let mut type_changed = EntityDoc::new();
        type_changed.insert("a", EntityValue::Int(1));
        type_changed.insert("b", EntityValue::Int(2));

        assert_eq!(
            entity_doc_payload_digest(&first),
            entity_doc_payload_digest(&second)
        );
        assert_ne!(
            entity_doc_payload_digest(&first),
            entity_doc_payload_digest(&type_changed)
        );
    }

    #[test]
    fn patch_op_payload_digest_distinguishes_set_remove_and_type() {
        let set_string = PatchOp::set("name", EntityValue::Str("1".into()));
        let set_int = PatchOp::set("name", EntityValue::Int(1));
        let remove = PatchOp::remove("name");

        assert!(is_canonical_sha256_hex(&patch_op_payload_digest(
            &set_string
        )));
        assert_ne!(
            patch_op_payload_digest(&set_string),
            patch_op_payload_digest(&set_int)
        );
        assert_ne!(
            patch_op_payload_digest(&set_string),
            patch_op_payload_digest(&remove)
        );
    }

    #[test]
    fn windowed_page_consumes_only_page_plus_one() {
        struct CountingDocs {
            emitted: std::rc::Rc<std::cell::Cell<usize>>,
            remaining: usize,
        }

        impl Iterator for CountingDocs {
            type Item = EntityDoc;

            fn next(&mut self) -> Option<Self::Item> {
                if self.remaining == 0 {
                    return None;
                }
                self.remaining -= 1;
                self.emitted.set(self.emitted.get() + 1);
                Some(EntityDoc::new())
            }
        }

        let emitted = std::rc::Rc::new(std::cell::Cell::new(0));
        let page = windowed_page(
            CountingDocs {
                emitted: emitted.clone(),
                remaining: 1_000,
            },
            None,
            100,
        )
        .unwrap();

        assert_eq!(page.items.len(), 100);
        assert_eq!(page.next_cursor.as_ref().map(Cursor::as_str), Some("100"));
        assert_eq!(emitted.get(), 101);
    }

    #[test]
    fn btree_keyset_page_uses_last_key_cursor() {
        let docs: BTreeMap<String, EntityDoc> = (1..=201)
            .map(|i| (format!("doc-{i:08}"), EntityDoc::new()))
            .collect();

        let first = btree_keyset_page(Some(&docs), None, 100).unwrap();
        assert_eq!(first.items.len(), 100);
        assert_eq!(
            first.next_cursor.as_ref().map(Cursor::as_str),
            Some("doc-00000100")
        );

        let second = btree_keyset_page(Some(&docs), first.next_cursor.as_ref(), 100).unwrap();
        assert_eq!(second.items.len(), 100);
        assert_eq!(
            second.next_cursor.as_ref().map(Cursor::as_str),
            Some("doc-00000200")
        );

        let exact: BTreeMap<String, EntityDoc> = (1..=100)
            .map(|i| (format!("doc-{i:08}"), EntityDoc::new()))
            .collect();
        let exact_page = btree_keyset_page(Some(&exact), None, 100).unwrap();
        assert_eq!(exact_page.items.len(), 100);
        assert!(exact_page.next_cursor.is_none());
    }

    #[test]
    fn entity_doc_is_sorted_canonical() {
        let mut d = EntityDoc::new();
        d.insert("b", EntityValue::Int(2));
        d.insert("a", EntityValue::Int(1));
        let keys: Vec<&String> = d.iter().map(|(k, _)| k).collect();
        assert_eq!(keys, vec!["a", "b"]);
        assert_eq!(d.len(), 2);
        assert!(!d.is_empty());
    }

    #[test]
    fn patch_op_set_and_remove_distinct() {
        let s = PatchOp::set("name", EntityValue::Str("x".into()));
        let r = PatchOp::remove("name");
        assert!(s.value.is_some());
        assert!(r.value.is_none());
    }

    #[test]
    fn capabilities_all_includes_everything() {
        let c = ConnectorCapabilities::ALL;
        assert!(c.list && c.get && c.create && c.update && c.delete && c.subscribe);
    }

    #[test]
    fn capabilities_read_only_excludes_writes() {
        let c = ConnectorCapabilities::READ_ONLY;
        assert!(c.list && c.get);
        assert!(!c.create && !c.update && !c.delete && !c.subscribe);
    }

    #[test]
    fn connector_error_display_distinct() {
        let msgs: Vec<String> = vec![
            ConnectorError::InvalidArgument("a".into()).to_string(),
            ConnectorError::Unsupported("a".into()).to_string(),
            ConnectorError::NotFound("a".into()).to_string(),
            ConnectorError::IdempotencyConflict("a".into()).to_string(),
            ConnectorError::UpstreamRejected("a".into()).to_string(),
            ConnectorError::RateLimited("a".into()).to_string(),
            ConnectorError::Unreachable("a".into()).to_string(),
            ConnectorError::AuthFailed("a".into()).to_string(),
            ConnectorError::AuditSealFailed("a".into()).to_string(),
        ];
        let unique: std::collections::HashSet<_> = msgs.iter().collect();
        assert_eq!(unique.len(), msgs.len());
    }

    #[test]
    fn ontology_projection_builds_field_map() {
        let p = OntologyProjection::new("Employee", "wd:1234")
            .map_field("givenName", "first_name")
            .map_field("familyName", "last_name");
        assert_eq!(p.object_type, "Employee");
        assert_eq!(p.object_id, "wd:1234");
        assert_eq!(p.field_map.len(), 2);
    }

    #[test]
    fn ctx_round_trip_fields_accessible() {
        let c = fixture_ctx();
        assert_eq!(c.tenant_id().as_str(), "t-1");
        assert_eq!(c.principal_id().as_str(), "svc-1");
        assert_eq!(c.secret_ref().path(), "sref://t-1/workday");
        assert_eq!(c.trace_ctx().traceparent(), "00-trace-span-01");
        assert_eq!(c.audit_handle().chain_id(), "chain-1");
    }
}
