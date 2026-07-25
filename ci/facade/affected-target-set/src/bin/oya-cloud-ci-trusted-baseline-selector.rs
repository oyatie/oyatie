//! Network-free trusted baseline selector for GH #1323.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fs;
use std::process::ExitCode;

use ci_affected_target_set::select_trusted_baseline_artifacts;
use serde_json::json;

const SCHEMA_VERSION: u64 = 1;

struct Args {
    merge_base_sha: String,
    workflow_runs_json: String,
    workflow_artifacts_json: String,
}

fn parse_args(mut argv: impl Iterator<Item = String>) -> Result<Args, String> {
    let _binary = argv.next();
    let mut merge_base_sha = None;
    let mut workflow_runs_json = None;
    let mut workflow_artifacts_json = None;
    while let Some(arg) = argv.next() {
        let value = match arg.as_str() {
            "--merge-base-sha" | "--workflow-runs-json" | "--workflow-artifacts-json" => argv
                .next()
                .ok_or_else(|| format!("`{arg}` requires a value"))?,
            other => return Err(format!("unknown argument `{other}`")),
        };
        match arg.as_str() {
            "--merge-base-sha" => merge_base_sha = Some(value),
            "--workflow-runs-json" => workflow_runs_json = Some(value),
            "--workflow-artifacts-json" => workflow_artifacts_json = Some(value),
            _ => unreachable!("validated argument"),
        }
    }
    Ok(Args {
        merge_base_sha: merge_base_sha.ok_or("`--merge-base-sha <sha>` is required")?,
        workflow_runs_json: workflow_runs_json
            .ok_or("`--workflow-runs-json <path>` is required")?,
        workflow_artifacts_json: workflow_artifacts_json
            .ok_or("`--workflow-artifacts-json <path>` is required")?,
    })
}

fn read_input(path: &str, label: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("cannot read {label} `{path}`: {e}"))
}

fn error_receipt(merge_base_sha: Option<&str>, error: &str) -> String {
    serde_json::to_string(&json!({
        "schema_version": SCHEMA_VERSION,
        "decision": "ERROR",
        "merge_base_sha": merge_base_sha,
        "error": error,
    }))
    .unwrap_or_else(|_| {
        "{\"schema_version\":1,\"decision\":\"ERROR\",\"error\":\"receipt serialization failed\"}"
            .to_owned()
    })
}

fn run(argv: impl Iterator<Item = String>) -> Result<String, (Option<String>, String)> {
    let args = parse_args(argv).map_err(|error| (None, error))?;
    let sha = args.merge_base_sha.clone();
    let runs = read_input(&args.workflow_runs_json, "workflow-runs JSON")
        .map_err(|error| (Some(sha.clone()), error))?;
    let artifacts = read_input(&args.workflow_artifacts_json, "workflow-artifacts JSON")
        .map_err(|error| (Some(sha.clone()), error))?;

    match select_trusted_baseline_artifacts(&runs, &artifacts, &sha)
        .map_err(|error| (Some(sha.clone()), error))?
    {
        Some(selection) => serde_json::to_string(&json!({
            "schema_version": SCHEMA_VERSION,
            "decision": "SELECTED",
            "merge_base_sha": selection.merge_base_sha,
            "run_id": selection.run_id,
            "repository_id": selection.repository_id,
            "build_artifact": {
                "id": selection.build_artifact.id,
                "name": selection.build_artifact.name,
            },
            "test_artifact": {
                "id": selection.test_artifact.id,
                "name": selection.test_artifact.name,
            },
        }))
        .map_err(|error| (Some(sha), format!("serialize selected receipt: {error}"))),
        None => serde_json::to_string(&json!({
            "schema_version": SCHEMA_VERSION,
            "decision": "FALLBACK",
            "merge_base_sha": sha,
            "reason": "atomic trusted BUILD + TEST baseline pair unavailable; use clean-worktree cold rebuild",
        }))
        .map_err(|error| (None, format!("serialize fallback receipt: {error}"))),
    }
}

fn main() -> ExitCode {
    match run(std::env::args()) {
        Ok(receipt) => {
            println!("{receipt}");
            ExitCode::SUCCESS
        }
        Err((sha, error)) => {
            eprintln!("{}", error_receipt(sha.as_deref(), &error));
            ExitCode::from(2)
        }
    }
}
