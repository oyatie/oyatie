#![forbid(unsafe_code)]
mod attempt;
mod transaction;
mod wire;
use attempt::Failure;
mod mx;
pub use hickory_resolver::TokioResolver;
use mail_api::{DeliveryOutcome, MailTransport, QueuedMessage};
use mail_kernel::{Error, valid_address};
pub use mx::{MxConfig, MxTransport};
use rustls::{ClientConfig, pki_types::ServerName};
use std::{future::Future, pin::Pin, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncWrite, BufReader},
    net::TcpStream,
};
use tokio_rustls::TlsConnector;
use wire::{capability, command, expect, temporary};

pub struct RelayConfig {
    pub host: String,
    pub port: u16,
    pub helo: String,
    pub implicit_tls: bool,
    pub credentials: Option<(String, String)>,
}

pub struct Relay {
    config: RelayConfig,
    tls: TlsConnector,
}

impl Relay {
    pub fn new(config: RelayConfig, trust: Arc<ClientConfig>) -> Result<Self, Error> {
        if config.port == 0
            || ServerName::try_from(config.host.clone()).is_err()
            || !valid_address(&format!("postmaster@{}", config.helo))
            || config.credentials.as_ref().is_some_and(|(user, pass)| {
                user.is_empty()
                    || pass.is_empty()
                    || user.len() + pass.len() > 4096
                    || user.chars().chain(pass.chars()).any(char::is_control)
            })
        {
            return Err(Error::Invalid);
        }
        Ok(Self {
            config,
            tls: TlsConnector::from(trust),
        })
    }

    async fn deliver(&self, recipient: &str, message: &QueuedMessage) -> Result<(), Failure> {
        let tcp = tokio::time::timeout(
            Duration::from_secs(300),
            TcpStream::connect((self.config.host.as_str(), self.config.port)),
        )
        .await
        .map_err(|_| temporary())?
        .map_err(|_| temporary())?;
        self.deliver_connected(tcp, recipient, message, true).await
    }

    async fn deliver_connected(
        &self,
        tcp: TcpStream,
        recipient: &str,
        message: &QueuedMessage,
        require_tls: bool,
    ) -> Result<(), Failure> {
        let mut tcp = BufReader::new(tcp);
        if !self.config.implicit_tls {
            expect(wire::reply(&mut tcp).await?, 220).map_err(|_| temporary())?;
            let hello = self.hello(&mut tcp, !require_tls).await?;
            if !capability(&hello, "STARTTLS") {
                return if require_tls {
                    Err(temporary().into())
                } else {
                    self.transaction(&mut tcp, recipient, message, hello).await
                };
            }
            expect(command(&mut tcp, b"STARTTLS\r\n").await?, 220).map_err(|_| temporary())?;
            if !tcp.buffer().is_empty() {
                return Err(temporary().into());
            }
        }
        let name = ServerName::try_from(self.config.host.clone()).map_err(|_| temporary())?;
        let stream = tokio::time::timeout(
            Duration::from_secs(60),
            self.tls.connect(name, tcp.into_inner()),
        )
        .await
        .map_err(|_| temporary())?
        .map_err(|_| temporary())?;
        let mut stream = BufReader::new(stream);
        if self.config.implicit_tls {
            expect(wire::reply(&mut stream).await?, 220).map_err(|_| temporary())?;
        }
        let hello = self.hello(&mut stream, false).await?;
        self.transaction(&mut stream, recipient, message, hello)
            .await
    }

    async fn hello<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        stream: &mut BufReader<S>,
        allow_legacy: bool,
    ) -> Result<smtp_proto::Response<String>, DeliveryOutcome> {
        let reply = command(stream, format!("EHLO {}\r\n", self.config.helo).as_bytes()).await?;
        if allow_legacy && matches!(reply.code, 500 | 502) {
            let mut reply = expect(
                command(stream, format!("HELO {}\r\n", self.config.helo).as_bytes()).await?,
                250,
            )
            .map_err(|_| temporary())?;
            // HELO text is not ESMTP capability advertisement.
            reply.message.clear();
            Ok(reply)
        } else {
            expect(reply, 250).map_err(|_| temporary())
        }
    }
}

impl MailTransport for Relay {
    fn send<'a>(
        &'a self,
        recipient: &'a str,
        message: &'a QueuedMessage,
    ) -> Pin<Box<dyn Future<Output = DeliveryOutcome> + Send + 'a>> {
        Box::pin(async move {
            if !valid_address(&message.sender)
                || !valid_address(recipient)
                || !wire::message_valid(&message.raw)
            {
                return DeliveryOutcome::Permanent(554);
            }
            match self.deliver(recipient, message).await {
                Ok(()) => DeliveryOutcome::Delivered,
                Err(failure) => failure.outcome,
            }
        })
    }
}
