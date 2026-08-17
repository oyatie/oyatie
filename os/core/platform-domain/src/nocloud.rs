//! NoCloud config source, mirroring
//! `internal/app/machined/pkg/runtime/v1alpha1/platform/nocloud/{nocloud,metadata}.go`.
//!
//! NoCloud (cloud-init's "NoCloud" datasource) gets its config from one of two
//! places, decided by the SMBIOS system serial number
//! (`s.SystemInformation.SerialNumber`), which Talos parses as `;`-separated
//! `key=value` options:
//!
//! - `ds=nocloud-net` + `s=<http base url>` → fetch over HTTP from the seed
//!   server: `meta-base/{meta-data,network-config,user-data}` (trailing slash
//!   added to the base if absent). The machine config is `user-data`.
//! - otherwise → read from a mounted CD-ROM/disk with filesystem label
//!   `cidata` (Talos probes both lower- and upper-case), files
//!   `meta-data`, `network-config`, `user-data`.
//!
//! Per `nocloud.Configuration`, a `user-data` whose first line is
//! `#cloud-config` is rejected (Talos does not support cloud-config), and a
//! first line of `#include` triggers an include-fetch. We model the
//! `#cloud-config` rejection in [`NoCloud::configuration`].

use crate::source::ConfigSource;
use crate::store::ConfigStore;
use crate::{Mode, Platform};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use os_kernel::error::Result;

/// CD-ROM / disk filesystem volume label (`configISOLabel`).
pub const ISO_LABEL: &str = "cidata";

/// meta-data file name (`configMetaDataPath`).
pub const META_DATA_PATH: &str = "meta-data";

/// network-config file name (`configNetworkConfigPath`).
pub const NETWORK_CONFIG_PATH: &str = "network-config";

/// user-data file name — holds the machine config (`configUserDataPath`).
pub const USER_DATA_PATH: &str = "user-data";

/// SMBIOS serial option requesting the network datasource (`ds=nocloud-net`).
pub const DS_NETWORK: &str = "nocloud-net";

/// Where NoCloud reads its seed from, derived from the SMBIOS serial.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Seed {
    /// HTTP seed server base URL (normalized to end with `/`).
    Network(String),
    /// CD-ROM / disk labeled `cidata`.
    Cidata,
}

/// NoCloud platform config source.
///
/// Mirrors `nocloud.Nocloud`. [`Mode`] is [`Mode::Cloud`].
#[derive(Debug, Clone)]
pub struct NoCloud {
    seed: Seed,
}

impl Default for NoCloud {
    fn default() -> Self {
        NoCloud::from_cidata()
    }
}

impl NoCloud {
    /// Construct a CD-ROM (`cidata`) backed NoCloud source.
    pub fn from_cidata() -> Self {
        NoCloud { seed: Seed::Cidata }
    }

    /// Construct a network-seeded NoCloud source from a base URL. The base is
    /// normalized to end with `/` exactly as `acquireConfig` does.
    pub fn from_network(base_url: impl Into<String>) -> Self {
        let mut base = base_url.into();

        if !base.ends_with('/') {
            base.push('/');
        }

        NoCloud {
            seed: Seed::Network(base),
        }
    }

    /// Parse a SMBIOS system serial number (`;`-separated `key=value` options)
    /// into a NoCloud source, mirroring `acquireConfig`.
    ///
    /// Recognized keys: `ds` (`nocloud-net` selects network), `s` (http base
    /// URL). When `ds=nocloud-net` and a valid `s=<http...>` base URL is given,
    /// the network seed is used; otherwise the CD-ROM (`cidata`).
    pub fn from_smbios_serial(serial: &str) -> Self {
        let mut network_source = false;
        let mut meta_base: Option<String> = None;

        for option in serial.split(';') {
            if let Some((key, value)) = option.split_once('=') {
                match key {
                    "ds" if value == DS_NETWORK => {
                        network_source = true;
                    }
                    "s" if value.starts_with("http") => {
                        let mut base = String::from(value);

                        if !base.ends_with('/') {
                            base.push('/');
                        }

                        meta_base = Some(base);
                    }
                    _ => {}
                }
            }
        }

        match (network_source, meta_base) {
            (true, Some(base)) => NoCloud {
                seed: Seed::Network(base),
            },
            _ => NoCloud::from_cidata(),
        }
    }

    /// The HTTP base URL when network-seeded.
    pub fn meta_base_url(&self) -> Option<&str> {
        match &self.seed {
            Seed::Network(base) => Some(base),
            Seed::Cidata => None,
        }
    }
}

impl Platform for NoCloud {
    fn name(&self) -> &str {
        "nocloud"
    }

    fn mode(&self) -> Mode {
        Mode::Cloud
    }

    fn config_sources(&self) -> Vec<ConfigSource> {
        match &self.seed {
            Seed::Network(base) => {
                vec![ConfigSource::http_no_headers(alloc::format!(
                    "{base}{USER_DATA_PATH}"
                ))]
            }
            Seed::Cidata => {
                // Talos probes both lower- and upper-case label variants.
                vec![ConfigSource::disk(&["cidata", "CIDATA"], USER_DATA_PATH)]
            }
        }
    }

    fn configuration(&self, store: &dyn ConfigStore) -> Result<Vec<u8>> {
        for source in self.config_sources() {
            if let Some(bytes) = store.fetch(&source) {
                if bytes.is_empty() || bytes.iter().all(u8::is_ascii_whitespace) {
                    continue;
                }

                // Mirror nocloud.Configuration: reject #cloud-config, which
                // Talos does not support.
                let first_line = bytes.split(|&b| b == b'\n').next().unwrap_or(&[]);
                let trimmed: &[u8] = {
                    let start = first_line
                        .iter()
                        .position(|b| !b.is_ascii_whitespace())
                        .unwrap_or(first_line.len());
                    let end = first_line
                        .iter()
                        .rposition(|b| !b.is_ascii_whitespace())
                        .map_or(start, |p| p + 1);
                    &first_line[start..end]
                };

                if trimmed == b"#cloud-config" {
                    return Err(crate::no_config_source());
                }

                return Ok(bytes);
            }
        }

        Err(crate::no_config_source())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryStore;

    #[test]
    fn name_and_mode() {
        let p = NoCloud::from_cidata();
        assert_eq!(p.name(), "nocloud");
        assert_eq!(p.mode(), Mode::Cloud);
    }

    #[test]
    fn cidata_disk_source_is_faithful() {
        let p = NoCloud::from_cidata();
        let src = &p.config_sources()[0];
        assert_eq!(src.path(), Some("user-data"));
        assert_eq!(src.labels(), &["cidata", "CIDATA"]);
    }

    #[test]
    fn network_user_data_url_is_faithful() {
        let p = NoCloud::from_network("http://10.0.0.1/seed");
        let src = &p.config_sources()[0];
        // base normalized with a trailing slash, then user-data appended
        assert_eq!(src.url(), Some("http://10.0.0.1/seed/user-data"));
    }

    #[test]
    fn network_url_keeps_existing_trailing_slash() {
        let p = NoCloud::from_network("http://10.0.0.1/seed/");
        assert_eq!(p.meta_base_url(), Some("http://10.0.0.1/seed/"));
        assert_eq!(
            p.config_sources()[0].url(),
            Some("http://10.0.0.1/seed/user-data")
        );
    }

    #[test]
    fn smbios_serial_selects_network() {
        let p = NoCloud::from_smbios_serial("ds=nocloud-net;s=http://seed.local/ds;h=node1;i=i-1");
        assert_eq!(p.meta_base_url(), Some("http://seed.local/ds/"));
        assert_eq!(
            p.config_sources()[0].url(),
            Some("http://seed.local/ds/user-data")
        );
    }

    #[test]
    fn smbios_serial_without_net_uses_cidata() {
        // s= present but ds is not nocloud-net -> CD-ROM
        let p = NoCloud::from_smbios_serial("s=http://seed.local/ds");
        assert!(p.meta_base_url().is_none());
        assert!(matches!(&p.config_sources()[0], ConfigSource::Disk { .. }));
    }

    #[test]
    fn smbios_serial_non_http_base_falls_back_to_cidata() {
        let p = NoCloud::from_smbios_serial("ds=nocloud-net;s=ftp://nope/");
        assert!(p.meta_base_url().is_none());
    }

    #[test]
    fn configuration_reads_user_data_from_cidata() {
        let p = NoCloud::from_cidata();
        let store = MemoryStore::new().with(&p.config_sources()[0], b"version: v1alpha1".to_vec());
        assert_eq!(
            p.configuration(&store as &dyn ConfigStore).unwrap(),
            b"version: v1alpha1".to_vec()
        );
    }

    #[test]
    fn configuration_rejects_cloud_config() {
        let p = NoCloud::from_cidata();
        let store =
            MemoryStore::new().with(&p.config_sources()[0], b"#cloud-config\nfoo: bar".to_vec());
        let err = p.configuration(&store as &dyn ConfigStore).unwrap_err();
        assert_eq!(err, crate::no_config_source());
    }

    #[test]
    fn configuration_errors_without_seed() {
        let p = NoCloud::from_cidata();
        let err = p
            .configuration(&MemoryStore::new() as &dyn ConfigStore)
            .unwrap_err();
        assert_eq!(err, crate::no_config_source());
    }
}
