//! Azure config source, mirroring
//! `internal/app/machined/pkg/runtime/v1alpha1/platform/azure/{azure,metadata}.go`.
//!
//! Azure is a hybrid. The **machine config** itself is *not* in IMDS — upstream
//! `Azure.Configuration` calls `configFromCD()`, which mounts the provisioning
//! CD-ROM (a `udf` volume on `/dev/sr0` etc.) and reads `ovf-env.xml`, then
//! base64-decodes the `<CustomData>` element. So the primary [`ConfigSource`]
//! here is the CD-ROM file `ovf-env.xml`.
//!
//! Azure IMDS *is* used for network metadata and is encoded here as data for
//! faithfulness. The IMDS endpoints all require the `Metadata: true` header
//! (`AzureMetadataEndpoint` / `AzureInterfacesEndpoint` / `AzureLoadbalancerEndpoint`).

use crate::source::{ConfigSource, Header};
use crate::{Mode, Platform};
use alloc::vec;
use alloc::vec::Vec;

/// Provisioning CD-ROM file holding base64 `CustomData` (the machine config).
pub const OVF_ENV_FILE: &str = "ovf-env.xml";

/// Azure IMDS metadata service version (`AzureVersion`).
pub const IMDS_VERSION: &str = "2021-12-13";

/// Azure IMDS fallback version, e.g. Azure Stack Hub (`AzureVersionFallback`).
pub const IMDS_VERSION_FALLBACK: &str = "2019-06-01";

/// IMDS compute metadata endpoint template (`AzureMetadataEndpoint`), `%s` = api-version.
pub const COMPUTE_METADATA_ENDPOINT: &str =
    "http://169.254.169.254/metadata/instance/compute?api-version=%s&format=json";

/// IMDS network interfaces endpoint template (`AzureInterfacesEndpoint`).
pub const INTERFACES_ENDPOINT: &str =
    "http://169.254.169.254/metadata/instance/network/interface?api-version=%s&format=json";

/// IMDS load-balancer endpoint template (`AzureLoadbalancerEndpoint`).
pub const LOADBALANCER_ENDPOINT: &str =
    "http://169.254.169.254/metadata/loadbalancer?api-version=%s&format=json";

/// Azure Internal Channel endpoint (`AzureInternalEndpoint`, 168.63.129.16),
/// used by the WALinuxAgent goalstate/health flow.
pub const INTERNAL_ENDPOINT: &str = "http://168.63.129.16";

/// Required IMDS header name.
pub const METADATA_HEADER: &str = "Metadata";

/// Required IMDS header value.
pub const METADATA_VALUE: &str = "true";

/// Build an IMDS endpoint URL by substituting the api-version into a `%s` template.
fn with_version(template: &str, version: &str) -> alloc::string::String {
    template.replacen("%s", version, 1)
}

/// The IMDS compute-metadata endpoint for the default api-version, with header.
pub fn compute_metadata_source() -> ConfigSource {
    ConfigSource::http(
        with_version(COMPUTE_METADATA_ENDPOINT, IMDS_VERSION),
        vec![Header::new(METADATA_HEADER, METADATA_VALUE)],
    )
}

/// The IMDS network-interfaces endpoint for the default api-version, with header.
pub fn interfaces_source() -> ConfigSource {
    ConfigSource::http(
        with_version(INTERFACES_ENDPOINT, IMDS_VERSION),
        vec![Header::new(METADATA_HEADER, METADATA_VALUE)],
    )
}

/// Azure platform config source.
///
/// Mirrors `azure.Azure`. [`Mode`] is [`Mode::Cloud`].
#[derive(Debug, Default, Clone, Copy)]
pub struct Azure;

impl Azure {
    /// Construct the Azure source.
    pub fn new() -> Self {
        Azure
    }
}

impl Platform for Azure {
    fn name(&self) -> &str {
        "azure"
    }

    fn mode(&self) -> Mode {
        Mode::Cloud
    }

    fn config_sources(&self) -> Vec<ConfigSource> {
        // Machine config comes from the provisioning CD-ROM ovf-env.xml.
        // Talos matches devices /dev/(sr[0-9]|hd[c-z]|cdrom[0-9]|cd[0-9]); we
        // model the volume by the file it must contain rather than a label.
        vec![ConfigSource::disk(&["sr0", "cdrom0", "cd0"], OVF_ENV_FILE)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{ConfigStore, MemoryStore};

    #[test]
    fn name_and_mode() {
        let p = Azure::new();
        assert_eq!(p.name(), "azure");
        assert_eq!(p.mode(), Mode::Cloud);
    }

    #[test]
    fn config_from_cdrom_ovf_env() {
        let src = &Azure::new().config_sources()[0];
        assert_eq!(src.path(), Some("ovf-env.xml"));
    }

    #[test]
    fn compute_metadata_endpoint_is_faithful() {
        let src = compute_metadata_source();
        assert_eq!(
            src.url(),
            Some(
                "http://169.254.169.254/metadata/instance/compute?api-version=2021-12-13&format=json"
            )
        );
        assert_eq!(src.header("Metadata"), Some("true"));
    }

    #[test]
    fn interfaces_endpoint_is_faithful() {
        let src = interfaces_source();
        assert_eq!(
            src.url(),
            Some(
                "http://169.254.169.254/metadata/instance/network/interface?api-version=2021-12-13&format=json"
            )
        );
        assert_eq!(src.header("Metadata"), Some("true"));
    }

    #[test]
    fn fallback_version_substitutes() {
        let url = with_version(COMPUTE_METADATA_ENDPOINT, IMDS_VERSION_FALLBACK);
        assert!(url.contains("api-version=2019-06-01"));
    }

    #[test]
    fn configuration_reads_custom_data_from_cd() {
        let p = Azure::new();
        // CustomData is base64 upstream; the decoded bytes are what we store.
        let store = MemoryStore::new().with(
            &p.config_sources()[0],
            b"version: v1alpha1\nmachine: {}".to_vec(),
        );
        assert_eq!(
            p.configuration(&store as &dyn ConfigStore).unwrap(),
            b"version: v1alpha1\nmachine: {}".to_vec()
        );
    }

    #[test]
    fn configuration_errors_without_cd() {
        let p = Azure::new();
        let err = p
            .configuration(&MemoryStore::new() as &dyn ConfigStore)
            .unwrap_err();
        assert_eq!(err, crate::no_config_source());
    }
}
