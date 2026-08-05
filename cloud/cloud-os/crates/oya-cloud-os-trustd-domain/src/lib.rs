//! # talos-trustd
//!
//! A faithful Rust model of the Talos (`siderolabs/talos`) `trustd` subsystem:
//! the cluster certificate authority, PKI primitives, certificate
//! issuance/signing, the `SecurityService` gRPC API surface, cluster join-token
//! verification, and node certificate rotation.
//!
//! `trustd` is the small daemon that runs on Talos control-plane nodes and lets
//! a freshly-booted worker obtain a signed certificate: the worker authenticates
//! with the shared cluster join token, submits a CSR, and trustd (holding the CA
//! private key) returns a signed leaf plus the CA chain.
//!
//! ## Layering
//!
//! * [`x509`] — PEM/DER envelopes, distinguished names, validity windows, key
//!   pairs.
//! * [`signer`] — the [`signer::SigningBackend`] trait modeling the crypto
//!   boundary, with a deterministic in-memory implementation for tests.
//! * [`certificate`] — the issued [`certificate::Certificate`] and its bundled
//!   [`certificate::IssuedIdentity`].
//! * [`token`] — cluster [`token::JoinToken`] validation and constant-time
//!   verification.
//! * [`ca`] — the [`ca::CertificateAuthority`], CSR type, and issuance policy.
//! * [`service`] — the [`service::SecurityService`] API surface tying it all
//!   together, plus authorization and renewal.
//!
//! Real cryptography, networking, and gRPC transport are modeled as boundaries
//! (traits / plain request-response types); everything here is pure, offline,
//! and depends only on the standard library and `talos-core`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
// These pedantic lints are pure documentation/annotation churn for a crate of
// small infallible accessors and `Result`-returning constructors; annotating
// every getter with `#[must_use]` and every fallible fn with an `# Errors`
// section adds noise without improving the code, so we opt out crate-wide.
#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::missing_errors_doc
)]

pub mod bundle;
pub mod ca;
pub mod certificate;
pub mod crl;
pub mod error;
pub mod persistence;
pub mod service;
pub mod signer;
pub mod token;
pub mod x509;

pub use bundle::TrustBundle;
pub use ca::{CertificateAuthority, CertificateSigningRequest, IssuancePolicy};
pub use certificate::{CertUsage, Certificate, IssuedIdentity};
pub use crl::{RevocationEntry, RevocationList, RevocationReason};
pub use error::{Result, TrustError};
pub use persistence::{KeySealer, StaticKeySealer};
pub use service::{CertificateRequest, CertificateResponse, SecurityService};
pub use signer::{InMemorySigner, SigningBackend};
pub use token::JoinToken;
pub use x509::{DistinguishedName, KeyPair, PEMEncoded, PEMLabel, SubjectAltNames, Validity};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use oya_cloud_os_kernel::Role;

    /// End-to-end: bootstrap a CA, stand up the service, have a node join with
    /// the cluster token, obtain a cert, get authorized, then renew it.
    #[test]
    fn full_join_and_renew_flow() {
        let token = JoinToken::new("kube01.s3cr3ttoken99").unwrap();
        let ca = CertificateAuthority::bootstrap(
            "talos-ca",
            KeyPair::from_seed(b"ca-root-key"),
            InMemorySigner::from_seed("ca-root-key"),
            1_000,
            10_000_000,
        )
        .unwrap();
        let mut svc = SecurityService::new(token, ca);

        let node_key = KeyPair::from_seed(b"worker-1-key");
        let csr = CertificateSigningRequest::for_node(
            "worker-1",
            &node_key,
            CertUsage::ClientAuth,
            3_600,
        )
        .requesting_role("os:reader")
        .with_dns("worker-1.cluster.local");

        let req = CertificateRequest {
            join_token: "kube01.s3cr3ttoken99".to_string(),
            csr,
        };
        let resp = svc.handle_certificate(&req, &node_key, 2_000).unwrap();
        let cert = resp.identity.certificate.clone();

        // The issued cert is authorized as a reader but not an admin.
        assert!(svc.require_role(&cert, Role::Reader, 2_500).is_ok());
        assert!(svc.require_role(&cert, Role::Admin, 2_500).is_err());

        // The SAN we requested is present.
        assert!(cert.sans.covers_dns("worker-1.cluster.local"));

        // Near expiry it renews and yields a fresh serial.
        let renewed = svc
            .renew_if_needed(&cert, &node_key, 5_000)
            .unwrap()
            .expect("should renew near expiry");
        assert!(renewed.certificate.serial > cert.serial);
        assert!(
            svc.require_role(&renewed.certificate, Role::Reader, 5_500)
                .is_ok()
        );
    }

    /// A node that presents the wrong join token is refused before any cert is
    /// minted.
    #[test]
    fn wrong_token_blocks_issuance() {
        let token = JoinToken::new("kube01.s3cr3ttoken99").unwrap();
        let ca = CertificateAuthority::bootstrap(
            "talos-ca",
            KeyPair::from_seed(b"ca-root-key"),
            InMemorySigner::from_seed("ca-root-key"),
            1_000,
            10_000_000,
        )
        .unwrap();
        let mut svc = SecurityService::new(token, ca);
        let node_key = KeyPair::from_seed(b"worker-1-key");
        let csr = CertificateSigningRequest::for_node(
            "worker-1",
            &node_key,
            CertUsage::ClientAuth,
            3_600,
        );
        let req = CertificateRequest {
            join_token: "kube01.WRONGTOKEN99".to_string(),
            csr,
        };
        assert_eq!(
            svc.handle_certificate(&req, &node_key, 2_000)
                .unwrap_err()
                .kind(),
            "token_mismatch"
        );
    }
}
