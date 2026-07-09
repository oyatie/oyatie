//! FRIC-017 runner disk-reclaim preflight (Rust-first; ADR-0548 pipeline-as-product).
//!
//! Invoked as a CI runner step BEFORE the buck-out warm restore — so it must build/run with
//! ZERO dependency on the buck-out cache. Replaces the two duplicated inline `sudo rm -rf …`
//! blocks in `.github/workflows/oya-ci-required.yml`. Reads the data-driven policy
//! (`runner-disk-reclaim-policy.json`), best-effort removes the profile's vendor preinstall
//! dirs, logs structured disk-before/after, and reports the post-reclaim free-disk floor:
//!
//!   * floor met    ⇒ exit 0, "FRIC-017 preflight ok: freed X, free=NgiB"
//!   * floor missed (default / fail-closed)       ⇒ exit 3, "FRIC-017 infra-red: …"
//!   * floor missed (fail-open-with-waiver)       ⇒ exit 0 only with typed waiver fields and an
//!     operator artifact recording the waiver
//!   * usage error  ⇒ exit 2 (bad args / missing policy / malformed profile)
//!
//! LOCAL BRIDGE invocation per the founder cli_surface_policy: merge authority lives in the
//! conformance gate test, never this binary; its successor is a reconciler (ADR-0548 D3).

use ci_runner_disk_reclaim::{
    DirOutcome, DiskOps, GIB, InfraRedPolicy, InfraRedWaiver, POLICY_REL_PATH, ReclaimReport,
    parse_profile, run_reclaim, runner_disk_reclaim_operator_artifact,
    validate_infra_red_exit_contract,
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
                "usage: oya-cloud-ci-runner-disk-reclaim --profile <id> [--policy <path>] [--root <path>] [--artifact-out <path>] [--infra-red-policy fail-closed|fail-open-with-waiver] [--infra-red-waiver-id <id> --infra-red-waiver-reason <reason>]"
            );
            ExitCode::from(EXIT_USAGE)
        }
    }
}

fn run(args: Vec<String>) -> Result<ExitCode, String> {
    let mut profile_id: Option<String> = None;
    let mut policy_path: PathBuf = PathBuf::from(DEFAULT_POLICY);
    let mut root: PathBuf = PathBuf::from("/");
    let mut infra_red_policy = InfraRedPolicy::FailClosed;
    let mut waiver_id: Option<String> = None;
    let mut waiver_reason: Option<String> = None;
    let mut artifact_out: Option<PathBuf> = None;

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
            "--strict" => {
                infra_red_policy = InfraRedPolicy::FailClosed;
            }
            "--artifact-out" => {
                artifact_out = Some(PathBuf::from(
                    it.next().ok_or("--artifact-out requires a value")?,
                ));
            }
            "--infra-red-policy" => {
                infra_red_policy = match it
                    .next()
                    .ok_or("--infra-red-policy requires a value")?
                    .as_str()
                {
                    "fail-closed" => InfraRedPolicy::FailClosed,
                    "fail-open-with-waiver" => InfraRedPolicy::FailOpenWithWaiver,
                    other => {
                        return Err(format!(
                            "--infra-red-policy must be fail-closed|fail-open-with-waiver, got `{other}`"
                        ));
                    }
                };
            }
            "--infra-red-waiver-id" => {
                waiver_id = Some(it.next().ok_or("--infra-red-waiver-id requires a value")?);
            }
            "--infra-red-waiver-reason" => {
                waiver_reason = Some(
                    it.next()
                        .ok_or("--infra-red-waiver-reason requires a value")?,
                );
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }

    let profile_id = profile_id.ok_or("--profile <id> is required")?;
    let policy_text = std::fs::read_to_string(&policy_path)
        .map_err(|e| format!("read policy {}: {e}", policy_path.display()))?;
    let profile = parse_profile(&policy_text, &profile_id).map_err(|e| e.to_string())?;

    let report = run_reclaim(&RealDiskOps, &root, &profile)
        .map_err(|e| format!("free-disk stat failed (fail-loud): {e}"))?;

    let waiver = match (waiver_id, waiver_reason) {
        (Some(id), Some(reason)) => Some(InfraRedWaiver::new(id, reason)?),
        (None, None) => None,
        _ => {
            return Err(
                "--infra-red-waiver-id and --infra-red-waiver-reason must be supplied together"
                    .to_owned(),
            );
        }
    };

    validate_infra_red_exit_contract(
        &report,
        infra_red_policy,
        waiver.as_ref(),
        artifact_out.is_some(),
    )?;

    let artifact = runner_disk_reclaim_operator_artifact(
        &profile_id,
        &report,
        infra_red_policy,
        waiver.as_ref(),
    )?;
    if let Some(path) = artifact_out {
        let bytes = serde_json::to_vec_pretty(&artifact)
            .map_err(|e| format!("serialize operator artifact: {e}"))?;
        std::fs::write(&path, bytes)
            .map_err(|e| format!("write operator artifact {}: {e}", path.display()))?;
        println!("FRIC-017 operator-artifact: {}", path.display());
    }

    Ok(emit(
        &profile_id,
        &report,
        infra_red_policy,
        waiver.as_ref(),
    ))
}

/// Emit the structured report + return the exit code.
///
/// Default and workflow policy is fail-closed: threshold-miss exits `EXIT_INFRA_RED` (3) so the
/// required context cannot silently green. A temporary fail-open is allowed only when the caller
/// selects `fail-open-with-waiver` and supplies a typed waiver; the machine-readable artifact records
/// that waiver.
fn emit(
    profile_id: &str,
    report: &ReclaimReport,
    infra_red_policy: InfraRedPolicy,
    waiver: Option<&InfraRedWaiver>,
) -> ExitCode {
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
            DirOutcome::Rejected(reason) => format!("REJECTED (safety guard): {reason}"),
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
             a downstream disk-exhaustion here is INFRA, not CODE; policy={})",
            report.free_gib_after(),
            report.min_free_gib_after,
            infra_red_policy.as_str(),
        );
        match infra_red_policy {
            InfraRedPolicy::FailClosed => ExitCode::from(EXIT_INFRA_RED),
            InfraRedPolicy::FailOpenWithWaiver => {
                if let Some(waiver) = waiver {
                    println!(
                        "FRIC-017 infra-red waiver: id={} reason={}",
                        waiver.waiver_id, waiver.reason
                    );
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(EXIT_USAGE)
                }
            }
        }
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
