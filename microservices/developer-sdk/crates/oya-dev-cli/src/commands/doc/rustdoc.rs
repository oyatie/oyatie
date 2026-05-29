use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant, SystemTime};

use crate::command_output::{OutputFormat as DevCheckOutputFormat, json_escape};
use crate::command_process::{process_status_label, replay_process_output};

pub(super) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    match parse_doc_rustdoc_args(args, usage) {
        Ok(args) => run_doc_rustdoc(args),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DocRustdocArgs {
    target_dir: PathBuf,
    rustdoc_path: Option<PathBuf>,
    cargo_path: PathBuf,
    output_format: DevCheckOutputFormat,
    clean_target_dir: bool,
}

fn parse_doc_rustdoc_args(args: Vec<String>, usage: &str) -> Result<DocRustdocArgs, String> {
    let mut parsed = DocRustdocArgs {
        target_dir: PathBuf::from("target/oya-rustdoc-check"),
        rustdoc_path: None,
        cargo_path: PathBuf::from("cargo"),
        output_format: DevCheckOutputFormat::Text,
        clean_target_dir: true,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--target-dir" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.target_dir = PathBuf::from(value);
            }
            "--rustdoc" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.rustdoc_path = Some(PathBuf::from(value));
            }
            "--cargo" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.cargo_path = PathBuf::from(value);
            }
            "--format" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.output_format =
                    DevCheckOutputFormat::parse(&value).ok_or_else(|| usage.to_owned())?;
            }
            "--keep-target-dir" => parsed.clean_target_dir = false,
            _ => return Err(usage.to_owned()),
        }
    }
    Ok(parsed)
}

fn run_doc_rustdoc(args: DocRustdocArgs) -> ExitCode {
    let rustdoc_path = match resolve_doc_rustdoc_path(args.rustdoc_path.as_deref()) {
        Ok(path) => path,
        Err(message) => {
            eprintln!("rustdoc generation failed: {message}");
            return ExitCode::FAILURE;
        }
    };

    let _target_lock = match RustdocTargetLock::acquire(&args.target_dir) {
        Ok(lock) => lock,
        Err(message) => {
            eprintln!("rustdoc generation failed: {message}");
            return ExitCode::FAILURE;
        }
    };

    if args.clean_target_dir
        && let Err(message) = clean_rustdoc_target_dir(&args.target_dir)
    {
        eprintln!("rustdoc generation failed: {message}");
        return ExitCode::FAILURE;
    }

    let output = match std::process::Command::new(&args.cargo_path)
        .args(["doc", "--workspace", "--no-deps", "--all-features"])
        .env("CARGO_TARGET_DIR", &args.target_dir)
        .env("RUSTDOC", &rustdoc_path)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            eprintln!(
                "rustdoc generation failed: could not run {}: {error}",
                args.cargo_path.display()
            );
            return ExitCode::FAILURE;
        }
    };

    match args.output_format {
        DevCheckOutputFormat::Text => render_doc_rustdoc_text(&args, &rustdoc_path, &output),
        DevCheckOutputFormat::Json => render_doc_rustdoc_json(&args, &rustdoc_path, &output),
    }
}

#[derive(Debug)]
struct RustdocTargetLock {
    path: PathBuf,
}

impl RustdocTargetLock {
    // Grounded concurrency guard for the generated rustdoc scratch target:
    // `scripts/check.sh`, pre-push checks, and manual documentation checks can
    // run in parallel in local agent sessions. The lock protects the clean +
    // cargo-doc critical section, and PID ownership lets a later run reclaim a
    // lock left behind by an interrupted local process instead of deleting an
    // active target directory.
    const OWNER_FILE: &'static str = "owner";
    const OWNERLESS_STALE_AFTER: Duration = Duration::from_secs(30);
    const RETRY_INTERVAL: Duration = Duration::from_millis(200);
    const TIMEOUT: Duration = Duration::from_secs(600);

    fn acquire(target_dir: &Path) -> Result<Self, String> {
        let lock_path = rustdoc_target_lock_path(target_dir)?;
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "could not create rustdoc target lock parent {}: {error}",
                    parent.display()
                )
            })?;
        }

        let started = Instant::now();
        loop {
            match fs::create_dir(&lock_path) {
                Ok(()) => {
                    if let Err(error) = write_rustdoc_target_lock_owner(&lock_path) {
                        let _ = fs::remove_dir_all(&lock_path);
                        return Err(error);
                    }
                    return Ok(Self { path: lock_path });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    if reclaim_stale_rustdoc_target_lock(&lock_path)? {
                        continue;
                    }
                    if started.elapsed() >= Self::TIMEOUT {
                        return Err(format!(
                            "timed out waiting for rustdoc target lock {}; another `oya doc rustdoc` may still be running",
                            lock_path.display()
                        ));
                    }
                    std::thread::sleep(Self::RETRY_INTERVAL);
                }
                Err(error) => {
                    return Err(format!(
                        "could not acquire rustdoc target lock {}: {error}",
                        lock_path.display()
                    ));
                }
            }
        }
    }
}

impl Drop for RustdocTargetLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_rustdoc_target_lock_owner(lock_path: &Path) -> Result<(), String> {
    fs::write(
        lock_path.join(RustdocTargetLock::OWNER_FILE),
        format!("pid={}\n", std::process::id()),
    )
    .map_err(|error| {
        format!(
            "could not write rustdoc target lock owner {}: {error}",
            lock_path.join(RustdocTargetLock::OWNER_FILE).display()
        )
    })
}

fn reclaim_stale_rustdoc_target_lock(lock_path: &Path) -> Result<bool, String> {
    if let Some(pid) = rustdoc_target_lock_owner_pid(lock_path) {
        if process_is_running(pid) {
            return Ok(false);
        }
        remove_stale_rustdoc_target_lock(lock_path)?;
        return Ok(true);
    }

    if ownerless_rustdoc_target_lock_is_stale(lock_path) {
        remove_stale_rustdoc_target_lock(lock_path)?;
        return Ok(true);
    }

    Ok(false)
}

fn rustdoc_target_lock_owner_pid(lock_path: &Path) -> Option<u32> {
    let owner = fs::read_to_string(lock_path.join(RustdocTargetLock::OWNER_FILE)).ok()?;
    owner.lines().find_map(|line| {
        line.strip_prefix("pid=")
            .and_then(|value| value.trim().parse::<u32>().ok())
    })
}

fn process_is_running(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|status| status.success())
        .unwrap_or(true)
}

fn ownerless_rustdoc_target_lock_is_stale(lock_path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(lock_path) else {
        return true;
    };
    let Ok(modified) = metadata.modified() else {
        return false;
    };
    SystemTime::now()
        .duration_since(modified)
        .is_ok_and(|age| age >= RustdocTargetLock::OWNERLESS_STALE_AFTER)
}

fn remove_stale_rustdoc_target_lock(lock_path: &Path) -> Result<(), String> {
    match fs::remove_dir_all(lock_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "could not remove stale rustdoc target lock {}: {error}",
            lock_path.display()
        )),
    }
}

fn rustdoc_target_lock_path(target_dir: &Path) -> Result<PathBuf, String> {
    let parent = target_dir.parent().ok_or_else(|| {
        format!(
            "rustdoc target dir {} has no parent for lock placement",
            target_dir.display()
        )
    })?;
    let file_name = target_dir.file_name().ok_or_else(|| {
        format!(
            "rustdoc target dir {} has no final path component for lock placement",
            target_dir.display()
        )
    })?;
    Ok(parent.join(format!("{}.lock", file_name.to_string_lossy())))
}

fn resolve_doc_rustdoc_path(explicit_path: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = explicit_path {
        return Ok(path.to_path_buf());
    }
    let output = std::process::Command::new("rustup")
        .args(["which", "rustdoc"])
        .output()
        .map_err(|error| format!("could not resolve rustup rustdoc: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "rustup which rustdoc exited with {}: {}",
            process_status_label(&output.status),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        Err("rustup which rustdoc returned an empty path".into())
    } else {
        Ok(PathBuf::from(path))
    }
}

fn clean_rustdoc_target_dir(target_dir: &Path) -> Result<(), String> {
    if !is_safe_rustdoc_target_dir(target_dir) {
        return Err(format!(
            "refusing to clean non-generated rustdoc target dir {}; use target/oya-rustdoc-check or pass --keep-target-dir",
            target_dir.display()
        ));
    }
    if target_dir.exists() {
        fs::remove_dir_all(target_dir).map_err(|error| {
            format!(
                "could not clean rustdoc target dir {}: {error}",
                target_dir.display()
            )
        })?;
    }
    Ok(())
}

fn is_safe_rustdoc_target_dir(target_dir: &Path) -> bool {
    if target_dir
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return false;
    }
    target_dir.file_name().and_then(|name| name.to_str()) == Some("oya-rustdoc-check")
        && target_dir
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            == Some("target")
}

fn render_doc_rustdoc_text(
    args: &DocRustdocArgs,
    rustdoc_path: &Path,
    output: &std::process::Output,
) -> ExitCode {
    if let Err(error) = replay_process_output(output) {
        eprintln!("rustdoc generation failed: {error}");
        return ExitCode::FAILURE;
    }
    if output.status.success() {
        println!(
            "rustdoc generation passed: target_dir={}, rustdoc={}",
            args.target_dir.display(),
            rustdoc_path.display()
        );
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "rustdoc generation failed: cargo doc exited with {}",
            process_status_label(&output.status)
        );
        ExitCode::FAILURE
    }
}

fn render_doc_rustdoc_json(
    args: &DocRustdocArgs,
    rustdoc_path: &Path,
    output: &std::process::Output,
) -> ExitCode {
    let status = if output.status.success() {
        "passed"
    } else {
        "failed"
    };
    println!(
        "{{\"command\":\"oya doc rustdoc\",\"target_dir\":\"{}\",\"rustdoc\":\"{}\",\"status\":\"{}\",\"exit_code\":{},\"stdout\":\"{}\",\"stderr\":\"{}\"}}",
        json_escape(&args.target_dir.display().to_string()),
        json_escape(&rustdoc_path.display().to_string()),
        status,
        output.status.code().unwrap_or(-1),
        json_escape(&String::from_utf8_lossy(&output.stdout)),
        json_escape(&String::from_utf8_lossy(&output.stderr)),
    );
    if output.status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
