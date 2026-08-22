//! Differential tests: operating-system Rust port vs. the Go oracle vectors.
//!
//! Each TSV file under `vectors/` was written by the authoritative Go oracle
//! (`talos-reference/_oracle/main.go`) from the REAL Talos machinery packages.
//! For every record we call the corresponding Rust function and assert the Rust
//! output equals the oracle column byte-for-byte.
//!
//! TSV format (see oracle): tab-separated, one record per line, LF endings,
//! trailing LF on the last line, no header, no quoting/escaping. Some value
//! columns intentionally carry leading/trailing spaces and MUST NOT be trimmed
//! by this parser. Record kinds are discriminated by column 1.
//!
//! One `#[test]` per TSV file. On any mismatch the test fails, printing the
//! input, the expected (Go) value, and the actual (Rust) value.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use os_machine_config_domain::corpus::{CorpusConfig, Validity, load_record};

use os_kernel::NodeAddress;
use os_kernel::machine_type::MachineType;
use os_kernel::role::{Role, RoleSet};
use os_kernel::version::build as version_build;
use os_kubernetes_domain::secrets::REQUIRED_SECRETS;
use os_kubernetes_domain::{
    ClusterEndpoint, ControlPlaneConfig, FileMode, K8sComponent, K8sConfig, K8sSecrets, NodeName,
    StaticPod, StaticPodPhase, kubeconfig_data, launch_static_pods_on_cri, provision_control_plane,
    static_pod::Container as KubernetesContainer,
};
use os_network_domain::nethelpers::{
    AdLacpActive, AdSelect, AddressFlag, AddressSortAlgorithm, ArpAllTargets, ArpValidate,
    AutoHostnameKind, BondMode, BondXmitHashPolicy, ClientIdentifier, ConntrackState,
    DefaultAction, DnsProtocol, Duplex, FailOverMac, Family, LacpRate, LinkFlag, LinkType,
    MatchOperator, NfTablesChainHook, NfTablesChainPriority, NfTablesVerdict, OperationalState,
    Port, PrimaryReselect, Protocol, RouteFlag, RouteProtocol, RouteType, RoutingRuleAction,
    RoutingTable, Scope, Status, VlanProtocol, WolMode,
};
use os_network_domain::{
    ClientIdentifierSpec, Dhcp6ClientConfig, Dhcp6ClientIdentifier, Dhcp6SendTarget, LinkStatus,
    LinkType as NetworkLinkType, OperState, OperatorKind, OperatorSpec,
    RouteProtocol as NetworkRouteProtocol, build_dhcp6_rapid_solicit,
    build_dhcp6_request_with_ia_na, parse_dhcp4_ack, parse_dhcp6_reply,
};
use os_platform_domain::metal_oauth::{OAuthConfigError, new_config as new_metal_oauth_config};
use os_platform_domain::metal_url::{UrlVariableValues, populate_url, required_variables};
use os_runtime_cri_domain::{CriContainerState, CriRuntime, PodSandboxState, RuntimeService};
use os_secrets_domain::certsans::CertSans;
use os_secrets_domain::etcd::{EtcdCert, EtcdController};
use os_secrets_domain::kubernetes::{K8sCert as SecretK8sCert, KubernetesController};
use os_secrets_domain::{
    CaKind, KUBERNETES_SECRET_PROJECTION_NAMES, SecretsBundle, kubernetes_secret_entries,
};

/// Root of the oracle fixtures. Under cargo, CARGO_MANIFEST_DIR points at the
/// crate dir; under buck2 the rust_test target injects OYATIE_TESTDATA_DIR via
/// `$(location :testdata)` (CARGO_MANIFEST_DIR is not defined there).
fn fixtures_root() -> PathBuf {
    if let Ok(dir) = std::env::var("OYATIE_TESTDATA_DIR") {
        return PathBuf::from(dir);
    }
    PathBuf::from(option_env!("CARGO_MANIFEST_DIR").unwrap_or("."))
}

/// Absolute path to a vector file inside this crate's `vectors/` directory.
fn vector_path(name: &str) -> PathBuf {
    let mut p = fixtures_root();
    p.push("vectors");
    p.push(name);
    p
}

/// Read a vector file as a UTF-8 string (preserving all whitespace exactly).
fn read_vector(name: &str) -> String {
    let path = vector_path(name);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read vector {}: {e}", path.display()))
}

/// Split a TSV body into records.
///
/// The oracle writes a trailing LF on the last line; splitting on `\n` would
/// then yield a final empty element, which we drop. We deliberately do NOT trim
/// the individual lines or fields: some value columns carry significant
/// leading/trailing spaces.
fn records(body: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = body.split('\n').collect();
    // Drop the single trailing empty element produced by the trailing LF.
    if let Some(last) = lines.last()
        && last.is_empty()
    {
        lines.pop();
    }
    lines
}

/// Whether `role_string` is a member of upstream `role.All`.
///
/// Upstream computes `role.All.Includes(role.Role(roleString))`, i.e. exact
/// string membership of a raw role string in the set of canonical role strings.
/// In the Rust port, `RoleSet::all()` holds the known `Role` variants whose
/// canonical certificate string is `Role::as_ou()`. A raw string is therefore a
/// member iff it equals the canonical OU of one of the roles in `all()`.
fn role_all_includes(role_string: &str) -> bool {
    let all = RoleSet::all();

    all.iter().any(|r: Role| r.as_ou() == role_string)
}

// ---------------------------------------------------------------------------
// platform_metal_oauth.tsv
// ---------------------------------------------------------------------------

#[test]
fn platform_metal_oauth_tsv() {
    let body = read_vector("platform_metal_oauth.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        match cols[0] {
            "absent" => {
                assert_eq!(
                    cols.len(),
                    3,
                    "platform_metal_oauth.tsv line {}: expected 3 columns in absent record, got {}: {:?}",
                    lineno + 1,
                    cols.len(),
                    line
                );

                let actual = new_metal_oauth_config(cols[1], cols[2]).unwrap_err();
                assert_eq!(
                    actual,
                    OAuthConfigError::NotConfigured,
                    "metal OAuth absent mismatch\n cmdline  = {:?}\n expected = NotConfigured (Go os.ErrNotExist)\n actual   = {:?} (Rust)",
                    cols[1],
                    actual
                );
                checked += 1;
            }
            "config" => {
                assert_eq!(
                    cols.len(),
                    10,
                    "platform_metal_oauth.tsv line {}: expected 10 columns in config record, got {}: {:?}",
                    lineno + 1,
                    cols.len(),
                    line
                );

                let actual = new_metal_oauth_config(cols[1], cols[2]).unwrap();
                assert_eq!(
                    actual.client_id,
                    cols[3],
                    "metal OAuth client_id mismatch on line {}",
                    lineno + 1
                );
                assert_eq!(
                    actual.client_secret,
                    cols[4],
                    "metal OAuth client_secret mismatch on line {}",
                    lineno + 1
                );
                assert_eq!(
                    actual.audience,
                    cols[5],
                    "metal OAuth audience mismatch on line {}",
                    lineno + 1
                );
                assert_eq!(
                    actual.scopes.join(","),
                    cols[6],
                    "metal OAuth scopes mismatch on line {}",
                    lineno + 1
                );
                assert_eq!(
                    actual.extra_variables.join(","),
                    cols[7],
                    "metal OAuth extra_variables mismatch on line {}",
                    lineno + 1
                );
                assert_eq!(
                    actual.device_auth_url,
                    cols[8],
                    "metal OAuth device_auth_url mismatch on line {}",
                    lineno + 1
                );
                assert_eq!(
                    actual.token_url,
                    cols[9],
                    "metal OAuth token_url mismatch on line {}",
                    lineno + 1
                );
                checked += 1;
            }
            other => panic!(
                "platform_metal_oauth.tsv line {}: unknown record kind {:?}",
                lineno + 1,
                other
            ),
        }
    }

    assert_eq!(
        checked, 4,
        "platform_metal_oauth.tsv: expected 4 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// platform_metal_url.tsv
// ---------------------------------------------------------------------------

#[test]
fn platform_metal_url_tsv() {
    let body = read_vector("platform_metal_url.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        match cols[0] {
            "required" => {
                assert_eq!(
                    cols.len(),
                    3,
                    "platform_metal_url.tsv line {}: expected 3 columns in required record, got {}: {:?}",
                    lineno + 1,
                    cols.len(),
                    line
                );

                let actual = required_variables(cols[1]).join(",");
                assert_eq!(
                    actual, cols[2],
                    "metal URL required variable mismatch\n input    = {:?}\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
                    cols[1], cols[2], actual
                );
                checked += 1;
            }
            "populate" => {
                assert_eq!(
                    cols.len(),
                    8,
                    "platform_metal_url.tsv line {}: expected 8 columns in populate record, got {}: {:?}",
                    lineno + 1,
                    cols.len(),
                    line
                );

                let mut values = UrlVariableValues::new();
                if !cols[2].is_empty() {
                    values = values.with_uuid(cols[2]);
                }
                if !cols[3].is_empty() {
                    values = values.with_serial(cols[3]);
                }
                if !cols[4].is_empty() {
                    values = values.with_mac(cols[4]);
                }
                if !cols[5].is_empty() {
                    values = values.with_hostname(cols[5]);
                }
                if !cols[6].is_empty() {
                    values = values.with_code(cols[6]);
                }

                let actual = populate_url(cols[1], &values).unwrap_or_else(|err| err.to_string());
                assert_eq!(
                    actual, cols[7],
                    "metal URL populate mismatch\n input    = {:?}\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
                    cols[1], cols[7], actual
                );
                checked += 1;
            }
            other => panic!(
                "platform_metal_url.tsv line {}: unknown record kind {:?}",
                lineno + 1,
                other
            ),
        }
    }

    assert_eq!(
        checked, 10,
        "platform_metal_url.tsv: expected 10 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// dhcp4_ack.tsv
// ---------------------------------------------------------------------------

fn decode_hex(s: &str) -> Vec<u8> {
    assert!(
        s.len().is_multiple_of(2),
        "hex string must have an even number of characters"
    );
    (0..s.len())
        .step_by(2)
        .map(|idx| {
            u8::from_str_radix(&s[idx..idx + 2], 16)
                .unwrap_or_else(|err| panic!("invalid hex at offset {idx}: {err}"))
        })
        .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn route_protocol_name(protocol: NetworkRouteProtocol) -> &'static str {
    match protocol {
        NetworkRouteProtocol::Static => "static",
        NetworkRouteProtocol::Boot => "boot",
        NetworkRouteProtocol::Kernel => "kernel",
        NetworkRouteProtocol::Dhcp => "dhcp",
    }
}

fn dhcp4_ack_summary(packet_hex: &str, use_hostname: bool) -> String {
    let packet = decode_hex(packet_hex);
    let lease = parse_dhcp4_ack(&packet).unwrap();
    let output = OperatorSpec::dhcp4("eth0")
        .apply_dhcp4_lease(&lease, use_hostname)
        .unwrap();

    let address = output
        .addresses
        .first()
        .map(|addr| format!("{}/{}", addr.address, addr.prefix_len))
        .unwrap_or_default();

    let routes = output
        .routes
        .iter()
        .map(|route| {
            let destination = match route.destination {
                Some(dest) => format!("{}/{}", dest, route.prefix_len),
                None => format!("default/{}", route.prefix_len),
            };
            let next_hop = match route.gateway {
                Some(gateway) => format!("via {gateway}"),
                None => "onlink".to_string(),
            };
            format!(
                "{destination} {next_hop} metric {} proto {}",
                route.metric,
                route_protocol_name(route.protocol)
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let (dns, search) = match output.resolver {
        Some(resolver) => (
            resolver
                .servers
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
            resolver.search_domains.join(","),
        ),
        None => (String::new(), String::new()),
    };

    let hostname = output
        .hostname
        .as_ref()
        .map(|hostname| hostname.fqdn())
        .unwrap_or_default();
    let mtu = lease.mtu.map(|mtu| mtu.to_string()).unwrap_or_default();
    let ntp = output.time_servers.join(",");

    format!(
        "addr={address};routes={routes};dns={dns};search={search};hostname={hostname};mtu={mtu};ntp={ntp}"
    )
}

#[test]
fn dhcp4_ack_tsv() {
    let body = read_vector("dhcp4_ack.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            4,
            "dhcp4_ack.tsv line {}: expected 4 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let use_hostname = match cols[1] {
            "true" => true,
            "false" => false,
            other => panic!(
                "dhcp4_ack.tsv line {}: invalid use_hostname {other:?}",
                lineno + 1
            ),
        };
        let actual = dhcp4_ack_summary(cols[2], use_hostname);
        assert_eq!(
            actual, cols[3],
            "DHCPv4 ACK parity mismatch for case {}\n expected = {:?} (Go/Talos semantics vector)\n actual   = {:?} (Rust)",
            cols[0], cols[3], actual
        );
        checked += 1;
    }

    assert_eq!(
        checked, 4,
        "dhcp4_ack.tsv: expected 4 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// dhcp6_reply.tsv
// ---------------------------------------------------------------------------

fn dhcp6_reply_summary(packet_hex: &str, use_hostname: bool) -> String {
    let packet = decode_hex(packet_hex);
    let lease = parse_dhcp6_reply(&packet).unwrap();
    let output = OperatorSpec::dhcp6("eth0")
        .apply_dhcp6_lease(&lease, use_hostname)
        .unwrap();

    let address = output
        .addresses
        .first()
        .map(|addr| format!("{}/{}", addr.address, addr.prefix_len))
        .unwrap_or_default();

    let dns = output
        .resolver
        .as_ref()
        .map(|resolver| {
            resolver
                .servers
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();

    let (hostname, domain) = match output.hostname.as_ref() {
        Some(hostname) => (
            hostname.hostname.as_str().to_string(),
            hostname.domainname.clone().unwrap_or_default(),
        ),
        None => (String::new(), String::new()),
    };

    let ntp = output.time_servers.join(",");

    format!(
        "lease={};addr={address};dns={dns};hostname={hostname};domain={domain};ntp={ntp}",
        lease.lease_time_secs()
    )
}

#[test]
fn dhcp6_reply_tsv() {
    let body = read_vector("dhcp6_reply.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            4,
            "dhcp6_reply.tsv line {}: expected 4 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let use_hostname = match cols[1] {
            "true" => true,
            "false" => false,
            other => panic!(
                "dhcp6_reply.tsv line {}: invalid use_hostname {other:?}",
                lineno + 1
            ),
        };
        let actual = dhcp6_reply_summary(cols[2], use_hostname);
        assert_eq!(
            actual, cols[3],
            "DHCPv6 Reply parity mismatch for case {}\n expected = {:?} (Go/Talos semantics vector)\n actual   = {:?} (Rust)",
            cols[0], cols[3], actual
        );
        checked += 1;
    }

    assert_eq!(
        checked, 5,
        "dhcp6_reply.tsv: expected 5 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// dhcp6_wire_outbound.tsv
// ---------------------------------------------------------------------------

fn parse_tx_hex(s: &str) -> [u8; 3] {
    let bytes = decode_hex(s);
    assert_eq!(bytes.len(), 3, "transaction id must be exactly 3 bytes");
    [bytes[0], bytes[1], bytes[2]]
}

fn parse_u32_hex(s: &str) -> u32 {
    let bytes = decode_hex(s);
    assert_eq!(bytes.len(), 4, "u32 hex field must be exactly 4 bytes");
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn dhcp6_wire_options(packet: &[u8]) -> Vec<(u16, Vec<u8>)> {
    assert!(
        packet.len() >= 4,
        "DHCPv6 packet must contain message header"
    );
    let mut input = &packet[4..];
    let mut options = Vec::new();
    while !input.is_empty() {
        assert!(
            input.len() >= 4,
            "DHCPv6 option header truncated in packet {}",
            encode_hex(packet)
        );
        let code = u16::from_be_bytes([input[0], input[1]]);
        let len = usize::from(u16::from_be_bytes([input[2], input[3]]));
        input = &input[4..];
        assert!(
            input.len() >= len,
            "DHCPv6 option {code} truncated in packet {}",
            encode_hex(packet)
        );
        let (data, rest) = input.split_at(len);
        options.push((code, data.to_vec()));
        input = rest;
    }
    options
}

fn dhcp6_wire_option(packet: &[u8], code: u16) -> Vec<u8> {
    dhcp6_wire_options(packet)
        .into_iter()
        .find(|(candidate, _)| *candidate == code)
        .map(|(_, data)| data)
        .unwrap_or_default()
}

fn dhcp6_target_name(target: Dhcp6SendTarget) -> &'static str {
    match target {
        Dhcp6SendTarget::AllDhcpRelayAgentsAndServers => "all-relays-and-servers",
    }
}

fn dhcp6_message_name(message_type: u8) -> &'static str {
    match message_type {
        1 => "solicit",
        3 => "request",
        _ => "unknown",
    }
}

#[allow(clippy::too_many_arguments)] // arity mirrors the oracle TSV column set
fn dhcp6_wire_outbound_summary(
    phase: &str,
    transaction_id_hex: &str,
    request_transaction_id_hex: &str,
    iaid_hex: &str,
    duid_hex: &str,
    elapsed_centisecs: u16,
    server_id_hex: &str,
    ia_na_hex: &str,
) -> (String, String) {
    let config = Dhcp6ClientConfig::new(
        parse_tx_hex(transaction_id_hex),
        parse_u32_hex(iaid_hex),
        Dhcp6ClientIdentifier::Duid(decode_hex(duid_hex)),
    )
    .with_request_transaction_id(parse_tx_hex(request_transaction_id_hex));

    let outbound = match phase {
        "solicit" => build_dhcp6_rapid_solicit(&config, elapsed_centisecs)
            .map(|packet| (packet, Dhcp6SendTarget::AllDhcpRelayAgentsAndServers))
            .unwrap(),
        "request" => build_dhcp6_request_with_ia_na(
            &config,
            &decode_hex(server_id_hex),
            &decode_hex(ia_na_hex),
            elapsed_centisecs,
        )
        .map(|packet| (packet, Dhcp6SendTarget::AllDhcpRelayAgentsAndServers))
        .unwrap(),
        other => panic!("unknown DHCPv6 wire phase {other:?}"),
    };

    let (packet, target) = outbound;
    let options = dhcp6_wire_options(&packet)
        .iter()
        .map(|(code, _)| code.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let elapsed = encode_hex(&dhcp6_wire_option(&packet, 8));
    let ia_na = dhcp6_wire_option(&packet, 3);
    let iaid = encode_hex(&ia_na[..core::cmp::min(ia_na.len(), 4)]);
    let server = encode_hex(&dhcp6_wire_option(&packet, 2));
    let summary = format!(
        "msg={};tx={};target={};options={options};elapsed={elapsed};iaid={iaid};server={server}",
        dhcp6_message_name(packet[0]),
        encode_hex(&packet[1..4]),
        dhcp6_target_name(target)
    );

    (encode_hex(&packet), summary)
}

#[test]
fn dhcp6_wire_outbound_tsv() {
    let body = read_vector("dhcp6_wire_outbound.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            11,
            "dhcp6_wire_outbound.tsv line {}: expected 11 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );
        let elapsed_centisecs = cols[6]
            .parse::<u16>()
            .unwrap_or_else(|err| panic!("line {} invalid elapsed: {err}", lineno + 1));

        let (actual_hex, actual_summary) = dhcp6_wire_outbound_summary(
            cols[1],
            cols[2],
            cols[3],
            cols[4],
            cols[5],
            elapsed_centisecs,
            cols[7],
            cols[8],
        );
        assert_eq!(
            actual_hex, cols[9],
            "DHCPv6 outbound packet mismatch for case {}\n expected = {:?} (Go/Talos semantics vector)\n actual   = {:?} (Rust)",
            cols[0], cols[9], actual_hex
        );
        assert_eq!(
            actual_summary, cols[10],
            "DHCPv6 outbound summary mismatch for case {}\n expected = {:?}\n actual   = {:?}",
            cols[0], cols[10], actual_summary
        );
        checked += 1;
    }

    assert_eq!(
        checked, 3,
        "dhcp6_wire_outbound.tsv: expected 3 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// dhcp6_identity.tsv
// ---------------------------------------------------------------------------

#[test]
fn dhcp6_identity_tsv() {
    let body = read_vector("dhcp6_identity.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            5,
            "dhcp6_identity.tsv line {}: expected 5 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );
        let mac_bytes = decode_hex(cols[1]);
        assert_eq!(mac_bytes.len(), 6, "MAC field must be exactly 6 bytes");
        let mac = [
            mac_bytes[0],
            mac_bytes[1],
            mac_bytes[2],
            mac_bytes[3],
            mac_bytes[4],
            mac_bytes[5],
        ];
        let seconds_since_2000 = parse_u32_hex(cols[2]);
        let expected_iaid = parse_u32_hex(cols[3]);
        let config = Dhcp6ClientConfig::new(
            [0x01, 0x02, 0x03],
            expected_iaid,
            Dhcp6ClientIdentifier::DuidLlt {
                mac,
                seconds_since_2000,
            },
        );
        let expected_duid = decode_hex(cols[4]);

        assert_eq!(
            u32::from_be_bytes([mac[2], mac[3], mac[4], mac[5]]),
            expected_iaid,
            "DHCPv6 IAID parity mismatch for case {}",
            cols[0]
        );
        assert_eq!(
            config.client_duid().unwrap().unwrap(),
            expected_duid,
            "DHCPv6 DUID-LLT parity mismatch for case {}",
            cols[0]
        );
        checked += 1;
    }

    assert_eq!(
        checked, 2,
        "dhcp6_identity.tsv: expected 2 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// dhcp6_operator_config.tsv
// ---------------------------------------------------------------------------

fn parse_mac_hex(s: &str) -> [u8; 6] {
    let bytes = decode_hex(s);
    assert_eq!(bytes.len(), 6, "MAC field must be exactly 6 bytes");
    [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]]
}

fn dhcp6_client_identifier_spec(kind: &str, duid_raw_hex: &str) -> ClientIdentifierSpec {
    match kind {
        "none" => ClientIdentifierSpec::none(),
        "mac" => ClientIdentifierSpec::mac(),
        "duid" => ClientIdentifierSpec::duid(decode_hex(duid_raw_hex)),
        other => panic!("unknown DHCP client identifier kind {other:?}"),
    }
}

#[test]
fn dhcp6_operator_config_tsv() {
    let body = read_vector("dhcp6_operator_config.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            9,
            "dhcp6_operator_config.tsv line {}: expected 9 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let spec = dhcp6_client_identifier_spec(cols[1], cols[2]);
        let mac = parse_mac_hex(cols[3]);
        let seconds_since_2000 = parse_u32_hex(cols[4]);
        let skip_hostname = match cols[5] {
            "true" => true,
            "false" => false,
            other => panic!("invalid skip_hostname value {other:?}"),
        };
        let lease = parse_dhcp6_reply(&decode_hex(cols[6])).unwrap();
        let op = OperatorSpec::dhcp6("eth0")
            .with_client_identifier(spec)
            .with_skip_hostname_request(skip_hostname);
        let identity = op.dhcp6_client_identifier(mac, seconds_since_2000).unwrap();
        let duid = Dhcp6ClientConfig::new([0x01, 0x02, 0x03], 0x5e10_2030, identity)
            .client_duid()
            .unwrap()
            .unwrap();
        let output = op.apply_dhcp6_lease_from_spec(&lease).unwrap();
        let hostname = output
            .hostname
            .map(|host| match host.domainname {
                Some(domain) => format!("{}.{}", host.hostname.as_str(), domain),
                None => host.hostname.to_string(),
            })
            .unwrap_or_default();
        let actual = format!("duid={};hostname={hostname}", encode_hex(&duid));

        assert_eq!(
            actual, cols[7],
            "DHCPv6 operator config mismatch for case {}",
            cols[0]
        );
        assert_eq!(
            op.route_metric.to_string(),
            cols[8],
            "DHCPv6 operator route metric mismatch for case {}",
            cols[0]
        );
        checked += 1;
    }

    assert_eq!(
        checked, 4,
        "dhcp6_operator_config.tsv: expected 4 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// dhcpv4_config_doc_operator.tsv
// ---------------------------------------------------------------------------

fn dhcpv4_config_doc(
    name: &str,
    route_metric: &str,
    ignore_hostname: &str,
    client_identifier: &str,
    duid_raw: &str,
) -> String {
    format!(
        "version: v1alpha1\nmachine:\n  type: worker\n---\napiVersion: v1alpha1\nkind: DHCPv4Config\nname: {name}\n{}{}{}{}",
        optional_doc_scalar("routeMetric", route_metric),
        optional_doc_scalar("ignoreHostname", ignore_hostname),
        optional_doc_scalar("clientIdentifier", client_identifier),
        optional_doc_scalar("duidRaw", duid_raw),
    )
}

#[test]
fn dhcpv4_config_doc_operator_tsv() {
    let body = read_vector("dhcpv4_config_doc_operator.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            10,
            "dhcpv4_config_doc_operator.tsv line {}: expected 10 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let config = dhcpv4_config_doc(cols[1], cols[2], cols[3], cols[4], cols[5]);
        let operators = os_init_app::config::machine_config_dhcp_operators(&config).unwrap();
        assert_eq!(
            operators.len(),
            1,
            "DHCPv4Config case {} should materialize one operator",
            cols[0]
        );
        let op = &operators[0];
        assert_eq!(op.kind, OperatorKind::Dhcp4);
        assert_eq!(op.link_name, cols[1]);
        assert_eq!(op.route_metric.to_string(), cols[6]);
        assert_eq!(op.skip_hostname_request.to_string(), cols[7]);
        let expected_client = match cols[8] {
            "none" => ClientIdentifier::None,
            "mac" => ClientIdentifier::Mac,
            "duid" => ClientIdentifier::Duid,
            other => panic!("unknown expected client identifier {other}"),
        };
        assert_eq!(op.client_identifier.client_identifier, expected_client);
        assert_eq!(encode_hex(&op.client_identifier.duid_raw), cols[9]);
        checked += 1;
    }

    assert_eq!(
        checked, 5,
        "dhcpv4_config_doc_operator.tsv: expected 5 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// dhcpv6_config_doc_operator.tsv
// ---------------------------------------------------------------------------

fn optional_doc_scalar(key: &str, value: &str) -> String {
    if value == "-" {
        String::new()
    } else {
        format!("{key}: {value}\n")
    }
}

fn dhcpv6_config_doc(
    name: &str,
    route_metric: &str,
    ignore_hostname: &str,
    client_identifier: &str,
    duid_raw: &str,
) -> String {
    format!(
        "version: v1alpha1\nmachine:\n  type: worker\n---\napiVersion: v1alpha1\nkind: DHCPv6Config\nname: {name}\n{}{}{}{}",
        optional_doc_scalar("routeMetric", route_metric),
        optional_doc_scalar("ignoreHostname", ignore_hostname),
        optional_doc_scalar("clientIdentifier", client_identifier),
        optional_doc_scalar("duidRaw", duid_raw),
    )
}

#[test]
fn dhcpv6_config_doc_operator_tsv() {
    let body = read_vector("dhcpv6_config_doc_operator.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            11,
            "dhcpv6_config_doc_operator.tsv line {}: expected 11 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let config = dhcpv6_config_doc(cols[1], cols[2], cols[3], cols[4], cols[5]);
        let operators = os_init_app::config::machine_config_dhcp_operators(&config).unwrap();
        assert_eq!(
            operators.len(),
            1,
            "DHCPv6Config case {} should materialize one operator",
            cols[0]
        );
        let op = &operators[0];
        assert_eq!(op.kind, OperatorKind::Dhcp6);
        assert_eq!(op.link_name, cols[1]);
        assert_eq!(op.route_metric.to_string(), cols[8]);
        assert_eq!(op.skip_hostname_request.to_string(), cols[9]);

        let mac = parse_mac_hex(cols[6]);
        let seconds_since_2000 = parse_u32_hex(cols[7]);
        let identity = op.dhcp6_client_identifier(mac, seconds_since_2000).unwrap();
        let duid = Dhcp6ClientConfig::new([0x01, 0x02, 0x03], 0x5e10_2030, identity)
            .client_duid()
            .unwrap()
            .unwrap();
        assert_eq!(
            encode_hex(&duid),
            cols[10],
            "DHCPv6Config case {} DUID mismatch",
            cols[0]
        );
        checked += 1;
    }

    assert_eq!(
        checked, 5,
        "dhcpv6_config_doc_operator.tsv: expected 5 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// resolver_config_doc_operator.tsv
// ---------------------------------------------------------------------------

fn csv_nameserver_sequence(csv: &str) -> String {
    if csv == "-" || csv.is_empty() {
        String::new()
    } else {
        let items = csv
            .split(',')
            .map(|item| format!("  - address: {item}\n"))
            .collect::<String>();
        format!("nameservers:\n{items}")
    }
}

fn resolver_config_doc(name: &str, servers: &str, search_domains: &str) -> String {
    let _case_label = name;
    let search = if search_domains == "-" || search_domains.is_empty() {
        String::new()
    } else {
        let items = search_domains
            .split(',')
            .map(|item| format!("    - {item}\n"))
            .collect::<String>();
        format!("searchDomains:\n  domains:\n{items}")
    };
    format!(
        "version: v1alpha1\nmachine:\n  type: worker\n---\napiVersion: v1alpha1\nkind: ResolverConfig\n{}{}",
        csv_nameserver_sequence(servers),
        search,
    )
}

#[test]
fn resolver_config_doc_operator_tsv() {
    let body = read_vector("resolver_config_doc_operator.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            9,
            "resolver_config_doc_operator.tsv line {}: expected 9 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let config = resolver_config_doc(cols[1], cols[2], cols[3]);
        let specs = os_init_app::config::machine_config_resolver_specs(&config).unwrap();
        assert_eq!(
            specs.len(),
            1,
            "ResolverConfig case {} should materialize one resolver spec",
            cols[0]
        );
        let spec = &specs[0];
        let actual_servers = spec
            .servers
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let actual_search = spec.search_domains.join(",");
        assert_eq!(actual_servers, cols[5]);
        assert_eq!(actual_search, cols[6]);
        assert_eq!(spec.layer.as_str(), cols[7]);
        assert_eq!(
            format!(
                "source={};servers={};search={};layer={}",
                cols[4],
                actual_servers,
                actual_search,
                spec.layer.as_str()
            ),
            cols[8],
            "ResolverConfig network-config fingerprint mismatch for case {}",
            cols[0]
        );
        checked += 1;
    }

    assert_eq!(
        checked, 1,
        "resolver_config_doc_operator.tsv: expected 1 record, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// link_config_projection.tsv
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)] // arity mirrors the oracle TSV column set
fn link_config_doc(
    kind: &str,
    name: &str,
    parent: &str,
    vlan_id: &str,
    vlan_mode: &str,
    up: &str,
    mtu: &str,
    multicast: &str,
    addresses: &str,
    routes: &str,
) -> String {
    let mut doc = format!(
        "version: v1alpha1\nmachine:\n  type: worker\n---\napiVersion: v1alpha1\nkind: {kind}\nname: {name}\n"
    );

    if kind == "VLANConfig" {
        doc.push_str(&optional_doc_scalar("parent", parent));
        doc.push_str(&optional_doc_scalar("vlanID", vlan_id));
        doc.push_str(&optional_doc_scalar("vlanMode", vlan_mode));
    }

    doc.push_str(&optional_doc_scalar("up", up));
    doc.push_str(&optional_doc_scalar("mtu", mtu));
    doc.push_str(&optional_doc_scalar("multicast", multicast));
    doc.push_str(&link_config_addresses_doc(addresses));
    doc.push_str(&link_config_routes_doc(routes));
    doc
}

fn link_config_addresses_doc(encoded: &str) -> String {
    if encoded == "-" || encoded.is_empty() {
        return String::new();
    }

    let mut out = String::from("addresses:\n");
    for entry in encoded.split(';') {
        let cols: Vec<&str> = entry.split('|').collect();
        assert_eq!(
            cols.len(),
            2,
            "LinkConfig address vector entry should be address|routePriority: {entry:?}"
        );
        out.push_str(&format!("  - address: {}\n", cols[0]));
        if cols[1] != "-" {
            out.push_str(&format!("    routePriority: {}\n", cols[1]));
        }
    }
    out
}

fn link_config_routes_doc(encoded: &str) -> String {
    if encoded == "-" || encoded.is_empty() {
        return String::new();
    }

    let mut out = String::from("routes:\n");
    for entry in encoded.split(';') {
        let cols: Vec<&str> = entry.split('|').collect();
        assert_eq!(
            cols.len(),
            6,
            "LinkConfig route vector entry should be destination|gateway|source|metric|mtu|table: {entry:?}"
        );
        let scalars = [
            ("destination", cols[0]),
            ("gateway", cols[1]),
            ("source", cols[2]),
            ("metric", cols[3]),
            ("mtu", cols[4]),
            ("table", cols[5]),
        ];
        let mut wrote_any = false;
        for (key, value) in scalars {
            if value != "-" {
                if wrote_any {
                    out.push_str(&format!("    {key}: {value}\n"));
                } else {
                    out.push_str(&format!("  - {key}: {value}\n"));
                }
                wrote_any = true;
            }
        }
        assert!(
            wrote_any,
            "LinkConfig route vector entry must set at least one route field"
        );
    }
    out
}

fn link_fields_fingerprint(fields: &os_machine_config_domain::LinkFields) -> String {
    let up = fields
        .up
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let multicast = fields
        .multicast
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let addresses = if fields.addresses.is_empty() {
        "-".to_string()
    } else {
        fields
            .addresses
            .iter()
            .map(|address| format!("{}@{}", address.address, address.route_priority))
            .collect::<Vec<_>>()
            .join(";")
    };
    let routes = if fields.routes.is_empty() {
        "-".to_string()
    } else {
        fields
            .routes
            .iter()
            .map(|route| {
                format!(
                    "dst={},gw={},src={},metric={},mtu={},table={}",
                    route.destination,
                    route.gateway,
                    route.source,
                    route.metric,
                    route.mtu,
                    route.table
                )
            })
            .collect::<Vec<_>>()
            .join(";")
    };

    format!(
        "up={up};mtu={};multicast={multicast};addresses={addresses};routes={routes}",
        fields.mtu
    )
}

fn link_config_parse_fingerprint(config: &str, kind: &str) -> String {
    let container = os_machine_config_domain::load_from_bytes(config).unwrap();
    match kind {
        "LinkConfig" => {
            let links = os_machine_config_domain::link_configs(&container).unwrap();
            assert_eq!(
                links.len(),
                1,
                "LinkConfig vector should decode one LinkConfig"
            );
            let link = &links[0];
            format!(
                "kind=LinkConfig;name={};{}",
                link.name,
                link_fields_fingerprint(&link.link)
            )
        }
        "VLANConfig" => {
            let vlans = os_machine_config_domain::vlan_configs(&container).unwrap();
            assert_eq!(
                vlans.len(),
                1,
                "VLANConfig vector should decode one VLANConfig"
            );
            let vlan = &vlans[0];
            format!(
                "kind=VLANConfig;name={};parent={};vlanID={};vlanMode={};{}",
                vlan.name,
                vlan.parent,
                vlan.vlan_id,
                vlan.vlan_mode_or_default().as_str(),
                link_fields_fingerprint(&vlan.link)
            )
        }
        other => panic!("unknown link config vector kind {other:?}"),
    }
}

fn link_config_projection_fingerprint(config: &str, _kind: &str) -> String {
    match os_controllers_domain::machine_config_link_projection_fingerprint(config) {
        Ok(fingerprint) => fingerprint,
        Err(error) => format!("error={error}"),
    }
}

#[test]
fn link_config_projection_tsv() {
    let body = read_vector("link_config_projection.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            13,
            "link_config_projection.tsv line {}: expected 13 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let config = link_config_doc(
            cols[1], cols[2], cols[3], cols[4], cols[5], cols[6], cols[7], cols[8], cols[9],
            cols[10],
        );
        let actual_parse = link_config_parse_fingerprint(&config, cols[1]);
        assert_eq!(
            actual_parse, cols[11],
            "LinkConfig/VLANConfig parse fingerprint mismatch for case {}",
            cols[0]
        );

        let actual_projection = link_config_projection_fingerprint(&config, cols[1]);
        assert_eq!(
            actual_projection, cols[12],
            "LinkConfig/VLANConfig projection fingerprint mismatch for case {}",
            cols[0]
        );
        checked += 1;
    }

    assert_eq!(
        checked, 6,
        "link_config_projection.tsv: expected 6 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// volume_config_doc_projection.tsv
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)] // arity mirrors the oracle TSV column set
fn volume_config_doc(
    kind: &str,
    name: &str,
    volume_type: &str,
    min_size: &str,
    max_size: &str,
    grow: &str,
    disk_selector: &str,
    filesystem: &str,
) -> String {
    let mut doc = format!(
        "version: v1alpha1\nmachine:\n  type: worker\n---\napiVersion: v1alpha1\nkind: {kind}\nname: {name}\n"
    );
    if kind == "UserVolumeConfig" && volume_type != "-" {
        doc.push_str(&format!("volumeType: {volume_type}\n"));
    }

    let mut provisioning = String::new();
    if min_size != "-" {
        provisioning.push_str(&format!("  minSize: {min_size}\n"));
    }
    if max_size != "-" {
        provisioning.push_str(&format!("  maxSize: {max_size}\n"));
    }
    if grow != "-" {
        provisioning.push_str(&format!("  grow: {grow}\n"));
    }
    if disk_selector != "-" {
        provisioning.push_str("  diskSelector:\n");
        provisioning.push_str(&format!("    match: {disk_selector}\n"));
    }
    if !provisioning.is_empty() {
        doc.push_str("provisioning:\n");
        doc.push_str(&provisioning);
    }

    if kind == "UserVolumeConfig" && filesystem != "-" {
        doc.push_str("filesystem:\n");
        doc.push_str(&format!("  type: {filesystem}\n"));
    }

    doc
}

fn option_u64_fingerprint(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn size_limit_fingerprint(value: Option<os_machine_config_domain::SizeLimit>) -> String {
    match value {
        Some(os_machine_config_domain::SizeLimit::Absolute(bytes)) => bytes.to_string(),
        Some(os_machine_config_domain::SizeLimit::RelativePercent(percent)) => {
            format!("{percent}%")
        }
        Some(os_machine_config_domain::SizeLimit::NegativeBytes(bytes)) => format!("-{bytes}"),
        Some(os_machine_config_domain::SizeLimit::NegativeRelativePercent(percent)) => {
            format!("-{percent}%")
        }
        None => "-".to_string(),
    }
}

fn block_max_size_fingerprint(
    max_size: Option<u64>,
    relative_max_size: Option<u64>,
    negative_max_size: bool,
) -> String {
    if let Some(percent) = relative_max_size {
        if negative_max_size {
            format!("-{percent}%")
        } else {
            format!("{percent}%")
        }
    } else if let Some(bytes) = max_size {
        if negative_max_size {
            format!("-{bytes}")
        } else {
            bytes.to_string()
        }
    } else {
        "-".to_string()
    }
}

fn bool_fingerprint(value: Option<bool>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn volume_config_parse_fingerprint(config: &str, kind: &str) -> String {
    let container = match os_machine_config_domain::load_from_bytes(config) {
        Ok(container) => container,
        Err(error) => return format!("error={error}"),
    };
    match kind {
        "VolumeConfig" => {
            let docs = match os_machine_config_domain::volume_configs(&container) {
                Ok(docs) => docs,
                Err(error) => return format!("error={error}"),
            };
            assert_eq!(docs.len(), 1, "vector should decode one VolumeConfig");
            let doc = &docs[0];
            format!(
                "kind=VolumeConfig;name={};min={};max={};grow={};selector={}",
                doc.name,
                option_u64_fingerprint(doc.provisioning.min_size),
                size_limit_fingerprint(doc.provisioning.max_size),
                bool_fingerprint(doc.provisioning.grow),
                doc.provisioning.disk_selector.as_deref().unwrap_or("-")
            )
        }
        "UserVolumeConfig" => {
            let docs = match os_machine_config_domain::user_volume_configs(&container) {
                Ok(docs) => docs,
                Err(error) => return format!("error={error}"),
            };
            assert_eq!(docs.len(), 1, "vector should decode one UserVolumeConfig");
            let doc = &docs[0];
            format!(
                "kind=UserVolumeConfig;name={};id={};type={};min={};max={};grow={};selector={};fs={}",
                doc.name,
                doc.volume_id(),
                doc.volume_type.as_str(),
                option_u64_fingerprint(doc.provisioning.min_size),
                size_limit_fingerprint(doc.provisioning.max_size),
                bool_fingerprint(doc.provisioning.grow),
                doc.provisioning.disk_selector.as_deref().unwrap_or("-"),
                doc.filesystem.filesystem.as_str()
            )
        }
        "RawVolumeConfig" => {
            let docs = match os_machine_config_domain::raw_volume_configs(&container) {
                Ok(docs) => docs,
                Err(error) => return format!("error={error}"),
            };
            assert_eq!(docs.len(), 1, "vector should decode one RawVolumeConfig");
            let doc = &docs[0];
            format!(
                "kind=RawVolumeConfig;name={};id={};min={};max={};grow={};selector={}",
                doc.name,
                doc.volume_id(),
                option_u64_fingerprint(doc.provisioning.min_size),
                size_limit_fingerprint(doc.provisioning.max_size),
                bool_fingerprint(doc.provisioning.grow),
                doc.provisioning.disk_selector.as_deref().unwrap_or("-")
            )
        }
        other => panic!("unknown volume config vector kind {other:?}"),
    }
}

fn volume_config_projection_fingerprint(config: &str) -> String {
    let manager = match os_init_app::config::machine_config_volume_manager(config) {
        Ok(manager) => manager,
        Err(error) => return format!("error={error}"),
    };
    manager
        .ordered()
        .into_iter()
        .map(|volume| {
            let max = block_max_size_fingerprint(
                volume.config.max_size,
                volume.config.relative_max_size,
                volume.config.negative_max_size,
            );
            let fs = volume
                .config
                .filesystem
                .map(|fs| fs.as_str())
                .unwrap_or("-");
            let grow = bool_fingerprint(volume.config.grow);
            format!(
                "{}:{:?}:{}:{}:{}:{}:{}",
                volume.config.id,
                volume.class,
                volume.priority,
                volume.config.min_size,
                max,
                grow,
                fs
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

#[test]
fn volume_config_doc_projection_tsv() {
    let body = read_vector("volume_config_doc_projection.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            11,
            "volume_config_doc_projection.tsv line {}: expected 11 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let config = volume_config_doc(
            cols[1], cols[2], cols[3], cols[4], cols[5], cols[6], cols[7], cols[8],
        );
        let actual_parse = volume_config_parse_fingerprint(&config, cols[1]);
        assert_eq!(
            actual_parse, cols[9],
            "VolumeConfig/UserVolumeConfig/RawVolumeConfig parse fingerprint mismatch for case {}",
            cols[0]
        );

        let actual_projection = volume_config_projection_fingerprint(&config);
        assert_eq!(
            actual_projection, cols[10],
            "VolumeConfig/UserVolumeConfig projection fingerprint mismatch for case {}",
            cols[0]
        );
        checked += 1;
    }

    assert_eq!(
        checked, 15,
        "volume_config_doc_projection.tsv: expected 15 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// default_dhcp_operator_synthesis.tsv
// ---------------------------------------------------------------------------

fn vip_operator_config(case_kind: &str, link: &str, ip: &str) -> String {
    match case_kind {
        "legacy-interface" => format!(
            "\
version: v1alpha1
machine:
  type: controlplane
  network:
    interfaces:
      - interface: {link}
        vip:
          ip: {ip}
"
        ),
        "legacy-vlan" => format!(
            "\
version: v1alpha1
machine:
  type: controlplane
  network:
    interfaces:
      - interface: {link}
        vlans:
          - vlanId: 26
            vip:
              ip: {ip}
"
        ),
        "layer2-doc" => format!(
            "\
version: v1alpha1
machine:
  type: controlplane
---
apiVersion: v1alpha1
kind: Layer2VIPConfig
name: {ip}
link: {link}
"
        ),
        other => panic!("unknown VIP operator config case {other:?}"),
    }
}

#[test]
fn vip_operator_config_tsv() {
    let body = read_vector("vip_operator_config.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            5,
            "vip_operator_config.tsv line {}: expected 5 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let config = vip_operator_config(cols[1], cols[2], cols[3]);
        let actual =
            os_controllers_domain::network::machine_config_operator_specs_fingerprint(&config)
                .unwrap_or_else(|error| format!("error={error}"));
        assert_eq!(
            actual, cols[4],
            "VIP operator projection mismatch for case {}",
            cols[0]
        );
        checked += 1;
    }

    assert_eq!(
        checked, 3,
        "vip_operator_config.tsv: expected 3 records, checked {checked}"
    );
}

fn default_dhcp_config_case(case: &str) -> Option<String> {
    match case {
        "absent" => None,
        "minimal" => Some("version: v1alpha1\nmachine:\n  type: worker\n".to_string()),
        "legacy-static" => Some(
            "\
version: v1alpha1
machine:
  type: worker
  network:
    interfaces:
      - interface: eth0
        dhcp: false
"
            .to_string(),
        ),
        "legacy-ignore" => Some(
            "\
version: v1alpha1
machine:
  type: worker
  network:
    interfaces:
      - interface: eth0
        dhcp: true
        ignore: true
"
            .to_string(),
        ),
        "linkconfig" => Some(
            "\
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: LinkConfig
name: eth0
"
            .to_string(),
        ),
        "dhcpv4doc" => Some(
            "\
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: DHCPv4Config
name: eth0
"
            .to_string(),
        ),
        other => panic!("unknown default DHCP config case {other:?}"),
    }
}

fn default_dhcp_status(
    name: &str,
    link_type: &str,
    kind: &str,
    aliases: &str,
    carrier: &str,
) -> LinkStatus {
    LinkStatus {
        name: name.to_string(),
        link_type: match link_type {
            "ether" => NetworkLinkType::Ether,
            other => NetworkLinkType::Other(other.to_string()),
        },
        kind: kind.to_string(),
        aliases: if aliases == "-" || aliases.is_empty() {
            Vec::new()
        } else {
            aliases.split(',').map(str::to_string).collect()
        },
        admin_up: carrier == "true",
        oper_state: if carrier == "true" {
            OperState::Up
        } else {
            OperState::Down
        },
        carrier: carrier == "true",
        hardware_addr: [0x02, 0, 0, 0, 0, 1],
        mtu: 1500,
    }
}

#[test]
fn default_dhcp_operator_synthesis_tsv() {
    let body = read_vector("default_dhcp_operator_synthesis.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            8,
            "default_dhcp_operator_synthesis.tsv line {}: expected 8 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let config = default_dhcp_config_case(cols[1]);
        let status = default_dhcp_status(cols[2], cols[3], cols[4], cols[5], cols[6]);
        let actual = os_controllers_domain::default_dhcp_link_status_projection_fingerprint(
            config.as_deref(),
            &[status],
        )
        .unwrap_or_else(|error| format!("error={error}"));
        assert_eq!(
            actual, cols[7],
            "default DHCP projection mismatch for case {}",
            cols[0]
        );
        checked += 1;
    }

    assert_eq!(
        checked, 10,
        "default_dhcp_operator_synthesis.tsv: expected 10 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// role.tsv
// ---------------------------------------------------------------------------

#[test]
fn role_tsv() {
    let body = read_vector("role.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        match cols[0] {
            "parse" => {
                assert_eq!(
                    cols.len(),
                    4,
                    "role.tsv line {}: expected 4 columns in parse record, got {}: {:?}",
                    lineno + 1,
                    cols.len(),
                    line
                );
                let input = cols[1];
                let expected_class = cols[2];
                let expected_canonical = cols[3];

                // Mirror role.Parse([]string{input}).
                let (set, unknown) = RoleSet::parse([input]);

                let (actual_class, actual_canonical): (&str, String) = if !unknown.is_empty() {
                    // Non-empty unknown slice => "unknown", empty canonical.
                    ("unknown", String::new())
                } else if set.strings().is_empty() {
                    // Dropped entirely (empty / whitespace-only after trim).
                    ("known", String::new())
                } else {
                    // Recognized canonical role; single-element input => one.
                    ("known", set.strings()[0].clone())
                };

                assert_eq!(
                    actual_class, expected_class,
                    "role parse class mismatch\n input    = {:?}\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
                    input, expected_class, actual_class
                );
                assert_eq!(
                    actual_canonical, expected_canonical,
                    "role parse canonical mismatch\n input    = {:?}\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
                    input, expected_canonical, actual_canonical
                );
                checked += 1;
            }
            "included" => {
                assert_eq!(
                    cols.len(),
                    3,
                    "role.tsv line {}: expected 3 columns in included record, got {}: {:?}",
                    lineno + 1,
                    cols.len(),
                    line
                );
                let role_string = cols[1];
                let expected = cols[2];

                let actual = if role_all_includes(role_string) {
                    "true"
                } else {
                    "false"
                };

                assert_eq!(
                    actual, expected,
                    "role included mismatch\n input    = {:?}\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
                    role_string, expected, actual
                );
                checked += 1;
            }
            other => panic!(
                "role.tsv line {}: unknown record kind {:?}",
                lineno + 1,
                other
            ),
        }
    }

    // 15 parse records + 11 included records.
    assert_eq!(
        checked, 26,
        "role.tsv: expected 26 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// version.tsv
// ---------------------------------------------------------------------------

#[test]
fn version_tsv() {
    let body = read_vector("version.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols[0],
            "tag",
            "version.tsv line {}: unknown record kind {:?}",
            lineno + 1,
            cols[0]
        );
        assert_eq!(
            cols.len(),
            3,
            "version.tsv line {}: expected 3 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let input = cols[1];
        let expected_short = cols[2];

        // input encodes the triple "name|tag|sha"; only name and tag feed Short().
        let parts: Vec<&str> = input.splitn(3, '|').collect();
        assert_eq!(
            parts.len(),
            3,
            "version.tsv line {}: malformed input column {:?}",
            lineno + 1,
            input
        );
        let name = parts[0];
        let tag = parts[1];

        // version.Short() == fmt.Sprintf("%s %s", Name, Tag).
        let actual_short = version_build::short(name, tag);

        assert_eq!(
            actual_short, expected_short,
            "version short mismatch\n input    = {:?}\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
            input, expected_short, actual_short
        );
        checked += 1;
    }

    assert_eq!(
        checked, 6,
        "version.tsv: expected 6 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// nethelpers.tsv
// ---------------------------------------------------------------------------

/// Look up an enum's canonical string by integer value, or `None` if no variant
/// has that value. Returns the string the Rust port produces for that value.
type ToStrByValue = fn(u64) -> Option<&'static str>;
/// Parse a string via the Rust port; `Some(value)` on success, `None` on error.
type ParseToValue = fn(&str) -> Option<u64>;

fn bondmode_to_str(v: u64) -> Option<&'static str> {
    [
        BondMode::Roundrobin,
        BondMode::ActiveBackup,
        BondMode::Xor,
        BondMode::Broadcast,
        BondMode::Ieee8023ad,
        BondMode::Tlb,
        BondMode::Alb,
    ]
    .into_iter()
    .find(|m| m.as_value() as u64 == v)
    .map(|m| m.to_str())
}

fn bondmode_parse(s: &str) -> Option<u64> {
    BondMode::parse(s).ok().map(|m| m.as_value() as u64)
}

fn arpvalidate_to_str(v: u64) -> Option<&'static str> {
    [
        ArpValidate::None,
        ArpValidate::Active,
        ArpValidate::Backup,
        ArpValidate::All,
        ArpValidate::Filter,
        ArpValidate::FilterActive,
        ArpValidate::FilterBackup,
    ]
    .into_iter()
    .find(|m| m.as_value() as u64 == v)
    .map(|m| m.to_str())
}

fn arpvalidate_parse(s: &str) -> Option<u64> {
    ArpValidate::parse(s).ok().map(|m| m.as_value() as u64)
}

fn adselect_to_str(v: u64) -> Option<&'static str> {
    [AdSelect::Stable, AdSelect::Bandwidth, AdSelect::Count]
        .into_iter()
        .find(|m| m.as_value() as u64 == v)
        .map(|m| m.to_str())
}

fn adselect_parse(s: &str) -> Option<u64> {
    AdSelect::parse(s).ok().map(|m| m.as_value() as u64)
}

fn failovermac_to_str(v: u64) -> Option<&'static str> {
    [FailOverMac::None, FailOverMac::Active, FailOverMac::Follow]
        .into_iter()
        .find(|m| m.as_value() as u64 == v)
        .map(|m| m.to_str())
}

fn failovermac_parse(s: &str) -> Option<u64> {
    FailOverMac::parse(s).ok().map(|m| m.as_value() as u64)
}

/// Resolve the (to_str-by-value, parse-to-value) function pair for an enum name.
fn enum_funcs(name: &str) -> Option<(ToStrByValue, ParseToValue)> {
    match name {
        "BondMode" => Some((bondmode_to_str, bondmode_parse)),
        "ARPValidate" => Some((arpvalidate_to_str, arpvalidate_parse)),
        "ADSelect" => Some((adselect_to_str, adselect_parse)),
        "FailOverMAC" => Some((failovermac_to_str, failovermac_parse)),
        _ => None,
    }
}

#[test]
fn nethelpers_tsv() {
    let body = read_vector("nethelpers.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            3,
            "nethelpers.tsv line {}: expected 3 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let kind = cols[0];

        if let Some(enum_name) = kind.strip_suffix("-parse") {
            // Kind B: {Enum}-parse<TAB>{StringValue}<TAB>{intValueOrError}.
            let (_to_str, parse) = enum_funcs(enum_name).unwrap_or_else(|| {
                panic!(
                    "nethelpers.tsv line {}: unknown enum {:?}",
                    lineno + 1,
                    enum_name
                )
            });
            let input = cols[1];
            let expected = cols[2];

            let actual = match parse(input) {
                Some(v) => v.to_string(),
                None => "error".to_string(),
            };

            assert_eq!(
                actual, expected,
                "{} parse mismatch\n input    = {:?}\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
                enum_name, input, expected, actual
            );
            checked += 1;
        } else {
            // Kind A: {Enum}<TAB>{intValue}<TAB>{StringValue}.
            let (to_str, _parse) = enum_funcs(kind).unwrap_or_else(|| {
                panic!(
                    "nethelpers.tsv line {}: unknown enum {:?}",
                    lineno + 1,
                    kind
                )
            });
            let int_value: u64 = cols[1].parse().unwrap_or_else(|_| {
                panic!(
                    "nethelpers.tsv line {}: non-numeric int value {:?}",
                    lineno + 1,
                    cols[1]
                )
            });
            let expected_string = cols[2];

            let actual_string = to_str(int_value).unwrap_or_else(|| {
                panic!(
                    "{} has no variant for value {} (expected {:?})",
                    kind, int_value, expected_string
                )
            });

            assert_eq!(
                actual_string, expected_string,
                "{} to_str mismatch\n value    = {}\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
                kind, int_value, expected_string, actual_string
            );
            checked += 1;
        }
    }

    assert_eq!(
        checked, 44,
        "nethelpers.tsv: expected 44 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// Generic enum vector harness (one TSV file per enumer-generated enum).
//
// Every nethelpers_*.tsv and machine_type.tsv share the same shape, written by
// the oracle's `writeEnumFile` / `writeMachineType`:
//
//   val<TAB>{intValue}<TAB>{String()output}
//   parse<TAB>{String()output}<TAB>{intValueOrERR}
//
// `val` records exercise the Rust enum's to_str-by-value path; `parse` records
// exercise the Rust parse path (expected "ERR" means upstream parse failed, so
// Rust must fail too). Value columns may be empty (e.g. Port(3).String() == "")
// and MUST NOT be trimmed.
// ---------------------------------------------------------------------------

/// Run a single enum vector file.
///
/// `to_str_by_value(v)` returns the Rust canonical string for integer value `v`,
/// or `None` if no variant has that value. `parse_to_value(s)` returns the Rust
/// integer value parsed from `s`, or `None` on parse error.
fn run_enum_file(
    file: &str,
    to_str_by_value: impl Fn(i64) -> Option<String>,
    parse_to_value: impl Fn(&str) -> Option<i64>,
) -> usize {
    let body = read_vector(file);
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            3,
            "{file} line {}: expected 3 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        match cols[0] {
            "val" => {
                let value: i64 = cols[1].parse().unwrap_or_else(|_| {
                    panic!(
                        "{file} line {}: non-numeric int value {:?}",
                        lineno + 1,
                        cols[1]
                    )
                });
                let expected = cols[2];
                let actual = to_str_by_value(value).unwrap_or_else(|| {
                    panic!(
                        "{file} to_str: no variant for value {} (expected {:?})",
                        value, expected
                    )
                });
                assert_eq!(
                    actual, expected,
                    "{file} to_str mismatch\n value    = {}\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
                    value, expected, actual
                );
                checked += 1;
            }
            "parse" => {
                let input = cols[1];
                let expected = cols[2];
                let actual = match parse_to_value(input) {
                    Some(v) => v.to_string(),
                    None => "ERR".to_string(),
                };
                assert_eq!(
                    actual, expected,
                    "{file} parse mismatch\n input    = {:?}\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
                    input, expected, actual
                );
                checked += 1;
            }
            other => panic!(
                "{file} line {}: unknown record kind {:?}",
                lineno + 1,
                other
            ),
        }
    }

    checked
}

/// Generate a `#[test]` for a standard nethelpers enum whose API is
/// `Enum::parse(&str) -> Result`, `variant.as_value()` (any integer width), and
/// `variant.to_str() -> &'static str`, plus a slice of all variants.
macro_rules! enum_vector_test {
    ($test:ident, $file:literal, $ty:ty, $expected:expr, [$($variant:expr),+ $(,)?]) => {
        #[test]
        fn $test() {
            let to_str = |v: i64| -> Option<String> {
                [$($variant),+]
                    .into_iter()
                    .find(|m: &$ty| m.as_value() as i64 == v)
                    .map(|m| m.to_str().to_string())
            };
            let parse = |s: &str| -> Option<i64> {
                <$ty>::parse(s).ok().map(|m| m.as_value() as i64)
            };
            let checked = run_enum_file($file, to_str, parse);
            assert_eq!(checked, $expected, concat!($file, ": expected {} records, checked {}"), $expected, checked);
        }
    };
}

enum_vector_test!(
    nethelpers_ad_lacp_active_tsv,
    "nethelpers_ad_lacp_active.tsv",
    AdLacpActive,
    5,
    [AdLacpActive::Off, AdLacpActive::On]
);

enum_vector_test!(
    nethelpers_ad_select_tsv,
    "nethelpers_ad_select.tsv",
    AdSelect,
    7,
    [AdSelect::Stable, AdSelect::Bandwidth, AdSelect::Count]
);

enum_vector_test!(
    nethelpers_address_flag_tsv,
    "nethelpers_address_flag.tsv",
    AddressFlag,
    25,
    [
        AddressFlag::Temporary,
        AddressFlag::NoDad,
        AddressFlag::Optimistic,
        AddressFlag::DadFailed,
        AddressFlag::Home,
        AddressFlag::Deprecated,
        AddressFlag::Tentative,
        AddressFlag::Permanent,
        AddressFlag::ManagementTemp,
        AddressFlag::NoPrefixRoute,
        AddressFlag::McAutoJoin,
        AddressFlag::StablePrivacy,
    ]
);

enum_vector_test!(
    nethelpers_address_sort_algorithm_tsv,
    "nethelpers_address_sort_algorithm.tsv",
    AddressSortAlgorithm,
    5,
    [AddressSortAlgorithm::V1, AddressSortAlgorithm::V2]
);

enum_vector_test!(
    nethelpers_arp_all_targets_tsv,
    "nethelpers_arp_all_targets.tsv",
    ArpAllTargets,
    5,
    [ArpAllTargets::Any, ArpAllTargets::All]
);

enum_vector_test!(
    nethelpers_arp_validate_tsv,
    "nethelpers_arp_validate.tsv",
    ArpValidate,
    15,
    [
        ArpValidate::None,
        ArpValidate::Active,
        ArpValidate::Backup,
        ArpValidate::All,
        ArpValidate::Filter,
        ArpValidate::FilterActive,
        ArpValidate::FilterBackup,
    ]
);

enum_vector_test!(
    nethelpers_auto_hostname_kind_tsv,
    "nethelpers_auto_hostname_kind.tsv",
    AutoHostnameKind,
    7,
    [
        AutoHostnameKind::Off,
        AutoHostnameKind::Addr,
        AutoHostnameKind::Stable
    ]
);

enum_vector_test!(
    nethelpers_bond_mode_tsv,
    "nethelpers_bond_mode.tsv",
    BondMode,
    15,
    [
        BondMode::Roundrobin,
        BondMode::ActiveBackup,
        BondMode::Xor,
        BondMode::Broadcast,
        BondMode::Ieee8023ad,
        BondMode::Tlb,
        BondMode::Alb,
    ]
);

enum_vector_test!(
    nethelpers_bond_xmit_hash_policy_tsv,
    "nethelpers_bond_xmit_hash_policy.tsv",
    BondXmitHashPolicy,
    11,
    [
        BondXmitHashPolicy::Layer2,
        BondXmitHashPolicy::Layer34,
        BondXmitHashPolicy::Layer23,
        BondXmitHashPolicy::Encap23,
        BondXmitHashPolicy::Encap34,
    ]
);

enum_vector_test!(
    nethelpers_client_identifier_tsv,
    "nethelpers_client_identifier.tsv",
    ClientIdentifier,
    7,
    [
        ClientIdentifier::None,
        ClientIdentifier::Mac,
        ClientIdentifier::Duid
    ]
);

enum_vector_test!(
    nethelpers_conntrack_state_tsv,
    "nethelpers_conntrack_state.tsv",
    ConntrackState,
    9,
    [
        ConntrackState::New,
        ConntrackState::Related,
        ConntrackState::Established,
        ConntrackState::Invalid,
    ]
);

enum_vector_test!(
    nethelpers_default_action_tsv,
    "nethelpers_default_action.tsv",
    DefaultAction,
    5,
    [DefaultAction::Accept, DefaultAction::Block]
);

enum_vector_test!(
    nethelpers_dns_protocol_tsv,
    "nethelpers_dns_protocol.tsv",
    DnsProtocol,
    7,
    [
        DnsProtocol::Default,
        DnsProtocol::DnsOverTls,
        DnsProtocol::DnsOverHttp
    ]
);

enum_vector_test!(
    nethelpers_duplex_tsv,
    "nethelpers_duplex.tsv",
    Duplex,
    7,
    [Duplex::Half, Duplex::Full, Duplex::Unknown]
);

enum_vector_test!(
    nethelpers_fail_over_mac_tsv,
    "nethelpers_fail_over_mac.tsv",
    FailOverMac,
    7,
    [FailOverMac::None, FailOverMac::Active, FailOverMac::Follow]
);

enum_vector_test!(
    nethelpers_family_tsv,
    "nethelpers_family.tsv",
    Family,
    5,
    [Family::Inet4, Family::Inet6]
);

enum_vector_test!(
    nethelpers_lacp_rate_tsv,
    "nethelpers_lacp_rate.tsv",
    LacpRate,
    5,
    [LacpRate::Slow, LacpRate::Fast]
);

enum_vector_test!(
    nethelpers_link_flag_tsv,
    "nethelpers_link_flag.tsv",
    LinkFlag,
    39,
    [
        LinkFlag::Up,
        LinkFlag::Broadcast,
        LinkFlag::Debug,
        LinkFlag::Loopback,
        LinkFlag::PointToPoint,
        LinkFlag::NoTrailers,
        LinkFlag::Running,
        LinkFlag::NoArp,
        LinkFlag::Promisc,
        LinkFlag::AllMulti,
        LinkFlag::Master,
        LinkFlag::Slave,
        LinkFlag::Multicast,
        LinkFlag::Portsel,
        LinkFlag::AutoMedia,
        LinkFlag::Dynamic,
        LinkFlag::LowerUp,
        LinkFlag::Dormant,
        LinkFlag::Echo,
    ]
);

enum_vector_test!(
    nethelpers_match_operator_tsv,
    "nethelpers_match_operator.tsv",
    MatchOperator,
    5,
    [MatchOperator::Equal, MatchOperator::NotEqual]
);

enum_vector_test!(
    nethelpers_nftables_chain_hook_tsv,
    "nethelpers_nftables_chain_hook.tsv",
    NfTablesChainHook,
    11,
    [
        NfTablesChainHook::Prerouting,
        NfTablesChainHook::Input,
        NfTablesChainHook::Forward,
        NfTablesChainHook::Output,
        NfTablesChainHook::Postrouting,
    ]
);

enum_vector_test!(
    nethelpers_nftables_chain_priority_tsv,
    "nethelpers_nftables_chain_priority.tsv",
    NfTablesChainPriority,
    27,
    [
        NfTablesChainPriority::First,
        NfTablesChainPriority::ConntrackDefrag,
        NfTablesChainPriority::Raw,
        NfTablesChainPriority::SelinuxFirst,
        NfTablesChainPriority::Conntrack,
        NfTablesChainPriority::Mangle,
        NfTablesChainPriority::NatDest,
        NfTablesChainPriority::Filter,
        NfTablesChainPriority::Security,
        NfTablesChainPriority::NatSource,
        NfTablesChainPriority::SelinuxLast,
        NfTablesChainPriority::ConntrackHelper,
        NfTablesChainPriority::Last,
    ]
);

enum_vector_test!(
    nethelpers_nftables_verdict_tsv,
    "nethelpers_nftables_verdict.tsv",
    NfTablesVerdict,
    5,
    [NfTablesVerdict::Drop, NfTablesVerdict::Accept]
);

enum_vector_test!(
    nethelpers_operational_state_tsv,
    "nethelpers_operational_state.tsv",
    OperationalState,
    15,
    [
        OperationalState::Unknown,
        OperationalState::NotPresent,
        OperationalState::Down,
        OperationalState::LowerLayerDown,
        OperationalState::Testing,
        OperationalState::Dormant,
        OperationalState::Up,
    ]
);

enum_vector_test!(
    nethelpers_port_tsv,
    "nethelpers_port.tsv",
    Port,
    17,
    [
        Port::TwistedPair,
        Port::Aui,
        Port::Mii,
        Port::Fibre,
        Port::Bnc,
        Port::DirectAttach,
        Port::None,
        Port::Other,
    ]
);

enum_vector_test!(
    nethelpers_primary_reselect_tsv,
    "nethelpers_primary_reselect.tsv",
    PrimaryReselect,
    7,
    [
        PrimaryReselect::Always,
        PrimaryReselect::Better,
        PrimaryReselect::Failure
    ]
);

enum_vector_test!(
    nethelpers_protocol_tsv,
    "nethelpers_protocol.tsv",
    Protocol,
    9,
    [
        Protocol::Icmp,
        Protocol::Tcp,
        Protocol::Udp,
        Protocol::Icmpv6
    ]
);

enum_vector_test!(
    nethelpers_route_flag_tsv,
    "nethelpers_route_flag.tsv",
    RouteFlag,
    17,
    [
        RouteFlag::Notify,
        RouteFlag::Cloned,
        RouteFlag::Equalize,
        RouteFlag::Prefix,
        RouteFlag::LookupTable,
        RouteFlag::FibMatch,
        RouteFlag::Offload,
        RouteFlag::Trap,
    ]
);

enum_vector_test!(
    nethelpers_route_protocol_tsv,
    "nethelpers_route_protocol.tsv",
    RouteProtocol,
    45,
    [
        RouteProtocol::Unspec,
        RouteProtocol::Redirect,
        RouteProtocol::Kernel,
        RouteProtocol::Boot,
        RouteProtocol::Static,
        RouteProtocol::Ra,
        RouteProtocol::Mrt,
        RouteProtocol::Zebra,
        RouteProtocol::Bird,
        RouteProtocol::Dnrouted,
        RouteProtocol::Xorp,
        RouteProtocol::Ntk,
        RouteProtocol::Dhcp,
        RouteProtocol::Mrtd,
        RouteProtocol::Keepalived,
        RouteProtocol::Babel,
        RouteProtocol::Openr,
        RouteProtocol::Bgp,
        RouteProtocol::Isis,
        RouteProtocol::Ospf,
        RouteProtocol::Rip,
        RouteProtocol::Eigrp,
    ]
);

enum_vector_test!(
    nethelpers_route_type_tsv,
    "nethelpers_route_type.tsv",
    RouteType,
    25,
    [
        RouteType::Unspec,
        RouteType::Unicast,
        RouteType::Local,
        RouteType::Broadcast,
        RouteType::Anycast,
        RouteType::Multicast,
        RouteType::Blackhole,
        RouteType::Unreachable,
        RouteType::Prohibit,
        RouteType::Throw,
        RouteType::Nat,
        RouteType::XResolve,
    ]
);

enum_vector_test!(
    nethelpers_routing_rule_action_tsv,
    "nethelpers_routing_rule_action.tsv",
    RoutingRuleAction,
    11,
    [
        RoutingRuleAction::Unspec,
        RoutingRuleAction::Unicast,
        RoutingRuleAction::Blackhole,
        RoutingRuleAction::Unreachable,
        RoutingRuleAction::Prohibit,
    ]
);

enum_vector_test!(
    nethelpers_scope_tsv,
    "nethelpers_scope.tsv",
    Scope,
    11,
    [
        Scope::Global,
        Scope::Site,
        Scope::Link,
        Scope::Host,
        Scope::Nowhere
    ]
);

enum_vector_test!(
    nethelpers_status_tsv,
    "nethelpers_status.tsv",
    Status,
    9,
    [
        Status::Addresses,
        Status::Connectivity,
        Status::Hostname,
        Status::EtcFiles
    ]
);

enum_vector_test!(
    nethelpers_vlan_protocol_tsv,
    "nethelpers_vlan_protocol.tsv",
    VlanProtocol,
    5,
    [VlanProtocol::Ieee8021q, VlanProtocol::Ieee8021ad]
);

enum_vector_test!(
    nethelpers_wol_mode_tsv,
    "nethelpers_wol_mode.tsv",
    WolMode,
    15,
    [
        WolMode::Phy,
        WolMode::Unicast,
        WolMode::Multicast,
        WolMode::Broadcast,
        WolMode::Magic,
        WolMode::MagicSecure,
        WolMode::Filter,
    ]
);

// LinkType is large; use its ALL slice rather than re-listing every variant.
#[test]
fn nethelpers_link_type_tsv() {
    let to_str = |v: i64| -> Option<String> {
        LinkType::ALL
            .into_iter()
            .find(|m| m.as_value() as i64 == v)
            .map(|m| m.to_str().to_string())
    };
    let parse = |s: &str| -> Option<i64> { LinkType::parse(s).ok().map(|m| m.as_value() as i64) };
    let checked = run_enum_file("nethelpers_link_type.tsv", to_str, parse);
    assert_eq!(
        checked, 153,
        "nethelpers_link_type.tsv: expected 153 records, checked {checked}"
    );
}

// RoutingTable is a numeric wrapper, not a unit enum: 0/253/254/255 are named,
// 1..=252 stringify to their decimal form. Use the dedicated API.
#[test]
fn nethelpers_routing_table_tsv() {
    let to_str = |v: i64| -> Option<String> {
        if (0..=255).contains(&v) {
            Some(RoutingTable(v as u32).to_string_value())
        } else {
            None
        }
    };
    let parse =
        |s: &str| -> Option<i64> { RoutingTable::parse(s).ok().map(|t| t.as_value() as i64) };
    let checked = run_enum_file("nethelpers_routing_table.tsv", to_str, parse);
    assert_eq!(
        checked, 513,
        "nethelpers_routing_table.tsv: expected 513 records, checked {checked}"
    );
}

// machine_type.tsv mirrors machine.Type: String()/ParseType round-trip.
#[test]
fn machine_type_tsv() {
    use core::str::FromStr;
    let to_str = |v: i64| -> Option<String> {
        MachineType::from_i32(v as i32)
            .ok()
            .map(|t| t.as_str().to_string())
    };
    let parse =
        |s: &str| -> Option<i64> { MachineType::from_str(s).ok().map(|t| t.as_i32() as i64) };
    let checked = run_enum_file("machine_type.tsv", to_str, parse);
    assert_eq!(
        checked, 9,
        "machine_type.tsv: expected 9 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// config.tsv
//
// Each record names a v1alpha1 machine-config file under `configs/` and the
// four fields the Go oracle produced for it: the `valid`/`invalid` verdict, the
// `machineType`, the `hostname`, and the `installDisk`. For every record we load
// the matching config via `talos-machine-config` and assert the Rust-produced
// fields equal the Go columns byte-for-byte.
//
// TSV layout (5 columns, tab-separated, trailing LF):
//   {basename}<TAB>{valid|invalid}<TAB>{machineType}<TAB>{hostname}<TAB>{installDisk}
//
// Value columns may be empty and MUST NOT be trimmed.
// ---------------------------------------------------------------------------

/// Absolute path to a config file inside this crate's `configs/` directory.
fn config_path(name: &str) -> PathBuf {
    let mut p = fixtures_root();
    p.push("configs");
    p.push(name);
    p
}

#[test]
fn config_tsv() {
    let body = read_vector("config.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            5,
            "config.tsv line {}: expected 5 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let basename = cols[0];
        let expected_valid = cols[1];
        let expected_type = cols[2];
        let expected_hostname = cols[3];
        let expected_disk = cols[4];

        let path = config_path(basename);
        let source = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("config.tsv: failed to read config {}: {e}", path.display())
        });

        let (validity, mtype, hostname, disk) = load_record(&source);
        let actual_valid = match validity {
            Validity::Valid => "valid",
            Validity::Invalid(_) => "invalid",
        };

        assert_eq!(
            actual_valid, expected_valid,
            "config valid mismatch for {}\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
            basename, expected_valid, actual_valid
        );
        assert_eq!(
            mtype, expected_type,
            "config machineType mismatch for {}\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
            basename, expected_type, mtype
        );
        assert_eq!(
            hostname, expected_hostname,
            "config hostname mismatch for {}\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
            basename, expected_hostname, hostname
        );
        assert_eq!(
            disk, expected_disk,
            "config installDisk mismatch for {}\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
            basename, expected_disk, disk
        );

        checked += 1;
    }

    assert_eq!(
        checked, 20,
        "config.tsv: expected 20 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// config_fields.tsv
//
// Each record names a VALID v1alpha1 machine-config file under `configs/`, one
// of the oracle field paths, and that field's canonical string value. Only
// files that load AND validate in container mode are covered, and every covered
// file emits all 16 field paths (see `FIELD_PATH_ORDER`). For each record we
// resolve the same field via `CorpusConfig::config_field` and assert it equals
// the Go column byte-for-byte.
//
// TSV layout (3 columns, tab-separated, trailing LF):
//   {basename}<TAB>{fieldPath}<TAB>{canonicalValue}
//
// Value columns may be empty and MUST NOT be trimmed.
// ---------------------------------------------------------------------------

#[test]
fn config_fields_tsv() {
    let body = read_vector("config_fields.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(
            cols.len() == 2 || cols.len() == 3,
            "config_fields.tsv line {}: expected 2 or 3 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let basename = cols[0];
        let field_path = cols[1];
        let expected = cols.get(2).copied().unwrap_or("");

        let path = config_path(basename);
        let source = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "config_fields.tsv: failed to read config {}: {e}",
                path.display()
            )
        });

        let cfg = CorpusConfig::load(&source)
            .unwrap_or_else(|e| panic!("config_fields.tsv: failed to load {basename}: {e}"));
        let actual = cfg.config_field(field_path).unwrap_or_else(|| {
            panic!("config_fields.tsv: unknown field path {field_path:?} for {basename}")
        });

        assert_eq!(
            actual, expected,
            "config field mismatch for {} [{}]\n expected = {:?} (Go)\n actual   = {:?} (Rust)",
            basename, field_path, expected, actual
        );

        checked += 1;
    }

    // 15 valid files * 20 field paths each.
    assert_eq!(
        checked, 300,
        "config_fields.tsv: expected 300 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// cri_static_pod_contract.tsv
// ---------------------------------------------------------------------------

#[test]
fn cri_static_pod_contract_tsv() {
    let body = read_vector("cri_static_pod_contract.tsv");
    let mut checked = 0usize;

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            11,
            "cri_static_pod_contract.tsv line {}: expected 11 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );

        let node = NodeName::new(cols[1]).unwrap();
        let host_network = match cols[5] {
            "true" => true,
            "false" => false,
            other => panic!(
                "cri_static_pod_contract.tsv line {}: invalid host_network {other:?}",
                lineno + 1
            ),
        };
        let mut pods = vec![StaticPod {
            name: cols[2].to_string(),
            namespace: cols[4].to_string(),
            containers: vec![KubernetesContainer {
                name: cols[7].to_string(),
                image: cols[3].to_string(),
                command: vec![cols[7].to_string(), cols[8].to_string()],
                host_network,
            }],
            phase: StaticPodPhase::Pending,
        }];
        let mut runtime = CriRuntime::new();

        let reports = launch_static_pods_on_cri(&mut runtime, &mut pods, &node).unwrap();
        assert_eq!(reports.len(), 1);
        let report = &reports[0];
        assert_eq!(report.pod_name, cols[2], "case {} pod name", cols[0]);
        assert_eq!(report.namespace, cols[4], "case {} namespace", cols[0]);
        assert_eq!(
            report.host_network.to_string(),
            cols[5],
            "case {} hostNetwork",
            cols[0]
        );
        assert_eq!(report.pod_uid, cols[6], "case {} mirror UID", cols[0]);
        assert_eq!(
            report.images,
            vec![cols[9].to_string()],
            "case {} image",
            cols[0]
        );
        assert_eq!(
            format!("{:?}", pods[0].phase),
            cols[10],
            "case {} phase",
            cols[0]
        );

        let sandbox = runtime.pod_sandbox_status(&report.sandbox_id).unwrap();
        assert_eq!(sandbox.state, PodSandboxState::Ready);
        assert_eq!(sandbox.config.host_network.to_string(), cols[5]);
        let container = runtime.container_status(&report.container_ids[0]).unwrap();
        assert_eq!(container.state, CriContainerState::Running);
        assert_eq!(container.config.command, vec![cols[7].to_string()]);
        assert_eq!(container.config.args, vec![cols[8].to_string()]);
        checked += 1;
    }

    assert_eq!(
        checked, 3,
        "cri_static_pod_contract.tsv: expected 3 records, checked {checked}"
    );
}

// ---------------------------------------------------------------------------
// control_plane_pki_file_closure.tsv
// ---------------------------------------------------------------------------

fn control_plane_pki_cfg() -> K8sConfig {
    K8sConfig {
        node_name: NodeName::new("cp-1").unwrap(),
        cluster_domain: "cluster.local".into(),
        pod_cidrs: vec!["10.244.0.0/16".into()],
        service_cidrs: vec!["10.96.0.0/12".into()],
        endpoint: ClusterEndpoint::new("api.example.com", 6443).unwrap(),
        version: "1.30.0".into(),
        control_plane: true,
    }
}

fn control_plane_pki_secrets() -> K8sSecrets {
    let mut secrets = K8sSecrets::new();
    for name in REQUIRED_SECRETS {
        secrets.insert(*name, format!("VECTOR-{name}").into_bytes());
    }
    secrets
}

fn control_plane_referenced_pki_paths(cfg: K8sConfig) -> BTreeSet<String> {
    let cp = ControlPlaneConfig::new(cfg).unwrap();
    let mut referenced_pki_paths = BTreeSet::new();

    for component in K8sComponent::ALL {
        for arg in cp.args_for(component).unwrap() {
            let Some((_, value)) = arg.split_once('=') else {
                continue;
            };
            if value.starts_with("/etc/kubernetes/pki/") {
                referenced_pki_paths.insert(value.to_string());
            }
        }
    }

    referenced_pki_paths
}

fn control_plane_referenced_kubernetes_paths(cfg: K8sConfig) -> BTreeSet<String> {
    let cp = ControlPlaneConfig::new(cfg).unwrap();
    let mut referenced_paths = BTreeSet::new();

    for component in K8sComponent::ALL {
        for arg in cp.args_for(component).unwrap() {
            let Some((_, value)) = arg.split_once('=') else {
                continue;
            };
            if value.starts_with("/etc/kubernetes/") {
                referenced_paths.insert(value.to_string());
            }
        }
    }

    referenced_paths
}

fn kubeconfig_field<'a>(body: &'a str, field: &str) -> &'a str {
    let prefix = format!("    {field}: ");
    body.lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap_or_else(|| panic!("missing kubeconfig field {field}"))
}

#[test]
fn control_plane_pki_file_closure_tsv() {
    let body = read_vector("control_plane_pki_file_closure.tsv");
    let cfg = control_plane_pki_cfg();
    let secrets = control_plane_pki_secrets();
    let output = provision_control_plane(&cfg, &secrets, "talos").unwrap();
    let rendered: BTreeMap<&str, &os_kubernetes_domain::RenderedFile> = output
        .files()
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect();
    let referenced_pki_paths = control_plane_referenced_pki_paths(cfg);

    let mut checked = 0usize;
    let mut vector_paths = BTreeSet::new();
    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            4,
            "control_plane_pki_file_closure.tsv line {}: expected 4 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );
        let path = cols[0];
        let expected_mode = match cols[1] {
            "config" => FileMode::CONFIG,
            "secret" => FileMode::SECRET,
            other => panic!(
                "control_plane_pki_file_closure.tsv line {}: invalid mode {other:?}",
                lineno + 1
            ),
        };
        let expected_secret = cols[2];
        let expected_contents = format!("VECTOR-{expected_secret}");
        let expected_referenced_by = cols[3];
        let file = rendered
            .get(path)
            .unwrap_or_else(|| panic!("missing rendered PKI file {path}"));

        assert_eq!(file.mode, expected_mode, "{path} mode");
        assert_eq!(
            file.contents.as_slice(),
            expected_contents.as_bytes(),
            "{path} source secret"
        );
        assert_eq!(
            referenced_pki_paths.contains(path).to_string(),
            expected_referenced_by,
            "{path} control-plane arg reference flag"
        );
        vector_paths.insert(path.to_string());
        checked += 1;
    }

    assert_eq!(
        vector_paths, referenced_pki_paths,
        "control_plane_pki_file_closure.tsv must cover every PKI path referenced by control-plane args"
    );
    assert_eq!(
        checked, 14,
        "control_plane_pki_file_closure.tsv: expected 14 records, checked {checked}"
    );
}

fn generated_k8s_pki_projection(
    cfg: &K8sConfig,
) -> (SecretsBundle, KubernetesController, EtcdController) {
    let mut bundle = SecretsBundle::generate("wave59-generated-k8s-pki", 1000).unwrap();
    let mut sans = CertSans::new();
    sans.append(&cfg.endpoint.host).unwrap();
    sans.append(cfg.node_name.as_str()).unwrap();
    let mut k8s = KubernetesController::new(sans, &cfg.cluster_domain).unwrap();
    let mut etcd = EtcdController::new(
        cfg.node_name.as_str(),
        &[NodeAddress::parse("10.0.0.5").unwrap()],
    )
    .unwrap();
    k8s.reconcile(&mut bundle, 1000).unwrap();
    etcd.reconcile(&mut bundle, 1000).unwrap();
    (bundle, k8s, etcd)
}

fn generated_k8s_pki_expected_bytes(
    bundle: &SecretsBundle,
    k8s: &KubernetesController,
    etcd: &EtcdController,
    source_kind: &str,
    source_id: &str,
) -> Vec<u8> {
    match (source_kind, source_id) {
        ("ca-cert", "kubernetes") => bundle
            .ca(CaKind::Kubernetes)
            .certificate()
            .model_certificate_bytes(),
        ("ca-key", "kubernetes") => bundle
            .ca(CaKind::Kubernetes)
            .keypair()
            .model_private_key_bytes(),
        ("ca-cert", "etcd") => bundle
            .ca(CaKind::Etcd)
            .certificate()
            .model_certificate_bytes(),
        ("ca-cert", "aggregator") => bundle
            .ca(CaKind::Aggregator)
            .certificate()
            .model_certificate_bytes(),
        ("service-account", "private") => bundle.service_account_key().model_private_key_bytes(),
        ("service-account", "public") => bundle.service_account_key().model_public_key_bytes(),
        ("leaf-cert", "ApiServer") => k8s
            .certificate(SecretK8sCert::ApiServer)
            .unwrap()
            .model_certificate_bytes(),
        ("leaf-key", "ApiServer") => SecretK8sCert::ApiServer.keypair().model_private_key_bytes(),
        ("leaf-cert", "ApiServerEtcdClient") => etcd
            .certificate(EtcdCert::ApiServerClient)
            .unwrap()
            .model_certificate_bytes(),
        ("leaf-key", "ApiServerEtcdClient") => EtcdCert::ApiServerClient
            .keypair(etcd.node_name())
            .model_private_key_bytes(),
        ("leaf-cert", "FrontProxy") => k8s
            .certificate(SecretK8sCert::FrontProxy)
            .unwrap()
            .model_certificate_bytes(),
        ("leaf-key", "FrontProxy") => SecretK8sCert::FrontProxy
            .keypair()
            .model_private_key_bytes(),
        ("leaf-cert", "ApiServerKubeletClient") => k8s
            .certificate(SecretK8sCert::ApiServerKubeletClient)
            .unwrap()
            .model_certificate_bytes(),
        ("leaf-key", "ApiServerKubeletClient") => SecretK8sCert::ApiServerKubeletClient
            .keypair()
            .model_private_key_bytes(),
        ("leaf-cert", "ControllerManager") => k8s
            .certificate(SecretK8sCert::ControllerManager)
            .unwrap()
            .model_certificate_bytes(),
        ("leaf-key", "ControllerManager") => SecretK8sCert::ControllerManager
            .keypair()
            .model_private_key_bytes(),
        ("leaf-cert", "Scheduler") => k8s
            .certificate(SecretK8sCert::Scheduler)
            .unwrap()
            .model_certificate_bytes(),
        ("leaf-key", "Scheduler") => SecretK8sCert::Scheduler.keypair().model_private_key_bytes(),
        ("leaf-cert", "Admin") => k8s
            .certificate(SecretK8sCert::Admin)
            .unwrap()
            .model_certificate_bytes(),
        ("leaf-key", "Admin") => SecretK8sCert::Admin.keypair().model_private_key_bytes(),
        other => panic!("unknown generated PKI source contract {other:?}"),
    }
}

#[test]
fn generated_k8s_pki_secret_map_tsv() {
    let body = read_vector("generated_k8s_pki_secret_map.tsv");
    let cfg = control_plane_pki_cfg();
    let (bundle, k8s, etcd) = generated_k8s_pki_projection(&cfg);
    let entries = kubernetes_secret_entries(&bundle, &k8s, &etcd).unwrap();
    let projected_names: Vec<&str> = entries.iter().map(|entry| entry.name).collect();
    assert_eq!(projected_names, KUBERNETES_SECRET_PROJECTION_NAMES);

    let secrets =
        K8sSecrets::from_required_entries(entries.into_iter().map(|entry| entry.into_pair()))
            .unwrap();
    let output = provision_control_plane(&cfg, &secrets, "talos").unwrap();
    let rendered: BTreeMap<&str, &os_kubernetes_domain::RenderedFile> = output
        .files()
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect();
    let referenced_control_plane_paths = control_plane_referenced_kubernetes_paths(cfg);

    let mut checked = 0usize;
    let mut vector_referenced_paths = BTreeSet::new();
    let mut vector_secret_names = Vec::new();

    for (lineno, line) in records(&body).iter().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            cols.len(),
            7,
            "generated_k8s_pki_secret_map.tsv line {}: expected 7 columns, got {}: {:?}",
            lineno + 1,
            cols.len(),
            line
        );
        let secret_name = cols[0];
        let rendered_path = cols[1];
        let expected_mode = match cols[2] {
            "config" => FileMode::CONFIG,
            "secret" => FileMode::SECRET,
            other => panic!(
                "generated_k8s_pki_secret_map.tsv line {}: invalid mode {other:?}",
                lineno + 1
            ),
        };
        let expected_bytes =
            generated_k8s_pki_expected_bytes(&bundle, &k8s, &etcd, cols[3], cols[4]);
        let rendering = cols[5];
        let file = rendered
            .get(rendered_path)
            .unwrap_or_else(|| panic!("missing rendered generated PKI file {rendered_path}"));

        assert_eq!(
            secrets.get(secret_name),
            Some(expected_bytes.as_slice()),
            "{secret_name} generated secret source"
        );
        assert_eq!(file.mode, expected_mode, "{rendered_path} mode");
        let body = String::from_utf8_lossy(&file.contents);

        match rendering {
            "raw-file" => {
                assert!(
                    !rendered_path.ends_with(".conf"),
                    "raw-file rendering cannot target kubeconfig {rendered_path}"
                );
                assert_eq!(
                    file.contents.as_slice(),
                    expected_bytes.as_slice(),
                    "{rendered_path} rendered generated bytes"
                );
                assert!(
                    body.contains("KUBEROS-MODEL-"),
                    "{rendered_path} should contain generated model bytes"
                );
            }
            "kubeconfig-base64-ca"
            | "kubeconfig-base64-client-cert"
            | "kubeconfig-base64-client-key" => {
                assert!(
                    rendered_path.ends_with(".conf"),
                    "kubeconfig base64 rendering must target kubeconfig {rendered_path}"
                );
                let field = match rendering {
                    "kubeconfig-base64-ca" => "certificate-authority-data",
                    "kubeconfig-base64-client-cert" => "client-certificate-data",
                    "kubeconfig-base64-client-key" => "client-key-data",
                    _ => unreachable!(),
                };
                let value = kubeconfig_field(&body, field);
                assert_eq!(
                    value,
                    kubeconfig_data(expected_bytes.as_slice()),
                    "{rendered_path} generated kubeconfig {field} for {secret_name}"
                );
                assert!(
                    !value.contains("KUBEROS-MODEL-") && !value.contains("\\n"),
                    "{rendered_path} should base64-encode, not raw-escape, {secret_name}"
                );
                assert!(
                    !body.contains("KUBEROS-MODEL-"),
                    "{rendered_path} should not embed raw generated model bytes"
                );
                assert!(
                    String::from_utf8_lossy(&expected_bytes).contains("KUBEROS-MODEL-"),
                    "{secret_name} source bytes should remain explicit model material before encoding"
                );
                assert!(
                    !body.contains("client-certificate-data: CERT\n")
                        && !body.contains("client-key-data: KEY\n"),
                    "{rendered_path} should not contain placeholder kubeconfig credentials"
                );
            }
            other => panic!(
                "generated_k8s_pki_secret_map.tsv line {}: invalid rendering {other:?}",
                lineno + 1
            ),
        }

        assert_eq!(
            referenced_control_plane_paths
                .contains(rendered_path)
                .to_string(),
            cols[6],
            "{rendered_path} control-plane arg reference flag"
        );

        assert!(
            !body.contains("PEM-") && !body.contains("VECTOR-"),
            "{rendered_path} should not contain fixture placeholder bytes"
        );
        assert_ne!(file.contents.as_slice(), b"CERT", "{rendered_path}");
        assert_ne!(file.contents.as_slice(), b"KEY", "{rendered_path}");

        if vector_secret_names.last().copied() != Some(secret_name)
            && !vector_secret_names.contains(&secret_name)
        {
            vector_secret_names.push(secret_name);
        }
        if cols[6] == "true" {
            vector_referenced_paths.insert(rendered_path.to_string());
        }
        checked += 1;
    }

    assert_eq!(vector_secret_names, KUBERNETES_SECRET_PROJECTION_NAMES);
    assert_eq!(
        vector_referenced_paths, referenced_control_plane_paths,
        "generated_k8s_pki_secret_map.tsv must cover every generated secret-backed path referenced by control-plane args"
    );
    assert_eq!(
        checked, 23,
        "generated_k8s_pki_secret_map.tsv: expected 23 records, checked {checked}"
    );
}
