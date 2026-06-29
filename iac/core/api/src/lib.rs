//! Cloud IaC API boundary for the OpenTofu module registry protocol.
//!
//! This crate owns request/path authorization and response DTO construction for
//! the module-registry surface before any future REST server, router, database,
//! object-store, signer, OpenTofu CLI runner, or provider runtime exists.
//! It intentionally performs no network, filesystem, signing, SLSA, provider,
//! state-backend, plan, apply, or cloud I/O.

#![forbid(unsafe_code)]

use iac_domain::{CloudIacError, ModuleRegistry};

pub mod authz {
    //! Fail-closed authorization seam for the Cloud IaC OpenTofu module-registry
    //! supply-chain surface (AUTH-005 / C-class; ADR-0587).
    //!
    //! ## Why this module exists
    //!
    //! The module-registry boundary
    //! ([`crate::discover_module_registry_from_api`],
    //! [`crate::list_module_versions_from_api`],
    //! [`crate::get_module_download_from_api`]) serves OpenTofu module ZIPs that are
    //! applied to live infrastructure — a supply-chain surface. Before this seam the
    //! only "authz" was the request-supplied `CloudIacModuleRegistryApiAuthorization`
    //! blob, whose `allowed_surfaces` list the boundary merely cross-checked for
    //! internal membership. Any caller who can reach the call sets
    //! `allowed_surfaces = [<every surface>]` (with a self-attested `principal_id` /
    //! `decision_id`) and the request is accepted — a STATIC ALLOW-ALL control plane
    //! (the AUTH-005 class the founder mandate requires to be impossible to ship).
    //! The composition root made this concrete: it baked a fabricated all-surfaces
    //! authorization blob into the handler and cloned it into every request.
    //!
    //! This module closes that gap by mirroring the proven fail-closed doctrine in
    //! `network/ports/dns/src/authz.rs` (#974) and `iam/ports/policy-cedar-api`
    //! (#815):
    //!
    //! 1. A real principal is VERIFIED from a credential the caller cannot forge — a
    //!    bearer token compared in constant time against a configured secret (the
    //!    [`PrincipalVerifier`] port; an mTLS/SPIFFE peer-SVID verifier is a drop-in
    //!    alternate adapter). The request-supplied principal id is NEVER the source
    //!    of truth.
    //! 2. The verified principal is AUTHORIZED for the requested module-registry
    //!    `surface` via a PDP [`ModuleRegistryAuthorizer`] port (`ensure_authorized`).
    //!    The decision is deny-by-default; any refusal is treated as deny
    //!    (fail-closed).
    //! 3. The boundary REFUSES TO SERVE without both ports configured (no
    //!    default-allow fallback): [`CloudIacModuleRegistryAuthzProvider`] has no
    //!    `Default` and the boundary fns take it by reference.
    //!
    //! ## Clean architecture (ADR-0131 / ports-for-owned-stack doctrine)
    //!
    //! [`PrincipalVerifier`] and [`ModuleRegistryAuthorizer`] are PORTS owned by this
    //! boundary crate. The concrete cloud-iam PDP client and the bearer/SVID
    //! credential store are ADAPTERS that live OUTSIDE this crate (the owned W5
    //! destination). The port shapes model that destination so they do not change at
    //! cutover; transient infra is absorbed by the adapter.

    // ADR-0083 Tier 3: production code stays panic-free.
    #![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

    use std::collections::BTreeSet;

    /// The credential a caller presents to prove a real principal identity.
    ///
    /// Today this is a bearer token (constant-time compared by
    /// [`ConfiguredBearerPrincipalVerifier`]); an mTLS/SPIFFE peer-SVID adapter is a
    /// drop-in alternate that consumes a verified peer leaf instead. There is NO
    /// caller-asserted principal id alongside it — the verifier is the sole source of
    /// truth for caller identity (a request blob never authorizes).
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct CallerCredential {
        /// Raw `Authorization` header value (e.g. `"Bearer abc..."`), if present.
        pub authorization: Option<String>, // data_class: SECRET
    }

    /// A principal whose identity has been verified from a caller credential.
    ///
    /// ## Type-level defense-in-depth (NOT a cryptographic guarantee)
    ///
    /// The field is **private**; there is no public constructor — external crates
    /// cannot build a `VerifiedPrincipal` by struct literal or any public API.
    /// [`VerifiedPrincipal::new`] is `pub(crate)`, callable only by
    /// [`PrincipalVerifier`] implementations inside this crate. External crates must
    /// obtain one by running a real [`PrincipalVerifier`] (e.g.
    /// [`ConfiguredBearerPrincipalVerifier`]).
    ///
    /// **Limits of this guarantee:** this is *structural* defense-in-depth, not a
    /// cryptographic proof. It prevents accidental struct-literal forging and proves
    /// that *some* `PrincipalVerifier` ran. The real security guarantee comes from
    /// the combination of: (1) verifying the credential before any serving, and (2)
    /// the PDP authorization decision against the requested surface.
    ///
    /// Within the same crate, tests use the `#[cfg(test)]` constructor
    /// [`VerifiedPrincipal::new_for_test`] to mint tokens without a real credential.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct VerifiedPrincipal {
        principal_id: String, // data_class: INTERNAL_ONLY — private: see unforgeability note
    }

    impl VerifiedPrincipal {
        /// Mint a verified principal. **`pub(crate)` only** — callers outside this
        /// crate cannot call this; they must go through a [`PrincipalVerifier`].
        pub(crate) fn new(principal_id: impl Into<String>) -> Self {
            Self {
                principal_id: principal_id.into(),
            }
        }

        /// The authoritative principal id bound from the verified credential. The
        /// audit identity of a served request is this value, never a request blob.
        #[must_use]
        pub fn principal_id(&self) -> &str {
            &self.principal_id
        }

        /// Test-only constructor that mints a token without a real credential.
        /// Only available inside this crate under `#[cfg(test)]`.
        #[cfg(test)]
        pub(crate) fn new_for_test(principal_id: impl Into<String>) -> Self {
            Self::new(principal_id)
        }
    }

    /// Why principal verification refused. Every variant is fail-closed: the caller
    /// maps it to HTTP 401 and the request never reaches the authorizer.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum PrincipalVerificationError {
        /// No credential was presented (no `Authorization` header).
        MissingCredential,
        /// A credential was presented but did not verify (bad bearer, untrusted
        /// SVID, expired, …). Deliberately opaque so probing cannot distinguish
        /// "wrong token" from "no such principal".
        InvalidCredential,
    }

    /// Why authorization refused. Each variant maps to HTTP 403 (the principal is
    /// authenticated but not permitted for this surface). The caller maps Denied and
    /// Refused IDENTICALLY so a prober cannot distinguish an explicit deny from a PDP
    /// fault.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum ModuleRegistryAuthorizationError {
        /// The PDP returned a deny decision for this principal/surface.
        Denied,
        /// The PDP refused to decide (fail-closed: a refusal is treated as deny).
        Refused,
    }

    /// PORT: verify a caller credential into a [`VerifiedPrincipal`].
    ///
    /// Adapters: a configured-bearer verifier (this crate's
    /// [`ConfiguredBearerPrincipalVerifier`]) or a cloud-iam mTLS/SPIFFE peer-SVID
    /// verifier (the W5 destination). The verifier — not the headers — is the source
    /// of truth for caller identity.
    pub trait PrincipalVerifier: Send + Sync {
        /// Verify `credential` and return the authoritative principal, or refuse.
        ///
        /// # Errors
        /// [`PrincipalVerificationError`] when no credential is presented or it does
        /// not verify (fail-closed: the caller MUST treat this as 401).
        fn verify_principal(
            &self,
            credential: &CallerCredential,
        ) -> Result<VerifiedPrincipal, PrincipalVerificationError>;
    }

    /// PORT: decide whether `principal` may use the module-registry `surface`.
    ///
    /// The decision is `decide(principal, action = surface)`. Adapter: the cloud-iam
    /// PDP client (the owned W5 destination). The default posture is deny; any
    /// refusal is treated as deny (fail-closed).
    ///
    /// ## Adapter implementation contract (MUST follow; enforcement is by convention)
    ///
    /// 1. **Map every internal fault to `Err(Refused)`.** Network errors, timeouts,
    ///    parse failures, and unavailability MUST all return
    ///    `Err(ModuleRegistryAuthorizationError::Refused)` so the caller can map them
    ///    to HTTP 403 (fail-closed). Never propagate an internal error as `Ok(())`.
    /// 2. **Enforce a deadline.** A hung PDP hangs the caller. Adapters MUST enforce
    ///    their own deadline and map expiry to `Err(Refused)`.
    /// 3. **Do not panic.** The release profile uses `panic = "abort"`, so a panic
    ///    in production terminates the process rather than being catchable. Adapters
    ///    MUST NOT panic — use `Err(Refused)` for every recoverable and
    ///    unrecoverable fault.
    pub trait ModuleRegistryAuthorizer: Send + Sync {
        /// Authorize `principal` to use the module-registry `surface`, or refuse.
        ///
        /// # Errors
        /// [`ModuleRegistryAuthorizationError`] on an explicit deny or any PDP fault
        /// (timeout, network, unavailability — all MUST be `Refused`; fail-closed:
        /// the caller maps this to HTTP 403).
        fn ensure_authorized(
            &self,
            principal: &VerifiedPrincipal,
            surface: &str,
        ) -> Result<(), ModuleRegistryAuthorizationError>;
    }

    /// The authz provider the boundary depends on: a principal verifier PORT plus a
    /// module-registry authorizer PORT. The boundary REFUSES to serve without one
    /// configured (no default-allow fallback) — there is no `Default` impl.
    pub struct CloudIacModuleRegistryAuthzProvider {
        verifier: std::sync::Arc<dyn PrincipalVerifier>, // data_class: INTERNAL_ONLY
        authorizer: std::sync::Arc<dyn ModuleRegistryAuthorizer>, // data_class: INTERNAL_ONLY
    }

    impl CloudIacModuleRegistryAuthzProvider {
        /// Assemble the provider from a principal verifier and a module-registry
        /// authorizer. There is deliberately no `Default`: a process that cannot
        /// prove a credential root and a PDP decision must never serve.
        #[must_use]
        pub fn new(
            verifier: std::sync::Arc<dyn PrincipalVerifier>,
            authorizer: std::sync::Arc<dyn ModuleRegistryAuthorizer>,
        ) -> Self {
            Self {
                verifier,
                authorizer,
            }
        }

        /// Verify the caller principal via the [`PrincipalVerifier`] port. The
        /// headers are never trusted as identity.
        ///
        /// # Errors
        /// [`PrincipalVerificationError`] — caller maps to HTTP 401.
        pub fn verify_principal(
            &self,
            credential: &CallerCredential,
        ) -> Result<VerifiedPrincipal, PrincipalVerificationError> {
            self.verifier.verify_principal(credential)
        }

        /// Authorize the verified principal for the module-registry `surface` via the
        /// PDP port. Default-deny / fail-closed.
        ///
        /// # Errors
        /// [`ModuleRegistryAuthorizationError`] — caller maps to HTTP 403.
        pub fn ensure_authorized(
            &self,
            principal: &VerifiedPrincipal,
            surface: &str,
        ) -> Result<(), ModuleRegistryAuthorizationError> {
            self.authorizer.ensure_authorized(principal, surface)
        }
    }

    /// Constant-time byte comparison (no early-exit) so a bearer compare cannot be
    /// timing-probed. Mirrors `network/ports/dns/src/authz.rs` — NEVER use a naive
    /// `==` on secret material.
    ///
    /// **Residual:** the length of both inputs is visible from the XOR seed
    /// (`a.len() ^ b.len()`). This is the accepted repo-wide residual; bearer tokens
    /// are fixed-length secrets. Use a MAC (HMAC-SHA256) if length-hiding is
    /// required.
    #[must_use]
    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        let max_len = a.len().max(b.len());
        let mut diff = a.len() ^ b.len();
        for index in 0..max_len {
            let left = a.get(index).copied().unwrap_or(0);
            let right = b.get(index).copied().unwrap_or(0);
            diff |= (left ^ right) as usize;
        }
        diff == 0
    }

    /// A reference [`PrincipalVerifier`] adapter that verifies a bearer token by a
    /// constant-time compare against a configured secret, then binds the principal
    /// identity from the configured value (NOT from the caller headers).
    ///
    /// ## ⚠ BREAK-GLASS ONLY — NOT multi-tenant production
    ///
    /// This adapter binds ONE static `principal_id` to a single shared secret. It is
    /// suitable only as a single-principal break-glass credential or for integration
    /// tests. The production W5 adapter is the cloud-iam mTLS/SPIFFE peer-SVID
    /// verifier, which derives the principal from the verified peer certificate, not
    /// from a configured value.
    ///
    /// Construction REFUSES an empty bearer secret or bound identity so a provider
    /// that cannot prove a credential root can never authenticate a caller.
    pub struct ConfiguredBearerPrincipalVerifier {
        bearer_secret: String,      // data_class: SECRET
        bound_principal_id: String, // data_class: INTERNAL_ONLY
    }

    impl ConfiguredBearerPrincipalVerifier {
        /// Construct, REFUSING an empty bearer secret or empty bound identity. A
        /// process that cannot prove a credential root must never authenticate.
        ///
        /// # Errors
        /// [`AuthzProviderConfigError`] when the secret or bound identity is empty.
        pub fn new(
            bearer_secret: impl Into<String>,
            bound_principal_id: impl Into<String>,
        ) -> Result<Self, AuthzProviderConfigError> {
            let bearer_secret = bearer_secret.into();
            let bound_principal_id = bound_principal_id.into();
            if bearer_secret.trim().is_empty() {
                return Err(AuthzProviderConfigError::EmptyBearerSecret);
            }
            if bound_principal_id.trim().is_empty() {
                return Err(AuthzProviderConfigError::EmptyBoundIdentity);
            }
            Ok(Self {
                bearer_secret,
                bound_principal_id,
            })
        }
    }

    impl PrincipalVerifier for ConfiguredBearerPrincipalVerifier {
        fn verify_principal(
            &self,
            credential: &CallerCredential,
        ) -> Result<VerifiedPrincipal, PrincipalVerificationError> {
            let Some(authorization) = credential.authorization.as_deref() else {
                return Err(PrincipalVerificationError::MissingCredential);
            };
            let Some(presented) = authorization.strip_prefix("Bearer ") else {
                return Err(PrincipalVerificationError::InvalidCredential);
            };
            if !constant_time_eq(presented.as_bytes(), self.bearer_secret.as_bytes()) {
                return Err(PrincipalVerificationError::InvalidCredential);
            }
            Ok(VerifiedPrincipal::new(self.bound_principal_id.clone()))
        }
    }

    /// A reference [`ModuleRegistryAuthorizer`] adapter: a deny-by-default break-glass
    /// authorizer that permits ONLY the surfaces in its configured allow-set. Any
    /// surface not explicitly permitted is denied. The production W5 adapter is the
    /// cloud-iam Cedar PDP client; this in-process set authorizer is the break-glass
    /// stand-in with the same fail-closed default-deny posture.
    pub struct ConfiguredSurfaceAuthorizer {
        permitted_surfaces: BTreeSet<String>, // data_class: INTERNAL_ONLY
    }

    impl ConfiguredSurfaceAuthorizer {
        /// Construct from the set of permitted surfaces. An empty set denies every
        /// surface (the safest default), never an error.
        #[must_use]
        pub fn new(permitted_surfaces: impl IntoIterator<Item = String>) -> Self {
            Self {
                permitted_surfaces: permitted_surfaces.into_iter().collect(),
            }
        }
    }

    impl ModuleRegistryAuthorizer for ConfiguredSurfaceAuthorizer {
        fn ensure_authorized(
            &self,
            _principal: &VerifiedPrincipal,
            surface: &str,
        ) -> Result<(), ModuleRegistryAuthorizationError> {
            if self.permitted_surfaces.contains(surface) {
                Ok(())
            } else {
                Err(ModuleRegistryAuthorizationError::Denied)
            }
        }
    }

    /// Why the authz provider refused construction. Boot-fatal: the composition root
    /// MUST refuse to serve.
    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum AuthzProviderConfigError {
        /// The bearer secret was empty/whitespace (no provable credential root).
        EmptyBearerSecret,
        /// The bound principal identity was empty.
        EmptyBoundIdentity,
    }

    impl std::fmt::Display for AuthzProviderConfigError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::EmptyBearerSecret => {
                    write!(f, "authz provider bearer secret must be non-empty")
                }
                Self::EmptyBoundIdentity => {
                    write!(f, "authz provider bound principal must be non-empty")
                }
            }
        }
    }

    impl std::error::Error for AuthzProviderConfigError {}
}

pub use authz::{
    AuthzProviderConfigError, CallerCredential, CloudIacModuleRegistryAuthzProvider,
    ConfiguredBearerPrincipalVerifier, ConfiguredSurfaceAuthorizer,
    ModuleRegistryAuthorizationError, ModuleRegistryAuthorizer, PrincipalVerificationError,
    PrincipalVerifier, VerifiedPrincipal, constant_time_eq,
};

pub const CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE: &str = "cloud.iac.module_registry.discovery";
pub const CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE: &str = "cloud.iac.module_registry.versions";
pub const CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE: &str = "cloud.iac.module_registry.download";
pub const OPENTOFU_MODULE_REGISTRY_HTTP_GET: &str = "GET";
pub const OPENTOFU_SERVICE_DISCOVERY_PATH: &str = "/.well-known/terraform.json";
pub const OPENTOFU_MODULES_V1_BASE_PATH: &str = "/v1/modules/";
pub const CLOUD_IAC_MODULE_REGISTRY_API_BOUNDARY_NON_CLAIM: &str =
    "pure-api-boundary-no-rest-server-no-live-registry-runtime";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudIacModuleRegistryApiError {
    EmptyRequestId,
    /// The caller credential did not verify (missing or invalid). 401.
    Unauthenticated,
    MethodNotAllowed {
        method: String,
    },
    RouteNotFound {
        path: String,
    },
    /// The PDP denied OR refused (fail-closed) the requested surface. 403. Deny
    /// and refuse are reported IDENTICALLY so probing cannot distinguish them.
    Forbidden {
        surface: String,
    },
    Domain(CloudIacError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIacModuleRegistryApiBoundaryContext {
    pub request_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIacModuleVersionsApiRequest {
    pub boundary: CloudIacModuleRegistryApiBoundaryContext, // data_class: INTERNAL_ONLY
    /// The caller credential (e.g. bearer token). The boundary verifies this via
    /// the injected [`authz::PrincipalVerifier`] and authorizes the surface via
    /// the [`authz::ModuleRegistryAuthorizer`] PDP port. No request blob grants.
    pub credential: authz::CallerCredential, // data_class: SECRET
    pub namespace: String,                                  // data_class: PUBLIC
    pub name: String,                                       // data_class: PUBLIC
    pub system: String,                                     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIacModuleDownloadApiRequest {
    pub boundary: CloudIacModuleRegistryApiBoundaryContext, // data_class: INTERNAL_ONLY
    /// The caller credential (e.g. bearer token); see
    /// [`CloudIacModuleVersionsApiRequest::credential`].
    pub credential: authz::CallerCredential, // data_class: SECRET
    pub namespace: String,                                  // data_class: PUBLIC
    pub name: String,                                       // data_class: PUBLIC
    pub system: String,                                     // data_class: PUBLIC
    pub version: String,                                    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudIacModuleRegistryRouteRequest {
    pub boundary: CloudIacModuleRegistryApiBoundaryContext, // data_class: INTERNAL_ONLY
    /// The caller credential (e.g. bearer token); see
    /// [`CloudIacModuleVersionsApiRequest::credential`].
    pub credential: authz::CallerCredential, // data_class: SECRET
    pub method: String,                                     // data_class: PUBLIC
    pub path: String,                                       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleRegistryDiscoveryResponse {
    pub path: String,       // data_class: PUBLIC
    pub modules_v1: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleVersionEntry {
    pub version: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleVersionsResponseModule {
    pub versions: Vec<ModuleVersionEntry>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleVersionsResponse {
    pub modules: Vec<ModuleVersionsResponseModule>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleDownloadResponse {
    pub location: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudIacModuleRegistryRouteResponse {
    Discovery(ModuleRegistryDiscoveryResponse),
    Versions(ModuleVersionsResponse),
    Download(ModuleDownloadResponse),
}

pub fn discover_module_registry_from_api(
    boundary: &CloudIacModuleRegistryApiBoundaryContext,
    authz_provider: &authz::CloudIacModuleRegistryAuthzProvider,
    credential: &authz::CallerCredential,
) -> Result<ModuleRegistryDiscoveryResponse, CloudIacModuleRegistryApiError> {
    validate_boundary(boundary)?;
    let _principal = authorize(
        authz_provider,
        credential,
        CLOUD_IAC_MODULE_REGISTRY_DISCOVERY_SURFACE,
    )?;
    Ok(ModuleRegistryDiscoveryResponse {
        path: OPENTOFU_SERVICE_DISCOVERY_PATH.to_string(),
        modules_v1: OPENTOFU_MODULES_V1_BASE_PATH.to_string(),
    })
}

pub fn route_module_registry_request(
    registry: &ModuleRegistry,
    authz_provider: &authz::CloudIacModuleRegistryAuthzProvider,
    request: CloudIacModuleRegistryRouteRequest,
) -> Result<CloudIacModuleRegistryRouteResponse, CloudIacModuleRegistryApiError> {
    if request.method != OPENTOFU_MODULE_REGISTRY_HTTP_GET {
        return Err(CloudIacModuleRegistryApiError::MethodNotAllowed {
            method: request.method,
        });
    }

    if request.path == OPENTOFU_SERVICE_DISCOVERY_PATH {
        return discover_module_registry_from_api(
            &request.boundary,
            authz_provider,
            &request.credential,
        )
        .map(CloudIacModuleRegistryRouteResponse::Discovery);
    }

    let Some(module_path) = request.path.strip_prefix(OPENTOFU_MODULES_V1_BASE_PATH) else {
        return Err(CloudIacModuleRegistryApiError::RouteNotFound { path: request.path });
    };
    let segments = module_path.split('/').collect::<Vec<_>>();
    if segments.iter().any(|segment| segment.is_empty()) {
        return Err(CloudIacModuleRegistryApiError::RouteNotFound { path: request.path });
    }

    match segments.as_slice() {
        [namespace, name, system, "versions"] => list_module_versions_from_api(
            registry,
            authz_provider,
            CloudIacModuleVersionsApiRequest {
                boundary: request.boundary,
                credential: request.credential,
                namespace: (*namespace).to_string(),
                name: (*name).to_string(),
                system: (*system).to_string(),
            },
        )
        .map(CloudIacModuleRegistryRouteResponse::Versions),
        [namespace, name, system, version, "download"] => get_module_download_from_api(
            registry,
            authz_provider,
            CloudIacModuleDownloadApiRequest {
                boundary: request.boundary,
                credential: request.credential,
                namespace: (*namespace).to_string(),
                name: (*name).to_string(),
                system: (*system).to_string(),
                version: (*version).to_string(),
            },
        )
        .map(CloudIacModuleRegistryRouteResponse::Download),
        _ => Err(CloudIacModuleRegistryApiError::RouteNotFound { path: request.path }),
    }
}

pub fn list_module_versions_from_api(
    registry: &ModuleRegistry,
    authz_provider: &authz::CloudIacModuleRegistryAuthzProvider,
    request: CloudIacModuleVersionsApiRequest,
) -> Result<ModuleVersionsResponse, CloudIacModuleRegistryApiError> {
    validate_boundary(&request.boundary)?;
    let _principal = authorize(
        authz_provider,
        &request.credential,
        CLOUD_IAC_MODULE_REGISTRY_VERSIONS_SURFACE,
    )?;
    let versions = registry
        .versions(&request.namespace, &request.name, &request.system)?
        .into_iter()
        .map(|release| ModuleVersionEntry {
            version: release.version().to_string(),
        })
        .collect();

    Ok(ModuleVersionsResponse {
        modules: vec![ModuleVersionsResponseModule { versions }],
    })
}

pub fn get_module_download_from_api(
    registry: &ModuleRegistry,
    authz_provider: &authz::CloudIacModuleRegistryAuthzProvider,
    request: CloudIacModuleDownloadApiRequest,
) -> Result<ModuleDownloadResponse, CloudIacModuleRegistryApiError> {
    validate_boundary(&request.boundary)?;
    let _principal = authorize(
        authz_provider,
        &request.credential,
        CLOUD_IAC_MODULE_REGISTRY_DOWNLOAD_SURFACE,
    )?;
    let release = registry.resolve(
        &request.namespace,
        &request.name,
        &request.system,
        &request.version,
    )?;
    Ok(ModuleDownloadResponse {
        location: release.source().to_string(),
    })
}

fn validate_boundary(
    boundary: &CloudIacModuleRegistryApiBoundaryContext,
) -> Result<(), CloudIacModuleRegistryApiError> {
    if boundary.request_id.trim().is_empty() {
        Err(CloudIacModuleRegistryApiError::EmptyRequestId)
    } else {
        Ok(())
    }
}

/// FAIL-CLOSED authorization: VERIFY the caller credential (missing/invalid →
/// 401), then ask the PDP whether the verified principal may use `surface` (deny
/// OR any fault → 403, reported identically). No request-supplied blob authorizes
/// anything. Returns the verified principal so the served request's audit
/// identity is the verified `principal_id()`, never a caller claim.
///
// ponytail: returns the VerifiedPrincipal for the audit identity; this pure
// boundary has no audit sink yet, so callers bind `_principal`. Wire the sink
// here when one exists rather than re-deriving the principal downstream.
fn authorize(
    authz_provider: &authz::CloudIacModuleRegistryAuthzProvider,
    credential: &authz::CallerCredential,
    surface: &str,
) -> Result<authz::VerifiedPrincipal, CloudIacModuleRegistryApiError> {
    let verified = authz_provider
        .verify_principal(credential)
        .map_err(|error| match error {
            authz::PrincipalVerificationError::MissingCredential
            | authz::PrincipalVerificationError::InvalidCredential => {
                CloudIacModuleRegistryApiError::Unauthenticated
            }
        })?;
    authz_provider
        .ensure_authorized(&verified, surface)
        .map_err(|error| match error {
            authz::ModuleRegistryAuthorizationError::Denied
            | authz::ModuleRegistryAuthorizationError::Refused => {
                CloudIacModuleRegistryApiError::Forbidden {
                    surface: surface.to_string(),
                }
            }
        })?;
    Ok(verified)
}

impl From<CloudIacError> for CloudIacModuleRegistryApiError {
    fn from(value: CloudIacError) -> Self {
        Self::Domain(value)
    }
}
