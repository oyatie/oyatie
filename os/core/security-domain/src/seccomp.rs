//! Seccomp profiles.
//!
//! Mirrors Talos `pkg/machinery/config` seccomp-profile handling and the
//! `SeccompProfile` resource the machined writes under
//! `/var/lib/kubelet/seccomp/profiles`. A profile has a default action and an
//! ordered list of syscall rules; evaluating a syscall picks the first rule
//! that names it, else the default action.

use std::collections::BTreeSet;
use std::fmt;

use crate::kernel_param::KernelParamError;

/// The action seccomp takes for a syscall. Names match the OCI / libseccomp
/// `SCMP_ACT_*` constants used in Kubernetes seccomp profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeccompAction {
    /// Allow the syscall.
    Allow,
    /// Return an errno without executing.
    Errno,
    /// Kill the thread.
    KillThread,
    /// Kill the whole process.
    KillProcess,
    /// Send SIGSYS but allow tracing.
    Trap,
    /// Log the syscall and allow it.
    Log,
}

impl SeccompAction {
    /// The OCI string form (`SCMP_ACT_ALLOW`).
    pub fn as_oci(self) -> &'static str {
        match self {
            SeccompAction::Allow => "SCMP_ACT_ALLOW",
            SeccompAction::Errno => "SCMP_ACT_ERRNO",
            SeccompAction::KillThread => "SCMP_ACT_KILL_THREAD",
            SeccompAction::KillProcess => "SCMP_ACT_KILL_PROCESS",
            SeccompAction::Trap => "SCMP_ACT_TRAP",
            SeccompAction::Log => "SCMP_ACT_LOG",
        }
    }

    /// Parse from the OCI string form (case-insensitive).
    pub fn parse(s: &str) -> Result<Self, KernelParamError> {
        match s.trim().to_ascii_uppercase().as_str() {
            "SCMP_ACT_ALLOW" => Ok(SeccompAction::Allow),
            "SCMP_ACT_ERRNO" => Ok(SeccompAction::Errno),
            "SCMP_ACT_KILL_THREAD" | "SCMP_ACT_KILL" => Ok(SeccompAction::KillThread),
            "SCMP_ACT_KILL_PROCESS" => Ok(SeccompAction::KillProcess),
            "SCMP_ACT_TRAP" => Ok(SeccompAction::Trap),
            "SCMP_ACT_LOG" => Ok(SeccompAction::Log),
            other => Err(KernelParamError::InvalidValue(format!(
                "unknown seccomp action: {other}"
            ))),
        }
    }

    /// Whether this action permits the syscall to proceed.
    pub fn is_permissive(self) -> bool {
        matches!(self, SeccompAction::Allow | SeccompAction::Log)
    }
}

impl fmt::Display for SeccompAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_oci())
    }
}

/// A single seccomp rule: one action applied to a set of named syscalls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeccompRule {
    /// The action for the named syscalls.
    pub action: SeccompAction,
    /// The syscall names this rule covers.
    pub names: BTreeSet<String>,
}

impl SeccompRule {
    /// Build a rule from an action and a list of syscall names.
    pub fn new(action: SeccompAction, names: &[&str]) -> Result<Self, KernelParamError> {
        let mut set = BTreeSet::new();
        for n in names {
            let n = n.trim();
            if n.is_empty() {
                return Err(KernelParamError::InvalidValue("empty syscall name".into()));
            }
            set.insert(n.to_string());
        }
        if set.is_empty() {
            return Err(KernelParamError::InvalidValue(
                "rule has no syscalls".into(),
            ));
        }
        Ok(SeccompRule { action, names: set })
    }

    /// Whether this rule names a given syscall.
    pub fn matches(&self, syscall: &str) -> bool {
        self.names.contains(syscall)
    }
}

/// A seccomp profile: a default action plus ordered syscall rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeccompProfile {
    /// The fallback action when no rule matches.
    pub default_action: SeccompAction,
    /// Ordered rules; first matching rule wins.
    pub rules: Vec<SeccompRule>,
}

impl SeccompProfile {
    /// A profile with the given default action and no rules.
    pub fn new(default_action: SeccompAction) -> Self {
        SeccompProfile {
            default_action,
            rules: Vec::new(),
        }
    }

    /// Add a rule, returning self (builder style).
    pub fn with_rule(mut self, rule: SeccompRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Resolve the action for a syscall: first matching rule wins, else the
    /// default action.
    pub fn resolve(&self, syscall: &str) -> SeccompAction {
        self.rules
            .iter()
            .find(|r| r.matches(syscall))
            .map_or(self.default_action, |r| r.action)
    }

    /// Whether a syscall is permitted under this profile.
    pub fn allows(&self, syscall: &str) -> bool {
        self.resolve(syscall).is_permissive()
    }

    /// Validate the profile: a non-permissive default with at least an allow
    /// list is the Kubernetes-recommended shape; an `Allow` default with no
    /// rules is the (insecure) unconfined profile, which we flag.
    pub fn validate(&self) -> Result<(), KernelParamError> {
        if self.default_action == SeccompAction::Allow && self.rules.is_empty() {
            return Err(KernelParamError::InvalidValue(
                "unconfined seccomp profile (allow-all with no rules)".into(),
            ));
        }
        for rule in &self.rules {
            if rule.names.is_empty() {
                return Err(KernelParamError::InvalidValue("empty rule".into()));
            }
        }
        Ok(())
    }

    /// The Kubernetes `RuntimeDefault`-style baseline: deny by default
    /// (`Errno`) but allow a small set of always-safe syscalls.
    pub fn runtime_default() -> Self {
        let allow = SeccompRule::new(
            SeccompAction::Allow,
            &[
                "read",
                "write",
                "exit",
                "exit_group",
                "rt_sigreturn",
                "futex",
            ],
        )
        .expect("static rule is valid");
        SeccompProfile::new(SeccompAction::Errno).with_rule(allow)
    }

    /// All syscall names mentioned by any rule, sorted and deduplicated.
    pub fn syscalls(&self) -> std::collections::BTreeSet<String> {
        let mut out = std::collections::BTreeSet::new();
        for rule in &self.rules {
            out.extend(rule.names.iter().cloned());
        }
        out
    }

    /// Add a single syscall to the rule for `action`, creating a rule for that
    /// action if none exists yet. Returns self (builder style).
    pub fn allow(self, syscall: &str) -> Self {
        self.add_syscall(SeccompAction::Allow, syscall)
    }

    /// Add `syscall` to the (first) rule with `action`, or append a new rule.
    pub fn add_syscall(mut self, action: SeccompAction, syscall: &str) -> Self {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.action == action) {
            rule.names.insert(syscall.to_string());
        } else if let Ok(rule) = SeccompRule::new(action, &[syscall]) {
            self.rules.push(rule);
        }
        self
    }

    /// Merge another profile's rules into this one. Rules sharing the same
    /// action are unioned; otherwise the other profile's rules are appended
    /// (after this profile's, so this profile keeps precedence for conflicts).
    /// The default action is taken from `self` (the base profile).
    pub fn merge(mut self, other: &SeccompProfile) -> Self {
        for orule in &other.rules {
            if let Some(existing) = self.rules.iter_mut().find(|r| r.action == orule.action) {
                existing.names.extend(orule.names.iter().cloned());
            } else {
                self.rules.push(orule.clone());
            }
        }
        self
    }

    /// The on-disk path Talos writes a named profile to under the kubelet
    /// seccomp root (`/var/lib/kubelet/seccomp/profiles/<name>.json`).
    pub fn profile_path(name: &str) -> String {
        format!("/var/lib/kubelet/seccomp/profiles/{name}.json")
    }

    /// Serialize to the OCI/Kubernetes seccomp JSON shape (no external crates).
    /// Produces `{"defaultAction":...,"syscalls":[{"names":[...],"action":...}]}`.
    pub fn to_json(&self) -> String {
        let mut s = String::from("{\"defaultAction\":\"");
        s.push_str(self.default_action.as_oci());
        s.push_str("\",\"syscalls\":[");
        for (i, rule) in self.rules.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str("{\"names\":[");
            for (j, name) in rule.names.iter().enumerate() {
                if j > 0 {
                    s.push(',');
                }
                s.push('"');
                s.push_str(name);
                s.push('"');
            }
            s.push_str("],\"action\":\"");
            s.push_str(rule.action.as_oci());
            s.push_str("\"}");
        }
        s.push_str("]}");
        s
    }
}

impl From<KernelParamError> for SeccompError {
    fn from(e: KernelParamError) -> Self {
        SeccompError(e)
    }
}

/// Thin newtype so seccomp APIs can expose a focused error while reusing the
/// crate-wide [`KernelParamError`] variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeccompError(pub KernelParamError);

impl fmt::Display for SeccompError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "seccomp: {}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_oci_roundtrip() {
        for a in [
            SeccompAction::Allow,
            SeccompAction::Errno,
            SeccompAction::KillThread,
            SeccompAction::KillProcess,
            SeccompAction::Trap,
            SeccompAction::Log,
        ] {
            assert_eq!(SeccompAction::parse(a.as_oci()).unwrap(), a);
        }
        assert_eq!(
            SeccompAction::parse("scmp_act_kill").unwrap(),
            SeccompAction::KillThread
        );
        assert!(SeccompAction::parse("nope").is_err());
    }

    #[test]
    fn rule_construction_validates() {
        assert!(SeccompRule::new(SeccompAction::Allow, &[]).is_err());
        assert!(SeccompRule::new(SeccompAction::Allow, &[""]).is_err());
        let r = SeccompRule::new(SeccompAction::Errno, &["ptrace", "mount"]).unwrap();
        assert!(r.matches("ptrace"));
        assert!(!r.matches("read"));
    }

    #[test]
    fn resolve_first_match_wins() {
        let profile = SeccompProfile::new(SeccompAction::Errno)
            .with_rule(SeccompRule::new(SeccompAction::Allow, &["read", "write"]).unwrap())
            .with_rule(SeccompRule::new(SeccompAction::KillProcess, &["ptrace"]).unwrap());
        assert_eq!(profile.resolve("read"), SeccompAction::Allow);
        assert_eq!(profile.resolve("ptrace"), SeccompAction::KillProcess);
        // Unlisted syscall falls back to default.
        assert_eq!(profile.resolve("mount"), SeccompAction::Errno);
        assert!(profile.allows("read"));
        assert!(!profile.allows("mount"));
    }

    #[test]
    fn unconfined_profile_fails_validation() {
        let unconfined = SeccompProfile::new(SeccompAction::Allow);
        assert!(unconfined.validate().is_err());
        assert!(SeccompProfile::runtime_default().validate().is_ok());
    }

    #[test]
    fn runtime_default_allows_safe_calls_denies_rest() {
        let p = SeccompProfile::runtime_default();
        assert!(p.allows("read"));
        assert!(p.allows("futex"));
        assert_eq!(p.resolve("ptrace"), SeccompAction::Errno);
    }

    #[test]
    fn error_newtype_display() {
        let e: SeccompError = KernelParamError::InvalidValue("x".into()).into();
        assert!(e.to_string().starts_with("seccomp:"));
    }

    #[test]
    fn syscalls_collects_all_names() {
        let p = SeccompProfile::new(SeccompAction::Errno)
            .with_rule(SeccompRule::new(SeccompAction::Allow, &["read", "write"]).unwrap())
            .with_rule(SeccompRule::new(SeccompAction::KillProcess, &["ptrace"]).unwrap());
        let calls = p.syscalls();
        assert_eq!(calls.len(), 3);
        assert!(calls.contains("ptrace"));
    }

    #[test]
    fn allow_builder_unions_into_one_rule() {
        let p = SeccompProfile::new(SeccompAction::Errno)
            .allow("read")
            .allow("write")
            .allow("read");
        // Only one Allow rule, with two distinct names.
        let allow_rules = p
            .rules
            .iter()
            .filter(|r| r.action == SeccompAction::Allow)
            .count();
        assert_eq!(allow_rules, 1);
        assert!(p.allows("read"));
        assert!(p.allows("write"));
        assert!(!p.allows("ptrace"));
    }

    #[test]
    fn merge_unions_same_action_appends_other() {
        let base = SeccompProfile::new(SeccompAction::Errno)
            .with_rule(SeccompRule::new(SeccompAction::Allow, &["read"]).unwrap());
        let extra = SeccompProfile::new(SeccompAction::Allow)
            .with_rule(SeccompRule::new(SeccompAction::Allow, &["write"]).unwrap())
            .with_rule(SeccompRule::new(SeccompAction::KillProcess, &["ptrace"]).unwrap());
        let merged = base.merge(&extra);
        // Default action stays Errno (base wins).
        assert_eq!(merged.default_action, SeccompAction::Errno);
        assert!(merged.allows("read"));
        assert!(merged.allows("write"));
        assert_eq!(merged.resolve("ptrace"), SeccompAction::KillProcess);
    }

    #[test]
    fn profile_path_is_under_kubelet_seccomp_root() {
        assert_eq!(
            SeccompProfile::profile_path("audit"),
            "/var/lib/kubelet/seccomp/profiles/audit.json"
        );
    }

    #[test]
    fn json_serialization_shape() {
        let p = SeccompProfile::new(SeccompAction::Errno)
            .with_rule(SeccompRule::new(SeccompAction::Allow, &["read", "write"]).unwrap());
        let json = p.to_json();
        assert!(json.starts_with("{\"defaultAction\":\"SCMP_ACT_ERRNO\""));
        assert!(json.contains("\"action\":\"SCMP_ACT_ALLOW\""));
        assert!(json.contains("\"read\""));
        assert!(json.contains("\"write\""));
        assert!(json.ends_with("]}"));
    }

    #[test]
    fn json_empty_rules() {
        let p = SeccompProfile::new(SeccompAction::KillProcess);
        assert_eq!(
            p.to_json(),
            "{\"defaultAction\":\"SCMP_ACT_KILL_PROCESS\",\"syscalls\":[]}"
        );
    }
}
