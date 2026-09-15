use axum::serve::Listener;
use std::{
    io,
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpListener, TcpStream},
    sync::{OwnedSemaphorePermit, Semaphore},
    task::JoinSet,
};
use tokio_rustls::{TlsAcceptor, server::TlsStream};

pub(super) struct HttpsListener {
    tcp: TcpListener,
    tls: TlsAcceptor,
    capacity: Arc<Semaphore>,
    handshakes: JoinSet<io::Result<(Connection, SocketAddr)>>,
}

pub(super) struct Connection {
    stream: TlsStream<TcpStream>,
    // Admission covers the entire connection, including its TLS handshake.
    _permit: OwnedSemaphorePermit,
}

impl HttpsListener {
    pub(super) fn new(tcp: TcpListener, tls: TlsAcceptor) -> Self {
        Self {
            tcp,
            tls,
            capacity: Arc::new(Semaphore::new(128)),
            handshakes: JoinSet::new(),
        }
    }
}

impl Listener for HttpsListener {
    type Io = Connection;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            tokio::select! {
                // Reap completed tasks before accepting more peers under load.
                biased;
                Some(result) = self.handshakes.join_next(), if !self.handshakes.is_empty() => {
                    if let Ok(Ok(connection)) = result { return connection; }
                }
                connection = self.tcp.accept() => {
                    match connection {
                        Ok((stream, address)) => {
                            let Ok(permit) = self.capacity.clone().try_acquire_owned() else { continue; };
                            let tls = self.tls.clone();
                            self.handshakes.spawn(async move {
                                let stream = tokio::time::timeout(Duration::from_secs(10), tls.accept(stream)).await??;
                                Ok((Connection { stream, _permit: permit }, address))
                            });
                        }
                        Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
                    }
                }
            }
        }
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.tcp.local_addr()
    }
}

impl AsyncRead for Connection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for Connection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().stream).poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}
