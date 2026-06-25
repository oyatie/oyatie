//! Conformance test suite for `oya-ci-materializer-kernel`.
//!
//! Proves CP-1..CP-6 (the universality certificate) plus the MF-2 (banned-symbol
//! purity) and MF-3 (no oyatie leak) source-grep properties.
//!
//! ALL tests are pure: no I/O, no clock, no subprocess, no real buck2/git.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;

use oya_ci_materializer_kernel::{
    ControlPlane, FindingCode, MaterializeScope, OutputSink, evaluate, plan,
};

// ─── Fixture helpers ─────────────────────────────────────────────────────────

/// Synthetic-repo fixture embedded at compile time (buck2 includes it via srcs glob).
const SYNTHETIC_FIXTURE_JSON: &str = include_str!("fixtures/synthetic-repo/control-plane.json");

/// Minimal 3-node producer -> intermediate -> leaf chain. Exercises the TRANSITIVE
/// (multi-hop) input_contract closure: the leaf's input_contract references only the
/// intermediate, so the producer is reachable ONLY through a second hop.
const TRANSITIVE_CHAIN_FIXTURE_JSON: &str =
    include_str!("fixtures/synthetic-repo/transitive-chain.json");

fn synthetic_manifest() -> ControlPlane {
    ControlPlane::from_json(SYNTHETIC_FIXTURE_JSON).expect("synthetic fixture must parse")
}

fn transitive_chain_manifest() -> ControlPlane {
    ControlPlane::from_json(TRANSITIVE_CHAIN_FIXTURE_JSON)
        .expect("transitive-chain fixture must parse")
}

fn minimal_two_artifact_manifest() -> ControlPlane {
    let json = r#"{
      "schema_version": 2,
      "runner_registry": [
        {"runner_id":"buck2","canonical_target_prefix":"//","lowering":"build-target-then-exec"}
      ],
      "artifacts": [
        {
          "artifact_id": "emitter",
          "path": "ci/emitter.generated.json",
          "materialization_mode": "not-tracked-in-git",
          "generator": {
            "runner": "buck2",
            "generator_target": "//ci:emitter",
            "operation_id": "emit-emitter",
            "input_contract": ["repo-root"],
            "output_mode": "declared-artifact-path-write"
          }
        },
        {
          "artifact_id": "producer",
          "path": "ci/producer.generated.json",
          "materialization_mode": "not-tracked-in-git",
          "generator": {
            "runner": "buck2",
            "generator_target": "//ci:producer",
            "operation_id": "emit-producer",
            "input_contract": ["repo-root", "emit-emitter"],
            "output_mode": "stdout-json"
          }
        }
      ]
    }"#;
    ControlPlane::from_json(json).unwrap()
}

/// Locate the repo root by walking up from current_dir until we find specs/root-hub-pointers.json.
/// Used only by source-grep tests (MF-2, MF-3) which need to read kernel source files at runtime.
fn repo_root() -> std::path::PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root from test current_dir");
}

// ─── CP-1: Plan determinism ───────────────────────────────────────────────────

/// CP-1: `plan(manifest, Consume)` is byte-identical across two calls with the same
/// manifest. No clock, pid, or path leakage can make the plan differ.
#[test]
fn cp1_plan_determinism() {
    let manifest = minimal_two_artifact_manifest();
    let scope_a = MaterializeScope::Consume {
        target_paths: BTreeSet::new(),
    };
    let scope_b = MaterializeScope::Consume {
        target_paths: BTreeSet::new(),
    };
    let plan_a = plan(&manifest, scope_a).expect("plan must succeed");
    let plan_b = plan(&manifest, scope_b).expect("plan must succeed");
    assert_eq!(plan_a, plan_b, "CP-1: plan() must be deterministic");
}

/// CP-1 extended: deterministic on the synthetic fixture too.
#[test]
fn cp1_plan_determinism_synthetic() {
    let manifest = synthetic_manifest();
    let plan_a = plan(
        &manifest,
        MaterializeScope::Consume {
            target_paths: BTreeSet::new(),
        },
    )
    .unwrap();
    let plan_b = plan(
        &manifest,
        MaterializeScope::Consume {
            target_paths: BTreeSet::new(),
        },
    )
    .unwrap();
    assert_eq!(
        plan_a, plan_b,
        "CP-1: synthetic fixture plan must be deterministic"
    );
}

// ─── CP-2: Topological order from input_contract ──────────────────────────────

/// CP-2: emitter comes before producer because producer's input_contract references
/// "emit-emitter" (the emitter's operation_id). No target strings in engine code.
#[test]
fn cp2_topological_order_from_input_contract() {
    let manifest = minimal_two_artifact_manifest();
    let result = plan(
        &manifest,
        MaterializeScope::Consume {
            target_paths: BTreeSet::new(),
        },
    )
    .unwrap();
    let ids: Vec<&str> = result
        .steps
        .iter()
        .map(|s| s.artifact_id.as_str())
        .collect();
    let emitter_pos = ids.iter().position(|&id| id == "emitter").unwrap();
    let producer_pos = ids.iter().position(|&id| id == "producer").unwrap();
    assert!(
        emitter_pos < producer_pos,
        "CP-2: emitter must precede producer"
    );
}

/// CP-2 extended: in the synthetic fixture, ts-schema precedes api-client because
/// api-client's input_contract references "emit-schema-types" (ts-schema's operation_id).
#[test]
fn cp2_topological_order_node_codegen_synthetic() {
    let manifest = synthetic_manifest();
    let result = plan(
        &manifest,
        MaterializeScope::Consume {
            target_paths: BTreeSet::new(),
        },
    )
    .unwrap();
    let ids: Vec<&str> = result
        .steps
        .iter()
        .map(|s| s.artifact_id.as_str())
        .collect();
    let schema_pos = ids
        .iter()
        .position(|&id| id == "synthetic-ts-schema")
        .unwrap();
    let client_pos = ids
        .iter()
        .position(|&id| id == "synthetic-api-client")
        .unwrap();
    assert!(
        schema_pos < client_pos,
        "CP-2: ts-schema must precede api-client"
    );
}

// ─── CP-2 / closure: transitive target expansion ──────────────────────────────

/// CP-closure: requesting ONLY a leaf artifact path must pull its full transitive
/// `input_contract` closure into the plan — not just the directly-referenced parent.
///
/// Chain: chain-producer -> chain-intermediate -> chain-leaf. The leaf's input_contract
/// references only the INTERMEDIATE (`emit-chain-intermediate`); the producer is reachable
/// solely through a SECOND hop. Asking for the leaf path alone must therefore include the
/// producer (the transitive hop) AND order it before the leaf.
///
/// RED before the closure fix: the old `plan()` used the directly-matched seed set verbatim,
/// so this plan would contain ONLY `chain-leaf` and `position("chain-producer")` would panic.
#[test]
fn cp_closure_includes_transitive_deps() {
    let manifest = transitive_chain_manifest();

    // Request ONLY the leaf path.
    let mut targets = BTreeSet::new();
    targets.insert("gen/chain/leaf.ts".to_owned());

    let result = plan(
        &manifest,
        MaterializeScope::Consume {
            target_paths: targets,
        },
    )
    .unwrap();
    let ids: Vec<&str> = result
        .steps
        .iter()
        .map(|s| s.artifact_id.as_str())
        .collect();

    // The transitive producer (two hops up from the leaf) MUST be in the plan.
    assert!(
        ids.contains(&"chain-producer"),
        "closure: transitive producer 'chain-producer' must be included when only the leaf is requested; got {ids:?}"
    );
    // The direct intermediate parent MUST be in the plan.
    assert!(
        ids.contains(&"chain-intermediate"),
        "closure: intermediate 'chain-intermediate' must be included; got {ids:?}"
    );
    assert!(
        ids.contains(&"chain-leaf"),
        "closure: leaf must be in the plan; got {ids:?}"
    );

    // The transitive producer must be ORDERED BEFORE the leaf.
    let producer_pos = ids.iter().position(|&id| id == "chain-producer").unwrap();
    let intermediate_pos = ids
        .iter()
        .position(|&id| id == "chain-intermediate")
        .unwrap();
    let leaf_pos = ids.iter().position(|&id| id == "chain-leaf").unwrap();
    assert!(
        producer_pos < intermediate_pos && intermediate_pos < leaf_pos,
        "closure: order must be producer < intermediate < leaf; got {ids:?}"
    );
}

// ─── CP-3: Canary catches nondeterminism ──────────────────────────────────────

/// CP-3: two different canary passes -> RED finding. Deterministic passes -> GREEN.
#[test]
fn cp3_canary_catches_nondeterminism() {
    let manifest = minimal_two_artifact_manifest();

    // Nondeterministic producer: pass A and pass B differ.
    let pass_a = vec![
        ("emitter".to_owned(), "emitter-bytes".to_owned()),
        ("producer".to_owned(), "producer-bytes-v1".to_owned()),
    ];
    let pass_b = vec![
        ("emitter".to_owned(), "emitter-bytes".to_owned()),
        (
            "producer".to_owned(),
            "producer-bytes-v2-DIFFERENT".to_owned(),
        ),
    ];
    let findings = evaluate(&pass_a, &pass_b, &[], &manifest);
    assert!(
        !findings.is_green(),
        "CP-3: nondeterministic producer must yield RED"
    );
    assert!(
        findings
            .findings
            .iter()
            .any(|f| f.code == FindingCode::GeneratedArtifactNondeterministic),
        "CP-3: must emit GeneratedArtifactNondeterministic"
    );

    // Deterministic: GREEN.
    let pass_a2 = vec![
        ("emitter".to_owned(), "bytes".to_owned()),
        ("producer".to_owned(), "bytes".to_owned()),
    ];
    let pass_b2 = pass_a2.clone();
    assert!(
        evaluate(&pass_a2, &pass_b2, &[], &manifest).is_green(),
        "CP-3: deterministic -> GREEN"
    );
}

// ─── CP-4: Single-build invariant ─────────────────────────────────────────────

/// CP-4: In DeterminismCanary scope, de-commit-class steps have multiplicity=2 and
/// TwoCapturedBuffers. Committed-class steps have multiplicity=1 even in canary scope.
#[test]
fn cp4_single_build_invariant() {
    let manifest = minimal_two_artifact_manifest();
    let result = plan(
        &manifest,
        MaterializeScope::DeterminismCanary {
            target_paths: BTreeSet::new(),
        },
    )
    .unwrap();

    for step in &result.steps {
        assert_eq!(
            step.multiplicity, 2,
            "CP-4: de-commit step '{}' must have multiplicity=2 in canary scope",
            step.artifact_id
        );
        assert_eq!(
            step.output,
            OutputSink::TwoCapturedBuffers,
            "CP-4: de-commit step '{}' must use TwoCapturedBuffers",
            step.artifact_id
        );
    }

    // Committed-class artifact in canary scope must still have multiplicity=1.
    let committed_json = r#"{
      "schema_version": 2,
      "runner_registry": [{"runner_id":"buck2","canonical_target_prefix":"//","lowering":"x"}],
      "artifacts": [{
        "artifact_id": "committed-face",
        "path": "ci/face.generated.json",
        "materialization_mode": "merge-candidate",
        "generator": {
          "runner": "buck2",
          "generator_target": "//ci:face",
          "operation_id": "emit-face",
          "output_mode": "stdout-json"
        }
      }]
    }"#;
    let committed_manifest = ControlPlane::from_json(committed_json).unwrap();
    let committed_plan = plan(
        &committed_manifest,
        MaterializeScope::DeterminismCanary {
            target_paths: BTreeSet::new(),
        },
    )
    .unwrap();
    let step = committed_plan
        .steps
        .iter()
        .find(|s| s.artifact_id == "committed-face")
        .unwrap();
    assert_eq!(
        step.multiplicity, 1,
        "CP-4: committed-class step must have multiplicity=1"
    );
}

// ─── CP-5: Anti-forgery ───────────────────────────────────────────────────────

/// CP-5a: An evil manifest row at `evil/scm-facts.generated.json` (same basename as a
/// canonical committed artifact) does NOT exempt the canonical artifact from byte-parity.
/// Full-path keying in the manifest makes basename collision structurally impossible.
#[test]
fn cp5_anti_forgery_full_path_keying() {
    let json = r#"{
      "schema_version": 2,
      "runner_registry": [{"runner_id":"buck2","canonical_target_prefix":"//","lowering":"x"}],
      "artifacts": [
        {
          "artifact_id": "canonical",
          "path": "ci/scm-facts.generated.json",
          "materialization_mode": "merge-candidate",
          "generator": {
            "runner": "buck2", "generator_target": "//ci:canonical",
            "operation_id": "emit-canonical", "output_mode": "stdout-json"
          }
        },
        {
          "artifact_id": "evil",
          "path": "evil/scm-facts.generated.json",
          "materialization_mode": "not-tracked-in-git",
          "generator": {
            "runner": "buck2", "generator_target": "//evil:evil",
            "operation_id": "emit-evil", "output_mode": "stdout-json"
          }
        }
      ]
    }"#;
    let manifest = ControlPlane::from_json(json).unwrap();

    // Canonical path must NOT be in the de-commit set.
    let decommit = manifest.decommit_paths();
    assert!(
        !decommit.contains(&"ci/scm-facts.generated.json"),
        "CP-5: canonical path must not be de-committed by the evil row"
    );
    assert!(
        decommit.contains(&"evil/scm-facts.generated.json"),
        "CP-5: evil path is in de-commit set (it was declared so)"
    );

    // The canonical artifact's byte-parity check must fire even though the evil row exists.
    let pass_a = vec![
        ("canonical".to_owned(), "regenerated".to_owned()),
        ("evil".to_owned(), "evil-bytes".to_owned()),
    ];
    let committed = vec![("canonical".to_owned(), "old-committed".to_owned())];
    let pass_b = pass_a.clone();
    let findings = evaluate(&pass_a, &pass_b, &committed, &manifest);
    assert!(
        findings
            .findings
            .iter()
            .any(|f| f.artifact_id == "canonical" && f.code == FindingCode::GeneratedArtifactStale),
        "CP-5: canonical artifact must receive stale finding regardless of evil row"
    );
}

/// CP-5b: Unregistered runner -> Err.
#[test]
fn cp5_unregistered_runner_is_err() {
    use oya_ci_materializer_kernel::PlanError;
    let json = r#"{
      "schema_version": 2,
      "runner_registry": [{"runner_id":"buck2","canonical_target_prefix":"//","lowering":"x"}],
      "artifacts": [{
        "artifact_id": "x", "path": "x.generated.json",
        "materialization_mode": "not-tracked-in-git",
        "generator": {
          "runner": "evil-runner", "generator_target": "evil://target",
          "operation_id": "emit-x", "output_mode": "stdout-json"
        }
      }]
    }"#;
    let manifest = ControlPlane::from_json(json).unwrap();
    let err = plan(
        &manifest,
        MaterializeScope::Consume {
            target_paths: BTreeSet::new(),
        },
    )
    .unwrap_err();
    assert!(matches!(err, PlanError::UnregisteredRunner { .. }));
}

/// CP-5c: Non-canonical target -> Err.
#[test]
fn cp5_non_canonical_target_is_err() {
    use oya_ci_materializer_kernel::PlanError;
    let json = r#"{
      "schema_version": 2,
      "runner_registry": [{"runner_id":"buck2","canonical_target_prefix":"//","lowering":"x"}],
      "artifacts": [{
        "artifact_id": "x", "path": "x.generated.json",
        "materialization_mode": "not-tracked-in-git",
        "generator": {
          "runner": "buck2", "generator_target": "bad-target",
          "operation_id": "emit-x", "output_mode": "stdout-json"
        }
      }]
    }"#;
    let manifest = ControlPlane::from_json(json).unwrap();
    let err = plan(
        &manifest,
        MaterializeScope::Consume {
            target_paths: BTreeSet::new(),
        },
    )
    .unwrap_err();
    assert!(matches!(err, PlanError::NonCanonicalTarget { .. }));
}

/// CP-5d: shell runner -> ShellRunnerForbidden.
#[test]
fn cp5_shell_runner_forbidden() {
    use oya_ci_materializer_kernel::PlanError;
    let json = r#"{
      "schema_version": 2,
      "runner_registry": [{"runner_id":"shell","canonical_target_prefix":"","lowering":"exec"}],
      "artifacts": []
    }"#;
    let manifest = ControlPlane::from_json(json).unwrap();
    let err = plan(
        &manifest,
        MaterializeScope::Consume {
            target_paths: BTreeSet::new(),
        },
    )
    .unwrap_err();
    assert_eq!(err, PlanError::ShellRunnerForbidden);
}

// ─── CP-6: Repo-agnosticism ───────────────────────────────────────────────────

/// CP-6: Engine produces a valid plan for the synthetic fixture (buck2 + node-codegen).
/// ZERO oyatie paths appear in the plan CODE — only fixture manifest data drives it.
#[test]
fn cp6_repo_agnosticism_synthetic_fixture() {
    let manifest = synthetic_manifest();
    let result = plan(
        &manifest,
        MaterializeScope::Consume {
            target_paths: BTreeSet::new(),
        },
    )
    .expect("CP-6: engine must plan the synthetic fixture without error");

    // 5 artifacts in the fixture (4 not-tracked + 1 committed).
    assert_eq!(
        result.steps.len(),
        5,
        "CP-6: plan must cover all 5 synthetic artifacts"
    );

    // 2 node-codegen steps.
    let node_count = result
        .steps
        .iter()
        .filter(|s| s.runner_id == "node-codegen")
        .count();
    assert_eq!(
        node_count, 2,
        "CP-6: plan must include 2 node-codegen steps"
    );

    // Every target in the plan exactly matches a declared generator_target from the fixture.
    let expected: BTreeSet<&str> = [
        "//ci/emitter:emitter-bin",
        "//ci/producer:producer-bin",
        "npm://codegen/typescript",
        "npm://codegen/openapi",
        "//docs:openapi-gen",
    ]
    .iter()
    .copied()
    .collect();
    let actual: BTreeSet<&str> = result
        .steps
        .iter()
        .map(|s| s.generator_target.as_str())
        .collect();
    assert_eq!(
        actual, expected,
        "CP-6: plan targets must be exactly what the fixture declares"
    );
}

/// CP-6 / MF-3: Source-grep — the kernel src MUST NOT contain any oyatie-specific literals.
/// This is the universality certificate: the engine carries zero hardcoded oyatie values.
#[test]
fn cp6_mf3_no_oyatie_literals_in_kernel_source() {
    let root = repo_root();
    let kernel_src = root.join("libs/oya-ci-materializer-kernel/src");

    let forbidden = ["//cloud/", "oya-cloud-ci-", "cloud/cloud-ci"];
    let mut violations: Vec<String> = Vec::new();

    for entry in walkdir(&kernel_src) {
        if !entry.to_string_lossy().ends_with(".rs") {
            continue;
        }
        let content = std::fs::read_to_string(&entry).unwrap_or_default();
        for lit in &forbidden {
            for (lineno, line) in content.lines().enumerate() {
                if line.contains(lit) {
                    // Skip comment lines that document the ban itself.
                    if line.trim().starts_with("//") || line.trim().starts_with("*") {
                        continue;
                    }
                    violations.push(format!(
                        "{}:{}: forbidden oyatie literal {:?}: {}",
                        entry.display(),
                        lineno + 1,
                        lit,
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "MF-3/CP-6: kernel source must contain ZERO oyatie-specific literals.\nViolations:\n{}",
        violations.join("\n")
    );
}

/// MF-2: Banned-symbol purity — the kernel src must NOT use std::process, std::time
/// (SystemTime/Instant), std::net, std::fs, std::env, or rand.
///
/// ADR-0547's kernel-purity gate bans dep CRATES; this test fills the gap by checking
/// banned std SYMBOLS at the source level.
#[test]
fn mf2_no_banned_symbols_in_kernel_source() {
    let root = repo_root();
    let kernel_src = root.join("libs/oya-ci-materializer-kernel/src");

    // These must never appear as live code in the kernel (comments exempted).
    let banned = [
        "std::process",
        "std::time::SystemTime",
        "std::time::Instant",
        "std::net::",
        "use std::fs",
        "std::fs::",
        "std::env::",
        "use rand",
        "rand::",
    ];

    let mut violations: Vec<String> = Vec::new();

    for entry in walkdir(&kernel_src) {
        if !entry.to_string_lossy().ends_with(".rs") {
            continue;
        }
        let content = std::fs::read_to_string(&entry).unwrap_or_default();
        for pattern in &banned {
            for (lineno, line) in content.lines().enumerate() {
                if line.contains(pattern) {
                    let trimmed = line.trim();
                    // Skip comment lines documenting the ban.
                    if trimmed.starts_with("//") || trimmed.starts_with("*") {
                        continue;
                    }
                    violations.push(format!(
                        "{}:{}: banned symbol {:?}: {}",
                        entry.display(),
                        lineno + 1,
                        pattern,
                        trimmed
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "MF-2: kernel must not use banned std symbols (process/time/net/fs/env/rand).\nViolations:\n{}",
        violations.join("\n")
    );
}

/// MF-1 shape test: `evaluate()` accepts `manifest` as a separate parameter so E3
/// can pass the merge-base manifest for the exemption-set without a schema break.
#[test]
fn mf1_evaluate_accepts_separate_manifest_parameter() {
    // Candidate manifest: artifact is de-commit-class.
    let candidate_json = r#"{
      "schema_version": 2, "runner_registry": [],
      "artifacts": [{
        "artifact_id": "a", "path": "ci/a.generated.json",
        "materialization_mode": "not-tracked-in-git",
        "generator": {"runner":"buck2","generator_target":"//ci:a",
          "operation_id":"emit-a","output_mode":"stdout-json"}
      }]
    }"#;
    // Merge-base manifest: same artifact is committed-class.
    let mergebase_json = r#"{
      "schema_version": 2, "runner_registry": [],
      "artifacts": [{
        "artifact_id": "a", "path": "ci/a.generated.json",
        "materialization_mode": "merge-candidate",
        "generator": {"runner":"buck2","generator_target":"//ci:a",
          "operation_id":"emit-a","output_mode":"stdout-json"}
      }]
    }"#;
    let candidate = ControlPlane::from_json(candidate_json).unwrap();
    let mergebase = ControlPlane::from_json(mergebase_json).unwrap();

    let pass_a = vec![("a".to_owned(), "bytes".to_owned())];
    let pass_b = vec![("a".to_owned(), "bytes".to_owned())];

    // With candidate manifest (de-commit, deterministic) -> GREEN.
    assert!(
        evaluate(&pass_a, &pass_b, &[], &candidate).is_green(),
        "candidate de-commit + deterministic -> GREEN"
    );

    // With merge-base manifest (committed, parity mismatch) -> RED.
    let committed = vec![("a".to_owned(), "DIFFERENT".to_owned())];
    assert!(
        !evaluate(&pass_a, &pass_b, &committed, &mergebase).is_green(),
        "merge-base committed + parity mismatch -> RED (E3 can swap manifest without schema break)"
    );
}

// ─── Helper ──────────────────────────────────────────────────────────────────

fn walkdir(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return result;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            result.extend(walkdir(&path));
        } else {
            result.push(path);
        }
    }
    result
}
