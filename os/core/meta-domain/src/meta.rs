//! The high-level typed key/value facade over the ADV container.
//!
//! [`Meta`] mirrors the role of `internal/pkg/meta.Meta` in Talos: it wraps the
//! raw [`Adv`] document with ergonomic, type-aware accessors for the well-known
//! machine-state keys (staged upgrade image, machine token, etc.) and tracks
//! whether the in-memory copy has diverged from what is persisted on disk.
//!
//! The flow modelled here is:
//!
//! 1. Load the partition bytes and [`Meta::decode`] them into a `Meta`.
//! 2. Mutate via the typed setters; each mutation marks the document *dirty*.
//! 3. [`Meta::encode`] back to ADV1 bytes and write them out, then call
//!    [`Meta::mark_clean`] to record that memory and disk now agree.

use crate::adv::Adv;
use crate::key::MetaKey;
use crate::value::MetaValue;
use os_kernel::{Error, Result};

/// A typed view over the META key/value store.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Meta {
    adv: Adv,
    /// True when in-memory state has unflushed changes relative to disk.
    dirty: bool,
}

impl Meta {
    /// Creates an empty, clean META document.
    pub fn new() -> Self {
        Self {
            adv: Adv::new(),
            dirty: false,
        }
    }

    /// Borrows the underlying ADV document.
    pub fn adv(&self) -> &Adv {
        &self.adv
    }

    /// Whether there are unflushed in-memory changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Records that the in-memory copy has been persisted to disk.
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Number of stored keys.
    pub fn len(&self) -> usize {
        self.adv.len()
    }

    /// Whether the store holds no keys.
    pub fn is_empty(&self) -> bool {
        self.adv.is_empty()
    }

    /// Reads the raw value stored under `key`.
    pub fn get(&self, key: MetaKey) -> Option<&MetaValue> {
        self.adv.get(key)
    }

    /// Reads `key` as a UTF-8 string, erroring if the value is not valid UTF-8.
    pub fn get_str(&self, key: MetaKey) -> Result<Option<&str>> {
        match self.adv.get(key) {
            Some(v) => v.as_str().map(Some),
            None => Ok(None),
        }
    }

    /// Inserts or replaces a raw value, marking the document dirty.
    ///
    /// Returns the previously stored value, if any.
    pub fn set(&mut self, key: MetaKey, value: MetaValue) -> Option<MetaValue> {
        let prev = self.adv.set(key, value);
        // Only mark dirty if the value actually changed.
        if prev.as_ref() != self.adv.get(key) {
            self.dirty = true;
        }
        prev
    }

    /// Convenience setter for a string value.
    pub fn set_str(&mut self, key: MetaKey, value: &str) -> Result<Option<MetaValue>> {
        Ok(self.set(key, MetaValue::from_str(value)?))
    }

    /// Removes a key, marking the document dirty if something was removed.
    pub fn delete(&mut self, key: MetaKey) -> Option<MetaValue> {
        let prev = self.adv.delete(key);
        if prev.is_some() {
            self.dirty = true;
        }
        prev
    }

    // --- Typed accessors for well-known machine-state keys ---------------

    /// The staged upgrade installer image reference, if set.
    pub fn staged_upgrade_image_ref(&self) -> Result<Option<&str>> {
        self.get_str(MetaKey::StagedUpgradeImageRef)
    }

    /// Stages an upgrade by recording the installer image reference.
    ///
    /// The reference must be non-empty; an empty image ref is rejected as it
    /// would silently disable the staged upgrade.
    pub fn set_staged_upgrade_image_ref(&mut self, image_ref: &str) -> Result<()> {
        if image_ref.trim().is_empty() {
            return Err(Error::invalid("staged upgrade image ref must not be empty"));
        }
        self.set_str(MetaKey::StagedUpgradeImageRef, image_ref)?;
        Ok(())
    }

    /// The unique machine token used to register with the management plane.
    pub fn unique_machine_token(&self) -> Result<Option<&str>> {
        self.get_str(MetaKey::UniqueMachineToken)
    }

    /// Sets the unique machine token. Rejects empty tokens.
    pub fn set_unique_machine_token(&mut self, token: &str) -> Result<()> {
        if token.is_empty() {
            return Err(Error::invalid("machine token must not be empty"));
        }
        self.set_str(MetaKey::UniqueMachineToken, token)?;
        Ok(())
    }

    /// Clears a staged upgrade, returning true if one was present. Clears both
    /// the staged image ref and any recorded install options.
    pub fn clear_staged_upgrade(&mut self) -> bool {
        let img = self.delete(MetaKey::StagedUpgradeImageRef).is_some();
        let opts = self.delete(MetaKey::StagedUpgradeInstallOptions).is_some();
        img || opts
    }

    /// The serialized install options recorded for a staged upgrade, if any.
    pub fn staged_upgrade_install_options(&self) -> Result<Option<&str>> {
        self.get_str(MetaKey::StagedUpgradeInstallOptions)
    }

    /// Records serialized install options alongside a staged upgrade.
    pub fn set_staged_upgrade_install_options(&mut self, opts: &str) -> Result<()> {
        self.set_str(MetaKey::StagedUpgradeInstallOptions, opts)?;
        Ok(())
    }

    /// The previous-version upgrade marker used for rollback, if recorded.
    pub fn upgrade(&self) -> Result<Option<&str>> {
        self.get_str(MetaKey::Upgrade)
    }

    /// Records the previous-version upgrade marker used for rollback.
    pub fn set_upgrade(&mut self, prev_version: &str) -> Result<()> {
        if prev_version.trim().is_empty() {
            return Err(Error::invalid("upgrade marker must not be empty"));
        }
        self.set_str(MetaKey::Upgrade, prev_version)?;
        Ok(())
    }

    /// The SMBIOS/hardware machine-UUID override, if set.
    pub fn uuid_override(&self) -> Result<Option<&str>> {
        self.get_str(MetaKey::UuidOverride)
    }

    /// Overrides the machine UUID.
    ///
    /// The UUID must look like a canonical RFC-4122 string: 36 chars in the
    /// `8-4-4-4-12` hex-with-dashes form. This mirrors Talos validating the
    /// override before persisting it.
    pub fn set_uuid_override(&mut self, uuid: &str) -> Result<()> {
        validate_uuid(uuid)?;
        self.set_str(MetaKey::UuidOverride, uuid)?;
        Ok(())
    }

    /// Clears any machine-UUID override, returning true if one was present.
    pub fn clear_uuid_override(&mut self) -> bool {
        self.delete(MetaKey::UuidOverride).is_some()
    }

    /// The serialized STATE-partition encryption config, if set.
    pub fn state_encryption_config(&self) -> Option<&MetaValue> {
        self.get(MetaKey::StateEncryptionConfig)
    }

    /// Stores the serialized STATE-partition encryption config (opaque bytes).
    pub fn set_state_encryption_config(&mut self, config: MetaValue) -> Result<()> {
        if config.is_empty() {
            return Err(Error::invalid("state encryption config must not be empty"));
        }
        self.set(MetaKey::StateEncryptionConfig, config);
        Ok(())
    }

    /// The bare-metal network platform config, if set.
    pub fn metal_network_platform_config(&self) -> Option<&MetaValue> {
        self.get(MetaKey::MetalNetworkPlatformConfig)
    }

    /// Stores the bare-metal network platform config (opaque bytes).
    pub fn set_metal_network_platform_config(&mut self, config: MetaValue) {
        self.set(MetaKey::MetalNetworkPlatformConfig, config);
    }

    /// The machine-config download-URL override, if set.
    pub fn download_url_override(&self) -> Result<Option<&str>> {
        self.get_str(MetaKey::DownloadUrlOverride)
    }

    /// Overrides the machine-config download URL. Rejects empty URLs.
    pub fn set_download_url_override(&mut self, url: &str) -> Result<()> {
        if url.trim().is_empty() {
            return Err(Error::invalid("download URL override must not be empty"));
        }
        self.set_str(MetaKey::DownloadUrlOverride, url)?;
        Ok(())
    }

    // --- bulk import/export --------------------------------------------------

    /// Exports the document as a JSON array of records (the transport form used
    /// by `talosctl meta`). All values must be valid UTF-8.
    pub fn to_json(&self) -> Result<String> {
        crate::codec::adv_to_json(&self.adv)
    }

    /// Imports a JSON array of records, *merging* them into this document and
    /// marking it dirty if anything changed. Existing keys not present in the
    /// JSON are left untouched.
    pub fn import_json(&mut self, json: &str) -> Result<()> {
        let incoming = crate::codec::adv_from_json(json)?;
        for (key, value) in incoming.iter() {
            self.set(*key, value.clone());
        }
        Ok(())
    }

    /// Replaces the entire document with the records decoded from `json`,
    /// marking it dirty if the contents changed.
    pub fn replace_from_json(&mut self, json: &str) -> Result<()> {
        let incoming = crate::codec::adv_from_json(json)?;
        if incoming != self.adv {
            self.adv = incoming;
            self.dirty = true;
        }
        Ok(())
    }

    /// Serializes the typed view back into ADV1 bytes.
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.adv.encode()
    }

    /// Decodes ADV1 bytes into a clean `Meta`.
    pub fn decode(buf: &[u8]) -> Result<Self> {
        Ok(Self {
            adv: Adv::decode(buf)?,
            dirty: false,
        })
    }
}

/// Validates a canonical RFC-4122 UUID string (`8-4-4-4-12` hex, dashed).
fn validate_uuid(uuid: &str) -> Result<()> {
    let groups = [8usize, 4, 4, 4, 12];
    let parts: Vec<&str> = uuid.split('-').collect();
    if parts.len() != groups.len() {
        return Err(Error::invalid(format!(
            "UUID {uuid:?} must have 5 dash-separated groups"
        )));
    }
    for (part, expected) in parts.iter().zip(groups) {
        if part.len() != expected || !part.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(Error::invalid(format!(
                "UUID {uuid:?} has a malformed group"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty_and_clean() {
        let m = Meta::new();
        assert!(m.is_empty());
        assert!(!m.is_dirty());
        assert_eq!(m.len(), 0);
    }

    #[test]
    fn typed_setters_mark_dirty_and_round_trip() {
        let mut m = Meta::new();
        m.set_staged_upgrade_image_ref("ghcr.io/siderolabs/installer:v1.7.0")
            .unwrap();
        m.set_unique_machine_token("tok-xyz").unwrap();
        assert!(m.is_dirty());
        assert_eq!(
            m.staged_upgrade_image_ref().unwrap().unwrap(),
            "ghcr.io/siderolabs/installer:v1.7.0"
        );
        assert_eq!(m.unique_machine_token().unwrap().unwrap(), "tok-xyz");
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn empty_image_ref_and_token_rejected() {
        let mut m = Meta::new();
        assert!(m.set_staged_upgrade_image_ref("   ").is_err());
        assert!(m.set_unique_machine_token("").is_err());
        // Nothing was stored, so it should still be clean and empty.
        assert!(m.is_empty());
        assert!(!m.is_dirty());
    }

    #[test]
    fn mark_clean_and_decode_are_clean() {
        let mut m = Meta::new();
        m.set_unique_machine_token("tok").unwrap();
        let bytes = m.encode().unwrap();
        m.mark_clean();
        assert!(!m.is_dirty());

        let decoded = Meta::decode(&bytes).unwrap();
        assert!(!decoded.is_dirty());
        assert_eq!(decoded, m);
    }

    #[test]
    fn clear_staged_upgrade_reports_presence() {
        let mut m = Meta::new();
        assert!(!m.clear_staged_upgrade());
        m.set_staged_upgrade_image_ref("img:v1").unwrap();
        m.mark_clean();
        assert!(m.clear_staged_upgrade());
        assert!(m.is_dirty());
        assert!(m.staged_upgrade_image_ref().unwrap().is_none());
    }

    #[test]
    fn get_str_on_non_utf8_errors() {
        let mut m = Meta::new();
        m.set(
            MetaKey::Custom(0x40),
            MetaValue::new(vec![0xff, 0x00]).unwrap(),
        );
        assert!(m.get_str(MetaKey::Custom(0x40)).is_err());
        assert!(m.get_str(MetaKey::Upgrade).unwrap().is_none());
    }

    #[test]
    fn uuid_override_validates() {
        let mut m = Meta::new();
        assert!(
            m.set_uuid_override("12345678-1234-1234-1234-1234567890ab")
                .is_ok()
        );
        assert_eq!(
            m.uuid_override().unwrap().unwrap(),
            "12345678-1234-1234-1234-1234567890ab"
        );
        // bad shapes
        assert!(m.set_uuid_override("not-a-uuid").is_err());
        assert!(m.set_uuid_override("12345678-1234-1234-1234").is_err());
        assert!(
            m.set_uuid_override("ZZZZZZZZ-1234-1234-1234-1234567890ab")
                .is_err()
        );
        assert!(m.clear_uuid_override());
        assert!(m.uuid_override().unwrap().is_none());
    }

    #[test]
    fn staged_upgrade_install_options_round_trip() {
        let mut m = Meta::new();
        m.set_staged_upgrade_image_ref("installer:v1").unwrap();
        m.set_staged_upgrade_install_options("--wipe").unwrap();
        assert_eq!(
            m.staged_upgrade_install_options().unwrap().unwrap(),
            "--wipe"
        );
        // clearing the staged upgrade also clears the options.
        assert!(m.clear_staged_upgrade());
        assert!(m.staged_upgrade_install_options().unwrap().is_none());
        assert!(m.staged_upgrade_image_ref().unwrap().is_none());
    }

    #[test]
    fn upgrade_marker_round_trip_and_rejects_empty() {
        let mut m = Meta::new();
        assert!(m.set_upgrade("  ").is_err());
        m.set_upgrade("v1.5.0").unwrap();
        assert_eq!(m.upgrade().unwrap().unwrap(), "v1.5.0");
    }

    #[test]
    fn binary_configs_store_opaque_bytes() {
        let mut m = Meta::new();
        let cfg = MetaValue::new(vec![0x00, 0xff, 0x10]).unwrap();
        m.set_state_encryption_config(cfg.clone()).unwrap();
        assert_eq!(m.state_encryption_config().unwrap(), &cfg);
        assert!(m.set_state_encryption_config(MetaValue::default()).is_err());

        m.set_metal_network_platform_config(MetaValue::new(vec![1, 2, 3]).unwrap());
        assert_eq!(
            m.metal_network_platform_config().unwrap().as_bytes(),
            &[1, 2, 3]
        );
    }

    #[test]
    fn download_url_override_round_trip() {
        let mut m = Meta::new();
        assert!(m.set_download_url_override("   ").is_err());
        m.set_download_url_override("https://example.com/config")
            .unwrap();
        assert_eq!(
            m.download_url_override().unwrap().unwrap(),
            "https://example.com/config"
        );
    }

    #[test]
    fn json_export_import_round_trip() {
        let mut m = Meta::new();
        m.set_unique_machine_token("tok").unwrap();
        m.set_upgrade("v1.5.0").unwrap();
        let json = m.to_json().unwrap();

        let mut m2 = Meta::new();
        m2.import_json(&json).unwrap();
        assert_eq!(m2.unique_machine_token().unwrap().unwrap(), "tok");
        assert_eq!(m2.upgrade().unwrap().unwrap(), "v1.5.0");
        assert!(m2.is_dirty());
    }

    #[test]
    fn import_json_merges_without_clobbering() {
        let mut m = Meta::new();
        m.set_unique_machine_token("keep").unwrap();
        m.import_json("[{\"key\":6,\"value\":\"v1.6.0\"}]").unwrap();
        assert_eq!(m.unique_machine_token().unwrap().unwrap(), "keep");
        assert_eq!(m.upgrade().unwrap().unwrap(), "v1.6.0");
    }

    #[test]
    fn replace_from_json_replaces_everything() {
        let mut m = Meta::new();
        m.set_unique_machine_token("gone").unwrap();
        m.replace_from_json("[{\"key\":6,\"value\":\"v1.7.0\"}]")
            .unwrap();
        assert!(m.unique_machine_token().unwrap().is_none());
        assert_eq!(m.upgrade().unwrap().unwrap(), "v1.7.0");
        assert_eq!(m.len(), 1);
    }
}
