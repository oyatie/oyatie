//! Workload X.509-SVID trustd adapter (G002 slice-1 + slice-1b-i; ADR-0561).
//!
//! Implements the [`iam_identity_workload_svid_kernel`] issuer/verifier ports
//! over the `os-trustd-domain` CA stack with REAL X.509 crypto:
//!
//! - [`TrustdSvidIssuer`] issues an SVID by driving
//!   [`SecurityService::handle_certificate`] with the `for_workload` URI-SAN CSR
//!   shape (the SPIFFE id becomes the leaf's single URI SAN), join-token gated
//!   exactly as node issuance is, then minting the REAL ASN.1 DER leaf (real
//!   ECDSA-P256 signature) via [`os_trustd_domain::der`].
//! - [`TrustdSvidVerifier`] verifies a presented peer leaf — REAL DER — against a
//!   [`TrustBundle`] (real signature + validity) with `x509-parser`
//!   (`verify-aws`, NO ring), and extracts its single SPIFFE URI SAN, parsing it
//!   into a cell-rooted [`SpiffeId`].
//!
//! ## Real-crypto boundary (ADR-0561 slice-1b-i lands here; slice-1b-ii deferred)
//!
//! Slice-1b-i (THIS surface) makes issuance emit real DER and verification parse
//! real DER — the `SigningBackend` seam is unchanged, the leaves are
//! `EcdsaP256Signer`-signed real X.509. What remains DEFERRED to slice-1b-ii is
//! the LIVE transport: a rustls `ServerConfig` requiring a client cert on the PDP
//! listeners, handing the post-handshake peer-leaf DER to
//! [`SvidVerifier::verify_peer`]. The custom `ClientCertVerifier` + the rustls
//! wiring are slice-1b-ii (FRIC-1781490000), NOT this surface. Everything here is
//! fully testable in-process without K8s or a TLS terminator.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use os_trustd_domain::ca::CertificateSigningRequest;
use os_trustd_domain::certificate::Certificate;
use os_trustd_domain::der;
use os_trustd_domain::service::{CertificateRequest, SecurityService};
use os_trustd_domain::signer::EcdsaP256Signer;
use os_trustd_domain::x509::KeyPair;
use os_trustd_domain::{TrustBundle, TrustError};

use iam_identity_workload_svid_kernel::{
    IssueError, SpiffeId, SvidRequest, SvidVerifier, VerifyError, WorkloadIdentityIssuer, X509Svid,
};

pub mod leaf_der;

use leaf_der::LeafVerifyError;

/// Issues workload SVIDs over a trustd [`SecurityService`] backed by a REAL
/// ECDSA-P256 CA signer.
///
/// Holds the join token the issuance call is gated on, the workload's REAL
/// [`EcdsaP256Signer`] (whose public half the CSR carries and whose private half
/// signs the workload's own future handshakes), and the CA's [`EcdsaP256Signer`]
/// (used to mint the real leaf DER). [`issue`] takes the service by `&mut`
/// because minting advances the CA serial counter.
///
/// [`issue`]: TrustdSvidIssuer::issue
pub struct TrustdSvidIssuer<'a> {
    service: &'a mut SecurityService<EcdsaP256Signer>,
    join_token: String,
    workload_signer: EcdsaP256Signer,
    ca_signer: EcdsaP256Signer,
    workload_name: String,
}

impl<'a> TrustdSvidIssuer<'a> {
    /// Bind an issuer to a trustd service, the cluster join token, the workload's
    /// real signer, the issuing CA's real signer, and the workload CN.
    pub fn new(
        service: &'a mut SecurityService<EcdsaP256Signer>,
        join_token: impl Into<String>,
        workload_signer: EcdsaP256Signer,
        ca_signer: EcdsaP256Signer,
        workload_name: impl Into<String>,
    ) -> Self {
        Self {
            service,
            join_token: join_token.into(),
            workload_signer,
            ca_signer,
            workload_name: workload_name.into(),
        }
    }

    /// Issue an X.509-SVID for `request` as of `now`. Drives the join-token gated
    /// `handle_certificate` flow with the `for_workload` URI-SAN CSR, then mints
    /// the REAL ASN.1 DER leaf (real ECDSA signature over the TBS incl. the URI
    /// SAN) via [`der::encode_leaf_der`].
    ///
    /// # Errors
    /// [`IssueError`] when the trustd service refuses issuance (bad join token,
    /// policy rejection, CA expired) or real-DER minting fails. Fail-closed:
    /// never returns a partial SVID.
    pub fn issue(&mut self, request: &SvidRequest, now: u64) -> Result<X509Svid, IssueError> {
        // The CSR carries the workload's REAL public key (SPKI DER).
        let requester_key = KeyPair::new(
            self.workload_signer.private_key_der(),
            self.workload_signer.public_key_spki_der(),
        );
        let csr = CertificateSigningRequest::for_workload(
            self.workload_name.clone(),
            request.spiffe_id.as_uri(),
            &requester_key,
            request.ttl_secs,
        );
        let cert_request = CertificateRequest {
            join_token: self.join_token.clone(),
            csr,
        };
        let response = self
            .service
            .handle_certificate(&cert_request, &requester_key, now)
            .map_err(|err| IssueError::new(err.to_string()))?;
        let cert = response.identity.certificate;
        // Mint the REAL leaf DER, signed by the CA's real key.
        let leaf_der = der::encode_leaf_der(
            &cert,
            &self.workload_signer,
            self.service.ca_certificate(),
            &self.ca_signer,
        )
        .map_err(|err| IssueError::new(format!("real DER issuance failed: {err}")))?;
        Ok(X509Svid {
            spiffe_id: request.spiffe_id.clone(),
            leaf_der,
        })
    }
}

impl WorkloadIdentityIssuer for TrustdSvidIssuer<'_> {
    fn issue_x509_svid(&self, _request: &SvidRequest, _now: u64) -> Result<X509Svid, IssueError> {
        // A `&self` issuance is impossible against the trustd service (minting
        // advances the CA serial). The mutable [`TrustdSvidIssuer::issue`] is the
        // real path; this trait impl exists so a caller can hold the issuer behind
        // the kernel trait object for the read-only request shape.
        Err(IssueError::new(
            "use TrustdSvidIssuer::issue (the trait's &self shape cannot advance the CA serial)",
        ))
    }
}

/// Verifies presented peer SVIDs (REAL X.509 DER) against a trustd
/// [`TrustBundle`].
///
/// The bundle is the trust root: a leaf's REAL signature must verify under one of
/// the bundle's CA public keys, it must be within its validity window, and it
/// must carry exactly one SPIFFE URI SAN — only then is that URI parsed into a
/// [`SpiffeId`]. Generic over the bundle's `SigningBackend` so the same verifier
/// works for any anchor type; the cryptographic check itself reads the anchors'
/// real SubjectPublicKeyInfo DER, not the `SigningBackend` MAC.
pub struct TrustdSvidVerifier<'a, S: os_trustd_domain::signer::SigningBackend> {
    bundle: &'a TrustBundle<S>,
}

impl<'a, S: os_trustd_domain::signer::SigningBackend> TrustdSvidVerifier<'a, S> {
    /// Bind a verifier to a trust bundle.
    pub fn new(bundle: &'a TrustBundle<S>) -> Self {
        Self { bundle }
    }

    /// Verify an already-decoded trustd [`Certificate`] against the bundle and
    /// extract its SPIFFE id. This is the typed shape-model core retained for the
    /// in-process trustd-domain verification path (used by callers that already
    /// hold a parsed trustd `Certificate`); the byte-oriented [`verify_peer`] port
    /// uses the REAL-DER path below.
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
        self.bundle.verify_leaf(cert, now).map_err(|err| match err {
            TrustError::Expired(_) => VerifyError::Expired,
            other => VerifyError::UntrustedIssuer {
                detail: other.to_string(),
            },
        })?;
        match cert.sans.uris.as_slice() {
            [] => Err(VerifyError::NoSpiffeUriSan),
            [uri] => SpiffeId::parse(uri).map_err(VerifyError::MalformedSpiffeId),
            _ => Err(VerifyError::AmbiguousUriSan),
        }
    }
}

impl<S: os_trustd_domain::signer::SigningBackend> SvidVerifier
    for TrustdSvidVerifier<'_, S>
{
    fn verify_peer(&self, leaf_der: &[u8], now: u64) -> Result<SpiffeId, VerifyError> {
        // Parse + verify the REAL leaf DER against the bundle's CA public keys.
        let trusted = self.bundle.trusted_ca_spki_ders();
        let uri = leaf_der::verify_leaf_der(leaf_der, &trusted, now).map_err(map_leaf_err)?;
        // The chain + validity passed; now the URI must be a cell-rooted SVID.
        SpiffeId::parse(&uri).map_err(VerifyError::MalformedSpiffeId)
    }
}

/// Map a real-DER parse/verify failure onto the fail-closed kernel `VerifyError`.
fn map_leaf_err(err: LeafVerifyError) -> VerifyError {
    match err {
        LeafVerifyError::Expired => VerifyError::Expired,
        LeafVerifyError::NoUriSan => VerifyError::NoSpiffeUriSan,
        LeafVerifyError::AmbiguousUriSan => VerifyError::AmbiguousUriSan,
        LeafVerifyError::Undecodable(detail) => VerifyError::UntrustedIssuer {
            detail: format!("undecodable peer leaf: {detail}"),
        },
        LeafVerifyError::UntrustedIssuer(detail) => VerifyError::UntrustedIssuer { detail },
        // A CA-capable leaf or a leaf missing the clientAuth EKU is a fail-closed
        // untrusted deny: it presented material that is not a valid caller leaf.
        LeafVerifyError::CaCapableLeaf => VerifyError::UntrustedIssuer {
            detail: "peer leaf is CA-capable (basicConstraints cA TRUE); a CA must not authenticate as a workload".to_string(),
        },
        LeafVerifyError::MissingClientAuthEku => VerifyError::UntrustedIssuer {
            detail: "peer leaf does not carry the clientAuth extended key usage required of a caller".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_trustd_domain::ca::CertificateAuthority;
    use os_trustd_domain::certificate::CertUsage;
    use os_trustd_domain::JoinToken;

    const JOIN_TOKEN: &str = "clusterid.clustersecret";

    /// A REAL-crypto trustd service + the CA signer that anchors it. The CA's
    /// trustd `KeyPair` carries the CA's real PKCS#8 + SPKI so the bundle anchor's
    /// `public_key_der` is the real CA SubjectPublicKeyInfo.
    fn real_service() -> (SecurityService<EcdsaP256Signer>, EcdsaP256Signer) {
        let ca_signer = EcdsaP256Signer::generate().unwrap();
        let ca_key = KeyPair::new(ca_signer.private_key_der(), ca_signer.public_key_spki_der());
        let token = JoinToken::new(JOIN_TOKEN).unwrap();
        let ca = CertificateAuthority::bootstrap(
            "oyatie-cell-7-ca",
            ca_key,
            ca_signer.clone(),
            1_000,
            10_000_000,
        )
        .unwrap();
        (SecurityService::new(token, ca), ca_signer)
    }

    fn bundle_for(
        svc: &SecurityService<EcdsaP256Signer>,
        ca_signer: &EcdsaP256Signer,
    ) -> TrustBundle<EcdsaP256Signer> {
        let mut bundle = TrustBundle::new();
        bundle
            .add_anchor(svc.ca_certificate().clone(), ca_signer.clone())
            .unwrap();
        bundle
    }

    #[test]
    fn issue_then_verify_round_trips_spiffe_id() {
        let (mut svc, ca_signer) = real_service();
        let wl = EcdsaP256Signer::generate().unwrap();
        let uri = "spiffe://oyatie.cell-7/tenant/ten_acme/secrets-sync";
        let mut issuer = TrustdSvidIssuer::new(
            &mut svc,
            JOIN_TOKEN,
            wl,
            ca_signer.clone(),
            "secrets-sync",
        );
        let request = SvidRequest::new(SpiffeId::parse(uri).unwrap(), 3_600);
        let svid = issuer.issue(&request, 2_000).unwrap();
        assert_eq!(svid.spiffe_id.as_uri(), uri);

        let bundle = bundle_for(&svc, &ca_signer);
        let verifier = TrustdSvidVerifier::new(&bundle);
        // verify_peer parses the REAL leaf DER and recovers the SPIFFE id.
        let recovered = verifier.verify_peer(&svid.leaf_der, 2_500).unwrap();
        assert_eq!(recovered.as_uri(), uri);
    }

    #[test]
    fn forged_leaf_from_untrusted_ca_is_denied() {
        // Mint a REAL leaf from a ROGUE real CA carrying a valid SPIFFE URI; the
        // verifier (bundle trusts only the real CA) must reject it on SIGNATURE.
        let (mut rogue_svc, rogue_signer) = real_service();
        let wl = EcdsaP256Signer::generate().unwrap();
        let mut issuer = TrustdSvidIssuer::new(
            &mut rogue_svc,
            JOIN_TOKEN,
            wl,
            rogue_signer.clone(),
            "evil",
        );
        let request = SvidRequest::new(
            SpiffeId::parse("spiffe://oyatie.cell-7/tenant/ten_acme/evil").unwrap(),
            3_600,
        );
        let forged = issuer.issue(&request, 2_000).unwrap();

        let (svc, ca_signer) = real_service();
        let bundle = bundle_for(&svc, &ca_signer);
        let verifier = TrustdSvidVerifier::new(&bundle);
        let err = verifier.verify_peer(&forged.leaf_der, 2_500).unwrap_err();
        assert!(matches!(err, VerifyError::UntrustedIssuer { .. }));
    }

    #[test]
    fn expired_leaf_is_denied_distinctly() {
        let (mut svc, ca_signer) = real_service();
        let wl = EcdsaP256Signer::generate().unwrap();
        let mut issuer =
            TrustdSvidIssuer::new(&mut svc, JOIN_TOKEN, wl, ca_signer.clone(), "x");
        let request = SvidRequest::new(
            SpiffeId::parse("spiffe://oyatie.cell-7/platform/cloud-iam-pdp").unwrap(),
            3_600,
        );
        let svid = issuer.issue(&request, 2_000).unwrap(); // valid [2000, 5600)
        let bundle = bundle_for(&svc, &ca_signer);
        let verifier = TrustdSvidVerifier::new(&bundle);
        assert_eq!(
            verifier.verify_peer(&svid.leaf_der, 6_000).unwrap_err(),
            VerifyError::Expired
        );
    }

    #[test]
    fn leaf_without_uri_san_is_denied() {
        // A plain node cert (DNS SAN only, no URI) minted as REAL DER must not
        // authenticate as a workload — no SPIFFE identity is present.
        let (mut svc, ca_signer) = real_service();
        let node = EcdsaP256Signer::generate().unwrap();
        let node_key = KeyPair::new(node.private_key_der(), node.public_key_spki_der());
        let node_csr =
            CertificateSigningRequest::for_node("node-1", &node_key, CertUsage::ClientAuth, 3_600)
                .with_dns("node-1.cluster.local");
        let cert_request = CertificateRequest {
            join_token: JOIN_TOKEN.to_string(),
            csr: node_csr,
        };
        let resp = svc.handle_certificate(&cert_request, &node_key, 2_000).unwrap();
        let leaf_der = der::encode_leaf_der(
            &resp.identity.certificate,
            &node,
            svc.ca_certificate(),
            &ca_signer,
        )
        .unwrap();
        let bundle = bundle_for(&svc, &ca_signer);
        let verifier = TrustdSvidVerifier::new(&bundle);
        assert_eq!(
            verifier.verify_peer(&leaf_der, 2_500).unwrap_err(),
            VerifyError::NoSpiffeUriSan
        );
    }

    #[test]
    fn bad_join_token_refuses_issuance() {
        let (mut svc, ca_signer) = real_service();
        let wl = EcdsaP256Signer::generate().unwrap();
        let mut issuer =
            TrustdSvidIssuer::new(&mut svc, "clusterid.WRONG", wl, ca_signer, "x");
        let request = SvidRequest::new(
            SpiffeId::parse("spiffe://oyatie.cell-7/platform/x").unwrap(),
            3_600,
        );
        assert!(issuer.issue(&request, 2_000).is_err());
    }

    #[test]
    fn garbage_leaf_is_untrusted_deny() {
        let (svc, ca_signer) = real_service();
        let bundle = bundle_for(&svc, &ca_signer);
        let verifier = TrustdSvidVerifier::new(&bundle);
        let err = verifier
            .verify_peer(b"garbage-not-a-real-der-leaf", 2_500)
            .unwrap_err();
        assert!(matches!(err, VerifyError::UntrustedIssuer { .. }));
    }
}
