//! The kubelet-facing CRI (Container Runtime Interface) API.
//!
//! Distinct from the lower-level [`crate::client`] containerd client: this models
//! the gRPC API that the kubelet calls on the CRI plugin (`RuntimeService` and
//! `ImageService`). Talos configures the CRI plugin inside containerd; the
//! kubelet drives pod sandboxes and containers through it.
//!
//! The CRI lifecycle is pod-centric:
//!
//! * `RunPodSandbox` creates the pod's network/ipc sandbox (the `pause` infra
//!   container) and returns a sandbox id.
//! * `CreateContainer` creates a container *within* a sandbox.
//! * `StartContainer` / `StopContainer` / `RemoveContainer` drive its lifecycle.
//! * `StopPodSandbox` / `RemovePodSandbox` tear the pod down; a sandbox cannot be
//!   removed while it still has containers.
//!
//! Every boundary is modeled as the [`RuntimeService`] / [`ImageService`] traits
//! with an in-memory [`CriRuntime`] implementation enforcing those invariants.

use crate::{
    image::ImageRef,
    image_cache::{
        ImageCacheConfigResource, ImageCacheStatus, REGISTRYD_LISTEN_ADDRESS,
        image_cache_config_key,
    },
};
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
};
use os_kernel::{
    ResourceId,
    error::{Error, Result},
};
use os_cosi_domain::{
    Controller, ControllerError, Event, EventKind, Input, Metadata, Output, ReconcileContext,
    ReconcileResult, Resource, ResourceKind, Spec, State,
};

/// Talos CRI resource namespace.
pub const CRI_NAMESPACE: &str = "cri";

/// Talos CRI `RegistriesConfig` resource type.
pub const REGISTRIES_CONFIG_TYPE: &str = "RegistryConfigs.cri.talos.dev";

/// Talos CRI `RegistriesConfig` singleton id.
pub const REGISTRIES_CONFIG_ID: &str = "registries";

/// Source controller name for `RegistriesConfigController`.
pub const REGISTRIES_CONFIG_CONTROLLER_NAME: &str = "cri.RegistriesConfigController";

/// Talos config resource namespace used by the source controller.
pub const MACHINE_CONFIG_NAMESPACE: &str = "config";

/// Talos active machine config resource type.
pub const MACHINE_CONFIG_TYPE: &str = "MachineConfigs.config.talos.dev";

/// Talos active machine config singleton id.
pub const MACHINE_CONFIG_ACTIVE_ID: &str = "v1alpha1";

/// Source `RegistryBuilder` option hook.
pub type RegistryConfigOption = fn(&mut RegistriesConfigSpec);

/// A single registry mirror endpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryEndpointConfig {
    /// Endpoint URL.
    pub endpoint: String,
    /// Whether the endpoint uses source `overridePath`.
    pub override_path: bool,
}

impl RegistryEndpointConfig {
    /// Source `Endpoint()` accessor.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Source `OverridePath()` accessor.
    pub fn override_path(&self) -> bool {
        self.override_path
    }
}

/// Mirror configuration for one registry host.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryMirrorConfig {
    /// Ordered mirror endpoints.
    pub endpoints: Vec<RegistryEndpointConfig>,
    /// Whether to skip fallback to the upstream registry.
    pub skip_fallback: bool,
}

impl RegistryMirrorConfig {
    /// Source `SkipFallback()` accessor.
    pub fn skip_fallback(&self) -> bool {
        self.skip_fallback
    }

    /// Source `Endpoints()` accessor.
    pub fn endpoints(&self) -> Vec<RegistryEndpointConfig> {
        self.endpoints.clone()
    }
}

/// Authentication configuration for one registry host.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryAuthConfig {
    /// Basic auth username.
    pub username: String,
    /// Basic auth password.
    pub password: String,
    /// Precomputed auth value.
    pub auth: String,
    /// Identity token.
    pub identity_token: String,
}

impl RegistryAuthConfig {
    /// Source `Username()` accessor.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Source `Password()` accessor.
    pub fn password(&self) -> &str {
        &self.password
    }

    /// Source `Auth()` accessor.
    pub fn auth(&self) -> &str {
        &self.auth
    }

    /// Source `IdentityToken()` accessor.
    pub fn identity_token(&self) -> &str {
        &self.identity_token
    }
}

/// TLS client identity bytes for one registry host.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryClientIdentity {
    /// PEM certificate bytes.
    pub cert: Vec<u8>,
    /// PEM private-key bytes.
    pub key: Vec<u8>,
}

/// TLS configuration for one registry host.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryTlsConfig {
    /// Optional PEM CA bundle bytes.
    pub ca: Vec<u8>,
    /// Optional PEM client identity.
    pub client_identity: Option<RegistryClientIdentity>,
    /// Whether TLS verification is disabled.
    pub insecure_skip_verify: bool,
}

impl RegistryTlsConfig {
    /// Source `ClientIdentity()` accessor.
    pub fn client_identity(&self) -> Option<&RegistryClientIdentity> {
        self.client_identity.as_ref()
    }

    /// Source `CA()` accessor.
    pub fn ca(&self) -> &[u8] {
        &self.ca
    }

    /// Source `InsecureSkipVerify()` accessor.
    pub fn insecure_skip_verify(&self) -> bool {
        self.insecure_skip_verify
    }
}

/// Source-shaped `RegistriesConfigSpec` model.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistriesConfigSpec {
    /// Registry host -> mirror config.
    pub mirrors: BTreeMap<String, RegistryMirrorConfig>,
    /// Registry host -> auth config.
    pub auths: BTreeMap<String, RegistryAuthConfig>,
    /// Registry host -> TLS config.
    pub tls: BTreeMap<String, RegistryTlsConfig>,
}

impl RegistriesConfigSpec {
    /// Source `Mirrors()` accessor.
    pub fn mirrors(&self) -> BTreeMap<String, RegistryMirrorConfig> {
        self.mirrors.clone()
    }

    /// Source `Auths()` accessor.
    pub fn auths(&self) -> BTreeMap<String, RegistryAuthConfig> {
        self.auths.clone()
    }

    /// Source `TLSs()` accessor.
    pub fn tls(&self) -> BTreeMap<String, RegistryTlsConfig> {
        self.tls.clone()
    }
}

/// Source registryd mirror endpoint injected by `RegistriesConfigController`.
pub fn registryd_mirror_endpoint() -> String {
    format!("http://{REGISTRYD_LISTEN_ADDRESS}")
}

/// Reconcile the CRI registries config output from machine config and image-cache readiness.
pub fn reconcile_registries_config(
    spec: &mut RegistriesConfigSpec,
    config: Option<&RegistriesConfigSpec>,
    image_cache_ready: bool,
) {
    spec.auths.clear();
    spec.mirrors.clear();
    spec.tls.clear();

    if let Some(config) = config {
        spec.mirrors = config.mirrors();
        spec.auths = config.auths();
        spec.tls = config.tls();
    }

    if image_cache_ready {
        spec.mirrors.entry("*".to_string()).or_default();

        let registryd_endpoint = RegistryEndpointConfig {
            endpoint: registryd_mirror_endpoint(),
            override_path: false,
        };
        for mirror in spec.mirrors.values_mut() {
            mirror.endpoints.insert(0, registryd_endpoint.clone());
        }
    }
}

/// COSI resource form of Talos's CRI `RegistriesConfig`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistriesConfigResource {
    meta: Metadata,
    /// Projected registry config spec.
    pub spec: RegistriesConfigSpec,
}

impl RegistriesConfigResource {
    /// Build the singleton registries config resource.
    pub fn new(spec: RegistriesConfigSpec) -> Self {
        RegistriesConfigResource {
            meta: registries_config_metadata(),
            spec,
        }
    }

    /// Kind descriptor for `RegistriesConfig`.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(CRI_NAMESPACE, REGISTRIES_CONFIG_TYPE)
    }

    /// Borrow the COSI metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.meta
    }

    /// Mutably borrow the COSI metadata.
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    /// Convert a type-erased same-kind COSI resource back into registries config.
    pub fn from_resource(resource: &dyn Resource) -> RegistryBuilderResult<Self> {
        if resource.resource_kind() != Self::kind() {
            return Err(RegistryBuilderError::MalformedRegistriesConfig {
                key: resource.metadata().key(),
                fingerprint: resource.spec_fingerprint(),
            });
        }

        let fingerprint = resource.spec_fingerprint();
        let spec = registries_config_from_fingerprint(&fingerprint).ok_or_else(|| {
            RegistryBuilderError::MalformedRegistriesConfig {
                key: resource.metadata().key(),
                fingerprint: fingerprint.clone(),
            }
        })?;

        Ok(RegistriesConfigResource {
            meta: resource.metadata().clone(),
            spec,
        })
    }
}

impl Resource for RegistriesConfigResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        registries_config_fingerprint(&self.spec)
    }

    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// Errors returned while evaluating CRI registry builder watches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryBuilderError {
    /// The requested COSI watch channel no longer exists.
    MissingWatch {
        /// Watched resource kind.
        kind: ResourceKind,
        /// Watch index returned by [`watch_registries_config`].
        index: usize,
    },
    /// The watch channel overran its bounded buffer.
    WatchOverrun {
        /// Watched resource kind.
        kind: ResourceKind,
        /// Watch index returned by [`watch_registries_config`].
        index: usize,
    },
    /// A same-kind singleton registry config could not be decoded.
    MalformedRegistriesConfig {
        /// Resource key.
        key: String,
        /// Resource fingerprint that failed to decode.
        fingerprint: String,
    },
    /// A registries config state create/update failed.
    StateWrite {
        /// Resource key.
        key: String,
        /// Store error message.
        message: String,
    },
    /// The optional image-cache input could not be decoded.
    MalformedImageCacheConfig {
        /// Resource key.
        key: String,
        /// Decode error message.
        message: String,
    },
    /// The active machine-config input could not be decoded.
    MalformedMachineConfig {
        /// Resource key.
        key: String,
        /// Decode error message.
        message: String,
    },
}

impl fmt::Display for RegistryBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryBuilderError::MissingWatch { kind, index } => {
                write!(f, "registry config watch {kind}#{index} is not registered")
            }
            RegistryBuilderError::WatchOverrun { kind, index } => {
                write!(f, "registry config watch {kind}#{index} overran its buffer")
            }
            RegistryBuilderError::MalformedRegistriesConfig { key, fingerprint } => write!(
                f,
                "registry config {key} has malformed fingerprint {fingerprint:?}"
            ),
            RegistryBuilderError::StateWrite { key, message } => {
                write!(f, "failed to write registry config {key}: {message}")
            }
            RegistryBuilderError::MalformedImageCacheConfig { key, message } => {
                write!(f, "image cache config {key} is malformed: {message}")
            }
            RegistryBuilderError::MalformedMachineConfig { key, message } => {
                write!(f, "machine config {key} is malformed: {message}")
            }
        }
    }
}

impl std::error::Error for RegistryBuilderError {}

/// Result alias for registry-builder watch evaluation.
pub type RegistryBuilderResult<T> = std::result::Result<T, RegistryBuilderError>;

/// Canonical COSI key for the CRI `RegistriesConfig` singleton.
pub fn registries_config_key() -> Result<String> {
    Ok(registries_config_metadata().key())
}

/// Canonical COSI key for the active Talos `MachineConfig` singleton.
pub fn machine_config_key() -> Result<String> {
    Ok(machine_config_metadata().key())
}

/// Source `RegistriesConfigController.Inputs()` / `Outputs()` declaration.
pub fn registries_config_controller_spec() -> Spec {
    Spec::new()
        .with_input(
            Input::weak(ResourceKind::new(
                MACHINE_CONFIG_NAMESPACE,
                MACHINE_CONFIG_TYPE,
            ))
            .with_id(MACHINE_CONFIG_ACTIVE_ID),
        )
        .with_input(Input::weak(ImageCacheConfigResource::kind()))
        .with_output(Output::exclusive(RegistriesConfigResource::kind()))
}

/// Source-shaped COSI controller for CRI `RegistriesConfig`.
#[derive(Debug, Default)]
pub struct RegistriesConfigController;

impl RegistriesConfigController {
    /// Build a stateless `RegistriesConfigController`.
    pub fn new() -> Self {
        Self
    }
}

impl Controller for RegistriesConfigController {
    fn name(&self) -> &str {
        REGISTRIES_CONFIG_CONTROLLER_NAME
    }

    fn spec(&self) -> Spec {
        registries_config_controller_spec()
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult {
        ctx.start_tracking_outputs();
        apply_registries_config_controller_inputs_to_context(ctx)
            .map_err(|err| ControllerError::Failed(err.to_string()))?;
        ctx.cleanup_outputs().map_err(ControllerError::Store)?;
        Ok(())
    }
}

/// Register a COSI watch for source `RegistryBuilder`.
pub fn watch_registries_config(state: &mut State, capacity: usize) -> usize {
    state.watch_kind(RegistriesConfigResource::kind(), capacity)
}

/// Register and immediately poll the current state for source `RegistryBuilder`.
pub fn registry_builder_from_state(
    state: &mut State,
    watch_capacity: usize,
    options: &[RegistryConfigOption],
) -> RegistryBuilderResult<Option<RegistriesConfigSpec>> {
    let watch_index = watch_registries_config(state, watch_capacity);
    poll_registry_builder(state, watch_index, options)
}

/// Poll a registered watch for source `RegistryBuilder` config availability.
pub fn poll_registry_builder(
    state: &mut State,
    watch_index: usize,
    options: &[RegistryConfigOption],
) -> RegistryBuilderResult<Option<RegistriesConfigSpec>> {
    let kind = RegistriesConfigResource::kind();
    let channel =
        state
            .watch_mut(&kind, watch_index)
            .ok_or_else(|| RegistryBuilderError::MissingWatch {
                kind: kind.clone(),
                index: watch_index,
            })?;

    if channel.is_overran() {
        return Err(RegistryBuilderError::WatchOverrun {
            kind,
            index: watch_index,
        });
    }

    let events = channel.drain();
    if channel.is_overran() {
        return Err(RegistryBuilderError::WatchOverrun {
            kind,
            index: watch_index,
        });
    }

    registry_builder_events_spec(&events, options)
}

/// Return whether the source `ImageCacheConfig` input is present and ready.
pub fn image_cache_ready_from_state(state: &State) -> RegistryBuilderResult<bool> {
    let key = image_cache_config_key().map_err(|err| {
        RegistryBuilderError::MalformedImageCacheConfig {
            key: crate::image_cache::IMAGE_CACHE_CONFIG_ID.to_string(),
            message: err.to_string(),
        }
    })?;

    let Some(resource) = state.get(&key) else {
        return Ok(false);
    };

    let resource = ImageCacheConfigResource::from_resource(resource.as_ref()).map_err(|err| {
        RegistryBuilderError::MalformedImageCacheConfig {
            key: key.clone(),
            message: err.to_string(),
        }
    })?;

    Ok(resource.spec.status == ImageCacheStatus::Ready)
}

/// Return whether the source `ImageCacheConfig` input is present and ready in a reconcile context.
pub fn image_cache_ready_from_context(ctx: &ReconcileContext<'_>) -> RegistryBuilderResult<bool> {
    let key = image_cache_config_key().map_err(|err| {
        RegistryBuilderError::MalformedImageCacheConfig {
            key: crate::image_cache::IMAGE_CACHE_CONFIG_ID.to_string(),
            message: err.to_string(),
        }
    })?;

    let Some(resource) = ctx.get(&key) else {
        return Ok(false);
    };

    let resource = ImageCacheConfigResource::from_resource(resource.as_ref()).map_err(|err| {
        RegistryBuilderError::MalformedImageCacheConfig {
            key: key.clone(),
            message: err.to_string(),
        }
    })?;

    Ok(resource.spec.status == ImageCacheStatus::Ready)
}

/// Decode the active Talos machine-config input into source registry settings.
pub fn machine_config_registries_from_state(
    state: &State,
) -> RegistryBuilderResult<Option<RegistriesConfigSpec>> {
    let key = machine_config_key_for_builder()?;

    let Some(resource) = state.get(&key) else {
        return Ok(None);
    };

    let contents = machine_config_contents_from_resource(resource.as_ref(), &key)?;
    registries_config_from_machine_config_contents(&contents)
        .map(Some)
        .map_err(|message| RegistryBuilderError::MalformedMachineConfig { key, message })
}

/// Decode active Talos machine-config contents from a reconcile context.
pub fn machine_config_contents_from_context(
    ctx: &ReconcileContext<'_>,
) -> RegistryBuilderResult<Option<String>> {
    let key = machine_config_key_for_builder()?;

    let Some(resource) = ctx.get(&key) else {
        return Ok(None);
    };

    machine_config_contents_from_resource(resource.as_ref(), &key).map(Some)
}

/// Decode active Talos machine-config input from a reconcile context.
pub fn machine_config_registries_from_context(
    ctx: &ReconcileContext<'_>,
) -> RegistryBuilderResult<Option<RegistriesConfigSpec>> {
    let key = machine_config_key_for_builder()?;
    let Some(contents) = machine_config_contents_from_context(ctx)? else {
        return Ok(None);
    };

    registries_config_from_machine_config_contents(&contents)
        .map(Some)
        .map_err(|message| RegistryBuilderError::MalformedMachineConfig { key, message })
}

/// Decode source `machine.registries` YAML into the controller's registry spec.
pub fn registries_config_from_machine_config_contents(
    contents: &str,
) -> std::result::Result<RegistriesConfigSpec, String> {
    let root = os_machine_config_domain::yaml::parse(contents).map_err(|err| err.to_string())?;
    let Some(machine) = root.get("machine") else {
        return Ok(RegistriesConfigSpec::default());
    };
    let machine = yaml_mapping(machine, "machine")?;
    let Some(registries) = machine.get("registries") else {
        return Ok(RegistriesConfigSpec::default());
    };
    let registries = yaml_mapping(registries, "machine.registries")?;

    let mut spec = RegistriesConfigSpec::default();

    if let Some(mirrors) = registries.get("mirrors") {
        let mirrors = yaml_mapping(mirrors, "machine.registries.mirrors")?;
        for (host, mirror) in mirrors {
            let mirror = yaml_mapping(mirror, &format!("machine.registries.mirrors.{host}"))?;
            let override_path = yaml_optional_bool(
                mirror.get("overridePath"),
                &format!("machine.registries.mirrors.{host}.overridePath"),
            )?
            .unwrap_or(false);
            let skip_fallback = yaml_optional_bool(
                mirror.get("skipFallback"),
                &format!("machine.registries.mirrors.{host}.skipFallback"),
            )?
            .unwrap_or(false);
            let endpoints = yaml_optional_string_sequence(
                mirror.get("endpoints"),
                &format!("machine.registries.mirrors.{host}.endpoints"),
            )?
            .into_iter()
            .map(|endpoint| RegistryEndpointConfig {
                endpoint,
                override_path,
            })
            .collect();

            spec.mirrors.insert(
                host.clone(),
                RegistryMirrorConfig {
                    endpoints,
                    skip_fallback,
                },
            );
        }
    }

    if let Some(configs) = registries.get("config") {
        let configs = yaml_mapping(configs, "machine.registries.config")?;
        for (host, config) in configs {
            let config = yaml_mapping(config, &format!("machine.registries.config.{host}"))?;

            if let Some(auth) = config.get("auth") {
                spec.auths.insert(
                    host.clone(),
                    registry_auth_from_yaml(
                        auth,
                        &format!("machine.registries.config.{host}.auth"),
                    )?,
                );
            }

            if let Some(tls) = config.get("tls") {
                spec.tls.insert(
                    host.clone(),
                    registry_tls_from_yaml(tls, &format!("machine.registries.config.{host}.tls"))?,
                );
            }
        }
    }

    Ok(spec)
}

/// Apply source `RegistriesConfigController` using COSI image-cache readiness input.
pub fn apply_registries_config_controller_to_state(
    state: &mut State,
    config: Option<&RegistriesConfigSpec>,
) -> RegistryBuilderResult<RegistriesConfigSpec> {
    let image_cache_ready = image_cache_ready_from_state(state)?;
    apply_registries_config_to_state(state, config, image_cache_ready)
}

/// Apply source `RegistriesConfigController` using both COSI controller inputs.
pub fn apply_registries_config_controller_inputs_to_state(
    state: &mut State,
) -> RegistryBuilderResult<RegistriesConfigSpec> {
    let config = machine_config_registries_from_state(state)?;
    let image_cache_ready = image_cache_ready_from_state(state)?;
    apply_registries_config_to_state(state, config.as_ref(), image_cache_ready)
}

/// Apply source `RegistriesConfigController` through a COSI reconcile context.
pub fn apply_registries_config_controller_inputs_to_context(
    ctx: &mut ReconcileContext<'_>,
) -> RegistryBuilderResult<RegistriesConfigSpec> {
    let config = machine_config_registries_from_context(ctx)?;
    let image_cache_ready = image_cache_ready_from_context(ctx)?;
    apply_registries_config_to_context(ctx, config.as_ref(), image_cache_ready)
}

/// Apply source `RegistriesConfigController` projection to the singleton COSI resource.
pub fn apply_registries_config_to_state(
    state: &mut State,
    config: Option<&RegistriesConfigSpec>,
    image_cache_ready: bool,
) -> RegistryBuilderResult<RegistriesConfigSpec> {
    let key = registries_config_key_for_builder()?;

    if let Some(resource) = state.get(&key) {
        let existing = RegistriesConfigResource::from_resource(resource.as_ref())?;
        let expected_version = existing.metadata().version();
        let metadata = existing.metadata().clone();
        let mut spec = existing.spec;
        reconcile_registries_config(&mut spec, config, image_cache_ready);

        let mut updated = RegistriesConfigResource::new(spec.clone());
        *updated.metadata_mut() = metadata;
        state
            .update(Box::new(updated), expected_version)
            .map_err(|err| RegistryBuilderError::StateWrite {
                key: key.clone(),
                message: err.to_string(),
            })?;

        return Ok(spec);
    }

    let mut spec = RegistriesConfigSpec::default();
    reconcile_registries_config(&mut spec, config, image_cache_ready);
    state
        .create(Box::new(RegistriesConfigResource::new(spec.clone())))
        .map_err(|err| RegistryBuilderError::StateWrite {
            key,
            message: err.to_string(),
        })?;

    Ok(spec)
}

/// Apply source `RegistriesConfigController` projection through a reconcile context.
pub fn apply_registries_config_to_context(
    ctx: &mut ReconcileContext<'_>,
    config: Option<&RegistriesConfigSpec>,
    image_cache_ready: bool,
) -> RegistryBuilderResult<RegistriesConfigSpec> {
    let key = registries_config_key_for_builder()?;

    if let Some(resource) = ctx.get(&key) {
        let existing = RegistriesConfigResource::from_resource(resource.as_ref())?;
        let expected_version = existing.metadata().version();
        let metadata = existing.metadata().clone();
        let mut spec = existing.spec;
        reconcile_registries_config(&mut spec, config, image_cache_ready);

        let mut updated = RegistriesConfigResource::new(spec.clone());
        *updated.metadata_mut() = metadata;
        ctx.update(Box::new(updated), expected_version)
            .map_err(|err| RegistryBuilderError::StateWrite {
                key: key.clone(),
                message: err.to_string(),
            })?;

        return Ok(spec);
    }

    let mut spec = RegistriesConfigSpec::default();
    reconcile_registries_config(&mut spec, config, image_cache_ready);
    ctx.create(Box::new(RegistriesConfigResource::new(spec.clone())))
        .map_err(|err| RegistryBuilderError::StateWrite {
            key,
            message: err.to_string(),
        })?;

    Ok(spec)
}

fn registry_builder_events_spec(
    events: &[Event],
    options: &[RegistryConfigOption],
) -> RegistryBuilderResult<Option<RegistriesConfigSpec>> {
    let config_key = registries_config_key_for_builder()?;

    for event in events {
        if !matches!(event.kind(), EventKind::Created | EventKind::Updated) {
            continue;
        }

        let Some(resource) = event.resource() else {
            continue;
        };
        if resource.metadata().key() != config_key {
            continue;
        }

        let mut spec = RegistriesConfigResource::from_resource(resource.as_ref())?.spec;
        for option in options {
            option(&mut spec);
        }
        return Ok(Some(spec));
    }

    Ok(None)
}

fn registries_config_key_for_builder() -> RegistryBuilderResult<String> {
    registries_config_key().map_err(|err| RegistryBuilderError::MalformedRegistriesConfig {
        key: REGISTRIES_CONFIG_ID.to_string(),
        fingerprint: err.to_string(),
    })
}

fn machine_config_key_for_builder() -> RegistryBuilderResult<String> {
    machine_config_key().map_err(|err| RegistryBuilderError::MalformedMachineConfig {
        key: MACHINE_CONFIG_ACTIVE_ID.to_string(),
        message: err.to_string(),
    })
}

fn registries_config_metadata() -> Metadata {
    Metadata::new(
        CRI_NAMESPACE,
        REGISTRIES_CONFIG_TYPE,
        ResourceId::new(REGISTRIES_CONFIG_ID)
            .expect("Talos registries config id is a valid COSI resource id"),
    )
}

fn machine_config_metadata() -> Metadata {
    Metadata::new(
        MACHINE_CONFIG_NAMESPACE,
        MACHINE_CONFIG_TYPE,
        ResourceId::new(MACHINE_CONFIG_ACTIVE_ID)
            .expect("Talos active machine config id is a valid COSI resource id"),
    )
}

fn machine_config_contents_from_resource(
    resource: &dyn Resource,
    key: &str,
) -> RegistryBuilderResult<String> {
    if resource.resource_kind() != ResourceKind::new(MACHINE_CONFIG_NAMESPACE, MACHINE_CONFIG_TYPE)
    {
        return Err(RegistryBuilderError::MalformedMachineConfig {
            key: key.to_string(),
            message: format!("unexpected kind {}", resource.resource_kind()),
        });
    }

    let fingerprint = resource.spec_fingerprint();
    let Some(contents_hex) = fingerprint.strip_prefix("contents=") else {
        return Err(RegistryBuilderError::MalformedMachineConfig {
            key: key.to_string(),
            message: format!("unexpected fingerprint {fingerprint:?}"),
        });
    };

    let bytes = parse_hex_bytes(contents_hex).map_err(|message| {
        RegistryBuilderError::MalformedMachineConfig {
            key: key.to_string(),
            message,
        }
    })?;

    String::from_utf8(bytes).map_err(|err| RegistryBuilderError::MalformedMachineConfig {
        key: key.to_string(),
        message: err.to_string(),
    })
}

fn yaml_mapping<'a>(
    value: &'a os_machine_config_domain::yaml::Yaml,
    field: &str,
) -> std::result::Result<&'a BTreeMap<String, os_machine_config_domain::yaml::Yaml>, String> {
    value
        .as_mapping()
        .ok_or_else(|| format!("{field} must be a mapping"))
}

fn yaml_optional_string_sequence(
    value: Option<&os_machine_config_domain::yaml::Yaml>,
    field: &str,
) -> std::result::Result<Vec<String>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let values = value
        .as_sequence()
        .ok_or_else(|| format!("{field} must be a sequence"))?;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{field}[{index}] must be a string"))
        })
        .collect()
}

fn yaml_optional_bool(
    value: Option<&os_machine_config_domain::yaml::Yaml>,
    field: &str,
) -> std::result::Result<Option<bool>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    value
        .as_bool()
        .map(Some)
        .ok_or_else(|| format!("{field} must be a boolean"))
}

fn yaml_optional_string(
    value: Option<&os_machine_config_domain::yaml::Yaml>,
    field: &str,
) -> std::result::Result<String, String> {
    let Some(value) = value else {
        return Ok(String::new());
    };
    value
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| format!("{field} must be a string"))
}

fn registry_auth_from_yaml(
    value: &os_machine_config_domain::yaml::Yaml,
    field: &str,
) -> std::result::Result<RegistryAuthConfig, String> {
    let auth = yaml_mapping(value, field)?;
    Ok(RegistryAuthConfig {
        username: yaml_optional_string(auth.get("username"), &format!("{field}.username"))?,
        password: yaml_optional_string(auth.get("password"), &format!("{field}.password"))?,
        auth: yaml_optional_string(auth.get("auth"), &format!("{field}.auth"))?,
        identity_token: yaml_optional_string(
            auth.get("identityToken"),
            &format!("{field}.identityToken"),
        )?,
    })
}

fn registry_tls_from_yaml(
    value: &os_machine_config_domain::yaml::Yaml,
    field: &str,
) -> std::result::Result<RegistryTlsConfig, String> {
    let tls = yaml_mapping(value, field)?;
    let client_identity = match tls.get("clientIdentity") {
        Some(identity) => {
            let identity = yaml_mapping(identity, &format!("{field}.clientIdentity"))?;
            Some(RegistryClientIdentity {
                cert: yaml_optional_base64_bytes(
                    identity.get("crt"),
                    &format!("{field}.clientIdentity.crt"),
                )?,
                key: yaml_optional_base64_bytes(
                    identity.get("key"),
                    &format!("{field}.clientIdentity.key"),
                )?,
            })
        }
        None => None,
    };

    Ok(RegistryTlsConfig {
        ca: yaml_optional_base64_bytes(tls.get("ca"), &format!("{field}.ca"))?,
        client_identity,
        insecure_skip_verify: yaml_optional_bool(
            tls.get("insecureSkipVerify"),
            &format!("{field}.insecureSkipVerify"),
        )?
        .unwrap_or(false),
    })
}

fn yaml_optional_base64_bytes(
    value: Option<&os_machine_config_domain::yaml::Yaml>,
    field: &str,
) -> std::result::Result<Vec<u8>, String> {
    let value = yaml_optional_string(value, field)?;
    if value.is_empty() {
        return Ok(Vec::new());
    }
    parse_base64_bytes(&value).map_err(|err| format!("{field}: {err}"))
}

fn parse_hex_bytes(s: &str) -> std::result::Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("hex byte string has odd length".to_string());
    }

    (0..s.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&s[index..index + 2], 16)
                .map_err(|_| format!("invalid hex byte {:?}", &s[index..index + 2]))
        })
        .collect()
}

fn parse_base64_bytes(s: &str) -> std::result::Result<Vec<u8>, String> {
    let non_padding_len = s.trim_end_matches('=').len();
    if s[non_padding_len..].chars().any(|c| c != '=') {
        return Err("base64 padding must be trailing".to_string());
    }
    if non_padding_len % 4 == 1 {
        return Err("base64 length is invalid".to_string());
    }

    let mut output = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u8;

    for byte in s[..non_padding_len].bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return Err(format!("invalid base64 character {:?}", char::from(byte))),
        };
        buffer = (buffer << 6) | u32::from(value);
        bits += 6;
        while bits >= 8 {
            bits -= 8;
            output.push(((buffer >> bits) & 0xff) as u8);
            buffer &= (1 << bits) - 1;
        }
    }

    Ok(output)
}

fn registries_config_fingerprint(spec: &RegistriesConfigSpec) -> String {
    format!(
        "mirrors=[{}];auths=[{}];tls=[{}]",
        spec.mirrors
            .iter()
            .map(|(host, mirror)| registry_mirror_fingerprint(host, mirror))
            .collect::<Vec<_>>()
            .join("|"),
        spec.auths
            .iter()
            .map(|(host, auth)| registry_auth_fingerprint(host, auth))
            .collect::<Vec<_>>()
            .join("|"),
        spec.tls
            .iter()
            .map(|(host, tls)| registry_tls_fingerprint(host, tls))
            .collect::<Vec<_>>()
            .join("|")
    )
}

fn registry_mirror_fingerprint(host: &str, mirror: &RegistryMirrorConfig) -> String {
    let endpoints = mirror
        .endpoints
        .iter()
        .map(|endpoint| {
            format!(
                "{}:{}",
                hex_encode(endpoint.endpoint.as_bytes()),
                endpoint.override_path
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{}:{}:{}",
        hex_encode(host.as_bytes()),
        mirror.skip_fallback,
        endpoints
    )
}

fn registry_auth_fingerprint(host: &str, auth: &RegistryAuthConfig) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        hex_encode(host.as_bytes()),
        hex_encode(auth.username.as_bytes()),
        hex_encode(auth.password.as_bytes()),
        hex_encode(auth.auth.as_bytes()),
        hex_encode(auth.identity_token.as_bytes())
    )
}

fn registry_tls_fingerprint(host: &str, tls: &RegistryTlsConfig) -> String {
    let identity = tls.client_identity.as_ref().map_or_else(
        || "none".to_string(),
        |identity| {
            format!(
                "some:{}:{}",
                hex_encode(&identity.cert),
                hex_encode(&identity.key)
            )
        },
    );
    format!(
        "{}:{}:{}:{}",
        hex_encode(host.as_bytes()),
        hex_encode(&tls.ca),
        tls.insecure_skip_verify,
        identity
    )
}

fn registries_config_from_fingerprint(fingerprint: &str) -> Option<RegistriesConfigSpec> {
    let rest = fingerprint.strip_prefix("mirrors=[")?;
    let (mirrors, rest) = rest.split_once("];auths=[")?;
    let (auths, rest) = rest.split_once("];tls=[")?;
    let tls = rest.strip_suffix(']')?;

    Some(RegistriesConfigSpec {
        mirrors: parse_registry_mirrors(mirrors)?,
        auths: parse_registry_auths(auths)?,
        tls: parse_registry_tls(tls)?,
    })
}

fn parse_registry_mirrors(value: &str) -> Option<BTreeMap<String, RegistryMirrorConfig>> {
    let mut mirrors = BTreeMap::new();
    if value.is_empty() {
        return Some(mirrors);
    }

    for entry in value.split('|') {
        let mut parts = entry.splitn(3, ':');
        let host = hex_decode_string(parts.next()?)?;
        let skip_fallback = parse_bool(parts.next()?)?;
        let endpoints = parts.next()?;
        let endpoints = if endpoints.is_empty() {
            Vec::new()
        } else {
            endpoints
                .split(',')
                .map(parse_registry_endpoint)
                .collect::<Option<Vec<_>>>()?
        };
        mirrors.insert(
            host,
            RegistryMirrorConfig {
                endpoints,
                skip_fallback,
            },
        );
    }

    Some(mirrors)
}

fn parse_registry_endpoint(value: &str) -> Option<RegistryEndpointConfig> {
    let (endpoint, override_path) = value.split_once(':')?;
    Some(RegistryEndpointConfig {
        endpoint: hex_decode_string(endpoint)?,
        override_path: parse_bool(override_path)?,
    })
}

fn parse_registry_auths(value: &str) -> Option<BTreeMap<String, RegistryAuthConfig>> {
    let mut auths = BTreeMap::new();
    if value.is_empty() {
        return Some(auths);
    }

    for entry in value.split('|') {
        let parts = entry.split(':').collect::<Vec<_>>();
        if parts.len() != 5 {
            return None;
        }
        auths.insert(
            hex_decode_string(parts[0])?,
            RegistryAuthConfig {
                username: hex_decode_string(parts[1])?,
                password: hex_decode_string(parts[2])?,
                auth: hex_decode_string(parts[3])?,
                identity_token: hex_decode_string(parts[4])?,
            },
        );
    }

    Some(auths)
}

fn parse_registry_tls(value: &str) -> Option<BTreeMap<String, RegistryTlsConfig>> {
    let mut tls_map = BTreeMap::new();
    if value.is_empty() {
        return Some(tls_map);
    }

    for entry in value.split('|') {
        let mut parts = entry.splitn(4, ':');
        let host = hex_decode_string(parts.next()?)?;
        let ca = hex_decode(parts.next()?)?;
        let insecure_skip_verify = parse_bool(parts.next()?)?;
        let identity = parse_registry_client_identity(parts.next()?)?;
        tls_map.insert(
            host,
            RegistryTlsConfig {
                ca,
                insecure_skip_verify,
                client_identity: identity,
            },
        );
    }

    Some(tls_map)
}

fn parse_registry_client_identity(value: &str) -> Option<Option<RegistryClientIdentity>> {
    if value == "none" {
        return Some(None);
    }

    let rest = value.strip_prefix("some:")?;
    let (cert, key) = rest.split_once(':')?;
    Some(Some(RegistryClientIdentity {
        cert: hex_decode(cert)?,
        key: hex_decode(key)?,
    }))
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn hex_decode_string(value: &str) -> Option<String> {
    String::from_utf8(hex_decode(value)?).ok()
}

fn hex_decode(value: &str) -> Option<Vec<u8>> {
    let bytes = value.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return None;
    }

    let mut out = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let high = hex_nibble(chunk[0])?;
        let low = hex_nibble(chunk[1])?;
        out.push((high << 4) | low);
    }
    Some(out)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// CRI pod sandbox state, mirroring `PodSandboxState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PodSandboxState {
    /// Sandbox is set up and ready.
    Ready,
    /// Sandbox has been stopped (network torn down) but not yet removed.
    NotReady,
}

/// CRI container state, mirroring `ContainerState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CriContainerState {
    /// Created but not started.
    Created,
    /// Running.
    Running,
    /// Exited.
    Exited,
}

/// Identifying metadata for a pod sandbox, mirroring `PodSandboxMetadata`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodSandboxConfig {
    /// Pod name.
    pub name: String,
    /// Pod namespace (Kubernetes namespace, not containerd namespace).
    pub namespace: String,
    /// Pod UID.
    pub uid: String,
    /// Pod-level labels.
    pub labels: HashMap<String, String>,
    /// Whether the pod requests host networking.
    pub host_network: bool,
}

impl PodSandboxConfig {
    /// Build a pod sandbox config; name/namespace/uid are required.
    pub fn new(
        name: impl Into<String>,
        namespace: impl Into<String>,
        uid: impl Into<String>,
    ) -> Result<Self> {
        let name = name.into();
        let namespace = namespace.into();
        let uid = uid.into();
        if name.is_empty() || namespace.is_empty() || uid.is_empty() {
            return Err(Error::invalid(
                "pod sandbox requires name, namespace and uid",
            ));
        }
        Ok(PodSandboxConfig {
            name,
            namespace,
            uid,
            labels: HashMap::new(),
            host_network: false,
        })
    }

    /// Builder: request host networking (no per-pod network namespace).
    pub fn with_host_network(mut self) -> Self {
        self.host_network = true;
        self
    }

    /// Builder: attach a label.
    pub fn with_label(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.labels.insert(k.into(), v.into());
        self
    }
}

/// A live pod sandbox record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodSandbox {
    /// Sandbox id assigned by the runtime.
    pub id: String,
    /// The config it was created from.
    pub config: PodSandboxConfig,
    /// Current state.
    pub state: PodSandboxState,
}

/// Config to create a container within a sandbox, mirroring `ContainerConfig`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriContainerConfig {
    /// Container name (unique within the pod).
    pub name: String,
    /// Image to run.
    pub image: ImageRef,
    /// Entry command (argv).
    pub command: Vec<String>,
    /// Args appended to the command.
    pub args: Vec<String>,
}

impl CriContainerConfig {
    /// Build a CRI container config.
    pub fn new(name: impl Into<String>, image: ImageRef, command: Vec<String>) -> Result<Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(Error::invalid("container name must not be empty"));
        }
        if command.is_empty() {
            return Err(Error::invalid("container command must not be empty"));
        }
        Ok(CriContainerConfig {
            name,
            image,
            command,
            args: Vec::new(),
        })
    }

    /// Builder: append container args.
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
}

/// A live container record within a sandbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriContainer {
    /// Container id assigned by the runtime.
    pub id: String,
    /// The sandbox it belongs to.
    pub sandbox_id: String,
    /// The config it was created from.
    pub config: CriContainerConfig,
    /// Current state.
    pub state: CriContainerState,
    /// Exit code, once exited.
    pub exit_code: Option<i32>,
}

/// The CRI `RuntimeService` surface the kubelet drives.
pub trait RuntimeService {
    /// Create and start a pod sandbox; returns the sandbox id.
    fn run_pod_sandbox(&mut self, config: PodSandboxConfig) -> Result<String>;
    /// Stop a sandbox: tears down networking, moves to `NotReady`.
    fn stop_pod_sandbox(&mut self, sandbox_id: &str) -> Result<()>;
    /// Remove a sandbox; must be stopped and have no containers.
    fn remove_pod_sandbox(&mut self, sandbox_id: &str) -> Result<()>;
    /// List sandbox ids (sorted).
    fn list_pod_sandboxes(&self) -> Vec<&str>;

    /// Create a container in a (ready) sandbox; returns the container id.
    fn create_container(&mut self, sandbox_id: &str, config: CriContainerConfig) -> Result<String>;
    /// Start a created container.
    fn start_container(&mut self, container_id: &str) -> Result<()>;
    /// Stop a running container with the given exit code.
    fn stop_container(&mut self, container_id: &str, exit_code: i32) -> Result<()>;
    /// Remove an exited (or created) container.
    fn remove_container(&mut self, container_id: &str) -> Result<()>;
    /// Fetch a container record.
    fn container_status(&self, container_id: &str) -> Option<&CriContainer>;
}

/// The CRI `ImageService` surface.
pub trait ImageService {
    /// Pull an image, returning its (canonical) image ref string.
    fn pull_image(&mut self, image: &ImageRef) -> Result<String>;
    /// Whether an image is present in the runtime's store.
    fn image_status(&self, image: &ImageRef) -> bool;
    /// Remove an image from the store.
    fn remove_image(&mut self, image: &ImageRef) -> Result<()>;
    /// List present image references (sorted, canonical).
    fn list_images(&self) -> Vec<String>;
}

/// In-memory CRI runtime implementing both services.
#[derive(Debug, Default)]
pub struct CriRuntime {
    sandboxes: HashMap<String, PodSandbox>,
    containers: HashMap<String, CriContainer>,
    images: HashMap<String, ImageRef>,
    next_id: u64,
}

impl CriRuntime {
    /// Create an empty CRI runtime.
    pub fn new() -> Self {
        CriRuntime::default()
    }

    fn alloc_id(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}-{:016x}", self.next_id)
    }

    /// Count containers belonging to a sandbox.
    pub fn containers_in_sandbox(&self, sandbox_id: &str) -> usize {
        self.containers
            .values()
            .filter(|c| c.sandbox_id == sandbox_id)
            .count()
    }

    /// Fetch a sandbox record.
    pub fn pod_sandbox_status(&self, sandbox_id: &str) -> Option<&PodSandbox> {
        self.sandboxes.get(sandbox_id)
    }
}

impl RuntimeService for CriRuntime {
    fn run_pod_sandbox(&mut self, config: PodSandboxConfig) -> Result<String> {
        let id = self.alloc_id("sandbox");
        self.sandboxes.insert(
            id.clone(),
            PodSandbox {
                id: id.clone(),
                config,
                state: PodSandboxState::Ready,
            },
        );
        Ok(id)
    }

    fn stop_pod_sandbox(&mut self, sandbox_id: &str) -> Result<()> {
        // Stopping the sandbox stops all its containers (kubelet semantics).
        let cids: Vec<String> = self
            .containers
            .values()
            .filter(|c| c.sandbox_id == sandbox_id && c.state == CriContainerState::Running)
            .map(|c| c.id.clone())
            .collect();
        for cid in cids {
            if let Some(c) = self.containers.get_mut(&cid) {
                c.state = CriContainerState::Exited;
                c.exit_code = Some(137); // 128 + SIGKILL
            }
        }
        let sb = self
            .sandboxes
            .get_mut(sandbox_id)
            .ok_or_else(|| Error::not_found("pod sandbox"))?;
        sb.state = PodSandboxState::NotReady;
        Ok(())
    }

    fn remove_pod_sandbox(&mut self, sandbox_id: &str) -> Result<()> {
        let sb = self
            .sandboxes
            .get(sandbox_id)
            .ok_or_else(|| Error::not_found("pod sandbox"))?;
        if sb.state == PodSandboxState::Ready {
            return Err(Error::invalid_state(
                "sandbox must be stopped before removal",
            ));
        }
        if self.containers_in_sandbox(sandbox_id) > 0 {
            return Err(Error::invalid_state("sandbox still has containers"));
        }
        self.sandboxes.remove(sandbox_id);
        Ok(())
    }

    fn list_pod_sandboxes(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.sandboxes.keys().map(String::as_str).collect();
        ids.sort_unstable();
        ids
    }

    fn create_container(&mut self, sandbox_id: &str, config: CriContainerConfig) -> Result<String> {
        let sb = self
            .sandboxes
            .get(sandbox_id)
            .ok_or_else(|| Error::not_found("pod sandbox"))?;
        if sb.state != PodSandboxState::Ready {
            return Err(Error::invalid_state("sandbox is not ready"));
        }
        // The image must have been pulled.
        if !self.images.contains_key(&config.image.canonical()) {
            return Err(Error::not_found("image not pulled"));
        }
        // Container name must be unique within the pod.
        let dup = self
            .containers
            .values()
            .any(|c| c.sandbox_id == sandbox_id && c.config.name == config.name);
        if dup {
            return Err(Error::invalid_state(
                "container name already exists in sandbox",
            ));
        }
        let id = self.alloc_id("container");
        self.containers.insert(
            id.clone(),
            CriContainer {
                id: id.clone(),
                sandbox_id: sandbox_id.to_string(),
                config,
                state: CriContainerState::Created,
                exit_code: None,
            },
        );
        Ok(id)
    }

    fn start_container(&mut self, container_id: &str) -> Result<()> {
        let c = self
            .containers
            .get_mut(container_id)
            .ok_or_else(|| Error::not_found("container"))?;
        if c.state != CriContainerState::Created {
            return Err(Error::invalid_state(
                "only created containers can be started",
            ));
        }
        c.state = CriContainerState::Running;
        Ok(())
    }

    fn stop_container(&mut self, container_id: &str, exit_code: i32) -> Result<()> {
        let c = self
            .containers
            .get_mut(container_id)
            .ok_or_else(|| Error::not_found("container"))?;
        if c.state != CriContainerState::Running {
            return Err(Error::invalid_state(
                "only running containers can be stopped",
            ));
        }
        c.state = CriContainerState::Exited;
        c.exit_code = Some(exit_code);
        Ok(())
    }

    fn remove_container(&mut self, container_id: &str) -> Result<()> {
        let c = self
            .containers
            .get(container_id)
            .ok_or_else(|| Error::not_found("container"))?;
        if c.state == CriContainerState::Running {
            return Err(Error::invalid_state("cannot remove a running container"));
        }
        self.containers.remove(container_id);
        Ok(())
    }

    fn container_status(&self, container_id: &str) -> Option<&CriContainer> {
        self.containers.get(container_id)
    }
}

impl ImageService for CriRuntime {
    fn pull_image(&mut self, image: &ImageRef) -> Result<String> {
        let key = image.canonical();
        self.images.insert(key.clone(), image.clone());
        Ok(key)
    }

    fn image_status(&self, image: &ImageRef) -> bool {
        self.images.contains_key(&image.canonical())
    }

    fn remove_image(&mut self, image: &ImageRef) -> Result<()> {
        if self.images.remove(&image.canonical()).is_none() {
            return Err(Error::not_found("image"));
        }
        Ok(())
    }

    fn list_images(&self) -> Vec<String> {
        let mut v: Vec<String> = self.images.keys().cloned().collect();
        v.sort();
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pause() -> ImageRef {
        ImageRef::parse("registry.k8s.io/pause:3.9").unwrap()
    }

    fn app() -> ImageRef {
        ImageRef::parse("registry.k8s.io/coredns:1.11").unwrap()
    }

    fn cfg() -> PodSandboxConfig {
        PodSandboxConfig::new("coredns", "kube-system", "uid-1").unwrap()
    }

    fn add_test_registry_auth(spec: &mut RegistriesConfigSpec) {
        spec.auths.insert(
            "docker.io".to_string(),
            RegistryAuthConfig {
                username: "agent".to_string(),
                password: "secret".to_string(),
                auth: String::new(),
                identity_token: String::new(),
            },
        );
    }

    #[derive(Debug, Clone)]
    struct TestMachineConfigDocument {
        meta: Metadata,
        contents: String,
    }

    impl TestMachineConfigDocument {
        fn new(contents: impl Into<String>) -> Self {
            Self {
                meta: Metadata::new(
                    MACHINE_CONFIG_NAMESPACE,
                    MACHINE_CONFIG_TYPE,
                    ResourceId::new(MACHINE_CONFIG_ACTIVE_ID).unwrap(),
                ),
                contents: contents.into(),
            }
        }
    }

    impl Resource for TestMachineConfigDocument {
        fn metadata(&self) -> &Metadata {
            &self.meta
        }

        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.meta
        }

        fn spec_fingerprint(&self) -> String {
            format!("contents={}", test_hex_bytes(self.contents.as_bytes()))
        }

        fn clone_box(&self) -> Box<dyn Resource> {
            Box::new(self.clone())
        }
    }

    fn test_hex_bytes(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[test]
    fn registry_builder_from_state_uses_created_config_and_applies_options_to_clone() {
        let mut state = os_cosi_domain::State::new();
        let mut spec = RegistriesConfigSpec::default();
        spec.mirrors.insert(
            "docker.io".to_string(),
            RegistryMirrorConfig {
                endpoints: vec![RegistryEndpointConfig {
                    endpoint: "https://registry-cache.local".to_string(),
                    override_path: true,
                }],
                skip_fallback: true,
            },
        );
        state
            .create(Box::new(RegistriesConfigResource::new(spec)))
            .unwrap();

        let built = registry_builder_from_state(&mut state, 8, &[add_test_registry_auth])
            .unwrap()
            .unwrap();

        let mirror = built.mirrors.get("docker.io").unwrap();
        assert!(mirror.skip_fallback);
        assert_eq!(mirror.endpoints[0].endpoint, "https://registry-cache.local");
        assert!(mirror.endpoints[0].override_path);
        assert_eq!(built.auths.get("docker.io").unwrap().username, "agent");

        let stored = state.get(&registries_config_key().unwrap()).unwrap();
        let stored = RegistriesConfigResource::from_resource(stored.as_ref()).unwrap();
        assert!(stored.spec.auths.is_empty());
    }

    #[test]
    fn registry_builder_watch_uses_created_and_updated_events() {
        let mut state = os_cosi_domain::State::new();
        let watch_index = watch_registries_config(&mut state, 8);
        assert!(
            poll_registry_builder(&mut state, watch_index, &[])
                .unwrap()
                .is_none()
        );

        state
            .create(Box::new(RegistriesConfigResource::new(
                RegistriesConfigSpec::default(),
            )))
            .unwrap();
        assert!(
            poll_registry_builder(&mut state, watch_index, &[])
                .unwrap()
                .unwrap()
                .mirrors
                .is_empty()
        );

        let existing = state.get(&registries_config_key().unwrap()).unwrap();
        let mut updated = RegistriesConfigResource::new(RegistriesConfigSpec::default());
        updated.spec.tls.insert(
            "registry.example.com".to_string(),
            RegistryTlsConfig {
                ca: b"ca-pem".to_vec(),
                insecure_skip_verify: true,
                client_identity: Some(RegistryClientIdentity {
                    cert: b"cert".to_vec(),
                    key: b"key".to_vec(),
                }),
            },
        );
        *updated.metadata_mut() = existing.metadata().clone();
        state
            .update(Box::new(updated), existing.metadata().version())
            .unwrap();

        let built = poll_registry_builder(&mut state, watch_index, &[])
            .unwrap()
            .unwrap();
        let tls = built.tls.get("registry.example.com").unwrap();
        assert_eq!(tls.ca, b"ca-pem");
        assert!(tls.insecure_skip_verify);
        assert_eq!(tls.client_identity.as_ref().unwrap().cert, b"cert");
    }

    #[test]
    fn registry_config_spec_accessors_return_owned_views() {
        let mut spec = RegistriesConfigSpec::default();
        spec.mirrors.insert(
            "docker.io".to_string(),
            RegistryMirrorConfig {
                endpoints: vec![RegistryEndpointConfig {
                    endpoint: "https://mirror.local".to_string(),
                    override_path: true,
                }],
                skip_fallback: true,
            },
        );
        spec.auths.insert(
            "docker.io".to_string(),
            RegistryAuthConfig {
                username: "user".to_string(),
                password: "pass".to_string(),
                auth: "auth".to_string(),
                identity_token: "token".to_string(),
            },
        );
        spec.tls.insert(
            "registry.local".to_string(),
            RegistryTlsConfig {
                ca: b"ca".to_vec(),
                client_identity: Some(RegistryClientIdentity {
                    cert: b"cert".to_vec(),
                    key: b"key".to_vec(),
                }),
                insecure_skip_verify: true,
            },
        );

        let mut mirrors = spec.mirrors();
        mirrors
            .get_mut("docker.io")
            .unwrap()
            .endpoints
            .push(RegistryEndpointConfig {
                endpoint: "https://mutated.local".to_string(),
                override_path: false,
            });
        mirrors.get_mut("docker.io").unwrap().skip_fallback = false;

        let mut auths = spec.auths();
        auths.get_mut("docker.io").unwrap().username = "mutated".to_string();

        let mut tls = spec.tls();
        tls.get_mut("registry.local").unwrap().ca = b"mutated".to_vec();

        let original_mirror = spec.mirrors.get("docker.io").unwrap();
        assert_eq!(original_mirror.endpoints.len(), 1);
        assert!(original_mirror.skip_fallback);
        assert_eq!(spec.auths.get("docker.io").unwrap().username, "user");
        assert_eq!(spec.tls.get("registry.local").unwrap().ca, b"ca");
    }

    #[test]
    fn registry_config_leaf_accessors_match_source_names() {
        let endpoint = RegistryEndpointConfig {
            endpoint: "https://mirror.local/v2".to_string(),
            override_path: true,
        };
        assert_eq!(endpoint.endpoint(), "https://mirror.local/v2");
        assert!(endpoint.override_path());

        let mirror = RegistryMirrorConfig {
            endpoints: vec![endpoint],
            skip_fallback: true,
        };
        assert!(mirror.skip_fallback());
        let mut endpoints = mirror.endpoints();
        endpoints[0].endpoint = "https://mutated.local".to_string();
        assert_eq!(mirror.endpoints[0].endpoint, "https://mirror.local/v2");

        let auth = RegistryAuthConfig {
            username: "user".to_string(),
            password: "pass".to_string(),
            auth: "auth".to_string(),
            identity_token: "token".to_string(),
        };
        assert_eq!(auth.username(), "user");
        assert_eq!(auth.password(), "pass");
        assert_eq!(auth.auth(), "auth");
        assert_eq!(auth.identity_token(), "token");

        let tls = RegistryTlsConfig {
            ca: b"ca-pem".to_vec(),
            client_identity: Some(RegistryClientIdentity {
                cert: b"cert-pem".to_vec(),
                key: b"key-pem".to_vec(),
            }),
            insecure_skip_verify: true,
        };
        assert_eq!(tls.ca(), b"ca-pem");
        assert!(tls.insecure_skip_verify());
        assert_eq!(tls.client_identity().unwrap().cert, b"cert-pem");
    }

    #[test]
    fn registry_config_controller_projection_clears_stale_and_copies_config() {
        let mut existing = RegistriesConfigSpec::default();
        existing.mirrors.insert(
            "stale.local".to_string(),
            RegistryMirrorConfig {
                endpoints: vec![RegistryEndpointConfig {
                    endpoint: "https://stale.local".to_string(),
                    override_path: false,
                }],
                skip_fallback: false,
            },
        );
        existing.auths.insert(
            "stale.local".to_string(),
            RegistryAuthConfig {
                username: "stale".to_string(),
                password: "stale".to_string(),
                auth: String::new(),
                identity_token: String::new(),
            },
        );

        let mut configured = RegistriesConfigSpec::default();
        configured.mirrors.insert(
            "docker.io".to_string(),
            RegistryMirrorConfig {
                endpoints: vec![RegistryEndpointConfig {
                    endpoint: "https://cache.local".to_string(),
                    override_path: true,
                }],
                skip_fallback: true,
            },
        );
        configured.auths.insert(
            "docker.io".to_string(),
            RegistryAuthConfig {
                username: "user".to_string(),
                password: "pass".to_string(),
                auth: "auth".to_string(),
                identity_token: "token".to_string(),
            },
        );
        configured.tls.insert(
            "registry.local".to_string(),
            RegistryTlsConfig {
                ca: b"ca".to_vec(),
                client_identity: Some(RegistryClientIdentity {
                    cert: b"cert".to_vec(),
                    key: b"key".to_vec(),
                }),
                insecure_skip_verify: true,
            },
        );

        reconcile_registries_config(&mut existing, Some(&configured), false);

        assert!(!existing.mirrors.contains_key("stale.local"));
        assert!(!existing.auths.contains_key("stale.local"));
        assert_eq!(
            existing.mirrors["docker.io"].endpoints[0].endpoint,
            "https://cache.local"
        );
        assert!(existing.mirrors["docker.io"].endpoints[0].override_path);
        assert!(existing.mirrors["docker.io"].skip_fallback);
        assert_eq!(existing.auths["docker.io"].identity_token, "token");
        assert_eq!(existing.tls["registry.local"].ca, b"ca");
        assert_eq!(
            existing.tls["registry.local"]
                .client_identity
                .as_ref()
                .unwrap()
                .key,
            b"key"
        );
    }

    #[test]
    fn registry_config_controller_projection_injects_registryd_when_image_cache_ready() {
        let mut configured = RegistriesConfigSpec::default();
        configured.mirrors.insert(
            "docker.io".to_string(),
            RegistryMirrorConfig {
                endpoints: vec![RegistryEndpointConfig {
                    endpoint: "https://cache.local".to_string(),
                    override_path: true,
                }],
                skip_fallback: true,
            },
        );

        let mut projected = RegistriesConfigSpec::default();
        reconcile_registries_config(&mut projected, Some(&configured), true);

        assert_eq!(
            projected.mirrors["docker.io"].endpoints[0].endpoint,
            registryd_mirror_endpoint()
        );
        assert!(!projected.mirrors["docker.io"].endpoints[0].override_path);
        assert_eq!(
            projected.mirrors["docker.io"].endpoints[1].endpoint,
            "https://cache.local"
        );
        assert!(projected.mirrors["docker.io"].skip_fallback);
        assert_eq!(
            projected.mirrors["*"].endpoints[0].endpoint,
            registryd_mirror_endpoint()
        );
    }

    #[test]
    fn registries_config_state_apply_creates_singleton_and_emits_created() {
        let mut state = os_cosi_domain::State::new();
        let watch_index = watch_registries_config(&mut state, 8);
        assert!(
            poll_registry_builder(&mut state, watch_index, &[])
                .unwrap()
                .is_none()
        );

        let mut configured = RegistriesConfigSpec::default();
        configured.mirrors.insert(
            "docker.io".to_string(),
            RegistryMirrorConfig {
                endpoints: vec![RegistryEndpointConfig {
                    endpoint: "https://cache.local".to_string(),
                    override_path: true,
                }],
                skip_fallback: true,
            },
        );

        let applied =
            apply_registries_config_to_state(&mut state, Some(&configured), true).unwrap();

        let stored = state.get(&registries_config_key().unwrap()).unwrap();
        assert_eq!(stored.metadata().version(), 1);
        let stored = RegistriesConfigResource::from_resource(stored.as_ref()).unwrap();
        assert_eq!(stored.spec, applied);
        assert_eq!(
            applied.mirrors["docker.io"].endpoints[0].endpoint,
            registryd_mirror_endpoint()
        );
        assert_eq!(
            applied.mirrors["docker.io"].endpoints[1].endpoint,
            "https://cache.local"
        );
        assert_eq!(
            applied.mirrors["*"].endpoints[0].endpoint,
            registryd_mirror_endpoint()
        );

        let watched = poll_registry_builder(&mut state, watch_index, &[])
            .unwrap()
            .unwrap();
        assert_eq!(watched, applied);
    }

    #[test]
    fn registries_config_state_apply_updates_existing_and_clears_stale() {
        let mut state = os_cosi_domain::State::new();
        let mut stale = RegistriesConfigSpec::default();
        stale.mirrors.insert(
            "stale.local".to_string(),
            RegistryMirrorConfig {
                endpoints: vec![RegistryEndpointConfig {
                    endpoint: "https://stale.local".to_string(),
                    override_path: false,
                }],
                skip_fallback: false,
            },
        );
        stale.auths.insert(
            "stale.local".to_string(),
            RegistryAuthConfig {
                username: "stale".to_string(),
                password: "stale".to_string(),
                auth: String::new(),
                identity_token: String::new(),
            },
        );
        stale.tls.insert(
            "stale.local".to_string(),
            RegistryTlsConfig {
                ca: b"stale-ca".to_vec(),
                client_identity: None,
                insecure_skip_verify: false,
            },
        );
        state
            .create(Box::new(RegistriesConfigResource::new(stale)))
            .unwrap();
        let watch_index = watch_registries_config(&mut state, 8);
        assert!(
            poll_registry_builder(&mut state, watch_index, &[])
                .unwrap()
                .is_some()
        );

        let mut configured = RegistriesConfigSpec::default();
        configured.mirrors.insert(
            "registry.local".to_string(),
            RegistryMirrorConfig {
                endpoints: vec![RegistryEndpointConfig {
                    endpoint: "https://registry.local/cache".to_string(),
                    override_path: true,
                }],
                skip_fallback: true,
            },
        );
        configured.auths.insert(
            "registry.local".to_string(),
            RegistryAuthConfig {
                username: "user".to_string(),
                password: "pass".to_string(),
                auth: "auth".to_string(),
                identity_token: "token".to_string(),
            },
        );
        configured.tls.insert(
            "registry.local".to_string(),
            RegistryTlsConfig {
                ca: b"ca".to_vec(),
                client_identity: Some(RegistryClientIdentity {
                    cert: b"cert".to_vec(),
                    key: b"key".to_vec(),
                }),
                insecure_skip_verify: true,
            },
        );

        let applied =
            apply_registries_config_to_state(&mut state, Some(&configured), false).unwrap();

        let stored = state.get(&registries_config_key().unwrap()).unwrap();
        assert_eq!(stored.metadata().version(), 2);
        let stored = RegistriesConfigResource::from_resource(stored.as_ref()).unwrap();
        assert_eq!(stored.spec, applied);
        assert!(!stored.spec.mirrors.contains_key("stale.local"));
        assert!(!stored.spec.auths.contains_key("stale.local"));
        assert!(!stored.spec.tls.contains_key("stale.local"));
        assert_eq!(
            stored.spec.mirrors["registry.local"].endpoints[0].endpoint,
            "https://registry.local/cache"
        );
        assert!(stored.spec.mirrors["registry.local"].endpoints[0].override_path);
        assert!(stored.spec.mirrors["registry.local"].skip_fallback);
        assert_eq!(stored.spec.auths["registry.local"].identity_token, "token");
        assert_eq!(stored.spec.tls["registry.local"].ca, b"ca");
        assert!(!stored.spec.mirrors.contains_key("*"));

        let watched = poll_registry_builder(&mut state, watch_index, &[])
            .unwrap()
            .unwrap();
        assert_eq!(watched, applied);
    }

    #[test]
    fn registries_config_controller_spec_matches_source_declarations() {
        assert_eq!(
            REGISTRIES_CONFIG_CONTROLLER_NAME,
            "cri.RegistriesConfigController"
        );
        assert_eq!(MACHINE_CONFIG_NAMESPACE, "config");
        assert_eq!(MACHINE_CONFIG_TYPE, "MachineConfigs.config.talos.dev");
        assert_eq!(MACHINE_CONFIG_ACTIVE_ID, "v1alpha1");

        let spec = registries_config_controller_spec();
        assert_eq!(spec.inputs().len(), 2);
        assert_eq!(spec.outputs().len(), 1);

        let machine_config = &spec.inputs()[0];
        assert_eq!(
            machine_config.kind(),
            &ResourceKind::new(MACHINE_CONFIG_NAMESPACE, MACHINE_CONFIG_TYPE)
        );
        assert_eq!(machine_config.id(), Some(MACHINE_CONFIG_ACTIVE_ID));
        assert_eq!(machine_config.strength(), os_cosi_domain::InputKind::Weak);

        let image_cache = &spec.inputs()[1];
        assert_eq!(
            image_cache.kind(),
            &crate::image_cache::ImageCacheConfigResource::kind()
        );
        assert_eq!(image_cache.id(), None);
        assert_eq!(image_cache.strength(), os_cosi_domain::InputKind::Weak);

        let output = &spec.outputs()[0];
        assert_eq!(output.kind(), &RegistriesConfigResource::kind());
        assert!(output.is_exclusive());
    }

    #[test]
    fn registries_config_controller_state_apply_uses_image_cache_ready_input() {
        let mut state = os_cosi_domain::State::new();
        let mut configured = RegistriesConfigSpec::default();
        configured.mirrors.insert(
            "docker.io".to_string(),
            RegistryMirrorConfig {
                endpoints: vec![RegistryEndpointConfig {
                    endpoint: "https://cache.local".to_string(),
                    override_path: true,
                }],
                skip_fallback: true,
            },
        );

        let missing_image_cache =
            apply_registries_config_controller_to_state(&mut state, Some(&configured)).unwrap();
        assert_eq!(missing_image_cache.mirrors["docker.io"].endpoints.len(), 1);
        assert!(!missing_image_cache.mirrors.contains_key("*"));
        assert!(!image_cache_ready_from_state(&state).unwrap());

        state
            .create(Box::new(crate::image_cache::ImageCacheConfigResource::new(
                crate::image_cache::ImageCacheConfig {
                    status: crate::image_cache::ImageCacheStatus::Ready,
                    copy_status: crate::image_cache::ImageCacheCopyStatus::Skipped,
                    roots: vec!["/system/imagecache/disk".to_string()],
                },
            )))
            .unwrap();
        assert!(image_cache_ready_from_state(&state).unwrap());

        let ready_image_cache =
            apply_registries_config_controller_to_state(&mut state, Some(&configured)).unwrap();
        assert_eq!(
            ready_image_cache.mirrors["docker.io"].endpoints[0].endpoint,
            registryd_mirror_endpoint()
        );
        assert_eq!(
            ready_image_cache.mirrors["docker.io"].endpoints[1].endpoint,
            "https://cache.local"
        );
        assert_eq!(
            ready_image_cache.mirrors["*"].endpoints[0].endpoint,
            registryd_mirror_endpoint()
        );
    }

    #[test]
    fn registries_config_controller_state_apply_ignores_non_ready_image_cache_input() {
        let mut state = os_cosi_domain::State::new();
        state
            .create(Box::new(crate::image_cache::ImageCacheConfigResource::new(
                crate::image_cache::ImageCacheConfig {
                    status: crate::image_cache::ImageCacheStatus::Preparing,
                    copy_status: crate::image_cache::ImageCacheCopyStatus::Pending,
                    roots: vec!["/system/imagecache/disk".to_string()],
                },
            )))
            .unwrap();
        assert!(!image_cache_ready_from_state(&state).unwrap());

        let mut configured = RegistriesConfigSpec::default();
        configured.mirrors.insert(
            "registry.local".to_string(),
            RegistryMirrorConfig {
                endpoints: vec![RegistryEndpointConfig {
                    endpoint: "https://registry.local/cache".to_string(),
                    override_path: false,
                }],
                skip_fallback: false,
            },
        );

        let applied =
            apply_registries_config_controller_to_state(&mut state, Some(&configured)).unwrap();
        assert_eq!(applied.mirrors["registry.local"].endpoints.len(), 1);
        assert_eq!(
            applied.mirrors["registry.local"].endpoints[0].endpoint,
            "https://registry.local/cache"
        );
        assert!(!applied.mirrors.contains_key("*"));
    }

    #[test]
    fn registries_config_controller_state_apply_rejects_malformed_image_cache_input() {
        #[derive(Debug, Clone)]
        struct MalformedImageCacheConfigResource {
            meta: Metadata,
        }

        impl MalformedImageCacheConfigResource {
            fn new() -> Self {
                Self {
                    meta: Metadata::new(
                        crate::image_cache::IMAGE_CACHE_NAMESPACE,
                        crate::image_cache::IMAGE_CACHE_CONFIG_TYPE,
                        ResourceId::new(crate::image_cache::IMAGE_CACHE_CONFIG_ID).unwrap(),
                    ),
                }
            }
        }

        impl Resource for MalformedImageCacheConfigResource {
            fn metadata(&self) -> &Metadata {
                &self.meta
            }

            fn metadata_mut(&mut self) -> &mut Metadata {
                &mut self.meta
            }

            fn spec_fingerprint(&self) -> String {
                "not-an-image-cache-config-fingerprint".to_string()
            }

            fn clone_box(&self) -> Box<dyn Resource> {
                Box::new(self.clone())
            }
        }

        let mut state = os_cosi_domain::State::new();
        state
            .create(Box::new(MalformedImageCacheConfigResource::new()))
            .unwrap();

        let err = image_cache_ready_from_state(&state).unwrap_err();
        assert!(matches!(
            err,
            RegistryBuilderError::MalformedImageCacheConfig { .. }
        ));
        assert!(
            err.to_string()
                .contains(crate::image_cache::IMAGE_CACHE_CONFIG_ID)
        );
        assert!(
            apply_registries_config_controller_to_state(&mut state, None)
                .unwrap_err()
                .to_string()
                .contains("malformed")
        );
    }

    #[test]
    fn registries_config_controller_inputs_apply_reads_active_machine_config_registries() {
        let mut state = os_cosi_domain::State::new();
        state
            .create(Box::new(TestMachineConfigDocument::new(
                r#"
version: v1alpha1
machine:
  registries:
    mirrors:
      docker.io:
        endpoints:
          - https://cache.local/v2
          - https://fallback.local/v2
        overridePath: true
        skipFallback: true
    config:
      registry.local:
        auth:
          username: puller
          password: secret
          auth: encoded-auth
          identityToken: identity-token
        tls:
          insecureSkipVerify: true
          ca: Y2EtcGVt
          clientIdentity:
            crt: Y2VydC1wZW0=
            key: a2V5LXBlbQ==
"#,
            )))
            .unwrap();
        state
            .create(Box::new(crate::image_cache::ImageCacheConfigResource::new(
                crate::image_cache::ImageCacheConfig {
                    status: crate::image_cache::ImageCacheStatus::Ready,
                    copy_status: crate::image_cache::ImageCacheCopyStatus::Skipped,
                    roots: vec!["/system/imagecache/disk".to_string()],
                },
            )))
            .unwrap();

        let from_machine = machine_config_registries_from_state(&state)
            .unwrap()
            .unwrap();
        assert_eq!(from_machine.mirrors["docker.io"].endpoints.len(), 2);
        assert!(from_machine.mirrors["docker.io"].endpoints[0].override_path);
        assert!(from_machine.mirrors["docker.io"].skip_fallback);
        assert_eq!(from_machine.auths["registry.local"].username, "puller");
        assert_eq!(from_machine.auths["registry.local"].password, "secret");
        assert_eq!(from_machine.auths["registry.local"].auth, "encoded-auth");
        assert_eq!(
            from_machine.auths["registry.local"].identity_token,
            "identity-token"
        );
        assert!(from_machine.tls["registry.local"].insecure_skip_verify);
        assert_eq!(from_machine.tls["registry.local"].ca, b"ca-pem");
        assert_eq!(
            from_machine.tls["registry.local"]
                .client_identity
                .as_ref()
                .unwrap()
                .cert,
            b"cert-pem"
        );

        let applied = apply_registries_config_controller_inputs_to_state(&mut state).unwrap();
        let docker = &applied.mirrors["docker.io"];
        assert_eq!(docker.endpoints[0].endpoint, registryd_mirror_endpoint());
        assert_eq!(docker.endpoints[1].endpoint, "https://cache.local/v2");
        assert!(docker.endpoints[1].override_path);
        assert_eq!(
            applied.mirrors["*"].endpoints[0].endpoint,
            registryd_mirror_endpoint()
        );
        assert_eq!(applied.auths["registry.local"].username, "puller");
        assert_eq!(applied.tls["registry.local"].ca, b"ca-pem");
    }

    #[test]
    fn registries_config_controller_inputs_apply_tolerates_missing_machine_config() {
        let mut state = os_cosi_domain::State::new();
        state
            .create(Box::new(crate::image_cache::ImageCacheConfigResource::new(
                crate::image_cache::ImageCacheConfig {
                    status: crate::image_cache::ImageCacheStatus::Ready,
                    copy_status: crate::image_cache::ImageCacheCopyStatus::Skipped,
                    roots: vec!["/system/imagecache/disk".to_string()],
                },
            )))
            .unwrap();

        assert!(
            machine_config_registries_from_state(&state)
                .unwrap()
                .is_none()
        );
        let applied = apply_registries_config_controller_inputs_to_state(&mut state).unwrap();
        assert_eq!(applied.mirrors.len(), 1);
        assert_eq!(
            applied.mirrors["*"].endpoints[0].endpoint,
            registryd_mirror_endpoint()
        );
    }

    #[test]
    fn registries_config_controller_inputs_apply_rejects_malformed_machine_config() {
        #[derive(Debug, Clone)]
        struct MalformedMachineConfigDocument {
            meta: Metadata,
        }

        impl MalformedMachineConfigDocument {
            fn new() -> Self {
                Self {
                    meta: Metadata::new(
                        MACHINE_CONFIG_NAMESPACE,
                        MACHINE_CONFIG_TYPE,
                        ResourceId::new(MACHINE_CONFIG_ACTIVE_ID).unwrap(),
                    ),
                }
            }
        }

        impl Resource for MalformedMachineConfigDocument {
            fn metadata(&self) -> &Metadata {
                &self.meta
            }

            fn metadata_mut(&mut self) -> &mut Metadata {
                &mut self.meta
            }

            fn spec_fingerprint(&self) -> String {
                "contents=not-hex".to_string()
            }

            fn clone_box(&self) -> Box<dyn Resource> {
                Box::new(self.clone())
            }
        }

        let mut state = os_cosi_domain::State::new();
        state
            .create(Box::new(MalformedMachineConfigDocument::new()))
            .unwrap();

        let err = machine_config_registries_from_state(&state).unwrap_err();
        assert!(matches!(
            err,
            RegistryBuilderError::MalformedMachineConfig { .. }
        ));
        assert!(err.to_string().contains(MACHINE_CONFIG_ACTIVE_ID));
        assert!(
            apply_registries_config_controller_inputs_to_state(&mut state)
                .unwrap_err()
                .to_string()
                .contains("machine config")
        );
    }

    #[test]
    fn registries_config_controller_inputs_apply_rejects_malformed_machine_config_tls_base64() {
        let mut state = os_cosi_domain::State::new();
        state
            .create(Box::new(TestMachineConfigDocument::new(
                r#"
version: v1alpha1
machine:
  registries:
    config:
      registry.local:
        tls:
          ca: not-base64*
"#,
            )))
            .unwrap();

        let err = machine_config_registries_from_state(&state).unwrap_err();
        assert!(matches!(
            err,
            RegistryBuilderError::MalformedMachineConfig { .. }
        ));
        assert!(err.to_string().contains("base64"));
        assert!(
            apply_registries_config_controller_inputs_to_state(&mut state)
                .unwrap_err()
                .to_string()
                .contains("machine config")
        );
    }

    #[test]
    fn registries_config_controller_context_apply_tracks_output_and_cleans_stale() {
        let mut state = os_cosi_domain::State::new();
        state
            .create(Box::new(TestMachineConfigDocument::new(
                r#"
version: v1alpha1
machine:
  registries:
    mirrors:
      docker.io:
        endpoints:
          - https://cache.local/v2
"#,
            )))
            .unwrap();
        state
            .create(Box::new(crate::image_cache::ImageCacheConfigResource::new(
                crate::image_cache::ImageCacheConfig {
                    status: crate::image_cache::ImageCacheStatus::Ready,
                    copy_status: crate::image_cache::ImageCacheCopyStatus::Skipped,
                    roots: vec!["/system/imagecache/disk".to_string()],
                },
            )))
            .unwrap();

        let stale_id = ResourceId::new("stale").unwrap();
        let mut stale = RegistriesConfigResource::new(RegistriesConfigSpec::default());
        *stale.metadata_mut() = Metadata::new(CRI_NAMESPACE, REGISTRIES_CONFIG_TYPE, stale_id);
        stale
            .metadata_mut()
            .set_owner(REGISTRIES_CONFIG_CONTROLLER_NAME);
        state.create(Box::new(stale)).unwrap();

        let applied = {
            let mut ctx = ReconcileContext::new(
                &mut state,
                REGISTRIES_CONFIG_CONTROLLER_NAME,
                registries_config_controller_spec(),
            );
            ctx.start_tracking_outputs();
            let applied = apply_registries_config_controller_inputs_to_context(&mut ctx).unwrap();
            let cleaned = ctx.cleanup_outputs().unwrap();

            assert_eq!(cleaned, 1);
            assert!(ctx.writes() >= 3);
            applied
        };

        assert_eq!(
            applied.mirrors["docker.io"].endpoints[0].endpoint,
            registryd_mirror_endpoint()
        );
        assert!(!state.contains("cri/RegistryConfigs.cri.talos.dev/stale"));

        let stored = state.get(&registries_config_key().unwrap()).unwrap();
        assert_eq!(stored.metadata().owner(), REGISTRIES_CONFIG_CONTROLLER_NAME);
        let stored = RegistriesConfigResource::from_resource(stored.as_ref()).unwrap();
        assert_eq!(
            stored.spec.mirrors["docker.io"].endpoints[1].endpoint,
            "https://cache.local/v2"
        );
    }

    #[test]
    fn registries_config_controller_runtime_reconciles_inputs_with_cleanup_outputs() {
        let mut state = os_cosi_domain::State::new();
        state
            .create(Box::new(TestMachineConfigDocument::new(
                r#"
version: v1alpha1
machine:
  registries:
    mirrors:
      registry.local:
        endpoints:
          - https://registry.local/cache
"#,
            )))
            .unwrap();
        state
            .create(Box::new(crate::image_cache::ImageCacheConfigResource::new(
                crate::image_cache::ImageCacheConfig {
                    status: crate::image_cache::ImageCacheStatus::Ready,
                    copy_status: crate::image_cache::ImageCacheCopyStatus::Skipped,
                    roots: vec!["/system/imagecache/disk".to_string()],
                },
            )))
            .unwrap();

        let mut stale = RegistriesConfigResource::new(RegistriesConfigSpec::default());
        *stale.metadata_mut() = Metadata::new(
            CRI_NAMESPACE,
            REGISTRIES_CONFIG_TYPE,
            ResourceId::new("stale").unwrap(),
        );
        stale
            .metadata_mut()
            .set_owner(REGISTRIES_CONFIG_CONTROLLER_NAME);
        state.create(Box::new(stale)).unwrap();

        let mut runtime = os_cosi_domain::Runtime::with_state(state);
        runtime
            .register(Box::new(RegistriesConfigController::new()))
            .unwrap();
        let passes = runtime.run().unwrap();

        assert_eq!(passes, 1);
        assert_eq!(runtime.history().len(), 1);
        assert!(runtime.history()[0].writes >= 3);
        assert!(
            !runtime
                .state()
                .contains("cri/RegistryConfigs.cri.talos.dev/stale")
        );

        let stored = runtime
            .state()
            .get(&registries_config_key().unwrap())
            .unwrap();
        assert_eq!(stored.metadata().owner(), REGISTRIES_CONFIG_CONTROLLER_NAME);
        let stored = RegistriesConfigResource::from_resource(stored.as_ref()).unwrap();
        let mirror = &stored.spec.mirrors["registry.local"];
        assert_eq!(mirror.endpoints[0].endpoint, registryd_mirror_endpoint());
        assert_eq!(mirror.endpoints[1].endpoint, "https://registry.local/cache");
    }

    #[test]
    fn registries_config_controller_event_pass_bootstrap_writes_empty_config_when_inputs_absent() {
        let mut runtime = os_cosi_domain::Runtime::new();
        runtime
            .register(Box::new(RegistriesConfigController::new()))
            .unwrap();

        let passes = runtime.run_event_pass().unwrap();

        assert_eq!(passes, 1);
        assert_eq!(
            runtime.history()[0].controller,
            REGISTRIES_CONFIG_CONTROLLER_NAME
        );
        let stored = runtime
            .state()
            .get(&registries_config_key().unwrap())
            .unwrap();
        assert_eq!(stored.metadata().owner(), REGISTRIES_CONFIG_CONTROLLER_NAME);
        let stored = RegistriesConfigResource::from_resource(stored.as_ref()).unwrap();
        assert!(stored.spec.mirrors.is_empty());
        assert!(stored.spec.auths.is_empty());
        assert!(stored.spec.tls.is_empty());

        let passes = runtime.run_event_pass().unwrap();
        assert_eq!(passes, 0);
        assert!(runtime.history().is_empty());
    }

    #[test]
    fn registries_config_controller_event_pass_returns_malformed_machine_config_error() {
        let mut state = os_cosi_domain::State::new();
        state
            .create(Box::new(TestMachineConfigDocument::new(
                r#"
version: v1alpha1
machine:
  registries:
    config:
      registry.local:
        tls:
          ca: not-base64*
"#,
            )))
            .unwrap();

        let mut runtime = os_cosi_domain::Runtime::with_state(state);
        runtime
            .register(Box::new(RegistriesConfigController::new()))
            .unwrap();

        let err = runtime.run_event_pass().unwrap_err();

        assert!(matches!(
            err,
            os_cosi_domain::RuntimeError::ControllerFailed {
                controller,
                error
            } if controller == REGISTRIES_CONFIG_CONTROLLER_NAME
                && error.contains("machine config")
                && error.contains("base64")
        ));
        assert_eq!(runtime.history().len(), 1);
        assert_eq!(
            runtime.history()[0].controller,
            REGISTRIES_CONFIG_CONTROLLER_NAME
        );
        assert!(!runtime.history()[0].ok);
    }

    #[test]
    fn registries_config_controller_event_pass_ignores_non_active_machine_config_events() {
        let mut runtime = os_cosi_domain::Runtime::new();
        runtime
            .register(Box::new(RegistriesConfigController::new()))
            .unwrap();

        assert_eq!(runtime.run_event_pass().unwrap(), 1);

        let inactive_machine_config = TestMachineConfigDocument {
            meta: Metadata::new(
                MACHINE_CONFIG_NAMESPACE,
                MACHINE_CONFIG_TYPE,
                ResourceId::new("inactive").unwrap(),
            ),
            contents: r#"
version: v1alpha1
machine:
  registries:
    mirrors:
      registry.local:
        endpoints:
          - https://registry.local/cache
"#
            .to_string(),
        };
        runtime
            .state_mut()
            .create(Box::new(inactive_machine_config))
            .unwrap();

        let passes = runtime.run_event_pass().unwrap();
        assert_eq!(passes, 0);
        assert!(runtime.history().is_empty());

        let stored = runtime
            .state()
            .get(&registries_config_key().unwrap())
            .unwrap();
        let stored = RegistriesConfigResource::from_resource(stored.as_ref()).unwrap();
        assert!(stored.spec.mirrors.is_empty());
        assert!(stored.spec.auths.is_empty());
        assert!(stored.spec.tls.is_empty());
    }

    #[test]
    fn pod_sandbox_config_validation() {
        assert!(PodSandboxConfig::new("", "ns", "uid").is_err());
        assert!(PodSandboxConfig::new("n", "", "uid").is_err());
        assert!(PodSandboxConfig::new("n", "ns", "").is_err());
        let c = cfg().with_host_network().with_label("app", "dns");
        assert!(c.host_network);
        assert_eq!(c.labels.get("app").map(String::as_str), Some("dns"));
    }

    #[test]
    fn full_pod_lifecycle() {
        let mut rt = CriRuntime::new();
        rt.pull_image(&pause()).unwrap();
        rt.pull_image(&app()).unwrap();

        let sb = rt.run_pod_sandbox(cfg()).unwrap();
        assert_eq!(
            rt.pod_sandbox_status(&sb).unwrap().state,
            PodSandboxState::Ready
        );

        let ccfg = CriContainerConfig::new("dns", app(), vec!["/coredns".to_string()]).unwrap();
        let cid = rt.create_container(&sb, ccfg).unwrap();
        assert_eq!(
            rt.container_status(&cid).unwrap().state,
            CriContainerState::Created
        );

        rt.start_container(&cid).unwrap();
        assert_eq!(
            rt.container_status(&cid).unwrap().state,
            CriContainerState::Running
        );

        // Cannot remove a running container, nor remove a ready sandbox.
        assert_eq!(
            rt.remove_container(&cid).unwrap_err().kind(),
            "invalid_state"
        );
        assert_eq!(
            rt.remove_pod_sandbox(&sb).unwrap_err().kind(),
            "invalid_state"
        );

        rt.stop_container(&cid, 0).unwrap();
        assert_eq!(rt.container_status(&cid).unwrap().exit_code, Some(0));
        rt.remove_container(&cid).unwrap();

        rt.stop_pod_sandbox(&sb).unwrap();
        assert_eq!(
            rt.pod_sandbox_status(&sb).unwrap().state,
            PodSandboxState::NotReady
        );
        rt.remove_pod_sandbox(&sb).unwrap();
        assert!(rt.pod_sandbox_status(&sb).is_none());
    }

    #[test]
    fn create_requires_pulled_image() {
        let mut rt = CriRuntime::new();
        let sb = rt.run_pod_sandbox(cfg()).unwrap();
        let ccfg = CriContainerConfig::new("dns", app(), vec!["/coredns".to_string()]).unwrap();
        assert_eq!(
            rt.create_container(&sb, ccfg).unwrap_err().kind(),
            "not_found"
        );
    }

    #[test]
    fn create_requires_ready_sandbox() {
        let mut rt = CriRuntime::new();
        rt.pull_image(&app()).unwrap();
        let sb = rt.run_pod_sandbox(cfg()).unwrap();
        rt.stop_pod_sandbox(&sb).unwrap();
        let ccfg = CriContainerConfig::new("dns", app(), vec!["/coredns".to_string()]).unwrap();
        assert_eq!(
            rt.create_container(&sb, ccfg).unwrap_err().kind(),
            "invalid_state"
        );
    }

    #[test]
    fn duplicate_container_name_rejected() {
        let mut rt = CriRuntime::new();
        rt.pull_image(&app()).unwrap();
        let sb = rt.run_pod_sandbox(cfg()).unwrap();
        let c1 = CriContainerConfig::new("dns", app(), vec!["/coredns".to_string()]).unwrap();
        let c2 = CriContainerConfig::new("dns", app(), vec!["/coredns".to_string()]).unwrap();
        rt.create_container(&sb, c1).unwrap();
        assert_eq!(
            rt.create_container(&sb, c2).unwrap_err().kind(),
            "invalid_state"
        );
    }

    #[test]
    fn stopping_sandbox_kills_running_containers() {
        let mut rt = CriRuntime::new();
        rt.pull_image(&app()).unwrap();
        let sb = rt.run_pod_sandbox(cfg()).unwrap();
        let ccfg = CriContainerConfig::new("dns", app(), vec!["/coredns".to_string()]).unwrap();
        let cid = rt.create_container(&sb, ccfg).unwrap();
        rt.start_container(&cid).unwrap();
        rt.stop_pod_sandbox(&sb).unwrap();
        let c = rt.container_status(&cid).unwrap();
        assert_eq!(c.state, CriContainerState::Exited);
        assert_eq!(c.exit_code, Some(137));
    }

    #[test]
    fn remove_sandbox_blocked_by_containers() {
        let mut rt = CriRuntime::new();
        rt.pull_image(&app()).unwrap();
        let sb = rt.run_pod_sandbox(cfg()).unwrap();
        let ccfg = CriContainerConfig::new("dns", app(), vec!["/coredns".to_string()]).unwrap();
        let cid = rt.create_container(&sb, ccfg).unwrap();
        rt.stop_pod_sandbox(&sb).unwrap();
        assert_eq!(rt.containers_in_sandbox(&sb), 1);
        assert_eq!(
            rt.remove_pod_sandbox(&sb).unwrap_err().kind(),
            "invalid_state"
        );
        rt.remove_container(&cid).unwrap();
        rt.remove_pod_sandbox(&sb).unwrap();
    }

    #[test]
    fn image_service_lifecycle() {
        let mut rt = CriRuntime::new();
        assert!(!rt.image_status(&pause()));
        let r = rt.pull_image(&pause()).unwrap();
        assert_eq!(r, "registry.k8s.io/pause:3.9");
        assert!(rt.image_status(&pause()));
        rt.pull_image(&app()).unwrap();
        assert_eq!(rt.list_images().len(), 2);
        rt.remove_image(&pause()).unwrap();
        assert!(!rt.image_status(&pause()));
        assert_eq!(rt.remove_image(&pause()).unwrap_err().kind(), "not_found");
    }

    #[test]
    fn start_twice_rejected() {
        let mut rt = CriRuntime::new();
        rt.pull_image(&app()).unwrap();
        let sb = rt.run_pod_sandbox(cfg()).unwrap();
        let ccfg = CriContainerConfig::new("dns", app(), vec!["/coredns".to_string()]).unwrap();
        let cid = rt.create_container(&sb, ccfg).unwrap();
        rt.start_container(&cid).unwrap();
        assert_eq!(
            rt.start_container(&cid).unwrap_err().kind(),
            "invalid_state"
        );
    }

    #[test]
    fn unknown_ids_not_found() {
        let mut rt = CriRuntime::new();
        assert_eq!(rt.stop_pod_sandbox("nope").unwrap_err().kind(), "not_found");
        assert_eq!(rt.start_container("nope").unwrap_err().kind(), "not_found");
        assert_eq!(
            rt.stop_container("nope", 0).unwrap_err().kind(),
            "not_found"
        );
        assert_eq!(rt.remove_container("nope").unwrap_err().kind(), "not_found");
        assert!(rt.container_status("nope").is_none());
    }

    #[test]
    fn list_sandboxes_sorted() {
        let mut rt = CriRuntime::new();
        let a = rt.run_pod_sandbox(cfg()).unwrap();
        let b = rt
            .run_pod_sandbox(PodSandboxConfig::new("p2", "ns", "uid2").unwrap())
            .unwrap();
        let list = rt.list_pod_sandboxes();
        assert_eq!(list.len(), 2);
        let mut expect = vec![a.as_str(), b.as_str()];
        expect.sort_unstable();
        assert_eq!(list, expect);
    }
}
