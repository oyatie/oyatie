use iac_operator_access_app::{AccessError, Profile, runtime};
use std::{path::PathBuf, process::ExitCode};

fn execute() -> Result<(), AccessError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--help"] {
        println!(
            "iac-operator-access-app <status|inspect> <profile-name>\nProfiles: $OYATIE_ACCESS_PROFILE_DIR/<name>.json or $HOME/.config/oyatie/operator-access/<name>.json\nRead-only status or fixed Kubernetes/Talos disk inventory; no provisioning or reset. Inspection reports observations, not deployment readiness. Requires oci, age, tar, ssh, talosctl, kubectl.\nCredentials stay in memory. Bastion sessions expire after 30 minutes even if cleanup is interrupted by SIGKILL or host loss."
        );
        return Ok(());
    }
    if args.len() != 2
        || !matches!(args[0].as_str(), "status" | "inspect")
        || args[1].is_empty()
        || !args[1]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err(AccessError::InvalidProfile);
    }
    let directory = match std::env::var_os("OYATIE_ACCESS_PROFILE_DIR") {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(std::env::var_os("HOME").ok_or(AccessError::InvalidProfile)?)
            .join(".config/oyatie/operator-access"),
    };
    let bytes = std::fs::read(directory.join(format!("{}.json", args[1])))
        .map_err(|_| AccessError::InvalidProfile)?;
    let profile: Profile =
        serde_json::from_slice(&bytes).map_err(|_| AccessError::InvalidProfile)?;
    runtime::install_signal_handlers()?;
    let report = if args[0] == "inspect" {
        runtime::inspect(&profile)?
    } else {
        runtime::status(&profile)?
    };
    println!("{report}");
    Ok(())
}

fn main() -> ExitCode {
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
