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
pub mod der;
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
pub use der::{IssuedDer, encode_ca_der, encode_leaf_der, issue_der};
pub use error::{Result, TrustError};
pub use persistence::KeySealer;
pub use service::{CertificateRequest, CertificateResponse, SecurityService};
pub use signer::{EcdsaP256Signer, SigningBackend};
#[cfg(any(test, feature = "modeled-crypto"))]
pub use signer::InMemorySigner;
pub use token::JoinToken;
pub use x509::{DistinguishedName, KeyPair, PEMEncoded, PEMLabel, SubjectAltNames, Validity};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use os_kernel::Role;

    /// The feature must never become a DEFAULT feature.
    ///
    /// Non-defaultness is the load-bearing property of the whole gate: the
    /// `cfg` above only strips `InMemorySigner` and `KeyPair::from_seed` while
    /// the feature is off, and a `default` entry turns it on for every
    /// `cargo build` in the workspace. Before this test that property was
    /// carried by a comment in `Cargo.toml` and nothing else, and buck2 cannot
    /// notice the change because buck2 features come from the target attribute
    /// and never consult `Cargo.toml`.
    ///
    /// Sibling of the same guard in `os-secrets-domain` and
    /// `os-cluster-mgmt-domain`. Proven to fire by adding
    /// `default = ["modeled-crypto"]` to `[features]`.
    #[test]
    fn cargo_manifest_declares_no_default_feature() {
        let features: Vec<&str> = include_str!("../Cargo.toml")
            .lines()
            .skip_while(|l| l.trim() != "[features]")
            .skip(1)
            .take_while(|l| !l.trim_start().starts_with('['))
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();

        assert_eq!(
            features,
            ["modeled-crypto = []"],
            "Cargo.toml [features] must not declare `default`: modeled-crypto \
             is non-default and that is the whole barrier"
        );
    }

    /// No buck2 target in this package may enable the model.
    ///
    /// The sibling of `no_production_buck_target_enables_the_model` in
    /// `os-secrets-domain`, and the reason it has to exist here too: that crate
    /// closed the INSTANCE and this one was left open, which is the exact class
    /// failure this branch is about. `Cargo.toml` and the `cfg` attributes are
    /// half the barrier; the other half is the build graph, and buck2 features
    /// come from the target attribute, so a `features = ["modeled-crypto"]` on
    /// the production `rust_library` would hand `InMemorySigner` and
    /// `KeyPair::from_seed` to every consumer without touching a line of Rust
    /// or of the manifest — invisible to both existing guards.
    ///
    /// Zero, not one: unlike `os-secrets-domain` there is no modeled *target*
    /// here. The gate is `cfg(any(test, feature = "modeled-crypto"))`, so the
    /// `rust_test` already sees the modeled items through `test` and no target
    /// in this package ever needs the feature. Any occurrence is therefore a
    /// production one.
    ///
    /// The `contains` line below is what makes this anti-vacuous: an assertion
    /// that a count is zero passes on an empty string, so the test first proves
    /// it read the BUCK file it meant to read.
    ///
    /// Proven to fire, by mutation: adding `features = ["modeled-crypto"]` to
    /// the `os-trustd-domain` `rust_library` gives
    ///
    /// ```text
    /// assertion `left == right` failed: no buck2 target in this package may
    /// enable modeled-crypto
    ///   left: 1
    ///  right: 0
    /// ```
    #[test]
    fn no_buck_target_enables_the_model() {
        let buck: String = include_str!("../BUCK")
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            buck.contains("os-trustd-domain"),
            "read the wrong BUCK file: a zero-count assertion below would pass vacuously"
        );
        assert_eq!(
            buck.matches("modeled-crypto").count(),
            0,
            "no buck2 target in this package may enable modeled-crypto: the crate gate \
             is cfg(any(test, feature = \"modeled-crypto\")), so the rust_test already \
             sees the modeled items via `test`, and the only thing a feature attribute \
             here could do is put them in a production library"
        );
    }

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
