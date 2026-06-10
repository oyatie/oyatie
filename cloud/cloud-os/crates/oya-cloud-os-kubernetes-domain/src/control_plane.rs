//! Desired configuration of the control plane and per-component rendering.
//!
//! Mirrors Talos `k8s.ControlPlane*` controllers: from the cluster config and
//! secrets, machined renders one static pod per control-plane component
//! (apiserver, controller-manager, scheduler). We model the component identity,
//! its argument rendering, and gating on the secret bundle.

use crate::config::K8sConfig;
use crate::error::{K8sError, Result};
use crate::pki::{SubjectAltName, apiserver_sans};
use crate::secrets::K8sSecrets;
use crate::static_pod::StaticPod;

/// A control-plane component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum K8sComponent {
    /// `kube-apiserver`.
    ApiServer,
    /// `kube-controller-manager`.
    ControllerManager,
    /// `kube-scheduler`.
    Scheduler,
}

impl K8sComponent {
    /// All control-plane components, in start order.
    pub const ALL: [K8sComponent; 3] = [
        K8sComponent::ApiServer,
        K8sComponent::ControllerManager,
        K8sComponent::Scheduler,
    ];

    /// The component's pod / binary name.
    pub fn name(self) -> &'static str {
        match self {
            K8sComponent::ApiServer => "kube-apiserver",
            K8sComponent::ControllerManager => "kube-controller-manager",
            K8sComponent::Scheduler => "kube-scheduler",
        }
    }

    /// Parse a component from its canonical name.
    pub fn parse(name: &str) -> Result<Self> {
        match name {
            "kube-apiserver" => Ok(K8sComponent::ApiServer),
            "kube-controller-manager" => Ok(K8sComponent::ControllerManager),
            "kube-scheduler" => Ok(K8sComponent::Scheduler),
            other => Err(K8sError::UnknownComponent(other.to_string())),
        }
    }

    /// The registry image reference for this component at `version`.
    pub fn image(self, version: &str) -> String {
        format!("registry.k8s.io/{}:v{}", self.name(), version)
    }
}

/// The desired control-plane configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlPlaneConfig {
    /// The node-level config (CIDRs, endpoint, version).
    pub node: K8sConfig,
    /// Extra args per component, applied after the controller-owned defaults.
    pub extra_args: Vec<(K8sComponent, String, String)>,
}

impl ControlPlaneConfig {
    /// Build from a node config, requiring it to be a control-plane node.
    pub fn new(node: K8sConfig) -> Result<Self> {
        node.validate()?;
        if !node.control_plane {
            return Err(K8sError::InvalidConfig(
                "control-plane config built for a worker node".to_string(),
            ));
        }
        Ok(ControlPlaneConfig {
            node,
            extra_args: Vec::new(),
        })
    }

    /// Register an extra argument for one component.
    pub fn with_extra_arg(
        mut self,
        component: K8sComponent,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.extra_args.push((
            component,
            key.into().trim_start_matches('-').to_string(),
            value.into(),
        ));
        self
    }

    /// Render the controller-owned default arguments for a component.
    pub fn args_for(&self, component: K8sComponent) -> Result<Vec<String>> {
        let n = &self.node;
        let mut args: Vec<String> = match component {
            K8sComponent::ApiServer => vec![
                "--allow-privileged=true".to_string(),
                "--authorization-mode=Node,RBAC".to_string(),
                format!("--service-cluster-ip-range={}", n.service_cidrs.join(",")),
                "--secure-port=6443".to_string(),
                "--tls-cert-file=/etc/kubernetes/pki/apiserver.crt".to_string(),
                "--tls-private-key-file=/etc/kubernetes/pki/apiserver.key".to_string(),
                "--client-ca-file=/etc/kubernetes/pki/ca.crt".to_string(),
                "--service-account-key-file=/etc/kubernetes/pki/sa.pub".to_string(),
                "--service-account-signing-key-file=/etc/kubernetes/pki/sa.key".to_string(),
                format!("--service-account-issuer={}", n.endpoint.url()),
                "--etcd-servers=https://127.0.0.1:2379".to_string(),
                "--etcd-cafile=/etc/kubernetes/pki/etcd/ca.crt".to_string(),
                "--etcd-certfile=/etc/kubernetes/pki/apiserver-etcd-client.crt".to_string(),
                "--etcd-keyfile=/etc/kubernetes/pki/apiserver-etcd-client.key".to_string(),
                "--requestheader-client-ca-file=/etc/kubernetes/pki/front-proxy-ca.crt".to_string(),
                "--proxy-client-cert-file=/etc/kubernetes/pki/front-proxy-client.crt".to_string(),
                "--proxy-client-key-file=/etc/kubernetes/pki/front-proxy-client.key".to_string(),
                "--requestheader-allowed-names=front-proxy-client".to_string(),
                "--requestheader-extra-headers-prefix=X-Remote-Extra-".to_string(),
                "--requestheader-group-headers=X-Remote-Group".to_string(),
                "--requestheader-username-headers=X-Remote-User".to_string(),
                "--enable-admission-plugins=NodeRestriction".to_string(),
                "--kubelet-client-certificate=/etc/kubernetes/pki/apiserver-kubelet-client.crt"
                    .to_string(),
                "--kubelet-client-key=/etc/kubernetes/pki/apiserver-kubelet-client.key".to_string(),
            ],
            K8sComponent::ControllerManager => vec![
                "--bind-address=127.0.0.1".to_string(),
                "--leader-elect=true".to_string(),
                format!("--cluster-cidr={}", n.pod_cidrs.join(",")),
                format!("--service-cluster-ip-range={}", n.service_cidrs.join(",")),
                "--cluster-name=kubernetes".to_string(),
                "--kubeconfig=/etc/kubernetes/controller-manager.conf".to_string(),
                "--authentication-kubeconfig=/etc/kubernetes/controller-manager.conf".to_string(),
                "--authorization-kubeconfig=/etc/kubernetes/controller-manager.conf".to_string(),
                "--root-ca-file=/etc/kubernetes/pki/ca.crt".to_string(),
                "--cluster-signing-cert-file=/etc/kubernetes/pki/ca.crt".to_string(),
                "--cluster-signing-key-file=/etc/kubernetes/pki/ca.key".to_string(),
                "--service-account-private-key-file=/etc/kubernetes/pki/sa.key".to_string(),
                "--use-service-account-credentials=true".to_string(),
                "--controllers=*,bootstrapsigner,tokencleaner".to_string(),
            ],
            K8sComponent::Scheduler => vec![
                "--bind-address=127.0.0.1".to_string(),
                "--leader-elect=true".to_string(),
                "--kubeconfig=/etc/kubernetes/scheduler.conf".to_string(),
                "--authentication-kubeconfig=/etc/kubernetes/scheduler.conf".to_string(),
                "--authorization-kubeconfig=/etc/kubernetes/scheduler.conf".to_string(),
            ],
        };

        // Append, then de-dup by flag name (last wins, mirroring Talos merge).
        let mut extras: Vec<(String, String)> = self
            .extra_args
            .iter()
            .filter(|(c, _, _)| *c == component)
            .map(|(_, k, v)| (k.clone(), v.clone()))
            .collect();
        extras.sort();
        for (k, v) in extras {
            args.push(format!("--{k}={v}"));
        }
        Ok(args)
    }

    /// The apiserver `--tls-sans` value: the comma-joined DNS/IP SAN hosts the
    /// serving certificate must cover (mirrors Talos `apiServerCertSANs`).
    pub fn apiserver_tls_sans(&self) -> Result<String> {
        let sans = apiserver_sans(&self.node)?;
        let hosts: Vec<String> = sans
            .into_iter()
            .map(|s| match s {
                SubjectAltName::Dns(d) => d,
                SubjectAltName::Ip(ip) => ip,
            })
            .collect();
        Ok(hosts.join(","))
    }

    /// The full apiserver argument list including the rendered `--tls-sans`.
    pub fn apiserver_args_with_sans(&self) -> Result<Vec<String>> {
        let mut args = self.args_for(K8sComponent::ApiServer)?;
        args.push(format!("--tls-sans={}", self.apiserver_tls_sans()?));
        Ok(args)
    }

    /// Render the static pod for a component, gating on the secret bundle.
    pub fn render_pod(&self, component: K8sComponent, secrets: &K8sSecrets) -> Result<StaticPod> {
        secrets.require_complete()?;
        let mut command = vec![component.name().to_string()];
        command.extend(self.args_for(component)?);
        StaticPod::control_plane(
            component.name(),
            component.image(&self.node.version),
            command,
        )
    }

    /// Render all three control-plane static pods.
    pub fn render_all(&self, secrets: &K8sSecrets) -> Result<Vec<StaticPod>> {
        secrets.require_complete()?;
        K8sComponent::ALL
            .iter()
            .map(|c| self.render_pod(*c, secrets))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ClusterEndpoint, NodeName};
    use crate::secrets::REQUIRED_SECRETS;

    fn cp_node(control_plane: bool) -> K8sConfig {
        K8sConfig {
            node_name: NodeName::new("cp-1").unwrap(),
            cluster_domain: "cluster.local".into(),
            pod_cidrs: vec!["10.244.0.0/16".into()],
            service_cidrs: vec!["10.96.0.0/12".into()],
            endpoint: ClusterEndpoint::new("api.example.com", 6443).unwrap(),
            version: "1.30.0".into(),
            control_plane,
        }
    }

    fn complete_secrets() -> K8sSecrets {
        let mut s = K8sSecrets::new();
        for name in REQUIRED_SECRETS {
            s.insert(*name, b"pem".to_vec());
        }
        s
    }

    #[test]
    fn component_parse_and_image() {
        assert_eq!(
            K8sComponent::parse("kube-scheduler").unwrap(),
            K8sComponent::Scheduler
        );
        assert!(K8sComponent::parse("nope").is_err());
        assert_eq!(
            K8sComponent::ApiServer.image("1.30.0"),
            "registry.k8s.io/kube-apiserver:v1.30.0"
        );
    }

    #[test]
    fn rejects_worker_node() {
        assert!(ControlPlaneConfig::new(cp_node(false)).is_err());
    }

    #[test]
    fn apiserver_args_include_service_range() {
        let cp = ControlPlaneConfig::new(cp_node(true)).unwrap();
        let args = cp.args_for(K8sComponent::ApiServer).unwrap();
        assert!(
            args.iter()
                .any(|a| a == "--service-cluster-ip-range=10.96.0.0/12")
        );
        assert!(args.iter().any(|a| a == "--authorization-mode=Node,RBAC"));
    }

    #[test]
    fn render_gated_on_secrets() {
        let cp = ControlPlaneConfig::new(cp_node(true)).unwrap();
        let err = cp
            .render_pod(K8sComponent::ApiServer, &K8sSecrets::new())
            .unwrap_err();
        assert_eq!(err.kind(), "missing_secret");
    }

    #[test]
    fn render_all_produces_three_pods() {
        let cp = ControlPlaneConfig::new(cp_node(true)).unwrap();
        let pods = cp.render_all(&complete_secrets()).unwrap();
        assert_eq!(pods.len(), 3);
        assert_eq!(pods[0].name, "kube-apiserver");
        assert!(
            pods[0].containers[0]
                .image
                .ends_with("kube-apiserver:v1.30.0")
        );
    }

    #[test]
    fn apiserver_args_include_etcd_and_front_proxy() {
        let cp = ControlPlaneConfig::new(cp_node(true)).unwrap();
        let args = cp.args_for(K8sComponent::ApiServer).unwrap();
        assert!(
            args.iter()
                .any(|a| a == "--etcd-cafile=/etc/kubernetes/pki/etcd/ca.crt")
        );
        assert!(
            args.iter()
                .any(|a| a
                    == "--requestheader-client-ca-file=/etc/kubernetes/pki/front-proxy-ca.crt")
        );
        assert!(
            args.iter()
                .any(|a| a == "--enable-admission-plugins=NodeRestriction")
        );
        assert!(
            args.iter()
                .any(|a| a.starts_with("--service-account-issuer="))
        );
    }

    #[test]
    fn controller_manager_has_signing_and_kubeconfig() {
        let cp = ControlPlaneConfig::new(cp_node(true)).unwrap();
        let args = cp.args_for(K8sComponent::ControllerManager).unwrap();
        assert!(
            args.iter()
                .any(|a| a == "--cluster-signing-cert-file=/etc/kubernetes/pki/ca.crt")
        );
        assert!(
            args.iter()
                .any(|a| a == "--kubeconfig=/etc/kubernetes/controller-manager.conf")
        );
    }

    #[test]
    fn scheduler_uses_kubeconfig() {
        let cp = ControlPlaneConfig::new(cp_node(true)).unwrap();
        let args = cp.args_for(K8sComponent::Scheduler).unwrap();
        assert!(
            args.iter()
                .any(|a| a == "--kubeconfig=/etc/kubernetes/scheduler.conf")
        );
    }

    #[test]
    fn apiserver_tls_sans_cover_endpoint_and_service_ip() {
        let cp = ControlPlaneConfig::new(cp_node(true)).unwrap();
        let sans = cp.apiserver_tls_sans().unwrap();
        assert!(sans.contains("api.example.com"));
        assert!(sans.contains("10.96.0.1"));
        assert!(sans.contains("kubernetes.default.svc.cluster.local"));
        let args = cp.apiserver_args_with_sans().unwrap();
        assert!(args.iter().any(|a| a.starts_with("--tls-sans=")));
    }

    #[test]
    fn extra_args_are_appended() {
        let cp = ControlPlaneConfig::new(cp_node(true))
            .unwrap()
            .with_extra_arg(K8sComponent::ApiServer, "--max-requests-inflight", "800");
        let args = cp.args_for(K8sComponent::ApiServer).unwrap();
        assert!(args.iter().any(|a| a == "--max-requests-inflight=800"));
    }
}
