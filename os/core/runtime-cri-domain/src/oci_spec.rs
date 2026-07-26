//! OCI runtime spec construction.
//!
//! Mirrors `pkg/containers` / `oci.SpecOpts` used by Talos to build the
//! `runtime-spec` config.json passed to containerd's runc shim. We model the
//! subset that Talos actually configures for system containers: process,
//! mounts, capabilities, namespaces, and the read-only-root toggle.

use os_kernel::error::{Error, Result};

/// A single bind/volume mount in the OCI spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    /// In-container path.
    pub destination: String,
    /// Host path (for bind mounts) or source identifier.
    pub source: String,
    /// Mount type (e.g. `bind`, `tmpfs`).
    pub mount_type: String,
    /// Mount options (e.g. `rbind`, `ro`, `nosuid`).
    pub options: Vec<String>,
}

impl Mount {
    /// A read-only recursive bind mount, the most common system mount.
    pub fn ro_bind(source: impl Into<String>, destination: impl Into<String>) -> Self {
        Mount {
            destination: destination.into(),
            source: source.into(),
            mount_type: "bind".to_string(),
            options: vec!["rbind".to_string(), "ro".to_string()],
        }
    }

    /// A read-write recursive bind mount.
    pub fn rw_bind(source: impl Into<String>, destination: impl Into<String>) -> Self {
        Mount {
            destination: destination.into(),
            source: source.into(),
            mount_type: "bind".to_string(),
            options: vec!["rbind".to_string(), "rw".to_string()],
        }
    }

    /// Whether this mount is read-only.
    pub fn is_read_only(&self) -> bool {
        self.options.iter().any(|o| o == "ro")
    }
}

/// Linux namespace kinds the spec may request the runtime to create or join.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxNamespace {
    Pid,
    Network,
    Mount,
    Ipc,
    Uts,
    User,
    Cgroup,
}

/// The process portion of the OCI spec.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Process {
    /// argv; index 0 is the binary.
    pub args: Vec<String>,
    /// Environment variables as `KEY=VALUE`.
    pub env: Vec<String>,
    /// Working directory; defaults to `/`.
    pub cwd: String,
    /// Bounding/effective capability set (without the `CAP_` prefix duplicated).
    pub capabilities: Vec<String>,
    /// Run with terminal allocated.
    pub terminal: bool,
}

/// Linux resource limits (cgroup constraints) applied to the container.
///
/// Mirrors the subset of `runtime-spec`'s `LinuxResources` Talos configures for
/// system services: a memory limit, a CPU quota/period pair, and an OOM-score
/// adjustment. Talos pins critical services (etcd, kubelet) with a strongly
/// negative `oom_score_adj` so they survive memory pressure.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LinuxResources {
    /// Hard memory limit in bytes (`None` = unlimited).
    pub memory_limit: Option<u64>,
    /// CPU quota in microseconds per `cpu_period` (`None` = unlimited).
    pub cpu_quota: Option<u64>,
    /// CPU period in microseconds (defaults to 100000 when a quota is set).
    pub cpu_period: Option<u64>,
    /// OOM score adjustment in the range `-1000..=1000`.
    pub oom_score_adj: Option<i32>,
}

impl LinuxResources {
    /// The conventional default CPU period (100ms) in microseconds.
    pub const DEFAULT_CPU_PERIOD: u64 = 100_000;

    /// Validate the resource constraints.
    pub fn validate(&self) -> Result<()> {
        if let Some(0) = self.memory_limit {
            return Err(Error::invalid("memory limit must be non-zero"));
        }
        if let Some(adj) = self.oom_score_adj
            && !(-1000..=1000).contains(&adj)
        {
            return Err(Error::invalid("oom_score_adj out of range (-1000..=1000)"));
        }
        if self.cpu_quota.is_some() {
            let period = self.cpu_period.unwrap_or(Self::DEFAULT_CPU_PERIOD);
            if period == 0 {
                return Err(Error::invalid("cpu period must be non-zero"));
            }
        }
        if let Some(0) = self.cpu_quota {
            return Err(Error::invalid("cpu quota must be non-zero"));
        }
        Ok(())
    }

    /// The effective fractional number of CPUs implied by quota/period.
    #[allow(clippy::cast_precision_loss)] // quota/period are small µs values; f64 is exact here
    pub fn effective_cpus(&self) -> Option<f64> {
        self.cpu_quota.map(|q| {
            let period = self.cpu_period.unwrap_or(Self::DEFAULT_CPU_PERIOD);
            q as f64 / period as f64
        })
    }
}

/// An OCI runtime spec (`config.json`) builder/model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OciSpec {
    /// OCI spec version, e.g. `1.1.0`.
    pub oci_version: String,
    /// Hostname inside the container.
    pub hostname: String,
    /// The process to run.
    pub process: Process,
    /// Whether the root filesystem is mounted read-only.
    pub root_readonly: bool,
    /// Mounts in addition to the runtime defaults.
    pub mounts: Vec<Mount>,
    /// Linux namespaces to create.
    pub namespaces: Vec<LinuxNamespace>,
    /// Cgroup resource limits.
    pub resources: LinuxResources,
    /// The cgroups path the runtime should place the container in.
    pub cgroups_path: Option<String>,
}

impl Default for OciSpec {
    fn default() -> Self {
        OciSpec {
            oci_version: "1.1.0".to_string(),
            hostname: String::new(),
            process: Process {
                cwd: "/".to_string(),
                ..Process::default()
            },
            root_readonly: false,
            mounts: Vec::new(),
            namespaces: vec![
                LinuxNamespace::Pid,
                LinuxNamespace::Mount,
                LinuxNamespace::Ipc,
                LinuxNamespace::Uts,
            ],
            resources: LinuxResources::default(),
            cgroups_path: None,
        }
    }
}

impl OciSpec {
    /// Start a spec for the given argv. The first element becomes the binary.
    pub fn new(args: Vec<String>) -> Result<Self> {
        if args.is_empty() {
            return Err(Error::invalid("process args must not be empty"));
        }
        let default = OciSpec::default();
        Ok(OciSpec {
            process: Process {
                args,
                ..default.process
            },
            ..default
        })
    }

    /// Builder: set the hostname.
    pub fn with_hostname(mut self, hostname: impl Into<String>) -> Self {
        self.hostname = hostname.into();
        self
    }

    /// Builder: append an environment variable, validating `KEY=VALUE` shape.
    pub fn with_env(mut self, kv: impl Into<String>) -> Result<Self> {
        let kv = kv.into();
        match kv.split_once('=') {
            Some((k, _)) if !k.is_empty() => {
                self.process.env.push(kv);
                Ok(self)
            }
            _ => Err(Error::invalid("env must be KEY=VALUE")),
        }
    }

    /// Builder: add a mount.
    pub fn with_mount(mut self, m: Mount) -> Self {
        self.mounts.push(m);
        self
    }

    /// Builder: mark root read-only (Talos system services run with this set).
    pub fn read_only_root(mut self) -> Self {
        self.root_readonly = true;
        self
    }

    /// Builder: grant a capability.
    pub fn with_capability(mut self, cap: impl Into<String>) -> Self {
        self.process.capabilities.push(cap.into());
        self
    }

    /// Builder: set a hard memory limit in bytes.
    pub fn with_memory_limit(mut self, bytes: u64) -> Self {
        self.resources.memory_limit = Some(bytes);
        self
    }

    /// Builder: set a CPU quota (microseconds per default 100ms period).
    pub fn with_cpu_quota(mut self, quota_us: u64) -> Self {
        self.resources.cpu_quota = Some(quota_us);
        self
    }

    /// Builder: set the OOM score adjustment (clamped at validation time).
    pub fn with_oom_score_adj(mut self, adj: i32) -> Self {
        self.resources.oom_score_adj = Some(adj);
        self
    }

    /// Builder: set the cgroups path.
    pub fn with_cgroups_path(mut self, path: impl Into<String>) -> Self {
        self.cgroups_path = Some(path.into());
        self
    }

    /// Builder: request host networking by dropping the network namespace.
    pub fn host_network(mut self) -> Self {
        self.namespaces.retain(|n| *n != LinuxNamespace::Network);
        self
    }

    /// Whether the container has its own network namespace.
    pub fn has_network_namespace(&self) -> bool {
        self.namespaces.contains(&LinuxNamespace::Network)
    }

    /// Validate the assembled spec.
    pub fn validate(&self) -> Result<()> {
        if self.process.args.is_empty() {
            return Err(Error::invalid("spec has no process args"));
        }
        if !self.process.args[0].starts_with('/') {
            return Err(Error::invalid("process binary must be an absolute path"));
        }
        for m in &self.mounts {
            if !m.destination.starts_with('/') {
                return Err(Error::invalid("mount destination must be absolute"));
            }
        }
        if self.oci_version.is_empty() {
            return Err(Error::invalid("oci version required"));
        }
        if let Some(p) = &self.cgroups_path
            && !p.starts_with('/')
        {
            return Err(Error::invalid("cgroups path must be absolute"));
        }
        self.resources.validate()?;
        Ok(())
    }

    /// The container's entrypoint binary.
    pub fn binary(&self) -> Option<&str> {
        self.process.args.first().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_namespaces_include_pid_and_mount() {
        let spec = OciSpec::default();
        assert!(spec.namespaces.contains(&LinuxNamespace::Pid));
        assert!(spec.namespaces.contains(&LinuxNamespace::Mount));
        assert!(!spec.root_readonly);
        assert_eq!(spec.process.cwd, "/");
    }

    #[test]
    fn builder_assembles_and_validates() {
        let spec = OciSpec::new(vec!["/usr/local/bin/etcd".to_string()])
            .unwrap()
            .with_hostname("controlplane-1")
            .with_env("ETCD_NAME=cp1")
            .unwrap()
            .with_mount(Mount::rw_bind("/var/lib/etcd", "/var/lib/etcd"))
            .with_capability("CAP_NET_BIND_SERVICE")
            .read_only_root();
        assert!(spec.validate().is_ok());
        assert_eq!(spec.binary(), Some("/usr/local/bin/etcd"));
        assert!(spec.root_readonly);
        assert_eq!(spec.process.env, vec!["ETCD_NAME=cp1".to_string()]);
    }

    #[test]
    fn empty_args_rejected() {
        assert!(OciSpec::new(Vec::new()).is_err());
    }

    #[test]
    fn validate_requires_absolute_binary() {
        let spec = OciSpec::new(vec!["etcd".to_string()]).unwrap();
        assert!(spec.validate().is_err());
    }

    #[test]
    fn env_must_be_key_value() {
        let spec = OciSpec::new(vec!["/bin/sh".to_string()]).unwrap();
        assert!(spec.clone().with_env("NOEQUALS").is_err());
        assert!(spec.with_env("=noval").is_err());
    }

    #[test]
    fn ro_bind_is_read_only() {
        let m = Mount::ro_bind("/etc/ssl", "/etc/ssl");
        assert!(m.is_read_only());
        assert!(!Mount::rw_bind("/a", "/a").is_read_only());
    }

    #[test]
    fn resource_limits_apply_and_validate() {
        let spec = OciSpec::new(vec!["/usr/local/bin/etcd".to_string()])
            .unwrap()
            .with_memory_limit(512 * 1024 * 1024)
            .with_cpu_quota(50_000)
            .with_oom_score_adj(-998)
            .with_cgroups_path("/system/etcd");
        assert!(spec.validate().is_ok());
        assert_eq!(spec.resources.memory_limit, Some(512 * 1024 * 1024));
        assert_eq!(spec.resources.oom_score_adj, Some(-998));
        // 50000us quota / 100000us period = 0.5 CPU.
        assert_eq!(spec.resources.effective_cpus(), Some(0.5));
    }

    #[test]
    fn invalid_resources_rejected() {
        let r = LinuxResources {
            memory_limit: Some(0),
            ..Default::default()
        };
        assert!(r.validate().is_err());

        let r = LinuxResources {
            oom_score_adj: Some(-2000),
            ..Default::default()
        };
        assert!(r.validate().is_err());

        let r = LinuxResources {
            cpu_quota: Some(0),
            ..Default::default()
        };
        assert!(r.validate().is_err());
    }

    #[test]
    fn cgroups_path_must_be_absolute() {
        let spec = OciSpec::new(vec!["/bin/sh".to_string()])
            .unwrap()
            .with_cgroups_path("relative/path");
        assert!(spec.validate().is_err());
    }

    #[test]
    fn host_network_drops_net_namespace() {
        let spec = OciSpec::new(vec!["/bin/sh".to_string()]).unwrap();
        assert!(!spec.has_network_namespace()); // default has none anyway
        let spec = OciSpec {
            namespaces: vec![LinuxNamespace::Network, LinuxNamespace::Pid],
            ..spec
        };
        assert!(spec.has_network_namespace());
        let spec = spec.host_network();
        assert!(!spec.has_network_namespace());
        assert!(spec.namespaces.contains(&LinuxNamespace::Pid));
    }

    #[test]
    fn effective_cpus_none_when_unlimited() {
        assert_eq!(LinuxResources::default().effective_cpus(), None);
    }
}
