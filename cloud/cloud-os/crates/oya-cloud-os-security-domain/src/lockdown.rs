//! Kernel lockdown mode (`/sys/kernel/security/lockdown`, `lockdown=` cmdline).
//!
//! Mirrors the Talos security controller's handling of the Linux kernel
//! lockdown LSM. Lockdown has three levels — `none`, `integrity`, and
//! `confidentiality` — and the running level is exposed as
//! `/sys/kernel/security/lockdown` formatted like
//! `none [integrity] confidentiality` (brackets mark the active level).
//!
//! Talos prefers booting with `lockdown=confidentiality` (or at minimum
//! `integrity`) when Secure Boot is in use, and the security controller audits
//! the running level against the desired one.

use std::fmt;

use crate::kernel_param::KernelParamError;

/// The path the kernel exposes the active lockdown level at.
pub const LOCKDOWN_SYSFS_PATH: &str = "/sys/kernel/security/lockdown";

/// A kernel lockdown level, ordered weakest-to-strongest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LockdownMode {
    /// No lockdown; all kernel interfaces available.
    None,
    /// Block interfaces that allow modifying the running kernel.
    Integrity,
    /// Integrity, plus blocking interfaces that could leak kernel memory.
    Confidentiality,
}

impl LockdownMode {
    /// All modes weakest-first.
    pub fn all() -> &'static [LockdownMode] {
        &[
            LockdownMode::None,
            LockdownMode::Integrity,
            LockdownMode::Confidentiality,
        ]
    }

    /// The lowercase kernel name (`integrity`).
    pub fn name(self) -> &'static str {
        match self {
            LockdownMode::None => "none",
            LockdownMode::Integrity => "integrity",
            LockdownMode::Confidentiality => "confidentiality",
        }
    }

    /// Parse a lockdown level from its kernel name (case-insensitive). Accepts
    /// an optional surrounding `[..]` (as it appears in the sysfs file).
    pub fn parse(s: &str) -> Result<Self, KernelParamError> {
        let s = s.trim().trim_start_matches('[').trim_end_matches(']');
        match s.to_ascii_lowercase().as_str() {
            "none" => Ok(LockdownMode::None),
            "integrity" => Ok(LockdownMode::Integrity),
            "confidentiality" => Ok(LockdownMode::Confidentiality),
            other => Err(KernelParamError::InvalidValue(format!(
                "unknown lockdown mode: {other}"
            ))),
        }
    }

    /// Parse the bracketed sysfs contents (`none [integrity] confidentiality`)
    /// and return the active level (the bracketed token).
    pub fn parse_sysfs(contents: &str) -> Result<Self, KernelParamError> {
        for token in contents.split_whitespace() {
            if token.starts_with('[') && token.ends_with(']') {
                return LockdownMode::parse(token);
            }
        }
        Err(KernelParamError::Parse(format!(
            "no active lockdown level in: {contents}"
        )))
    }

    /// Whether `self` is at least as strong as `required`.
    pub fn satisfies(self, required: LockdownMode) -> bool {
        self >= required
    }

    /// Whether lockdown is enforced at all (anything above `None`).
    pub fn is_enforced(self) -> bool {
        self != LockdownMode::None
    }

    /// Render the bracketed sysfs line a kernel would print for this active
    /// level, e.g. `none [integrity] confidentiality`.
    pub fn render_sysfs(self) -> String {
        LockdownMode::all()
            .iter()
            .map(|m| {
                if *m == self {
                    format!("[{}]", m.name())
                } else {
                    m.name().to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl fmt::Display for LockdownMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The result of auditing the running lockdown level against a desired minimum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockdownAudit {
    /// The level the kernel reports as active.
    pub active: LockdownMode,
    /// The minimum level required by policy.
    pub required: LockdownMode,
}

impl LockdownAudit {
    /// Audit `active` against the `required` minimum.
    pub fn new(active: LockdownMode, required: LockdownMode) -> Self {
        LockdownAudit { active, required }
    }

    /// Whether the active level satisfies the requirement.
    pub fn is_compliant(&self) -> bool {
        self.active.satisfies(self.required)
    }

    /// A human-readable compliance message.
    pub fn message(&self) -> String {
        if self.is_compliant() {
            format!(
                "lockdown {} satisfies required {}",
                self.active, self.required
            )
        } else {
            format!(
                "lockdown {} is weaker than required {}",
                self.active, self.required
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_is_weakest_to_strongest() {
        assert!(LockdownMode::None < LockdownMode::Integrity);
        assert!(LockdownMode::Integrity < LockdownMode::Confidentiality);
        assert!(LockdownMode::Confidentiality.satisfies(LockdownMode::Integrity));
        assert!(!LockdownMode::Integrity.satisfies(LockdownMode::Confidentiality));
    }

    #[test]
    fn parse_names_and_brackets() {
        assert_eq!(LockdownMode::parse("none").unwrap(), LockdownMode::None);
        assert_eq!(
            LockdownMode::parse("[integrity]").unwrap(),
            LockdownMode::Integrity
        );
        assert_eq!(
            LockdownMode::parse("CONFIDENTIALITY").unwrap(),
            LockdownMode::Confidentiality
        );
        assert!(LockdownMode::parse("bogus").is_err());
    }

    #[test]
    fn parse_sysfs_picks_active_bracketed_level() {
        let active = LockdownMode::parse_sysfs("none [integrity] confidentiality").unwrap();
        assert_eq!(active, LockdownMode::Integrity);
        let none = LockdownMode::parse_sysfs("[none] integrity confidentiality").unwrap();
        assert_eq!(none, LockdownMode::None);
        assert!(LockdownMode::parse_sysfs("none integrity confidentiality").is_err());
    }

    #[test]
    fn render_sysfs_roundtrips_through_parse() {
        for m in LockdownMode::all().iter().copied() {
            let line = m.render_sysfs();
            assert_eq!(LockdownMode::parse_sysfs(&line).unwrap(), m);
        }
        assert_eq!(
            LockdownMode::Integrity.render_sysfs(),
            "none [integrity] confidentiality"
        );
    }

    #[test]
    fn is_enforced() {
        assert!(!LockdownMode::None.is_enforced());
        assert!(LockdownMode::Integrity.is_enforced());
    }

    #[test]
    fn audit_compliance_and_message() {
        let ok = LockdownAudit::new(LockdownMode::Confidentiality, LockdownMode::Integrity);
        assert!(ok.is_compliant());
        assert!(ok.message().contains("satisfies"));

        let bad = LockdownAudit::new(LockdownMode::None, LockdownMode::Integrity);
        assert!(!bad.is_compliant());
        assert!(bad.message().contains("weaker"));
    }

    #[test]
    fn display() {
        assert_eq!(LockdownMode::Confidentiality.to_string(), "confidentiality");
    }
}
