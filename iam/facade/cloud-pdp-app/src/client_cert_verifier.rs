//! rustls `ClientCertVerifier` that defers leaf trust to the SVID verifier
//! (G002 slice-1b-ii; ADR-0561, ADR-0506).
//!
//! This is the live-transport edge of the mTLS PEP. The rustls server handshake
//! REQUIRES a client certificate (`client_auth_mandatory == true`) and hands the
//! presented peer leaf to [`TrustdSvidVerifier::verify_peer`] over the trust
//! bundle — the EXACT same real-DER verification the in-process PEP uses, so the
//! transport check and the PEP check can never diverge. A leaf that does not
//! verify (untrusted issuer, bad signature, expired, malformed) aborts the
//! handshake with [`rustls::CertificateError`]; the tenant-binding decision then
//! runs at the application layer in [`crate::mtls::SpiffeCallerAuth`].
//!
//! ## Crypto provider (aws-lc-rs ONLY, NO ring)
//!
//! TLS-message signature verification (`verify_tls12_signature` /
//! `verify_tls13_signature` / `supported_verify_schemes`) delegates to the
//! aws-lc-rs provider's [`WebPkiSupportedAlgorithms`]
//! (`rustls::crypto::aws_lc_rs::default_provider().signature_verification_algorithms`).
//! This is distinct from the certificate-chain check, which the SVID verifier
//! performs with `x509-parser` on the aws-lc backend — neither path touches ring
//! (ADR-0506).
//!
//! ## `root_hint_subjects` is intentionally empty
//!
//! The hint list is advisory: it tells a client which CA subjects the server
//! would accept, to help it pick a cert. A workload always presents its single
//! SVID, so an empty list (RFC 8446: "send any client certificate you have") is
//! both correct and the right choice. The trust decision is made entirely in
//! [`Self::verify_client_cert`] -> the SVID verifier, never from the hints.
//! (`TrustBundle::trusted_ca_spki_ders` returns SubjectPublicKeyInfo, which is
//! the public key — not the DER subject distinguished name a hint requires — so
//! it must not be used here.)

use std::sync::Arc;

use rustls::crypto::WebPkiSupportedAlgorithms;
use rustls::pki_types::{CertificateDer, UnixTime};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::{DigitallySignedStruct, DistinguishedName, Error, SignatureScheme};

use iam_identity_workload_svid_kernel::SvidVerifier;
use iam_identity_workload_svid_trustd::TrustdSvidVerifier;
use os_trustd_domain::TrustBundle;
use os_trustd_domain::signer::SigningBackend;

/// A rustls client-certificate verifier over a trustd [`TrustBundle`].
///
/// Owns the trust bundle so the verifier has no borrow lifetime escaping into
/// the rustls `ServerConfig`. Construction REQUIRES a non-empty bundle (a server
/// that cannot prove a trust root must never accept a client), mirroring
/// [`crate::mtls::SpiffeCallerAuth::new`].
pub struct SvidClientCertVerifier<S: SigningBackend> {
    bundle: Arc<TrustBundle<S>>,
    supported_algs: WebPkiSupportedAlgorithms,
    /// Empty by construction — see the module note on `root_hint_subjects`.
    no_hints: Vec<DistinguishedName>,
}

// `ClientCertVerifier: Debug`, but neither `TrustBundle<S>` nor
// `WebPkiSupportedAlgorithms` is `Debug`; print only a static label (the bundle
// holds trust material that must not leak into logs anyway).
impl<S: SigningBackend> std::fmt::Debug for SvidClientCertVerifier<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SvidClientCertVerifier")
    }
}

impl<S: SigningBackend> SvidClientCertVerifier<S> {
    /// Build the verifier over a shared trust bundle, using the aws-lc-rs
    /// provider's signature-verification algorithms (NO ring).
    #[must_use]
    pub fn new(bundle: Arc<TrustBundle<S>>) -> Self {
        let supported_algs =
            rustls::crypto::aws_lc_rs::default_provider().signature_verification_algorithms;
        Self {
            bundle,
            supported_algs,
            no_hints: Vec::new(),
        }
    }
}

impl<S: SigningBackend + Send + Sync + 'static> ClientCertVerifier for SvidClientCertVerifier<S> {
    fn offer_client_auth(&self) -> bool {
        true
    }

    fn client_auth_mandatory(&self) -> bool {
        // Fail-closed: a connection without a client cert is refused at the
        // handshake, never silently downgraded to anonymous.
        true
    }

    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &self.no_hints
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        now: UnixTime,
    ) -> Result<ClientCertVerified, Error> {
        // Defer the leaf trust decision to the SVID verifier: real chain +
        // signature + validity against the bundle's CA SPKIs, then SPIFFE-id
        // parse. Any failure is an InvalidCertificate handshake abort — never a
        // silent accept (fail-closed). The tenant binding runs later in the PEP.
        let verifier = TrustdSvidVerifier::new(&self.bundle);
        match verifier.verify_peer(end_entity.as_ref(), now.as_secs()) {
            Ok(_spiffe_id) => Ok(ClientCertVerified::assertion()),
            Err(_err) => Err(Error::InvalidCertificate(
                rustls::CertificateError::ApplicationVerificationFailure,
            )),
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, Error> {
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.supported_algs)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.supported_algs)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.supported_algs.supported_schemes()
    }
}
