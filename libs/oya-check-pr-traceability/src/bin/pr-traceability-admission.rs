use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use oya_check_pr_traceability::{
    PrTraceabilityDocument, PrTraceabilityPolicy, scaffold_pr_body, validate_pr_traceability,
    validate_pr_traceability_all,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct Args {
    pr_body_path: PathBuf,
    pr_title: String,
    require_code_review: bool,
    forbid_code_review: bool,
    scaffold: bool,
    all_violations: bool,
}

fn main() -> ExitCode {
    let args = match parse_args(env::args().skip(1).collect()) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("PR review admission failed: {message}");
            return ExitCode::FAILURE;
        }
    };

    if args.scaffold {
        // Prints ONLY the template (no banner) so `--scaffold > body.md` is clean to redirect.
        print!("{}", scaffold_pr_body());
        return ExitCode::SUCCESS;
    }

    match run(args) {
        Ok((sections, code_review_present)) => {
            println!(
                "PR review admission passed: {sections} required sections, code_review_present={code_review_present}"
            );
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("PR review admission failed: {message}");
            // Name the remedy in the failure itself. The required shape is not
            // guessable — it is a fixed set of section headings and field labels
            // that this binary can emit verbatim — and `gh pr create --body`
            // bypasses .github/pull_request_template.md, so a CLI author
            // otherwise rediscovers one literal per CI round trip.
            eprintln!("\nThe exact required shape is available from this binary:");
            eprintln!(
                "  buck2 run //libs/oya-check-pr-traceability:pr-traceability-admission-bin -- --scaffold"
            );
            eprintln!("\nValidate a body locally before pushing (same code path as CI):");
            eprintln!(
                "  buck2 run //libs/oya-check-pr-traceability:pr-traceability-admission-bin -- \\"
            );
            eprintln!("    --pr-title \"<title>\" --pr-body <path> --require-code-review");
            eprintln!("\nAdd --all-violations to see every problem at once instead of the first.");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(usize, bool), String> {
    let body = fs::read_to_string(&args.pr_body_path).map_err(|error| {
        format!(
            "PR body unreadable {}: {error}",
            args.pr_body_path.display()
        )
    })?;
    let document = PrTraceabilityDocument {
        document_id: args.pr_body_path.display().to_string(),
        title: args.pr_title,
        body,
    };
    let policy = PrTraceabilityPolicy {
        require_code_review: args.require_code_review,
        forbid_code_review: args.forbid_code_review,
    };

    if args.all_violations {
        let errors = validate_pr_traceability_all(&document, policy);
        if !errors.is_empty() {
            let listed = errors
                .iter()
                .map(|error| format!("  - {error:?}"))
                .collect::<Vec<_>>()
                .join("\n");
            return Err(format!("{} violation(s):\n{listed}", errors.len()));
        }
    }

    let report =
        validate_pr_traceability(&document, policy).map_err(|error| format!("{error:?}"))?;
    Ok((report.required_sections_checked, report.code_review_present))
}

fn parse_args(args: Vec<String>) -> Result<Args, String> {
    let mut parsed = Args {
        pr_body_path: PathBuf::from("docs/templates/pull-request-template.md"),
        pr_title: String::new(),
        require_code_review: true,
        forbid_code_review: false,
        scaffold: false,
        all_violations: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--pr-body" | "--check" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.pr_body_path = PathBuf::from(path);
            }
            "--pr-title" => {
                let Some(title) = iter.next() else {
                    return Err(usage());
                };
                parsed.pr_title = title;
            }
            "--require-code-review" => {
                parsed.require_code_review = true;
                parsed.forbid_code_review = false;
            }
            "--forbid-code-review" => {
                parsed.forbid_code_review = true;
                parsed.require_code_review = false;
            }
            "--scaffold" => parsed.scaffold = true,
            "--all-violations" => parsed.all_violations = true,
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

fn usage() -> String {
    "usage: pr-traceability-admission [--scaffold] [--pr-title <title>] [--pr-body|--check <path>] [--require-code-review|--forbid-code-review] [--all-violations]".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_defaults_to_merge_admission_policy() {
        assert_eq!(
            parse_args(Vec::new()),
            Ok(Args {
                pr_body_path: PathBuf::from("docs/templates/pull-request-template.md"),
                pr_title: String::new(),
                require_code_review: true,
                forbid_code_review: false,
                scaffold: false,
                all_violations: false,
            })
        );
    }

    #[test]
    fn parse_accepts_title_body_and_author_policy() {
        assert_eq!(
            parse_args(vec![
                "--pr-title".into(),
                "Ready".into(),
                "--pr-body".into(),
                "body.md".into(),
                "--forbid-code-review".into(),
            ]),
            Ok(Args {
                pr_body_path: PathBuf::from("body.md"),
                pr_title: "Ready".into(),
                require_code_review: false,
                forbid_code_review: true,
                scaffold: false,
                all_violations: false,
            })
        );
    }

    #[test]
    fn parse_accepts_scaffold_check_and_all_violations_flags() {
        assert_eq!(
            parse_args(vec![
                "--scaffold".into(),
                "--check".into(),
                "body.md".into(),
                "--all-violations".into(),
            ]),
            Ok(Args {
                pr_body_path: PathBuf::from("body.md"),
                pr_title: String::new(),
                require_code_review: true,
                forbid_code_review: false,
                scaffold: true,
                all_violations: true,
            })
        );
    }

    #[test]
    fn scaffold_round_trips_through_check_with_only_the_pending_verdict_failing() {
        let path = env::temp_dir().join(format!(
            "pr-traceability-admission-scaffold-test-{}.md",
            std::process::id()
        ));
        fs::write(&path, scaffold_pr_body()).expect("write scaffold to temp file");

        let result = run(Args {
            pr_body_path: path.clone(),
            pr_title: "Scaffolded PR".into(),
            require_code_review: true,
            forbid_code_review: false,
            scaffold: false,
            all_violations: true,
        });

        let _ = fs::remove_file(&path);
        assert_eq!(
            result,
            Err("1 violation(s):\n  - MissingCodeReviewApproval".into())
        );
    }
}
