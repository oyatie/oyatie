//! The imager **profile**: the declarative description of an image to build.
//!
//! Mirrors Talos `pkg/imager/profile`. A profile selects an architecture, a
//! [`SecureBootMode`], a base [`crate::output::OutputKind`], the input boot
//! assets (kernel/initramfs/cmdline), a set of bundled system extensions and
//! [`crate::overlay::Overlay`], and per-output options such as the disk size,
//! disk format and compression. The imager validates a profile before it will
//! attempt a build.

use crate::output::{OutputKind, OutputOptions};
use std::fmt;

/// The CPU architecture an image targets. Talos ships `amd64` and `arm64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch {
    /// x86-64 (`amd64`).
    Amd64,
    /// 64-bit ARM (`arm64` / `aarch64`).
    Arm64,
}

impl Arch {
    /// Canonical Talos name (`amd64`/`arm64`).
    pub fn as_str(self) -> &'static str {
        match self {
            Arch::Amd64 => "amd64",
            Arch::Arm64 => "arm64",
        }
    }

    /// Parse from the Talos or GOARCH spelling.
    pub fn parse(s: &str) -> Option<Arch> {
        match s.trim().to_ascii_lowercase().as_str() {
            "amd64" | "x86_64" | "x86-64" => Some(Arch::Amd64),
            "arm64" | "aarch64" => Some(Arch::Arm64),
            _ => None,
        }
    }

    /// The Linux kernel image filename produced for this architecture.
    pub fn kernel_image_name(self) -> &'static str {
        match self {
            Arch::Amd64 => "vmlinuz-amd64",
            Arch::Arm64 => "vmlinuz-arm64",
        }
    }

    /// The UEFI default boot path stub for this architecture, used when an
    /// image must be placed at the removable-media fallback location.
    pub fn efi_boot_path(self) -> &'static str {
        match self {
            Arch::Amd64 => "EFI/BOOT/BOOTX64.EFI",
            Arch::Arm64 => "EFI/BOOT/BOOTAA64.EFI",
        }
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Whether an image is built for SecureBoot. Talos builds a separate UKI-based
/// image for SecureBoot vs. the GRUB-based legacy image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecureBootMode {
    /// Legacy / non-SecureBoot image (GRUB or systemd-boot, unsigned).
    Disabled,
    /// SecureBoot image: a signed UKI is produced and GRUB is not used.
    Enabled,
}

impl SecureBootMode {
    /// Whether SecureBoot is enabled.
    pub fn is_enabled(self) -> bool {
        matches!(self, SecureBootMode::Enabled)
    }
}

/// A bundled system extension reference (an OCI image reference in Talos).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemExtension {
    /// OCI image reference, e.g. `ghcr.io/siderolabs/gvisor:20240305.0`.
    pub image_ref: String,
}

impl SystemExtension {
    /// Construct an extension from an image reference.
    pub fn new(image_ref: impl Into<String>) -> SystemExtension {
        SystemExtension {
            image_ref: image_ref.into(),
        }
    }

    /// The bare name portion (last path segment without the tag).
    pub fn name(&self) -> &str {
        let no_tag = self
            .image_ref
            .split(['@', ':'])
            .next()
            .unwrap_or(&self.image_ref);
        no_tag.rsplit('/').next().unwrap_or(no_tag)
    }
}

/// The input boot assets a profile consumes. In Talos these are extracted from
/// the base installer image; here they are modeled as content-addressable
/// blobs identified by name and a byte length / digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    /// Kernel command line appended to the default Talos args.
    pub cmdline_extra: Vec<String>,
    /// Base Talos version string baked into the image (e.g. `v1.7.0`).
    pub version: String,
}

impl Input {
    /// A default input for the given version with no extra cmdline.
    pub fn new(version: impl Into<String>) -> Input {
        Input {
            cmdline_extra: Vec::new(),
            version: version.into(),
        }
    }

    /// Add an extra kernel cmdline argument.
    pub fn with_cmdline(mut self, arg: impl Into<String>) -> Input {
        self.cmdline_extra.push(arg.into());
        self
    }
}

/// The fully-specified description of an image to build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    /// Target architecture.
    pub arch: Arch,
    /// Target Talos platform (metal/aws/gcp/...). Reused from `talos-core`.
    pub platform: os_kernel::Platform,
    /// SecureBoot mode.
    pub secureboot: SecureBootMode,
    /// The kind of artifact to produce.
    pub output: OutputKind,
    /// Per-output options (disk size, format, compression).
    pub options: OutputOptions,
    /// Boot asset inputs.
    pub input: Input,
    /// Bundled system extensions.
    pub extensions: Vec<SystemExtension>,
    /// Optional overlay (board) name, e.g. `rpi_generic`.
    pub overlay: Option<String>,
}

impl Profile {
    /// Construct a minimal profile with default options for the output kind.
    pub fn new(
        arch: Arch,
        platform: os_kernel::Platform,
        output: OutputKind,
        version: impl Into<String>,
    ) -> Profile {
        Profile {
            arch,
            platform,
            secureboot: SecureBootMode::Disabled,
            options: OutputOptions::defaults_for(output),
            output,
            input: Input::new(version),
            extensions: Vec::new(),
            overlay: None,
        }
    }

    /// Enable SecureBoot on this profile (builder style).
    pub fn with_secureboot(mut self) -> Profile {
        self.secureboot = SecureBootMode::Enabled;
        self
    }

    /// Attach an overlay/board (builder style).
    pub fn with_overlay(mut self, name: impl Into<String>) -> Profile {
        self.overlay = Some(name.into());
        self
    }

    /// Add a system extension (builder style).
    pub fn with_extension(mut self, ext: SystemExtension) -> Profile {
        self.extensions.push(ext);
        self
    }

    /// Validate the profile, returning every problem found. An empty vector
    /// means the profile is buildable.
    pub fn validate(&self) -> Vec<ProfileError> {
        let mut errs = Vec::new();

        if self.input.version.trim().is_empty() {
            errs.push(ProfileError::EmptyVersion);
        }

        // SecureBoot only makes sense for UEFI-bootable outputs.
        if self.secureboot.is_enabled() && !self.output.supports_secureboot() {
            errs.push(ProfileError::SecureBootUnsupported(self.output));
        }

        // Overlays (boards) only apply to disk images / metal artifacts.
        if self.overlay.is_some() && !self.output.supports_overlay() {
            errs.push(ProfileError::OverlayUnsupported(self.output));
        }

        // arm64 metal disk images conventionally require an overlay (board
        // definition) because there is no single generic arm64 boot flow.
        if self.arch == Arch::Arm64
            && self.output == OutputKind::DiskImage
            && self.overlay.is_none()
        {
            errs.push(ProfileError::Arm64RequiresOverlay);
        }

        // Duplicate extensions are rejected.
        for (i, ext) in self.extensions.iter().enumerate() {
            if self.extensions[..i]
                .iter()
                .any(|e| e.image_ref == ext.image_ref)
            {
                errs.push(ProfileError::DuplicateExtension(ext.image_ref.clone()));
            }
        }

        // Output-option level validation.
        errs.extend(
            self.options
                .validate(self.output)
                .into_iter()
                .map(ProfileError::Options),
        );

        errs
    }

    /// Convenience: whether the profile passes validation.
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }

    /// The kernel command line this profile produces: the platform default plus
    /// any SecureBoot marker plus the profile's extra args.
    pub fn cmdline(&self) -> String {
        let mut parts: Vec<String> = vec![
            "talos.platform=".to_string() + self.platform.as_str(),
            "console=ttyS0".to_string(),
        ];
        if self.secureboot.is_enabled() {
            parts.push("talos.secureboot=1".to_string());
        }
        parts.extend(self.input.cmdline_extra.iter().cloned());
        parts.join(" ")
    }
}

/// A single validation failure for a [`Profile`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    /// The Talos version string is empty.
    EmptyVersion,
    /// SecureBoot was requested for an output that cannot boot via UEFI/UKI.
    SecureBootUnsupported(OutputKind),
    /// An overlay was attached to an output kind that does not use boards.
    OverlayUnsupported(OutputKind),
    /// An arm64 disk image was requested without a board overlay.
    Arm64RequiresOverlay,
    /// The same extension image was listed twice.
    DuplicateExtension(String),
    /// An output-option level error.
    Options(crate::output::OptionError),
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfileError::EmptyVersion => write!(f, "version must not be empty"),
            ProfileError::SecureBootUnsupported(k) => {
                write!(f, "secureboot is not supported for output '{}'", k.as_str())
            }
            ProfileError::OverlayUnsupported(k) => {
                write!(f, "overlay is not supported for output '{}'", k.as_str())
            }
            ProfileError::Arm64RequiresOverlay => {
                write!(f, "arm64 disk-image requires a board overlay")
            }
            ProfileError::DuplicateExtension(r) => {
                write!(f, "duplicate system extension '{r}'")
            }
            ProfileError::Options(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ProfileError {}

/// Convert a list of profile errors into a single `os_kernel::Error`.
impl From<&ProfileError> for os_kernel::Error {
    fn from(e: &ProfileError) -> Self {
        os_kernel::Error::invalid(e.to_string())
    }
}
