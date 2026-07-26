//! Core kernel-parameter abstractions: the COSI [`KernelParamSpec`] /
//! [`KernelParamStatus`] resources, the crate error enum, and the
//! [`KernelParamSink`] trait that models the boundary to the kernel
//! (`/proc/sys` for sysctl, `/sys` for sysfs).
//!
//! Mirrors Talos `pkg/kernel` + the `runtime.KernelParam*` resources, where the
//! controllers read desired specs and write them to the kernel, recording the
//! value that was actually applied (and the default that was overwritten, so it
//! can be restored on teardown).

use std::collections::BTreeMap;
use std::fmt;

/// Errors produced while validating, parsing, or applying kernel parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelParamError {
    /// The parameter key is empty or malformed.
    InvalidKey(String),
    /// The parameter value is empty, out of range, or otherwise invalid.
    InvalidValue(String),
    /// A parameter that was required to exist was not found in the sink.
    NotFound(String),
    /// The sink rejected a write (e.g. read-only / unknown kernel key).
    WriteRejected(String),
    /// Parsing structured input (cmdline, proc path) failed.
    Parse(String),
    /// The operation conflicts with another spec (duplicate key, etc.).
    Conflict(String),
}

impl KernelParamError {
    /// A short, stable kind string for logging/matching.
    pub fn kind(&self) -> &'static str {
        match self {
            KernelParamError::InvalidKey(_) => "invalid_key",
            KernelParamError::InvalidValue(_) => "invalid_value",
            KernelParamError::NotFound(_) => "not_found",
            KernelParamError::WriteRejected(_) => "write_rejected",
            KernelParamError::Parse(_) => "parse",
            KernelParamError::Conflict(_) => "conflict",
        }
    }
}

impl fmt::Display for KernelParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KernelParamError::InvalidKey(m) => write!(f, "invalid kernel param key: {m}"),
            KernelParamError::InvalidValue(m) => write!(f, "invalid kernel param value: {m}"),
            KernelParamError::NotFound(m) => write!(f, "kernel param not found: {m}"),
            KernelParamError::WriteRejected(m) => write!(f, "kernel param write rejected: {m}"),
            KernelParamError::Parse(m) => write!(f, "kernel param parse error: {m}"),
            KernelParamError::Conflict(m) => write!(f, "kernel param conflict: {m}"),
        }
    }
}

/// Convert this crate's error into the workspace-wide [`os_kernel::Error`].
impl From<KernelParamError> for os_kernel::Error {
    fn from(e: KernelParamError) -> Self {
        match e {
            KernelParamError::InvalidKey(m) | KernelParamError::InvalidValue(m) => {
                os_kernel::Error::Invalid(m)
            }
            KernelParamError::NotFound(m) => os_kernel::Error::NotFound(m),
            KernelParamError::WriteRejected(m) => os_kernel::Error::InvalidState(m),
            KernelParamError::Parse(m) => os_kernel::Error::Parse(m),
            KernelParamError::Conflict(m) => os_kernel::Error::Invalid(m),
        }
    }
}

/// Whether a kernel parameter is a sysctl (`/proc/sys/...`) or a sysfs
/// (`/sys/...`) parameter. Talos manages both with the same machinery, but they
/// live at different mount points and use a different path separator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelParamKind {
    /// `/proc/sys`, dotted keys like `net.ipv4.ip_forward`.
    Sysctl,
    /// `/sys`, slash keys like `kernel/mm/transparent_hugepage/enabled`.
    Sysfs,
}

impl KernelParamKind {
    /// The procfs/sysfs mount root for this kind.
    pub fn root(self) -> &'static str {
        match self {
            KernelParamKind::Sysctl => "/proc/sys",
            KernelParamKind::Sysfs => "/sys",
        }
    }

    /// The separator used inside the canonical key for this kind.
    pub fn separator(self) -> char {
        match self {
            KernelParamKind::Sysctl => '.',
            KernelParamKind::Sysfs => '/',
        }
    }
}

/// Desired state of a single kernel parameter — the COSI `KernelParamSpec`.
///
/// Talos lets values be marked *ignore-on-failure* (the kernel may not expose
/// the key on every platform/arch) and remembers whether the value came from an
/// operator (machine config) or from a built-in default (KSPP), which affects
/// precedence during reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelParamSpec {
    /// Canonical key, e.g. `net.ipv4.ip_forward`.
    pub key: String,
    /// Desired value to write.
    pub value: String,
    /// Whether this is a sysctl or sysfs parameter.
    pub kind: KernelParamKind,
    /// If true, a write failure is tolerated and recorded but not fatal.
    pub ignore_failure: bool,
}

impl KernelParamSpec {
    /// Build a validated sysctl spec.
    pub fn sysctl(
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, KernelParamError> {
        Self::new(key, value, KernelParamKind::Sysctl)
    }

    /// Build a validated sysfs spec.
    pub fn sysfs(
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, KernelParamError> {
        Self::new(key, value, KernelParamKind::Sysfs)
    }

    /// Build and validate a spec of a given kind.
    pub fn new(
        key: impl Into<String>,
        value: impl Into<String>,
        kind: KernelParamKind,
    ) -> Result<Self, KernelParamError> {
        let spec = KernelParamSpec {
            key: key.into(),
            value: value.into(),
            kind,
            ignore_failure: false,
        };
        spec.validate()?;
        Ok(spec)
    }

    /// Mark this spec as tolerating write failures.
    pub fn ignoring_failure(mut self) -> Self {
        self.ignore_failure = true;
        self
    }

    /// Validate the key and value.
    ///
    /// Rules (mirroring Talos / the kernel):
    /// * key non-empty, no leading/trailing/double separators, no whitespace,
    /// * value non-empty, single line, no NUL.
    pub fn validate(&self) -> Result<(), KernelParamError> {
        let sep = self.kind.separator();
        if self.key.is_empty() {
            return Err(KernelParamError::InvalidKey("empty key".to_string()));
        }
        if self.key.starts_with(sep) || self.key.ends_with(sep) {
            return Err(KernelParamError::InvalidKey(self.key.clone()));
        }
        if self.key.contains(&[' ', '\t', '\n'][..]) {
            return Err(KernelParamError::InvalidKey(self.key.clone()));
        }
        // No empty path segments (double separators).
        let mut prev = '\0';
        for c in self.key.chars() {
            if c == sep && prev == sep {
                return Err(KernelParamError::InvalidKey(self.key.clone()));
            }
            prev = c;
        }
        if self.value.is_empty() {
            return Err(KernelParamError::InvalidValue("empty value".to_string()));
        }
        if self.value.contains('\n') || self.value.contains('\0') {
            return Err(KernelParamError::InvalidValue(self.value.clone()));
        }
        Ok(())
    }

    /// The absolute kernel path for this parameter. Sysctl keys translate dots
    /// to slashes under `/proc/sys`; sysfs keys are used as-is under `/sys`.
    pub fn path(&self) -> String {
        match self.kind {
            KernelParamKind::Sysctl => {
                let mut p = String::from(self.kind.root());
                p.push('/');
                p.push_str(&self.key.replace('.', "/"));
                p
            }
            KernelParamKind::Sysfs => {
                let mut p = String::from(self.kind.root());
                p.push('/');
                p.push_str(&self.key);
                p
            }
        }
    }
}

/// Observed state of a kernel parameter — the COSI `KernelParamStatus`.
///
/// Records the value currently applied to the kernel, plus the *default* value
/// that was present before Talos took over (so it can be restored on teardown,
/// exactly as `KernelParamConfigController` does).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelParamStatus {
    /// Canonical key.
    pub key: String,
    /// Value currently applied.
    pub current: String,
    /// The default value present before Talos managed this key, if known.
    pub default: Option<String>,
    /// True if this key is unsupported by the running kernel and was skipped.
    pub unsupported: bool,
}

impl KernelParamStatus {
    /// Construct a status for a freshly applied parameter.
    pub fn applied(
        key: impl Into<String>,
        current: impl Into<String>,
        default: Option<String>,
    ) -> Self {
        KernelParamStatus {
            key: key.into(),
            current: current.into(),
            default,
            unsupported: false,
        }
    }

    /// Construct a status for a key the kernel does not support.
    pub fn unsupported(key: impl Into<String>) -> Self {
        KernelParamStatus {
            key: key.into(),
            current: String::new(),
            default: None,
            unsupported: true,
        }
    }

    /// Whether the applied value differs from the recorded default (i.e. Talos
    /// actually changed something that must be restored on teardown).
    pub fn was_changed(&self) -> bool {
        match &self.default {
            Some(d) => *d != self.current,
            None => false,
        }
    }
}

/// The boundary to the kernel. Real implementations write to `/proc/sys` and
/// `/sys`; tests use [`MemoryParamSink`].
///
/// Modeled after the file-IO Talos performs in `pkg/kernel`: read the current
/// value, write a new one. Implementations must reject unknown keys with
/// [`KernelParamError::NotFound`] and read-only keys with
/// [`KernelParamError::WriteRejected`].
pub trait KernelParamSink {
    /// Read the current value of `key`, or `NotFound` if absent.
    fn read(&self, key: &str) -> Result<String, KernelParamError>;

    /// Write `value` to `key`. Implementations may reject unknown/read-only
    /// keys.
    fn write(&mut self, key: &str, value: &str) -> Result<(), KernelParamError>;

    /// Whether the sink exposes `key` at all (used to detect unsupported keys).
    fn exists(&self, key: &str) -> bool {
        self.read(key).is_ok()
    }
}

/// In-memory [`KernelParamSink`] used by the controller in tests and for
/// dry-run reconciliation. Models a fixed set of kernel keys; writing an
/// unknown key fails unless [`MemoryParamSink::allow_unknown`] is set.
#[derive(Debug, Default, Clone)]
pub struct MemoryParamSink {
    values: BTreeMap<String, String>,
    read_only: Vec<String>,
    allow_unknown: bool,
}

impl MemoryParamSink {
    /// An empty sink that rejects writes to keys it does not already know.
    pub fn new() -> Self {
        MemoryParamSink::default()
    }

    /// Allow writes to create previously-unknown keys (used to model platforms
    /// where the key space is open).
    pub fn allowing_unknown(mut self) -> Self {
        self.allow_unknown = true;
        self
    }

    /// Pre-seed a key with a value (its "default").
    pub fn with(mut self, key: &str, value: &str) -> Self {
        self.values.insert(key.to_string(), value.to_string());
        self
    }

    /// Mark a key as read-only (writes are rejected).
    pub fn read_only(mut self, key: &str) -> Self {
        self.read_only.push(key.to_string());
        self
    }
}

impl KernelParamSink for MemoryParamSink {
    fn read(&self, key: &str) -> Result<String, KernelParamError> {
        self.values
            .get(key)
            .cloned()
            .ok_or_else(|| KernelParamError::NotFound(key.to_string()))
    }

    fn write(&mut self, key: &str, value: &str) -> Result<(), KernelParamError> {
        if self.read_only.iter().any(|k| k == key) {
            return Err(KernelParamError::WriteRejected(key.to_string()));
        }
        if !self.allow_unknown && !self.values.contains_key(key) {
            return Err(KernelParamError::NotFound(key.to_string()));
        }
        self.values.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn exists(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sysctl_spec_validates_and_builds_path() {
        let s = KernelParamSpec::sysctl("net.ipv4.ip_forward", "1").unwrap();
        assert_eq!(s.path(), "/proc/sys/net/ipv4/ip_forward");
        assert!(!s.ignore_failure);
    }

    #[test]
    fn sysfs_spec_path_uses_slashes_verbatim() {
        let s =
            KernelParamSpec::sysfs("kernel/mm/transparent_hugepage/enabled", "madvise").unwrap();
        assert_eq!(s.path(), "/sys/kernel/mm/transparent_hugepage/enabled");
    }

    #[test]
    fn validation_rejects_bad_keys_and_values() {
        assert!(matches!(
            KernelParamSpec::sysctl("", "1"),
            Err(KernelParamError::InvalidKey(_))
        ));
        assert!(matches!(
            KernelParamSpec::sysctl(".net.ipv4", "1"),
            Err(KernelParamError::InvalidKey(_))
        ));
        assert!(matches!(
            KernelParamSpec::sysctl("net..ipv4", "1"),
            Err(KernelParamError::InvalidKey(_))
        ));
        assert!(matches!(
            KernelParamSpec::sysctl("net.ipv4.ip_forward", ""),
            Err(KernelParamError::InvalidValue(_))
        ));
        assert!(matches!(
            KernelParamSpec::sysctl("net.ipv4.ip_forward", "1\n2"),
            Err(KernelParamError::InvalidValue(_))
        ));
    }

    #[test]
    fn memory_sink_read_write_and_unknown_rejection() {
        let mut sink = MemoryParamSink::new().with("net.ipv4.ip_forward", "0");
        assert_eq!(sink.read("net.ipv4.ip_forward").unwrap(), "0");
        sink.write("net.ipv4.ip_forward", "1").unwrap();
        assert_eq!(sink.read("net.ipv4.ip_forward").unwrap(), "1");
        // Unknown key is rejected unless allow_unknown is set.
        assert!(matches!(
            sink.write("unknown.key", "1"),
            Err(KernelParamError::NotFound(_))
        ));
    }

    #[test]
    fn memory_sink_read_only_rejects_writes() {
        let mut sink = MemoryParamSink::new()
            .with("kernel.kexec_load_disabled", "0")
            .read_only("kernel.kexec_load_disabled");
        assert!(matches!(
            sink.write("kernel.kexec_load_disabled", "1"),
            Err(KernelParamError::WriteRejected(_))
        ));
    }

    #[test]
    fn status_change_detection() {
        let s = KernelParamStatus::applied("net.ipv4.ip_forward", "1", Some("0".into()));
        assert!(s.was_changed());
        let same = KernelParamStatus::applied("k", "1", Some("1".into()));
        assert!(!same.was_changed());
        let unsup = KernelParamStatus::unsupported("k");
        assert!(unsup.unsupported);
        assert!(!unsup.was_changed());
    }

    #[test]
    fn error_kind_and_conversion() {
        assert_eq!(
            KernelParamError::InvalidKey("x".into()).kind(),
            "invalid_key"
        );
        let e: os_kernel::Error = KernelParamError::NotFound("k".into()).into();
        assert_eq!(e.kind(), "not_found");
    }
}
