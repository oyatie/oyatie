//! GCP config source, mirroring
//! `internal/app/machined/pkg/runtime/v1alpha1/platform/gcp/{gcp,metadata}.go`.
//!
//! On GCP the machine config is the instance attribute **`user-data`** fetched
//! from the metadata server. Upstream uses `cloud.google.com/go/compute/metadata`
//! (`metadata.InstanceAttributeValueWithContext(ctx, "user-data")`), which the
//! library resolves to:
//!
//! `GET http://metadata.google.internal/computeMetadata/v1/instance/attributes/user-data`
//!
//! Every request to the GCP metadata server **must** carry the
//! `Metadata-Flavor: Google` header (the library sets it unconditionally), which
//! we encode as data here.

use crate::source::{ConfigSource, Header};
use crate::{Mode, Platform};
use alloc::vec;
use alloc::vec::Vec;

/// GCP metadata server host. `cloud.google.com/go/compute/metadata` defaults to
/// `metadata.google.internal`.
pub const METADATA_HOST: &str = "metadata.google.internal";

/// Metadata API base path (`computeMetadata/v1`).
pub const METADATA_BASE: &str = "http://metadata.google.internal/computeMetadata/v1";

/// Path to the `user-data` instance attribute holding the machine config.
pub const USER_DATA_PATH: &str = "/instance/attributes/user-data";

/// Required metadata-server header name.
pub const FLAVOR_HEADER: &str = "Metadata-Flavor";

/// Required metadata-server header value.
pub const FLAVOR_VALUE: &str = "Google";

/// GCP platform config source.
///
/// Mirrors `gcp.GCP`. [`Mode`] is [`Mode::Cloud`].
#[derive(Debug, Default, Clone, Copy)]
pub struct Gcp;

impl Gcp {
    /// Construct the GCP source.
    pub fn new() -> Self {
        Gcp
    }
}

impl Platform for Gcp {
    fn name(&self) -> &str {
        "gcp"
    }

    fn mode(&self) -> Mode {
        Mode::Cloud
    }

    fn config_sources(&self) -> Vec<ConfigSource> {
        vec![ConfigSource::http(
            alloc::format!("{METADATA_BASE}{USER_DATA_PATH}"),
            vec![Header::new(FLAVOR_HEADER, FLAVOR_VALUE)],
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{ConfigStore, MemoryStore};

    #[test]
    fn name_and_mode() {
        let p = Gcp::new();
        assert_eq!(p.name(), "gcp");
        assert_eq!(p.mode(), Mode::Cloud);
    }

    #[test]
    fn user_data_endpoint_is_faithful() {
        let src = &Gcp::new().config_sources()[0];
        assert_eq!(
            src.url(),
            Some(
                "http://metadata.google.internal/computeMetadata/v1/instance/attributes/user-data"
            )
        );
    }

    #[test]
    fn metadata_flavor_header_required() {
        let src = &Gcp::new().config_sources()[0];
        assert_eq!(src.header("Metadata-Flavor"), Some("Google"));
        // header name case is preserved from the Go source
        assert_eq!(src.headers()[0].name, "Metadata-Flavor");
    }

    #[test]
    fn configuration_reads_attribute() {
        let p = Gcp::new();
        let store = MemoryStore::new().with(&p.config_sources()[0], b"version: v1alpha1".to_vec());
        assert_eq!(
            p.configuration(&store as &dyn ConfigStore).unwrap(),
            b"version: v1alpha1".to_vec()
        );
    }

    #[test]
    fn configuration_errors_when_absent() {
        let p = Gcp::new();
        let err = p
            .configuration(&MemoryStore::new() as &dyn ConfigStore)
            .unwrap_err();
        assert_eq!(err, crate::no_config_source());
    }
}
