// Purpose: render or check the M02 exit-gate checklist markdown from canonical
// P22 inputs (phase-spec, impl-plan, INDEX). Ported from
// `scripts/render-m02-exit-checklist.py` per
// `evidence/audits/shell-python-replacement-audit-2026-05-15.md` row B-8.
// Naming-justification: `m02_exit_checklist` lives under
// `commands/doc/` mirroring the existing `doc adr-index` / `doc mdbook`
// renderer family; the CLI surface `doc render m02-exit-checklist` is
// canonical kebab-case verb-noun (ADR-0105 v4 BNF).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

const REQUIRED_INPUTS: &[&str] = &["phase-spec.md", "impl-plan.md", "INDEX.md"];
const REQUIRED_PHRASES: &[&str] = &[
    "Flip all 14 CI fitness lanes",
    "Application B2B shell",
    "sibling-team self-sufficiency",
];

const DEFAULT_PHASE_DIR: &str = ".omc/plans/milestones/M02b-substrate/phases/P22-m02-exit-gate";
const DEFAULT_OUTPUT_PATH: &str = "docs/architecture/m02-exit-checklist.md";

pub(super) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    match parse_args(args, usage) {
        Ok(args) => execute(args),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct M02ExitChecklistArgs {
    phase_dir: PathBuf,
    output: PathBuf,
    check: bool,
    write: bool,
}

fn parse_args(args: Vec<String>, usage: &str) -> Result<M02ExitChecklistArgs, String> {
    let mut parsed = M02ExitChecklistArgs {
        phase_dir: PathBuf::from(DEFAULT_PHASE_DIR),
        output: PathBuf::from(DEFAULT_OUTPUT_PATH),
        check: false,
        write: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--check" => parsed.check = true,
            "--write" => parsed.write = true,
            "--phase-dir" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.phase_dir = PathBuf::from(value);
            }
            "--output" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.output = PathBuf::from(value);
            }
            _ => return Err(usage.to_owned()),
        }
    }
    Ok(parsed)
}

fn execute(args: M02ExitChecklistArgs) -> ExitCode {
    let rendered = match render_from_phase_dir(&args.phase_dir) {
        Ok(text) => text,
        Err(message) => {
            eprintln!("render-m02-exit-checklist: {message}");
            return ExitCode::FAILURE;
        }
    };
    if args.write {
        if let Some(parent) = args.output.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            eprintln!(
                "render-m02-exit-checklist: output dir unwritable {}: {error}",
                parent.display()
            );
            return ExitCode::FAILURE;
        }
        if let Err(error) = std::fs::write(&args.output, &rendered) {
            eprintln!(
                "render-m02-exit-checklist: output unwritable {}: {error}",
                args.output.display()
            );
            return ExitCode::FAILURE;
        }
        println!("wrote {}", args.output.display());
        return ExitCode::SUCCESS;
    }
    if args.check {
        if args.output.exists() {
            match std::fs::read_to_string(&args.output) {
                Ok(current) => {
                    if current != rendered {
                        eprintln!(
                            "render-m02-exit-checklist: {} is stale; run with --write",
                            args.output.display()
                        );
                        return ExitCode::FAILURE;
                    }
                    println!("render-m02-exit-checklist: output parity ok");
                }
                Err(error) => {
                    eprintln!(
                        "render-m02-exit-checklist: output unreadable {}: {error}",
                        args.output.display()
                    );
                    return ExitCode::FAILURE;
                }
            }
        } else {
            println!(
                "render-m02-exit-checklist: output absent because P22 is not active; source inputs ok"
            );
        }
        return ExitCode::SUCCESS;
    }
    print!("{rendered}");
    ExitCode::SUCCESS
}

pub(crate) fn render_from_phase_dir(phase_dir: &Path) -> Result<String, String> {
    let mut contents: Vec<(String, String)> = Vec::new();
    for name in REQUIRED_INPUTS {
        let path = phase_dir.join(name);
        if !path.exists() {
            return Err(format!("{}", path.display()));
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("{} unreadable: {error}", path.display()))?;
        contents.push(((*name).to_string(), text));
    }
    let phase_spec = contents
        .iter()
        .find(|(name, _)| name == "phase-spec.md")
        .map(|(_, text)| text.as_str())
        .unwrap_or("");
    let missing: Vec<&&str> = REQUIRED_PHRASES
        .iter()
        .filter(|phrase| !phase_spec.contains(**phrase))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "phase-spec missing expected P22 phrases: {}",
            missing
                .into_iter()
                .map(|phrase| (*phrase).to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    Ok(render_template())
}

fn render_template() -> String {
    let lines = [
        "# M02 Exit Gate Checklist",
        "",
        "<!-- generated by oya doc render m02-exit-checklist; write with --write when P22 is active -->",
        "",
        "**Milestone:** M02b-substrate",
        "**Phase:** P22-m02-exit-gate",
        "**Status:** not-yet-active; source inputs present",
        "",
        "## Source inputs",
        "",
        "| Source | Status |",
        "|---|---|",
        "| `.omc/plans/milestones/M02b-substrate/phases/P22-m02-exit-gate/phase-spec.md` | present |",
        "| `.omc/plans/milestones/M02b-substrate/phases/P22-m02-exit-gate/impl-plan.md` | present |",
        "| `.omc/plans/milestones/M02b-substrate/phases/P22-m02-exit-gate/INDEX.md` | present |",
        "",
        "## Gate families",
        "",
        "- 14 CI fitness lanes flipped from report-only to BLOCKER.",
        "- Application B2B shell deployability evidence.",
        "- Sibling-team onboarding validation evidence.",
        "- Full workspace compile, nextest, deny, plane, and wave gates.",
        "",
    ];
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_phase_dir(test_name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("oya-dev-cli-m02-exit-checklist-{test_name}"));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temp dir create");
        path
    }

    fn write_canonical_inputs(dir: &Path) {
        fs::write(
            dir.join("phase-spec.md"),
            "# P22\n\nFlip all 14 CI fitness lanes from report-only to BLOCKER.\n\
             Application B2B shell deployability evidence.\nsibling-team self-sufficiency.\n",
        )
        .expect("write phase-spec");
        fs::write(dir.join("impl-plan.md"), "impl-plan\n").expect("write impl-plan");
        fs::write(dir.join("INDEX.md"), "INDEX\n").expect("write INDEX");
    }

    #[test]
    fn render_passes_on_canonical_inputs() {
        let dir = temp_phase_dir("ok");
        write_canonical_inputs(&dir);
        let rendered = render_from_phase_dir(&dir).expect("render must pass");
        assert!(rendered.contains("M02 Exit Gate Checklist"));
        assert!(rendered.contains("BLOCKER"));
    }

    #[test]
    fn render_rejects_phase_spec_missing_phrase() {
        let dir = temp_phase_dir("missing-phrase");
        fs::write(dir.join("phase-spec.md"), "wrong content\n").expect("write phase-spec");
        fs::write(dir.join("impl-plan.md"), "x\n").expect("write impl-plan");
        fs::write(dir.join("INDEX.md"), "x\n").expect("write INDEX");
        let error = render_from_phase_dir(&dir).expect_err("missing phrases must fail");
        assert!(error.contains("phase-spec missing"));
    }

    #[test]
    fn render_rejects_missing_input_file() {
        let dir = temp_phase_dir("missing-file");
        let error = render_from_phase_dir(&dir).expect_err("missing inputs must fail");
        assert!(error.contains("phase-spec.md"));
    }
}
