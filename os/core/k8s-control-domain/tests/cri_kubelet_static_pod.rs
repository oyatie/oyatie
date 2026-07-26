use os_kernel::address::NodeAddress;
use os_k8s_control_domain::static_pod_controller::STATIC_POD_PATH as CONTROL_STATIC_POD_PATH;
use os_k8s_control_domain::{
    ComponentSpec, ControlPlaneComponent, InMemoryFileSink, StaticPodController,
};
use os_kubelet_domain::{
    KubeletConfig, KubeletSpec, Nodename, config::DEFAULT_RUNTIME_ENDPOINT,
    spec::STATIC_POD_PATH as KUBELET_STATIC_POD_PATH,
};
use os_runtime_cri_domain::address::{CRI_CONTAINERD_ADDRESS, SYSTEM_CONTAINERD_ADDRESS};
use os_runtime_cri_domain::{
    ContainerdAddress, CriContainerConfig, CriContainerState, CriRuntime, ImageRef, ImageService,
    PodSandboxConfig, PodSandboxState, RuntimeService,
};

const APISERVER_IMAGE: &str = "registry.k8s.io/kube-apiserver:v1.30.0";
const APISERVER_NAME: &str = "kube-apiserver";
const CONTROL_PLANE_NS: &str = "kube-system";
const NODE_NAME: &str = "cp-1";

fn kubelet_spec() -> KubeletSpec {
    let cfg = KubeletConfig::with_dns_from_service_cidr("10.96.0.0/12").unwrap();
    let node = Nodename::new(NODE_NAME).unwrap();
    KubeletSpec::render(
        &cfg,
        &node,
        &[NodeAddress::parse_v4("10.0.0.10").unwrap()],
        &[],
    )
    .unwrap()
}

fn apiserver_spec() -> ComponentSpec {
    ComponentSpec::new(ControlPlaneComponent::ApiServer, APISERVER_IMAGE)
        .unwrap()
        .with_arg("--secure-port=6443")
        .unwrap()
}

#[test]
fn kubelet_static_pod_path_matches_controller_manifest_path() {
    let spec = kubelet_spec();
    assert_eq!(
        spec.flag_value("pod-manifest-path"),
        Some(KUBELET_STATIC_POD_PATH)
    );
    assert_eq!(KUBELET_STATIC_POD_PATH, CONTROL_STATIC_POD_PATH);
    assert!(
        ControlPlaneComponent::ApiServer
            .manifest_path()
            .starts_with(CONTROL_STATIC_POD_PATH)
    );

    let mut controller = StaticPodController::new();
    let output = controller.reconcile(&[apiserver_spec()]).unwrap().output;
    let mut sink = InMemoryFileSink::new();
    output.flush(&mut sink).unwrap();
    assert!(
        sink.get("/etc/kubernetes/manifests/kube-apiserver.yaml")
            .is_some()
    );
}

#[test]
fn kubelet_uses_cri_containerd_endpoint() {
    let spec = kubelet_spec();
    assert_eq!(
        spec.flag_value("container-runtime-endpoint"),
        Some(DEFAULT_RUNTIME_ENDPOINT)
    );
    let parsed = ContainerdAddress::parse(spec.runtime_endpoint.as_str()).unwrap();
    assert_eq!(parsed, ContainerdAddress::cri());
    assert_eq!(parsed.endpoint, CRI_CONTAINERD_ADDRESS);
    assert_ne!(parsed, ContainerdAddress::system());
}

#[test]
fn containerd_kubelet_resource_markers_match_boot_service_contract() {
    let spec = kubelet_spec();
    assert_eq!(
        spec.flag_value("container-runtime-endpoint"),
        Some(DEFAULT_RUNTIME_ENDPOINT)
    );
    let parsed = ContainerdAddress::parse(spec.runtime_endpoint.as_str()).unwrap();
    assert_eq!(parsed, ContainerdAddress::cri());
    assert_eq!(parsed.endpoint, CRI_CONTAINERD_ADDRESS);
    assert_ne!(parsed.endpoint, SYSTEM_CONTAINERD_ADDRESS);
    assert_ne!(parsed, ContainerdAddress::system());

    assert_eq!(
        spec.flag_value("pod-manifest-path"),
        Some(CONTROL_STATIC_POD_PATH)
    );
    assert_eq!(KUBELET_STATIC_POD_PATH, CONTROL_STATIC_POD_PATH);
    assert!(
        ControlPlaneComponent::ApiServer
            .manifest_path()
            .starts_with(CONTROL_STATIC_POD_PATH)
    );
}

#[test]
fn static_pod_manifest_can_drive_cri_pod_lifecycle() {
    let mut controller = StaticPodController::new();
    let output = controller.reconcile(&[apiserver_spec()]).unwrap().output;
    let manifest = output
        .get("/etc/kubernetes/manifests/kube-apiserver.yaml")
        .unwrap()
        .as_str()
        .unwrap();
    assert!(manifest.contains("hostNetwork: true"));
    assert!(manifest.contains("--secure-port=6443"));

    let mut runtime = CriRuntime::new();
    let image = ImageRef::parse(APISERVER_IMAGE).unwrap();
    runtime.pull_image(&image).unwrap();
    let sandbox_id = runtime
        .run_pod_sandbox(
            PodSandboxConfig::new(
                APISERVER_NAME,
                CONTROL_PLANE_NS,
                format!("{APISERVER_NAME}-{NODE_NAME}"),
            )
            .unwrap()
            .with_host_network(),
        )
        .unwrap();
    let container_id = runtime
        .create_container(
            &sandbox_id,
            CriContainerConfig::new(APISERVER_NAME, image, vec![APISERVER_NAME.to_string()])
                .unwrap()
                .with_args(vec!["--secure-port=6443".to_string()]),
        )
        .unwrap();
    runtime.start_container(&container_id).unwrap();

    assert_eq!(
        runtime.pod_sandbox_status(&sandbox_id).unwrap().state,
        PodSandboxState::Ready
    );
    assert_eq!(
        runtime.container_status(&container_id).unwrap().state,
        CriContainerState::Running
    );
}

#[test]
fn static_pod_requires_image_pull_before_container_create() {
    let mut runtime = CriRuntime::new();
    let sandbox_id = runtime
        .run_pod_sandbox(PodSandboxConfig::new(APISERVER_NAME, CONTROL_PLANE_NS, "uid-1").unwrap())
        .unwrap();
    let image = ImageRef::parse(APISERVER_IMAGE).unwrap();
    let err = runtime
        .create_container(
            &sandbox_id,
            CriContainerConfig::new(APISERVER_NAME, image, vec![APISERVER_NAME.to_string()])
                .unwrap(),
        )
        .unwrap_err();

    assert_eq!(err.kind(), "not_found");
}

#[test]
fn stopping_static_pod_sandbox_exits_running_container() {
    let mut runtime = CriRuntime::new();
    let image = ImageRef::parse(APISERVER_IMAGE).unwrap();
    runtime.pull_image(&image).unwrap();
    let sandbox_id = runtime
        .run_pod_sandbox(PodSandboxConfig::new(APISERVER_NAME, CONTROL_PLANE_NS, "uid-1").unwrap())
        .unwrap();
    let container_id = runtime
        .create_container(
            &sandbox_id,
            CriContainerConfig::new(APISERVER_NAME, image, vec![APISERVER_NAME.to_string()])
                .unwrap(),
        )
        .unwrap();
    runtime.start_container(&container_id).unwrap();

    runtime.stop_pod_sandbox(&sandbox_id).unwrap();

    assert_eq!(
        runtime.pod_sandbox_status(&sandbox_id).unwrap().state,
        PodSandboxState::NotReady
    );
    let status = runtime.container_status(&container_id).unwrap();
    assert_eq!(status.state, CriContainerState::Exited);
    assert_eq!(status.exit_code, Some(137));
}
