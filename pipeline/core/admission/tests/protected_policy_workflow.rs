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

const CHECKOUT: &str = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
const TOOLCHAIN: &str = "dtolnay/rust-toolchain@21dc36fb71dd22e3317045c0c31a3f4249868b17";

#[derive(Clone, Copy)]
struct JobSpec {
    id: &'static str,
    next_id: &'static str,
    build_name: &'static str,
    admit_name: &'static str,
    target_dir: &'static str,
    package: &'static str,
}

const JOBS: [JobSpec; 2] = [
    JobSpec {
        id: "layout",
        next_id: "occupancy",
        build_name: "Build protected path-layout application",
        admit_name: "Admit candidate tree",
        target_dir: "oyatie-layout-admission",
        package: "pipeline-path-layout-app",
    },
    JobSpec {
        id: "occupancy",
        next_id: "lint",
        build_name: "Build path occupancy application",
        admit_name: "Admit complete Git path-set",
        target_dir: "oyatie-occupancy-admission",
        package: "pipeline-path-occupancy-app",
    },
];

fn ensure(ok: bool, message: impl Into<String>) -> Result<(), String> {
    if ok { Ok(()) } else { Err(message.into()) }
}

fn job_body(yaml: &str, spec: JobSpec) -> Result<&str, String> {
    let start = format!("\n  {}:\n", spec.id);
    let end = format!("\n  {}:\n", spec.next_id);
    let rest = yaml
        .split_once(&start)
        .map(|(_, rest)| rest)
        .ok_or_else(|| format!("missing {} job", spec.id))?;
    rest.split_once(&end)
        .map(|(body, _)| body)
        .ok_or_else(|| format!("{} must precede {}", spec.id, spec.next_id))
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

fn content_lines(block: &str, indent: usize) -> Vec<String> {
    block
        .lines()
        .filter(|line| indentation(line) >= indent)
        .map(|line| line[indent..].to_owned())
        .collect()
}

fn validate_checkout(
    step: &str,
    label: &str,
    reference: &str,
    path: &str,
    fetch_depth: &str,
) -> Result<(), String> {
    ensure(
        direct_entries(step, 8) == [("uses", CHECKOUT), ("with", "")],
        format!("{label} must have one pinned checkout action and one with block"),
    )?;
    ensure(
        direct_entries(step, 10)
            == [
                ("ref", reference),
                ("path", path),
                ("persist-credentials", "false"),
                ("fetch-depth", fetch_depth),
            ],
        format!("{label} checkout inputs must be exact and unique"),
    )
}

fn validate_toolchain(step: &str, job: &str) -> Result<(), String> {
    ensure(
        direct_entries(step, 8) == [("with", "")]
            && direct_entries(step, 10) == [("toolchain", "\"1.98.0\"")],
        format!("{job} toolchain step must be pinned and closed"),
    )
}

fn validate_build(step: &str, spec: JobSpec) -> Result<(), String> {
    ensure(
        direct_entries(step, 8) == [("working-directory", "${{ runner.temp }}"), ("run", ">-")],
        format!("{} build step fields must be closed", spec.id),
    )?;
    let actual = content_lines(step, 10);
    let expected = vec![
        "cargo build --locked".to_owned(),
        "--manifest-path \"$GITHUB_WORKSPACE/trusted/Cargo.toml\"".to_owned(),
        "--target x86_64-unknown-linux-gnu".to_owned(),
        format!("--target-dir \"$RUNNER_TEMP/{}\"", spec.target_dir),
        format!("-p {}", spec.package),
    ];
    ensure(
        actual == expected,
        format!("{} must build only the protected source", spec.id),
    )
}

fn validate_admit(step: &str, spec: JobSpec) -> Result<(), String> {
    if spec.id == "layout" {
        ensure(
            direct_entries(step, 8) == [("working-directory", "candidate"), ("run", "|")],
            "layout admission fields must be closed",
        )?;
        let expected = [
            "set -euo pipefail",
            "base_sha=\"$(git rev-parse --verify 'HEAD^1^{commit}')\"",
            "head_sha=\"$(git rev-parse --verify 'HEAD^{commit}')\"",
            "\"$RUNNER_TEMP/oyatie-layout-admission/x86_64-unknown-linux-gnu/debug/pipeline-path-layout-app\" \"$base_sha\" \"$head_sha\"",
        ];
        return ensure(
            content_lines(step, 10) == expected,
            "layout admission must execute the protected binary over candidate HEAD^1 and HEAD",
        );
    }
    ensure(
        direct_entries(step, 8)
            == [
                ("working-directory", "candidate"),
                ("env", ""),
                (
                    "run",
                    "\"$RUNNER_TEMP/oyatie-occupancy-admission/x86_64-unknown-linux-gnu/debug/pipeline-path-occupancy-app\"",
                ),
            ],
        "occupancy admission fields must be closed",
    )?;
    ensure(
        direct_entries(step, 10)
            == [
                ("GH_TOKEN", "${{ github.token }}"),
                (
                    "OYATIE_PULL_REQUEST",
                    "${{ github.event.pull_request.number }}",
                ),
                ("OYATIE_REPOSITORY", "${{ github.repository }}"),
            ],
        "occupancy admission environment must be exact",
    )
}

fn validate_job(yaml: &str, spec: JobSpec) -> Result<(), String> {
    let body = job_body(yaml, spec)?;
    let steps: Vec<_> = body.split("\n      - ").skip(1).collect();
    ensure(
        steps.len() == 5,
        format!("{} must have exactly five steps", spec.id),
    )?;
    let actual_headers: Vec<_> = steps
        .iter()
        .map(|step| step.lines().next().unwrap_or_default().to_owned())
        .collect();
    let expected_headers = [
        "name: Check out candidate tree".to_owned(),
        "name: Check out protected admission source".to_owned(),
        format!("uses: {TOOLCHAIN}"),
        format!("name: {}", spec.build_name),
        format!("name: {}", spec.admit_name),
    ];
    ensure(
        actual_headers == expected_headers,
        format!("{} step occupants or order drifted", spec.id),
    )?;
    validate_checkout(steps[0], "candidate", "${{ github.sha }}", "candidate", "0")?;
    validate_checkout(
        steps[1],
        "protected policy",
        "${{ github.workflow_sha }}",
        "trusted",
        "1",
    )?;
    validate_toolchain(steps[2], spec.id)?;
    validate_build(steps[3], spec)?;
    validate_admit(steps[4], spec)
}

fn validate_workflow(yaml: &str) -> Result<(), String> {
    for spec in JOBS {
        validate_job(yaml, spec)?;
    }
    Ok(())
}

#[test]
fn protected_admission_jobs_have_closed_step_graphs() {
    validate_workflow(&workflow()).unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn protected_admission_graph_rejects_selector_and_overwrite_mutations() {
    let yaml = workflow();
    let protected_ref = "          ref: ${{ github.workflow_sha }}";
    let decoy = yaml.replacen(
        protected_ref,
        "          # ref: ${{ github.workflow_sha }}\n          ref: ${{ github.event.pull_request.merge_commit_sha }}",
        1,
    );
    assert!(validate_workflow(&decoy).is_err());

    let duplicate = yaml.replacen(
        protected_ref,
        "          ref: ${{ github.workflow_sha }}\n          ref: ${{ github.sha }}",
        1,
    );
    assert!(validate_workflow(&duplicate).is_err());

    for wrong in [
        "${{ github.event.pull_request.base.sha }}",
        "${{ github.event.merge_group.base_sha }}",
        "refs/heads/dev",
        "${{ github.sha }}",
        "${{ inputs.policy_sha }}",
    ] {
        let selector = yaml.replacen(protected_ref, &format!("          ref: {wrong}"), 1);
        assert!(validate_workflow(&selector).is_err(), "{wrong}");
    }

    let build = "      - name: Build protected path-layout application";
    let overwrite = yaml.replacen(
        build,
        "      - name: Overwrite protected source\n        uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1\n        with:\n          ref: ${{ github.sha }}\n          path: trusted\n          persist-credentials: false\n          fetch-depth: 1\n      - name: Build protected path-layout application",
        1,
    );
    assert!(validate_workflow(&overwrite).is_err());

    let run = yaml.replacen(
        build,
        "      - name: Mutate protected source\n        run: cp -R candidate/. trusted/\n      - name: Build protected path-layout application",
        1,
    );
    assert!(validate_workflow(&run).is_err());

    let protected_binary = "          \"$RUNNER_TEMP/oyatie-layout-admission/x86_64-unknown-linux-gnu/debug/pipeline-path-layout-app\" \"$base_sha\" \"$head_sha\"";
    let candidate_command = yaml.replacen(
        protected_binary,
        "          cargo run --manifest-path candidate/Cargo.toml -p pipeline-path-layout-app -- \"$base_sha\" \"$head_sha\"",
        1,
    );
    assert!(validate_workflow(&candidate_command).is_err());
}
