//! mTLS caller authentication PEP (G002 slice-1; ADR-0561).
//!
//! Closes the gap where the PDP trusted a caller-supplied `tenant_id` verbatim
//! (grpc.rs `tenant_id: request.tenant_id`, proto tenant_id=2). The
//! [`SpiffeCallerAuth`] PEP authenticates a caller by its VERIFIED peer SVID and
//! derives the authorized tenant from the SVID path — the request-body tenant is
//! only ever a cross-check input, never the source of truth (the #717 closure).
//!
//! ## Boot-refusal (mirror pdp-kernel)
//!
//! [`SpiffeCallerAuth::new`] REFUSES construction when the trust bundle is empty
//! (missing/garbage), exactly as the PDP refuses to boot without a policy bundle
//! (`server.rs` `build_state`): a process that cannot prove a trust root must
//! never authenticate a caller.
//!
//! ## Fail-closed reject paths (gRPC PermissionDenied / REST 403, NEVER 404)
//!
//! Every rejection denies with [`tonic::Code::PermissionDenied`] on gRPC and
//! HTTP 403 on REST — never 404 (which would leak "this tenant/route exists"),
//! never a silent fall-through:
//!
//! 1. no client cert presented            → [`CallerAuthRejection::NoClientCert`]
//! 2. untrusted issuer / undecodable leaf  → [`CallerAuthRejection::UntrustedSvid`]
//! 3. expired SVID                         → [`CallerAuthRejection::ExpiredSvid`]
//! 4. SVID tenant ≠ requested tenant       → [`CallerAuthRejection::TenantMismatch`]
//! 5. malformed requested tenant id        → [`CallerAuthRejection::MalformedRequestTenant`]
//! 6. platform SVID asserting a tenant     → [`CallerAuthRejection::PlatformSvidCannotActAsTenant`]
//!
//! ## Fidelity boundary (ADR-0561 slice-1b deferral)
//!
//! The peer-leaf bytes this PEP consumes are what a rustls server handshake
//! would hand it post-verification. The REAL rustls `ServerConfig` requiring a
//! client cert on `server.rs`'s listeners, and the cloud-kms signer swap, are
//! the explicitly DEFERRED slice-1b. This PEP is the in-process
//! verify→bind→deny logic, fully testable without K8s or a TLS terminator.

use oya_cloud_os_trustd_domain::signer::SigningBackend;
use oya_cloud_os_trustd_domain::TrustBundle;
use oya_identity_workload_domain::TenantId;
use oya_identity_workload_svid_kernel::{
    bind_caller_tenant, SvidVerifier, TenantBindingError, VerifyError,
};
use oya_identity_workload_svid_trustd_adapter::TrustdSvidVerifier;
use tonic::{Code, Status};

/// Why the mTLS PEP refused to come up. Boot-fatal: the caller (composition
/// root) MUST refuse to serve, mirroring [`crate::server::StartError`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MtlsBootError {
    /// The trust bundle held no anchors (missing/garbage) — there is no root to
    /// verify a caller SVID against.
    TrustBundleEmpty,
    /// The rustls `ServerConfig` could not be built from the supplied PDP server
    /// leaf/key (malformed DER, key/cert mismatch, or no usable protocol
    /// version) — a server that cannot present its own identity must never boot.
    ServerConfig(String),
}

impl std::fmt::Display for MtlsBootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TrustBundleEmpty => f.write_str(
                "mTLS trust bundle is empty (missing/garbage), refusing to authenticate callers",
            ),
            Self::ServerConfig(detail) => {
                write!(f, "mTLS server config rejected, refusing to boot: {detail}")
            }
        }
    }
}

impl std::error::Error for MtlsBootError {}

/// A caller-authentication rejection. Every variant is a DENY and maps to gRPC
/// `PermissionDenied` / HTTP 403 (never 404, never a fall-through).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallerAuthRejection {
    /// No client certificate was presented (the connection was not mTLS).
    NoClientCert,
    /// The presented SVID did not chain to a trusted anchor, or did not decode.
    UntrustedSvid {
        /// Diagnostic detail (never echoed to the caller body).
        detail: String,
    },
    /// The presented SVID was outside its validity window.
    ExpiredSvid,
    /// The presented SVID was malformed (no/ambiguous URI SAN or a non-SVID URI).
    MalformedSvid {
        /// Diagnostic detail.
        detail: String,
    },
    /// The SVID authorizes a different tenant than the request asked for.
    TenantMismatch {
        /// The tenant the SVID authorizes.
        svid_tenant: String,
        /// The tenant the request body asked for.
        requested_tenant: String,
    },
    /// The request-body tenant id was not a valid `ten_<slug>`.
    MalformedRequestTenant,
    /// A platform SVID tried to act as a tenant (it owns none).
    PlatformSvidCannotActAsTenant,
}

impl CallerAuthRejection {
    /// The gRPC status — ALWAYS `PermissionDenied`, never another code, so a PEP
    /// can never confuse an auth failure with a not-found or invalid-argument.
    #[must_use]
    pub fn to_grpc_status(&self) -> Status {
        Status::new(Code::PermissionDenied, self.public_message())
    }

    /// The REST status code — ALWAYS 403 Forbidden, never 404.
    #[must_use]
    pub fn rest_status_code(&self) -> u16 {
        403
    }

    /// The caller-facing message. Deliberately coarse: it states the auth
    /// failure class without leaking which trust anchor, tenant, or path the
    /// caller missed (anti-enumeration).
    #[must_use]
    pub fn public_message(&self) -> &'static str {
        match self {
            Self::NoClientCert => "client certificate required",
            Self::UntrustedSvid { .. } | Self::MalformedSvid { .. } => {
                "caller SVID is not trusted"
            }
            Self::ExpiredSvid => "caller SVID is expired",
            Self::TenantMismatch { .. } | Self::PlatformSvidCannotActAsTenant => {
                "caller is not authorized for the requested tenant"
            }
            Self::MalformedRequestTenant => "requested tenant id is malformed",
        }
    }
}

impl std::fmt::Display for CallerAuthRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoClientCert => f.write_str("no client certificate presented"),
            Self::UntrustedSvid { detail } => write!(f, "untrusted caller SVID: {detail}"),
            Self::ExpiredSvid => f.write_str("caller SVID expired"),
            Self::MalformedSvid { detail } => write!(f, "malformed caller SVID: {detail}"),
            Self::TenantMismatch {
                svid_tenant,
                requested_tenant,
            } => write!(
                f,
                "SVID tenant '{svid_tenant}' does not match requested tenant '{requested_tenant}'"
            ),
            Self::MalformedRequestTenant => f.write_str("requested tenant id malformed"),
            Self::PlatformSvidCannotActAsTenant => {
                f.write_str("platform SVID cannot act as a tenant")
            }
        }
    }
}

/// The mTLS caller-authentication PEP over a trustd trust bundle.
///
/// Holds the verifier; `authenticate_caller` is called BEFORE the entity-ref
/// build in `grpc.rs`/`rest.rs` and returns the SVID-derived [`TenantId`] that
/// the decision path must use in place of the caller-body tenant.
pub struct SpiffeCallerAuth<'a, S: SigningBackend> {
    verifier: TrustdSvidVerifier<'a, S>,
}

impl<'a, S: SigningBackend> SpiffeCallerAuth<'a, S> {
    /// Build the PEP over a trust bundle, REFUSING (boot-fatal) when the bundle
    /// is empty (missing/garbage).
    ///
    /// # Errors
    /// [`MtlsBootError::TrustBundleEmpty`] when the bundle holds no anchors.
    pub fn new(bundle: &'a TrustBundle<S>) -> Result<Self, MtlsBootError> {
        if bundle.is_empty() {
            return Err(MtlsBootError::TrustBundleEmpty);
        }
        Ok(Self {
            verifier: TrustdSvidVerifier::new(bundle),
        })
    }

    /// Authenticate a caller by its verified peer SVID and bind the authorized
    /// tenant from the SVID path. `peer_leaf` is the post-handshake peer leaf
    /// bytes (`None` when no client cert was presented); `requested_tenant` is
    /// the raw request-body tenant id; `now` is epoch seconds.
    ///
    /// Returns the SVID-derived [`TenantId`] — the decision path MUST use this,
    /// not the request body. Any deviation is a fail-closed DENY.
    ///
    /// # Errors
    /// [`CallerAuthRejection`] on every reject path (no cert, untrusted,
    /// expired, malformed, tenant mismatch, malformed request tenant, platform
    /// SVID acting as tenant).
    pub fn authenticate_caller(
        &self,
        peer_leaf: Option<&[u8]>,
        requested_tenant: &str,
        now: u64,
    ) -> Result<TenantId, CallerAuthRejection> {
        // Reject path 1: no client cert.
        let leaf = peer_leaf.ok_or(CallerAuthRejection::NoClientCert)?;

        // Reject paths 2/3/(malformed): verify the chain + extract the SPIFFE id.
        let svid = self.verifier.verify_peer(leaf, now).map_err(|err| match err {
            VerifyError::Expired => CallerAuthRejection::ExpiredSvid,
            VerifyError::NoSpiffeUriSan | VerifyError::AmbiguousUriSan => {
                CallerAuthRejection::MalformedSvid {
                    detail: err.to_string(),
                }
            }
            VerifyError::MalformedSpiffeId(_) => CallerAuthRejection::MalformedSvid {
                detail: err.to_string(),
            },
            VerifyError::UntrustedIssuer { detail } => {
                CallerAuthRejection::UntrustedSvid { detail }
            }
        })?;

        // Reject path 5: the request-body tenant must itself be a valid id.
        let requested = TenantId::new(requested_tenant)
            .map_err(|_| CallerAuthRejection::MalformedRequestTenant)?;

        // Reject paths 4/6: bind the tenant from the SVID, rejecting a mismatch
        // or a platform SVID. This is the #717 closure — the returned tenant is
        // SVID-derived, never the request body.
        bind_caller_tenant(&svid, &requested).map_err(|err| match err {
            TenantBindingError::PlatformSvidCannotBindTenant => {
                CallerAuthRejection::PlatformSvidCannotActAsTenant
            }
            TenantBindingError::TenantMismatch {
                svid_tenant,
                requested_tenant,
            } => CallerAuthRejection::TenantMismatch {
                svid_tenant,
                requested_tenant,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_cloud_os_trustd_domain::ca::{CertificateAuthority, CertificateSigningRequest};
    use oya_cloud_os_trustd_domain::certificate::{CertUsage, Certificate};
    use oya_cloud_os_trustd_domain::der;
    use oya_cloud_os_trustd_domain::service::{CertificateRequest, SecurityService};
    use oya_cloud_os_trustd_domain::signer::EcdsaP256Signer;
    use oya_cloud_os_trustd_domain::x509::KeyPair;
    use oya_cloud_os_trustd_domain::JoinToken;

    const JOIN_TOKEN: &str = "clusterid.clustersecret";

    /// A REAL-crypto trustd CA: an `EcdsaP256Signer` plus a trustd `Certificate`
    /// whose `public_key_der` is the CA's real SubjectPublicKeyInfo (so the bundle
    /// anchor verifies real leaf signatures, slice-1b-i).
    fn ca(name: &str) -> (CertificateAuthority<EcdsaP256Signer>, EcdsaP256Signer) {
        let signer = EcdsaP256Signer::generate().unwrap();
        let key = KeyPair::new(signer.private_key_der(), signer.public_key_spki_der());
        let ca = CertificateAuthority::bootstrap(name, key, signer.clone(), 1_000, 10_000_000)
            .unwrap();
        (ca, signer)
    }

    /// A trustd service backed by the real CA, plus the CA signer for minting.
    fn service() -> (SecurityService<EcdsaP256Signer>, EcdsaP256Signer) {
        let (ca, signer) = ca("oyatie-cell-7-ca");
        (SecurityService::new(JoinToken::new(JOIN_TOKEN).unwrap(), ca), signer)
    }

    fn trusted_bundle(
        svc: &SecurityService<EcdsaP256Signer>,
        ca_signer: &EcdsaP256Signer,
    ) -> TrustBundle<EcdsaP256Signer> {
        let mut bundle = TrustBundle::new();
        bundle
            .add_anchor(svc.ca_certificate().clone(), ca_signer.clone())
            .unwrap();
        bundle
    }

    /// Issue a real (trusted) workload SVID leaf — REAL X.509 DER — for `uri`.
    fn issue_leaf(
        svc: &mut SecurityService<EcdsaP256Signer>,
        ca_signer: &EcdsaP256Signer,
        uri: &str,
    ) -> Vec<u8> {
        let wl = EcdsaP256Signer::generate().unwrap();
        let key = KeyPair::new(wl.private_key_der(), wl.public_key_spki_der());
        let csr = CertificateSigningRequest::for_workload("wl", uri, &key, 3_600);
        let req = CertificateRequest {
            join_token: JOIN_TOKEN.to_string(),
            csr,
        };
        let resp = svc.handle_certificate(&req, &key, 2_000).unwrap();
        der::encode_leaf_der(&resp.identity.certificate, &wl, svc.ca_certificate(), ca_signer)
            .unwrap()
    }

    // RED-fixture: boot_refuses_without_trust_bundle.
    #[test]
    fn boot_refuses_without_trust_bundle() {
        let empty: TrustBundle<EcdsaP256Signer> = TrustBundle::new();
        assert_eq!(
            SpiffeCallerAuth::new(&empty).map(|_| ()).unwrap_err(),
            MtlsBootError::TrustBundleEmpty
        );
    }

    // RED-fixture: no_client_cert_denied.
    #[test]
    fn no_client_cert_denied() {
        let (svc, ca_signer) = service();
        let bundle = trusted_bundle(&svc, &ca_signer);
        let pep = SpiffeCallerAuth::new(&bundle).unwrap();
        let rej = pep.authenticate_caller(None, "ten_acme", 2_500).unwrap_err();
        assert_eq!(rej, CallerAuthRejection::NoClientCert);
        assert_eq!(rej.to_grpc_status().code(), Code::PermissionDenied);
        assert_eq!(rej.rest_status_code(), 403);
    }

    // RED-fixture: forged_svid_rejected (untrusted-issuer leaf → PermissionDenied).
    #[test]
    fn forged_svid_rejected() {
        let (svc, ca_signer) = service();
        let bundle = trusted_bundle(&svc, &ca_signer);
        let pep = SpiffeCallerAuth::new(&bundle).unwrap();
        // Mint a REAL leaf from a ROGUE real CA the bundle does not trust; the
        // verifier rejects it on real SIGNATURE verification, not a name check.
        let (mut rogue, rogue_signer) = ca("rogue-ca");
        let wl = EcdsaP256Signer::generate().unwrap();
        let key = KeyPair::new(wl.private_key_der(), wl.public_key_spki_der());
        let csr = CertificateSigningRequest::for_workload(
            "evil",
            "spiffe://oyatie.cell-7/tenant/ten_acme/evil",
            &key,
            3_600,
        );
        let leaf = rogue.sign_csr(&csr, 2_000).unwrap();
        let forged =
            der::encode_leaf_der(&leaf, &wl, rogue.certificate(), &rogue_signer).unwrap();
        let rej = pep
            .authenticate_caller(Some(&forged), "ten_acme", 2_500)
            .unwrap_err();
        assert!(matches!(rej, CallerAuthRejection::UntrustedSvid { .. }));
        assert_eq!(rej.to_grpc_status().code(), Code::PermissionDenied);
    }

    // RED-fixture: expired_svid_denied.
    #[test]
    fn expired_svid_denied() {
        let (mut svc, ca_signer) = service();
        let leaf = issue_leaf(&mut svc, &ca_signer, "spiffe://oyatie.cell-7/tenant/ten_acme/wl"); // [2000,5600)
        let bundle = trusted_bundle(&svc, &ca_signer);
        let pep = SpiffeCallerAuth::new(&bundle).unwrap();
        let rej = pep
            .authenticate_caller(Some(&leaf), "ten_acme", 6_000)
            .unwrap_err();
        assert_eq!(rej, CallerAuthRejection::ExpiredSvid);
        assert_eq!(rej.to_grpc_status().code(), Code::PermissionDenied);
    }

    // RED-fixture: tenant_binding_enforced (THE #717 closure).
    #[test]
    fn tenant_binding_enforced() {
        let (mut svc, ca_signer) = service();
        // SVID authorizes ten_acme.
        let leaf = issue_leaf(
            &mut svc,
            &ca_signer,
            "spiffe://oyatie.cell-7/tenant/ten_acme/secrets-sync",
        );
        let bundle = trusted_bundle(&svc, &ca_signer);
        let pep = SpiffeCallerAuth::new(&bundle).unwrap();

        // Matching tenant → ALLOW, and the bound tenant is the SVID's.
        let bound = pep
            .authenticate_caller(Some(&leaf), "ten_acme", 2_500)
            .unwrap();
        assert_eq!(bound.as_str(), "ten_acme");

        // Request body asks for ten_globex → DENY (the spoof attempt).
        let rej = pep
            .authenticate_caller(Some(&leaf), "ten_globex", 2_500)
            .unwrap_err();
        assert_eq!(
            rej,
            CallerAuthRejection::TenantMismatch {
                svid_tenant: "ten_acme".to_string(),
                requested_tenant: "ten_globex".to_string(),
            }
        );
        assert_eq!(rej.to_grpc_status().code(), Code::PermissionDenied);
    }

    // RED-fixture: issuance_policy_rejects_ca_leaf — regression guard that a
    // CA-capable leaf is never minted as a workload SVID (ca.rs approve()).
    #[test]
    fn issuance_policy_rejects_ca_leaf() {
        let (mut svc, _ca_signer) = service();
        let wl = EcdsaP256Signer::generate().unwrap();
        let key = KeyPair::new(wl.private_key_der(), wl.public_key_spki_der());
        let mut csr = CertificateSigningRequest::for_workload(
            "evil",
            "spiffe://oyatie.cell-7/platform/evil",
            &key,
            3_600,
        );
        csr.usage = CertUsage::CertificateAuthority;
        let req = CertificateRequest {
            join_token: JOIN_TOKEN.to_string(),
            csr,
        };
        assert_eq!(
            svc.handle_certificate(&req, &key, 2_000).unwrap_err().kind(),
            "csr_rejected"
        );
    }

    // A platform SVID is rejected when it tries to act as a tenant.
    #[test]
    fn platform_svid_cannot_act_as_tenant() {
        let (mut svc, ca_signer) = service();
        let leaf = issue_leaf(&mut svc, &ca_signer, "spiffe://oyatie.cell-7/platform/cloud-iam-pdp");
        let bundle = trusted_bundle(&svc, &ca_signer);
        let pep = SpiffeCallerAuth::new(&bundle).unwrap();
        let rej = pep
            .authenticate_caller(Some(&leaf), "ten_acme", 2_500)
            .unwrap_err();
        assert_eq!(rej, CallerAuthRejection::PlatformSvidCannotActAsTenant);
        assert_eq!(rej.to_grpc_status().code(), Code::PermissionDenied);
    }

    // A malformed request-body tenant is rejected (never reaches the SVID body).
    #[test]
    fn malformed_request_tenant_denied() {
        let (mut svc, ca_signer) = service();
        let leaf = issue_leaf(&mut svc, &ca_signer, "spiffe://oyatie.cell-7/tenant/ten_acme/wl");
        let bundle = trusted_bundle(&svc, &ca_signer);
        let pep = SpiffeCallerAuth::new(&bundle).unwrap();
        let rej = pep
            .authenticate_caller(Some(&leaf), "not-a-tenant", 2_500)
            .unwrap_err();
        assert_eq!(rej, CallerAuthRejection::MalformedRequestTenant);
    }

    // An undecodable leaf is an untrusted DENY, never a panic or fall-through.
    #[test]
    fn garbage_leaf_is_untrusted_deny() {
        let (svc, ca_signer) = service();
        let bundle = trusted_bundle(&svc, &ca_signer);
        let pep = SpiffeCallerAuth::new(&bundle).unwrap();
        let rej = pep
            .authenticate_caller(Some(b"garbage-not-a-real-der-leaf"), "ten_acme", 2_500)
            .unwrap_err();
        assert!(matches!(rej, CallerAuthRejection::UntrustedSvid { .. }));
    }

    // Belt-and-suspenders: a node cert (no URI SAN) from the TRUSTED CA, minted as
    // REAL DER, is still rejected — chain-valid is necessary but not sufficient; a
    // SPIFFE identity must be present.
    #[test]
    fn trusted_but_non_svid_leaf_denied() {
        let (mut svc, ca_signer) = service();
        let node = EcdsaP256Signer::generate().unwrap();
        let key = KeyPair::new(node.private_key_der(), node.public_key_spki_der());
        let csr = CertificateSigningRequest::for_node("node-1", &key, CertUsage::ClientAuth, 3_600);
        let req = CertificateRequest {
            join_token: JOIN_TOKEN.to_string(),
            csr,
        };
        let resp = svc.handle_certificate(&req, &key, 2_000).unwrap();
        let leaf =
            der::encode_leaf_der(&resp.identity.certificate, &node, svc.ca_certificate(), &ca_signer)
                .unwrap();
        let bundle = trusted_bundle(&svc, &ca_signer);
        let pep = SpiffeCallerAuth::new(&bundle).unwrap();
        let rej = pep
            .authenticate_caller(Some(&leaf), "ten_acme", 2_500)
            .unwrap_err();
        assert!(matches!(rej, CallerAuthRejection::MalformedSvid { .. }));
    }

    // Compile-time-ish guard: a verified Certificate value is reachable through
    // the typed verifier core too (used by future in-process PEP wiring).
    #[allow(dead_code)]
    fn _typed_core_is_reachable(cert: &Certificate, bundle: &TrustBundle<EcdsaP256Signer>) {
        let v = TrustdSvidVerifier::new(bundle);
        let _ = v.verify_certificate(cert, 0);
    }
}
