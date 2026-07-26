//! Static-pod launch wiring against the kubelet-facing CRI model.
//!
//! Talos renders control-plane components as static pod manifests, then kubelet
//! observes those manifests and drives containerd through the CRI runtime
//! service. This module models that seam explicitly: a [`StaticPod`] is
//! translated into a CRI pod sandbox plus one container per manifest container,
//! images are pulled before container creation, and pod phase transitions mirror
//! kubelet progress.

use crate::config::NodeName;
use crate::error::{K8sError, Result};
use crate::static_pod::{StaticPod, StaticPodPhase};
use os_runtime_cri_domain::{
    CriContainerConfig, ImageRef, ImageService, PodSandboxConfig, RuntimeService,
};

/// Stable Kubernetes/CRI pod labels used by kubelet-created sandboxes.
pub const POD_NAME_LABEL: &str = "io.kubernetes.pod.name";
/// Stable Kubernetes/CRI pod namespace label.
pub const POD_NAMESPACE_LABEL: &str = "io.kubernetes.pod.namespace";
/// Stable Kubernetes/CRI pod UID label.
pub const POD_UID_LABEL: &str = "io.kubernetes.pod.uid";
/// Marker label for the static-pod path rather than an API-scheduled pod.
pub const STATIC_POD_LABEL: &str = "io.kubernetes.pod.static";

/// Result of launching one static pod through the modeled CRI service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticPodLaunchReport {
    /// Pod name from the static-pod manifest.
    pub pod_name: String,
    /// Kubernetes namespace from the static-pod manifest.
    pub namespace: String,
    /// Deterministic mirror-pod UID/name (`<pod>-<node>` in this model).
    pub pod_uid: String,
    /// Runtime-assigned pod sandbox id.
    pub sandbox_id: String,
    /// Runtime-assigned container ids in manifest order.
    pub container_ids: Vec<String>,
    /// Canonical image refs pulled for the containers in manifest order.
    pub images: Vec<String>,
    /// Whether any manifest container requested host networking.
    pub host_network: bool,
}

/// Launch a batch of static pods through the CRI runtime in manifest order.
pub fn launch_static_pods_on_cri<R>(
    runtime: &mut R,
    pods: &mut [StaticPod],
    node_name: &NodeName,
) -> Result<Vec<StaticPodLaunchReport>>
where
    R: RuntimeService + ImageService,
{
    let mut reports = Vec::with_capacity(pods.len());
    for pod in pods {
        reports.push(launch_static_pod_on_cri(runtime, pod, node_name)?);
    }
    Ok(reports)
}

/// Launch one static pod through the CRI runtime.
pub fn launch_static_pod_on_cri<R>(
    runtime: &mut R,
    pod: &mut StaticPod,
    node_name: &NodeName,
) -> Result<StaticPodLaunchReport>
where
    R: RuntimeService + ImageService,
{
    if pod.containers.is_empty() {
        return Err(K8sError::Render(format!(
            "static pod {} has no containers",
            pod.name
        )));
    }
    if !pod.phase.can_transition_to(StaticPodPhase::Creating) {
        return Err(K8sError::EtcdState(format!(
            "illegal static pod transition {:?} -> {:?}",
            pod.phase,
            StaticPodPhase::Creating
        )));
    }

    let planned_containers = pod
        .containers
        .iter()
        .map(|container| {
            let (entrypoint, args) = container.command.split_first().ok_or_else(|| {
                K8sError::Render(format!(
                    "static pod {} container {} has no command",
                    pod.name, container.name
                ))
            })?;
            let image = ImageRef::parse(&container.image)
                .map_err(|err| runtime_error("parse static pod image", err))?;
            Ok(PlannedContainer {
                name: container.name.clone(),
                image,
                command: vec![entrypoint.clone()],
                args: args.to_vec(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let pod_uid = pod.mirror_pod_name(node_name.as_str());
    let host_network = pod
        .containers
        .iter()
        .any(|container| container.host_network);
    let mut sandbox = PodSandboxConfig::new(&pod.name, &pod.namespace, &pod_uid)
        .map_err(|err| runtime_error("build pod sandbox config", err))?
        .with_label(POD_NAME_LABEL, pod.name.clone())
        .with_label(POD_NAMESPACE_LABEL, pod.namespace.clone())
        .with_label(POD_UID_LABEL, pod_uid.clone())
        .with_label(STATIC_POD_LABEL, "true");
    if host_network {
        sandbox = sandbox.with_host_network();
    }

    let sandbox_id = runtime
        .run_pod_sandbox(sandbox)
        .map_err(|err| runtime_error("run pod sandbox", err))?;

    let mut container_ids = Vec::with_capacity(planned_containers.len());
    let mut images = Vec::with_capacity(planned_containers.len());
    for planned in planned_containers {
        let canonical = runtime
            .pull_image(&planned.image)
            .map_err(|err| runtime_error("pull static pod image", err))?;
        let config = CriContainerConfig::new(&planned.name, planned.image, planned.command)
            .map_err(|err| runtime_error("build CRI container config", err))?
            .with_args(planned.args);
        let container_id = runtime
            .create_container(&sandbox_id, config)
            .map_err(|err| runtime_error("create static pod container", err))?;
        images.push(canonical);
        container_ids.push(container_id);
    }

    pod.advance_to(StaticPodPhase::Creating)?;
    for container_id in &container_ids {
        runtime
            .start_container(container_id)
            .map_err(|err| runtime_error("start static pod container", err))?;
    }
    pod.advance_to(StaticPodPhase::Running)?;

    Ok(StaticPodLaunchReport {
        pod_name: pod.name.clone(),
        namespace: pod.namespace.clone(),
        pod_uid,
        sandbox_id,
        container_ids,
        images,
        host_network,
    })
}

struct PlannedContainer {
    name: String,
    image: ImageRef,
    command: Vec<String>,
    args: Vec<String>,
}

fn runtime_error(context: &str, err: os_kernel::Error) -> K8sError {
    K8sError::Bootstrap(format!("{context}: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ClusterEndpoint, K8sConfig};
    use crate::control_plane::ControlPlaneConfig;
    use crate::secrets::{K8sSecrets, REQUIRED_SECRETS};
    use crate::static_pod::Container;
    use os_runtime_cri_domain::{CriContainerState, CriRuntime, PodSandboxState};

    fn node_name() -> NodeName {
        NodeName::new("cp-1").unwrap()
    }

    fn complete_secrets() -> K8sSecrets {
        let mut secrets = K8sSecrets::new();
        for name in REQUIRED_SECRETS {
            secrets.insert(*name, b"pem".to_vec());
        }
        secrets
    }

    fn control_plane_config() -> ControlPlaneConfig {
        ControlPlaneConfig::new(K8sConfig {
            node_name: node_name(),
            cluster_domain: "cluster.local".into(),
            pod_cidrs: vec!["10.244.0.0/16".into()],
            service_cidrs: vec!["10.96.0.0/12".into()],
            endpoint: ClusterEndpoint::new("api.example.com", 6443).unwrap(),
            version: "1.30.0".into(),
            control_plane: true,
        })
        .unwrap()
    }

    #[test]
    fn launches_control_plane_static_pods_through_cri_in_order() {
        let mut runtime = CriRuntime::new();
        let mut pods = control_plane_config()
            .render_all(&complete_secrets())
            .unwrap();
        let reports = launch_static_pods_on_cri(&mut runtime, &mut pods, &node_name()).unwrap();

        assert_eq!(reports.len(), 3);
        assert_eq!(
            reports
                .iter()
                .map(|report| report.pod_name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "kube-apiserver",
                "kube-controller-manager",
                "kube-scheduler"
            ]
        );
        assert_eq!(runtime.list_pod_sandboxes().len(), 3);
        assert!(
            reports
                .iter()
                .all(|report| report.namespace == crate::CONTROL_PLANE_NAMESPACE)
        );
        assert!(pods.iter().all(|pod| pod.phase == StaticPodPhase::Running));

        for report in &reports {
            let sandbox = runtime.pod_sandbox_status(&report.sandbox_id).unwrap();
            assert_eq!(sandbox.state, PodSandboxState::Ready);
            assert_eq!(
                sandbox
                    .config
                    .labels
                    .get(POD_NAME_LABEL)
                    .map(String::as_str),
                Some(report.pod_name.as_str())
            );
            assert_eq!(
                sandbox.config.labels.get(POD_UID_LABEL).map(String::as_str),
                Some(report.pod_uid.as_str())
            );
            assert_eq!(
                sandbox
                    .config
                    .labels
                    .get(STATIC_POD_LABEL)
                    .map(String::as_str),
                Some("true")
            );
            assert_eq!(report.container_ids.len(), 1);
            let container = runtime.container_status(&report.container_ids[0]).unwrap();
            assert_eq!(container.state, CriContainerState::Running);
            assert_eq!(
                container.config.args,
                pods_by_name(&pods, &report.pod_name).command_args()
            );
        }
    }

    #[test]
    fn preserves_host_network_and_canonicalizes_images() {
        let mut runtime = CriRuntime::new();
        let mut pod = StaticPod {
            name: "kube-scheduler".into(),
            namespace: crate::CONTROL_PLANE_NAMESPACE.into(),
            containers: vec![Container {
                name: "kube-scheduler".into(),
                image: "registry.k8s.io/kube-scheduler:v1.30.0".into(),
                command: vec!["kube-scheduler".into(), "--leader-elect=true".into()],
                host_network: false,
            }],
            phase: StaticPodPhase::Pending,
        };

        let report = launch_static_pod_on_cri(&mut runtime, &mut pod, &node_name()).unwrap();

        assert!(!report.host_network);
        assert_eq!(
            report.images,
            vec!["registry.k8s.io/kube-scheduler:v1.30.0"]
        );
        let sandbox = runtime.pod_sandbox_status(&report.sandbox_id).unwrap();
        assert!(!sandbox.config.host_network);
        assert_eq!(report.pod_uid, "kube-scheduler-cp-1");
    }

    #[test]
    fn rejects_empty_container_command_before_phase_advance() {
        let mut runtime = CriRuntime::new();
        let mut pod = StaticPod {
            name: "bad".into(),
            namespace: crate::CONTROL_PLANE_NAMESPACE.into(),
            containers: vec![Container {
                name: "bad".into(),
                image: "registry.k8s.io/bad:v1".into(),
                command: Vec::new(),
                host_network: true,
            }],
            phase: StaticPodPhase::Pending,
        };

        let err = launch_static_pod_on_cri(&mut runtime, &mut pod, &node_name()).unwrap_err();

        assert_eq!(err.kind(), "render");
        assert_eq!(pod.phase, StaticPodPhase::Pending);
        assert_eq!(runtime.list_pod_sandboxes().len(), 0);
    }

    #[test]
    fn rejects_terminal_phase_before_runtime_side_effects() {
        let mut runtime = CriRuntime::new();
        let mut pod = StaticPod::control_plane(
            "kube-apiserver",
            "registry.k8s.io/kube-apiserver:v1.30.0",
            vec!["kube-apiserver".into()],
        )
        .unwrap();
        pod.advance_to(StaticPodPhase::Creating).unwrap();
        pod.advance_to(StaticPodPhase::Running).unwrap();
        pod.advance_to(StaticPodPhase::Succeeded).unwrap();

        let err = launch_static_pod_on_cri(&mut runtime, &mut pod, &node_name()).unwrap_err();

        assert_eq!(err.kind(), "etcd_state");
        assert_eq!(runtime.list_pod_sandboxes().len(), 0);
    }

    trait TestPodExt {
        fn command_args(&self) -> Vec<String>;
    }

    impl TestPodExt for StaticPod {
        fn command_args(&self) -> Vec<String> {
            self.containers[0].command[1..].to_vec()
        }
    }

    fn pods_by_name<'a>(pods: &'a [StaticPod], name: &str) -> &'a StaticPod {
        pods.iter().find(|pod| pod.name == name).unwrap()
    }
}
