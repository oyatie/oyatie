//! Sysctl specs (`/proc/sys`). Thin, validated wrapper over
//! [`KernelParamSpec`] of kind [`KernelParamKind::Sysctl`], plus helpers for
//! the common Talos networking sysctls.
//!
//! Mirrors `runtime.KernelParamConfig` for sysctls in Talos machine config.

use crate::kernel_param::{KernelParamError, KernelParamSpec};

/// A sysctl parameter, e.g. `net.ipv4.ip_forward = 1`.
///
/// Newtype over [`KernelParamSpec`] that guarantees the [`KernelParamKind`] is
/// always `Sysctl` and exposes sysctl-flavored constructors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysctlSpec(KernelParamSpec);

impl SysctlSpec {
    /// Build a validated sysctl spec.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Result<Self, KernelParamError> {
        Ok(SysctlSpec(KernelParamSpec::sysctl(key, value)?))
    }

    /// The dotted key (`net.ipv4.ip_forward`).
    pub fn key(&self) -> &str {
        &self.0.key
    }

    /// The desired value.
    pub fn value(&self) -> &str {
        &self.0.value
    }

    /// The `/proc/sys/...` path.
    pub fn path(&self) -> String {
        self.0.path()
    }

    /// Borrow the underlying generic spec.
    pub fn spec(&self) -> &KernelParamSpec {
        &self.0
    }

    /// Consume into the underlying generic spec.
    pub fn into_spec(self) -> KernelParamSpec {
        self.0
    }
}

/// Parse a `key=value` sysctl line (as found in machine config or
/// `sysctl.conf`). Whitespace around the `=` is trimmed; comment lines (`#`/`;`)
/// and blanks yield `Ok(None)`.
pub fn parse_line(line: &str) -> Result<Option<SysctlSpec>, KernelParamError> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
        return Ok(None);
    }
    let (k, v) = line
        .split_once('=')
        .ok_or_else(|| KernelParamError::Parse(line.into()))?;
    Ok(Some(SysctlSpec::new(k.trim(), v.trim())?))
}

/// Parse a multi-line sysctl document into validated specs, skipping comments
/// and blanks.
pub fn parse_document(doc: &str) -> Result<Vec<SysctlSpec>, KernelParamError> {
    let mut out = Vec::new();
    for line in doc.lines() {
        if let Some(spec) = parse_line(line)? {
            out.push(spec);
        }
    }
    Ok(out)
}

/// The handful of network sysctls Talos always sets for Kubernetes to work
/// (bridge netfilter + forwarding). Returns validated specs.
pub fn kubernetes_network_defaults() -> Vec<SysctlSpec> {
    [
        ("net.ipv4.ip_forward", "1"),
        ("net.ipv6.conf.all.forwarding", "1"),
        ("net.bridge.bridge-nf-call-iptables", "1"),
        ("net.bridge.bridge-nf-call-ip6tables", "1"),
    ]
    .into_iter()
    .map(|(k, v)| SysctlSpec::new(k, v).expect("static defaults are valid"))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel_param::KernelParamKind;

    #[test]
    fn build_and_path() {
        let s = SysctlSpec::new("net.ipv4.ip_forward", "1").unwrap();
        assert_eq!(s.key(), "net.ipv4.ip_forward");
        assert_eq!(s.value(), "1");
        assert_eq!(s.path(), "/proc/sys/net/ipv4/ip_forward");
        assert_eq!(s.spec().kind, KernelParamKind::Sysctl);
    }

    #[test]
    fn parse_line_handles_comments_and_spacing() {
        assert_eq!(parse_line("# a comment").unwrap(), None);
        assert_eq!(parse_line("   ").unwrap(), None);
        let s = parse_line("  net.ipv4.ip_forward = 1  ").unwrap().unwrap();
        assert_eq!(s.key(), "net.ipv4.ip_forward");
        assert_eq!(s.value(), "1");
    }

    #[test]
    fn parse_line_rejects_missing_equals() {
        assert!(matches!(
            parse_line("net.ipv4.ip_forward"),
            Err(KernelParamError::Parse(_))
        ));
    }

    #[test]
    fn parse_document_skips_noise() {
        let doc = "# header\nnet.ipv4.ip_forward=1\n\n; semicolon comment\nvm.swappiness = 0\n";
        let specs = parse_document(doc).unwrap();
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[1].key(), "vm.swappiness");
    }

    #[test]
    fn k8s_defaults_present_and_valid() {
        let defs = kubernetes_network_defaults();
        assert_eq!(defs.len(), 4);
        assert!(defs.iter().all(|s| s.spec().validate().is_ok()));
        assert!(defs.iter().any(|s| s.key() == "net.ipv4.ip_forward"));
    }
}
