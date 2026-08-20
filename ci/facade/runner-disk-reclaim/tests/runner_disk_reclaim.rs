//! Live-corpus conformance for the FRIC-017 runner disk-reclaim preflight (ADR-0548
//! pipeline-as-product). Reads the COMMITTED policy and asserts: the seeded
//! `github-hosted-ubuntu-latest` profile carries exactly the 5 vendor preinstall dirs that the
//! retired inline `sudo rm -rf` blocks removed, a positive free-disk floor, and that the
//! threshold/INFRA-RED predicate DISCRIMINATES (below-floor ⇒ INFRA-RED; at/above-floor ⇒ ok).

use ci_runner_disk_reclaim::{GIB, POLICY_REL_PATH, ReclaimReport, parse_profile, repo_root_from};

/// Locate the committed policy by walking up to the repo root (works under both buck2 — cwd =
/// project root — and cargo — cwd = crate dir — without `CARGO_MANIFEST_DIR`).
fn policy_text() -> String {
    let cwd = std::env::current_dir().expect("current_dir");
    let root = repo_root_from(&cwd)
        .unwrap_or_else(|| panic!("failed to locate repo root from {}", cwd.display()));
    let path = root.join(POLICY_REL_PATH);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read committed policy {}: {e}", path.display()))
}

#[test]
fn seeded_profile_carries_the_five_retired_inline_shell_dirs() {
    let profile = parse_profile(&policy_text(), "github-hosted-ubuntu-latest")
        .expect("seeded profile parses");
    assert_eq!(
        profile.reclaim_dirs,
        vec![
            "/usr/share/dotnet".to_owned(),
            "/usr/local/lib/android".to_owned(),
            "/opt/ghc".to_owned(),
            "/usr/local/.ghcup".to_owned(),
            "/opt/hostedtoolcache/CodeQL".to_owned(),
        ],
        "the seeded profile must carry exactly the 5 dirs the retired inline `rm -rf` removed"
    );
    assert!(
        profile.min_free_gib_after > 0,
        "the post-reclaim free-disk floor must be a positive GiB count"
    );
}

#[test]
fn unknown_profile_is_fail_loud() {
    assert!(parse_profile(&policy_text(), "no-such-profile").is_err());
}

#[test]
fn threshold_predicate_discriminates_infra_red_from_ok() {
    let profile = parse_profile(&policy_text(), "github-hosted-ubuntu-latest")
        .expect("seeded profile parses");
    let floor = profile.min_free_gib_after;

    let below = ReclaimReport {
        free_before: 5 * GIB,
        free_after: (floor.saturating_sub(1)) * GIB,
        outcomes: vec![],
        min_free_gib_after: floor,
    };
    assert!(below.is_infra_red(), "below the floor must be INFRA-RED");

    let at = ReclaimReport {
        free_before: 5 * GIB,
        free_after: floor * GIB,
        outcomes: vec![],
        min_free_gib_after: floor,
    };
    assert!(!at.is_infra_red(), "at the floor must NOT be INFRA-RED");
}
