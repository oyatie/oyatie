//! End-to-end `talosctl cluster create` planning: turning a high-level cluster
//! spec into a config bundle plus a provisioner [`ClusterRequest`].
//!
//! Talos's `talosctl cluster create` glues the pieces together: it allocates
//! node IPs from the cluster network, generates a config bundle, attaches the
//! per-role config to each node request, and hands the request to a
//! provisioner. This module models that planning step as [`ClusterSpec`] ->
//! [`ClusterPlan`].

use crate::ClusterError;
use crate::r#gen::{ConfigBundle, GenInput};
use crate::provisioner::{ClusterRequest, NetworkRequest, NodeRequest, ProvisionerKind};
use os_kernel::machine_type::MachineType;
use os_kernel::version::Version;

/// The high-level description of a cluster to create.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterSpec {
    /// Cluster name.
    pub name: String,
    /// Number of control-plane nodes.
    pub control_planes: u32,
    /// Number of worker nodes.
    pub workers: u32,
    /// The cluster network CIDR (IPv4), e.g. `10.5.0.0/24`.
    pub cidr: String,
    /// Provisioner backend.
    pub provisioner: ProvisionerKind,
    /// Boot/installer image.
    pub image: String,
    /// Kubernetes version.
    pub kubernetes_version: Version,
    /// Talos version.
    pub talos_version: Version,
    /// Per-node memory in MiB.
    pub memory_mib: u32,
    /// Per-node vCPUs.
    pub vcpus: u32,
    /// Per-node disk in MiB (qemu only).
    pub disk_mib: u32,
}

impl ClusterSpec {
    /// Construct a spec with sensible defaults for resources and versions.
    pub fn new(
        name: impl Into<String>,
        control_planes: u32,
        workers: u32,
        provisioner: ProvisionerKind,
    ) -> Self {
        ClusterSpec {
            name: name.into(),
            control_planes,
            workers,
            cidr: "10.5.0.0/24".to_string(),
            provisioner,
            image: "ghcr.io/siderolabs/talos:v1.7.0".to_string(),
            kubernetes_version: Version::new(1, 30, 0),
            talos_version: Version::new(1, 7, 0),
            memory_mib: 2048,
            vcpus: 2,
            disk_mib: if provisioner == ProvisionerKind::Qemu {
                6144
            } else {
                0
            },
        }
    }

    /// Validate the spec.
    pub fn validate(&self) -> Result<(), ClusterError> {
        if self.name.trim().is_empty() {
            return Err(ClusterError::invalid("cluster name is empty"));
        }
        if self.control_planes == 0 {
            return Err(ClusterError::invalid(
                "cluster needs at least one control-plane node",
            ));
        }
        if self.control_planes.is_multiple_of(2) {
            // etcd quorum requires an odd number of control-plane nodes.
            return Err(ClusterError::invalid(
                "control-plane count should be odd for etcd quorum",
            ));
        }
        let total = self.control_planes + self.workers;
        if total > 250 {
            return Err(ClusterError::invalid(
                "cluster exceeds the 250-node host limit of /24",
            ));
        }
        Ok(())
    }
}

/// The fully-planned cluster: the generated config bundle and the provisioner
/// request with per-node configs attached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterPlan {
    /// The generated config bundle.
    pub bundle: ConfigBundle,
    /// The provisioner request to hand to a [`crate::provisioner::Provisioner`].
    pub request: ClusterRequest,
    /// The chosen control-plane endpoint URL.
    pub control_plane_endpoint: String,
}

impl ClusterPlan {
    /// Plan a cluster from a spec: allocate IPs, generate configs, build the
    /// provisioner request.
    #[cfg(any(test, feature = "modeled-crypto"))]
    pub fn plan(spec: &ClusterSpec) -> Result<Self, ClusterError> {
        spec.validate()?;

        let network = NetworkRequest::new(format!("{}-net", spec.name), spec.cidr.clone())?;
        let base = ipv4_network_base(&spec.cidr)?;

        // IP allocation: gateway is host .1 (set on the network); nodes start
        // at host .2. The first control-plane node's IP is the VIP/endpoint.
        let mut host = 2u8;
        let mut nodes = Vec::new();

        let first_cp_ip = format!("{}.{}.{}.{}", base[0], base[1], base[2], host);
        let control_plane_endpoint = format!("https://{first_cp_ip}:6443");

        let input = GenInput::new(
            spec.name.clone(),
            control_plane_endpoint.clone(),
            spec.kubernetes_version.clone(),
            spec.talos_version.clone(),
        );
        let bundle = ConfigBundle::generate(&input)?;

        for i in 0..spec.control_planes {
            let ip = format!("{}.{}.{}.{}", base[0], base[1], base[2], host);
            host = host
                .checked_add(1)
                .ok_or_else(|| ClusterError::invalid("ran out of host addresses in /24"))?;
            let machine_type = if i == 0 {
                MachineType::Init
            } else {
                MachineType::ControlPlane
            };
            let cfg = bundle.config_for(machine_type)?;
            nodes.push(NodeRequest {
                name: format!("{}-controlplane-{}", spec.name, i + 1),
                machine_type,
                ip,
                vcpus: spec.vcpus,
                memory_mib: spec.memory_mib,
                disk_mib: spec.disk_mib,
                config: render_config_marker(cfg),
            });
        }

        for i in 0..spec.workers {
            let ip = format!("{}.{}.{}.{}", base[0], base[1], base[2], host);
            host = host
                .checked_add(1)
                .ok_or_else(|| ClusterError::invalid("ran out of host addresses in /24"))?;
            let cfg = bundle.config_for(MachineType::Worker)?;
            nodes.push(NodeRequest {
                name: format!("{}-worker-{}", spec.name, i + 1),
                machine_type: MachineType::Worker,
                ip,
                vcpus: spec.vcpus,
                memory_mib: spec.memory_mib,
                disk_mib: spec.disk_mib,
                config: render_config_marker(cfg),
            });
        }

        let request = ClusterRequest {
            name: spec.name.clone(),
            network,
            nodes,
            image: spec.image.clone(),
        };
        request.validate(spec.provisioner)?;

        Ok(ClusterPlan {
            bundle,
            request,
            control_plane_endpoint,
        })
    }

    /// All node IPs in allocation order.
    pub fn node_ips(&self) -> Vec<String> {
        self.request.nodes.iter().map(|n| n.ip.clone()).collect()
    }

    /// The `(ip, machine_type)` pairs, useful for seeding a bootstrap
    /// orchestrator.
    pub fn node_types(&self) -> Vec<(String, MachineType)> {
        self.request
            .nodes
            .iter()
            .map(|n| (n.ip.clone(), n.machine_type))
            .collect()
    }
}

/// Render a stable, human-readable marker for a machine config (a real
/// implementation would serialize YAML here).
fn render_config_marker(cfg: &crate::r#gen::MachineConfig) -> String {
    format!(
        "type={} cluster={} k8s={} talos={} caCert={}",
        cfg.machine_type.as_str(),
        cfg.cluster_name,
        cfg.kubernetes_version,
        cfg.talos_version,
        cfg.os_ca_cert.as_str()
    )
}

/// Parse the network base octets of an IPv4 CIDR (ignoring the host bits).
fn ipv4_network_base(cidr: &str) -> Result<[u8; 4], ClusterError> {
    let addr = cidr.split('/').next().unwrap_or(cidr);
    let octets: Vec<&str> = addr.split('.').collect();
    if octets.len() != 4 {
        return Err(ClusterError::invalid(format!(
            "CIDR {cidr:?} is not a valid IPv4 address"
        )));
    }
    let mut parsed = [0u8; 4];
    for (i, o) in octets.iter().enumerate() {
        parsed[i] = o
            .parse()
            .map_err(|_| ClusterError::invalid(format!("CIDR {cidr:?} has a bad octet")))?;
    }
    Ok(parsed)
}
