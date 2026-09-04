//! Live-Postgres tests must not silently skip when the live job enabled them.
//! Default nextest must not claim it ran them.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

const LIVE_CRATE_DIRS: &[&str] = &[
    "compute/adapters/k8s-lifecycle-repository-postgres",
    "tenancy/adapters/tenant-lifecycle-store-postgres",
    "iam/adapters/identity-scim-store-postgres",
    "iam/facade/identity-service",
    "tenancy/facade/tenant-lifecycle-app",
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("entry");
        let path = entry.path();
        let ft = entry.file_type().expect("ft");
        if ft.is_dir() {
            rust_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

struct LiveTest {
    file: PathBuf,
    name: String,
    has_ignore: bool,
    silent_skip: bool,
}

fn parse_live_tests(src: &str, file: &Path) -> Vec<LiveTest> {
    let mut out = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let Some(rest) = trimmed
            .strip_prefix("async fn ")
            .or_else(|| trimmed.strip_prefix("fn "))
        else {
            continue;
        };
        let Some(name) = rest.split('(').next() else {
            continue;
        };
        if !name.starts_with("live_") {
            continue;
        }
        let window = lines[i.saturating_sub(6)..i].join("\n");
        if !window.contains("#[test]") && !window.contains("#[tokio::test") {
            continue;
        }
        let body: String = lines[i..]
            .iter()
            .take(25)
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        let silent_skip = body.contains("eprintln!") && body.contains("return;")
            || body.contains("if !enabled()") && body.contains("return;")
            || body.contains("if !live_enabled()") && body.contains("return;");
        out.push(LiveTest {
            file: file.to_path_buf(),
            name: name.to_owned(),
            has_ignore: window.contains("#[ignore"),
            silent_skip,
        });
    }
    out
}

fn all_live_tests() -> Vec<LiveTest> {
    let root = repo_root();
    let mut files = Vec::new();
    for dir in LIVE_CRATE_DIRS {
        rust_files(&root.join(dir), &mut files);
    }
    let mut tests = Vec::new();
    for file in files {
        let src = std::fs::read_to_string(&file).expect("read rust");
        tests.extend(parse_live_tests(&src, &file));
    }
    tests
}

#[test]
fn live_tests_exist_and_are_named_live_star() {
    let tests = all_live_tests();
    assert!(
        tests.len() >= 10,
        "expected the RLS/durability live tests, got {} ({:?})",
        tests.len(),
        tests.iter().map(|t| t.name.as_str()).collect::<Vec<_>>()
    );
    for t in &tests {
        assert!(
            t.name.starts_with("live_"),
            "{} is not live_* — nextest profile.live would miss it",
            t.name
        );
    }
}

#[test]
fn live_tests_are_ignored_on_the_default_profile() {
    let missing: Vec<String> = all_live_tests()
        .into_iter()
        .filter(|t| !t.has_ignore)
        .map(|t| format!("{}:{}", t.file.display(), t.name))
        .collect();
    assert!(
        missing.is_empty(),
        "live tests must be #[ignore] so workspace nextest does not count a skip as a pass:\n{missing:#?}"
    );
}

#[test]
fn live_tests_do_not_silently_skip_when_enabled() {
    let silent: Vec<String> = all_live_tests()
        .into_iter()
        .filter(|t| t.silent_skip)
        .map(|t| format!("{}:{}", t.file.display(), t.name))
        .collect();
    assert!(
        silent.is_empty(),
        "live tests must fail (not return) when the enable env is missing:\n{silent:#?}"
    );
}

/// After-impl: the fail-closed helpers actually exist (not just the absence of skip).
#[test]
fn live_crates_have_fail_closed_helpers() {
    let root = repo_root();
    let adapters = [
        "compute/adapters/k8s-lifecycle-repository-postgres/tests/support/mod.rs",
        "tenancy/adapters/tenant-lifecycle-store-postgres/tests/live_rls.rs",
        "iam/adapters/identity-scim-store-postgres/tests/live_rls.rs",
    ];
    for rel in adapters {
        let src = std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
        assert!(
            src.contains("fn require_enabled()"),
            "{rel} missing require_enabled"
        );
    }
    let facades = [
        "iam/facade/identity-service/tests/e2e_service.rs",
        "tenancy/facade/tenant-lifecycle-app/tests/acceptance.rs",
    ];
    for rel in facades {
        let src = std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
        assert!(
            src.contains("fn require_live_app_url()"),
            "{rel} missing require_live_app_url"
        );
    }
}

#[test]
fn compute_live_postgres_is_a_separate_database_cell() {
    let workflow = std::fs::read_to_string(repo_root().join(".github/workflows/live-postgres.yml"))
        .expect("live-postgres.yml");
    let (backbone, compute_and_verdict) = workflow
        .split_once("\n  compute-lifecycle-postgres:\n")
        .expect("dedicated Compute live-Postgres job");
    let (compute, verdict) = compute_and_verdict
        .split_once("\n  live-postgres-verdict:\n")
        .expect("live-Postgres cell verdict");

    assert_eq!(workflow.matches("image: postgres:16").count(), 2);
    assert!(backbone.contains("name: backbone live Postgres"));
    assert!(backbone.contains("if: inputs.run_backbone"));
    assert!(!backbone.contains("compute-k8s-lifecycle-repository-postgres"));
    assert!(compute.contains("name: Compute lifecycle live Postgres"));
    assert!(compute.contains("if: inputs.run_compute_lifecycle"));
    assert!(compute.contains("PG_COMPUTE_APP_URL=postgres://compute_app:"));
    assert!(compute.contains(
        "CREATE ROLE compute_app LOGIN PASSWORD %L NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE NOREPLICATION INHERIT"
    ));
    assert!(compute.contains("OYATIE_BACKBONE_POSTGRES_APP_URL: ${{ env.PG_COMPUTE_APP_URL }}"));
    assert!(compute.contains("-p compute-k8s-lifecycle-repository-postgres"));
    assert!(!compute.contains("tenancy_lifecycle_runtime"));
    assert!(!compute.contains("identity_scim_runtime"));
    assert!(verdict.contains("name: live Postgres cell verdict"));
    assert!(verdict.contains("needs: [live-postgres, compute-lifecycle-postgres]"));
    assert!(verdict.contains("requested && cell"));
    assert!(!verdict.contains("POSTGRES_PASSWORD"));
}

/// Adapter live_* tests DROP SCHEMA CASCADE on two shared schemas. nextest
/// profile.live must stay single-threaded or those drops race CREATE TABLE /
/// CREATE POLICY / FORCE RLS (postsubmit live-postgres on a20b2c808).
#[test]
fn live_nextest_profile_is_serialized() {
    let src = std::fs::read_to_string(repo_root().join(".config/nextest.toml"))
        .expect(".config/nextest.toml");
    let live = src
        .split("[profile.live]")
        .nth(1)
        .expect("[profile.live] missing");
    let live = live.split("[profile.").next().unwrap_or(live);
    assert!(
        live.contains("test-threads = 1"),
        "profile.live must set test-threads = 1 so DROP SCHEMA CASCADE live_* tests cannot race"
    );
}
