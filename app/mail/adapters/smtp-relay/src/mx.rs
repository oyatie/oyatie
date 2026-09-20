use super::{Relay, RelayConfig, wire};
use hickory_resolver::{
    Resolver, TokioResolver,
    config::{LookupIpStrategy, NameServerConfig, ResolveHosts, ResolverConfig},
    net::runtime::TokioRuntimeProvider,
    proto::rr::RData,
};
use mail_api::{DeliveryOutcome, MailTransport, QueuedMessage};
use mail_kernel::{Error, valid_address};
use rustls::ClientConfig;
use std::{
    collections::hash_map::RandomState, future::Future, hash::BuildHasher, net::SocketAddr,
    pin::Pin, sync::Arc, time::Duration,
};
use tokio::net::TcpStream;

/// Direct delivery uses the supplied recursive DNS servers, including their TCP
/// fallback. TLS is always verified when offered; required TLS also refuses a
/// destination which omits STARTTLS. No relay credentials are sent to MX hosts.
pub struct MxConfig {
    pub helo: String,
    pub port: u16,
    pub dns_servers: Vec<SocketAddr>,
    pub require_tls: bool,
}

pub struct MxTransport {
    config: MxConfig,
    resolver: TokioResolver,
    trust: Arc<ClientConfig>,
}

impl MxTransport {
    pub fn new(config: MxConfig, trust: Arc<ClientConfig>) -> Result<Self, Error> {
        if config.port == 0
            || config.dns_servers.is_empty()
            || config.dns_servers.len() > 8
            || config
                .dns_servers
                .iter()
                .any(|s| s.port() == 0 || s.ip().is_unspecified() || s.ip().is_multicast())
            || !valid_address(&format!("postmaster@{}", config.helo))
        {
            return Err(Error::Invalid);
        }
        let servers = config
            .dns_servers
            .iter()
            .map(|address| {
                let mut server = NameServerConfig::udp_and_tcp(address.ip());
                for connection in &mut server.connections {
                    connection.port = address.port();
                }
                server
            })
            .collect();
        let mut builder = Resolver::builder_with_config(
            ResolverConfig::from_name_servers(servers),
            TokioRuntimeProvider::default(),
        );
        builder.options_mut().ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
        builder.options_mut().use_hosts_file = ResolveHosts::Never;
        Ok(Self {
            config,
            resolver: builder.build().map_err(|_| Error::Unavailable)?,
            trust,
        })
    }

    /// The resolver this transport already validated and built.
    ///
    /// Message authentication resolves the same names against the same
    /// servers, and a second resolver would be a second DNS configuration to
    /// keep in step. Cloning is cheap: the pool is shared behind an `Arc`.
    pub fn resolver(&self) -> TokioResolver {
        self.resolver.clone()
    }

    async fn hosts(&self, domain: &str) -> Result<Vec<String>, DeliveryOutcome> {
        let answer = match self.resolver.mx_lookup(format!("{domain}.")).await {
            Ok(answer) => answer,
            Err(error) if error.is_nx_domain() => return Err(DeliveryOutcome::Permanent(556)),
            Err(error) if error.is_no_records_found() => {
                if domain.eq_ignore_ascii_case(self.config.helo.trim_end_matches('.')) {
                    return Err(DeliveryOutcome::Permanent(554));
                }
                return Ok(vec![domain.to_string()]);
            }
            Err(_) => return Err(wire::temporary()),
        };
        let mut hosts = Vec::new();
        for record in answer.answers() {
            if let RData::MX(mx) = &record.data {
                hosts.push((
                    mx.preference,
                    mx.exchange
                        .to_utf8()
                        .trim_end_matches('.')
                        .to_ascii_lowercase(),
                ));
            }
        }
        if hosts.is_empty() || hosts.len() > 64 {
            return Err(wire::temporary());
        }
        // RFC 7505: a sole null MX explicitly declares that mail is not accepted.
        // Mixed null/ordinary MX is invalid configuration; never resolve root.
        if hosts.iter().any(|(_, host)| host.is_empty()) {
            return Err(DeliveryOutcome::Permanent(556));
        }
        // RFC 5321: remove this system and every equal-or-worse preference to
        // prevent routing loops when our own hostname appears in the MX set.
        if let Some(preference) = hosts
            .iter()
            .filter(|(_, host)| host.eq_ignore_ascii_case(self.config.helo.trim_end_matches('.')))
            .map(|(p, _)| *p)
            .min()
        {
            hosts.retain(|(p, _)| *p < preference);
        }
        if hosts.is_empty() {
            return Err(DeliveryOutcome::Permanent(554));
        }
        let random = RandomState::new();
        hosts.sort_by_key(|(preference, host)| (*preference, random.hash_one(host)));
        hosts.dedup();
        Ok(hosts.into_iter().map(|(_, host)| host).collect())
    }

    async fn deliver(&self, recipient: &str, message: &QueuedMessage) -> DeliveryOutcome {
        let domain = recipient.rsplit_once('@').unwrap().1;
        let hosts = match self.hosts(domain).await {
            Ok(hosts) => hosts,
            Err(error) => return error,
        };
        let mut outcome = DeliveryOutcome::Permanent(550);
        for host in hosts {
            let addresses = match self.resolver.lookup_ip(format!("{host}.")).await {
                Ok(addresses) => addresses,
                Err(error) => {
                    if !error.is_no_records_found() {
                        outcome = wire::temporary();
                    }
                    continue;
                }
            };
            let addresses: Vec<_> = addresses.iter().take(65).collect();
            if addresses.len() > 64 {
                return wire::temporary();
            }
            let relay = match Relay::new(
                RelayConfig {
                    host,
                    port: self.config.port,
                    helo: self.config.helo.clone(),
                    implicit_tls: false,
                    credentials: None,
                },
                self.trust.clone(),
            ) {
                Ok(relay) => relay,
                Err(_) => {
                    outcome = wire::temporary();
                    continue;
                }
            };
            for ip in addresses {
                let stream = match tokio::time::timeout(
                    Duration::from_secs(300),
                    TcpStream::connect(SocketAddr::new(ip, self.config.port)),
                )
                .await
                {
                    Ok(Ok(stream)) => stream,
                    _ => {
                        outcome = wire::temporary();
                        continue;
                    }
                };
                match relay
                    .deliver_connected(stream, recipient, message, self.config.require_tls)
                    .await
                {
                    Ok(()) => return DeliveryOutcome::Delivered,
                    Err(error)
                        if !error.retry_route
                            || matches!(error.outcome, DeliveryOutcome::Permanent(_)) =>
                    {
                        return error.outcome;
                    }
                    Err(error) => outcome = error.outcome,
                }
            }
        }
        outcome
    }
}

impl MailTransport for MxTransport {
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
            // A bounded attempt cannot indefinitely occupy a queue worker while
            // malicious DNS advertises many unresponsive destinations.
            tokio::time::timeout(Duration::from_secs(1800), self.deliver(recipient, message))
                .await
                .unwrap_or_else(|_| wire::temporary())
        })
    }
}
