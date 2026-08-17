//! Runtime kernel-parameter (sysctl/sysfs) controllers.
//!
//! Mirrors Talos `internal/app/machined/pkg/controllers/runtime/kernel_param_*`:
//! a `KernelParamConfig` (desired) is reconciled into a `KernelParamStatus`
//! (applied) by writing to `/proc/sys` (sysctl) or `/sys` (sysfs). Talos also
//! computes defaults (`kernel_param_defaults.go`) and records the previous
//! value so it can be restored on teardown.
//!
//! The kernel write surface is modeled by the [`KernelWriter`] trait with an
//! in-memory implementation used by tests.

use crate::reconcile::{
    Controller, Input, Output, ReconcileContext, ReconcileError, ReconcileResult,
};
use os_cosi_domain::resource::ResourceKind;
use os_cosi_domain::{Metadata, Resource};
use os_kernel::{Error, ResourceId, Result};
use std::collections::BTreeMap;

/// Which kernel pseudo-filesystem a parameter targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    /// `/proc/sys/...`, keys written dotted (`net.ipv4.ip_forward`).
    Sysctl,
    /// `/sys/...`, keys are paths.
    Sysfs,
}

impl ParamKind {
    /// The pseudo-fs mount root.
    pub fn root(&self) -> &'static str {
        match self {
            ParamKind::Sysctl => "/proc/sys",
            ParamKind::Sysfs => "/sys",
        }
    }

    /// Translate a logical key into a filesystem path.
    ///
    /// Sysctl keys use `.` separators which map to `/`; sysfs keys are already
    /// path-like.
    pub fn to_path(&self, key: &str) -> String {
        match self {
            ParamKind::Sysctl => format!("{}/{}", self.root(), key.replace('.', "/")),
            ParamKind::Sysfs => format!("{}/{}", self.root(), key.trim_start_matches('/')),
        }
    }
}

/// Validate a kernel parameter key. Keys may not be empty, contain whitespace,
/// or escape the root with `..`.
pub fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(Error::invalid("kernel param key is empty"));
    }
    if key.chars().any(|c| c.is_whitespace()) {
        return Err(Error::invalid("kernel param key contains whitespace"));
    }
    // Reject any path-escape: a ".." segment when split on either separator,
    // or a literal ".." substring (e.g. "net/../root" or "net..forward").
    if key.contains("..") || key.split('/').any(|seg| seg == "..") {
        return Err(Error::invalid("kernel param key escapes root"));
    }
    Ok(())
}

/// The desired kernel parameter (a COSI spec resource).
#[derive(Debug, Clone)]
pub struct KernelParamConfig {
    meta: Metadata,
    /// sysctl or sysfs.
    pub param_kind: ParamKind,
    /// Logical key, e.g. `net.ipv4.ip_forward`.
    pub key: String,
    /// Desired value.
    pub value: String,
    /// If `true`, do not error when the underlying path does not exist.
    pub ignore_missing: bool,
}

impl KernelParamConfig {
    /// Build a desired sysctl parameter.
    pub fn sysctl(key: &str, value: impl Into<String>) -> Result<Self> {
        validate_key(key)?;
        Ok(KernelParamConfig {
            meta: Metadata::new(
                "runtime",
                "KernelParamConfig",
                ResourceId::new(key).unwrap(),
            ),
            param_kind: ParamKind::Sysctl,
            key: key.to_string(),
            value: value.into(),
            ignore_missing: false,
        })
    }

    /// Build a desired sysfs parameter.
    pub fn sysfs(key: &str, value: impl Into<String>) -> Result<Self> {
        validate_key(key)?;
        Ok(KernelParamConfig {
            meta: Metadata::new(
                "runtime",
                "KernelParamConfig",
                ResourceId::new(key).unwrap(),
            ),
            param_kind: ParamKind::Sysfs,
            key: key.to_string(),
            value: value.into(),
            ignore_missing: false,
        })
    }

    /// Set the ignore-missing flag (builder).
    pub fn ignoring_missing(mut self) -> Self {
        self.ignore_missing = true;
        self
    }

    /// The config resource kind.
    pub fn kind() -> ResourceKind {
        ResourceKind::new("runtime", "KernelParamConfig")
    }

    /// The filesystem path this parameter writes to.
    pub fn path(&self) -> String {
        self.param_kind.to_path(&self.key)
    }
}

impl Resource for KernelParamConfig {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }
    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }
    fn spec_fingerprint(&self) -> String {
        format!(
            "kind={};key={};value={};ignore={}",
            self.param_kind.root(),
            self.key,
            self.value,
            self.ignore_missing
        )
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// The applied kernel parameter status (a COSI output resource).
#[derive(Debug, Clone)]
pub struct KernelParamStatus {
    meta: Metadata,
    /// The applied value.
    pub current: String,
    /// The default/previous value captured before applying (for restore).
    pub default: String,
}

impl KernelParamStatus {
    /// The status resource kind.
    pub fn kind() -> ResourceKind {
        ResourceKind::new("runtime", "KernelParamStatus")
    }
}

impl Resource for KernelParamStatus {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }
    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }
    fn spec_fingerprint(&self) -> String {
        format!("current={};default={}", self.current, self.default)
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
}

/// The OS boundary for writing kernel parameters. Real Talos writes to
/// `/proc/sys` and `/sys`; here it is a trait with an in-memory impl.
pub trait KernelWriter {
    /// Read the current value at `path`, or `None` if the path is absent.
    fn read(&self, path: &str) -> Option<String>;

    /// Write `value` to `path`. Returns an error if the path does not exist
    /// (the caller decides whether that is fatal).
    fn write(&mut self, path: &str, value: &str) -> Result<()>;

    /// Whether the path exists.
    fn exists(&self, path: &str) -> bool {
        self.read(path).is_some()
    }
}

/// An in-memory [`KernelWriter`] used by tests and as the default modeling
/// backend. Paths must be pre-populated to model the kernel exposing them.
#[derive(Debug, Default, Clone)]
pub struct InMemoryKernel {
    values: BTreeMap<String, String>,
    /// Paths that exist but whose value starts unset.
    existing: BTreeMap<String, bool>,
}

impl InMemoryKernel {
    /// An empty kernel surface.
    pub fn new() -> Self {
        InMemoryKernel {
            values: BTreeMap::new(),
            existing: BTreeMap::new(),
        }
    }

    /// Declare a path as existing with an initial value.
    pub fn with_path(mut self, path: &str, value: &str) -> Self {
        self.values.insert(path.to_string(), value.to_string());
        self.existing.insert(path.to_string(), true);
        self
    }

    /// Direct read of a stored value (test helper).
    pub fn value_of(&self, path: &str) -> Option<&str> {
        self.values.get(path).map(|s| s.as_str())
    }
}

impl KernelWriter for InMemoryKernel {
    fn read(&self, path: &str) -> Option<String> {
        if *self.existing.get(path).unwrap_or(&false) {
            Some(self.values.get(path).cloned().unwrap_or_default())
        } else {
            None
        }
    }

    fn write(&mut self, path: &str, value: &str) -> Result<()> {
        if !*self.existing.get(path).unwrap_or(&false) {
            return Err(Error::not_found(format!(
                "kernel path {path} does not exist"
            )));
        }
        self.values.insert(path.to_string(), value.to_string());
        Ok(())
    }
}

/// The kernel-parameter controller. Reconciles every [`KernelParamConfig`] by
/// writing through a [`KernelWriter`] and recording a [`KernelParamStatus`].
pub struct KernelParamController<W: KernelWriter> {
    writer: W,
}

impl<W: KernelWriter> KernelParamController<W> {
    /// Build the controller over a kernel writer.
    pub fn new(writer: W) -> Self {
        KernelParamController { writer }
    }

    /// Borrow the underlying writer (e.g. to assert on applied values).
    pub fn writer(&self) -> &W {
        &self.writer
    }
}

impl<W: KernelWriter> Controller for KernelParamController<W> {
    fn name(&self) -> &str {
        "runtime.KernelParamController"
    }

    fn inputs(&self) -> Vec<Input> {
        vec![Input::strong(KernelParamConfig::kind())]
    }

    fn outputs(&self) -> Vec<Output> {
        vec![Output::new(KernelParamStatus::kind())]
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult<()> {
        for cfg in ctx.list(&KernelParamConfig::kind()) {
            let fp = cfg.spec_fingerprint();
            let parsed = parse_config(&fp);
            let path = parsed.param_kind.to_path(&parsed.key);

            let default = self.writer.read(&path).unwrap_or_default();
            let existed = self.writer.exists(&path);

            if !existed {
                if parsed.ignore_missing {
                    continue;
                }
                return Err(ReconcileError::Invalid(format!(
                    "kernel path {path} does not exist"
                )));
            }

            self.writer
                .write(&path, &parsed.value)
                .map_err(|e| ReconcileError::Store(e.to_string()))?;

            let status = KernelParamStatus {
                meta: Metadata::new("runtime", "KernelParamStatus", cfg.metadata().id().clone()),
                current: parsed.value.clone(),
                default,
            };
            ctx.write(Box::new(status))?;
        }
        Ok(())
    }
}

struct ParsedConfig {
    param_kind: ParamKind,
    key: String,
    value: String,
    ignore_missing: bool,
}

fn parse_config(fp: &str) -> ParsedConfig {
    let mut param_kind = ParamKind::Sysctl;
    let mut key = String::new();
    let mut value = String::new();
    let mut ignore_missing = false;
    for part in fp.split(';') {
        if let Some(v) = part.strip_prefix("kind=") {
            param_kind = if v == "/sys" {
                ParamKind::Sysfs
            } else {
                ParamKind::Sysctl
            };
        } else if let Some(v) = part.strip_prefix("key=") {
            key = v.to_string();
        } else if let Some(v) = part.strip_prefix("value=") {
            value = v.to_string();
        } else if let Some(v) = part.strip_prefix("ignore=") {
            ignore_missing = v == "true";
        }
    }
    ParsedConfig {
        param_kind,
        key,
        value,
        ignore_missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_cosi_domain::State;

    #[test]
    fn sysctl_path_translation() {
        let c = KernelParamConfig::sysctl("net.ipv4.ip_forward", "1").unwrap();
        assert_eq!(c.path(), "/proc/sys/net/ipv4/ip_forward");
    }

    #[test]
    fn sysfs_path_translation() {
        let c =
            KernelParamConfig::sysfs("kernel/mm/transparent_hugepage/enabled", "madvise").unwrap();
        assert_eq!(c.path(), "/sys/kernel/mm/transparent_hugepage/enabled");
    }

    #[test]
    fn key_validation() {
        assert!(KernelParamConfig::sysctl("", "1").is_err());
        assert!(KernelParamConfig::sysctl("net. forward", "1").is_err());
        assert!(KernelParamConfig::sysctl("net/../root", "1").is_err());
        assert!(KernelParamConfig::sysctl("net.ipv4.ip_forward", "1").is_ok());
    }

    #[test]
    fn in_memory_kernel_write_requires_existing_path() {
        let mut k = InMemoryKernel::new();
        assert!(k.write("/proc/sys/x", "1").is_err());
        let mut k = InMemoryKernel::new().with_path("/proc/sys/x", "0");
        assert!(k.write("/proc/sys/x", "1").is_ok());
        assert_eq!(k.value_of("/proc/sys/x"), Some("1"));
    }

    fn run(
        state: &mut State,
        ctrl: &mut KernelParamController<InMemoryKernel>,
    ) -> ReconcileResult<()> {
        let mut ctx = ReconcileContext::new(
            state,
            "runtime.KernelParamController",
            vec![KernelParamStatus::kind()],
        );
        ctrl.reconcile(&mut ctx)
    }

    #[test]
    fn applies_config_and_records_status() {
        let mut state = State::new();
        state
            .create(Box::new(
                KernelParamConfig::sysctl("net.ipv4.ip_forward", "1").unwrap(),
            ))
            .unwrap();
        let kernel = InMemoryKernel::new().with_path("/proc/sys/net/ipv4/ip_forward", "0");
        let mut ctrl = KernelParamController::new(kernel);

        run(&mut state, &mut ctrl).unwrap();

        assert_eq!(
            ctrl.writer().value_of("/proc/sys/net/ipv4/ip_forward"),
            Some("1")
        );
        let status = state
            .get("runtime/KernelParamStatus/net.ipv4.ip_forward")
            .unwrap();
        assert_eq!(status.spec_fingerprint(), "current=1;default=0");
    }

    #[test]
    fn missing_path_errors_unless_ignored() {
        let mut state = State::new();
        state
            .create(Box::new(
                KernelParamConfig::sysctl("net.ipv4.absent", "1").unwrap(),
            ))
            .unwrap();
        let mut ctrl = KernelParamController::new(InMemoryKernel::new());
        let err = run(&mut state, &mut ctrl).unwrap_err();
        assert!(matches!(err, ReconcileError::Invalid(_)));

        // Now with ignore_missing the pass succeeds and writes no status.
        let mut state = State::new();
        state
            .create(Box::new(
                KernelParamConfig::sysctl("net.ipv4.absent", "1")
                    .unwrap()
                    .ignoring_missing(),
            ))
            .unwrap();
        let mut ctrl = KernelParamController::new(InMemoryKernel::new());
        run(&mut state, &mut ctrl).unwrap();
        assert!(!state.contains("runtime/KernelParamStatus/net.ipv4.absent"));
    }
}
