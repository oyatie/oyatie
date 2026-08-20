//! Cargo test-resource adapter for inventory-backed gate integration tests.
//!
//! The admission workflow materializes SCM facts and binds the accounting producer before it runs
//! the workspace. A direct `cargo test --workspace` has neither resource. Cargo also does not
//! propagate another package's `CARGO_BIN_EXE_*` value, so repository Cargo config points the gate
//! tests at this adapter. If their declared SCM snapshot is absent, the adapter invokes the exact
//! Rust SCM emitter into an isolated temporary directory, substitutes that path, and delegates to
//! the exact accounting producer. Existing or malformed declared inputs are never replaced.
//!
//! This binary is test plumbing, not admission authority and not an operator CLI. It performs no
//! evaluation and invents no success: missing, ambiguous, non-regular, failed, or signalled tools
//! remain hard failures.

#![forbid(unsafe_code)]

use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const PRODUCER_ENV: &str = "OYA_CI_CARGO_TEST_PRODUCER_BIN";
const EMITTER_ENV: &str = "OYA_CI_CARGO_TEST_SCM_FACTS_EMITTER_BIN";
const REPO_ROOT_FLAG: &str = "--repo-root";
const SCM_FACTS_FLAG: &str = "--scm-facts";
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn main() -> ExitCode {
    match run(std::env::args_os().skip(1).collect()) {
        Ok(status) => exit_code(status),
        Err(error) => {
            eprintln!("oya-cloud-ci-cargo-test-producer-adapter: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(mut args: Vec<OsString>) -> Result<ExitStatus, String> {
    let repo_root = required_flag_path(&args, REPO_ROOT_FLAG)?;
    let scm_facts_index = required_flag_value_index(&args, SCM_FACTS_FLAG)?;
    let declared_scm_facts = PathBuf::from(&args[scm_facts_index]);
    let producer = required_binary(&repo_root, PRODUCER_ENV)?;

    let temporary_inputs = if declared_scm_facts.exists() {
        None
    } else {
        let inputs = TemporaryInputs::create()?;
        let emitter = required_binary(&repo_root, EMITTER_ENV)?;
        emit_scm_facts(&emitter, &repo_root, &inputs)?;
        require_regular_file(&inputs.stable, "materialized SCM facts")?;
        args[scm_facts_index] = inputs.stable.as_os_str().to_owned();
        Some(inputs)
    };

    let status = Command::new(&producer)
        .args(&args)
        .status()
        .map_err(|error| format!("run accounting producer {}: {error}", producer.display()))?;

    // Keep the temporary inputs alive until the delegated producer has exited.
    drop(temporary_inputs);
    Ok(status)
}

fn required_flag_path(args: &[OsString], flag: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(
        args.get(required_flag_value_index(args, flag)?)
            .expect("validated flag value index"),
    ))
}

fn required_flag_value_index(args: &[OsString], flag: &str) -> Result<usize, String> {
    let mut matches = args
        .iter()
        .enumerate()
        .filter(|(_, arg)| arg.as_os_str() == OsStr::new(flag));
    let Some((index, _)) = matches.next() else {
        return Err(format!("missing required {flag} argument"));
    };
    if matches.next().is_some() {
        return Err(format!("ambiguous repeated {flag} argument"));
    }
    let value_index = index + 1;
    let Some(value) = args.get(value_index) else {
        return Err(format!("{flag} requires a value"));
    };
    if value.is_empty() {
        return Err(format!("{flag} requires a non-empty value"));
    }
    Ok(value_index)
}

fn required_binary(repo_root: &Path, variable: &str) -> Result<PathBuf, String> {
    let value = std::env::var_os(variable)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("missing required {variable}"))?;
    let declared = PathBuf::from(&value);
    if !declared.is_absolute()
        && !value.to_str().is_some_and(|value| {
            value.starts_with(ci_path_resolver_adapters::CARGO_TEST_BINARY_PREFIX)
        })
    {
        return Err(format!(
            "{variable} must bind an absolute path or {}<name>, got {}",
            ci_path_resolver_adapters::CARGO_TEST_BINARY_PREFIX,
            declared.display()
        ));
    }
    let path = ci_path_resolver_adapters::resolve_cargo_test_binary(repo_root, &value)?;
    require_regular_file(&path, variable)?;
    Ok(path)
}

fn require_regular_file(path: &Path, label: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("{label} {} is unavailable: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "{label} {} is not a regular non-symlink file",
            path.display()
        ));
    }
    Ok(())
}

fn emit_scm_facts(
    emitter: &Path,
    repo_root: &Path,
    inputs: &TemporaryInputs,
) -> Result<(), String> {
    let output = Command::new(emitter)
        .arg(REPO_ROOT_FLAG)
        .arg(repo_root)
        .arg("--out")
        .arg(&inputs.stable)
        .arg("--volatile-out")
        .arg(&inputs.volatile)
        .output()
        .map_err(|error| format!("run SCM facts emitter {}: {error}", emitter.display()))?;
    if !output.status.success() {
        return Err(format!(
            "SCM facts emitter {} failed with {}: {}",
            emitter.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}

fn exit_code(status: ExitStatus) -> ExitCode {
    status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .map_or(ExitCode::FAILURE, ExitCode::from)
}

struct TemporaryInputs {
    directory: PathBuf,
    stable: PathBuf,
    volatile: PathBuf,
}

impl TemporaryInputs {
    fn create() -> Result<Self, String> {
        let base = std::env::temp_dir().join("oya-ci-cargo-test-producer");
        fs::create_dir_all(&base)
            .map_err(|error| format!("create temporary input root {}: {error}", base.display()))?;

        for _ in 0..32 {
            let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| format!("read system clock for temporary inputs: {error}"))?
                .as_nanos();
            let directory = base.join(format!("{}-{nanos}-{sequence}", std::process::id()));
            match fs::create_dir(&directory) {
                Ok(()) => {
                    return Ok(Self {
                        stable: directory.join("scm-facts.generated.json"),
                        volatile: directory.join("scm-volatile-facts.generated.json"),
                        directory,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!(
                        "create temporary input directory {}: {error}",
                        directory.display()
                    ));
                }
            }
        }
        Err("could not allocate a unique temporary input directory after 32 attempts".to_owned())
    }
}

impl Drop for TemporaryInputs {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn required_flag_value_is_exact_and_unambiguous() {
        let values = args(&[REPO_ROOT_FLAG, "/repo", SCM_FACTS_FLAG, "/facts"]);
        assert_eq!(required_flag_value_index(&values, REPO_ROOT_FLAG), Ok(1));
        assert_eq!(required_flag_value_index(&values, SCM_FACTS_FLAG), Ok(3));
    }

    #[test]
    fn missing_empty_and_repeated_flags_fail_closed() {
        assert!(required_flag_value_index(&[], SCM_FACTS_FLAG).is_err());
        assert!(required_flag_value_index(&args(&[SCM_FACTS_FLAG]), SCM_FACTS_FLAG).is_err());
        assert!(
            required_flag_value_index(
                &args(&[SCM_FACTS_FLAG, "", REPO_ROOT_FLAG, "/repo"]),
                SCM_FACTS_FLAG
            )
            .is_err()
        );
        assert!(
            required_flag_value_index(
                &args(&[SCM_FACTS_FLAG, "/one", SCM_FACTS_FLAG, "/two"]),
                SCM_FACTS_FLAG
            )
            .is_err()
        );
    }

    #[test]
    fn temporary_inputs_are_unique_and_removed_on_drop() {
        let first = TemporaryInputs::create().expect("first temporary inputs");
        let first_dir = first.directory.clone();
        let second = TemporaryInputs::create().expect("second temporary inputs");
        assert_ne!(first.directory, second.directory);
        assert!(first.directory.is_dir());
        drop(first);
        assert!(!first_dir.exists());
    }

    #[test]
    fn regular_file_validation_rejects_directories_and_missing_paths() {
        let inputs = TemporaryInputs::create().expect("temporary inputs");
        assert!(require_regular_file(&inputs.directory, "fixture").is_err());
        assert!(require_regular_file(&inputs.stable, "fixture").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn regular_file_validation_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let inputs = TemporaryInputs::create().expect("temporary inputs");
        fs::write(&inputs.stable, "fixture").expect("write target");
        symlink(&inputs.stable, &inputs.volatile).expect("create symlink");
        assert!(require_regular_file(&inputs.volatile, "fixture").is_err());
    }

    #[test]
    fn cargo_test_binding_rejects_machine_relative_and_traversal_paths() {
        for invalid in [
            "target/debug/producer",
            "cargo-test-binary:",
            "cargo-test-binary:../producer",
            "cargo-test-binary:dir/producer",
            "cargo-test-binary:dir\\producer",
        ] {
            assert!(
                required_binary_binding_for_test(OsStr::new(invalid)).is_err(),
                "binding {invalid:?} must fail closed"
            );
        }
    }

    fn required_binary_binding_for_test(value: &OsStr) -> Result<PathBuf, String> {
        let declared = PathBuf::from(value);
        if !declared.is_absolute()
            && !value.to_str().is_some_and(|value| {
                value.starts_with(ci_path_resolver_adapters::CARGO_TEST_BINARY_PREFIX)
            })
        {
            return Err("ordinary relative binary binding is forbidden".to_owned());
        }
        ci_path_resolver_adapters::resolve_cargo_test_binary_from_executable(
            Path::new("/repo"),
            value,
            Path::new("/runtime-target/custom-profile/deps/adapter-test"),
        )
    }
}
