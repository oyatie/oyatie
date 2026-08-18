//! Config-bytes load/save, mirroring the Talos `configloader` package
//! (`config.NewFromBytes`, `Config.Bytes`).
//!
//! The loader takes raw multi-document config bytes, splits them into encoded
//! documents (see [`crate::encoder`]), resolves each document's `kind` against a
//! [`Registry`], decodes the mandatory legacy `v1alpha1` document into a typed
//! [`V1Alpha1Config`], and collects the remaining typed documents into a
//! [`Config`] container. The reverse direction re-encodes the container back to
//! bytes.
//!
//! Decoding of the v1alpha1 body is a small, dependency-free field reader that
//! understands the flat `machine.*` / `cluster.*` scalars this port models. It
//! is intentionally not a full YAML parser, but it faithfully round-trips the
//! fields the rest of the crate exposes.

use crate::cluster::ControlPlaneEndpoint;
use crate::container::{AuxDocument, Config};
use crate::dhcpv4::{DHCPV4_CONFIG_KIND, decode_dhcpv4_config_body};
use crate::dhcpv6::{DHCPV6_CONFIG_KIND, decode_dhcpv6_config_body};
use crate::document::ConfigVersion;
use crate::encoder::{EncodedDocument, decode_documents, encode_documents};
use crate::link_config::{
    LINK_CONFIG_KIND, VLAN_CONFIG_KIND, decode_link_config_body, decode_vlan_config_body,
};
use crate::machine::{
    DhcpOptions, InstallConfig, MachineFeatures, NetworkConfig, NetworkInterface, NetworkVlan,
    SystemDiskEncryption,
};
use crate::registry::Registry;
use crate::resolver::{RESOLVER_CONFIG_KIND, ResolverConfig, decode_resolver_config_body};
use crate::v1alpha1::V1Alpha1Config;
use crate::volume_config::{
    EXISTING_VOLUME_CONFIG_KIND, EXTERNAL_VOLUME_CONFIG_KIND, EncryptionKeyProvider,
    EncryptionSpec, RAW_VOLUME_CONFIG_KIND, SWAP_VOLUME_CONFIG_KIND, USER_VOLUME_CONFIG_KIND,
    VOLUME_CONFIG_KIND, decode_encryption, decode_existing_volume_config_body,
    decode_external_volume_config_body, decode_raw_volume_config_body,
    decode_swap_volume_config_body, decode_user_volume_config_body, decode_volume_config_body,
};
use crate::yaml::{self, Yaml};
use os_kernel::error::{Error, Result};
use os_kernel::machine_type::MachineType;
use std::collections::BTreeSet;

/// Read an indented `key: value` scalar from a YAML-ish body, honoring a parent
/// path of `section.subkey` style (e.g. `machine.install.disk`). Returns the
/// unquoted value if present.
fn nested_scalar<'a>(body: &'a str, path: &str) -> Option<&'a str> {
    let keys: Vec<&str> = path.split('.').collect();
    let mut depth = 0usize; // index into keys we've matched so far
    let mut base_indent: Vec<usize> = Vec::new();
    for line in body.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim_start();
        // Pop matched levels whose indentation we've left.
        while depth > 0 && indent <= *base_indent.last().unwrap() {
            base_indent.pop();
            depth -= 1;
        }
        let want = keys.get(depth)?;
        if let Some(rest) = trimmed.strip_prefix(want) {
            let rest = rest.trim_start();
            if let Some(after) = rest.strip_prefix(':') {
                let value = after.trim();
                if depth + 1 == keys.len() {
                    if value.is_empty() {
                        return None;
                    }
                    return Some(value.trim_matches('"').trim_matches('\''));
                }
                // Descend into this section.
                base_indent.push(indent);
                depth += 1;
            }
        }
    }
    None
}

/// Decode the legacy v1alpha1 document body into a typed [`V1Alpha1Config`].
fn decode_v1alpha1(body: &str) -> Result<V1Alpha1Config> {
    let mut cfg = V1Alpha1Config::default();

    if let Some(t) = nested_scalar(body, "machine.type") {
        cfg.machine.machine_type = t
            .parse::<MachineType>()
            .map_err(|_| Error::parse(format!("invalid machine.type '{t}'")))?;
    }
    if let Some(tok) = nested_scalar(body, "machine.token") {
        cfg.machine.token = tok.to_string();
    }
    if let Some(disk) = nested_scalar(body, "machine.install.disk") {
        let image = nested_scalar(body, "machine.install.image").unwrap_or("");
        cfg.machine.install = InstallConfig::new(disk, image);
    }
    if let Some(name) = nested_scalar(body, "cluster.clusterName") {
        cfg.cluster.name = name.to_string();
    }
    if let Some(ep) = nested_scalar(body, "cluster.controlPlane.endpoint") {
        cfg.cluster.endpoint = ControlPlaneEndpoint::parse(ep)?;
    }
    if let Some(persist) = nested_scalar(body, "persist") {
        cfg.persist = persist == "true";
    }
    cfg.machine.network = decode_network(body)?;
    cfg.machine.features = decode_features(body)?;
    cfg.machine.system_disk_encryption = decode_system_disk_encryption(body)?;
    Ok(cfg)
}

/// Decode legacy `machine.systemDiskEncryption` fallback settings.
fn decode_system_disk_encryption(body: &str) -> Result<SystemDiskEncryption> {
    let root = yaml::parse(body).map_err(|e| Error::parse(e.to_string()))?;
    let encryption = root
        .get("machine")
        .and_then(|machine| machine.get("systemDiskEncryption"));
    let Some(encryption) = encryption else {
        return Ok(SystemDiskEncryption::default());
    };
    if encryption.as_mapping().is_none() {
        return Err(Error::parse(
            "machine.systemDiskEncryption must be a mapping",
        ));
    }

    Ok(SystemDiskEncryption {
        state: decode_encryption(
            encryption.get("state"),
            "machine.systemDiskEncryption.state",
        )?,
        ephemeral: decode_encryption(
            encryption.get("ephemeral"),
            "machine.systemDiskEncryption.ephemeral",
        )?,
    })
}

/// Decode the subset of `machine.features` that this crate currently models.
fn decode_features(body: &str) -> Result<MachineFeatures> {
    let root = yaml::parse(body).map_err(|e| Error::parse(e.to_string()))?;
    let features = root.get("machine").and_then(|m| m.get("features"));
    Ok(MachineFeatures {
        disk_quota_support: optional_bool(
            features.and_then(|f| f.get("diskQuotaSupport")),
            "machine.features.diskQuotaSupport",
        )?
        .unwrap_or(false),
    })
}

/// Decode the subset of `machine.network` that this crate currently models.
fn decode_network(body: &str) -> Result<NetworkConfig> {
    let root = yaml::parse(body).map_err(|e| Error::parse(e.to_string()))?;
    let Some(interfaces) = root
        .get("machine")
        .and_then(|m| m.get("network"))
        .and_then(|n| n.get("interfaces"))
    else {
        return Ok(NetworkConfig::default());
    };
    let Some(items) = interfaces.as_sequence() else {
        return Err(Error::parse(
            "machine.network.interfaces must be a sequence",
        ));
    };

    let mut network = NetworkConfig::default();
    for item in items {
        if item.as_mapping().is_none() {
            return Err(Error::parse(
                "machine.network.interfaces[] must be a mapping",
            ));
        }
        let interface = item.get_str("interface").unwrap_or("").to_string();
        let dhcp =
            optional_bool(item.get("dhcp"), "machine.network.interfaces[].dhcp")?.unwrap_or(false);
        let ignore = optional_bool(item.get("ignore"), "machine.network.interfaces[].ignore")?
            .unwrap_or(false);
        let dhcp_options = match item.get("dhcpOptions") {
            Some(opts) => decode_dhcp_options(opts, "machine.network.interfaces[].dhcpOptions")?,
            None => DhcpOptions::default(),
        };
        let vlans = match item.get("vlans") {
            Some(vlans) => decode_vlans(vlans)?,
            None => Vec::new(),
        };
        network.interfaces.push(NetworkInterface {
            interface,
            dhcp,
            ignore,
            dhcp_options,
            vlans,
        });
    }

    Ok(network)
}

fn validate_resolver_v1alpha1_conflicts(core_body: &str, resolver: &ResolverConfig) -> Result<()> {
    let root = yaml::parse(core_body).map_err(|e| Error::parse(e.to_string()))?;
    let machine = root.get("machine");
    let network = machine.and_then(|m| m.get("network"));

    if network.and_then(|n| n.get("searchDomains")).is_some() {
        return Err(Error::invalid(
            ".machine.network.searchDomains is already set in v1alpha1 config",
        ));
    }
    if network.and_then(|n| n.get("nameservers")).is_some() {
        return Err(Error::invalid(
            ".machine.network.nameservers is already set in v1alpha1 config",
        ));
    }
    if optional_bool(
        network.and_then(|n| n.get("disableSearchDomain")),
        ".machine.network.disableSearchDomain",
    )?
    .unwrap_or(false)
    {
        return Err(Error::invalid(
            ".machine.network.disableSearchDomain is already set in v1alpha1 config",
        ));
    }

    if !resolver.host_dns.is_zero()
        && machine
            .and_then(|m| m.get("features"))
            .and_then(|f| f.get("hostDNS"))
            .is_some()
    {
        return Err(Error::invalid(
            ".machine.features.hostDNS is already set in v1alpha1 config",
        ));
    }

    Ok(())
}

fn decode_vlans(value: &Yaml) -> Result<Vec<NetworkVlan>> {
    let Some(items) = value.as_sequence() else {
        return Err(Error::parse(
            "machine.network.interfaces[].vlans must be a sequence",
        ));
    };

    let mut vlans = Vec::new();
    for item in items {
        if item.as_mapping().is_none() {
            return Err(Error::parse(
                "machine.network.interfaces[].vlans[] must be a mapping",
            ));
        }
        let vlan_id = optional_u16(
            item.get("vlanId"),
            "machine.network.interfaces[].vlans[].vlanId",
        )?
        .unwrap_or(0);
        let dhcp = optional_bool(
            item.get("dhcp"),
            "machine.network.interfaces[].vlans[].dhcp",
        )?
        .unwrap_or(false);
        let dhcp_options = match item.get("dhcpOptions") {
            Some(opts) => {
                decode_dhcp_options(opts, "machine.network.interfaces[].vlans[].dhcpOptions")?
            }
            None => DhcpOptions::default(),
        };
        vlans.push(NetworkVlan {
            vlan_id,
            dhcp,
            dhcp_options,
        });
    }

    Ok(vlans)
}

fn decode_dhcp_options(value: &Yaml, field: &str) -> Result<DhcpOptions> {
    if value.as_mapping().is_none() {
        return Err(Error::parse(format!("{field} must be a mapping")));
    }
    Ok(DhcpOptions {
        route_metric: optional_u32(value.get("routeMetric"), &format!("{field}.routeMetric"))?
            .unwrap_or(0),
        ipv4: optional_bool(value.get("ipv4"), &format!("{field}.ipv4"))?,
        ipv6: optional_bool(value.get("ipv6"), &format!("{field}.ipv6"))?,
        duid_v6: value.get_str("duidv6").unwrap_or("").to_string(),
    })
}

fn optional_bool(value: Option<&Yaml>, field: &str) -> Result<Option<bool>> {
    let Some(value) = value else {
        return Ok(None);
    };
    value
        .as_bool()
        .map(Some)
        .ok_or_else(|| Error::parse(format!("{field} must be a boolean")))
}

fn optional_u16(value: Option<&Yaml>, field: &str) -> Result<Option<u16>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(raw) = value.as_str() else {
        return Err(Error::parse(format!("{field} must be an integer")));
    };
    raw.parse::<u16>()
        .map(Some)
        .map_err(|_| Error::parse(format!("{field} must be an unsigned 16-bit integer")))
}

fn optional_u32(value: Option<&Yaml>, field: &str) -> Result<Option<u32>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(raw) = value.as_str() else {
        return Err(Error::parse(format!("{field} must be an integer")));
    };
    raw.parse::<u32>()
        .map(Some)
        .map_err(|_| Error::parse(format!("{field} must be an unsigned 32-bit integer")))
}

/// Encode a [`V1Alpha1Config`] back to its document body text.
fn encode_v1alpha1(cfg: &V1Alpha1Config) -> String {
    use core::fmt::Write as _;

    let mut out = String::new();
    out.push_str("version: v1alpha1\n");
    let _ = writeln!(out, "persist: {}", cfg.persist);
    out.push_str("machine:\n");
    let _ = writeln!(out, "  type: {}", cfg.machine.machine_type.as_str());
    if !cfg.machine.token.is_empty() {
        let _ = writeln!(out, "  token: {}", cfg.machine.token);
    }
    if cfg.machine.install.has_disk() {
        out.push_str("  install:\n");
        let _ = writeln!(out, "    disk: {}", cfg.machine.install.disk);
        if !cfg.machine.install.image.is_empty() {
            let _ = writeln!(out, "    image: {}", cfg.machine.install.image);
        }
    }
    if !cfg.machine.network.interfaces.is_empty() {
        out.push_str("  network:\n");
        out.push_str("    interfaces:\n");
        for interface in &cfg.machine.network.interfaces {
            let _ = writeln!(out, "      - interface: {}", interface.interface);
            let _ = writeln!(out, "        dhcp: {}", interface.dhcp);
            if interface.ignore {
                out.push_str("        ignore: true\n");
            }
            write_dhcp_options(&mut out, &interface.dhcp_options, "        ");
            if !interface.vlans.is_empty() {
                out.push_str("        vlans:\n");
                for vlan in &interface.vlans {
                    let _ = writeln!(out, "          - vlanId: {}", vlan.vlan_id);
                    let _ = writeln!(out, "            dhcp: {}", vlan.dhcp);
                    write_dhcp_options(&mut out, &vlan.dhcp_options, "            ");
                }
            }
        }
    }
    if cfg.machine.features.disk_quota_support {
        out.push_str("  features:\n");
        out.push_str("    diskQuotaSupport: true\n");
    }
    write_system_disk_encryption(&mut out, &cfg.machine.system_disk_encryption);
    out.push_str("cluster:\n");
    if !cfg.cluster.name.is_empty() {
        let _ = writeln!(out, "  clusterName: {}", cfg.cluster.name);
    }
    if cfg.cluster.has_endpoint() {
        out.push_str("  controlPlane:\n");
        let _ = writeln!(out, "    endpoint: {}", cfg.cluster.endpoint.to_url());
    }
    out
}

fn write_system_disk_encryption(out: &mut String, encryption: &SystemDiskEncryption) {
    if !encryption.is_enabled() {
        return;
    }

    out.push_str("  systemDiskEncryption:\n");
    if let Some(state) = &encryption.state {
        write_encryption_spec(out, "state", state);
    }
    if let Some(ephemeral) = &encryption.ephemeral {
        write_encryption_spec(out, "ephemeral", ephemeral);
    }
}

fn write_encryption_spec(out: &mut String, name: &str, encryption: &EncryptionSpec) {
    use core::fmt::Write as _;

    let _ = writeln!(out, "    {name}:");
    let _ = writeln!(out, "      provider: {}", encryption.provider);
    out.push_str("      keys:\n");
    for key in &encryption.keys {
        let _ = writeln!(out, "        - slot: {}", key.slot);
        match &key.provider {
            EncryptionKeyProvider::Static { passphrase } => {
                out.push_str("          static:\n");
                let _ = writeln!(out, "            passphrase: {passphrase}");
            }
            EncryptionKeyProvider::NodeId => {
                out.push_str("          nodeID: {}\n");
            }
            EncryptionKeyProvider::Kms { endpoint } => {
                out.push_str("          kms:\n");
                let _ = writeln!(out, "            endpoint: {endpoint}");
            }
            EncryptionKeyProvider::Tpm {
                check_secureboot_status_on_enroll,
                pcrs,
            } => {
                out.push_str("          tpm:\n");
                if *check_secureboot_status_on_enroll {
                    out.push_str("            checkSecurebootStatusOnEnroll: true\n");
                }
                if pcrs != &[7] {
                    out.push_str("            options:\n");
                    out.push_str("              pcrs:\n");
                    for pcr in pcrs {
                        let _ = writeln!(out, "                - {pcr}");
                    }
                }
            }
        }
        if key.lock_to_state {
            out.push_str("          lockToState: true\n");
        }
    }
    if let Some(cipher) = &encryption.cipher {
        let _ = writeln!(out, "      cipher: {cipher}");
    }
    if let Some(key_size) = encryption.key_size {
        let _ = writeln!(out, "      keySize: {key_size}");
    }
    if let Some(block_size) = encryption.block_size {
        let _ = writeln!(out, "      blockSize: {block_size}");
    }
    if !encryption.options.is_empty() {
        out.push_str("      options:\n");
        for option in &encryption.options {
            let _ = writeln!(out, "        - {option}");
        }
    }
}

fn write_dhcp_options(out: &mut String, dhcp_options: &DhcpOptions, indent: &str) {
    use core::fmt::Write as _;

    if dhcp_options == &DhcpOptions::default() {
        return;
    }

    let child_indent = format!("{indent}  ");
    out.push_str(indent);
    out.push_str("dhcpOptions:\n");
    if dhcp_options.route_metric != 0 {
        let _ = writeln!(
            out,
            "{child_indent}routeMetric: {}",
            dhcp_options.route_metric
        );
    }
    if let Some(ipv4) = dhcp_options.ipv4 {
        let _ = writeln!(out, "{child_indent}ipv4: {ipv4}");
    }
    if let Some(ipv6) = dhcp_options.ipv6 {
        let _ = writeln!(out, "{child_indent}ipv6: {ipv6}");
    }
    if !dhcp_options.duid_v6.is_empty() {
        let _ = writeln!(out, "{child_indent}duidv6: {}", dhcp_options.duid_v6);
    }
}

/// Load a [`Config`] container from raw multi-document bytes using the builtin
/// [`Registry`]. Mirrors Talos `config.NewFromBytes`.
pub fn load_from_bytes(input: &str) -> Result<Config> {
    load_from_bytes_with(input, &Registry::with_builtins())
}

/// Load a [`Config`] container, validating every document kind against the
/// supplied registry and rejecting unknown kinds.
pub fn load_from_bytes_with(input: &str, registry: &Registry) -> Result<Config> {
    let docs = decode_documents(input)?;
    let mut core: Option<V1Alpha1Config> = None;
    let mut container: Option<Config> = None;
    let mut core_body: Option<String> = None;
    let mut dhcpv4_names = BTreeSet::new();
    let mut dhcpv6_names = BTreeSet::new();
    let mut link_names = BTreeSet::new();
    let mut vlan_names = BTreeSet::new();
    let mut volume_names = BTreeSet::new();
    let mut user_volume_names = BTreeSet::new();
    let mut raw_volume_names = BTreeSet::new();
    let mut existing_volume_names = BTreeSet::new();
    let mut external_volume_names = BTreeSet::new();
    let mut swap_volume_names = BTreeSet::new();

    for doc in &docs {
        registry.resolve(&doc.meta.kind)?;
        if doc.meta.kind == "v1alpha1" {
            if core.is_some() {
                return Err(Error::invalid("multiple v1alpha1 documents in config"));
            }
            let parsed = decode_v1alpha1(&doc.body)?;
            container = Some(Config::new(parsed.clone()));
            core = Some(parsed);
            core_body = Some(doc.body.clone());
        } else {
            let c = container
                .as_mut()
                .ok_or_else(|| Error::invalid("v1alpha1 document must come first"))?;
            let spec = registry.resolve(&doc.meta.kind)?;
            if doc.meta.kind == DHCPV4_CONFIG_KIND {
                let parsed = decode_dhcpv4_config_body(&doc.body)?;
                if !dhcpv4_names.insert(parsed.name.clone()) {
                    return Err(Error::invalid(format!(
                        "duplicate DHCPv4Config document for link '{}'",
                        parsed.name
                    )));
                }
            }
            if doc.meta.kind == DHCPV6_CONFIG_KIND {
                let parsed = decode_dhcpv6_config_body(&doc.body)?;
                if !dhcpv6_names.insert(parsed.name.clone()) {
                    return Err(Error::invalid(format!(
                        "duplicate DHCPv6Config document for link '{}'",
                        parsed.name
                    )));
                }
            }
            if doc.meta.kind == LINK_CONFIG_KIND {
                let parsed = decode_link_config_body(&doc.body)?;
                if !link_names.insert(parsed.name.clone()) {
                    return Err(Error::invalid(format!(
                        "duplicate LinkConfig document for link '{}'",
                        parsed.name
                    )));
                }
            }
            if doc.meta.kind == VLAN_CONFIG_KIND {
                let parsed = decode_vlan_config_body(&doc.body)?;
                if !vlan_names.insert(parsed.name.clone()) {
                    return Err(Error::invalid(format!(
                        "duplicate VLANConfig document for link '{}'",
                        parsed.name
                    )));
                }
            }
            if doc.meta.kind == RESOLVER_CONFIG_KIND {
                let parsed = decode_resolver_config_body(&doc.body)?;
                validate_resolver_v1alpha1_conflicts(core_body.as_deref().unwrap_or(""), &parsed)?;
            }
            if doc.meta.kind == VOLUME_CONFIG_KIND {
                let parsed = decode_volume_config_body(&doc.body)?;
                if !volume_names.insert(parsed.name.clone()) {
                    return Err(Error::invalid(format!(
                        "duplicate VolumeConfig document for volume '{}'",
                        parsed.name
                    )));
                }
            }
            if doc.meta.kind == USER_VOLUME_CONFIG_KIND {
                let parsed = decode_user_volume_config_body(&doc.body)?;
                if existing_volume_names.contains(&parsed.name) {
                    return Err(Error::invalid(format!(
                        "UserVolumeConfig document for volume '{}' conflicts with ExistingVolumeConfig document",
                        parsed.name
                    )));
                }
                if !user_volume_names.insert(parsed.name.clone()) {
                    return Err(Error::invalid(format!(
                        "duplicate UserVolumeConfig document for volume '{}'",
                        parsed.name
                    )));
                }
            }
            if doc.meta.kind == RAW_VOLUME_CONFIG_KIND {
                let parsed = decode_raw_volume_config_body(&doc.body)?;
                if !raw_volume_names.insert(parsed.name.clone()) {
                    return Err(Error::invalid(format!(
                        "duplicate RawVolumeConfig document for volume '{}'",
                        parsed.name
                    )));
                }
            }
            if doc.meta.kind == EXISTING_VOLUME_CONFIG_KIND {
                let parsed = decode_existing_volume_config_body(&doc.body)?;
                if user_volume_names.contains(&parsed.name) {
                    return Err(Error::invalid(format!(
                        "ExistingVolumeConfig document for volume '{}' conflicts with UserVolumeConfig document",
                        parsed.name
                    )));
                }
                if !existing_volume_names.insert(parsed.name.clone()) {
                    return Err(Error::invalid(format!(
                        "duplicate ExistingVolumeConfig document for volume '{}'",
                        parsed.name
                    )));
                }
            }
            if doc.meta.kind == EXTERNAL_VOLUME_CONFIG_KIND {
                let parsed = decode_external_volume_config_body(&doc.body)?;
                if !external_volume_names.insert(parsed.name.clone()) {
                    return Err(Error::invalid(format!(
                        "duplicate ExternalVolumeConfig document for volume '{}'",
                        parsed.name
                    )));
                }
            }
            if doc.meta.kind == SWAP_VOLUME_CONFIG_KIND {
                let parsed = decode_swap_volume_config_body(&doc.body)?;
                if !swap_volume_names.insert(parsed.name.clone()) {
                    return Err(Error::invalid(format!(
                        "duplicate SwapVolumeConfig document for volume '{}'",
                        parsed.name
                    )));
                }
            }
            let mut aux = AuxDocument::new(doc.meta.kind.clone(), doc.body.clone());
            aux.allow_multiple = spec.allows_multiple();
            c.add_document(aux)?;
        }
    }

    container.ok_or_else(|| Error::invalid("config is missing the mandatory v1alpha1 document"))
}

/// Serialize a [`Config`] container back to multi-document bytes. Mirrors Talos
/// `Config.Bytes`.
pub fn save_to_bytes(config: &Config) -> String {
    let mut docs: Vec<EncodedDocument> = Vec::new();
    let core_body = encode_v1alpha1(config.core());
    docs.push(EncodedDocument::new(
        crate::document::DocumentMeta::v1alpha1(),
        core_body,
    ));
    for aux in config.documents() {
        docs.push(EncodedDocument::new(aux.meta.clone(), aux.body.clone()));
    }
    encode_documents(&docs)
}

/// Detect the schema version of a config blob without fully decoding it.
/// Mirrors the Talos version-sniffing step of the loader.
pub fn detect_version(input: &str) -> Result<ConfigVersion> {
    let docs = decode_documents(input)?;
    Ok(docs[0].meta.version)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = "version: v1alpha1\npersist: true\nmachine:\n  type: controlplane\n  token: tok\n  install:\n    disk: /dev/sda\n    image: ghcr.io/img\ncluster:\n  clusterName: prod\n  controlPlane:\n    endpoint: https://10.0.0.1:6443\n---\napiVersion: v1alpha1\nkind: SideroLinkConfig\napiUrl: grpc://example\n";

    #[test]
    fn nested_scalar_reads_indented_fields() {
        assert_eq!(nested_scalar(CONFIG, "machine.type"), Some("controlplane"));
        assert_eq!(
            nested_scalar(CONFIG, "machine.install.disk"),
            Some("/dev/sda")
        );
        assert_eq!(
            nested_scalar(CONFIG, "cluster.controlPlane.endpoint"),
            Some("https://10.0.0.1:6443")
        );
        assert_eq!(nested_scalar(CONFIG, "machine.nope"), None);
    }

    #[test]
    fn load_decodes_core_and_aux() {
        let c = load_from_bytes(CONFIG).unwrap();
        assert_eq!(c.core().machine.machine_type, MachineType::ControlPlane);
        assert_eq!(c.core().machine.token, "tok");
        assert_eq!(c.core().machine.install.disk, "/dev/sda");
        assert_eq!(c.core().cluster.name, "prod");
        assert_eq!(c.core().cluster.endpoint.port, 6443);
        assert_eq!(c.len(), 2);
        assert!(c.document("SideroLinkConfig").is_some());
    }

    #[test]
    fn round_trip_through_bytes() {
        let c = load_from_bytes(CONFIG).unwrap();
        let bytes = save_to_bytes(&c);
        let again = load_from_bytes(&bytes).unwrap();
        assert_eq!(again.core().machine.machine_type, MachineType::ControlPlane);
        assert_eq!(
            again.core().cluster.endpoint.to_url(),
            "https://10.0.0.1:6443"
        );
        assert!(again.document("SideroLinkConfig").is_some());
    }

    #[test]
    fn load_decodes_disk_quota_feature() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  features:
    diskQuotaSupport: true
";
        let loaded = load_from_bytes(cfg).unwrap();
        assert!(loaded.core().machine.features.disk_quota_support_enabled());

        let round_tripped = save_to_bytes(&loaded);
        let again = load_from_bytes(&round_tripped).unwrap();
        assert!(
            again.core().machine.features.disk_quota_support_enabled(),
            "diskQuotaSupport survives save/load"
        );
    }

    #[test]
    fn load_decodes_legacy_system_disk_encryption() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  systemDiskEncryption:
    state:
      provider: luks2
      keys:
        - slot: 0
          static:
            passphrase: state-secret
      cipher: aes-xts-plain64
      keySize: 512
      blockSize: 4096
      options:
        - no_read_workqueue
    ephemeral:
      provider: luks2
      keys:
        - slot: 0
          nodeID: {}
        - slot: 1
          kms:
            endpoint: https://kms.example
";

        let loaded = load_from_bytes(cfg).unwrap();
        let system_disk_encryption = &loaded.core().machine.system_disk_encryption;
        let state = system_disk_encryption
            .state
            .as_ref()
            .expect("state encryption");
        assert_eq!(state.provider, "luks2");
        assert_eq!(state.cipher.as_deref(), Some("aes-xts-plain64"));
        assert_eq!(state.key_size, Some(512));
        assert_eq!(state.block_size, Some(4096));
        assert_eq!(state.options, vec!["no_read_workqueue"]);
        assert_eq!(
            state.keys[0].provider,
            EncryptionKeyProvider::Static {
                passphrase: "state-secret".to_string()
            }
        );

        let ephemeral = system_disk_encryption
            .ephemeral
            .as_ref()
            .expect("ephemeral encryption");
        assert_eq!(ephemeral.provider, "luks2");
        assert_eq!(ephemeral.keys.len(), 2);
        assert_eq!(ephemeral.keys[0].provider, EncryptionKeyProvider::NodeId);
        assert_eq!(
            ephemeral.keys[1].provider,
            EncryptionKeyProvider::Kms {
                endpoint: "https://kms.example".to_string()
            }
        );

        let round_tripped = save_to_bytes(&loaded);
        let again = load_from_bytes(&round_tripped).unwrap();
        assert!(
            again.core().machine.system_disk_encryption.is_enabled(),
            "legacy systemDiskEncryption survives save/load"
        );
    }

    #[test]
    fn unknown_kind_rejected() {
        let bad = "version: v1alpha1\nmachine:\n  type: worker\n---\napiVersion: v1alpha1\nkind: TotallyMadeUp\nx: 1\n";
        let err = load_from_bytes(bad).unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }

    #[test]
    fn resolver_config_rejects_legacy_v1alpha1_dns_conflicts() {
        let cases = [
            (
                "machine:\n  type: worker\n  network:\n    nameservers:\n      - 1.1.1.1\n",
                ".machine.network.nameservers",
                "nameservers:\n  - address: 9.9.9.9\n",
            ),
            (
                "machine:\n  type: worker\n  network:\n    searchDomains:\n      - example.com\n",
                ".machine.network.searchDomains",
                "searchDomains:\n  domains:\n    - cluster.local\n",
            ),
            (
                "machine:\n  type: worker\n  network:\n    disableSearchDomain: true\n",
                ".machine.network.disableSearchDomain",
                "searchDomains:\n  domains:\n    - cluster.local\n",
            ),
            (
                "machine:\n  type: worker\n  features:\n    hostDNS:\n      enabled: true\n",
                ".machine.features.hostDNS",
                "hostDNS:\n  enabled: true\n",
            ),
        ];

        for (core, expected, resolver_body) in cases {
            let cfg = format!(
                "version: v1alpha1\n{core}---\napiVersion: v1alpha1\nkind: ResolverConfig\n{resolver_body}"
            );
            let err = load_from_bytes(&cfg).unwrap_err();
            assert!(
                err.to_string().contains(expected),
                "expected {expected} in {err}"
            );
        }
    }

    #[test]
    fn resolver_config_allows_false_disable_and_legacy_hostdns_without_modern_hostdns() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  network:
    disableSearchDomain: false
  features:
    hostDNS:
      enabled: true
---
apiVersion: v1alpha1
kind: ResolverConfig
nameservers:
  - address: 9.9.9.9
";
        let loaded = load_from_bytes(cfg).unwrap();
        assert!(loaded.document(RESOLVER_CONFIG_KIND).is_some());
    }

    #[test]
    fn missing_core_document_rejected() {
        // A lone non-v1alpha1 document has no core to attach to.
        let bad = "apiVersion: v1alpha1\nkind: SideroLinkConfig\napiUrl: x\n";
        let err = load_from_bytes(bad).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn duplicate_singleton_aux_rejected_on_load() {
        let dup = "version: v1alpha1\nmachine:\n  type: worker\n---\napiVersion: v1alpha1\nkind: KmsgLogConfig\na: 1\n---\napiVersion: v1alpha1\nkind: KmsgLogConfig\nb: 2\n";
        let err = load_from_bytes(dup).unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn repeatable_aux_allowed_on_load() {
        let multi = "version: v1alpha1\nmachine:\n  type: worker\n---\napiVersion: v1alpha1\nkind: NetworkRuleConfig\na: 1\n---\napiVersion: v1alpha1\nkind: NetworkRuleConfig\nb: 2\n";
        let c = load_from_bytes(multi).unwrap();
        assert_eq!(c.documents().len(), 2);
    }

    #[test]
    fn detect_version_works() {
        assert_eq!(detect_version(CONFIG).unwrap(), ConfigVersion::V1Alpha1);
    }

    #[test]
    fn persist_field_roundtrips() {
        let body = "version: v1alpha1\npersist: false\nmachine:\n  type: worker\n";
        let c = load_from_bytes(body).unwrap();
        assert!(!c.core().persist);
    }

    #[test]
    fn worker_without_aux_loads() {
        let body = "version: v1alpha1\nmachine:\n  type: worker\n  token: jointoken\n";
        let c = load_from_bytes(body).unwrap();
        assert_eq!(c.core().machine.machine_type, MachineType::Worker);
        assert_eq!(c.core().machine.token, "jointoken");
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn load_decodes_legacy_interface_dhcp_options() {
        let body = "version: v1alpha1\npersist: true\nmachine:\n  type: worker\n  token: tok\n  network:\n    interfaces:\n      - interface: eth0\n        dhcp: true\n        dhcpOptions:\n          routeMetric: 2048\n          ipv4: false\n          ipv6: true\n          duidv6: 00030001aabbccddeeff\n      - interface: eth1\n        dhcp: true\ncluster:\n  controlPlane:\n    endpoint: https://10.0.0.1:6443\n";
        let c = load_from_bytes(body).unwrap();
        let interfaces = &c.core().machine.network.interfaces;

        assert_eq!(interfaces.len(), 2);
        assert_eq!(interfaces[0].interface, "eth0");
        assert!(interfaces[0].dhcp);
        assert_eq!(interfaces[0].dhcp_options.route_metric, 2048);
        assert!(!interfaces[0].dhcp_options.ipv4());
        assert!(interfaces[0].dhcp_options.ipv6());
        assert_eq!(interfaces[0].dhcp_options.duid_v6, "00030001aabbccddeeff");
        assert_eq!(interfaces[1], NetworkInterface::dhcp("eth1"));
    }

    #[test]
    fn legacy_interface_dhcp_options_round_trip() {
        let body = "version: v1alpha1\nmachine:\n  type: worker\n  token: tok\n  network:\n    interfaces:\n      - interface: eth0\n        dhcp: true\n        dhcpOptions:\n          routeMetric: 2048\n          ipv4: false\n          ipv6: true\n          duidv6: 00030001aabbccddeeff\n";
        let c = load_from_bytes(body).unwrap();
        let bytes = save_to_bytes(&c);
        let again = load_from_bytes(&bytes).unwrap();
        let opts = &again.core().machine.network.interfaces[0].dhcp_options;

        assert_eq!(opts.route_metric, 2048);
        assert_eq!(opts.ipv4, Some(false));
        assert_eq!(opts.ipv6, Some(true));
        assert_eq!(opts.duid_v6, "00030001aabbccddeeff");
    }

    #[test]
    fn legacy_interface_ignore_round_trip() {
        let body = "version: v1alpha1\nmachine:\n  type: worker\n  token: tok\n  network:\n    interfaces:\n      - interface: eth0\n        dhcp: true\n        ignore: true\n";
        let c = load_from_bytes(body).unwrap();
        assert!(c.core().machine.network.interfaces[0].ignore);

        let bytes = save_to_bytes(&c);
        let again = load_from_bytes(&bytes).unwrap();

        assert!(bytes.contains("        ignore: true\n"));
        assert!(again.core().machine.network.interfaces[0].ignore);
    }

    #[test]
    fn load_decodes_vlan_dhcp_options() {
        let body = "version: v1alpha1\nmachine:\n  type: worker\n  token: tok\n  network:\n    interfaces:\n      - interface: eth0\n        dhcp: false\n        vlans:\n          - vlanId: 100\n            dhcp: true\n            dhcpOptions:\n              routeMetric: 4096\n              ipv4: false\n              ipv6: true\n              duidv6: 00030001aabbccddeeff\n          - vlanId: 200\n            dhcp: true\n";
        let c = load_from_bytes(body).unwrap();
        let vlans = &c.core().machine.network.interfaces[0].vlans;

        assert_eq!(vlans.len(), 2);
        assert_eq!(vlans[0].vlan_id, 100);
        assert!(vlans[0].dhcp);
        assert_eq!(vlans[0].dhcp_options.route_metric, 4096);
        assert!(!vlans[0].dhcp_options.ipv4());
        assert!(vlans[0].dhcp_options.ipv6());
        assert_eq!(vlans[0].dhcp_options.duid_v6, "00030001aabbccddeeff");
        assert_eq!(vlans[1].vlan_id, 200);
        assert_eq!(vlans[1].dhcp_options, DhcpOptions::default());
    }

    #[test]
    fn vlan_dhcp_options_round_trip() {
        let body = "version: v1alpha1\nmachine:\n  type: worker\n  token: tok\n  network:\n    interfaces:\n      - interface: eth0\n        dhcp: false\n        vlans:\n          - vlanId: 100\n            dhcp: true\n            dhcpOptions:\n              routeMetric: 4096\n              ipv4: false\n              ipv6: true\n              duidv6: 00030001aabbccddeeff\n";
        let c = load_from_bytes(body).unwrap();
        let bytes = save_to_bytes(&c);
        let again = load_from_bytes(&bytes).unwrap();
        let vlan = &again.core().machine.network.interfaces[0].vlans[0];

        assert_eq!(vlan.vlan_id, 100);
        assert!(vlan.dhcp);
        assert_eq!(vlan.dhcp_options.route_metric, 4096);
        assert_eq!(vlan.dhcp_options.ipv4, Some(false));
        assert_eq!(vlan.dhcp_options.ipv6, Some(true));
        assert_eq!(vlan.dhcp_options.duid_v6, "00030001aabbccddeeff");
    }

    #[test]
    fn vlan_dhcp_options_reject_bad_types() {
        let bad_metric = "version: v1alpha1\nmachine:\n  type: worker\n  network:\n    interfaces:\n      - interface: eth0\n        vlans:\n          - vlanId: 100\n            dhcpOptions:\n              routeMetric: not-a-number\n";
        let err = load_from_bytes(bad_metric).unwrap_err();
        assert_eq!(err.kind(), "parse");

        let bad_bool = "version: v1alpha1\nmachine:\n  type: worker\n  network:\n    interfaces:\n      - interface: eth0\n        vlans:\n          - vlanId: 100\n            dhcpOptions:\n              ipv6: maybe\n";
        let err = load_from_bytes(bad_bool).unwrap_err();
        assert_eq!(err.kind(), "parse");
    }

    #[test]
    fn legacy_interface_dhcp_options_reject_bad_types() {
        let bad_metric = "version: v1alpha1\nmachine:\n  type: worker\n  network:\n    interfaces:\n      - interface: eth0\n        dhcpOptions:\n          routeMetric: not-a-number\n";
        let err = load_from_bytes(bad_metric).unwrap_err();
        assert_eq!(err.kind(), "parse");

        let bad_bool = "version: v1alpha1\nmachine:\n  type: worker\n  network:\n    interfaces:\n      - interface: eth0\n        dhcpOptions:\n          ipv6: maybe\n";
        let err = load_from_bytes(bad_bool).unwrap_err();
        assert_eq!(err.kind(), "parse");
    }
}
