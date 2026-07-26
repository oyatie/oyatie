//! Corpus config loading, introspection, and validation.
//!
//! This module loads the v1alpha1 machine-config YAML files in
//! `testdata/configs/` using the dependency-free [`crate::yaml`] parser, maps
//! the parsed tree into a small introspection model, and classifies each file
//! `valid` / `invalid` the same way the real Talos loader
//! (`configloader.NewFromBytes`) plus `provider.Validate` does for these cases.
//!
//! The behavior is pinned to the oracle captured from the Talos 1.10.5 tree in
//! container mode (`RuntimeMode{ String:"container", RequiresInstall:false,
//! InContainer:true }`):
//!
//! - **Load failure** (malformed YAML that does not decode) → `invalid`, and the
//!   `machine_type` / `hostname` / `install_disk` accessors all return `""`.
//! - **Load ok, Validate error** → `invalid`, accessors still resolve from the
//!   loaded tree.
//! - **Load ok, Validate ok** → `valid`.
//!
//! Container mode does not require an install disk, so `machine.install` is
//! optional. The validation rules reproduced here are the subset the corpus
//! actually exercises:
//!
//! 1. `machine:` block is required (its absence is a load-level "machine
//!    instructions are required", but the loaded provider still resolves
//!    `Machine().Type()` to `worker` via `ParseType("")`).
//! 2. `cluster.controlPlane.endpoint` is required.
//! 3. The machine type must be a known type (`init` / `controlplane` /
//!    `worker`); an unrecognized value such as `foobar` resolves to `unknown`
//!    and is rejected.
//! 4. On non-control-plane (worker) nodes the issuing CA key
//!    (`machine.ca.key`) is not allowed.

use crate::yaml::{self, Yaml};
use os_kernel::address::NodeAddress;
use os_kernel::machine_type::MachineType;

/// Talos default cluster DNS domain when `cluster.network.dnsDomain` is omitted.
const DEFAULT_DNS_DOMAIN: &str = "cluster.local";
/// Talos default pod CIDR when `cluster.network.podSubnets` is omitted.
const DEFAULT_POD_SUBNET: &str = "10.244.0.0/16";
/// Talos default service CIDR when `cluster.network.serviceSubnets` is omitted.
const DEFAULT_SERVICE_SUBNET: &str = "10.96.0.0/12";
/// Talos default kubelet image when `machine.kubelet.image` is omitted.
///
/// Mirrors `Kubelet().Image()` falling back to
/// `fmt.Sprintf("%s:v%s", constants.KubeletImage, constants.DefaultKubernetesVersion)`
/// (`KubeletImage = "ghcr.io/siderolabs/kubelet"`,
/// `DefaultKubernetesVersion = "1.36.1"`).
const DEFAULT_KUBELET_IMAGE: &str = "ghcr.io/siderolabs/kubelet:v1.36.1";

/// A loaded corpus config plus its computed introspection.
///
/// Construct via [`CorpusConfig::load`]. If the YAML fails to decode, `load`
/// returns [`LoadError`] and the caller treats the file as `invalid` with all
/// accessors empty (matching the oracle's load-failure record).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusConfig {
    /// The decoded YAML document root.
    root: Yaml,
}

/// A load-level failure: the document did not decode into the supported subset.
///
/// Mirrors `configloader.NewFromBytes` returning an error: the resulting record
/// is `invalid` with empty `machine_type` / `hostname` / `install_disk`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadError {
    /// Human-readable reason (the underlying parse error message).
    pub reason: String,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "config load error: {}", self.reason)
    }
}

impl std::error::Error for LoadError {}

/// The outcome of validating a corpus config: `valid` or `invalid` with reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Validity {
    /// The config loaded and passed validation.
    Valid,
    /// The config failed validation (or, at the call site, failed to load).
    Invalid(String),
}

impl Validity {
    /// Whether this outcome is `valid`.
    pub fn is_valid(&self) -> bool {
        matches!(self, Validity::Valid)
    }
}

impl CorpusConfig {
    /// Load and decode a config document from YAML bytes.
    ///
    /// # Errors
    ///
    /// Returns [`LoadError`] if the document does not decode as the YAML subset
    /// (the oracle's "load failure" case).
    pub fn load(source: &str) -> Result<Self, LoadError> {
        let root = yaml::parse(source).map_err(|e| LoadError {
            reason: e.to_string(),
        })?;
        Ok(CorpusConfig { root })
    }

    /// The `machine:` mapping, if present.
    fn machine(&self) -> Option<&Yaml> {
        self.root
            .get("machine")
            .filter(|m| m.as_mapping().is_some())
    }

    /// The `cluster:` mapping, if present.
    fn cluster(&self) -> Option<&Yaml> {
        self.root
            .get("cluster")
            .filter(|c| c.as_mapping().is_some())
    }

    /// The machine type, matching Go `cfg.Machine().Type().String()`.
    ///
    /// `ParseType` maps `init` / `controlplane` / `worker` to themselves,
    /// `""` / `join` to `worker`, and anything else to `unknown`. When the
    /// `machine:` block is absent the loaded provider still resolves a default
    /// `worker` via `ParseType("")`, so this returns `"worker"`, matching the
    /// oracle for `10-invalid-missing-machine.yaml`.
    pub fn machine_type(&self) -> String {
        let raw = self.machine().and_then(|m| m.get_str("type")).unwrap_or("");
        parse_type(raw).as_str().to_string()
    }

    /// The hostname (`machine.network.hostname`), or `""` if unset.
    ///
    /// Matches `provider.NetworkHostnameConfig().Hostname()` which falls back to
    /// the v1alpha1 `machine.network.hostname`.
    pub fn hostname(&self) -> String {
        self.machine()
            .and_then(|m| m.get("network"))
            .and_then(|n| n.get_str("hostname"))
            .unwrap_or("")
            .to_string()
    }

    /// The install disk (`machine.install.disk`), or `""` if unset.
    ///
    /// Matches `cfg.Machine().Install().Disk()`.
    pub fn install_disk(&self) -> String {
        self.machine()
            .and_then(|m| m.get("install"))
            .and_then(|i| i.get_str("disk"))
            .unwrap_or("")
            .to_string()
    }

    /// The install image (`machine.install.image`), or `""`.
    ///
    /// Matches `cfg.Machine().Install().Image()`.
    pub fn install_image(&self) -> String {
        self.machine()
            .and_then(|m| m.get("install"))
            .and_then(|i| i.get_str("image"))
            .unwrap_or("")
            .to_string()
    }

    /// The install wipe flag (`machine.install.wipe`, the v1alpha1 `wipe` field
    /// surfaced as `Install().Zero()`), serialized `"true"`/`"false"`.
    ///
    /// `"false"` when the install section (or the flag) is absent, matching the
    /// oracle which seeds `machine.install.wipe = "false"`.
    pub fn install_wipe(&self) -> String {
        let wipe = self
            .machine()
            .and_then(|m| m.get("install"))
            .and_then(|i| i.get("wipe"))
            .and_then(Yaml::as_bool)
            .unwrap_or(false);
        if wipe {
            "true".to_string()
        } else {
            "false".to_string()
        }
    }

    /// The kubelet image (`machine.kubelet.image`).
    ///
    /// Matches `cfg.Machine().Kubelet().Image()`, which falls back to the Talos
    /// default ([`DEFAULT_KUBELET_IMAGE`]) when the field (or kubelet section) is
    /// absent or empty — so this never returns `""` for a loaded config.
    pub fn kubelet_image(&self) -> String {
        self.machine()
            .and_then(|m| m.get("kubelet"))
            .and_then(|k| k.get_str("image"))
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_KUBELET_IMAGE)
            .to_string()
    }

    /// The UTF-8 byte length of the join token (`machine.token`) as a decimal
    /// string, matching the oracle `len(Security().Token())`. `"0"` when absent.
    ///
    /// The value itself is redacted; only the length is exposed.
    pub fn token_len(&self) -> String {
        let token = self
            .machine()
            .and_then(|m| m.get_str("token"))
            .unwrap_or("");
        token.len().to_string()
    }

    /// The resolved nameservers (`machine.network.nameservers`), comma-joined in
    /// slice order with no spaces, matching the oracle's
    /// `NetworkResolverConfig().Resolvers()` projection.
    ///
    /// Each entry is parsed as an IP address; non-IP entries are dropped (the
    /// provider only surfaces parseable resolver addresses), and the survivors
    /// are re-serialized via their canonical address form (matching
    /// `netip.Addr.String()` for IPv4).
    pub fn nameservers(&self) -> String {
        let Some(seq) = self
            .machine()
            .and_then(|m| m.get("network"))
            .and_then(|n| n.get("nameservers"))
            .and_then(Yaml::as_sequence)
        else {
            return String::new();
        };
        let addrs: Vec<String> = seq
            .iter()
            .filter_map(Yaml::as_str)
            .filter_map(|raw| NodeAddress::parse(raw).ok().map(|a| a.to_string()))
            .collect();
        addrs.join(",")
    }

    /// The first legacy interface DHCP option map.
    ///
    /// The oracle path uses `interfaces[]` rather than an indexed selector. The
    /// corpus keeps this deterministic by projecting the first interface, which
    /// is how the existing list-shaped field paths in the generated oracle are
    /// consumed for scalar comparisons. Missing interfaces/options use Talos'
    /// DHCPOptions defaults.
    fn first_interface_dhcp_options(&self) -> Option<&Yaml> {
        self.machine()
            .and_then(|m| m.get("network"))
            .and_then(|n| n.get("interfaces"))
            .and_then(Yaml::as_sequence)
            .and_then(|items| items.first())
            .and_then(|item| item.get("dhcpOptions"))
            .filter(|opts| opts.as_mapping().is_some())
    }

    /// The first interface DHCP route metric, defaulting to `0`.
    pub fn interface_dhcp_route_metric(&self) -> String {
        self.first_interface_dhcp_options()
            .and_then(|opts| opts.get_str("routeMetric"))
            .unwrap_or("0")
            .to_string()
    }

    /// The first interface DHCP IPv4 enablement, defaulting to `true`.
    pub fn interface_dhcp_ipv4(&self) -> String {
        self.first_interface_dhcp_options()
            .and_then(|opts| opts.get("ipv4"))
            .and_then(Yaml::as_bool)
            .unwrap_or(true)
            .to_string()
    }

    /// The first interface DHCP IPv6 enablement, defaulting to `false`.
    pub fn interface_dhcp_ipv6(&self) -> String {
        self.first_interface_dhcp_options()
            .and_then(|opts| opts.get("ipv6"))
            .and_then(Yaml::as_bool)
            .unwrap_or(false)
            .to_string()
    }

    /// The first interface DHCPv6 DUID override, or `""` when unset.
    pub fn interface_dhcp_duidv6(&self) -> String {
        self.first_interface_dhcp_options()
            .and_then(|opts| opts.get_str("duidv6"))
            .unwrap_or("")
            .to_string()
    }

    /// The sysctls map (`machine.sysctls`) canonically serialized as
    /// `k=v` pairs joined with `;`, keys sorted ascending byte-wise. `""` when
    /// the section is empty or absent. Matches the oracle `kvJoin(m.Sysctls())`.
    pub fn sysctls(&self) -> String {
        kv_join(self.machine().and_then(|m| m.get("sysctls")))
    }

    /// The environment map (`machine.env`) canonically serialized the same way
    /// as [`Self::sysctls`]. Values may contain `,` (the separator is `;`).
    /// Matches the oracle `kvJoin(m.Env())`.
    pub fn env(&self) -> String {
        kv_join(self.machine().and_then(|m| m.get("env")))
    }

    /// The cluster name (`cluster.clusterName`), or `""`. Matches `Cluster().Name()`.
    pub fn cluster_name(&self) -> String {
        self.cluster()
            .and_then(|c| c.get_str("clusterName"))
            .unwrap_or("")
            .to_string()
    }

    /// The cluster id (`cluster.id`), or `""`. Matches `Cluster().ID()`.
    pub fn cluster_id(&self) -> String {
        self.cluster()
            .and_then(|c| c.get_str("id"))
            .unwrap_or("")
            .to_string()
    }

    /// The control-plane endpoint (`cluster.controlPlane.endpoint`), or `""`.
    ///
    /// Matches the oracle `Cluster().Endpoint().String()` (the parsed `*url.URL`
    /// re-serialized). For the corpus, which carries already-canonical
    /// `https://host:port` URLs, this is the verbatim value.
    pub fn endpoint(&self) -> String {
        self.cluster()
            .and_then(|c| c.get("controlPlane"))
            .and_then(|cp| cp.get_str("endpoint"))
            .unwrap_or("")
            .to_string()
    }

    /// The cluster DNS domain (`cluster.network.dnsDomain`).
    ///
    /// The provider fills the Talos default (`cluster.local`) when the network
    /// section or field is omitted, so this returns that default rather than `""`.
    pub fn dns_domain(&self) -> String {
        self.cluster()
            .and_then(|c| c.get("network"))
            .and_then(|n| n.get_str("dnsDomain"))
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_DNS_DOMAIN)
            .to_string()
    }

    /// The pod CIDRs (`cluster.network.podSubnets`), comma-joined in slice order.
    ///
    /// Defaults to the Talos default (`10.244.0.0/16`) when omitted, matching the
    /// oracle `Cluster().Network().PodCIDRs()`.
    pub fn pod_subnets(&self) -> String {
        cidr_list(self, "podSubnets", DEFAULT_POD_SUBNET)
    }

    /// The service CIDRs (`cluster.network.serviceSubnets`), comma-joined in
    /// slice order. Defaults to the Talos default (`10.96.0.0/12`) when omitted,
    /// matching the oracle `Cluster().Network().ServiceCIDRs()`.
    pub fn service_subnets(&self) -> String {
        cidr_list(self, "serviceSubnets", DEFAULT_SERVICE_SUBNET)
    }

    /// Resolve a single oracle field path to its canonical string value.
    ///
    /// The path set mirrors the oracle `fieldPathOrder`; an unrecognized path
    /// returns `None`. This lets the diff harness iterate the field paths
    /// generically and compare each value byte-for-byte against the oracle.
    pub fn config_field(&self, path: &str) -> Option<String> {
        let value = match path {
            "machine.type" => self.machine_type(),
            "machine.token.len" => self.token_len(),
            "machine.network.hostname" => self.hostname(),
            "machine.install.disk" => self.install_disk(),
            "machine.install.image" => self.install_image(),
            "machine.install.wipe" => self.install_wipe(),
            "machine.kubelet.image" => self.kubelet_image(),
            "machine.network.nameservers" => self.nameservers(),
            "machine.network.interfaces[].dhcpOptions.routeMetric" => {
                self.interface_dhcp_route_metric()
            }
            "machine.network.interfaces[].dhcpOptions.ipv4" => self.interface_dhcp_ipv4(),
            "machine.network.interfaces[].dhcpOptions.ipv6" => self.interface_dhcp_ipv6(),
            "machine.network.interfaces[].dhcpOptions.duidv6" => self.interface_dhcp_duidv6(),
            "machine.sysctls" => self.sysctls(),
            "machine.env" => self.env(),
            "cluster.name" => self.cluster_name(),
            "cluster.id" => self.cluster_id(),
            "cluster.controlPlane.endpoint" => self.endpoint(),
            "cluster.network.dnsDomain" => self.dns_domain(),
            "cluster.network.podSubnets" => self.pod_subnets(),
            "cluster.network.serviceSubnets" => self.service_subnets(),
            _ => return None,
        };
        Some(value)
    }

    /// Validate this config the same way the Go loader + `Validate(container)`
    /// classifies the corpus files.
    ///
    /// Returns [`Validity::Valid`] when the file passes, or
    /// [`Validity::Invalid`] with the first failing reason.
    pub fn validate(&self) -> Validity {
        // 1. machine: block required.
        let Some(machine) = self.machine() else {
            return Validity::Invalid("machine instructions are required".to_string());
        };

        // 2. cluster.controlPlane.endpoint required.
        let endpoint = self
            .cluster()
            .and_then(|c| c.get("controlPlane"))
            .and_then(|cp| cp.get_str("endpoint"))
            .filter(|e| !e.is_empty());
        if endpoint.is_none() {
            return Validity::Invalid("cluster controlplane endpoint is required".to_string());
        }

        // 3. machine type must be known.
        let raw_type = machine.get_str("type").unwrap_or("");
        let mtype = parse_type(raw_type);

        // 4. issuing CA / CA key rules by node type.
        let ca = machine.get("ca");
        let ca_key = ca.and_then(|c| c.get_str("key")).unwrap_or("");
        let ca_crt = ca.and_then(|c| c.get_str("crt")).unwrap_or("");

        // issuing CA (or accepted CAs) required; the corpus always supplies a
        // crt, so an absent/empty ca block would trip this.
        if ca_crt.is_empty() && ca_key.is_empty() {
            return Validity::Invalid(
                "issuing CA or some accepted CAs are required (.machine.ca, machine.acceptedCAs)"
                    .to_string(),
            );
        }

        match mtype {
            MachineType::Init | MachineType::ControlPlane => {
                // Control-plane nodes require an issuing CA key.
                if ca_crt.is_empty() {
                    return Validity::Invalid("issuing CA is required (.machine.ca)".to_string());
                }
                if ca_key.is_empty() {
                    return Validity::Invalid(
                        "issuing CA key is required for controlplane nodes (.machine.ca.key)"
                            .to_string(),
                    );
                }
            }
            MachineType::Worker => {
                // Worker nodes must NOT carry an issuing CA key.
                if !ca_key.is_empty() {
                    return Validity::Invalid(
                        "issuing Talos API CA key is not allowed on non-controlplane nodes (.machine.ca)"
                            .to_string(),
                    );
                }
            }
            MachineType::Unknown => {
                return Validity::Invalid(format!("unknown machine type {raw_type:?}"));
            }
        }

        Validity::Valid
    }
}

/// The fixed emission order of oracle field paths within each valid file.
///
/// Mirrors the oracle `fieldPathOrder`; the diff harness iterates this slice so
/// the Rust output appears in the exact same order as `config_fields.tsv`.
pub const FIELD_PATH_ORDER: [&str; 20] = [
    "machine.type",
    "machine.token.len",
    "machine.network.hostname",
    "machine.install.disk",
    "machine.install.image",
    "machine.install.wipe",
    "machine.kubelet.image",
    "machine.network.nameservers",
    "machine.network.interfaces[].dhcpOptions.routeMetric",
    "machine.network.interfaces[].dhcpOptions.ipv4",
    "machine.network.interfaces[].dhcpOptions.ipv6",
    "machine.network.interfaces[].dhcpOptions.duidv6",
    "machine.sysctls",
    "machine.env",
    "cluster.name",
    "cluster.id",
    "cluster.controlPlane.endpoint",
    "cluster.network.dnsDomain",
    "cluster.network.podSubnets",
    "cluster.network.serviceSubnets",
];

/// Canonically serialize a YAML mapping of string scalars as `k=v` pairs joined
/// with `;`, keys sorted ascending byte-wise (matching Go `sort.Strings`).
///
/// Returns `""` when the value is absent, not a mapping, or empty. Mirrors the
/// oracle `kvJoin`.
fn kv_join(map: Option<&Yaml>) -> String {
    let Some(mapping) = map.and_then(Yaml::as_mapping) else {
        return String::new();
    };
    if mapping.is_empty() {
        return String::new();
    }
    // `BTreeMap` already orders keys by Rust's `Ord for String`, which is
    // byte-wise lexicographic — identical to Go `sort.Strings`.
    let parts: Vec<String> = mapping
        .iter()
        .filter_map(|(k, v)| v.as_str().map(|val| format!("{k}={val}")))
        .collect();
    parts.join(";")
}

/// Comma-join a `cluster.network.<which>` CIDR list in slice order, defaulting
/// to `default_cidr` when the list (or the network section) is absent/empty.
fn cidr_list(cfg: &CorpusConfig, which: &str, default_cidr: &str) -> String {
    let list = cfg
        .cluster()
        .and_then(|c| c.get("network"))
        .and_then(|n| n.get(which))
        .and_then(Yaml::as_sequence);
    match list {
        Some(seq) if !seq.is_empty() => seq
            .iter()
            .filter_map(Yaml::as_str)
            .collect::<Vec<_>>()
            .join(","),
        _ => default_cidr.to_string(),
    }
}

/// Parse a raw `machine.type` string the way Go `machine.ParseType` does.
fn parse_type(raw: &str) -> MachineType {
    match raw {
        "init" => MachineType::Init,
        "controlplane" => MachineType::ControlPlane,
        "worker" | "join" | "" => MachineType::Worker,
        _ => MachineType::Unknown,
    }
}

/// Load a config and produce the four oracle TSV fields:
/// `(valid, machine_type, hostname, install_disk)`.
///
/// On a load failure the record is `("invalid", "", "", "")`, matching the
/// oracle's behavior for malformed YAML.
pub fn load_record(source: &str) -> (Validity, String, String, String) {
    match CorpusConfig::load(source) {
        Err(e) => (
            Validity::Invalid(e.reason),
            String::new(),
            String::new(),
            String::new(),
        ),
        Ok(cfg) => {
            let machine_type = cfg.machine_type();
            let hostname = cfg.hostname();
            let install_disk = cfg.install_disk();
            (cfg.validate(), machine_type, hostname, install_disk)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// Root of the corpus fixtures. Under cargo, CARGO_MANIFEST_DIR points at the
    /// crate dir; under buck2 the rust_test target injects OYA_TESTDATA_DIR via
    /// `$(location :testdata)` (CARGO_MANIFEST_DIR is not defined there).
    fn testdata_root() -> PathBuf {
        if let Ok(dir) = std::env::var("OYA_TESTDATA_DIR") {
            return PathBuf::from(dir);
        }
        PathBuf::from(option_env!("CARGO_MANIFEST_DIR").unwrap_or(".")).join("testdata")
    }

    fn configs_dir() -> PathBuf {
        testdata_root().join("configs")
    }

    fn read(name: &str) -> String {
        fs::read_to_string(configs_dir().join(name)).unwrap_or_else(|e| panic!("read {name}: {e}"))
    }

    #[test]
    fn parse_type_matches_go() {
        assert_eq!(parse_type("init"), MachineType::Init);
        assert_eq!(parse_type("controlplane"), MachineType::ControlPlane);
        assert_eq!(parse_type("worker"), MachineType::Worker);
        assert_eq!(parse_type("join"), MachineType::Worker);
        assert_eq!(parse_type(""), MachineType::Worker);
        assert_eq!(parse_type("foobar"), MachineType::Unknown);
    }

    #[test]
    fn cp_full_introspection() {
        let cfg = CorpusConfig::load(&read("01-controlplane-full.yaml")).unwrap();
        assert_eq!(cfg.machine_type(), "controlplane");
        assert_eq!(cfg.hostname(), "cp-01");
        assert_eq!(cfg.install_disk(), "/dev/sda");
        assert!(cfg.validate().is_valid());
    }

    #[test]
    fn worker_install_no_hostname() {
        let cfg = CorpusConfig::load(&read("07-worker-install-no-hostname.yaml")).unwrap();
        assert_eq!(cfg.machine_type(), "worker");
        assert_eq!(cfg.hostname(), "");
        assert_eq!(cfg.install_disk(), "/dev/nvme0n1");
        assert!(cfg.validate().is_valid());
    }

    #[test]
    fn certsans_parses_and_valid() {
        let cfg = CorpusConfig::load(&read("08-controlplane-certsans.yaml")).unwrap();
        assert_eq!(cfg.machine_type(), "controlplane");
        assert!(cfg.validate().is_valid());
    }

    #[test]
    fn init_full() {
        let cfg = CorpusConfig::load(&read("14-init-full.yaml")).unwrap();
        assert_eq!(cfg.machine_type(), "init");
        assert_eq!(cfg.hostname(), "init-14");
        assert_eq!(cfg.install_disk(), "/dev/sda");
        assert!(cfg.validate().is_valid());
    }

    #[test]
    fn bad_type_is_unknown_and_invalid() {
        let (validity, mtype, host, disk) = load_record(&read("09-invalid-bad-type.yaml"));
        assert!(!validity.is_valid());
        assert_eq!(mtype, "unknown");
        assert_eq!(host, "");
        assert_eq!(disk, "");
    }

    #[test]
    fn missing_machine_is_worker_and_invalid() {
        let (validity, mtype, host, disk) = load_record(&read("10-invalid-missing-machine.yaml"));
        assert!(!validity.is_valid());
        assert_eq!(mtype, "worker");
        assert_eq!(host, "");
        assert_eq!(disk, "");
    }

    #[test]
    fn malformed_load_fails_empty_fields() {
        let (validity, mtype, host, disk) = load_record(&read("11-invalid-malformed.yaml"));
        assert!(!validity.is_valid());
        assert_eq!(mtype, "");
        assert_eq!(host, "");
        assert_eq!(disk, "");
    }

    #[test]
    fn no_endpoint_invalid_but_type_resolves() {
        let (validity, mtype, _h, _d) = load_record(&read("12-invalid-no-endpoint.yaml"));
        assert!(!validity.is_valid());
        assert_eq!(mtype, "controlplane");
    }

    #[test]
    fn worker_ca_key_invalid() {
        let (validity, mtype, _h, _d) = load_record(&read("13-invalid-worker-ca-key.yaml"));
        assert!(!validity.is_valid());
        assert_eq!(mtype, "worker");
    }

    #[test]
    fn rich_controlplane_fields() {
        let cfg = CorpusConfig::load(&read("15-controlplane-rich-network.yaml")).unwrap();
        assert_eq!(cfg.token_len(), "23");
        assert_eq!(cfg.install_image(), "ghcr.io/siderolabs/installer:v1.7.0");
        assert_eq!(cfg.install_wipe(), "true");
        assert_eq!(cfg.kubelet_image(), "ghcr.io/siderolabs/kubelet:v1.36.1");
        assert_eq!(cfg.nameservers(), "8.8.8.8,1.1.1.1,9.9.9.9");
        assert_eq!(
            cfg.sysctls(),
            "net.core.somaxconn=65535;net.ipv4.ip_forward=1;vm.max_map_count=262144"
        );
        assert_eq!(
            cfg.env(),
            "GRPC_GO_LOG_SEVERITY_LEVEL=info;GRPC_GO_LOG_VERBOSITY_LEVEL=99"
        );
        assert_eq!(cfg.cluster_name(), "prod-cluster");
        assert_eq!(cfg.cluster_id(), "dGhpcy1pcy1jbHVzdGVyLWlk");
        assert_eq!(cfg.endpoint(), "https://10.0.0.1:6443");
        assert_eq!(cfg.dns_domain(), "cluster.local");
        assert_eq!(cfg.pod_subnets(), "10.244.0.0/16");
        assert_eq!(cfg.service_subnets(), "10.96.0.0/12");
    }

    #[test]
    fn worker_dual_stack_and_comma_env() {
        let cfg = CorpusConfig::load(&read("16-worker-rich.yaml")).unwrap();
        assert_eq!(cfg.install_wipe(), "false");
        assert_eq!(cfg.nameservers(), "192.168.1.1");
        assert_eq!(cfg.interface_dhcp_route_metric(), "2048");
        assert_eq!(cfg.interface_dhcp_ipv4(), "false");
        assert_eq!(cfg.interface_dhcp_ipv6(), "true");
        assert_eq!(cfg.interface_dhcp_duidv6(), "00030001aabbccddeeff");
        assert_eq!(cfg.pod_subnets(), "10.244.0.0/16,fd00:10:244::/56");
        assert_eq!(cfg.service_subnets(), "10.96.0.0/12,fd00:10:96::/112");
        // Env value legitimately contains a comma; the separator is ';'.
        assert_eq!(
            cfg.env(),
            "HTTP_PROXY=http://proxy.example.com:8080;NO_PROXY=10.0.0.0/8,localhost"
        );
        assert_eq!(cfg.cluster_name(), "");
        assert_eq!(cfg.cluster_id(), "");
    }

    #[test]
    fn install_image_only_custom_dns() {
        let cfg = CorpusConfig::load(&read("17-controlplane-install-image-only.yaml")).unwrap();
        assert_eq!(
            cfg.install_image(),
            "factory.talos.dev/installer/abc123:v1.7.0"
        );
        assert_eq!(cfg.install_wipe(), "false");
        assert_eq!(cfg.dns_domain(), "my.cluster.internal");
        assert_eq!(cfg.nameservers(), "");
    }

    #[test]
    fn sysctls_env_key_sorting_and_network_defaults() {
        let cfg = CorpusConfig::load(&read("18-worker-sysctls-env-only.yaml")).unwrap();
        // Keys must be byte-wise ascending regardless of source order.
        assert_eq!(
            cfg.sysctls(),
            "fs.inotify.max_user_instances=8192;fs.inotify.max_user_watches=1048576;kernel.pid_max=65536;net.bridge.bridge-nf-call-iptables=1"
        );
        assert_eq!(cfg.env(), "AAA_FIRST=aaa;MMM_MID=mid;ZZZ_LAST=zzz");
        // cluster.network omitted -> Talos defaults surface.
        assert_eq!(cfg.dns_domain(), "cluster.local");
        assert_eq!(cfg.pod_subnets(), "10.244.0.0/16");
        assert_eq!(cfg.service_subnets(), "10.96.0.0/12");
    }

    #[test]
    fn init_cluster_network_custom_cidrs() {
        let cfg = CorpusConfig::load(&read("19-init-cluster-network.yaml")).unwrap();
        assert_eq!(cfg.machine_type(), "init");
        assert_eq!(cfg.cluster_name(), "init-cluster");
        assert_eq!(cfg.cluster_id(), "aW5pdC1jbHVzdGVyLWlkZW50aWZpZXI=");
        assert_eq!(cfg.nameservers(), "10.0.0.53,10.0.0.54");
        assert_eq!(cfg.pod_subnets(), "100.64.0.0/14");
        assert_eq!(cfg.service_subnets(), "100.96.0.0/12");
    }

    #[test]
    fn worker_minimal_kubelet_all_defaults() {
        let cfg = CorpusConfig::load(&read("20-worker-minimal-kubelet.yaml")).unwrap();
        assert_eq!(cfg.kubelet_image(), "ghcr.io/siderolabs/kubelet:v1.36.1");
        assert_eq!(cfg.install_image(), "");
        assert_eq!(cfg.install_disk(), "");
        assert_eq!(cfg.install_wipe(), "false");
        assert_eq!(cfg.hostname(), "");
        assert_eq!(cfg.sysctls(), "");
        assert_eq!(cfg.env(), "");
        assert_eq!(cfg.interface_dhcp_route_metric(), "0");
        assert_eq!(cfg.interface_dhcp_ipv4(), "true");
        assert_eq!(cfg.interface_dhcp_ipv6(), "false");
        assert_eq!(cfg.interface_dhcp_duidv6(), "");
        assert_eq!(cfg.dns_domain(), "cluster.local");
        assert_eq!(cfg.pod_subnets(), "10.244.0.0/16");
        assert_eq!(cfg.service_subnets(), "10.96.0.0/12");
    }

    #[test]
    fn config_field_dispatch_and_unknown() {
        let cfg = CorpusConfig::load(&read("15-controlplane-rich-network.yaml")).unwrap();
        for path in FIELD_PATH_ORDER {
            assert!(
                cfg.config_field(path).is_some(),
                "missing field path {path}"
            );
        }
        assert_eq!(
            cfg.config_field("machine.type").as_deref(),
            Some("controlplane")
        );
        assert_eq!(cfg.config_field("nonexistent.path"), None);
    }

    /// Every valid corpus file resolves all 20 field paths against the oracle's
    /// `config_fields.tsv` byte-for-byte.
    #[test]
    fn config_fields_match_oracle() {
        let tsv_path = testdata_root().join("vectors").join("config_fields.tsv");
        let body = fs::read_to_string(&tsv_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", tsv_path.display()));

        let mut checked = 0usize;
        for line in body.lines() {
            if line.is_empty() {
                continue;
            }
            let cols: Vec<&str> = line.split('\t').collect();
            assert!(
                cols.len() == 2 || cols.len() == 3,
                "expected 2 or 3 columns: {line:?}"
            );
            let (file, path, expected) = (cols[0], cols[1], cols.get(2).copied().unwrap_or(""));
            let cfg = CorpusConfig::load(&read(file)).unwrap();
            let actual = cfg
                .config_field(path)
                .unwrap_or_else(|| panic!("unknown field path {path} for {file}"));
            assert_eq!(
                actual, expected,
                "field {path} mismatch for {file}\n expected = {expected:?} (Go)\n actual   = {actual:?} (Rust)"
            );
            checked += 1;
        }
        // 15 valid files * 20 field paths.
        assert_eq!(
            checked, 300,
            "expected 300 field-path records, checked {checked}"
        );
    }

    /// End-to-end: every corpus file matches its expected TSV record.
    #[test]
    fn full_corpus_matches_expected() {
        // (file, valid, machineType, hostname, installDisk)
        let expected: &[(&str, bool, &str, &str, &str)] = &[
            (
                "01-controlplane-full.yaml",
                true,
                "controlplane",
                "cp-01",
                "/dev/sda",
            ),
            (
                "02-controlplane-no-hostname-no-install.yaml",
                true,
                "controlplane",
                "",
                "",
            ),
            (
                "03-controlplane-hostname-no-install.yaml",
                true,
                "controlplane",
                "cp-03",
                "",
            ),
            (
                "04-worker-full.yaml",
                true,
                "worker",
                "worker-04",
                "/dev/vda",
            ),
            (
                "05-worker-no-hostname-no-install.yaml",
                true,
                "worker",
                "",
                "",
            ),
            (
                "06-worker-hostname-no-install.yaml",
                true,
                "worker",
                "worker-06",
                "",
            ),
            (
                "07-worker-install-no-hostname.yaml",
                true,
                "worker",
                "",
                "/dev/nvme0n1",
            ),
            (
                "08-controlplane-certsans.yaml",
                true,
                "controlplane",
                "cp-08",
                "/dev/sda",
            ),
            ("09-invalid-bad-type.yaml", false, "unknown", "", ""),
            ("10-invalid-missing-machine.yaml", false, "worker", "", ""),
            ("11-invalid-malformed.yaml", false, "", "", ""),
            ("12-invalid-no-endpoint.yaml", false, "controlplane", "", ""),
            ("13-invalid-worker-ca-key.yaml", false, "worker", "", ""),
            ("14-init-full.yaml", true, "init", "init-14", "/dev/sda"),
        ];

        let mut valid_count = 0;
        let mut invalid_count = 0;
        for (file, exp_valid, exp_type, exp_host, exp_disk) in expected {
            let (validity, mtype, host, disk) = load_record(&read(file));
            assert_eq!(
                validity.is_valid(),
                *exp_valid,
                "validity mismatch for {file}: {validity:?}"
            );
            assert_eq!(mtype, *exp_type, "machineType mismatch for {file}");
            assert_eq!(host, *exp_host, "hostname mismatch for {file}");
            assert_eq!(disk, *exp_disk, "installDisk mismatch for {file}");
            if *exp_valid {
                valid_count += 1;
            } else {
                invalid_count += 1;
            }
        }
        assert_eq!(valid_count, 9, "expected 9 valid corpus files");
        assert_eq!(invalid_count, 5, "expected 5 invalid corpus files");
        assert_eq!(expected.len(), 14, "expected 14 corpus files");
    }
}
