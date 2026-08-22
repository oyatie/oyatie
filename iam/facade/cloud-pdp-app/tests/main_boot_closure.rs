//! Production-path closure E2E for the PDP mTLS boot (G002 slice-1b-iii-a/b;
//! ADR-0561, ADR-0506).
//!
//! This is the OVERCLAIM GUARD: it boots the PDP through the ACTUAL production
//! boot helper `server::boot_from_config` — the SAME function `main()` calls —
//! pointed at a delivered cert mount written in the operator's kubernetes.io/tls
//! shape (`tls.crt`/`tls.key`/`ca.crt`, PEM), then drives REAL rustls client
//! handshakes against the live socket. There is NO parallel test-only wiring: the
//! material flows through `MtlsContext::from_path` (the production runtime source)
//! and the boot decision is `boot_from_config`'s, not the test's.
//!
//! Fixtures:
//! 1. production-path closure: trusted SVID -> ALLOW bound to the SVID tenant;
//!    cross-tenant body -> 403/PermissionDenied (the #717 closure, on the real
//!    production boot path).
//! 2. fail-closed RED: an empty/absent mount -> `boot_from_config` returns a
//!    boot-refusal error (the `main` path would exit non-zero); NO socket accepts
//!    a plain (non-TLS) connection.
//! 3. from_path unit RED fixtures: missing tls.key -> MountUnreadable; empty
//!    ca.crt -> Empty; ca.crt with zero CA certs -> NoCaAnchors; garbage PEM ->
//!    MalformedPem.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use base64::Engine as _;
use http_runtime_hyper_adapter::pqc_hybrid_tls13_client_config_builder;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, Error, SignatureScheme};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use iam_cloud_pdp_app::mtls_transport::{MtlsContext, MtlsMaterialError};
use iam_cloud_pdp_app::server::{self, BootError};
use iam_cloud_pdp_kernel::PdpConfig;

use os_trustd_domain::JoinToken;
use os_trustd_domain::ca::{CertificateAuthority, CertificateSigningRequest};
use os_trustd_domain::certificate::CertUsage;
use os_trustd_domain::der;
use os_trustd_domain::service::{CertificateRequest, SecurityService};
use os_trustd_domain::signer::EcdsaP256Signer;
use os_trustd_domain::x509::KeyPair;

const JOIN_TOKEN: &str = "clusterid.clustersecret";

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// A real trustd CA + the CA signer that anchors it.
fn ca(name: &str) -> (CertificateAuthority<EcdsaP256Signer>, EcdsaP256Signer) {
    let signer = EcdsaP256Signer::generate().unwrap();
    let key = KeyPair::new(signer.private_key_der(), signer.public_key_spki_der());
    let now = now_secs();
    let ca =
        CertificateAuthority::bootstrap(name, key, signer.clone(), now, now + 10_000_000).unwrap();
    (ca, signer)
}

fn service() -> (SecurityService<EcdsaP256Signer>, EcdsaP256Signer) {
    let (ca, signer) = ca("oyatie-cell-7-ca");
    (
        SecurityService::new(JoinToken::new(JOIN_TOKEN).unwrap(), ca),
        signer,
    )
}

/// Mint a workload SVID leaf (real DER) for `uri`. Returns (leaf_der, key_pkcs8).
fn issue_svid(
    svc: &mut SecurityService<EcdsaP256Signer>,
    ca_signer: &EcdsaP256Signer,
    uri: &str,
    iat: u64,
    ttl: u64,
) -> (Vec<u8>, Vec<u8>) {
    let wl = EcdsaP256Signer::generate().unwrap();
    let key = KeyPair::new(wl.private_key_der(), wl.public_key_spki_der());
    let csr = CertificateSigningRequest::for_workload("wl", uri, &key, ttl);
    let req = CertificateRequest {
        join_token: JOIN_TOKEN.to_string(),
        csr,
    };
    let resp = svc.handle_certificate(&req, &key, iat).unwrap();
    let leaf = der::encode_leaf_der(
        &resp.identity.certificate,
        &wl,
        svc.ca_certificate(),
        ca_signer,
    )
    .unwrap();
    (leaf, wl.private_key_der())
}

/// The PDP server SVID identity the operator always mints — the cell pin
/// (`oyatie.cell-7`) is derived from this at boot. Fidelity with the real
/// operator leaf (`identity-workload-svid-operator-k8s::mint`).
const PDP_SERVER_SPIFFE: &str = "spiffe://oyatie.cell-7/platform/cloud-iam-pdp";

/// Mint the PDP server leaf (serverAuth + a DNS SAN the client checks + the
/// cell-rooted PDP SPIFFE URI SAN the cell pin is derived from — fidelity with
/// the operator leaf, without which the fail-closed cell-pin boot precondition
/// correctly refuses to serve). Returns (leaf_der, key_pkcs8).
fn issue_server_leaf(
    svc: &mut SecurityService<EcdsaP256Signer>,
    ca_signer: &EcdsaP256Signer,
    iat: u64,
    ttl: u64,
) -> (Vec<u8>, Vec<u8>) {
    let srv = EcdsaP256Signer::generate().unwrap();
    let key = KeyPair::new(srv.private_key_der(), srv.public_key_spki_der());
    let mut csr =
        CertificateSigningRequest::for_node("cloud-iam-pdp", &key, CertUsage::ServerAuth, ttl);
    csr.sans.dns_names.push("localhost".to_owned());
    // The SPIFFE id the PDP's cell pin is derived from at boot (mandatory).
    csr.sans.uris.push(PDP_SERVER_SPIFFE.to_owned());
    let req = CertificateRequest {
        join_token: JOIN_TOKEN.to_string(),
        csr,
    };
    let resp = svc.handle_certificate(&req, &key, iat).unwrap();
    let leaf = der::encode_leaf_der(
        &resp.identity.certificate,
        &srv,
        svc.ca_certificate(),
        ca_signer,
    )
    .unwrap();
    (leaf, srv.private_key_der())
}

/// Serialize DER as a base64 PEM block (the on-mount shape `from_path` parses).
fn pem(label: &str, der_bytes: &[u8]) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(der_bytes);
    let mut body = String::new();
    for chunk in b64.as_bytes().chunks(64) {
        body.push_str(std::str::from_utf8(chunk).unwrap());
        body.push('\n');
    }
    format!("-----BEGIN {label}-----\n{body}-----END {label}-----\n")
}

/// Write an operator-SHAPED real cert mount: real PDP server leaf + key, and the
/// real CA serialized into `ca.crt` (real X.509 DER, so `from_path` extracts the
/// real CA SPKI). Returns the mount directory.
fn write_real_mount(
    tag: &str,
    server_leaf_der: &[u8],
    server_key_pkcs8: &[u8],
    ca_der: &[u8],
) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "cloud-iam-pdp-boot-{}-{tag}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tls.crt"), pem("CERTIFICATE", server_leaf_der)).unwrap();
    std::fs::write(dir.join("tls.key"), pem("PRIVATE KEY", server_key_pkcs8)).unwrap();
    std::fs::write(dir.join("ca.crt"), pem("CERTIFICATE", ca_der)).unwrap();
    dir
}

fn config_for(bundle_path: &Path, mtls_cert_dir: &Path) -> PdpConfig {
    PdpConfig {
        bundle_path: bundle_path.to_string_lossy().into_owned(),
        bundle_trust_dir: common::trust_dir("boot-closure")
            .to_string_lossy()
            .into_owned(),
        rest_addr: "127.0.0.1:0".to_owned(),
        grpc_addr: "127.0.0.1:0".to_owned(),
        decision_cache_capacity: 64,
        mtls_cert_dir: mtls_cert_dir.to_string_lossy().into_owned(),
    }
}

/// A client-side server-cert verifier that accepts the trustd-CA-signed PDP
/// server leaf (trustd leaves are not browser-PKI). Test-only.
#[derive(Debug)]
struct AcceptTrustdServer {
    provider: Arc<rustls::crypto::CryptoProvider>,
}

impl ServerCertVerifier for AcceptTrustdServer {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}

/// Build a rustls client presenting `client` (or anonymous when `None`).
fn client_config(client: Option<(Vec<u8>, Vec<u8>)>) -> ClientConfig {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let verifier = Arc::new(AcceptTrustdServer {
        provider: Arc::clone(&provider),
    });
    let builder = pqc_hybrid_tls13_client_config_builder()
        .dangerous()
        .with_custom_certificate_verifier(verifier);
    match client {
        Some((leaf, key)) => builder
            .with_client_auth_cert(
                vec![CertificateDer::from(leaf)],
                PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key)),
            )
            .unwrap(),
        None => builder.with_no_client_auth(),
    }
}
fn assert_pqc_hybrid_tls13(connection: &rustls::ClientConnection) {
    assert_eq!(
        connection.protocol_version(),
        Some(rustls::ProtocolVersion::TLSv1_3),
        "production boot mTLS clients must negotiate TLS 1.3 through the shared PQC-hybrid policy"
    );
    assert_eq!(
        connection
            .negotiated_key_exchange_group()
            .map(|group| group.name()),
        Some(rustls::NamedGroup::X25519MLKEM768),
        "production boot mTLS clients must negotiate the X25519MLKEM768 hybrid key-share first"
    );
}

/// Drive one real mTLS HTTP/1.1 POST /v1/authorize and return (status, body).
async fn post_authorize(
    addr: std::net::SocketAddr,
    client: Option<(Vec<u8>, Vec<u8>)>,
    json_body: &str,
) -> std::io::Result<(u16, String)> {
    let cfg = client_config(client);
    let connector = TlsConnector::from(Arc::new(cfg));
    let tcp = TcpStream::connect(addr).await?;
    let domain = ServerName::try_from("localhost").unwrap();
    let mut tls = connector.connect(domain, tcp).await?;
    assert_pqc_hybrid_tls13(tls.get_ref().1);
    let req = format!(
        "POST /v1/authorize HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        json_body.len(),
        json_body
    );
    tls.write_all(req.as_bytes()).await?;
    tls.flush().await?;
    let mut raw = Vec::new();
    tls.read_to_end(&mut raw).await?;
    let text = String::from_utf8_lossy(&raw).into_owned();
    let status = text
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let body = text.split("\r\n\r\n").nth(1).unwrap_or("").to_owned();
    Ok((status, body))
}

fn authorize_body(tenant: &str) -> String {
    serde_json::json!({
        "request": common::request(
            "req-boot-closure",
            tenant,
            common::entity_ref("OyaPlatform::Principal", "alice"),
            "tenant.administer",
            common::entity_ref("OyaPlatform::Tenant", tenant),
        ),
        "entities": common::entity_slice().entities,
    })
    .to_string()
}

/// Write a unique SIGNED seed bundle file (the ConfigMap stand-in) and return
/// its path. The envelope is signed by the process-global test key the matching
/// `config_for` trust dir trusts (G004 bundle-signing slice).
fn seed_bundle_file(tag: &str) -> PathBuf {
    let seed = common::seed_bundle(common::SEED_VERSION, vec![]);
    let inner = serde_json::to_string(&seed).unwrap();
    common::temp_bundle_file(tag, &common::signed_bundle_doc(&inner))
}

// =====================================================================
// Fixture 1: PRODUCTION-PATH CLOSURE. The boot decision is boot_from_config's
// (the SAME helper main runs), reading real material via MtlsContext::from_path.
// =====================================================================
#[tokio::test(flavor = "multi_thread")]
async fn production_boot_from_mount_allows_trusted_svid_and_denies_cross_tenant() {
    let iat = now_secs();
    let (mut svc, ca_signer) = service();
    let (server_leaf, server_key) = issue_server_leaf(&mut svc, &ca_signer, iat, 10_000_000);
    // Real CA DER serialized into ca.crt -> from_path extracts the REAL CA SPKI.
    let ca_der = der::encode_ca_der(svc.ca_certificate(), &ca_signer).unwrap();
    let mount = write_real_mount("closure", &server_leaf, &server_key, &ca_der);
    let bundle_path = seed_bundle_file("closure");

    // BOOT THROUGH THE PRODUCTION HELPER — the exact code path main() runs.
    let handle = server::boot_from_config(&config_for(&bundle_path, &mount))
        .await
        .expect("production boot_from_config must succeed on a real mount");

    // Trusted client SVID for ten_acme -> reaches the decision, bound to ten_acme.
    let (leaf, key) = issue_svid(
        &mut svc,
        &ca_signer,
        "spiffe://oyatie.cell-7/tenant/ten_acme/wl",
        iat,
        10_000_000,
    );
    let (status, body) = post_authorize(
        handle.rest_addr,
        Some((leaf.clone(), key.clone())),
        &authorize_body("ten_acme"),
    )
    .await
    .expect("handshake + request");
    assert_eq!(
        status, 200,
        "trusted SVID must reach a decision through the production boot: {body}"
    );
    assert!(
        body.contains("\"decision\""),
        "200 must carry a decision body, got {body}"
    );

    // Cross-tenant: real SVID ten_acme, body ten_globex -> 403 (never 200/404).
    let (status, body) = post_authorize(
        handle.rest_addr,
        Some((leaf, key)),
        &authorize_body("ten_globex"),
    )
    .await
    .expect("handshake + request");
    assert_eq!(
        status, 403,
        "cross-tenant spoof must be 403 on the production boot: {body}"
    );

    handle.shutdown().await;
}

// =====================================================================
// Fixture 2: FAIL-CLOSED RED. An absent mount -> the production boot helper
// returns a boot-refusal error (main would exit non-zero); NO plain socket binds.
// =====================================================================
#[tokio::test(flavor = "multi_thread")]
async fn production_boot_fails_closed_on_absent_mount() {
    let bundle_path = seed_bundle_file("absent");
    let absent = std::env::temp_dir().join(format!(
        "cloud-iam-pdp-boot-{}-absent-does-not-exist",
        std::process::id()
    ));
    // Ensure it truly does not exist.
    let _ = std::fs::remove_dir_all(&absent);

    let err = server::boot_from_config(&config_for(&bundle_path, &absent))
        .await
        .err()
        .expect("absent mount must be a boot refusal, never plain TCP");
    // It is a material refusal (the mount is unreadable), not a Start error.
    assert!(
        matches!(
            err,
            BootError::Material(MtlsMaterialError::MountUnreadable { .. })
        ),
        "absent mount must be BootError::Material(MountUnreadable), got {err:?}"
    );
    assert!(
        err.to_string().contains("refusing to boot"),
        "boot refusal must be legible, got {err}"
    );
    // No ServiceHandle was returned, so no socket (plain or TLS) is accepting:
    // the production boot bound NOTHING. The RED contract is the Err above.
}

#[tokio::test(flavor = "multi_thread")]
async fn production_boot_fails_closed_on_empty_mount_dir() {
    let bundle_path = seed_bundle_file("empty-dir");
    let empty_dir = std::env::temp_dir().join(format!(
        "cloud-iam-pdp-boot-{}-empty-dir",
        std::process::id()
    ));
    std::fs::create_dir_all(&empty_dir).unwrap();
    // Directory exists but carries no tls.crt/tls.key/ca.crt.

    let err = server::boot_from_config(&config_for(&bundle_path, &empty_dir))
        .await
        .err()
        .expect("empty mount dir must be a boot refusal");
    assert!(
        matches!(err, BootError::Material(_)),
        "empty mount dir must be BootError::Material, got {err:?}"
    );
}

// =====================================================================
// Fixture 3: from_path unit RED fixtures.
// =====================================================================

/// `from_path` Err extractor: `MtlsContext` is not `Debug` (it owns the trust
/// bundle, which must not render), so `Result::unwrap_err` is unavailable.
fn from_path_err(dir: &Path) -> MtlsMaterialError {
    match MtlsContext::from_path(dir) {
        Ok(_) => panic!("expected from_path to fail-close for {}", dir.display()),
        Err(e) => e,
    }
}

/// Build a complete real mount, then mutate one file per RED case.
fn real_mount_parts() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let iat = now_secs();
    let (mut svc, ca_signer) = service();
    let (server_leaf, server_key) = issue_server_leaf(&mut svc, &ca_signer, iat, 10_000_000);
    let ca_der = der::encode_ca_der(svc.ca_certificate(), &ca_signer).unwrap();
    (server_leaf, server_key, ca_der)
}

#[test]
fn from_path_missing_tls_key_is_mount_unreadable() {
    let (leaf, _key, ca_der) = real_mount_parts();
    let dir = std::env::temp_dir().join(format!(
        "cloud-iam-pdp-frompath-{}-missing-key",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tls.crt"), pem("CERTIFICATE", &leaf)).unwrap();
    std::fs::write(dir.join("ca.crt"), pem("CERTIFICATE", &ca_der)).unwrap();
    // tls.key deliberately absent.
    let err = from_path_err(&dir);
    assert!(
        matches!(err, MtlsMaterialError::MountUnreadable { .. }),
        "missing tls.key must be MountUnreadable, got {err:?}"
    );
}

#[test]
fn from_path_empty_ca_crt_is_empty() {
    let (leaf, key, _ca) = real_mount_parts();
    let dir = std::env::temp_dir().join(format!(
        "cloud-iam-pdp-frompath-{}-empty-ca",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tls.crt"), pem("CERTIFICATE", &leaf)).unwrap();
    std::fs::write(dir.join("tls.key"), pem("PRIVATE KEY", &key)).unwrap();
    std::fs::write(dir.join("ca.crt"), "   \n\t\n").unwrap(); // whitespace-only.
    let err = from_path_err(&dir);
    assert!(
        matches!(err, MtlsMaterialError::Empty { .. }),
        "empty ca.crt must be Empty, got {err:?}"
    );
}

#[test]
fn from_path_ca_crt_with_zero_ca_certs_is_no_ca_anchors() {
    let (leaf, key, _ca) = real_mount_parts();
    let dir = std::env::temp_dir().join(format!(
        "cloud-iam-pdp-frompath-{}-no-anchors",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tls.crt"), pem("CERTIFICATE", &leaf)).unwrap();
    std::fs::write(dir.join("tls.key"), pem("PRIVATE KEY", &key)).unwrap();
    // A well-formed PEM block that is NOT a CERTIFICATE -> zero CA anchors.
    std::fs::write(dir.join("ca.crt"), pem("PUBLIC KEY", &[1u8, 2, 3, 4])).unwrap();
    let err = from_path_err(&dir);
    assert!(
        matches!(err, MtlsMaterialError::NoCaAnchors { .. }),
        "ca.crt with zero CA certs must be NoCaAnchors, got {err:?}"
    );
}

#[test]
fn from_path_garbage_pem_is_malformed() {
    let (leaf, key, _ca) = real_mount_parts();
    let dir = std::env::temp_dir().join(format!(
        "cloud-iam-pdp-frompath-{}-garbage",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("tls.crt"), pem("CERTIFICATE", &leaf)).unwrap();
    std::fs::write(dir.join("tls.key"), pem("PRIVATE KEY", &key)).unwrap();
    // A CERTIFICATE PEM block whose body is not valid DER -> MalformedPem on the
    // anchor DER parse (it base64-decodes but is not a real X.509 cert).
    std::fs::write(
        dir.join("ca.crt"),
        pem("CERTIFICATE", b"not-a-real-x509-certificate-der"),
    )
    .unwrap();
    let err = from_path_err(&dir);
    assert!(
        matches!(err, MtlsMaterialError::MalformedPem { .. }),
        "garbage CERTIFICATE body must be MalformedPem, got {err:?}"
    );
}

// =====================================================================
// Fixture 4: KEYSTONE — the PDP boots from OPERATOR-PRODUCED material.
// The mount is written by the live SVID-delivery operator's issuance→Secret path
// (NOT hand-written by this test), and the caller SVID chains to the SAME CA the
// operator delivered in ca.crt. THIS is what flips FRIC-1781490000 closed:
// an operator-produced Secret yields a real ALLOW/deny mTLS handshake.
// =====================================================================

use iam_identity_workload_svid_operator_k8s::{
    SvidSecretMaterial, TrustdEcdsaIssuanceBackend, run_reconcile_once,
};
use iam_identity_workload_svid_operator_kernel::{
    Action, Clock as OperatorClock, DesiredState, ObservedState,
};

const OPERATOR_JOIN_TOKEN: &str = "clusterid.clustersecret";

#[derive(Clone, Copy)]
struct FixedOperatorClock {
    now: u64,
}

impl OperatorClock for FixedOperatorClock {
    fn now_epoch_seconds(&self) -> u64 {
        self.now
    }
}

/// Write the operator-produced PEM members to a mount dir the PDP boots from.
fn write_operator_mount(tag: &str, material: &SvidSecretMaterial) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "cloud-iam-pdp-operator-{}-{tag}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    // The operator already emits PEM; write it verbatim (the on-mount shape).
    std::fs::write(dir.join("tls.crt"), &material.tls_crt_pem).unwrap();
    std::fs::write(dir.join("tls.key"), &material.tls_key_pem).unwrap();
    std::fs::write(dir.join("ca.crt"), &material.ca_crt_pem).unwrap();
    dir
}

#[tokio::test(flavor = "multi_thread")]
async fn operator_produced_secret_boots_pdp_and_yields_real_allow_deny_handshake() {
    let iat = now_secs();

    // 1. The LIVE operator mints the PDP server SVID Secret (cold-start Issue),
    //    driving the SAME pure-kernel → trustd-issuer → kubernetes.io/tls path the
    //    in-cluster operator runs. The mount is OPERATOR-PRODUCED, not test-written.
    let mut backend = TrustdEcdsaIssuanceBackend::bootstrap(
        "oyatie-cloud-iam-pdp-svid-ca",
        OPERATOR_JOIN_TOKEN,
        iat,
        10_000_000,
    )
    .expect("operator CA bootstrap");
    let desired = DesiredState {
        spiffe_id: "spiffe://oyatie.cell-7/platform/cloud-iam-pdp".to_owned(),
        ttl_secs: 10_000_000,
        rotation_window_secs: 600,
        secret_name: "cloud-iam-pdp-svid".to_owned(),
        secret_namespace: "cloud-iam".to_owned(),
    };
    let (report, material) = run_reconcile_once(
        &ObservedState::absent(),
        &desired,
        &mut backend,
        &FixedOperatorClock { now: iat },
    )
    .expect("operator reconcile issues on cold start");
    assert!(
        matches!(report.action, Action::Issue { .. }),
        "cold start must be an Issue"
    );
    let material = material.expect("Issue must produce Secret material");
    let mount = write_operator_mount("closure", &material);
    let bundle_path = seed_bundle_file("operator-closure");

    // 2. Boot the PDP through the PRODUCTION helper from the OPERATOR-produced mount.
    let handle = server::boot_from_config(&config_for(&bundle_path, &mount))
        .await
        .expect("PDP must boot from the operator-produced Secret");

    // 3. A caller SVID minted from the SAME operator CA (so it chains to the
    //    delivered ca.crt) for ten_acme -> ALLOW, bound to the SVID tenant.
    let (leaf, key) = backend
        .issue_caller_svid("spiffe://oyatie.cell-7/tenant/ten_acme/wl", 10_000_000, iat)
        .expect("operator CA issues the caller SVID");
    let (status, body) = post_authorize(
        handle.rest_addr,
        Some((leaf.clone(), key.clone())),
        &authorize_body("ten_acme"),
    )
    .await
    .expect("handshake + request");
    assert_eq!(
        status, 200,
        "operator-delivered trust must ALLOW the trusted caller SVID: {body}"
    );
    assert!(
        body.contains("\"decision\""),
        "200 must carry a decision body, got {body}"
    );

    // 4. Cross-tenant: real SVID ten_acme, body ten_globex -> 403 (never 200/404).
    let (status, body) = post_authorize(
        handle.rest_addr,
        Some((leaf, key)),
        &authorize_body("ten_globex"),
    )
    .await
    .expect("handshake + request");
    assert_eq!(
        status, 403,
        "cross-tenant spoof must be 403 on the operator-booted PDP: {body}"
    );

    handle.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn pdp_fails_closed_when_operator_has_not_produced_the_secret() {
    // The operator-produced Secret is the ONLY trust source: with no Secret yet
    // (the operator has not run / the mount is absent), the PDP boot REFUSES —
    // never falls back to plain TCP. This is the fail-closed guard for the
    // cert-delivery dimension the operator owns.
    let bundle_path = seed_bundle_file("operator-absent");
    let absent = std::env::temp_dir().join(format!(
        "cloud-iam-pdp-operator-{}-absent-secret",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&absent);

    let err = server::boot_from_config(&config_for(&bundle_path, &absent))
        .await
        .err()
        .expect("absent operator Secret must be a boot refusal, never plain TCP");
    assert!(
        matches!(
            err,
            BootError::Material(MtlsMaterialError::MountUnreadable { .. })
        ),
        "absent operator Secret must be BootError::Material(MountUnreadable), got {err:?}"
    );
}
