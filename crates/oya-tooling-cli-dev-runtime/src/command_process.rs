use std::io::{self, Write};
use std::path::Path;

pub(crate) fn run_check_script_status_streaming(
    check_script_path: &Path,
) -> io::Result<std::process::ExitStatus> {
    let mut command = std::process::Command::new("bash");
    command
        .arg(check_script_path)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());
    scrub_cargo_harness_env(&mut command);
    command.status()
}

pub(crate) fn run_check_script_process(
    check_script_path: &Path,
) -> io::Result<std::process::Output> {
    let mut command = std::process::Command::new("bash");
    command.arg(check_script_path);
    scrub_cargo_harness_env(&mut command);
    command.output()
}

fn scrub_cargo_harness_env(command: &mut std::process::Command) {
    // Grounded harness isolation: repo-level check scripts are often launched
    // from binaries that were themselves started by `cargo run` or `cargo test`.
    // Cargo injects package metadata into that runtime environment; nested cargo
    // subcommands/plugins must see a normal shell environment instead. Preserve
    // user/tooling configuration such as PATH, RUSTC_WRAPPER, CARGO_HOME, and
    // CARGO_TARGET_DIR while removing Cargo's package/run metadata.
    const EXACT_KEYS: &[&str] = &[
        "CARGO",
        "CARGO_BIN_NAME",
        "CARGO_CRATE_NAME",
        "CARGO_MANIFEST_DIR",
        "CARGO_MANIFEST_PATH",
        "CARGO_PRIMARY_PACKAGE",
    ];
    for key in EXACT_KEYS {
        command.env_remove(key);
    }
    for (key, _) in std::env::vars_os() {
        let Some(key) = key.to_str() else {
            continue;
        };
        if key.starts_with("CARGO_PKG_") || key.starts_with("CARGO_BIN_EXE_") {
            command.env_remove(key);
        }
    }
}

pub(crate) fn replay_process_output(output: &std::process::Output) -> Result<(), String> {
    io::stdout()
        .write_all(&output.stdout)
        .map_err(|error| format!("could not write child stdout: {error}"))?;
    io::stderr()
        .write_all(&output.stderr)
        .map_err(|error| format!("could not write child stderr: {error}"))?;
    Ok(())
}

pub(crate) fn process_status_label(status: &std::process::ExitStatus) -> String {
    status
        .code()
        .map(|code| format!("exit code {code}"))
        .unwrap_or_else(|| "signal termination".into())
}
