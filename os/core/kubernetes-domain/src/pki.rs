//! Kubernetes PKI: certificate authorities and leaf certificate specs.
//!
//! Mirrors the Talos `pkg/kubernetes` / `k8s.Secrets` PKI layer and the
//! upstream `kubeadm` certificate tree: a set of CAs (cluster, etcd,
//! front-proxy) plus the leaf certificates the control-plane components present
//! (apiserver serving cert, apiserver-etcd-client, front-proxy-client, ...).
//!
//! We do not perform real crypto. Instead we model the *shape* of the PKI: the
//! CA identities, the Subject Alternative Names the apiserver serving cert must
//! carry, the key usages each leaf requires, and which CA signs which leaf. This
//! is exactly the metadata machined needs to decide what to render and to gate
//! static-pod rendering on a complete certificate tree.

use crate::config::K8sConfig;
use crate::error::{K8sError, Result};
use std::collections::BTreeSet;

/// A certificate authority in the Kubernetes PKI tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CertAuthority {
    /// The cluster CA (`ca.crt`/`ca.key`): signs the apiserver serving cert,
    /// kubelet client certs, and the admin cert.
    Cluster,
    /// The etcd CA (`etcd/ca.crt`): signs etcd peer/server/client certs.
    Etcd,
    /// The front-proxy CA (`front-proxy-ca.crt`): signs the aggregation-layer
    /// proxy client cert.
    FrontProxy,
}

impl CertAuthority {
    /// All certificate authorities in the tree.
    pub const ALL: [CertAuthority; 3] = [
        CertAuthority::Cluster,
        CertAuthority::Etcd,
        CertAuthority::FrontProxy,
    ];

    /// The CA's certificate filename relative to the PKI directory.
    pub fn cert_file(self) -> &'static str {
        match self {
            CertAuthority::Cluster => "ca.crt",
            CertAuthority::Etcd => "etcd/ca.crt",
            CertAuthority::FrontProxy => "front-proxy-ca.crt",
        }
    }

    /// The CA's private-key filename relative to the PKI directory.
    pub fn key_file(self) -> &'static str {
        match self {
            CertAuthority::Cluster => "ca.key",
            CertAuthority::Etcd => "etcd/ca.key",
            CertAuthority::FrontProxy => "front-proxy-ca.key",
        }
    }
}

/// A key usage bit a leaf certificate declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeyUsage {
    /// TLS server authentication (serving certs).
    ServerAuth,
    /// TLS client authentication (client certs).
    ClientAuth,
    /// Digital signature (always present on leaves).
    DigitalSignature,
    /// Key encipherment (RSA serving certs).
    KeyEncipherment,
}

/// A Subject Alternative Name entry on a serving certificate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubjectAltName {
    /// A DNS name.
    Dns(String),
    /// An IP address (kept as a string; we don't parse the family).
    Ip(String),
}

impl SubjectAltName {
    /// The rendered `DNS:`/`IP:` form used in a CSR / cert.
    pub fn render(&self) -> String {
        match self {
            SubjectAltName::Dns(d) => format!("DNS:{d}"),
            SubjectAltName::Ip(ip) => format!("IP:{ip}"),
        }
    }
}

/// The Subject Alternative Names the apiserver serving certificate must carry.
///
/// Mirrors kubeadm's `GetAPIServerAltNames`: the in-cluster service DNS names,
/// the kubernetes service IP (first service-CIDR address), localhost, and the
/// configured control-plane endpoint host.
pub fn apiserver_sans(cfg: &K8sConfig) -> Result<Vec<SubjectAltName>> {
    cfg.validate()?;
    let mut sans: Vec<SubjectAltName> = Vec::new();

    // Well-known in-cluster DNS names for the kubernetes service.
    let domain = cfg.cluster_domain.trim_end_matches('.');
    for dns in [
        "kubernetes".to_string(),
        "kubernetes.default".to_string(),
        "kubernetes.default.svc".to_string(),
        format!("kubernetes.default.svc.{domain}"),
    ] {
        sans.push(SubjectAltName::Dns(dns));
    }

    // The control-plane endpoint host (a DNS name or IP).
    let host = cfg.endpoint.host.clone();
    if is_ip_like(&host) {
        sans.push(SubjectAltName::Ip(host));
    } else {
        sans.push(SubjectAltName::Dns(host));
    }

    // The node's own registration name.
    sans.push(SubjectAltName::Dns(cfg.node_name.as_str().to_string()));

    // Loopback.
    sans.push(SubjectAltName::Ip("127.0.0.1".to_string()));

    // The first service-CIDR address (the in-cluster kubernetes service IP).
    let svc_ip = first_service_address(cfg)?;
    sans.push(SubjectAltName::Ip(svc_ip));

    // De-duplicate while preserving order.
    let mut seen: BTreeSet<String> = BTreeSet::new();
    sans.retain(|s| seen.insert(s.render()));
    Ok(sans)
}

/// The `kubernetes` service `ClusterIP`: the first usable address of the first
/// service CIDR (`a.b.c.0/n` -> `a.b.c.1`).
pub fn first_service_address(cfg: &K8sConfig) -> Result<String> {
    let cidr = cfg
        .service_cidrs
        .first()
        .ok_or_else(|| K8sError::InvalidConfig("no service CIDRs".to_string()))?;
    let base = cidr.split('/').next().unwrap_or(cidr);
    let mut octets: Vec<&str> = base.split('.').collect();
    if octets.len() != 4 {
        return Err(K8sError::InvalidConfig(format!(
            "service CIDR is not IPv4: {cidr}"
        )));
    }
    octets[3] = "1";
    Ok(octets.join("."))
}

/// A rough "looks like an IPv4 address" check (4 dot-separated numeric octets).
fn is_ip_like(host: &str) -> bool {
    let parts: Vec<&str> = host.split('.').collect();
    parts.len() == 4
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

/// A leaf certificate the PKI must provision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeafCert {
    /// Logical name, e.g. `"apiserver"`.
    pub name: String,
    /// The CA that signs this leaf.
    pub signer: CertAuthority,
    /// The certificate's common name.
    pub common_name: String,
    /// Organizations (O=) the cert is a member of; drives RBAC group binding.
    pub organizations: Vec<String>,
    /// Declared key usages.
    pub usages: Vec<KeyUsage>,
    /// Subject alternative names (empty for pure client certs).
    pub sans: Vec<SubjectAltName>,
}

impl LeafCert {
    /// The apiserver serving certificate, signed by the cluster CA.
    pub fn apiserver(cfg: &K8sConfig) -> Result<Self> {
        Ok(LeafCert {
            name: "apiserver".to_string(),
            signer: CertAuthority::Cluster,
            common_name: "kube-apiserver".to_string(),
            organizations: Vec::new(),
            usages: vec![
                KeyUsage::ServerAuth,
                KeyUsage::DigitalSignature,
                KeyUsage::KeyEncipherment,
            ],
            sans: apiserver_sans(cfg)?,
        })
    }

    /// The apiserver-to-etcd client certificate, signed by the etcd CA.
    pub fn apiserver_etcd_client() -> Self {
        LeafCert {
            name: "apiserver-etcd-client".to_string(),
            signer: CertAuthority::Etcd,
            common_name: "kube-apiserver-etcd-client".to_string(),
            // system:masters so the apiserver has full etcd access.
            organizations: vec!["system:masters".to_string()],
            usages: vec![KeyUsage::ClientAuth, KeyUsage::DigitalSignature],
            sans: Vec::new(),
        }
    }

    /// The front-proxy aggregation-layer client certificate.
    pub fn front_proxy_client() -> Self {
        LeafCert {
            name: "front-proxy-client".to_string(),
            signer: CertAuthority::FrontProxy,
            common_name: "front-proxy-client".to_string(),
            organizations: Vec::new(),
            usages: vec![KeyUsage::ClientAuth, KeyUsage::DigitalSignature],
            sans: Vec::new(),
        }
    }

    /// The apiserver-to-kubelet client certificate.
    pub fn apiserver_kubelet_client() -> Self {
        LeafCert {
            name: "apiserver-kubelet-client".to_string(),
            signer: CertAuthority::Cluster,
            common_name: "apiserver-kubelet-client".to_string(),
            organizations: vec!["system:masters".to_string()],
            usages: vec![KeyUsage::ClientAuth, KeyUsage::DigitalSignature],
            sans: Vec::new(),
        }
    }

    /// The admin client certificate (`O=system:masters`), signed by cluster CA.
    pub fn admin() -> Self {
        LeafCert {
            name: "admin".to_string(),
            signer: CertAuthority::Cluster,
            common_name: "kubernetes-admin".to_string(),
            organizations: vec!["system:masters".to_string()],
            usages: vec![KeyUsage::ClientAuth, KeyUsage::DigitalSignature],
            sans: Vec::new(),
        }
    }

    /// Whether this leaf is a serving (server-auth) certificate.
    pub fn is_serving(&self) -> bool {
        self.usages.contains(&KeyUsage::ServerAuth)
    }

    /// Whether this leaf is a client-auth certificate.
    pub fn is_client(&self) -> bool {
        self.usages.contains(&KeyUsage::ClientAuth)
    }

    /// Validate the leaf's invariants: a serving cert must carry SANs, and a
    /// non-serving cert must not.
    pub fn validate(&self) -> Result<()> {
        if self.common_name.trim().is_empty() {
            return Err(K8sError::InvalidConfig(
                "leaf cert has empty CN".to_string(),
            ));
        }
        if self.is_serving() && self.sans.is_empty() {
            return Err(K8sError::InvalidConfig(format!(
                "serving cert {} has no SANs",
                self.name
            )));
        }
        if !self.is_serving() && !self.sans.is_empty() {
            return Err(K8sError::InvalidConfig(format!(
                "client cert {} should not declare SANs",
                self.name
            )));
        }
        Ok(())
    }
}

/// The complete set of leaf certificates the control plane requires.
pub fn control_plane_leaves(cfg: &K8sConfig) -> Result<Vec<LeafCert>> {
    Ok(vec![
        LeafCert::apiserver(cfg)?,
        LeafCert::apiserver_etcd_client(),
        LeafCert::front_proxy_client(),
        LeafCert::apiserver_kubelet_client(),
        LeafCert::admin(),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ClusterEndpoint, NodeName};

    fn cfg(host: &str) -> K8sConfig {
        K8sConfig {
            node_name: NodeName::new("cp-1").unwrap(),
            cluster_domain: "cluster.local".into(),
            pod_cidrs: vec!["10.244.0.0/16".into()],
            service_cidrs: vec!["10.96.0.0/12".into()],
            endpoint: ClusterEndpoint::new(host, 6443).unwrap(),
            version: "1.30.0".into(),
            control_plane: true,
        }
    }

    #[test]
    fn ca_file_paths() {
        assert_eq!(CertAuthority::Cluster.cert_file(), "ca.crt");
        assert_eq!(CertAuthority::Etcd.key_file(), "etcd/ca.key");
        assert_eq!(CertAuthority::FrontProxy.cert_file(), "front-proxy-ca.crt");
        assert_eq!(CertAuthority::ALL.len(), 3);
    }

    #[test]
    fn first_service_address_is_dot_one() {
        assert_eq!(
            first_service_address(&cfg("api.example.com")).unwrap(),
            "10.96.0.1"
        );
    }

    #[test]
    fn apiserver_sans_include_well_known_dns_and_ips() {
        let sans = apiserver_sans(&cfg("api.example.com")).unwrap();
        let rendered: Vec<String> = sans.iter().map(super::SubjectAltName::render).collect();
        assert!(rendered.contains(&"DNS:kubernetes".to_string()));
        assert!(rendered.contains(&"DNS:kubernetes.default.svc.cluster.local".to_string()));
        assert!(rendered.contains(&"DNS:api.example.com".to_string()));
        assert!(rendered.contains(&"IP:127.0.0.1".to_string()));
        assert!(rendered.contains(&"IP:10.96.0.1".to_string()));
    }

    #[test]
    fn apiserver_sans_treats_ip_endpoint_as_ip() {
        let sans = apiserver_sans(&cfg("10.0.0.5")).unwrap();
        let rendered: Vec<String> = sans.iter().map(super::SubjectAltName::render).collect();
        assert!(rendered.contains(&"IP:10.0.0.5".to_string()));
        assert!(!rendered.contains(&"DNS:10.0.0.5".to_string()));
    }

    #[test]
    fn apiserver_sans_are_deduplicated() {
        let sans = apiserver_sans(&cfg("api.example.com")).unwrap();
        let mut rendered: Vec<String> = sans.iter().map(super::SubjectAltName::render).collect();
        let before = rendered.len();
        rendered.sort();
        rendered.dedup();
        assert_eq!(before, rendered.len());
    }

    #[test]
    fn apiserver_leaf_is_serving_with_sans() {
        let leaf = LeafCert::apiserver(&cfg("api.example.com")).unwrap();
        assert!(leaf.is_serving());
        assert!(!leaf.is_client());
        assert!(!leaf.sans.is_empty());
        assert_eq!(leaf.signer, CertAuthority::Cluster);
        leaf.validate().unwrap();
    }

    #[test]
    fn etcd_client_leaf_is_masters_client() {
        let leaf = LeafCert::apiserver_etcd_client();
        assert!(leaf.is_client());
        assert!(!leaf.is_serving());
        assert_eq!(leaf.signer, CertAuthority::Etcd);
        assert!(leaf.organizations.contains(&"system:masters".to_string()));
        leaf.validate().unwrap();
    }

    #[test]
    fn admin_leaf_is_in_masters_group() {
        let leaf = LeafCert::admin();
        assert_eq!(leaf.common_name, "kubernetes-admin");
        assert!(leaf.organizations.contains(&"system:masters".to_string()));
        leaf.validate().unwrap();
    }

    #[test]
    fn apiserver_kubelet_client_leaf_is_cluster_admin_client() {
        let leaf = LeafCert::apiserver_kubelet_client();
        assert_eq!(leaf.name, "apiserver-kubelet-client");
        assert_eq!(leaf.signer, CertAuthority::Cluster);
        assert!(leaf.is_client());
        assert!(!leaf.is_serving());
        assert!(leaf.organizations.contains(&"system:masters".to_string()));
        leaf.validate().unwrap();
    }

    #[test]
    fn validate_rejects_serving_cert_without_sans() {
        let mut leaf = LeafCert::admin();
        leaf.usages.push(KeyUsage::ServerAuth);
        // Now serving but no SANs.
        assert!(leaf.validate().is_err());
    }

    #[test]
    fn validate_rejects_client_cert_with_sans() {
        let mut leaf = LeafCert::admin();
        leaf.sans.push(SubjectAltName::Dns("x".into()));
        assert!(leaf.validate().is_err());
    }

    #[test]
    fn control_plane_leaves_has_all_boot_static_pod_leaves() {
        let leaves = control_plane_leaves(&cfg("api.example.com")).unwrap();
        assert_eq!(leaves.len(), 5);
        for l in &leaves {
            l.validate().unwrap();
        }
        let names: BTreeSet<&str> = leaves.iter().map(|l| l.name.as_str()).collect();
        assert!(names.contains("apiserver"));
        assert!(names.contains("apiserver-etcd-client"));
        assert!(names.contains("front-proxy-client"));
        assert!(names.contains("apiserver-kubelet-client"));
        assert!(names.contains("admin"));
        let signers: BTreeSet<CertAuthority> = leaves.iter().map(|l| l.signer).collect();
        assert!(signers.contains(&CertAuthority::Cluster));
        assert!(signers.contains(&CertAuthority::Etcd));
        assert!(signers.contains(&CertAuthority::FrontProxy));
    }

    #[test]
    fn san_render_forms() {
        assert_eq!(SubjectAltName::Dns("a.b".into()).render(), "DNS:a.b");
        assert_eq!(SubjectAltName::Ip("1.2.3.4".into()).render(), "IP:1.2.3.4");
    }
}
