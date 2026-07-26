//! End-to-end control-plane provisioning: the rendered file set machined writes.
//!
//! Mirrors the way the Talos k8s controllers fan a validated cluster config plus
//! the secret bundle out into the concrete files on a control-plane node:
//!
//! * `/etc/kubernetes/manifests/<component>.yaml` — the control-plane static pods.
//! * `/etc/kubernetes/pki/...` — the CA, service-account, and component
//!   certificate/key files.
//! * `/etc/kubernetes/<component>.conf` — the component kubeconfigs with
//!   standard-base64 `*-data` credential fields.
//! * `/etc/kubernetes/admin.conf` — the admin kubeconfig with standard-base64
//!   `*-data` credential fields.
//! * `/etc/kubernetes/kubelet.yaml` — the kubelet config.
//!
//! Everything is gated on a complete secret bundle, and the result is a
//! [`RenderedOutput`] that can be flushed through any [`crate::rendered::FileSink`].

use crate::config::K8sConfig;
use crate::control_plane::{ControlPlaneConfig, K8sComponent};
use crate::encoding::kubeconfig_data;
use crate::error::{K8sError, Result};
use crate::kubeconfig::KubeConfig;
use crate::kubelet::{KubeletConfig, KubeletSpec};
use crate::rendered::{FileMode, RenderedFile, RenderedOutput};
use crate::secrets::K8sSecrets;

/// The directory control-plane configs live under.
pub const KUBERNETES_DIR: &str = "/etc/kubernetes";
/// The PKI sub-directory.
pub const PKI_DIR: &str = "/etc/kubernetes/pki";

#[derive(Debug, Clone, Copy)]
struct PkiSecretFile {
    secret_name: &'static str,
    relative_path: &'static str,
    mode: FileMode,
}

/// Kubernetes PKI files that must be present before control-plane static pods
/// can boot. The source secret names intentionally match
/// [`crate::secrets::REQUIRED_SECRETS`], while the relative paths match the
/// static-pod arguments rendered by [`ControlPlaneConfig`].
const CONTROL_PLANE_PKI_SECRET_FILES: &[PkiSecretFile] = &[
    PkiSecretFile {
        secret_name: "ca.crt",
        relative_path: "ca.crt",
        mode: FileMode::CONFIG,
    },
    PkiSecretFile {
        secret_name: "ca.key",
        relative_path: "ca.key",
        mode: FileMode::SECRET,
    },
    PkiSecretFile {
        secret_name: "sa.key",
        relative_path: "sa.key",
        mode: FileMode::SECRET,
    },
    PkiSecretFile {
        secret_name: "sa.pub",
        relative_path: "sa.pub",
        mode: FileMode::CONFIG,
    },
    PkiSecretFile {
        secret_name: "etcd-ca.crt",
        relative_path: "etcd/ca.crt",
        mode: FileMode::CONFIG,
    },
    PkiSecretFile {
        secret_name: "front-proxy-ca.crt",
        relative_path: "front-proxy-ca.crt",
        mode: FileMode::CONFIG,
    },
    PkiSecretFile {
        secret_name: "apiserver.crt",
        relative_path: "apiserver.crt",
        mode: FileMode::CONFIG,
    },
    PkiSecretFile {
        secret_name: "apiserver.key",
        relative_path: "apiserver.key",
        mode: FileMode::SECRET,
    },
    PkiSecretFile {
        secret_name: "apiserver-etcd-client.crt",
        relative_path: "apiserver-etcd-client.crt",
        mode: FileMode::CONFIG,
    },
    PkiSecretFile {
        secret_name: "apiserver-etcd-client.key",
        relative_path: "apiserver-etcd-client.key",
        mode: FileMode::SECRET,
    },
    PkiSecretFile {
        secret_name: "front-proxy-client.crt",
        relative_path: "front-proxy-client.crt",
        mode: FileMode::CONFIG,
    },
    PkiSecretFile {
        secret_name: "front-proxy-client.key",
        relative_path: "front-proxy-client.key",
        mode: FileMode::SECRET,
    },
    PkiSecretFile {
        secret_name: "apiserver-kubelet-client.crt",
        relative_path: "apiserver-kubelet-client.crt",
        mode: FileMode::CONFIG,
    },
    PkiSecretFile {
        secret_name: "apiserver-kubelet-client.key",
        relative_path: "apiserver-kubelet-client.key",
        mode: FileMode::SECRET,
    },
];

fn render_pki_secret_files(out: &mut RenderedOutput, secrets: &K8sSecrets) -> Result<()> {
    for spec in CONTROL_PLANE_PKI_SECRET_FILES {
        let data = secrets
            .get(spec.secret_name)
            .ok_or_else(|| K8sError::MissingSecret(spec.secret_name.to_string()))?;
        out.add(RenderedFile::new(
            format!("{PKI_DIR}/{}", spec.relative_path),
            data.to_vec(),
            spec.mode,
        )?)?;
    }
    Ok(())
}

fn kubeconfig_secret_data(secrets: &K8sSecrets, name: &str) -> Result<String> {
    let data = secrets
        .get(name)
        .ok_or_else(|| K8sError::MissingSecret(name.to_string()))?;
    Ok(kubeconfig_data(data))
}

/// Provision the full control-plane file set for one node.
///
/// Gates on a complete secret bundle and a control-plane node config, then
/// renders the static pods, CA certs, kubeconfigs, and kubelet config into a
/// single [`RenderedOutput`].
pub fn provision_control_plane(
    cfg: &K8sConfig,
    secrets: &K8sSecrets,
    cluster_name: &str,
) -> Result<RenderedOutput> {
    cfg.validate()?;
    if !cfg.control_plane {
        return Err(K8sError::InvalidConfig(
            "provision_control_plane called for a worker node".to_string(),
        ));
    }
    secrets.require_complete()?;

    let mut out = RenderedOutput::new();

    // 1. CA cert/key files, service-account keys, and control-plane leaf certs
    // from the secret bundle, written into the PKI dir.
    render_pki_secret_files(&mut out, secrets)?;

    // 2. The control-plane static pods.
    let cp = ControlPlaneConfig::new(cfg.clone())?;
    for component in K8sComponent::ALL {
        let pod = cp.render_pod(component, secrets)?;
        let path = format!("{KUBERNETES_DIR}/manifests/{}", pod.manifest_filename());
        out.add(RenderedFile::new(
            path,
            pod.render().into_bytes(),
            FileMode::CONFIG,
        )?)?;
    }

    // 3. The component kubeconfigs (controller-manager, scheduler) + admin.
    let ca_pem = kubeconfig_secret_data(secrets, "ca.crt")?;
    for (user, file, cert, key) in [
        (
            "system:kube-controller-manager",
            "controller-manager.conf",
            "controller-manager.crt",
            "controller-manager.key",
        ),
        (
            "system:kube-scheduler",
            "scheduler.conf",
            "scheduler.crt",
            "scheduler.key",
        ),
    ] {
        let kc = KubeConfig::component(
            user,
            cluster_name,
            ca_pem.clone(),
            kubeconfig_secret_data(secrets, cert)?,
            kubeconfig_secret_data(secrets, key)?,
        )?;
        out.add(RenderedFile::new(
            format!("{KUBERNETES_DIR}/{file}"),
            kc.render().into_bytes(),
            FileMode::SECRET,
        )?)?;
    }
    let admin = KubeConfig::admin(
        cfg,
        cluster_name,
        ca_pem,
        kubeconfig_secret_data(secrets, "admin.crt")?,
        kubeconfig_secret_data(secrets, "admin.key")?,
    )?;
    out.add(RenderedFile::new(
        format!("{KUBERNETES_DIR}/admin.conf"),
        admin.render().into_bytes(),
        FileMode::SECRET,
    )?)?;

    // 4. The kubelet config.
    let kubelet = KubeletConfig::from_node_config(cfg)?;
    let spec = KubeletSpec::render(&kubelet)?;
    out.add(RenderedFile::new(
        format!("{KUBERNETES_DIR}/kubelet.yaml"),
        spec.args.join("\n").into_bytes(),
        FileMode::CONFIG,
    )?)?;

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ClusterEndpoint, NodeName};
    use crate::rendered::InMemoryFileSink;
    use crate::secrets::REQUIRED_SECRETS;
    use std::collections::{BTreeMap, BTreeSet};
    use os_kernel::NodeAddress;
    use os_secrets_domain::certsans::CertSans;
    use os_secrets_domain::etcd::EtcdController;
    use os_secrets_domain::kubernetes::KubernetesController;
    use os_secrets_domain::{
        ModelPemSecretMaterialEncoder, SecretsBundle, kubernetes_secret_entries,
        kubernetes_secret_entries_with_encoder,
    };

    fn cfg(control_plane: bool) -> K8sConfig {
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

    fn secrets() -> K8sSecrets {
        let mut s = K8sSecrets::new();
        for name in REQUIRED_SECRETS {
            s.insert(*name, format!("PEM-{name}").into_bytes());
        }
        s
    }

    fn rendered_by_path(out: &RenderedOutput) -> BTreeMap<&str, &RenderedFile> {
        out.files()
            .iter()
            .map(|f| (f.path.as_str(), f))
            .collect::<BTreeMap<_, _>>()
    }

    fn kubeconfig_field<'a>(body: &'a str, field: &str) -> &'a str {
        let prefix = format!("    {field}: ");
        body.lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap_or_else(|| panic!("missing kubeconfig field {field}"))
    }

    fn generated_secrets() -> K8sSecrets {
        let mut bundle = SecretsBundle::generate("generated-provision", 1000).unwrap();
        let mut sans = CertSans::new();
        sans.append("api.example.com").unwrap();
        sans.append("cp-1").unwrap();
        let mut k8s = KubernetesController::new(sans, "cluster.local").unwrap();
        let mut etcd =
            EtcdController::new("cp-1", &[NodeAddress::parse("10.0.0.5").unwrap()]).unwrap();
        k8s.reconcile(&mut bundle, 1000).unwrap();
        etcd.reconcile(&mut bundle, 1000).unwrap();
        let entries = kubernetes_secret_entries(&bundle, &k8s, &etcd).unwrap();
        K8sSecrets::from_required_entries(entries.into_iter().map(|entry| entry.into_pair()))
            .unwrap()
    }

    fn generated_pem_secrets() -> K8sSecrets {
        let mut bundle = SecretsBundle::generate("generated-pem-provision", 1000).unwrap();
        let mut sans = CertSans::new();
        sans.append("api.example.com").unwrap();
        sans.append("cp-1").unwrap();
        let mut k8s = KubernetesController::new(sans, "cluster.local").unwrap();
        let mut etcd =
            EtcdController::new("cp-1", &[NodeAddress::parse("10.0.0.5").unwrap()]).unwrap();
        k8s.reconcile(&mut bundle, 1000).unwrap();
        etcd.reconcile(&mut bundle, 1000).unwrap();
        let entries = kubernetes_secret_entries_with_encoder(
            &bundle,
            &k8s,
            &etcd,
            &ModelPemSecretMaterialEncoder,
        )
        .unwrap();
        K8sSecrets::from_required_entries(entries.into_iter().map(|entry| entry.into_pair()))
            .unwrap()
    }

    #[test]
    fn provision_rejects_worker_and_incomplete_secrets() {
        assert!(provision_control_plane(&cfg(false), &secrets(), "talos").is_err());
        assert!(provision_control_plane(&cfg(true), &K8sSecrets::new(), "talos").is_err());
    }

    #[test]
    fn provision_renders_all_files() {
        let out = provision_control_plane(&cfg(true), &secrets(), "talos").unwrap();
        let paths: Vec<&str> = out.files().iter().map(|f| f.path.as_str()).collect();
        assert!(paths.contains(&"/etc/kubernetes/pki/ca.crt"));
        assert!(paths.contains(&"/etc/kubernetes/pki/ca.key"));
        assert!(paths.contains(&"/etc/kubernetes/pki/etcd/ca.crt"));
        assert!(paths.contains(&"/etc/kubernetes/pki/sa.key"));
        assert!(paths.contains(&"/etc/kubernetes/pki/apiserver.crt"));
        assert!(paths.contains(&"/etc/kubernetes/pki/apiserver.key"));
        assert!(paths.contains(&"/etc/kubernetes/pki/apiserver-etcd-client.crt"));
        assert!(paths.contains(&"/etc/kubernetes/pki/apiserver-etcd-client.key"));
        assert!(paths.contains(&"/etc/kubernetes/pki/front-proxy-client.crt"));
        assert!(paths.contains(&"/etc/kubernetes/pki/front-proxy-client.key"));
        assert!(paths.contains(&"/etc/kubernetes/pki/apiserver-kubelet-client.crt"));
        assert!(paths.contains(&"/etc/kubernetes/pki/apiserver-kubelet-client.key"));
        assert!(paths.contains(&"/etc/kubernetes/manifests/kube-apiserver.yaml"));
        assert!(paths.contains(&"/etc/kubernetes/manifests/kube-scheduler.yaml"));
        assert!(paths.contains(&"/etc/kubernetes/admin.conf"));
        assert!(paths.contains(&"/etc/kubernetes/controller-manager.conf"));
        assert!(paths.contains(&"/etc/kubernetes/kubelet.yaml"));
    }

    #[test]
    fn secret_files_have_secret_mode() {
        let out = provision_control_plane(&cfg(true), &secrets(), "talos").unwrap();
        let sa_key = out
            .files()
            .iter()
            .find(|f| f.path == "/etc/kubernetes/pki/sa.key")
            .unwrap();
        assert!(sa_key.is_secret());
        let admin = out
            .files()
            .iter()
            .find(|f| f.path == "/etc/kubernetes/admin.conf")
            .unwrap();
        assert!(admin.is_secret());
        let ca = out
            .files()
            .iter()
            .find(|f| f.path == "/etc/kubernetes/pki/ca.crt")
            .unwrap();
        assert!(!ca.is_secret());
    }

    #[test]
    fn pki_file_closure_renders_every_control_plane_arg_path() {
        let cfg = cfg(true);
        let out = provision_control_plane(&cfg, &secrets(), "talos").unwrap();
        let rendered = rendered_by_path(&out);
        let cp = ControlPlaneConfig::new(cfg).unwrap();
        let mut referenced_pki_paths = BTreeSet::new();

        for component in K8sComponent::ALL {
            for arg in cp.args_for(component).unwrap() {
                let Some((_, value)) = arg.split_once('=') else {
                    continue;
                };
                if value.starts_with(PKI_DIR) {
                    referenced_pki_paths.insert(value.to_string());
                }
            }
        }

        assert!(
            !referenced_pki_paths.is_empty(),
            "control-plane args should reference PKI paths"
        );
        for path in referenced_pki_paths {
            assert!(
                rendered.contains_key(path.as_str()),
                "control-plane arg references unrendered PKI file {path}"
            );
        }
    }

    #[test]
    fn pki_file_closure_modes_and_contents_follow_source_secrets() {
        let out = provision_control_plane(&cfg(true), &secrets(), "talos").unwrap();
        let rendered = rendered_by_path(&out);

        for spec in CONTROL_PLANE_PKI_SECRET_FILES {
            let path = format!("{PKI_DIR}/{}", spec.relative_path);
            let file = rendered
                .get(path.as_str())
                .unwrap_or_else(|| panic!("missing rendered PKI file {path}"));
            assert_eq!(file.mode, spec.mode, "{path} mode");
            assert_eq!(
                file.contents,
                format!("PEM-{}", spec.secret_name).into_bytes(),
                "{path} source secret"
            );
        }
    }

    #[test]
    fn generated_k8s_pki_drives_provisioned_control_plane_files() {
        let secrets = generated_secrets();
        let out = provision_control_plane(&cfg(true), &secrets, "talos").unwrap();
        let rendered = rendered_by_path(&out);

        for spec in CONTROL_PLANE_PKI_SECRET_FILES {
            let path = format!("{PKI_DIR}/{}", spec.relative_path);
            let file = rendered
                .get(path.as_str())
                .unwrap_or_else(|| panic!("missing rendered generated PKI file {path}"));
            assert_eq!(
                file.contents.as_slice(),
                secrets.get(spec.secret_name).unwrap(),
                "{path}"
            );
            let body = String::from_utf8_lossy(&file.contents);
            assert!(
                body.contains("KUBEROS-MODEL-"),
                "{path} should contain generated model bytes"
            );
            assert!(
                !body.contains("PEM-") && !body.contains("VECTOR-"),
                "{path} should not contain fixture placeholder bytes"
            );
            assert_ne!(file.contents.as_slice(), b"CERT", "{path}");
            assert_ne!(file.contents.as_slice(), b"KEY", "{path}");
        }

        for (path, cert, key) in [
            (
                "/etc/kubernetes/controller-manager.conf",
                "controller-manager.crt",
                "controller-manager.key",
            ),
            (
                "/etc/kubernetes/scheduler.conf",
                "scheduler.crt",
                "scheduler.key",
            ),
            ("/etc/kubernetes/admin.conf", "admin.crt", "admin.key"),
        ] {
            let file = rendered
                .get(path)
                .unwrap_or_else(|| panic!("missing generated kubeconfig {path}"));
            assert_eq!(file.mode, FileMode::SECRET, "{path} mode");
            let body = String::from_utf8_lossy(&file.contents);
            assert_eq!(
                kubeconfig_field(&body, "certificate-authority-data"),
                kubeconfig_data(secrets.get("ca.crt").unwrap()),
                "{path} generated certificate authority data"
            );
            assert_eq!(
                kubeconfig_field(&body, "client-certificate-data"),
                kubeconfig_data(secrets.get(cert).unwrap()),
                "{path} generated client certificate"
            );
            assert_eq!(
                kubeconfig_field(&body, "client-key-data"),
                kubeconfig_data(secrets.get(key).unwrap()),
                "{path} generated client key"
            );
            assert!(
                !body.contains("KUBEROS-MODEL-"),
                "{path} should base64-encode generated model credentials instead of embedding raw model text"
            );
            assert!(
                !body.contains("client-certificate-data: CERT\n")
                    && !body.contains("client-key-data: KEY\n"),
                "{path} should not contain placeholder kubeconfig credentials"
            );
            assert!(
                !body.contains("PEM-") && !body.contains("VECTOR-"),
                "{path} should not contain fixture placeholder bytes"
            );
        }

        assert!(rendered.contains_key("/etc/kubernetes/manifests/kube-apiserver.yaml"));
    }

    #[test]
    fn kubeconfig_data_base64_encodes_opaque_bytes() {
        assert_eq!(kubeconfig_data(b"A\nB"), "QQpC");
        assert_eq!(kubeconfig_data(&[0x00, 0xff, b'\\']), "AP9c");
        assert_ne!(kubeconfig_data(b"A\nB"), "A\\nB");
    }

    #[test]
    fn generated_k8s_pki_kubeconfigs_embed_base64_credentials() {
        let secrets = generated_secrets();
        let out = provision_control_plane(&cfg(true), &secrets, "talos").unwrap();
        let rendered = rendered_by_path(&out);

        for (path, cert, key) in [
            (
                "/etc/kubernetes/controller-manager.conf",
                "controller-manager.crt",
                "controller-manager.key",
            ),
            (
                "/etc/kubernetes/scheduler.conf",
                "scheduler.crt",
                "scheduler.key",
            ),
            ("/etc/kubernetes/admin.conf", "admin.crt", "admin.key"),
        ] {
            let body = String::from_utf8_lossy(
                &rendered
                    .get(path)
                    .unwrap_or_else(|| panic!("missing generated kubeconfig {path}"))
                    .contents,
            );
            for (field, secret_name) in [
                ("certificate-authority-data", "ca.crt"),
                ("client-certificate-data", cert),
                ("client-key-data", key),
            ] {
                let value = kubeconfig_field(&body, field);
                assert_eq!(value, kubeconfig_data(secrets.get(secret_name).unwrap()));
                assert!(!value.contains("KUBEROS-MODEL-"));
                assert!(!value.contains("\\n"));
            }
            assert!(!body.contains("KUBEROS-MODEL-"));
            assert!(String::from_utf8_lossy(secrets.get(cert).unwrap()).contains("KUBEROS-MODEL-"));
            assert!(String::from_utf8_lossy(secrets.get(key).unwrap()).contains("KUBEROS-MODEL-"));
        }
    }

    #[test]
    fn model_pem_encoder_output_drives_provisioned_control_plane_files_and_base64_kubeconfigs() {
        let secrets = generated_pem_secrets();
        let out = provision_control_plane(&cfg(true), &secrets, "talos").unwrap();
        let rendered = rendered_by_path(&out);

        for spec in CONTROL_PLANE_PKI_SECRET_FILES {
            let path = format!("{PKI_DIR}/{}", spec.relative_path);
            let file = rendered
                .get(path.as_str())
                .unwrap_or_else(|| panic!("missing rendered PEM PKI file {path}"));
            assert_eq!(
                file.contents.as_slice(),
                secrets.get(spec.secret_name).unwrap(),
                "{path}"
            );
            let body = String::from_utf8_lossy(&file.contents);
            assert!(body.starts_with("-----BEGIN "), "{path}");
            assert!(!body.contains("KUBEROS-MODEL-"), "{path}");
            if spec.relative_path.ends_with(".crt") {
                assert!(body.contains("-----BEGIN CERTIFICATE-----"), "{path}");
            } else if spec.relative_path.ends_with(".key") {
                assert!(body.contains("-----BEGIN PRIVATE KEY-----"), "{path}");
            } else if spec.relative_path.ends_with(".pub") {
                assert!(body.contains("-----BEGIN PUBLIC KEY-----"), "{path}");
            }
        }

        for (path, cert, key) in [
            (
                "/etc/kubernetes/controller-manager.conf",
                "controller-manager.crt",
                "controller-manager.key",
            ),
            (
                "/etc/kubernetes/scheduler.conf",
                "scheduler.crt",
                "scheduler.key",
            ),
            ("/etc/kubernetes/admin.conf", "admin.crt", "admin.key"),
        ] {
            let body = String::from_utf8_lossy(
                &rendered
                    .get(path)
                    .unwrap_or_else(|| panic!("missing PEM-backed kubeconfig {path}"))
                    .contents,
            );
            for (field, secret_name) in [
                ("certificate-authority-data", "ca.crt"),
                ("client-certificate-data", cert),
                ("client-key-data", key),
            ] {
                let value = kubeconfig_field(&body, field);
                assert_eq!(value, kubeconfig_data(secrets.get(secret_name).unwrap()));
                assert!(!value.contains("-----BEGIN"));
                assert!(!value.contains("KUBEROS-MODEL-"));
            }
            assert!(!body.contains("-----BEGIN"));
            assert!(!body.contains("KUBEROS-MODEL-"));
        }
    }

    #[test]
    fn provision_flushes_through_sink() {
        let out = provision_control_plane(&cfg(true), &secrets(), "talos").unwrap();
        let n = out.len();
        let mut sink = InMemoryFileSink::new();
        out.flush(&mut sink).unwrap();
        assert_eq!(sink.count(), n);
        let apiserver = sink
            .get("/etc/kubernetes/manifests/kube-apiserver.yaml")
            .unwrap();
        assert!(String::from_utf8_lossy(&apiserver.contents).contains("kind: Pod"));
    }

    #[test]
    fn ca_cert_contents_come_from_secrets() {
        let out = provision_control_plane(&cfg(true), &secrets(), "talos").unwrap();
        let ca = out
            .files()
            .iter()
            .find(|f| f.path == "/etc/kubernetes/pki/ca.crt")
            .unwrap();
        assert_eq!(ca.contents, b"PEM-ca.crt".to_vec());
    }

    #[test]
    fn admin_conf_references_cluster_endpoint() {
        let out = provision_control_plane(&cfg(true), &secrets(), "talos").unwrap();
        let admin = out
            .files()
            .iter()
            .find(|f| f.path == "/etc/kubernetes/admin.conf")
            .unwrap();
        let body = String::from_utf8_lossy(&admin.contents);
        assert!(body.contains("server: https://api.example.com:6443"));
        assert!(body.contains("current-context: admin@talos"));
    }
}
