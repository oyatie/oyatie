//! Workload X.509-SVID kernel — PURE (no I/O, no clock, no crypto).
//!
//! This crate is the canonical value-and-port core for SPIFFE-style X.509-SVID
//! caller authentication (G002 slice-1; ADR-0561). It gives the cloud-iam Cedar
//! PDP a way to derive the *authorized* tenant of a caller from a
//! cryptographically-verified peer SVID, closing the gap where the PDP trusts a
//! caller-body `tenant_id` verbatim.
//!
//! ## Trust-domain naming — CELL-ROOTED (ADR-0561)
//!
//! A platform service (the PDP) serves *all* tenants and is owned by *none*, so
//! the legacy tenant-rooted model (`spiffe://<tenant>` in
//! `identity-workload-domain::TrustDomain`) cannot name it. This kernel
//! adopts the cell-rooted authority + tenant-in-path scheme already used by the
//! api-gateway / SPIRE precedent (ADR-0295):
//!
//! - platform workload: `spiffe://oyatie.cell-<id>/platform/<service>`
//! - tenant workload:   `spiffe://oyatie.cell-<id>/tenant/<ten_x>/<workload>`
//!
//! This kernel is ADDITIVE: it does NOT touch the existing tenant-rooted
//! `TrustDomain` (ADR-0561 schedules that convergence as a tracked follow-up).
//! It reuses `identity-workload-domain::TenantId` for the bound tenant.
//!
//! ## Fail-closed
//!
//! Every parser and every gate is total and fail-closed: a malformed SVID, a
//! platform SVID asked to authorize a tenant, or a path tenant that disagrees
//! with the caller-body tenant all yield `Err`, never a silent default.

// ADR-0083 Tier 3: production code stays panic-free (deny in release); inline
// `mod tests` may use unwrap/expect/panic under cfg(test) only.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

use iam_identity_workload_domain::{TenantId, WorkloadIdentityError};

/// The SPIFFE URI scheme prefix every SVID carries.
const SPIFFE_SCHEME: &str = "spiffe://";

/// The cell-rooted trust-domain authority prefix (`oyatie.cell-<id>`), matching
/// the api-gateway / SPIRE precedent (ADR-0295).
const CELL_AUTHORITY_PREFIX: &str = "oyatie.cell-";

/// The path segment introducing a platform (tenant-agnostic) workload.
const PLATFORM_SEGMENT: &str = "platform";

/// The path segment introducing a tenant-scoped workload.
const TENANT_SEGMENT: &str = "tenant";

/// Errors produced while parsing a [`SpiffeId`] from a URI SAN. Exhaustive and
/// fail-closed: a SVID that does not parse cleanly authenticates nobody.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpiffeIdError {
    /// The URI did not start with the `spiffe://` scheme.
    MissingScheme,
    /// The authority segment was empty.
    EmptyAuthority,
    /// The authority was not a cell-rooted `oyatie.cell-<id>` authority.
    NotCellRooted,
    /// The path was empty (a bare trust-domain id, not an SVID).
    EmptyPath,
    /// A path segment was empty (e.g. a `//` in the path) or carried a
    /// whitespace/control character.
    MalformedPath,
    /// The path did not match a recognised workload shape
    /// (`platform/<service>` or `tenant/<ten_x>/<workload>`).
    UnrecognizedWorkloadShape,
    /// The tenant segment of a tenant-scoped path was not a valid `ten_<slug>`.
    InvalidTenantSegment,
}

impl fmt::Display for SpiffeIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingScheme => f.write_str("SVID URI missing spiffe:// scheme"),
            Self::EmptyAuthority => f.write_str("SVID URI has an empty trust-domain authority"),
            Self::NotCellRooted => {
                f.write_str("SVID authority is not cell-rooted (expected oyatie.cell-<id>)")
            }
            Self::EmptyPath => f.write_str("SVID URI has no path (bare trust domain, not an SVID)"),
            Self::MalformedPath => f.write_str("SVID URI path has an empty or malformed segment"),
            Self::UnrecognizedWorkloadShape => {
                f.write_str("SVID path is neither platform/<service> nor tenant/<ten_x>/<workload>")
            }
            Self::InvalidTenantSegment => {
                f.write_str("SVID tenant path segment is not a valid ten_<slug>")
            }
        }
    }
}

impl std::error::Error for SpiffeIdError {}

/// The classified workload path of a cell-rooted SVID.
///
/// A `Platform` workload (the PDP itself) is owned by no tenant; a `Tenant`
/// workload carries the tenant it speaks for in its path. This distinction is
/// load-bearing: [`bind_caller_tenant`] derives the authorized tenant from a
/// `Tenant` path and refuses a `Platform` SVID outright (a platform identity
/// must never assert it *is* a tenant).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkloadPath {
    /// `platform/<service>` — a tenant-agnostic platform service.
    Platform {
        /// The service name segment (e.g. `cloud-iam-pdp`).
        service: String,
    },
    /// `tenant/<ten_x>/<workload>` — a workload acting for a specific tenant.
    Tenant {
        /// The tenant the workload speaks for.
        tenant: TenantId,
        /// The workload name segment (e.g. `secrets-sync`).
        workload: String,
    },
}

impl WorkloadPath {
    /// Parse the path portion (everything after the authority) of an SVID URI
    /// into a classified [`WorkloadPath`]. `path` is the slash-joined segment
    /// list WITHOUT a leading slash (e.g. `platform/cloud-iam-pdp`).
    fn parse(path: &str) -> Result<Self, SpiffeIdError> {
        if path.is_empty() {
            return Err(SpiffeIdError::EmptyPath);
        }
        let segments: Vec<&str> = path.split('/').collect();
        // Every segment must be non-empty and free of whitespace/control chars.
        for segment in &segments {
            if segment.is_empty() {
                return Err(SpiffeIdError::MalformedPath);
            }
            if segment.chars().any(|c| c.is_whitespace() || c.is_control()) {
                return Err(SpiffeIdError::MalformedPath);
            }
        }
        match segments.as_slice() {
            [PLATFORM_SEGMENT, service] => Ok(WorkloadPath::Platform {
                service: (*service).to_string(),
            }),
            [TENANT_SEGMENT, tenant, workload] => {
                let tenant = TenantId::new(*tenant).map_err(|err| match err {
                    WorkloadIdentityError::InvalidTenantId => SpiffeIdError::InvalidTenantSegment,
                    // TenantId::new only ever yields InvalidTenantId; map any
                    // other (unreachable) variant to the same fail-closed error.
                    _ => SpiffeIdError::InvalidTenantSegment,
                })?;
                Ok(WorkloadPath::Tenant {
                    tenant,
                    workload: (*workload).to_string(),
                })
            }
            _ => Err(SpiffeIdError::UnrecognizedWorkloadShape),
        }
    }

    /// Whether this is a platform (tenant-agnostic) workload path.
    #[must_use]
    pub fn is_platform(&self) -> bool {
        matches!(self, WorkloadPath::Platform { .. })
    }

    /// The tenant this path speaks for, if it is a tenant-scoped path.
    #[must_use]
    pub fn tenant(&self) -> Option<&TenantId> {
        match self {
            WorkloadPath::Tenant { tenant, .. } => Some(tenant),
            WorkloadPath::Platform { .. } => None,
        }
    }
}

/// A parsed, cell-rooted SPIFFE id: a trust-domain authority plus a classified
/// workload path. Construction validates the cell-rooted scheme; the value
/// cannot exist in an invalid shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpiffeId {
    trust_domain_authority: String,
    path: WorkloadPath,
    /// The exact URI this id parsed from, preserved for byte-faithful echo.
    raw: String,
}

impl SpiffeId {
    /// Parse a `spiffe://oyatie.cell-<id>/<path>` URI into a [`SpiffeId`].
    ///
    /// # Errors
    /// [`SpiffeIdError`] for any deviation from the cell-rooted SVID shape
    /// (missing scheme, non-cell authority, empty/malformed path, unrecognised
    /// workload shape, invalid tenant segment).
    pub fn parse(uri: &str) -> Result<Self, SpiffeIdError> {
        let rest = uri
            .strip_prefix(SPIFFE_SCHEME)
            .ok_or(SpiffeIdError::MissingScheme)?;
        // Split authority (up to the first '/') from the path (the remainder).
        let (authority, path) = match rest.split_once('/') {
            Some((authority, path)) => (authority, path),
            // No '/' at all → bare trust domain, no SVID path.
            None => (rest, ""),
        };
        if authority.is_empty() {
            return Err(SpiffeIdError::EmptyAuthority);
        }
        // Cell-rooted authority: `oyatie.cell-<id>` with a non-empty id and no
        // whitespace/control characters.
        let Some(cell_id) = authority.strip_prefix(CELL_AUTHORITY_PREFIX) else {
            return Err(SpiffeIdError::NotCellRooted);
        };
        if cell_id.is_empty()
            || cell_id
                .chars()
                .any(|c| c.is_whitespace() || c.is_control() || c == '/')
        {
            return Err(SpiffeIdError::NotCellRooted);
        }
        let path = WorkloadPath::parse(path)?;
        Ok(SpiffeId {
            trust_domain_authority: authority.to_string(),
            path,
            raw: uri.to_string(),
        })
    }

    /// The trust-domain authority (`oyatie.cell-<id>`).
    #[must_use]
    pub fn trust_domain_authority(&self) -> &str {
        &self.trust_domain_authority
    }

    /// The classified workload path.
    #[must_use]
    pub fn path(&self) -> &WorkloadPath {
        &self.path
    }

    /// The exact SVID URI this id was parsed from.
    #[must_use]
    pub fn as_uri(&self) -> &str {
        &self.raw
    }
}

impl fmt::Display for SpiffeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

/// Why a tenant could not be bound to a caller SVID. Every variant is a DENY:
/// a caller that does not provably authorize the requested tenant is refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantBindingError {
    /// The SVID is a platform identity, which owns no tenant and may never
    /// assert it is acting *as* a tenant. Platform callers must reach the PDP
    /// through a path that does not claim a tenant identity.
    PlatformSvidCannotBindTenant,
    /// The tenant derived from the SVID path does not equal the tenant the
    /// caller asked to act for (the #717 cross-tenant spoof attempt).
    TenantMismatch {
        /// The tenant the SVID authorizes (from its verified path).
        svid_tenant: String,
        /// The tenant the caller's request body asked for.
        requested_tenant: String,
    },
}

impl fmt::Display for TenantBindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlatformSvidCannotBindTenant => {
                f.write_str("platform SVID cannot bind a tenant (it owns none)")
            }
            Self::TenantMismatch {
                svid_tenant,
                requested_tenant,
            } => write!(
                f,
                "SVID authorizes tenant '{svid_tenant}' but request asked for '{requested_tenant}'"
            ),
        }
    }
}

impl std::error::Error for TenantBindingError {}

/// The fail-closed tenant-binding gate (the #717 closure).
///
/// Given a *verified* caller [`SpiffeId`] and the tenant the caller's request
/// body asked to act for, return the authorized [`TenantId`] ONLY when the SVID
/// path proves the caller may speak for that tenant:
///
/// - a `tenant/<ten_x>/<workload>` SVID authorizes EXACTLY `ten_x`: a request
///   for any other tenant is [`TenantBindingError::TenantMismatch`] (DENY);
/// - a `platform/<service>` SVID owns no tenant and is refused with
///   [`TenantBindingError::PlatformSvidCannotBindTenant`].
///
/// The returned tenant is derived from the SVID, NEVER from the request body:
/// the body's tenant is only ever used to detect a mismatch, never trusted as
/// the answer.
///
/// # Errors
/// [`TenantBindingError`] on any platform SVID or any tenant mismatch.
pub fn bind_caller_tenant(
    svid: &SpiffeId,
    requested_tenant: &TenantId,
) -> Result<TenantId, TenantBindingError> {
    match svid.path() {
        WorkloadPath::Platform { .. } => Err(TenantBindingError::PlatformSvidCannotBindTenant),
        WorkloadPath::Tenant { tenant, .. } => {
            if tenant == requested_tenant {
                // Bind the SVID-derived tenant (identical to requested, but we
                // return the SVID copy on purpose: the SVID is the source of
                // truth, the request body is only a cross-check input).
                Ok(tenant.clone())
            } else {
                Err(TenantBindingError::TenantMismatch {
                    svid_tenant: tenant.as_str().to_string(),
                    requested_tenant: requested_tenant.as_str().to_string(),
                })
            }
        }
    }
}

// =====================================================================
// Ports (no-IO seams; the trustd adapter implements them)
// =====================================================================

/// A requested X.509-SVID issuance: the SPIFFE id to embed and the desired TTL.
/// Pure data — the issuer adapter turns this into a CSR + CA call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SvidRequest {
    /// The SPIFFE id the issued SVID must carry as its URI SAN.
    pub spiffe_id: SpiffeId,
    /// Requested certificate lifetime in seconds.
    pub ttl_secs: u64,
}

impl SvidRequest {
    /// Construct an issuance request.
    #[must_use]
    pub fn new(spiffe_id: SpiffeId, ttl_secs: u64) -> Self {
        Self {
            spiffe_id,
            ttl_secs,
        }
    }
}

/// An issued X.509-SVID: the leaf certificate DER plus the SPIFFE id it binds.
/// The DER is opaque to the kernel (the adapter owns the cert shape); the
/// kernel only reasons about the bound [`SpiffeId`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct X509Svid {
    /// The bound SPIFFE id (also encoded as the leaf's URI SAN).
    pub spiffe_id: SpiffeId,
    /// The issued leaf certificate, DER-shaped (opaque bytes).
    pub leaf_der: Vec<u8>,
}

/// A failure to issue an X.509-SVID. Carried as an opaque, fail-closed reason.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssueError {
    /// Human-legible reason (never a decision input; for logs/diagnostics).
    pub detail: String,
}

impl IssueError {
    /// Construct an issuance error.
    #[must_use]
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }
}

impl fmt::Display for IssueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SVID issuance failed: {}", self.detail)
    }
}

impl std::error::Error for IssueError {}

/// A failure to verify a presented peer SVID. Every variant is a DENY.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerifyError {
    /// No SPIFFE URI SAN was present on the peer certificate.
    NoSpiffeUriSan,
    /// More than one URI SAN was present (a SPIFFE SVID carries exactly one).
    AmbiguousUriSan,
    /// The leaf did not chain to a trusted CA, or its signature did not verify.
    UntrustedIssuer {
        /// Diagnostic detail (never a decision input).
        detail: String,
    },
    /// The leaf was outside its validity window at the verification instant.
    Expired,
    /// The URI SAN was present and the chain verified, but the URI did not
    /// parse as a cell-rooted SVID.
    MalformedSpiffeId(SpiffeIdError),
}

impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSpiffeUriSan => f.write_str("peer certificate carries no SPIFFE URI SAN"),
            Self::AmbiguousUriSan => f.write_str("peer certificate carries more than one URI SAN"),
            Self::UntrustedIssuer { detail } => {
                write!(
                    f,
                    "peer certificate did not verify against the trust bundle: {detail}"
                )
            }
            Self::Expired => f.write_str("peer SVID is expired"),
            Self::MalformedSpiffeId(err) => write!(f, "peer SVID URI is malformed: {err}"),
        }
    }
}

impl std::error::Error for VerifyError {}

/// Issues X.509-SVIDs for a requested SPIFFE id (the issuance port). The
/// adapter binds this to the trustd `SecurityService` issuance flow.
pub trait WorkloadIdentityIssuer {
    /// Issue an X.509-SVID for `request` as of `now` (epoch seconds).
    ///
    /// # Errors
    /// [`IssueError`] when issuance is refused (policy rejection, CA expired,
    /// etc.). Fail-closed: never returns a partial or unsigned SVID.
    fn issue_x509_svid(&self, request: &SvidRequest, now: u64) -> Result<X509Svid, IssueError>;
}

/// Verifies a presented peer leaf certificate and extracts its SPIFFE id (the
/// verification port). The adapter binds this to the trustd `TrustBundle`
/// chain-verification + URI-SAN extraction.
pub trait SvidVerifier {
    /// Verify `leaf_der` against the trust bundle as of `now` and return the
    /// SPIFFE id it binds.
    ///
    /// # Errors
    /// [`VerifyError`] on a missing/ambiguous URI SAN, an untrusted issuer, an
    /// expired leaf, or a malformed SPIFFE id. Fail-closed: a leaf that does
    /// not verify cleanly authenticates nobody.
    fn verify_peer(&self, leaf_der: &[u8], now: u64) -> Result<SpiffeId, VerifyError>;
}

/// The set of trusted SVID material a verifier checks against (the trust-bundle
/// port). Returns an opaque, adapter-owned bundle handle; the kernel never
/// inspects its contents.
pub trait TrustBundleSource {
    /// The opaque bundle type the adapter verifies against.
    type Bundle;

    /// The current trust bundle. Boot-refusal on a missing/garbage bundle is
    /// the COMPOSITION root's obligation (mirror pdp-kernel boot-refusal): this
    /// port assumes a valid bundle was proven present before the service bound
    /// a socket.
    fn current_bundle(&self) -> &Self::Bundle;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ten(slug: &str) -> TenantId {
        TenantId::new(slug).expect("valid tenant slug")
    }

    // ---- SpiffeId parsing --------------------------------------------------

    #[test]
    fn parses_platform_svid() {
        let id = SpiffeId::parse("spiffe://oyatie.cell-7/platform/cloud-iam-pdp").unwrap();
        assert_eq!(id.trust_domain_authority(), "oyatie.cell-7");
        assert!(id.path().is_platform());
        assert_eq!(id.path().tenant(), None);
        assert_eq!(id.as_uri(), "spiffe://oyatie.cell-7/platform/cloud-iam-pdp");
    }

    #[test]
    fn parses_tenant_svid() {
        let id = SpiffeId::parse("spiffe://oyatie.cell-3/tenant/ten_acme/secrets-sync").unwrap();
        assert_eq!(id.trust_domain_authority(), "oyatie.cell-3");
        assert!(!id.path().is_platform());
        assert_eq!(id.path().tenant(), Some(&ten("ten_acme")));
    }

    #[test]
    fn rejects_non_spiffe_scheme() {
        assert_eq!(
            SpiffeId::parse("https://oyatie.cell-7/platform/x"),
            Err(SpiffeIdError::MissingScheme)
        );
    }

    #[test]
    fn rejects_non_cell_authority() {
        // The legacy tenant-rooted authority is NOT a cell authority.
        assert_eq!(
            SpiffeId::parse("spiffe://ten_acme/tenant/ten_acme/x"),
            Err(SpiffeIdError::NotCellRooted)
        );
    }

    #[test]
    fn rejects_empty_authority_and_path() {
        assert_eq!(
            SpiffeId::parse("spiffe:///platform/x"),
            Err(SpiffeIdError::EmptyAuthority)
        );
        // bare trust domain, no path
        assert_eq!(
            SpiffeId::parse("spiffe://oyatie.cell-7"),
            Err(SpiffeIdError::EmptyPath)
        );
    }

    #[test]
    fn rejects_malformed_and_unrecognized_paths() {
        // double slash → empty segment
        assert_eq!(
            SpiffeId::parse("spiffe://oyatie.cell-7/platform//x"),
            Err(SpiffeIdError::MalformedPath)
        );
        // unknown leading segment
        assert_eq!(
            SpiffeId::parse("spiffe://oyatie.cell-7/agent/x"),
            Err(SpiffeIdError::UnrecognizedWorkloadShape)
        );
        // tenant shape but invalid tenant slug
        assert_eq!(
            SpiffeId::parse("spiffe://oyatie.cell-7/tenant/acme/wl"),
            Err(SpiffeIdError::InvalidTenantSegment)
        );
    }

    // ---- bind_caller_tenant (the #717 closure) -----------------------------

    #[test]
    fn tenant_binding_allows_matching_tenant() {
        let svid = SpiffeId::parse("spiffe://oyatie.cell-1/tenant/ten_acme/wl").unwrap();
        let bound = bind_caller_tenant(&svid, &ten("ten_acme")).unwrap();
        assert_eq!(bound, ten("ten_acme"));
    }

    #[test]
    fn tenant_binding_denies_mismatch() {
        // SVID authorizes ten_acme; request body asked for ten_globex → DENY.
        let svid = SpiffeId::parse("spiffe://oyatie.cell-1/tenant/ten_acme/wl").unwrap();
        let err = bind_caller_tenant(&svid, &ten("ten_globex")).unwrap_err();
        assert_eq!(
            err,
            TenantBindingError::TenantMismatch {
                svid_tenant: "ten_acme".to_string(),
                requested_tenant: "ten_globex".to_string(),
            }
        );
    }

    #[test]
    fn tenant_binding_refuses_platform_svid() {
        let svid = SpiffeId::parse("spiffe://oyatie.cell-1/platform/cloud-iam-pdp").unwrap();
        assert_eq!(
            bind_caller_tenant(&svid, &ten("ten_acme")),
            Err(TenantBindingError::PlatformSvidCannotBindTenant)
        );
    }

    // ---- port value types --------------------------------------------------

    #[test]
    fn svid_request_and_errors_construct() {
        let id = SpiffeId::parse("spiffe://oyatie.cell-1/platform/x").unwrap();
        let req = SvidRequest::new(id.clone(), 3600);
        assert_eq!(req.spiffe_id, id);
        assert_eq!(req.ttl_secs, 3600);
        assert!(IssueError::new("boom").to_string().contains("boom"));
        assert!(
            VerifyError::NoSpiffeUriSan
                .to_string()
                .contains("no SPIFFE URI SAN")
        );
    }
}
