//! Semantic version handling, mirroring Talos `pkg/machinery/version`.

use crate::error::{Error, Result};
use alloc::string::{String, ToString};
use core::cmp::Ordering;
use core::fmt;

/// A semantic version (`MAJOR.MINOR.PATCH` with an optional pre-release tag).
///
/// This is intentionally a small subset of full `SemVer`: it covers the
/// `vX.Y.Z` / `vX.Y.Z-pre` forms Talos uses for the OS and Kubernetes versions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
    /// Major component.
    pub major: u64,
    /// Minor component.
    pub minor: u64,
    /// Patch component.
    pub patch: u64,
    /// Optional pre-release tag (the part after `-`), without the leading dash.
    pub pre_release: Option<String>,
}

impl Version {
    /// Construct a release version with no pre-release tag.
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Version {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }

    /// Parse a version string. A leading `v` is accepted and stripped.
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        let s = s.strip_prefix('v').unwrap_or(s);
        if s.is_empty() {
            return Err(Error::parse("empty version string"));
        }

        // Split off the pre-release tag at the first '-'.
        let (core_part, pre) = match s.split_once('-') {
            Some((c, p)) => {
                if p.is_empty() {
                    return Err(Error::parse("empty pre-release tag"));
                }
                (c, Some(p.to_string()))
            }
            None => (s, None),
        };

        let mut parts = core_part.split('.');
        let major = Self::parse_component(parts.next())?;
        let minor = Self::parse_component(parts.next())?;
        let patch = Self::parse_component(parts.next())?;
        if parts.next().is_some() {
            return Err(Error::parse(alloc::format!(
                "too many version components in '{s}'"
            )));
        }

        Ok(Version {
            major,
            minor,
            patch,
            pre_release: pre,
        })
    }

    fn parse_component(c: Option<&str>) -> Result<u64> {
        let c = c.ok_or_else(|| Error::parse("missing version component"))?;
        c.parse::<u64>()
            .map_err(|_| Error::parse(alloc::format!("invalid numeric component '{c}'")))
    }

    /// Whether this is a pre-release (has a pre-release tag).
    pub fn is_pre_release(&self) -> bool {
        self.pre_release.is_some()
    }

    /// True if `self` is compatible with `other` under the same major version
    /// and `self.minor >= other.minor` (typical "client newer-or-equal" check).
    pub fn is_compatible_with(&self, other: &Version) -> bool {
        self.major == other.major && self.minor >= other.minor
    }

    /// The `MAJOR.MINOR` short form as a string.
    pub fn short(&self) -> String {
        alloc::format!("{}.{}", self.major, self.minor)
    }

    /// A copy with the major component incremented and minor/patch/pre reset.
    pub fn next_major(&self) -> Version {
        Version::new(self.major + 1, 0, 0)
    }

    /// A copy with the minor component incremented and patch/pre reset.
    pub fn next_minor(&self) -> Version {
        Version::new(self.major, self.minor + 1, 0)
    }

    /// A copy with the patch component incremented and pre-release dropped.
    pub fn next_patch(&self) -> Version {
        Version::new(self.major, self.minor, self.patch + 1)
    }

    /// The release version corresponding to this one (pre-release tag removed).
    pub fn to_release(&self) -> Version {
        Version::new(self.major, self.minor, self.patch)
    }

    /// Whether `other` is the immediately adjacent or same minor within the same
    /// major: the Talos upgrade rule that you may only skip at most one minor
    /// version. Returns true if upgrading from `self` to `other` is allowed.
    pub fn is_upgrade_allowed_to(&self, other: &Version) -> bool {
        if other <= self {
            return false;
        }
        if other.major != self.major {
            // Only a single major step, landing on .0, is allowed.
            return other.major == self.major + 1 && other.minor == 0;
        }
        other.minor <= self.minor + 1
    }

    /// The Talos "contract" form: the `MAJOR.MINOR` pair as a packed integer
    /// (`major*100 + minor`), used to gate feature availability the way
    /// `config.VersionContract` does.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "major/minor are small version components that fit in u32"
    )]
    pub fn contract(&self) -> u32 {
        (self.major as u32) * 100 + (self.minor as u32)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre_release {
            write!(f, "-{pre}")?;
        }
        Ok(())
    }
}

impl core::str::FromStr for Version {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Version::parse(s)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare numeric components first.
        match self
            .major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
        {
            Ordering::Equal => {}
            non_eq => return non_eq,
        }

        // A version with a pre-release tag is LOWER precedence than one without
        // (per SemVer §11), comparing tags lexically when both are present.
        match (&self.pre_release, &other.pre_release) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

/// An inclusive version range `[min, max]` used for compatibility gating, e.g.
/// the supported Kubernetes versions for a Talos release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionRange {
    /// Inclusive lower bound.
    pub min: Version,
    /// Inclusive upper bound.
    pub max: Version,
}

impl VersionRange {
    /// Construct a range, validating `min <= max`.
    pub fn new(min: Version, max: Version) -> Result<Self> {
        if min > max {
            return Err(Error::invalid("version range min exceeds max"));
        }
        Ok(VersionRange { min, max })
    }

    /// Whether `v` falls within the inclusive range.
    pub fn contains(&self, v: &Version) -> bool {
        *v >= self.min && *v <= self.max
    }

    /// Clamp `v` into the range, returning the nearest bound if outside.
    pub fn clamp<'a>(&'a self, v: &'a Version) -> &'a Version {
        if *v < self.min {
            &self.min
        } else if *v > self.max {
            &self.max
        } else {
            v
        }
    }
}

impl fmt::Display for VersionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..={}", self.min, self.max)
    }
}

/// Build-time version metadata and formatting, mirroring the upstream
/// `pkg/machinery/version` package.
///
/// In Talos the `Name`, `Tag`, `SHA`, `Built` and `PkgsVersion` values are
/// injected at build time (via `gendata`). Here they default to empty strings;
/// a real build wires them up through `set_build_info`. The formatting
/// functions (`short`, `trim`, `long_version`) reproduce the upstream output
/// byte-for-byte.
pub mod build {
    use alloc::format;
    use alloc::string::String;
    use core::fmt::Write as _;

    /// Information describing a build, equivalent to the upstream
    /// `machineapi.VersionInfo` carried into the long-version formatter.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub struct VersionInfo {
        /// The release tag, e.g. `v1.8.0`.
        pub tag: String,
        /// The git SHA the build was cut from.
        pub sha: String,
        /// The build timestamp.
        pub built: String,
        /// The toolchain version string (upstream: `runtime.Version()`).
        pub go_version: String,
        /// Operating system (upstream: `runtime.GOOS`).
        pub os: String,
        /// Architecture (upstream: `runtime.GOARCH`).
        pub arch: String,
    }

    /// The short version string: `"<name> <tag>"`.
    ///
    /// Mirrors upstream `version.Short()` which formats as `"%s %s"`.
    pub fn short(name: &str, tag: &str) -> String {
        format!("{name} {tag}")
    }

    /// Remove anything extra after the semantic-version core, e.g.
    /// `v0.3.2-1-gabcdef-dirty` -> `v0.3.2`.
    ///
    /// Reproduces upstream `version.Trim`, which strips a trailing match of the
    /// regular expression `(-\d+(-g[0-9a-f]+)?(-dirty)?)$`:
    ///
    /// * `-` followed by one or more decimal digits,
    /// * optionally `-g` followed by one or more lowercase hex digits,
    /// * optionally `-dirty`,
    /// * anchored at the end of the string.
    ///
    /// If the suffix does not match exactly, the input is returned unchanged.
    pub fn trim(version: &str) -> String {
        match trimmed_len(version) {
            Some(end) => version[..end].into(),
            None => version.into(),
        }
    }

    /// Returns the byte offset at which the trailing suffix begins, if the tail
    /// of `version` matches the upstream regex. Returns `None` when there is no
    /// match (and the string is therefore returned unchanged by [`trim`]).
    fn trimmed_len(version: &str) -> Option<usize> {
        let bytes = version.as_bytes();
        let mut end = bytes.len();

        // Optional trailing `-dirty`.
        if let Some(rest) = strip_suffix(bytes, end, b"-dirty") {
            end = rest;
        }

        // Optional `-g<hex>` group.
        if let Some(after_hex) = strip_g_hex(bytes, end) {
            end = after_hex;
        }

        // Mandatory `-<digits>` group.
        let after_digits = strip_dash_digits(bytes, end)?;
        Some(after_digits)
    }

    /// If `bytes[..end]` ends with `suffix`, return the new end offset.
    fn strip_suffix(bytes: &[u8], end: usize, suffix: &[u8]) -> Option<usize> {
        let start = end.checked_sub(suffix.len())?;
        if &bytes[start..end] == suffix {
            Some(start)
        } else {
            None
        }
    }

    /// Strip a trailing `-g[0-9a-f]+` group from `bytes[..end]`, returning the
    /// new end offset, or `None` if the tail does not match.
    fn strip_g_hex(bytes: &[u8], end: usize) -> Option<usize> {
        let mut i = end;
        let mut hex = 0usize;
        // Upstream uses `[0-9a-f]` (lowercase hex only).
        while i > 0 {
            let c = bytes[i - 1];
            if c.is_ascii_digit() || (b'a'..=b'f').contains(&c) {
                i -= 1;
                hex += 1;
            } else {
                break;
            }
        }
        if hex == 0 {
            return None;
        }
        // Require the `-g` prefix immediately before the hex run.
        if i >= 2 && bytes[i - 1] == b'g' && bytes[i - 2] == b'-' {
            Some(i - 2)
        } else {
            None
        }
    }

    /// Strip a trailing `-[0-9]+` group from `bytes[..end]`, returning the new
    /// end offset, or `None` if the tail does not match.
    fn strip_dash_digits(bytes: &[u8], end: usize) -> Option<usize> {
        let mut i = end;
        let mut digits = 0usize;
        while i > 0 && bytes[i - 1].is_ascii_digit() {
            i -= 1;
            digits += 1;
        }
        if digits == 0 {
            return None;
        }
        if i >= 1 && bytes[i - 1] == b'-' {
            Some(i - 1)
        } else {
            None
        }
    }

    /// The verbose, multi-line version string, byte-for-byte equivalent to the
    /// upstream `printLong` output (which formats `versionTemplate`).
    ///
    /// Each line is indented with a leading tab, and the block ends with a
    /// trailing newline.
    pub fn long_version(v: &VersionInfo) -> String {
        let mut s = String::new();
        // Matches upstream `versionTemplate` exactly.
        let _ = write!(
            s,
            "\tTag:         {}\n\tSHA:         {}\n\tBuilt:       {}\n\tGo version:  {}\n\tOS/Arch:     {}/{}\n",
            v.tag, v.sha, v.built, v.go_version, v.os, v.arch,
        );
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_with_optional_v_prefix() {
        assert_eq!(Version::parse("v1.7.0").unwrap(), Version::new(1, 7, 0));
        assert_eq!(Version::parse("1.7.0").unwrap(), Version::new(1, 7, 0));
        assert_eq!(
            Version::parse("  v0.14.3 ").unwrap(),
            Version::new(0, 14, 3)
        );
    }

    #[test]
    fn parses_pre_release() {
        let v = Version::parse("v1.8.0-alpha.1").unwrap();
        assert!(v.is_pre_release());
        assert_eq!(v.pre_release.as_deref(), Some("alpha.1"));
        assert_eq!(v.to_string(), "v1.8.0-alpha.1");
    }

    #[test]
    fn rejects_malformed() {
        assert!(Version::parse("").is_err());
        assert!(Version::parse("1.2").is_err());
        assert!(Version::parse("1.2.3.4").is_err());
        assert!(Version::parse("1.x.0").is_err());
        assert!(Version::parse("1.2.3-").is_err());
    }

    #[test]
    fn ordering_respects_prerelease_precedence() {
        let release = Version::parse("v1.8.0").unwrap();
        let pre = Version::parse("v1.8.0-rc.1").unwrap();
        assert!(pre < release);
        assert!(Version::parse("v1.7.9").unwrap() < release);
        assert!(Version::parse("v2.0.0").unwrap() > release);
    }

    #[test]
    fn compatibility_and_short() {
        let client = Version::new(1, 8, 2);
        let server = Version::new(1, 7, 0);
        assert!(client.is_compatible_with(&server));
        assert!(!server.is_compatible_with(&client));
        assert!(!Version::new(2, 0, 0).is_compatible_with(&server));
        assert_eq!(client.short(), "1.8");
    }

    #[test]
    fn increment_helpers() {
        let v = Version::parse("v1.7.3-rc.1").unwrap();
        assert_eq!(v.next_major(), Version::new(2, 0, 0));
        assert_eq!(v.next_minor(), Version::new(1, 8, 0));
        assert_eq!(v.next_patch(), Version::new(1, 7, 4));
        assert_eq!(v.to_release(), Version::new(1, 7, 3));
        assert!(!v.to_release().is_pre_release());
    }

    #[test]
    fn upgrade_rules() {
        let from = Version::new(1, 7, 0);
        assert!(from.is_upgrade_allowed_to(&Version::new(1, 8, 5)));
        assert!(from.is_upgrade_allowed_to(&Version::new(1, 7, 4)));
        assert!(!from.is_upgrade_allowed_to(&Version::new(1, 9, 0))); // skips a minor
        assert!(!from.is_upgrade_allowed_to(&Version::new(1, 6, 0))); // downgrade
        assert!(from.is_upgrade_allowed_to(&Version::new(2, 0, 0))); // single major to .0
        assert!(!from.is_upgrade_allowed_to(&Version::new(2, 1, 0)));
        assert!(!from.is_upgrade_allowed_to(&Version::new(3, 0, 0)));
    }

    #[test]
    fn contract_packing() {
        assert_eq!(Version::new(1, 8, 0).contract(), 108);
        assert_eq!(Version::new(1, 10, 2).contract(), 110);
        assert!(Version::new(1, 8, 0).contract() > Version::new(1, 7, 9).contract());
    }

    #[test]
    fn version_range_contains_and_clamp() {
        let range = VersionRange::new(Version::new(1, 28, 0), Version::new(1, 31, 0)).unwrap();
        assert!(range.contains(&Version::new(1, 29, 4)));
        assert!(!range.contains(&Version::new(1, 27, 0)));
        assert!(!range.contains(&Version::new(1, 32, 0)));

        assert_eq!(
            *range.clamp(&Version::new(1, 20, 0)),
            Version::new(1, 28, 0)
        );
        assert_eq!(*range.clamp(&Version::new(2, 0, 0)), Version::new(1, 31, 0));
        assert_eq!(
            *range.clamp(&Version::new(1, 30, 0)),
            Version::new(1, 30, 0)
        );

        assert!(VersionRange::new(Version::new(2, 0, 0), Version::new(1, 0, 0)).is_err());
        assert_eq!(range.to_string(), "v1.28.0..=v1.31.0");
    }

    #[test]
    fn build_short_matches_upstream() {
        assert_eq!(build::short("Talos", "v1.8.0"), "Talos v1.8.0");
        assert_eq!(build::short("", ""), " ");
    }

    #[test]
    fn build_trim_matches_upstream() {
        // Captured from running the upstream `version.Trim` regex.
        assert_eq!(build::trim("v0.3.2-1-abcd"), "v0.3.2-1-abcd");
        assert_eq!(build::trim("v0.3.2-1-gabcdef"), "v0.3.2");
        assert_eq!(build::trim("v0.3.2-1-gabcdef-dirty"), "v0.3.2");
        assert_eq!(build::trim("v0.3.2"), "v0.3.2");
        assert_eq!(build::trim("v1.8.0-alpha.1"), "v1.8.0-alpha.1");
        assert_eq!(build::trim("v1.8.0-12"), "v1.8.0");
        assert_eq!(build::trim("v1.8.0-12-dirty"), "v1.8.0");
        assert_eq!(build::trim("v0.3.2-dirty"), "v0.3.2-dirty");
        assert_eq!(build::trim("v1.2.3-1-g0a1b2c3-dirty"), "v1.2.3");
    }

    #[test]
    fn build_long_version_matches_template() {
        let info = build::VersionInfo {
            tag: "v1.8.0".into(),
            sha: "abcdef0".into(),
            built: "2024-01-01".into(),
            go_version: "go1.26".into(),
            os: "linux".into(),
            arch: "amd64".into(),
        };
        assert_eq!(
            build::long_version(&info),
            "\tTag:         v1.8.0\n\tSHA:         abcdef0\n\tBuilt:       2024-01-01\n\tGo version:  go1.26\n\tOS/Arch:     linux/amd64\n",
        );
    }
}
