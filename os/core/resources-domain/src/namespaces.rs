//! The well-known COSI namespaces used by Talos.
//!
//! Mirrors the `Namespace*` constants scattered across Talos's
//! `pkg/machinery/resources/*` packages (config, network, k8s, cluster,
//! runtime, secrets, hardware, perf, etcd, files, ...). COSI partitions
//! resources by namespace; controllers and `talosctl get -n <ns>` address them
//! by these stable string names.

/// A well-known namespace and a human description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Namespace {
    name: &'static str,
    description: &'static str,
}

impl Namespace {
    /// The namespace string used in resource metadata.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// A short human-readable description.
    pub fn description(&self) -> &'static str {
        self.description
    }
}

impl std::fmt::Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

macro_rules! namespaces {
    ($($const_name:ident = $value:literal => $desc:literal;)*) => {
        $(
            #[doc = $desc]
            pub const $const_name: Namespace = Namespace { name: $value, description: $desc };
        )*

        /// Every well-known namespace, in a stable order.
        pub const ALL: &[Namespace] = &[$($const_name),*];
    };
}

namespaces! {
    CONFIG    = "config"     => "User and machine configuration resources.";
    RUNTIME   = "runtime"    => "Runtime/system state: meta keys, mounts, services, events.";
    NETWORK   = "network"    => "Network configuration and live status resources.";
    NETWORK_CONFIG = "network-config" => "Intermediate network config-layer resources before final merge.";
    K8S       = "k8s"        => "Kubernetes control-plane and kubelet resources.";
    CLUSTER   = "cluster"    => "Cluster membership and discovery resources.";
    SECRETS   = "secrets"    => "Generated secrets (etcd, kubernetes, trustd, OS).";
    HARDWARE  = "hardware"   => "Discovered hardware: CPU, memory, PCI, system info.";
    PERF      = "perf"       => "Performance/metrics resources (CPU and memory stats).";
    ETCD      = "etcd"       => "etcd member and configuration resources.";
    FILES     = "files"      => "Managed on-disk files (CRI config, resolv.conf, etc.).";
    META      = "meta"       => "COSI meta namespace: resource and namespace definitions.";
    SIDEROLINK = "siderolink" => "SideroLink WireGuard tunnel resources.";
}

/// Look up a namespace by name.
pub fn find(name: &str) -> Option<Namespace> {
    ALL.iter().copied().find(|ns| ns.name == name)
}

/// Whether `name` is a registered namespace.
pub fn is_known(name: &str) -> bool {
    find(name).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_namespaces_resolve() {
        assert_eq!(find("config"), Some(CONFIG));
        assert_eq!(find("k8s").unwrap().name(), "k8s");
        assert!(is_known("network"));
        assert!(is_known("network-config"));
        assert!(is_known("siderolink"));
        assert!(!is_known("does-not-exist"));
    }

    #[test]
    fn all_namespaces_unique_and_nonempty() {
        let mut seen = std::collections::BTreeSet::new();
        for ns in ALL {
            assert!(!ns.name().is_empty());
            assert!(!ns.description().is_empty());
            assert!(seen.insert(ns.name()), "duplicate namespace {}", ns.name());
        }
        assert!(ALL.len() >= 10);
    }

    #[test]
    fn display_is_name() {
        assert_eq!(CONFIG.to_string(), "config");
        assert_eq!(META.to_string(), "meta");
    }
}
