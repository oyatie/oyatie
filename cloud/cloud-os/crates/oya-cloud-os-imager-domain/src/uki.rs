//! SecureBoot Unified Kernel Image (UKI) assembly.
//!
//! Mirrors Talos `internal/pkg/secureboot/uki`: a UKI is a single PE/COFF EFI
//! binary that bundles the kernel, initramfs, kernel cmdline, os-release and a
//! `.uname` section, signed so that UEFI SecureBoot will load it. This module
//! lays out the sections, computes the total size and models the signing
//! boundary as a [`UkiSigner`] trait.

use crate::profile::Arch;

/// A PE section embedded in the UKI, in load order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UkiSection {
    /// PE section name (`.linux`, `.initrd`, `.cmdline`, ...).
    pub name: String,
    /// Section payload length in bytes.
    pub len: u64,
}

impl UkiSection {
    /// Construct a section.
    pub fn new(name: impl Into<String>, len: u64) -> UkiSection {
        UkiSection {
            name: name.into(),
            len,
        }
    }
}

/// The unsigned layout of a UKI prior to signing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UkiLayout {
    /// Target architecture.
    pub arch: Arch,
    /// Ordered PE sections.
    pub sections: Vec<UkiSection>,
}

impl UkiLayout {
    /// Build the canonical Talos UKI layout from its component sizes.
    pub fn build(
        arch: Arch,
        kernel_len: u64,
        initramfs_len: u64,
        cmdline: &str,
        os_release: &str,
        uname: &str,
    ) -> UkiLayout {
        let sections = vec![
            UkiSection::new(".osrel", os_release.len() as u64),
            UkiSection::new(".cmdline", cmdline.len() as u64),
            UkiSection::new(".uname", uname.len() as u64),
            UkiSection::new(".linux", kernel_len),
            UkiSection::new(".initrd", initramfs_len),
        ];
        UkiLayout { arch, sections }
    }

    /// Look a section up by name.
    pub fn section(&self, name: &str) -> Option<&UkiSection> {
        self.sections.iter().find(|s| s.name == name)
    }

    /// Total size of all section payloads plus a fixed PE header overhead.
    pub fn total_len(&self) -> u64 {
        const PE_HEADER_OVERHEAD: u64 = 4096;
        PE_HEADER_OVERHEAD + self.sections.iter().map(|s| s.len).sum::<u64>()
    }

    /// Validate the layout: kernel and initrd must be present and non-empty.
    pub fn validate(&self) -> Result<(), UkiError> {
        let linux = self
            .section(".linux")
            .ok_or(UkiError::MissingSection(".linux"))?;
        if linux.len == 0 {
            return Err(UkiError::EmptySection(".linux"));
        }
        let initrd = self
            .section(".initrd")
            .ok_or(UkiError::MissingSection(".initrd"))?;
        if initrd.len == 0 {
            return Err(UkiError::EmptySection(".initrd"));
        }
        if self.section(".cmdline").is_none() {
            return Err(UkiError::MissingSection(".cmdline"));
        }
        Ok(())
    }
}

/// The signing boundary. Real signing uses `pesign`/`sbsign` with a private key
/// that never crosses this boundary; here it is modeled by a trait.
pub trait UkiSigner {
    /// Sign the given (validated) UKI layout, returning a [`SignedUki`].
    /// Implementations must reject an invalid layout.
    fn sign(&self, layout: &UkiLayout) -> Result<SignedUki, UkiError>;
}

/// A signed UKI ready to be placed on the EFI system partition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedUki {
    /// The layout that was signed.
    pub layout: UkiLayout,
    /// Hex fingerprint of the signing certificate.
    pub signer_fingerprint: String,
    /// Total signed size (layout + a fixed `.sbat`+signature section).
    pub signed_len: u64,
}

impl SignedUki {
    /// Whether this UKI is signed by the key with the given fingerprint.
    pub fn signed_by(&self, fingerprint: &str) -> bool {
        self.signer_fingerprint == fingerprint
    }
}

/// An in-memory signer that appends a fixed-size signature with a known
/// fingerprint. Used by tests and offline builds.
#[derive(Debug, Clone)]
pub struct InMemoryUkiSigner {
    /// The fingerprint this signer stamps onto produced UKIs.
    pub fingerprint: String,
}

impl InMemoryUkiSigner {
    /// Construct a signer with the given certificate fingerprint.
    pub fn new(fingerprint: impl Into<String>) -> InMemoryUkiSigner {
        InMemoryUkiSigner {
            fingerprint: fingerprint.into(),
        }
    }
}

/// Modeled byte cost of the SBAT + PKCS#7 signature appended on signing.
pub const SIGNATURE_OVERHEAD: u64 = 8192;

impl UkiSigner for InMemoryUkiSigner {
    fn sign(&self, layout: &UkiLayout) -> Result<SignedUki, UkiError> {
        layout.validate()?;
        Ok(SignedUki {
            layout: layout.clone(),
            signer_fingerprint: self.fingerprint.clone(),
            signed_len: layout.total_len() + SIGNATURE_OVERHEAD,
        })
    }
}

/// A UKI assembly / signing failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UkiError {
    /// A required PE section is absent.
    MissingSection(&'static str),
    /// A required PE section is present but empty.
    EmptySection(&'static str),
    /// The signer was given a layout it could not sign.
    SigningFailed(String),
}

impl std::fmt::Display for UkiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UkiError::MissingSection(s) => write!(f, "UKI missing required section '{s}'"),
            UkiError::EmptySection(s) => write!(f, "UKI section '{s}' is empty"),
            UkiError::SigningFailed(m) => write!(f, "UKI signing failed: {m}"),
        }
    }
}

impl std::error::Error for UkiError {}
