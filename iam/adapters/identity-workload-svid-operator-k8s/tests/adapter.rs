//! Adapter closure-proof + reconcile-actuation tests (G002 slice-1b-iii-c).
//!
//! The CRITICAL test is `produced_secret_leaf_verifies_against_produced_ca`: it
//! proves the Secret material the operator mints is INTERNALLY CONSISTENT in the
//! exact way the PDP consumer's verify path requires — the `tls.crt` leaf's REAL
//! ECDSA signature verifies under the REAL SubjectPublicKeyInfo extracted from the
//! produced `ca.crt`. This is the same SPKI-anchored real-signature check
//! `MtlsContext::from_path` + `TrustBundle::trusted_ca_spki_ders` perform; the
//! cross-crate keystone (a real rustls handshake) lives in
//! iam/facade/pdp-app/tests/main_boot_closure.rs.

use iam_identity_workload_svid_operator_k8s::{
    CA_CRT_KEY, SvidIssuanceBackend, SvidSecretMaterial, TLS_CRT_KEY, TLS_KEY_KEY, TLS_SECRET_TYPE,
    TrustdEcdsaIssuanceBackend, observed_secret_from_leaf_pem, run_reconcile_once, secret_manifest,
};
use iam_identity_workload_svid_operator_kernel::{Action, Clock, DesiredState, ObservedState};

use x509_parser::certificate::X509Certificate;
use x509_parser::pem::Pem;
use x509_parser::prelude::FromDer;

const JOIN_TOKEN: &str = "clusterid.clustersecret";

#[derive(Clone, Copy)]
struct FixedClock {
    now: u64,
}

impl Clock for FixedClock {
    fn now_epoch_seconds(&self) -> u64 {
        self.now
    }
}

fn desired() -> DesiredState {
    DesiredState {
        spiffe_id: "spiffe://oyatie.cell-7/platform/cloud-iam-pdp".to_owned(),
        ttl_secs: 3_600,
        rotation_window_secs: 600,
        secret_name: "cloud-iam-pdp-svid".to_owned(),
        secret_namespace: "cloud-iam".to_owned(),
    }
}

fn backend() -> TrustdEcdsaIssuanceBackend {
    TrustdEcdsaIssuanceBackend::bootstrap(
        "oyatie-cell-7-pdp-svid-ca",
        JOIN_TOKEN,
        1_000,
        10_000_000,
    )
    .expect("CA bootstrap")
}

/// Extract the single CERTIFICATE block DER from a PEM string.
fn first_cert_der(pem: &str) -> Vec<u8> {
    for block in Pem::iter_from_buffer(pem.as_bytes()) {
        let block = block.expect("PEM parse");
        if block.label == "CERTIFICATE" {
            return block.contents;
        }
    }
    panic!("no CERTIFICATE block in PEM");
}

#[test]
fn mint_produces_tls_crt_key_ca_crt_in_consumer_pem_shape() {
    let mut be = backend();
    let material = be.mint(&desired(), 2_000).expect("mint");
    // The three PEM members are present and well-labeled (the from_path contract).
    assert!(material.tls_crt_pem.contains("-----BEGIN CERTIFICATE-----"));
    assert!(material.tls_key_pem.contains("-----BEGIN PRIVATE KEY-----"));
    assert!(material.ca_crt_pem.contains("-----BEGIN CERTIFICATE-----"));
    // The leaf is valid for ttl_secs from issuance.
    assert_eq!(material.leaf_not_after_epoch_seconds, 2_000 + 3_600);
}

/// THE CLOSURE PROOF: the produced `tls.crt` leaf's REAL signature verifies under
/// the REAL SPKI extracted from the produced `ca.crt` — the exact anchor check the
/// PDP consumer's verify path performs. A husk (mismatched/empty material) fails
/// here.
#[test]
fn produced_secret_leaf_verifies_against_produced_ca() {
    let mut be = backend();
    let material = be.mint(&desired(), 2_000).expect("mint");

    let leaf_der = first_cert_der(&material.tls_crt_pem);
    let ca_der = first_cert_der(&material.ca_crt_pem);

    // The real CA SubjectPublicKeyInfo DER — the value the rustls verify path
    // consults (mirrors MtlsContext::from_path::ca_anchor_from_der).
    let (_rest, ca_cert) = X509Certificate::from_der(&ca_der).expect("CA DER parse");
    let ca_spki = ca_cert.public_key().raw.to_vec();

    // The leaf's REAL signature must verify under the CA SPKI.
    let (_lrest, leaf) = X509Certificate::from_der(&leaf_der).expect("leaf DER parse");
    let (_srest, spki) =
        x509_parser::x509::SubjectPublicKeyInfo::from_der(&ca_spki).expect("SPKI parse");
    leaf.verify_signature(Some(&spki))
        .expect("produced leaf must verify under the produced CA (closure proof)");

    // The leaf carries the platform SVID URI SAN (the PDP server identity).
    let san = leaf
        .subject_alternative_name()
        .expect("SAN ext")
        .expect("SAN present");
    let has_spiffe_uri = san.value.general_names.iter().any(|gn| {
        matches!(gn, x509_parser::extensions::GeneralName::URI(uri)
            if *uri == "spiffe://oyatie.cell-7/platform/cloud-iam-pdp")
    });
    assert!(
        has_spiffe_uri,
        "leaf must carry the PDP platform SVID URI SAN"
    );
}

/// A leaf forged from a DIFFERENT CA must NOT verify under this Secret's `ca.crt`
/// (negative control on the closure proof — proves the check is real, not a tautology).
#[test]
fn leaf_from_a_different_ca_does_not_verify_against_this_ca() {
    let mut be_a = backend();
    let mut be_b = backend(); // an independent CA
    let material_a = be_a.mint(&desired(), 2_000).expect("mint a");
    let material_b = be_b.mint(&desired(), 2_000).expect("mint b");

    let leaf_b_der = first_cert_der(&material_b.tls_crt_pem);
    let ca_a_der = first_cert_der(&material_a.ca_crt_pem);

    let (_r, ca_a) = X509Certificate::from_der(&ca_a_der).expect("ca a parse");
    let ca_a_spki = ca_a.public_key().raw.to_vec();
    let (_lr, leaf_b) = X509Certificate::from_der(&leaf_b_der).expect("leaf b parse");
    let (_sr, spki) =
        x509_parser::x509::SubjectPublicKeyInfo::from_der(&ca_a_spki).expect("spki parse");
    assert!(
        leaf_b.verify_signature(Some(&spki)).is_err(),
        "a leaf from CA-B must NOT verify under CA-A"
    );
}

#[test]
fn observed_secret_from_leaf_pem_recovers_leaf_expiry() {
    let mut be = backend();
    let material = be.mint(&desired(), 2_000).expect("mint");
    let observed = observed_secret_from_leaf_pem(&material.tls_crt_pem).expect("project");
    assert_eq!(observed.leaf_not_after_epoch_seconds, 2_000 + 3_600);
}

#[test]
fn secret_manifest_is_kubernetes_io_tls_with_three_data_members() {
    let material = SvidSecretMaterial {
        tls_crt_pem: "crt".to_owned(),
        tls_key_pem: "key".to_owned(),
        ca_crt_pem: "ca".to_owned(),
        leaf_not_after_epoch_seconds: 9_999,
    };
    let manifest = secret_manifest("cloud-iam-pdp-svid", "iam", &material);
    assert_eq!(manifest["type"], TLS_SECRET_TYPE);
    assert_eq!(manifest["metadata"]["name"], "cloud-iam-pdp-svid");
    assert_eq!(manifest["metadata"]["namespace"], "cloud-iam");
    // base64 of the three PEM members under the standard keys.
    use base64::Engine as _;
    let b64 = |s: &str| base64::engine::general_purpose::STANDARD.encode(s.as_bytes());
    assert_eq!(manifest["data"][TLS_CRT_KEY], b64("crt"));
    assert_eq!(manifest["data"][TLS_KEY_KEY], b64("key"));
    assert_eq!(manifest["data"][CA_CRT_KEY], b64("ca"));
}

#[test]
fn reconcile_once_issues_on_cold_start_and_noops_when_fresh() {
    let want = desired();
    let mut be = backend();

    // Cold start → Issue + produce material.
    let (report, material) = run_reconcile_once(
        &ObservedState::absent(),
        &want,
        &mut be,
        &FixedClock { now: 2_000 },
    )
    .expect("issue cycle");
    assert!(matches!(report.action, Action::Issue { .. }));
    assert!(report.mutated);
    let material = material.expect("issue produces material");

    // Observe the produced leaf; re-reconcile at the same instant → Noop.
    let observed = ObservedState {
        secret: Some(observed_secret_from_leaf_pem(&material.tls_crt_pem).expect("project")),
    };
    let (report, material) =
        run_reconcile_once(&observed, &want, &mut be, &FixedClock { now: 2_000 })
            .expect("noop cycle");
    assert_eq!(report.action, Action::Noop);
    assert!(!report.mutated);
    assert!(material.is_none());
}

#[test]
fn reconcile_once_rotates_when_within_window() {
    let want = desired();
    let mut be = backend();
    // A leaf expiring at 2_500; now=2_000 → 500s remaining ≤ 600s window ⇒ Rotate.
    let observed = ObservedState::present(2_500);
    let (report, material) =
        run_reconcile_once(&observed, &want, &mut be, &FixedClock { now: 2_000 })
            .expect("rotate cycle");
    assert!(matches!(report.action, Action::Rotate { .. }));
    assert!(report.mutated);
    assert!(material.is_some());
}
