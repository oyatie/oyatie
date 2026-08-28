use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

fn workflow() -> String {
    std::fs::read_to_string(repo_root().join(".github/workflows/presubmit.yml"))
        .expect("presubmit workflow")
}

fn ensure(ok: bool, message: impl Into<String>) -> Result<(), String> {
    if ok { Ok(()) } else { Err(message.into()) }
}

fn indentation(line: &str) -> usize {
    line.bytes().take_while(|byte| *byte == b' ').count()
}

fn direct_entries(block: &str, indent: usize) -> Vec<(&str, &str)> {
    block
        .lines()
        .filter_map(|line| {
            if indentation(line) != indent {
                return None;
            }
            let direct = &line[indent..];
            if direct.is_empty() || direct.starts_with('#') {
                return None;
            }
            direct
                .split_once(':')
                .map(|(key, value)| (key.trim(), value.trim()))
        })
        .collect()
}

fn between<'a>(yaml: &'a str, start: &str, end: &str) -> Result<&'a str, String> {
    let rest = yaml
        .split_once(start)
        .map(|(_, rest)| rest)
        .ok_or_else(|| format!("missing {start:?}"))?;
    rest.split_once(end)
        .map(|(body, _)| body)
        .ok_or_else(|| format!("missing {end:?} after {start:?}"))
}

fn validate_root(yaml: &str) -> Result<(), String> {
    ensure(
        direct_entries(yaml, 0)
            == [
                ("name", "presubmit"),
                ("on", ""),
                ("permissions", ""),
                ("env", ""),
                ("jobs", ""),
            ],
        "workflow root keys must be closed; defaults are forbidden",
    )?;
    let triggers = between(yaml, "\non:\n", "\npermissions:\n")?;
    ensure(
        direct_entries(triggers, 2) == [("pull_request", ""), ("merge_group", "")],
        "workflow triggers must be only pull request and merge group",
    )?;
    let pull_request = between(triggers, "  pull_request:\n", "\n  merge_group:\n")?;
    ensure(
        direct_entries(pull_request, 4)
            == [
                ("branches", "[dev]"),
                ("types", "[opened, reopened, synchronize]"),
            ],
        "pull request trigger must target dev with the exact activity set",
    )?;
    let merge_group = triggers
        .split_once("\n  merge_group:\n")
        .map(|(_, body)| body)
        .ok_or_else(|| "missing merge group trigger".to_owned())?;
    ensure(
        direct_entries(merge_group, 4) == [("types", "[checks_requested]")],
        "merge group trigger must use checks_requested",
    )?;
    let permissions = between(yaml, "\npermissions:\n", "\nenv:\n")?;
    ensure(
        direct_entries(permissions, 2) == [("contents", "read"), ("pull-requests", "read")],
        "workflow permissions must remain least privilege",
    )?;
    let env = between(yaml, "\nenv:\n", "\njobs:\n")?;
    ensure(
        direct_entries(env, 2)
            == [
                ("CARGO_PROFILE_DEV_DEBUG", "\"0\""),
                ("CARGO_PROFILE_TEST_DEBUG", "\"0\""),
                ("CARGO_INCREMENTAL", "\"0\""),
            ],
        "workflow env must contain only the three inert Cargo controls",
    )
}

fn validate_job(yaml: &str, id: &str, next_id: &str) -> Result<(), String> {
    let start = format!("\n  {id}:\n");
    let end = format!("\n  {next_id}:\n");
    let body = between(yaml, &start, &end)?;
    let expected = if id == "layout" {
        vec![
            ("name", "repository layout"),
            ("runs-on", "ubuntu-24.04"),
            ("timeout-minutes", "10"),
            ("steps", ""),
        ]
    } else {
        vec![
            ("name", "occupancy (path-set)"),
            ("if", "github.event_name == 'pull_request'"),
            ("runs-on", "ubuntu-24.04"),
            ("timeout-minutes", "10"),
            ("steps", ""),
        ]
    };
    ensure(
        direct_entries(body, 4) == expected,
        format!("{id} job execution context must be exact"),
    )
}

fn validate_execution_context(yaml: &str) -> Result<(), String> {
    validate_root(yaml)?;
    validate_job(yaml, "layout", "occupancy")?;
    validate_job(yaml, "occupancy", "lint")
}

#[test]
fn protected_policy_execution_context_is_closed() {
    validate_execution_context(&workflow()).unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn execution_context_rejects_candidate_and_fail_open_mutations() {
    let yaml = workflow();
    let steps = "    steps:";
    let job_wrapper = yaml.replacen(
        steps,
        "    env:\n      RUSTC_WRAPPER: ${{ github.workspace }}/candidate/rustc-wrapper\n    steps:",
        1,
    );
    assert!(validate_execution_context(&job_wrapper).is_err());

    let fail_open = yaml.replacen(steps, "    continue-on-error: true\n    steps:", 1);
    assert!(validate_execution_context(&fail_open).is_err());

    let root_env = "  CARGO_INCREMENTAL: \"0\"";
    let root_wrapper = yaml.replacen(
        root_env,
        "  CARGO_INCREMENTAL: \"0\"\n  RUSTC_WRAPPER: ${{ github.workspace }}/candidate/rustc-wrapper",
        1,
    );
    assert!(validate_execution_context(&root_wrapper).is_err());

    let defaults = yaml.replacen(
        "\njobs:\n",
        "\ndefaults:\n  run:\n    shell: candidate/runner {0}\njobs:\n",
        1,
    );
    assert!(validate_execution_context(&defaults).is_err());

    let merge_group = "  merge_group:\n    types: [checks_requested]";
    for event in ["workflow_call", "push"] {
        let trigger = yaml.replacen(merge_group, &format!("{merge_group}\n  {event}:"), 1);
        assert!(validate_execution_context(&trigger).is_err(), "{event}");
    }
}
