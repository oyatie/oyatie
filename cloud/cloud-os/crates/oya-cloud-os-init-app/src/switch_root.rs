//! `switch_root` — pivot from the initramfs to the real root and hand off.
//!
//! Talos boots in two stages. PID 1 (`cmd/init`) runs in the initramfs, mounts
//! the essential filesystems, and then *switches root* to the freshly-mounted
//! real rootfs and `exec`s the long-running `machined` as the new PID 1. The
//! switch-root dance, mirroring `util-linux`'s `switch_root`, is:
//!
//! 1. Verify the new root is a mount point and the target init exists.
//! 2. Recursively delete the old initramfs contents to free the tmpfs RAM
//!    (only files on the same device as `/` — never cross a mount boundary).
//! 3. `mount --move` the new root onto `/`.
//! 4. `chroot .` and `chdir /`.
//! 5. `execve` the new init, replacing PID 1.
//!
//! Every kernel-touching step is modeled behind the [`RootFs`] trait so the
//! whole plan is validated on the host with [`FakeRootFs`].

/// Errors that can abort a switch_root. These mirror the fatal conditions
/// `util-linux` checks before it is willing to delete the old root.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SwitchRootError {
    /// The new-root path is not itself a mount point.
    NewRootNotMounted(String),
    /// The target init binary does not exist under the new root.
    InitMissing(String),
    /// `mount --move` of new root onto `/` failed.
    MoveMountFailed(String),
    /// `chroot`/`chdir` into the new root failed.
    ChrootFailed(String),
    /// `execve` of the new init failed (this is the point of no return — if it
    /// returns at all, it failed).
    ExecFailed(String),
}

impl std::fmt::Display for SwitchRootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwitchRootError::NewRootNotMounted(p) => {
                write!(f, "new root {p} is not a mount point")
            }
            SwitchRootError::InitMissing(p) => write!(f, "init {p} missing in new root"),
            SwitchRootError::MoveMountFailed(e) => write!(f, "mount --move failed: {e}"),
            SwitchRootError::ChrootFailed(e) => write!(f, "chroot failed: {e}"),
            SwitchRootError::ExecFailed(e) => write!(f, "execve failed: {e}"),
        }
    }
}

impl std::error::Error for SwitchRootError {}

/// A planned switch_root operation.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SwitchRootPlan {
    /// Path to the new root, e.g. `/root` after the real rootfs is mounted.
    pub new_root: String,
    /// Init to exec relative to the new root, e.g. `/sbin/machined`.
    pub init: String,
    /// Argv to pass to the new init (argv[0] is conventionally the init path).
    pub argv: Vec<String>,
    /// Whether to wipe the old initramfs to reclaim RAM (Talos: yes).
    pub cleanup_old_root: bool,
}

impl SwitchRootPlan {
    /// The standard Talos plan: pivot to `new_root` and exec `machined`.
    pub fn to_machined(new_root: &str) -> Self {
        SwitchRootPlan {
            new_root: new_root.to_string(),
            init: "/sbin/machined".to_string(),
            argv: vec!["/sbin/machined".to_string()],
            cleanup_old_root: true,
        }
    }

    /// Absolute path the init resolves to once `new_root` becomes `/`. We join
    /// without crossing `..` and collapse a doubled slash.
    pub fn init_in_new_root(&self) -> String {
        let root = self.new_root.trim_end_matches('/');
        let init = if self.init.starts_with('/') {
            self.init.clone()
        } else {
            format!("/{}", self.init)
        };
        format!("{root}{init}")
    }
}

/// Abstraction over the root-pivoting syscalls.
pub trait RootFs {
    /// Is `path` a mount point (different device from its parent)?
    fn is_mount_point(&self, path: &str) -> bool;
    /// Does a file exist at `path`?
    fn exists(&self, path: &str) -> bool;
    /// Recursively delete everything under `path` that lives on the same device
    /// (never crossing into a sub-mount). Returns the count removed.
    fn recursive_delete_same_fs(&mut self, path: &str) -> Result<usize, String>;
    /// `mount --move src` onto `dst`.
    fn move_mount(&mut self, src: &str, dst: &str) -> Result<(), String>;
    /// `chroot(new_root)` followed by `chdir("/")`.
    fn chroot(&mut self, new_root: &str) -> Result<(), String>;
    /// `execve(init, argv)`. On success this never returns; modeled as `Ok(())`
    /// recording the exec for tests, or `Err` to model a failed exec.
    fn exec(&mut self, init: &str, argv: &[String]) -> Result<(), String>;
}

/// Validate a plan without performing it. Run before any destructive step so we
/// never wipe the old root unless the new one is sound.
pub fn validate(plan: &SwitchRootPlan, fs: &dyn RootFs) -> Result<(), SwitchRootError> {
    if !fs.is_mount_point(&plan.new_root) {
        return Err(SwitchRootError::NewRootNotMounted(plan.new_root.clone()));
    }
    let init_path = plan.init_in_new_root();
    if !fs.exists(&init_path) {
        return Err(SwitchRootError::InitMissing(init_path));
    }
    Ok(())
}

/// Result of a (modeled) successful switch_root, capturing what happened for
/// inspection. On a real kernel this is never constructed because `exec`
/// replaces the process; the fake returns it so tests can assert.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SwitchRootDone {
    pub files_removed: usize,
    pub execed: String,
    pub argv: Vec<String>,
}

/// Perform the full switch_root: validate, (optionally) wipe the old root,
/// move-mount, chroot, and exec. On a real kernel a success never returns; the
/// `RootFs::exec` impl is what does (or doesn't) come back.
pub fn switch_root(
    plan: &SwitchRootPlan,
    fs: &mut dyn RootFs,
) -> Result<SwitchRootDone, SwitchRootError> {
    validate(plan, fs)?;

    let mut files_removed = 0;
    if plan.cleanup_old_root {
        // Wipe old initramfs (everything on "/", same device only).
        files_removed = fs
            .recursive_delete_same_fs("/")
            .map_err(|e| SwitchRootError::MoveMountFailed(format!("cleanup: {e}")))?;
    }

    fs.move_mount(&plan.new_root, "/")
        .map_err(SwitchRootError::MoveMountFailed)?;

    fs.chroot(&plan.new_root)
        .map_err(SwitchRootError::ChrootFailed)?;

    fs.exec(&plan.init, &plan.argv)
        .map_err(SwitchRootError::ExecFailed)?;

    Ok(SwitchRootDone {
        files_removed,
        execed: plan.init.clone(),
        argv: plan.argv.clone(),
    })
}

/// In-memory [`RootFs`] for tests. Models a set of mount points and existing
/// files; records the move/chroot/exec it would perform.
#[derive(Default)]
pub struct FakeRootFs {
    pub mount_points: Vec<String>,
    pub files: Vec<String>,
    pub old_root_file_count: usize,
    pub moved: Option<(String, String)>,
    pub chrooted: Option<String>,
    pub execed: Option<(String, Vec<String>)>,
    /// If set, `exec` fails with this message (modeling a bad init binary).
    pub exec_error: Option<String>,
    /// If set, `move_mount` fails.
    pub move_error: Option<String>,
}

impl FakeRootFs {
    /// A healthy environment where `new_root` is mounted and contains `init`.
    pub fn healthy(new_root: &str, init_in_root: &str, old_files: usize) -> Self {
        FakeRootFs {
            mount_points: vec![new_root.to_string()],
            files: vec![init_in_root.to_string()],
            old_root_file_count: old_files,
            ..Default::default()
        }
    }
}

impl RootFs for FakeRootFs {
    fn is_mount_point(&self, path: &str) -> bool {
        self.mount_points.iter().any(|m| m == path)
    }

    fn exists(&self, path: &str) -> bool {
        self.files.iter().any(|f| f == path)
    }

    fn recursive_delete_same_fs(&mut self, _path: &str) -> Result<usize, String> {
        let n = self.old_root_file_count;
        self.old_root_file_count = 0;
        Ok(n)
    }

    fn move_mount(&mut self, src: &str, dst: &str) -> Result<(), String> {
        if let Some(e) = &self.move_error {
            return Err(e.clone());
        }
        self.moved = Some((src.to_string(), dst.to_string()));
        Ok(())
    }

    fn chroot(&mut self, new_root: &str) -> Result<(), String> {
        self.chrooted = Some(new_root.to_string());
        Ok(())
    }

    fn exec(&mut self, init: &str, argv: &[String]) -> Result<(), String> {
        if let Some(e) = &self.exec_error {
            return Err(e.clone());
        }
        self.execed = Some((init.to_string(), argv.to_vec()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machined_plan_defaults() {
        let plan = SwitchRootPlan::to_machined("/root");
        assert_eq!(plan.new_root, "/root");
        assert_eq!(plan.init, "/sbin/machined");
        assert_eq!(plan.argv, vec!["/sbin/machined".to_string()]);
        assert!(plan.cleanup_old_root);
    }

    #[test]
    fn init_in_new_root_joins_paths() {
        let plan = SwitchRootPlan::to_machined("/root");
        assert_eq!(plan.init_in_new_root(), "/root/sbin/machined");
        let plan2 = SwitchRootPlan::to_machined("/root/");
        assert_eq!(plan2.init_in_new_root(), "/root/sbin/machined");
    }

    #[test]
    fn validate_ok_for_healthy_fs() {
        let plan = SwitchRootPlan::to_machined("/root");
        let fs = FakeRootFs::healthy("/root", "/root/sbin/machined", 100);
        assert!(validate(&plan, &fs).is_ok());
    }

    #[test]
    fn validate_rejects_unmounted_new_root() {
        let plan = SwitchRootPlan::to_machined("/root");
        let fs = FakeRootFs {
            mount_points: vec![],
            files: vec!["/root/sbin/machined".to_string()],
            ..Default::default()
        };
        assert_eq!(
            validate(&plan, &fs),
            Err(SwitchRootError::NewRootNotMounted("/root".to_string()))
        );
    }

    #[test]
    fn validate_rejects_missing_init() {
        let plan = SwitchRootPlan::to_machined("/root");
        let fs = FakeRootFs {
            mount_points: vec!["/root".to_string()],
            files: vec![],
            ..Default::default()
        };
        assert_eq!(
            validate(&plan, &fs),
            Err(SwitchRootError::InitMissing(
                "/root/sbin/machined".to_string()
            ))
        );
    }

    #[test]
    fn full_switch_root_happy_path() {
        let plan = SwitchRootPlan::to_machined("/root");
        let mut fs = FakeRootFs::healthy("/root", "/root/sbin/machined", 250);
        let done = switch_root(&plan, &mut fs).unwrap();
        assert_eq!(done.files_removed, 250);
        assert_eq!(done.execed, "/sbin/machined");
        assert_eq!(fs.moved, Some(("/root".to_string(), "/".to_string())));
        assert_eq!(fs.chrooted, Some("/root".to_string()));
        assert_eq!(
            fs.execed,
            Some((
                "/sbin/machined".to_string(),
                vec!["/sbin/machined".to_string()]
            ))
        );
    }

    #[test]
    fn no_cleanup_skips_delete() {
        let mut plan = SwitchRootPlan::to_machined("/root");
        plan.cleanup_old_root = false;
        let mut fs = FakeRootFs::healthy("/root", "/root/sbin/machined", 999);
        let done = switch_root(&plan, &mut fs).unwrap();
        assert_eq!(done.files_removed, 0);
        // Old root untouched.
        assert_eq!(fs.old_root_file_count, 999);
    }

    #[test]
    fn validation_runs_before_destructive_cleanup() {
        // Missing init must abort BEFORE we wipe the old root.
        let plan = SwitchRootPlan::to_machined("/root");
        let mut fs = FakeRootFs {
            mount_points: vec!["/root".to_string()],
            files: vec![],
            old_root_file_count: 500,
            ..Default::default()
        };
        let err = switch_root(&plan, &mut fs).unwrap_err();
        assert!(matches!(err, SwitchRootError::InitMissing(_)));
        // Old root NOT wiped.
        assert_eq!(fs.old_root_file_count, 500);
        assert!(fs.moved.is_none());
    }

    #[test]
    fn failed_exec_is_reported() {
        let plan = SwitchRootPlan::to_machined("/root");
        let mut fs = FakeRootFs::healthy("/root", "/root/sbin/machined", 10);
        fs.exec_error = Some("ENOENT".to_string());
        let err = switch_root(&plan, &mut fs).unwrap_err();
        assert_eq!(err, SwitchRootError::ExecFailed("ENOENT".to_string()));
    }

    #[test]
    fn failed_move_mount_is_reported() {
        let plan = SwitchRootPlan::to_machined("/root");
        let mut fs = FakeRootFs::healthy("/root", "/root/sbin/machined", 10);
        fs.move_error = Some("EINVAL".to_string());
        let err = switch_root(&plan, &mut fs).unwrap_err();
        assert_eq!(err, SwitchRootError::MoveMountFailed("EINVAL".to_string()));
    }

    #[test]
    fn error_display_messages() {
        assert!(
            SwitchRootError::InitMissing("/x".into())
                .to_string()
                .contains("/x")
        );
        assert!(
            SwitchRootError::ExecFailed("boom".into())
                .to_string()
                .contains("boom")
        );
    }
}
