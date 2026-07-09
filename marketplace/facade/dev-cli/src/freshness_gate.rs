use std::path::PathBuf;

use ci_generated_artifact_freshness::{CheckReport, check_repo};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FreshnessGateArgs {
    pub(crate) repo_root: PathBuf,
}

pub(crate) fn parse_freshness_gate_args(args: Vec<String>) -> Result<FreshnessGateArgs, String> {
    let mut repo_root = PathBuf::from(".");
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return Err("freshness: --repo-root requires a path".to_owned());
                };
                repo_root = PathBuf::from(value);
            }
            _ => return Err(crate::usage()),
        }
    }
    Ok(FreshnessGateArgs { repo_root })
}

pub(crate) fn validate_freshness_gate(args: FreshnessGateArgs) -> Result<CheckReport, String> {
    check_repo(&args.repo_root).map_err(|error| error.to_string())
}
