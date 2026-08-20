//! # tenancy-tenant-lifecycle-authz-pdp
//!
//! PDP-backed adapter for the tenancy authorization port (AUTH-005,
//! ADR-0564 D7). Implements [`TenantLifecycleAuthorizer`] by DOGFOODING the
//! repo's own embedded Cedar PDP substrate — `oya-shared-pdp-kernel`'s
//! [`PolicyDecisionPoint`] port realized by `iam-pdp-cedar`'s
//! `CedarPdp` (ADR-0536 D-2; cloud-iam is the IdP/PDP per ADR-0559). The
//! tenancy service is therefore a Policy Enforcement Point over the SAME
//! formally-verified Cedar engine cloud-iam ships, not a parallel authz stack.
//!
//! ## What this adapter owns
//!
//! - The tenancy authorization BUNDLE (embedded Cedar schema + policies):
//!   deny-by-default, a structural cross-tenant forbid, a per-tenant operator
//!   permit, and platform-admin permits for register/list.
//! - The PEP entity-slice assembly: it materializes the caller principal, the
//!   target tenant, and the control-plane singleton from the VERIFIED
//!   [`CallerIdentity`] the facade presents — never from an unverified URL
//!   segment.
//! - The decision mapping: a PDP `Allow` is the port's `Allow`; a PDP `Deny`
//!   AND any PDP error are both the port's deny (fail-closed).
//!
//! ## Layering (ADR-0131 / ADR-0562 faces)
//!
//! adapter → { port, libs PDP kernel/adapter, contracts kernel }, all
//! path-inward. ZERO dependency on the facade. The facade depends on the PORT
//! (not on this adapter type) and is handed an `Arc<dyn TenantLifecycleAuthorizer>`
//! at the composition root.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use iam_pdp_cedar::CedarPdp;
use oya_shared_pdp_kernel::{
    EntityRecord, EntitySlice, PdpError, PolicyBundle, PolicyDecisionPoint,
};
use oya_shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, AuthorizationResponse, Decision, EntityRef, Obligation, PolicyVersion,
};
use oya_shared_ulid_id_kernel::{IdGenerator, IdGeneratorError, Ulid};

use tenancy_tenant_lifecycle_authz_port::{
    AuthorizationDecision, AuthorizationOutcome, AuthorizationQuery, AuthzError,
    TenantLifecycleAction, TenantLifecycleAuthorizer,
};

/// The embedded Cedar schema for the tenancy authz model.
const SCHEMA_SRC: &str = include_str!("../cedar/tenancy.cedarschema");
/// The embedded Cedar policy set (structural forbid + operator/admin permits).
const POLICIES_SRC: &str = include_str!("../cedar/tenancy-policies.cedar");

/// Opaque bundle version for the seed tenancy bundle. A future policy-store
/// delivery fabric overwrites this with a content-addressed token.
const SEED_POLICY_VERSION: &str = "tnpv-000001";

/// Decision-cache capacity for the embedded PDP. Bounded; per-process.
const DECISION_CACHE_CAPACITY: usize = 256;

/// Cedar entity-type names (must match `cedar/tenancy.cedarschema`).
const PRINCIPAL_TYPE: &str = "OyaTenancy::Principal";
const TENANT_TYPE: &str = "OyaTenancy::Tenant";
const CONTROL_PLANE_TYPE: &str = "OyaTenancy::TenancyControlPlane";
/// The singleton control-plane resource id register/list act on.
const CONTROL_PLANE_ID: &str = "tenancy";

/// Cedar action UIDs the action map resolves the port's slugs to.
const ADMINISTER_TENANT_UID: &str = r#"OyaTenancy::Action::"AdministerTenant""#;
const REGISTER_TENANT_UID: &str = r#"OyaTenancy::Action::"RegisterTenant""#;
const LIST_TENANTS_UID: &str = r#"OyaTenancy::Action::"ListTenants""#;

/// Build the tenancy authorization bundle (schema + policies + action map).
/// The action map binds each port action slug to its Cedar action UID; per the
/// embedded-PDP contract an unmapped slug fails closed (`UnknownAction`).
fn tenancy_bundle() -> Result<PolicyBundle, PdpError> {
    let version =
        PolicyVersion::new(SEED_POLICY_VERSION).map_err(|violations| PdpError::BundleRejected {
            detail: format!("seed policy version rejected: {violations:?}"),
        })?;
    let action_map = BTreeMap::from([
        (
            TenantLifecycleAction::Read.slug().to_owned(),
            ADMINISTER_TENANT_UID.to_owned(),
        ),
        (
            TenantLifecycleAction::Provision.slug().to_owned(),
            ADMINISTER_TENANT_UID.to_owned(),
        ),
        (
            TenantLifecycleAction::Suspend.slug().to_owned(),
            ADMINISTER_TENANT_UID.to_owned(),
        ),
        (
            TenantLifecycleAction::Resume.slug().to_owned(),
            ADMINISTER_TENANT_UID.to_owned(),
        ),
        (
            TenantLifecycleAction::Retire.slug().to_owned(),
            ADMINISTER_TENANT_UID.to_owned(),
        ),
        (
            TenantLifecycleAction::Register.slug().to_owned(),
            REGISTER_TENANT_UID.to_owned(),
        ),
        (
            TenantLifecycleAction::List.slug().to_owned(),
            LIST_TENANTS_UID.to_owned(),
        ),
    ]);
    Ok(PolicyBundle {
        version,
        schema_src: SCHEMA_SRC.to_owned(),
        policies_src: POLICIES_SRC.to_owned(),
        tenant_policies: BTreeMap::new(),
        templates: Vec::new(),
        template_links: Vec::new(),
        action_map,
    })
}

fn entity_ref(entity_type: &str, entity_id: &str) -> EntityRef {
    EntityRef {
        entity_type: entity_type.to_owned(),
        entity_id: entity_id.to_owned(),
    }
}

/// The PDP-backed tenancy authorizer. Holds the embedded Cedar PDP over the
/// tenancy bundle; one instance per process, shared behind an `Arc` by the
/// facade.
pub struct PdpTenantLifecycleAuthorizer {
    pdp: CedarPdp,
}

impl PdpTenantLifecycleAuthorizer {
    /// Compile and strict-validate the embedded tenancy bundle, then serve from
    /// it. This is the production constructor the composition root calls; a
    /// bundle-compile failure is a HARD error so the binary can REFUSE to boot
    /// rather than serve without authz (no default-allow fallback, ever).
    ///
    /// # Errors
    /// [`PdpError`] when the embedded bundle fails to compile, link, or
    /// strict-validate — the caller MUST refuse to serve.
    pub fn from_seed_bundle() -> Result<Self, PdpError> {
        Self::from_bundle(&tenancy_bundle()?, Arc::new(SystemUlidIdGenerator::new()))
    }

    /// Build over an explicit bundle + id generator (the seam tests and a
    /// future policy-store delivery fabric use).
    ///
    /// # Errors
    /// [`PdpError`] when `bundle` fails to compile/link/strict-validate.
    pub fn from_bundle(
        bundle: &PolicyBundle,
        id_gen: Arc<dyn IdGenerator>,
    ) -> Result<Self, PdpError> {
        let pdp = CedarPdp::load(bundle, id_gen, DECISION_CACHE_CAPACITY)?;
        Ok(Self { pdp })
    }

    /// The loaded bundle's version token (for readiness/observability).
    #[must_use]
    pub fn loaded_policy_version(&self) -> PolicyVersion {
        self.pdp.loaded_policy_version()
    }
}

/// Closed JSON body accepted by the cloud PDP REST decision surface.
#[derive(Debug, serde::Serialize)]
struct NetworkAuthorizeBody<'a> {
    request: &'a AuthorizationRequest,
    entities: &'a [EntityRecord],
}

/// A synchronous network PDP transport behind the tenancy authorizer port.
///
/// The tenancy facade's [`TenantLifecycleAuthorizer`] port is synchronous today,
/// so this trait is deliberately synchronous too. Transport failures are
/// fail-closed [`AuthzError`] values, never fallback triggers.
pub trait PdpDecisionTransport: Send + Sync {
    /// Decide one PARC request with its PEP-assembled entity slice.
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<AuthorizationResponse, AuthzError>;
}

/// Blocking HTTP transport for the cloud PDP REST `/v1/authorize` contract.
#[derive(Clone)]
pub struct ReqwestPdpDecisionTransport {
    endpoint: String,
    client: reqwest::blocking::Client,
}

impl ReqwestPdpDecisionTransport {
    /// Build a transport from either a service base URL or a full
    /// `/v1/authorize` endpoint. HTTPS is required except for loopback HTTP
    /// used by hermetic tests and local developer wiring.
    ///
    /// # Errors
    /// [`NetworkPdpEndpointError`] when the endpoint is empty, malformed, or
    /// would send PDP decisions over non-loopback plaintext HTTP.
    pub fn new(endpoint_or_base_url: &str) -> Result<Self, NetworkPdpEndpointError> {
        let endpoint = normalize_network_pdp_authorize_endpoint(endpoint_or_base_url)?;
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|error| NetworkPdpEndpointError::Client(error.to_string()))?;
        Ok(Self { endpoint, client })
    }

    /// Build a transport and verify the configured PDP is ready before the
    /// service advertises health.
    ///
    /// # Errors
    /// [`NetworkPdpEndpointError`] when endpoint validation/client construction
    /// fails or `/readyz` is unavailable/non-2xx.
    pub fn new_with_readiness_preflight(
        endpoint_or_base_url: &str,
    ) -> Result<Self, NetworkPdpEndpointError> {
        let transport = Self::new(endpoint_or_base_url)?;
        transport.preflight_ready()?;
        Ok(transport)
    }

    /// The normalized authorize endpoint this transport calls.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// The normalized readiness endpoint this transport preflights.
    ///
    /// # Errors
    /// [`NetworkPdpEndpointError`] if the already-normalized authorize endpoint
    /// cannot be parsed as a URL.
    pub fn readiness_endpoint(&self) -> Result<String, NetworkPdpEndpointError> {
        network_pdp_readyz_endpoint_from_authorize_endpoint(&self.endpoint)
    }

    /// Confirm the configured PDP is reachable and ready. This is intentionally
    /// a boot-time check: a configured network PDP must fail closed at boot, not
    /// lazily time out protected requests after the service reports healthy.
    ///
    /// # Errors
    /// [`NetworkPdpEndpointError::Readiness`] on request failure or non-2xx.
    pub fn preflight_ready(&self) -> Result<(), NetworkPdpEndpointError> {
        let readyz = self.readiness_endpoint()?;
        let response = self.client.get(&readyz).send().map_err(|error| {
            NetworkPdpEndpointError::Readiness(format!("readyz request failed: {error}"))
        })?;
        let status = response.status();
        if !status.is_success() {
            let detail = response
                .text()
                .unwrap_or_else(|error| format!("response body unavailable: {error}"));
            return Err(NetworkPdpEndpointError::Readiness(format!(
                "readyz returned status {status}: {detail}"
            )));
        }
        Ok(())
    }
}

impl PdpDecisionTransport for ReqwestPdpDecisionTransport {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<AuthorizationResponse, AuthzError> {
        let body = NetworkAuthorizeBody {
            request,
            entities: &entities.entities,
        };
        let response = self
            .client
            .post(&self.endpoint)
            .json(&body)
            .send()
            .map_err(|error| {
                AuthzError::EngineRefused(format!("network PDP request failed: {error}"))
            })?;
        let status = response.status();
        if !status.is_success() {
            let detail = response
                .text()
                .unwrap_or_else(|error| format!("response body unavailable: {error}"));
            return Err(AuthzError::EngineRefused(format!(
                "network PDP refused with status {status}: {detail}"
            )));
        }
        response.json::<AuthorizationResponse>().map_err(|error| {
            AuthzError::EngineRefused(format!("network PDP response decode failed: {error}"))
        })
    }
}

/// Endpoint validation failure for [`ReqwestPdpDecisionTransport`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkPdpEndpointError {
    /// The configured endpoint is empty after trimming whitespace.
    Empty,
    /// The endpoint is not an absolute URL.
    InvalidUrl(String),
    /// Only HTTPS, plus loopback HTTP for tests/local wiring, is accepted.
    UnsupportedScheme(String),
    /// Plain HTTP is allowed only for loopback hosts.
    PlainHttpNonLoopback(String),
    /// The blocking HTTP client could not be constructed.
    Client(String),
    /// The configured PDP failed its boot-time readiness preflight.
    Readiness(String),
}

impl fmt::Display for NetworkPdpEndpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "network PDP endpoint is empty"),
            Self::InvalidUrl(error) => write!(f, "network PDP endpoint is invalid: {error}"),
            Self::UnsupportedScheme(scheme) => {
                write!(f, "network PDP endpoint uses unsupported scheme {scheme:?}")
            }
            Self::PlainHttpNonLoopback(host) => write!(
                f,
                "network PDP endpoint uses plaintext HTTP for non-loopback host {host:?}"
            ),
            Self::Client(error) => write!(f, "network PDP HTTP client build failed: {error}"),
            Self::Readiness(error) => write!(f, "network PDP readiness preflight failed: {error}"),
        }
    }
}

impl std::error::Error for NetworkPdpEndpointError {}

/// Normalize a service base URL or full authorize URL into `/v1/authorize`.
///
/// HTTPS is required for non-loopback hosts; `http://127.0.0.1`, `localhost`,
/// and `::1` are accepted for hermetic tests and local dev only.
pub fn normalize_network_pdp_authorize_endpoint(
    endpoint_or_base_url: &str,
) -> Result<String, NetworkPdpEndpointError> {
    let trimmed = endpoint_or_base_url.trim();
    if trimmed.is_empty() {
        return Err(NetworkPdpEndpointError::Empty);
    }
    let endpoint = if trimmed.ends_with("/v1/authorize") {
        trimmed.to_owned()
    } else {
        format!("{}/v1/authorize", trimmed.trim_end_matches('/'))
    };
    let parsed = reqwest::Url::parse(&endpoint)
        .map_err(|error| NetworkPdpEndpointError::InvalidUrl(error.to_string()))?;
    match parsed.scheme() {
        "https" => Ok(endpoint),
        "http" => {
            let host = parsed.host_str().unwrap_or_default();
            if is_loopback_host(host) {
                Ok(endpoint)
            } else {
                Err(NetworkPdpEndpointError::PlainHttpNonLoopback(
                    host.to_owned(),
                ))
            }
        }
        scheme => Err(NetworkPdpEndpointError::UnsupportedScheme(
            scheme.to_owned(),
        )),
    }
}

/// Derive the REST readiness endpoint corresponding to an authorize endpoint.
///
/// `https://host/v1/authorize` becomes `https://host/readyz`; if the service is
/// mounted under a prefix, `https://host/prefix/v1/authorize` becomes
/// `https://host/prefix/readyz`.
pub fn network_pdp_readyz_endpoint_from_authorize_endpoint(
    authorize_endpoint: &str,
) -> Result<String, NetworkPdpEndpointError> {
    let mut parsed = reqwest::Url::parse(authorize_endpoint)
        .map_err(|error| NetworkPdpEndpointError::InvalidUrl(error.to_string()))?;
    let path = parsed.path();
    let prefix = path.strip_suffix("/v1/authorize").unwrap_or("");
    let ready_path = if prefix.is_empty() {
        "/readyz".to_owned()
    } else {
        format!("{}/readyz", prefix.trim_end_matches('/'))
    };
    parsed.set_path(&ready_path);
    parsed.set_query(None);
    parsed.set_fragment(None);
    Ok(parsed.to_string())
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "::1" | "[::1]")
}

/// Network-backed tenancy authorizer that dogfoods the cloud PDP REST decision
/// surface while preserving the unchanged [`TenantLifecycleAuthorizer`] port.
pub struct NetworkPdpTenantLifecycleAuthorizer<T = ReqwestPdpDecisionTransport> {
    transport: T,
}

impl NetworkPdpTenantLifecycleAuthorizer<ReqwestPdpDecisionTransport> {
    /// Build the production network authorizer from a configured cloud-PDP
    /// endpoint or base URL.
    ///
    /// # Errors
    /// [`NetworkPdpEndpointError`] when endpoint validation/client construction
    /// fails. Callers MUST refuse to boot rather than falling back to embedded
    /// PDP when a network PDP endpoint was configured.
    pub fn from_endpoint(endpoint_or_base_url: &str) -> Result<Self, NetworkPdpEndpointError> {
        Ok(Self {
            transport: ReqwestPdpDecisionTransport::new(endpoint_or_base_url)?,
        })
    }

    /// Build the production network authorizer and preflight `/readyz`.
    ///
    /// # Errors
    /// [`NetworkPdpEndpointError`] when endpoint validation/client construction
    /// fails or the configured PDP is not ready. Callers MUST refuse boot rather
    /// than falling back to embedded PDP.
    pub fn from_endpoint_with_readiness_preflight(
        endpoint_or_base_url: &str,
    ) -> Result<Self, NetworkPdpEndpointError> {
        Ok(Self {
            transport: ReqwestPdpDecisionTransport::new_with_readiness_preflight(
                endpoint_or_base_url,
            )?,
        })
    }
}

impl<T> NetworkPdpTenantLifecycleAuthorizer<T>
where
    T: PdpDecisionTransport,
{
    /// Build over an explicit transport (tests and future mTLS transports).
    #[must_use]
    pub fn from_transport(transport: T) -> Self {
        Self { transport }
    }

    /// The wrapped transport.
    #[must_use]
    pub fn transport(&self) -> &T {
        &self.transport
    }
}

impl<T> TenantLifecycleAuthorizer for NetworkPdpTenantLifecycleAuthorizer<T>
where
    T: PdpDecisionTransport,
{
    fn authorize(
        &self,
        query: &AuthorizationQuery<'_>,
    ) -> Result<AuthorizationOutcome, AuthzError> {
        let (request, entities) = build_decision_input(query)?;
        let response = self.transport.authorize(&request, &entities)?;
        response_to_authorization_outcome(&request, response)
    }
}

/// Assemble the PARC request + PEP entity slice for one query. The verified
/// caller's attributes are materialized HERE; the target tenant id only ever
/// appears as a resource the caller is checked against — it never authorizes by
/// itself.
fn build_decision_input(
    query: &AuthorizationQuery<'_>,
) -> Result<(AuthorizationRequest, EntitySlice), AuthzError> {
    let caller = query.caller;
    let action_slug = query.action.slug();

    // Principal entity: proven tenant scope + platform-admin axis, both from the
    // VERIFIED credential. tenant_id is set only when the caller is tenant-scoped.
    let mut principal_attrs: BTreeMap<String, serde_json::Value> = BTreeMap::from([(
        "platform_admin".to_owned(),
        serde_json::Value::Bool(caller.platform_admin),
    )]);
    if let Some(scope) = &caller.tenant_scope {
        principal_attrs.insert("tenant_id".to_owned(), serde_json::json!(scope));
    }
    let principal = entity_ref(PRINCIPAL_TYPE, &caller.principal_id);
    let principal_record = EntityRecord {
        uid: principal.clone(),
        attributes: principal_attrs,
        parents: Vec::new(),
    };

    // The request tenant axis the locked PDP contract carries: the caller's
    // proven scope for a per-tenant op, else the control-plane sentinel for a
    // platform op. The bearer alone NEVER sets this from the URL.
    let request_tenant = match query.action.is_platform_scoped() {
        true => CONTROL_PLANE_ID.to_owned(),
        false => caller.tenant_scope.clone().unwrap_or_default(),
    };

    let (resource_record, request_tenant_for_contract) = if query.action.is_platform_scoped() {
        let resource = entity_ref(CONTROL_PLANE_TYPE, CONTROL_PLANE_ID);
        let record = EntityRecord {
            uid: resource.clone(),
            attributes: BTreeMap::from([(
                "control_plane".to_owned(),
                serde_json::json!(CONTROL_PLANE_ID),
            )]),
            parents: Vec::new(),
        };
        (record, request_tenant)
    } else {
        let target = query.target_tenant_id.ok_or_else(|| {
            AuthzError::InvalidQuery(format!(
                "per-tenant action {action_slug} requires a target tenant id"
            ))
        })?;
        let resource = entity_ref(TENANT_TYPE, target);
        let record = EntityRecord {
            uid: resource.clone(),
            attributes: BTreeMap::from([("tenant_id".to_owned(), serde_json::json!(target))]),
            parents: Vec::new(),
        };
        // For a per-tenant op, the contract tenant axis is the caller's proven
        // scope (a tenant-scoped caller); a platform-only caller has none and
        // falls through to deny-by-default on this op.
        (record, request_tenant)
    };

    let resource = resource_record.uid.clone();
    let entities = EntitySlice {
        entities: vec![principal_record, resource_record],
    };

    let request = AuthorizationRequest {
        // A fresh opaque correlation id per decision; the audit chain keys on
        // the engine-minted decision id, this is only the request echo.
        request_id: format!("tenancy-authz-{action_slug}"),
        // tenant_id must be a slug; an empty axis (platform-only caller on a
        // per-tenant op) cannot be a valid slug, so we map that to a clean
        // InvalidQuery → deny rather than letting the contract reject it.
        tenant_id: nonempty_tenant_axis(&request_tenant_for_contract, action_slug)?,
        principal,
        action: action_slug.to_owned(),
        resource,
        context: BTreeMap::new(),
        min_policy_version: None,
    };
    Ok((request, entities))
}

fn ensure_no_unenforced_obligations(obligations: &[Obligation]) -> Result<(), AuthzError> {
    if obligations.is_empty() {
        return Ok(());
    }
    let ids = obligations
        .iter()
        .map(|obligation| obligation.obligation_id.as_str())
        .collect::<Vec<_>>()
        .join(",");
    Err(AuthzError::EngineRefused(format!(
        "PDP returned obligations this PEP cannot enforce: {ids}"
    )))
}

fn response_to_authorization_outcome(
    request: &AuthorizationRequest,
    response: AuthorizationResponse,
) -> Result<AuthorizationOutcome, AuthzError> {
    if response.request_id != request.request_id {
        return Err(AuthzError::EngineRefused(format!(
            "network PDP response request_id mismatch: expected {}, got {}",
            request.request_id, response.request_id
        )));
    }
    response.validate().map_err(|violations| {
        AuthzError::EngineRefused(format!(
            "network PDP response violated contract: {violations:?}"
        ))
    })?;
    ensure_no_unenforced_obligations(&response.obligations)?;
    let decision = match response.decision {
        Decision::Allow => AuthorizationDecision::Allow,
        Decision::Deny => AuthorizationDecision::Deny,
    };
    Ok(AuthorizationOutcome {
        decision,
        decision_id: response.decision_id,
        determining_policy_ids: response.determining_policy_ids,
    })
}

/// Assemble the PARC request + PEP entity slice for one query, then ask the
/// embedded PDP.
fn decide(
    pdp: &CedarPdp,
    query: &AuthorizationQuery<'_>,
) -> Result<AuthorizationOutcome, AuthzError> {
    let (request, entities) = build_decision_input(query)?;

    match pdp.authorize(&request, &entities) {
        Ok(outcome) => {
            let decision = match outcome.response.decision {
                Decision::Allow => AuthorizationDecision::Allow,
                Decision::Deny => AuthorizationDecision::Deny,
            };
            ensure_no_unenforced_obligations(&outcome.response.obligations)?;
            // Preserve the full audit record — NEVER discard it. The PEP
            // emits a structured tracing event from these fields (AC-W-13).
            Ok(AuthorizationOutcome {
                decision,
                decision_id: outcome.audit.decision_id,
                determining_policy_ids: outcome.audit.determining_policy_ids,
            })
        }
        // Fail-closed: ANY engine refusal is a deny, surfaced as an error the
        // PEP enforces as a deny (never a bypass).
        Err(error) => Err(AuthzError::EngineRefused(error.to_string())),
    }
}

/// The locked PDP contract requires a non-empty slug tenant axis. A
/// platform-only caller attempting a per-tenant op has no proven tenant scope;
/// that is an unauthorized request, mapped to a clean deny (InvalidQuery)
/// rather than a contract-violation refusal.
fn nonempty_tenant_axis(value: &str, action_slug: &str) -> Result<String, AuthzError> {
    if value.is_empty() {
        return Err(AuthzError::InvalidQuery(format!(
            "caller has no proven tenant scope for per-tenant action {action_slug}"
        )));
    }
    Ok(value.to_owned())
}

impl TenantLifecycleAuthorizer for PdpTenantLifecycleAuthorizer {
    fn authorize(
        &self,
        query: &AuthorizationQuery<'_>,
    ) -> Result<AuthorizationOutcome, AuthzError> {
        decide(&self.pdp, query)
    }
}

/// Crockford-base32 alphabet (ULID spec: no I, L, O, U).
const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Wall-clock + CSPRNG [`IdGenerator`] for the embedded PDP's decision-id
/// minting. Mirrors the ADR-0506 blessed-crypto precedent set by the cloud-iam
/// PDP app (48-bit ms timestamp + 80 bits of aws-lc-rs entropy,
/// Crockford-base32). An entropy/clock failure yields an error, never a
/// degraded id — the PDP then refuses with `DecisionIdUnavailable` (fail-closed).
#[derive(Debug, Default)]
pub struct SystemUlidIdGenerator {
    rng: aws_lc_rs::rand::SystemRandom,
}

impl SystemUlidIdGenerator {
    /// Build a generator over the process-global CSPRNG.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rng: aws_lc_rs::rand::SystemRandom::new(),
        }
    }
}

/// Encode the 128-bit ULID value as 26 Crockford-base32 characters (MSB-first;
/// the leading character carries only 3 significant bits, so a 48-bit-masked
/// timestamp always satisfies the spec's `0..=7` constraint).
fn encode_ulid(value: u128) -> String {
    let mut out = String::with_capacity(26);
    for i in 0..26 {
        let shift = 5 * (25 - i);
        let digit = ((value >> shift) & 0x1F) as usize;
        // digit < 32 by construction (5-bit mask); indexing is total.
        out.push(char::from(CROCKFORD[digit]));
    }
    out
}

impl IdGenerator for SystemUlidIdGenerator {
    fn new_ulid(&self) -> Result<Ulid, IdGeneratorError> {
        use aws_lc_rs::rand::SecureRandom as _;
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0); // pre-epoch clock: encode as the epoch, never panic
        let timestamp = millis & ((1u128 << 48) - 1);
        let mut entropy = [0u8; 10];
        self.rng.fill(&mut entropy).map_err(|_| {
            // The kernel error enum has no entropy variant; name the real
            // failure so the PDP surfaces it as DecisionIdUnavailable.
            IdGeneratorError::MalformedUlid("csprng entropy unavailable".to_owned())
        })?;
        let mut entropy_bits: u128 = 0;
        for byte in entropy {
            entropy_bits = (entropy_bits << 8) | u128::from(byte);
        }
        let value = (timestamp << 80) | entropy_bits;
        Ulid::try_new(encode_ulid(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_shared_ulid_id_kernel::SeededIdGenerator;
    use tenancy_tenant_lifecycle_authz_port::CallerIdentity;

    fn authorizer() -> PdpTenantLifecycleAuthorizer {
        PdpTenantLifecycleAuthorizer::from_bundle(
            &tenancy_bundle().expect("seed bundle builds"),
            Arc::new(SeededIdGenerator::default()),
        )
        .expect("seed bundle must compile and strict-validate")
    }

    fn tenant_caller(principal: &str, tenant: &str) -> CallerIdentity {
        CallerIdentity {
            principal_id: principal.to_owned(),
            tenant_scope: Some(tenant.to_owned()),
            platform_admin: false,
        }
    }

    fn platform_admin(principal: &str) -> CallerIdentity {
        CallerIdentity {
            principal_id: principal.to_owned(),
            tenant_scope: None,
            platform_admin: true,
        }
    }

    fn decide_action(
        az: &PdpTenantLifecycleAuthorizer,
        caller: &CallerIdentity,
        action: TenantLifecycleAction,
        target: Option<&str>,
    ) -> Result<AuthorizationOutcome, AuthzError> {
        az.authorize(&AuthorizationQuery {
            caller,
            action,
            target_tenant_id: target,
        })
    }

    #[test]
    fn seed_bundle_compiles_and_strict_validates() {
        // from_bundle returning Ok proves the embedded schema + policies pass
        // the Cedar strict validator (the fail-closed boot precondition).
        let _ = authorizer();
    }

    #[test]
    fn tenant_operator_administers_own_tenant() {
        let az = authorizer();
        let caller = tenant_caller("acme-operator", "acme");
        for action in [
            TenantLifecycleAction::Read,
            TenantLifecycleAction::Provision,
            TenantLifecycleAction::Suspend,
            TenantLifecycleAction::Resume,
            TenantLifecycleAction::Retire,
        ] {
            let outcome = decide_action(&az, &caller, action, Some("acme")).unwrap();
            assert_eq!(
                outcome.decision,
                AuthorizationDecision::Allow,
                "operator must administer own tenant for {action:?}",
            );
        }
    }

    #[test]
    fn tenant_operator_denied_cross_tenant() {
        let az = authorizer();
        let caller = tenant_caller("acme-operator", "acme");
        // Acting on a DIFFERENT tenant (globex): the structural forbid denies.
        for action in [
            TenantLifecycleAction::Read,
            TenantLifecycleAction::Suspend,
            TenantLifecycleAction::Retire,
        ] {
            let outcome = decide_action(&az, &caller, action, Some("globex")).unwrap();
            assert_eq!(
                outcome.decision,
                AuthorizationDecision::Deny,
                "cross-tenant {action:?} must be denied",
            );
        }
    }

    #[test]
    fn tenant_operator_cannot_register_or_list() {
        let az = authorizer();
        let caller = tenant_caller("acme-operator", "acme");
        assert_eq!(
            decide_action(&az, &caller, TenantLifecycleAction::Register, None)
                .unwrap()
                .decision,
            AuthorizationDecision::Deny,
        );
        assert_eq!(
            decide_action(&az, &caller, TenantLifecycleAction::List, None)
                .unwrap()
                .decision,
            AuthorizationDecision::Deny,
        );
    }

    #[test]
    fn platform_admin_registers_and_lists() {
        let az = authorizer();
        let caller = platform_admin("platform-admin");
        assert_eq!(
            decide_action(&az, &caller, TenantLifecycleAction::Register, None)
                .unwrap()
                .decision,
            AuthorizationDecision::Allow,
        );
        assert_eq!(
            decide_action(&az, &caller, TenantLifecycleAction::List, None)
                .unwrap()
                .decision,
            AuthorizationDecision::Allow,
        );
    }

    #[test]
    fn platform_admin_denied_per_tenant_ops() {
        // A platform admin has no proven tenant scope; per-tenant ops fall
        // through to deny-by-default (mapped to a clean InvalidQuery deny).
        let az = authorizer();
        let caller = platform_admin("platform-admin");
        let result = decide_action(&az, &caller, TenantLifecycleAction::Retire, Some("acme"));
        assert!(
            matches!(result, Err(AuthzError::InvalidQuery(_))),
            "platform admin with no tenant scope must be denied per-tenant ops, got {result:?}",
        );
    }

    #[test]
    fn per_tenant_action_without_target_is_invalid_query() {
        let az = authorizer();
        let caller = tenant_caller("acme-operator", "acme");
        let result = decide_action(&az, &caller, TenantLifecycleAction::Suspend, None);
        assert!(matches!(result, Err(AuthzError::InvalidQuery(_))));
    }

    #[test]
    fn unscoped_caller_denied_everywhere() {
        // A caller with neither tenant scope nor platform-admin (a bearer with
        // no axis) is denied on every surface — bearer alone grants nothing.
        let az = authorizer();
        let caller = CallerIdentity {
            principal_id: "bearer-only".to_owned(),
            tenant_scope: None,
            platform_admin: false,
        };
        assert_eq!(
            decide_action(&az, &caller, TenantLifecycleAction::Register, None)
                .unwrap()
                .decision,
            AuthorizationDecision::Deny,
        );
        assert!(matches!(
            decide_action(&az, &caller, TenantLifecycleAction::Read, Some("acme")),
            Err(AuthzError::InvalidQuery(_)),
        ));
    }

    // ── Audit record tests (AC-W-13) ──────────────────────────────────────────

    #[test]
    fn allow_outcome_carries_non_empty_decision_id_and_policy_ids() {
        // Every ALLOW decision must produce a non-empty decision_id (the PDP-minted
        // ULID) and at least one determining_policy_id (the permit that fired).
        let az = authorizer();
        let caller = tenant_caller("acme-operator", "acme");
        let outcome =
            decide_action(&az, &caller, TenantLifecycleAction::Read, Some("acme")).unwrap();
        assert_eq!(outcome.decision, AuthorizationDecision::Allow);
        assert!(
            !outcome.decision_id.is_empty(),
            "allow decision_id must be non-empty (AC-W-13)",
        );
        assert!(
            !outcome.determining_policy_ids.is_empty(),
            "allow must name the determining policy ids, got empty vec",
        );
    }

    #[test]
    fn deny_outcome_carries_non_empty_decision_id() {
        // Every DENY decision — including a structural forbid — must produce a
        // non-empty decision_id so the deny is attributable in the audit trail.
        // determining_policy_ids may or may not be populated (deny-by-default has
        // none; a forbid names the forbid policy id).
        let az = authorizer();
        let caller = tenant_caller("acme-operator", "acme");
        // Cross-tenant deny: driven by the structural-tenant-isolation forbid.
        let outcome =
            decide_action(&az, &caller, TenantLifecycleAction::Read, Some("globex")).unwrap();
        assert_eq!(outcome.decision, AuthorizationDecision::Deny);
        assert!(
            !outcome.decision_id.is_empty(),
            "deny decision_id must be non-empty (AC-W-13)",
        );
    }

    #[test]
    fn system_ulid_generator_emits_valid_distinct_ulids() {
        let id_gen = SystemUlidIdGenerator::new();
        let a = id_gen.new_ulid().expect("a");
        let b = id_gen.new_ulid().expect("b");
        assert_eq!(a.as_str().len(), 26);
        assert_ne!(a, b);
    }
    struct EchoAllowTransport;

    impl PdpDecisionTransport for EchoAllowTransport {
        fn authorize(
            &self,
            request: &AuthorizationRequest,
            entities: &EntitySlice,
        ) -> Result<AuthorizationResponse, AuthzError> {
            assert_eq!(request.tenant_id, "acme");
            assert_eq!(request.action, TenantLifecycleAction::Read.slug());
            assert_eq!(request.resource.entity_id, "acme");
            assert_eq!(entities.entities.len(), 2);
            Ok(AuthorizationResponse {
                decision_id: "dec-network-allow".to_owned(),
                request_id: request.request_id.clone(),
                decision: Decision::Allow,
                policy_version: PolicyVersion::new("tnpv-network-1").expect("policy version"),
                determining_policy_ids: vec!["permit-tenant-operator-administer-tenant".to_owned()],
                obligations: Vec::new(),
            })
        }
    }

    struct MismatchedRequestIdTransport;

    impl PdpDecisionTransport for MismatchedRequestIdTransport {
        fn authorize(
            &self,
            _request: &AuthorizationRequest,
            _entities: &EntitySlice,
        ) -> Result<AuthorizationResponse, AuthzError> {
            Ok(AuthorizationResponse {
                decision_id: "dec-network-mismatch".to_owned(),
                request_id: "different-request".to_owned(),
                decision: Decision::Deny,
                policy_version: PolicyVersion::new("tnpv-network-1").expect("policy version"),
                determining_policy_ids: Vec::new(),
                obligations: Vec::new(),
            })
        }
    }

    struct ObligationTransport;

    impl PdpDecisionTransport for ObligationTransport {
        fn authorize(
            &self,
            request: &AuthorizationRequest,
            _entities: &EntitySlice,
        ) -> Result<AuthorizationResponse, AuthzError> {
            Ok(AuthorizationResponse {
                decision_id: "dec-network-obligation".to_owned(),
                request_id: request.request_id.clone(),
                decision: Decision::Allow,
                policy_version: PolicyVersion::new("tnpv-network-1").expect("policy version"),
                determining_policy_ids: vec!["permit-with-obligation".to_owned()],
                obligations: vec![Obligation {
                    obligation_id: "emit-audit-event".to_owned(),
                    parameters: BTreeMap::new(),
                }],
            })
        }
    }

    #[test]
    fn network_authorizer_preserves_decision_id_and_policy_ids() {
        let az = NetworkPdpTenantLifecycleAuthorizer::from_transport(EchoAllowTransport);
        let caller = tenant_caller("acme-operator", "acme");

        let outcome = az
            .authorize(&AuthorizationQuery {
                caller: &caller,
                action: TenantLifecycleAction::Read,
                target_tenant_id: Some("acme"),
            })
            .expect("network PDP decision");

        assert_eq!(outcome.decision, AuthorizationDecision::Allow);
        assert_eq!(outcome.decision_id, "dec-network-allow");
        assert_eq!(
            outcome.determining_policy_ids,
            vec!["permit-tenant-operator-administer-tenant"]
        );
    }

    #[test]
    fn network_authorizer_fails_closed_on_protocol_mismatch() {
        let az = NetworkPdpTenantLifecycleAuthorizer::from_transport(MismatchedRequestIdTransport);
        let caller = tenant_caller("acme-operator", "acme");

        let err = az
            .authorize(&AuthorizationQuery {
                caller: &caller,
                action: TenantLifecycleAction::Read,
                target_tenant_id: Some("acme"),
            })
            .expect_err("mismatched network response must fail closed");

        assert!(matches!(err, AuthzError::EngineRefused(_)));
    }

    #[test]
    fn network_authorizer_fails_closed_on_unenforced_obligation() {
        let az = NetworkPdpTenantLifecycleAuthorizer::from_transport(ObligationTransport);
        let caller = tenant_caller("acme-operator", "acme");

        let err = az
            .authorize(&AuthorizationQuery {
                caller: &caller,
                action: TenantLifecycleAction::Read,
                target_tenant_id: Some("acme"),
            })
            .expect_err("unenforced obligations must fail closed");

        assert!(matches!(err, AuthzError::EngineRefused(_)));
    }

    #[test]
    fn network_endpoint_normalization_requires_https_except_loopback() {
        assert_eq!(
            normalize_network_pdp_authorize_endpoint("https://pdp.internal").unwrap(),
            "https://pdp.internal/v1/authorize"
        );
        assert_eq!(
            normalize_network_pdp_authorize_endpoint("http://127.0.0.1:8181/v1/authorize").unwrap(),
            "http://127.0.0.1:8181/v1/authorize"
        );
        assert_eq!(
            network_pdp_readyz_endpoint_from_authorize_endpoint(
                "https://pdp.internal/prefix/v1/authorize",
            )
            .unwrap(),
            "https://pdp.internal/prefix/readyz"
        );
        assert!(matches!(
            normalize_network_pdp_authorize_endpoint("http://pdp.internal"),
            Err(NetworkPdpEndpointError::PlainHttpNonLoopback(host)) if host == "pdp.internal"
        ));
    }
}
