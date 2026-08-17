// cloud-ci-anti-drift-drift-grep gate (ADR-0711 Amendment D / INV-DOC-2).
//
// 1. Policy pack cites specs/integ-branch-envelopes.json#anti_drift.prose_must_cite_not_enumerate
//    — never re-lists the cite set.
// 2. RED fixture: prose root enumeration / freeze path table MUST Refuse.
// 3. GREEN fixture: cite-only Swarm surface is Green.
// 4. When ADR-0711 + envelopes exist on tip: live-bind cite authority + evaluate in-scope prose.
//    When absent (parked integ/ci before #1644 land): skip live load; fixture proofs still bind.
//
// ADR-0083 Tier-3: integration tests use unwrap/expect/panic to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use ci_affected_target_set::anti_drift_drift_grep::{
    AntiDriftDriftGrepPolicy, CODE_PROSE_FREEZE_PATH_TABLE, CODE_PROSE_ROOT_ENUMERATION, GATE_ID,
    PROSE_MUST_CITE_POINTER, ProseSurface, SCANNED_SURFACES_POINTER, Verdict, evaluate,
    fixture_prose_must_cite, prose_must_cite_from_envelopes,
};
use serde_json::Value;

const POLICY_PATH: &str = "ci/facade/affected-target-set/anti-drift-drift-grep-policy.json";
const ENVELOPES_PATH: &str = "specs/integ-branch-envelopes.json";
const ADR_PATH: &str = "docs/decisions/ADR-0711-swarm-delivery-law-integ-branch-topology.md";
const PORTABLE_PATH: &str = ".grok/programs/delivery-fabric/evidence/PORTABLE-SWARM-CONTRACT.md";

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join(POLICY_PATH).is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root (dir holding {POLICY_PATH})");
}

fn load_policy(root: &PathBuf) -> AntiDriftDriftGrepPolicy {
    let raw =
        fs::read_to_string(root.join(POLICY_PATH)).expect("read anti-drift-drift-grep policy");
    let doc: Value = serde_json::from_str(&raw).expect("policy JSON");
    AntiDriftDriftGrepPolicy::from_json(&doc)
}

#[test]
fn shipped_policy_cites_pointers_and_gate_id() {
    let root = repo_root();
    let policy = load_policy(&root);
    assert_eq!(policy.gate_id, GATE_ID);
    assert_eq!(policy.prose_must_cite_authority, PROSE_MUST_CITE_POINTER);
    assert_eq!(policy.scanned_surfaces_authority, SCANNED_SURFACES_POINTER);
    assert!(!policy.require_surfaces);

    // Anti-drift: policy body must cite the pointer, not embed a cite-set array / #roots list.
    let raw = fs::read_to_string(root.join(POLICY_PATH)).expect("read");
    assert!(
        raw.contains(PROSE_MUST_CITE_POINTER),
        "policy must cite {PROSE_MUST_CITE_POINTER}"
    );
    assert!(
        !raw.contains("\"#roots\""),
        "policy must not embed #roots (dual-truth vs envelopes)"
    );
    assert!(
        !raw.contains("\"#hubs.paths\""),
        "policy must not embed #hubs.paths (dual-truth vs envelopes)"
    );
    assert!(
        !raw.contains("\"prose_must_cite_not_enumerate\":"),
        "policy must not inline a prose_must_cite_not_enumerate array"
    );
}

#[test]
fn red_fixture_root_enumeration_refuses() {
    let root = repo_root();
    let policy = load_policy(&root);
    let text = "\
## Amendment D\nINV-DOC-2\n\
Roots: `os`, `ci`, `governance`.\n\
Cite specs/integ-branch-envelopes.json#roots #hubs.paths reorg_debt_freeze.rows.\n";
    let surfaces = [ProseSurface {
        path: ADR_PATH.to_owned(),
        text: text.to_owned(),
    }];
    let report = evaluate(&policy, &fixture_prose_must_cite(), &surfaces);
    assert_eq!(report.verdict, Verdict::Refuse);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == CODE_PROSE_ROOT_ENUMERATION),
        "expected prose_root_enumeration, got {:?}",
        report.findings
    );
}

#[test]
fn red_fixture_freeze_table_refuses() {
    let root = repo_root();
    let policy = load_policy(&root);
    let text = "\
Amendment D INV-DOC-2\n\
| current path | action |\n| tools/ | reorg_now |\n\
#roots #hubs.paths reorg_debt_freeze.rows\n";
    let surfaces = [ProseSurface {
        path: PORTABLE_PATH.to_owned(),
        text: text.to_owned(),
    }];
    let report = evaluate(&policy, &fixture_prose_must_cite(), &surfaces);
    assert_eq!(report.verdict, Verdict::Refuse);
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.code == CODE_PROSE_FREEZE_PATH_TABLE),
        "expected prose_freeze_path_table, got {:?}",
        report.findings
    );
}

#[test]
fn green_fixture_cite_only_passes() {
    let root = repo_root();
    let policy = load_policy(&root);
    let text = "\
### Amendment D\n**INV-DOC-2:** cite `#roots`, `#planes`, `#hubs.paths`, `#reorg_debt_freeze.rows`.\n\
SSOT: specs/integ-branch-envelopes.json#anti_drift.\n";
    let surfaces = [ProseSurface {
        path: ADR_PATH.to_owned(),
        text: text.to_owned(),
    }];
    let report = evaluate(&policy, &fixture_prose_must_cite(), &surfaces);
    assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
}

#[test]
fn live_surfaces_bind_when_present() {
    let root = repo_root();
    let envelopes = root.join(ENVELOPES_PATH);
    let adr = root.join(ADR_PATH);
    if !envelopes.is_file() || !adr.is_file() {
        eprintln!(
            "skip live anti-drift bind: envelopes and/or ADR-0711 absent on tip (expected until integ/specs lands)"
        );
        return;
    }

    let env_raw = fs::read_to_string(&envelopes).expect("read envelopes");
    let env_doc: Value = serde_json::from_str(&env_raw).expect("envelopes JSON");
    let cite_authority = prose_must_cite_from_envelopes(&env_doc).expect("cite parse");
    assert!(
        !cite_authority.pointers.is_empty(),
        "live {PROSE_MUST_CITE_POINTER} must be non-empty"
    );

    let mut surfaces = Vec::new();
    let adr_text = fs::read_to_string(&adr).expect("read ADR");
    surfaces.push(ProseSurface {
        path: ADR_PATH.to_owned(),
        text: adr_text,
    });
    let portable = root.join(PORTABLE_PATH);
    if portable.is_file() {
        surfaces.push(ProseSurface {
            path: PORTABLE_PATH.to_owned(),
            text: fs::read_to_string(&portable).expect("read PORTABLE"),
        });
    }

    let policy = load_policy(&root);
    let report = evaluate(&policy, &cite_authority, &surfaces);
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "live ADR-0711/PORTABLE must be anti-drift Green, got {:?}",
        report.findings
    );
}
