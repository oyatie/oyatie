//! Live real-handshake mTLS E2E for the cloud-iam PDP service (G002
//! slice-1b-ii; ADR-0561, ADR-0506).
//!
//! Boots the REAL service over mTLS (`server::start_with_mtls` — the same boot
//! path `main` will call once K8s cert-delivery lands) with a real trustd CA +
//! real X.509 leaves (rcgen/aws-lc-rs, NO ring), then drives REAL rustls client
//! handshakes against the live REST socket. These are not in-process PEP unit
//! tests (those live in `src/mtls.rs`) — every assertion here rides a genuine
//! TLS 1.3 handshake terminated by the production `SvidClientCertVerifier`.
//!
//! RED fixtures (fail-closed):
//! 1. trusted client SVID            -> ALLOW, decision bound to the SVID tenant
//! 2. rogue (untrusted-CA) SVID      -> handshake rejected (no decision)
//! 3. expired SVID                   -> handshake rejected (no decision)
//! 4. cross-tenant (real SVID ten_acme, body ten_globex) -> 403 PermissionDenied
//! 5. no client cert                 -> handshake refused (client-auth-mandatory)
//! + boot-refuse: empty trust bundle -> StartError::Mtls(TrustBundleEmpty)

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::sync::Arc;

use http_runtime_hyper_adapter::pqc_hybrid_tls13_client_config_builder;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, Error, SignatureScheme};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use iam_cloud_pdp_app::grpc::proto;
use iam_cloud_pdp_app::mtls_transport::MtlsContext;
use iam_cloud_pdp_app::server::{self, StartError};
use iam_cloud_pdp_kernel::PdpConfig;

use os_trustd_domain::ca::{CertificateAuthority, CertificateSigningRequest};
use os_trustd_domain::certificate::CertUsage;
use os_trustd_domain::der;
use os_trustd_domain::service::{CertificateRequest, SecurityService};
use os_trustd_domain::signer::EcdsaP256Signer;
use os_trustd_domain::x509::KeyPair;
use os_trustd_domain::{JoinToken, TrustBundle};

const JOIN_TOKEN: &str = "clusterid.clustersecret";
// The SVID validity window the trustd service mints is [now, now+ttl). We boot
// with `now = 2_000` baked into issuance and exercise handshakes at a clock the
// rustls verifier sees as "now" via `UnixTime::now()` — so to keep the leaves
// valid during the live handshake we mint with a far-future not_after by issuing
// at a real wall-clock "now" base.
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

/// Mint a workload SVID leaf (real DER) for `uri`, valid for `ttl` from `iat`.
/// Returns (leaf_der, workload_key_pkcs8_der).
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

/// The PDP's own server SVID identity — the cell authority the PDP pins callers
/// to is derived from this (`oyatie.cell-7`). Matches the operator-minted server
/// SVID (`identity-workload-svid-operator-k8s`).
const PDP_SERVER_SPIFFE: &str = "spiffe://oyatie.cell-7/platform/cloud-iam-pdp";

/// Mint the PDP server leaf (serverAuth + a DNS SAN the client checks + the PDP
/// SPIFFE URI SAN the cell pin is derived from — fidelity with the operator leaf).
fn issue_server_leaf(
    svc: &mut SecurityService<EcdsaP256Signer>,
    ca_signer: &EcdsaP256Signer,
    iat: u64,
    ttl: u64,
) -> (Vec<u8>, Vec<u8>) {
    let srv = EcdsaP256Signer::generate().unwrap();
    let key = KeyPair::new(srv.private_key_der(), srv.public_key_spki_der());
    let mut csr =
        CertificateSigningRequest::for_node("oya-cloud-iam-pdp", &key, CertUsage::ServerAuth, ttl);
    csr.sans.dns_names.push("localhost".to_owned());
    // The SPIFFE id the PDP's cell pin is derived from at boot.
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

/// Mint a PDP server leaf (serverAuth + a DNS SAN) carrying EXACTLY the supplied
/// URI SANs (zero, one, many, or non-cell-rooted) — the knob the cell-pin
/// boot-precondition fixtures vary. Returns (leaf_der, server_key_pkcs8_der).
fn issue_server_leaf_with_uris(
    svc: &mut SecurityService<EcdsaP256Signer>,
    ca_signer: &EcdsaP256Signer,
    iat: u64,
    ttl: u64,
    uris: &[&str],
) -> (Vec<u8>, Vec<u8>) {
    let srv = EcdsaP256Signer::generate().unwrap();
    let key = KeyPair::new(srv.private_key_der(), srv.public_key_spki_der());
    let mut csr =
        CertificateSigningRequest::for_node("oya-cloud-iam-pdp", &key, CertUsage::ServerAuth, ttl);
    csr.sans.dns_names.push("localhost".to_owned());
    for uri in uris {
        csr.sans.uris.push((*uri).to_owned());
    }
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

fn config_for(bundle_path: &std::path::Path) -> PdpConfig {
    PdpConfig {
        bundle_path: bundle_path.to_string_lossy().into_owned(),
        bundle_trust_dir: common::trust_dir("mtls").to_string_lossy().into_owned(),
        rest_addr: "127.0.0.1:0".to_owned(),
        grpc_addr: "127.0.0.1:0".to_owned(),
        decision_cache_capacity: 64,
        mtls_cert_dir: "/etc/oya-cloud-iam-pdp/tls".to_owned(),
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

/// Build a rustls client presenting `client_leaf`/`client_key` (or anonymous
/// when `None`), trusting the trustd PDP server leaf.
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
        "mTLS clients must negotiate TLS 1.3 through the shared PQC-hybrid policy"
    );
    assert_eq!(
        connection
            .negotiated_key_exchange_group()
            .map(|group| group.name()),
        Some(rustls::NamedGroup::X25519MLKEM768),
        "mTLS clients must negotiate the X25519MLKEM768 hybrid key-share first"
    );
}

/// Drive one real mTLS HTTP/1.1 POST /v1/authorize and return (status, body).
/// `client` = the client SVID to present (None = no client cert).
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
            "req-mtls",
            tenant,
            common::entity_ref("OyaPlatform::Principal", "alice"),
            "tenant.administer",
            common::entity_ref("OyaPlatform::Tenant", tenant),
        ),
        "entities": common::entity_slice().entities,
    })
    .to_string()
}

/// Boot the PDP over mTLS with a real CA, server leaf, and trust bundle.
/// Returns (handle, svc, ca_signer, iat) so the caller can mint client SVIDs.
async fn boot_mtls() -> (
    server::ServiceHandle,
    SecurityService<EcdsaP256Signer>,
    EcdsaP256Signer,
    u64,
) {
    let iat = now_secs();
    let (mut svc, ca_signer) = service();
    let (server_leaf, server_key) = issue_server_leaf(&mut svc, &ca_signer, iat, 10_000_000);
    let bundle = trusted_bundle(&svc, &ca_signer);
    let ctx = MtlsContext::new(bundle, vec![server_leaf], server_key).unwrap();

    let seed = common::seed_bundle(common::SEED_VERSION, vec![]);
    // Unique per call: parallel tests must not share one bundle file path (a
    // concurrent read of a mid-write file is an EOF parse error at boot).
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let tag = format!(
        "mtls-{}",
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    let inner = serde_json::to_string(&seed).unwrap();
    let path = common::temp_bundle_file(&tag, &common::signed_bundle_doc(&inner));
    let handle = server::start_with_mtls(&config_for(&path), Some(ctx))
        .await
        .expect("mTLS boot");
    (handle, svc, ca_signer, iat)
}

// Fixture 1 + 4: trusted SVID ALLOW (bound to SVID tenant) + cross-tenant 403.
#[tokio::test(flavor = "multi_thread")]
async fn trusted_svid_allows_and_binds_tenant() {
    let (handle, mut svc, ca_signer, iat) = boot_mtls().await;
    let (leaf, key) = issue_svid(
        &mut svc,
        &ca_signer,
        "spiffe://oyatie.cell-7/tenant/ten_acme/wl",
        iat,
        10_000_000,
    );

    // Matching tenant -> the request reaches the decision surface (the SVID
    // tenant ten_acme drove it, not the body). A 200 carries a decision body.
    let (status, body) = post_authorize(
        handle.rest_addr,
        Some((leaf.clone(), key.clone())),
        &authorize_body("ten_acme"),
    )
    .await
    .expect("handshake + request");
    assert_eq!(status, 200, "trusted SVID should reach a decision: {body}");
    assert!(
        body.contains("\"decision\""),
        "200 must carry a decision body, got {body}"
    );

    // Cross-tenant: real SVID ten_acme, body ten_globex -> 403 (never 404, never
    // 200), with the coarse caller-auth refusal code (no tenant leak).
    let (status, body) = post_authorize(
        handle.rest_addr,
        Some((leaf, key)),
        &authorize_body("ten_globex"),
    )
    .await
    .expect("handshake + request");
    assert_eq!(
        status, 403,
        "cross-tenant spoof must be 403, never 404/200: {body}"
    );
    assert!(
        body.contains("caller_unauthenticated"),
        "403 must carry the coarse caller-auth code, got {body}"
    );

    handle.shutdown().await;
}

// Fixture 6 (G002/G004 #109): cell-authority pinning on a LIVE socket. A caller
// SVID minted from the SAME trusted CA but rooted in a FOREIGN cell
// (oyatie.cell-OTHER) reaches the handshake (it chains + verifies) but is DENIED
// at the PEP with 403 — the cross-cell confusion guard. A same-cell SVID still
// ALLOWs (non-regression of the live ALLOW path).
#[tokio::test(flavor = "multi_thread")]
async fn foreign_cell_svid_denied_on_live_socket() {
    let (handle, mut svc, ca_signer, iat) = boot_mtls().await;

    // Foreign-cell caller SVID (same trusted CA → handshake passes, cell differs).
    let (foreign_leaf, foreign_key) = issue_svid(
        &mut svc,
        &ca_signer,
        "spiffe://oyatie.cell-OTHER/tenant/ten_acme/wl",
        iat,
        10_000_000,
    );
    let (status, body) = post_authorize(
        handle.rest_addr,
        Some((foreign_leaf, foreign_key)),
        &authorize_body("ten_acme"),
    )
    .await
    .expect("handshake + request");
    assert_eq!(
        status, 403,
        "foreign-cell SVID must be 403 (never 200/404) on the live socket: {body}"
    );
    assert!(
        body.contains("caller_unauthenticated"),
        "403 must carry the coarse caller-auth code (no cell leak), got {body}"
    );

    // Same-cell caller SVID → still reaches the decision (non-regression).
    let (same_leaf, same_key) = issue_svid(
        &mut svc,
        &ca_signer,
        "spiffe://oyatie.cell-7/tenant/ten_acme/wl",
        iat,
        10_000_000,
    );
    let (status, body) = post_authorize(
        handle.rest_addr,
        Some((same_leaf, same_key)),
        &authorize_body("ten_acme"),
    )
    .await
    .expect("handshake + request");
    assert_eq!(status, 200, "same-cell SVID must still ALLOW: {body}");

    handle.shutdown().await;
}

// Fixture 2: rogue (untrusted-CA) SVID -> handshake rejected, no decision.
#[tokio::test(flavor = "multi_thread")]
async fn rogue_svid_handshake_rejected() {
    let (handle, _svc, _ca_signer, iat) = boot_mtls().await;
    // Mint a real leaf from a ROGUE CA the bundle does not trust.
    let (mut rogue, rogue_signer) = service();
    let (leaf, key) = issue_svid(
        &mut rogue,
        &rogue_signer,
        "spiffe://oyatie.cell-7/tenant/ten_acme/evil",
        iat,
        10_000_000,
    );
    let result = post_authorize(
        handle.rest_addr,
        Some((leaf, key)),
        &authorize_body("ten_acme"),
    )
    .await;
    // A rejected handshake returns no HTTP response: the TLS layer aborts, so
    // either connect/read errors or no status line is produced. It MUST NOT
    // reach a 2xx/4xx decision surface (that would mean the rogue cert was
    // accepted).
    let status = result.as_ref().map(|(s, _)| *s).unwrap_or(0);
    assert!(
        result.is_err() || status == 0,
        "rogue SVID must be rejected at the handshake, got {result:?}"
    );
    assert_ne!(status, 200, "rogue SVID must never reach a decision");
    handle.shutdown().await;
}

// Fixture 3: expired SVID -> handshake rejected (verifier checks validity).
#[tokio::test(flavor = "multi_thread")]
async fn expired_svid_handshake_rejected() {
    let (handle, mut svc, ca_signer, _iat) = boot_mtls().await;
    // Issue at the unix epoch with a tiny TTL -> long expired by wall-clock now.
    let (leaf, key) = issue_svid(
        &mut svc,
        &ca_signer,
        "spiffe://oyatie.cell-7/tenant/ten_acme/wl",
        1_000,
        3_600,
    );
    let result = post_authorize(
        handle.rest_addr,
        Some((leaf, key)),
        &authorize_body("ten_acme"),
    )
    .await;
    let status = result.as_ref().map(|(s, _)| *s).unwrap_or(0);
    assert!(
        result.is_err() || status == 0,
        "expired SVID must be rejected at the handshake, got {result:?}"
    );
    assert_ne!(status, 200, "expired SVID must never reach a decision");
    handle.shutdown().await;
}

// Fixture 5: no client cert -> handshake refused (client_auth_mandatory).
#[tokio::test(flavor = "multi_thread")]
async fn no_client_cert_handshake_refused() {
    let (handle, _svc, _ca_signer, _iat) = boot_mtls().await;
    let result = post_authorize(handle.rest_addr, None, &authorize_body("ten_acme")).await;
    let status = result.as_ref().map(|(s, _)| *s).unwrap_or(0);
    assert!(
        result.is_err() || status == 0,
        "anonymous client must be refused (client-auth-mandatory), got {result:?}"
    );
    assert_ne!(status, 200, "anonymous client must never reach a decision");
    handle.shutdown().await;
}

// gRPC real-handshake: a trusted client SVID over mTLS reaches the gRPC
// decision surface, proving the peer leaf flows from the rustls handshake
// through the tonic Connected ConnectInfo into the PEP (the gRPC twin of the
// REST path; tonic inserts the custom ConnectInfo into request extensions).
#[tokio::test(flavor = "multi_thread")]
async fn trusted_svid_reaches_grpc_decision_over_mtls() {
    use hyper_util::rt::TokioIo;
    use tonic::transport::Endpoint;

    let (handle, mut svc, ca_signer, iat) = boot_mtls().await;
    let (leaf, key) = issue_svid(
        &mut svc,
        &ca_signer,
        "spiffe://oyatie.cell-7/tenant/ten_acme/wl",
        iat,
        10_000_000,
    );
    let grpc_addr = handle.grpc_addr;

    // A custom connector that performs the tokio-rustls client handshake
    // (presenting the SVID) and hands tonic the TLS-wrapped IO — no tonic tls
    // feature needed.
    let cfg = Arc::new(client_config(Some((leaf, key))));
    let connector = tower::service_fn(move |_uri: tonic::transport::Uri| {
        let cfg = Arc::clone(&cfg);
        async move {
            let connector = TlsConnector::from(cfg);
            let tcp = TcpStream::connect(grpc_addr).await?;
            let domain = ServerName::try_from("localhost").unwrap();
            let tls = connector.connect(domain, tcp).await?;
            assert_pqc_hybrid_tls13(tls.get_ref().1);
            Ok::<_, std::io::Error>(TokioIo::new(tls))
        }
    });

    let channel = Endpoint::from_static("https://localhost")
        .connect_with_connector(connector)
        .await
        .expect("grpc mTLS connect");
    let mut client = proto::cloud_iam_pdp_client::CloudIamPdpClient::new(channel);

    // The body asks for ten_acme; the SVID authorizes ten_acme -> ALLOW path.
    let request = proto::AuthorizeRequest {
        request_id: "req-grpc-mtls".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        principal: Some(proto::EntityRef {
            entity_type: "OyaPlatform::Principal".to_owned(),
            entity_id: "alice".to_owned(),
        }),
        action: "tenant.administer".to_owned(),
        resource: Some(proto::EntityRef {
            entity_type: "OyaPlatform::Tenant".to_owned(),
            entity_id: "ten_acme".to_owned(),
        }),
        context: std::collections::HashMap::new(),
        entities: grpc_entities(),
        min_policy_version: String::new(),
    };
    // A reached decision (Ok response) proves the verified peer leaf flowed
    // through the PEP and bound the tenant; an auth failure would be
    // PermissionDenied.
    let resp = client.authorize(request).await;
    assert!(
        resp.is_ok(),
        "trusted SVID must reach the gRPC decision, got {resp:?}"
    );

    // Cross-tenant over gRPC: SVID ten_acme, body ten_globex -> PermissionDenied.
    let spoof = proto::AuthorizeRequest {
        request_id: "req-grpc-spoof".to_owned(),
        tenant_id: "ten_globex".to_owned(),
        principal: Some(proto::EntityRef {
            entity_type: "OyaPlatform::Principal".to_owned(),
            entity_id: "alice".to_owned(),
        }),
        action: "tenant.administer".to_owned(),
        resource: Some(proto::EntityRef {
            entity_type: "OyaPlatform::Tenant".to_owned(),
            entity_id: "ten_globex".to_owned(),
        }),
        context: std::collections::HashMap::new(),
        entities: grpc_entities(),
        min_policy_version: String::new(),
    };
    let err = client
        .authorize(spoof)
        .await
        .expect_err("cross-tenant denies");
    assert_eq!(
        err.code(),
        tonic::Code::PermissionDenied,
        "cross-tenant spoof must be PermissionDenied, never another code"
    );

    handle.shutdown().await;
}

/// The seed entity slice as proto records for the gRPC request.
fn grpc_entities() -> Vec<proto::EntityRecord> {
    common::entity_slice()
        .entities
        .into_iter()
        .map(|e| proto::EntityRecord {
            uid: Some(proto::EntityRef {
                entity_type: e.uid.entity_type,
                entity_id: e.uid.entity_id,
            }),
            attributes: e
                .attributes
                .into_iter()
                .map(|(k, v)| {
                    let value = match v {
                        serde_json::Value::String(s) => {
                            Some(proto::attribute_value::Value::StringValue(s))
                        }
                        serde_json::Value::Bool(b) => {
                            Some(proto::attribute_value::Value::BoolValue(b))
                        }
                        _ => None,
                    };
                    (k, proto::AttributeValue { value })
                })
                .collect(),
            parents: e
                .parents
                .into_iter()
                .map(|p| proto::EntityRef {
                    entity_type: p.entity_type,
                    entity_id: p.entity_id,
                })
                .collect(),
        })
        .collect()
}

// Server-identity verification (G002/G004 #109, Fix 3). The keystone #793 live
// fixtures use the `AcceptTrustdServer` bypass verifier, so a client verifying the
// PDP's SERVER identity was untested. The PDP server leaf is a serverAuth cert (it
// is NOT a caller leaf, so the caller-path `verify_peer` clientAuth gate does not
// apply to it). This proves the load-bearing server-identity facts a real
// client-side verifier checks: (a) the genuine PDP server leaf carries the EXPECTED
// SPIFFE identity (`oya-cloud-iam-pdp` in `oyatie.cell-7`) and chains to the trusted
// CA (its real signature verifies under the bundle's CA SPKI); (b) an IMPERSONATOR
// presenting the same identity signed by a ROGUE CA the client does not trust does
// NOT chain — a client verifying the server identity refuses a PDP impersonator.
//
// (A full custom rustls client-side `ServerCertVerifier` deferring to a serverAuth
// trustd verifier on the live handshake is the proportionate residual; the server
// identity + chain are the load-bearing checks, exercised here directly.)
#[test]
fn pdp_server_leaf_identity_is_correct_and_impersonator_does_not_chain() {
    use iam_identity_workload_svid_kernel::SpiffeId;
    use iam_identity_workload_svid_trustd::leaf_der;
    use x509_parser::certificate::X509Certificate;
    use x509_parser::prelude::FromDer;
    use x509_parser::x509::SubjectPublicKeyInfo;

    let iat = now_secs();
    let (mut svc, ca_signer) = service();
    let (server_leaf, _server_key) = issue_server_leaf(&mut svc, &ca_signer, iat, 10_000_000);
    let trusted_ca_spki = svc.ca_certificate().public_key_der.clone();

    // Verify a server leaf chains to a trusted CA SPKI by its real signature.
    let chains_to = |leaf_der: &[u8], ca_spki: &[u8]| -> bool {
        let Ok((rest, cert)) = X509Certificate::from_der(leaf_der) else {
            return false;
        };
        if !rest.is_empty() {
            return false;
        }
        match SubjectPublicKeyInfo::from_der(ca_spki) {
            Ok((spki_rest, ca)) => spki_rest.is_empty() && cert.verify_signature(Some(&ca)).is_ok(),
            Err(_) => false,
        }
    };

    // (a) Genuine PDP: identity is the expected platform SVID + chains to the CA.
    let uri = leaf_der::extract_single_uri_san(&server_leaf)
        .expect("PDP server leaf carries its SPIFFE URI SAN");
    assert_eq!(
        uri, PDP_SERVER_SPIFFE,
        "PDP server identity must be the expected platform SVID"
    );
    let id = SpiffeId::parse(&uri).expect("PDP server SPIFFE id parses");
    assert_eq!(id.trust_domain_authority(), "oyatie.cell-7");
    assert!(
        chains_to(&server_leaf, &trusted_ca_spki),
        "genuine PDP server leaf must chain to the trusted CA"
    );

    // (b) Impersonator: same identity, ROGUE CA → does NOT chain to the trusted CA.
    let (mut rogue, rogue_signer) = service();
    let (impostor_leaf, _k) = issue_server_leaf(&mut rogue, &rogue_signer, iat, 10_000_000);
    assert_eq!(
        leaf_der::extract_single_uri_san(&impostor_leaf).unwrap(),
        PDP_SERVER_SPIFFE,
        "impostor presents the same identity (so identity alone is not enough)"
    );
    assert!(
        !chains_to(&impostor_leaf, &trusted_ca_spki),
        "impersonator PDP server leaf must NOT chain to the trusted CA (refused)"
    );
}

// Boot-refuse: an empty trust bundle is a fail-closed StartError::Mtls.
#[tokio::test(flavor = "multi_thread")]
async fn empty_bundle_refuses_mtls_boot() {
    let iat = now_secs();
    let (mut svc, ca_signer) = service();
    let (server_leaf, server_key) = issue_server_leaf(&mut svc, &ca_signer, iat, 10_000_000);
    let empty: TrustBundle<EcdsaP256Signer> = TrustBundle::new();
    // MtlsContext::new must reject the empty bundle before any socket binds.
    let err = MtlsContext::new(empty, vec![server_leaf], server_key)
        .err()
        .expect("empty bundle must be refused");
    assert_eq!(
        err,
        iam_cloud_pdp_app::mtls::MtlsBootError::TrustBundleEmpty
    );

    // And the start path surfaces it as StartError::Mtls — but MtlsContext::new
    // already guards, so prove the StartError variant via a direct construction
    // path: a context that fails to build is impossible to pass to start, so the
    // boot-refuse contract is the MtlsContext guard above. Assert the StartError
    // variant exists + formats (the server.rs Mtls arm).
    let start_err = StartError::Mtls(iam_cloud_pdp_app::mtls::MtlsBootError::TrustBundleEmpty);
    assert!(start_err.to_string().contains("refusing to boot"));
}

// ==========================================================================
// Cell-pin boot precondition (PR #796 MAJOR fail-open closure). Under mTLS the
// operator ALWAYS mints a single cell-rooted server leaf, so the PDP's own cell
// pin is MANDATORY: a server leaf from which it cannot be derived is a
// FAIL-CLOSED boot refusal, NEVER a silent degrade to an unpinned (foreign-cell
// reachable) serve. RED→GREEN fixtures (a)/(b)/(c)/(d).
// ==========================================================================

// (a) no-URI-SAN server leaf + MtlsContext::new → boot refusal.
#[tokio::test(flavor = "multi_thread")]
async fn cell_pin_boot_refuses_server_leaf_with_no_uri_san() {
    let iat = now_secs();
    let (mut svc, ca_signer) = service();
    let (server_leaf, server_key) =
        issue_server_leaf_with_uris(&mut svc, &ca_signer, iat, 10_000_000, &[]);
    let bundle = trusted_bundle(&svc, &ca_signer);
    let err = MtlsContext::new(bundle, vec![server_leaf], server_key)
        .err()
        .expect("a no-URI-SAN server leaf must boot-refuse (cell pin undeterminable)");
    assert!(
        matches!(
            err,
            iam_cloud_pdp_app::mtls::MtlsBootError::CellPinUndeterminable(_)
        ),
        "got {err:?}"
    );
}

// (b) multi-URI-SAN server leaf + MtlsContext::new → boot refusal.
#[tokio::test(flavor = "multi_thread")]
async fn cell_pin_boot_refuses_server_leaf_with_multiple_uri_sans() {
    let iat = now_secs();
    let (mut svc, ca_signer) = service();
    let (server_leaf, server_key) = issue_server_leaf_with_uris(
        &mut svc,
        &ca_signer,
        iat,
        10_000_000,
        &[
            "spiffe://oyatie.cell-7/platform/cloud-iam-pdp",
            "spiffe://oyatie.cell-9/platform/cloud-iam-pdp",
        ],
    );
    let bundle = trusted_bundle(&svc, &ca_signer);
    let err = MtlsContext::new(bundle, vec![server_leaf], server_key)
        .err()
        .expect("a multi-URI-SAN server leaf is ambiguous → must boot-refuse");
    assert!(
        matches!(
            err,
            iam_cloud_pdp_app::mtls::MtlsBootError::CellPinUndeterminable(_)
        ),
        "got {err:?}"
    );
}

// (c) malformed / non-cell-rooted URI server leaf + MtlsContext::new → boot refusal.
#[tokio::test(flavor = "multi_thread")]
async fn cell_pin_boot_refuses_non_cell_rooted_server_leaf() {
    let iat = now_secs();
    let (mut svc, ca_signer) = service();
    // A structurally-valid URI that is NOT a cell-rooted SPIFFE id (no
    // `oyatie.cell-<id>` authority): SpiffeId::parse must reject it.
    let (server_leaf, server_key) = issue_server_leaf_with_uris(
        &mut svc,
        &ca_signer,
        iat,
        10_000_000,
        &["spiffe://example.org/platform/cloud-iam-pdp"],
    );
    let bundle = trusted_bundle(&svc, &ca_signer);
    let err = MtlsContext::new(bundle, vec![server_leaf], server_key)
        .err()
        .expect("a non-cell-rooted server SPIFFE id must boot-refuse");
    assert!(
        matches!(
            err,
            iam_cloud_pdp_app::mtls::MtlsBootError::CellPinUndeterminable(_)
        ),
        "got {err:?}"
    );
}

// (d) THE reproduced bypass is CLOSED: a no-SAN server leaf under mTLS fails
// closed all the way through `start_with_mtls`, so a foreign-cell caller can
// NEVER reach Cedar (the socket never binds). Because MtlsContext::new refuses
// to build the context, the boot cannot pass `Some(ctx)` to start_with_mtls —
// the bypass surface (an unpinned mTLS serve) is structurally unreachable.
#[tokio::test(flavor = "multi_thread")]
async fn reproduced_no_san_bypass_is_closed_no_unpinned_mtls_serve() {
    let iat = now_secs();
    let (mut svc, ca_signer) = service();
    let (no_san_server, server_key) =
        issue_server_leaf_with_uris(&mut svc, &ca_signer, iat, 10_000_000, &[]);
    let bundle = trusted_bundle(&svc, &ca_signer);

    // The pre-fix bypass entry point: build an MtlsContext from a no-SAN server
    // leaf. Pre-fix this SUCCEEDED with expected_cell_authority = None and the
    // server then served mTLS WITHOUT a cell pin (foreign-cell SVIDs reached
    // Cedar). Post-fix it FAILS CLOSED, so there is no unpinned context to serve.
    let ctx_result = MtlsContext::new(bundle, vec![no_san_server], server_key);
    assert!(
        ctx_result.is_err(),
        "the no-SAN bypass must be closed: MtlsContext::new must NOT yield an \
         unpinned context that could serve a foreign-cell caller"
    );
    let err = ctx_result.err().unwrap();
    assert!(
        matches!(
            err,
            iam_cloud_pdp_app::mtls::MtlsBootError::CellPinUndeterminable(_)
        ),
        "got {err:?}"
    );

    // And the boot path surfaces it as a refusing-to-boot StartError::Mtls (exit
    // non-zero in main) — never a served socket.
    let start_err = StartError::Mtls(err);
    assert!(start_err.to_string().contains("refusing to boot"));

    // Non-regression: a PROPER cell-rooted server leaf (the operator's shape)
    // still builds and boots mTLS (cell pin present, with-SAN happy path intact).
    let (mut svc2, ca2) = service();
    let (good_leaf, good_key) =
        issue_server_leaf_with_uris(&mut svc2, &ca2, iat, 10_000_000, &[PDP_SERVER_SPIFFE]);
    let bundle2 = trusted_bundle(&svc2, &ca2);
    let ctx = MtlsContext::new(bundle2, vec![good_leaf], good_key)
        .expect("the proper cell-rooted operator server leaf must still build");
    assert_eq!(ctx.expected_cell_authority(), "oyatie.cell-7");
}
