//! Live-corpus enforcement for the ADR-0711 Phase B envelope gate.
//!
//! Two things run here against the real tree:
//!
//!   1. the envelope authority itself is loaded and structurally validated —
//!      a malformed adjunct claim or a waiver naming a non-hub path is a defect
//!      in the law today, whether or not any PR is in flight;
//!   2. the resolver is exercised over every registered envelope glob, so
//!      longest-match-wins is proven on the real 73-root authority rather than
//!      only on fixtures.
//!
//! The per-PR check (changed paths vs envelope) needs the PR context and lives in
//! the workflow step; this file is what makes the crate a live gate rather than a
//! fixture-only one.

#![allow(clippy::expect_used, clippy::panic)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use check_integ_envelope::{
    AdjunctClaim, Authority, HubWaiver, Owner, evaluate, owner_of, validate_authority,
};

fn repo_root() -> PathBuf {
    let mut directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if directory.join("specs/root-hub-pointers.json").is_file() {
            return directory;
        }
        assert!(
            directory.pop(),
            "repository root not found above the manifest dir"
        );
    }
}

const ENVELOPES: &str = "specs/integ-branch-envelopes.json";
const WAIVER_DIR: &str = "governance/check/integ-envelope/waivers";

/// Parse the live authority, plus the per-claim missing-field map the structural
/// validator needs.
fn live_authority(root: &Path) -> (Authority, BTreeMap<String, Vec<String>>) {
    let text = fs::read_to_string(root.join(ENVELOPES))
        .unwrap_or_else(|error| panic!("read {ENVELOPES}: {error}"));
    let doc: serde_json::Value =
        serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {ENVELOPES}: {error}"));

    let mut owners = Vec::new();
    for group in ["roots", "planes"] {
        let Some(map) = doc[group].as_object() else {
            continue;
        };
        for (_name, entry) in map {
            let Some(branch) = entry["branch"].as_str() else {
                continue;
            };
            let globs: Vec<String> = entry["envelope_globs"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|g| g.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default();
            owners.push(Owner {
                branch: branch.to_owned(),
                globs,
            });
        }
    }

    let hub_paths: BTreeSet<String> = doc["hubs"]["paths"]
        .as_array()
        .expect("hubs.paths must be an array")
        .iter()
        .filter_map(|p| p.as_str().map(str::to_owned))
        .collect();

    let required: Vec<String> = doc["adjunct_claims"]["required_fields"]
        .as_array()
        .expect("adjunct_claims.required_fields")
        .iter()
        .filter_map(|f| f.as_str().map(str::to_owned))
        .collect();

    let mut adjunct_claims = Vec::new();
    let mut missing_fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for claim in doc["adjunct_claims"]["active"]
        .as_array()
        .expect("adjunct_claims.active")
    {
        let path_glob = claim["path_glob"]
            .as_str()
            .unwrap_or("<no path_glob>")
            .to_owned();
        let missing: Vec<String> = required
            .iter()
            .filter(|f| {
                claim
                    .get(f.as_str())
                    .and_then(|v| v.as_str())
                    .is_none_or(|s| s.trim().is_empty())
            })
            .cloned()
            .collect();
        if !missing.is_empty() {
            missing_fields.insert(path_glob.clone(), missing);
        }
        if let Some(branch) = claim["claiming_branch"].as_str() {
            adjunct_claims.push(AdjunctClaim {
                path_glob,
                claiming_branch: branch.to_owned(),
            });
        }
    }

    // Waivers are one YAML file each, with a flat `branch:` / `hub:` shape. A tiny
    // reader beats a YAML dependency for a kernel-tier crate, and it fails loudly
    // rather than silently skipping a file it cannot understand.
    let mut hub_waivers = BTreeSet::new();
    let dir = root.join(WAIVER_DIR);
    assert!(
        dir.is_dir(),
        "{WAIVER_DIR} must exist — it is the declared home of hub waivers"
    );
    for entry in fs::read_dir(&dir).expect("read waiver dir") {
        let path = entry.expect("waiver entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read waiver");
        let field = |key: &str| -> Option<String> {
            text.lines()
                .find_map(|l| l.trim().strip_prefix(&format!("{key}:")))
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
        };
        let (branch, hub) = (field("branch"), field("hub"));
        assert!(
            branch.is_some() && hub.is_some(),
            "waiver {} must declare both `branch:` and `hub:`",
            path.display()
        );
        assert!(
            field("expires_at_or_wave").is_some(),
            "waiver {} must declare `expires_at_or_wave` — an unexpiring waiver is a permanent \
             unaudited exemption",
            path.display()
        );
        hub_waivers.insert(HubWaiver {
            branch: branch.expect("branch"),
            hub: hub.expect("hub"),
        });
    }

    (
        Authority {
            owners,
            hub_paths,
            adjunct_claims,
            hub_waivers,
        },
        missing_fields,
    )
}

/// Adjunct claims that overlap another rail's envelope, frozen shrink-only.
///
/// Every one of these gives `integ/specs` a path another rail already owns. Each
/// side is individually admissible — the claimant by its claim, the owner by its
/// envelope — so no per-PR check refuses either, and the conflict only surfaces
/// as a deadlock when both try to land. The live example that motivated the
/// check: root `Cargo.toml` is claimed by `integ/specs` while the
/// `root_manifests` plane names `integ/build` its sole writer, so integ/build
/// cannot perform a root-manifest edit while that claim stands.
///
/// These are pre-existing and are tolerated so the gate can land. Releasing a
/// claim REMOVES its entry; nothing may be added. The count is pinned so a new
/// overlap cannot be absorbed silently alongside a release.
const BASELINED_CLAIM_OVERLAPS: [&str; 17] = [
    ".claude/workflows/**",
    "Cargo.toml",
    ".grok/harness/model-routing.v1.json",
    ".grok/programs/delivery-fabric/evidence/PORTABLE-SWARM-CONTRACT.md",
    "ci/facade/action-item-accounting/friction-accounting-baseline.json",
    "ci/facade/action-item-accounting/friction-accounting-policy.json",
    "ci/facade/action-item-accounting/friction-ledger.jsonl",
    "ci/facade/contract-slice-conformance/**",
    "ci/facade/lifecycle-status/**",
    "docs/decisions/ADR-0711-swarm-delivery-law-integ-branch-topology.md",
    "docs/machine-readable/decisions.json",
    "flags/release/runtime-safety-policy.json",
    "governance/check/integ-envelope/OWNERS",
    "governance/check/integ-envelope/judgments/**",
    "governance/check/integ-envelope/waivers/**",
    "libs/oya-governance-lifecycle-kernel/src/lib.rs",
    "registry/vcs/concurrent-safe-paths.yaml",
];

#[test]
fn the_live_envelope_authority_is_structurally_valid() {
    let root = repo_root();
    let (authority, missing) = live_authority(&root);

    // Anti-vacuity: a collapsed parse would validate an empty authority and pass.
    assert!(
        authority.owners.len() >= 60,
        "expected the full root+plane authority, got {} owners — the parse collapsed",
        authority.owners.len()
    );
    assert_eq!(
        authority.hub_paths.len(),
        8,
        "ADR-0711 declares exactly eight hub paths; got {:?}",
        authority.hub_paths
    );

    let all = validate_authority(&authority, &missing);

    // Split the frozen pre-existing overlaps from everything else. A NEW overlap,
    // or any other structural defect, is born-blocking.
    let frozen: BTreeSet<&str> = BASELINED_CLAIM_OVERLAPS.into_iter().collect();
    let observed_overlaps: BTreeSet<String> = all
        .iter()
        .filter(|f| f.code == check_integ_envelope::CODE_FOREIGN_ENVELOPE)
        .map(|f| f.subject.clone())
        .collect();
    let new_overlaps: Vec<&String> = observed_overlaps
        .iter()
        .filter(|s| !frozen.contains(s.as_str()))
        .collect();
    assert!(
        new_overlaps.is_empty(),
        "NEW adjunct-claim overlap(s), which create two writers for one path: {new_overlaps:?}"
    );
    let released: Vec<&&str> = frozen
        .iter()
        .filter(|s| !observed_overlaps.contains(**s))
        .collect();
    assert!(
        released.is_empty(),
        "these claim overlaps are gone — remove them from BASELINED_CLAIM_OVERLAPS and lower the \
         count in the same change so the win is recorded: {released:?}"
    );
    assert_eq!(
        observed_overlaps.len(),
        BASELINED_CLAIM_OVERLAPS.len(),
        "the reviewed overlap ceiling moved"
    );

    let findings: Vec<_> = all
        .iter()
        .filter(|f| f.code != check_integ_envelope::CODE_FOREIGN_ENVELOPE)
        .cloned()
        .collect();
    assert!(
        findings.is_empty(),
        "the live envelope authority must be structurally valid; got {} finding(s):\n{}",
        findings.len(),
        findings
            .iter()
            .map(|f| format!("  {} {}: {}", f.code, f.subject, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );

    eprintln!(
        "INTEG-ENVELOPE live authority: owners={} hubs={} adjunct_claims={} waivers={}",
        authority.owners.len(),
        authority.hub_paths.len(),
        authority.adjunct_claims.len(),
        authority.hub_waivers.len()
    );
}

/// Longest-match-wins must resolve to exactly one owner for every registered
/// glob. A tie is what the authority itself calls "a defect, not a choice".
#[test]
fn every_registered_envelope_resolves_to_exactly_one_owner() {
    let root = repo_root();
    let (authority, _) = live_authority(&root);

    let mut ties = Vec::new();
    let mut probed = 0usize;
    for owner in &authority.owners {
        for glob in &owner.globs {
            // Probe a concrete path the glob owns. An EXACT-path glob (no `/**`,
            // no trailing `/`) owns only itself, so appending a segment would
            // build a path it correctly does not match — that would be a bug in
            // the probe, not a defect in the authority.
            let probe = if glob.ends_with("/**") || glob.ends_with('/') {
                format!(
                    "{}/__probe__",
                    glob.trim_end_matches("/**").trim_end_matches('/')
                )
            } else {
                (*glob).clone()
            };
            probed += 1;
            match owner_of(&authority, &probe) {
                Err(tied) => ties.push(format!(
                    "`{probe}` is claimed at equal specificity by {:?}",
                    tied.iter().map(|o| &o.branch).collect::<Vec<_>>()
                )),
                Ok(None) => ties.push(format!(
                    "`{probe}` resolves to NO owner although `{}` declares `{glob}`",
                    owner.branch
                )),
                Ok(Some(_)) => {}
            }
        }
    }

    assert!(
        probed >= 60,
        "probed only {probed} globs — the walk collapsed"
    );
    assert!(
        ties.is_empty(),
        "every envelope must resolve to exactly one owner; got {} problem(s):\n  {}",
        ties.len(),
        ties.join("\n  ")
    );
}

/// The gate must stay silent on the branch shapes the repository actually uses
/// today, or it would forbid current practice rather than enforce the law.
#[test]
fn todays_non_integ_heads_are_out_of_scope() {
    let root = repo_root();
    let (authority, _) = live_authority(&root);

    for head in [
        "fix/whatever",
        "chore/whatever",
        "feat/whatever",
        "deps/whatever",
    ] {
        let findings = evaluate(
            &authority,
            head,
            &["Cargo.lock".to_owned(), "specs/masterplan.json".to_owned()],
        );
        assert!(
            findings.is_empty(),
            "`{head}` must be out of scope until Phase C lands branch protection; got {findings:?}"
        );
    }
}
