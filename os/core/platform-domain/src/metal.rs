//! Metal config source, mirroring
//! `internal/app/machined/pkg/runtime/v1alpha1/platform/metal/metal.go`.
//!
//! On bare metal the config source is the kernel command line parameter
//! `talos.config=` (`constants.KernelParamConfig`):
//!
//! - unset → no config source (`ErrNoConfigSource`).
//! - `talos.config=none` (`constants.ConfigNone`) → no config source.
//! - `talos.config=metal-iso` (`constants.MetalConfigISOLabel`) → read
//!   `config.yaml` (`constants.ConfigFilename`) from the volume labeled
//!   `metal-iso`.
//! - `talos.config=<url>` → download from the (templated) URL. Talos
//!   interpolates variables in the URL (e.g. `${uuid}`, `${mac}`) via
//!   `metal/url.Populate`, and may attach OAuth2 bearer headers
//!   (`talos.config.oauth.*`). We model the raw URL plus any extra headers as
//!   data.
//!
//! [`Metal::mode`] is [`Mode::Metal`], or [`Mode::MetalAgent`] when the node is
//! running as a metal agent (`Metal{IsAgent: true}`).

use crate::metal_oauth::{OAuthConfig, OAuthConfigError, new_config as new_oauth_config};
use crate::metal_url::{UrlPopulateError, UrlVariableValues, populate_url};
use crate::source::{ConfigSource, Header};
use crate::{Mode, Platform};
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

/// Kernel parameter that carries the metal config source
/// (`constants.KernelParamConfig`).
pub const KERNEL_PARAM_CONFIG: &str = "talos.config";

/// Sentinel value disabling config fetch (`constants.ConfigNone`).
pub const CONFIG_NONE: &str = "none";

/// Sentinel value selecting the ISO volume source (`constants.MetalConfigISOLabel`).
pub const METAL_ISO_LABEL: &str = "metal-iso";

/// Config file name read from the `metal-iso` volume (`constants.ConfigFilename`).
pub const CONFIG_FILENAME: &str = "config.yaml";

/// Metal platform config source.
///
/// Mirrors `metal.Metal`.
#[derive(Debug, Default, Clone)]
pub struct Metal {
    /// The `talos.config=` value from the kernel cmdline, if present.
    config: Option<String>,
    /// Extra request headers (e.g. OAuth2 bearer) for URL-based fetch.
    headers: Vec<Header>,
    /// Whether this node runs as a metal agent (changes [`Mode`]).
    is_agent: bool,
}

impl Metal {
    /// Construct a metal source with no `talos.config=` on the cmdline.
    pub fn new() -> Self {
        Metal::default()
    }

    /// Construct from a `talos.config=` kernel cmdline value.
    pub fn from_cmdline(value: impl Into<String>) -> Self {
        Metal {
            config: Some(value.into()),
            headers: Vec::new(),
            is_agent: false,
        }
    }

    /// Attach extra request headers (e.g. OAuth2 `Authorization`) for URL fetch.
    pub fn with_headers(mut self, headers: Vec<Header>) -> Self {
        self.headers = headers;
        self
    }

    /// Populate Talos metal URL variables (`${uuid}`, legacy `?uuid=`, etc.)
    /// in a URL-based `talos.config=` value.
    ///
    /// Upstream waits for runtime resources before doing this replacement. The
    /// Rust model keeps the same URL rules but asks callers to provide already
    /// discovered values explicitly.
    pub fn with_url_values(mut self, values: &UrlVariableValues) -> Result<Self, UrlPopulateError> {
        if let Some(config) = self.config.as_deref()
            && config != CONFIG_NONE
            && config != METAL_ISO_LABEL
        {
            self.config = Some(populate_url(config, values)?);
        }

        Ok(self)
    }

    /// Parse OAuth2 settings from the kernel command line for this URL-based
    /// metal config source.
    ///
    /// Upstream only consults `talos.config.oauth.*` for URL downloads, not for
    /// `none`, an absent config, or `metal-iso`. This helper mirrors that gate
    /// and leaves the live device authorization flow to a later networking
    /// layer.
    pub fn oauth_config_from_cmdline(
        &self,
        cmdline: &str,
    ) -> Result<Option<OAuthConfig>, OAuthConfigError> {
        match self.config.as_deref() {
            Some(config) if config != CONFIG_NONE && config != METAL_ISO_LABEL => {
                match new_oauth_config(cmdline, config) {
                    Ok(config) => Ok(Some(config)),
                    Err(OAuthConfigError::NotConfigured) => Ok(None),
                    Err(err) => Err(err),
                }
            }
            _ => Ok(None),
        }
    }

    /// Mark this node as a metal agent (`Metal{IsAgent: true}`).
    pub fn as_agent(mut self) -> Self {
        self.is_agent = true;
        self
    }
}

impl Platform for Metal {
    fn name(&self) -> &str {
        // constants.PlatformMetal
        "metal"
    }

    fn mode(&self) -> Mode {
        if self.is_agent {
            Mode::MetalAgent
        } else {
            Mode::Metal
        }
    }

    fn config_sources(&self) -> Vec<ConfigSource> {
        match self.config.as_deref() {
            // unset or explicit "none" → no config source.
            None | Some(CONFIG_NONE) => Vec::new(),
            // ISO sentinel → read config.yaml from the metal-iso volume.
            Some(METAL_ISO_LABEL) => {
                vec![ConfigSource::disk(&[METAL_ISO_LABEL], CONFIG_FILENAME)]
            }
            // anything else is a download URL.
            Some(url) => vec![ConfigSource::http(url.to_owned(), self.headers.clone())],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{ConfigStore, MemoryStore};

    #[test]
    fn name_and_mode() {
        let p = Metal::new();
        assert_eq!(p.name(), "metal");
        assert_eq!(p.mode(), Mode::Metal);
    }

    #[test]
    fn agent_mode() {
        let p = Metal::from_cmdline("https://x/config.yaml").as_agent();
        assert_eq!(p.mode(), Mode::MetalAgent);
    }

    #[test]
    fn no_cmdline_means_no_source() {
        assert!(Metal::new().config_sources().is_empty());
    }

    #[test]
    fn config_none_means_no_source() {
        assert!(Metal::from_cmdline("none").config_sources().is_empty());
    }

    #[test]
    fn metal_iso_reads_config_yaml_from_labeled_volume() {
        let p = Metal::from_cmdline("metal-iso");
        let src = &p.config_sources()[0];
        assert_eq!(src.labels(), &["metal-iso"]);
        assert_eq!(src.path(), Some("config.yaml"));
    }

    #[test]
    fn url_source_is_faithful() {
        let p = Metal::from_cmdline("https://pxe.local/configs/node.yaml");
        let src = &p.config_sources()[0];
        assert_eq!(src.url(), Some("https://pxe.local/configs/node.yaml"));
        assert!(src.headers().is_empty());
    }

    #[test]
    fn oauth_headers_attached_to_url_source() {
        let p = Metal::from_cmdline("https://pxe.local/node.yaml")
            .with_headers(vec![Header::new("Authorization", "Bearer abc123")]);
        let src = &p.config_sources()[0];
        assert_eq!(src.header("Authorization"), Some("Bearer abc123"));
    }

    #[test]
    fn url_variables_are_populated_before_source_is_reported() {
        let values = UrlVariableValues::new()
            .with_uuid("0000-0000")
            .with_mac("12:34:56:78:90:ab");
        let p = Metal::from_cmdline("https://pxe.local/node?uuid=&mac=${mac}")
            .with_url_values(&values)
            .unwrap();
        let src = &p.config_sources()[0];
        assert_eq!(
            src.url(),
            Some("https://pxe.local/node?mac=12%3A34%3A56%3A78%3A90%3Aab&uuid=0000-0000")
        );
    }

    #[test]
    fn oauth_config_is_available_for_url_sources_only() {
        let p = Metal::from_cmdline("https://example.com/my/config");
        let cfg = p
            .oauth_config_from_cmdline("talos.config.oauth.client_id=device_client_id")
            .unwrap()
            .unwrap();

        assert_eq!(cfg.client_id, "device_client_id");
        assert_eq!(cfg.device_auth_url, "https://example.com/device/code");
        assert_eq!(cfg.token_url, "https://example.com/token");

        assert_eq!(
            Metal::from_cmdline("metal-iso")
                .oauth_config_from_cmdline("talos.config.oauth.client_id=device_client_id")
                .unwrap(),
            None
        );
    }

    #[test]
    fn configuration_downloads_from_url() {
        let p = Metal::from_cmdline("https://pxe.local/node.yaml");
        let store = MemoryStore::new().with(&p.config_sources()[0], b"version: v1alpha1".to_vec());
        assert_eq!(
            p.configuration(&store as &dyn ConfigStore).unwrap(),
            b"version: v1alpha1".to_vec()
        );
    }

    #[test]
    fn configuration_reads_from_metal_iso() {
        let p = Metal::from_cmdline("metal-iso");
        let store = MemoryStore::new().with(&p.config_sources()[0], b"machine: {}".to_vec());
        assert_eq!(
            p.configuration(&store as &dyn ConfigStore).unwrap(),
            b"machine: {}".to_vec()
        );
    }

    #[test]
    fn configuration_errors_when_unset() {
        let p = Metal::new();
        let err = p
            .configuration(&MemoryStore::new() as &dyn ConfigStore)
            .unwrap_err();
        assert_eq!(err, crate::no_config_source());
    }
}
