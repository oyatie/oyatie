//! Projection of generated secret controllers into Kubernetes control-plane files.
//!
//! Talos generates cluster root material and leaf certificates in the secrets
//! controllers, then the Kubernetes controllers consume those values as named
//! files under `/etc/kubernetes/pki`. This module is the pure-model bridge
//! between those layers: it turns a reconciled [`SecretsBundle`],
//! [`KubernetesController`], and [`EtcdController`] into named opaque blobs that
//! can be fed into `talos-kubernetes::K8sSecrets` without adding a dependency
//! from this crate to `talos-kubernetes`.

use crate::bundle::{
    CaKind, Certificate, KeyPair, ModelSecretMaterialEncoder, SecretMaterialEncoder, SecretsBundle,
};
use crate::etcd::{EtcdCert, EtcdController};
use crate::kubernetes::{K8sCert, KubernetesController};
use os_kernel::error::{Error, Result};

/// One named Kubernetes secret file payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubernetesSecretEntry {
    /// Logical secret name, matching the Kubernetes control-plane file closure.
    pub name: &'static str,
    /// Opaque encoded bytes for the cert/key/public key.
    pub data: Vec<u8>,
}

impl KubernetesSecretEntry {
    fn new(name: &'static str, data: Vec<u8>) -> Result<Self> {
        if data.is_empty() {
            return Err(Error::invalid(format!(
                "encoded Kubernetes secret {name} is empty"
            )));
        }
        Ok(KubernetesSecretEntry { name, data })
    }

    fn cert(
        name: &'static str,
        cert: &Certificate,
        encoder: &impl SecretMaterialEncoder,
    ) -> Result<Self> {
        Self::new(name, encoder.certificate_bytes(cert)?)
    }

    fn private_key(
        name: &'static str,
        keypair: &KeyPair,
        encoder: &impl SecretMaterialEncoder,
    ) -> Result<Self> {
        Self::new(name, encoder.private_key_bytes(keypair)?)
    }

    fn public_key(
        name: &'static str,
        keypair: &KeyPair,
        encoder: &impl SecretMaterialEncoder,
    ) -> Result<Self> {
        Self::new(name, encoder.public_key_bytes(keypair)?)
    }

    /// Convert into a tuple accepted by `talos-kubernetes::K8sSecrets` builders.
    pub fn into_pair(self) -> (&'static str, Vec<u8>) {
        (self.name, self.data)
    }
}

/// The secret names this projection emits, in stable render order.
pub const KUBERNETES_SECRET_PROJECTION_NAMES: &[&str] = &[
    "ca.crt",
    "ca.key",
    "sa.key",
    "sa.pub",
    "etcd-ca.crt",
    "front-proxy-ca.crt",
    "apiserver.crt",
    "apiserver.key",
    "apiserver-etcd-client.crt",
    "apiserver-etcd-client.key",
    "front-proxy-client.crt",
    "front-proxy-client.key",
    "apiserver-kubelet-client.crt",
    "apiserver-kubelet-client.key",
    "controller-manager.crt",
    "controller-manager.key",
    "scheduler.crt",
    "scheduler.key",
    "admin.crt",
    "admin.key",
];

fn k8s_cert(controller: &KubernetesController, which: K8sCert) -> Result<&Certificate> {
    controller
        .certificate(which)
        .ok_or_else(|| Error::not_found(format!("Kubernetes certificate {which:?} not issued")))
}

fn etcd_cert(controller: &EtcdController, which: EtcdCert) -> Result<&Certificate> {
    controller
        .certificate(which)
        .ok_or_else(|| Error::not_found(format!("etcd certificate {which:?} not issued")))
}

/// Project a reconciled secrets bundle into Kubernetes control-plane secret
/// entries.
///
/// This function requires the caller to reconcile the Kubernetes and etcd
/// certificate controllers first. It fails closed if any leaf certificate is
/// missing instead of synthesizing replacement bytes.
pub fn kubernetes_secret_entries(
    bundle: &SecretsBundle,
    k8s: &KubernetesController,
    etcd: &EtcdController,
) -> Result<Vec<KubernetesSecretEntry>> {
    kubernetes_secret_entries_with_encoder(bundle, k8s, etcd, &ModelSecretMaterialEncoder)
}

/// Project a reconciled secrets bundle with an explicit material encoder.
///
/// This is the seam for replacing deterministic model file bytes with a real
/// PEM/DER backend while preserving the controller inputs, secret names, render
/// order, and fail-closed missing-leaf behavior.
pub fn kubernetes_secret_entries_with_encoder(
    bundle: &SecretsBundle,
    k8s: &KubernetesController,
    etcd: &EtcdController,
    encoder: &impl SecretMaterialEncoder,
) -> Result<Vec<KubernetesSecretEntry>> {
    let cluster_ca = bundle.ca(CaKind::Kubernetes);
    let etcd_ca = bundle.ca(CaKind::Etcd);
    let aggregator_ca = bundle.ca(CaKind::Aggregator);
    let service_account_key = bundle.service_account_key();

    let apiserver_cert = k8s_cert(k8s, K8sCert::ApiServer)?;
    let apiserver_key = K8sCert::ApiServer.keypair();
    let apiserver_etcd_client_cert = etcd_cert(etcd, EtcdCert::ApiServerClient)?;
    let apiserver_etcd_client_key = EtcdCert::ApiServerClient.keypair(etcd.node_name());
    let front_proxy_client_cert = k8s_cert(k8s, K8sCert::FrontProxy)?;
    let front_proxy_client_key = K8sCert::FrontProxy.keypair();
    let apiserver_kubelet_client_cert = k8s_cert(k8s, K8sCert::ApiServerKubeletClient)?;
    let apiserver_kubelet_client_key = K8sCert::ApiServerKubeletClient.keypair();
    let controller_manager_cert = k8s_cert(k8s, K8sCert::ControllerManager)?;
    let controller_manager_key = K8sCert::ControllerManager.keypair();
    let scheduler_cert = k8s_cert(k8s, K8sCert::Scheduler)?;
    let scheduler_key = K8sCert::Scheduler.keypair();
    let admin_cert = k8s_cert(k8s, K8sCert::Admin)?;
    let admin_key = K8sCert::Admin.keypair();

    Ok(vec![
        KubernetesSecretEntry::cert("ca.crt", cluster_ca.certificate(), encoder)?,
        KubernetesSecretEntry::private_key("ca.key", cluster_ca.keypair(), encoder)?,
        KubernetesSecretEntry::private_key("sa.key", service_account_key, encoder)?,
        KubernetesSecretEntry::public_key("sa.pub", service_account_key, encoder)?,
        KubernetesSecretEntry::cert("etcd-ca.crt", etcd_ca.certificate(), encoder)?,
        KubernetesSecretEntry::cert("front-proxy-ca.crt", aggregator_ca.certificate(), encoder)?,
        KubernetesSecretEntry::cert("apiserver.crt", apiserver_cert, encoder)?,
        KubernetesSecretEntry::private_key("apiserver.key", &apiserver_key, encoder)?,
        KubernetesSecretEntry::cert(
            "apiserver-etcd-client.crt",
            apiserver_etcd_client_cert,
            encoder,
        )?,
        KubernetesSecretEntry::private_key(
            "apiserver-etcd-client.key",
            &apiserver_etcd_client_key,
            encoder,
        )?,
        KubernetesSecretEntry::cert("front-proxy-client.crt", front_proxy_client_cert, encoder)?,
        KubernetesSecretEntry::private_key(
            "front-proxy-client.key",
            &front_proxy_client_key,
            encoder,
        )?,
        KubernetesSecretEntry::cert(
            "apiserver-kubelet-client.crt",
            apiserver_kubelet_client_cert,
            encoder,
        )?,
        KubernetesSecretEntry::private_key(
            "apiserver-kubelet-client.key",
            &apiserver_kubelet_client_key,
            encoder,
        )?,
        KubernetesSecretEntry::cert("controller-manager.crt", controller_manager_cert, encoder)?,
        KubernetesSecretEntry::private_key(
            "controller-manager.key",
            &controller_manager_key,
            encoder,
        )?,
        KubernetesSecretEntry::cert("scheduler.crt", scheduler_cert, encoder)?,
        KubernetesSecretEntry::private_key("scheduler.key", &scheduler_key, encoder)?,
        KubernetesSecretEntry::cert("admin.crt", admin_cert, encoder)?,
        KubernetesSecretEntry::private_key("admin.key", &admin_key, encoder)?,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ModelPemSecretMaterialEncoder;
    use crate::bundle::hex;
    use crate::certsans::CertSans;
    use os_kernel::NodeAddress;
    use os_kernel::error::{Error, Result};
    use std::collections::BTreeSet;

    fn bundle() -> SecretsBundle {
        SecretsBundle::generate("projection-cluster", 1000).unwrap()
    }

    fn k8s_controller() -> KubernetesController {
        let mut sans = CertSans::new();
        sans.append("api.example.com").unwrap();
        KubernetesController::new(sans, "cluster.local").unwrap()
    }

    fn etcd_controller() -> EtcdController {
        EtcdController::new("cp-1", &[NodeAddress::parse("10.0.0.5").unwrap()]).unwrap()
    }

    fn reconciled_projection_inputs() -> (SecretsBundle, KubernetesController, EtcdController) {
        let mut bundle = bundle();
        let mut k8s = k8s_controller();
        let mut etcd = etcd_controller();
        k8s.reconcile(&mut bundle, 1000).unwrap();
        etcd.reconcile(&mut bundle, 1000).unwrap();
        (bundle, k8s, etcd)
    }

    fn entry<'a>(entries: &'a [KubernetesSecretEntry], name: &str) -> &'a KubernetesSecretEntry {
        entries
            .iter()
            .find(|entry| entry.name == name)
            .unwrap_or_else(|| panic!("missing projected secret entry {name}"))
    }

    #[test]
    fn projection_requires_reconciled_leaf_certificates() {
        let bundle = bundle();
        let k8s = k8s_controller();
        let etcd = etcd_controller();

        let err = kubernetes_secret_entries(&bundle, &k8s, &etcd).unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }

    #[test]
    fn projection_emits_complete_kubernetes_secret_closure() {
        let (bundle, k8s, etcd) = reconciled_projection_inputs();

        let entries = kubernetes_secret_entries(&bundle, &k8s, &etcd).unwrap();
        let names: Vec<&str> = entries.iter().map(|entry| entry.name).collect();
        assert_eq!(names, KUBERNETES_SECRET_PROJECTION_NAMES);
        assert!(entries.iter().all(|entry| !entry.data.is_empty()));

        let unique: BTreeSet<&str> = names.iter().copied().collect();
        assert_eq!(unique.len(), names.len());

        let apiserver = entries
            .iter()
            .find(|entry| entry.name == "apiserver.crt")
            .unwrap();
        let apiserver_text = String::from_utf8(apiserver.data.clone()).unwrap();
        assert!(apiserver_text.contains("subject=CN=kube-apiserver"));
        assert!(apiserver_text.contains("usage=ServerAuth"));
        assert!(apiserver_text.contains("api.example.com"));

        let etcd_client = entries
            .iter()
            .find(|entry| entry.name == "apiserver-etcd-client.crt")
            .unwrap();
        let etcd_client_text = String::from_utf8(etcd_client.data.clone()).unwrap();
        assert!(etcd_client_text.contains("subject=CN=kube-apiserver-etcd-client"));
        assert!(
            etcd_client_text.contains(&hex(EtcdCert::ApiServerClient
                .keypair(etcd.node_name())
                .public_der(),))
        );

        let ca_key = entries.iter().find(|entry| entry.name == "ca.key").unwrap();
        assert!(
            String::from_utf8(ca_key.data.clone())
                .unwrap()
                .contains("KUBEROS-MODEL-PRIVATE-KEY")
        );
    }

    #[test]
    fn projection_default_encoder_preserves_wave59_projection_names_and_model_markers() {
        let (bundle, k8s, etcd) = reconciled_projection_inputs();

        let entries = kubernetes_secret_entries(&bundle, &k8s, &etcd).unwrap();
        let names: Vec<&str> = entries.iter().map(|entry| entry.name).collect();
        assert_eq!(names, KUBERNETES_SECRET_PROJECTION_NAMES);
        assert_eq!(entries.len(), 20);
        assert!(entries.iter().all(|entry| !entry.data.is_empty()));

        assert_eq!(
            entry(&entries, "ca.crt").data,
            bundle
                .ca(CaKind::Kubernetes)
                .certificate()
                .model_certificate_bytes()
        );
        assert_eq!(
            entry(&entries, "ca.key").data,
            bundle
                .ca(CaKind::Kubernetes)
                .keypair()
                .model_private_key_bytes()
        );
        assert_eq!(
            entry(&entries, "sa.pub").data,
            bundle.service_account_key().model_public_key_bytes()
        );
        assert_eq!(
            entry(&entries, "apiserver.crt").data,
            k8s.certificate(K8sCert::ApiServer)
                .unwrap()
                .model_certificate_bytes()
        );
        assert_eq!(
            entry(&entries, "admin.key").data,
            K8sCert::Admin.keypair().model_private_key_bytes()
        );

        let combined = entries
            .iter()
            .flat_map(|entry| entry.data.iter().copied())
            .collect::<Vec<_>>();
        let combined = String::from_utf8_lossy(&combined);
        assert!(combined.contains("KUBEROS-MODEL-CERTIFICATE"));
        assert!(combined.contains("KUBEROS-MODEL-PRIVATE-KEY"));
        assert!(combined.contains("KUBEROS-MODEL-PUBLIC-KEY"));
    }

    #[derive(Debug, Clone, Copy)]
    struct TaggedMaterialEncoder;

    impl SecretMaterialEncoder for TaggedMaterialEncoder {
        fn certificate_bytes(&self, cert: &Certificate) -> Result<Vec<u8>> {
            Ok(format!(
                "CERT|subject={}|issuer={}|public={}",
                cert.subject.to_rfc(),
                cert.issuer.to_rfc(),
                hex(&cert.public_key_der)
            )
            .into_bytes())
        }

        fn private_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>> {
            Ok(format!("PRIVATE|public={}", hex(keypair.public_der())).into_bytes())
        }

        fn public_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>> {
            Ok(format!("PUBLIC|public={}", hex(keypair.public_der())).into_bytes())
        }
    }

    #[test]
    fn projection_accepts_pluggable_crypto_material_encoder_for_all_material_kinds() {
        let (bundle, k8s, etcd) = reconciled_projection_inputs();

        let entries =
            kubernetes_secret_entries_with_encoder(&bundle, &k8s, &etcd, &TaggedMaterialEncoder)
                .unwrap();
        let names: Vec<&str> = entries.iter().map(|entry| entry.name).collect();
        assert_eq!(names, KUBERNETES_SECRET_PROJECTION_NAMES);
        assert!(entries.iter().all(|entry| !entry.data.is_empty()));

        let text = |name: &str| -> String {
            let entry = entries.iter().find(|entry| entry.name == name).unwrap();
            String::from_utf8(entry.data.clone()).unwrap()
        };

        assert!(text("ca.crt").starts_with("CERT|subject=CN=kubernetes|issuer=CN=kubernetes"));
        assert!(text("apiserver.crt").starts_with("CERT|subject=CN=kube-apiserver|"));
        assert!(text("ca.key").starts_with("PRIVATE|public="));
        assert!(text("sa.pub").starts_with("PUBLIC|public="));
        assert!(
            entries
                .iter()
                .all(|entry| !String::from_utf8_lossy(&entry.data).contains("KUBEROS-MODEL-"))
        );
    }

    #[test]
    fn model_pem_encoder_emits_pem_blocks_for_all_20_kubernetes_secret_entries() {
        let (bundle, k8s, etcd) = reconciled_projection_inputs();

        let entries = kubernetes_secret_entries_with_encoder(
            &bundle,
            &k8s,
            &etcd,
            &ModelPemSecretMaterialEncoder,
        )
        .unwrap();
        let names: Vec<&str> = entries.iter().map(|entry| entry.name).collect();
        assert_eq!(names, KUBERNETES_SECRET_PROJECTION_NAMES);
        assert_eq!(entries.len(), 20);
        assert!(entries.iter().all(|entry| !entry.data.is_empty()));

        let text = |name: &str| -> String {
            let entry = entries.iter().find(|entry| entry.name == name).unwrap();
            String::from_utf8(entry.data.clone()).unwrap()
        };

        assert!(text("ca.crt").starts_with("-----BEGIN CERTIFICATE-----\n"));
        assert!(text("apiserver.crt").starts_with("-----BEGIN CERTIFICATE-----\n"));
        assert!(text("ca.key").starts_with("-----BEGIN PRIVATE KEY-----\n"));
        assert!(text("sa.key").starts_with("-----BEGIN PRIVATE KEY-----\n"));
        assert!(text("sa.pub").starts_with("-----BEGIN PUBLIC KEY-----\n"));
        assert!(text("admin.key").ends_with("-----END PRIVATE KEY-----\n"));
        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.name.ends_with(".crt"))
                .count(),
            10
        );
        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.name.ends_with(".key"))
                .count(),
            9
        );
        assert_eq!(
            entries
                .iter()
                .filter(|entry| entry.name == "sa.pub")
                .count(),
            1
        );
        assert!(
            entries
                .iter()
                .all(|entry| !String::from_utf8_lossy(&entry.data).contains("KUBEROS-MODEL-"))
        );
    }

    #[derive(Debug, Clone, Copy)]
    struct EmptyPrivateKeyEncoder;

    impl SecretMaterialEncoder for EmptyPrivateKeyEncoder {
        fn certificate_bytes(&self, cert: &Certificate) -> Result<Vec<u8>> {
            TaggedMaterialEncoder.certificate_bytes(cert)
        }

        fn private_key_bytes(&self, _keypair: &KeyPair) -> Result<Vec<u8>> {
            Ok(Vec::new())
        }

        fn public_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>> {
            TaggedMaterialEncoder.public_key_bytes(keypair)
        }
    }

    #[test]
    fn projection_fails_closed_when_encoder_returns_empty_private_key_material() {
        let (bundle, k8s, etcd) = reconciled_projection_inputs();

        let err =
            kubernetes_secret_entries_with_encoder(&bundle, &k8s, &etcd, &EmptyPrivateKeyEncoder)
                .unwrap_err();
        assert_eq!(err.kind(), "invalid");
        assert!(
            err.to_string()
                .contains("encoded Kubernetes secret ca.key is empty"),
            "{err}"
        );
    }

    #[derive(Debug, Clone, Copy)]
    struct RejectCertificateEncoder;

    impl SecretMaterialEncoder for RejectCertificateEncoder {
        fn certificate_bytes(&self, _cert: &Certificate) -> Result<Vec<u8>> {
            Err(Error::invalid_state(
                "certificate material encoder rejected material",
            ))
        }

        fn private_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>> {
            TaggedMaterialEncoder.private_key_bytes(keypair)
        }

        fn public_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>> {
            TaggedMaterialEncoder.public_key_bytes(keypair)
        }
    }

    #[test]
    fn projection_fails_closed_when_encoder_rejects_certificate_material() {
        let (bundle, k8s, etcd) = reconciled_projection_inputs();

        let err =
            kubernetes_secret_entries_with_encoder(&bundle, &k8s, &etcd, &RejectCertificateEncoder)
                .unwrap_err();
        assert_eq!(err.kind(), "invalid_state");
        assert!(
            err.to_string()
                .contains("certificate material encoder rejected material"),
            "{err}"
        );
    }

    #[derive(Debug, Clone, Copy)]
    struct RejectPublicKeyEncoder;

    impl SecretMaterialEncoder for RejectPublicKeyEncoder {
        fn certificate_bytes(&self, cert: &Certificate) -> Result<Vec<u8>> {
            TaggedMaterialEncoder.certificate_bytes(cert)
        }

        fn private_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>> {
            TaggedMaterialEncoder.private_key_bytes(keypair)
        }

        fn public_key_bytes(&self, _keypair: &KeyPair) -> Result<Vec<u8>> {
            Err(Error::invalid_state(
                "public key material encoder rejected material",
            ))
        }
    }

    #[test]
    fn projection_fails_closed_when_encoder_rejects_public_key_material() {
        let (bundle, k8s, etcd) = reconciled_projection_inputs();

        let err =
            kubernetes_secret_entries_with_encoder(&bundle, &k8s, &etcd, &RejectPublicKeyEncoder)
                .unwrap_err();
        assert_eq!(err.kind(), "invalid_state");
        assert!(
            err.to_string()
                .contains("public key material encoder rejected material"),
            "{err}"
        );
    }

    #[derive(Debug, Clone, Copy)]
    struct EmptyCertificateEncoder;

    impl SecretMaterialEncoder for EmptyCertificateEncoder {
        fn certificate_bytes(&self, _cert: &Certificate) -> Result<Vec<u8>> {
            Ok(Vec::new())
        }

        fn private_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>> {
            TaggedMaterialEncoder.private_key_bytes(keypair)
        }

        fn public_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>> {
            TaggedMaterialEncoder.public_key_bytes(keypair)
        }
    }

    #[test]
    fn encoder_returns_empty_certificate_material_fails_closed() {
        let (bundle, k8s, etcd) = reconciled_projection_inputs();

        let err =
            kubernetes_secret_entries_with_encoder(&bundle, &k8s, &etcd, &EmptyCertificateEncoder)
                .unwrap_err();
        assert_eq!(err.kind(), "invalid");
        assert!(
            err.to_string()
                .contains("encoded Kubernetes secret ca.crt is empty"),
            "{err}"
        );
    }

    #[derive(Debug, Clone, Copy)]
    struct EmptyPublicKeyEncoder;

    impl SecretMaterialEncoder for EmptyPublicKeyEncoder {
        fn certificate_bytes(&self, cert: &Certificate) -> Result<Vec<u8>> {
            TaggedMaterialEncoder.certificate_bytes(cert)
        }

        fn private_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>> {
            TaggedMaterialEncoder.private_key_bytes(keypair)
        }

        fn public_key_bytes(&self, _keypair: &KeyPair) -> Result<Vec<u8>> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn encoder_returns_empty_public_key_material_fails_closed() {
        let (bundle, k8s, etcd) = reconciled_projection_inputs();

        let err =
            kubernetes_secret_entries_with_encoder(&bundle, &k8s, &etcd, &EmptyPublicKeyEncoder)
                .unwrap_err();
        assert_eq!(err.kind(), "invalid");
        assert!(
            err.to_string()
                .contains("encoded Kubernetes secret sa.pub is empty"),
            "{err}"
        );
    }

    #[derive(Debug, Clone, Copy)]
    struct RejectPrivateKeyEncoder;

    impl SecretMaterialEncoder for RejectPrivateKeyEncoder {
        fn certificate_bytes(&self, cert: &Certificate) -> Result<Vec<u8>> {
            TaggedMaterialEncoder.certificate_bytes(cert)
        }

        fn private_key_bytes(&self, _keypair: &KeyPair) -> Result<Vec<u8>> {
            Err(Error::invalid_state(
                "private key material encoder rejected material",
            ))
        }

        fn public_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>> {
            TaggedMaterialEncoder.public_key_bytes(keypair)
        }
    }

    #[test]
    fn encoder_rejects_private_key_material_fails_closed() {
        let (bundle, k8s, etcd) = reconciled_projection_inputs();

        let err =
            kubernetes_secret_entries_with_encoder(&bundle, &k8s, &etcd, &RejectPrivateKeyEncoder)
                .unwrap_err();
        assert_eq!(err.kind(), "invalid_state");
        assert!(
            err.to_string()
                .contains("private key material encoder rejected material"),
            "{err}"
        );
    }
}
