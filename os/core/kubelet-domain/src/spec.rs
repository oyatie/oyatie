//! Kubelet service spec rendering.
//!
//! Mirrors `internal/app/machined/pkg/controllers/k8s.KubeletSpecController`:
//! given a validated [`KubeletConfig`], the resolved node name, and the selected
//! node IP(s), it renders the ordered command-line the service manager runs plus
//! the file-backed config the kubelet reads. Protected flags Talos owns are
//! emitted here; user `extraArgs` are appended last, sorted for determinism.

use os_kernel::address::NodeAddress;
use os_kernel::error::{Error, Result};

use crate::config::{DEFAULT_RUNTIME_ENDPOINT, KUBELET_BINARY_PATH, KubeletConfig};
use crate::node_ip::NodeTaint;
use crate::nodename::Nodename;

/// Path of the kubelet's generated config file.
pub const KUBELET_CONFIG_PATH: &str = "/etc/kubernetes/kubelet.yaml";
/// Path of the kubelet TLS kubeconfig.
pub const KUBELET_KUBECONFIG_PATH: &str = "/etc/kubernetes/kubeconfig-kubelet";
/// Path of the bootstrap kubeconfig used before the node CSR is approved.
pub const KUBELET_BOOTSTRAP_KUBECONFIG_PATH: &str = "/etc/kubernetes/bootstrap-kubeconfig";
/// PKI directory where the kubelet stores its client/serving certs.
pub const KUBELET_CERT_DIR: &str = "/var/lib/kubelet/pki";
/// Static-pod manifest directory.
pub const STATIC_POD_PATH: &str = "/etc/kubernetes/manifests";

/// The fully rendered kubelet service spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubeletSpec {
    /// Path to the kubelet binary.
    pub command: String,
    /// The ordered argument list.
    pub args: Vec<String>,
    /// The container-runtime endpoint.
    pub runtime_endpoint: String,
    /// The node name the kubelet registers under.
    pub node_name: String,
    /// The advertised node IP(s).
    pub node_ips: Vec<String>,
    /// Taints applied at registration.
    pub register_taints: Vec<NodeTaint>,
}

impl KubeletSpec {
    /// Render the kubelet process spec.
    ///
    /// `taints` are applied via `--register-with-taints`; on a control-plane
    /// node the caller passes [`NodeTaint::control_plane`].
    pub fn render(
        config: &KubeletConfig,
        node_name: &Nodename,
        node_ips: &[NodeAddress],
        taints: &[NodeTaint],
    ) -> Result<Self> {
        config.validate()?;
        if node_ips.is_empty() {
            return Err(Error::invalid("kubelet spec needs at least one node IP"));
        }
        let node_ip_str = node_ips
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");

        let mut args = vec![
            format!("--hostname-override={node_name}"),
            format!("--node-ip={node_ip_str}"),
            format!("--kubeconfig={KUBELET_KUBECONFIG_PATH}"),
            format!("--bootstrap-kubeconfig={KUBELET_BOOTSTRAP_KUBECONFIG_PATH}"),
            format!("--config={KUBELET_CONFIG_PATH}"),
            format!("--cert-dir={KUBELET_CERT_DIR}"),
            format!("--container-runtime-endpoint={DEFAULT_RUNTIME_ENDPOINT}"),
            format!("--cgroup-driver={}", config.cgroup_driver),
            format!("--pod-manifest-path={STATIC_POD_PATH}"),
        ];

        if !config.register_node {
            args.push("--register-node=false".to_string());
        }
        if !taints.is_empty() {
            let rendered: Vec<String> = taints.iter().map(NodeTaint::render).collect();
            args.push(format!("--register-with-taints={}", rendered.join(",")));
        }

        // Extra args appended last; BTreeMap iteration is already sorted.
        for (k, v) in &config.extra_args {
            args.push(format!("--{k}={v}"));
        }

        Ok(KubeletSpec {
            command: KUBELET_BINARY_PATH.to_string(),
            args,
            runtime_endpoint: DEFAULT_RUNTIME_ENDPOINT.to_string(),
            node_name: node_name.as_str().to_string(),
            node_ips: node_ips.iter().map(ToString::to_string).collect(),
            register_taints: taints.to_vec(),
        })
    }

    /// True if the rendered args contain the given flag (`--flag=`).
    pub fn has_flag(&self, flag: &str) -> bool {
        let needle = format!("--{}=", flag.trim_start_matches('-'));
        self.args.iter().any(|a| a.starts_with(&needle))
    }

    /// The value of a rendered flag, if present.
    pub fn flag_value(&self, flag: &str) -> Option<&str> {
        let needle = format!("--{}=", flag.trim_start_matches('-'));
        self.args
            .iter()
            .find(|a| a.starts_with(&needle))
            .map(|a| &a[needle.len()..])
    }

    /// Render the kubelet `config.yaml` body as a minimal, deterministic set of
    /// `key: value` lines. This is not a full YAML serializer; it captures the
    /// fields Talos writes that aren't passed as flags.
    pub fn render_config_yaml(&self, config: &KubeletConfig) -> String {
        let mut out = String::new();
        out.push_str("apiVersion: kubelet.config.k8s.io/v1beta1\n");
        out.push_str("kind: KubeletConfiguration\n");
        out.push_str(&format!("clusterDomain: {}\n", config.cluster_domain));
        out.push_str("clusterDNS:\n");
        for dns in &config.cluster_dns {
            out.push_str(&format!("  - {dns}\n"));
        }
        out.push_str(&format!("cgroupDriver: {}\n", config.cgroup_driver));
        out.push_str(&format!(
            "seccompDefault: {}\n",
            config.default_runtime_seccomp_enabled
        ));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::KubeletConfig;

    fn base_config() -> KubeletConfig {
        KubeletConfig::with_dns_from_service_cidr("10.96.0.0/12").unwrap()
    }

    fn node() -> Nodename {
        Nodename::new("worker-1").unwrap()
    }

    #[test]
    fn renders_core_flags() {
        let spec = KubeletSpec::render(
            &base_config(),
            &node(),
            &[NodeAddress::parse_v4("10.0.0.5").unwrap()],
            &[],
        )
        .unwrap();
        assert_eq!(spec.command, KUBELET_BINARY_PATH);
        assert!(spec.has_flag("hostname-override"));
        assert!(spec.has_flag("node-ip"));
        assert!(spec.has_flag("bootstrap-kubeconfig"));
        assert!(spec.has_flag("cgroup-driver"));
        assert_eq!(spec.flag_value("node-ip"), Some("10.0.0.5"));
        assert_eq!(spec.flag_value("cgroup-driver"), Some("systemd"));
    }

    #[test]
    fn dual_stack_node_ip_joined() {
        let spec = KubeletSpec::render(
            &base_config(),
            &node(),
            &[
                NodeAddress::parse_v4("10.0.0.5").unwrap(),
                NodeAddress::V6([0xfd00, 0, 0, 0, 0, 0, 0, 1]),
            ],
            &[],
        )
        .unwrap();
        assert_eq!(
            spec.flag_value("node-ip"),
            Some("10.0.0.5,fd00:0:0:0:0:0:0:1")
        );
    }

    #[test]
    fn taints_rendered_as_register_with_taints() {
        let spec = KubeletSpec::render(
            &base_config(),
            &node(),
            &[NodeAddress::parse_v4("10.0.0.5").unwrap()],
            &[NodeTaint::control_plane()],
        )
        .unwrap();
        assert_eq!(
            spec.flag_value("register-with-taints"),
            Some("node-role.kubernetes.io/control-plane:NoSchedule")
        );
    }

    #[test]
    fn extra_args_appended_sorted_and_last() {
        let cfg = base_config()
            .with_extra_arg("max-pods", "200")
            .unwrap()
            .with_extra_arg("eviction-hard", "memory.available<100Mi")
            .unwrap();
        let spec = KubeletSpec::render(
            &cfg,
            &node(),
            &[NodeAddress::parse_v4("10.0.0.5").unwrap()],
            &[],
        )
        .unwrap();
        let evic = spec
            .args
            .iter()
            .position(|a| a.starts_with("--eviction-hard="))
            .unwrap();
        let maxp = spec
            .args
            .iter()
            .position(|a| a.starts_with("--max-pods="))
            .unwrap();
        let cgroup = spec
            .args
            .iter()
            .position(|a| a.starts_with("--cgroup-driver="))
            .unwrap();
        assert!(evic < maxp, "extra args sorted");
        assert!(cgroup < evic, "extra args appended after owned flags");
    }

    #[test]
    fn empty_node_ips_rejected() {
        assert!(KubeletSpec::render(&base_config(), &node(), &[], &[]).is_err());
    }

    #[test]
    fn config_yaml_contains_dns_and_domain() {
        let cfg = base_config();
        let spec = KubeletSpec::render(
            &cfg,
            &node(),
            &[NodeAddress::parse_v4("10.0.0.5").unwrap()],
            &[],
        )
        .unwrap();
        let yaml = spec.render_config_yaml(&cfg);
        assert!(yaml.contains("clusterDomain: cluster.local"));
        assert!(yaml.contains("- 10.96.0.10"));
        assert!(yaml.contains("cgroupDriver: systemd"));
    }
}
