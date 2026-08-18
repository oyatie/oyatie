//! Network helper enums, ported 1:1 from
//! `pkg/machinery/nethelpers` in upstream Talos.
//!
//! Each enum reproduces the exact integer values from upstream and the exact
//! `String()` / parse mappings produced by Talos's `enumer`-generated code
//! (which uses the `-linecomment` text as the canonical string form). Every
//! enum exposes:
//!
//! - `to_str(self) -> &'static str`: the canonical string (the upstream
//!   `String()` line-comment value).
//! - `parse(s) -> Result<Self>`: parses the canonical string back to the value,
//!   matching upstream's `<Type>String(s)` generated function (case-insensitive
//!   on the canonical name, mirroring enumer's lowercase fallback).
//!
//! The numeric value of each variant is exposed via `as_value(self)` returning
//! the same width/value as the upstream Go constant.
//!
//! This module is `no_std` (uses only `core`/`alloc` and `os_kernel`).

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use os_kernel::error::{Error, Result};

/// Helper to build a consistent "unknown value" parse error.
fn unknown(type_name: &str, s: &str) -> Error {
    Error::parse(format!("{s} does not belong to {type_name} values"))
}

// ---------------------------------------------------------------------------
// BondMode (bondmode.go) — uint8, iota
// ---------------------------------------------------------------------------

/// A bond mode (`linux/if_bonding.h`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BondMode {
    /// balance-rr (round robin), value 0.
    Roundrobin,
    /// active-backup, value 1.
    ActiveBackup,
    /// balance-xor, value 2.
    Xor,
    /// broadcast, value 3.
    Broadcast,
    /// 802.3ad LACP, value 4.
    Ieee8023ad,
    /// balance-tlb, value 5.
    Tlb,
    /// balance-alb, value 6.
    Alb,
}

impl BondMode {
    /// Numeric value matching upstream `BondMode`.
    pub fn as_value(self) -> u8 {
        match self {
            BondMode::Roundrobin => 0,
            BondMode::ActiveBackup => 1,
            BondMode::Xor => 2,
            BondMode::Broadcast => 3,
            BondMode::Ieee8023ad => 4,
            BondMode::Tlb => 5,
            BondMode::Alb => 6,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            BondMode::Roundrobin => "balance-rr",
            BondMode::ActiveBackup => "active-backup",
            BondMode::Xor => "balance-xor",
            BondMode::Broadcast => "broadcast",
            BondMode::Ieee8023ad => "802.3ad",
            BondMode::Tlb => "balance-tlb",
            BondMode::Alb => "balance-alb",
        }
    }

    /// Parse from canonical string. Mirrors `BondModeByName`, which also accepts
    /// the empty string as a synonym for `balance-rr`.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "" | "balance-rr" => Ok(BondMode::Roundrobin),
            "active-backup" => Ok(BondMode::ActiveBackup),
            "balance-xor" => Ok(BondMode::Xor),
            "broadcast" => Ok(BondMode::Broadcast),
            "802.3ad" => Ok(BondMode::Ieee8023ad),
            "balance-tlb" => Ok(BondMode::Tlb),
            "balance-alb" => Ok(BondMode::Alb),
            other => Err(unknown("BondMode", other)),
        }
    }
}

// ---------------------------------------------------------------------------
// BondXmitHashPolicy (bondxmithashpolicy.go) — uint8, iota
// ---------------------------------------------------------------------------

/// A bond transmit hash policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BondXmitHashPolicy {
    /// layer2, value 0.
    Layer2,
    /// layer3+4, value 1.
    Layer34,
    /// layer2+3, value 2.
    Layer23,
    /// encap2+3, value 3.
    Encap23,
    /// encap3+4, value 4.
    Encap34,
}

impl BondXmitHashPolicy {
    /// Numeric value matching upstream `BondXmitHashPolicy`.
    pub fn as_value(self) -> u8 {
        match self {
            BondXmitHashPolicy::Layer2 => 0,
            BondXmitHashPolicy::Layer34 => 1,
            BondXmitHashPolicy::Layer23 => 2,
            BondXmitHashPolicy::Encap23 => 3,
            BondXmitHashPolicy::Encap34 => 4,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            BondXmitHashPolicy::Layer2 => "layer2",
            BondXmitHashPolicy::Layer34 => "layer3+4",
            BondXmitHashPolicy::Layer23 => "layer2+3",
            BondXmitHashPolicy::Encap23 => "encap2+3",
            BondXmitHashPolicy::Encap34 => "encap3+4",
        }
    }

    /// Parse from canonical string. Mirrors `BondXmitHashPolicyByName`, which
    /// accepts the empty string as a synonym for `layer2`.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "" | "layer2" => Ok(BondXmitHashPolicy::Layer2),
            "layer3+4" => Ok(BondXmitHashPolicy::Layer34),
            "layer2+3" => Ok(BondXmitHashPolicy::Layer23),
            "encap2+3" => Ok(BondXmitHashPolicy::Encap23),
            "encap3+4" => Ok(BondXmitHashPolicy::Encap34),
            other => Err(unknown("BondXmitHashPolicy", other)),
        }
    }
}

// ---------------------------------------------------------------------------
// ARPAllTargets (arpalltargets.go) — uint32, iota
// ---------------------------------------------------------------------------

/// An ARP all-targets mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpAllTargets {
    /// any, value 0.
    Any,
    /// all, value 1.
    All,
}

impl ArpAllTargets {
    /// Numeric value matching upstream `ARPAllTargets`.
    pub fn as_value(self) -> u32 {
        match self {
            ArpAllTargets::Any => 0,
            ArpAllTargets::All => 1,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            ArpAllTargets::Any => "any",
            ArpAllTargets::All => "all",
        }
    }

    /// Parse from canonical string. Mirrors `ARPAllTargetsByName`, which accepts
    /// the empty string as a synonym for `any`.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "" | "any" => Ok(ArpAllTargets::Any),
            "all" => Ok(ArpAllTargets::All),
            other => Err(unknown("ARPAllTargets", other)),
        }
    }
}

// ---------------------------------------------------------------------------
// ARPValidate (arpvalidate.go) — uint32, iota
// ---------------------------------------------------------------------------

/// An ARP validation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpValidate {
    /// none, value 0.
    None,
    /// active, value 1.
    Active,
    /// backup, value 2.
    Backup,
    /// all, value 3.
    All,
    /// filter, value 4.
    Filter,
    /// filter-active, value 5.
    FilterActive,
    /// filter-backup, value 6.
    FilterBackup,
}

impl ArpValidate {
    /// Numeric value matching upstream `ARPValidate`.
    pub fn as_value(self) -> u32 {
        match self {
            ArpValidate::None => 0,
            ArpValidate::Active => 1,
            ArpValidate::Backup => 2,
            ArpValidate::All => 3,
            ArpValidate::Filter => 4,
            ArpValidate::FilterActive => 5,
            ArpValidate::FilterBackup => 6,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            ArpValidate::None => "none",
            ArpValidate::Active => "active",
            ArpValidate::Backup => "backup",
            ArpValidate::All => "all",
            ArpValidate::Filter => "filter",
            ArpValidate::FilterActive => "filter-active",
            ArpValidate::FilterBackup => "filter-backup",
        }
    }

    /// Parse from canonical string. Mirrors `ARPValidateByName`, which accepts
    /// the empty string as a synonym for `none`.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "" | "none" => Ok(ArpValidate::None),
            "active" => Ok(ArpValidate::Active),
            "backup" => Ok(ArpValidate::Backup),
            "all" => Ok(ArpValidate::All),
            "filter" => Ok(ArpValidate::Filter),
            "filter-active" => Ok(ArpValidate::FilterActive),
            "filter-backup" => Ok(ArpValidate::FilterBackup),
            other => Err(unknown("ARPValidate", other)),
        }
    }
}

// ---------------------------------------------------------------------------
// AddressSortAlgorithm (addresssortalgorithm.go) — int, iota
// ---------------------------------------------------------------------------

/// Internal address sorting algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressSortAlgorithm {
    /// v1, value 0.
    V1,
    /// v2, value 1.
    V2,
}

impl AddressSortAlgorithm {
    /// Numeric value matching upstream `AddressSortAlgorithm`.
    pub fn as_value(self) -> i32 {
        match self {
            AddressSortAlgorithm::V1 => 0,
            AddressSortAlgorithm::V2 => 1,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            AddressSortAlgorithm::V1 => "v1",
            AddressSortAlgorithm::V2 => "v2",
        }
    }

    /// Parse from canonical string (mirrors `AddressSortAlgorithmString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "v1" => Ok(AddressSortAlgorithm::V1),
            "v2" => Ok(AddressSortAlgorithm::V2),
            _ => Err(unknown("AddressSortAlgorithm", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// ADSelect (adselect.go) — uint8, iota
// ---------------------------------------------------------------------------

/// 802.3ad aggregation selection logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdSelect {
    /// stable, value 0.
    Stable,
    /// bandwidth, value 1.
    Bandwidth,
    /// count, value 2.
    Count,
}

impl AdSelect {
    /// Numeric value matching upstream `ADSelect`.
    pub fn as_value(self) -> u8 {
        match self {
            AdSelect::Stable => 0,
            AdSelect::Bandwidth => 1,
            AdSelect::Count => 2,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            AdSelect::Stable => "stable",
            AdSelect::Bandwidth => "bandwidth",
            AdSelect::Count => "count",
        }
    }

    /// Parse from canonical string. Mirrors `ADSelectByName`, which accepts the
    /// empty string as a synonym for `stable`.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "" | "stable" => Ok(AdSelect::Stable),
            "bandwidth" => Ok(AdSelect::Bandwidth),
            "count" => Ok(AdSelect::Count),
            other => Err(unknown("ADSelect", other)),
        }
    }
}

// ---------------------------------------------------------------------------
// ADLACPActive (adlacpactive.go) — uint8, iota
// ---------------------------------------------------------------------------

/// 802.3ad LACP active flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdLacpActive {
    /// off, value 0.
    Off,
    /// on, value 1.
    On,
}

impl AdLacpActive {
    /// Numeric value matching upstream `ADLACPActive`.
    pub fn as_value(self) -> u8 {
        match self {
            AdLacpActive::Off => 0,
            AdLacpActive::On => 1,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            AdLacpActive::Off => "off",
            AdLacpActive::On => "on",
        }
    }

    /// Parse from canonical string (mirrors `ADLACPActiveString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "off" => Ok(AdLacpActive::Off),
            "on" => Ok(AdLacpActive::On),
            _ => Err(unknown("ADLACPActive", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// AutoHostnameKind (autohostnamekind.go) — byte, iota
// ---------------------------------------------------------------------------

/// Kind of automatically generated hostname.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoHostnameKind {
    /// off, value 0.
    Off,
    /// talos-addr, value 1 (legacy).
    Addr,
    /// stable, value 2.
    Stable,
}

impl AutoHostnameKind {
    /// Numeric value matching upstream `AutoHostnameKind`.
    pub fn as_value(self) -> u8 {
        match self {
            AutoHostnameKind::Off => 0,
            AutoHostnameKind::Addr => 1,
            AutoHostnameKind::Stable => 2,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            AutoHostnameKind::Off => "off",
            AutoHostnameKind::Addr => "talos-addr",
            AutoHostnameKind::Stable => "stable",
        }
    }

    /// Parse from canonical string (mirrors `AutoHostnameKindString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "off" => Ok(AutoHostnameKind::Off),
            "talos-addr" => Ok(AutoHostnameKind::Addr),
            "stable" => Ok(AutoHostnameKind::Stable),
            _ => Err(unknown("AutoHostnameKind", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// ClientIdentifier (client_identifier.go) — int, iota
// ---------------------------------------------------------------------------

/// A DHCP client identifier kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientIdentifier {
    /// none, value 0.
    None,
    /// mac, value 1.
    Mac,
    /// duid, value 2.
    Duid,
}

impl ClientIdentifier {
    /// Numeric value matching upstream `ClientIdentifier`.
    pub fn as_value(self) -> i32 {
        match self {
            ClientIdentifier::None => 0,
            ClientIdentifier::Mac => 1,
            ClientIdentifier::Duid => 2,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            ClientIdentifier::None => "none",
            ClientIdentifier::Mac => "mac",
            ClientIdentifier::Duid => "duid",
        }
    }

    /// Parse from canonical string (mirrors `ClientIdentifierString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "none" => Ok(ClientIdentifier::None),
            "mac" => Ok(ClientIdentifier::Mac),
            "duid" => Ok(ClientIdentifier::Duid),
            _ => Err(unknown("ClientIdentifier", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// ConntrackState (conntrack_state.go) — uint32, explicit
// ---------------------------------------------------------------------------

/// A conntrack state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConntrackState {
    /// new, value 0x08.
    New,
    /// related, value 0x04.
    Related,
    /// established, value 0x02.
    Established,
    /// invalid, value 0x01.
    Invalid,
}

impl ConntrackState {
    /// Numeric value matching upstream `ConntrackState`.
    pub fn as_value(self) -> u32 {
        match self {
            ConntrackState::New => 0x08,
            ConntrackState::Related => 0x04,
            ConntrackState::Established => 0x02,
            ConntrackState::Invalid => 0x01,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            ConntrackState::New => "new",
            ConntrackState::Related => "related",
            ConntrackState::Established => "established",
            ConntrackState::Invalid => "invalid",
        }
    }

    /// Parse from canonical string (mirrors `ConntrackStateString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "new" => Ok(ConntrackState::New),
            "related" => Ok(ConntrackState::Related),
            "established" => Ok(ConntrackState::Established),
            "invalid" => Ok(ConntrackState::Invalid),
            _ => Err(unknown("ConntrackState", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// DefaultAction (default_action.go) — int, iota
// ---------------------------------------------------------------------------

/// A default firewall action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultAction {
    /// accept, value 0.
    Accept,
    /// block, value 1.
    Block,
}

impl DefaultAction {
    /// Numeric value matching upstream `DefaultAction`.
    pub fn as_value(self) -> i32 {
        match self {
            DefaultAction::Accept => 0,
            DefaultAction::Block => 1,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            DefaultAction::Accept => "accept",
            DefaultAction::Block => "block",
        }
    }

    /// Parse from canonical string (mirrors `DefaultActionString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "accept" => Ok(DefaultAction::Accept),
            "block" => Ok(DefaultAction::Block),
            _ => Err(unknown("DefaultAction", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// DNSProtocol (dnsprotocol.go) — byte, iota
// ---------------------------------------------------------------------------

/// A kind of DNS protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsProtocol {
    /// Do53, value 0.
    Default,
    /// DoT, value 1.
    DnsOverTls,
    /// DoH, value 2.
    DnsOverHttp,
}

impl DnsProtocol {
    /// Numeric value matching upstream `DNSProtocol`.
    pub fn as_value(self) -> u8 {
        match self {
            DnsProtocol::Default => 0,
            DnsProtocol::DnsOverTls => 1,
            DnsProtocol::DnsOverHttp => 2,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            DnsProtocol::Default => "Do53",
            DnsProtocol::DnsOverTls => "DoT",
            DnsProtocol::DnsOverHttp => "DoH",
        }
    }

    /// Parse from canonical string (mirrors `DNSProtocolString`; enumer's
    /// lowercase fallback also accepts the all-lowercase form).
    pub fn parse(s: &str) -> Result<Self> {
        if s == "Do53" {
            return Ok(DnsProtocol::Default);
        }
        if s == "DoT" {
            return Ok(DnsProtocol::DnsOverTls);
        }
        if s == "DoH" {
            return Ok(DnsProtocol::DnsOverHttp);
        }
        match lower(s).as_str() {
            "do53" => Ok(DnsProtocol::Default),
            "dot" => Ok(DnsProtocol::DnsOverTls),
            "doh" => Ok(DnsProtocol::DnsOverHttp),
            _ => Err(unknown("DNSProtocol", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// Duplex (duplex.go) — wraps ethtool.Duplex
// ---------------------------------------------------------------------------

/// Link duplex (wraps `ethtool.Duplex`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duplex {
    /// Half, value 0x00.
    Half,
    /// Full, value 0x01.
    Full,
    /// Unknown, value 0xff.
    Unknown,
}

impl Duplex {
    /// Numeric value matching upstream `Duplex` / `ethtool.Duplex`.
    pub fn as_value(self) -> u8 {
        match self {
            Duplex::Half => 0x00,
            Duplex::Full => 0x01,
            Duplex::Unknown => 0xff,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            Duplex::Half => "Half",
            Duplex::Full => "Full",
            Duplex::Unknown => "Unknown",
        }
    }

    /// Parse from canonical string (mirrors `DuplexString`).
    pub fn parse(s: &str) -> Result<Self> {
        if s == "Half" {
            return Ok(Duplex::Half);
        }
        if s == "Full" {
            return Ok(Duplex::Full);
        }
        if s == "Unknown" {
            return Ok(Duplex::Unknown);
        }
        match lower(s).as_str() {
            "half" => Ok(Duplex::Half),
            "full" => Ok(Duplex::Full),
            "unknown" => Ok(Duplex::Unknown),
            _ => Err(unknown("Duplex", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// Port (port.go) — wraps ethtool.Port; no line comments => names are the strings
// ---------------------------------------------------------------------------

/// Physical port type (wraps `ethtool.Port`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Port {
    /// TwistedPair, value 0x00.
    TwistedPair,
    /// AUI, value 0x01.
    Aui,
    /// MII, value 0x02.
    Mii,
    /// Fibre, value 0x03.
    Fibre,
    /// BNC, value 0x04.
    Bnc,
    /// DirectAttach, value 0x05.
    DirectAttach,
    /// None, value 0xef.
    None,
    /// Other, value 0xff.
    Other,
}

impl Port {
    /// Numeric value matching upstream `Port` / `ethtool.Port`.
    pub fn as_value(self) -> u8 {
        match self {
            Port::TwistedPair => 0x00,
            Port::Aui => 0x01,
            Port::Mii => 0x02,
            Port::Fibre => 0x03,
            Port::Bnc => 0x04,
            Port::DirectAttach => 0x05,
            Port::None => 0xef,
            Port::Other => 0xff,
        }
    }

    /// Canonical string. The upstream constants carry no `-linecomment`, so
    /// enumer's `String()` uses the constant identifiers verbatim — except for
    /// `Fibre`, whose generated `String()` is the empty string (matching the
    /// oracle's `PortString`/`PortValues` output).
    pub fn to_str(self) -> &'static str {
        match self {
            Port::TwistedPair => "TwistedPair",
            Port::Aui => "AUI",
            Port::Mii => "MII",
            Port::Fibre => "",
            Port::Bnc => "BNC",
            Port::DirectAttach => "DirectAttach",
            Port::None => "None",
            Port::Other => "Other",
        }
    }

    /// Parse from canonical string (mirrors `PortString`).
    pub fn parse(s: &str) -> Result<Self> {
        for v in [
            Port::TwistedPair,
            Port::Aui,
            Port::Mii,
            Port::Fibre,
            Port::Bnc,
            Port::DirectAttach,
            Port::None,
            Port::Other,
        ] {
            if v.to_str() == s {
                return Ok(v);
            }
        }
        // enumer lowercase fallback
        let l = lower(s);
        for v in [
            Port::TwistedPair,
            Port::Aui,
            Port::Mii,
            Port::Fibre,
            Port::Bnc,
            Port::DirectAttach,
            Port::None,
            Port::Other,
        ] {
            if lower(v.to_str()) == l {
                return Ok(v);
            }
        }
        Err(unknown("Port", s))
    }
}

// ---------------------------------------------------------------------------
// OperationalState (operstate.go) — wraps rtnetlink.OperationalState (uint8)
// ---------------------------------------------------------------------------

/// Operational state of a link (wraps `rtnetlink.OperationalState`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationalState {
    /// unknown, value 0.
    Unknown,
    /// notPresent, value 1.
    NotPresent,
    /// down, value 2.
    Down,
    /// lowerLayerDown, value 3.
    LowerLayerDown,
    /// testing, value 4.
    Testing,
    /// dormant, value 5.
    Dormant,
    /// up, value 6.
    Up,
}

impl OperationalState {
    /// Numeric value matching upstream `OperationalState`.
    pub fn as_value(self) -> u8 {
        match self {
            OperationalState::Unknown => 0,
            OperationalState::NotPresent => 1,
            OperationalState::Down => 2,
            OperationalState::LowerLayerDown => 3,
            OperationalState::Testing => 4,
            OperationalState::Dormant => 5,
            OperationalState::Up => 6,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            OperationalState::Unknown => "unknown",
            OperationalState::NotPresent => "notPresent",
            OperationalState::Down => "down",
            OperationalState::LowerLayerDown => "lowerLayerDown",
            OperationalState::Testing => "testing",
            OperationalState::Dormant => "dormant",
            OperationalState::Up => "up",
        }
    }

    /// Parse from canonical string (mirrors `OperationalStateString`).
    pub fn parse(s: &str) -> Result<Self> {
        if let Some(v) = exact_or_lower(
            s,
            &[
                (OperationalState::Unknown, "unknown"),
                (OperationalState::NotPresent, "notPresent"),
                (OperationalState::Down, "down"),
                (OperationalState::LowerLayerDown, "lowerLayerDown"),
                (OperationalState::Testing, "testing"),
                (OperationalState::Dormant, "dormant"),
                (OperationalState::Up, "up"),
            ],
        ) {
            Ok(v)
        } else {
            Err(unknown("OperationalState", s))
        }
    }
}

// ---------------------------------------------------------------------------
// Scope (scope.go) — uint8, explicit
// ---------------------------------------------------------------------------

/// An address scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// global, value 0.
    Global,
    /// site, value 200.
    Site,
    /// link, value 253.
    Link,
    /// host, value 254.
    Host,
    /// nowhere, value 255.
    Nowhere,
}

impl Scope {
    /// Numeric value matching upstream `Scope`.
    pub fn as_value(self) -> u8 {
        match self {
            Scope::Global => 0,
            Scope::Site => 200,
            Scope::Link => 253,
            Scope::Host => 254,
            Scope::Nowhere => 255,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            Scope::Global => "global",
            Scope::Site => "site",
            Scope::Link => "link",
            Scope::Host => "host",
            Scope::Nowhere => "nowhere",
        }
    }

    /// Parse from canonical string (mirrors `ScopeString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "global" => Ok(Scope::Global),
            "site" => Ok(Scope::Site),
            "link" => Ok(Scope::Link),
            "host" => Ok(Scope::Host),
            "nowhere" => Ok(Scope::Nowhere),
            _ => Err(unknown("Scope", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// Status (status.go) — int, explicit (starts at 1)
// ---------------------------------------------------------------------------

/// A network status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// addresses, value 1.
    Addresses,
    /// connectivity, value 2.
    Connectivity,
    /// hostname, value 3.
    Hostname,
    /// etcfiles, value 4.
    EtcFiles,
}

impl Status {
    /// Numeric value matching upstream `Status`.
    pub fn as_value(self) -> i32 {
        match self {
            Status::Addresses => 1,
            Status::Connectivity => 2,
            Status::Hostname => 3,
            Status::EtcFiles => 4,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            Status::Addresses => "addresses",
            Status::Connectivity => "connectivity",
            Status::Hostname => "hostname",
            Status::EtcFiles => "etcfiles",
        }
    }

    /// Parse from canonical string (mirrors `StatusString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "addresses" => Ok(Status::Addresses),
            "connectivity" => Ok(Status::Connectivity),
            "hostname" => Ok(Status::Hostname),
            "etcfiles" => Ok(Status::EtcFiles),
            _ => Err(unknown("Status", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// Family (family.go) — uint8, explicit
// ---------------------------------------------------------------------------

/// A network address family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// inet4, value 2.
    Inet4,
    /// inet6, value 10.
    Inet6,
}

impl Family {
    /// Numeric value matching upstream `Family`.
    pub fn as_value(self) -> u8 {
        match self {
            Family::Inet4 => 2,
            Family::Inet6 => 10,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            Family::Inet4 => "inet4",
            Family::Inet6 => "inet6",
        }
    }

    /// Parse from canonical string (mirrors `FamilyString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "inet4" => Ok(Family::Inet4),
            "inet6" => Ok(Family::Inet6),
            _ => Err(unknown("Family", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// Protocol (protocol.go) — uint8, explicit
// ---------------------------------------------------------------------------

/// An inet protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// icmp, value 0x1.
    Icmp,
    /// tcp, value 0x6.
    Tcp,
    /// udp, value 0x11.
    Udp,
    /// icmpv6, value 0x3a.
    Icmpv6,
}

impl Protocol {
    /// Numeric value matching upstream `Protocol`.
    pub fn as_value(self) -> u8 {
        match self {
            Protocol::Icmp => 0x1,
            Protocol::Tcp => 0x6,
            Protocol::Udp => 0x11,
            Protocol::Icmpv6 => 0x3a,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            Protocol::Icmp => "icmp",
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
            Protocol::Icmpv6 => "icmpv6",
        }
    }

    /// Parse from canonical string (mirrors `ProtocolString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "icmp" => Ok(Protocol::Icmp),
            "tcp" => Ok(Protocol::Tcp),
            "udp" => Ok(Protocol::Udp),
            "icmpv6" => Ok(Protocol::Icmpv6),
            _ => Err(unknown("Protocol", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// RouteType (routetype.go) — uint8, iota
// ---------------------------------------------------------------------------

/// A route type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteType {
    /// unspec, value 0.
    Unspec,
    /// unicast, value 1.
    Unicast,
    /// local, value 2.
    Local,
    /// broadcast, value 3.
    Broadcast,
    /// anycast, value 4.
    Anycast,
    /// multicast, value 5.
    Multicast,
    /// blackhole, value 6.
    Blackhole,
    /// unreachable, value 7.
    Unreachable,
    /// prohibit, value 8.
    Prohibit,
    /// throw, value 9.
    Throw,
    /// nat, value 10.
    Nat,
    /// xresolve, value 11.
    XResolve,
}

impl RouteType {
    /// Numeric value matching upstream `RouteType`.
    pub fn as_value(self) -> u8 {
        match self {
            RouteType::Unspec => 0,
            RouteType::Unicast => 1,
            RouteType::Local => 2,
            RouteType::Broadcast => 3,
            RouteType::Anycast => 4,
            RouteType::Multicast => 5,
            RouteType::Blackhole => 6,
            RouteType::Unreachable => 7,
            RouteType::Prohibit => 8,
            RouteType::Throw => 9,
            RouteType::Nat => 10,
            RouteType::XResolve => 11,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            RouteType::Unspec => "unspec",
            RouteType::Unicast => "unicast",
            RouteType::Local => "local",
            RouteType::Broadcast => "broadcast",
            RouteType::Anycast => "anycast",
            RouteType::Multicast => "multicast",
            RouteType::Blackhole => "blackhole",
            RouteType::Unreachable => "unreachable",
            RouteType::Prohibit => "prohibit",
            RouteType::Throw => "throw",
            RouteType::Nat => "nat",
            RouteType::XResolve => "xresolve",
        }
    }

    /// Parse from canonical string (mirrors `RouteTypeString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "unspec" => Ok(RouteType::Unspec),
            "unicast" => Ok(RouteType::Unicast),
            "local" => Ok(RouteType::Local),
            "broadcast" => Ok(RouteType::Broadcast),
            "anycast" => Ok(RouteType::Anycast),
            "multicast" => Ok(RouteType::Multicast),
            "blackhole" => Ok(RouteType::Blackhole),
            "unreachable" => Ok(RouteType::Unreachable),
            "prohibit" => Ok(RouteType::Prohibit),
            "throw" => Ok(RouteType::Throw),
            "nat" => Ok(RouteType::Nat),
            "xresolve" => Ok(RouteType::XResolve),
            _ => Err(unknown("RouteType", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// RouteProtocol (routeprotocol.go) — uint8, explicit
// ---------------------------------------------------------------------------

/// A routing protocol (`RTPROT_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteProtocol {
    /// unspec, value 0.
    Unspec,
    /// redirect, value 1.
    Redirect,
    /// kernel, value 2.
    Kernel,
    /// boot, value 3.
    Boot,
    /// static, value 4.
    Static,
    /// ra, value 9.
    Ra,
    /// mrt, value 10.
    Mrt,
    /// zebra, value 11.
    Zebra,
    /// bird, value 12.
    Bird,
    /// dnrouted, value 13.
    Dnrouted,
    /// xorp, value 14.
    Xorp,
    /// ntk, value 15.
    Ntk,
    /// dhcp, value 16.
    Dhcp,
    /// mrtd, value 17.
    Mrtd,
    /// keepalived, value 18.
    Keepalived,
    /// babel, value 42.
    Babel,
    /// openr, value 99.
    Openr,
    /// bgp, value 186.
    Bgp,
    /// isis, value 187.
    Isis,
    /// ospf, value 188.
    Ospf,
    /// rip, value 189.
    Rip,
    /// eigrp, value 192.
    Eigrp,
}

impl RouteProtocol {
    /// Numeric value matching upstream `RouteProtocol`.
    pub fn as_value(self) -> u8 {
        match self {
            RouteProtocol::Unspec => 0,
            RouteProtocol::Redirect => 1,
            RouteProtocol::Kernel => 2,
            RouteProtocol::Boot => 3,
            RouteProtocol::Static => 4,
            RouteProtocol::Ra => 9,
            RouteProtocol::Mrt => 10,
            RouteProtocol::Zebra => 11,
            RouteProtocol::Bird => 12,
            RouteProtocol::Dnrouted => 13,
            RouteProtocol::Xorp => 14,
            RouteProtocol::Ntk => 15,
            RouteProtocol::Dhcp => 16,
            RouteProtocol::Mrtd => 17,
            RouteProtocol::Keepalived => 18,
            RouteProtocol::Babel => 42,
            RouteProtocol::Openr => 99,
            RouteProtocol::Bgp => 186,
            RouteProtocol::Isis => 187,
            RouteProtocol::Ospf => 188,
            RouteProtocol::Rip => 189,
            RouteProtocol::Eigrp => 192,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            RouteProtocol::Unspec => "unspec",
            RouteProtocol::Redirect => "redirect",
            RouteProtocol::Kernel => "kernel",
            RouteProtocol::Boot => "boot",
            RouteProtocol::Static => "static",
            RouteProtocol::Ra => "ra",
            RouteProtocol::Mrt => "mrt",
            RouteProtocol::Zebra => "zebra",
            RouteProtocol::Bird => "bird",
            RouteProtocol::Dnrouted => "dnrouted",
            RouteProtocol::Xorp => "xorp",
            RouteProtocol::Ntk => "ntk",
            RouteProtocol::Dhcp => "dhcp",
            RouteProtocol::Mrtd => "mrtd",
            RouteProtocol::Keepalived => "keepalived",
            RouteProtocol::Babel => "babel",
            RouteProtocol::Openr => "openr",
            RouteProtocol::Bgp => "bgp",
            RouteProtocol::Isis => "isis",
            RouteProtocol::Ospf => "ospf",
            RouteProtocol::Rip => "rip",
            RouteProtocol::Eigrp => "eigrp",
        }
    }

    /// Parse from canonical string (mirrors `RouteProtocolString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "unspec" => Ok(RouteProtocol::Unspec),
            "redirect" => Ok(RouteProtocol::Redirect),
            "kernel" => Ok(RouteProtocol::Kernel),
            "boot" => Ok(RouteProtocol::Boot),
            "static" => Ok(RouteProtocol::Static),
            "ra" => Ok(RouteProtocol::Ra),
            "mrt" => Ok(RouteProtocol::Mrt),
            "zebra" => Ok(RouteProtocol::Zebra),
            "bird" => Ok(RouteProtocol::Bird),
            "dnrouted" => Ok(RouteProtocol::Dnrouted),
            "xorp" => Ok(RouteProtocol::Xorp),
            "ntk" => Ok(RouteProtocol::Ntk),
            "dhcp" => Ok(RouteProtocol::Dhcp),
            "mrtd" => Ok(RouteProtocol::Mrtd),
            "keepalived" => Ok(RouteProtocol::Keepalived),
            "babel" => Ok(RouteProtocol::Babel),
            "openr" => Ok(RouteProtocol::Openr),
            "bgp" => Ok(RouteProtocol::Bgp),
            "isis" => Ok(RouteProtocol::Isis),
            "ospf" => Ok(RouteProtocol::Ospf),
            "rip" => Ok(RouteProtocol::Rip),
            "eigrp" => Ok(RouteProtocol::Eigrp),
            _ => Err(unknown("RouteProtocol", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// RoutingRuleAction (routingruleaction.go) — uint8, explicit
// ---------------------------------------------------------------------------

/// A routing rule action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingRuleAction {
    /// unspec, value 0.
    Unspec,
    /// unicast, value 1.
    Unicast,
    /// blackhole, value 6.
    Blackhole,
    /// unreachable, value 7.
    Unreachable,
    /// prohibit, value 8.
    Prohibit,
}

impl RoutingRuleAction {
    /// Numeric value matching upstream `RoutingRuleAction`.
    pub fn as_value(self) -> u8 {
        match self {
            RoutingRuleAction::Unspec => 0,
            RoutingRuleAction::Unicast => 1,
            RoutingRuleAction::Blackhole => 6,
            RoutingRuleAction::Unreachable => 7,
            RoutingRuleAction::Prohibit => 8,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            RoutingRuleAction::Unspec => "unspec",
            RoutingRuleAction::Unicast => "unicast",
            RoutingRuleAction::Blackhole => "blackhole",
            RoutingRuleAction::Unreachable => "unreachable",
            RoutingRuleAction::Prohibit => "prohibit",
        }
    }

    /// Parse from canonical string (mirrors `RoutingRuleActionString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "unspec" => Ok(RoutingRuleAction::Unspec),
            "unicast" => Ok(RoutingRuleAction::Unicast),
            "blackhole" => Ok(RoutingRuleAction::Blackhole),
            "unreachable" => Ok(RoutingRuleAction::Unreachable),
            "prohibit" => Ok(RoutingRuleAction::Prohibit),
            _ => Err(unknown("RoutingRuleAction", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// LACPRate (lacprate.go) — uint8, iota
// ---------------------------------------------------------------------------

/// A LACP rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LacpRate {
    /// slow, value 0.
    Slow,
    /// fast, value 1.
    Fast,
}

impl LacpRate {
    /// Numeric value matching upstream `LACPRate`.
    pub fn as_value(self) -> u8 {
        match self {
            LacpRate::Slow => 0,
            LacpRate::Fast => 1,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            LacpRate::Slow => "slow",
            LacpRate::Fast => "fast",
        }
    }

    /// Parse from canonical string. Mirrors `LACPRateByName`, which accepts the
    /// empty string as a synonym for `slow`.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "" | "slow" => Ok(LacpRate::Slow),
            "fast" => Ok(LacpRate::Fast),
            other => Err(unknown("LACPRate", other)),
        }
    }
}

// ---------------------------------------------------------------------------
// MatchOperator (match_operator.go) — int, explicit
// ---------------------------------------------------------------------------

/// A netfilter match operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchOperator {
    /// ==, value 0.
    Equal,
    /// !=, value 1.
    NotEqual,
}

impl MatchOperator {
    /// Numeric value matching upstream `MatchOperator`.
    pub fn as_value(self) -> i32 {
        match self {
            MatchOperator::Equal => 0,
            MatchOperator::NotEqual => 1,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            MatchOperator::Equal => "==",
            MatchOperator::NotEqual => "!=",
        }
    }

    /// Parse from canonical string (mirrors `MatchOperatorString`).
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "==" => Ok(MatchOperator::Equal),
            "!=" => Ok(MatchOperator::NotEqual),
            other => Err(unknown("MatchOperator", other)),
        }
    }
}

// ---------------------------------------------------------------------------
// NfTablesChainHook (nftables_chain_hook.go) — uint32, explicit
// ---------------------------------------------------------------------------

/// An nftables base-chain hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfTablesChainHook {
    /// prerouting, value 0.
    Prerouting,
    /// input, value 1.
    Input,
    /// forward, value 2.
    Forward,
    /// output, value 3.
    Output,
    /// postrouting, value 4.
    Postrouting,
}

impl NfTablesChainHook {
    /// Numeric value matching upstream `NfTablesChainHook`.
    pub fn as_value(self) -> u32 {
        match self {
            NfTablesChainHook::Prerouting => 0,
            NfTablesChainHook::Input => 1,
            NfTablesChainHook::Forward => 2,
            NfTablesChainHook::Output => 3,
            NfTablesChainHook::Postrouting => 4,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            NfTablesChainHook::Prerouting => "prerouting",
            NfTablesChainHook::Input => "input",
            NfTablesChainHook::Forward => "forward",
            NfTablesChainHook::Output => "output",
            NfTablesChainHook::Postrouting => "postrouting",
        }
    }

    /// Parse from canonical string (mirrors `NfTablesChainHookString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "prerouting" => Ok(NfTablesChainHook::Prerouting),
            "input" => Ok(NfTablesChainHook::Input),
            "forward" => Ok(NfTablesChainHook::Forward),
            "output" => Ok(NfTablesChainHook::Output),
            "postrouting" => Ok(NfTablesChainHook::Postrouting),
            _ => Err(unknown("NfTablesChainHook", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// NfTablesChainPriority (nftables_chain_priority.go) — int32, explicit
// ---------------------------------------------------------------------------

/// An nftables base-chain priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfTablesChainPriority {
    /// first, value i32::MIN.
    First,
    /// conntrack-defrag, value -400.
    ConntrackDefrag,
    /// raw, value -300.
    Raw,
    /// selinux-first, value -225.
    SelinuxFirst,
    /// conntrack, value -200.
    Conntrack,
    /// mangle, value -150.
    Mangle,
    /// nat-dest, value -100.
    NatDest,
    /// filter, value 0.
    Filter,
    /// security, value 50.
    Security,
    /// nat-source, value 100.
    NatSource,
    /// selinux-last, value 225.
    SelinuxLast,
    /// conntrack-helper, value 300.
    ConntrackHelper,
    /// last, value i32::MAX.
    Last,
}

impl NfTablesChainPriority {
    /// Numeric value matching upstream `NfTablesChainPriority`.
    pub fn as_value(self) -> i32 {
        match self {
            NfTablesChainPriority::First => i32::MIN,
            NfTablesChainPriority::ConntrackDefrag => -400,
            NfTablesChainPriority::Raw => -300,
            NfTablesChainPriority::SelinuxFirst => -225,
            NfTablesChainPriority::Conntrack => -200,
            NfTablesChainPriority::Mangle => -150,
            NfTablesChainPriority::NatDest => -100,
            NfTablesChainPriority::Filter => 0,
            NfTablesChainPriority::Security => 50,
            NfTablesChainPriority::NatSource => 100,
            NfTablesChainPriority::SelinuxLast => 225,
            NfTablesChainPriority::ConntrackHelper => 300,
            NfTablesChainPriority::Last => i32::MAX,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            NfTablesChainPriority::First => "first",
            NfTablesChainPriority::ConntrackDefrag => "conntrack-defrag",
            NfTablesChainPriority::Raw => "raw",
            NfTablesChainPriority::SelinuxFirst => "selinux-first",
            NfTablesChainPriority::Conntrack => "conntrack",
            NfTablesChainPriority::Mangle => "mangle",
            NfTablesChainPriority::NatDest => "nat-dest",
            NfTablesChainPriority::Filter => "filter",
            NfTablesChainPriority::Security => "security",
            NfTablesChainPriority::NatSource => "nat-source",
            NfTablesChainPriority::SelinuxLast => "selinux-last",
            NfTablesChainPriority::ConntrackHelper => "conntrack-helper",
            NfTablesChainPriority::Last => "last",
        }
    }

    /// Parse from canonical string (mirrors `NfTablesChainPriorityString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "first" => Ok(NfTablesChainPriority::First),
            "conntrack-defrag" => Ok(NfTablesChainPriority::ConntrackDefrag),
            "raw" => Ok(NfTablesChainPriority::Raw),
            "selinux-first" => Ok(NfTablesChainPriority::SelinuxFirst),
            "conntrack" => Ok(NfTablesChainPriority::Conntrack),
            "mangle" => Ok(NfTablesChainPriority::Mangle),
            "nat-dest" => Ok(NfTablesChainPriority::NatDest),
            "filter" => Ok(NfTablesChainPriority::Filter),
            "security" => Ok(NfTablesChainPriority::Security),
            "nat-source" => Ok(NfTablesChainPriority::NatSource),
            "selinux-last" => Ok(NfTablesChainPriority::SelinuxLast),
            "conntrack-helper" => Ok(NfTablesChainPriority::ConntrackHelper),
            "last" => Ok(NfTablesChainPriority::Last),
            _ => Err(unknown("NfTablesChainPriority", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// NfTablesVerdict (nftables_verdict.go) — int64, explicit
// ---------------------------------------------------------------------------

/// An nftables verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfTablesVerdict {
    /// drop, value 0.
    Drop,
    /// accept, value 1.
    Accept,
}

impl NfTablesVerdict {
    /// Numeric value matching upstream `NfTablesVerdict`.
    pub fn as_value(self) -> i64 {
        match self {
            NfTablesVerdict::Drop => 0,
            NfTablesVerdict::Accept => 1,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            NfTablesVerdict::Drop => "drop",
            NfTablesVerdict::Accept => "accept",
        }
    }

    /// Parse from canonical string (mirrors `NfTablesVerdictString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "drop" => Ok(NfTablesVerdict::Drop),
            "accept" => Ok(NfTablesVerdict::Accept),
            _ => Err(unknown("NfTablesVerdict", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// NfTablesChainType (nftables_chain_type.go) — string alias
// ---------------------------------------------------------------------------

/// An nftables base-chain type. Upstream models this as a `string` alias, so
/// the canonical string *is* the value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfTablesChainType {
    /// filter.
    Filter,
    /// route.
    Route,
    /// nat.
    Nat,
}

impl NfTablesChainType {
    /// Canonical string (the upstream string value).
    pub fn to_str(self) -> &'static str {
        match self {
            NfTablesChainType::Filter => "filter",
            NfTablesChainType::Route => "route",
            NfTablesChainType::Nat => "nat",
        }
    }

    /// Parse from the string value.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "filter" => Ok(NfTablesChainType::Filter),
            "route" => Ok(NfTablesChainType::Route),
            "nat" => Ok(NfTablesChainType::Nat),
            other => Err(unknown("NfTablesChainType", other)),
        }
    }
}

// ---------------------------------------------------------------------------
// PrimaryReselect (primaryreselect.go) — uint8, iota
// ---------------------------------------------------------------------------

/// A bond primary-reselect mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryReselect {
    /// always, value 0.
    Always,
    /// better, value 1.
    Better,
    /// failure, value 2.
    Failure,
}

impl PrimaryReselect {
    /// Numeric value matching upstream `PrimaryReselect`.
    pub fn as_value(self) -> u8 {
        match self {
            PrimaryReselect::Always => 0,
            PrimaryReselect::Better => 1,
            PrimaryReselect::Failure => 2,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            PrimaryReselect::Always => "always",
            PrimaryReselect::Better => "better",
            PrimaryReselect::Failure => "failure",
        }
    }

    /// Parse from canonical string. Mirrors `PrimaryReselectByName`, which
    /// accepts the empty string as a synonym for `always`.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "" | "always" => Ok(PrimaryReselect::Always),
            "better" => Ok(PrimaryReselect::Better),
            "failure" => Ok(PrimaryReselect::Failure),
            other => Err(unknown("PrimaryReselect", other)),
        }
    }
}

// ---------------------------------------------------------------------------
// VLANProtocol (vlanprotocol.go) — uint16, explicit
// ---------------------------------------------------------------------------

/// A VLAN protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VlanProtocol {
    /// 802.1q, value 33024.
    Ieee8021q,
    /// 802.1ad, value 34984.
    Ieee8021ad,
}

impl VlanProtocol {
    /// Numeric value matching upstream `VLANProtocol`.
    pub fn as_value(self) -> u16 {
        match self {
            VlanProtocol::Ieee8021q => 33024,
            VlanProtocol::Ieee8021ad => 34984,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            VlanProtocol::Ieee8021q => "802.1q",
            VlanProtocol::Ieee8021ad => "802.1ad",
        }
    }

    /// Parse from canonical string (mirrors `VLANProtocolString`).
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "802.1q" => Ok(VlanProtocol::Ieee8021q),
            "802.1ad" => Ok(VlanProtocol::Ieee8021ad),
            other => Err(unknown("VLANProtocol", other)),
        }
    }
}

// ---------------------------------------------------------------------------
// WOLMode (wol.go) — wraps ethtool.WOLMode (single-value enumer enum)
// ---------------------------------------------------------------------------

/// A Wake-on-LAN mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WolMode {
    /// phy, value 1.
    Phy,
    /// unicast, value 2.
    Unicast,
    /// multicast, value 4.
    Multicast,
    /// broadcast, value 8.
    Broadcast,
    /// magic, value 32.
    Magic,
    /// magicsecure, value 64.
    MagicSecure,
    /// filter, value 128.
    Filter,
}

impl WolMode {
    /// Numeric value matching upstream `WOLMode` / `ethtool.WOLMode`.
    pub fn as_value(self) -> i32 {
        match self {
            WolMode::Phy => 1 << 0,
            WolMode::Unicast => 1 << 1,
            WolMode::Multicast => 1 << 2,
            WolMode::Broadcast => 1 << 3,
            WolMode::Magic => 1 << 5,
            WolMode::MagicSecure => 1 << 6,
            WolMode::Filter => 1 << 7,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            WolMode::Phy => "phy",
            WolMode::Unicast => "unicast",
            WolMode::Multicast => "multicast",
            WolMode::Broadcast => "broadcast",
            WolMode::Magic => "magic",
            WolMode::MagicSecure => "magicsecure",
            WolMode::Filter => "filter",
        }
    }

    /// Parse from canonical string (mirrors `WOLModeString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "phy" => Ok(WolMode::Phy),
            "unicast" => Ok(WolMode::Unicast),
            "multicast" => Ok(WolMode::Multicast),
            "broadcast" => Ok(WolMode::Broadcast),
            "magic" => Ok(WolMode::Magic),
            "magicsecure" => Ok(WolMode::MagicSecure),
            "filter" => Ok(WolMode::Filter),
            _ => Err(unknown("WOLMode", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// FailOverMAC (failovermac.go) — uint8, iota
// ---------------------------------------------------------------------------

/// A MAC failover mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailOverMac {
    /// none, value 0.
    None,
    /// active, value 1.
    Active,
    /// follow, value 2.
    Follow,
}

impl FailOverMac {
    /// Numeric value matching upstream `FailOverMAC`.
    pub fn as_value(self) -> u8 {
        match self {
            FailOverMac::None => 0,
            FailOverMac::Active => 1,
            FailOverMac::Follow => 2,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            FailOverMac::None => "none",
            FailOverMac::Active => "active",
            FailOverMac::Follow => "follow",
        }
    }

    /// Parse from canonical string. Mirrors `FailOverMACString` (case-insensitive
    /// canonical name); `FailOverMACByName` additionally accepts the empty
    /// string as a synonym for `none`.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "" | "none" => Ok(FailOverMac::None),
            "active" => Ok(FailOverMac::Active),
            "follow" => Ok(FailOverMac::Follow),
            _ => match lower(s).as_str() {
                "none" => Ok(FailOverMac::None),
                "active" => Ok(FailOverMac::Active),
                "follow" => Ok(FailOverMac::Follow),
                _ => Err(unknown("FailOverMAC", s)),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// ICMPType (icmp_type.go) — byte, explicit
// ---------------------------------------------------------------------------

/// An ICMP packet type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IcmpType {
    /// timestamp-request, value 13.
    TimestampRequest,
    /// timestamp-reply, value 14.
    TimestampReply,
    /// address-mask-request, value 17.
    AddressMaskRequest,
    /// address-mask-reply, value 18.
    AddressMaskReply,
}

impl IcmpType {
    /// Numeric value matching upstream `ICMPType`.
    pub fn as_value(self) -> u8 {
        match self {
            IcmpType::TimestampRequest => 13,
            IcmpType::TimestampReply => 14,
            IcmpType::AddressMaskRequest => 17,
            IcmpType::AddressMaskReply => 18,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            IcmpType::TimestampRequest => "timestamp-request",
            IcmpType::TimestampReply => "timestamp-reply",
            IcmpType::AddressMaskRequest => "address-mask-request",
            IcmpType::AddressMaskReply => "address-mask-reply",
        }
    }

    /// Parse from canonical string (mirrors `ICMPTypeString`).
    pub fn parse(s: &str) -> Result<Self> {
        match lower_match(s).as_str() {
            "timestamp-request" => Ok(IcmpType::TimestampRequest),
            "timestamp-reply" => Ok(IcmpType::TimestampReply),
            "address-mask-request" => Ok(IcmpType::AddressMaskRequest),
            "address-mask-reply" => Ok(IcmpType::AddressMaskReply),
            _ => Err(unknown("ICMPType", s)),
        }
    }
}

// ---------------------------------------------------------------------------
// AddressFlag (address_flags.go) — uint32 bitmask, 1<<iota; individual flags
// ---------------------------------------------------------------------------

/// A single address attribute flag (`IFA_F_*`).
///
/// Mirrors the individual `AddressFlag` constants; the bitmask aggregate type
/// `AddressFlags` is modeled separately via [`address_flags_to_string`] /
/// [`address_flags_from_string`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFlag {
    /// temporary, value 1.
    Temporary,
    /// nodad, value 2.
    NoDad,
    /// optimistic, value 4.
    Optimistic,
    /// dadfailed, value 8.
    DadFailed,
    /// homeaddress, value 16.
    Home,
    /// deprecated, value 32.
    Deprecated,
    /// tentative, value 64.
    Tentative,
    /// permanent, value 128.
    Permanent,
    /// mngmtmpaddr, value 256.
    ManagementTemp,
    /// noprefixroute, value 512.
    NoPrefixRoute,
    /// mcautojoin, value 1024.
    McAutoJoin,
    /// stableprivacy, value 2048.
    StablePrivacy,
}

impl AddressFlag {
    /// All flags in ascending bit order (`Temporary..=StablePrivacy`).
    pub const ALL: [AddressFlag; 12] = [
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
    ];

    /// Numeric value matching upstream `AddressFlag`.
    pub fn as_value(self) -> u32 {
        match self {
            AddressFlag::Temporary => 1 << 0,
            AddressFlag::NoDad => 1 << 1,
            AddressFlag::Optimistic => 1 << 2,
            AddressFlag::DadFailed => 1 << 3,
            AddressFlag::Home => 1 << 4,
            AddressFlag::Deprecated => 1 << 5,
            AddressFlag::Tentative => 1 << 6,
            AddressFlag::Permanent => 1 << 7,
            AddressFlag::ManagementTemp => 1 << 8,
            AddressFlag::NoPrefixRoute => 1 << 9,
            AddressFlag::McAutoJoin => 1 << 10,
            AddressFlag::StablePrivacy => 1 << 11,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            AddressFlag::Temporary => "temporary",
            AddressFlag::NoDad => "nodad",
            AddressFlag::Optimistic => "optimistic",
            AddressFlag::DadFailed => "dadfailed",
            AddressFlag::Home => "homeaddress",
            AddressFlag::Deprecated => "deprecated",
            AddressFlag::Tentative => "tentative",
            AddressFlag::Permanent => "permanent",
            AddressFlag::ManagementTemp => "mngmtmpaddr",
            AddressFlag::NoPrefixRoute => "noprefixroute",
            AddressFlag::McAutoJoin => "mcautojoin",
            AddressFlag::StablePrivacy => "stableprivacy",
        }
    }

    /// Parse from canonical string (mirrors `AddressFlagString`).
    pub fn parse(s: &str) -> Result<Self> {
        let l = lower_match(s);
        for v in AddressFlag::ALL {
            if v.to_str() == l {
                return Ok(v);
            }
        }
        Err(unknown("AddressFlag", s))
    }
}

/// Render an `AddressFlags` bitmask the same way as upstream
/// `AddressFlags.String()`: comma-joined canonical names in ascending bit order.
pub fn address_flags_to_string(flags: u32) -> String {
    join_flags(flags, &AddressFlag::ALL, |f| (f.as_value(), f.to_str()))
}

/// Parse an `AddressFlags` bitmask from a comma-separated string, mirroring
/// upstream `AddressFlagsString`.
pub fn address_flags_from_string(s: &str) -> Result<u32> {
    let mut flags = 0u32;
    for part in s.split(',') {
        flags |= AddressFlag::parse(part)?.as_value();
    }
    Ok(flags)
}

// ---------------------------------------------------------------------------
// LinkFlag (linkflag.go) — uint32 bitmask, 1<<iota; individual flags
// ---------------------------------------------------------------------------

/// A single link flag (`IFF_*`).
///
/// Mirrors the individual `LinkFlag` constants; the bitmask aggregate type
/// `LinkFlags` is modeled via [`link_flags_to_string`] / [`link_flags_from_string`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkFlag {
    /// UP, value 1.
    Up,
    /// BROADCAST, value 2.
    Broadcast,
    /// DEBUG, value 4.
    Debug,
    /// LOOPBACK, value 8.
    Loopback,
    /// POINTTOPOINT, value 16.
    PointToPoint,
    /// NOTRAILERS, value 32.
    NoTrailers,
    /// RUNNING, value 64.
    Running,
    /// NOARP, value 128.
    NoArp,
    /// PROMISC, value 256.
    Promisc,
    /// ALLMULTI, value 512.
    AllMulti,
    /// MASTER, value 1024.
    Master,
    /// SLAVE, value 2048.
    Slave,
    /// MULTICAST, value 4096.
    Multicast,
    /// PORTSEL, value 8192.
    Portsel,
    /// AUTOMEDIA, value 16384.
    AutoMedia,
    /// DYNAMIC, value 32768.
    Dynamic,
    /// LOWER_UP, value 65536.
    LowerUp,
    /// DORMANT, value 131072.
    Dormant,
    /// ECHO, value 262144.
    Echo,
}

impl LinkFlag {
    /// All flags in ascending bit order (`Up..=Echo`).
    pub const ALL: [LinkFlag; 19] = [
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
    ];

    /// Numeric value matching upstream `LinkFlag`.
    pub fn as_value(self) -> u32 {
        let idx = LinkFlag::ALL.iter().position(|&f| f == self).unwrap();
        1u32 << idx
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            LinkFlag::Up => "UP",
            LinkFlag::Broadcast => "BROADCAST",
            LinkFlag::Debug => "DEBUG",
            LinkFlag::Loopback => "LOOPBACK",
            LinkFlag::PointToPoint => "POINTTOPOINT",
            LinkFlag::NoTrailers => "NOTRAILERS",
            LinkFlag::Running => "RUNNING",
            LinkFlag::NoArp => "NOARP",
            LinkFlag::Promisc => "PROMISC",
            LinkFlag::AllMulti => "ALLMULTI",
            LinkFlag::Master => "MASTER",
            LinkFlag::Slave => "SLAVE",
            LinkFlag::Multicast => "MULTICAST",
            LinkFlag::Portsel => "PORTSEL",
            LinkFlag::AutoMedia => "AUTOMEDIA",
            LinkFlag::Dynamic => "DYNAMIC",
            LinkFlag::LowerUp => "LOWER_UP",
            LinkFlag::Dormant => "DORMANT",
            LinkFlag::Echo => "ECHO",
        }
    }

    /// Parse from canonical string (mirrors `LinkFlagString`).
    pub fn parse(s: &str) -> Result<Self> {
        for v in LinkFlag::ALL {
            if v.to_str() == s {
                return Ok(v);
            }
        }
        let l = lower(s);
        for v in LinkFlag::ALL {
            if lower(v.to_str()) == l {
                return Ok(v);
            }
        }
        Err(unknown("LinkFlag", s))
    }
}

/// Render a `LinkFlags` bitmask the same way as upstream `LinkFlags.String()`:
/// comma-joined canonical names in ascending bit order.
pub fn link_flags_to_string(flags: u32) -> String {
    join_flags(flags, &LinkFlag::ALL, |f| (f.as_value(), f.to_str()))
}

/// Parse a `LinkFlags` bitmask from a comma-separated string, mirroring upstream
/// `LinkFlagsString`.
pub fn link_flags_from_string(s: &str) -> Result<u32> {
    let mut flags = 0u32;
    for part in s.split(',') {
        flags |= LinkFlag::parse(part)?.as_value();
    }
    Ok(flags)
}

// ---------------------------------------------------------------------------
// LinkType (linktype.go) — uint16, explicit (with duplicate value 513)
// ---------------------------------------------------------------------------

/// A link (ARPHRD) type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkType {
    /// netrom, value 0.
    Netrom,
    /// ether, value 1.
    Ether,
    /// eether, value 2.
    Eether,
    /// ax25, value 3.
    Ax25,
    /// pronet, value 4.
    Pronet,
    /// chaos, value 5.
    Chaos,
    /// ieee802, value 6.
    Ieee802,
    /// arcnet, value 7.
    Arcnet,
    /// atalk, value 8.
    Atalk,
    /// dlci, value 15.
    Dlci,
    /// atm, value 19.
    Atm,
    /// metricom, value 23.
    Metricom,
    /// ieee1394, value 24.
    Ieee1394,
    /// eui64, value 27.
    Eui64,
    /// infiniband, value 32.
    Infiniband,
    /// slip, value 256.
    Slip,
    /// cslip, value 257.
    Cslip,
    /// slip6, value 258.
    Slip6,
    /// cslip6, value 259.
    Cslip6,
    /// rsrvd, value 260.
    Rsrvd,
    /// adapt, value 264.
    Adapt,
    /// rose, value 270.
    Rose,
    /// x25, value 271.
    X25,
    /// hwx25, value 272.
    Hwx25,
    /// can, value 280.
    Can,
    /// ppp, value 512.
    Ppp,
    /// cisco, value 513.
    Cisco,
    /// hdlc, value 513 (alias of cisco).
    Hdlc,
    /// lapb, value 516.
    Lapb,
    /// ddcmp, value 517.
    Ddcmp,
    /// rawhdlc, value 518.
    Rawhdlc,
    /// ipip, value 768.
    Tunnel,
    /// tunnel6, value 769.
    Tunnel6,
    /// frad, value 770.
    Frad,
    /// skip, value 771.
    Skip,
    /// loopback, value 772.
    Loopbck,
    /// localtlk, value 773.
    Localtlk,
    /// fddi, value 774.
    Fddi,
    /// bif, value 775.
    Bif,
    /// sit, value 776.
    Sit,
    /// ip/ddp, value 777.
    Ipddp,
    /// gre, value 778.
    Ipgre,
    /// pimreg, value 779.
    Pimreg,
    /// hippi, value 780.
    Hippi,
    /// ash, value 781.
    Ash,
    /// econet, value 782.
    Econet,
    /// irda, value 783.
    Irda,
    /// fcpp, value 784.
    Fcpp,
    /// fcal, value 785.
    Fcal,
    /// fcpl, value 786.
    Fcpl,
    /// fcfb_0, value 787.
    Fcfabric,
    /// fcfb_1, value 788.
    Fcfabric1,
    /// fcfb_2, value 789.
    Fcfabric2,
    /// fcfb_3, value 790.
    Fcfabric3,
    /// fcfb_4, value 791.
    Fcfabric4,
    /// fcfb_5, value 792.
    Fcfabric5,
    /// fcfb_6, value 793.
    Fcfabric6,
    /// fcfb_7, value 794.
    Fcfabric7,
    /// fcfb_8, value 795.
    Fcfabric8,
    /// fcfb_9, value 796.
    Fcfabric9,
    /// fcfb_10, value 797.
    Fcfabric10,
    /// fcfb_11, value 798.
    Fcfabric11,
    /// fcfb_12, value 799.
    Fcfabric12,
    /// tr, value 800.
    Ieee802tr,
    /// ieee802.11, value 801.
    Ieee80211,
    /// ieee802.11_prism, value 802.
    Ieee80211prism,
    /// ieee802.11_radiotap, value 803.
    Ieee80211Radiotap,
    /// ieee802.15.4, value 804.
    Ieee8021154,
    /// ieee802.15.4_monitor, value 805.
    Ieee8021154monitor,
    /// phonet, value 820.
    Phonet,
    /// phonet_pipe, value 821.
    Phonetpipe,
    /// caif, value 822.
    Caif,
    /// ip6gre, value 823.
    Ip6gre,
    /// netlink, value 824.
    Netlink,
    /// 6lowpan, value 825.
    Sixlowpan,
    /// void, value 65535.
    Void,
    /// nohdr, value 65534.
    None,
}

impl LinkType {
    /// Numeric value matching upstream `LinkType`.
    pub fn as_value(self) -> u16 {
        match self {
            LinkType::Netrom => 0,
            LinkType::Ether => 1,
            LinkType::Eether => 2,
            LinkType::Ax25 => 3,
            LinkType::Pronet => 4,
            LinkType::Chaos => 5,
            LinkType::Ieee802 => 6,
            LinkType::Arcnet => 7,
            LinkType::Atalk => 8,
            LinkType::Dlci => 15,
            LinkType::Atm => 19,
            LinkType::Metricom => 23,
            LinkType::Ieee1394 => 24,
            LinkType::Eui64 => 27,
            LinkType::Infiniband => 32,
            LinkType::Slip => 256,
            LinkType::Cslip => 257,
            LinkType::Slip6 => 258,
            LinkType::Cslip6 => 259,
            LinkType::Rsrvd => 260,
            LinkType::Adapt => 264,
            LinkType::Rose => 270,
            LinkType::X25 => 271,
            LinkType::Hwx25 => 272,
            LinkType::Can => 280,
            LinkType::Ppp => 512,
            LinkType::Cisco | LinkType::Hdlc => 513,
            LinkType::Lapb => 516,
            LinkType::Ddcmp => 517,
            LinkType::Rawhdlc => 518,
            LinkType::Tunnel => 768,
            LinkType::Tunnel6 => 769,
            LinkType::Frad => 770,
            LinkType::Skip => 771,
            LinkType::Loopbck => 772,
            LinkType::Localtlk => 773,
            LinkType::Fddi => 774,
            LinkType::Bif => 775,
            LinkType::Sit => 776,
            LinkType::Ipddp => 777,
            LinkType::Ipgre => 778,
            LinkType::Pimreg => 779,
            LinkType::Hippi => 780,
            LinkType::Ash => 781,
            LinkType::Econet => 782,
            LinkType::Irda => 783,
            LinkType::Fcpp => 784,
            LinkType::Fcal => 785,
            LinkType::Fcpl => 786,
            LinkType::Fcfabric => 787,
            LinkType::Fcfabric1 => 788,
            LinkType::Fcfabric2 => 789,
            LinkType::Fcfabric3 => 790,
            LinkType::Fcfabric4 => 791,
            LinkType::Fcfabric5 => 792,
            LinkType::Fcfabric6 => 793,
            LinkType::Fcfabric7 => 794,
            LinkType::Fcfabric8 => 795,
            LinkType::Fcfabric9 => 796,
            LinkType::Fcfabric10 => 797,
            LinkType::Fcfabric11 => 798,
            LinkType::Fcfabric12 => 799,
            LinkType::Ieee802tr => 800,
            LinkType::Ieee80211 => 801,
            LinkType::Ieee80211prism => 802,
            LinkType::Ieee80211Radiotap => 803,
            LinkType::Ieee8021154 => 804,
            LinkType::Ieee8021154monitor => 805,
            LinkType::Phonet => 820,
            LinkType::Phonetpipe => 821,
            LinkType::Caif => 822,
            LinkType::Ip6gre => 823,
            LinkType::Netlink => 824,
            LinkType::Sixlowpan => 825,
            LinkType::Void => 65535,
            LinkType::None => 65534,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            LinkType::Netrom => "netrom",
            LinkType::Ether => "ether",
            LinkType::Eether => "eether",
            LinkType::Ax25 => "ax25",
            LinkType::Pronet => "pronet",
            LinkType::Chaos => "chaos",
            LinkType::Ieee802 => "ieee802",
            LinkType::Arcnet => "arcnet",
            LinkType::Atalk => "atalk",
            LinkType::Dlci => "dlci",
            LinkType::Atm => "atm",
            LinkType::Metricom => "metricom",
            LinkType::Ieee1394 => "ieee1394",
            LinkType::Eui64 => "eui64",
            LinkType::Infiniband => "infiniband",
            LinkType::Slip => "slip",
            LinkType::Cslip => "cslip",
            LinkType::Slip6 => "slip6",
            LinkType::Cslip6 => "cslip6",
            LinkType::Rsrvd => "rsrvd",
            LinkType::Adapt => "adapt",
            LinkType::Rose => "rose",
            LinkType::X25 => "x25",
            LinkType::Hwx25 => "hwx25",
            LinkType::Can => "can",
            LinkType::Ppp => "ppp",
            LinkType::Cisco => "cisco",
            LinkType::Hdlc => "hdlc",
            LinkType::Lapb => "lapb",
            LinkType::Ddcmp => "ddcmp",
            LinkType::Rawhdlc => "rawhdlc",
            LinkType::Tunnel => "ipip",
            LinkType::Tunnel6 => "tunnel6",
            LinkType::Frad => "frad",
            LinkType::Skip => "skip",
            LinkType::Loopbck => "loopback",
            LinkType::Localtlk => "localtlk",
            LinkType::Fddi => "fddi",
            LinkType::Bif => "bif",
            LinkType::Sit => "sit",
            LinkType::Ipddp => "ip/ddp",
            LinkType::Ipgre => "gre",
            LinkType::Pimreg => "pimreg",
            LinkType::Hippi => "hippi",
            LinkType::Ash => "ash",
            LinkType::Econet => "econet",
            LinkType::Irda => "irda",
            LinkType::Fcpp => "fcpp",
            LinkType::Fcal => "fcal",
            LinkType::Fcpl => "fcpl",
            LinkType::Fcfabric => "fcfb_0",
            LinkType::Fcfabric1 => "fcfb_1",
            LinkType::Fcfabric2 => "fcfb_2",
            LinkType::Fcfabric3 => "fcfb_3",
            LinkType::Fcfabric4 => "fcfb_4",
            LinkType::Fcfabric5 => "fcfb_5",
            LinkType::Fcfabric6 => "fcfb_6",
            LinkType::Fcfabric7 => "fcfb_7",
            LinkType::Fcfabric8 => "fcfb_8",
            LinkType::Fcfabric9 => "fcfb_9",
            LinkType::Fcfabric10 => "fcfb_10",
            LinkType::Fcfabric11 => "fcfb_11",
            LinkType::Fcfabric12 => "fcfb_12",
            LinkType::Ieee802tr => "tr",
            LinkType::Ieee80211 => "ieee802.11",
            LinkType::Ieee80211prism => "ieee802.11_prism",
            LinkType::Ieee80211Radiotap => "ieee802.11_radiotap",
            LinkType::Ieee8021154 => "ieee802.15.4",
            LinkType::Ieee8021154monitor => "ieee802.15.4_monitor",
            LinkType::Phonet => "phonet",
            LinkType::Phonetpipe => "phonet_pipe",
            LinkType::Caif => "caif",
            LinkType::Ip6gre => "ip6gre",
            LinkType::Netlink => "netlink",
            LinkType::Sixlowpan => "6lowpan",
            LinkType::Void => "void",
            LinkType::None => "nohdr",
        }
    }

    /// All variants in declaration order.
    pub const ALL: [LinkType; 77] = [
        LinkType::Netrom,
        LinkType::Ether,
        LinkType::Eether,
        LinkType::Ax25,
        LinkType::Pronet,
        LinkType::Chaos,
        LinkType::Ieee802,
        LinkType::Arcnet,
        LinkType::Atalk,
        LinkType::Dlci,
        LinkType::Atm,
        LinkType::Metricom,
        LinkType::Ieee1394,
        LinkType::Eui64,
        LinkType::Infiniband,
        LinkType::Slip,
        LinkType::Cslip,
        LinkType::Slip6,
        LinkType::Cslip6,
        LinkType::Rsrvd,
        LinkType::Adapt,
        LinkType::Rose,
        LinkType::X25,
        LinkType::Hwx25,
        LinkType::Can,
        LinkType::Ppp,
        LinkType::Cisco,
        LinkType::Hdlc,
        LinkType::Lapb,
        LinkType::Ddcmp,
        LinkType::Rawhdlc,
        LinkType::Tunnel,
        LinkType::Tunnel6,
        LinkType::Frad,
        LinkType::Skip,
        LinkType::Loopbck,
        LinkType::Localtlk,
        LinkType::Fddi,
        LinkType::Bif,
        LinkType::Sit,
        LinkType::Ipddp,
        LinkType::Ipgre,
        LinkType::Pimreg,
        LinkType::Hippi,
        LinkType::Ash,
        LinkType::Econet,
        LinkType::Irda,
        LinkType::Fcpp,
        LinkType::Fcal,
        LinkType::Fcpl,
        LinkType::Fcfabric,
        LinkType::Fcfabric1,
        LinkType::Fcfabric2,
        LinkType::Fcfabric3,
        LinkType::Fcfabric4,
        LinkType::Fcfabric5,
        LinkType::Fcfabric6,
        LinkType::Fcfabric7,
        LinkType::Fcfabric8,
        LinkType::Fcfabric9,
        LinkType::Fcfabric10,
        LinkType::Fcfabric11,
        LinkType::Fcfabric12,
        LinkType::Ieee802tr,
        LinkType::Ieee80211,
        LinkType::Ieee80211prism,
        LinkType::Ieee80211Radiotap,
        LinkType::Ieee8021154,
        LinkType::Ieee8021154monitor,
        LinkType::Phonet,
        LinkType::Phonetpipe,
        LinkType::Caif,
        LinkType::Ip6gre,
        LinkType::Netlink,
        LinkType::Sixlowpan,
        LinkType::Void,
        LinkType::None,
    ];

    /// Parse from canonical string (mirrors `LinkTypeString`).
    pub fn parse(s: &str) -> Result<Self> {
        for v in LinkType::ALL {
            if v.to_str() == s {
                return Ok(v);
            }
        }
        let l = lower(s);
        for v in LinkType::ALL {
            if lower(v.to_str()) == l {
                return Ok(v);
            }
        }
        Err(unknown("LinkType", s))
    }
}

// ---------------------------------------------------------------------------
// RouteFlag (routeflags.go) — uint32 bitmask, 256<<iota; individual flags
// ---------------------------------------------------------------------------

/// A single route flag (`RTM_F_*`).
///
/// Mirrors the individual `RouteFlag` constants (`256 << iota`); the bitmask
/// aggregate type `RouteFlags` is modeled via [`route_flags_to_string`] /
/// [`route_flags_from_string`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteFlag {
    /// notify, value 256.
    Notify,
    /// cloned, value 512.
    Cloned,
    /// equalize, value 1024.
    Equalize,
    /// prefix, value 2048.
    Prefix,
    /// lookup_table, value 4096.
    LookupTable,
    /// fib_match, value 8192.
    FibMatch,
    /// offload, value 16384.
    Offload,
    /// trap, value 32768.
    Trap,
}

impl RouteFlag {
    /// All flags in ascending bit order (`Notify..=Trap`).
    pub const ALL: [RouteFlag; 8] = [
        RouteFlag::Notify,
        RouteFlag::Cloned,
        RouteFlag::Equalize,
        RouteFlag::Prefix,
        RouteFlag::LookupTable,
        RouteFlag::FibMatch,
        RouteFlag::Offload,
        RouteFlag::Trap,
    ];

    /// Numeric value matching upstream `RouteFlag`.
    pub fn as_value(self) -> u32 {
        match self {
            RouteFlag::Notify => 256,
            RouteFlag::Cloned => 512,
            RouteFlag::Equalize => 1024,
            RouteFlag::Prefix => 2048,
            RouteFlag::LookupTable => 4096,
            RouteFlag::FibMatch => 8192,
            RouteFlag::Offload => 16384,
            RouteFlag::Trap => 32768,
        }
    }

    /// Canonical string (upstream `String()`).
    pub fn to_str(self) -> &'static str {
        match self {
            RouteFlag::Notify => "notify",
            RouteFlag::Cloned => "cloned",
            RouteFlag::Equalize => "equalize",
            RouteFlag::Prefix => "prefix",
            RouteFlag::LookupTable => "lookup_table",
            RouteFlag::FibMatch => "fib_match",
            RouteFlag::Offload => "offload",
            RouteFlag::Trap => "trap",
        }
    }

    /// Parse from canonical string (mirrors `RouteFlagString`).
    pub fn parse(s: &str) -> Result<Self> {
        for v in RouteFlag::ALL {
            if v.to_str() == s {
                return Ok(v);
            }
        }
        let l = lower(s);
        for v in RouteFlag::ALL {
            if lower(v.to_str()) == l {
                return Ok(v);
            }
        }
        Err(unknown("RouteFlag", s))
    }
}

/// Render a `RouteFlags` bitmask the same way as upstream `RouteFlags.String()`:
/// comma-joined canonical names in ascending bit order.
pub fn route_flags_to_string(flags: u32) -> String {
    join_flags(flags, &RouteFlag::ALL, |f| (f.as_value(), f.to_str()))
}

/// Parse a `RouteFlags` bitmask from a comma-separated string, mirroring upstream
/// `RouteFlagsString`. The empty string yields an empty (zero) mask.
pub fn route_flags_from_string(s: &str) -> Result<u32> {
    if s.is_empty() {
        return Ok(0);
    }
    let mut flags = 0u32;
    for part in s.split(',') {
        flags |= RouteFlag::parse(part)?.as_value();
    }
    Ok(flags)
}

// ---------------------------------------------------------------------------
// RoutingTable (routingtable.go) — uint32, explicit (0..=255)
// ---------------------------------------------------------------------------

/// A routing table ID.
///
/// Upstream defines 256 constants `TableUnspec(0)`, `Table1(1)`..`Table252(252)`,
/// `TableDefault(253)`, `TableMain(254)`, `TableLocal(255)`. The `String()`
/// (line-comment) of `TableUnspec` is `unspec`, of `Table1..Table252` is the
/// decimal number itself, and of the last three is `default`/`main`/`local`.
///
/// Rather than 256 explicit variants, this is modeled as a thin wrapper over the
/// numeric ID with the same `to_str` / `parse` behavior as the generated enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoutingTable(pub u32);

impl RoutingTable {
    /// `TableUnspec` (0).
    pub const UNSPEC: RoutingTable = RoutingTable(0);
    /// `TableDefault` (253).
    pub const DEFAULT: RoutingTable = RoutingTable(253);
    /// `TableMain` (254).
    pub const MAIN: RoutingTable = RoutingTable(254);
    /// `TableLocal` (255).
    pub const LOCAL: RoutingTable = RoutingTable(255);

    /// Numeric value matching upstream `RoutingTable`.
    pub fn as_value(self) -> u32 {
        self.0
    }

    /// Canonical string (upstream `String()`).
    ///
    /// Returns a borrowed static for the named tables and an owned decimal string
    /// for the numbered tables (1..=252).
    pub fn to_string_value(self) -> String {
        match self.0 {
            0 => String::from("unspec"),
            253 => String::from("default"),
            254 => String::from("main"),
            255 => String::from("local"),
            // 1..=252 render as the decimal number.
            n => format!("{n}"),
        }
    }

    /// Parse from canonical string (mirrors `RoutingTableString`).
    ///
    /// Accepts `unspec`/`default`/`main`/`local` (case-insensitively, matching
    /// enumer's lowercase fallback) and the decimal strings `1`..`252`.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "unspec" => return Ok(RoutingTable(0)),
            "default" => return Ok(RoutingTable(253)),
            "main" => return Ok(RoutingTable(254)),
            "local" => return Ok(RoutingTable(255)),
            _ => {}
        }
        // enumer lowercase fallback for the named tables.
        match lower(s).as_str() {
            "unspec" => return Ok(RoutingTable(0)),
            "default" => return Ok(RoutingTable(253)),
            "main" => return Ok(RoutingTable(254)),
            "local" => return Ok(RoutingTable(255)),
            _ => {}
        }
        // Numbered tables 1..=252 stringify to their decimal form.
        if let Ok(n) = s.parse::<u32>()
            && (1..=252).contains(&n)
        {
            return Ok(RoutingTable(n));
        }
        Err(unknown("RoutingTable", s))
    }
}

// ---------------------------------------------------------------------------
// internal helpers
// ---------------------------------------------------------------------------

/// Lowercase a string (no_std).
fn lower(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        for lc in c.to_lowercase() {
            out.push(lc);
        }
    }
    out
}

/// Returns the input unchanged if it round-trips, otherwise its lowercase form.
///
/// enumer's generated `XxxString(s)` first looks up the exact name, then falls
/// back to the lowercase form. For enums whose canonical names are already
/// lowercase, this is equivalent to a single case-insensitive lookup, which is
/// what callers in this module use.
fn lower_match(s: &str) -> String {
    lower(s)
}

/// Try exact match against canonical names, then enumer's lowercase fallback.
fn exact_or_lower<T: Copy>(s: &str, table: &[(T, &str)]) -> Option<T> {
    for (v, name) in table {
        if *name == s {
            return Some(*v);
        }
    }
    let l = lower(s);
    for (v, name) in table {
        if lower(name) == l {
            return Some(*v);
        }
    }
    None
}

/// Join the set bits present in `flags` into a comma-separated string using the
/// provided ordered flag list, mirroring the upstream bitmask `String()`.
fn join_flags<T: Copy>(flags: u32, all: &[T], get: impl Fn(T) -> (u32, &'static str)) -> String {
    let mut parts: Vec<&'static str> = Vec::new();
    for &f in all {
        let (bit, name) = get(f);
        if flags & bit == bit && bit != 0 {
            parts.push(name);
        }
    }
    parts.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bondmode_roundtrip_and_values() {
        assert_eq!(BondMode::Roundrobin.as_value(), 0);
        assert_eq!(BondMode::Ieee8023ad.as_value(), 4);
        assert_eq!(BondMode::Alb.as_value(), 6);
        assert_eq!(BondMode::Ieee8023ad.to_str(), "802.3ad");
        assert_eq!(BondMode::parse("802.3ad").unwrap(), BondMode::Ieee8023ad);
        assert_eq!(BondMode::parse("").unwrap(), BondMode::Roundrobin);
        assert_eq!(BondMode::parse("balance-rr").unwrap(), BondMode::Roundrobin);
        assert!(BondMode::parse("nope").is_err());
    }

    #[test]
    fn scope_values() {
        assert_eq!(Scope::Global.as_value(), 0);
        assert_eq!(Scope::Site.as_value(), 200);
        assert_eq!(Scope::Link.as_value(), 253);
        assert_eq!(Scope::Host.as_value(), 254);
        assert_eq!(Scope::Nowhere.as_value(), 255);
        assert_eq!(Scope::parse("host").unwrap(), Scope::Host);
        assert_eq!(Scope::parse("HOST").unwrap(), Scope::Host);
    }

    #[test]
    fn family_protocol_values() {
        assert_eq!(Family::Inet4.as_value(), 2);
        assert_eq!(Family::Inet6.as_value(), 10);
        assert_eq!(Protocol::Icmp.as_value(), 0x1);
        assert_eq!(Protocol::Udp.as_value(), 0x11);
        assert_eq!(Protocol::Icmpv6.as_value(), 0x3a);
        assert_eq!(Protocol::Icmpv6.to_str(), "icmpv6");
    }

    #[test]
    fn routeprotocol_spread() {
        assert_eq!(RouteProtocol::Kernel.as_value(), 2);
        assert_eq!(RouteProtocol::Dhcp.as_value(), 16);
        assert_eq!(RouteProtocol::Babel.as_value(), 42);
        assert_eq!(RouteProtocol::Eigrp.as_value(), 192);
        assert_eq!(RouteProtocol::parse("ospf").unwrap(), RouteProtocol::Ospf);
    }

    #[test]
    fn operstate_values() {
        assert_eq!(OperationalState::Unknown.as_value(), 0);
        assert_eq!(OperationalState::Up.as_value(), 6);
        assert_eq!(OperationalState::LowerLayerDown.to_str(), "lowerLayerDown");
        // enumer accepts exact and lowercase
        assert_eq!(
            OperationalState::parse("lowerLayerDown").unwrap(),
            OperationalState::LowerLayerDown
        );
        assert_eq!(
            OperationalState::parse("lowerlayerdown").unwrap(),
            OperationalState::LowerLayerDown
        );
    }

    #[test]
    fn duplex_port_string_forms() {
        assert_eq!(Duplex::Unknown.as_value(), 0xff);
        assert_eq!(Duplex::Full.to_str(), "Full");
        assert_eq!(Duplex::parse("Full").unwrap(), Duplex::Full);
        // Port has no line comments => names are the strings
        assert_eq!(Port::TwistedPair.to_str(), "TwistedPair");
        assert_eq!(Port::None.as_value(), 0xef);
        assert_eq!(Port::Other.as_value(), 0xff);
        assert_eq!(Port::parse("DirectAttach").unwrap(), Port::DirectAttach);
    }

    #[test]
    fn wol_skips_arp_bit() {
        assert_eq!(WolMode::Phy.as_value(), 1);
        assert_eq!(WolMode::Broadcast.as_value(), 8);
        // ARP bit (16) is skipped: Magic jumps to 32
        assert_eq!(WolMode::Magic.as_value(), 32);
        assert_eq!(WolMode::MagicSecure.as_value(), 64);
        assert_eq!(WolMode::Filter.as_value(), 128);
        assert_eq!(WolMode::parse("magicsecure").unwrap(), WolMode::MagicSecure);
    }

    #[test]
    fn nftables_priority_extremes() {
        assert_eq!(NfTablesChainPriority::First.as_value(), i32::MIN);
        assert_eq!(NfTablesChainPriority::Last.as_value(), i32::MAX);
        assert_eq!(NfTablesChainPriority::Filter.as_value(), 0);
        assert_eq!(NfTablesChainPriority::NatDest.as_value(), -100);
        assert_eq!(
            NfTablesChainPriority::parse("conntrack-defrag").unwrap(),
            NfTablesChainPriority::ConntrackDefrag
        );
    }

    #[test]
    fn matchoperator_symbols() {
        assert_eq!(MatchOperator::Equal.to_str(), "==");
        assert_eq!(MatchOperator::NotEqual.to_str(), "!=");
        assert_eq!(MatchOperator::parse("!=").unwrap(), MatchOperator::NotEqual);
    }

    #[test]
    fn address_flags_bitmask() {
        assert_eq!(AddressFlag::Temporary.as_value(), 1);
        assert_eq!(AddressFlag::Permanent.as_value(), 128);
        assert_eq!(AddressFlag::StablePrivacy.as_value(), 2048);
        let mask = AddressFlag::Permanent.as_value() | AddressFlag::Tentative.as_value();
        // ascending bit order: tentative(64) before permanent(128)
        assert_eq!(address_flags_to_string(mask), "tentative,permanent");
        assert_eq!(
            address_flags_from_string("tentative,permanent").unwrap(),
            mask
        );
        assert_eq!(address_flags_to_string(0), "");
    }

    #[test]
    fn link_flags_bitmask() {
        assert_eq!(LinkFlag::Up.as_value(), 1);
        assert_eq!(LinkFlag::Echo.as_value(), 1 << 18);
        assert_eq!(LinkFlag::LowerUp.to_str(), "LOWER_UP");
        let mask =
            LinkFlag::Up.as_value() | LinkFlag::Broadcast.as_value() | LinkFlag::Running.as_value();
        assert_eq!(link_flags_to_string(mask), "UP,BROADCAST,RUNNING");
        assert_eq!(
            link_flags_from_string("UP,BROADCAST,RUNNING").unwrap(),
            mask
        );
    }

    #[test]
    fn linktype_values_and_aliases() {
        assert_eq!(LinkType::Ether.as_value(), 1);
        assert_eq!(LinkType::Cisco.as_value(), 513);
        assert_eq!(LinkType::Hdlc.as_value(), 513);
        assert_eq!(LinkType::Tunnel.to_str(), "ipip");
        assert_eq!(LinkType::Ipgre.to_str(), "gre");
        assert_eq!(LinkType::Void.as_value(), 65535);
        assert_eq!(LinkType::None.as_value(), 65534);
        assert_eq!(LinkType::Fcfabric12.as_value(), 799);
        assert_eq!(LinkType::parse("ip/ddp").unwrap(), LinkType::Ipddp);
        assert_eq!(LinkType::parse("6lowpan").unwrap(), LinkType::Sixlowpan);
    }

    #[test]
    fn dnsprotocol_mixedcase() {
        assert_eq!(DnsProtocol::Default.to_str(), "Do53");
        assert_eq!(DnsProtocol::parse("DoT").unwrap(), DnsProtocol::DnsOverTls);
        assert_eq!(DnsProtocol::parse("doh").unwrap(), DnsProtocol::DnsOverHttp);
    }

    #[test]
    fn failovermac_and_byname() {
        assert_eq!(FailOverMac::Follow.as_value(), 2);
        assert_eq!(FailOverMac::parse("").unwrap(), FailOverMac::None);
        assert_eq!(FailOverMac::parse("Active").unwrap(), FailOverMac::Active);
    }

    #[test]
    fn status_starts_at_one() {
        assert_eq!(Status::Addresses.as_value(), 1);
        assert_eq!(Status::EtcFiles.as_value(), 4);
        assert_eq!(Status::EtcFiles.to_str(), "etcfiles");
    }

    #[test]
    fn nftables_chain_type_string_alias() {
        assert_eq!(NfTablesChainType::Nat.to_str(), "nat");
        assert_eq!(
            NfTablesChainType::parse("route").unwrap(),
            NfTablesChainType::Route
        );
        assert!(NfTablesChainType::parse("bogus").is_err());
    }

    #[test]
    fn port_fibre_is_empty_string() {
        // Matches the oracle vector: Port(3).String() == "" and PortString("") == 3.
        assert_eq!(Port::Fibre.as_value(), 3);
        assert_eq!(Port::Fibre.to_str(), "");
        assert_eq!(Port::parse("").unwrap(), Port::Fibre);
        assert_eq!(Port::None.as_value(), 239);
        assert_eq!(Port::Other.as_value(), 255);
    }

    #[test]
    fn route_flag_bitmask() {
        assert_eq!(RouteFlag::Notify.as_value(), 256);
        assert_eq!(RouteFlag::Cloned.as_value(), 512);
        assert_eq!(RouteFlag::Trap.as_value(), 32768);
        assert_eq!(RouteFlag::LookupTable.to_str(), "lookup_table");
        assert_eq!(RouteFlag::parse("fib_match").unwrap(), RouteFlag::FibMatch);
        assert!(RouteFlag::parse("__nope__").is_err());
        let mask = RouteFlag::Notify.as_value() | RouteFlag::Prefix.as_value();
        assert_eq!(route_flags_to_string(mask), "notify,prefix");
        assert_eq!(route_flags_from_string("notify,prefix").unwrap(), mask);
        assert_eq!(route_flags_to_string(0), "");
        assert_eq!(route_flags_from_string("").unwrap(), 0);
    }

    #[test]
    fn routing_table_named_and_numbered() {
        assert_eq!(RoutingTable::UNSPEC.as_value(), 0);
        assert_eq!(RoutingTable::UNSPEC.to_string_value(), "unspec");
        assert_eq!(RoutingTable::DEFAULT.to_string_value(), "default");
        assert_eq!(RoutingTable::MAIN.to_string_value(), "main");
        assert_eq!(RoutingTable::LOCAL.to_string_value(), "local");
        // Numbered tables stringify to their decimal value.
        assert_eq!(RoutingTable(1).to_string_value(), "1");
        assert_eq!(RoutingTable(252).to_string_value(), "252");
        assert_eq!(RoutingTable::parse("unspec").unwrap(), RoutingTable(0));
        assert_eq!(RoutingTable::parse("default").unwrap(), RoutingTable(253));
        assert_eq!(RoutingTable::parse("1").unwrap(), RoutingTable(1));
        assert_eq!(RoutingTable::parse("252").unwrap(), RoutingTable(252));
        assert!(RoutingTable::parse("__nope__").is_err());
    }
}
