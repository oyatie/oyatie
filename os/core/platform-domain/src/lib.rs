#![cfg_attr(not(test), no_std)]
// This crate models a broad surface of small data accessors describing each
// cloud platform's config-source endpoint shape. The pedantic lints below would
// require annotating many trivial methods without making the API clearer.
#![allow(
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::module_name_repetitions
)]
//! # talos-platform
//!
//! The cloud-native **config-source abstraction**: how a Talos/operating-system node
//! discovers its machine configuration at boot, per platform.
//!
//! This crate mirrors
//! `internal/app/machined/pkg/runtime/v1alpha1/platform/*` in
//! `siderolabs/talos`. Each cloud or metal provider answers one question:
//! *given this environment, where do the machine-config bytes come from?*
//!
//! Upstream that question is answered by the `runtime.Platform` interface
//! (`internal/app/machined/pkg/runtime/platform.go`):
//!
//! ```go
//! type Platform interface {
//!     Name() string
//!     Mode() Mode
//!     Configuration(context.Context, state.State) ([]byte, error)
//!     // ... KernelArgs, NetworkConfiguration
//! }
//! ```
//!
//! Here we model the config-source slice of that interface with the
//! [`Platform`] trait: [`Platform::name`], [`Platform::mode`], and
//! [`Platform::configuration`]. The faithful per-platform endpoint shapes
//! (URLs, HTTP headers, ISO labels, file paths, kernel `talos.config=`) are
//! encoded as **data** via [`ConfigSource`], so they can be asserted in unit
//! tests without any real network.
//!
//! Because there is no network in this environment, the actual fetch is
//! performed against a [`ConfigStore`] — an in-memory or file-backed fake that
//! maps a [`ConfigSource`] to bytes.
//!
//! ## Platforms modeled
//!
//! | Platform | [`Mode`] | config source |
//! |----------|----------|---------------|
//! | [`nocloud::NoCloud`] | cloud | cidata CD-ROM (`user-data`) or SMBIOS-seeded HTTP `meta-base/user-data` |
//! | [`aws::Aws`]   | cloud | IMDS `http://169.254.169.254/latest/user-data` |
//! | [`gcp::Gcp`]   | cloud | metadata server `instance/attributes/user-data` + `Metadata-Flavor: Google` |
//! | [`azure::Azure`] | cloud | IMDS compute metadata + `ovf-env.xml` CustomData on CD-ROM |
//! | [`metal::Metal`] | metal | kernel cmdline `talos.config=<url>` or `metal-iso` volume |

extern crate alloc;

pub mod aws;
pub mod azure;
pub mod gcp;
pub mod metal;
pub mod metal_oauth;
pub mod metal_url;
pub mod nocloud;
pub mod source;
pub mod store;

pub use source::{ConfigSource, Header};
pub use store::{ConfigStore, MemoryStore};

use os_kernel::error::Result;

/// Platform runtime mode, mirroring `runtime.Mode` in
/// `internal/app/machined/pkg/runtime/mode.go`.
///
/// Only the variants relevant to the config-source abstraction are modeled.
/// Upstream order: `ModeCloud, ModeContainer, ModeMetal, ModeMetalAgent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Cloud runtime mode (`runtime.ModeCloud`) — config comes from an
    /// instance metadata service or provider CD-ROM.
    Cloud,
    /// Container runtime mode (`runtime.ModeContainer`).
    Container,
    /// Bare-metal runtime mode (`runtime.ModeMetal`).
    Metal,
    /// Metal agent runtime mode (`runtime.ModeMetalAgent`).
    MetalAgent,
}

impl Mode {
    /// Canonical lowercase name, matching `Mode.String()` upstream.
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Cloud => "cloud",
            Mode::Container => "container",
            Mode::Metal => "metal",
            Mode::MetalAgent => "metal-agent",
        }
    }
}

/// The config-source abstraction: how a node gets its machine config for a
/// given platform.
///
/// Mirrors the config-discovery slice of `runtime.Platform`. Implementors are
/// pure descriptions of *where* the config lives; the actual bytes are read
/// from a [`ConfigStore`] fake so no network is required here.
pub trait Platform {
    /// Platform name, e.g. `"aws"`, `"gcp"`, `"metal"`.
    ///
    /// Mirrors `Platform.Name()`.
    fn name(&self) -> &str;

    /// Platform [`Mode`] (metal / cloud / ...).
    ///
    /// Mirrors `Platform.Mode()`.
    fn mode(&self) -> Mode;

    /// The ordered candidate [`ConfigSource`]s this platform consults to find
    /// the machine config, most-preferred first.
    ///
    /// This is the faithfully-encoded endpoint data (URL / headers / path /
    /// ISO label / kernel arg) per the Go source. It is what the unit tests
    /// assert against.
    fn config_sources(&self) -> alloc::vec::Vec<ConfigSource>;

    /// Fetch the machine-config bytes for this platform from `store`.
    ///
    /// Mirrors `Platform.Configuration(ctx, state)`: walks [`config_sources`]
    /// in order and returns the first that resolves to non-empty bytes.
    /// Returns [`os_kernel::error::Error::NotFound`] (modeling upstream
    /// `errors.ErrNoConfigSource`) when no source yields config.
    ///
    /// [`config_sources`]: Platform::config_sources
    fn configuration(&self, store: &dyn ConfigStore) -> Result<alloc::vec::Vec<u8>> {
        for source in self.config_sources() {
            if let Some(bytes) = store.fetch(&source)
                && !bytes.iter().all(u8::is_ascii_whitespace) && !bytes.is_empty() {
                    return Ok(bytes);
                }
        }

        Err(no_config_source())
    }
}

/// The error returned when no config source yields configuration, mirroring
/// upstream `platform/errors.ErrNoConfigSource`
/// (`"no configuration source"`).
pub fn no_config_source() -> os_kernel::error::Error {
    os_kernel::error::Error::not_found("no configuration source")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_names() {
        assert_eq!(Mode::Cloud.as_str(), "cloud");
        assert_eq!(Mode::Metal.as_str(), "metal");
        assert_eq!(Mode::MetalAgent.as_str(), "metal-agent");
        assert_eq!(Mode::Container.as_str(), "container");
    }
}
