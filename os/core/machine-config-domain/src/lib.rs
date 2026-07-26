//! # talos-machine-config
//!
//! Machine configuration subsystem for the operating-system Talos migration. This crate
//! ports `siderolabs/talos` `pkg/machinery/config`: the machine configuration
//! schema, the multi-document config container, versioned loading/decoding
//! (`v1alpha1` plus multi-doc), validation, strategic / JSON6902 patching, and
//! the config-provider accessor traits.
//!
//! The crate uses only the standard library plus `talos-core`; it pulls in no
//! external crates so the build stays fully offline.
//!
//! ## Module map
//!
//! - [`document`] — the [`Document`] trait and document metadata
//!   (apiVersion/kind), shared by every config document.
//! - [`v1alpha1`] — the legacy single-document [`V1Alpha1Config`] schema and its
//!   [`MachineConfig`] / [`ClusterConfig`] sub-trees.
//! - [`machine`] / [`cluster`] — the machine and cluster sub-schemas, including
//!   [`InstallConfig`].
//! - [`container`] — the multi-document [`Config`] container that holds one
//!   v1alpha1 doc plus any number of additional documents.
//! - [`provider`] — the read-only [`Provider`] accessor trait.
//! - [`validation`] — the [`Validator`] trait and [`ValidationError`] taxonomy.
//! - [`patch`] — [`ConfigPatch`] (strategic merge + RFC6902 JSON patch ops).
//! - [`encoder`] — a tiny dependency-free document encoder/decoder modeling the
//!   YAML multi-doc boundary.
//! - [`secrets`] — the cluster [`Secrets`] bundle (PKI / tokens).
//! - [`registry`] — the multi-document [`Registry`] mapping document kinds to
//!   their cardinality / mode constraints.
//! - [`load`] — config-bytes load/save ([`load_from_bytes`] / [`save_to_bytes`])
//!   that decodes a multi-doc blob into a typed [`Config`] container.

// Pedantic documentation/`#[must_use]` lints are intentionally not annotated
// across this schema-heavy crate: the simple data accessors and infallible
// builders would need ~200 boilerplate `#[must_use]` attributes and `# Errors`
// doc sections that restate the obvious, adding noise without improving clarity.
#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]

pub mod cluster;
pub mod container;
pub mod corpus;
pub mod dhcpv4;
pub mod dhcpv6;
pub mod document;
pub mod encoder;
pub mod link_config;
pub mod load;
pub mod machine;
pub mod patch;
pub mod provider;
pub mod registry;
pub mod resolver;
pub mod secrets;
pub mod v1alpha1;
pub mod validation;
pub mod volume_config;
pub mod yaml;

pub use cluster::{ClusterConfig, ControlPlaneEndpoint};
pub use container::Config;
pub use corpus::{CorpusConfig, FIELD_PATH_ORDER, LoadError, Validity, load_record};
pub use dhcpv4::{
    DHCPV4_CONFIG_KIND, DhcpV4ClientIdentifier, DhcpV4Config, decode_dhcpv4_config_body,
    dhcpv4_configs,
};
pub use dhcpv6::{
    DHCPV6_CONFIG_KIND, DhcpV6ClientIdentifier, DhcpV6Config, decode_dhcpv6_config_body,
    dhcpv6_configs,
};
pub use document::{ConfigVersion, Document, DocumentMeta};
pub use encoder::{EncodedDocument, decode_documents, encode_documents};
pub use link_config::{
    AddressConfig as LinkAddressConfig, LINK_CONFIG_KIND, LinkConfig, LinkFields,
    RouteConfig as LinkRouteConfig, VLAN_CONFIG_KIND, VlanConfig, VlanMode,
    decode_link_config_body, decode_vlan_config_body, link_configs, vlan_configs,
};
pub use load::{detect_version, load_from_bytes, load_from_bytes_with, save_to_bytes};
pub use machine::{
    DhcpOptions, InstallConfig, MachineConfig, MachineFeatures, NetworkConfig, NetworkInterface,
    SystemDiskEncryption,
};
pub use patch::{ConfigPatch, JsonPatchOp, PatchOp};
pub use provider::Provider;
pub use registry::{Cardinality, KindSpec, Registry};
pub use resolver::{
    DnsProtocol as ResolverDnsProtocol, HostDnsConfig, NameserverConfig, RESOLVER_CONFIG_KIND,
    ResolverConfig, SearchDomainsConfig, decode_resolver_config_body, resolver_config,
};
pub use secrets::Secrets;
pub use v1alpha1::V1Alpha1Config;
pub use validation::{ValidationError, ValidationMode, Validator};
pub use volume_config::{
    EXISTING_VOLUME_CONFIG_KIND, EXISTING_VOLUME_PREFIX, EXTERNAL_VOLUME_CONFIG_KIND,
    EXTERNAL_VOLUME_PREFIX, EncryptionKeyProvider, EncryptionKeySpec, EncryptionSpec,
    ExistingVolumeConfigDoc, ExternalMountSpec, ExternalVolumeConfigDoc, ExternalVolumeFilesystem,
    IMAGE_CACHE_VOLUME_NAME, ImportedMountSpec, MIN_USER_VOLUME_SIZE, ProvisioningSpec,
    RAW_VOLUME_CONFIG_KIND, RAW_VOLUME_PREFIX, RawVolumeConfigDoc, SWAP_VOLUME_CONFIG_KIND,
    SWAP_VOLUME_PREFIX, SizeLimit, SwapVolumeConfigDoc, USER_VOLUME_CONFIG_KIND,
    USER_VOLUME_PREFIX, UserFilesystemSpec, UserMountSpec, UserVolumeConfigDoc,
    UserVolumeFilesystem, UserVolumeType, VOLUME_CONFIG_KIND, VolumeConfigDoc,
    decode_encryption_meta_value, decode_existing_volume_config_body,
    decode_external_volume_config_body, decode_raw_volume_config_body,
    decode_swap_volume_config_body, decode_user_volume_config_body, decode_volume_config_body,
    existing_volume_configs, external_volume_configs, raw_volume_configs, swap_volume_configs,
    user_volume_configs, volume_configs,
};

/// Convenience re-export of the shared error type.
pub use os_kernel::{Error, Result};
