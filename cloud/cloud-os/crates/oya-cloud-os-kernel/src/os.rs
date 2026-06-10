//! OS-boundary traits and in-memory implementations.
//!
//! Talos isolates every interaction with the host kernel behind small
//! interfaces so the bulk of the OS can be unit-tested without touching real
//! syscalls. This module mirrors that pattern: it defines the [`Clock`],
//! [`FileSystem`], [`CommandExecutor`] and [`SyscallProvider`] traits together
//! with deterministic, allocation-only in-memory implementations used by tests
//! across the workspace.

use crate::error::{Error, Result};
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::Cell;

// ---------------------------------------------------------------------------
// Clock
// ---------------------------------------------------------------------------

/// Monotonic/wall-clock time source.
///
/// Time is expressed as whole nanoseconds since the Unix epoch (wall) or since
/// an arbitrary start (monotonic). Keeping it integer-based avoids floats in a
/// `no_std` crate.
pub trait Clock {
    /// Nanoseconds since the Unix epoch (wall-clock time).
    fn now_unix_nanos(&self) -> u64;

    /// Nanoseconds since an arbitrary, monotonic start point. Never goes
    /// backwards. Default derives from the wall clock.
    fn monotonic_nanos(&self) -> u64 {
        self.now_unix_nanos()
    }

    /// Whole seconds since the Unix epoch.
    fn now_unix_secs(&self) -> u64 {
        self.now_unix_nanos() / 1_000_000_000
    }
}

/// A deterministic clock whose value can be advanced manually. Ideal for tests
/// that assert on timeouts/intervals without sleeping.
#[derive(Debug)]
pub struct ManualClock {
    nanos: Cell<u64>,
}

impl ManualClock {
    /// Create a clock pinned at `start_nanos` since the epoch.
    pub fn new(start_nanos: u64) -> Self {
        ManualClock {
            nanos: Cell::new(start_nanos),
        }
    }

    /// Advance the clock by `delta` nanoseconds and return the new value.
    pub fn advance_nanos(&self, delta: u64) -> u64 {
        let v = self.nanos.get().saturating_add(delta);
        self.nanos.set(v);
        v
    }

    /// Advance the clock by whole seconds.
    pub fn advance_secs(&self, secs: u64) -> u64 {
        self.advance_nanos(secs.saturating_mul(1_000_000_000))
    }
}

impl Default for ManualClock {
    fn default() -> Self {
        ManualClock::new(0)
    }
}

impl Clock for ManualClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos.get()
    }
}

// ---------------------------------------------------------------------------
// FileSystem
// ---------------------------------------------------------------------------

/// A minimal filesystem abstraction covering the operations Talos services use
/// when reading config, writing state files, and probing for paths.
pub trait FileSystem {
    /// Read the full contents of a file.
    fn read(&self, path: &str) -> Result<Vec<u8>>;

    /// Write (creating or replacing) a file's contents.
    fn write(&mut self, path: &str, data: &[u8]) -> Result<()>;

    /// Remove a file. Errors if it does not exist.
    fn remove(&mut self, path: &str) -> Result<()>;

    /// Whether a path exists.
    fn exists(&self, path: &str) -> bool;

    /// List the immediate child entries of a directory prefix, returning their
    /// full paths sorted lexically.
    fn list(&self, dir: &str) -> Vec<String>;

    /// Convenience: read a file as UTF-8.
    fn read_to_string(&self, path: &str) -> Result<String> {
        let bytes = self.read(path)?;
        String::from_utf8(bytes).map_err(|_| Error::parse(alloc::format!("non-UTF8 file '{path}'")))
    }
}

/// An in-memory filesystem backed by a flat path -> bytes map.
///
/// Directory semantics are emulated by treating `/` as a separator; [`list`]
/// returns the distinct immediate children under a prefix.
#[derive(Debug, Default, Clone)]
pub struct MemoryFs {
    files: BTreeMap<String, Vec<u8>>,
}

impl MemoryFs {
    /// An empty filesystem.
    pub fn new() -> Self {
        MemoryFs {
            files: BTreeMap::new(),
        }
    }

    /// Number of files stored.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether the filesystem is empty.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    fn normalize(path: &str) -> String {
        // Collapse a trailing slash and ensure a single leading slash form is
        // preserved as-is (we treat keys verbatim aside from trailing slash).
        let trimmed = path.trim_end_matches('/');
        if trimmed.is_empty() {
            "/".to_string()
        } else {
            trimmed.to_string()
        }
    }
}

impl FileSystem for MemoryFs {
    fn read(&self, path: &str) -> Result<Vec<u8>> {
        let key = Self::normalize(path);
        self.files
            .get(&key)
            .cloned()
            .ok_or_else(|| Error::not_found(alloc::format!("no such file '{path}'")))
    }

    fn write(&mut self, path: &str, data: &[u8]) -> Result<()> {
        let key = Self::normalize(path);
        if key == "/" {
            return Err(Error::invalid("cannot write to root path"));
        }
        self.files.insert(key, data.to_vec());
        Ok(())
    }

    fn remove(&mut self, path: &str) -> Result<()> {
        let key = Self::normalize(path);
        self.files
            .remove(&key)
            .map(|_| ())
            .ok_or_else(|| Error::not_found(alloc::format!("no such file '{path}'")))
    }

    fn exists(&self, path: &str) -> bool {
        self.files.contains_key(&Self::normalize(path))
    }

    fn list(&self, dir: &str) -> Vec<String> {
        let prefix = {
            let d = Self::normalize(dir);
            if d == "/" {
                "/".to_string()
            } else {
                alloc::format!("{d}/")
            }
        };
        let mut out: Vec<String> = Vec::new();
        for key in self.files.keys() {
            if let Some(rest) = key.strip_prefix(&prefix) {
                if rest.is_empty() {
                    continue;
                }
                // Immediate child = up to the next separator.
                let child = match rest.split_once('/') {
                    Some((head, _)) => alloc::format!("{prefix}{head}"),
                    None => key.clone(),
                };
                if !out.contains(&child) {
                    out.push(child);
                }
            }
        }
        out.sort();
        out
    }
}

// ---------------------------------------------------------------------------
// Command execution (exec boundary)
// ---------------------------------------------------------------------------

/// The result of running a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    /// Process exit status code.
    pub exit_code: i32,
    /// Captured standard output.
    pub stdout: Vec<u8>,
    /// Captured standard error.
    pub stderr: Vec<u8>,
}

impl CommandOutput {
    /// Whether the command exited successfully (status 0).
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// stdout decoded lossily as UTF-8 (lossless if valid).
    pub fn stdout_str(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }
}

/// Abstraction over running external programs (the Talos `exec`/`cmd`
/// boundary). Lets controllers be tested without forking real processes.
pub trait CommandExecutor {
    /// Run `program` with `args` and return its captured output.
    fn run(&mut self, program: &str, args: &[&str]) -> Result<CommandOutput>;
}

/// A scripted, in-memory command executor. You register expected
/// `program + args` invocations and the outputs to return, and it records the
/// sequence actually invoked.
#[derive(Debug, Default)]
pub struct MockExecutor {
    responses: BTreeMap<String, CommandOutput>,
    default: Option<CommandOutput>,
    calls: Vec<String>,
}

impl MockExecutor {
    /// A new executor with no programmed responses.
    pub fn new() -> Self {
        MockExecutor {
            responses: BTreeMap::new(),
            default: None,
            calls: Vec::new(),
        }
    }

    fn key(program: &str, args: &[&str]) -> String {
        if args.is_empty() {
            program.to_string()
        } else {
            alloc::format!("{} {}", program, args.join(" "))
        }
    }

    /// Program an exact `program + args` command to return `output`.
    pub fn expect(&mut self, program: &str, args: &[&str], output: CommandOutput) {
        self.responses.insert(Self::key(program, args), output);
    }

    /// Set a fallback output returned for unregistered commands.
    pub fn set_default(&mut self, output: CommandOutput) {
        self.default = Some(output);
    }

    /// The ordered list of command lines that were executed.
    pub fn calls(&self) -> &[String] {
        &self.calls
    }
}

impl CommandExecutor for MockExecutor {
    fn run(&mut self, program: &str, args: &[&str]) -> Result<CommandOutput> {
        let key = Self::key(program, args);
        self.calls.push(key.clone());
        if let Some(out) = self.responses.get(&key) {
            return Ok(out.clone());
        }
        if let Some(out) = &self.default {
            return Ok(out.clone());
        }
        Err(Error::not_found(alloc::format!(
            "no mock response for command '{key}'"
        )))
    }
}

// ---------------------------------------------------------------------------
// Syscall boundary
// ---------------------------------------------------------------------------

/// Selected kernel operations Talos performs directly: hostname, mounts,
/// reboot/shutdown, and kernel module loading. Modeled as a trait so the
/// machined sequence logic can be exercised in tests.
pub trait SyscallProvider {
    /// Set the system hostname.
    fn set_hostname(&mut self, name: &str) -> Result<()>;

    /// Read the current system hostname.
    fn hostname(&self) -> Result<String>;

    /// Mount `source` at `target` with a filesystem type.
    fn mount(&mut self, source: &str, target: &str, fstype: &str) -> Result<()>;

    /// Unmount whatever is mounted at `target`.
    fn unmount(&mut self, target: &str) -> Result<()>;

    /// Whether `target` currently has something mounted on it.
    fn is_mounted(&self, target: &str) -> bool;

    /// Request a reboot. After this the provider reports `has_rebooted`.
    fn reboot(&mut self) -> Result<()>;

    /// Request a power-off.
    fn poweroff(&mut self) -> Result<()>;
}

/// A record of a mount entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountEntry {
    /// The mounted source device/path.
    pub source: String,
    /// The mount point.
    pub target: String,
    /// Filesystem type.
    pub fstype: String,
}

/// The terminal power action an [`InMemorySyscalls`] saw, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    /// `reboot()` was called.
    Reboot,
    /// `poweroff()` was called.
    Poweroff,
}

/// An in-memory [`SyscallProvider`] tracking hostname, mounts and power state.
#[derive(Debug, Default)]
pub struct InMemorySyscalls {
    hostname: Option<String>,
    mounts: Vec<MountEntry>,
    power: Option<PowerAction>,
}

impl InMemorySyscalls {
    /// A fresh provider with no hostname and nothing mounted.
    pub fn new() -> Self {
        InMemorySyscalls::default()
    }

    /// The currently active mount entries, in mount order.
    pub fn mounts(&self) -> &[MountEntry] {
        &self.mounts
    }

    /// The power action requested, if any.
    pub fn power_action(&self) -> Option<PowerAction> {
        self.power
    }

    /// Whether a terminal power action (reboot/poweroff) was requested.
    pub fn is_powered_down(&self) -> bool {
        self.power.is_some()
    }

    fn ensure_live(&self) -> Result<()> {
        if self.power.is_some() {
            return Err(Error::invalid_state("system has been powered down"));
        }
        Ok(())
    }
}

impl SyscallProvider for InMemorySyscalls {
    fn set_hostname(&mut self, name: &str) -> Result<()> {
        self.ensure_live()?;
        if name.is_empty() {
            return Err(Error::invalid("hostname is empty"));
        }
        self.hostname = Some(name.to_string());
        Ok(())
    }

    fn hostname(&self) -> Result<String> {
        self.hostname
            .clone()
            .ok_or_else(|| Error::not_found("hostname not set"))
    }

    fn mount(&mut self, source: &str, target: &str, fstype: &str) -> Result<()> {
        self.ensure_live()?;
        if self.is_mounted(target) {
            return Err(Error::invalid_state(alloc::format!(
                "'{target}' already mounted"
            )));
        }
        self.mounts.push(MountEntry {
            source: source.to_string(),
            target: target.to_string(),
            fstype: fstype.to_string(),
        });
        Ok(())
    }

    fn unmount(&mut self, target: &str) -> Result<()> {
        self.ensure_live()?;
        let before = self.mounts.len();
        self.mounts.retain(|m| m.target != target);
        if self.mounts.len() == before {
            return Err(Error::not_found(alloc::format!(
                "'{target}' is not mounted"
            )));
        }
        Ok(())
    }

    fn is_mounted(&self, target: &str) -> bool {
        self.mounts.iter().any(|m| m.target == target)
    }

    fn reboot(&mut self) -> Result<()> {
        self.ensure_live()?;
        self.power = Some(PowerAction::Reboot);
        Ok(())
    }

    fn poweroff(&mut self) -> Result<()> {
        self.ensure_live()?;
        self.power = Some(PowerAction::Poweroff);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_clock_advances_monotonically() {
        let c = ManualClock::new(1_000_000_000);
        assert_eq!(c.now_unix_secs(), 1);
        c.advance_secs(5);
        assert_eq!(c.now_unix_secs(), 6);
        c.advance_nanos(500);
        assert_eq!(c.now_unix_nanos(), 6_000_000_500);
        assert_eq!(c.monotonic_nanos(), 6_000_000_500);
    }

    #[test]
    fn manual_clock_saturates() {
        let c = ManualClock::new(u64::MAX - 1);
        c.advance_nanos(100);
        assert_eq!(c.now_unix_nanos(), u64::MAX);
    }

    #[test]
    fn memory_fs_read_write_remove() {
        let mut fs = MemoryFs::new();
        assert!(fs.is_empty());
        assert!(!fs.exists("/etc/hostname"));

        fs.write("/etc/hostname", b"node-1").unwrap();
        assert!(fs.exists("/etc/hostname"));
        assert_eq!(fs.read_to_string("/etc/hostname").unwrap(), "node-1");
        assert_eq!(fs.len(), 1);

        fs.remove("/etc/hostname").unwrap();
        assert!(!fs.exists("/etc/hostname"));
        assert!(fs.remove("/etc/hostname").is_err());
        assert!(fs.read("/etc/hostname").is_err());
    }

    #[test]
    fn memory_fs_trailing_slash_normalized() {
        let mut fs = MemoryFs::new();
        fs.write("/var/run/", b"x").unwrap();
        assert!(fs.exists("/var/run"));
        assert!(fs.exists("/var/run/"));
        assert!(fs.write("/", b"x").is_err());
    }

    #[test]
    fn memory_fs_lists_immediate_children() {
        let mut fs = MemoryFs::new();
        fs.write("/etc/kubernetes/admin.conf", b"a").unwrap();
        fs.write("/etc/kubernetes/pki/ca.crt", b"b").unwrap();
        fs.write("/etc/hostname", b"c").unwrap();

        let etc = fs.list("/etc");
        assert_eq!(etc, alloc::vec!["/etc/hostname", "/etc/kubernetes"]);

        let k8s = fs.list("/etc/kubernetes");
        assert_eq!(
            k8s,
            alloc::vec!["/etc/kubernetes/admin.conf", "/etc/kubernetes/pki"]
        );
    }

    #[test]
    fn memory_fs_non_utf8_errors() {
        let mut fs = MemoryFs::new();
        fs.write("/bin", &[0xff, 0xfe]).unwrap();
        assert!(fs.read_to_string("/bin").is_err());
    }

    #[test]
    fn mock_executor_returns_programmed_output() {
        let mut ex = MockExecutor::new();
        ex.expect(
            "etcdctl",
            &["snapshot", "save", "/snap.db"],
            CommandOutput {
                exit_code: 0,
                stdout: b"Snapshot saved".to_vec(),
                stderr: Vec::new(),
            },
        );

        let out = ex
            .run("etcdctl", &["snapshot", "save", "/snap.db"])
            .unwrap();
        assert!(out.success());
        assert_eq!(out.stdout_str(), "Snapshot saved");
        assert_eq!(ex.calls(), &["etcdctl snapshot save /snap.db".to_string()]);
    }

    #[test]
    fn mock_executor_default_and_missing() {
        let mut ex = MockExecutor::new();
        assert!(ex.run("missing", &[]).is_err());

        ex.set_default(CommandOutput {
            exit_code: 1,
            stdout: Vec::new(),
            stderr: b"boom".to_vec(),
        });
        let out = ex.run("anything", &["--flag"]).unwrap();
        assert!(!out.success());
        assert_eq!(out.exit_code, 1);
        assert_eq!(ex.calls().len(), 2);
    }

    #[test]
    fn syscalls_hostname_and_mounts() {
        let mut sys = InMemorySyscalls::new();
        assert!(sys.hostname().is_err());
        sys.set_hostname("talos-cp-1").unwrap();
        assert_eq!(sys.hostname().unwrap(), "talos-cp-1");
        assert!(sys.set_hostname("").is_err());

        sys.mount("/dev/sda1", "/boot", "vfat").unwrap();
        sys.mount("tmpfs", "/run", "tmpfs").unwrap();
        assert!(sys.is_mounted("/boot"));
        assert_eq!(sys.mounts().len(), 2);

        // Double mount of the same target fails.
        assert!(sys.mount("x", "/boot", "ext4").is_err());

        sys.unmount("/boot").unwrap();
        assert!(!sys.is_mounted("/boot"));
        assert!(sys.unmount("/boot").is_err());
    }

    #[test]
    fn syscalls_power_down_is_terminal() {
        let mut sys = InMemorySyscalls::new();
        assert!(!sys.is_powered_down());
        sys.reboot().unwrap();
        assert_eq!(sys.power_action(), Some(PowerAction::Reboot));
        assert!(sys.is_powered_down());

        // Any further syscall fails after power down.
        assert!(sys.set_hostname("x").is_err());
        assert!(sys.mount("a", "/b", "ext4").is_err());
        assert!(sys.poweroff().is_err());
    }
}
