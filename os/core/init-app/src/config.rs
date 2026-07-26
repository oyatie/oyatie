//! Pure machine-config parsing used during early boot.
//!
//! PID 1 reads the Talos machine config (baked into the initramfs or fetched per
//! the `talos.config=` kernel arg) and pulls out just what early userspace needs
//! before `machined` takes over: the hostname and the machine *type*
//! (controlplane vs worker). The full config is the domain of
//! `talos-machine-config`; here we route through its multi-document decoder and
//! then do a tiny indentation-scoped scan for the handful of scalars init uses.

use os_block_domain::{
    FilesystemType, IMAGE_CACHE_VOLUME_ID, VolumeConfig as BlockVolumeConfig, VolumeManager,
};
use os_machine_config_domain::{
    DhcpV4ClientIdentifier, DhcpV6ClientIdentifier, IMAGE_CACHE_VOLUME_NAME, MIN_USER_VOLUME_SIZE,
    RawVolumeConfigDoc, ResolverDnsProtocol, SizeLimit, UserVolumeConfigDoc, UserVolumeFilesystem,
    UserVolumeType, VolumeConfigDoc, decode_documents, dhcpv4_configs, dhcpv6_configs,
    load_from_bytes, raw_volume_configs, resolver_config, user_volume_configs, volume_configs,
};
use os_network_domain::{
    ClientIdentifierSpec, ConfigLayer, OperatorSpec, ResolverSpec, vlan_link_name,
};

/// The Talos machine role, decoded from `machine.type`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MachineType {
    ControlPlane,
    Worker,
    /// Legacy synonym for controlplane; still accepted by Talos.
    Init,
    Unknown,
}

impl MachineType {
    pub fn parse(s: &str) -> MachineType {
        match s.trim() {
            "controlplane" => MachineType::ControlPlane,
            "worker" => MachineType::Worker,
            "init" => MachineType::Init,
            _ => MachineType::Unknown,
        }
    }

    /// True if this node runs control-plane components.
    pub fn is_control_plane(self) -> bool {
        matches!(self, MachineType::ControlPlane | MachineType::Init)
    }
}

/// A static IPv4 address (dotted-quad + prefix length) declared on an interface.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StaticAddress {
    /// Dotted-quad address, e.g. `"10.0.2.15"`.
    pub addr: String,
    /// CIDR prefix length, e.g. `24`.
    pub prefix: u8,
}

/// The slice of machine config early userspace extracts.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct EarlyConfig {
    pub hostname: Option<String>,
    pub machine_type: Option<MachineType>,
    pub install_disk: Option<String>,
    /// The static IPv4 address declared on the first `machine.network.interfaces`
    /// entry (its `addresses:`/`address:` field), if any. Used by the network
    /// boot phase to call `add_ipv4("eth0", ...)`.
    pub first_iface_address: Option<StaticAddress>,
    /// DHCP operators declared by legacy `machine.network.interfaces[]`.
    pub dhcp_operators: Vec<OperatorSpec>,
    /// Non-fatal extraction errors captured by the compatibility [`early_config`]
    /// helper. Production boot paths call [`try_early_config`] and fail visibly
    /// instead of using this fallback.
    pub config_errors: Vec<String>,
}

/// Extract the early config from a (possibly multi-document) machine config.
pub fn early_config(contents: &str) -> EarlyConfig {
    match try_early_config(contents) {
        Ok(config) => config,
        Err(err) => {
            let body = v1alpha1_body(contents);
            EarlyConfig {
                hostname: extract_hostname(&body),
                machine_type: machine_scalar(&body, "type").map(|s| MachineType::parse(&s)),
                install_disk: install_disk(&body),
                first_iface_address: first_iface_address(&body),
                dhcp_operators: Vec::new(),
                config_errors: vec![format!("machine-config DHCP operator extraction: {err}")],
            }
        }
    }
}

/// Extract early config and propagate malformed DHCP/network documents.
pub fn try_early_config(contents: &str) -> os_kernel::Result<EarlyConfig> {
    let body = v1alpha1_body(contents);
    Ok(EarlyConfig {
        hostname: extract_hostname(&body),
        machine_type: machine_scalar(&body, "type").map(|s| MachineType::parse(&s)),
        install_disk: install_disk(&body),
        first_iface_address: first_iface_address(&body),
        dhcp_operators: machine_config_dhcp_operators(contents)?,
        config_errors: Vec::new(),
    })
}

/// Materialize legacy `machine.network.interfaces[].dhcpOptions` and modern
/// DHCP config documents into network operator specs.
///
/// Legacy Talos defaults are preserved: `dhcp: true` plus absent options yields
/// a DHCPv4 operator only (`ipv4` defaults true, `ipv6` defaults false). A
/// custom `duidv6` turns the DHCPv6 operator into an explicit raw-DUID client
/// identifier. Malformed configs are returned to callers so PID1 and
/// controller paths cannot silently erase operator specs on parse failure.
pub fn machine_config_dhcp_operators(contents: &str) -> os_kernel::Result<Vec<OperatorSpec>> {
    if contents.trim().is_empty() {
        return Ok(Vec::new());
    }

    let container = load_from_bytes(contents)?;
    let mut out = Vec::new();

    for iface in &container.core().machine.network.interfaces {
        if iface.interface.is_empty() || iface.ignore {
            continue;
        }
        if iface.dhcp {
            push_dhcp_operator_specs(&mut out, &iface.interface, &iface.dhcp_options)?;
        }

        for vlan in &iface.vlans {
            if !vlan.dhcp {
                continue;
            }
            let vlan_link = vlan_link_name(&iface.interface, vlan.vlan_id);
            push_dhcp_operator_specs(&mut out, &vlan_link, &vlan.dhcp_options)?;
        }
    }
    for config in dhcpv4_configs(&container)? {
        let mut op = OperatorSpec::dhcp4(&config.name);
        op.route_metric = config.route_metric_or(os_network_domain::DEFAULT_ROUTE_METRIC);
        op.skip_hostname_request = config.ignore_hostname;
        op.client_identifier = match config.client_identifier {
            DhcpV4ClientIdentifier::None => ClientIdentifierSpec::none(),
            DhcpV4ClientIdentifier::Mac => ClientIdentifierSpec::mac(),
            DhcpV4ClientIdentifier::Duid => ClientIdentifierSpec::duid(config.duid_raw),
        };
        op.validate()?;
        out.push(op);
    }

    for config in dhcpv6_configs(&container)? {
        let mut op = OperatorSpec::dhcp6(&config.name);
        op.route_metric = config.route_metric_or(os_network_domain::DEFAULT_ROUTE_METRIC);
        op.skip_hostname_request = config.ignore_hostname;
        op.client_identifier = match config.client_identifier {
            DhcpV6ClientIdentifier::None => ClientIdentifierSpec::none(),
            DhcpV6ClientIdentifier::Mac => ClientIdentifierSpec::mac(),
            DhcpV6ClientIdentifier::Duid => ClientIdentifierSpec::duid(config.duid_raw),
        };
        op.validate()?;
        out.push(op);
    }

    Ok(out)
}

/// Materialize a `ResolverConfig` document into source-layer resolver specs.
///
/// This first source-guided projection intentionally supports the fields the
/// current Rust [`ResolverSpec`] can represent losslessly: plain Do53
/// nameservers plus `searchDomains.domains`. Encrypted DNS and `hostDNS` remain
/// parsed/preserved by `talos-machine-config`; this projection returns an error
/// rather than silently downgrading unsupported resolver semantics.
pub fn machine_config_resolver_specs(contents: &str) -> os_kernel::Result<Vec<ResolverSpec>> {
    if contents.trim().is_empty() {
        return Ok(Vec::new());
    }

    let container = load_from_bytes(contents)?;
    let Some(config) = resolver_config(&container)? else {
        return Ok(Vec::new());
    };
    config.validate_projection_supported()?;

    let mut servers = Vec::new();
    for ns in &config.nameservers {
        if ns.protocol != ResolverDnsProtocol::Do53 || !ns.tls_server_name.is_empty() {
            return Err(os_kernel::Error::invalid(
                "ResolverConfig encrypted nameservers are parsed but not yet projectable to ResolverSpec",
            ));
        }
        servers.push(os_kernel::NodeAddress::parse(&ns.address)?);
    }

    if servers.is_empty() && config.search_domains.domains.is_empty() {
        return Ok(Vec::new());
    }

    Ok(vec![ResolverSpec::new_with_search(
        servers,
        config.search_domains.domains,
        ConfigLayer::Configuration,
    )?])
}

/// Materialize machine-config storage documents into a block volume manager.
///
/// The storage projection intentionally covers only the fields represented by
/// the current Rust block declaration seam: min/max sizes (absolute, relative,
/// and negative tail-reservation forms), partition volume IDs/labels,
/// raw-disk exact-one selectors, validated disk selectors, and XFS/ext4
/// filesystems. Parsed source fields that have no lossless
/// `talos-block::VolumeConfig` representation yet (encryption, mount knobs,
/// btrfs, and directory user volumes) return explicit errors.
pub fn machine_config_volume_manager(contents: &str) -> os_kernel::Result<VolumeManager> {
    if contents.trim().is_empty() {
        return Ok(VolumeManager::with_system_volumes());
    }

    let container = load_from_bytes(contents)?;
    let mut system_overrides = Vec::new();
    let mut user_volumes = Vec::new();

    for doc in volume_configs(&container)? {
        if let Some(config) = project_system_volume_config(&doc)? {
            system_overrides.push(config);
        }
    }

    for doc in user_volume_configs(&container)? {
        user_volumes.push(project_user_volume_config(&doc)?);
    }

    for doc in raw_volume_configs(&container)? {
        user_volumes.push(project_raw_volume_config(&doc)?);
    }

    VolumeManager::from_declarations(system_overrides, user_volumes)
        .map_err(os_kernel::Error::from)
}

/// Return `machine.features.imageCache.localEnabled`.
///
/// Source shape: Talos exposes Image Cache as
/// `machine.features.imageCache.localEnabled`; absence means disabled. The
/// value must be a boolean when present so PID1 cannot silently enable or
/// disable image-cache runtime effects from a malformed scalar.
pub fn machine_config_image_cache_local_enabled(contents: &str) -> os_kernel::Result<bool> {
    if contents.trim().is_empty() {
        return Ok(false);
    }

    let body = v1alpha1_body(contents);
    let doc = os_machine_config_domain::yaml::parse(&body).map_err(|err| {
        os_kernel::Error::parse(format!("machine.features.imageCache.localEnabled: {err}"))
    })?;

    let Some(local_enabled) = doc
        .get("machine")
        .and_then(|machine| machine.get("features"))
        .and_then(|features| features.get("imageCache"))
        .and_then(|image_cache| image_cache.get("localEnabled"))
    else {
        return Ok(false);
    };

    local_enabled.as_bool().ok_or_else(|| {
        os_kernel::Error::invalid("machine.features.imageCache.localEnabled must be boolean")
    })
}

fn project_system_volume_config(
    doc: &VolumeConfigDoc,
) -> os_kernel::Result<Option<BlockVolumeConfig>> {
    const IMAGE_CACHE_DEFAULT_MIN_SIZE: u64 = 500 * 1024 * 1024;
    const IMAGE_CACHE_DEFAULT_MAX_SIZE: u64 = 1024 * 1024 * 1024;
    const IMAGE_CACHE_DEFAULT_DISK_SELECTOR: &str = "system_disk";

    if doc.encryption_configured {
        return Err(os_kernel::Error::unsupported(format!(
            "VolumeConfig {} encryption is parsed but not yet projectable to talos-block VolumeConfig",
            doc.name
        )));
    }
    if doc.mount_secure.is_some() {
        return Err(os_kernel::Error::unsupported(format!(
            "VolumeConfig {} mount options are parsed but not yet projectable to talos-block VolumeConfig",
            doc.name
        )));
    }
    match doc.name.as_str() {
        "STATE" if doc.provisioning.is_zero() => Ok(None),
        "STATE" => Err(os_kernel::Error::invalid(
            "VolumeConfig STATE provisioning is invalid",
        )),
        "EPHEMERAL" => {
            let mut config = BlockVolumeConfig::partition(
                "EPHEMERAL",
                "EPHEMERAL",
                doc.provisioning.min_size.unwrap_or(0),
            );
            apply_max_size_projection(&mut config, doc.provisioning.max_size);
            config.grow = doc.provisioning.grow;
            config.filesystem = Some(FilesystemType::Xfs);
            config.disk_selector = doc.provisioning.disk_selector.clone();
            Ok(Some(config))
        }
        IMAGE_CACHE_VOLUME_NAME => {
            let mut config = BlockVolumeConfig::partition(
                IMAGE_CACHE_VOLUME_ID,
                IMAGE_CACHE_VOLUME_ID,
                doc.provisioning
                    .min_size
                    .unwrap_or(IMAGE_CACHE_DEFAULT_MIN_SIZE),
            );
            if doc.provisioning.max_size.is_some() {
                apply_max_size_projection(&mut config, doc.provisioning.max_size);
            } else {
                config.max_size = Some(IMAGE_CACHE_DEFAULT_MAX_SIZE);
            }
            config.grow = Some(doc.provisioning.grow.unwrap_or(false));
            config.filesystem = Some(FilesystemType::Ext4);
            config.disk_selector = Some(
                doc.provisioning
                    .disk_selector
                    .clone()
                    .unwrap_or_else(|| IMAGE_CACHE_DEFAULT_DISK_SELECTOR.to_string()),
            );
            Ok(Some(config))
        }
        other => Err(os_kernel::Error::invalid(format!(
            "unknown system VolumeConfig {other:?}"
        ))),
    }
}

fn project_user_volume_config(doc: &UserVolumeConfigDoc) -> os_kernel::Result<BlockVolumeConfig> {
    if doc.volume_type == UserVolumeType::Directory {
        return Err(os_kernel::Error::unsupported(format!(
            "UserVolumeConfig {} volumeType directory is parsed but not yet projectable to talos-block VolumeConfig",
            doc.name
        )));
    }
    if doc.encryption_configured {
        return Err(os_kernel::Error::unsupported(format!(
            "UserVolumeConfig {} encryption is parsed but not yet projectable to talos-block VolumeConfig",
            doc.name
        )));
    }
    if !doc.mount.is_zero() {
        return Err(os_kernel::Error::unsupported(format!(
            "UserVolumeConfig {} mount options are parsed but not yet projectable to talos-block VolumeConfig",
            doc.name
        )));
    }
    if doc.filesystem.project_quota_support == Some(true) {
        return Err(os_kernel::Error::unsupported(format!(
            "UserVolumeConfig {} projectQuotaSupport is parsed but not yet projectable to talos-block VolumeConfig",
            doc.name
        )));
    }
    let fs = match doc.filesystem.filesystem {
        UserVolumeFilesystem::Xfs => FilesystemType::Xfs,
        UserVolumeFilesystem::Ext4 => FilesystemType::Ext4,
        UserVolumeFilesystem::Btrfs => {
            return Err(os_kernel::Error::unsupported(format!(
                "UserVolumeConfig {} filesystem btrfs is parsed but not yet supported by talos-block",
                doc.name
            )));
        }
    };

    let id = doc.volume_id();
    match doc.volume_type {
        UserVolumeType::Directory => unreachable!("directory volume rejected above"),
        UserVolumeType::Disk => {
            let selector = doc.provisioning.disk_selector.clone().ok_or_else(|| {
                os_kernel::Error::invalid(format!(
                    "UserVolumeConfig {} volumeType disk requires provisioning.diskSelector.match",
                    doc.name
                ))
            })?;
            let mut config = BlockVolumeConfig::disk(id, selector);
            config.filesystem = Some(fs);
            Ok(config)
        }
        UserVolumeType::Partition => {
            let min_size = doc
                .provisioning
                .min_size
                .unwrap_or(0)
                .max(MIN_USER_VOLUME_SIZE);
            let mut config = BlockVolumeConfig::partition(id.clone(), id, min_size);
            apply_max_size_projection(&mut config, doc.provisioning.max_size);
            config.grow = doc.provisioning.grow;
            config.filesystem = Some(fs);
            config.disk_selector = doc.provisioning.disk_selector.clone();
            Ok(config)
        }
    }
}

fn project_raw_volume_config(doc: &RawVolumeConfigDoc) -> os_kernel::Result<BlockVolumeConfig> {
    if doc.encryption_configured {
        return Err(os_kernel::Error::unsupported(format!(
            "RawVolumeConfig {} encryption is parsed but not yet projectable to talos-block VolumeConfig",
            doc.name
        )));
    }

    let id = doc.volume_id();
    let min_size = doc
        .provisioning
        .min_size
        .unwrap_or(0)
        .max(MIN_USER_VOLUME_SIZE);
    let mut config = BlockVolumeConfig::raw_partition(id.clone(), id, min_size);
    apply_max_size_projection(&mut config, doc.provisioning.max_size);
    config.grow = doc.provisioning.grow;
    config.disk_selector = doc.provisioning.disk_selector.clone();
    Ok(config)
}

fn apply_max_size_projection(config: &mut BlockVolumeConfig, max_size: Option<SizeLimit>) {
    match max_size {
        Some(SizeLimit::Absolute(bytes)) => {
            config.max_size = Some(bytes);
            config.relative_max_size = None;
            config.negative_max_size = false;
        }
        Some(SizeLimit::RelativePercent(percent)) => {
            config.max_size = None;
            config.relative_max_size = Some(percent);
            config.negative_max_size = false;
        }
        Some(SizeLimit::NegativeBytes(bytes)) => {
            config.max_size = Some(bytes);
            config.relative_max_size = None;
            config.negative_max_size = true;
        }
        Some(SizeLimit::NegativeRelativePercent(percent)) => {
            config.max_size = None;
            config.relative_max_size = Some(percent);
            config.negative_max_size = true;
        }
        None => {
            config.max_size = None;
            config.relative_max_size = None;
            config.negative_max_size = false;
        }
    }
}

fn push_dhcp_operator_specs(
    out: &mut Vec<OperatorSpec>,
    link_name: &str,
    options: &os_machine_config_domain::DhcpOptions,
) -> os_kernel::Result<()> {
    if options.ipv4() {
        let mut op = OperatorSpec::dhcp4(link_name);
        op.route_metric = options.route_metric_or(os_network_domain::DEFAULT_ROUTE_METRIC);
        out.push(op);
    }

    if options.ipv6() {
        let mut op = OperatorSpec::dhcp6(link_name);
        op.route_metric = options.route_metric_or(os_network_domain::DEFAULT_ROUTE_METRIC);
        if !options.duid_v6.is_empty() {
            op =
                op.with_client_identifier(ClientIdentifierSpec::duid(hex_bytes(&options.duid_v6)?));
        }
        op.validate()?;
        out.push(op);
    }

    Ok(())
}

fn hex_bytes(s: &str) -> os_kernel::Result<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return Err(os_kernel::Error::invalid(
            "duidv6 must be an even-length hex string",
        ));
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    for idx in (0..s.len()).step_by(2) {
        let byte = u8::from_str_radix(&s[idx..idx + 2], 16)
            .map_err(|_| os_kernel::Error::invalid("duidv6 must be hexadecimal"))?;
        out.push(byte);
    }
    Ok(out)
}

/// Extract the first static IPv4 address under
/// `machine.network.interfaces[].addresses` (or the singular `address:`).
///
/// Accepts a CIDR string such as `10.0.2.15/24`; a bare address (no `/`) is
/// treated as a `/32` host route. This is the value the network boot phase
/// feeds to `add_ipv4`.
pub fn first_iface_address(body: &str) -> Option<StaticAddress> {
    let raw = first_iface_address_raw(body)?;
    parse_cidr(&raw)
}

/// Parse a `addr` or `addr/prefix` string into a [`StaticAddress`]. A missing
/// prefix defaults to `/32`. Returns `None` on a malformed prefix.
fn parse_cidr(raw: &str) -> Option<StaticAddress> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    match raw.split_once('/') {
        Some((addr, prefix)) => {
            let prefix = prefix.trim().parse::<u8>().ok()?;
            if prefix > 32 {
                return None;
            }
            Some(StaticAddress {
                addr: addr.trim().to_string(),
                prefix,
            })
        }
        None => Some(StaticAddress {
            addr: raw.to_string(),
            prefix: 32,
        }),
    }
}

/// Extract `machine.network.hostname`, falling back to a raw-text scan.
pub fn hostname_from_config(contents: &str) -> Option<String> {
    extract_hostname(&v1alpha1_body(contents))
}

/// Locate the `v1alpha1` document body within a multi-doc config, or fall back
/// to the whole text.
fn v1alpha1_body(contents: &str) -> String {
    match decode_documents(contents) {
        Ok(docs) => docs
            .iter()
            .find(|d| d.meta.kind == "v1alpha1")
            .map(|d| d.body.clone())
            .unwrap_or_else(|| contents.to_string()),
        Err(_) => contents.to_string(),
    }
}

/// Pull `machine.network.hostname` using indentation scoping.
pub fn extract_hostname(body: &str) -> Option<String> {
    let mut in_machine = false;
    let mut machine_indent = 0usize;
    let mut in_network = false;
    let mut network_indent = 0usize;

    for line in body.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if in_network && indent <= network_indent {
            in_network = false;
        }
        if in_machine && indent <= machine_indent && key_of(trimmed) != "machine" {
            in_machine = false;
        }

        if key_of(trimmed) == "machine" {
            in_machine = true;
            machine_indent = indent;
            continue;
        }

        if in_machine {
            if key_of(trimmed) == "network" {
                in_network = true;
                network_indent = indent;
                continue;
            }
            if in_network
                && let Some(value) = scalar(trimmed, "hostname")
                && !value.is_empty()
            {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Pull a direct child scalar of the top-level `machine:` block (e.g. `type`).
fn machine_scalar(body: &str, key: &str) -> Option<String> {
    let mut in_machine = false;
    let mut machine_indent = 0usize;

    for line in body.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if in_machine && indent <= machine_indent && key_of(trimmed) != "machine" {
            in_machine = false;
        }
        if key_of(trimmed) == "machine" {
            in_machine = true;
            machine_indent = indent;
            continue;
        }
        if in_machine
            && indent == machine_indent + 2
            && let Some(v) = scalar(trimmed, key)
            && !v.is_empty()
        {
            return Some(v.to_string());
        }
    }
    None
}

/// Pull `machine.install.disk`.
fn install_disk(body: &str) -> Option<String> {
    let mut in_machine = false;
    let mut machine_indent = 0usize;
    let mut in_install = false;
    let mut install_indent = 0usize;

    for line in body.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if in_install && indent <= install_indent {
            in_install = false;
        }
        if in_machine && indent <= machine_indent && key_of(trimmed) != "machine" {
            in_machine = false;
        }
        if key_of(trimmed) == "machine" {
            in_machine = true;
            machine_indent = indent;
            continue;
        }
        if in_machine {
            if key_of(trimmed) == "install" {
                in_install = true;
                install_indent = indent;
                continue;
            }
            if in_install
                && let Some(v) = scalar(trimmed, "disk")
                && !v.is_empty()
            {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Pull the raw address string from the first `machine.network.interfaces`
/// entry, using the structured YAML parser in `talos-machine-config` rather than
/// a hand-rolled scan. Accepts either a sequence under `addresses:` (first item)
/// or a singular `address:` scalar. Returns the raw (possibly CIDR) string, e.g.
/// `"10.0.2.15/24"`.
fn first_iface_address_raw(body: &str) -> Option<String> {
    let doc = os_machine_config_domain::yaml::parse(body).ok()?;
    let first = doc
        .get("machine")?
        .get("network")?
        .get("interfaces")?
        .as_sequence()?
        .first()?;
    // Prefer the plural `addresses:` sequence (or an inline scalar), then the
    // singular `address:` scalar — matching Talos's `Interfaces[].addresses`.
    if let Some(addresses) = first.get("addresses") {
        if let Some(s) = addresses
            .as_sequence()
            .and_then(|s| s.first())
            .and_then(|v| v.as_str())
        {
            return non_empty(s);
        }
        if let Some(s) = addresses.as_str() {
            return non_empty(s);
        }
    }
    first.get_str("address").and_then(non_empty)
}

/// Return `Some(owned)` for a non-empty trimmed string, else `None`.
fn non_empty(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Return the key portion (before the first `:`) of a `key: value` line.
pub fn key_of(line: &str) -> &str {
    line.split(':').next().unwrap_or("").trim()
}

/// If `line` is `key: value`, return the unquoted, trimmed value.
pub fn scalar<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(key)?;
    let rest = rest.trim_start();
    let value = rest.strip_prefix(':')?;
    Some(value.trim().trim_matches('"').trim_matches('\''))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL_CONFIG: &str = "\
version: v1alpha1
machine:
  type: controlplane
  network:
    hostname: node-01
    nameservers:
      - 1.1.1.1
  install:
    disk: /dev/sda
cluster:
  clusterName: prod
";

    #[test]
    fn extracts_hostname_from_full_config() {
        assert_eq!(
            hostname_from_config(FULL_CONFIG).as_deref(),
            Some("node-01")
        );
    }

    #[test]
    fn quoted_hostname_is_unquoted() {
        let cfg = "version: v1alpha1\nmachine:\n  network:\n    hostname: \"web-1\"\n";
        assert_eq!(hostname_from_config(cfg).as_deref(), Some("web-1"));
    }

    #[test]
    fn missing_network_yields_none() {
        let cfg = "version: v1alpha1\nmachine:\n  type: worker\n";
        assert_eq!(hostname_from_config(cfg), None);
    }

    #[test]
    fn empty_hostname_yields_none() {
        let cfg = "version: v1alpha1\nmachine:\n  network:\n    hostname:\n";
        assert_eq!(hostname_from_config(cfg), None);
    }

    #[test]
    fn hostname_outside_machine_is_ignored() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
cluster:
  network:
    hostname: not-this-one
";
        assert_eq!(hostname_from_config(cfg), None);
    }

    #[test]
    fn multi_document_config_uses_v1alpha1_body() {
        let cfg = "\
version: v1alpha1
machine:
  network:
    hostname: from-v1alpha1
---
apiVersion: v1alpha1
kind: SideroLinkConfig
apiUrl: grpc://example
";
        assert_eq!(hostname_from_config(cfg).as_deref(), Some("from-v1alpha1"));
    }

    #[test]
    fn key_of_extracts_key() {
        assert_eq!(key_of("network: stuff"), "network");
        assert_eq!(key_of("machine:"), "machine");
    }

    #[test]
    fn scalar_parses_value() {
        assert_eq!(scalar("hostname: foo", "hostname"), Some("foo"));
        assert_eq!(scalar("hostname: 'bar'", "hostname"), Some("bar"));
        assert_eq!(scalar("other: foo", "hostname"), None);
    }

    #[test]
    fn machine_type_parsing() {
        assert_eq!(
            MachineType::parse("controlplane"),
            MachineType::ControlPlane
        );
        assert_eq!(MachineType::parse("worker"), MachineType::Worker);
        assert_eq!(MachineType::parse("init"), MachineType::Init);
        assert_eq!(MachineType::parse("bogus"), MachineType::Unknown);
        assert!(MachineType::ControlPlane.is_control_plane());
        assert!(MachineType::Init.is_control_plane());
        assert!(!MachineType::Worker.is_control_plane());
    }

    #[test]
    fn early_config_extracts_all_fields() {
        let ec = early_config(FULL_CONFIG);
        assert_eq!(ec.hostname.as_deref(), Some("node-01"));
        assert_eq!(ec.machine_type, Some(MachineType::ControlPlane));
        assert_eq!(ec.install_disk.as_deref(), Some("/dev/sda"));
    }

    #[test]
    fn first_iface_address_from_addresses_list() {
        let cfg = "\
version: v1alpha1
machine:
  network:
    interfaces:
      - interface: eth0
        addresses:
          - 10.0.2.15/24
      - interface: eth1
        addresses:
          - 192.168.1.5/24
";
        let a = first_iface_address(&v1alpha1_body(cfg)).unwrap();
        assert_eq!(a.addr, "10.0.2.15");
        assert_eq!(a.prefix, 24);
        // Also reachable via early_config.
        assert_eq!(early_config(cfg).first_iface_address, Some(a));
    }

    #[test]
    fn first_iface_address_singular_scalar() {
        let cfg = "\
version: v1alpha1
machine:
  network:
    interfaces:
      - interface: eth0
        address: 10.0.2.15/24
";
        let a = early_config(cfg).first_iface_address.unwrap();
        assert_eq!(a.addr, "10.0.2.15");
        assert_eq!(a.prefix, 24);
    }

    #[test]
    fn first_iface_address_bare_addr_is_slash_32() {
        let cfg = "\
version: v1alpha1
machine:
  network:
    interfaces:
      - interface: eth0
        addresses:
          - 10.0.2.15
";
        let a = early_config(cfg).first_iface_address.unwrap();
        assert_eq!(a.addr, "10.0.2.15");
        assert_eq!(a.prefix, 32);
    }

    #[test]
    fn first_iface_address_dhcp_only_yields_none() {
        let cfg = "\
version: v1alpha1
machine:
  network:
    interfaces:
      - interface: eth0
        dhcp: true
";
        assert_eq!(early_config(cfg).first_iface_address, None);
    }

    #[test]
    fn machine_config_dhcp_options_materialize_operator_specs() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
  network:
    interfaces:
      - interface: eth0
        dhcp: true
        dhcpOptions:
          routeMetric: 2048
          ipv4: false
          ipv6: true
          duidv6: 00030001aabbccddeeff
      - interface: eth1
        dhcp: true
cluster:
  controlPlane:
    endpoint: https://10.0.0.1:6443
";
        let operators = machine_config_dhcp_operators(cfg).unwrap();

        assert_eq!(operators.len(), 2);
        assert_eq!(operators[0].kind, os_network_domain::OperatorKind::Dhcp6);
        assert_eq!(operators[0].link_name, "eth0");
        assert_eq!(operators[0].route_metric, 2048);
        assert_eq!(
            operators[0].client_identifier,
            ClientIdentifierSpec::duid(vec![0, 3, 0, 1, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff])
        );
        assert_eq!(operators[1].kind, os_network_domain::OperatorKind::Dhcp4);
        assert_eq!(operators[1].link_name, "eth1");
        assert_eq!(
            operators[1].route_metric,
            os_network_domain::DEFAULT_ROUTE_METRIC
        );

        assert_eq!(early_config(cfg).dhcp_operators, operators);
    }

    #[test]
    fn machine_config_dhcp_defaults_to_ipv4_only() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
  network:
    interfaces:
      - interface: eth0
        dhcp: true
";
        let operators = machine_config_dhcp_operators(cfg).unwrap();

        assert_eq!(operators.len(), 1);
        assert_eq!(operators[0].kind, os_network_domain::OperatorKind::Dhcp4);
        assert_eq!(
            operators[0].route_metric,
            os_network_domain::DEFAULT_ROUTE_METRIC
        );
    }

    #[test]
    fn machine_config_dhcp_skips_ignored_legacy_interface() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
  network:
    interfaces:
      - interface: eth0
        dhcp: true
        ignore: true
        vlans:
          - vlanId: 100
            dhcp: true
cluster:
  controlPlane:
    endpoint: https://10.0.0.1:6443
";
        let operators = machine_config_dhcp_operators(cfg).unwrap();

        assert!(operators.is_empty());
        assert!(early_config(cfg).dhcp_operators.is_empty());
    }

    #[test]
    fn machine_config_does_not_synthesize_default_dhcp_without_link_status_inputs() {
        // Talos docs say default DHCP runs on physical interfaces with link, and
        // explicit link configuration disables that default. Early config
        // extraction currently has machine-config documents only, not LinkStatus
        // inputs, so the fail-safe characterization is to synthesize no default
        // DHCP operators here rather than guessing which links are physical/up.
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
cluster:
  controlPlane:
    endpoint: https://10.0.0.1:6443
";
        let operators = machine_config_dhcp_operators(cfg).unwrap();

        assert!(operators.is_empty());
        assert!(early_config(cfg).dhcp_operators.is_empty());
    }

    #[test]
    fn explicit_static_interface_suppresses_default_dhcp_unless_dhcp_is_enabled() {
        // Characterizes Talos' documented suppression rule: once explicit link
        // configuration is present, DHCP must be opted into on desired links.
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
  network:
    interfaces:
      - interface: eth0
        addresses:
          - 10.0.2.15/24
cluster:
  controlPlane:
    endpoint: https://10.0.0.1:6443
";
        let operators = machine_config_dhcp_operators(cfg).unwrap();

        assert!(operators.is_empty());
        assert!(early_config(cfg).dhcp_operators.is_empty());
    }

    #[test]
    fn machine_config_vlan_dhcp_materializes_operator_specs() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
  network:
    interfaces:
      - interface: eth0
        dhcp: false
        vlans:
          - vlanId: 100
            dhcp: true
            dhcpOptions:
              routeMetric: 4096
              ipv4: false
              ipv6: true
              duidv6: 00030001aabbccddeeff
          - vlanId: 200
            dhcp: true
cluster:
  controlPlane:
    endpoint: https://10.0.0.1:6443
";
        let operators = machine_config_dhcp_operators(cfg).unwrap();

        assert_eq!(operators.len(), 2);
        assert_eq!(operators[0].kind, os_network_domain::OperatorKind::Dhcp6);
        assert_eq!(operators[0].link_name, "eth0.100");
        assert_eq!(operators[0].route_metric, 4096);
        assert_eq!(
            operators[0].client_identifier,
            ClientIdentifierSpec::duid(vec![0, 3, 0, 1, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff])
        );
        assert_eq!(operators[1].kind, os_network_domain::OperatorKind::Dhcp4);
        assert_eq!(operators[1].link_name, "eth0.200");
        assert_eq!(
            operators[1].route_metric,
            os_network_domain::DEFAULT_ROUTE_METRIC
        );
    }

    #[test]
    fn machine_config_vlan_dhcp_uses_talos_hashed_long_link_name() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
  network:
    interfaces:
      - interface: enx12545f8c99cd
        dhcp: false
        vlans:
          - vlanId: 25
            dhcp: true
";
        let operators = machine_config_dhcp_operators(cfg).unwrap();

        assert_eq!(operators.len(), 1);
        assert_eq!(operators[0].kind, os_network_domain::OperatorKind::Dhcp4);
        assert_eq!(operators[0].link_name, "enx1ee6413.25");
    }

    #[test]
    fn machine_config_resolver_config_materializes_plain_dns_and_search_domains() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: ResolverConfig
nameservers:
  - address: 1.1.1.1
  - address: 2606:4700:4700::1111
searchDomains:
  domains:
    - cluster.local
    - example.com
";
        let specs = machine_config_resolver_specs(cfg).unwrap();

        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].layer, ConfigLayer::Configuration);
        assert_eq!(
            specs[0]
                .servers
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["1.1.1.1", "2606:4700:4700:0:0:0:0:1111"]
        );
        assert_eq!(
            specs[0].search_domains,
            vec!["cluster.local".to_string(), "example.com".to_string()]
        );
    }

    #[test]
    fn machine_config_resolver_config_rejects_encrypted_projection_without_downgrade() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: ResolverConfig
nameservers:
  - address: 9.9.9.9
    protocol: DoT
    tlsServerName: dns.quad9.net
";
        let err = machine_config_resolver_specs(cfg).unwrap_err();

        assert!(err.to_string().contains("not yet projectable"));
    }

    #[test]
    fn machine_config_resolver_config_rejects_hostdns_without_downgrade() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: ResolverConfig
hostDNS:
  enabled: true
";
        let err = machine_config_resolver_specs(cfg).unwrap_err();

        assert!(err.to_string().contains("hostDNS"));
        assert!(err.to_string().contains("not yet projectable"));
    }

    #[test]
    fn machine_config_resolver_config_rejects_disable_default_without_downgrade() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: ResolverConfig
searchDomains:
  disableDefault: true
";
        let err = machine_config_resolver_specs(cfg).unwrap_err();

        assert!(err.to_string().contains("disableDefault"));
        assert!(err.to_string().contains("not yet projectable"));
    }

    #[test]
    fn dhcpv4_config_multidoc_materializes_operator_spec_fields() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: DHCPv4Config
name: eth0
routeMetric: 2048
ignoreHostname: true
clientIdentifier: duid
duidRaw: 00:03:00:01:aa:bb:cc:dd:ee:ff
";
        let operators = machine_config_dhcp_operators(cfg).unwrap();

        assert_eq!(operators.len(), 1);
        assert_eq!(operators[0].kind, os_network_domain::OperatorKind::Dhcp4);
        assert_eq!(operators[0].link_name, "eth0");
        assert_eq!(operators[0].route_metric, 2048);
        assert!(operators[0].skip_hostname_request);
        assert_eq!(
            operators[0].client_identifier,
            ClientIdentifierSpec::duid(vec![0, 3, 0, 1, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff])
        );
    }

    #[test]
    fn dhcpv4_config_defaults_to_mac_client_identifier() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: DHCPv4Config
name: eth0
";
        let operators = machine_config_dhcp_operators(cfg).unwrap();

        assert_eq!(operators.len(), 1);
        assert_eq!(operators[0].kind, os_network_domain::OperatorKind::Dhcp4);
        assert_eq!(
            operators[0].route_metric,
            os_network_domain::DEFAULT_ROUTE_METRIC
        );
        assert_eq!(operators[0].client_identifier, ClientIdentifierSpec::mac());
    }

    #[test]
    fn dhcpv4_config_materializes_none_client_identifier() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: DHCPv4Config
name: eth0
clientIdentifier: none
";
        let operators = machine_config_dhcp_operators(cfg).unwrap();

        assert_eq!(operators.len(), 1);
        assert_eq!(operators[0].kind, os_network_domain::OperatorKind::Dhcp4);
        assert_eq!(operators[0].client_identifier, ClientIdentifierSpec::none());
    }

    #[test]
    fn dhcpv6_config_multidoc_materializes_operator_spec_fields() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: DHCPv6Config
name: eth0
routeMetric: 2048
ignoreHostname: true
clientIdentifier: duid
duidRaw: 00:03:00:01:aa:bb:cc:dd:ee:ff
";
        let operators = machine_config_dhcp_operators(cfg).unwrap();

        assert_eq!(operators.len(), 1);
        assert_eq!(operators[0].kind, os_network_domain::OperatorKind::Dhcp6);
        assert_eq!(operators[0].link_name, "eth0");
        assert_eq!(operators[0].route_metric, 2048);
        assert!(operators[0].skip_hostname_request);
        assert_eq!(
            operators[0].client_identifier,
            ClientIdentifierSpec::duid(vec![0, 3, 0, 1, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff])
        );
    }

    #[test]
    fn dhcpv6_config_defaults_to_mac_client_identifier() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: DHCPv6Config
name: eth0
";
        let operators = machine_config_dhcp_operators(cfg).unwrap();

        assert_eq!(operators.len(), 1);
        assert_eq!(operators[0].kind, os_network_domain::OperatorKind::Dhcp6);
        assert_eq!(
            operators[0].route_metric,
            os_network_domain::DEFAULT_ROUTE_METRIC
        );
        assert_eq!(operators[0].client_identifier, ClientIdentifierSpec::mac());
    }

    #[test]
    fn dhcpv6_config_materializes_none_client_identifier() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: DHCPv6Config
name: eth0
clientIdentifier: none
";
        let operators = machine_config_dhcp_operators(cfg).unwrap();

        assert_eq!(operators.len(), 1);
        assert_eq!(operators[0].client_identifier, ClientIdentifierSpec::none());
    }

    #[test]
    fn machine_config_volume_manager_projects_system_and_user_volume_docs() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
provisioning:
  minSize: 1073741824
  maxSize: 2147483648
---
apiVersion: v1alpha1
kind: UserVolumeConfig
name: Data-01
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
  minSize: 536870912
filesystem:
  type: ext4
";
        let manager = machine_config_volume_manager(cfg).unwrap();

        let ephemeral = manager.volume("EPHEMERAL").unwrap();
        assert_eq!(ephemeral.class, os_block_domain::VolumeClass::System);
        assert_eq!(ephemeral.priority, 3);
        assert_eq!(ephemeral.config.min_size, 1_073_741_824);
        assert_eq!(ephemeral.config.max_size, Some(2_147_483_648));
        assert_eq!(ephemeral.config.relative_max_size, None);
        assert!(!ephemeral.config.negative_max_size);
        assert_eq!(ephemeral.config.grow, None);

        let user = manager.volume("u-Data-01").unwrap();
        assert_eq!(user.class, os_block_domain::VolumeClass::User);
        assert_eq!(user.priority, 100);
        assert_eq!(user.config.match_label.as_deref(), Some("u-Data-01"));
        assert_eq!(
            user.config.disk_selector.as_deref(),
            Some("disk.transport == \"nvme\"")
        );
        assert_eq!(user.config.min_size, 536_870_912);
        assert_eq!(user.config.grow, None);
        assert_eq!(user.config.filesystem, Some(FilesystemType::Ext4));
    }

    #[test]
    fn machine_config_volume_manager_defaults_user_min_size_and_xfs() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: UserVolumeConfig
name: logs
provisioning:
  diskSelector:
    match: disk.transport == \"sata\"
  maxSize: 1073741824
";
        let manager = machine_config_volume_manager(cfg).unwrap();
        let user = manager.volume("u-logs").unwrap();

        assert_eq!(user.config.min_size, MIN_USER_VOLUME_SIZE);
        assert_eq!(user.config.max_size, Some(1_073_741_824));
        assert_eq!(user.config.relative_max_size, None);
        assert!(!user.config.negative_max_size);
        assert_eq!(user.config.filesystem, Some(FilesystemType::Xfs));
    }

    #[test]
    fn raw_disk_user_volume_projects_to_disk_volume_config() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: UserVolumeConfig
name: data
volumeType: disk
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
filesystem:
  type: ext4
";
        let manager = machine_config_volume_manager(cfg).unwrap();
        let user = manager.volume("u-data").unwrap();

        assert_eq!(user.class, os_block_domain::VolumeClass::User);
        assert_eq!(user.priority, 100);
        assert_eq!(user.config.volume_type, os_block_domain::VolumeType::Disk);
        assert_eq!(user.config.match_label, None);
        assert_eq!(
            user.config.disk_selector.as_deref(),
            Some("disk.transport == \"nvme\"")
        );
        assert_eq!(user.config.min_size, 0);
        assert_eq!(user.config.max_size, None);
        assert_eq!(user.config.relative_max_size, None);
        assert!(!user.config.negative_max_size);
        assert_eq!(user.config.grow, None);
        assert_eq!(user.config.filesystem, Some(FilesystemType::Ext4));
    }

    #[test]
    fn raw_volume_config_projects_to_unformatted_partition_volume_config() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: RawVolumeConfig
name: local-data
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
  minSize: 1073741824
  maxSize: 2147483648
  grow: true
";
        let manager = machine_config_volume_manager(cfg).unwrap();
        let raw = manager.volume("r-local-data").unwrap();

        assert_eq!(raw.class, os_block_domain::VolumeClass::User);
        assert_eq!(raw.priority, 100);
        assert_eq!(raw.config.volume_type, os_block_domain::VolumeType::Partition);
        assert_eq!(raw.config.match_label.as_deref(), Some("r-local-data"));
        assert_eq!(
            raw.config.disk_selector.as_deref(),
            Some("disk.transport == \"nvme\"")
        );
        assert_eq!(raw.config.min_size, 1_073_741_824);
        assert_eq!(raw.config.max_size, Some(2_147_483_648));
        assert_eq!(raw.config.relative_max_size, None);
        assert!(!raw.config.negative_max_size);
        assert_eq!(raw.config.grow, Some(true));
        assert_eq!(raw.config.filesystem, None);
        assert_eq!(
            raw.config.partition_match_policy,
            os_block_domain::PartitionMatchPolicy::FirstMatch
        );
    }

    #[test]
    fn raw_volume_config_defaults_min_size_and_projects_rich_max_size() {
        let max_only = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: RawVolumeConfig
name: rawdata
provisioning:
  diskSelector:
    match: disk.transport == \"sata\"
  maxSize: 1073741824
";
        let manager = machine_config_volume_manager(max_only).unwrap();
        let raw = manager.volume("r-rawdata").unwrap();
        assert_eq!(raw.config.min_size, MIN_USER_VOLUME_SIZE);
        assert_eq!(raw.config.max_size, Some(1_073_741_824));
        assert_eq!(raw.config.relative_max_size, None);
        assert!(!raw.config.negative_max_size);
        assert_eq!(raw.config.filesystem, None);

        let relative_max_size = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: RawVolumeConfig
name: rawdata
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
  maxSize: 80%
";
        let manager = machine_config_volume_manager(relative_max_size).unwrap();
        let raw = manager.volume("r-rawdata").unwrap();
        assert_eq!(raw.config.max_size, None);
        assert_eq!(raw.config.relative_max_size, Some(80));
        assert!(!raw.config.negative_max_size);

        let negative_max_size = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: RawVolumeConfig
name: rawdata
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
  maxSize: -1GiB
";
        let manager = machine_config_volume_manager(negative_max_size).unwrap();
        let raw = manager.volume("r-rawdata").unwrap();
        assert_eq!(raw.config.max_size, Some(1_073_741_824));
        assert_eq!(raw.config.relative_max_size, None);
        assert!(raw.config.negative_max_size);

        let encryption = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: RawVolumeConfig
name: rawdata
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
  maxSize: 1073741824
encryption: {}
";
        let err = machine_config_volume_manager(encryption).unwrap_err();
        assert_eq!(err.kind(), "unsupported");
        assert!(
            err.to_string()
                .contains("RawVolumeConfig rawdata encryption")
        );
    }

    #[test]
    fn machine_config_volume_manager_projects_rich_max_size_and_rejects_unsupported_storage_semantics()
     {
        let relative_max_size = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: UserVolumeConfig
name: data
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
  maxSize: 80%
";
        let manager = machine_config_volume_manager(relative_max_size).unwrap();
        let user = manager.volume("u-data").unwrap();
        assert_eq!(user.config.max_size, None);
        assert_eq!(user.config.relative_max_size, Some(80));
        assert!(!user.config.negative_max_size);

        let negative_relative_max_size = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: UserVolumeConfig
name: data
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
  maxSize: -25%
";
        let manager = machine_config_volume_manager(negative_relative_max_size).unwrap();
        let user = manager.volume("u-data").unwrap();
        assert_eq!(user.config.max_size, None);
        assert_eq!(user.config.relative_max_size, Some(25));
        assert!(user.config.negative_max_size);
    }

    #[test]
    fn machine_config_volume_manager_projects_imagecache_system_volume_defaults() {
        let image_cache = "\
version: v1alpha1
machine:
  type: worker
  token: tok
---
apiVersion: v1alpha1
kind: VolumeConfig
name: IMAGECACHE
provisioning:
  maxSize: 10737418240
";
        let manager = machine_config_volume_manager(image_cache).unwrap();
        let image_cache = manager.volume("IMAGECACHE").unwrap();
        assert_eq!(image_cache.class, os_block_domain::VolumeClass::System);
        assert_eq!(image_cache.priority, 4);
        assert_eq!(
            image_cache.config.match_label.as_deref(),
            Some("IMAGECACHE")
        );
        assert_eq!(
            image_cache.config.disk_selector.as_deref(),
            Some("system_disk")
        );
        assert_eq!(image_cache.config.min_size, 500 * 1024 * 1024);
        assert_eq!(image_cache.config.max_size, Some(10 * 1024 * 1024 * 1024));
        assert_eq!(image_cache.config.grow, Some(false));
        assert_eq!(image_cache.config.filesystem, Some(FilesystemType::Ext4));
    }

    #[test]
    fn machine_config_image_cache_local_enabled_reads_feature_flag() {
        let cfg = "\
version: v1alpha1
machine:
  type: worker
  features:
    imageCache:
      localEnabled: true
";

        assert!(machine_config_image_cache_local_enabled(cfg).unwrap());
    }

    #[test]
    fn machine_config_image_cache_local_enabled_defaults_false_and_rejects_non_bool() {
        let absent = "version: v1alpha1\nmachine:\n  type: worker\n";
        assert!(!machine_config_image_cache_local_enabled(absent).unwrap());

        let explicit_false = "\
version: v1alpha1
machine:
  features:
    imageCache:
      localEnabled: false
";
        assert!(!machine_config_image_cache_local_enabled(explicit_false).unwrap());

        let invalid = "\
version: v1alpha1
machine:
  features:
    imageCache:
      localEnabled: sometimes
";
        assert!(machine_config_image_cache_local_enabled(invalid).is_err());
    }

    #[test]
    fn first_iface_address_no_interfaces_yields_none() {
        let cfg = "version: v1alpha1\nmachine:\n  network:\n    hostname: h\n";
        assert_eq!(early_config(cfg).first_iface_address, None);
    }

    #[test]
    fn early_config_worker_without_install() {
        let cfg = "version: v1alpha1\nmachine:\n  type: worker\n  network:\n    hostname: w1\n";
        let ec = early_config(cfg);
        assert_eq!(ec.machine_type, Some(MachineType::Worker));
        assert_eq!(ec.hostname.as_deref(), Some("w1"));
        assert_eq!(ec.install_disk, None);
    }

    #[test]
    fn machine_type_does_not_leak_from_cluster() {
        // A `type:` under cluster must not be read as machine.type.
        let cfg = "\
version: v1alpha1
machine:
  network:
    hostname: x
cluster:
  type: should-not-match
";
        let ec = early_config(cfg);
        assert_eq!(ec.machine_type, None);
    }

    #[test]
    fn empty_config_is_all_none() {
        let ec = early_config("");
        assert_eq!(ec, EarlyConfig::default());
    }
}
