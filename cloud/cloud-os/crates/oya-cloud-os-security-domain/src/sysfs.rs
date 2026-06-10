//! Sysfs specs (`/sys`). Like [`crate::sysctl`] but for slash-keyed sysfs
//! attributes (transparent hugepages, scheduler tunables, ...).
//!
//! Mirrors `runtime.KernelParamConfig` for sysfs entries in Talos.

use crate::kernel_param::{KernelParamError, KernelParamSpec};

/// A sysfs parameter, e.g.
/// `kernel/mm/transparent_hugepage/enabled = madvise`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysfsSpec(KernelParamSpec);

impl SysfsSpec {
    /// Build a validated sysfs spec.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Result<Self, KernelParamError> {
        Ok(SysfsSpec(KernelParamSpec::sysfs(key, value)?))
    }

    /// The slash key under `/sys`.
    pub fn key(&self) -> &str {
        &self.0.key
    }

    /// The desired value.
    pub fn value(&self) -> &str {
        &self.0.value
    }

    /// The `/sys/...` path.
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

/// Parse a `key = value` sysfs line. The key may be written with `/` or `.`
/// separators in config; dots are normalized to slashes for sysfs. Comments and
/// blanks yield `Ok(None)`.
pub fn parse_line(line: &str) -> Result<Option<SysfsSpec>, KernelParamError> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let (k, v) = line
        .split_once('=')
        .ok_or_else(|| KernelParamError::Parse(line.into()))?;
    let normalized = k.trim().replace('.', "/");
    Ok(Some(SysfsSpec::new(normalized, v.trim())?))
}

/// Parse a multi-line sysfs document.
pub fn parse_document(doc: &str) -> Result<Vec<SysfsSpec>, KernelParamError> {
    let mut out = Vec::new();
    for line in doc.lines() {
        if let Some(spec) = parse_line(line)? {
            out.push(spec);
        }
    }
    Ok(out)
}

/// The sysfs tunables Talos sets for node performance/stability under
/// Kubernetes: transparent hugepages to `madvise` (kubelet/containerd
/// recommendation) and disabling THP defrag stalls.
pub fn kubernetes_sysfs_defaults() -> Vec<SysfsSpec> {
    [
        ("kernel/mm/transparent_hugepage/enabled", "madvise"),
        ("kernel/mm/transparent_hugepage/defrag", "madvise"),
    ]
    .into_iter()
    .map(|(k, v)| SysfsSpec::new(k, v).expect("static defaults are valid"))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel_param::KernelParamKind;

    #[test]
    fn build_and_path() {
        let s = SysfsSpec::new("kernel/mm/transparent_hugepage/enabled", "madvise").unwrap();
        assert_eq!(s.path(), "/sys/kernel/mm/transparent_hugepage/enabled");
        assert_eq!(s.spec().kind, KernelParamKind::Sysfs);
    }

    #[test]
    fn parse_normalizes_dotted_keys() {
        let s = parse_line("kernel.mm.transparent_hugepage.enabled = never")
            .unwrap()
            .unwrap();
        assert_eq!(s.key(), "kernel/mm/transparent_hugepage/enabled");
        assert_eq!(s.value(), "never");
    }

    #[test]
    fn parse_skips_comments() {
        assert_eq!(parse_line("# comment").unwrap(), None);
        assert_eq!(parse_line("").unwrap(), None);
    }

    #[test]
    fn parse_document_collects() {
        let doc = "# devices\nkernel/mm/transparent_hugepage/enabled=madvise\ndevices/system/cpu/cpu0/online=1\n";
        let v = parse_document(doc).unwrap();
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn k8s_sysfs_defaults_present_and_valid() {
        let defs = kubernetes_sysfs_defaults();
        assert_eq!(defs.len(), 2);
        assert!(defs.iter().all(|s| s.spec().validate().is_ok()));
        assert!(
            defs.iter()
                .any(|s| s.key() == "kernel/mm/transparent_hugepage/enabled")
        );
        assert!(defs.iter().all(|s| s.path().starts_with("/sys/")));
    }
}
