//! DHCPv6 reply parsing and lease-to-operator translation.
//!
//! Mirrors the pure output boundary of Talos'
//! `internal/app/machined/pkg/controllers/network/operator/dhcp6.go`: the real
//! operator obtains a DHCPv6 `REPLY` through `insomniacslk/dhcp`'s rapid-solicit
//! client, then projects the reply into address, resolver, hostname, and NTP
//! time-server output. This module keeps the packet construction and parser
//! deterministic and socket-free. The live UDP/socket loop is a later
//! PID1/operator slice.

use crate::address::AddressSpec;
use crate::config_layer::ConfigLayer;
use crate::hostname::HostnameSpec;
use crate::operator::{OperatorOutput, OperatorSpec};
use crate::resolver::ResolverSpec;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};

const DHCP6_MESSAGE_SOLICIT: u8 = 1;
const DHCP6_MESSAGE_ADVERTISE: u8 = 2;
const DHCP6_MESSAGE_REQUEST: u8 = 3;
const DHCP6_MESSAGE_REPLY: u8 = 7;
const DHCP6_OPTION_CLIENT_ID: u16 = 1;
const DHCP6_OPTION_SERVER_ID: u16 = 2;
const DHCP6_OPTION_IA_NA: u16 = 3;
const DHCP6_OPTION_IA_ADDRESS: u16 = 5;
const DHCP6_OPTION_ORO: u16 = 6;
const DHCP6_OPTION_ELAPSED_TIME: u16 = 8;
const DHCP6_OPTION_RAPID_COMMIT: u16 = 14;
const DHCP6_OPTION_DNS_SERVERS: u16 = 23;
const DHCP6_OPTION_DOMAIN_SEARCH: u16 = 24;
const DHCP6_OPTION_FQDN: u16 = 39;
const DHCP6_OPTION_NTP_SERVER: u16 = 56;
const DHCP6_NTP_SUBOPTION_SERVER_ADDR: u16 = 1;
const DHCP6_MIN_RENEW_SECS: u64 = 5;
const DHCP6_DUID_TYPE_LLT: u16 = 1;
const DHCP6_DUID_TYPE_LL: u16 = 3;
const DHCP6_DUID_HW_TYPE_ETHERNET: u16 = 1;

/// A parsed DHCPv6 IA_NA address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dhcp6IaAddress {
    /// The IPv6 address offered by the server.
    pub address: NodeAddress,
    /// Preferred lifetime in seconds.
    pub preferred_lifetime_secs: u32,
    /// Valid lifetime in seconds. Talos uses this as the lease duration input.
    pub valid_lifetime_secs: u32,
}

/// Parsed DHCPv6 REPLY data used by the network operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dhcp6Lease {
    /// Transaction id from the DHCPv6 message header.
    pub transaction_id: [u8; 3],
    /// First IA_NA/IAADDR address, matching Talos' `OneIANA().OneAddress()` behavior.
    pub address: Option<Dhcp6IaAddress>,
    /// DNS recursive name servers from option 23.
    pub dns_servers: Vec<NodeAddress>,
    /// Fully-qualified hostname from option 39, if supplied.
    pub fqdn: Option<String>,
    /// NTP server addresses from option 56 suboption 1.
    pub ntp_servers: Vec<NodeAddress>,
}

impl Dhcp6Lease {
    /// Lease duration used by the Talos DHCPv6 renewal loop.
    pub fn lease_time_secs(&self) -> u32 {
        self.address
            .as_ref()
            .map(|address| address.valid_lifetime_secs)
            .unwrap_or(0)
    }
}

/// DHCPv6 client identifier policy for outbound requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dhcp6ClientIdentifier {
    /// Do not send DHCPv6 option 1.
    None,
    /// Build the upstream library default DUID-LLT value for Ethernet from the
    /// link MAC address and DHCPv6 time (seconds since 2000-01-01 UTC).
    DuidLlt {
        /// Link hardware address.
        mac: [u8; 6],
        /// DHCPv6 DUID-LLT time field.
        seconds_since_2000: u32,
    },
    /// Build a DUID-LL value for Ethernet from the link MAC address.
    Mac([u8; 6]),
    /// Send a caller-provided DUID value verbatim.
    Duid(Vec<u8>),
}

/// Stable inputs for deterministic DHCPv6 packet construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dhcp6ClientConfig {
    /// DHCPv6 transaction id.
    pub transaction_id: [u8; 3],
    /// Transaction id for the REQUEST generated after a non-rapid ADVERTISE.
    pub request_transaction_id: [u8; 3],
    /// IA_NA IAID requested by the client.
    pub iaid: u32,
    /// Client identifier behavior for option 1.
    pub client_identifier: Dhcp6ClientIdentifier,
    /// Whether to request option 39 (client FQDN) in the ORO.
    pub request_fqdn: bool,
}

impl Dhcp6ClientConfig {
    /// Build a DHCPv6 client config with Talos-like option request defaults.
    pub fn new(
        transaction_id: [u8; 3],
        iaid: u32,
        client_identifier: Dhcp6ClientIdentifier,
    ) -> Self {
        Dhcp6ClientConfig {
            transaction_id,
            request_transaction_id: transaction_id,
            iaid,
            client_identifier,
            request_fqdn: false,
        }
    }

    /// Return a copy with a deterministic non-rapid REQUEST transaction id.
    pub fn with_request_transaction_id(mut self, transaction_id: [u8; 3]) -> Self {
        self.request_transaction_id = transaction_id;
        self
    }

    /// Return a copy that requests the client FQDN option.
    pub fn with_fqdn_request(mut self) -> Self {
        self.request_fqdn = true;
        self
    }

    /// Return a copy that suppresses FQDN in the option request option.
    pub fn without_fqdn_request(mut self) -> Self {
        self.request_fqdn = false;
        self
    }

    /// Materialize the wire DUID implied by this configuration.
    pub fn client_duid(&self) -> Result<Option<Vec<u8>>> {
        match &self.client_identifier {
            Dhcp6ClientIdentifier::None => Ok(None),
            Dhcp6ClientIdentifier::DuidLlt {
                mac,
                seconds_since_2000,
            } => {
                let mut duid = Vec::with_capacity(14);
                duid.extend_from_slice(&DHCP6_DUID_TYPE_LLT.to_be_bytes());
                duid.extend_from_slice(&DHCP6_DUID_HW_TYPE_ETHERNET.to_be_bytes());
                duid.extend_from_slice(&seconds_since_2000.to_be_bytes());
                duid.extend_from_slice(mac);
                Ok(Some(duid))
            }
            Dhcp6ClientIdentifier::Mac(mac) => {
                let mut duid = Vec::with_capacity(10);
                duid.extend_from_slice(&DHCP6_DUID_TYPE_LL.to_be_bytes());
                duid.extend_from_slice(&DHCP6_DUID_HW_TYPE_ETHERNET.to_be_bytes());
                duid.extend_from_slice(mac);
                Ok(Some(duid))
            }
            Dhcp6ClientIdentifier::Duid(raw) => {
                if raw.is_empty() {
                    Err(Error::invalid("DHCPv6 DUID client identifier is empty"))
                } else {
                    Ok(Some(raw.clone()))
                }
            }
        }
    }
}

/// A network send target selected by the DHCPv6 state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dhcp6SendTarget {
    /// Multicast to ff02::1:2 / UDP port 547.
    AllDhcpRelayAgentsAndServers,
}

/// An outbound DHCPv6 packet and its intended target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dhcp6Outbound {
    /// Packet bytes suitable for UDP payload transmission.
    pub packet: Vec<u8>,
    /// Destination choice for the socket layer.
    pub target: Dhcp6SendTarget,
}

/// Pure DHCPv6 rapid-solicit/request state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dhcp6WireState {
    /// No packet sent yet.
    Init,
    /// SOLICIT has been sent, waiting for REPLY or ADVERTISE.
    Soliciting,
    /// REQUEST has been sent after a non-rapid ADVERTISE.
    Requesting,
    /// REPLY has been accepted.
    Bound,
}

/// Result of feeding a packet or timer event to [`Dhcp6WireTransaction`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dhcp6WireAction {
    /// Send this packet next.
    Send(Dhcp6Outbound),
    /// A lease has been accepted.
    Bound(Dhcp6Lease),
    /// Packet was well-formed enough to classify but irrelevant to this state.
    Ignored(&'static str),
}

/// Deterministic DHCPv6 SOLICIT/ADVERTISE/REQUEST/REPLY state machine.
///
/// The type performs no socket I/O and no sleeping. PID1 or the operator runtime
/// owns the actual UDP socket boundary; this struct owns Talos-compatible packet
/// construction, transaction matching, retry packet generation, and REPLY parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dhcp6WireTransaction {
    config: Dhcp6ClientConfig,
    state: Dhcp6WireState,
    server_identifier: Option<Vec<u8>>,
    selected_ia_na: Option<Vec<u8>>,
}

impl Dhcp6WireTransaction {
    /// Create a new transaction.
    pub fn new(config: Dhcp6ClientConfig) -> Self {
        Dhcp6WireTransaction {
            config,
            state: Dhcp6WireState::Init,
            server_identifier: None,
            selected_ia_na: None,
        }
    }

    /// Current state.
    pub fn state(&self) -> Dhcp6WireState {
        self.state
    }

    /// Start the transaction by emitting a rapid SOLICIT.
    pub fn start(&mut self) -> Result<Dhcp6Outbound> {
        self.state = Dhcp6WireState::Soliciting;
        self.server_identifier = None;
        self.selected_ia_na = None;
        self.solicit(0)
    }

    /// Generate a retry packet with a caller-measured elapsed time.
    pub fn retry(&mut self, elapsed_centisecs: u16) -> Result<Dhcp6Outbound> {
        match self.state {
            Dhcp6WireState::Init => {
                self.state = Dhcp6WireState::Soliciting;
                self.solicit(elapsed_centisecs)
            }
            Dhcp6WireState::Soliciting => self.solicit(elapsed_centisecs),
            Dhcp6WireState::Requesting => self.request_from_selected(elapsed_centisecs),
            Dhcp6WireState::Bound => {
                Err(Error::invalid_state("DHCPv6 transaction is already bound"))
            }
        }
    }

    /// Feed one inbound DHCPv6 packet into the state machine.
    pub fn handle_packet(&mut self, packet: &[u8]) -> Result<Dhcp6WireAction> {
        let (message_type, options) = match packet_header_and_options(packet) {
            Ok(parsed) => parsed,
            Err(err) => {
                if self.state == Dhcp6WireState::Init {
                    return Ok(Dhcp6WireAction::Ignored("not started"));
                }
                return Err(err);
            }
        };

        match self.state {
            Dhcp6WireState::Init => Ok(Dhcp6WireAction::Ignored("not started")),
            Dhcp6WireState::Soliciting => match message_type {
                DHCP6_MESSAGE_REPLY => {
                    if !self.matches_identity(packet, &options, self.config.transaction_id, None)? {
                        return Ok(Dhcp6WireAction::Ignored("not matching reply"));
                    }
                    let lease = parse_dhcp6_reply(packet)?;
                    self.state = Dhcp6WireState::Bound;
                    Ok(Dhcp6WireAction::Bound(lease))
                }
                DHCP6_MESSAGE_ADVERTISE => {
                    if !self.matches_identity(packet, &options, self.config.transaction_id, None)? {
                        return Ok(Dhcp6WireAction::Ignored("not matching advertise"));
                    }
                    let server_identifier = option_data(&options, DHCP6_OPTION_SERVER_ID)
                        .ok_or_else(|| {
                            Error::parse("DHCPv6 ADVERTISE missing server identifier")
                        })?;
                    let ia_na = option_data(&options, DHCP6_OPTION_IA_NA)
                        .ok_or_else(|| Error::parse("DHCPv6 ADVERTISE missing IA_NA option"))?;
                    self.server_identifier = Some(server_identifier.to_vec());
                    self.selected_ia_na = Some(ia_na.to_vec());
                    self.state = Dhcp6WireState::Requesting;
                    Ok(Dhcp6WireAction::Send(self.request_from_selected(0)?))
                }
                _ => Ok(Dhcp6WireAction::Ignored("not reply or advertise")),
            },
            Dhcp6WireState::Requesting => match message_type {
                DHCP6_MESSAGE_REPLY => {
                    let server_identifier = self.server_identifier.as_deref().ok_or_else(|| {
                        Error::invalid_state("DHCPv6 request has no server identifier")
                    })?;
                    if !self.matches_identity(
                        packet,
                        &options,
                        self.config.request_transaction_id,
                        Some(server_identifier),
                    )? {
                        return Ok(Dhcp6WireAction::Ignored("not matching reply"));
                    }
                    let lease = parse_dhcp6_reply(packet)?;
                    self.state = Dhcp6WireState::Bound;
                    Ok(Dhcp6WireAction::Bound(lease))
                }
                _ => Ok(Dhcp6WireAction::Ignored("not reply")),
            },
            Dhcp6WireState::Bound => Ok(Dhcp6WireAction::Ignored("already bound")),
        }
    }

    fn solicit(&self, elapsed_centisecs: u16) -> Result<Dhcp6Outbound> {
        Ok(Dhcp6Outbound {
            packet: build_dhcp6_rapid_solicit(&self.config, elapsed_centisecs)?,
            target: Dhcp6SendTarget::AllDhcpRelayAgentsAndServers,
        })
    }

    fn request_from_selected(&self, elapsed_centisecs: u16) -> Result<Dhcp6Outbound> {
        let server_identifier = self
            .server_identifier
            .as_deref()
            .ok_or_else(|| Error::invalid_state("DHCPv6 request has no server identifier"))?;
        let ia_na = self
            .selected_ia_na
            .as_deref()
            .ok_or_else(|| Error::invalid_state("DHCPv6 request has no selected IA_NA"))?;
        Ok(Dhcp6Outbound {
            packet: build_dhcp6_request_with_ia_na(
                &self.config,
                server_identifier,
                ia_na,
                elapsed_centisecs,
            )?,
            target: Dhcp6SendTarget::AllDhcpRelayAgentsAndServers,
        })
    }

    fn matches_identity(
        &self,
        packet: &[u8],
        options: &[Dhcp6Option<'_>],
        expected_transaction_id: [u8; 3],
        expected_server_identifier: Option<&[u8]>,
    ) -> Result<bool> {
        if packet.len() < 4 || packet[1..4] != expected_transaction_id[..] {
            return Ok(false);
        }

        if let Some(expected_client_identifier) = self.config.client_duid()? {
            match option_data(options, DHCP6_OPTION_CLIENT_ID) {
                Some(actual) if actual == expected_client_identifier.as_slice() => {}
                _ => return Ok(false),
            }
        }

        if let Some(actual_iaid) = first_ia_na_iaid(options)?
            && actual_iaid != self.config.iaid
        {
            return Ok(false);
        }

        if let Some(expected_server_identifier) = expected_server_identifier {
            match option_data(options, DHCP6_OPTION_SERVER_ID) {
                Some(actual) if actual == expected_server_identifier => {}
                _ => return Ok(false),
            }
        }

        Ok(true)
    }
}

/// Build a Talos-style DHCPv6 rapid SOLICIT packet.
pub fn build_dhcp6_rapid_solicit(
    config: &Dhcp6ClientConfig,
    elapsed_centisecs: u16,
) -> Result<Vec<u8>> {
    let mut packet = dhcp6_header(DHCP6_MESSAGE_SOLICIT, config.transaction_id);
    append_common_client_options(&mut packet, config, elapsed_centisecs)?;
    append_dhcp6_option(&mut packet, DHCP6_OPTION_RAPID_COMMIT, &[])?;
    Ok(packet)
}

/// Build a DHCPv6 REQUEST packet for a selected server.
pub fn build_dhcp6_request(
    config: &Dhcp6ClientConfig,
    server_identifier: &[u8],
    elapsed_centisecs: u16,
) -> Result<Vec<u8>> {
    let mut ia_na = Vec::new();
    ia_na.extend_from_slice(&config.iaid.to_be_bytes());
    ia_na.extend_from_slice(&0u32.to_be_bytes());
    ia_na.extend_from_slice(&0u32.to_be_bytes());
    build_dhcp6_request_with_ia_na(config, server_identifier, &ia_na, elapsed_centisecs)
}

/// Build a DHCPv6 REQUEST packet that copies the IA_NA body from an ADVERTISE.
pub fn build_dhcp6_request_with_ia_na(
    config: &Dhcp6ClientConfig,
    server_identifier: &[u8],
    ia_na: &[u8],
    elapsed_centisecs: u16,
) -> Result<Vec<u8>> {
    if server_identifier.is_empty() {
        return Err(Error::invalid("DHCPv6 server identifier is empty"));
    }
    if ia_na.len() < 12 {
        return Err(Error::invalid(
            "DHCPv6 IA_NA option shorter than fixed header",
        ));
    }
    let mut packet = dhcp6_header(DHCP6_MESSAGE_REQUEST, config.request_transaction_id);
    append_client_identifier(&mut packet, config)?;
    append_dhcp6_option(&mut packet, DHCP6_OPTION_SERVER_ID, server_identifier)?;
    append_dhcp6_option(
        &mut packet,
        DHCP6_OPTION_ELAPSED_TIME,
        &elapsed_centisecs.to_be_bytes(),
    )?;
    append_dhcp6_option(&mut packet, DHCP6_OPTION_IA_NA, ia_na)?;
    append_oro(&mut packet, config.request_fqdn)?;
    Ok(packet)
}

fn dhcp6_header(message_type: u8, transaction_id: [u8; 3]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(4);
    packet.push(message_type);
    packet.extend_from_slice(&transaction_id);
    packet
}

fn append_common_client_options(
    packet: &mut Vec<u8>,
    config: &Dhcp6ClientConfig,
    elapsed_centisecs: u16,
) -> Result<()> {
    append_client_identifier(packet, config)?;
    append_oro(packet, config.request_fqdn)?;
    append_dhcp6_option(
        packet,
        DHCP6_OPTION_ELAPSED_TIME,
        &elapsed_centisecs.to_be_bytes(),
    )?;
    append_ia_na(packet, config.iaid)?;
    Ok(())
}

fn append_client_identifier(packet: &mut Vec<u8>, config: &Dhcp6ClientConfig) -> Result<()> {
    if let Some(duid) = config.client_duid()? {
        append_dhcp6_option(packet, DHCP6_OPTION_CLIENT_ID, &duid)?;
    }
    Ok(())
}

fn append_ia_na(packet: &mut Vec<u8>, iaid: u32) -> Result<()> {
    let mut body = Vec::with_capacity(12);
    body.extend_from_slice(&iaid.to_be_bytes());
    body.extend_from_slice(&0u32.to_be_bytes());
    body.extend_from_slice(&0u32.to_be_bytes());
    append_dhcp6_option(packet, DHCP6_OPTION_IA_NA, &body)
}

fn append_oro(packet: &mut Vec<u8>, request_fqdn: bool) -> Result<()> {
    let mut body = Vec::with_capacity(if request_fqdn { 6 } else { 4 });
    body.extend_from_slice(&DHCP6_OPTION_DNS_SERVERS.to_be_bytes());
    body.extend_from_slice(&DHCP6_OPTION_DOMAIN_SEARCH.to_be_bytes());
    if request_fqdn {
        body.extend_from_slice(&DHCP6_OPTION_FQDN.to_be_bytes());
    }
    append_dhcp6_option(packet, DHCP6_OPTION_ORO, &body)
}

fn append_dhcp6_option(packet: &mut Vec<u8>, code: u16, data: &[u8]) -> Result<()> {
    if data.len() > usize::from(u16::MAX) {
        return Err(Error::invalid(alloc::format!(
            "DHCPv6 option {code} exceeds 65535 bytes"
        )));
    }
    packet.extend_from_slice(&code.to_be_bytes());
    packet.extend_from_slice(&(data.len() as u16).to_be_bytes());
    packet.extend_from_slice(data);
    Ok(())
}

fn packet_header_and_options(packet: &[u8]) -> Result<(u8, Vec<Dhcp6Option<'_>>)> {
    if packet.len() < 4 {
        return Err(Error::parse("DHCPv6 packet shorter than header"));
    }
    Ok((packet[0], parse_options(&packet[4..], "dhcp6 options")?))
}

/// Parse a DHCPv6 REPLY packet into deterministic lease data.
pub fn parse_dhcp6_reply(packet: &[u8]) -> Result<Dhcp6Lease> {
    if packet.len() < 4 {
        return Err(Error::parse("DHCPv6 packet shorter than header"));
    }
    if packet[0] != DHCP6_MESSAGE_REPLY {
        return Err(Error::parse(alloc::format!(
            "expected DHCPv6 REPLY message type 7, got {}",
            packet[0]
        )));
    }

    let transaction_id = [packet[1], packet[2], packet[3]];
    let options = parse_options(&packet[4..], "dhcp6 options")?;

    let address = options
        .iter()
        .filter(|option| option.code == DHCP6_OPTION_IA_NA)
        .find_map(|option| parse_ia_na(option.data).transpose())
        .transpose()?;

    let mut dns_servers = Vec::new();
    for option in options
        .iter()
        .filter(|option| option.code == DHCP6_OPTION_DNS_SERVERS)
    {
        dns_servers.extend(parse_ipv6_list(option.data, "dns")?);
    }

    let fqdn = match options
        .iter()
        .find(|option| option.code == DHCP6_OPTION_FQDN)
    {
        Some(option) => parse_fqdn(option.data)?,
        None => None,
    };

    let mut ntp_servers = Vec::new();
    for option in options
        .iter()
        .filter(|option| option.code == DHCP6_OPTION_NTP_SERVER)
    {
        ntp_servers.extend(parse_ntp_servers(option.data)?);
    }

    Ok(Dhcp6Lease {
        transaction_id,
        address,
        dns_servers,
        fqdn,
        ntp_servers,
    })
}

/// Compute Talos' next successful DHCPv6 renewal interval in seconds.
///
/// Upstream sleeps for half of the IAADDR valid lifetime, clamped to a minimum
/// of five seconds. A reply without IAADDR has no valid lifetime and falls back
/// to the minimum interval.
pub fn dhcp6_success_renew_interval_secs(lease_time_secs: u32) -> u64 {
    if lease_time_secs == 0 {
        DHCP6_MIN_RENEW_SECS
    } else {
        core::cmp::max(u64::from(lease_time_secs) / 2, DHCP6_MIN_RENEW_SECS)
    }
}

/// Compute Talos' retry interval after a DHCPv6 renew failure.
pub fn dhcp6_failure_retry_interval_secs(previous_interval_secs: u64) -> u64 {
    core::cmp::max(previous_interval_secs / 2, DHCP6_MIN_RENEW_SECS)
}

impl OperatorSpec {
    /// Translate a parsed DHCPv6 lease into operator-layer output using this
    /// operator spec's hostname policy.
    pub fn apply_dhcp6_lease_from_spec(&self, lease: &Dhcp6Lease) -> Result<OperatorOutput> {
        self.apply_dhcp6_lease(lease, self.uses_hostname())
    }

    /// Translate a parsed DHCPv6 lease into operator-layer output.
    pub fn apply_dhcp6_lease(
        &self,
        lease: &Dhcp6Lease,
        use_hostname: bool,
    ) -> Result<OperatorOutput> {
        let mut out = OperatorOutput::default();

        if let Some(address) = &lease.address {
            out.addresses.push(AddressSpec::new(
                address.address,
                128,
                self.link_name.clone(),
                ConfigLayer::Operator,
            )?);
        }

        if !lease.dns_servers.is_empty() {
            out.resolver = Some(ResolverSpec::new(
                lease.dns_servers.clone(),
                ConfigLayer::Operator,
            )?);
        }

        if use_hostname && let Some(fqdn) = &lease.fqdn {
            out.hostname = Some(hostname_from_fqdn(fqdn)?);
        }

        out.time_servers = lease.ntp_servers.iter().map(ToString::to_string).collect();

        Ok(out)
    }
}

#[derive(Debug, Clone, Copy)]
struct Dhcp6Option<'a> {
    code: u16,
    data: &'a [u8],
}

fn parse_options<'a>(mut input: &'a [u8], context: &str) -> Result<Vec<Dhcp6Option<'a>>> {
    let mut options = Vec::new();
    while !input.is_empty() {
        if input.len() < 4 {
            return Err(Error::parse(alloc::format!(
                "truncated DHCPv6 option header in {context}"
            )));
        }
        let code = u16::from_be_bytes([input[0], input[1]]);
        let len = usize::from(u16::from_be_bytes([input[2], input[3]]));
        input = &input[4..];
        if input.len() < len {
            return Err(Error::parse(alloc::format!(
                "truncated DHCPv6 option {code} in {context}"
            )));
        }
        let (data, rest) = input.split_at(len);
        options.push(Dhcp6Option { code, data });
        input = rest;
    }
    Ok(options)
}

fn option_data<'a>(options: &'a [Dhcp6Option<'a>], code: u16) -> Option<&'a [u8]> {
    options
        .iter()
        .find(|option| option.code == code)
        .map(|option| option.data)
}

fn first_ia_na_iaid(options: &[Dhcp6Option<'_>]) -> Result<Option<u32>> {
    let Some(ia_na) = option_data(options, DHCP6_OPTION_IA_NA) else {
        return Ok(None);
    };
    if ia_na.len() < 12 {
        return Err(Error::parse(
            "DHCPv6 IA_NA option shorter than fixed header",
        ));
    }
    Ok(Some(u32::from_be_bytes([
        ia_na[0], ia_na[1], ia_na[2], ia_na[3],
    ])))
}

fn parse_ia_na(data: &[u8]) -> Result<Option<Dhcp6IaAddress>> {
    if data.len() < 12 {
        return Err(Error::parse(
            "DHCPv6 IA_NA option shorter than fixed header",
        ));
    }
    let suboptions = parse_options(&data[12..], "IA_NA")?;
    suboptions
        .iter()
        .filter(|option| option.code == DHCP6_OPTION_IA_ADDRESS)
        .find_map(|option| parse_ia_address(option.data).transpose())
        .transpose()
}

fn parse_ia_address(data: &[u8]) -> Result<Option<Dhcp6IaAddress>> {
    if data.len() < 24 {
        return Err(Error::parse("DHCPv6 IAADDR option shorter than fixed body"));
    }
    let address = v6_from_bytes(&data[..16]);
    let preferred_lifetime_secs = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let valid_lifetime_secs = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    Ok(Some(Dhcp6IaAddress {
        address,
        preferred_lifetime_secs,
        valid_lifetime_secs,
    }))
}

fn parse_ipv6_list(data: &[u8], name: &str) -> Result<Vec<NodeAddress>> {
    if !data.len().is_multiple_of(16) {
        return Err(Error::parse(alloc::format!(
            "DHCPv6 {name} option length is not a multiple of 16"
        )));
    }
    Ok(data.chunks(16).map(v6_from_bytes).collect())
}

fn parse_ntp_servers(data: &[u8]) -> Result<Vec<NodeAddress>> {
    let options = parse_options(data, "NTP server")?;
    let mut out = Vec::new();
    for option in options {
        if option.code == DHCP6_NTP_SUBOPTION_SERVER_ADDR {
            out.extend(parse_ipv6_list(option.data, "ntp server address")?);
        }
    }
    Ok(out)
}

fn parse_fqdn(data: &[u8]) -> Result<Option<String>> {
    if data.is_empty() {
        return Ok(None);
    }
    let labels = parse_domain_name(&data[1..])?;
    if labels.is_empty() {
        Ok(None)
    } else {
        Ok(Some(labels.join(".")))
    }
}

fn parse_domain_name(data: &[u8]) -> Result<Vec<String>> {
    Ok(parse_domain_name_with_len(data)?.0)
}

fn parse_domain_name_with_len(mut data: &[u8]) -> Result<(Vec<String>, usize)> {
    let original_len = data.len();
    let mut labels = Vec::new();
    loop {
        if data.is_empty() {
            return Err(Error::parse("truncated DHCPv6 domain name"));
        }
        let len = usize::from(data[0]);
        data = &data[1..];
        if len == 0 {
            break;
        }
        if len & 0xc0 != 0 {
            return Err(Error::parse(
                "compressed DHCPv6 domain names are not supported",
            ));
        }
        if data.len() < len {
            return Err(Error::parse("truncated DHCPv6 domain label"));
        }
        let label = core::str::from_utf8(&data[..len])
            .map_err(|_| Error::parse("DHCPv6 domain label is not UTF-8"))?;
        labels.push(label.to_ascii_lowercase());
        data = &data[len..];
    }
    Ok((labels, original_len - data.len()))
}

fn hostname_from_fqdn(fqdn: &str) -> Result<HostnameSpec> {
    match fqdn.split_once('.') {
        Some((host, domain)) => HostnameSpec::with_domain(host, domain, ConfigLayer::Operator),
        None => HostnameSpec::new(fqdn.to_string(), ConfigLayer::Operator),
    }
}

fn v6_from_bytes(bytes: &[u8]) -> NodeAddress {
    let mut groups = [0u16; 8];
    for (idx, chunk) in bytes.chunks_exact(2).enumerate() {
        groups[idx] = u16::from_be_bytes([chunk[0], chunk[1]]);
    }
    NodeAddress::V6(groups)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn opt(code: u16, data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&code.to_be_bytes());
        out.extend_from_slice(&(data.len() as u16).to_be_bytes());
        out.extend_from_slice(data);
        out
    }

    fn v6_bytes(groups: [u16; 8]) -> Vec<u8> {
        let mut out = Vec::new();
        for group in groups {
            out.extend_from_slice(&group.to_be_bytes());
        }
        out
    }

    fn labels(parts: &[&str]) -> Vec<u8> {
        let mut out = Vec::new();
        for part in parts {
            out.push(part.len() as u8);
            out.extend_from_slice(part.as_bytes());
        }
        out.push(0);
        out
    }

    fn wire_duid() -> Vec<u8> {
        vec![0x00, 0x03, 0x00, 0x01, 0x02, 0x00, 0x5e, 0x10, 0x20, 0x30]
    }

    fn server_duid() -> Vec<u8> {
        vec![0x00, 0x02, 0x00, 0x01, 0xaa, 0xbb, 0xcc, 0xdd]
    }

    fn wire_config() -> Dhcp6ClientConfig {
        Dhcp6ClientConfig::new(
            [0x10, 0x20, 0x30],
            0x5e10_2030,
            Dhcp6ClientIdentifier::Duid(wire_duid()),
        )
        .with_request_transaction_id([0x40, 0x50, 0x60])
    }

    #[test]
    fn dhcp6_client_identifier_can_match_upstream_default_duid_llt() {
        let config = Dhcp6ClientConfig::new(
            [0x10, 0x20, 0x30],
            0x5e10_2030,
            Dhcp6ClientIdentifier::DuidLlt {
                mac: [0x02, 0x00, 0x5e, 0x10, 0x20, 0x30],
                seconds_since_2000: 0x0102_0304,
            },
        );

        assert_eq!(
            config.client_duid().unwrap().unwrap(),
            vec![
                0x00, 0x01, // DUID-LLT
                0x00, 0x01, // Ethernet
                0x01, 0x02, 0x03, 0x04, // time since 2000-01-01 UTC
                0x02, 0x00, 0x5e, 0x10, 0x20, 0x30,
            ]
        );
    }

    fn top_options(packet: &[u8]) -> Vec<Dhcp6Option<'_>> {
        parse_options(&packet[4..], "test").unwrap()
    }

    fn top_option(packet: &[u8], code: u16) -> Vec<u8> {
        let options = top_options(packet);
        options
            .iter()
            .find(|option| option.code == code)
            .map(|option| option.data.to_vec())
            .unwrap_or_else(|| panic!("missing option {code}"))
    }

    fn ia_na_body(iaid: u32) -> Vec<u8> {
        let mut ia_na = Vec::new();
        ia_na.extend_from_slice(&iaid.to_be_bytes());
        ia_na.extend_from_slice(&0u32.to_be_bytes());
        ia_na.extend_from_slice(&0u32.to_be_bytes());
        ia_na
    }

    fn advertise_packet(iaid: u32) -> Vec<u8> {
        let mut packet = vec![DHCP6_MESSAGE_ADVERTISE, 0x10, 0x20, 0x30];
        packet.extend_from_slice(&opt(DHCP6_OPTION_CLIENT_ID, &wire_duid()));
        packet.extend_from_slice(&opt(DHCP6_OPTION_SERVER_ID, &server_duid()));
        packet.extend_from_slice(&opt(DHCP6_OPTION_IA_NA, &ia_na_body(iaid)));
        packet
    }

    fn wire_reply(
        transaction_id: [u8; 3],
        client_duid: &[u8],
        server_duid: &[u8],
        iaid: u32,
    ) -> Vec<u8> {
        let mut packet = vec![
            DHCP6_MESSAGE_REPLY,
            transaction_id[0],
            transaction_id[1],
            transaction_id[2],
        ];
        packet.extend_from_slice(&opt(DHCP6_OPTION_CLIENT_ID, client_duid));
        packet.extend_from_slice(&opt(DHCP6_OPTION_SERVER_ID, server_duid));

        let mut ia_addr = v6_bytes([0x2001, 0x0db8, 0, 0, 0, 0, 0, 0x0020]);
        ia_addr.extend_from_slice(&30u32.to_be_bytes());
        ia_addr.extend_from_slice(&60u32.to_be_bytes());

        let mut ia_na = ia_na_body(iaid);
        ia_na.extend_from_slice(&opt(DHCP6_OPTION_IA_ADDRESS, &ia_addr));
        packet.extend_from_slice(&opt(DHCP6_OPTION_IA_NA, &ia_na));
        packet
    }

    fn sample_reply() -> Vec<u8> {
        let mut packet = vec![DHCP6_MESSAGE_REPLY, 0x01, 0x02, 0x03];

        let mut ia_addr = v6_bytes([0x2001, 0x0db8, 0, 0, 0, 0, 0, 0x0010]);
        ia_addr.extend_from_slice(&300u32.to_be_bytes());
        ia_addr.extend_from_slice(&600u32.to_be_bytes());

        let mut ia_na = Vec::new();
        ia_na.extend_from_slice(&1u32.to_be_bytes());
        ia_na.extend_from_slice(&60u32.to_be_bytes());
        ia_na.extend_from_slice(&120u32.to_be_bytes());
        ia_na.extend_from_slice(&opt(DHCP6_OPTION_IA_ADDRESS, &ia_addr));
        packet.extend_from_slice(&opt(DHCP6_OPTION_IA_NA, &ia_na));

        let mut dns = v6_bytes([0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111]);
        dns.extend_from_slice(&v6_bytes([0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1001]));
        packet.extend_from_slice(&opt(DHCP6_OPTION_DNS_SERVERS, &dns));

        let mut search = labels(&["svc", "example", "com"]);
        search.extend_from_slice(&labels(&["cluster", "local"]));
        packet.extend_from_slice(&opt(DHCP6_OPTION_DOMAIN_SEARCH, &search));

        let mut fqdn = vec![0u8];
        fqdn.extend_from_slice(&labels(&["node", "example", "com"]));
        packet.extend_from_slice(&opt(DHCP6_OPTION_FQDN, &fqdn));

        let ntp_addr = v6_bytes([0xfd00, 0x0ec2, 0, 0, 0, 0, 0, 0x0123]);
        let ntp = opt(DHCP6_NTP_SUBOPTION_SERVER_ADDR, &ntp_addr);
        packet.extend_from_slice(&opt(DHCP6_OPTION_NTP_SERVER, &ntp));

        packet
    }

    #[test]
    fn parses_reply_address_dns_fqdn_and_ntp() {
        let lease = parse_dhcp6_reply(&sample_reply()).unwrap();
        assert_eq!(lease.transaction_id, [0x01, 0x02, 0x03]);
        let address = lease.address.unwrap();
        assert_eq!(address.address.to_string(), "2001:db8:0:0:0:0:0:10");
        assert_eq!(address.preferred_lifetime_secs, 300);
        assert_eq!(address.valid_lifetime_secs, 600);
        assert_eq!(
            lease
                .dns_servers
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec![
                "2606:4700:4700:0:0:0:0:1111".to_string(),
                "2606:4700:4700:0:0:0:0:1001".to_string(),
            ]
        );
        assert_eq!(lease.fqdn.as_deref(), Some("node.example.com"));
        assert_eq!(
            lease
                .ntp_servers
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["fd00:ec2:0:0:0:0:0:123".to_string()]
        );
    }

    #[test]
    fn applies_dhcp6_reply_to_operator_output() {
        let lease = parse_dhcp6_reply(&sample_reply()).unwrap();
        let output = OperatorSpec::dhcp6("eth0")
            .apply_dhcp6_lease(&lease, true)
            .unwrap();

        assert_eq!(output.addresses.len(), 1);
        assert_eq!(output.addresses[0].id(), "eth0/2001:db8:0:0:0:0:0:10/128");
        assert!(output.routes.is_empty());
        let resolver = output.resolver.as_ref().unwrap();
        assert_eq!(
            resolver
                .servers
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec![
                "2606:4700:4700:0:0:0:0:1111".to_string(),
                "2606:4700:4700:0:0:0:0:1001".to_string(),
            ]
        );
        assert!(resolver.search_domains.is_empty());
        assert_eq!(
            output.time_servers,
            vec!["fd00:ec2:0:0:0:0:0:123".to_string()]
        );
        let hostname = output.hostname.unwrap();
        assert_eq!(hostname.hostname.as_str(), "node");
        assert_eq!(hostname.domainname.as_deref(), Some("example.com"));
    }

    #[test]
    fn ignores_dhcp6_domain_search_without_dns_servers_for_talos_parity() {
        let mut packet = vec![DHCP6_MESSAGE_REPLY, 0x22, 0x33, 0x44];
        let mut search = labels(&["svc", "example", "com"]);
        search.extend_from_slice(&labels(&["svc", "example", "com"]));
        search.extend_from_slice(&labels(&["node", "local"]));
        packet.extend_from_slice(&opt(DHCP6_OPTION_DOMAIN_SEARCH, &search));

        let lease = parse_dhcp6_reply(&packet).unwrap();
        assert!(lease.dns_servers.is_empty());
        let output = OperatorSpec::dhcp6("eth0")
            .apply_dhcp6_lease(&lease, true)
            .unwrap();
        assert!(output.resolver.is_none());
        assert!(output.routes.is_empty());
    }

    #[test]
    fn skips_hostname_when_requested() {
        let lease = parse_dhcp6_reply(&sample_reply()).unwrap();
        let output = OperatorSpec::dhcp6("eth0")
            .apply_dhcp6_lease(&lease, false)
            .unwrap();
        assert!(output.hostname.is_none());
    }

    #[test]
    fn applies_dhcp6_reply_using_operator_skip_hostname_policy() {
        let lease = parse_dhcp6_reply(&sample_reply()).unwrap();
        let output = OperatorSpec::dhcp6("eth0")
            .with_skip_hostname_request(true)
            .apply_dhcp6_lease_from_spec(&lease)
            .unwrap();
        assert!(output.hostname.is_none());
        assert_eq!(
            output.time_servers,
            vec!["fd00:ec2:0:0:0:0:0:123".to_string()]
        );
    }

    #[test]
    fn rejects_malformed_reply() {
        assert!(parse_dhcp6_reply(&[DHCP6_MESSAGE_REPLY, 1, 2]).is_err());
        assert!(parse_dhcp6_reply(&[2, 1, 2, 3]).is_err());
        let mut bad_dns = vec![DHCP6_MESSAGE_REPLY, 1, 2, 3];
        bad_dns.extend_from_slice(&opt(DHCP6_OPTION_DNS_SERVERS, &[1, 2, 3]));
        assert!(parse_dhcp6_reply(&bad_dns).is_err());
    }

    #[test]
    fn derives_renewal_intervals_from_valid_lifetime() {
        let lease = parse_dhcp6_reply(&sample_reply()).unwrap();
        assert_eq!(lease.lease_time_secs(), 600);
        assert_eq!(dhcp6_success_renew_interval_secs(600), 300);
        assert_eq!(dhcp6_success_renew_interval_secs(9), 5);
        assert_eq!(dhcp6_success_renew_interval_secs(0), 5);
        assert_eq!(dhcp6_failure_retry_interval_secs(300), 150);
        assert_eq!(dhcp6_failure_retry_interval_secs(9), 5);
    }

    #[test]
    fn dhcp6_wire_builds_rapid_solicit_with_identity_and_requested_options() {
        let packet = build_dhcp6_rapid_solicit(&wire_config(), 0).unwrap();

        assert_eq!(&packet[..4], &[DHCP6_MESSAGE_SOLICIT, 0x10, 0x20, 0x30]);
        assert_eq!(top_option(&packet, DHCP6_OPTION_CLIENT_ID), wire_duid());
        assert_eq!(
            top_option(&packet, DHCP6_OPTION_ORO),
            [
                0x00,
                DHCP6_OPTION_DNS_SERVERS as u8,
                0x00,
                DHCP6_OPTION_DOMAIN_SEARCH as u8
            ]
        );
        assert_eq!(top_option(&packet, DHCP6_OPTION_ELAPSED_TIME), [0x00, 0x00]);
        assert_eq!(
            top_option(&packet, DHCP6_OPTION_IA_NA),
            ia_na_body(0x5e10_2030)
        );
        assert_eq!(top_option(&packet, DHCP6_OPTION_RAPID_COMMIT), []);
    }

    #[test]
    fn dhcp6_wire_retries_use_same_transaction_id_and_elapsed_time() {
        let mut tx = Dhcp6WireTransaction::new(wire_config());
        let first = tx.start().unwrap();
        let retry = tx.retry(0).unwrap();

        assert_eq!(
            &first.packet[..4],
            &[DHCP6_MESSAGE_SOLICIT, 0x10, 0x20, 0x30]
        );
        assert_eq!(
            &retry.packet[..4],
            &[DHCP6_MESSAGE_SOLICIT, 0x10, 0x20, 0x30]
        );
        assert_eq!(
            top_option(&retry.packet, DHCP6_OPTION_ELAPSED_TIME),
            0u16.to_be_bytes()
        );
    }

    #[test]
    fn dhcp6_wire_accepts_matching_rapid_commit_reply() {
        let mut tx = Dhcp6WireTransaction::new(wire_config());
        tx.start().unwrap();
        let reply = wire_reply(
            [0x10, 0x20, 0x30],
            &wire_duid(),
            &server_duid(),
            0x5e10_2030,
        );

        let action = tx.handle_packet(&reply).unwrap();
        let Dhcp6WireAction::Bound(lease) = action else {
            panic!("expected bound action, got {action:?}");
        };
        assert_eq!(tx.state(), Dhcp6WireState::Bound);
        assert_eq!(lease.transaction_id, [0x10, 0x20, 0x30]);
        assert_eq!(lease.lease_time_secs(), 60);
    }

    #[test]
    fn dhcp6_wire_builds_request_from_non_rapid_advertise() {
        let mut tx = Dhcp6WireTransaction::new(wire_config());
        tx.start().unwrap();

        let action = tx.handle_packet(&advertise_packet(0x5e10_2030)).unwrap();
        let Dhcp6WireAction::Send(request) = action else {
            panic!("expected request action, got {action:?}");
        };

        assert_eq!(tx.state(), Dhcp6WireState::Requesting);
        assert_eq!(
            &request.packet[..4],
            &[DHCP6_MESSAGE_REQUEST, 0x40, 0x50, 0x60]
        );
        assert_eq!(
            top_option(&request.packet, DHCP6_OPTION_CLIENT_ID),
            wire_duid()
        );
        assert_eq!(
            top_option(&request.packet, DHCP6_OPTION_SERVER_ID),
            server_duid()
        );
        assert_eq!(
            top_option(&request.packet, DHCP6_OPTION_ELAPSED_TIME),
            [0, 0]
        );
        assert_eq!(
            top_option(&request.packet, DHCP6_OPTION_IA_NA),
            ia_na_body(0x5e10_2030)
        );
        assert_eq!(
            top_option(&request.packet, DHCP6_OPTION_ORO),
            [
                0x00,
                DHCP6_OPTION_DNS_SERVERS as u8,
                0x00,
                DHCP6_OPTION_DOMAIN_SEARCH as u8
            ]
        );
    }

    #[test]
    fn dhcp6_wire_binds_after_non_rapid_request_reply() {
        let mut tx = Dhcp6WireTransaction::new(wire_config());
        tx.start().unwrap();
        tx.handle_packet(&advertise_packet(0x5e10_2030)).unwrap();

        let reply = wire_reply(
            [0x40, 0x50, 0x60],
            &wire_duid(),
            &server_duid(),
            0x5e10_2030,
        );
        let action = tx.handle_packet(&reply).unwrap();
        let Dhcp6WireAction::Bound(lease) = action else {
            panic!("expected bound action, got {action:?}");
        };
        assert_eq!(lease.transaction_id, [0x40, 0x50, 0x60]);
    }

    #[test]
    fn dhcp6_wire_ignores_reply_with_wrong_transaction_id() {
        let mut tx = Dhcp6WireTransaction::new(wire_config());
        tx.start().unwrap();
        let reply = wire_reply(
            [0x99, 0x20, 0x30],
            &wire_duid(),
            &server_duid(),
            0x5e10_2030,
        );

        assert_eq!(
            tx.handle_packet(&reply).unwrap(),
            Dhcp6WireAction::Ignored("not matching reply")
        );
        assert_eq!(tx.state(), Dhcp6WireState::Soliciting);
    }

    #[test]
    fn dhcp6_wire_requires_matching_client_id_and_iaid() {
        let mut tx = Dhcp6WireTransaction::new(wire_config());
        tx.start().unwrap();
        let wrong_client = wire_reply(
            [0x10, 0x20, 0x30],
            &[0x00, 0x03, 0x00, 0x01, 1, 2, 3, 4, 5, 6],
            &server_duid(),
            0x5e10_2030,
        );
        let wrong_iaid = wire_reply(
            [0x10, 0x20, 0x30],
            &wire_duid(),
            &server_duid(),
            0x0102_0304,
        );

        assert_eq!(
            tx.handle_packet(&wrong_client).unwrap(),
            Dhcp6WireAction::Ignored("not matching reply")
        );
        assert_eq!(
            tx.handle_packet(&wrong_iaid).unwrap(),
            Dhcp6WireAction::Ignored("not matching reply")
        );
        assert_eq!(tx.state(), Dhcp6WireState::Soliciting);
    }
}
