//! Well-known bootstrap manifest templates.
//!
//! Mirrors Talos `BootstrapManifestController.render*` helpers: the fixed set of
//! cluster manifests machined renders and applies once at bootstrap — the
//! bootstrap RBAC bindings, the kube-proxy `DaemonSet`, the `CoreDNS` deployment and
//! its service, and the kubelet-bootstrap node-client CSR approval bindings.
//!
//! Each builder produces a [`Manifest`] with a real (if compact) YAML body
//! parameterised on the cluster config. The defaults below assemble the full
//! [`BootstrapManifests`] set in apply order.

use crate::bootstrap::BootstrapManifests;
use crate::config::K8sConfig;
use crate::error::Result;
use crate::manifests::{Manifest, ManifestKind};

/// The bootstrap RBAC: bind the `system:bootstrappers` group so node bootstrap
/// tokens can create CSRs, and auto-approve node-client certs.
pub fn bootstrap_rbac() -> Result<Manifest> {
    let body = "\
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: kubelet-bootstrap
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: system:node-bootstrapper
subjects:
- apiGroup: rbac.authorization.k8s.io
  kind: Group
  name: system:bootstrappers
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: kubelet-client-cert-approval
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: system:certificates.k8s.io:certificatesigningrequests:nodeclient
subjects:
- apiGroup: rbac.authorization.k8s.io
  kind: Group
  name: system:bootstrappers
";
    Manifest::new("11-bootstrap-rbac", ManifestKind::BootstrapRbac, body)
}

/// The kube-proxy `DaemonSet`, parameterised on the cluster CIDR and version.
pub fn kube_proxy(cfg: &K8sConfig) -> Result<Manifest> {
    cfg.validate()?;
    let cluster_cidr = cfg.pod_cidrs.join(",");
    let body = format!(
        "\
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: kube-proxy
  namespace: kube-system
spec:
  selector:
    matchLabels:
      k8s-app: kube-proxy
  template:
    metadata:
      labels:
        k8s-app: kube-proxy
    spec:
      hostNetwork: true
      tolerations:
      - operator: Exists
      containers:
      - name: kube-proxy
        image: registry.k8s.io/kube-proxy:v{version}
        command:
        - /usr/local/bin/kube-proxy
        - --cluster-cidr={cluster_cidr}
",
        version = cfg.version,
        cluster_cidr = cluster_cidr,
    );
    Manifest::new("10-kube-proxy", ManifestKind::KubeProxy, body)
}

/// The `CoreDNS` deployment + service, parameterised on the cluster DNS IP/domain.
pub fn coredns(cfg: &K8sConfig) -> Result<Manifest> {
    cfg.validate()?;
    let dns_ip = cfg.dns_service_ip()?;
    let domain = cfg.cluster_domain.trim_end_matches('.');
    let body = format!(
        "\
apiVersion: apps/v1
kind: Deployment
metadata:
  name: coredns
  namespace: kube-system
spec:
  replicas: 2
  selector:
    matchLabels:
      k8s-app: kube-dns
  template:
    metadata:
      labels:
        k8s-app: kube-dns
    spec:
      containers:
      - name: coredns
        image: registry.k8s.io/coredns/coredns:v1.11.1
        args:
        - -conf
        - /etc/coredns/Corefile
---
apiVersion: v1
kind: Service
metadata:
  name: kube-dns
  namespace: kube-system
spec:
  clusterIP: {dns_ip}
  ports:
  - name: dns
    port: 53
    protocol: UDP
  selector:
    k8s-app: kube-dns
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: coredns
  namespace: kube-system
data:
  Corefile: |
    .:53 {{
        kubernetes {domain} in-addr.arpa ip6.arpa
        forward . /etc/resolv.conf
    }}
",
    );
    Manifest::new("11-core-dns", ManifestKind::CoreDns, body)
}

/// The default pod-security policy manifest applied at bootstrap.
pub fn pod_security() -> Result<Manifest> {
    let body = "\
apiVersion: v1
kind: Namespace
metadata:
  name: kube-system
  labels:
    pod-security.kubernetes.io/enforce: privileged
";
    Manifest::new("00-pod-security-policy", ManifestKind::Policy, body)
}

/// Assemble the full default bootstrap manifest set in apply order.
///
/// Mirrors the controller's default rendering: RBAC, policy, kube-proxy, and
/// `CoreDNS`. The CNI is intentionally omitted (Talos applies a CNI manifest only
/// when one is configured), but a caller can `push` one before applying.
pub fn default_bootstrap_manifests(cfg: &K8sConfig) -> Result<BootstrapManifests> {
    let mut set = BootstrapManifests::new();
    set.push(bootstrap_rbac()?)?;
    set.push(pod_security()?)?;
    set.push(kube_proxy(cfg)?)?;
    set.push(coredns(cfg)?)?;
    Ok(set)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ClusterEndpoint, NodeName};

    fn cfg() -> K8sConfig {
        K8sConfig {
            node_name: NodeName::new("cp-1").unwrap(),
            cluster_domain: "cluster.local".into(),
            pod_cidrs: vec!["10.244.0.0/16".into()],
            service_cidrs: vec!["10.96.0.0/12".into()],
            endpoint: ClusterEndpoint::new("api.example.com", 6443).unwrap(),
            version: "1.30.0".into(),
            control_plane: true,
        }
    }

    #[test]
    fn bootstrap_rbac_binds_bootstrappers() {
        let m = bootstrap_rbac().unwrap();
        assert_eq!(m.kind, ManifestKind::BootstrapRbac);
        assert!(m.body.contains("system:node-bootstrapper"));
        assert!(m.body.contains("system:bootstrappers"));
    }

    #[test]
    fn kube_proxy_carries_cluster_cidr_and_version() {
        let m = kube_proxy(&cfg()).unwrap();
        assert!(m.body.contains("--cluster-cidr=10.244.0.0/16"));
        assert!(m.body.contains("kube-proxy:v1.30.0"));
        assert_eq!(m.kind, ManifestKind::KubeProxy);
    }

    #[test]
    fn coredns_carries_dns_ip_and_domain() {
        let m = coredns(&cfg()).unwrap();
        assert!(m.body.contains("clusterIP: 10.96.0.10"));
        assert!(m.body.contains("kubernetes cluster.local"));
        assert_eq!(m.kind, ManifestKind::CoreDns);
    }

    #[test]
    fn pod_security_is_a_policy() {
        let m = pod_security().unwrap();
        assert_eq!(m.kind, ManifestKind::Policy);
        assert!(m.body.contains("pod-security.kubernetes.io/enforce"));
    }

    #[test]
    fn default_set_is_ordered_and_applies() {
        let mut set = default_bootstrap_manifests(&cfg()).unwrap();
        assert_eq!(set.len(), 4);
        let ordered: Vec<&str> = set.ordered().iter().map(|m| m.name.as_str()).collect();
        // BootstrapRbac(0) < Policy(1) < KubeProxy(3) < CoreDns(4).
        assert_eq!(
            ordered,
            vec![
                "11-bootstrap-rbac",
                "00-pod-security-policy",
                "10-kube-proxy",
                "11-core-dns"
            ]
        );
        let applied = set.apply().unwrap();
        assert_eq!(applied.len(), 4);
    }

    #[test]
    fn can_push_cni_before_apply() {
        let mut set = default_bootstrap_manifests(&cfg()).unwrap();
        let cni = Manifest::new("05-flannel", ManifestKind::Cni, "kind: DaemonSet").unwrap();
        set.push(cni).unwrap();
        let ordered: Vec<&str> = set.ordered().iter().map(|m| m.name.as_str()).collect();
        // CNI(2) sits between Policy(1) and KubeProxy(3).
        let cni_pos = ordered.iter().position(|n| *n == "05-flannel").unwrap();
        let proxy_pos = ordered.iter().position(|n| *n == "10-kube-proxy").unwrap();
        assert!(cni_pos < proxy_pos);
    }

    #[test]
    fn content_hashes_differ_across_manifests() {
        let set = default_bootstrap_manifests(&cfg()).unwrap();
        let hashes: Vec<u64> = set.ordered().iter().map(|m| m.content_hash()).collect();
        let mut uniq = hashes.clone();
        uniq.sort_unstable();
        uniq.dedup();
        assert_eq!(hashes.len(), uniq.len());
    }
}
