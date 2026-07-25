//! Pure validation for the ADR-0515 preparation facts face.
//!
//! This is intentionally a non-claiming consumer: it validates that the face
//! remains a HOLD(Planning) preparation record and rejects any closure or
//! authority-shaped drift.

use std::collections::BTreeSet;

use serde::Deserialize;
use serde_json::Value;

use crate::Finding;

pub const ADR_0515_PREPARATION_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/adr-0515-preparation";
pub const ADR_0515_PREPARATION_CODE: &str = "adr_0515_preparation_facts_invalid";

const ADR_IDS: [&str; 7] = [
    "ADR-0124", "ADR-0349", "ADR-0359", "ADR-0361", "ADR-0511", "ADR-0513", "ADR-0514",
];
const PREDECESSOR_COMMIT_OID: &str = "1fa09da22be819b062881eb59252f4dd4c6b550a";
const PREDECESSOR_TREE_OID: &str = "d7b15539396db21b219d68779362850cce9afa8f";

struct SelectorBinding {
    adr_id: &'static str,
    path: &'static str,
    predecessor_blob_oid: &'static str,
    sha256: &'static str,
    byte_count: u64,
}

const SELECTOR_BINDINGS: [SelectorBinding; 7] = [
    SelectorBinding {
        adr_id: "ADR-0124",
        path: "docs/decisions/ADR-0124-own-merge-queue-webhook-driven.md",
        predecessor_blob_oid: "c3af0c22453e58749aa7173c11167e0fb66e1412",
        sha256: "sha256:d57d67d3502ca013ba4e1b67fec77eb902609c2897331ac8c851c76c70ad45f8",
        byte_count: 10228,
    },
    SelectorBinding {
        adr_id: "ADR-0349",
        path: "docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md",
        predecessor_blob_oid: "313f75b8d6d7cad21f76d6afa1807c22587da198",
        sha256: "sha256:c63967bb8d8255d5b13ca9f880e7dbd05782ed46cf51042c94b80ff4742895c2",
        byte_count: 107783,
    },
    SelectorBinding {
        adr_id: "ADR-0359",
        path: "docs/decisions/ADR-0359-jenkins-completely-replaces-github-actions.md",
        predecessor_blob_oid: "d173ecaf58df93d76bca53d191dbaddfa8b7b115",
        sha256: "sha256:206818ac56bfb5a2589dd2fb51b38045fce314c05b3d0472db223c9b1cc4ba7c",
        byte_count: 4143,
    },
    SelectorBinding {
        adr_id: "ADR-0361",
        path: "docs/decisions/ADR-0361-jenkins-native-cicd-revamp-execution.md",
        predecessor_blob_oid: "55a4e35b6652fea476729137676e84dc1eabbe6c",
        sha256: "sha256:ab1129a35dba587f3d9910bf48f0d6078bc9d072cb0c76ca47504980efb23dc6",
        byte_count: 4688,
    },
    SelectorBinding {
        adr_id: "ADR-0511",
        path: "docs/decisions/ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md",
        predecessor_blob_oid: "71c94bfcf4af408b78a6469330fda078809685b9",
        sha256: "sha256:b3ba611721739b47c58aad68dd7337de0c1ca321752de2c9ed1ba15ead0723f6",
        byte_count: 13740,
    },
    SelectorBinding {
        adr_id: "ADR-0513",
        path: "docs/decisions/ADR-0513-oya-ci-bespoke-rust-prow-cicd-platform.md",
        predecessor_blob_oid: "1964ccc1e440bcffdcc90f7bef8d617d6df8329f",
        sha256: "sha256:823ea8c0c18e51863bdd8a20e490df639a69d9b005d809a761e97ed56a448ac4",
        byte_count: 7750,
    },
    SelectorBinding {
        adr_id: "ADR-0514",
        path: "docs/decisions/ADR-0514-build-ci-cd-pipeline-target-architecture-hyperscaler-remediation.md",
        predecessor_blob_oid: "92e60c3cacafee6f8c4942598ec10ff5ac8ed593",
        sha256: "sha256:b6704250a0f35277ff5f808a5f70c3a99ac9de4c4e59e76467609b01f663b4fa",
        byte_count: 15773,
    },
];
const ROOT_FIELDS: &[&str] = &[
    "profile",
    "planning_state",
    "planning_impact",
    "dispatch_authorized",
    "authority_claim",
    "source_adr",
    "predecessor_snapshot",
    "object_facts",
    "closure_contract",
];

/// The materialized facts face is deliberately closed: parsing first makes an
/// unknown field a fail-closed contract violation instead of silently widening
/// a preparation-only record.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PreparationFacts {
    profile: String,
    planning_state: String,
    planning_impact: bool,
    dispatch_authorized: bool,
    authority_claim: String,
    source_adr: SourceAdr,
    predecessor_snapshot: PredecessorSnapshot,
    object_facts: Vec<ObjectFact>,
    closure_contract: ClosureContract,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceAdr {
    id: String,
    supersedes: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PredecessorSnapshot {
    commit_oid: String,
    tree_oid: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectFact {
    adr_id: String,
    path: String,
    predecessor_blob_oid: String,
    sha256: String,
    byte_count: u64,
    live_body_exists: bool,
    exact_readable_copies: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClosureContract {
    required: bool,
    receipt_path: String,
    status: String,
}

pub fn evaluate_adr_0515_preparation_facts(facts: &Value) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();
    let fail = |findings: &mut BTreeSet<Finding>, key: &str| {
        findings.insert(Finding::new(ADR_0515_PREPARATION_CODE, key));
    };

    // Keep the typed contract at the trust boundary.  The detailed checks below
    // retain stable finding keys for every immutable field.
    if serde_json::from_value::<PreparationFacts>(facts.clone()).is_err() {
        fail(&mut findings, "typed_contract");
    }

    let Some(root) = facts.as_object() else {
        fail(&mut findings, "root");
        return findings;
    };
    for key in root.keys() {
        if !ROOT_FIELDS.contains(&key.as_str()) {
            fail(&mut findings, &format!("unknown_field.{key}"));
        }
    }
    if facts.get("profile").and_then(Value::as_str)
        != Some("adr-0515-superseded-ci-cluster-preparation")
    {
        fail(&mut findings, "profile");
    }
    if facts.get("planning_state").and_then(Value::as_str) != Some("HOLD(Planning)") {
        fail(&mut findings, "planning_state");
    }
    if facts.get("planning_impact").and_then(Value::as_bool) != Some(false) {
        fail(&mut findings, "planning_impact");
    }
    if facts.get("dispatch_authorized").and_then(Value::as_bool) != Some(false) {
        fail(&mut findings, "dispatch_authorized");
    }
    if facts.get("authority_claim").and_then(Value::as_str) != Some("none") {
        fail(&mut findings, "authority_claim");
    }
    if facts
        .pointer("/predecessor_snapshot/commit_oid")
        .and_then(Value::as_str)
        != Some(PREDECESSOR_COMMIT_OID)
    {
        fail(&mut findings, "predecessor_snapshot.commit_oid");
    }
    if facts
        .pointer("/predecessor_snapshot/tree_oid")
        .and_then(Value::as_str)
        != Some(PREDECESSOR_TREE_OID)
    {
        fail(&mut findings, "predecessor_snapshot.tree_oid");
    }

    let source = facts.get("source_adr");
    if source
        .and_then(|value| value.get("id"))
        .and_then(Value::as_str)
        != Some("ADR-0515")
    {
        fail(&mut findings, "source_adr.id");
    }
    let source_ids = source
        .and_then(|value| value.get("supersedes"))
        .and_then(Value::as_array);
    if source_ids.map_or(true, |ids| ids.len() != ADR_IDS.len()) {
        fail(&mut findings, "source_adr.supersedes.count");
    }
    let source_id_set = source_ids
        .map(|ids| {
            ids.iter()
                .filter_map(Value::as_str)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if source_id_set.len() != ADR_IDS.len()
        || source_id_set != ADR_IDS.into_iter().collect::<BTreeSet<_>>()
    {
        fail(&mut findings, "source_adr.supersedes");
    }
    if source_ids.map(|ids| ids.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        != Some(ADR_IDS.to_vec())
    {
        fail(&mut findings, "source_adr.supersedes.order");
    }

    let object_facts = facts.get("object_facts").and_then(Value::as_array);
    if object_facts.map_or(true, |entries| entries.len() != ADR_IDS.len()) {
        fail(&mut findings, "object_facts.count");
    }
    let mut fact_ids = BTreeSet::new();
    for (index, entry) in object_facts.into_iter().flatten().enumerate() {
        let prefix = format!("object_facts[{index}]");
        let Some(object) = entry.as_object() else {
            fail(&mut findings, &prefix);
            continue;
        };
        let allowed = [
            "adr_id",
            "path",
            "predecessor_blob_oid",
            "sha256",
            "byte_count",
            "live_body_exists",
            "exact_readable_copies",
        ];
        if object.keys().any(|key| !allowed.contains(&key.as_str())) {
            fail(&mut findings, &format!("{prefix}.unknown_field"));
        }
        let Some(id) = entry.get("adr_id").and_then(Value::as_str) else {
            fail(&mut findings, &format!("{prefix}.adr_id"));
            continue;
        };
        if !fact_ids.insert(id) || !ADR_IDS.contains(&id) || ADR_IDS.get(index) != Some(&id) {
            fail(&mut findings, &format!("{prefix}.adr_id"));
            continue;
        }
        let Some(binding) = SELECTOR_BINDINGS
            .iter()
            .find(|binding| binding.adr_id == id)
        else {
            fail(&mut findings, "selector_binding_alignment");
            continue;
        };
        for (field, expected) in [
            ("path", binding.path),
            ("predecessor_blob_oid", binding.predecessor_blob_oid),
            ("sha256", binding.sha256),
        ] {
            if entry.get(field).and_then(Value::as_str) != Some(expected) {
                fail(&mut findings, &format!("{prefix}.{field}"));
            }
        }
        if entry.get("byte_count").and_then(Value::as_u64) != Some(binding.byte_count) {
            fail(&mut findings, &format!("{prefix}.byte_count"));
        }
        if entry.get("live_body_exists").and_then(Value::as_bool) != Some(true) {
            fail(&mut findings, &format!("{prefix}.live_body_exists"));
        }
        if entry
            .get("exact_readable_copies")
            .and_then(Value::as_array)
            .map_or(true, |copies| !copies.is_empty())
        {
            fail(&mut findings, &format!("{prefix}.exact_readable_copies"));
        }
    }
    if fact_ids != ADR_IDS.into_iter().collect::<BTreeSet<_>>() {
        fail(&mut findings, "object_facts.adr_ids");
    }

    let closure = facts.get("closure_contract");
    if closure
        .and_then(|value| value.get("required"))
        .and_then(Value::as_bool)
        != Some(true)
    {
        fail(&mut findings, "closure_contract.required");
    }
    if closure
        .and_then(|value| value.get("receipt_path"))
        .and_then(Value::as_str)
        != Some("evidence/history-only-retirement/adr-0515-superseded-ci-cluster-closure.json")
    {
        fail(&mut findings, "closure_contract.receipt_path");
    }
    if closure
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        != Some("not-created-by-preparation")
    {
        fail(&mut findings, "closure_contract.status");
    }
    findings
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn bindings() -> [(&'static str, &'static str, &'static str, &'static str, u64); 7] {
        [
            (
                "ADR-0124",
                "docs/decisions/ADR-0124-own-merge-queue-webhook-driven.md",
                "c3af0c22453e58749aa7173c11167e0fb66e1412",
                "sha256:d57d67d3502ca013ba4e1b67fec77eb902609c2897331ac8c851c76c70ad45f8",
                10228,
            ),
            (
                "ADR-0349",
                "docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md",
                "313f75b8d6d7cad21f76d6afa1807c22587da198",
                "sha256:c63967bb8d8255d5b13ca9f880e7dbd05782ed46cf51042c94b80ff4742895c2",
                107783,
            ),
            (
                "ADR-0359",
                "docs/decisions/ADR-0359-jenkins-completely-replaces-github-actions.md",
                "d173ecaf58df93d76bca53d191dbaddfa8b7b115",
                "sha256:206818ac56bfb5a2589dd2fb51b38045fce314c05b3d0472db223c9b1cc4ba7c",
                4143,
            ),
            (
                "ADR-0361",
                "docs/decisions/ADR-0361-jenkins-native-cicd-revamp-execution.md",
                "55a4e35b6652fea476729137676e84dc1eabbe6c",
                "sha256:ab1129a35dba587f3d9910bf48f0d6078bc9d072cb0c76ca47504980efb23dc6",
                4688,
            ),
            (
                "ADR-0511",
                "docs/decisions/ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md",
                "71c94bfcf4af408b78a6469330fda078809685b9",
                "sha256:b3ba611721739b47c58aad68dd7337de0c1ca321752de2c9ed1ba15ead0723f6",
                13740,
            ),
            (
                "ADR-0513",
                "docs/decisions/ADR-0513-oya-ci-bespoke-rust-prow-cicd-platform.md",
                "1964ccc1e440bcffdcc90f7bef8d617d6df8329f",
                "sha256:823ea8c0c18e51863bdd8a20e490df639a69d9b005d809a761e97ed56a448ac4",
                7750,
            ),
            (
                "ADR-0514",
                "docs/decisions/ADR-0514-build-ci-cd-pipeline-target-architecture-hyperscaler-remediation.md",
                "92e60c3cacafee6f8c4942598ec10ff5ac8ed593",
                "sha256:b6704250a0f35277ff5f808a5f70c3a99ac9de4c4e59e76467609b01f663b4fa",
                15773,
            ),
        ]
    }

    fn valid() -> Value {
        json!({
            "profile": "adr-0515-superseded-ci-cluster-preparation",
            "planning_state": "HOLD(Planning)", "planning_impact": false,
            "dispatch_authorized": false, "authority_claim": "none",
            "source_adr": {"id": "ADR-0515", "supersedes": ADR_IDS},
            "predecessor_snapshot": {"commit_oid": "1fa09da22be819b062881eb59252f4dd4c6b550a", "tree_oid": "d7b15539396db21b219d68779362850cce9afa8f"},
            "object_facts": bindings().map(|(adr_id, path, predecessor_blob_oid, sha256, byte_count)| json!({"adr_id": adr_id, "path": path, "predecessor_blob_oid": predecessor_blob_oid, "sha256": sha256, "byte_count": byte_count, "live_body_exists": true, "exact_readable_copies": []})),
            "closure_contract": {"required": true, "receipt_path": "evidence/history-only-retirement/adr-0515-superseded-ci-cluster-closure.json", "status": "not-created-by-preparation"}
        })
    }

    #[test]
    fn valid_preparation_is_non_claiming() {
        assert!(evaluate_adr_0515_preparation_facts(&valid()).is_empty());
    }

    #[test]
    fn unknown_contract_fields_fail_closed() {
        let mut facts = valid();
        facts["unreviewed_extension"] = json!(true);
        let findings = evaluate_adr_0515_preparation_facts(&facts);
        assert!(findings.contains(&Finding::new(ADR_0515_PREPARATION_CODE, "typed_contract")));
    }

    #[test]
    fn selector_binding_table_is_exactly_aligned_with_canonical_ids() {
        assert_eq!(ADR_IDS.len(), SELECTOR_BINDINGS.len());
        assert!(
            ADR_IDS
                .iter()
                .zip(SELECTOR_BINDINGS.iter())
                .all(|(id, binding)| *id == binding.adr_id)
        );
    }

    #[test]
    fn authority_or_closure_drift_is_rejected() {
        let mut facts = valid();
        facts["authority_claim"] = json!("qualified-human");
        facts["closure_contract"]["status"] = json!("created");
        let findings = evaluate_adr_0515_preparation_facts(&facts);
        assert!(findings.contains(&Finding::new(ADR_0515_PREPARATION_CODE, "authority_claim")));
        assert!(findings.contains(&Finding::new(
            ADR_0515_PREPARATION_CODE,
            "closure_contract.status"
        )));
    }

    #[test]
    fn reordered_supersession_and_missing_closure_binding_fail_closed() {
        let mut facts = valid();
        facts["source_adr"]["supersedes"]
            .as_array_mut()
            .expect("array")
            .swap(0, 1);
        facts["closure_contract"]["required"] = json!(false);
        facts["closure_contract"]["receipt_path"] = json!("evidence/forged.json");
        let findings = evaluate_adr_0515_preparation_facts(&facts);
        assert!(findings.contains(&Finding::new(
            ADR_0515_PREPARATION_CODE,
            "source_adr.supersedes.order"
        )));
        assert!(findings.contains(&Finding::new(
            ADR_0515_PREPARATION_CODE,
            "closure_contract.required"
        )));
        assert!(findings.contains(&Finding::new(
            ADR_0515_PREPARATION_CODE,
            "closure_contract.receipt_path"
        )));
    }

    #[test]
    fn immutable_predecessor_and_each_selector_binding_are_required() {
        let mut predecessor = valid();
        predecessor["predecessor_snapshot"]["commit_oid"] =
            json!("0000000000000000000000000000000000000000");
        predecessor["predecessor_snapshot"]["tree_oid"] =
            json!("0000000000000000000000000000000000000000");
        let findings = evaluate_adr_0515_preparation_facts(&predecessor);
        assert!(findings.contains(&Finding::new(
            ADR_0515_PREPARATION_CODE,
            "predecessor_snapshot.commit_oid"
        )));
        assert!(findings.contains(&Finding::new(
            ADR_0515_PREPARATION_CODE,
            "predecessor_snapshot.tree_oid"
        )));

        for (field, mutation) in [
            ("path", json!("docs/decisions/wrong.md")),
            (
                "predecessor_blob_oid",
                json!("0000000000000000000000000000000000000000"),
            ),
            ("sha256", json!("sha256:wrong")),
            ("byte_count", json!(0)),
        ] {
            for index in 0..ADR_IDS.len() {
                let mut facts = valid();
                facts["object_facts"][index][field] = mutation.clone();
                assert!(
                    evaluate_adr_0515_preparation_facts(&facts).contains(&Finding::new(
                        ADR_0515_PREPARATION_CODE,
                        &format!("object_facts[{index}].{field}"),
                    )),
                    "{field} mutation at selector {index} must fail closed"
                );
            }
        }
    }
}
