//! Live rustls mTLS transport for both PDP listeners (G002 slice-1b-ii;
//! ADR-0561, ADR-0506).
//!
//! Wraps each accepted TCP connection in a rustls server handshake that REQUIRES
//! and verifies a client SVID (via [`SvidClientCertVerifier`]), then surfaces the
//! verified peer-leaf DER to the application layer so
//! [`crate::mtls::SpiffeCallerAuth::authenticate_caller`] can bind the caller's
//! tenant from the SVID — the #717 closure, now on a live socket.
//!
//! ## One acceptor, both surfaces
//!
//! The SAME `tokio_rustls::TlsAcceptor` terminates REST (axum) and gRPC (tonic).
//! The peer leaf is captured at accept time and injected into the request:
//! - **gRPC**: a [`PeerCertStream`] yields [`PeerCertTlsStream`] values whose
//!   [`tonic::transport::server::Connected`] impl carries the leaf as
//!   [`PeerCertInfo`]; tonic inserts it into request extensions.
//! - **REST**: the manual hyper-util accept loop layers a tower
//!   [`Extension`](axum::Extension)`<PeerCertInfo>` onto the router per
//!   connection (axum 0.8 has no built-in rustls serve helper).
//!
//! This keeps ONE crypto policy (aws-lc-rs, TLS 1.3, X25519MLKEM768 first,
//! X25519 fallback, NO ring), ONE verifier, and ONE capture pattern across
//! both protocols — no tonic TLS feature is enabled.
//!
//! ## Fidelity boundary (ADR-0561 slice-1b-ii)
//!
//! The trust bundle + the PDP server identity are supplied as an
//! [`MtlsContext`]; the K8s cert-delivery (operator-reconciled projected Secret +
//! init-container SVID fetch) and the cloud-kms signer swap remain the residual
//! deferral. Everything here is exercised by the in-repo real-handshake E2E
//! fixtures, which build a real `MtlsContext` from the trustd `der` helpers.

use std::io;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use http_runtime_hyper_adapter::pqc_hybrid_tls13_server_config_builder;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;
use tonic::transport::server::{Connected, TcpConnectInfo};
use x509_parser::certificate::X509Certificate;
use x509_parser::pem::Pem;
use x509_parser::prelude::FromDer;

use os_trustd_domain::TrustBundle;
use os_trustd_domain::certificate::{CertUsage, Certificate};
use os_trustd_domain::signer::EcdsaP256Signer;
use os_trustd_domain::x509::{DistinguishedName, SubjectAltNames, Validity};

use iam_identity_workload_svid_kernel::SpiffeId;
use iam_identity_workload_svid_trustd::leaf_der;

use crate::client_cert_verifier::SvidClientCertVerifier;
use crate::mtls::{MtlsBootError, SpiffeCallerAuth};

/// The post-handshake peer leaf DER, carried into a request extension so the PEP
/// can authenticate the caller. `None` is impossible on a connection that
/// reached a handler (the handshake is client-auth-mandatory), but the type is
/// explicit so a handler still fail-closed-denies if the extension is absent.
#[derive(Clone, Debug)]
pub struct PeerCertInfo {
    /// The verified client leaf certificate DER (the SVID the PEP binds).
    pub leaf_der: Option<Vec<u8>>,
}

impl PeerCertInfo {
    fn from_session(session: &rustls::ServerConnection) -> Self {
        let leaf_der = session
            .peer_certificates()
            .and_then(|certs| certs.first())
            .map(|leaf| leaf.as_ref().to_vec());
        Self { leaf_der }
    }
}

/// The standard kubernetes.io/tls Secret member carrying the PDP server leaf
/// chain (PEM). Matches the cloud-kms operator + secrets-domain convention.
const TLS_CERT_FILE: &str = "tls.crt";
/// The standard kubernetes.io/tls Secret member carrying the PDP server private
/// key (PKCS#8 PEM).
const TLS_KEY_FILE: &str = "tls.key";
/// The CA-bundle member carrying one or more trusted CA certs (PEM).
const CA_CERT_FILE: &str = "ca.crt";

/// Why the delivered cert mount could not be turned into an [`MtlsContext`].
///
/// Every variant is fail-closed: a missing, empty, or malformed mount is a HARD
/// boot error, never a silent downgrade to plain TCP. The runtime source must
/// present complete, well-formed material or the process refuses to boot
/// (ADR-0561 slice-1b-iii; the SVID operator is the in-cluster delivery source).
#[derive(Debug)]
pub enum MtlsMaterialError {
    /// A required mount file is missing or could not be read.
    MountUnreadable {
        /// The file path that could not be read.
        path: PathBuf,
        /// The underlying IO error.
        source: io::Error,
    },
    /// A required mount file was present but zero-length / whitespace-only.
    Empty {
        /// The empty file path.
        path: PathBuf,
    },
    /// A mount file did not parse as the PEM material it must carry.
    MalformedPem {
        /// The malformed file path.
        path: PathBuf,
        /// Diagnostic detail (never trust material).
        detail: String,
    },
    /// `ca.crt` parsed but carried zero CA certificates — there is no trust
    /// root to anchor caller SVIDs against.
    NoCaAnchors {
        /// The `ca.crt` path.
        path: PathBuf,
    },
    /// The material was well-formed but [`MtlsContext::new`] rejected it (empty
    /// bundle or a server identity rustls cannot use) — boot-refused.
    Boot(MtlsBootError),
}

impl std::fmt::Display for MtlsMaterialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MountUnreadable { path, source } => {
                write!(f, "mTLS cert mount {} unreadable: {source}", path.display())
            }
            Self::Empty { path } => write!(f, "mTLS cert mount {} is empty", path.display()),
            Self::MalformedPem { path, detail } => {
                write!(f, "mTLS cert mount {} malformed: {detail}", path.display())
            }
            Self::NoCaAnchors { path } => write!(
                f,
                "mTLS cert mount {} carried no CA certificates",
                path.display()
            ),
            Self::Boot(err) => write!(f, "mTLS context rejected: {err}"),
        }
    }
}

impl std::error::Error for MtlsMaterialError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MountUnreadable { source, .. } => Some(source),
            Self::Boot(err) => Some(err),
            _ => None,
        }
    }
}

/// The PDP server identity + the trust root for caller SVIDs. Owns the bundle so
/// no borrow lifetime escapes into the boot signature (the production signer is
/// monomorphic [`EcdsaP256Signer`]; the cloud-kms swap replaces this one type).
pub struct MtlsContext {
    bundle: Arc<TrustBundle<EcdsaP256Signer>>,
    server_chain: Vec<CertificateDer<'static>>,
    server_key_pkcs8_der: Vec<u8>,
    /// The cell authority (`oyatie.cell-<id>`) the PDP serves, derived from its
    /// OWN server leaf's SPIFFE id (`spiffe://oyatie.cell-<id>/platform/cloud-iam-pdp`).
    /// ALWAYS present: deriving it is a FAIL-CLOSED boot precondition under mTLS
    /// ([`MtlsContext::new`] refuses to construct when it cannot be derived), so an
    /// `MtlsContext` can never serve mTLS without enforcing caller cell-pinning.
    expected_cell_authority: String,
}

impl MtlsContext {
    /// Build the mTLS context from a trust bundle, the PDP server leaf chain
    /// (DER), and the PDP server private key (PKCS#8 DER).
    ///
    /// Deriving the PDP's OWN cell authority from its server leaf is a FAIL-CLOSED
    /// boot precondition: under mTLS the operator ALWAYS mints a single
    /// cell-rooted server leaf (`spiffe://oyatie.cell-<id>/platform/cloud-iam-pdp`),
    /// so the cell pin is MANDATORY. An empty chain, a leaf with no / more than one
    /// URI SAN, or a URI that is not a cell-rooted SPIFFE id is a boot REFUSAL —
    /// the PDP never serves mTLS without enforcing caller cell-pinning (that would
    /// silently disable cell-isolation, letting a foreign-cell SVID reach Cedar).
    ///
    /// # Errors
    /// [`MtlsBootError::TrustBundleEmpty`] when the bundle holds no anchors — a
    /// server that cannot prove a trust root must never accept a caller
    /// (boot-fatal, mirroring [`SpiffeCallerAuth::new`]).
    /// [`MtlsBootError::CellPinUndeterminable`] when the server leaf does not yield
    /// a single cell-rooted SPIFFE identity (no/multi URI SAN, malformed, or
    /// non-cell-rooted) — the cell pin cannot be established, so the boot is
    /// refused fail-closed.
    pub fn new(
        bundle: TrustBundle<EcdsaP256Signer>,
        server_chain_der: Vec<Vec<u8>>,
        server_key_pkcs8_der: Vec<u8>,
    ) -> Result<Self, MtlsBootError> {
        if bundle.is_empty() {
            return Err(MtlsBootError::TrustBundleEmpty);
        }
        // Derive the PDP's OWN cell authority from its server leaf's SPIFFE id
        // (the leaf is the existing source of truth; the operator mints it as
        // `spiffe://oyatie.cell-<id>/platform/cloud-iam-pdp`). The leaf has already
        // been validated by rustls at boot, so we only READ its identity here —
        // but READING it is MANDATORY and FAIL-CLOSED: every failure below is a
        // boot refusal, never a silent `None` that would disable cell-pinning.
        let leaf = server_chain_der.first().ok_or_else(|| {
            MtlsBootError::CellPinUndeterminable("mTLS server leaf chain is empty".to_string())
        })?;
        let uri = leaf_der::extract_single_uri_san(leaf).map_err(|err| {
            MtlsBootError::CellPinUndeterminable(format!(
                "server leaf URI SAN unusable for cell derivation: {err:?}"
            ))
        })?;
        let server_spiffe_id = SpiffeId::parse(&uri).map_err(|err| {
            MtlsBootError::CellPinUndeterminable(format!(
                "server leaf SPIFFE id is not cell-rooted: {err}"
            ))
        })?;
        let expected_cell_authority = server_spiffe_id.trust_domain_authority().to_string();
        let server_chain = server_chain_der
            .into_iter()
            .map(CertificateDer::from)
            .collect();
        Ok(Self {
            bundle: Arc::new(bundle),
            server_chain,
            server_key_pkcs8_der,
            expected_cell_authority,
        })
    }

    /// Build the mTLS context from a delivered cert mount directory (the
    /// kubernetes.io/tls Secret projection: `tls.crt`, `tls.key`, `ca.crt`).
    ///
    /// This is the PRODUCTION runtime source `main` boots from (ADR-0561
    /// slice-1b-iii-a). It is fail-closed at EVERY step — a missing, empty, or
    /// malformed mount is a HARD error, never a downgrade to plain TCP:
    /// - `<dir>/tls.crt` — the PDP server leaf chain (PEM → `Vec<CertificateDer>`),
    ///   passed verbatim to [`MtlsContext::new`] (rustls re-validates in
    ///   [`MtlsContext::build_acceptor`]).
    /// - `<dir>/tls.key` — the PDP server private key (PKCS#8 PEM → PKCS#8 DER).
    /// - `<dir>/ca.crt` — one or more CA certs. For EACH, the REAL
    ///   `SubjectPublicKeyInfo` DER is extracted (via `x509-parser`) and a trust
    ///   anchor [`Certificate`] is reconstructed carrying that SPKI as its
    ///   `public_key_der` — the value the rustls verify path consults via
    ///   [`TrustBundle::trusted_ca_spki_ders`]. Zero CA certs → [`MtlsMaterialError::NoCaAnchors`].
    ///
    /// # Correctness (the rustls verify path consults only the real SPKI)
    /// The live `SvidClientCertVerifier` defers to `TrustdSvidVerifier`, which
    /// verifies a presented leaf's real signature against each anchor's
    /// `public_key_der`. The `EcdsaP256Signer` attached to each anchor and the
    /// shape-model `Certificate.signature` are NOT consulted on that path, so the
    /// anchor carries the parsed CA's real SPKI, a non-empty CN + sane validity +
    /// a placeholder signature (so `Certificate::validate` passes), and an inert
    /// freshly-generated signer (only to satisfy the `add_anchor` type).
    ///
    /// # Errors
    /// [`MtlsMaterialError`] for an unreadable/empty/malformed mount, a `ca.crt`
    /// with no CA certs, or an [`MtlsContext::new`] boot refusal.
    pub fn from_path(dir: &Path) -> Result<MtlsContext, MtlsMaterialError> {
        let server_chain_der = read_cert_chain_der(&dir.join(TLS_CERT_FILE))?;
        let server_key_pkcs8_der = read_private_key_pkcs8_der(&dir.join(TLS_KEY_FILE))?;
        let bundle = read_trust_bundle(&dir.join(CA_CERT_FILE))?;
        MtlsContext::new(bundle, server_chain_der, server_key_pkcs8_der)
            .map_err(MtlsMaterialError::Boot)
    }

    /// A clonable handle to the trust bundle (the PEP borrows it per request).
    #[must_use]
    pub fn bundle(&self) -> Arc<TrustBundle<EcdsaP256Signer>> {
        Arc::clone(&self.bundle)
    }

    /// The PDP's own cell authority (`oyatie.cell-<id>`), derived (fail-closed at
    /// construction) from its server leaf's SPIFFE id, used to pin a caller's
    /// cell. ALWAYS present — an `MtlsContext` cannot exist without it.
    #[must_use]
    pub fn expected_cell_authority(&self) -> &str {
        &self.expected_cell_authority
    }

    /// Build the rustls [`TlsAcceptor`] (aws-lc-rs provider, TLS 1.3, PQC-hybrid
    /// first, NO ring) requiring a verified client SVID.
    ///
    /// # Errors
    /// [`MtlsBootError::ServerConfig`] when the server leaf/key are rejected by
    /// rustls (malformed DER, key/cert mismatch) — boot-fatal.
    pub fn build_acceptor(&self) -> Result<TlsAcceptor, MtlsBootError> {
        let verifier = SvidClientCertVerifier::new(Arc::clone(&self.bundle));
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(self.server_key_pkcs8_der.clone()));
        let config = pqc_hybrid_tls13_server_config_builder()
            .with_client_cert_verifier(Arc::new(verifier))
            .with_single_cert(self.server_chain.clone(), key)
            .map_err(|err| MtlsBootError::ServerConfig(err.to_string()))?;
        Ok(TlsAcceptor::from(Arc::new(config)))
    }
}

/// Read a mount file, fail-closed: a missing/unreadable file is
/// [`MtlsMaterialError::MountUnreadable`]; a zero-length / whitespace-only file
/// is [`MtlsMaterialError::Empty`].
fn read_mount_bytes(path: &Path) -> Result<Vec<u8>, MtlsMaterialError> {
    let bytes = std::fs::read(path).map_err(|source| MtlsMaterialError::MountUnreadable {
        path: path.to_path_buf(),
        source,
    })?;
    if bytes.iter().all(|b| b.is_ascii_whitespace()) {
        return Err(MtlsMaterialError::Empty {
            path: path.to_path_buf(),
        });
    }
    Ok(bytes)
}

/// Parse `tls.crt` (the server leaf chain PEM) into DER chunks. Each `CERTIFICATE`
/// PEM block is one chain element; passed verbatim to [`MtlsContext::new`] (rustls
/// re-validates the chain + key match in `build_acceptor`). At least one block is
/// required.
fn read_cert_chain_der(path: &Path) -> Result<Vec<Vec<u8>>, MtlsMaterialError> {
    let bytes = read_mount_bytes(path)?;
    let mut chain = Vec::new();
    for block in Pem::iter_from_buffer(&bytes) {
        let pem = block.map_err(|e| MtlsMaterialError::MalformedPem {
            path: path.to_path_buf(),
            detail: format!("PEM parse failed: {e}"),
        })?;
        if pem.label == "CERTIFICATE" {
            chain.push(pem.contents);
        }
    }
    if chain.is_empty() {
        return Err(MtlsMaterialError::MalformedPem {
            path: path.to_path_buf(),
            detail: "no CERTIFICATE PEM block in server leaf chain".to_string(),
        });
    }
    Ok(chain)
}

/// Parse `tls.key` (the PDP server private key, PKCS#8 PEM) into PKCS#8 DER. The
/// first private-key PEM block is taken; rustls validates it against the leaf in
/// `build_acceptor` (a key/cert mismatch is a boot refusal there).
fn read_private_key_pkcs8_der(path: &Path) -> Result<Vec<u8>, MtlsMaterialError> {
    let bytes = read_mount_bytes(path)?;
    for block in Pem::iter_from_buffer(&bytes) {
        let pem = block.map_err(|e| MtlsMaterialError::MalformedPem {
            path: path.to_path_buf(),
            detail: format!("PEM parse failed: {e}"),
        })?;
        // PKCS#8 ("PRIVATE KEY") is the kubernetes.io/tls convention; accept the
        // RFC 5915 / SEC1 ("EC PRIVATE KEY") and RSA labels' bytes too so a
        // mislabeled-but-PKCS#8 key still flows (rustls is the final arbiter).
        if pem.label.ends_with("PRIVATE KEY") {
            return Ok(pem.contents);
        }
    }
    Err(MtlsMaterialError::MalformedPem {
        path: path.to_path_buf(),
        detail: "no PRIVATE KEY PEM block in server key".to_string(),
    })
}

/// Parse `ca.crt` (one or more CA certs, PEM) into a [`TrustBundle`]. For EACH
/// `CERTIFICATE` block, extract the REAL `SubjectPublicKeyInfo` DER and rebuild a
/// trust anchor [`Certificate`] carrying it as `public_key_der` (the value the
/// rustls verify path consults), with `usage = CertificateAuthority`, the parsed
/// CN, a sane validity, and a placeholder signature (so `validate()` passes). A
/// freshly-generated inert signer satisfies `add_anchor`'s type but is NOT on the
/// live verify path. Zero CA certs → [`MtlsMaterialError::NoCaAnchors`].
fn read_trust_bundle(path: &Path) -> Result<TrustBundle<EcdsaP256Signer>, MtlsMaterialError> {
    let bytes = read_mount_bytes(path)?;
    let mut bundle = TrustBundle::new();
    let mut anchors = 0usize;
    for block in Pem::iter_from_buffer(&bytes) {
        let pem = block.map_err(|e| MtlsMaterialError::MalformedPem {
            path: path.to_path_buf(),
            detail: format!("PEM parse failed: {e}"),
        })?;
        if pem.label != "CERTIFICATE" {
            continue;
        }
        let anchor = ca_anchor_from_der(&pem.contents, path)?;
        // A freshly-generated signer is inert on the live verify path (which
        // consults only the anchor SPKI); it exists only to type `add_anchor`.
        let inert_signer =
            EcdsaP256Signer::generate().map_err(|e| MtlsMaterialError::MalformedPem {
                path: path.to_path_buf(),
                detail: format!("inert anchor signer generation failed: {e}"),
            })?;
        bundle
            .add_anchor(anchor, inert_signer)
            .map_err(|e| MtlsMaterialError::MalformedPem {
                path: path.to_path_buf(),
                detail: format!("trust anchor rejected: {e}"),
            })?;
        anchors += 1;
    }
    if anchors == 0 {
        return Err(MtlsMaterialError::NoCaAnchors {
            path: path.to_path_buf(),
        });
    }
    Ok(bundle)
}

/// Reconstruct a trust-anchor [`Certificate`] from a real CA cert DER, carrying
/// the parsed CA's REAL SPKI as `public_key_der`. The CN + validity come from the
/// parsed cert; the signature is a non-empty placeholder (the live verify path
/// never consults the anchor's `signature`, but `Certificate::validate` requires
/// it non-empty).
fn ca_anchor_from_der(der: &[u8], path: &Path) -> Result<Certificate, MtlsMaterialError> {
    let (rest, parsed) =
        X509Certificate::from_der(der).map_err(|e| MtlsMaterialError::MalformedPem {
            path: path.to_path_buf(),
            detail: format!("CA certificate DER parse failed: {e}"),
        })?;
    if !rest.is_empty() {
        return Err(MtlsMaterialError::MalformedPem {
            path: path.to_path_buf(),
            detail: format!("{} trailing bytes after a CA certificate", rest.len()),
        });
    }

    // The REAL SubjectPublicKeyInfo DER — the value the rustls verify path checks
    // a presented leaf's signature against (the CRITICAL correctness finding).
    let spki_der = parsed.public_key().raw.to_vec();

    // Common name: the first CN attribute, falling back to the full subject DN.
    let common_name = parsed
        .subject()
        .iter_common_name()
        .next()
        .and_then(|cn| cn.as_str().ok().map(str::to_owned))
        .filter(|cn| !cn.is_empty())
        .unwrap_or_else(|| parsed.subject().to_string());
    if common_name.is_empty() {
        return Err(MtlsMaterialError::MalformedPem {
            path: path.to_path_buf(),
            detail: "CA certificate has no common name".to_string(),
        });
    }

    // Validity window from the parsed cert; clamp to a sane non-empty window so
    // `Certificate::validate` passes even for an exotic-but-valid CA window.
    let not_before = u64::try_from(parsed.validity().not_before.timestamp()).unwrap_or(0);
    let not_after_raw = u64::try_from(parsed.validity().not_after.timestamp()).unwrap_or(u64::MAX);
    let not_after = if not_after_raw > not_before {
        not_after_raw
    } else {
        not_before.saturating_add(1)
    };

    // Serial is not consulted on the live verify path; fold the real serial's
    // low bytes into a stable u64 (the shape model's serial type).
    let serial = parsed
        .raw_serial()
        .iter()
        .fold(0u64, |acc, &b| acc.wrapping_shl(8) | u64::from(b));

    Ok(Certificate {
        serial,
        subject: DistinguishedName::common(common_name),
        issuer: DistinguishedName::common(
            parsed
                .issuer()
                .iter_common_name()
                .next()
                .and_then(|cn| cn.as_str().ok().map(str::to_owned))
                .unwrap_or_else(|| "ca".to_string()),
        ),
        validity: Validity {
            not_before,
            not_after,
        },
        usage: CertUsage::CertificateAuthority,
        sans: SubjectAltNames::default(),
        public_key_der: spki_der,
        // Non-empty placeholder: NOT on the live verify path, but
        // `Certificate::validate` rejects an empty signature.
        signature: vec![0x01],
    })
}

/// Build a [`SpiffeCallerAuth`] PEP borrowing `bundle` (boot-refuses empty),
/// pinning the caller's cell to `expected_cell_authority` when one is supplied
/// (the PDP's own cell). `None` ⇒ no cell pin (legacy behaviour).
///
/// # Errors
/// [`MtlsBootError::TrustBundleEmpty`] (delegated to the constructor).
pub fn pep_for<'a>(
    bundle: &'a TrustBundle<EcdsaP256Signer>,
    expected_cell_authority: Option<&str>,
) -> Result<SpiffeCallerAuth<'a, EcdsaP256Signer>, MtlsBootError> {
    match expected_cell_authority {
        Some(cell) => SpiffeCallerAuth::with_cell_pin(bundle, cell),
        None => SpiffeCallerAuth::new(bundle),
    }
}

// ===================================================================
// gRPC: a Connected TLS stream carrying the captured peer leaf
// ===================================================================

/// A TLS-terminated connection whose [`Connected`] impl surfaces the verified
/// peer leaf as a [`PeerCertInfo`] request extension (no tonic TLS feature).
pub struct PeerCertTlsStream {
    inner: TlsStream<TcpStream>,
    peer: PeerCertInfo,
    tcp: TcpConnectInfo,
}

impl Connected for PeerCertTlsStream {
    type ConnectInfo = PeerCertInfo;

    fn connect_info(&self) -> Self::ConnectInfo {
        self.peer.clone()
    }
}

impl PeerCertTlsStream {
    /// The captured TCP connect info (kept for parity / future diagnostics).
    #[must_use]
    pub fn tcp_info(&self) -> &TcpConnectInfo {
        &self.tcp
    }
}

impl AsyncRead for PeerCertTlsStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for PeerCertTlsStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// Terminate one accepted TCP connection with `acceptor`, capturing the peer
/// leaf. Returns the connected stream ready to feed to tonic.
///
/// # Errors
/// The TLS handshake error (untrusted/absent client cert ⇒ aborted handshake) —
/// the caller drops the connection (fail-closed, the rogue/no-cert path).
pub async fn accept_grpc(acceptor: &TlsAcceptor, tcp: TcpStream) -> io::Result<PeerCertTlsStream> {
    let tcp_info = TcpConnectInfo {
        local_addr: tcp.local_addr().ok(),
        remote_addr: tcp.peer_addr().ok(),
    };
    let tls = acceptor.accept(tcp).await?;
    let peer = {
        let (_io, session) = tls.get_ref();
        PeerCertInfo::from_session(session)
    };
    Ok(PeerCertTlsStream {
        inner: tls,
        peer,
        tcp: tcp_info,
    })
}

/// Terminate one accepted TCP connection for the REST surface, returning the raw
/// [`TlsStream`] plus the captured peer leaf (the hyper-util loop wraps it).
///
/// # Errors
/// The TLS handshake error (fail-closed: rogue / no client cert).
pub async fn accept_rest(
    acceptor: &TlsAcceptor,
    tcp: TcpStream,
) -> io::Result<(TlsStream<TcpStream>, PeerCertInfo)> {
    let tls = acceptor.accept(tcp).await?;
    let peer = {
        let (_io, session) = tls.get_ref();
        PeerCertInfo::from_session(session)
    };
    Ok((tls, peer))
}

/// Accept one TCP connection from `listener`.
///
/// # Errors
/// The accept error (transient; the caller logs and continues).
pub async fn accept_tcp(listener: &TcpListener) -> io::Result<TcpStream> {
    let (tcp, _peer) = listener.accept().await?;
    let _ = tcp.set_nodelay(true);
    Ok(tcp)
}
