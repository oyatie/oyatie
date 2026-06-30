use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use oya_check_pr_traceability::{
    PrTraceabilityDocument, PrTraceabilityPolicy, validate_pr_traceability,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct Args {
    pr_body_path: PathBuf,
    pr_title: String,
    require_code_review: bool,
    forbid_code_review: bool,
}

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok((sections, code_review_present)) => {
            println!(
                "PR review admission passed: {sections} required sections, code_review_present={code_review_present}"
            );
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("PR review admission failed: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(usize, bool), String> {
    let args = parse_args(args)?;
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
    let report = validate_pr_traceability(
        &document,
        PrTraceabilityPolicy {
            require_code_review: args.require_code_review,
            forbid_code_review: args.forbid_code_review,
        },
    )
    .map_err(|error| format!("{error:?}"))?;
    Ok((report.required_sections_checked, report.code_review_present))
}

fn parse_args(args: Vec<String>) -> Result<Args, String> {
    let mut parsed = Args {
        pr_body_path: PathBuf::from("docs/templates/pull-request-template.md"),
        pr_title: String::new(),
        require_code_review: true,
        forbid_code_review: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--pr-body" => {
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
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

fn usage() -> String {
    "usage: pr-traceability-admission [--pr-title <title>] [--pr-body <path>] [--require-code-review|--forbid-code-review]".into()
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
            })
        );
    }
}
