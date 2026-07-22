use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use ci_cross_artifact_agreement::{
    IdeaArchiveMode, IdeaArchiveObservation, IdeaArchiveObservedNode, IdeaArchivePathKind,
    IdeaArchivePolicyError, IdeaArchiveTransitionError, IdeaArchiveVerifiedClosureProjection,
    collect_idea_archive_observation, evaluate_idea_archive_transition,
    immutable_idea_archive_baseline, parse_idea_archive_policy,
};
use serde_json::{Value, json};

fn policy(mode: &str, transition: Value) -> Value {
    json!({
        "retention_rules": {
            "idea_archive": {
                "policy_version": 1,
                "mode": mode,
                "transition": transition
            }
        }
    })
}

fn preparation_policy() -> Value {
    let baseline = immutable_idea_archive_baseline().expect("immutable baseline is valid");
    policy(
        "history-only-preparation",
        json!({
            "state": "open",
            "baseline_id": baseline.baseline_id,
            "manifest_path": "ci/facade/cross-artifact-agreement/src/idea-archive-transition-baseline.json",
            "sha256": baseline.manifest_sha256,
            "exception_semantics": "exact-path-and-byte-identity-only",
            "exception_expansion": "forbidden",
            "authority_state": "non-authoritative-transition-inputs",
            "completion_claim": false,
            "required_successor_epochs": ["E6", "E7", "E9", "E10"]
        }),
    )
}

fn strict_policy() -> Value {
    let baseline = immutable_idea_archive_baseline().expect("immutable baseline is valid");
    policy(
        "git-history-only",
        json!({
            "state": "closed",
            "baseline_id": baseline.baseline_id,
            "manifest_path": "ci/facade/cross-artifact-agreement/src/idea-archive-transition-baseline.json",
            "sha256": baseline.manifest_sha256,
            "closure_evidence_set_id": "adr-0388-transient-ideas-history-only-retirement-v1"
        }),
    )
}

fn exact_preparation_observation() -> IdeaArchiveObservation {
    let baseline = immutable_idea_archive_baseline().expect("immutable baseline is valid");
    let mut nodes = BTreeMap::new();
    let mut exact_body_locations = BTreeMap::new();
    for entry in baseline.entries {
        let path = format!("{}/{}", baseline.scope_root, entry.path);
        nodes.insert(
            path.clone(),
            IdeaArchiveObservedNode {
                kind: IdeaArchivePathKind::RegularFile,
                sha256: Some(entry.sha256.clone()),
                byte_length: Some(entry.byte_length),
            },
        );
        exact_body_locations.insert(entry.sha256, BTreeSet::from([path]));
    }
    IdeaArchiveObservation {
        archive_root_kind: IdeaArchivePathKind::Directory,
        nodes,
        exact_body_locations,
        verified_closure_projection: IdeaArchiveVerifiedClosureProjection::default(),
    }
}

#[test]
fn policy_grammar_is_required_and_fail_closed() {
    assert_eq!(
        parse_idea_archive_policy(&json!({})),
        Err(IdeaArchivePolicyError::MissingMode)
    );
    assert!(matches!(
        parse_idea_archive_policy(&policy("candidate-defined-mode", Value::Null)),
        Err(IdeaArchivePolicyError::UnknownMode(mode)) if mode == "candidate-defined-mode"
    ));
    assert!(matches!(
        parse_idea_archive_policy(&json!({
            "retention_rules": {
                "idea_archive": {
                    "policy_version": 2,
                    "mode": "current-tree-archive-compatible",
                    "transition": null
                }
            }
        })),
        Err(IdeaArchivePolicyError::InvalidTransition(_))
    ));
    assert!(matches!(
        parse_idea_archive_policy(&json!({
            "retention_rules": {
                "idea_archive": {
                    "policy_version": 1,
                    "mode": "current-tree-archive-compatible"
                }
            }
        })),
        Err(IdeaArchivePolicyError::InvalidTransition(_))
    ));
}

#[test]
fn compatibility_requires_null_transition_and_preserves_current_behavior() {
    let parsed = parse_idea_archive_policy(&policy("current-tree-archive-compatible", Value::Null))
        .expect("compatibility policy parses");
    assert_eq!(parsed.mode, IdeaArchiveMode::CurrentTreeArchiveCompatible);

    let observation = IdeaArchiveObservation {
        archive_root_kind: IdeaArchivePathKind::Directory,
        nodes: BTreeMap::from([(
            "docs/ideas/archive/unrestricted-compatibility.md".to_owned(),
            IdeaArchiveObservedNode {
                kind: IdeaArchivePathKind::RegularFile,
                sha256: Some("compatibility-does-not-freeze-the-corpus".to_owned()),
                byte_length: Some(1),
            },
        )]),
        exact_body_locations: BTreeMap::new(),
        verified_closure_projection: IdeaArchiveVerifiedClosureProjection::default(),
    };
    assert!(evaluate_idea_archive_transition(&parsed, &observation).is_ok());

    assert!(matches!(
        parse_idea_archive_policy(&policy(
            "current-tree-archive-compatible",
            json!({"baseline_id": "not-allowed"}),
        )),
        Err(IdeaArchivePolicyError::InvalidTransition(_))
    ));
}

#[test]
fn preparation_accepts_only_the_exact_immutable_baseline() {
    let parsed =
        parse_idea_archive_policy(&preparation_policy()).expect("preparation policy parses");
    assert_eq!(parsed.mode, IdeaArchiveMode::HistoryOnlyPreparation);
    assert!(evaluate_idea_archive_transition(&parsed, &exact_preparation_observation()).is_ok());
}

#[test]
fn preparation_rejects_missing_extra_renamed_symlinked_or_mutated_nodes() {
    let parsed =
        parse_idea_archive_policy(&preparation_policy()).expect("preparation policy parses");
    let baseline = immutable_idea_archive_baseline().expect("immutable baseline is valid");
    let first_path = format!("{}/{}", baseline.scope_root, baseline.entries[0].path);

    let mut missing = exact_preparation_observation();
    missing.nodes.remove(&first_path);
    assert!(matches!(
        evaluate_idea_archive_transition(&parsed, &missing),
        Err(IdeaArchiveTransitionError::BaselineMismatch(_))
    ));

    let mut extra = exact_preparation_observation();
    extra.nodes.insert(
        "docs/ideas/archive/new-archive-body.md".to_owned(),
        IdeaArchiveObservedNode {
            kind: IdeaArchivePathKind::RegularFile,
            sha256: Some("new".to_owned()),
            byte_length: Some(3),
        },
    );
    assert!(matches!(
        evaluate_idea_archive_transition(&parsed, &extra),
        Err(IdeaArchiveTransitionError::BaselineMismatch(_))
    ));

    let mut nested = exact_preparation_observation();
    nested.nodes.insert(
        "docs/ideas/archive/nested".to_owned(),
        IdeaArchiveObservedNode {
            kind: IdeaArchivePathKind::Directory,
            sha256: None,
            byte_length: None,
        },
    );
    assert!(matches!(
        evaluate_idea_archive_transition(&parsed, &nested),
        Err(IdeaArchiveTransitionError::BaselineMismatch(_))
    ));

    let mut symlink = exact_preparation_observation();
    symlink.nodes.get_mut(&first_path).expect("first node").kind = IdeaArchivePathKind::Symlink;
    assert!(matches!(
        evaluate_idea_archive_transition(&parsed, &symlink),
        Err(IdeaArchiveTransitionError::BaselineMismatch(_))
    ));

    let mut mutated = exact_preparation_observation();
    let first = mutated.nodes.get_mut(&first_path).expect("first node");
    first.sha256 = Some("00".repeat(32));
    assert!(matches!(
        evaluate_idea_archive_transition(&parsed, &mutated),
        Err(IdeaArchiveTransitionError::BaselineMismatch(_))
    ));

    let mut wrong_length = exact_preparation_observation();
    let first = wrong_length.nodes.get_mut(&first_path).expect("first node");
    first.byte_length = first.byte_length.map(|length| length + 1);
    assert!(matches!(
        evaluate_idea_archive_transition(&parsed, &wrong_length),
        Err(IdeaArchiveTransitionError::BaselineMismatch(_))
    ));
}

#[test]
fn preparation_rejects_exact_body_duplicates_outside_the_baseline_paths() {
    let parsed =
        parse_idea_archive_policy(&preparation_policy()).expect("preparation policy parses");
    let baseline = immutable_idea_archive_baseline().expect("immutable baseline is valid");
    let mut observed = exact_preparation_observation();
    observed
        .exact_body_locations
        .get_mut(&baseline.entries[0].sha256)
        .expect("first digest locations")
        .insert("docs/ideas/copied-live-body.md".to_owned());

    assert!(matches!(
        evaluate_idea_archive_transition(&parsed, &observed),
        Err(IdeaArchiveTransitionError::BaselineMismatch(_))
    ));
}

#[test]
fn strict_mode_requires_archive_and_all_baseline_bodies_to_be_absent() {
    let parsed = parse_idea_archive_policy(&strict_policy()).expect("strict policy parses");
    assert_eq!(parsed.mode, IdeaArchiveMode::GitHistoryOnly);
    let absent = IdeaArchiveObservation {
        archive_root_kind: IdeaArchivePathKind::Missing,
        nodes: BTreeMap::new(),
        exact_body_locations: BTreeMap::new(),
        verified_closure_projection: IdeaArchiveVerifiedClosureProjection {
            evidence_set_ids: BTreeSet::from([
                "adr-0388-transient-ideas-history-only-retirement-v1".to_owned(),
            ]),
        },
    };
    assert!(evaluate_idea_archive_transition(&parsed, &absent).is_ok());

    let mut archive_remains = absent.clone();
    archive_remains.archive_root_kind = IdeaArchivePathKind::Directory;
    assert!(matches!(
        evaluate_idea_archive_transition(&parsed, &archive_remains),
        Err(IdeaArchiveTransitionError::BaselineMismatch(_))
    ));

    let baseline = immutable_idea_archive_baseline().expect("immutable baseline is valid");
    let mut body_remains = absent;
    body_remains.exact_body_locations.insert(
        baseline.entries[0].sha256.clone(),
        BTreeSet::from(["docs/ideas/copied-live-body.md".to_owned()]),
    );
    assert!(matches!(
        evaluate_idea_archive_transition(&parsed, &body_remains),
        Err(IdeaArchiveTransitionError::BaselineMismatch(_))
    ));

    let mut missing_projection = IdeaArchiveObservation {
        archive_root_kind: IdeaArchivePathKind::Missing,
        nodes: BTreeMap::new(),
        exact_body_locations: BTreeMap::new(),
        verified_closure_projection: IdeaArchiveVerifiedClosureProjection::default(),
    };
    assert!(matches!(
        evaluate_idea_archive_transition(&parsed, &missing_projection),
        Err(IdeaArchiveTransitionError::BaselineMismatch(_))
    ));
    missing_projection
        .verified_closure_projection
        .evidence_set_ids
        .insert("candidate-defined-evidence".to_owned());
    assert!(matches!(
        evaluate_idea_archive_transition(&parsed, &missing_projection),
        Err(IdeaArchiveTransitionError::BaselineMismatch(_))
    ));

    assert!(matches!(
        parse_idea_archive_policy(&policy("git-history-only", Value::Null)),
        Err(IdeaArchivePolicyError::InvalidTransition(_))
    ));
}

#[test]
fn preparation_binding_cannot_redefine_or_expand_the_evaluator_contract() {
    let baseline = immutable_idea_archive_baseline().expect("immutable baseline is valid");
    let valid = preparation_policy()["retention_rules"]["idea_archive"]["transition"].clone();
    let mut transitions = Vec::new();
    for missing in [
        "state",
        "baseline_id",
        "manifest_path",
        "sha256",
        "exception_semantics",
        "exception_expansion",
        "authority_state",
        "completion_claim",
        "required_successor_epochs",
    ] {
        let mut transition = valid.clone();
        transition
            .as_object_mut()
            .expect("preparation transition object")
            .remove(missing);
        transitions.push(transition);
    }
    for (field, value) in [
        ("state", json!("closed")),
        ("baseline_id", json!("candidate-defined-baseline")),
        ("manifest_path", json!("candidate-defined.json")),
        ("sha256", json!("00".repeat(32))),
        ("exception_semantics", json!("path-only")),
        ("exception_expansion", json!("allowed")),
        ("authority_state", json!("authoritative")),
        ("completion_claim", json!(true)),
        ("required_successor_epochs", json!(["E6", "E7", "E9"])),
    ] {
        let mut transition = valid.clone();
        transition[field] = value;
        transitions.push(transition);
    }
    let mut extra = valid;
    extra["exceptions"] = json!([]);
    transitions.push(extra);

    for transition in transitions {
        assert!(matches!(
            parse_idea_archive_policy(&policy("history-only-preparation", transition)),
            Err(IdeaArchivePolicyError::BaselineMismatch(_))
                | Err(IdeaArchivePolicyError::InvalidTransition(_))
        ));
    }

    assert_eq!(baseline.entries.len(), 3);
}

#[test]
fn strict_binding_is_closed_and_cannot_self_select_closure_evidence() {
    let valid = strict_policy()["retention_rules"]["idea_archive"]["transition"].clone();
    let mut invalid = vec![Value::Null];
    for missing in [
        "state",
        "baseline_id",
        "manifest_path",
        "sha256",
        "closure_evidence_set_id",
    ] {
        let mut transition = valid.clone();
        transition
            .as_object_mut()
            .expect("strict transition object")
            .remove(missing);
        invalid.push(transition);
    }
    for (field, value) in [
        ("state", json!("open")),
        ("baseline_id", json!("stale-baseline")),
        ("manifest_path", json!("candidate-defined.json")),
        ("sha256", json!("00".repeat(32))),
        (
            "closure_evidence_set_id",
            json!("candidate-defined-evidence"),
        ),
    ] {
        let mut transition = valid.clone();
        transition[field] = value;
        invalid.push(transition);
    }
    let mut receipt_refs = valid.clone();
    receipt_refs["closure_receipt_refs"] =
        json!(["evidence/consolidation/candidate-selected.json"]);
    invalid.push(receipt_refs);

    for transition in invalid {
        assert!(matches!(
            parse_idea_archive_policy(&policy("git-history-only", transition)),
            Err(IdeaArchivePolicyError::BaselineMismatch(_))
                | Err(IdeaArchivePolicyError::InvalidTransition(_))
        ));
    }
}

#[test]
fn live_gate_parse_collect_evaluate_path_is_mode_generic() {
    let root = super::repo_root();
    let policy_path = std::env::var_os("OYA_IDEA_ARCHIVE_POLICY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("specs/markdown-retirement-policy.json"));
    let policy_bytes = fs::read(&policy_path).unwrap_or_else(|error| {
        panic!(
            "read live markdown retirement policy {}: {error}",
            policy_path.display()
        )
    });
    let policy_json: Value =
        serde_json::from_slice(&policy_bytes).expect("parse live markdown retirement policy");
    let policy = parse_idea_archive_policy(&policy_json).expect("live policy grammar is valid");

    let baseline = immutable_idea_archive_baseline().expect("immutable baseline is valid");
    assert_eq!(
        baseline.baseline_id,
        "IDEA-ARCHIVE-TRANSITION-2026-07-22-V1"
    );
    assert_eq!(
        baseline.captured_from.commit_oid,
        "1fa09da22be819b062881eb59252f4dd4c6b550a"
    );
    assert_eq!(
        baseline.captured_from.tree_oid,
        "d7b15539396db21b219d68779362850cce9afa8f"
    );
    assert_eq!(baseline.captured_from.object_format, "sha1");
    assert_eq!(baseline.scope_root, "docs/ideas/archive");
    assert_eq!(baseline.entries.len(), 3);

    let observation =
        collect_idea_archive_observation(&root, IdeaArchiveVerifiedClosureProjection::default())
            .expect("collect live candidate-tree observation");
    let report = evaluate_idea_archive_transition(&policy, &observation)
        .expect("live archive policy must match the candidate tree");
    assert_eq!(report.mode, policy.mode);
}
