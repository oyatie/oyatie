//! Well-known META key (tag) identifiers.
//!
//! In Talos every persisted value on the META partition is addressed by a
//! numeric ADV tag. A subset of those tags is reserved for first-party machine
//! state and given symbolic names in `pkg/machinery/meta/constants.go`. This
//! module models those reserved keys plus arbitrary user-defined tags.
//!
//! The numeric values mirror the constants used by Talos so that an ADV blob
//! produced here decodes to the same logical keys. The full first-party set
//! modelled here is:
//!
//! | Tag    | Name                          | Meaning                                  |
//! |--------|-------------------------------|------------------------------------------|
//! | `0x06` | `Upgrade`                     | previous version, for rollback           |
//! | `0x07` | `StagedUpgradeImageRef`       | installer image to boot into             |
//! | `0x08` | `StagedUpgradeInstallOptions` | serialized install options for upgrade   |
//! | `0x09` | `StateEncryptionConfig`       | STATE-partition encryption config        |
//! | `0x0a` | `MetalNetworkPlatformConfig`  | bare-metal network platform config       |
//! | `0x0b` | `UUIDOverride`                | override for the SMBIOS machine UUID      |
//! | `0x0c` | `UserDiskConfig`              | legacy user-disk partition config        |
//! | `0x0d` | `UniqueMachineToken`          | per-machine registration token           |
//! | `0x0e` | `DownloadURLOverride`         | override for the machine-config URL       |
//!
//! Tags `0x00`–`0x05` are reserved by the ADV/bootloader internals (`grub`,
//! `bootonce`, `menu_auto_default`, ...) and are rejected for machine state.

use os_kernel::{Error, Result};

/// A typed META key, mapping a symbolic name to its on-disk ADV tag value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MetaKey {
    /// Pending/most-recent upgrade information (e.g. the previous version),
    /// used so the machine can roll back on a failed upgrade. Tag `0x06`.
    Upgrade,
    /// Reference of the staged installer image to boot into for an upgrade.
    /// Tag `0x07`.
    StagedUpgradeImageRef,
    /// Serialized install options recorded alongside a staged upgrade so the
    /// upgrade boot reproduces the original install. Tag `0x08`.
    StagedUpgradeInstallOptions,
    /// Serialized STATE-partition encryption configuration. Tag `0x09`.
    StateEncryptionConfig,
    /// Bare-metal ("metal") network platform configuration injected at install
    /// time and consumed by the platform on boot. Tag `0x0a`.
    MetalNetworkPlatformConfig,
    /// Override for the SMBIOS/hardware machine UUID. Tag `0x0b`.
    UuidOverride,
    /// Legacy user-disk partition configuration. Tag `0x0c`.
    UserDiskConfig,
    /// The unique, per-machine token used to register with the management plane.
    /// Tag `0x0d`.
    UniqueMachineToken,
    /// Override for the machine-configuration download URL. Tag `0x0e`.
    DownloadUrlOverride,
    /// Any other tag not reserved by Talos. Tags `< 0x06` are reserved for the
    /// bootloader/ADV internals and are rejected by [`MetaKey::from_tag`].
    Custom(u8),
}

impl MetaKey {
    /// First tag value available to machine state. Lower tags are reserved by
    /// the ADV/bootloader internals (`grub`, `bootonce`, etc.).
    pub const FIRST_USER_TAG: u8 = 0x06;

    /// All first-party, named keys, in tag order. Useful for iteration and
    /// for tooling that wants to enumerate the known META keys.
    pub const ALL_RESERVED: [MetaKey; 9] = [
        MetaKey::Upgrade,
        MetaKey::StagedUpgradeImageRef,
        MetaKey::StagedUpgradeInstallOptions,
        MetaKey::StateEncryptionConfig,
        MetaKey::MetalNetworkPlatformConfig,
        MetaKey::UuidOverride,
        MetaKey::UserDiskConfig,
        MetaKey::UniqueMachineToken,
        MetaKey::DownloadUrlOverride,
    ];

    /// Returns the numeric ADV tag for this key.
    pub const fn tag(self) -> u8 {
        match self {
            MetaKey::Upgrade => 0x06,
            MetaKey::StagedUpgradeImageRef => 0x07,
            MetaKey::StagedUpgradeInstallOptions => 0x08,
            MetaKey::StateEncryptionConfig => 0x09,
            MetaKey::MetalNetworkPlatformConfig => 0x0a,
            MetaKey::UuidOverride => 0x0b,
            MetaKey::UserDiskConfig => 0x0c,
            MetaKey::UniqueMachineToken => 0x0d,
            MetaKey::DownloadUrlOverride => 0x0e,
            MetaKey::Custom(t) => t,
        }
    }

    /// Builds a [`MetaKey`] from a raw tag, rejecting the reserved low range.
    ///
    /// Returns [`Error::Invalid`] for tags below [`MetaKey::FIRST_USER_TAG`],
    /// since those belong to the ADV/bootloader namespace and must not be used
    /// to store machine state.
    pub fn from_tag(tag: u8) -> Result<Self> {
        match tag {
            0x06 => Ok(MetaKey::Upgrade),
            0x07 => Ok(MetaKey::StagedUpgradeImageRef),
            0x08 => Ok(MetaKey::StagedUpgradeInstallOptions),
            0x09 => Ok(MetaKey::StateEncryptionConfig),
            0x0a => Ok(MetaKey::MetalNetworkPlatformConfig),
            0x0b => Ok(MetaKey::UuidOverride),
            0x0c => Ok(MetaKey::UserDiskConfig),
            0x0d => Ok(MetaKey::UniqueMachineToken),
            0x0e => Ok(MetaKey::DownloadUrlOverride),
            t if t >= Self::FIRST_USER_TAG => Ok(MetaKey::Custom(t)),
            t => Err(Error::invalid(format!(
                "meta tag {t:#04x} is reserved and cannot be used for machine state"
            ))),
        }
    }

    /// Whether this key is one of the named, first-party machine-state keys.
    pub fn is_reserved(self) -> bool {
        !matches!(self, MetaKey::Custom(_))
    }

    /// Whether the given tag falls in the ADV/bootloader-reserved low range and
    /// thus cannot hold machine state.
    pub fn is_bootloader_reserved(tag: u8) -> bool {
        tag < Self::FIRST_USER_TAG
    }

    /// A short, stable human-readable name (used in logs / debug output).
    pub fn name(self) -> &'static str {
        match self {
            MetaKey::Upgrade => "upgrade",
            MetaKey::StagedUpgradeImageRef => "staged-upgrade-image-ref",
            MetaKey::StagedUpgradeInstallOptions => "staged-upgrade-install-options",
            MetaKey::StateEncryptionConfig => "state-encryption-config",
            MetaKey::MetalNetworkPlatformConfig => "metal-network-platform-config",
            MetaKey::UuidOverride => "uuid-override",
            MetaKey::UserDiskConfig => "user-disk-config",
            MetaKey::UniqueMachineToken => "unique-machine-token",
            MetaKey::DownloadUrlOverride => "download-url-override",
            MetaKey::Custom(_) => "custom",
        }
    }

    /// Looks up a key by its symbolic [`MetaKey::name`].
    ///
    /// Only the first-party named keys can be resolved this way; `"custom"` is
    /// rejected because it is ambiguous (it does not carry a tag value).
    pub fn from_name(name: &str) -> Result<Self> {
        Self::ALL_RESERVED
            .into_iter()
            .find(|k| k.name() == name)
            .ok_or_else(|| Error::not_found(format!("no META key named {name:?}")))
    }
}

impl core::fmt::Display for MetaKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MetaKey::Custom(t) => write!(f, "custom({t:#04x})"),
            other => write!(f, "{}", other.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_tags_round_trip() {
        for key in MetaKey::ALL_RESERVED {
            assert_eq!(MetaKey::from_tag(key.tag()).unwrap(), key);
            assert!(key.is_reserved());
        }
    }

    #[test]
    fn all_reserved_tags_are_unique_and_ordered() {
        let mut prev = None;
        for key in MetaKey::ALL_RESERVED {
            let t = key.tag();
            if let Some(p) = prev {
                assert!(t > p, "tags must be strictly increasing");
            }
            prev = Some(t);
        }
    }

    #[test]
    fn custom_tag_round_trips_and_is_not_reserved() {
        let k = MetaKey::from_tag(0x40).unwrap();
        assert_eq!(k, MetaKey::Custom(0x40));
        assert!(!k.is_reserved());
        assert_eq!(k.tag(), 0x40);
    }

    #[test]
    fn low_reserved_tags_are_rejected() {
        for t in 0x00..MetaKey::FIRST_USER_TAG {
            assert!(MetaKey::from_tag(t).is_err());
            assert!(MetaKey::is_bootloader_reserved(t));
        }
        assert!(!MetaKey::is_bootloader_reserved(MetaKey::FIRST_USER_TAG));
    }

    #[test]
    fn names_are_stable() {
        assert_eq!(MetaKey::UniqueMachineToken.name(), "unique-machine-token");
        assert_eq!(MetaKey::UuidOverride.name(), "uuid-override");
        assert_eq!(MetaKey::Custom(9).name(), "custom");
    }

    #[test]
    fn from_name_resolves_named_keys() {
        for key in MetaKey::ALL_RESERVED {
            assert_eq!(MetaKey::from_name(key.name()).unwrap(), key);
        }
        assert!(MetaKey::from_name("custom").is_err());
        assert!(MetaKey::from_name("nonsense").is_err());
    }

    #[test]
    fn display_distinguishes_custom() {
        assert_eq!(MetaKey::Upgrade.to_string(), "upgrade");
        assert_eq!(MetaKey::Custom(0x42).to_string(), "custom(0x42)");
    }

    #[test]
    fn new_named_tags_map_correctly() {
        assert_eq!(MetaKey::StagedUpgradeInstallOptions.tag(), 0x08);
        assert_eq!(MetaKey::MetalNetworkPlatformConfig.tag(), 0x0a);
        assert_eq!(MetaKey::UuidOverride.tag(), 0x0b);
        assert_eq!(MetaKey::UserDiskConfig.tag(), 0x0c);
        assert_eq!(MetaKey::UniqueMachineToken.tag(), 0x0d);
        assert_eq!(MetaKey::DownloadUrlOverride.tag(), 0x0e);
    }
}
