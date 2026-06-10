//! Kubelet configuration and the kubelet service spec.
//!
//! Mirrors Talos `internal/app/machined/pkg/controllers/k8s` kubelet controllers
//! (`KubeletConfigController` / `KubeletSpecController`): the inputs the machine
//! config fans out into the kubelet's `config.yaml` and the process command
//! line. We model the validated configuration and the rendered argument set
//! rather than the YAML serialization itself.

use crate::config::K8sConfig;
use crate::error::{K8sError, Result};

/// The well-known kubelet config defaults Talos applies.
pub const DEFAULT_CLUSTER_DNS_DOMAIN: &str = "cluster.local";
/// Default container runtime endpoint Talos points the kubelet at.
pub const DEFAULT_CONTAINER_RUNTIME_ENDPOINT: &str = "unix:///run/containerd/containerd.sock";

/// Validated kubelet configuration derived from the machine config.
///
/// This is the data behind the kubelet's `config.yaml`: the cluster DNS,
/// the registration node name, extra args, and feature toggles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubeletConfig {
    /// The node's registration name.
    pub node_name: String,
    /// The cluster DNS service IP(s) the kubelet hands to pods.
    pub cluster_dns: Vec<String>,
    /// The cluster DNS domain (`cluster.local`).
    pub cluster_domain: String,
    /// Whether the kubelet should register the node with the apiserver.
    pub register_node: bool,
    /// Extra `--key=value` style args appended verbatim, after validation.
    pub extra_args: Vec<(String, String)>,
    /// Node taints applied at registration (e.g. control-plane).
    pub taints: Vec<String>,
    /// Whether the static-pod path is enabled.
    pub static_pod_path: Option<String>,
}

impl KubeletConfig {
    /// Derive a kubelet config from the node-level [`K8sConfig`].
    ///
    /// On a control-plane node, Talos taints the node so workloads don't
    /// schedule there unless explicitly tolerated.
    pub fn from_node_config(cfg: &K8sConfig) -> Result<Self> {
        cfg.validate()?;
        let dns_ip = cfg.dns_service_ip()?;
        let mut taints = Vec::new();
        if cfg.control_plane {
            taints.push("node-role.kubernetes.io/control-plane:NoSchedule".to_string());
        }
        Ok(KubeletConfig {
            node_name: cfg.node_name.as_str().to_string(),
            cluster_dns: vec![dns_ip],
            cluster_domain: cfg.cluster_domain.clone(),
            register_node: true,
            extra_args: Vec::new(),
            taints,
            static_pod_path: Some(crate::STATIC_POD_PATH.to_string()),
        })
    }

    /// Add an extra kubelet argument, rejecting protected flags the controller
    /// owns (Talos refuses to let users override these).
    pub fn with_extra_arg(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self> {
        let key = key.into();
        let bare = key.trim_start_matches('-');
        if PROTECTED_ARGS.contains(&bare) {
            return Err(K8sError::InvalidConfig(format!(
                "kubelet arg '{bare}' is managed by Talos and cannot be overridden"
            )));
        }
        self.extra_args.push((bare.to_string(), value.into()));
        Ok(self)
    }

    /// Validate the config is internally consistent.
    pub fn validate(&self) -> Result<()> {
        if self.node_name.is_empty() {
            return Err(K8sError::InvalidConfig(
                "kubelet node name empty".to_string(),
            ));
        }
        if self.cluster_dns.is_empty() {
            return Err(K8sError::InvalidConfig(
                "kubelet has no cluster DNS".to_string(),
            ));
        }
        Ok(())
    }
}

/// Kubelet command-line arguments the controller owns and won't let users set.
pub const PROTECTED_ARGS: &[&str] = &[
    "hostname-override",
    "kubeconfig",
    "bootstrap-kubeconfig",
    "config",
    "cert-dir",
    "container-runtime-endpoint",
];

/// The rendered kubelet service spec: the process the service manager runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubeletSpec {
    /// Path to the kubelet binary.
    pub command: String,
    /// The fully rendered, ordered argument list.
    pub args: Vec<String>,
    /// The container runtime endpoint.
    pub runtime_endpoint: String,
}

impl KubeletSpec {
    /// Render the kubelet process spec from a validated [`KubeletConfig`].
    pub fn render(config: &KubeletConfig) -> Result<Self> {
        config.validate()?;
        let mut args = vec![
            format!("--hostname-override={}", config.node_name),
            "--kubeconfig=/etc/kubernetes/kubeconfig-kubelet".to_string(),
            "--config=/etc/kubernetes/kubelet.yaml".to_string(),
            "--cert-dir=/var/lib/kubelet/pki".to_string(),
            format!("--container-runtime-endpoint={DEFAULT_CONTAINER_RUNTIME_ENDPOINT}"),
        ];
        if let Some(path) = &config.static_pod_path {
            args.push(format!("--pod-manifest-path={path}"));
        }
        if !config.register_node {
            args.push("--register-node=false".to_string());
        }
        // Extra args are appended last, sorted for determinism.
        let mut extra: Vec<(String, String)> = config.extra_args.clone();
        extra.sort();
        for (k, v) in extra {
            args.push(format!("--{k}={v}"));
        }
        Ok(KubeletSpec {
            command: "/usr/local/bin/kubelet".to_string(),
            args,
            runtime_endpoint: DEFAULT_CONTAINER_RUNTIME_ENDPOINT.to_string(),
        })
    }

    /// True if the rendered args contain the given flag (`--flag`).
    pub fn has_flag(&self, flag: &str) -> bool {
        let needle = format!("--{}=", flag.trim_start_matches('-'));
        self.args.iter().any(|a| a.starts_with(&needle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ClusterEndpoint, NodeName};

    fn node_cfg(control_plane: bool) -> K8sConfig {
        K8sConfig {
            node_name: NodeName::new("worker-1").unwrap(),
            cluster_domain: "cluster.local".into(),
            pod_cidrs: vec!["10.244.0.0/16".into()],
            service_cidrs: vec!["10.96.0.0/12".into()],
            endpoint: ClusterEndpoint::new("api.example.com", 6443).unwrap(),
            version: "1.30.0".into(),
            control_plane,
        }
    }

    #[test]
    fn control_plane_node_gets_taint_and_dns() {
        let kc = KubeletConfig::from_node_config(&node_cfg(true)).unwrap();
        assert_eq!(kc.cluster_dns, vec!["10.96.0.10".to_string()]);
        assert!(kc.taints.iter().any(|t| t.contains("control-plane")));
    }

    #[test]
    fn worker_node_has_no_taint() {
        let kc = KubeletConfig::from_node_config(&node_cfg(false)).unwrap();
        assert!(kc.taints.is_empty());
    }

    #[test]
    fn protected_args_are_rejected() {
        let kc = KubeletConfig::from_node_config(&node_cfg(false)).unwrap();
        let err = kc
            .with_extra_arg("--hostname-override", "evil")
            .unwrap_err();
        assert_eq!(err.kind(), "invalid_config");
    }

    #[test]
    fn extra_args_render_sorted() {
        let kc = KubeletConfig::from_node_config(&node_cfg(false))
            .unwrap()
            .with_extra_arg("max-pods", "200")
            .unwrap()
            .with_extra_arg("cluster-domain", "cluster.local")
            .unwrap();
        let spec = KubeletSpec::render(&kc).unwrap();
        let max = spec
            .args
            .iter()
            .position(|a| a.contains("max-pods"))
            .unwrap();
        let dom = spec
            .args
            .iter()
            .position(|a| a.starts_with("--cluster-domain="))
            .unwrap();
        assert!(dom < max, "extra args should be sorted");
    }

    #[test]
    fn spec_renders_core_flags() {
        let kc = KubeletConfig::from_node_config(&node_cfg(true)).unwrap();
        let spec = KubeletSpec::render(&kc).unwrap();
        assert!(spec.has_flag("hostname-override"));
        assert!(spec.has_flag("pod-manifest-path"));
        assert_eq!(spec.command, "/usr/local/bin/kubelet");
    }

    #[test]
    fn register_node_false_emits_flag() {
        let mut kc = KubeletConfig::from_node_config(&node_cfg(false)).unwrap();
        kc.register_node = false;
        let spec = KubeletSpec::render(&kc).unwrap();
        assert!(spec.args.iter().any(|a| a == "--register-node=false"));
    }
}
