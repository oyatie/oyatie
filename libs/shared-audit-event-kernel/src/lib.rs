//! Audit-event kernel — ADR-0536 D-16 canonical audit emission contract.
//!
//! # Decision (ADR-0536 D-16)
//!
//! One audit crate emits CloudEvents-enveloped, AuditLog-shaped payloads
//! from HTTP middleware — services cannot choose whether to emit.
//! Asymmetric defaults: the admin/management event stream is ALWAYS-ON
//! with no kill switch; data-plane events are policy-opt-in. Integrity is
//! a signed digest chain anchored into CAS WORM storage (D-11).
//!
//! # Structural always-on (AMENDMENT 5 enforcement layering)
//!
//! The no-kill-switch rule is enforced by construction, not by lint alone:
//! [`EmissionPolicy`] has no representable state that suppresses
//! [`AuditStream::AdminActivity`]. Its only field gates data-access
//! emission and is private, so no caller can build a policy that turns the
//! admin stream off. The CI lint (gate app, later G008 sub-slice) is the
//! safety net; the type system is the load-bearing layer.
//!
//! # Digest chain (CloudTrail lineage)
//!
//! [`DigestChainLink`] models one sealed batch of audit events. Links are
//! chained by digesting each link's canonical bytes (signature included)
//! into the next link's `prev_link_digest_hex`. Hashing and signing are
//! PORTS ([`Digester`], [`ChainSigner`], [`ChainVerifier`]) shaped for the
//! owned KMS destination (AMENDMENT 2); the transitional aws-lc-rs impl
//! lives in `shared-audit-digest-adapter-awslc` (ADR-0506).
//!
//! # References
//!
//! - ADR-0536 D-16 (audit), D-6 (observability), D-11 (CAS WORM anchor).
//! - ADR-0145 Invariant 1 — `shared-audit-chain-client-kernel` owns
//!   the per-call SEAL surface; this crate owns the canonical EVENT shape
//!   and the chain integrity primitives. The two compose: seals reference
//!   event digests.
//! - CloudEvents 1.0 (CNCF), GCP AuditLog payload shape, AWS CloudTrail
//!   digest files.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

use serde::{Deserialize, Serialize};

/// Closed schema version for the AuditLog-shaped payload. Bump only via ADR.
pub const AUDIT_PAYLOAD_SCHEMA_VERSION: u32 = 1;

/// CloudEvents spec version this kernel emits and accepts.
pub const CLOUDEVENTS_SPEC_VERSION: &str = "1.0";

/// CloudEvents `type` for admin/management-plane audit events.
pub const EVENT_TYPE_ADMIN_ACTIVITY: &str = "com.oyatie.audit.admin-activity.v1";

/// CloudEvents `type` for data-plane access audit events.
pub const EVENT_TYPE_DATA_ACCESS: &str = "com.oyatie.audit.data-access.v1";

/// CloudEvents `datacontenttype` — payloads are always canonical JSON.
pub const DATA_CONTENT_TYPE_JSON: &str = "application/json";

/// Canonical tenant-id prefix (house convention, see
/// `cloud-observability-domain::TENANT_ID_PREFIX`).
pub const TENANT_ID_PREFIX: &str = "ten_";

/// `prev_link_digest_hex` value of the first link in a digest chain.
pub const GENESIS_PREV_LINK_DIGEST: &str = "genesis";

/// Domain-separation tag prepended to every link's canonical signing bytes.
pub const DIGEST_CHAIN_DOMAIN_TAG: &str = "audit-digest-chain.v1";

// ---------------------------------------------------------------------------
// Streams + asymmetric emission policy
// ---------------------------------------------------------------------------

/// The two audit streams with ASYMMETRIC defaults per ADR-0536 D-16.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditStream {
    /// Admin/management-plane events. Always-on; structurally
    /// non-disableable (no kill switch exists in this type system).
    AdminActivity,
    /// Data-plane access events. Policy-opt-in.
    DataAccess,
}

impl AuditStream {
    pub fn event_type(self) -> &'static str {
        match self {
            Self::AdminActivity => EVENT_TYPE_ADMIN_ACTIVITY,
            Self::DataAccess => EVENT_TYPE_DATA_ACCESS,
        }
    }
}

/// Emission policy with the no-kill-switch invariant encoded structurally.
///
/// The single private field gates ONLY data-access emission. There is no
/// constructor, field, serde surface, or method through which a caller can
/// express "do not emit admin activity" — [`Self::must_emit`] returns
/// `true` for [`AuditStream::AdminActivity`] unconditionally.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EmissionPolicy {
    data_access_enabled: bool, // data_class: INTERNAL_ONLY
}

impl EmissionPolicy {
    /// Admin stream only (the default posture). Data access not emitted.
    pub fn admin_only() -> Self {
        Self {
            data_access_enabled: false,
        }
    }

    /// Admin stream PLUS opt-in data-access emission.
    pub fn with_data_access() -> Self {
        Self {
            data_access_enabled: true,
        }
    }

    /// Whether `stream` must be emitted under this policy. The admin arm
    /// is a structural constant — not influenced by any state.
    pub fn must_emit(&self, stream: AuditStream) -> bool {
        match stream {
            AuditStream::AdminActivity => true,
            AuditStream::DataAccess => self.data_access_enabled,
        }
    }
}

// ---------------------------------------------------------------------------
// AuditLog-shaped payload (GCP Cloud Audit Logs lineage)
// ---------------------------------------------------------------------------

/// Who performed the operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthenticationInfo {
    /// Stable principal identifier (workload or human), never a secret.
    pub principal: String, // data_class: TENANT_SCOPED
}

/// One authorization decision consulted for the operation (one entry per
/// (resource, permission) tuple checked — Cedar PDP emits one per check).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthorizationInfo {
    pub resource: String,   // data_class: TENANT_SCOPED
    pub permission: String, // data_class: INTERNAL_ONLY
    pub granted: bool,      // data_class: INTERNAL_ONLY
    /// Content-addressed policy bundle version the decision was made
    /// against (ADR-0536 D-2); `None` only for pre-PDP bootstrap paths.
    pub policy_version: Option<String>, // data_class: INTERNAL_ONLY
}

/// Caller-context metadata. Optional fields are omitted, never faked.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct RequestMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caller_ip: Option<String>, // data_class: PII_QUASI_IDENTIFIER
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caller_supplied_user_agent: Option<String>, // data_class: PII_QUASI_IDENTIFIER
}

/// Operation outcome, gRPC-style status code space (0 = OK).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuditStatus {
    pub code: u32,       // data_class: INTERNAL_ONLY
    pub message: String, // data_class: INTERNAL_ONLY
}

impl AuditStatus {
    pub fn ok() -> Self {
        Self {
            code: 0,
            message: String::new(),
        }
    }
}

/// The AuditLog-shaped payload carried as CloudEvents `data`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuditLogPayload {
    /// Closed schema version (see [`AUDIT_PAYLOAD_SCHEMA_VERSION`]).
    pub schema_version: u32, // data_class: INTERNAL_ONLY
    pub stream: AuditStream, // data_class: INTERNAL_ONLY
    /// Emitting service (e.g. "cloud-tenancy").
    pub service_name: String, // data_class: INTERNAL_ONLY
    /// Fully-qualified method (e.g. "oya.cloud.tenancy.v1.CreateTenant").
    pub method_name: String, // data_class: INTERNAL_ONLY
    /// Resource the operation acted on (AIP-122 resource name).
    pub resource_name: String, // data_class: TENANT_SCOPED
    /// Tenant scope; `None` only for platform-level (tenantless) admin ops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>, // data_class: TENANT_SCOPED
    /// Cell that served the operation (first-class per cell doctrine).
    pub cell_id: String, // data_class: INTERNAL_ONLY
    pub authentication_info: AuthenticationInfo, // data_class: TENANT_SCOPED
    #[serde(default)]
    pub authorization_info: Vec<AuthorizationInfo>, // data_class: TENANT_SCOPED
    #[serde(default)]
    pub request_metadata: RequestMetadata, // data_class: PII_QUASI_IDENTIFIER
    pub status: AuditStatus, // data_class: INTERNAL_ONLY
}

// ---------------------------------------------------------------------------
// CloudEvents 1.0 envelope
// ---------------------------------------------------------------------------

/// CloudEvents 1.0 envelope around an [`AuditLogPayload`]. Closed shape:
/// unknown envelope fields are rejected on deserialization so the wire
/// contract cannot drift silently.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditCloudEvent {
    pub specversion: String, // data_class: INTERNAL_ONLY
    /// Producer-unique event id (ULID by house convention).
    pub id: String, // data_class: INTERNAL_ONLY
    /// Emitting context URI (e.g. "//oyatie.com/cloud-tenancy/cell/c1").
    pub source: String, // data_class: INTERNAL_ONLY
    #[serde(rename = "type")]
    pub event_type: String, // data_class: INTERNAL_ONLY
    /// RFC 3339 timestamp supplied by the caller's clock port — this
    /// kernel never reads an ambient clock.
    pub time: String, // data_class: INTERNAL_ONLY
    /// The acted-on resource (mirrors `data.resource_name`).
    pub subject: String, // data_class: TENANT_SCOPED
    pub datacontenttype: String, // data_class: INTERNAL_ONLY
    pub data: AuditLogPayload, // data_class: TENANT_SCOPED
}

impl AuditCloudEvent {
    /// Build a validated envelope. The CloudEvents `type` and `subject`
    /// are DERIVED from the payload — callers cannot mislabel a stream.
    ///
    /// # Errors
    /// Returns the first [`AuditEventError`] violated by the inputs.
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        time_rfc3339: impl Into<String>,
        payload: AuditLogPayload,
    ) -> Result<Self, AuditEventError> {
        let event = Self {
            specversion: CLOUDEVENTS_SPEC_VERSION.to_owned(),
            id: id.into(),
            source: source.into(),
            event_type: payload.stream.event_type().to_owned(),
            time: time_rfc3339.into(),
            subject: payload.resource_name.clone(),
            datacontenttype: DATA_CONTENT_TYPE_JSON.to_owned(),
            data: payload,
        };
        event.validate()?;
        Ok(event)
    }

    /// Validate envelope + payload invariants. Deserialized events MUST be
    /// re-validated before trust (wire data is data, not instructions).
    ///
    /// # Errors
    /// Returns the first [`AuditEventError`] found.
    pub fn validate(&self) -> Result<(), AuditEventError> {
        if self.specversion != CLOUDEVENTS_SPEC_VERSION {
            return Err(AuditEventError::UnsupportedSpecVersion(
                self.specversion.clone(),
            ));
        }
        if self.id.trim().is_empty() {
            return Err(AuditEventError::EmptyField("id"));
        }
        if self.source.trim().is_empty() {
            return Err(AuditEventError::EmptyField("source"));
        }
        if self.time.trim().is_empty() {
            return Err(AuditEventError::EmptyField("time"));
        }
        if self.event_type != self.data.stream.event_type() {
            return Err(AuditEventError::TypeStreamMismatch {
                event_type: self.event_type.clone(),
                stream: self.data.stream,
            });
        }
        if self.datacontenttype != DATA_CONTENT_TYPE_JSON {
            return Err(AuditEventError::UnsupportedContentType(
                self.datacontenttype.clone(),
            ));
        }
        let p = &self.data;
        if p.schema_version != AUDIT_PAYLOAD_SCHEMA_VERSION {
            return Err(AuditEventError::UnsupportedPayloadSchema(p.schema_version));
        }
        if p.service_name.trim().is_empty() {
            return Err(AuditEventError::EmptyField("service_name"));
        }
        if p.method_name.trim().is_empty() {
            return Err(AuditEventError::EmptyField("method_name"));
        }
        if p.resource_name.trim().is_empty() {
            return Err(AuditEventError::EmptyField("resource_name"));
        }
        if p.cell_id.trim().is_empty() {
            return Err(AuditEventError::EmptyField("cell_id"));
        }
        if p.authentication_info.principal.trim().is_empty() {
            return Err(AuditEventError::EmptyField("authentication_info.principal"));
        }
        if let Some(tenant) = &p.tenant_id
            && !tenant.starts_with(TENANT_ID_PREFIX)
        {
            return Err(AuditEventError::MalformedTenantId(tenant.clone()));
        }
        if self.subject != p.resource_name {
            return Err(AuditEventError::SubjectResourceMismatch);
        }
        Ok(())
    }

    /// Canonical JSON bytes for digesting/sealing this event.
    ///
    /// # Errors
    /// Returns [`AuditEventError::Serialization`] if serde fails (only
    /// possible via pathological float/UTF-8 states; struct is closed).
    pub fn canonical_json(&self) -> Result<Vec<u8>, AuditEventError> {
        serde_json::to_vec(self).map_err(|e| AuditEventError::Serialization(e.to_string()))
    }
}

/// Failure surface for envelope/payload validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditEventError {
    EmptyField(&'static str),
    UnsupportedSpecVersion(String),
    UnsupportedContentType(String),
    UnsupportedPayloadSchema(u32),
    MalformedTenantId(String),
    TypeStreamMismatch {
        event_type: String,
        stream: AuditStream,
    },
    SubjectResourceMismatch,
    Serialization(String),
    /// Sink rejected the emission (transport-level detail in message).
    SinkRejected(String),
}

impl fmt::Display for AuditEventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(name) => write!(f, "required audit field {name:?} is empty"),
            Self::UnsupportedSpecVersion(v) => {
                write!(f, "unsupported CloudEvents specversion {v:?}")
            }
            Self::UnsupportedContentType(t) => {
                write!(f, "unsupported datacontenttype {t:?}")
            }
            Self::UnsupportedPayloadSchema(v) => {
                write!(f, "unsupported audit payload schema_version {v}")
            }
            Self::MalformedTenantId(t) => {
                write!(f, "tenant_id {t:?} missing {TENANT_ID_PREFIX:?} prefix")
            }
            Self::TypeStreamMismatch { event_type, stream } => write!(
                f,
                "CloudEvents type {event_type:?} does not match stream {stream:?}"
            ),
            Self::SubjectResourceMismatch => {
                write!(f, "envelope subject does not mirror data.resource_name")
            }
            Self::Serialization(e) => write!(f, "audit event serialization failed: {e}"),
            Self::SinkRejected(e) => write!(f, "audit sink rejected emission: {e}"),
        }
    }
}

impl std::error::Error for AuditEventError {}

/// Port every emitting surface (HTTP middleware, reconcilers, workers)
/// writes audit events through. Implementations MUST be fail-closed for
/// [`AuditStream::AdminActivity`]: an admin event that cannot be durably
/// accepted fails the surrounding operation.
pub trait AuditEventSink: Send + Sync {
    /// Emit one validated event.
    ///
    /// # Errors
    /// [`AuditEventError::SinkRejected`] when the event cannot be accepted
    /// durably; callers on the admin stream must propagate the failure.
    fn emit(&self, event: &AuditCloudEvent) -> Result<(), AuditEventError>;
}

// ---------------------------------------------------------------------------
// Signed digest chain (CloudTrail digest-file lineage)
// ---------------------------------------------------------------------------

/// Content-digest port. Implementations return self-describing digests
/// (e.g. `sha256:<hex>`); the kernel never assumes an algorithm.
pub trait Digester: Send + Sync {
    /// Algorithm label (e.g. "sha256").
    fn algorithm(&self) -> &'static str;
    /// Self-describing digest of `bytes` (e.g. `sha256:<lowercase hex>`).
    fn digest_hex(&self, bytes: &[u8]) -> String;
}

/// Signing port, shaped for the owned KMS destination: signing happens
/// wherever the key custody lives; this kernel only sees opaque hex.
pub trait ChainSigner: Send + Sync {
    /// Stable key identifier recorded in each link for verification.
    fn key_id(&self) -> &str;
    /// Sign `message`, returning lowercase hex signature bytes.
    ///
    /// # Errors
    /// [`DigestChainError::SigningFailed`] when the key cannot sign.
    fn sign_hex(&self, message: &[u8]) -> Result<String, DigestChainError>;
}

/// Verification port over a key registry (key_id → public key).
pub trait ChainVerifier: Send + Sync {
    /// Verify `signature_hex` over `message` for `key_id`.
    ///
    /// # Errors
    /// [`DigestChainError::SignatureInvalid`] or
    /// [`DigestChainError::UnknownKeyId`].
    fn verify(
        &self,
        key_id: &str,
        message: &[u8],
        signature_hex: &str,
    ) -> Result<(), DigestChainError>;
}

/// One sealed link of the audit digest chain. Append-only; a link never
/// mutates after sealing.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DigestChainLink {
    /// Monotonic position, starting at 0 and incrementing by exactly 1.
    pub sequence: u64, // data_class: AUDIT
    /// Digest of the PREVIOUS link's canonical bytes + signature, or
    /// [`GENESIS_PREV_LINK_DIGEST`] for sequence 0.
    pub prev_link_digest_hex: String, // data_class: AUDIT
    /// Digest of the canonical bytes of the sealed event batch.
    pub events_digest_hex: String, // data_class: AUDIT
    /// Seal wall-clock (unix seconds) supplied by the caller's clock port.
    pub sealed_at_unix: i64, // data_class: AUDIT
    /// Key that produced `signature_hex`.
    pub key_id: String, // data_class: AUDIT
    /// Signature over [`canonical_signing_bytes`].
    pub signature_hex: String, // data_class: AUDIT
}

/// Deterministic byte string each link's signature covers. Domain-tagged
/// and newline-framed; every field is length-unambiguous in context
/// (digests are self-describing tokens without newlines).
pub fn canonical_signing_bytes(
    sequence: u64,
    prev_link_digest_hex: &str,
    events_digest_hex: &str,
    sealed_at_unix: i64,
    key_id: &str,
) -> Vec<u8> {
    format!(
        "{DIGEST_CHAIN_DOMAIN_TAG}\n{sequence}\n{prev_link_digest_hex}\n{events_digest_hex}\n{sealed_at_unix}\n{key_id}"
    )
    .into_bytes()
}

/// Digest a sealed link (canonical bytes + signature) — the value the NEXT
/// link must carry as `prev_link_digest_hex`. Including the signature means
/// the chain notarizes signatures too (CloudTrail digest-file semantics).
pub fn link_digest_hex(digester: &dyn Digester, link: &DigestChainLink) -> String {
    let mut bytes = canonical_signing_bytes(
        link.sequence,
        &link.prev_link_digest_hex,
        &link.events_digest_hex,
        link.sealed_at_unix,
        &link.key_id,
    );
    bytes.push(b'\n');
    bytes.extend_from_slice(link.signature_hex.as_bytes());
    digester.digest_hex(&bytes)
}

/// Seal the next link over `events_bytes` (the canonical concatenation of
/// the batch's event JSON, caller-framed).
///
/// # Errors
/// Propagates [`DigestChainError::SigningFailed`] from the signer.
pub fn seal_link(
    digester: &dyn Digester,
    signer: &dyn ChainSigner,
    sequence: u64,
    prev_link_digest_hex: &str,
    events_bytes: &[u8],
    sealed_at_unix: i64,
) -> Result<DigestChainLink, DigestChainError> {
    let events_digest_hex = digester.digest_hex(events_bytes);
    let message = canonical_signing_bytes(
        sequence,
        prev_link_digest_hex,
        &events_digest_hex,
        sealed_at_unix,
        signer.key_id(),
    );
    let signature_hex = signer.sign_hex(&message)?;
    Ok(DigestChainLink {
        sequence,
        prev_link_digest_hex: prev_link_digest_hex.to_owned(),
        events_digest_hex,
        sealed_at_unix,
        key_id: signer.key_id().to_owned(),
        signature_hex,
    })
}

/// Verify linkage + signatures across `links`. An empty chain is valid.
/// The first link must carry `expected_genesis_prev` (normally
/// [`GENESIS_PREV_LINK_DIGEST`]) and sequence 0 unless the caller passes a
/// later checkpoint (`first_sequence`, with the checkpoint link's digest
/// as `expected_genesis_prev`).
///
/// # Errors
/// First [`DigestChainError`] encountered, identifying the bad sequence.
pub fn verify_chain(
    digester: &dyn Digester,
    verifier: &dyn ChainVerifier,
    expected_genesis_prev: &str,
    first_sequence: u64,
    links: &[DigestChainLink],
) -> Result<(), DigestChainError> {
    let mut expected_prev = expected_genesis_prev.to_owned();
    let mut expected_sequence = first_sequence;
    for link in links {
        if link.sequence != expected_sequence {
            return Err(DigestChainError::SequenceGap {
                expected: expected_sequence,
                found: link.sequence,
            });
        }
        if link.prev_link_digest_hex != expected_prev {
            return Err(DigestChainError::PrevDigestMismatch {
                sequence: link.sequence,
            });
        }
        let message = canonical_signing_bytes(
            link.sequence,
            &link.prev_link_digest_hex,
            &link.events_digest_hex,
            link.sealed_at_unix,
            &link.key_id,
        );
        verifier
            .verify(&link.key_id, &message, &link.signature_hex)
            .map_err(|e| match e {
                DigestChainError::SignatureInvalid { .. } => DigestChainError::SignatureInvalid {
                    sequence: link.sequence,
                },
                other => other,
            })?;
        expected_prev = link_digest_hex(digester, link);
        expected_sequence = expected_sequence.saturating_add(1);
    }
    Ok(())
}

/// Failure surface for digest-chain sealing and verification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DigestChainError {
    SigningFailed(String),
    UnknownKeyId(String),
    SignatureInvalid { sequence: u64 },
    SequenceGap { expected: u64, found: u64 },
    PrevDigestMismatch { sequence: u64 },
    MalformedSignatureHex,
}

impl fmt::Display for DigestChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SigningFailed(e) => write!(f, "digest-chain signing failed: {e}"),
            Self::UnknownKeyId(k) => write!(f, "digest-chain key_id {k:?} unknown to verifier"),
            Self::SignatureInvalid { sequence } => {
                write!(f, "digest-chain signature invalid at sequence {sequence}")
            }
            Self::SequenceGap { expected, found } => {
                write!(
                    f,
                    "digest-chain sequence gap: expected {expected}, found {found}"
                )
            }
            Self::PrevDigestMismatch { sequence } => {
                write!(
                    f,
                    "digest-chain prev-link digest mismatch at sequence {sequence}"
                )
            }
            Self::MalformedSignatureHex => write!(f, "digest-chain signature is not valid hex"),
        }
    }
}

impl std::error::Error for DigestChainError {}

/// Encode bytes as lowercase hex (kernel-local; avoids a hex dependency).
pub fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use fmt::Write as _;
        // write! to a String cannot fail.
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Decode lowercase/uppercase hex into bytes.
///
/// # Errors
/// [`DigestChainError::MalformedSignatureHex`] on odd length or non-hex.
pub fn decode_hex(hex: &str) -> Result<Vec<u8>, DigestChainError> {
    if !hex.len().is_multiple_of(2) {
        return Err(DigestChainError::MalformedSignatureHex);
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(
                hex.get(i..i + 2)
                    .ok_or(DigestChainError::MalformedSignatureHex)?,
                16,
            )
            .map_err(|_| DigestChainError::MalformedSignatureHex)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(stream: AuditStream) -> AuditLogPayload {
        AuditLogPayload {
            schema_version: AUDIT_PAYLOAD_SCHEMA_VERSION,
            stream,
            service_name: "cloud-tenancy".into(),
            method_name: "oya.cloud.tenancy.v1.CreateTenant".into(),
            resource_name: "tenants/ten_acme".into(),
            tenant_id: Some("ten_acme".into()),
            cell_id: "cell-kr-1".into(),
            authentication_info: AuthenticationInfo {
                principal: "wl_tenancy_cp".into(),
            },
            authorization_info: vec![AuthorizationInfo {
                resource: "tenants/ten_acme".into(),
                permission: "cloud.tenancy.create".into(),
                granted: true,
                policy_version: Some("sha256:abc".into()),
            }],
            request_metadata: RequestMetadata::default(),
            status: AuditStatus::ok(),
        }
    }

    fn event(stream: AuditStream) -> AuditCloudEvent {
        AuditCloudEvent::new(
            "01HMZX0000000000000000XYZ1",
            "//oyatie.com/cloud-tenancy/cell/cell-kr-1",
            "2026-06-10T00:00:00Z",
            payload(stream),
        )
        .expect("well-formed event")
    }

    // -- asymmetric defaults: structural always-on admin stream ------------

    #[test]
    fn admin_stream_must_emit_under_every_constructible_policy() {
        // Exhaustive over the policy state space: both constructors plus
        // Default. No policy value can suppress AdminActivity.
        for policy in [
            EmissionPolicy::admin_only(),
            EmissionPolicy::with_data_access(),
            EmissionPolicy::default(),
        ] {
            assert!(policy.must_emit(AuditStream::AdminActivity));
        }
    }

    #[test]
    fn data_access_is_policy_opt_in() {
        assert!(!EmissionPolicy::admin_only().must_emit(AuditStream::DataAccess));
        assert!(!EmissionPolicy::default().must_emit(AuditStream::DataAccess));
        assert!(EmissionPolicy::with_data_access().must_emit(AuditStream::DataAccess));
    }

    // -- envelope ----------------------------------------------------------

    #[test]
    fn envelope_derives_type_and_subject_from_payload() {
        let admin = event(AuditStream::AdminActivity);
        assert_eq!(admin.event_type, EVENT_TYPE_ADMIN_ACTIVITY);
        assert_eq!(admin.subject, "tenants/ten_acme");
        let data = event(AuditStream::DataAccess);
        assert_eq!(data.event_type, EVENT_TYPE_DATA_ACCESS);
    }

    #[test]
    fn envelope_serde_round_trips_with_cloudevents_field_names() {
        let original = event(AuditStream::AdminActivity);
        let json = serde_json::to_value(&original).unwrap();
        assert_eq!(json["specversion"], "1.0");
        assert_eq!(json["type"], EVENT_TYPE_ADMIN_ACTIVITY);
        assert_eq!(json["datacontenttype"], "application/json");
        assert_eq!(json["data"]["stream"], "admin-activity");
        let back: AuditCloudEvent = serde_json::from_value(json).unwrap();
        back.validate().unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn unknown_envelope_fields_are_rejected() {
        let mut json = serde_json::to_value(event(AuditStream::AdminActivity)).unwrap();
        json["disable_admin_stream"] = serde_json::Value::Bool(true);
        let err = serde_json::from_value::<AuditCloudEvent>(json);
        assert!(err.is_err(), "closed envelope must reject unknown fields");
    }

    #[test]
    fn validation_rejects_empty_required_fields() {
        let mut e = event(AuditStream::AdminActivity);
        e.id = String::new();
        assert_eq!(e.validate(), Err(AuditEventError::EmptyField("id")));

        let mut e = event(AuditStream::AdminActivity);
        e.data.service_name = "  ".into();
        assert_eq!(
            e.validate(),
            Err(AuditEventError::EmptyField("service_name"))
        );

        let mut e = event(AuditStream::AdminActivity);
        e.data.authentication_info.principal = String::new();
        assert_eq!(
            e.validate(),
            Err(AuditEventError::EmptyField("authentication_info.principal"))
        );
    }

    #[test]
    fn validation_rejects_malformed_tenant_and_mislabeled_stream() {
        let mut e = event(AuditStream::AdminActivity);
        e.data.tenant_id = Some("acme".into());
        assert_eq!(
            e.validate(),
            Err(AuditEventError::MalformedTenantId("acme".into()))
        );

        let mut e = event(AuditStream::AdminActivity);
        e.event_type = EVENT_TYPE_DATA_ACCESS.into();
        assert!(matches!(
            e.validate(),
            Err(AuditEventError::TypeStreamMismatch { .. })
        ));
    }

    #[test]
    fn tenantless_platform_admin_event_is_valid() {
        let mut p = payload(AuditStream::AdminActivity);
        p.tenant_id = None;
        p.resource_name = "cells/cell-kr-1".into();
        let e = AuditCloudEvent::new("id-1", "//oyatie.com/cloud-cell", "2026-06-10T00:00:00Z", p);
        assert!(e.is_ok());
    }

    // -- digest chain -------------------------------------------------------

    /// Deterministic test digester: FNV-1a 64 over the bytes. NOT crypto —
    /// linkage-structure tests only; the real algorithm lives in the awslc
    /// adapter crate and is integration-tested there.
    struct FnvDigester;

    impl Digester for FnvDigester {
        fn algorithm(&self) -> &'static str {
            "fnv1a64"
        }
        fn digest_hex(&self, bytes: &[u8]) -> String {
            let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
            for b in bytes {
                hash ^= u64::from(*b);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
            format!("fnv1a64:{hash:016x}")
        }
    }

    /// Test signer/verifier pair: "signature" = digest of (key_id || msg).
    struct FakeSigner {
        key_id: String,
    }

    impl ChainSigner for FakeSigner {
        fn key_id(&self) -> &str {
            &self.key_id
        }
        fn sign_hex(&self, message: &[u8]) -> Result<String, DigestChainError> {
            let mut input = self.key_id.clone().into_bytes();
            input.extend_from_slice(message);
            Ok(FnvDigester.digest_hex(&input))
        }
    }

    struct FakeVerifier;

    impl ChainVerifier for FakeVerifier {
        fn verify(
            &self,
            key_id: &str,
            message: &[u8],
            signature_hex: &str,
        ) -> Result<(), DigestChainError> {
            let expected = FakeSigner {
                key_id: key_id.to_owned(),
            }
            .sign_hex(message)?;
            if expected == signature_hex {
                Ok(())
            } else {
                Err(DigestChainError::SignatureInvalid { sequence: 0 })
            }
        }
    }

    fn sealed_chain(n: u64) -> Vec<DigestChainLink> {
        let digester = FnvDigester;
        let signer = FakeSigner {
            key_id: "key-1".into(),
        };
        let mut links = Vec::new();
        let mut prev = GENESIS_PREV_LINK_DIGEST.to_owned();
        for sequence in 0..n {
            let batch = format!("event-batch-{sequence}");
            let link = seal_link(
                &digester,
                &signer,
                sequence,
                &prev,
                batch.as_bytes(),
                1_780_000_000 + sequence as i64,
            )
            .unwrap();
            prev = link_digest_hex(&digester, &link);
            links.push(link);
        }
        links
    }

    #[test]
    fn sealed_chain_verifies_green() {
        let links = sealed_chain(4);
        verify_chain(
            &FnvDigester,
            &FakeVerifier,
            GENESIS_PREV_LINK_DIGEST,
            0,
            &links,
        )
        .expect("intact chain must verify");
    }

    #[test]
    fn empty_chain_is_valid() {
        verify_chain(
            &FnvDigester,
            &FakeVerifier,
            GENESIS_PREV_LINK_DIGEST,
            0,
            &[],
        )
        .unwrap();
    }

    #[test]
    fn tampered_events_digest_breaks_chain_red() {
        let mut links = sealed_chain(3);
        links[1].events_digest_hex = "fnv1a64:dead".into();
        let err = verify_chain(
            &FnvDigester,
            &FakeVerifier,
            GENESIS_PREV_LINK_DIGEST,
            0,
            &links,
        )
        .expect_err("tampered digest must fail");
        // The forged field breaks the signature first (it is signed), and
        // would break linkage even if re-signed.
        assert_eq!(err, DigestChainError::SignatureInvalid { sequence: 1 });
    }

    #[test]
    fn resigned_tamper_still_breaks_linkage_red() {
        // Adversary with key custody re-signs a forged middle link: the
        // NEXT link's prev digest no longer matches.
        let digester = FnvDigester;
        let signer = FakeSigner {
            key_id: "key-1".into(),
        };
        let mut links = sealed_chain(3);
        links[1] = seal_link(
            &digester,
            &signer,
            1,
            &links[1].prev_link_digest_hex.clone(),
            b"forged-batch",
            links[1].sealed_at_unix,
        )
        .unwrap();
        let err = verify_chain(
            &digester,
            &FakeVerifier,
            GENESIS_PREV_LINK_DIGEST,
            0,
            &links,
        )
        .expect_err("re-signed forgery must still break linkage");
        assert_eq!(err, DigestChainError::PrevDigestMismatch { sequence: 2 });
    }

    #[test]
    fn truncated_or_reordered_chain_is_detected() {
        let links = sealed_chain(3);
        // Drop the middle link.
        let truncated = vec![links[0].clone(), links[2].clone()];
        let err = verify_chain(
            &FnvDigester,
            &FakeVerifier,
            GENESIS_PREV_LINK_DIGEST,
            0,
            &truncated,
        )
        .expect_err("gap must fail");
        assert_eq!(
            err,
            DigestChainError::SequenceGap {
                expected: 1,
                found: 2
            }
        );
    }

    #[test]
    fn checkpoint_resume_verifies_suffix() {
        let links = sealed_chain(4);
        let checkpoint = link_digest_hex(&FnvDigester, &links[1]);
        verify_chain(&FnvDigester, &FakeVerifier, &checkpoint, 2, &links[2..])
            .expect("suffix from checkpoint must verify");
    }

    #[test]
    fn link_serde_round_trips_and_rejects_unknown_fields() {
        let links = sealed_chain(1);
        let json = serde_json::to_value(&links[0]).unwrap();
        let back: DigestChainLink = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(back, links[0]);
        let mut forged = json;
        forged["skip_verification"] = serde_json::Value::Bool(true);
        assert!(serde_json::from_value::<DigestChainLink>(forged).is_err());
    }

    #[test]
    fn hex_round_trip_and_malformed_rejection() {
        let bytes = [0u8, 1, 0xab, 0xff];
        let hex = encode_hex(&bytes);
        assert_eq!(hex, "0001abff");
        assert_eq!(decode_hex(&hex).unwrap(), bytes.to_vec());
        assert_eq!(
            decode_hex("abc"),
            Err(DigestChainError::MalformedSignatureHex)
        );
        assert_eq!(
            decode_hex("zz"),
            Err(DigestChainError::MalformedSignatureHex)
        );
    }
}
