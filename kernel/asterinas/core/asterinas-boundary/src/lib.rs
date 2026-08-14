#![forbid(unsafe_code)]
//! Black-box pin + boot-readiness marker boundary for the Asterinas v0.17.2 real-boot slice.
//!
//! Every constant here is PUBLIC upstream release metadata copied verbatim from the published
//! GitHub release; no upstream file is modified (MPL-2.0 black-box boundary preserved). This
//! crate has ZERO dependencies so the pin stays a pure, auditable source of truth that the
//! network-touching harness (`kernel-asterinas-real-boot`) and later boot shards consume.
//!
//! data_class: PUBLIC — published release identity only.

/// Upstream repository. data_class: PUBLIC
pub const UPSTREAM_REPOSITORY: &str = "https://github.com/asterinas/asterinas";
/// Pinned release tag proven by this slice. data_class: PUBLIC
pub const RELEASE_TAG: &str = "v0.17.2";
/// Release commit for the pinned tag. data_class: PUBLIC
pub const RELEASE_COMMIT: &str = "23adfdfd72b05cee8d232809caea81a4b33d3488";
/// Human release page. data_class: PUBLIC
pub const RELEASE_URL: &str = "https://github.com/asterinas/asterinas/releases/tag/v0.17.2";

/// Directly-bootable published release asset (unmodified black-box artifact). data_class: PUBLIC
pub const BOOT_ISO_ASSET: &str = "asterinas-nixos-0.17.2-x86_64.iso";
/// Pinned sha256 of the release asset. data_class: PUBLIC
pub const BOOT_ISO_SHA256: &str =
    "bf6e161ecc8b8080b842a339cee5f55d18b93d99b1e39c7c07681ff3aca0090a";
/// Pinned byte size of the release asset. data_class: PUBLIC
pub const BOOT_ISO_BYTE_SIZE: u64 = 1_378_910_208;
/// Direct release-asset download URL. data_class: PUBLIC
pub const BOOT_ISO_DOWNLOAD_URL: &str = "https://github.com/asterinas/asterinas/releases/download/v0.17.2/asterinas-nixos-0.17.2-x86_64.iso";

/// The pin manifest (JSON), embedded so the compiled pin and the on-disk manifest cannot drift.
pub const PIN_MANIFEST: &str = include_str!("../pins/asterinas-release-v0.17.2.json");

/// Native architecture of the ISO (QEMU boot, dev-only). data_class: PUBLIC
pub const BOOT_ARCH: &str = "x86_64";

/// Closed set of boot-ready marker regexes (boot-ready constraint). The boot is reached only
/// when the raw captured serial log matches AT LEAST ONE of these exact regexes; no other
/// marker qualifies and no fallback/heuristic marker is permitted. Consumed by the later boot
/// shards (AC2/AC3); recorded here as the single source of truth for the allowed marker set.
/// data_class: PUBLIC
pub const BOOT_READY_MARKERS: [&str; 5] = [
    r"(?i)login:\s*$",
    r"[#$]\s$",
    r"Welcome to NixOS",
    r"Reached target .*(Multi-User|Basic System|Login Prompts)",
    r"systemd\[1\]:\s+Startup finished",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_manifest_embeds_release_identity() {
        assert!(PIN_MANIFEST.contains(RELEASE_TAG));
        assert!(PIN_MANIFEST.contains(BOOT_ISO_SHA256));
        assert!(PIN_MANIFEST.contains(BOOT_ISO_ASSET));
    }

    #[test]
    fn download_url_names_the_pinned_asset_and_tag() {
        assert!(BOOT_ISO_DOWNLOAD_URL.contains(RELEASE_TAG));
        assert!(BOOT_ISO_DOWNLOAD_URL.contains(BOOT_ISO_ASSET));
    }

    #[test]
    fn boot_ready_marker_set_is_closed_and_nonempty() {
        assert_eq!(BOOT_READY_MARKERS.len(), 5);
        assert!(BOOT_READY_MARKERS.iter().all(|m| !m.is_empty()));
    }
}
