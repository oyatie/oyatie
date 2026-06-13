//! Workload X.509-SVID trustd adapter (G002 slice-1; ADR-0561).
//!
//! Implements the [`oya_identity_workload_svid_kernel`] issuer/verifier ports
//! over the `oya-cloud-os-trustd-domain` CA stack:
//!
//! - [`TrustdSvidIssuer`] issues an SVID by driving
//!   [`SecurityService::handle_certificate`] with the `for_workload` URI-SAN
//!   CSR shape (the SPIFFE id becomes the leaf's single URI SAN), join-token
//!   gated exactly as node issuance is.
//! - [`TrustdSvidVerifier`] verifies a presented peer leaf against a
//!   [`TrustBundle`] (chain + validity + signature) and extracts its single
//!   SPIFFE URI SAN, parsing it into a cell-rooted [`SpiffeId`].
//!
//! ## Fidelity boundary (ADR-0561 slice-1b deferral)
//!
//! `oya-cloud-os-trustd-domain` is a faithful SHAPE model of Talos PKI: its
//! "DER" is a deterministic encoding and its signatures are keyed hashes, not
//! real ASN.1/ECDSA. This adapter therefore operates at that same shape
//! fidelity: a presented leaf is the trustd [`Certificate`] serialized through
//! the self-contained [`leaf_codec`] (a stand-in for DER, owned end-to-end by
//! this adapter). The REAL rustls transport handshake and the cloud-kms signer
//! swap are the explicitly DEFERRED slice-1b, not this slice — see ADR-0561.
//! Everything here is fully testable in-process without K8s, which is the
//! slice-1-core acceptance bar.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_cloud_os_trustd_domain::ca::CertificateSigningRequest;
use oya_cloud_os_trustd_domain::certificate::Certificate;
use oya_cloud_os_trustd_domain::service::{CertificateRequest, SecurityService};
use oya_cloud_os_trustd_domain::signer::SigningBackend;
use oya_cloud_os_trustd_domain::x509::KeyPair;
use oya_cloud_os_trustd_domain::{TrustBundle, TrustError};

use oya_identity_workload_svid_kernel::{
    IssueError, SpiffeId, SvidRequest, SvidVerifier, VerifyError, WorkloadIdentityIssuer, X509Svid,
};

pub mod leaf_codec;

/// Issues workload SVIDs over a trustd [`SecurityService`].
///
/// Holds the join token the issuance call is gated on plus the requester key
/// pair (the workload's local private key, whose public half the CSR carries).
/// Construction does not bind the service mutably; [`issue_x509_svid`] takes the
/// service by `&mut` because minting advances the CA serial counter.
///
/// [`issue_x509_svid`]: WorkloadIdentityIssuer::issue_x509_svid
pub struct TrustdSvidIssuer<'a, S: SigningBackend> {
    service: &'a mut SecurityService<S>,
    join_token: String,
    requester_key: KeyPair,
    workload_name: String,
}

impl<'a, S: SigningBackend> TrustdSvidIssuer<'a, S> {
    /// Bind an issuer to a trustd service, the cluster join token, the
    /// requester key pair, and the workload CN to stamp on the leaf.
    pub fn new(
        service: &'a mut SecurityService<S>,
        join_token: impl Into<String>,
        requester_key: KeyPair,
        workload_name: impl Into<String>,
    ) -> Self {
        Self {
            service,
            join_token: join_token.into(),
            requester_key,
            workload_name: workload_name.into(),
        }
    }
}

impl<S: SigningBackend> WorkloadIdentityIssuer for TrustdSvidIssuer<'_, S> {
    fn issue_x509_svid(&self, _request: &SvidRequest, _now: u64) -> Result<X509Svid, IssueError> {
        // A `&self` issuance is impossible against the trustd service (minting
        // advances the CA serial). The port is implemented on the mutable
        // `issue` method below; this trait impl exists so callers can hold the
        // issuer behind the kernel trait object for the read-only request shape
        // and is wired to the mutable path by the composition root.
        Err(IssueError::new(
            "use TrustdSvidIssuer::issue (the trait's &self shape cannot advance the CA serial)",
        ))
    }
}

impl<S: SigningBackend> TrustdSvidIssuer<'_, S> {
    /// Issue an X.509-SVID for `request` as of `now`. Drives the join-token
    /// gated `handle_certificate` flow with the `for_workload` URI-SAN CSR.
    ///
    /// # Errors
    /// [`IssueError`] when the trustd service refuses issuance (bad join token,
    /// policy rejection, CA expired). Fail-closed: never returns a partial SVID.
    pub fn issue(&mut self, request: &SvidRequest, now: u64) -> Result<X509Svid, IssueError> {
        let csr = CertificateSigningRequest::for_workload(
            self.workload_name.clone(),
            request.spiffe_id.as_uri(),
            &self.requester_key,
            request.ttl_secs,
        );
        let cert_request = CertificateRequest {
            join_token: self.join_token.clone(),
            csr,
        };
        let response = self
            .service
            .handle_certificate(&cert_request, &self.requester_key, now)
            .map_err(|err| IssueError::new(err.to_string()))?;
        let cert = response.identity.certificate;
        Ok(X509Svid {
            spiffe_id: request.spiffe_id.clone(),
            leaf_der: leaf_codec::encode(&cert),
        })
    }
}

/// Verifies presented peer SVIDs against a trustd [`TrustBundle`].
///
/// The bundle is the trust root: a leaf must chain to a trusted anchor, be
/// within its validity window, and carry a signature the anchor accepts. Only
/// then is its single SPIFFE URI SAN extracted and parsed.
pub struct TrustdSvidVerifier<'a, S: SigningBackend> {
    bundle: &'a TrustBundle<S>,
}

impl<'a, S: SigningBackend> TrustdSvidVerifier<'a, S> {
    /// Bind a verifier to a trust bundle.
    pub fn new(bundle: &'a TrustBundle<S>) -> Self {
        Self { bundle }
    }

    /// Verify an already-decoded trustd [`Certificate`] against the bundle and
    /// extract its SPIFFE id. This is the typed core the byte-oriented
    /// [`verify_peer`] port delegates to.
    ///
    /// [`verify_peer`]: SvidVerifier::verify_peer
    ///
    /// # Errors
    /// [`VerifyError`] on an untrusted/expired chain, a missing/ambiguous URI
    /// SAN, or a malformed SPIFFE id.
    pub fn verify_certificate(
        &self,
        cert: &Certificate,
        now: u64,
    ) -> Result<SpiffeId, VerifyError> {
        // Chain + validity + signature. Map an expiry to the dedicated DENY
        // variant so the PEP can log it distinctly; every other chain failure
        // is an untrusted-issuer DENY.
        self.bundle.verify_leaf(cert, now).map_err(|err| match err {
            TrustError::Expired(_) => VerifyError::Expired,
            other => VerifyError::UntrustedIssuer {
                detail: other.to_string(),
            },
        })?;
        // A SPIFFE SVID carries EXACTLY ONE URI SAN. Zero → unauthenticated,
        // more than one → ambiguous identity (both fail-closed).
        match cert.sans.uris.as_slice() {
            [] => Err(VerifyError::NoSpiffeUriSan),
            [uri] => SpiffeId::parse(uri).map_err(VerifyError::MalformedSpiffeId),
            _ => Err(VerifyError::AmbiguousUriSan),
        }
    }
}

impl<S: SigningBackend> SvidVerifier for TrustdSvidVerifier<'_, S> {
    fn verify_peer(&self, leaf_der: &[u8], now: u64) -> Result<SpiffeId, VerifyError> {
        // Decode the presented leaf bytes (DER stand-in). A leaf that does not
        // decode is an untrusted-issuer DENY — it never chains to anything.
        let cert = leaf_codec::decode(leaf_der).map_err(|detail| VerifyError::UntrustedIssuer {
            detail: format!("undecodable peer leaf: {detail}"),
        })?;
        self.verify_certificate(&cert, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_cloud_os_trustd_domain::ca::CertificateAuthority;
    use oya_cloud_os_trustd_domain::certificate::CertUsage;
    use oya_cloud_os_trustd_domain::signer::InMemorySigner;
    use oya_cloud_os_trustd_domain::JoinToken;

    const JOIN_TOKEN: &str = "clusterid.clustersecret";

    fn service() -> SecurityService<InMemorySigner> {
        let token = JoinToken::new(JOIN_TOKEN).unwrap();
        let ca = CertificateAuthority::bootstrap(
            "oyatie-cell-7-ca",
            KeyPair::from_seed(b"ca-seed"),
            InMemorySigner::from_seed("ca-seed"),
            1_000,
            10_000_000,
        )
        .unwrap();
        SecurityService::new(token, ca)
    }

    fn bundle_for(svc: &SecurityService<InMemorySigner>) -> TrustBundle<InMemorySigner> {
        let mut bundle = TrustBundle::new();
        bundle
            .add_anchor(svc.ca_certificate().clone(), InMemorySigner::from_seed("ca-seed"))
            .unwrap();
        bundle
    }

    #[test]
    fn issue_then_verify_round_trips_spiffe_id() {
        let mut svc = service();
        let wl_key = KeyPair::from_seed(b"wl-pdp");
        let uri = "spiffe://oyatie.cell-7/tenant/ten_acme/secrets-sync";
        let mut issuer =
            TrustdSvidIssuer::new(&mut svc, JOIN_TOKEN, wl_key.clone(), "secrets-sync");
        let request = SvidRequest::new(SpiffeId::parse(uri).unwrap(), 3_600);
        let svid = issuer.issue(&request, 2_000).unwrap();
        assert_eq!(svid.spiffe_id.as_uri(), uri);

        let bundle = bundle_for(&svc);
        let verifier = TrustdSvidVerifier::new(&bundle);
        let recovered = verifier.verify_peer(&svid.leaf_der, 2_500).unwrap();
        assert_eq!(recovered.as_uri(), uri);
    }

    #[test]
    fn forged_leaf_from_untrusted_ca_is_denied() {
        // Mint a leaf from a ROGUE CA carrying a valid-looking SPIFFE URI; the
        // verifier (bundle trusts only the real CA) must reject it.
        let mut rogue = CertificateAuthority::bootstrap(
            "rogue-ca",
            KeyPair::from_seed(b"rogue"),
            InMemorySigner::from_seed("rogue"),
            1_000,
            10_000_000,
        )
        .unwrap();
        let wl_key = KeyPair::from_seed(b"evil");
        let csr = CertificateSigningRequest::for_workload(
            "evil",
            "spiffe://oyatie.cell-7/tenant/ten_acme/evil",
            &wl_key,
            3_600,
        );
        let forged = rogue.sign_csr(&csr, 2_000).unwrap();

        let svc = service();
        let bundle = bundle_for(&svc);
        let verifier = TrustdSvidVerifier::new(&bundle);
        let err = verifier
            .verify_peer(&leaf_codec::encode(&forged), 2_500)
            .unwrap_err();
        assert!(matches!(err, VerifyError::UntrustedIssuer { .. }));
    }

    #[test]
    fn expired_leaf_is_denied_distinctly() {
        let mut svc = service();
        let wl_key = KeyPair::from_seed(b"wl-x");
        let mut issuer = TrustdSvidIssuer::new(&mut svc, JOIN_TOKEN, wl_key, "x");
        let request = SvidRequest::new(
            SpiffeId::parse("spiffe://oyatie.cell-7/platform/cloud-iam-pdp").unwrap(),
            3_600,
        );
        let svid = issuer.issue(&request, 2_000).unwrap(); // valid [2000, 5600)
        let bundle = bundle_for(&svc);
        let verifier = TrustdSvidVerifier::new(&bundle);
        // After not_after the dedicated Expired DENY fires.
        assert_eq!(
            verifier.verify_peer(&svid.leaf_der, 6_000).unwrap_err(),
            VerifyError::Expired
        );
    }

    #[test]
    fn leaf_without_uri_san_is_denied() {
        // A plain node cert (DNS SAN only, no URI) must not authenticate as a
        // workload — no SPIFFE identity is present.
        let mut svc = service();
        let key = KeyPair::from_seed(b"node");
        let node_csr =
            CertificateSigningRequest::for_node("node-1", &key, CertUsage::ClientAuth, 3_600)
                .with_dns("node-1.cluster.local");
        let cert_request = CertificateRequest {
            join_token: JOIN_TOKEN.to_string(),
            csr: node_csr,
        };
        let resp = svc.handle_certificate(&cert_request, &key, 2_000).unwrap();
        let bundle = bundle_for(&svc);
        let verifier = TrustdSvidVerifier::new(&bundle);
        assert_eq!(
            verifier
                .verify_peer(&leaf_codec::encode(&resp.identity.certificate), 2_500)
                .unwrap_err(),
            VerifyError::NoSpiffeUriSan
        );
    }

    #[test]
    fn bad_join_token_refuses_issuance() {
        let mut svc = service();
        let wl_key = KeyPair::from_seed(b"wl");
        let mut issuer = TrustdSvidIssuer::new(&mut svc, "clusterid.WRONG", wl_key, "x");
        let request = SvidRequest::new(
            SpiffeId::parse("spiffe://oyatie.cell-7/platform/x").unwrap(),
            3_600,
        );
        assert!(issuer.issue(&request, 2_000).is_err());
    }
}
