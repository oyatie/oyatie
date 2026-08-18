//! Retirement-marked `oya gen masterplan` adapter for the controller-owned Rust projection.
//!
//! Projection parsing, ordering, and serialization live in `ci-planning-projection`. This module
//! retains only the legacy CLI argument and output behavior; it is not materialization authority.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ci_generated_artifact_freshness::render_masterplan_projection_from_decisions;
use ci_planning_projection::MasterplanProjection;

const DEFAULT_DECISIONS_DIR: &str = "docs/decisions";
const DEFAULT_OUTPUT: &str = "docs/machine-readable/masterplan.generated.json";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GenMasterplanArgs {
    pub(crate) decisions_dir: PathBuf,
    pub(crate) output: PathBuf,
    pub(crate) write: bool,
    pub(crate) check: bool,
}

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let parsed = match parse_args(args, usage) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    execute(&parsed)
}

fn parse_args(args: Vec<String>, usage: &str) -> Result<GenMasterplanArgs, String> {
    let mut parsed = GenMasterplanArgs {
        decisions_dir: PathBuf::from(DEFAULT_DECISIONS_DIR),
        output: PathBuf::from(DEFAULT_OUTPUT),
        write: false,
        check: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--write" => parsed.write = true,
            "--check" => parsed.check = true,
            "--decisions-dir" => {
                parsed.decisions_dir = PathBuf::from(iter.next().ok_or_else(|| usage.to_owned())?);
            }
            "--output" => {
                parsed.output = PathBuf::from(iter.next().ok_or_else(|| usage.to_owned())?);
            }
            _ => return Err(usage.to_owned()),
        }
    }
    if parsed.write && parsed.check {
        return Err("gen masterplan: --write and --check are mutually exclusive".to_owned());
    }
    Ok(parsed)
}

fn execute(args: &GenMasterplanArgs) -> ExitCode {
    let (projection, json) = match render_projection(&args.decisions_dir) {
        Ok(rendered) => rendered,
        Err(message) => {
            eprintln!("gen masterplan: {message}");
            return ExitCode::FAILURE;
        }
    };

    if args.check {
        let committed = match std::fs::read_to_string(&args.output) {
            Ok(text) => text,
            Err(error) => {
                eprintln!(
                    "gen masterplan --check: committed projection unreadable {}: {error}",
                    args.output.display()
                );
                eprintln!("  run `oya gen masterplan --write` to generate it");
                return ExitCode::FAILURE;
            }
        };
        if committed == json {
            println!(
                "gen masterplan --check passed: {} matches the regenerated projection ({} ADRs, {} deliverables, {} milestones)",
                args.output.display(),
                projection.adr_count,
                projection.deliverable_count,
                projection.milestones.len()
            );
            return ExitCode::SUCCESS;
        }
        eprintln!(
            "gen masterplan --check failed: {} drifted from the regenerated projection",
            args.output.display()
        );
        eprintln!("  run `oya gen masterplan --write` to regenerate it");
        for line in first_diff_lines(&committed, &json) {
            eprintln!("  {line}");
        }
        return ExitCode::FAILURE;
    }

    if args.write {
        if let Some(parent) = args.output.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            eprintln!(
                "gen masterplan --write: output directory unwritable {}: {error}",
                parent.display()
            );
            return ExitCode::FAILURE;
        }
        if let Err(error) = std::fs::write(&args.output, &json) {
            eprintln!(
                "gen masterplan --write: output unwritable {}: {error}",
                args.output.display()
            );
            return ExitCode::FAILURE;
        }
        println!(
            "gen masterplan wrote {}: {} ADRs, {} deliverables, {} milestones",
            args.output.display(),
            projection.adr_count,
            projection.deliverable_count,
            projection.milestones.len()
        );
        return ExitCode::SUCCESS;
    }

    println!(
        "gen masterplan summary: {} accepted planning_impact ADRs, {} deliverables, {} milestones",
        projection.adr_count,
        projection.deliverable_count,
        projection.milestones.len()
    );
    for milestone in &projection.milestones {
        println!("  {} ({} ADRs)", milestone.milestone, milestone.adrs.len());
        for adr in &milestone.adrs {
            println!(
                "    {} [{}] {} deliverable(s)",
                adr.id,
                adr.status,
                adr.deliverables.len()
            );
        }
    }
    ExitCode::SUCCESS
}

pub(crate) fn render_projection(
    decisions_dir: &Path,
) -> Result<(MasterplanProjection, String), String> {
    render_masterplan_projection_from_decisions(decisions_dir)
}

fn first_diff_lines(committed: &str, regenerated: &str) -> Vec<String> {
    let committed_lines: Vec<&str> = committed.lines().collect();
    let regenerated_lines: Vec<&str> = regenerated.lines().collect();
    let max = committed_lines.len().max(regenerated_lines.len());
    let mut out = Vec::new();
    for index in 0..max {
        let committed_line = committed_lines.get(index).copied().unwrap_or("<absent>");
        let regenerated_line = regenerated_lines.get(index).copied().unwrap_or("<absent>");
        if committed_line != regenerated_line {
            out.push(format!("first drift at line {}:", index + 1));
            out.push(format!("    committed:    {committed_line}"));
            out.push(format!("    regenerated:  {regenerated_line}"));
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_defaults() {
        let parsed = parse_args(Vec::new(), "usage").expect("defaults parse");
        assert_eq!(parsed.decisions_dir, PathBuf::from(DEFAULT_DECISIONS_DIR));
        assert_eq!(parsed.output, PathBuf::from(DEFAULT_OUTPUT));
        assert!(!parsed.write);
        assert!(!parsed.check);
    }

    #[test]
    fn rejects_ambiguous_write_and_check_modes() {
        let error = parse_args(vec!["--write".into(), "--check".into()], "usage")
            .expect_err("mutually exclusive modes must fail");
        assert!(error.contains("mutually exclusive"));
    }

    #[test]
    fn first_diff_is_bounded_and_deterministic() {
        assert_eq!(
            first_diff_lines("same\nold\ntail\n", "same\nnew\nother\n"),
            [
                "first drift at line 2:",
                "    committed:    old",
                "    regenerated:  new",
            ]
        );
    }
}
