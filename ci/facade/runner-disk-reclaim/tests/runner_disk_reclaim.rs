//! Live-corpus conformance for the FRIC-017 runner disk-reclaim preflight (ADR-0548
//! pipeline-as-product). Reads the COMMITTED policy and asserts: the seeded
//! `github-hosted-ubuntu-latest` profile carries exactly the 5 vendor preinstall dirs that the
//! retired inline `sudo rm -rf` blocks removed, a positive free-disk floor, and that the
//! threshold/INFRA-RED predicate DISCRIMINATES (below-floor ⇒ INFRA-RED; at/above-floor ⇒ ok).

use ci_runner_disk_reclaim::{
    GIB, POLICY_REL_PATH, ReclaimReport, parse_profile, repo_root_from,
};

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

/// Measured 2026-07-29 on the owned Talos fleet: `.status.allocatable.ephemeral-storage` is
/// 45909593217 B on every node, and the `oya-arm64` AutoscalingRunnerSet declares
/// `maxRunners: 3` with no pod anti-affinity — so the worst case is all three runners on one
/// node and the per-runner ephemeral-storage budget is allocatable / maxRunners.
const OWNED_ARC_NODE_ALLOCATABLE_EPHEMERAL_BYTES: u64 = 45_909_593_217;
const OWNED_ARC_MAX_RUNNERS: u64 = 3;

/// ADR-0515 D5 owned-runner readiness: the container-runner profile must exist (an absent
/// profile is fail-loud, so the FIRST job routed to the owned fleet would INFRA-RED on a
/// configuration defect), must reclaim nothing (the runner image has no GitHub vendor
/// preinstall trees), and must carry a floor the fleet can actually honor. The floor bound is
/// what makes this gate RED-capable: copying the github-hosted 20 GiB number fails here.
#[test]
fn owned_arc_profile_floor_fits_the_measured_per_runner_ephemeral_budget() {
    let profile =
        parse_profile(&policy_text(), "owned-arc-arm64").expect("owned ARC profile must exist");

    assert!(
        profile.reclaim_dirs.is_empty(),
        "a container runner has no GitHub vendor preinstall trees to reclaim; a non-empty \
         list here is a copied-from-github-hosted no-op that lies about what ran: {:?}",
        profile.reclaim_dirs
    );

    let budget_gib = OWNED_ARC_NODE_ALLOCATABLE_EPHEMERAL_BYTES / OWNED_ARC_MAX_RUNNERS / GIB;
    assert!(
        profile.min_free_gib_after > 0,
        "a zero floor is a non-assertion, not a preflight"
    );
    assert!(
        profile.min_free_gib_after <= budget_gib,
        "owned-arc floor {} GiB exceeds the measured per-runner ephemeral-storage budget \
         {budget_gib} GiB (node allocatable {OWNED_ARC_NODE_ALLOCATABLE_EPHEMERAL_BYTES} B / \
         maxRunners {OWNED_ARC_MAX_RUNNERS}) — every job on the owned fleet would INFRA-RED",
        profile.min_free_gib_after
    );

    // The floor must still DISCRIMINATE, or it is an inert declaration.
    let floor = profile.min_free_gib_after;
    let below = ReclaimReport {
        free_before: floor * GIB,
        free_after: (floor - 1) * GIB,
        outcomes: vec![],
        min_free_gib_after: floor,
    };
    assert!(below.is_infra_red(), "below the owned floor must be INFRA-RED");
    let at = ReclaimReport {
        free_before: floor * GIB,
        free_after: floor * GIB,
        outcomes: vec![],
        min_free_gib_after: floor,
    };
    assert!(!at.is_infra_red(), "at the owned floor must NOT be INFRA-RED");
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
