//! Cluster provisioner abstraction (`pkg/provision`).
//!
//! Talos's `talosctl cluster create` drives a *provisioner* (docker or qemu)
//! that creates a virtual cluster: a network and a set of nodes. This module
//! models the provisioner as a [`Provisioner`] trait with an in-memory
//! [`InMemoryProvisioner`] implementation. The data model
//! ([`ClusterRequest`], [`NodeRequest`], [`NetworkRequest`]) mirrors
//! `provision.ClusterRequest`.

use crate::ClusterError;
use os_kernel::machine_type::MachineType;
use std::collections::BTreeMap;

/// Which backend provisions the cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvisionerKind {
    /// Containers as nodes (`talosctl cluster create --provisioner docker`).
    Docker,
    /// Virtual machines as nodes (`--provisioner qemu`).
    Qemu,
}

impl ProvisionerKind {
    /// Canonical CLI name.
    pub fn as_str(self) -> &'static str {
        match self {
            ProvisionerKind::Docker => "docker",
            ProvisionerKind::Qemu => "qemu",
        }
    }

    /// Parse a provisioner name.
    pub fn parse(s: &str) -> Result<Self, ClusterError> {
        match s {
            "docker" => Ok(ProvisionerKind::Docker),
            "qemu" => Ok(ProvisionerKind::Qemu),
            other => Err(ClusterError::invalid(format!(
                "unknown provisioner {other:?}"
            ))),
        }
    }

    /// Docker nodes share the host kernel and cannot model a real disk; qemu
    /// nodes boot a full VM. This gates which features a request may use.
    pub fn supports_disk_image(self) -> bool {
        matches!(self, ProvisionerKind::Qemu)
    }
}

/// The virtual network a cluster's nodes attach to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkRequest {
    /// Network name.
    pub name: String,
    /// CIDR the node IPs are allocated from, e.g. `10.5.0.0/24`.
    pub cidr: String,
    /// Maximum transmission unit.
    pub mtu: u32,
    /// Gateway address (first host of the CIDR by convention).
    pub gateway: String,
}

impl NetworkRequest {
    /// Construct a network request, validating the CIDR shape and MTU.
    pub fn new(name: impl Into<String>, cidr: impl Into<String>) -> Result<Self, ClusterError> {
        let cidr = cidr.into();
        let (gateway, _prefix) = parse_cidr_gateway(&cidr)?;
        Ok(NetworkRequest {
            name: name.into(),
            cidr,
            mtu: 1500,
            gateway,
        })
    }
}

/// A request to provision a single node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeRequest {
    /// Node name (also its hostname).
    pub name: String,
    /// The role this node will play.
    pub machine_type: MachineType,
    /// IP assigned to the node on the cluster network.
    pub ip: String,
    /// vCPUs to allocate (qemu) or the cgroup CPU limit (docker).
    pub vcpus: u32,
    /// Memory in MiB.
    pub memory_mib: u32,
    /// Disk size in MiB (qemu only).
    pub disk_mib: u32,
    /// The (already-rendered) machine config handed to the node.
    pub config: String,
}

impl NodeRequest {
    /// Validate a node request against a provisioner kind.
    pub fn validate(&self, kind: ProvisionerKind) -> Result<(), ClusterError> {
        if self.name.trim().is_empty() {
            return Err(ClusterError::invalid("node name is empty"));
        }
        if self.machine_type == MachineType::Unknown {
            return Err(ClusterError::invalid(format!(
                "node {} has an unknown machine type",
                self.name
            )));
        }
        if self.memory_mib < 256 {
            return Err(ClusterError::invalid(format!(
                "node {} requests less than 256 MiB of memory",
                self.name
            )));
        }
        if self.vcpus == 0 {
            return Err(ClusterError::invalid(format!(
                "node {} requests zero vCPUs",
                self.name
            )));
        }
        if self.disk_mib > 0 && !kind.supports_disk_image() {
            return Err(ClusterError::invalid(format!(
                "node {} requests a disk but the {} provisioner has no disk support",
                self.name,
                kind.as_str()
            )));
        }
        Ok(())
    }
}

/// The full request to provision a cluster (`provision.ClusterRequest`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterRequest {
    /// Cluster name.
    pub name: String,
    /// The network all nodes share.
    pub network: NetworkRequest,
    /// The nodes to create.
    pub nodes: Vec<NodeRequest>,
    /// The Talos installer/boot image reference.
    pub image: String,
}

impl ClusterRequest {
    /// Validate the whole request against the target provisioner.
    pub fn validate(&self, kind: ProvisionerKind) -> Result<(), ClusterError> {
        if self.name.trim().is_empty() {
            return Err(ClusterError::invalid("cluster name is empty"));
        }
        if self.nodes.is_empty() {
            return Err(ClusterError::invalid("cluster has no nodes"));
        }
        let cp = self
            .nodes
            .iter()
            .filter(|n| n.machine_type.is_control_plane())
            .count();
        if cp == 0 {
            return Err(ClusterError::invalid("cluster has no control-plane nodes"));
        }
        let mut seen_names = std::collections::BTreeSet::new();
        let mut seen_ips = std::collections::BTreeSet::new();
        for node in &self.nodes {
            node.validate(kind)?;
            if !seen_names.insert(node.name.clone()) {
                return Err(ClusterError::invalid(format!(
                    "duplicate node name {:?}",
                    node.name
                )));
            }
            if !seen_ips.insert(node.ip.clone()) {
                return Err(ClusterError::invalid(format!(
                    "duplicate node IP {:?}",
                    node.ip
                )));
            }
        }
        Ok(())
    }

    /// Count of control-plane nodes in the request.
    pub fn control_plane_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| n.machine_type.is_control_plane())
            .count()
    }

    /// Count of worker nodes in the request.
    pub fn worker_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| n.machine_type == MachineType::Worker)
            .count()
    }
}

/// Lifecycle state of a provisioned node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    /// The node has been created but not started.
    Created,
    /// The node is running.
    Running,
    /// The node has been stopped/destroyed.
    Destroyed,
}

/// A handle to a provisioned node returned by a [`Provisioner`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisionedNode {
    /// Node name.
    pub name: String,
    /// Assigned IP.
    pub ip: String,
    /// Role.
    pub machine_type: MachineType,
    /// Current lifecycle state.
    pub state: NodeState,
}

/// A handle to a provisioned cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisionedCluster {
    /// Cluster name.
    pub name: String,
    /// Provisioner backend used.
    pub kind: ProvisionerKind,
    /// The provisioned nodes.
    pub nodes: Vec<ProvisionedNode>,
}

impl ProvisionedCluster {
    /// All control-plane node IPs.
    pub fn control_plane_ips(&self) -> Vec<String> {
        self.nodes
            .iter()
            .filter(|n| n.machine_type.is_control_plane())
            .map(|n| n.ip.clone())
            .collect()
    }
}

/// The provisioner backend abstraction.
///
/// Mirrors `provision.Provisioner`: create a cluster from a request, reflect on
/// it, and destroy it. OS boundaries (docker/qemu/libvirt) live behind this
/// trait; tests use [`InMemoryProvisioner`].
pub trait Provisioner {
    /// The backend kind.
    fn kind(&self) -> ProvisionerKind;

    /// Provision a cluster from a request.
    fn create(&mut self, request: &ClusterRequest) -> Result<ProvisionedCluster, ClusterError>;

    /// Reflect the current state of a previously-created cluster.
    fn reflect(&self, name: &str) -> Result<ProvisionedCluster, ClusterError>;

    /// Destroy a cluster by name.
    fn destroy(&mut self, name: &str) -> Result<(), ClusterError>;
}

/// In-memory [`Provisioner`] used for tests and simulation.
#[derive(Debug)]
pub struct InMemoryProvisioner {
    kind: ProvisionerKind,
    clusters: BTreeMap<String, ProvisionedCluster>,
}

impl InMemoryProvisioner {
    /// Create an in-memory provisioner of the given kind.
    pub fn new(kind: ProvisionerKind) -> Self {
        InMemoryProvisioner {
            kind,
            clusters: BTreeMap::new(),
        }
    }

    /// Number of live clusters tracked.
    pub fn cluster_count(&self) -> usize {
        self.clusters.len()
    }
}

impl Provisioner for InMemoryProvisioner {
    fn kind(&self) -> ProvisionerKind {
        self.kind
    }

    fn create(&mut self, request: &ClusterRequest) -> Result<ProvisionedCluster, ClusterError> {
        request.validate(self.kind)?;
        if self.clusters.contains_key(&request.name) {
            return Err(ClusterError::invalid_state(format!(
                "cluster {:?} already exists",
                request.name
            )));
        }
        let cluster = ProvisionedCluster {
            name: request.name.clone(),
            kind: self.kind,
            nodes: request
                .nodes
                .iter()
                .map(|n| ProvisionedNode {
                    name: n.name.clone(),
                    ip: n.ip.clone(),
                    machine_type: n.machine_type,
                    state: NodeState::Running,
                })
                .collect(),
        };
        self.clusters.insert(request.name.clone(), cluster.clone());
        Ok(cluster)
    }

    fn reflect(&self, name: &str) -> Result<ProvisionedCluster, ClusterError> {
        self.clusters
            .get(name)
            .cloned()
            .ok_or_else(|| ClusterError::not_found(format!("cluster {name:?} not found")))
    }

    fn destroy(&mut self, name: &str) -> Result<(), ClusterError> {
        if self.clusters.remove(name).is_none() {
            return Err(ClusterError::not_found(format!(
                "cluster {name:?} not found"
            )));
        }
        Ok(())
    }
}

/// Parse a `a.b.c.d/prefix` CIDR, returning the conventional gateway (first
/// host: network address + 1) and the prefix length. IPv4 only.
fn parse_cidr_gateway(cidr: &str) -> Result<(String, u8), ClusterError> {
    let (addr, prefix) = cidr
        .split_once('/')
        .ok_or_else(|| ClusterError::invalid(format!("CIDR {cidr:?} missing '/'")))?;
    let prefix: u8 = prefix
        .parse()
        .map_err(|_| ClusterError::invalid(format!("CIDR {cidr:?} has a non-numeric prefix")))?;
    if prefix > 32 {
        return Err(ClusterError::invalid(format!(
            "CIDR {cidr:?} prefix exceeds 32"
        )));
    }
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
    // Gateway = network + 1 in the last octet (good enough for /24-style nets).
    let gw_last = parsed[3].wrapping_add(1);
    let gateway = format!("{}.{}.{}.{}", parsed[0], parsed[1], parsed[2], gw_last);
    Ok((gateway, prefix))
}
