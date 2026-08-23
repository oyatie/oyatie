//! Cadence graph contract. These tests pin what each phase is *allowed* to
//! run so a YAML edit cannot silently re-introduce duplicate jobs, fake
//! checks (`continue-on-error`), or legacy required names.
//!
//! They read the committed workflow files. They do not execute GHA.

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
    jobs.lines()
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
        .collect()
}

#[test]
fn presubmit_has_no_fake_or_legacy_jobs() {
    let y = read(".github/workflows/presubmit.yml");
    let jobs = job_ids(&y);
    for forbidden in [
        "clippy",
        "oya-ci-required",
        "merge-admission-required",
        "live-postgres-adapters",
        "live-postgres-facades",
    ] {
        assert!(
            !jobs.iter().any(|j| j == forbidden),
            "presubmit must not contain job `{forbidden}` (got {jobs:?})"
        );
    }
    assert!(
        !y.contains("continue-on-error"),
        "continue-on-error is a fake check: the job name goes green while the command fails"
    );
    assert!(
        !y.contains("sccache"),
        "test job must not run rust-cache and sccache together"
    );
    assert!(
        !y.contains("workflow_call:"),
        "presubmit must not be reusable from postsubmit (that re-pays the whole graph)"
    );
    assert!(
        !y.contains("codeql"),
        "CodeQL is not a presubmit/merge check"
    );
}

#[test]
fn presubmit_fan_in_is_one_real_context() {
    let y = read(".github/workflows/presubmit.yml");
    let jobs = job_ids(&y);
    assert_eq!(
        jobs.iter().filter(|j| *j == "presubmit").count(),
        1,
        "exactly one fan-in job named presubmit"
    );
    assert!(
        jobs.iter().any(|j| j == "lint"),
        "fmt --all is the unique whole-tree format proof"
    );
    assert!(jobs.iter().any(|j| j == "test"), "workspace nextest");
    assert!(jobs.iter().any(|j| j == "deny"), "licenses bans sources");
    assert!(jobs.iter().any(|j| j == "pg-gate"), "path gate");
    assert!(
        jobs.iter().any(|j| j == "live-postgres"),
        "one live Postgres job, not adapters+facades"
    );
    assert!(
        y.contains("needs: [lint, test, deny, pg-gate, live-postgres]"),
        "fan-in needs must match pipeline_admission::fan_in_ok"
    );
}

#[test]
fn presubmit_path_gates_pull_request_and_merge_group() {
    let y = read(".github/workflows/presubmit.yml");
    assert!(y.contains("pull_request"), "presubmit runs on pull_request");
    assert!(y.contains("merge_group"), "presubmit runs on merge_group");
    assert!(
        y.contains("merge_group.base_sha") || y.contains("github.event.merge_group.base_sha"),
        "merge_group must path-gate against merge_group.base_sha, not force live=true"
    );
    assert!(
        y.contains("github.event.pull_request.base.sha"),
        "pull_request path-gate uses PR base sha"
    );
}

#[test]
fn live_postgres_runs_ignored_live_tests_only() {
    let y = read(".github/workflows/live-postgres.yml");
    assert!(
        y.contains("--run-ignored only"),
        "live job must run #[ignore] live_* tests; default profile must not"
    );
    assert!(
        y.contains("--no-tests=error"),
        "zero matches must be red, not a silent empty pass"
    );
    assert!(
        y.contains("--profile live"),
        "live profile default-filter is test(/^live_/)"
    );
    for crate_name in [
        "tenancy-tenant-lifecycle-store-postgres",
        "identity-scim-store-postgres",
        "iam-identity-service",
        "tenancy-tenant-lifecycle-app",
    ] {
        assert!(
            y.contains(crate_name),
            "live job must name crate {crate_name}"
        );
    }
    assert!(
        y.contains("OYATIE_BACKBONE_LIVE_POSTGRES"),
        "live job must set the enable env"
    );
    assert!(
        !y.contains("continue-on-error"),
        "live postgres must not be advisory"
    );
}

#[test]
fn postsubmit_does_not_replay_presubmit() {
    let y = read(".github/workflows/postsubmit.yml");
    assert!(
        !y.contains("uses: ./.github/workflows/presubmit.yml"),
        "postsubmit must not re-enter the presubmit graph"
    );
    assert!(!y.contains("cargo fmt"), "fmt already proved on the PR");
    assert!(!y.contains("clippy"), "clippy is not a postsubmit job");
    assert!(
        !y.contains("licenses bans sources"),
        "deny licenses/bans already proved on the PR"
    );
    assert!(
        !y.contains("oya-ci-required") && !y.contains("merge-admission-required"),
        "legacy names are not posted on trunk"
    );
    assert!(!y.contains("codeql"), "CodeQL is not a postsubmit check");
    assert!(
        y.contains("cargo nextest run --locked --workspace --profile ci"),
        "postsubmit unique proof: workspace nextest on the merged SHA"
    );
    assert!(
        y.contains("live-postgres.yml") || y.contains("live-postgres:"),
        "postsubmit unique proof: always-on live Postgres"
    );
}

#[test]
fn nextest_live_profile_filters_live_tests() {
    let t = read(".config/nextest.toml");
    assert!(t.contains("[profile.live]"), "profile.live must exist");
    assert!(
        t.contains("test(/^live_/)"),
        "profile.live default-filter must be anchored live_ tests"
    );
}

#[test]
fn in_repo_workflow_set_is_closed_and_has_no_codeql() {
    let dir = repo_root().join(".github/workflows");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|e| {
            let n = e.ok()?.file_name().into_string().ok()?;
            n.ends_with(".yml").then_some(n)
        })
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "buck2-weekly-smoke.yml",
            "license-weekly-advisory.yml",
            "live-postgres.yml",
            "nightly.yml",
            "postsubmit.yml",
            "presubmit.yml",
            "promotion-predecessor.yml",
        ]
    );
    for name in &names {
        let body = read(&format!(".github/workflows/{name}"));
        assert!(
            !body.to_ascii_lowercase().contains("codeql"),
            "{name} must not run CodeQL (not a merge proof; default-setup stays off)"
        );
    }
}

#[test]
fn weekly_advisories_are_not_a_license_rerun() {
    let y = read(".github/workflows/license-weekly-advisory.yml");
    assert!(
        y.contains("command-arguments: advisories"),
        "weekly is advisories"
    );
    assert!(
        !y.contains("command-arguments: licenses"),
        "licenses already run on presubmit"
    );
    assert!(
        !y.contains("command-arguments: bans"),
        "bans already run on presubmit"
    );
}

#[test]
fn agents_md_does_not_hand_git_hooks_to_beads() {
    let agents = read("AGENTS.md");
    assert!(
        !agents.contains("git config core.hooksPath"),
        "git runs .git/hooks; do not redirect the hook dir"
    );
    assert!(
        !agents.contains(".beads/hooks"),
        "beads must not own git hooks"
    );
    assert!(
        agents.contains("git-common-dir)/hooks/"),
        "pipeline hooks install into native .git/hooks via git-common-dir"
    );
}

#[test]
fn hooks_are_fmt_only() {
    let pre_commit = read(".githooks/pre-commit");
    let pre_push = read(".githooks/pre-push");
    for (name, body) in [
        ("pre-commit", pre_commit.as_str()),
        ("pre-push", pre_push.as_str()),
    ] {
        assert!(body.contains("rustfmt --check"), "{name} runs rustfmt");
        assert!(
            !body.contains("cargo fmt"),
            "{name} must not call cargo fmt (that formats the whole workspace)"
        );
        assert!(
            !body.contains("cargo nextest"),
            "{name} must not run nextest"
        );
        assert!(!body.contains("cargo clippy"), "{name} must not run clippy");
        assert!(
            !body.contains("cargo test"),
            "{name} must not run cargo test"
        );
    }
}
