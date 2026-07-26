//! # talos-runtime-cri
//!
//! An in-memory model of the Talos container-runtime integration layer.
//!
//! This crate mirrors the parts of `siderolabs/talos` that sit between machined
//! and containerd:
//!
//! * [`namespace`] — containerd namespaces (`system`, `k8s.io`).
//! * [`image`] — OCI image reference parsing and the pull/unpack lifecycle.
//! * [`oci_spec`] — construction of the OCI runtime `config.json`.
//! * [`task`] — the running-task (process) state machine.
//! * [`container`] — static container metadata tying the above together.
//! * [`client`] — an in-memory containerd-client model that creates, runs and
//!   deletes containers within a namespace, modeling the trait boundary that in
//!   the real system would talk to the containerd gRPC API.
//! * [`address`] — containerd socket addresses and a dial registry modeling the
//!   `/run/containerd` and `/run/system` endpoints.
//! * [`cri`] — the kubelet-facing CRI `RuntimeService`/`ImageService` API with
//!   the pod-sandbox/container lifecycle.
//! * [`image_cache`] — source-guided image-cache runtime/mount orchestration.
//!
//! Every kernel/containerd boundary is modeled as an in-memory store so the
//! logic (validation, state machines, lifecycle ordering) is fully testable
//! offline. The crate is plain `std` and depends only on workspace crates.

// This crate is an in-memory model with small, infallible-to-misuse accessors
// and builder methods. The following pedantic lints fire pervasively on that
// shape and add doc/attribute noise without improving the API, so they are
// allowed crate-wide rather than annotated on dozens of methods individually:
//   * `must_use_candidate` / `return_self_not_must_use` — builders and pure
//     accessors; ignoring a return value here is harmless, not a bug.
//   * `missing_errors_doc` — every fallible method returns `os_kernel::Result`
//     with a self-describing `Error`; per-method `# Errors` prose is redundant.
//   * `module_name_repetitions` — re-exported types intentionally keep their
//     descriptive `Cri*`/`Containerd*` names for the public prelude.
#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]

pub mod address;
pub mod client;
pub mod container;
pub mod cri;
pub mod image;
pub mod image_cache;
pub mod namespace;
pub mod oci_spec;
pub mod task;

pub use address::{Connection, ContainerdAddress, DialRegistry, Scheme};
pub use client::{ContainerdClient, InMemoryContainerd, RunResult};
pub use container::{Container, ContainerStatus};
pub use cri::{
    CRI_NAMESPACE, CriContainer, CriContainerConfig, CriContainerState, CriRuntime, ImageService,
    MACHINE_CONFIG_ACTIVE_ID, MACHINE_CONFIG_NAMESPACE, MACHINE_CONFIG_TYPE, PodSandbox,
    PodSandboxConfig, PodSandboxState, REGISTRIES_CONFIG_CONTROLLER_NAME, REGISTRIES_CONFIG_ID,
    REGISTRIES_CONFIG_TYPE, RegistriesConfigController, RegistriesConfigResource,
    RegistriesConfigSpec, RegistryAuthConfig, RegistryBuilderError, RegistryClientIdentity,
    RegistryConfigOption, RegistryEndpointConfig, RegistryMirrorConfig, RegistryTlsConfig,
    RuntimeService, apply_registries_config_controller_inputs_to_context,
    apply_registries_config_controller_inputs_to_state,
    apply_registries_config_controller_to_state, apply_registries_config_to_context,
    apply_registries_config_to_state, image_cache_ready_from_context, image_cache_ready_from_state,
    machine_config_contents_from_context, machine_config_key,
    machine_config_registries_from_context, machine_config_registries_from_state,
    poll_registry_builder, reconcile_registries_config, registries_config_controller_spec,
    registries_config_from_machine_config_contents, registries_config_key,
    registry_builder_from_state, registryd_mirror_endpoint, watch_registries_config,
};
pub use image::{Descriptor, Image, ImageRef, ImageState, Manifest};
pub use image_cache::{
    IMAGE_CACHE_CONFIG_ID, IMAGE_CACHE_CONFIG_TYPE, IMAGE_CACHE_CONTROLLER_NAME,
    IMAGE_CACHE_COPY_STATE_ID, IMAGE_CACHE_COPY_STATE_TYPE, IMAGE_CACHE_DISK_MOUNT_POINT,
    IMAGE_CACHE_DISK_VOLUME_ID, IMAGE_CACHE_ISO_MOUNT_POINT, IMAGE_CACHE_ISO_VOLUME_ID,
    IMAGE_CACHE_NAMESPACE, ImageCacheConfig, ImageCacheConfigController, ImageCacheConfigResource,
    ImageCacheCopyError, ImageCacheCopyExecutionStatus, ImageCacheCopyGate, ImageCacheCopyPlan,
    ImageCacheCopyReport, ImageCacheCopyRuntimeAdapter, ImageCacheCopyRuntimeEnvironment,
    ImageCacheCopyState, ImageCacheCopyStateResource, ImageCacheCopyStatus,
    ImageCacheCosiController, ImageCacheFinalizerAction, ImageCacheFinalizerOperation,
    ImageCacheMountRequestPlan, ImageCacheReconcileInput, ImageCacheRuntimePlan, ImageCacheStatus,
    ImageCacheVolumeConfigResource, ImageCacheVolumeConfigSpec, ImageCacheVolumeMountSpec,
    ImageCacheVolumeProvisioningSpec, MAX_IMAGE_CACHE_SIZE_BYTES, MIN_IMAGE_CACHE_SIZE_BYTES,
    REGISTRYD_HEALTH_PATH, REGISTRYD_HEALTH_URL, REGISTRYD_LISTEN_ADDRESS, REGISTRYD_SERVICE_ID,
    RegistrydAction, RegistrydContentResponse, RegistrydHealthProbe, RegistrydHealthService,
    RegistrydHttpResponse, RegistrydRuntimeAdapter, RegistrydRuntimeRootSkip,
    RegistrydRuntimeService, RegistrydServiceError, RegistrydServiceExecutionStatus,
    RegistrydServiceManager, RegistrydServiceReport, RegistrydSourceRequestHeaders, RegistrydState,
    SourceBlockVolumeType, V1ALPHA1_NAMESPACE, V1ALPHA1_SERVICE_TYPE, V1Alpha1ServiceResource,
    V1Alpha1ServiceSpec, VOLUME_CONFIG_TYPE, apply_image_cache_copy_report_to_state,
    apply_image_cache_plan_to_state, execute_image_cache_copy_plan, image_cache_config_key,
    image_cache_copy_done_from_state, image_cache_copy_state_key,
    image_cache_local_enabled_from_machine_config_contents, image_cache_mount_status_id,
    image_cache_volume_config_key, image_cache_volume_configs_from_machine_config_contents,
    machine_config_kind, registryd_health_url, registryd_http_last_modified_value,
    registryd_service_key, registryd_service_kind, volume_config_kind,
};
pub use namespace::Namespace;
pub use oci_spec::{LinuxNamespace, LinuxResources, Mount, OciSpec, Process};
pub use task::{ExecProcess, ExitStatus, Signal, Task, TaskState};
