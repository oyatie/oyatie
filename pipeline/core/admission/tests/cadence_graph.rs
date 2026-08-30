//! Occupants of each cadence file. Equality on the set, not a denylist of
//! last week's extra job.

use pipeline_admission::{LIVE_POSTGRES_CRATES, POSTSUBMIT_JOBS, PRESUBMIT_JOBS, WORKFLOW_FILES};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn job_ids(yaml: &str) -> Vec<String> {
    let jobs = yaml
        .split_once("\njobs:\n")
        .map(|(_, rest)| rest)
        .unwrap_or("");
    let mut ids: Vec<String> = jobs
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("  ")?;
            if rest.starts_with(' ') || rest.starts_with('#') || rest.starts_with('-') {
                return None;
            }
            let id = rest.strip_suffix(':')?;
            if id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                Some(id.to_owned())
            } else {
                None
            }
        })
        .collect();
    ids.sort();
    ids
}

#[test]
fn presubmit_jobs_are_the_occupant_set() {
    let y = read(".github/workflows/presubmit.yml");
    assert_eq!(job_ids(&y), PRESUBMIT_JOBS);
    assert!(
        !y.contains("workflow_dispatch:"),
        "the required workflow runs only for PR and merge-group admission"
    );
    assert!(!y.contains("pull_request_target:"));
    assert!(
        !y.contains("concurrency:"),
        "ruleset-required workflows must not cancel or supersede an admission run"
    );
    assert!(
        y.contains("needs: [layout, occupancy, lint, clippy, test, deny, pg-gate, live-postgres]")
    );
    assert!(y.contains("needs: [layout, occupancy]"));
    let protected_source = "ref: ${{ github.workflow_sha }}";
    assert_eq!(
        y.matches(protected_source).count(),
        2,
        "layout and occupancy must compile from the immutable revision of the ruleset-selected workflow"
    );
    assert!(y.contains("ref: ${{ github.sha }}"));
    assert!(y.contains("git rev-parse --verify 'HEAD^1^{commit}'"));
    assert!(!y.contains("github.event.pull_request.head.sha"));
    assert!(!y.contains("github.event.merge_group.head_sha"));
    assert!(y.contains("pipeline-path-occupancy-app"));
    assert!(y.contains("pipeline-path-layout-app"));
    assert!(y.contains("path: candidate"));
    assert!(y.contains("path: trusted"));
    assert!(y.contains("name: Check out protected admission source"));
    assert!(y.contains("cargo build --locked"));
    assert_eq!(
        y.matches("working-directory: ${{ runner.temp }}").count(),
        3,
        "trusted admission builds and rustfmt must ignore candidate Cargo configuration"
    );
    assert!(y.contains("--manifest-path \"$GITHUB_WORKSPACE/trusted/Cargo.toml\""));
    assert!(y.contains("--manifest-path \"$GITHUB_WORKSPACE/Cargo.toml\""));
    assert!(y.contains("--target x86_64-unknown-linux-gnu"));
    assert!(y.contains("--target-dir \"$RUNNER_TEMP/oyatie-layout-admission\""));
    assert!(y.contains("--target-dir \"$RUNNER_TEMP/oyatie-occupancy-admission\""));
    let protected_builds = y
        .split("name: Build protected path-layout application")
        .nth(1)
        .and_then(|rest| rest.split("  lint:").next())
        .expect("protected admission jobs");
    assert!(
        !protected_builds.contains("rust-cache"),
        "protected admission must not restore candidate-writable caches"
    );
    assert!(y.contains("debug/pipeline-path-layout-app\""));
    assert!(y.contains("debug/pipeline-path-occupancy-app\""));
    assert!(!y.contains("cargo run -p pipeline-path-layout-app"));
    assert!(!y.contains("cargo run -p pipeline-path-occupancy-app"));
    assert!(y.contains("cargo clippy --locked --workspace --all-targets"));
    assert!(y.contains("-- -D warnings"));
    assert!(y.contains("req \"${{ needs.layout.result }}\""));
    assert!(y.contains("req \"${{ needs.clippy.result }}\""));
    assert!(y.contains("uses: oyatie/oyatie/.github/workflows/live-postgres.yml@dev"));
    assert!(
        !y.contains("uses: ./.github/workflows/live-postgres.yml"),
        "the ruleset-required caller must not resolve reusable workflow code from the candidate"
    );
    assert!(y.contains("cargo-nextest nextest run"));
    assert!(!y.contains("cargo nextest run"));
    assert!(y.contains("name: occupancy (path-set)\n    if: github.event_name == 'pull_request'"));
    assert!(y.contains("occ()"));
    assert!(y.contains("occ \"${{ needs.occupancy.result }}\""));
    assert!(y.contains("OYATIE_PULL_REQUEST: ${{ github.event.pull_request.number }}"));
    assert!(y.contains("OYATIE_REPOSITORY"));
    assert!(
        !y.contains("gh pr diff"),
        "gh pr diff 406s when the unified diff exceeds 20k lines"
    );
    assert!(
        !y.contains("/pulls/${1}/files") && !y.contains("/pulls/${n}/files"),
        "the REST files endpoint silently truncates after 3000 paths"
    );
    assert!(
        !y.contains("--limit 1000"),
        "open pull request enumeration must not have a silent ceiling"
    );
    assert!(
        !y.contains("openssl base64")
            && !y.contains("Collect path-sets")
            && !y.contains("paths_for_pr()"),
        "correctness-critical collection belongs in Rust"
    );
    assert!(
        !y.contains("pipeline-path-occupancy-app --locked --offline"),
        "the admission build must reach crates.io on a cold cache"
    );

    // The collector spans two files: `main.rs` decides, `git.rs` reads. Freeze
    // both, as the layout app below does — reading only `main.rs` would have
    // let the git plumbing move out from under these assertions and report
    // green, which is the failure mode this freeze exists to prevent.
    let collector = format!(
        "{}\n{}",
        read("pipeline/facade/path-occupancy-app/src/main.rs"),
        read("pipeline/facade/path-occupancy-app/src/git.rs")
    );
    assert!(collector.contains("Command::new(\"gh\")"));
    assert!(collector.contains("--paginate"));
    assert!(collector.contains("refs/pull/{number}/head"));
    assert!(collector.contains("--name-status"));
    assert!(collector.contains("\"-z\""));
    assert!(collector.contains("\"-M\""));
    assert!(collector.contains("x-access-token"));
    assert!(collector.contains("git_change_paths_from_name_status_z"));
    assert!(collector.contains("required_env(\"GITHUB_REF\")"));
    assert!(!collector.contains("/files"));
    // Bind the call site to the authored-path rule. Without these, reverting
    // `run()` to a raw `admit()` over every changed path leaves every unit test
    // green while restoring the wedge they exist to prevent.
    assert!(
        collector.contains("declared_mergeable(&attributes)")
            && collector.contains("admit_authored(&this, &in_flight, &mergeable)"),
        "occupancy must admit on authored paths, not on every changed path"
    );
    assert!(
        !collector.contains("admit(&this, &in_flight)"),
        "the raw all-paths admit is the wedge; it must not return"
    );

    let layout = format!(
        "{}\n{}",
        read("pipeline/facade/path-layout-app/src/main.rs"),
        read("pipeline/facade/path-layout-app/src/repository_checks.rs")
    );
    assert!(layout.contains("changed_layout_violations"));
    assert!(layout.contains("git_change_paths_from_name_status_z"));
    assert!(layout.contains("BUILD_ROOT_DIRS"));
    assert!(layout.contains("APP_PRODUCT_DIRS"));
    assert!(layout.contains("cargo_manifest_violations"));
    assert!(layout.contains("touched_manifests"));
    assert!(layout.contains("cargo_manifest_for_crate_path"));
    assert!(layout.contains("directory_exists(&head"));
    assert!(layout.contains("draft_dependency_violations"));
    assert!(layout.contains("workspace_draft_dependency_violations"));
    assert!(layout.contains("workspace_membership_violations"));
    assert!(layout.contains("repository_cargo_config_violations"));
    assert!(layout.contains("live_candidate_violations"));
    assert!(layout.contains("file_budget_violations"));
    assert!(layout.contains("owner_core_regression_violations"));
    assert!(layout.contains("entry_kind(&head"));
    assert!(layout.contains("RepositoryRead"));
    assert!(layout.contains("GitRepository"));

    let repository_port = read("pipeline/ports/draft/repository/src/lib.rs");
    assert!(repository_port.contains("pub trait RepositoryRead"));
    assert!(repository_port.contains("changed_name_status"));
    assert!(repository_port.contains("blob_text"));
    assert!(repository_port.contains("blob_bytes"));
    assert!(repository_port.contains("files_under"));

    let git_adapter = read("pipeline/adapters/draft/repository-git/src/lib.rs");
    assert!(git_adapter.contains("merge-base"));
    assert!(git_adapter.contains("--name-status"));
    assert!(git_adapter.contains("\"-z\""));
    assert!(git_adapter.contains("\"-M\""));
    assert!(git_adapter.contains("cat-file"));
    assert!(git_adapter.contains("ls-tree"));
}

#[test]
fn postsubmit_jobs_are_the_occupant_set() {
    let y = read(".github/workflows/postsubmit.yml");
    assert_eq!(job_ids(&y), POSTSUBMIT_JOBS);
    assert!(y.contains("cargo nextest run --locked --workspace --profile ci"));
    assert!(y.contains("live-postgres.yml"));
}

#[test]
fn workflow_dir_is_the_occupant_set() {
    let dir = repo_root().join(".github/workflows");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|e| {
            let n = e.ok()?.file_name().into_string().ok()?;
            n.ends_with(".yml").then_some(n)
        })
        .collect();
    names.sort();
    assert_eq!(names, WORKFLOW_FILES);
}

#[test]
fn live_postgres_job_is_filtered_nextest() {
    let y = read(".github/workflows/live-postgres.yml");
    assert!(y.contains("--run-ignored only"));
    assert!(y.contains("--no-tests=error"));
    assert!(y.contains("--profile live"));
    for crate_name in LIVE_POSTGRES_CRATES {
        assert!(y.contains(crate_name), "{crate_name}");
    }
}

#[test]
fn nextest_live_profile_occupies_live_filter() {
    let t = read(".config/nextest.toml");
    assert!(t.contains("[profile.live]"));
    assert!(t.contains("test(/^live_/)"));
}

#[test]
fn weekly_deny_occupies_advisories() {
    let y = read(".github/workflows/license-weekly-advisory.yml");
    assert!(y.contains("command-arguments: advisories"));
}

#[test]
fn hooks_occupy_rustfmt_on_the_file_list() {
    for rel in [".githooks/pre-commit", ".githooks/pre-push"] {
        let body = read(rel);
        assert!(body.contains("rustfmt --check"), "{rel}");
        assert!(body.contains("xargs -0 rustfmt"), "{rel}");
    }
}

#[test]
fn agents_md_installs_hooks_in_git_common_dir() {
    let agents = read("AGENTS.md");
    assert!(agents.contains("git-common-dir)/hooks/"));
    assert!(agents.contains("ADR.md"));
    assert!(agents.contains("PRD.md"));
    assert!(agents.contains("SPEC.md"));
    assert!(agents.contains("PLAN.md"));
}
