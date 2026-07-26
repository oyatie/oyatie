//! Mounting of the core virtual filesystems during early userspace.
//!
//! Talos' PID 1 (`cmd/init` upstream) mounts the kernel pseudo-filesystems
//! before anything else can work: `/proc`, `/sys`, `/dev`, `/run`, plus
//! `devpts`, `shm`, and the cgroup hierarchy. The real mount happens through
//! `mount(2)`; here we model the *policy* — what gets mounted, in what order,
//! with which flags — behind a [`Mounter`] trait so the entire sequence is
//! exercisable on a non-Linux host with an in-memory fake.
//!
//! The Linux PID 1 binary plugs a real `mount(2)`-backed [`Mounter`] into
//! [`mount_essential`]; the tests plug [`RecordingMounter`] in and assert on the
//! recorded operations.

use std::collections::BTreeMap;
use std::fmt;

/// Mount flags mirroring the subset of `MS_*` constants Talos actually uses for
/// the early pseudo-filesystems. Modeled as bitflags without pulling in a
/// crate.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct MountFlags(pub u64);

impl MountFlags {
    /// `MS_NOSUID` — ignore set-user-ID and set-group-ID bits.
    pub const NOSUID: MountFlags = MountFlags(1 << 0);
    /// `MS_NODEV` — disallow access to device special files.
    pub const NODEV: MountFlags = MountFlags(1 << 1);
    /// `MS_NOEXEC` — disallow program execution.
    pub const NOEXEC: MountFlags = MountFlags(1 << 2);
    /// `MS_RELATIME` — update atime relative to mtime/ctime.
    pub const RELATIME: MountFlags = MountFlags(1 << 3);
    /// `MS_RDONLY` — mount read-only.
    pub const RDONLY: MountFlags = MountFlags(1 << 4);

    /// Empty flag set.
    pub const fn empty() -> Self {
        MountFlags(0)
    }

    /// Returns true if `other`'s bits are all set in `self`.
    pub fn contains(self, other: MountFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Union of two flag sets.
    pub fn union(self, other: MountFlags) -> MountFlags {
        MountFlags(self.0 | other.0)
    }
}

impl std::ops::BitOr for MountFlags {
    type Output = MountFlags;
    fn bitor(self, rhs: MountFlags) -> MountFlags {
        self.union(rhs)
    }
}

impl fmt::Debug for MountFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.contains(MountFlags::NOSUID) {
            parts.push("nosuid");
        }
        if self.contains(MountFlags::NODEV) {
            parts.push("nodev");
        }
        if self.contains(MountFlags::NOEXEC) {
            parts.push("noexec");
        }
        if self.contains(MountFlags::RELATIME) {
            parts.push("relatime");
        }
        if self.contains(MountFlags::RDONLY) {
            parts.push("ro");
        }
        if parts.is_empty() {
            write!(f, "(none)")
        } else {
            write!(f, "{}", parts.join(","))
        }
    }
}

/// A single mount to perform: source device/fstype name, mount point, fstype,
/// flags, and optional comma-separated data (e.g. `mode=0755`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MountPoint {
    pub source: String,
    pub target: String,
    pub fstype: String,
    pub flags: MountFlags,
    pub data: Option<String>,
}

impl MountPoint {
    /// Construct a pseudo-filesystem mount (source == fstype name).
    pub fn pseudo(fstype: &str, target: &str, flags: MountFlags) -> Self {
        MountPoint {
            source: fstype.to_string(),
            target: target.to_string(),
            fstype: fstype.to_string(),
            flags,
            data: None,
        }
    }

    /// Attach fs-specific data options.
    pub fn with_data(mut self, data: &str) -> Self {
        self.data = Some(data.to_string());
        self
    }

    /// Set an explicit source distinct from the fstype.
    pub fn with_source(mut self, source: &str) -> Self {
        self.source = source.to_string();
        self
    }
}

/// Outcome of a mount attempt.
///
/// `Skipped` is the best-effort case Talos tolerates: under a sandbox the
/// kernel pseudo-filesystems are frequently *already provided* and re-mounting
/// over the top is forbidden (gVisor's Sentry already presents `/proc`/`/sys`,
/// the kernel pre-mounts `devtmpfs` on `/dev`, an unprivileged container cannot
/// mount at all). Those come back as a small set of expected errnos
/// (`EBUSY`/`EPERM`/`EACCES`/`ENODEV`); the carried string is the classified
/// errno name. On a real privileged kernel the mount simply `Mounted`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MountOutcome {
    Mounted,
    /// Mounting was skipped because the target is already provided / forbidden
    /// under the sandbox. Carries the classified errno name (e.g. `"EBUSY"`).
    Skipped(&'static str),
    Failed(String),
}

impl MountOutcome {
    pub fn is_ok(&self) -> bool {
        matches!(self, MountOutcome::Mounted | MountOutcome::Skipped(_))
    }
}

/// Best-effort pseudo-filesystem mount classifier (re-exported from
/// `talos-machined`). Given the raw OS errno of a failed `mount(2)`, returns
/// `Some(errno_name)` when the failure is a benign "already provided / skip and
/// continue" case (sandbox / unprivileged / already-mounted), or `None` when the
/// error is genuinely unexpected and must fail.
pub use os_machined_domain::boot::mount_skip_reason;

/// Abstraction over the `mount(2)` syscall. The Linux binary supplies a real
/// implementation; tests supply [`RecordingMounter`].
pub trait Mounter {
    /// Ensure the target directory exists (mkdir -p semantics).
    fn ensure_dir(&mut self, target: &str) -> Result<(), String>;
    /// Perform the mount, returning a [`MountOutcome`].
    fn mount(&mut self, mp: &MountPoint) -> MountOutcome;
}

/// The canonical early-boot mount table Talos installs. Order matters: `/proc`
/// and `/sys` first (controllers read them immediately), then `/dev` and its
/// children, then `/run`.
pub fn essential_mounts() -> Vec<MountPoint> {
    let secure = MountFlags::NOSUID | MountFlags::NODEV | MountFlags::NOEXEC;
    vec![
        MountPoint::pseudo(
            "proc",
            "/proc",
            MountFlags::NOSUID | MountFlags::NODEV | MountFlags::NOEXEC | MountFlags::RELATIME,
        ),
        MountPoint::pseudo(
            "sysfs",
            "/sys",
            MountFlags::NOSUID | MountFlags::NODEV | MountFlags::NOEXEC | MountFlags::RELATIME,
        ),
        MountPoint::pseudo("devtmpfs", "/dev", MountFlags::NOSUID).with_data("mode=0755"),
        MountPoint::pseudo("devpts", "/dev/pts", secure).with_data("mode=0620,gid=5,ptmxmode=666"),
        MountPoint::pseudo("tmpfs", "/dev/shm", secure)
            .with_source("shm")
            .with_data("mode=1777"),
        MountPoint::pseudo("tmpfs", "/run", MountFlags::NOSUID | MountFlags::NODEV)
            .with_data("mode=0755"),
        MountPoint::pseudo("tmpfs", "/tmp", secure).with_data("mode=1777"),
    ]
}

/// A single mount step's result, for reporting/inspection.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MountResult {
    pub point: MountPoint,
    pub outcome: MountOutcome,
}

/// Mount the essential pseudo-filesystems in order, tolerating `EBUSY`
/// (already-mounted). Returns the per-mount results. A genuinely failed mount is
/// recorded but does not abort the sequence — Talos logs and continues, because
/// a missing `/tmp` should not block reaching `machined`.
pub fn mount_essential(mounter: &mut dyn Mounter, table: &[MountPoint]) -> Vec<MountResult> {
    let mut results = Vec::with_capacity(table.len());
    for mp in table {
        if let Err(e) = mounter.ensure_dir(&mp.target) {
            results.push(MountResult {
                point: mp.clone(),
                outcome: MountOutcome::Failed(format!("mkdir failed: {e}")),
            });
            continue;
        }
        let outcome = mounter.mount(mp);
        results.push(MountResult {
            point: mp.clone(),
            outcome,
        });
    }
    results
}

/// True if every essential mount succeeded (or was tolerably already mounted).
pub fn all_ok(results: &[MountResult]) -> bool {
    results.iter().all(|r| r.outcome.is_ok())
}

/// Human-readable one-line summary of a mount result, matching the style the
/// PID 1 console prints.
pub fn describe(result: &MountResult) -> String {
    match &result.outcome {
        MountOutcome::Mounted => format!(
            "mount: {} -> {} ({}) ok",
            result.point.source, result.point.target, result.point.fstype
        ),
        MountOutcome::Skipped(reason) => format!(
            "[seq] mount {}: skipped ({reason}, already provided)",
            result.point.target
        ),
        MountOutcome::Failed(e) => format!(
            "mount: {} -> {} ({}) failed: {e} (continuing)",
            result.point.source, result.point.target, result.point.fstype
        ),
    }
}

/// In-memory [`Mounter`] used by tests. Records every directory creation and
/// mount, and can be primed to return specific outcomes (e.g. simulate `EBUSY`
/// on `/dev`).
#[derive(Default)]
pub struct RecordingMounter {
    pub dirs_created: Vec<String>,
    pub mounts: Vec<MountPoint>,
    /// Targets that should report a best-effort skip, keyed by the raw errno the
    /// simulated `mount(2)` returns (classified through [`mount_skip_reason`]).
    pub skip_targets: BTreeMap<String, i32>,
    /// Targets that should report `Failed`.
    pub fail_targets: BTreeMap<String, String>,
    /// Targets for which `ensure_dir` should fail.
    pub mkdir_fail_targets: Vec<String>,
}

impl RecordingMounter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Prime a target to report `EBUSY` (already mounted), the classic
    /// devtmpfs-pre-mounted case.
    pub fn busy(self, target: &str) -> Self {
        self.skipping(target, 16) // EBUSY
    }

    /// Prime a target to fail its `mount(2)` with the given raw errno. Errnos in
    /// the best-effort set become a [`MountOutcome::Skipped`]; anything else
    /// becomes a [`MountOutcome::Failed`].
    pub fn skipping(mut self, target: &str, errno: i32) -> Self {
        self.skip_targets.insert(target.to_string(), errno);
        self
    }

    /// Prime a target to fail with the given message.
    pub fn failing(mut self, target: &str, msg: &str) -> Self {
        self.fail_targets
            .insert(target.to_string(), msg.to_string());
        self
    }

    /// True if a mount with the given target was recorded.
    pub fn mounted(&self, target: &str) -> bool {
        self.mounts.iter().any(|m| m.target == target)
    }
}

impl Mounter for RecordingMounter {
    fn ensure_dir(&mut self, target: &str) -> Result<(), String> {
        if self.mkdir_fail_targets.iter().any(|t| t == target) {
            return Err(format!("permission denied: {target}"));
        }
        self.dirs_created.push(target.to_string());
        Ok(())
    }

    fn mount(&mut self, mp: &MountPoint) -> MountOutcome {
        if let Some(msg) = self.fail_targets.get(&mp.target) {
            return MountOutcome::Failed(msg.clone());
        }
        if let Some(&errno) = self.skip_targets.get(&mp.target) {
            // Classify the simulated errno exactly as the real mount path does:
            // a benign sandbox/already-provided errno is a skip; anything else
            // is a genuine failure.
            return match mount_skip_reason(errno) {
                Some(reason) => MountOutcome::Skipped(reason),
                None => MountOutcome::Failed(format!("mount failed (errno {errno})")),
            };
        }
        self.mounts.push(mp.clone());
        MountOutcome::Mounted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_contains_and_union() {
        let f = MountFlags::NOSUID | MountFlags::NODEV;
        assert!(f.contains(MountFlags::NOSUID));
        assert!(f.contains(MountFlags::NODEV));
        assert!(!f.contains(MountFlags::NOEXEC));
        assert!(f.contains(MountFlags::empty()));
    }

    #[test]
    fn flags_debug_renders_names() {
        let f = MountFlags::NOSUID | MountFlags::NOEXEC | MountFlags::RDONLY;
        assert_eq!(format!("{f:?}"), "nosuid,noexec,ro");
        assert_eq!(format!("{:?}", MountFlags::empty()), "(none)");
    }

    #[test]
    fn essential_mounts_ordered_and_complete() {
        let table = essential_mounts();
        let targets: Vec<&str> = table.iter().map(|m| m.target.as_str()).collect();
        assert_eq!(
            targets,
            vec![
                "/proc", "/sys", "/dev", "/dev/pts", "/dev/shm", "/run", "/tmp"
            ]
        );
        // /proc before /sys before /dev — controllers depend on this ordering.
        let proc_idx = targets.iter().position(|t| *t == "/proc").unwrap();
        let dev_idx = targets.iter().position(|t| *t == "/dev").unwrap();
        let pts_idx = targets.iter().position(|t| *t == "/dev/pts").unwrap();
        assert!(proc_idx < dev_idx);
        assert!(dev_idx < pts_idx);
    }

    #[test]
    fn shm_uses_tmpfs_with_shm_source() {
        let table = essential_mounts();
        let shm = table.iter().find(|m| m.target == "/dev/shm").unwrap();
        assert_eq!(shm.fstype, "tmpfs");
        assert_eq!(shm.source, "shm");
        assert_eq!(shm.data.as_deref(), Some("mode=1777"));
    }

    #[test]
    fn dev_is_nosuid_but_allows_devices() {
        let table = essential_mounts();
        let dev = table.iter().find(|m| m.target == "/dev").unwrap();
        assert!(dev.flags.contains(MountFlags::NOSUID));
        // /dev must NOT be nodev — it holds device nodes.
        assert!(!dev.flags.contains(MountFlags::NODEV));
    }

    #[test]
    fn mount_essential_records_all_in_order() {
        let mut m = RecordingMounter::new();
        let table = essential_mounts();
        let results = mount_essential(&mut m, &table);
        assert_eq!(results.len(), table.len());
        assert!(all_ok(&results));
        assert_eq!(m.mounts.len(), table.len());
        assert!(m.mounted("/proc"));
        assert!(m.mounted("/run"));
        // Dirs created before mounts, same order.
        assert_eq!(m.dirs_created[0], "/proc");
    }

    #[test]
    fn ebusy_is_tolerated() {
        let mut m = RecordingMounter::new().busy("/dev");
        let table = essential_mounts();
        let results = mount_essential(&mut m, &table);
        assert!(all_ok(&results));
        let dev = results.iter().find(|r| r.point.target == "/dev").unwrap();
        assert_eq!(dev.outcome, MountOutcome::Skipped("EBUSY"));
        // EBUSY means we did NOT record it as a fresh mount.
        assert!(!m.mounted("/dev"));
    }

    #[test]
    fn sandbox_mount_errnos_are_best_effort() {
        // gVisor Sentry already provides /proc and /sys and forbids mounting
        // over them (EPERM); an unprivileged container is denied (EACCES); a
        // devtmpfs the kernel already mounted is EBUSY; an unsupported fstype is
        // ENODEV. All four are skipped, the rest of the table still mounts, and
        // the overall sequence is OK.
        let mut m = RecordingMounter::new()
            .skipping("/proc", 1) // EPERM
            .skipping("/sys", 13) // EACCES
            .skipping("/dev", 16) // EBUSY
            .skipping("/run", 19); // ENODEV
        let table = essential_mounts();
        let results = mount_essential(&mut m, &table);
        assert!(all_ok(&results));
        let reason = |t: &str| {
            results
                .iter()
                .find(|r| r.point.target == t)
                .map(|r| r.outcome.clone())
                .unwrap()
        };
        assert_eq!(reason("/proc"), MountOutcome::Skipped("EPERM"));
        assert_eq!(reason("/sys"), MountOutcome::Skipped("EACCES"));
        assert_eq!(reason("/dev"), MountOutcome::Skipped("EBUSY"));
        assert_eq!(reason("/run"), MountOutcome::Skipped("ENODEV"));
        // None of the skipped targets were recorded as fresh mounts; the
        // non-skipped ones (e.g. /tmp) still were.
        assert!(!m.mounted("/proc"));
        assert!(m.mounted("/tmp"));
    }

    #[test]
    fn unexpected_mount_errno_is_failure() {
        // A genuinely unexpected mount errno (EINVAL: bad flags/fstype) is NOT
        // a benign skip — it must surface as a failure.
        let mut m = RecordingMounter::new().skipping("/proc", 22); // EINVAL
        let table = essential_mounts();
        let results = mount_essential(&mut m, &table);
        assert!(!all_ok(&results));
        let proc = results.iter().find(|r| r.point.target == "/proc").unwrap();
        assert!(matches!(proc.outcome, MountOutcome::Failed(_)));
    }

    #[test]
    fn failed_mount_does_not_abort_sequence() {
        let mut m = RecordingMounter::new().failing("/tmp", "no space");
        let table = essential_mounts();
        let results = mount_essential(&mut m, &table);
        assert!(!all_ok(&results));
        // Everything before /tmp still succeeded.
        let run = results.iter().find(|r| r.point.target == "/run").unwrap();
        assert!(run.outcome.is_ok());
        let tmp = results.iter().find(|r| r.point.target == "/tmp").unwrap();
        assert_eq!(tmp.outcome, MountOutcome::Failed("no space".to_string()));
    }

    #[test]
    fn mkdir_failure_skips_mount() {
        let mut m = RecordingMounter::new();
        m.mkdir_fail_targets.push("/proc".to_string());
        let table = vec![MountPoint::pseudo("proc", "/proc", MountFlags::empty())];
        let results = mount_essential(&mut m, &table);
        assert!(matches!(results[0].outcome, MountOutcome::Failed(_)));
        assert!(!m.mounted("/proc"));
    }

    #[test]
    fn describe_formats_each_outcome() {
        let mp = MountPoint::pseudo("proc", "/proc", MountFlags::empty());
        let ok = MountResult {
            point: mp.clone(),
            outcome: MountOutcome::Mounted,
        };
        assert!(describe(&ok).contains("ok"));
        let busy = MountResult {
            point: mp.clone(),
            outcome: MountOutcome::Skipped("EBUSY"),
        };
        let busy_line = describe(&busy);
        // Matches the required best-effort skip log form.
        assert_eq!(
            busy_line,
            "[seq] mount /proc: skipped (EBUSY, already provided)"
        );
        let failed = MountResult {
            point: mp,
            outcome: MountOutcome::Failed("x".into()),
        };
        assert!(describe(&failed).contains("failed: x"));
    }

    #[test]
    fn with_data_and_source_builders() {
        let mp = MountPoint::pseudo("tmpfs", "/run", MountFlags::empty())
            .with_source("run")
            .with_data("size=10%");
        assert_eq!(mp.source, "run");
        assert_eq!(mp.data.as_deref(), Some("size=10%"));
    }

    #[test]
    fn mount_skip_reason_classifies_sandbox_vs_unexpected() {
        // Best-effort "already provided" errnos are skipped.
        assert_eq!(mount_skip_reason(1), Some("EPERM"));
        assert_eq!(mount_skip_reason(13), Some("EACCES"));
        assert_eq!(mount_skip_reason(16), Some("EBUSY"));
        assert_eq!(mount_skip_reason(19), Some("ENODEV"));
        // Genuinely unexpected errnos propagate as a failure.
        assert_eq!(mount_skip_reason(2), None); // ENOENT
        assert_eq!(mount_skip_reason(22), None); // EINVAL
        assert_eq!(mount_skip_reason(-1), None); // no errno
    }
}
