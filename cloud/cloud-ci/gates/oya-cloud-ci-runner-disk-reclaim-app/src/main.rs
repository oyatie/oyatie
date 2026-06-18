//! FRIC-017 runner disk-reclaim preflight (Rust-first; ADR-0548 pipeline-as-product).
//!
//! Invoked as a CI runner step BEFORE the buck-out warm restore — so it must build/run with
//! ZERO dependency on the buck-out cache. Replaces the two duplicated inline `sudo rm -rf …`
//! blocks in `.github/workflows/oya-ci-required.yml`. Reads the data-driven policy
//! (`runner-disk-reclaim-policy.json`), best-effort removes the profile's vendor preinstall
//! dirs, logs structured disk-before/after, and asserts the post-reclaim free-disk floor:
//!
//!   * floor met    ⇒ exit 0, "FRIC-017 preflight ok: freed X, free=NgiB"
//!   * floor missed ⇒ exit 3 (INFRA-RED), "FRIC-017 infra-red: free=NgiB < min=MgiB"
//!   * usage error  ⇒ exit 2 (bad args / missing policy / malformed profile)
//!
//! LOCAL BRIDGE invocation per the founder cli_surface_policy: merge authority lives in the
//! conformance gate test, never this binary; its successor is a reconciler (ADR-0548 D3).

use oya_cloud_ci_runner_disk_reclaim_app::{
    DirOutcome, DiskOps, GIB, POLICY_REL_PATH, ReclaimReport, parse_profile, run_reclaim,
};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Default policy path, resolved relative to the repo root (the runner's working dir). Single
/// source of truth shared with the gate test via the lib constant.
const DEFAULT_POLICY: &str = POLICY_REL_PATH;

/// Exit codes (distinct so a downstream consumer can attribute a failure).
const EXIT_USAGE: u8 = 2;
const EXIT_INFRA_RED: u8 = 3;

/// Real filesystem effects: `statvfs(2)` for free bytes, `std::fs::remove_dir_all` for reclaim.
struct RealDiskOps;

impl DiskOps for RealDiskOps {
    fn free_bytes(&self, path: &Path) -> std::io::Result<u64> {
        free_bytes_statvfs(path)
    }
    fn remove_dir_all(&self, dir: &Path) -> std::io::Result<()> {
        std::fs::remove_dir_all(dir)
    }
}

/// Free bytes on the filesystem containing `path`, via `statvfs(2)`. Available-to-unprivileged
/// blocks (`f_bavail`) × fragment size (`f_frsize`) — the same number `df` reports for "Avail".
fn free_bytes_statvfs(path: &Path) -> std::io::Result<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c_path = std::ffi::CString::new(path.as_os_str().as_bytes())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    // SAFETY: `stat` is a valid, zeroed `statvfs` out-param; `c_path` is a NUL-terminated C string
    // that outlives the call. `statvfs` only writes `stat` and reads `c_path`.
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let frsize = stat.f_frsize as u64;
    let bavail = stat.f_bavail as u64;
    Ok(frsize.saturating_mul(bavail))
}

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(code) => code,
        Err(usage) => {
            eprintln!("FRIC-017 preflight usage error: {usage}");
            eprintln!(
                "usage: oya-cloud-ci-runner-disk-reclaim --profile <id> [--policy <path>] [--root <path>]"
            );
            ExitCode::from(EXIT_USAGE)
        }
    }
}

fn run(args: Vec<String>) -> Result<ExitCode, String> {
    let mut profile_id: Option<String> = None;
    let mut policy_path: PathBuf = PathBuf::from(DEFAULT_POLICY);
    let mut root: PathBuf = PathBuf::from("/");

    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--profile" => {
                profile_id = Some(it.next().ok_or("--profile requires a value")?);
            }
            "--policy" => {
                policy_path = PathBuf::from(it.next().ok_or("--policy requires a value")?);
            }
            "--root" => {
                root = PathBuf::from(it.next().ok_or("--root requires a value")?);
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }

    let profile_id = profile_id.ok_or("--profile <id> is required")?;
    let policy_text = std::fs::read_to_string(&policy_path)
        .map_err(|e| format!("read policy {}: {e}", policy_path.display()))?;
    let profile =
        parse_profile(&policy_text, &profile_id).map_err(|e| e.to_string())?;

    let report = run_reclaim(&RealDiskOps, &root, &profile)
        .map_err(|e| format!("free-disk stat failed (fail-loud): {e}"))?;

    Ok(emit(&profile_id, &report))
}

/// Emit the structured report + return the exit code (0 ok, EXIT_INFRA_RED below floor).
fn emit(profile_id: &str, report: &ReclaimReport) -> ExitCode {
    println!(
        "FRIC-017 disk-before: free={} bytes ({} GiB) profile={profile_id}",
        report.free_before,
        report.free_before / GIB
    );
    for (dir, outcome) in &report.outcomes {
        let label = match outcome {
            DirOutcome::Removed => "removed".to_owned(),
            DirOutcome::Absent => "absent".to_owned(),
            DirOutcome::Failed(e) => format!("failed: {e}"),
        };
        println!("FRIC-017 reclaim-dir: {dir} -> {label}");
    }
    println!(
        "FRIC-017 disk-after: free={} bytes ({} GiB)",
        report.free_after,
        report.free_gib_after()
    );

    if report.is_infra_red() {
        println!(
            "FRIC-017 infra-red: free={}giB < min={}giB (post-reclaim runner capacity insufficient; \
             a downstream disk-exhaustion here is INFRA, not CODE)",
            report.free_gib_after(),
            report.min_free_gib_after
        );
        ExitCode::from(EXIT_INFRA_RED)
    } else {
        println!(
            "FRIC-017 preflight ok: freed {} bytes ({} GiB), free={}giB (>= min={}giB)",
            report.freed_bytes(),
            report.freed_bytes() / GIB,
            report.free_gib_after(),
            report.min_free_gib_after
        );
        ExitCode::SUCCESS
    }
}
