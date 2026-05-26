//! `oya gate validate masterplan-drift` (ADR-0364 D4).
//!
//! The drift gate is the inspection mechanism that keeps the generated
//! masterplan projection honest: the committed
//! `docs/machine-readable/masterplan.generated.json` must equal the projection
//! regenerated in-memory from the ADR log. This wraps
//! `oya gen masterplan --check` (Amazon "mechanisms, not intentions").

use std::path::PathBuf;
use std::process::ExitCode;

use crate::commands::generate::masterplan::{self, GenMasterplanArgs};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MasterplanDriftArgs {
    pub(crate) decisions_dir: PathBuf,
    pub(crate) output: PathBuf,
}

pub(crate) fn parse_masterplan_drift_args(
    args: Vec<String>,
) -> Result<MasterplanDriftArgs, String> {
    let mut parsed = MasterplanDriftArgs {
        decisions_dir: PathBuf::from("docs/decisions"),
        output: PathBuf::from("docs/machine-readable/masterplan.generated.json"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--decisions-dir" => {
                parsed.decisions_dir = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--decisions-dir requires a value".to_string())?,
                );
            }
            "--masterplan" | "--output" => {
                parsed.output = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--masterplan requires a value".to_string())?,
                );
            }
            other => {
                return Err(format!(
                    "masterplan-drift: unknown flag {other:?}; allowed: --decisions-dir, --masterplan"
                ));
            }
        }
    }
    Ok(parsed)
}

pub(crate) fn run_masterplan_drift(args: Vec<String>) -> ExitCode {
    let parsed = match parse_masterplan_drift_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    // Delegate to the generator's --check path: regenerate in-memory and diff
    // against the committed projection. The generator owns the diff + messaging.
    masterplan::run(
        vec![
            "--check".to_string(),
            "--decisions-dir".to_string(),
            parsed.decisions_dir.to_string_lossy().into_owned(),
            "--output".to_string(),
            parsed.output.to_string_lossy().into_owned(),
        ],
        "oya gate validate masterplan-drift [--decisions-dir <docs/decisions>] [--masterplan <docs/machine-readable/masterplan.generated.json>]",
    )
}

// Keep the parsed args constructible into generator args for any future
// callers that want the typed form rather than the argv reconstruction above.
impl From<MasterplanDriftArgs> for GenMasterplanArgs {
    fn from(args: MasterplanDriftArgs) -> Self {
        GenMasterplanArgs {
            decisions_dir: args.decisions_dir,
            output: args.output,
            write: false,
            check: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_defaults() {
        let parsed = parse_masterplan_drift_args(vec![]).expect("defaults");
        assert_eq!(parsed.decisions_dir, PathBuf::from("docs/decisions"));
        assert_eq!(
            parsed.output,
            PathBuf::from("docs/machine-readable/masterplan.generated.json")
        );
    }

    #[test]
    fn rejects_unknown_flag() {
        assert!(parse_masterplan_drift_args(vec!["--nope".into()]).is_err());
    }

    #[test]
    fn converts_to_check_generator_args() {
        let drift = parse_masterplan_drift_args(vec![]).expect("defaults");
        let generator: GenMasterplanArgs = drift.into();
        assert!(generator.check);
        assert!(!generator.write);
    }
}
