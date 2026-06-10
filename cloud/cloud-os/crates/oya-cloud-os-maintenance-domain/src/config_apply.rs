//! The `ApplyConfiguration` flow served in maintenance mode.
//!
//! This mirrors `internal/app/maintenance/server.ApplyConfiguration` plus the
//! shared apply-config machinery in `internal/app/machined`. In maintenance the
//! node has no config yet, so the only meaningful apply modes are the ones that
//! lead the node *out* of maintenance:
//!
//! - [`ApplyMode::Reboot`] — persist the config and reboot so the normal boot
//!   sequence picks it up (and runs an install if the config requests one).
//! - [`ApplyMode::Auto`] — like `Reboot` for the first config in maintenance.
//!
//! The "try" and "no-reboot" staged modes that mutate a *running* config are
//! rejected in maintenance, because there is nothing running to patch. Talos
//! enforces the same restriction.
//!
//! Config is validated with the maintenance validation mode, persisted through
//! the [`ConfigSink`] boundary (normally `/system/state/config.yaml`), and the
//! outcome tells the caller whether to reboot and whether an install is needed.

use std::fmt;

/// How an applied configuration should take effect.
///
/// Mirrors the Talos `machine.ApplyConfigurationRequest_Mode` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyMode {
    /// Persist and reboot to apply (the only fully-supported maintenance mode).
    Reboot,
    /// Auto: behaves like `Reboot` when no config exists yet.
    Auto,
    /// Apply to the running config immediately without reboot. Not valid in
    /// maintenance (nothing is running yet).
    NoReboot,
    /// Stage the config to be applied on the next reboot. Not valid in
    /// maintenance.
    Staged,
    /// Validate only ("try"), with an automatic revert. Not valid in
    /// maintenance.
    Try,
}

impl ApplyMode {
    /// The wire name used by the gRPC API.
    pub fn wire_name(self) -> &'static str {
        match self {
            ApplyMode::Reboot => "REBOOT",
            ApplyMode::Auto => "AUTO",
            ApplyMode::NoReboot => "NO_REBOOT",
            ApplyMode::Staged => "STAGED",
            ApplyMode::Try => "TRY",
        }
    }

    /// Parse an apply mode from its wire name (case-insensitive).
    pub fn parse(name: &str) -> Result<Self, ApplyError> {
        match name.to_ascii_uppercase().as_str() {
            "REBOOT" => Ok(ApplyMode::Reboot),
            "AUTO" => Ok(ApplyMode::Auto),
            "NO_REBOOT" | "NOREBOOT" => Ok(ApplyMode::NoReboot),
            "STAGED" | "STAGE" => Ok(ApplyMode::Staged),
            "TRY" => Ok(ApplyMode::Try),
            other => Err(ApplyError::InvalidMode(other.to_string())),
        }
    }

    /// Whether this mode is permitted while the node is in maintenance.
    ///
    /// Only modes that lead to a reboot (`REBOOT`/`AUTO`) are allowed; the
    /// modes that mutate a running config are rejected because there is no
    /// running config yet.
    pub fn allowed_in_maintenance(self) -> bool {
        matches!(self, ApplyMode::Reboot | ApplyMode::Auto)
    }

    /// Whether applying in this mode results in a reboot.
    pub fn reboots(self) -> bool {
        matches!(self, ApplyMode::Reboot | ApplyMode::Auto)
    }
}

/// The input to an apply-configuration call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyConfigInput {
    /// Raw machine-config bytes (YAML multi-document blob in the real world).
    pub data: Vec<u8>,
    /// The requested apply mode.
    pub mode: ApplyMode,
    /// Whether the config explicitly requests a disk install (`machine.install`
    /// present). Determined by the validator; carried so the outcome can signal
    /// the install flow.
    pub dry_run: bool,
}

impl ApplyConfigInput {
    /// Build a reboot-mode apply input from raw bytes.
    pub fn reboot(data: impl Into<Vec<u8>>) -> Self {
        ApplyConfigInput {
            data: data.into(),
            mode: ApplyMode::Reboot,
            dry_run: false,
        }
    }

    /// Set the apply mode.
    pub fn with_mode(mut self, mode: ApplyMode) -> Self {
        self.mode = mode;
        self
    }

    /// Mark this as a dry-run (validate only, do not persist).
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }
}

/// Errors that can occur while applying a configuration in maintenance mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyError {
    /// The config bytes were empty.
    Empty,
    /// The apply mode string could not be parsed.
    InvalidMode(String),
    /// The apply mode is not permitted in maintenance mode.
    ModeNotAllowedInMaintenance(ApplyMode),
    /// The config failed validation; carries the human-readable reasons.
    Validation(Vec<String>),
    /// Persisting the config through the [`ConfigSink`] failed.
    Persist(String),
    /// The node was not in a state where a config could be applied.
    WrongState(String),
}

impl fmt::Display for ApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApplyError::Empty => write!(f, "empty configuration"),
            ApplyError::InvalidMode(m) => write!(f, "invalid apply mode '{m}'"),
            ApplyError::ModeNotAllowedInMaintenance(m) => {
                write!(
                    f,
                    "apply mode {} not allowed in maintenance mode",
                    m.wire_name()
                )
            }
            ApplyError::Validation(reasons) => {
                write!(f, "config validation failed: {}", reasons.join("; "))
            }
            ApplyError::Persist(reason) => write!(f, "failed to persist config: {reason}"),
            ApplyError::WrongState(s) => write!(f, "cannot apply config: {s}"),
        }
    }
}

impl std::error::Error for ApplyError {}

/// A parsed/validated configuration ready to be persisted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredConfig {
    /// The raw bytes as written to disk.
    pub data: Vec<u8>,
    /// The detected config version (e.g. `v1alpha1`).
    pub version: String,
    /// Whether the config requests an install (`machine.install.disk` present).
    pub requests_install: bool,
    /// The install disk requested, if any.
    pub install_disk: Option<String>,
}

/// Validates raw maintenance-mode config bytes.
///
/// Real Talos parses the multi-doc YAML and runs the full
/// `config.Validate(validation.ModeMaintenance)` rule set. This trait models
/// that boundary; [`DefaultConfigValidator`] implements a faithful subset:
/// non-empty, recognizable `version:`/`kind:` line, and extraction of the
/// install disk.
pub trait ConfigValidator {
    /// Validate and parse the config bytes into a [`StoredConfig`].
    fn validate(&self, data: &[u8]) -> Result<StoredConfig, Vec<String>>;
}

/// A dependency-free maintenance config validator.
///
/// It treats the input as a textual machine config and enforces the minimal
/// invariants the maintenance flow needs before persisting:
///
/// - the blob is non-empty and valid UTF-8,
/// - it declares a known top-level `version:` (only `v1alpha1` is supported by
///   maintenance apply, matching Talos),
/// - it declares `machine.type` (controlplane/worker/init),
/// - if a `machine.install.disk` is present it is recorded so the boot sequence
///   can run the installer.
#[derive(Debug, Default, Clone)]
pub struct DefaultConfigValidator;

impl DefaultConfigValidator {
    /// A new validator.
    pub fn new() -> Self {
        Self
    }

    fn scalar<'a>(text: &'a str, key: &str) -> Option<&'a str> {
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix(key) {
                let rest = rest.trim_start();
                if let Some(value) = rest.strip_prefix(':') {
                    return Some(value.trim().trim_matches(['"', '\'']));
                }
            }
        }
        None
    }
}

impl ConfigValidator for DefaultConfigValidator {
    fn validate(&self, data: &[u8]) -> Result<StoredConfig, Vec<String>> {
        let mut errors = Vec::new();

        if data.is_empty() {
            return Err(vec!["configuration is empty".to_string()]);
        }
        let text = match std::str::from_utf8(data) {
            Ok(t) => t,
            Err(_) => return Err(vec!["configuration is not valid UTF-8".to_string()]),
        };
        if text.trim().is_empty() {
            return Err(vec!["configuration is blank".to_string()]);
        }

        let version = match Self::scalar(text, "version") {
            Some(v) if !v.is_empty() => v.to_string(),
            _ => {
                errors.push("missing required field: version".to_string());
                String::new()
            }
        };
        if !version.is_empty() && version != "v1alpha1" {
            errors.push(format!(
                "unsupported config version '{version}': maintenance apply supports v1alpha1"
            ));
        }

        match Self::scalar(text, "type") {
            Some("controlplane" | "worker" | "init") => {}
            Some(t) => errors.push(format!("invalid machine.type '{t}'")),
            None => errors.push("missing required field: machine.type".to_string()),
        }

        let install_disk = Self::scalar(text, "disk")
            .map(str::to_string)
            .filter(|d| !d.is_empty());
        let requests_install = install_disk.is_some();

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(StoredConfig {
            data: data.to_vec(),
            version,
            requests_install,
            install_disk,
        })
    }
}

/// The persistence boundary for an applied configuration.
///
/// In a real node this writes `/system/state/config.yaml` (and possibly stages
/// it). The in-memory implementation keeps the last persisted config so tests
/// can assert on it.
pub trait ConfigSink {
    /// Persist the validated configuration. Returns an error string on failure.
    fn persist(&mut self, config: &StoredConfig) -> Result<(), String>;

    /// The most recently persisted config, if any.
    fn current(&self) -> Option<&StoredConfig>;
}

/// In-memory [`ConfigSink`] used by tests.
#[derive(Debug, Default, Clone)]
pub struct InMemoryConfigSink {
    stored: Option<StoredConfig>,
    fail_next: bool,
    persist_count: usize,
}

impl InMemoryConfigSink {
    /// A new, empty sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// Make the next [`persist`](ConfigSink::persist) call fail (fault
    /// injection for tests).
    pub fn fail_next_persist(&mut self) {
        self.fail_next = true;
    }

    /// How many times persist has succeeded.
    pub fn persist_count(&self) -> usize {
        self.persist_count
    }
}

impl ConfigSink for InMemoryConfigSink {
    fn persist(&mut self, config: &StoredConfig) -> Result<(), String> {
        if self.fail_next {
            self.fail_next = false;
            return Err("disk write failed (injected)".to_string());
        }
        self.stored = Some(config.clone());
        self.persist_count += 1;
        Ok(())
    }

    fn current(&self) -> Option<&StoredConfig> {
        self.stored.as_ref()
    }
}

/// The result of a successful apply-configuration call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyConfigOutcome {
    /// The config that was persisted (or would have been, for a dry run).
    pub stored: StoredConfig,
    /// Whether the node should reboot to leave maintenance.
    pub reboot: bool,
    /// Whether the boot sequence should run a disk install.
    pub install: bool,
    /// Whether this was a dry run (nothing persisted).
    pub dry_run: bool,
}

/// Run the maintenance apply-configuration flow: validate, check the apply
/// mode, and persist.
///
/// This is the pure core used by [`crate::server::MaintenanceServer`]; it does
/// not itself reboot — it reports whether a reboot/install is needed so the
/// caller can drive the OS boundaries.
pub fn apply_configuration<V: ConfigValidator, S: ConfigSink>(
    validator: &V,
    sink: &mut S,
    input: &ApplyConfigInput,
) -> Result<ApplyConfigOutcome, ApplyError> {
    if input.data.is_empty() {
        return Err(ApplyError::Empty);
    }
    if !input.mode.allowed_in_maintenance() {
        return Err(ApplyError::ModeNotAllowedInMaintenance(input.mode));
    }

    let stored = validator
        .validate(&input.data)
        .map_err(ApplyError::Validation)?;

    let install = stored.requests_install;

    if input.dry_run {
        return Ok(ApplyConfigOutcome {
            stored,
            reboot: false,
            install: false,
            dry_run: true,
        });
    }

    sink.persist(&stored).map_err(ApplyError::Persist)?;

    Ok(ApplyConfigOutcome {
        stored,
        reboot: input.mode.reboots(),
        install,
        dry_run: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD_CONFIG: &str = "version: v1alpha1\nmachine:\n  type: controlplane\n";
    const GOOD_WITH_INSTALL: &str =
        "version: v1alpha1\nmachine:\n  type: worker\n  install:\n    disk: /dev/sda\n";

    #[test]
    fn apply_mode_parse_and_wire_roundtrip() {
        for m in [
            ApplyMode::Reboot,
            ApplyMode::Auto,
            ApplyMode::NoReboot,
            ApplyMode::Staged,
            ApplyMode::Try,
        ] {
            assert_eq!(ApplyMode::parse(m.wire_name()).unwrap(), m);
        }
        assert_eq!(ApplyMode::parse("noreboot").unwrap(), ApplyMode::NoReboot);
        assert!(ApplyMode::parse("bogus").is_err());
    }

    #[test]
    fn only_reboot_modes_allowed_in_maintenance() {
        assert!(ApplyMode::Reboot.allowed_in_maintenance());
        assert!(ApplyMode::Auto.allowed_in_maintenance());
        assert!(!ApplyMode::NoReboot.allowed_in_maintenance());
        assert!(!ApplyMode::Staged.allowed_in_maintenance());
        assert!(!ApplyMode::Try.allowed_in_maintenance());
    }

    #[test]
    fn validator_accepts_good_config() {
        let v = DefaultConfigValidator::new();
        let stored = v.validate(GOOD_CONFIG.as_bytes()).unwrap();
        assert_eq!(stored.version, "v1alpha1");
        assert!(!stored.requests_install);
        assert_eq!(stored.install_disk, None);
    }

    #[test]
    fn validator_extracts_install_disk() {
        let v = DefaultConfigValidator::new();
        let stored = v.validate(GOOD_WITH_INSTALL.as_bytes()).unwrap();
        assert!(stored.requests_install);
        assert_eq!(stored.install_disk.as_deref(), Some("/dev/sda"));
    }

    #[test]
    fn validator_rejects_empty_and_missing_fields() {
        let v = DefaultConfigValidator::new();
        assert!(v.validate(b"").is_err());
        assert!(v.validate(b"   \n  ").is_err());
        let errs = v.validate(b"version: v1alpha1\n").unwrap_err();
        assert!(errs.iter().any(|e| e.contains("machine.type")));
    }

    #[test]
    fn validator_rejects_unsupported_version() {
        let v = DefaultConfigValidator::new();
        let errs = v
            .validate(b"version: v1alpha2\nmachine:\n  type: worker\n")
            .unwrap_err();
        assert!(
            errs.iter()
                .any(|e| e.contains("unsupported config version"))
        );
    }

    #[test]
    fn apply_persists_and_signals_reboot() {
        let v = DefaultConfigValidator::new();
        let mut sink = InMemoryConfigSink::new();
        let input = ApplyConfigInput::reboot(GOOD_CONFIG.as_bytes());
        let out = apply_configuration(&v, &mut sink, &input).unwrap();
        assert!(out.reboot);
        assert!(!out.install);
        assert_eq!(sink.persist_count(), 1);
        assert_eq!(sink.current().unwrap().version, "v1alpha1");
    }

    #[test]
    fn apply_with_install_signals_install() {
        let v = DefaultConfigValidator::new();
        let mut sink = InMemoryConfigSink::new();
        let input = ApplyConfigInput::reboot(GOOD_WITH_INSTALL.as_bytes());
        let out = apply_configuration(&v, &mut sink, &input).unwrap();
        assert!(out.reboot);
        assert!(out.install);
    }

    #[test]
    fn apply_rejects_noreboot_mode_in_maintenance() {
        let v = DefaultConfigValidator::new();
        let mut sink = InMemoryConfigSink::new();
        let input = ApplyConfigInput::reboot(GOOD_CONFIG.as_bytes()).with_mode(ApplyMode::NoReboot);
        let err = apply_configuration(&v, &mut sink, &input).unwrap_err();
        assert_eq!(
            err,
            ApplyError::ModeNotAllowedInMaintenance(ApplyMode::NoReboot)
        );
        assert_eq!(sink.persist_count(), 0);
    }

    #[test]
    fn dry_run_does_not_persist() {
        let v = DefaultConfigValidator::new();
        let mut sink = InMemoryConfigSink::new();
        let input = ApplyConfigInput::reboot(GOOD_CONFIG.as_bytes()).dry_run();
        let out = apply_configuration(&v, &mut sink, &input).unwrap();
        assert!(out.dry_run);
        assert!(!out.reboot);
        assert_eq!(sink.persist_count(), 0);
    }

    #[test]
    fn apply_propagates_persist_failure() {
        let v = DefaultConfigValidator::new();
        let mut sink = InMemoryConfigSink::new();
        sink.fail_next_persist();
        let input = ApplyConfigInput::reboot(GOOD_CONFIG.as_bytes());
        let err = apply_configuration(&v, &mut sink, &input).unwrap_err();
        assert!(matches!(err, ApplyError::Persist(_)));
    }

    #[test]
    fn apply_empty_input_errors() {
        let v = DefaultConfigValidator::new();
        let mut sink = InMemoryConfigSink::new();
        let input = ApplyConfigInput::reboot(Vec::new());
        assert_eq!(
            apply_configuration(&v, &mut sink, &input).unwrap_err(),
            ApplyError::Empty
        );
    }
}
