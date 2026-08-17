//! DHCPv4 ACK parsing and lease-to-operator translation.
//!
//! Mirrors the pure parsing slice of Talos'
//! `internal/app/machined/pkg/controllers/network/operator/internal/dhcpparse/dhcp4.go`.
//! The real Talos operator obtains a DHCP ACK from `insomniacslk/dhcp` and then
//! converts it into address/route/resolver/hostname specs. This module keeps
//! the same pure conversion boundary in Rust: parse enough of a DHCPv4 ACK to
//! model the lease deterministically, then let [`OperatorSpec`] publish the
//! operator-layer output.
//!
//! This is intentionally **not** a wire client. Raw socket discovery/retry and
//! renewal are separate operator work; this module is the source-guided ACK
//! parser used by that future client.

use crate::address::AddressSpec;
use crate::config_layer::ConfigLayer;
use crate::hostname::HostnameSpec;
use crate::link::AddressFamily;
use crate::operator::{OperatorOutput, OperatorSpec};
use crate::resolver::ResolverSpec;
use crate::route::{RouteProtocol, RouteSpec, RouteTable};
use alloc::string::{String, ToString};
use alloc::{vec, vec::Vec};
use core::str;
use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};

const BOOTP_MIN_LEN: usize = 240;
const DHCP_XID_OFFSET: usize = 4;
const DHCP_SECS_OFFSET: usize = 8;
const DHCP_FLAGS_OFFSET: usize = 10;
const DHCP_CIADDR_OFFSET: usize = 12;
const YIADDR_OFFSET: usize = 16;
const DHCP_CHADDR_OFFSET: usize = 28;
const MAGIC_COOKIE_OFFSET: usize = 236;
const DHCP_MAGIC_COOKIE: [u8; 4] = [0x63, 0x82, 0x53, 0x63];
const DHCP_OPCODE_BOOTREQUEST: u8 = 1;
const DHCP_OPCODE_BOOTREPLY: u8 = 2;
const DHCP_HTYPE_ETHERNET: u8 = 1;
const DHCP_HLEN_ETHERNET: u8 = 6;
const DHCP_BROADCAST_FLAG: u16 = 0x8000;
const DHCP_OPTION_PAD: u8 = 0;
const DHCP_OPTION_SUBNET_MASK: u8 = 1;
const DHCP_OPTION_ROUTER: u8 = 3;
const DHCP_OPTION_DNS: u8 = 6;
const DHCP_OPTION_HOSTNAME: u8 = 12;
const DHCP_OPTION_DOMAIN_NAME: u8 = 15;
const DHCP_OPTION_INTERFACE_MTU: u8 = 26;
const DHCP_OPTION_NTP: u8 = 42;
const DHCP_OPTION_REQUESTED_IP: u8 = 50;
const DHCP_OPTION_IP_ADDRESS_LEASE_TIME: u8 = 51;
const DHCP_OPTION_MESSAGE_TYPE: u8 = 53;
const DHCP_OPTION_SERVER_IDENTIFIER: u8 = 54;
const DHCP_OPTION_PARAMETER_REQUEST_LIST: u8 = 55;
const DHCP_OPTION_MAX_MESSAGE_SIZE: u8 = 57;
const DHCP_OPTION_RENEWAL_TIME: u8 = 58;
const DHCP_OPTION_REBINDING_TIME: u8 = 59;
const DHCP_OPTION_CLIENT_IDENTIFIER: u8 = 61;
const DHCP_OPTION_DOMAIN_SEARCH: u8 = 119;
const DHCP_OPTION_CLASSLESS_STATIC_ROUTE: u8 = 121;
const DHCP_OPTION_MS_CLASSLESS_STATIC_ROUTE: u8 = 249;
const DHCP_OPTION_END: u8 = 255;
const DHCP_MESSAGE_DISCOVER: u8 = 1;
const DHCP_MESSAGE_OFFER: u8 = 2;
const DHCP_MESSAGE_REQUEST: u8 = 3;
const DHCP_MESSAGE_ACK: u8 = 5;
const DHCP_MESSAGE_NAK: u8 = 6;
const DHCP_MAX_MESSAGE_SIZE: u16 = 1500;
const DHCP_DEFAULT_INITIAL_TIMEOUT_SECS: u16 = 4;
const DHCP_DEFAULT_MAX_ATTEMPTS: u8 = 4;
const DHCP_DEFAULT_LEASE_TIME_SECS: u32 = 30 * 60;
const DHCP_MIN_RENEW_INTERVAL_SECS: u32 = 5;

/// A classless static route from DHCP option 121 (RFC 3442).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dhcp4ClasslessRoute {
    /// Destination network. `None` with prefix `0` is the default route.
    pub destination: Option<NodeAddress>,
    /// Destination prefix length.
    pub prefix_len: u8,
    /// Next-hop router. `None` models the on-link `0.0.0.0` router value.
    pub router: Option<NodeAddress>,
}

/// Parsed DHCPv4 ACK lease data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dhcp4Lease {
    /// Offered address (`yiaddr`).
    pub address: NodeAddress,
    /// Prefix length derived from the subnet-mask option.
    pub prefix_len: u8,
    /// Router option gateways. Ignored when [`classless_routes`] is non-empty.
    pub routers: Vec<NodeAddress>,
    /// RFC 3442 classless routes. When present, Talos ignores the router option.
    pub classless_routes: Vec<Dhcp4ClasslessRoute>,
    /// DNS servers from option 6.
    pub dns_servers: Vec<NodeAddress>,
    /// NTP servers from option 42.
    pub ntp_servers: Vec<NodeAddress>,
    /// Hostname from option 12, if present and non-empty.
    pub hostname: Option<String>,
    /// Domain name from option 15, if present and non-empty.
    pub domain_name: Option<String>,
    /// Domain search list from option 119 plus a de-duplicated domain-name.
    pub search_domains: Vec<String>,
    /// Interface MTU from option 26.
    pub mtu: Option<u16>,
    /// Lease lifetime from option 51, or Talos' 30-minute default when absent.
    pub lease_time_secs: u32,
    /// Optional renewal (T1) timer from option 58.
    pub renewal_time_secs: Option<u32>,
    /// Optional rebinding (T2) timer from option 59.
    pub rebinding_time_secs: Option<u32>,
}

impl Dhcp4Lease {
    fn subnet_contains(&self, addr: NodeAddress) -> bool {
        let (NodeAddress::V4(base), NodeAddress::V4(candidate)) = (self.address, addr) else {
            return false;
        };
        prefix_contains(base, self.prefix_len, candidate)
    }
}

/// DHCPv4 client identifier policy for outbound requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dhcp4ClientIdentifier {
    /// Do not send DHCP option 61.
    None,
    /// Send RFC 2132 Ethernet client identifier: hardware type `1` + MAC.
    Mac,
    /// Send a caller-provided option 61 payload, used by DUID-style configs.
    Raw(Vec<u8>),
}

/// Stable inputs for building DHCPv4 wire-client packets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dhcp4ClientConfig {
    /// DHCP transaction id (`xid`).
    pub transaction_id: u32,
    /// Ethernet client hardware address.
    pub client_hardware_addr: [u8; 6],
    /// Seconds since the current DHCP acquisition/renewal started.
    pub secs: u16,
    /// Whether to request option 26 (interface MTU). Talos disables this for Azure.
    pub request_mtu: bool,
    /// Whether to request DHCP-provided hostname/domain options when not sending
    /// an explicit hostname.
    pub request_hostname: bool,
    /// Hostname to send in option 12. When present, hostname parroting
    /// protection stops requesting option 12 in the parameter request list.
    pub hostname: Option<String>,
    /// Domain name to send in option 15.
    pub domain_name: Option<String>,
    /// Client identifier behavior for option 61.
    pub client_identifier: Dhcp4ClientIdentifier,
}

impl Dhcp4ClientConfig {
    /// Build a config with Talos-like defaults for a new DHCPv4 exchange.
    pub fn new(transaction_id: u32, client_hardware_addr: [u8; 6]) -> Self {
        Dhcp4ClientConfig {
            transaction_id,
            client_hardware_addr,
            secs: 0,
            request_mtu: true,
            request_hostname: true,
            hostname: None,
            domain_name: None,
            client_identifier: Dhcp4ClientIdentifier::Mac,
        }
    }

    /// Return a copy with a different `secs` field.
    pub fn with_secs(mut self, secs: u16) -> Self {
        self.secs = secs;
        self
    }

    /// Return a copy that sends hostname/domain to the DHCP server.
    pub fn with_hostname(
        mut self,
        hostname: impl Into<String>,
        domain_name: Option<impl Into<String>>,
    ) -> Self {
        self.hostname = Some(hostname.into());
        self.domain_name = domain_name.map(Into::into);
        self
    }

    /// Return a copy that does not request interface MTU.
    pub fn without_mtu_request(mut self) -> Self {
        self.request_mtu = false;
        self
    }

    /// Return a copy that suppresses DHCP hostname/domain requests.
    pub fn without_hostname_request(mut self) -> Self {
        self.request_hostname = false;
        self
    }

    /// Return a copy that suppresses option 61.
    pub fn without_client_identifier(mut self) -> Self {
        self.client_identifier = Dhcp4ClientIdentifier::None;
        self
    }
}

/// A network send target selected by the DHCPv4 state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dhcp4SendTarget {
    /// Broadcast to all DHCP servers (`255.255.255.255:67`).
    Broadcast,
    /// Unicast to a known DHCP server, used for renewals.
    Unicast(NodeAddress),
}

/// An outbound DHCPv4 packet and its intended target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dhcp4Outbound {
    /// Packet bytes suitable for UDP payload transmission.
    pub packet: Vec<u8>,
    /// Destination choice for the socket layer.
    pub target: Dhcp4SendTarget,
}

/// Parsed DHCPv4 OFFER metadata needed to build the SELECTING-state REQUEST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dhcp4Offer {
    /// Transaction id from the OFFER.
    pub transaction_id: u32,
    /// Client hardware address echoed by the server.
    pub client_hardware_addr: [u8; 6],
    /// Offered IPv4 address (`yiaddr`).
    pub address: NodeAddress,
    /// DHCP server identifier (option 54).
    pub server_identifier: NodeAddress,
}

/// Parsed DHCPv4 response for the REQUESTING state.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum Dhcp4RequestResponse {
    /// Server accepted the request and returned an ACK lease.
    Ack(Dhcp4Lease),
    /// Server rejected the request with DHCPNAK.
    Nak,
}

/// Pure DHCPv4 wire-client state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dhcp4WireState {
    /// No packet sent yet.
    Init,
    /// DISCOVER has been sent, waiting for OFFER.
    Selecting,
    /// REQUEST has been sent, waiting for ACK/NAK.
    Requesting,
    /// ACK has been accepted.
    Bound,
}

/// Result of feeding a packet or timer event to [`Dhcp4WireTransaction`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dhcp4WireAction {
    /// Send this packet next.
    Send(Dhcp4Outbound),
    /// A lease has been accepted.
    Bound(Dhcp4Lease),
    /// Packet was well-formed enough to classify but irrelevant to this state.
    Ignored(&'static str),
}

/// Deterministic DHCPv4 DISCOVER/OFFER/REQUEST/ACK state machine.
///
/// The type performs no socket I/O and no sleeping. PID1 or the operator runtime
/// owns the actual UDP/raw socket boundary; this struct owns Talos-compatible
/// packet construction, transaction matching, retry packet generation, and ACK
/// parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dhcp4WireTransaction {
    config: Dhcp4ClientConfig,
    state: Dhcp4WireState,
    selected_offer: Option<Dhcp4Offer>,
    attempts: u8,
    max_attempts: u8,
    initial_timeout_secs: u16,
}

impl Dhcp4WireTransaction {
    /// Create a new transaction with source-compatible retry defaults.
    pub fn new(config: Dhcp4ClientConfig) -> Self {
        Dhcp4WireTransaction {
            config,
            state: Dhcp4WireState::Init,
            selected_offer: None,
            attempts: 0,
            max_attempts: DHCP_DEFAULT_MAX_ATTEMPTS,
            initial_timeout_secs: DHCP_DEFAULT_INITIAL_TIMEOUT_SECS,
        }
    }

    /// Configure the retry schedule.
    pub fn with_retry_schedule(mut self, initial_timeout_secs: u16, max_attempts: u8) -> Self {
        self.initial_timeout_secs = initial_timeout_secs.max(1);
        self.max_attempts = max_attempts.max(1);
        self
    }

    /// Current state.
    pub fn state(&self) -> Dhcp4WireState {
        self.state
    }

    /// Number of packets sent in the current state.
    pub fn attempts(&self) -> u8 {
        self.attempts
    }

    /// Start the transaction by emitting the first DISCOVER.
    pub fn start(&mut self) -> Result<Dhcp4Outbound> {
        self.state = Dhcp4WireState::Selecting;
        self.selected_offer = None;
        self.attempts = 1;
        self.discover_for_attempt(0)
    }

    /// Generate a retry packet after the current timeout expires.
    pub fn handle_timeout(&mut self) -> Result<Dhcp4Outbound> {
        if self.state == Dhcp4WireState::Init {
            return self.start();
        }
        if self.state == Dhcp4WireState::Bound {
            return Err(Error::invalid_state("DHCPv4 transaction is already bound"));
        }
        if self.attempts >= self.max_attempts {
            return Err(Error::Timeout);
        }

        let attempt_index = self.attempts;
        self.attempts += 1;
        match self.state {
            Dhcp4WireState::Selecting => self.discover_for_attempt(attempt_index),
            Dhcp4WireState::Requesting => {
                let offer = self
                    .selected_offer
                    .clone()
                    .ok_or_else(|| Error::invalid_state("DHCPv4 request has no selected offer"))?;
                self.request_for_offer_attempt(&offer, attempt_index)
            }
            Dhcp4WireState::Init | Dhcp4WireState::Bound => unreachable!(),
        }
    }

    /// Feed one inbound DHCP packet into the state machine.
    pub fn handle_packet(&mut self, packet: &[u8]) -> Result<Dhcp4WireAction> {
        match self.state {
            Dhcp4WireState::Init => Ok(Dhcp4WireAction::Ignored("not started")),
            Dhcp4WireState::Selecting => {
                let Some(offer) = parse_dhcp4_offer(
                    packet,
                    self.config.transaction_id,
                    self.config.client_hardware_addr,
                )?
                else {
                    return Ok(Dhcp4WireAction::Ignored("not matching offer"));
                };
                self.state = Dhcp4WireState::Requesting;
                self.selected_offer = Some(offer.clone());
                self.attempts = 1;
                Ok(Dhcp4WireAction::Send(
                    self.request_for_offer_attempt(&offer, 0)?,
                ))
            }
            Dhcp4WireState::Requesting => {
                let offer = self
                    .selected_offer
                    .clone()
                    .ok_or_else(|| Error::invalid_state("DHCPv4 request has no selected offer"))?;
                let Some(response) = parse_dhcp4_request_response(
                    packet,
                    self.config.transaction_id,
                    self.config.client_hardware_addr,
                    offer.server_identifier,
                )?
                else {
                    return Ok(Dhcp4WireAction::Ignored("not matching ack/nak"));
                };
                match response {
                    Dhcp4RequestResponse::Ack(lease) => {
                        self.state = Dhcp4WireState::Bound;
                        Ok(Dhcp4WireAction::Bound(lease))
                    }
                    Dhcp4RequestResponse::Nak => {
                        self.state = Dhcp4WireState::Selecting;
                        self.selected_offer = None;
                        self.attempts = 1;
                        Ok(Dhcp4WireAction::Send(self.discover_for_attempt(0)?))
                    }
                }
            }
            Dhcp4WireState::Bound => Ok(Dhcp4WireAction::Ignored("already bound")),
        }
    }

    fn config_for_attempt(&self, attempt_index: u8) -> Dhcp4ClientConfig {
        self.config
            .clone()
            .with_secs(dhcp4_elapsed_secs_for_attempt(
                self.initial_timeout_secs,
                attempt_index,
            ))
    }

    fn discover_for_attempt(&self, attempt_index: u8) -> Result<Dhcp4Outbound> {
        Ok(Dhcp4Outbound {
            packet: build_dhcp4_discover(&self.config_for_attempt(attempt_index))?,
            target: Dhcp4SendTarget::Broadcast,
        })
    }

    fn request_for_offer_attempt(
        &self,
        offer: &Dhcp4Offer,
        attempt_index: u8,
    ) -> Result<Dhcp4Outbound> {
        Ok(Dhcp4Outbound {
            packet: build_dhcp4_request_from_offer(&self.config_for_attempt(attempt_index), offer)?,
            target: Dhcp4SendTarget::Broadcast,
        })
    }
}

/// Cumulative elapsed seconds to encode in BOOTP `secs` for a retry attempt.
pub fn dhcp4_elapsed_secs_for_attempt(initial_timeout_secs: u16, attempt_index: u8) -> u16 {
    let mut elapsed = 0u32;
    let mut timeout = u32::from(initial_timeout_secs.max(1));
    for _ in 0..attempt_index {
        elapsed = elapsed.saturating_add(timeout);
        timeout = timeout.saturating_mul(2);
    }
    elapsed.min(u32::from(u16::MAX)) as u16
}

/// Talos' next successful renewal delay for a DHCPv4 ACK lease lifetime.
///
/// The live Go operator asks the DHCP library for option 51 with a 30-minute
/// fallback, sleeps for half that lifetime after successful requests/renews,
/// and clamps very small leases to at least five seconds.
pub fn dhcp4_renew_interval_secs(lease_time_secs: u32) -> u32 {
    (lease_time_secs / 2).max(DHCP_MIN_RENEW_INTERVAL_SECS)
}

/// Talos' retry delay after a failed request/renew attempt.
///
/// Failures halve the previous interval and use the same five-second lower
/// bound as successful lease renewals.
pub fn dhcp4_failed_renew_interval_secs(previous_interval_secs: u32) -> u32 {
    (previous_interval_secs / 2).max(DHCP_MIN_RENEW_INTERVAL_SECS)
}

/// Build a DHCPDISCOVER packet.
pub fn build_dhcp4_discover(config: &Dhcp4ClientConfig) -> Result<Vec<u8>> {
    let mut packet = build_boot_request(config, DHCP_BROADCAST_FLAG, None)?;
    append_common_request_options(&mut packet, config, DHCP_MESSAGE_DISCOVER)?;
    packet.push(DHCP_OPTION_END);
    Ok(packet)
}

/// Build a SELECTING-state DHCPREQUEST from an OFFER.
pub fn build_dhcp4_request_from_offer(
    config: &Dhcp4ClientConfig,
    offer: &Dhcp4Offer,
) -> Result<Vec<u8>> {
    if offer.transaction_id != config.transaction_id {
        return Err(Error::invalid("DHCPv4 offer xid does not match config"));
    }
    if offer.client_hardware_addr != config.client_hardware_addr {
        return Err(Error::invalid("DHCPv4 offer chaddr does not match config"));
    }

    let mut packet = build_boot_request(config, DHCP_BROADCAST_FLAG, None)?;
    append_common_request_options(&mut packet, config, DHCP_MESSAGE_REQUEST)?;
    append_option(
        &mut packet,
        DHCP_OPTION_REQUESTED_IP,
        &v4_octets(offer.address, "offered address")?,
    )?;
    append_option(
        &mut packet,
        DHCP_OPTION_SERVER_IDENTIFIER,
        &v4_octets(offer.server_identifier, "server identifier")?,
    )?;
    packet.push(DHCP_OPTION_END);
    Ok(packet)
}

/// Build an INIT-REBOOT DHCPREQUEST for a previously assigned address.
pub fn build_dhcp4_reboot_request(
    config: &Dhcp4ClientConfig,
    previous_ip: NodeAddress,
) -> Result<Vec<u8>> {
    let mut packet = build_boot_request(config, DHCP_BROADCAST_FLAG, None)?;
    append_common_request_options(&mut packet, config, DHCP_MESSAGE_REQUEST)?;
    append_option(
        &mut packet,
        DHCP_OPTION_REQUESTED_IP,
        &v4_octets(previous_ip, "previous address")?,
    )?;
    packet.push(DHCP_OPTION_END);
    Ok(packet)
}

/// Build a RENEWING-state unicast DHCPREQUEST from an active lease.
pub fn build_dhcp4_renew_request(
    config: &Dhcp4ClientConfig,
    client_ip: NodeAddress,
    server_identifier: NodeAddress,
) -> Result<Dhcp4Outbound> {
    let mut packet = build_boot_request(config, 0, Some(client_ip))?;
    append_common_request_options(&mut packet, config, DHCP_MESSAGE_REQUEST)?;
    packet.push(DHCP_OPTION_END);
    Ok(Dhcp4Outbound {
        packet,
        target: Dhcp4SendTarget::Unicast(server_identifier),
    })
}

/// Parse a matching DHCPOFFER. Non-matching packets return `Ok(None)`.
pub fn parse_dhcp4_offer(
    packet: &[u8],
    expected_xid: u32,
    expected_chaddr: [u8; 6],
) -> Result<Option<Dhcp4Offer>> {
    if !reply_matches(packet, expected_xid, expected_chaddr)? {
        return Ok(None);
    }
    let options = parse_options(&packet[BOOTP_MIN_LEN..])?;
    let message_type = dhcp_message_type(&options)?;
    if message_type != DHCP_MESSAGE_OFFER {
        return Ok(None);
    }
    let address = read_v4(&packet[YIADDR_OFFSET..YIADDR_OFFSET + 4])?;
    if address.is_unspecified() {
        return Err(Error::parse("DHCPv4 OFFER has unspecified yiaddr"));
    }
    let server_identifier = parse_server_identifier(&options)?;
    Ok(Some(Dhcp4Offer {
        transaction_id: expected_xid,
        client_hardware_addr: expected_chaddr,
        address,
        server_identifier,
    }))
}

/// Parse a matching ACK/NAK for the REQUESTING state.
pub fn parse_dhcp4_request_response(
    packet: &[u8],
    expected_xid: u32,
    expected_chaddr: [u8; 6],
    expected_server_identifier: NodeAddress,
) -> Result<Option<Dhcp4RequestResponse>> {
    if !reply_matches(packet, expected_xid, expected_chaddr)? {
        return Ok(None);
    }
    let options = parse_options(&packet[BOOTP_MIN_LEN..])?;
    let message_type = dhcp_message_type(&options)?;
    if message_type != DHCP_MESSAGE_ACK && message_type != DHCP_MESSAGE_NAK {
        return Ok(None);
    }
    let server_identifier = parse_server_identifier(&options)?;
    if server_identifier != expected_server_identifier {
        return Ok(None);
    }
    if message_type == DHCP_MESSAGE_NAK {
        return Ok(Some(Dhcp4RequestResponse::Nak));
    }
    Ok(Some(Dhcp4RequestResponse::Ack(parse_dhcp4_ack(packet)?)))
}

/// Parse a DHCPv4 ACK packet into a deterministic lease model.
pub fn parse_dhcp4_ack(packet: &[u8]) -> Result<Dhcp4Lease> {
    if packet.len() < BOOTP_MIN_LEN {
        return Err(Error::parse("DHCPv4 packet shorter than BOOTP header"));
    }
    if packet[MAGIC_COOKIE_OFFSET..MAGIC_COOKIE_OFFSET + 4] != DHCP_MAGIC_COOKIE {
        return Err(Error::parse("DHCPv4 packet missing magic cookie"));
    }

    let options = parse_options(&packet[BOOTP_MIN_LEN..])?;
    let message_type = option(&options, DHCP_OPTION_MESSAGE_TYPE)
        .and_then(|data| data.first().copied())
        .ok_or_else(|| Error::parse("DHCPv4 packet missing message type"))?;
    if message_type != DHCP_MESSAGE_ACK {
        return Err(Error::invalid("DHCPv4 packet is not an ACK"));
    }

    let yiaddr = read_v4(&packet[YIADDR_OFFSET..YIADDR_OFFSET + 4])?;
    if yiaddr.is_unspecified() {
        return Err(Error::parse("DHCPv4 ACK has unspecified yiaddr"));
    }

    let mask = option(&options, DHCP_OPTION_SUBNET_MASK)
        .ok_or_else(|| Error::parse("DHCPv4 ACK missing subnet mask"))?;
    if mask.len() != 4 {
        return Err(Error::parse("DHCPv4 subnet mask option must be 4 bytes"));
    }
    let prefix_len = mask_to_prefix([mask[0], mask[1], mask[2], mask[3]])?;

    let routers = parse_v4_list(
        option(&options, DHCP_OPTION_ROUTER).unwrap_or(&[]),
        "router",
    )?;
    let dns_servers = parse_v4_list(option(&options, DHCP_OPTION_DNS).unwrap_or(&[]), "dns")?;
    let ntp_servers = parse_v4_list(option(&options, DHCP_OPTION_NTP).unwrap_or(&[]), "ntp")?;

    let classless_data = option(&options, DHCP_OPTION_CLASSLESS_STATIC_ROUTE)
        .or_else(|| option(&options, DHCP_OPTION_MS_CLASSLESS_STATIC_ROUTE));
    let classless_routes = match classless_data {
        Some(data) => parse_classless_routes(data)?,
        None => Vec::new(),
    };

    let hostname = option(&options, DHCP_OPTION_HOSTNAME)
        .map(parse_text_option)
        .transpose()?
        .filter(|s| !s.is_empty());
    let domain_name = option(&options, DHCP_OPTION_DOMAIN_NAME)
        .map(parse_text_option)
        .transpose()?
        .filter(|s| !s.is_empty());

    let mut search_domains = match option(&options, DHCP_OPTION_DOMAIN_SEARCH) {
        Some(data) => parse_domain_search(data)?,
        None => Vec::new(),
    };
    if let Some(domain) = &domain_name
        && !search_domains.contains(domain)
    {
        search_domains.push(domain.clone());
    }

    let mtu = match option(&options, DHCP_OPTION_INTERFACE_MTU) {
        Some([hi, lo]) => Some(u16::from_be_bytes([*hi, *lo])),
        Some(_) => return Err(Error::parse("DHCPv4 interface MTU option must be 2 bytes")),
        None => None,
    };
    let lease_time_secs = option_u32(&options, DHCP_OPTION_IP_ADDRESS_LEASE_TIME, "lease time")?
        .unwrap_or(DHCP_DEFAULT_LEASE_TIME_SECS);
    let renewal_time_secs = option_u32(&options, DHCP_OPTION_RENEWAL_TIME, "renewal time")?;
    let rebinding_time_secs = option_u32(&options, DHCP_OPTION_REBINDING_TIME, "rebinding time")?;

    Ok(Dhcp4Lease {
        address: yiaddr,
        prefix_len,
        routers,
        classless_routes,
        dns_servers,
        ntp_servers,
        hostname,
        domain_name,
        search_domains,
        mtu,
        lease_time_secs,
        renewal_time_secs,
        rebinding_time_secs,
    })
}

impl OperatorSpec {
    /// Translate a DHCPv4 lease into operator-layer specs.
    ///
    /// Talos treats classless static routes as authoritative over the router
    /// option. For off-subnet gateways, it also emits an on-link helper route so
    /// the kernel can install the gateway route; this is required for AWS
    /// IPv6-only metadata leases where a `/32` link-local lease routes IMDS via
    /// `169.254.0.1`.
    pub fn apply_dhcp4_lease(
        &self,
        lease: &Dhcp4Lease,
        use_hostname: bool,
    ) -> Result<OperatorOutput> {
        let mut out = OperatorOutput::default();

        out.addresses.push(AddressSpec::new(
            lease.address,
            lease.prefix_len,
            self.link_name.clone(),
            ConfigLayer::Operator,
        )?);

        if !lease.classless_routes.is_empty() {
            let mut helper_gateways = Vec::new();
            for route in &lease.classless_routes {
                out.routes.push(self.dhcp4_route(
                    route.destination,
                    route.prefix_len,
                    route.router,
                )?);

                if let Some(gateway) = route.router
                    && !gateway.is_unspecified()
                    && !lease.subnet_contains(gateway)
                    && !helper_gateways.contains(&gateway)
                {
                    helper_gateways.push(gateway);
                    out.routes.push(self.dhcp4_route(Some(gateway), 32, None)?);
                }
            }
        } else {
            for &gateway in &lease.routers {
                out.routes.push(self.dhcp4_route(None, 0, Some(gateway))?);
                if !lease.subnet_contains(gateway) {
                    out.routes.push(self.dhcp4_route(Some(gateway), 32, None)?);
                }
            }
        }

        if use_hostname && let Some(hostname) = &lease.hostname {
            let spec = match hostname.split_once('.') {
                Some((host, domain)) => {
                    HostnameSpec::with_domain(host, domain, ConfigLayer::Operator)?
                }
                None => match &lease.domain_name {
                    Some(domain) => HostnameSpec::with_domain(
                        hostname.clone(),
                        domain.clone(),
                        ConfigLayer::Operator,
                    )?,
                    None => HostnameSpec::new(hostname.clone(), ConfigLayer::Operator)?,
                },
            };
            out.hostname = Some(spec);
        }

        if !lease.dns_servers.is_empty() || !lease.search_domains.is_empty() {
            out.resolver = Some(ResolverSpec::new_with_search(
                lease.dns_servers.clone(),
                lease.search_domains.clone(),
                ConfigLayer::Operator,
            )?);
        }

        out.time_servers = lease.ntp_servers.iter().map(ToString::to_string).collect();

        Ok(out)
    }

    fn dhcp4_route(
        &self,
        destination: Option<NodeAddress>,
        prefix_len: u8,
        gateway: Option<NodeAddress>,
    ) -> Result<RouteSpec> {
        let route = RouteSpec {
            destination: if prefix_len == 0 { None } else { destination },
            prefix_len,
            source: None,
            gateway: gateway.filter(|gw| !gw.is_unspecified()),
            out_link: self.link_name.clone(),
            family: AddressFamily::Inet4,
            metric: self.route_metric,
            mtu: 0,
            table: RouteTable::Main,
            // Match Talos' DHCP ACK parser, which publishes DHCP-installed
            // routes with ProtocolBoot rather than ProtocolDHCP.
            protocol: RouteProtocol::Boot,
            layer: ConfigLayer::Operator,
        };
        route.validate()?;
        Ok(route)
    }
}

fn build_boot_request(
    config: &Dhcp4ClientConfig,
    flags: u16,
    ciaddr: Option<NodeAddress>,
) -> Result<Vec<u8>> {
    let mut packet = vec![0u8; BOOTP_MIN_LEN];
    packet[0] = DHCP_OPCODE_BOOTREQUEST;
    packet[1] = DHCP_HTYPE_ETHERNET;
    packet[2] = DHCP_HLEN_ETHERNET;
    packet[DHCP_XID_OFFSET..DHCP_XID_OFFSET + 4]
        .copy_from_slice(&config.transaction_id.to_be_bytes());
    packet[DHCP_SECS_OFFSET..DHCP_SECS_OFFSET + 2].copy_from_slice(&config.secs.to_be_bytes());
    packet[DHCP_FLAGS_OFFSET..DHCP_FLAGS_OFFSET + 2].copy_from_slice(&flags.to_be_bytes());
    if let Some(ciaddr) = ciaddr {
        packet[DHCP_CIADDR_OFFSET..DHCP_CIADDR_OFFSET + 4]
            .copy_from_slice(&v4_octets(ciaddr, "client address")?);
    }
    packet[DHCP_CHADDR_OFFSET..DHCP_CHADDR_OFFSET + 6]
        .copy_from_slice(&config.client_hardware_addr);
    packet[MAGIC_COOKIE_OFFSET..MAGIC_COOKIE_OFFSET + 4].copy_from_slice(&DHCP_MAGIC_COOKIE);
    Ok(packet)
}

fn append_common_request_options(
    packet: &mut Vec<u8>,
    config: &Dhcp4ClientConfig,
    message_type: u8,
) -> Result<()> {
    append_option(packet, DHCP_OPTION_MESSAGE_TYPE, &[message_type])?;
    append_option(
        packet,
        DHCP_OPTION_MAX_MESSAGE_SIZE,
        &DHCP_MAX_MESSAGE_SIZE.to_be_bytes(),
    )?;
    append_option(
        packet,
        DHCP_OPTION_PARAMETER_REQUEST_LIST,
        &parameter_request_list(config),
    )?;

    match &config.client_identifier {
        Dhcp4ClientIdentifier::None => {}
        Dhcp4ClientIdentifier::Mac => {
            let mut identifier = Vec::with_capacity(7);
            identifier.push(DHCP_HTYPE_ETHERNET);
            identifier.extend_from_slice(&config.client_hardware_addr);
            append_option(packet, DHCP_OPTION_CLIENT_IDENTIFIER, &identifier)?;
        }
        Dhcp4ClientIdentifier::Raw(raw) => {
            append_option(packet, DHCP_OPTION_CLIENT_IDENTIFIER, raw)?;
        }
    }

    if let Some(hostname) = &config.hostname {
        append_option(packet, DHCP_OPTION_HOSTNAME, hostname.as_bytes())?;
    }
    if let Some(domain_name) = &config.domain_name {
        append_option(packet, DHCP_OPTION_DOMAIN_NAME, domain_name.as_bytes())?;
    }

    Ok(())
}

fn parameter_request_list(config: &Dhcp4ClientConfig) -> Vec<u8> {
    // Match the insomniacslk DHCPv4 defaults that Talos builds on, then append
    // Talos-specific operator requests in source order, de-duping like
    // `OptionCodeList.Add`.
    let mut requested = vec![
        DHCP_OPTION_SUBNET_MASK,
        DHCP_OPTION_ROUTER,
        DHCP_OPTION_DOMAIN_NAME,
        DHCP_OPTION_DNS,
    ];
    add_unique(&mut requested, DHCP_OPTION_CLASSLESS_STATIC_ROUTE);
    add_unique(&mut requested, DHCP_OPTION_DNS);
    add_unique(&mut requested, DHCP_OPTION_DOMAIN_SEARCH);
    add_unique(&mut requested, DHCP_OPTION_NTP);
    if config.request_mtu {
        add_unique(&mut requested, DHCP_OPTION_INTERFACE_MTU);
    }
    if should_request_hostname(config) {
        add_unique(&mut requested, DHCP_OPTION_HOSTNAME);
        add_unique(&mut requested, DHCP_OPTION_DOMAIN_NAME);
    }
    requested
}

fn should_request_hostname(config: &Dhcp4ClientConfig) -> bool {
    config.request_hostname && config.hostname.is_none() && config.domain_name.is_none()
}

fn add_unique(values: &mut Vec<u8>, value: u8) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn append_option(packet: &mut Vec<u8>, code: u8, data: &[u8]) -> Result<()> {
    if data.len() > u8::MAX as usize {
        return Err(Error::invalid(alloc::format!(
            "DHCPv4 option {code} exceeds 255 bytes"
        )));
    }
    packet.push(code);
    packet.push(data.len() as u8);
    packet.extend_from_slice(data);
    Ok(())
}

fn reply_matches(packet: &[u8], expected_xid: u32, expected_chaddr: [u8; 6]) -> Result<bool> {
    if packet.len() < BOOTP_MIN_LEN {
        return Err(Error::parse("DHCPv4 packet shorter than BOOTP header"));
    }
    if packet[0] != DHCP_OPCODE_BOOTREPLY {
        return Ok(false);
    }
    if packet[MAGIC_COOKIE_OFFSET..MAGIC_COOKIE_OFFSET + 4] != DHCP_MAGIC_COOKIE {
        return Err(Error::parse("DHCPv4 packet missing magic cookie"));
    }
    let xid = u32::from_be_bytes([
        packet[DHCP_XID_OFFSET],
        packet[DHCP_XID_OFFSET + 1],
        packet[DHCP_XID_OFFSET + 2],
        packet[DHCP_XID_OFFSET + 3],
    ]);
    if xid != expected_xid {
        return Ok(false);
    }
    if packet[DHCP_CHADDR_OFFSET..DHCP_CHADDR_OFFSET + 6] != expected_chaddr {
        return Ok(false);
    }
    Ok(true)
}

fn dhcp_message_type(options: &[ParsedOption<'_>]) -> Result<u8> {
    option(options, DHCP_OPTION_MESSAGE_TYPE)
        .and_then(|data| data.first().copied())
        .ok_or_else(|| Error::parse("DHCPv4 packet missing message type"))
}

fn parse_server_identifier(options: &[ParsedOption<'_>]) -> Result<NodeAddress> {
    let data = option(options, DHCP_OPTION_SERVER_IDENTIFIER)
        .ok_or_else(|| Error::parse("DHCPv4 packet missing server identifier"))?;
    if data.len() != 4 {
        return Err(Error::parse(
            "DHCPv4 server identifier option must be 4 bytes",
        ));
    }
    read_v4(data)
}

fn v4_octets(addr: NodeAddress, what: &str) -> Result<[u8; 4]> {
    match addr {
        NodeAddress::V4(octets) => Ok(octets),
        NodeAddress::V6(_) => Err(Error::invalid(alloc::format!(
            "DHCPv4 {what} must be an IPv4 address"
        ))),
    }
}

#[derive(Debug, Clone, Copy)]
struct ParsedOption<'a> {
    code: u8,
    data: &'a [u8],
}

fn parse_options(data: &[u8]) -> Result<Vec<ParsedOption<'_>>> {
    let mut options = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        let code = data[pos];
        pos += 1;
        match code {
            DHCP_OPTION_PAD => {}
            DHCP_OPTION_END => return Ok(options),
            _ => {
                if pos >= data.len() {
                    return Err(Error::parse("DHCPv4 option missing length"));
                }
                let len = data[pos] as usize;
                pos += 1;
                if pos + len > data.len() {
                    return Err(Error::parse("DHCPv4 option length exceeds packet"));
                }
                options.push(ParsedOption {
                    code,
                    data: &data[pos..pos + len],
                });
                pos += len;
            }
        }
    }
    Err(Error::parse("DHCPv4 options missing end marker"))
}

fn option<'a>(options: &'a [ParsedOption<'a>], code: u8) -> Option<&'a [u8]> {
    options
        .iter()
        .find(|opt| opt.code == code)
        .map(|opt| opt.data)
}

fn option_u32(options: &[ParsedOption<'_>], code: u8, name: &str) -> Result<Option<u32>> {
    match option(options, code) {
        Some([a, b, c, d]) => Ok(Some(u32::from_be_bytes([*a, *b, *c, *d]))),
        Some(_) => Err(Error::parse(alloc::format!(
            "DHCPv4 {name} option must be 4 bytes"
        ))),
        None => Ok(None),
    }
}

fn read_v4(data: &[u8]) -> Result<NodeAddress> {
    if data.len() != 4 {
        return Err(Error::parse("IPv4 option must be 4 bytes"));
    }
    Ok(NodeAddress::V4([data[0], data[1], data[2], data[3]]))
}

fn parse_v4_list(data: &[u8], name: &str) -> Result<Vec<NodeAddress>> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    if !data.len().is_multiple_of(4) {
        return Err(Error::parse(alloc::format!(
            "DHCPv4 {name} option length is not a multiple of 4"
        )));
    }
    data.chunks_exact(4).map(read_v4).collect()
}

fn mask_to_prefix(mask: [u8; 4]) -> Result<u8> {
    let bits = u32::from_be_bytes(mask);
    let mut prefix = 0u8;
    let mut seen_zero = false;
    for bit in 0..32 {
        let is_one = (bits & (1 << (31 - bit))) != 0;
        if is_one {
            if seen_zero {
                return Err(Error::parse("DHCPv4 subnet mask is not contiguous"));
            }
            prefix += 1;
        } else {
            seen_zero = true;
        }
    }
    Ok(prefix)
}

fn parse_text_option(data: &[u8]) -> Result<String> {
    let text = str::from_utf8(data)
        .map_err(|_| Error::parse("DHCPv4 text option is not valid UTF-8"))?
        .trim_end_matches('\0')
        .to_string();
    Ok(text)
}

fn parse_classless_routes(data: &[u8]) -> Result<Vec<Dhcp4ClasslessRoute>> {
    let mut routes = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        let prefix_len = data[pos];
        pos += 1;
        if prefix_len > 32 {
            return Err(Error::parse("DHCPv4 classless route prefix exceeds 32"));
        }
        let dest_octets = usize::from(prefix_len.div_ceil(8));
        if pos + dest_octets + 4 > data.len() {
            return Err(Error::parse("DHCPv4 classless route option is truncated"));
        }
        let mut destination = [0u8; 4];
        destination[..dest_octets].copy_from_slice(&data[pos..pos + dest_octets]);
        pos += dest_octets;
        let router = [data[pos], data[pos + 1], data[pos + 2], data[pos + 3]];
        pos += 4;

        routes.push(Dhcp4ClasslessRoute {
            destination: if prefix_len == 0 {
                None
            } else {
                Some(NodeAddress::V4(destination))
            },
            prefix_len,
            router: if router == [0, 0, 0, 0] {
                None
            } else {
                Some(NodeAddress::V4(router))
            },
        });
    }
    Ok(routes)
}

fn parse_domain_search(data: &[u8]) -> Result<Vec<String>> {
    let mut domains = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        if data[pos] == 0 {
            pos += 1;
            continue;
        }
        let (domain, next) = decode_domain_name(data, pos)?;
        if !domain.is_empty() && !domains.contains(&domain) {
            domains.push(domain);
        }
        pos = next;
    }
    Ok(domains)
}

fn decode_domain_name(data: &[u8], mut pos: usize) -> Result<(String, usize)> {
    let mut labels = Vec::new();
    let mut jumped = false;
    let mut next = pos;
    let mut guard = 0usize;

    loop {
        if pos >= data.len() {
            return Err(Error::parse("DHCPv4 domain-search label exceeds option"));
        }
        guard += 1;
        if guard > data.len() {
            return Err(Error::parse("DHCPv4 domain-search compression loop"));
        }

        let len = data[pos];
        if len == 0 {
            if !jumped {
                next = pos + 1;
            }
            break;
        }
        if (len & 0xc0) == 0xc0 {
            if pos + 1 >= data.len() {
                return Err(Error::parse("DHCPv4 domain-search pointer truncated"));
            }
            let ptr = (usize::from(len & 0x3f) << 8) | usize::from(data[pos + 1]);
            if ptr >= data.len() {
                return Err(Error::parse("DHCPv4 domain-search pointer out of range"));
            }
            if !jumped {
                next = pos + 2;
            }
            jumped = true;
            pos = ptr;
            continue;
        }
        if (len & 0xc0) != 0 {
            return Err(Error::parse(
                "DHCPv4 domain-search unsupported label encoding",
            ));
        }

        let label_len = usize::from(len);
        pos += 1;
        if label_len == 0 || pos + label_len > data.len() {
            return Err(Error::parse("DHCPv4 domain-search label truncated"));
        }
        let label = str::from_utf8(&data[pos..pos + label_len])
            .map_err(|_| Error::parse("DHCPv4 domain-search label is not UTF-8"))?;
        labels.push(label.to_string());
        pos += label_len;
        if !jumped {
            next = pos;
        }
    }

    Ok((labels.join("."), next))
}

fn prefix_contains(network: [u8; 4], prefix_len: u8, candidate: [u8; 4]) -> bool {
    if prefix_len == 0 {
        return true;
    }
    let network = u32::from_be_bytes(network);
    let candidate = u32::from_be_bytes(candidate);
    let mask = u32::MAX << (32 - u32::from(prefix_len));
    (network & mask) == (candidate & mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v4(s: &str) -> NodeAddress {
        NodeAddress::parse_v4(s).unwrap()
    }

    fn ack(yiaddr: [u8; 4], mut opts: Vec<(u8, Vec<u8>)>) -> Vec<u8> {
        let mut packet = vec![0u8; BOOTP_MIN_LEN];
        packet[0] = 2; // BOOTREPLY
        packet[YIADDR_OFFSET..YIADDR_OFFSET + 4].copy_from_slice(&yiaddr);
        packet[MAGIC_COOKIE_OFFSET..MAGIC_COOKIE_OFFSET + 4].copy_from_slice(&DHCP_MAGIC_COOKIE);
        opts.insert(0, (DHCP_OPTION_MESSAGE_TYPE, vec![DHCP_MESSAGE_ACK]));
        for (code, data) in opts {
            packet.push(code);
            packet.push(data.len() as u8);
            packet.extend_from_slice(&data);
        }
        packet.push(DHCP_OPTION_END);
        packet
    }

    fn opt(code: u8, data: &[u8]) -> (u8, Vec<u8>) {
        (code, data.to_vec())
    }

    fn csr(prefix_len: u8, destination: [u8; 4], router: [u8; 4]) -> Vec<u8> {
        let mut out = vec![prefix_len];
        let octets = usize::from(prefix_len.div_ceil(8));
        out.extend_from_slice(&destination[..octets]);
        out.extend_from_slice(&router);
        out
    }

    fn dns_name(labels: &[&str]) -> Vec<u8> {
        let mut out = Vec::new();
        for label in labels {
            out.push(label.len() as u8);
            out.extend_from_slice(label.as_bytes());
        }
        out.push(0);
        out
    }

    fn wire_config() -> Dhcp4ClientConfig {
        Dhcp4ClientConfig::new(0x3903_f326, [0x02, 0x00, 0x5e, 0x10, 0x20, 0x30])
    }

    fn packet_option(packet: &[u8], code: u8) -> Option<Vec<u8>> {
        let options = parse_options(&packet[BOOTP_MIN_LEN..]).unwrap();
        option(&options, code).map(|data| data.to_vec())
    }

    fn packet_secs(packet: &[u8]) -> u16 {
        u16::from_be_bytes([packet[DHCP_SECS_OFFSET], packet[DHCP_SECS_OFFSET + 1]])
    }

    fn packet_xid(packet: &[u8]) -> u32 {
        u32::from_be_bytes([
            packet[DHCP_XID_OFFSET],
            packet[DHCP_XID_OFFSET + 1],
            packet[DHCP_XID_OFFSET + 2],
            packet[DHCP_XID_OFFSET + 3],
        ])
    }

    fn reply(
        yiaddr: [u8; 4],
        xid: u32,
        chaddr: [u8; 6],
        message_type: u8,
        mut opts: Vec<(u8, Vec<u8>)>,
    ) -> Vec<u8> {
        let mut packet = vec![0u8; BOOTP_MIN_LEN];
        packet[0] = DHCP_OPCODE_BOOTREPLY;
        packet[1] = DHCP_HTYPE_ETHERNET;
        packet[2] = DHCP_HLEN_ETHERNET;
        packet[DHCP_XID_OFFSET..DHCP_XID_OFFSET + 4].copy_from_slice(&xid.to_be_bytes());
        packet[YIADDR_OFFSET..YIADDR_OFFSET + 4].copy_from_slice(&yiaddr);
        packet[DHCP_CHADDR_OFFSET..DHCP_CHADDR_OFFSET + 6].copy_from_slice(&chaddr);
        packet[MAGIC_COOKIE_OFFSET..MAGIC_COOKIE_OFFSET + 4].copy_from_slice(&DHCP_MAGIC_COOKIE);
        opts.insert(0, (DHCP_OPTION_MESSAGE_TYPE, vec![message_type]));
        for (code, data) in opts {
            packet.push(code);
            packet.push(data.len() as u8);
            packet.extend_from_slice(&data);
        }
        packet.push(DHCP_OPTION_END);
        packet
    }

    fn basic_offer() -> Vec<u8> {
        reply(
            [10, 0, 0, 5],
            0x3903_f326,
            [0x02, 0x00, 0x5e, 0x10, 0x20, 0x30],
            DHCP_MESSAGE_OFFER,
            vec![
                opt(DHCP_OPTION_SERVER_IDENTIFIER, &[10, 0, 0, 1]),
                opt(DHCP_OPTION_SUBNET_MASK, &[255, 255, 255, 0]),
                opt(DHCP_OPTION_ROUTER, &[10, 0, 0, 1]),
            ],
        )
    }

    fn basic_ack_reply() -> Vec<u8> {
        reply(
            [10, 0, 0, 5],
            0x3903_f326,
            [0x02, 0x00, 0x5e, 0x10, 0x20, 0x30],
            DHCP_MESSAGE_ACK,
            vec![
                opt(DHCP_OPTION_SERVER_IDENTIFIER, &[10, 0, 0, 1]),
                opt(DHCP_OPTION_SUBNET_MASK, &[255, 255, 255, 0]),
                opt(DHCP_OPTION_ROUTER, &[10, 0, 0, 1]),
                opt(DHCP_OPTION_DNS, &[8, 8, 8, 8, 8, 8, 4, 4]),
                opt(DHCP_OPTION_HOSTNAME, b"leased-host"),
                opt(DHCP_OPTION_DOMAIN_NAME, b"example.com"),
                opt(DHCP_OPTION_INTERFACE_MTU, &[0x05, 0xdc]),
                opt(DHCP_OPTION_NTP, &[169, 254, 169, 123]),
            ],
        )
    }

    #[test]
    fn dhcp4_wire_builds_discover_with_fixed_xid_chaddr_and_requested_options() {
        let packet = build_dhcp4_discover(&wire_config()).unwrap();

        assert_eq!(packet[0], DHCP_OPCODE_BOOTREQUEST);
        assert_eq!(packet[1], DHCP_HTYPE_ETHERNET);
        assert_eq!(packet[2], DHCP_HLEN_ETHERNET);
        assert_eq!(packet_xid(&packet), 0x3903_f326);
        assert_eq!(packet_secs(&packet), 0);
        assert_eq!(
            &packet[DHCP_FLAGS_OFFSET..DHCP_FLAGS_OFFSET + 2],
            &DHCP_BROADCAST_FLAG.to_be_bytes()
        );
        assert_eq!(
            &packet[DHCP_CHADDR_OFFSET..DHCP_CHADDR_OFFSET + 6],
            &[0x02, 0x00, 0x5e, 0x10, 0x20, 0x30]
        );
        assert_eq!(
            &packet[MAGIC_COOKIE_OFFSET..MAGIC_COOKIE_OFFSET + 4],
            &DHCP_MAGIC_COOKIE
        );
        assert_eq!(
            packet_option(&packet, DHCP_OPTION_MESSAGE_TYPE).unwrap(),
            vec![DHCP_MESSAGE_DISCOVER]
        );
        assert_eq!(
            packet_option(&packet, DHCP_OPTION_CLIENT_IDENTIFIER).unwrap(),
            vec![1, 0x02, 0x00, 0x5e, 0x10, 0x20, 0x30]
        );
        assert_eq!(
            packet_option(&packet, DHCP_OPTION_MAX_MESSAGE_SIZE).unwrap(),
            DHCP_MAX_MESSAGE_SIZE.to_be_bytes()
        );
        assert_eq!(
            packet_option(&packet, DHCP_OPTION_PARAMETER_REQUEST_LIST).unwrap(),
            vec![
                DHCP_OPTION_SUBNET_MASK,
                DHCP_OPTION_ROUTER,
                DHCP_OPTION_DOMAIN_NAME,
                DHCP_OPTION_DNS,
                DHCP_OPTION_CLASSLESS_STATIC_ROUTE,
                DHCP_OPTION_DOMAIN_SEARCH,
                DHCP_OPTION_NTP,
                DHCP_OPTION_INTERFACE_MTU,
                DHCP_OPTION_HOSTNAME,
            ]
        );
        assert!(packet_option(&packet, DHCP_OPTION_REQUESTED_IP).is_none());
        assert!(packet_option(&packet, DHCP_OPTION_SERVER_IDENTIFIER).is_none());
        assert!(matches!(packet.last(), Some(&DHCP_OPTION_END)));
    }

    #[test]
    fn dhcp4_wire_builds_discover_with_hostname_payload_without_hostname_prl() {
        let config = wire_config()
            .without_mtu_request()
            .with_hostname("node-a", Some("example.com"));
        let packet = build_dhcp4_discover(&config).unwrap();
        let prl = packet_option(&packet, DHCP_OPTION_PARAMETER_REQUEST_LIST).unwrap();

        assert_eq!(
            packet_option(&packet, DHCP_OPTION_HOSTNAME).unwrap(),
            b"node-a"
        );
        assert_eq!(
            packet_option(&packet, DHCP_OPTION_DOMAIN_NAME).unwrap(),
            b"example.com"
        );
        assert!(!prl.contains(&DHCP_OPTION_HOSTNAME));
        assert!(!prl.contains(&DHCP_OPTION_INTERFACE_MTU));
        assert!(prl.contains(&DHCP_OPTION_DOMAIN_NAME)); // inherited library default
        assert!(prl.contains(&DHCP_OPTION_CLASSLESS_STATIC_ROUTE));
    }

    #[test]
    fn dhcp4_wire_builds_request_from_offer_and_ack_state() {
        let config = wire_config();
        let offer = parse_dhcp4_offer(
            &basic_offer(),
            config.transaction_id,
            config.client_hardware_addr,
        )
        .unwrap()
        .unwrap();
        let request = build_dhcp4_request_from_offer(&config, &offer).unwrap();

        assert_eq!(
            packet_option(&request, DHCP_OPTION_MESSAGE_TYPE).unwrap(),
            vec![DHCP_MESSAGE_REQUEST]
        );
        assert_eq!(
            packet_option(&request, DHCP_OPTION_REQUESTED_IP).unwrap(),
            vec![10, 0, 0, 5]
        );
        assert_eq!(
            packet_option(&request, DHCP_OPTION_SERVER_IDENTIFIER).unwrap(),
            vec![10, 0, 0, 1]
        );
        assert_eq!(
            &request[DHCP_CIADDR_OFFSET..DHCP_CIADDR_OFFSET + 4],
            &[0, 0, 0, 0]
        );

        let mut tx = Dhcp4WireTransaction::new(config);
        let discover = tx.start().unwrap();
        assert_eq!(discover.target, Dhcp4SendTarget::Broadcast);
        assert_eq!(tx.state(), Dhcp4WireState::Selecting);

        let action = tx.handle_packet(&basic_offer()).unwrap();
        let Dhcp4WireAction::Send(outbound_request) = action else {
            panic!("offer should trigger request");
        };
        assert_eq!(tx.state(), Dhcp4WireState::Requesting);
        assert_eq!(outbound_request.target, Dhcp4SendTarget::Broadcast);

        let action = tx.handle_packet(&basic_ack_reply()).unwrap();
        let Dhcp4WireAction::Bound(lease) = action else {
            panic!("ack should bind lease");
        };
        assert_eq!(tx.state(), Dhcp4WireState::Bound);
        assert_eq!(lease.address, v4("10.0.0.5"));
        assert_eq!(lease.prefix_len, 24);
        assert_eq!(lease.routers, vec![v4("10.0.0.1")]);
    }

    #[test]
    fn dhcp4_wire_builds_reboot_and_renew_requests() {
        let config = wire_config();
        let reboot = build_dhcp4_reboot_request(&config, v4("10.0.0.5")).unwrap();
        assert_eq!(
            packet_option(&reboot, DHCP_OPTION_MESSAGE_TYPE).unwrap(),
            vec![DHCP_MESSAGE_REQUEST]
        );
        assert_eq!(
            packet_option(&reboot, DHCP_OPTION_REQUESTED_IP).unwrap(),
            vec![10, 0, 0, 5]
        );
        assert!(packet_option(&reboot, DHCP_OPTION_SERVER_IDENTIFIER).is_none());
        assert_eq!(
            &reboot[DHCP_CIADDR_OFFSET..DHCP_CIADDR_OFFSET + 4],
            &[0, 0, 0, 0]
        );

        let renew = build_dhcp4_renew_request(&config, v4("10.0.0.5"), v4("10.0.0.1")).unwrap();
        assert_eq!(renew.target, Dhcp4SendTarget::Unicast(v4("10.0.0.1")));
        assert_eq!(
            packet_option(&renew.packet, DHCP_OPTION_MESSAGE_TYPE).unwrap(),
            vec![DHCP_MESSAGE_REQUEST]
        );
        assert_eq!(
            &renew.packet[DHCP_CIADDR_OFFSET..DHCP_CIADDR_OFFSET + 4],
            &[10, 0, 0, 5]
        );
        assert!(packet_option(&renew.packet, DHCP_OPTION_REQUESTED_IP).is_none());
        assert!(packet_option(&renew.packet, DHCP_OPTION_SERVER_IDENTIFIER).is_none());
        assert_eq!(
            &renew.packet[DHCP_FLAGS_OFFSET..DHCP_FLAGS_OFFSET + 2],
            &[0, 0]
        );
    }

    #[test]
    fn dhcp4_wire_retries_use_same_xid_and_cumulative_secs() {
        assert_eq!(dhcp4_elapsed_secs_for_attempt(4, 0), 0);
        assert_eq!(dhcp4_elapsed_secs_for_attempt(4, 1), 4);
        assert_eq!(dhcp4_elapsed_secs_for_attempt(4, 2), 12);
        assert_eq!(dhcp4_elapsed_secs_for_attempt(4, 3), 28);

        let mut tx = Dhcp4WireTransaction::new(wire_config()).with_retry_schedule(4, 4);
        let packets = [
            tx.start().unwrap().packet,
            tx.handle_timeout().unwrap().packet,
            tx.handle_timeout().unwrap().packet,
            tx.handle_timeout().unwrap().packet,
        ];
        assert!(matches!(tx.handle_timeout(), Err(Error::Timeout)));

        let secs: Vec<u16> = packets.iter().map(|packet| packet_secs(packet)).collect();
        assert_eq!(secs, vec![0, 4, 12, 28]);
        assert!(
            packets
                .iter()
                .all(|packet| packet_xid(packet) == 0x3903_f326)
        );
    }

    #[test]
    fn dhcp4_ack_lease_timers_drive_source_renew_intervals() {
        let packet = ack(
            [10, 0, 0, 5],
            vec![
                opt(DHCP_OPTION_SUBNET_MASK, &[255, 255, 255, 0]),
                opt(DHCP_OPTION_IP_ADDRESS_LEASE_TIME, &120u32.to_be_bytes()),
                opt(DHCP_OPTION_RENEWAL_TIME, &45u32.to_be_bytes()),
                opt(DHCP_OPTION_REBINDING_TIME, &90u32.to_be_bytes()),
            ],
        );

        let lease = parse_dhcp4_ack(&packet).unwrap();
        assert_eq!(lease.lease_time_secs, 120);
        assert_eq!(lease.renewal_time_secs, Some(45));
        assert_eq!(lease.rebinding_time_secs, Some(90));
        assert_eq!(dhcp4_renew_interval_secs(lease.lease_time_secs), 60);

        let defaulted = parse_dhcp4_ack(&ack(
            [10, 0, 0, 5],
            vec![opt(DHCP_OPTION_SUBNET_MASK, &[255, 255, 255, 0])],
        ))
        .unwrap();
        assert_eq!(defaulted.lease_time_secs, 30 * 60);
        assert_eq!(
            dhcp4_renew_interval_secs(defaulted.lease_time_secs),
            15 * 60
        );
        assert_eq!(dhcp4_renew_interval_secs(4), 5);
        assert_eq!(dhcp4_failed_renew_interval_secs(60), 30);
        assert_eq!(dhcp4_failed_renew_interval_secs(4), 5);
    }

    #[test]
    fn rejects_malformed_dhcp4_lease_timers() {
        let packet = ack(
            [10, 0, 0, 5],
            vec![
                opt(DHCP_OPTION_SUBNET_MASK, &[255, 255, 255, 0]),
                opt(DHCP_OPTION_IP_ADDRESS_LEASE_TIME, &[0, 1, 2]),
            ],
        );

        assert!(parse_dhcp4_ack(&packet).is_err());
    }

    #[test]
    fn dhcp4_wire_rejects_wrong_xid_chaddr_and_nak_restarts() {
        let config = wire_config();
        let mut wrong_xid = basic_offer();
        wrong_xid[DHCP_XID_OFFSET..DHCP_XID_OFFSET + 4]
            .copy_from_slice(&0xdead_beefu32.to_be_bytes());
        assert!(
            parse_dhcp4_offer(
                &wrong_xid,
                config.transaction_id,
                config.client_hardware_addr
            )
            .unwrap()
            .is_none()
        );

        let mut wrong_chaddr = basic_offer();
        wrong_chaddr[DHCP_CHADDR_OFFSET + 5] ^= 0xff;
        assert!(
            parse_dhcp4_offer(
                &wrong_chaddr,
                config.transaction_id,
                config.client_hardware_addr
            )
            .unwrap()
            .is_none()
        );

        let mut tx = Dhcp4WireTransaction::new(config);
        tx.start().unwrap();
        tx.handle_packet(&basic_offer()).unwrap();
        assert_eq!(tx.state(), Dhcp4WireState::Requesting);

        let nak = reply(
            [0, 0, 0, 0],
            0x3903_f326,
            [0x02, 0x00, 0x5e, 0x10, 0x20, 0x30],
            DHCP_MESSAGE_NAK,
            vec![opt(DHCP_OPTION_SERVER_IDENTIFIER, &[10, 0, 0, 1])],
        );
        let action = tx.handle_packet(&nak).unwrap();
        let Dhcp4WireAction::Send(discover) = action else {
            panic!("nak should restart discovery");
        };
        assert_eq!(tx.state(), Dhcp4WireState::Selecting);
        assert_eq!(
            packet_option(&discover.packet, DHCP_OPTION_MESSAGE_TYPE).unwrap(),
            vec![DHCP_MESSAGE_DISCOVER]
        );
    }

    #[test]
    fn parses_basic_ack_and_applies_operator_output() {
        let packet = ack(
            [10, 0, 0, 5],
            vec![
                opt(DHCP_OPTION_SUBNET_MASK, &[255, 255, 255, 0]),
                opt(DHCP_OPTION_ROUTER, &[10, 0, 0, 1]),
                opt(DHCP_OPTION_DNS, &[8, 8, 8, 8, 8, 8, 4, 4]),
                opt(DHCP_OPTION_HOSTNAME, b"leased-host"),
                opt(DHCP_OPTION_DOMAIN_NAME, b"example.com"),
                opt(DHCP_OPTION_INTERFACE_MTU, &[0x05, 0xdc]),
                opt(DHCP_OPTION_NTP, &[169, 254, 169, 123]),
            ],
        );

        let lease = parse_dhcp4_ack(&packet).unwrap();
        assert_eq!(lease.address, v4("10.0.0.5"));
        assert_eq!(lease.prefix_len, 24);
        assert_eq!(lease.routers, vec![v4("10.0.0.1")]);
        assert_eq!(lease.dns_servers, vec![v4("8.8.8.8"), v4("8.8.4.4")]);
        assert_eq!(lease.ntp_servers, vec![v4("169.254.169.123")]);
        assert_eq!(lease.hostname.as_deref(), Some("leased-host"));
        assert_eq!(lease.domain_name.as_deref(), Some("example.com"));
        assert_eq!(lease.search_domains, vec!["example.com".to_string()]);
        assert_eq!(lease.mtu, Some(1500));

        let output = OperatorSpec::dhcp4("eth0")
            .apply_dhcp4_lease(&lease, true)
            .unwrap();
        assert_eq!(output.addresses[0].id(), "eth0/10.0.0.5/24");
        assert_eq!(output.routes.len(), 1);
        assert!(output.routes[0].is_default());
        assert_eq!(output.routes[0].gateway, Some(v4("10.0.0.1")));
        assert_eq!(output.routes[0].protocol, RouteProtocol::Boot);
        assert_eq!(output.routes[0].metric, 1024);
        assert_eq!(
            output.resolver.unwrap().servers,
            vec![v4("8.8.8.8"), v4("8.8.4.4")]
        );
        assert_eq!(output.time_servers, vec!["169.254.169.123".to_string()]);
        let hostname = output.hostname.unwrap();
        assert_eq!(hostname.hostname.as_str(), "leased-host");
        assert_eq!(hostname.domainname.as_deref(), Some("example.com"));
    }

    #[test]
    fn classless_routes_ignore_router_and_add_off_subnet_helper_once() {
        let mut routes = Vec::new();
        routes.extend(csr(32, [169, 254, 169, 123], [169, 254, 0, 1]));
        routes.extend(csr(32, [169, 254, 169, 249], [169, 254, 0, 1]));
        routes.extend(csr(31, [169, 254, 169, 254], [169, 254, 0, 1]));

        let packet = ack(
            [169, 254, 251, 148],
            vec![
                opt(DHCP_OPTION_SUBNET_MASK, &[255, 255, 255, 255]),
                opt(DHCP_OPTION_ROUTER, &[10, 0, 0, 1]),
                opt(DHCP_OPTION_CLASSLESS_STATIC_ROUTE, &routes),
            ],
        );

        let lease = parse_dhcp4_ack(&packet).unwrap();
        assert_eq!(lease.classless_routes.len(), 3);

        let output = OperatorSpec::dhcp4("eth0")
            .apply_dhcp4_lease(&lease, false)
            .unwrap();
        assert_eq!(output.routes.len(), 4);
        assert_eq!(output.routes[0].destination, Some(v4("169.254.169.123")));
        assert_eq!(output.routes[0].prefix_len, 32);
        assert_eq!(output.routes[0].gateway, Some(v4("169.254.0.1")));
        assert_eq!(output.routes[1].destination, Some(v4("169.254.0.1")));
        assert_eq!(output.routes[1].prefix_len, 32);
        assert_eq!(output.routes[1].gateway, None);
        assert_eq!(output.routes[2].destination, Some(v4("169.254.169.249")));
        assert_eq!(output.routes[3].destination, Some(v4("169.254.169.254")));
        assert_eq!(output.routes[3].prefix_len, 31);
    }

    #[test]
    fn classless_on_link_router_has_no_gateway() {
        let route = csr(24, [192, 168, 1, 0], [0, 0, 0, 0]);
        let packet = ack(
            [10, 0, 0, 5],
            vec![
                opt(DHCP_OPTION_SUBNET_MASK, &[255, 255, 255, 0]),
                opt(DHCP_OPTION_CLASSLESS_STATIC_ROUTE, &route),
            ],
        );

        let lease = parse_dhcp4_ack(&packet).unwrap();
        let output = OperatorSpec::dhcp4("eth0")
            .apply_dhcp4_lease(&lease, false)
            .unwrap();
        assert_eq!(output.routes.len(), 1);
        assert_eq!(output.routes[0].destination, Some(v4("192.168.1.0")));
        assert_eq!(output.routes[0].gateway, None);
    }

    #[test]
    fn domain_search_appends_domain_name_without_duplication() {
        let mut search = dns_name(&["corp", "example", "com"]);
        search.extend(dns_name(&["example", "com"]));
        let packet = ack(
            [10, 0, 0, 5],
            vec![
                opt(DHCP_OPTION_SUBNET_MASK, &[255, 255, 255, 0]),
                opt(DHCP_OPTION_DNS, &[8, 8, 8, 8]),
                opt(DHCP_OPTION_DOMAIN_SEARCH, &search),
                opt(DHCP_OPTION_DOMAIN_NAME, b"example.com\0\0"),
            ],
        );

        let lease = parse_dhcp4_ack(&packet).unwrap();
        assert_eq!(
            lease.search_domains,
            vec!["corp.example.com".to_string(), "example.com".to_string()]
        );
    }

    #[test]
    fn dhcp4_lease_materializes_resolver_with_dns_and_search_domains() {
        let mut search = dns_name(&["corp", "example", "com"]);
        search.extend(dns_name(&["example", "com"]));
        let packet = ack(
            [10, 0, 0, 5],
            vec![
                opt(DHCP_OPTION_SUBNET_MASK, &[255, 255, 255, 0]),
                opt(DHCP_OPTION_DNS, &[8, 8, 8, 8, 8, 8, 4, 4]),
                opt(DHCP_OPTION_DOMAIN_SEARCH, &search),
                opt(DHCP_OPTION_DOMAIN_NAME, b"example.com"),
            ],
        );

        let lease = parse_dhcp4_ack(&packet).unwrap();
        let output = OperatorSpec::dhcp4("eth0")
            .apply_dhcp4_lease(&lease, false)
            .unwrap();
        let resolver = output.resolver.unwrap();
        assert_eq!(resolver.servers, vec![v4("8.8.8.8"), v4("8.8.4.4")]);
        assert_eq!(
            resolver.search_domains,
            vec!["corp.example.com".to_string(), "example.com".to_string()]
        );
        assert_eq!(
            resolver.render_resolv_conf(),
            "search corp.example.com example.com\nnameserver 8.8.8.8\nnameserver 8.8.4.4\n"
        );
    }

    #[test]
    fn dhcp4_search_only_lease_materializes_resolver_conf() {
        let packet = ack(
            [10, 0, 0, 5],
            vec![
                opt(DHCP_OPTION_SUBNET_MASK, &[255, 255, 255, 0]),
                opt(DHCP_OPTION_DOMAIN_NAME, b"example.com"),
            ],
        );

        let lease = parse_dhcp4_ack(&packet).unwrap();
        let output = OperatorSpec::dhcp4("eth0")
            .apply_dhcp4_lease(&lease, false)
            .unwrap();
        let resolver = output.resolver.unwrap();
        assert!(resolver.servers.is_empty());
        assert_eq!(resolver.search_domains, vec!["example.com".to_string()]);
        assert_eq!(resolver.render_resolv_conf(), "search example.com\n");
    }

    #[test]
    fn rejects_non_ack_and_malformed_options() {
        let mut packet = ack(
            [10, 0, 0, 5],
            vec![opt(DHCP_OPTION_SUBNET_MASK, &[255, 255, 255, 0])],
        );
        let msg_type = BOOTP_MIN_LEN + 2;
        packet[msg_type] = 2; // OFFER
        assert!(parse_dhcp4_ack(&packet).is_err());

        let mut malformed = ack(
            [10, 0, 0, 5],
            vec![opt(DHCP_OPTION_SUBNET_MASK, &[255, 0, 255, 0])],
        );
        // Keep the packet otherwise valid; non-contiguous subnet mask should fail.
        assert!(parse_dhcp4_ack(&malformed).is_err());
        malformed.truncate(BOOTP_MIN_LEN - 1);
        assert!(parse_dhcp4_ack(&malformed).is_err());
    }
}
