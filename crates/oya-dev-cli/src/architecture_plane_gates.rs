use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlanesValidateArgs {
    repo_root: PathBuf,
    all: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaneValidationReport {
    pub planes_checked: usize,
    pub lanes_checked: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WaveIntegrationValidateArgs {
    milestone: String,
    manifest: PathBuf,
    phases_dir: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WaveIntegrationReport {
    pub phases_checked: usize,
    pub dependencies_checked: usize,
}

pub(crate) fn parse_planes_validate_args(args: Vec<String>) -> Result<PlanesValidateArgs, String> {
    let mut repo_root = PathBuf::from(".");
    let mut all = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--repo-root requires a path".to_string())?,
                );
            }
            "--all" => all = true,
            other => return Err(format!("unexpected planes argument: {other}")),
        }
    }
    Ok(PlanesValidateArgs { repo_root, all })
}

pub(crate) fn validate_planes_gate(
    args: PlanesValidateArgs,
) -> Result<PlaneValidationReport, String> {
    if !args.all {
        return Err("planes validation currently requires --all".to_string());
    }
    let architecture_dir = args.repo_root.join("docs/architecture");
    if !architecture_dir.is_dir() {
        return Err(format!(
            "architecture plane artifacts are not present: {}",
            architecture_dir.display()
        ));
    }

    let mut planes_checked = 0_usize;
    let mut lanes_checked = 0_usize;
    for entry in fs::read_dir(&architecture_dir)
        .map_err(|error| format!("{}: {error}", architecture_dir.display()))?
    {
        let entry = entry.map_err(|error| format!("{}: {error}", architecture_dir.display()))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            let content = fs::read_to_string(&path)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            if content.contains("plane") || content.contains("Plane") {
                planes_checked += 1;
            }
            lanes_checked += content.matches("lane").count() + content.matches("Lane").count();
        }
    }

    if planes_checked == 0 || lanes_checked == 0 {
        return Err("architecture plane artifacts are incomplete".to_string());
    }

    Ok(PlaneValidationReport {
        planes_checked,
        lanes_checked,
    })
}

pub(crate) fn parse_wave_integration_validate_args(
    args: Vec<String>,
) -> Result<WaveIntegrationValidateArgs, String> {
    let mut milestone = None;
    let mut manifest = PathBuf::from(".omc/plans/M01-M03-parallelization-manifest.md");
    let mut phases_dir = PathBuf::from(".omc/plans/milestones/M02-substrate/phases");
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--milestone" => {
                milestone = Some(
                    iter.next()
                        .ok_or_else(|| "--milestone requires a value".to_string())?,
                );
            }
            "--manifest" => {
                manifest = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--manifest requires a path".to_string())?,
                );
            }
            "--phases-dir" => {
                phases_dir = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--phases-dir requires a path".to_string())?,
                );
            }
            other => return Err(format!("unexpected wave-integration argument: {other}")),
        }
    }
    Ok(WaveIntegrationValidateArgs {
        milestone: milestone.ok_or_else(|| "--milestone is required".to_string())?,
        manifest,
        phases_dir,
    })
}

pub(crate) fn validate_wave_integration_gate(
    args: WaveIntegrationValidateArgs,
) -> Result<WaveIntegrationReport, String> {
    let manifest = read_required(&args.manifest)?;
    if !manifest.contains(&args.milestone) {
        return Err(format!(
            "wave integration manifest does not mention milestone {}",
            args.milestone
        ));
    }
    if !args.phases_dir.is_dir() {
        return Err(format!(
            "wave integration phases directory is not present: {}",
            args.phases_dir.display()
        ));
    }

    let mut phases_checked = 0_usize;
    let mut dependencies_checked = 0_usize;
    for entry in fs::read_dir(&args.phases_dir)
        .map_err(|error| format!("{}: {error}", args.phases_dir.display()))?
    {
        let entry = entry.map_err(|error| format!("{}: {error}", args.phases_dir.display()))?;
        if entry.path().is_dir() {
            phases_checked += 1;
            let phase_spec = entry.path().join("phase-spec.md");
            if phase_spec.exists() {
                let content = read_required(&phase_spec)?;
                dependencies_checked += content.matches("depends").count()
                    + content.matches("dependency").count()
                    + content.matches("prerequisite").count();
            }
        }
    }

    if phases_checked == 0 || dependencies_checked == 0 {
        return Err("wave integration artifacts are incomplete".to_string());
    }

    Ok(WaveIntegrationReport {
        phases_checked,
        dependencies_checked,
    })
}

fn read_required(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}
