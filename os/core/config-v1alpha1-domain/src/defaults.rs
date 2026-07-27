//! Default values applied to a v1alpha1 config, mirroring the constants Talos
//! keeps in `pkg/machinery/constants` and the defaulting it performs while
//! decoding / generating a config.

/// Default Kubernetes API server bind port.
pub const DEFAULT_APISERVER_PORT: u16 = 6443;

/// Default cluster DNS domain.
pub const DEFAULT_DNS_DOMAIN: &str = "cluster.local";

/// Default pod subnet (Talos uses a /16 by default for the CNI).
pub const DEFAULT_POD_SUBNET: &str = "10.244.0.0/16";

/// Default service subnet.
pub const DEFAULT_SERVICE_SUBNET: &str = "10.96.0.0/12";

/// Default install disk on bare metal when nothing is specified.
pub const DEFAULT_INSTALL_DISK: &str = "/dev/sda";

/// Default kubelet image registry path (tag is appended at runtime).
pub const DEFAULT_KUBELET_IMAGE: &str = "ghcr.io/siderolabs/kubelet";

/// Default Talos installer image.
pub const DEFAULT_INSTALL_IMAGE: &str = "ghcr.io/siderolabs/installer";

/// Default etcd advertised client port.
pub const DEFAULT_ETCD_CLIENT_PORT: u16 = 2379;

/// Default etcd peer port.
pub const DEFAULT_ETCD_PEER_PORT: u16 = 2380;

/// Default cluster network MTU.
pub const DEFAULT_MTU: u32 = 1500;

/// Default node port range used by kube-proxy (`--service-node-port-range`).
pub const DEFAULT_NODE_PORT_RANGE: (u16, u16) = (30000, 32767);

/// Default kube-proxy mode.
pub const DEFAULT_PROXY_MODE: &str = "iptables";

/// Default kubelet `clusterDNS` derived from the service subnet's tenth address
/// (Talos picks `<service-subnet-network>.10`). For the default `10.96.0.0/12`
/// this is `10.96.0.10`.
pub const DEFAULT_CLUSTER_DNS: &str = "10.96.0.10";

/// Default control-plane scheme.
pub const DEFAULT_ENDPOINT_SCHEME: &str = "https";

/// Default discovery service endpoint.
pub const DEFAULT_DISCOVERY_ENDPOINT: &str = "https://discovery.talos.dev/";

/// Compute the conventional cluster DNS address (the `.10` host of the first
/// service subnet). Returns `None` if the subnet isn't a parseable IPv4 CIDR.
pub fn cluster_dns_for_service_subnet(cidr: &str) -> Option<String> {
    let (addr, _prefix) = cidr.split_once('/')?;
    let mut octets: Vec<&str> = addr.split('.').collect();
    if octets.len() != 4 {
        return None;
    }
    // Replace the final octet with `10`.
    octets[3] = "10";
    Some(octets.join("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dns_address_from_service_subnet() {
        assert_eq!(
            cluster_dns_for_service_subnet("10.96.0.0/12").as_deref(),
            Some("10.96.0.10")
        );
        assert_eq!(
            cluster_dns_for_service_subnet("172.20.0.0/16").as_deref(),
            Some("172.20.0.10")
        );
        assert_eq!(cluster_dns_for_service_subnet("garbage"), None);
    }

    #[test]
    fn constants_are_sane() {
        assert_eq!(DEFAULT_APISERVER_PORT, 6443);
        assert!(DEFAULT_NODE_PORT_RANGE.0 < DEFAULT_NODE_PORT_RANGE.1);
        assert!(DEFAULT_POD_SUBNET.contains('/'));
    }
}
