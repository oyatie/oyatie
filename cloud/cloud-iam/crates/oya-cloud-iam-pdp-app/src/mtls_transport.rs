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
//! This keeps ONE crypto provider (aws-lc-rs, NO ring), ONE verifier, and ONE
//! capture pattern across both protocols — no tonic TLS feature is enabled.
//!
//! ## Fidelity boundary (ADR-0561 slice-1b-ii)
//!
//! The trust bundle + the PDP server identity are supplied as an
//! [`MtlsContext`]; the K8s cert-delivery (operator-reconciled projected Secret +
//! init-container SVID fetch) and the cloud-kms signer swap remain the residual
//! deferral. Everything here is exercised by the in-repo real-handshake E2E
//! fixtures, which build a real `MtlsContext` from the trustd `der` helpers.

use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::ServerConfig;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;
use tonic::transport::server::{Connected, TcpConnectInfo};

use oya_cloud_os_trustd_domain::signer::EcdsaP256Signer;
use oya_cloud_os_trustd_domain::TrustBundle;

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

/// The PDP server identity + the trust root for caller SVIDs. Owns the bundle so
/// no borrow lifetime escapes into the boot signature (the production signer is
/// monomorphic [`EcdsaP256Signer`]; the cloud-kms swap replaces this one type).
pub struct MtlsContext {
    bundle: Arc<TrustBundle<EcdsaP256Signer>>,
    server_chain: Vec<CertificateDer<'static>>,
    server_key_pkcs8_der: Vec<u8>,
}

impl MtlsContext {
    /// Build the mTLS context from a trust bundle, the PDP server leaf chain
    /// (DER), and the PDP server private key (PKCS#8 DER).
    ///
    /// # Errors
    /// [`MtlsBootError::TrustBundleEmpty`] when the bundle holds no anchors — a
    /// server that cannot prove a trust root must never accept a caller
    /// (boot-fatal, mirroring [`SpiffeCallerAuth::new`]).
    pub fn new(
        bundle: TrustBundle<EcdsaP256Signer>,
        server_chain_der: Vec<Vec<u8>>,
        server_key_pkcs8_der: Vec<u8>,
    ) -> Result<Self, MtlsBootError> {
        if bundle.is_empty() {
            return Err(MtlsBootError::TrustBundleEmpty);
        }
        let server_chain = server_chain_der.into_iter().map(CertificateDer::from).collect();
        Ok(Self {
            bundle: Arc::new(bundle),
            server_chain,
            server_key_pkcs8_der,
        })
    }

    /// A clonable handle to the trust bundle (the PEP borrows it per request).
    #[must_use]
    pub fn bundle(&self) -> Arc<TrustBundle<EcdsaP256Signer>> {
        Arc::clone(&self.bundle)
    }

    /// Build the rustls [`TlsAcceptor`] (aws-lc-rs provider, NO ring) requiring a
    /// verified client SVID.
    ///
    /// # Errors
    /// [`MtlsBootError::ServerConfig`] when the server leaf/key are rejected by
    /// rustls (malformed DER, key/cert mismatch) — boot-fatal.
    pub fn build_acceptor(&self) -> Result<TlsAcceptor, MtlsBootError> {
        let verifier = SvidClientCertVerifier::new(Arc::clone(&self.bundle));
        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
            self.server_key_pkcs8_der.clone(),
        ));
        let config = ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::aws_lc_rs::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .map_err(|err| MtlsBootError::ServerConfig(err.to_string()))?
        .with_client_cert_verifier(Arc::new(verifier))
        .with_single_cert(self.server_chain.clone(), key)
        .map_err(|err| MtlsBootError::ServerConfig(err.to_string()))?;
        Ok(TlsAcceptor::from(Arc::new(config)))
    }
}

/// Build a [`SpiffeCallerAuth`] PEP borrowing `bundle` (boot-refuses empty).
///
/// # Errors
/// [`MtlsBootError::TrustBundleEmpty`] (delegated to [`SpiffeCallerAuth::new`]).
pub fn pep_for(
    bundle: &TrustBundle<EcdsaP256Signer>,
) -> Result<SpiffeCallerAuth<'_, EcdsaP256Signer>, MtlsBootError> {
    SpiffeCallerAuth::new(bundle)
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
pub async fn accept_grpc(
    acceptor: &TlsAcceptor,
    tcp: TcpStream,
) -> io::Result<PeerCertTlsStream> {
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
