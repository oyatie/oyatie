// GATE-1 cloud-ci-cross-artifact-agreement: RED/GREEN fixture corpus + born-blocking
// live-corpus self-test. ADR-0083 Tier-3: integration tests use unwrap/expect to assert
// invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod idea_archive_transition;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use ci_cross_artifact_agreement::{
    AdrDecisionRecord, GateCoverageBaseline, RatchetReport, RawHistoryOnlyRetirementReceipt,
    Verdict, derive_masterplan_md_projection, evaluate, evaluate_adr_index_projection_parity,
    evaluate_adr_prose_frontmatter_status,
    evaluate_and_project_history_only_retirement_facts_with_control_plane,
    evaluate_masterplan_plan_evidence_crosscheck, evaluate_masterplan_projection_rederivation,
    evaluate_masterplan_read_surface_resurrections, evaluate_masterplan_v2_authority,
    evaluate_masterplan_v2_entry_surfaces, evaluate_masterplan_v2_evidence_state,
    evaluate_masterplan_v2_plan_evidence_drift, evaluate_masterplan_v2_preplanning_candidate_facts,
    evaluate_masterplan_v2_program_coverage, evaluate_masterplan_v2_projection_freshness,
    evaluate_masterplan_v2_ratification_digest, evaluate_masterplan_v2_read_contract_archives,
    evaluate_masterplan_v2_sequencing, evaluate_registry_derived_policy_sync,
    normalize_closure_evidence_ref, ratchet,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Walk up to the repo root (the dir holding specs/root-hub-pointers.json), matching the
/// existing kernel-test convention.
fn repo_root() -> PathBuf {
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

fn producer_binary(root: &Path, producer_bin: Option<&str>) -> Result<PathBuf, String> {
    let Some(bin) = producer_bin else {
        return Err(
            "FAIL-CLOSED: missing OYA_CI_PRODUCER_BIN; Cargo fallback is forbidden".to_owned(),
        );
    };
    Ok(if Path::new(bin).is_absolute() {
        PathBuf::from(bin)
    } else {
        root.join(bin)
    })
}

#[test]
fn producer_binary_env_is_required_for_hermetic_gate() {
    let err = producer_binary(Path::new("/repo"), None)
        .expect_err("missing OYA_CI_PRODUCER_BIN must fail closed");
    assert!(err.contains("OYA_CI_PRODUCER_BIN"));
}

fn fixture_dir() -> PathBuf {
    repo_root().join("specs/fixtures/cross-artifact-agreement")
}

fn load_json(path: &PathBuf) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn named_workflow_step<'a>(workflow: &'a str, name: &str) -> &'a str {
    let marker = format!("      - name: {name}");
    let start = workflow
        .find(&marker)
        .unwrap_or_else(|| panic!("workflow step {name}"));
    let tail = &workflow[start..];
    let end = tail.find("\n      - name: ").unwrap_or(tail.len());
    &tail[..end]
}

fn workflow_job<'a>(workflow: &'a str, job_name: &str) -> &'a str {
    let marker = format!("  {job_name}:\n");
    let start = workflow
        .find(&marker)
        .unwrap_or_else(|| panic!("workflow job {job_name}"));
    let tail = &workflow[start..];
    let mut cursor = marker.len();
    let end = loop {
        let Some(offset) = tail[cursor..].find("\n  ") else {
            break tail.len();
        };
        let candidate = cursor + offset;
        if tail
            .as_bytes()
            .get(candidate + 3)
            .is_some_and(u8::is_ascii_alphanumeric)
        {
            break candidate;
        }
        cursor = candidate + 3;
    };
    &tail[..end]
}

fn named_job_step<'a>(job: &'a str, name: &str) -> &'a str {
    named_workflow_step(job, name)
}

/// Drop whole-line YAML comments so the counting assertions below measure EFFECTIVE workflow
/// content, never prose.
///
/// These assertions count raw substrings, and a comment is part of that text — so a comment that
/// merely QUOTES an asserted literal silently pushes the count to 2 and reds the gate. That is not
/// hypothetical: #1469 added a comment documenting the failure string
/// `Artifact not found for name: generated-faces`, which made `name: generated-faces` occur twice
/// inside the producer's upload step. This gate then reported a workflow-topology defect that did
/// not exist, on a workflow whose topology was correct.
///
/// A gate whose verdict depends on comment prose is measuring the wrong thing. Documenting a
/// string must never be indistinguishable from declaring it. The second half of this same test
/// already had the right discipline (exact-line equality against `line.trim()`); this extends it
/// to the substring assertions rather than leaving two standards in one file.
///
/// Only WHOLE-LINE comments are dropped. A trailing `#` inside a value is left alone: stripping it
/// needs a YAML-aware scanner, and no assertion here targets one.
fn effective_yaml(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

fn count_effective(haystack: &str, needle: &str) -> usize {
    effective_yaml(haystack).match_indices(needle).count()
}

fn assert_occurs_exactly_once(haystack: &str, needle: &str) {
    assert_eq!(
        count_effective(haystack, needle),
        1,
        "{needle:?} must occur exactly once (comments excluded)"
    );
}

fn assert_absent(haystack: &str, needle: &str, why: &str) {
    assert_eq!(
        count_effective(haystack, needle),
        0,
        "{needle:?} must NOT appear (comments excluded): {why}"
    );
}

fn prefixed_sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => value.to_string(),
        Value::String(_) => serde_json::to_string(value).expect("serialize fixture string"),
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Object(values) => {
            let ordered = values.iter().collect::<BTreeMap<_, _>>();
            format!(
                "{{{}}}",
                ordered
                    .into_iter()
                    .map(|(key, value)| format!(
                        "{}:{}",
                        serde_json::to_string(key).expect("serialize fixture key"),
                        canonical_json(value)
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
    }
}

fn declared_raw_history_only_receipts(root: &Path, facts: &Value) -> Vec<(String, Vec<u8>)> {
    facts["receipts"]
        .as_array()
        .expect("history-only facts receipts array")
        .iter()
        .map(|metadata| {
            let receipt_path = metadata["receipt_path"]
                .as_str()
                .expect("history-only receipt metadata path");
            assert!(
                Path::new(receipt_path)
                    .components()
                    .all(|component| matches!(component, Component::Normal(_))),
                "history-only receipt path must be repo-relative and canonical: {receipt_path}"
            );
            let path = root.join(receipt_path);
            let bytes = fs::read(&path).unwrap_or_else(|error| {
                panic!("read declared raw receipt {}: {error}", path.display())
            });
            (receipt_path.to_owned(), bytes)
        })
        .collect()
}

fn installed_dormant_history_only_facts_fixture(control_plane_bytes: &[u8]) -> Value {
    let control_plane: Value =
        serde_json::from_slice(control_plane_bytes).expect("fixture control plane parses");
    let entries = control_plane["entries"]
        .as_array()
        .expect("fixture control-plane entries")
        .clone();
    let entry_hashes = entries
        .iter()
        .map(|entry| {
            serde_json::json!({
                "scope_ref": entry["scope_ref"],
                "sha256": prefixed_sha256(canonical_json(entry).as_bytes())
            })
        })
        .collect::<Vec<_>>();
    let control_plane_sha256 = prefixed_sha256(control_plane_bytes);
    let control_plane_byte_count = control_plane_bytes.len();
    let control_plane_blob_oid = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let predecessor_commit_oid = control_plane["predecessor_snapshot"]["commit_oid"]
        .as_str()
        .expect("fixture predecessor commit");
    let predecessor_tree_oid = control_plane["predecessor_snapshot"]["tree_oid"]
        .as_str()
        .expect("fixture predecessor tree");
    serde_json::json!({
        "receipts": [],
        "scm_facts": {
            "retirement_receipt_coverage": {
                "protected_base_ref": "origin/dev",
                "protected_receipt_paths": [],
                "candidate_receipt_paths": [],
                "carried_receipt_paths": [],
                "new_receipt_paths": [],
                "scopes": [],
                "required_retired_paths": []
            },
            "retirement_receipt_object_facts": [],
            "protected_scm_context": {
                "protected_base_ref": "origin/dev",
                "protected_base_commit_oid": "1111111111111111111111111111111111111111",
                "protected_base_tree_oid": "2222222222222222222222222222222222222222",
                "evaluated_commit_oid": "3333333333333333333333333333333333333333",
                "evaluated_tree_oid": "4444444444444444444444444444444444444444",
                "subject_commit_oid": "3333333333333333333333333333333333333333",
                "subject_tree_oid": "4444444444444444444444444444444444444444",
                "scm_event_name": "push",
                "subject_relationship": "evaluated-self",
                "protected_base_is_ancestor_of_evaluated": true,
                "protected_base_is_evaluated_first_parent": true,
                "subject_is_evaluated_second_parent": false,
                "predecessor_commit_oid": predecessor_commit_oid,
                "predecessor_tree_oid": predecessor_tree_oid,
                "predecessor_commit_exists": true,
                "predecessor_tree_exists": true,
                "predecessor_commit_tree_bound": true,
                "predecessor_is_ancestor_of_protected_base": true,
                "protected_preparation_receipts": []
            },
            "retirement_control_plane_context": {
                "control_plane_path": "registry/history-only-retirement/control-plane.json",
                "receipt_root": "evidence/history-only-retirement",
                "bootstrap": false,
                "protected_control_plane_blob_oid": control_plane_blob_oid,
                "protected_control_plane_sha256": control_plane_sha256,
                "protected_control_plane_byte_count": control_plane_byte_count,
                "candidate_control_plane_blob_oid": control_plane_blob_oid,
                "candidate_control_plane_sha256": control_plane_sha256,
                "candidate_control_plane_byte_count": control_plane_byte_count,
                "control_plane_entries": entries,
                "control_plane_entry_hashes": entry_hashes,
                "protected_receipt_root_paths": [],
                "candidate_receipt_root_paths": [],
                "unexpected_protected_receipt_paths": [],
                "unexpected_candidate_receipt_paths": []
            }
        }
    })
}

#[test]
fn protected_scm_context_excludes_candidate_authored_facts() {
    let control_plane =
        fs::read(repo_root().join("registry/history-only-retirement/control-plane.json"))
            .expect("read canonical retirement control plane");
    let facts = installed_dormant_history_only_facts_fixture(&control_plane);
    let protected_context = facts["scm_facts"]["protected_scm_context"]
        .as_object()
        .expect("protected SCM context object");

    for candidate_field in ["prepared_receipt_paths", "control_plane_entries"] {
        assert!(
            !protected_context.contains_key(candidate_field),
            "{candidate_field} is candidate-authored and must not be labeled protected"
        );
    }
}

#[test]
fn retirement_sources_do_not_silently_amend_accepted_adr_0613() {
    let root = repo_root();
    let adr = fs::read_to_string(root.join("docs/decisions/ADR-0700-ci-admission-live-apex.md"))
        .expect("read accepted ADR-0613");
    assert!(
        !adr.contains("### E7 history-only retirement facts"),
        "implementation provenance must not silently widen an Accepted ADR"
    );

    let reachability: Value =
        serde_json::from_slice(&fs::read(root.join("specs/reachability-registry.json")).unwrap())
            .expect("parse reachability registry");
    let rows = reachability["registered"]
        .as_array()
        .expect("registered reachability rows");
    // `registry/history-only-retirement/OWNERS` was in this list until OWNERS files became
    // accounted BY CONSTRUCTION (`OWNERS_SCHEMA_ANCHOR`): its registry row — like all 49
    // OWNERS rows — is deleted, so there is no anchor left that could claim an unrecorded
    // ADR-0613 amendment for it. The prose ceiling it carried is not weakened by the removal,
    // it is made unnecessary: the derived anchor is a fixed literal no author can write prose
    // into. The four rows that still exist are still checked, and
    // `owners_files_are_never_registered_in_the_reachability_registry` (total-accounting gate)
    // is what keeps the OWNERS row from coming back.
    for prefix in [
        "ci/facade/scm-facts-snapshot/src/retirement.rs",
        "registry/history-only-retirement/control-plane.json",
        "specs/history-only-retirement-control-plane.schema.json",
        "specs/history-only-retirement-facts.schema.json",
    ] {
        let anchor = rows
            .iter()
            .find(|row| row["prefix"].as_str() == Some(prefix))
            .and_then(|row| row["anchor"].as_str())
            .unwrap_or_else(|| panic!("reachability anchor for {prefix}"));
        assert!(
            !anchor.contains("ADR-0613"),
            "{prefix} must not claim an unrecorded ADR-0613 amendment"
        );
        assert!(
            anchor.contains("HOLD(Planning)"),
            "{prefix} must retain the authority ceiling"
        );
    }
}

/// Accepted portal authority must remain a single coherent contract. The generic bridge roadmap
/// may retain an import transition, but must not resurrect Backstage as a parallel runtime.
#[test]
fn portal_authority_keeps_backstage_one_way_and_runtime_free() {
    let root = repo_root();
    let first_party =
        fs::read_to_string(root.join("docs/adr-archive/ADR-0394-bespoke-rust-idp-central-hub.md"))
            .expect("read ADR-0394");
    let roadmap =
        fs::read_to_string(root.join("docs/adr-archive/ADR-0482-bespoke-substrate-roadmap.md"))
            .expect("read ADR-0482");

    assert!(
        first_party.contains("amends: [ADR-0482]")
            && roadmap.contains("amended_by: [kubers-anchor-2026-05-28, ADR-0394,"),
        "ADR-0394 and ADR-0482 must carry reciprocal amendment edges"
    );
    assert!(
        first_party.contains("bounded one-way import source")
            && first_party.contains("It is not a runtime dependency, plugin host"),
        "ADR-0394 must keep Backstage bounded to one-way import and forbid runtime authority"
    );
    assert!(
        roadmap.contains(
            "Bounded one-way import of Backstage Catalog YAML; no Backstage runtime or plugin host"
        ) && !roadmap.contains("| Rust-native portal (ADR-0434) | Backstage (ADR-0410) |"),
        "ADR-0482 must not resurrect the retired Backstage runtime bridge"
    );
}

/// ADR-0614 was accepted after its resolver hardening shipped, and ADR-0616 then reversed the
/// committed frozen-reference posture. Stale proposal-era prose would make live authority lie.
#[test]
fn move_manifest_authority_matches_fail_closed_resolver_and_adr_0616() {
    let root = repo_root();
    let move_manifest = fs::read_to_string(
        root.join("docs/adr-archive/ADR-0614-de-commit-reorg-move-manifest-bijection.md"),
    )
    .expect("read ADR-0614");
    let frozen_reference = fs::read_to_string(
        root.join("docs/adr-archive/ADR-0616-de-commit-firewall-frozen-reference-baseline.md"),
    )
    .expect("read ADR-0616");

    assert!(
        move_manifest.contains("amended_by: [ADR-0616]")
            && frozen_reference.contains("amends: [ADR-0604, ADR-0614]"),
        "ADR-0614 and ADR-0616 must carry reciprocal amendment edges"
    );
    for stale in [
        "still return `Self` infallibly",
        "mapping absent → `empty()`",
        "Latent hazard recorded, not dropped",
        "De-commit AND refactor `load` to fail-closed in one PR",
        "Reaffirms ADR-0596",
    ] {
        assert!(
            !move_manifest.contains(stale),
            "ADR-0614 contains stale authority phrase {stale:?}"
        );
    }
    assert!(
        move_manifest.contains("return `Result` and hard-error when")
            && move_manifest
                .contains("ADR-0616 supersedes ADR-0596's committed frozen-reference rule"),
        "ADR-0614 must describe the shipped fail-closed resolver and current ADR-0616 posture"
    );
    for stale in [
        "is the firewall ratchet's FROZEN reference and the LAST committed generated",
        "The emitter reads it as the committed blob",
        "src/main.rs:737",
    ] {
        assert!(
            !frozen_reference.contains(stale),
            "ADR-0616 contains stale live-baseline claim {stale:?}"
        );
    }
    assert!(
        frozen_reference.contains("Before this decision")
            && frozen_reference
                .contains("regenerates the baseline twice from the immutable merge-base source")
            && frozen_reference.contains("resolve_merge_base_baseline_snapshot")
            && frozen_reference.contains("provides no committed-face")
            && frozen_reference.contains("`git show` fallback"),
        "ADR-0616 must distinguish historical committed state from immutable-source regeneration"
    );
}

/// ADR-0565 is the one-way zero-GraphQL authority. The two older API ADRs may retain only
/// explicitly historical/rejected discussion and must expose normalized reciprocal edges,
/// including later product-protocol amendments.
#[test]
fn zero_graphql_authority_has_reciprocal_edges_and_no_live_surface() {
    let root = repo_root();
    let zero_graphql = fs::read_to_string(
        root.join("docs/adr-archive/ADR-0565-zero-graphql-in-the-owned-api-surface.md"),
    )
    .expect("read ADR-0565");
    let network = fs::read_to_string(
        root.join("docs/adr-archive/ADR-0253-network-topology-edge-service-mesh.md"),
    )
    .expect("read ADR-0253");
    let versioning =
        fs::read_to_string(root.join("docs/adr-archive/ADR-0258-api-versioning-model.md"))
            .expect("read ADR-0258");

    assert!(
        zero_graphql.contains("amends: [ADR-0056, ADR-0105, ADR-0253, ADR-0258]")
            && network.contains("amended_by:\n  - ADR-0565\n")
            && versioning.contains("amended_by: [ADR-0565, ADR-0632]"),
        "ADR-0565, ADR-0253, and ADR-0258 must carry normalized reciprocal amendment edges"
    );
    for stale in [
        "D-14 (OpenAPI 3.2 + GraphQL Fed + gRPC + AsyncAPI)",
        "GraphQL is a fraction of the public surface",
        "the GraphQL deprecation directive remains available within GraphQL surfaces",
        "public REST/gRPC/AsyncAPI/~~GraphQL~~ surface",
    ] {
        assert!(
            !network.contains(stale) && !versioning.contains(stale),
            "older API authority retains live GraphQL claim {stale:?}"
        );
    }
    assert!(
        network.contains("historical rejected context only")
            && versioning.contains("GraphQL is historical rejected context only")
            && versioning.contains(
                "GraphQL Federation as the version-management substrate (historical, rejected)"
            ),
        "surviving GraphQL prose must be explicitly historical or rejected"
    );
}

fn normalizes_to_public_grpc_contradiction(text: &str) -> bool {
    let normalized = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    [
        "public rest/grpc",
        "public http/grpc",
        "external http/grpc",
        "public grpc is",
        "public grpc api",
        "public grpc surface",
        "public µservice rpc (http + grpc",
        "proto3 services exposed externally",
        "proto3 reserved field oyatie_version",
    ]
    .iter()
    .any(|contradiction| normalized.contains(contradiction))
}

/// ADR-0203 owns the self-contained public documentation boundary in this authority epoch:
/// OpenAPI REST plus AsyncAPI event/webhook/streaming references, with Protobuf descriptors
/// visible only to internal service owners. Every Accepted protocol ADR in the explicit authority
/// set, ADR-0258, and the absorbed sequencing sidecar must preserve that boundary.
#[test]
fn public_protocol_authority_keeps_grpc_and_proto_internal() {
    let root = repo_root();
    let documentation = fs::read_to_string(
        root.join("docs/adr-archive/ADR-0203-documentation-engine-three-tier.md"),
    )
    .expect("read ADR-0203");
    let versioning =
        fs::read_to_string(root.join("docs/adr-archive/ADR-0258-api-versioning-model.md"))
            .expect("read ADR-0258");
    let sequencing = load_json(&root.join("specs/master-plan-sequencing.json"));
    let protocol_adrs = [
        "docs/adr-archive/ADR-0157-api-gateway-tier.md",
        "docs/adr-archive/ADR-0167-tenant-cli.md",
        "docs/adr-archive/ADR-0176-brownout-degradation-signal-api.md",
        "docs/adr-archive/ADR-0182-api-gateway-north-south-vs-service-mesh-east-west-separation.md",
    ];

    // Historical protocol ADRs may be Superseded by apex; authority text + reciprocal
    // related edges remain the binding corpus check (live apex is ADR-0705/0709).
    assert!(
        (documentation.contains("- Status: Accepted")
            || documentation.contains("status: Superseded"))
            && documentation.contains("ADR-0258 (API versioning model)")
            && (versioning.contains("status: Accepted")
                || versioning.contains("status: Superseded"))
            && versioning.contains("ADR-0203")
            && versioning.contains("## ADR-0203 public-contract reconciliation"),
        "ADR-0203 and ADR-0258 must stay related and explicitly reconciled (Accepted or Superseded historical)"
    );
    for required in [
        "OpenAPI 3.2 REST",
        "AsyncAPI\n3.1 event, webhook, and streaming references",
        "Public gRPC or proto3 exposure is not authorized",
        "internal service-to-service\nRPC under mTLS",
        "displaying internal Protobuf descriptors does not create a public contract",
    ] {
        assert!(
            versioning.contains(required),
            "ADR-0258 must preserve the reconciled protocol invariant {required:?}"
        );
    }
    assert!(
        documentation.contains(
            "Protobuf descriptors may be shown for internal service owners but do not create a public gRPC\n  contract"
        ),
        "ADR-0203 must keep Protobuf documentation internal-only"
    );

    for path in protocol_adrs {
        let adr = fs::read_to_string(root.join(path))
            .unwrap_or_else(|error| panic!("read protocol authority {path}: {error}"));
        assert!(
            (adr.contains("status: Accepted") || adr.contains("status: Superseded"))
                && adr.contains("last_reconciled: 2026-08-01")
                && adr.contains("reconciled_with: [ADR-0203, ADR-0258, ADR-0632]")
                && adr.contains("### Public-contract reconciliation"),
            "{path} must remain Accepted and carry reciprocal protocol reconciliation metadata"
        );
        for required in [
            "REST documented by OpenAPI 3.2",
            "webhooks, events, and streams documented by AsyncAPI 3.1",
            "gRPC over HTTP/2 (H2) with proto3 is",
            "internal-only service-to-service traffic under mTLS",
            "it is not a public API contract",
        ] {
            assert!(
                adr.contains(required),
                "{path} must preserve protocol invariant {required:?}"
            );
        }
        assert!(
            !normalizes_to_public_grpc_contradiction(&adr),
            "{path} reintroduced contradictory public gRPC authority"
        );
    }

    assert!(
        !normalizes_to_public_grpc_contradiction(&versioning),
        "ADR-0258 reintroduced contradictory public gRPC authority"
    );

    let rendered_plan = serde_json::to_string(&sequencing)
        .expect("sequencing must serialize")
        .to_ascii_lowercase();
    for contradiction in [
        "public rest/asyncapi/proto3",
        "proto3 services exposed externally",
        "proto3 reserved field oyatie_version",
        "versionsservice",
        "contracts/*.proto",
    ] {
        assert!(
            !rendered_plan.contains(contradiction),
            "sequencing reintroduced public RPC authority {contradiction:?}"
        );
    }
    let wave = &sequencing["realignment_wave_sequence"]["waves_15_plus"]["sub_wave_landings"]["15V-API-Versioning-Adoption"];
    let rendered_wave = serde_json::to_string(wave)
        .expect("15V must serialize")
        .to_ascii_lowercase();
    for required in [
        "openapi 3.2.0",
        "signed/versioned webhook",
        "asyncapi 3.1.0",
        "sse",
        "websocket",
        "internal-mesh grpc/proto3",
        "exempt",
    ] {
        assert!(
            rendered_wave.contains(required),
            "15V must preserve the carrier boundary {required:?}"
        );
    }
}

#[test]
fn public_protocol_guard_rejects_known_public_grpc_mutations() {
    for mutation in [
        "Public REST/gRPC is supported.",
        "Public HTTP/gRPC API.",
        "Every external HTTP/gRPC request enters here.",
        "Every public µservice RPC (HTTP + gRPC + AsyncAPI) emits metadata.",
        "Public gRPC is a Tier-A contract.",
        "Proto3 services exposed externally.",
        "Proto3 reserved field oyatie_version.",
    ] {
        assert!(
            normalizes_to_public_grpc_contradiction(mutation),
            "public protocol guard false-green for mutation {mutation:?}"
        );
    }
}

/// Developer-SDK IP file bindings are optional. When present, each binding must identify an exact
/// implementation-plan artifact for that row; broad doctrine notes and ADRs cannot stand in for it.
#[test]
fn developer_sdk_ip_file_bindings_are_exact_implementation_plans() {
    let root = repo_root();
    let manifest = load_json(&root.join("marketplace/developer-sdk/manifest.json"));
    let ips = manifest["ips"].as_array().expect("developer-sdk ips array");
    let mut ids = BTreeSet::new();

    for row in ips {
        let id = row["id"].as_str().expect("developer-sdk IP id");
        assert!(ids.insert(id), "duplicate developer-sdk IP id {id}");

        let Some(file) = row.get("file") else {
            continue;
        };
        let file = file
            .as_str()
            .unwrap_or_else(|| panic!("developer-sdk IP {id} file must be a string"));
        let expected_path = format!("marketplace/developer-sdk/implementation-plans/{id}.md");
        assert_eq!(
            file, expected_path,
            "developer-sdk IP {id} may bind only its exact implementation-plan artifact"
        );
        assert!(
            root.join(file).is_file(),
            "developer-sdk IP {id} file must resolve to {file}"
        );

        let artifact = fs::read_to_string(root.join(file))
            .unwrap_or_else(|error| panic!("read developer-sdk IP {id} file {file}: {error}"));
        let frontmatter = artifact
            .strip_prefix("---\n")
            .and_then(|tail| {
                tail.split_once("\n---\n")
                    .map(|(frontmatter, _)| frontmatter)
            })
            .unwrap_or_else(|| panic!("developer-sdk IP {id} file must have YAML frontmatter"));
        let exact_id = format!("id: {id}");
        assert!(
            frontmatter.lines().any(|line| line.trim() == exact_id),
            "developer-sdk IP {id} file must declare its exact identity"
        );
        assert!(
            frontmatter
                .lines()
                .any(|line| line.trim() == "artifact_class: implementation-plan"),
            "developer-sdk IP {id} file must declare artifact_class: implementation-plan"
        );
    }
}

/// Legacy Backstage-shaped records are bounded one-way import data, but their source URLs must
/// still resolve to the canonical post-reorg compliance artifacts.
#[test]
fn compliance_catalog_import_urls_resolve_to_canonical_paths() {
    let root = repo_root();
    let prefix = "https://github.com/oyadev/oyatie/blob/main/";
    for (catalog_path, canonical_path) in [
        (
            "compliance/catalog/api-asyncapi.yaml",
            "compliance/contracts/asyncapi.yaml",
        ),
        (
            "compliance/catalog/api-rest.yaml",
            "compliance/contracts/openapi.yaml",
        ),
        (
            "compliance/catalog/component-info.yaml",
            "compliance/manifest.json",
        ),
    ] {
        let catalog = fs::read_to_string(root.join(catalog_path))
            .unwrap_or_else(|error| panic!("read {catalog_path}: {error}"));
        let url = format!("{prefix}{canonical_path}");
        assert!(
            catalog.contains(&url) && !catalog.contains("microservices/compliance/"),
            "{catalog_path} must reference canonical compliance path {canonical_path}"
        );
        assert!(
            root.join(canonical_path).is_file(),
            "catalog reference must resolve to {canonical_path}"
        );
    }
}

#[test]
fn production_evaluator_rejects_installed_dormant_control_plane_byte_drift() {
    let control_plane =
        fs::read(repo_root().join("registry/history-only-retirement/control-plane.json"))
            .expect("read canonical retirement control plane");
    let mut facts = installed_dormant_history_only_facts_fixture(&control_plane);
    facts["scm_facts"]["retirement_control_plane_context"]["candidate_control_plane_byte_count"] =
        serde_json::json!(control_plane.len() + 1);

    let evaluation = evaluate_and_project_history_only_retirement_facts_with_control_plane(
        &facts,
        &[],
        &control_plane,
    );
    assert!(
        evaluation.findings.iter().any(|finding| {
            finding.key == "retirement_control_plane_context.candidate_raw_binding"
        }),
        "candidate control-plane byte drift must fail closed through the production evaluator: {:?}",
        evaluation.findings
    );
    assert!(evaluation.projection.is_none());
}

#[test]
fn production_evaluator_accepts_installed_dormant_facts() {
    let control_plane =
        fs::read(repo_root().join("registry/history-only-retirement/control-plane.json"))
            .expect("read canonical retirement control plane");
    let facts = installed_dormant_history_only_facts_fixture(&control_plane);
    let evaluation = evaluate_and_project_history_only_retirement_facts_with_control_plane(
        &facts,
        &[],
        &control_plane,
    );
    assert!(
        evaluation.findings.is_empty(),
        "controller-bound dormant facts must validate through the production evaluator: {:?}",
        evaluation.findings
    );
    assert!(
        evaluation
            .projection
            .expect("validated dormant projection")
            .evidence_set_ids()
            .is_empty()
    );
}

#[test]
fn production_evaluator_rejects_candidate_control_plane_extensions() {
    let canonical =
        fs::read(repo_root().join("registry/history-only-retirement/control-plane.json"))
            .expect("read canonical retirement control plane");
    let mut extended: Value =
        serde_json::from_slice(&canonical).expect("parse canonical retirement control plane");
    extended["unexpected_candidate_authority"] = serde_json::json!(true);
    let extended_bytes =
        serde_json::to_vec(&extended).expect("serialize extended retirement control plane");
    let facts = installed_dormant_history_only_facts_fixture(&extended_bytes);

    let evaluation = evaluate_and_project_history_only_retirement_facts_with_control_plane(
        &facts,
        &[],
        &extended_bytes,
    );
    assert!(
        evaluation.findings.iter().any(|finding| {
            finding.key == "retirement_control_plane_context.candidate_raw_header"
        }),
        "unexpected raw control-plane fields must fail closed: {:?}",
        evaluation.findings
    );
    assert!(evaluation.projection.is_none());
}

#[test]
fn production_evaluator_rejects_candidate_predecessor_drift() {
    let control_plane =
        fs::read(repo_root().join("registry/history-only-retirement/control-plane.json"))
            .expect("read canonical retirement control plane");
    let mut facts = installed_dormant_history_only_facts_fixture(&control_plane);
    facts["scm_facts"]["protected_scm_context"]["predecessor_commit_oid"] =
        serde_json::json!("5555555555555555555555555555555555555555");

    let evaluation = evaluate_and_project_history_only_retirement_facts_with_control_plane(
        &facts,
        &[],
        &control_plane,
    );
    assert!(
        evaluation.findings.iter().any(|finding| {
            finding.key == "retirement_control_plane_context.candidate_raw_predecessor_binding"
        }),
        "raw predecessor identity must bind the materialized SCM context: {:?}",
        evaluation.findings
    );
    assert!(evaluation.projection.is_none());
}

#[test]
fn production_evaluator_rejects_candidate_control_plane_entry_reordering() {
    let canonical =
        fs::read(repo_root().join("registry/history-only-retirement/control-plane.json"))
            .expect("read canonical retirement control plane");
    let mut reordered: Value =
        serde_json::from_slice(&canonical).expect("parse canonical retirement control plane");
    reordered["entries"]
        .as_array_mut()
        .expect("control-plane entries")
        .swap(0, 1);
    let reordered_bytes =
        serde_json::to_vec(&reordered).expect("serialize reordered retirement control plane");
    let facts = installed_dormant_history_only_facts_fixture(&reordered_bytes);

    let evaluation = evaluate_and_project_history_only_retirement_facts_with_control_plane(
        &facts,
        &[],
        &reordered_bytes,
    );
    assert!(
        evaluation.findings.iter().any(|finding| {
            finding.key == "retirement_control_plane_context.candidate_raw_entries"
        }),
        "canonical control-plane entry order must fail closed: {:?}",
        evaluation.findings
    );
    assert!(evaluation.projection.is_none());
}

#[test]
fn production_evaluator_rejects_malformed_or_duplicate_control_plane_hash_rows() {
    let control_plane =
        fs::read(repo_root().join("registry/history-only-retirement/control-plane.json"))
            .expect("read canonical retirement control plane");
    let facts = installed_dormant_history_only_facts_fixture(&control_plane);

    let mut malformed = facts.clone();
    malformed["scm_facts"]["retirement_control_plane_context"]["control_plane_entry_hashes"][0]["unexpected"] =
        serde_json::json!(true);
    let mut duplicate = facts;
    let duplicate_row =
        duplicate["scm_facts"]["retirement_control_plane_context"]["control_plane_entry_hashes"][0]
            .clone();
    duplicate["scm_facts"]["retirement_control_plane_context"]["control_plane_entry_hashes"]
        .as_array_mut()
        .expect("control-plane hash rows")
        .push(duplicate_row);

    for drifted in [&malformed, &duplicate] {
        let evaluation = evaluate_and_project_history_only_retirement_facts_with_control_plane(
            drifted,
            &[],
            &control_plane,
        );
        assert!(
            evaluation.findings.iter().any(|finding| {
                finding.key == "retirement_control_plane_context.candidate_raw_hashes"
            }),
            "malformed or duplicate control-plane hash rows must fail closed: {:?}",
            evaluation.findings
        );
        assert!(evaluation.projection.is_none());
    }
}

#[test]
fn production_evaluator_rejects_reordered_control_plane_hash_rows() {
    let control_plane =
        fs::read(repo_root().join("registry/history-only-retirement/control-plane.json"))
            .expect("read canonical retirement control plane");
    let mut reordered = installed_dormant_history_only_facts_fixture(&control_plane);
    reordered["scm_facts"]["retirement_control_plane_context"]["control_plane_entry_hashes"]
        .as_array_mut()
        .expect("control-plane hash rows")
        .swap(0, 1);

    let evaluation = evaluate_and_project_history_only_retirement_facts_with_control_plane(
        &reordered,
        &[],
        &control_plane,
    );
    assert!(
        evaluation.findings.iter().any(|finding| {
            finding.key == "retirement_control_plane_context.candidate_raw_hashes"
        }),
        "reordered control-plane hash rows must fail closed: {:?}",
        evaluation.findings
    );
    assert!(evaluation.projection.is_none());
}

#[test]
fn production_evaluator_rejects_duplicate_raw_control_plane_keys() {
    let canonical =
        fs::read_to_string(repo_root().join("registry/history-only-retirement/control-plane.json"))
            .expect("read canonical retirement control plane");
    let duplicate = canonical.replacen('{', "{\"schema_version\":1,", 1);
    let facts = installed_dormant_history_only_facts_fixture(duplicate.as_bytes());

    let evaluation = evaluate_and_project_history_only_retirement_facts_with_control_plane(
        &facts,
        &[],
        duplicate.as_bytes(),
    );
    assert!(
        evaluation.findings.iter().any(|finding| {
            finding.key == "retirement_control_plane_context.candidate_raw_parse"
        }),
        "duplicate raw control-plane keys must fail closed: {:?}",
        evaluation.findings
    );
    assert!(evaluation.projection.is_none());
}

#[test]
fn production_evaluator_accepts_dormant_bootstrap_without_a_protected_blob_claim() {
    let control_plane =
        fs::read(repo_root().join("registry/history-only-retirement/control-plane.json"))
            .expect("read canonical retirement control plane");
    let mut facts = installed_dormant_history_only_facts_fixture(&control_plane);
    let context = &mut facts["scm_facts"]["retirement_control_plane_context"];
    context["bootstrap"] = serde_json::json!(true);
    context["protected_control_plane_blob_oid"] = Value::Null;
    context["protected_control_plane_sha256"] = Value::Null;
    context["protected_control_plane_byte_count"] = Value::Null;

    let evaluation = evaluate_and_project_history_only_retirement_facts_with_control_plane(
        &facts,
        &[],
        &control_plane,
    );
    assert!(
        evaluation.findings.is_empty(),
        "bootstrap raw-source facts must remain non-claiming and valid: {:?}",
        evaluation.findings
    );
    assert!(
        evaluation
            .projection
            .expect("validated dormant bootstrap projection")
            .evidence_set_ids()
            .is_empty()
    );
}

#[test]
fn history_only_retirement_control_plane_declares_workflow_and_event_identity_inputs() {
    let manifest = load_json(&repo_root().join("registry/generated-artifact-control-plane.json"));
    let row = manifest["artifacts"]
        .as_array()
        .and_then(|rows| {
            rows.iter().find(|row| {
                row.get("artifact_id").and_then(Value::as_str)
                    == Some("history-only-retirement-facts")
            })
        })
        .expect("history-only retirement facts control-plane row");
    let inputs = row["source_inputs"]
        .as_array()
        .expect("history-only retirement source inputs array");
    for required in [
        ".github/workflows/oya-ci-required.yml",
        "specs/history-only-retirement-facts.schema.json",
    ] {
        assert!(
            inputs.iter().any(|input| input.as_str() == Some(required)),
            "source_inputs must declare {required}"
        );
    }
    let contract = row["generator"]["input_contract"]
        .as_array()
        .expect("history-only retirement generator input contract array");
    assert!(
        contract
            .iter()
            .any(|input| input.as_str() == Some("scm-event-identity")),
        "generator input contract must declare scm-event-identity"
    );
}

#[test]
fn retirement_workflow_transports_the_provider_tuple_once_and_all_candidate_regenerators_use_it() {
    let workflow = fs::read_to_string(repo_root().join(".github/workflows/oya-ci-required.yml"))
        .expect("read oya-ci-required workflow");
    assert_occurs_exactly_once(&workflow, "merge_group:\n    types: [checks_requested]");

    for (key, binding) in [
        (
            "EVENT_EVALUATED_SHA:",
            "EVENT_EVALUATED_SHA: ${{ github.sha }}",
        ),
        (
            "EVENT_PULL_REQUEST_BASE_SHA:",
            "EVENT_PULL_REQUEST_BASE_SHA: ${{ github.event.pull_request.base.sha || '' }}",
        ),
        (
            "EVENT_PULL_REQUEST_HEAD_SHA:",
            "EVENT_PULL_REQUEST_HEAD_SHA: ${{ github.event.pull_request.head.sha || '' }}",
        ),
        (
            "EVENT_PUSH_BEFORE_SHA:",
            "EVENT_PUSH_BEFORE_SHA: ${{ github.event.before || '' }}",
        ),
        (
            "EVENT_PUSH_AFTER_SHA:",
            "EVENT_PUSH_AFTER_SHA: ${{ github.event.after || '' }}",
        ),
        (
            "EVENT_MERGE_GROUP_BASE_SHA:",
            "EVENT_MERGE_GROUP_BASE_SHA: ${{ github.event.merge_group.base_sha || '' }}",
        ),
        (
            "EVENT_MERGE_GROUP_HEAD_SHA:",
            "EVENT_MERGE_GROUP_HEAD_SHA: ${{ github.event.merge_group.head_sha || '' }}",
        ),
        ("EVENT_NAME:", "EVENT_NAME: ${{ github.event_name }}"),
        ("EVENT_REF:", "EVENT_REF: ${{ github.ref }}"),
        (
            "EVENT_PULL_REQUEST_BASE_REF:",
            "EVENT_PULL_REQUEST_BASE_REF: ${{ github.event.pull_request.base.ref || '' }}",
        ),
        (
            "EVENT_MERGE_GROUP_BASE_REF:",
            "EVENT_MERGE_GROUP_BASE_REF: ${{ github.event.merge_group.base_ref || '' }}",
        ),
    ] {
        assert_occurs_exactly_once(&workflow, key);
        assert_occurs_exactly_once(&workflow, binding);
    }
    let test_job = workflow_job(&workflow, "test");
    let producer = named_job_step(test_job, "Materialize generated faces");
    assert_occurs_exactly_once(producer, "--github-event");
    assert!(
        producer.contains(
            "cargo run --locked -p ci-generated-artifact-freshness --bin oya-cloud-ci-materialize-generated-faces -- --repo-root . --github-event"
        ),
        "the cargo materializer must own provider-tuple interpretation"
    );
    let workspace_tests = named_job_step(test_job, "Workspace tests");
    assert!(
        workspace_tests.contains("cargo test --locked --workspace"),
        "the gate fleet, scm-facts census receipt included, must run under the cargo workspace tests"
    );
    assert_absent(
        &workflow,
        "matrix.crate",
        "the buck2 gate matrix is retired; no buck2 test step may survive in the merge path",
    );
    let candidate_materializer_lines = workflow
        .lines()
        .filter(|line| {
            (line.contains("oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .")
                || line.contains("oya-cloud-ci-materialize-generated-faces -- --repo-root ."))
                && !line.contains("--help")
                && !line.contains("historical_retirement_args")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        candidate_materializer_lines.len(),
        2,
        "test job and cross-platform smoke must both materialize faces; extra unbound invocations must be enumerated"
    );
    for line in candidate_materializer_lines {
        assert!(
            line.contains("--github-event"),
            "candidate materializer must be provider-event-bound: {line}"
        );
    }
    for legacy_binding in [
        "EVENT_PROTECTED_SHA:",
        "EVENT_SUBJECT_SHA:",
        "EVENT_BASE_REF:",
        "--retirement-control-plane",
        "--retirement-facts-out",
        "--protected-base-commit",
        "--evaluated-commit",
        "--scm-event-name",
        "--scm-event-ref",
        "--scm-event-base-ref",
        "--subject-commit",
        "git rev-list",
        "git rev-parse",
        "git cat-file",
        "HEAD^1",
    ] {
        assert!(
            !producer.contains(legacy_binding),
            "candidate materialization must pass the complete GitHub event tuple to Rust, not retain legacy manual identity binding {legacy_binding:?}"
        );
    }
}

#[test]
fn broad_workflow_consumers_require_the_producer_artifact_and_keep_the_merge_base_historical() {
    let workflow = fs::read_to_string(repo_root().join(".github/workflows/oya-ci-required.yml"))
        .expect("read oya-ci-required workflow");

    // THE INVARIANT: a broad consumer must HOLD the generated faces before it tests.
    // ADR-0716 retired producer-regen / artifact download; every remaining consumer
    // self-materializes in-job. The affected-set historical merge-base lane retired
    // with that job (`cargo test --workspace` is the affected set).
    assert_absent(
        &workflow,
        "  producer-regen:",
        "producer-regen is retired; consumers self-materialize instead of downloading faces",
    );
    assert_absent(
        &workflow,
        "Download regenerated faces",
        "artifact hop is retired; downloading faces a job immediately overwrites is dead weight",
    );
    assert_absent(
        &workflow,
        "  gate-affected-target-set:",
        "the affected-set lane is retired; cargo test --workspace is the affected set (ADR-0716)",
    );
    assert_absent(
        &workflow,
        "--historical-merge-base",
        "historical merge-base materialization lived on the retired affected-set lane",
    );

    let materialize_step_name = "Materialize generated faces";
    let cargo_materializer = "cargo run --locked -p ci-generated-artifact-freshness --bin oya-cloud-ci-materialize-generated-faces -- --repo-root . --github-event";
    for (job_name, broad_step) in [
        ("test", "Workspace tests"),
        ("cross-platform-smoke", "Cross-platform smoke tests"),
    ] {
        let job = workflow_job(&workflow, job_name);
        assert_occurs_exactly_once(job, &format!("- name: {materialize_step_name}"));
        assert!(
            job.find(materialize_step_name) < job.find(broad_step),
            "{job_name} must materialize the faces before its broad test"
        );
        let materialize = named_job_step(job, materialize_step_name);
        assert!(
            materialize.contains(cargo_materializer),
            "{job_name} materializer must be provider-event-bound"
        );
        for legacy_topology_shell in [
            "git cat-file",
            "git rev-list",
            "git rev-parse",
            "HEAD^1",
            "--retirement-control-plane",
            "--retirement-facts-out",
            "--protected-base-commit",
            "--evaluated-commit",
            "--scm-event-name",
            "--scm-event-ref",
            "--scm-event-base-ref",
            "--subject-commit",
        ] {
            assert!(
                !materialize.contains(legacy_topology_shell),
                "materialization must delegate topology and event identity to Rust, not retain shell authority {legacy_topology_shell:?}"
            );
        }
    }
}

#[test]
fn live_history_only_retirement_facts_are_bound_to_the_controller_control_plane() {
    let root = repo_root();
    let relative_path = std::env::var("OYA_HISTORY_ONLY_RETIREMENT_FACTS")
        .expect("FAIL-CLOSED: OYA_HISTORY_ONLY_RETIREMENT_FACTS must name the materialized face");
    assert_eq!(
        relative_path, "ci/facade/scm-facts-snapshot/history-only-retirement-facts.generated.json",
        "history-only retirement facts must use the canonical controller-owned path"
    );
    let facts_path = root.join(&relative_path);
    let facts_bytes = fs::read(&facts_path)
        .unwrap_or_else(|error| panic!("read materialized {}: {error}", facts_path.display()));
    let facts: Value = serde_json::from_slice(&facts_bytes)
        .unwrap_or_else(|error| panic!("parse materialized {}: {error}", facts_path.display()));
    let control_plane_path = root.join("registry/history-only-retirement/control-plane.json");
    let control_plane_bytes = fs::read(&control_plane_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", control_plane_path.display()));

    let raw_storage = declared_raw_history_only_receipts(&root, &facts);
    let raw_receipts = raw_storage
        .iter()
        .map(|(receipt_path, bytes)| RawHistoryOnlyRetirementReceipt {
            receipt_path,
            bytes,
        })
        .collect::<Vec<_>>();
    let evaluation = evaluate_and_project_history_only_retirement_facts_with_control_plane(
        &facts,
        &raw_receipts,
        &control_plane_bytes,
    );
    assert!(
        evaluation.findings.is_empty(),
        "history-only facts plus declared raw receipts failed: {:?}",
        evaluation.findings
    );
    assert!(
        evaluation
            .projection
            .expect("validated history-only projection")
            .evidence_set_ids()
            .is_empty(),
        "dormant live facts must not project a closure"
    );
}

fn expected_violations(fixture: &Value) -> BTreeSet<String> {
    fixture["expected_violations"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn architecture_rule_set(architecture: &Value, context: &str) -> BTreeSet<String> {
    if let Some(required_rules) = architecture.get("required_rules") {
        return string_array_set(required_rules, &format!("{context}.required_rules"));
    }

    let mut rules = BTreeSet::new();
    for key in [
        "required_microservice_rules",
        "required_clean_architecture_rules",
        "required_api_first_rules",
        "required_hyperscaler_pattern_rules",
    ] {
        if let Some(values) = architecture.get(key) {
            rules.extend(string_array_set(values, &format!("{context}.{key}")));
        }
    }
    assert!(
        !rules.is_empty(),
        "{context} must define required_rules or split required_*_rules arrays"
    );
    rules
}

fn string_array_set(value: &Value, context: &str) -> BTreeSet<String> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("{context} must be an array"))
        .iter()
        .map(|item| {
            item.as_str()
                .unwrap_or_else(|| panic!("{context} contains a non-string item"))
                .to_owned()
        })
        .collect()
}

fn missing_from<'a>(expected: &'a BTreeSet<String>, actual: &'a BTreeSet<String>) -> Vec<&'a str> {
    expected
        .difference(actual)
        .map(String::as_str)
        .collect::<Vec<_>>()
}

fn assert_same_rule_set(
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
    left_name: &str,
    right_name: &str,
) {
    assert_eq!(
        left,
        right,
        "{left_name} and {right_name} architecture rule sets drifted; missing_from_{left_name}={:?}; missing_from_{right_name}={:?}",
        missing_from(right, left),
        missing_from(left, right)
    );
}

#[test]
fn cross_artifact_fixtures_execute_red_green_cases() {
    let dir = fixture_dir();
    let mut tc_paths: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("tc-") && n.ends_with(".json"))
        })
        .collect();
    tc_paths.sort();
    assert!(
        !tc_paths.is_empty(),
        "cross-artifact-agreement fixture corpus must not be empty"
    );

    let mut seen_green = false;
    let mut seen_red = false;

    for path in &tc_paths {
        let fixture = load_json(path);
        let report = evaluate(&fixture);
        let expected = expected_violations(&fixture);
        let label = path.file_name().unwrap().to_string_lossy().to_string();

        match fixture["expected_verdict"].as_str() {
            Some("GREEN") => {
                seen_green = true;
                assert_eq!(
                    report.verdict,
                    Verdict::Green,
                    "{label} should be GREEN, got violations {:?}",
                    report.violations
                );
                assert!(
                    report.violations.is_empty(),
                    "{label} GREEN must have zero violations, got {:?}",
                    report.violations
                );
            }
            Some("RED") => {
                seen_red = true;
                assert_eq!(report.verdict, Verdict::Red, "{label} should be RED");
                assert_eq!(report.violations, expected, "{label} violations mismatch");
            }
            other => panic!("{label} has unsupported expected_verdict {other:?}"),
        }
    }

    assert!(
        seen_green && seen_red,
        "cross-artifact-agreement fixtures must include BOTH RED and GREEN cases"
    );
}

#[test]
fn masterplan_v2_live_authority_contract_is_green() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let findings = evaluate_masterplan_v2_authority(&masterplan);
    assert!(
        findings.is_empty(),
        "masterplan v2 authority contract must stay green: {findings:?}"
    );
}

/// Sub-AC 4.1 masterplan structural gate: the frozen fixture corpus must keep one
/// ISOLATED fail-closed RED fixture per structural failure class — duplicate
/// work-item ids, dependency cycles, and dangling (orphan) dependency references.
/// The generic runner above only demands "some RED fixture exists"; this test pins
/// each named failure mode to its exact violation set so none can be silently
/// dropped or diluted.
#[test]
fn masterplan_structural_failure_mode_fixtures_fail_closed() {
    let cases: [(&str, &[&str]); 3] = [
        (
            "tc-XA-bad-masterplan-duplicate-work-item-id.json",
            &["masterplan_work_item_id_collision"],
        ),
        (
            "tc-XA-bad-masterplan-dependency-cycle.json",
            &[
                "masterplan_dependency_dag_invalid",
                "masterplan_sequencing_invalid",
            ],
        ),
        (
            "tc-XA-bad-masterplan-dangling-dependency-ref.json",
            &["masterplan_dependency_dag_invalid"],
        ),
    ];

    for (fixture_name, expected_codes) in cases {
        let path = fixture_dir().join(fixture_name);
        assert!(
            path.is_file(),
            "structural failure-mode fixture must exist: {}",
            path.display()
        );
        let fixture = load_json(&path);
        let report = evaluate(&fixture);
        assert_eq!(
            report.verdict,
            Verdict::Red,
            "{fixture_name} must fail closed (RED)"
        );
        let expected: BTreeSet<String> = expected_codes
            .iter()
            .map(|code| (*code).to_owned())
            .collect();
        assert_eq!(
            report.violations, expected,
            "{fixture_name} must emit exactly the pinned structural violation set"
        );
        assert_eq!(
            expected_violations(&fixture),
            expected,
            "{fixture_name} expected_violations must stay in sync with the pinned set"
        );
    }
}
#[test]
fn masterplan_v2_external_completion_claims_are_unverified_until_evidence_attaches() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let findings = evaluate_masterplan_v2_evidence_state(&masterplan);
    assert!(
        findings.is_empty(),
        "masterplan v2 evidence-state policy must keep unverified external completion claims out of done: {findings:?}"
    );

    if let Some(imports) =
        masterplan["masterplan_v2"]["external_work_item_claim_imports"].as_array()
    {
        for claim in imports {
            let refs = claim["evidence_refs"].as_array().unwrap_or_else(|| {
                panic!("each external completion claim must carry an evidence_refs array")
            });
            if refs.is_empty() {
                assert_eq!(
                    claim["masterplan_status"].as_str(),
                    Some("claimed-done-unverified")
                );
                assert_eq!(
                    claim["evidence_state"].as_str(),
                    Some("claimed-done-unverified")
                );
            }
        }
    }
}

/// Sub-AC 3 verifiability clause over the optional provider-neutral import:
/// absence is clean, malformed presence fails closed, and imported completion
/// claims cannot carry verified status without recorded evidence.
#[test]
fn masterplan_v2_external_completion_claim_import_is_optional_and_fail_closed() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let mut absent = masterplan.clone();
    absent["masterplan_v2"]
        .as_object_mut()
        .expect("masterplan_v2 must be an object")
        .remove("external_work_item_claim_imports");
    assert!(evaluate_masterplan_v2_evidence_state(&absent).is_empty());

    let mut malformed = absent;
    malformed["masterplan_v2"]["external_work_item_claim_imports"] =
        serde_json::json!("not-an-array");
    let findings = evaluate_masterplan_v2_evidence_state(&malformed);
    assert!(findings.iter().any(|finding| {
        finding.code == "masterplan_evidence_state_invalid"
            && finding.key == "masterplan_v2.external_work_item_claim_imports"
    }));
}
#[test]
fn masterplan_v2_plan_vs_evidence_drift_contract_is_green() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let findings = evaluate_masterplan_v2_plan_evidence_drift(&masterplan);
    assert!(
        findings.is_empty(),
        "masterplan v2 plan-vs-evidence drift policy must stay green: {findings:?}"
    );

    assert_eq!(
        masterplan["masterplan_v2"]["evidence_state_policy"]["validator"].as_str(),
        Some("cloud-ci-cross-artifact-agreement/masterplan-v2-plan-vs-evidence-drift"),
        "masterplan v2 must name the plan-vs-evidence drift validator as the evidence-state policy writer"
    );
}

#[test]
fn masterplan_v2_current_preplanning_candidate_matches_cited_evidence() {
    let (masterplan, evidence) = live_preplanning_candidate_fixture();

    let findings = evaluate_masterplan_v2_preplanning_candidate_facts(&masterplan, &evidence);
    assert!(
        findings.is_empty(),
        "masterplan current-candidate facts must agree with their cited time-scoped receipt: {findings:?}"
    );
}

#[test]
fn closed_planning_entry_preserves_the_historical_open_candidate_receipt() {
    let (mut masterplan, evidence) = live_preplanning_candidate_fixture();
    let contract = &mut masterplan["masterplan_v2"]["planning_entry_contract"];
    contract["state"] = serde_json::json!("closed");
    contract["binding_plan_approval_allowed"] = serde_json::json!(true);
    contract["dispatch_allowed"] = serde_json::json!(true);
    contract["closure_evidence"] = serde_json::json!({});

    let findings = evaluate_masterplan_v2_preplanning_candidate_facts(&masterplan, &evidence);
    assert!(
        findings.is_empty(),
        "the closure transition closes through its separate closure-evidence chain \
         without rewriting the digest-pinned historical candidate receipt: {findings:?}"
    );
}

#[test]
fn preplanning_candidate_paired_missing_fields_fail_closed() {
    let (mut masterplan, mut evidence) = live_preplanning_candidate_fixture();
    masterplan["masterplan_v2"]["planning_entry_contract"]["current_pr_candidate_state"]
        .as_object_mut()
        .expect("current candidate state must be an object")
        .remove("protected_pr_number");
    evidence["present"]["repository_baseline"]
        .as_object_mut()
        .expect("repository baseline must be an object")
        .remove("protected_pr_number");

    assert_preplanning_candidate_drift_reason(&masterplan, &evidence, "missing_or_malformed");
}

#[test]
fn preplanning_candidate_wrong_field_types_fail_closed() {
    let (mut masterplan, mut evidence) = live_preplanning_candidate_fixture();
    masterplan["masterplan_v2"]["planning_entry_contract"]["current_pr_candidate_state"]["protected_pr_number"] =
        serde_json::json!("1340");
    evidence["present"]["repository_baseline"]["protected_pr_number"] = serde_json::json!("1340");

    assert_preplanning_candidate_drift(&masterplan, &evidence);
}

#[test]
fn preplanning_candidate_contract_field_drift_has_a_keyed_reason() {
    let (mut masterplan, evidence) = live_preplanning_candidate_fixture();
    masterplan["masterplan_v2"]["planning_entry_contract"]["state"] =
        serde_json::json!("unsupported");

    assert_preplanning_candidate_drift_reason(&masterplan, &evidence, "field_mismatch");
}

#[test]
fn preplanning_candidate_baseline_must_match_immutable_pr_facts() {
    let (mut masterplan, mut evidence) = live_preplanning_candidate_fixture();
    let divergent_base = serde_json::json!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    masterplan["masterplan_v2"]["planning_entry_contract"]["current_pr_candidate_state"]["baseline_commit"] =
        divergent_base.clone();
    masterplan["masterplan_v2"]["planning_entry_contract"]["current_pr_candidate_state"]["candidate_base"] =
        divergent_base.clone();
    evidence["present"]["repository_baseline"]["pr_base"] = divergent_base;

    assert_preplanning_candidate_drift(&masterplan, &evidence);
}

#[test]
fn preplanning_candidate_review_must_be_approved_on_final_head() {
    let (masterplan, mut evidence) = live_preplanning_candidate_fixture();
    evidence["present"]["factual_reconciliation"]["github_approved_review_receipt"]["state"] =
        serde_json::json!("CHANGES_REQUESTED");
    evidence["present"]["factual_reconciliation"]["github_approved_review_receipt"]["commit_sha"] =
        serde_json::json!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");

    assert_preplanning_candidate_drift(&masterplan, &evidence);
}

#[test]
fn preplanning_candidate_pr_identity_cannot_move_in_lockstep() {
    let (mut masterplan, mut evidence) = live_preplanning_candidate_fixture();
    masterplan["masterplan_v2"]["planning_entry_contract"]["current_pr_candidate_state"]["protected_pr_number"] =
        serde_json::json!(9999);
    masterplan["masterplan_v2"]["planning_entry_contract"]["current_pr_candidate_state"]["protected_pr_url"] =
        serde_json::json!("https://github.com/jason931225/oyatie/pull/9999");
    evidence["present"]["repository_baseline"]["protected_pr_number"] = serde_json::json!(9999);
    evidence["present"]["repository_baseline"]["protected_pr_url"] =
        serde_json::json!("https://github.com/jason931225/oyatie/pull/9999");
    evidence["present"]["factual_reconciliation"]["immutable_pull_request_facts"]["number"] =
        serde_json::json!(9999);
    evidence["present"]["factual_reconciliation"]["immutable_pull_request_facts"]["url"] =
        serde_json::json!("https://github.com/jason931225/oyatie/pull/9999");

    assert_preplanning_candidate_drift(&masterplan, &evidence);
}

#[test]
fn preplanning_candidate_state_cannot_move_in_lockstep() {
    let (mut masterplan, mut evidence) = live_preplanning_candidate_fixture();
    let false_state = serde_json::json!("merged-and-all-authority-closed");
    masterplan["masterplan_v2"]["planning_entry_contract"]["current_pr_candidate_state"]["recorded_candidate_state"] =
        false_state.clone();
    evidence["present"]["repository_baseline"]["candidate_state"] = false_state;
    evidence["present"]["factual_reconciliation"]["immutable_pull_request_facts"]["candidate_state"] =
        serde_json::json!("merged-and-all-authority-closed");

    assert_preplanning_candidate_drift(&masterplan, &evidence);
}

#[test]
fn preplanning_candidate_claim_ceiling_cannot_move_in_lockstep() {
    let (mut masterplan, mut evidence) = live_preplanning_candidate_fixture();
    let false_claim = serde_json::json!("PR #1340 closes every authority and product gate.");
    masterplan["masterplan_v2"]["planning_entry_contract"]["current_pr_candidate_state"]["claim_ceiling"] =
        false_claim.clone();
    evidence["present"]["repository_baseline"]["claim_ceiling"] = false_claim;
    evidence["present"]["factual_reconciliation"]["immutable_pull_request_facts"]["claim_ceiling"] =
        serde_json::json!("PR #1340 closes every authority and product gate.");

    assert_preplanning_candidate_drift(&masterplan, &evidence);
}

#[test]
fn preplanning_candidate_first_commit_cannot_move_in_lockstep() {
    let (mut masterplan, mut evidence) = live_preplanning_candidate_fixture();
    let false_first_commit = serde_json::json!("cccccccccccccccccccccccccccccccccccccccc");
    masterplan["masterplan_v2"]["planning_entry_contract"]["current_pr_candidate_state"]["candidate_first_content_commit"] =
        false_first_commit.clone();
    evidence["present"]["repository_baseline"]["candidate_first_content_commit"] =
        false_first_commit.clone();
    evidence["present"]["repository_baseline"]["pr_opened_on_head"] = false_first_commit;

    assert_preplanning_candidate_drift(&masterplan, &evidence);
}

#[test]
fn preplanning_candidate_final_head_cannot_move_in_lockstep_with_its_receipts() {
    let (mut masterplan, mut evidence) = live_preplanning_candidate_fixture();
    let false_head = serde_json::json!("dddddddddddddddddddddddddddddddddddddddd");
    masterplan["masterplan_v2"]["planning_entry_contract"]["current_pr_candidate_state"]["candidate_final_head"] =
        false_head.clone();
    evidence["present"]["repository_baseline"]["pr_final_head"] = false_head.clone();
    evidence["present"]["factual_reconciliation"]["immutable_pull_request_facts"]["head_sha"] =
        false_head.clone();
    evidence["present"]["factual_reconciliation"]["github_approved_review_receipt"]["commit_sha"] =
        false_head.clone();
    evidence["present"]["factual_reconciliation"]["protected_context_receipts"][0]["commit_sha"] =
        false_head;

    assert_preplanning_candidate_drift(&masterplan, &evidence);
}

#[test]
fn preplanning_candidate_merge_sha_cannot_move_in_lockstep_with_its_receipt() {
    let (mut masterplan, mut evidence) = live_preplanning_candidate_fixture();
    let false_merge = serde_json::json!("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
    masterplan["masterplan_v2"]["planning_entry_contract"]["current_pr_candidate_state"]["merge_commit"] =
        false_merge.clone();
    evidence["present"]["repository_baseline"]["merge_commit"] = false_merge.clone();
    evidence["present"]["factual_reconciliation"]["immutable_pull_request_facts"]["merge_commit_sha"] =
        false_merge.clone();
    evidence["present"]["factual_reconciliation"]["protected_context_receipts"][1]["commit_sha"] =
        false_merge;

    assert_preplanning_candidate_drift(&masterplan, &evidence);
}

#[test]
fn preplanning_candidate_review_receipt_cannot_mutate_in_lockstep() {
    let (mut masterplan, mut evidence) = live_preplanning_candidate_fixture();
    masterplan["masterplan_v2"]["planning_entry_contract"]["current_pr_candidate_state"]["github_approved_reviewer"] =
        serde_json::json!("replacement-reviewer");
    evidence["present"]["factual_reconciliation"]["github_approved_review_receipt"]["review_id"] =
        serde_json::json!(999_999);
    evidence["present"]["factual_reconciliation"]["github_approved_review_receipt"]["reviewer"] =
        serde_json::json!("replacement-reviewer");
    evidence["present"]["factual_reconciliation"]["github_approved_review_receipt"]["submitted_at"] =
        serde_json::json!("2026-07-14T00:00:00Z");
    evidence["present"]["factual_reconciliation"]["github_approved_review_receipt"]["url"] =
        serde_json::json!("https://github.com/jason931225/oyatie/pull/1340#replacement");

    assert_preplanning_candidate_drift(&masterplan, &evidence);
}

#[test]
fn preplanning_candidate_protected_context_receipt_cannot_mutate() {
    let (masterplan, mut evidence) = live_preplanning_candidate_fixture();
    evidence["present"]["factual_reconciliation"]["protected_context_receipts"][0]["details_url"] =
        serde_json::json!("https://github.com/jason931225/oyatie/actions/runs/replaced");

    assert_preplanning_candidate_drift_reason(&masterplan, &evidence, "candidate_receipt_digest");
}

fn live_preplanning_candidate_fixture() -> (Value, Value) {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let evidence_ref =
        masterplan["masterplan_v2"]["planning_entry_contract"]["current_pr_candidate"]
            .as_str()
            .expect("current_pr_candidate must cite a repository-relative evidence file");
    let evidence = load_json(&root.join(evidence_ref));
    (masterplan, evidence)
}

fn assert_preplanning_candidate_drift(masterplan: &Value, evidence: &Value) {
    let findings = evaluate_masterplan_v2_preplanning_candidate_facts(masterplan, evidence);
    assert!(
        findings.iter().any(|finding| {
            finding.code == "masterplan_plan_evidence_drift"
                && finding.key.starts_with(
                    "masterplan_v2.planning_entry_contract.current_pr_candidate_state.",
                )
        }),
        "candidate drift must fail closed: {findings:?}"
    );
}

fn assert_preplanning_candidate_drift_reason(masterplan: &Value, evidence: &Value, reason: &str) {
    let findings = evaluate_masterplan_v2_preplanning_candidate_facts(masterplan, evidence);
    assert!(
        findings.iter().any(|finding| {
            finding.code == "masterplan_plan_evidence_drift"
                && finding.key
                    == format!(
                        "masterplan_v2.planning_entry_contract.current_pr_candidate_state.{reason}"
                    )
        }),
        "candidate drift must identify reason {reason}: {findings:?}"
    );
}

/// Sub-AC 4.3 plan-vs-evidence cross-check lane, born-blocking over the live
/// tree: every masterplan work-item status claim and evidence-attached external
/// import must cross-check against RECORDED completion evidence. The
/// resolution universe is the committed scm-facts face `tracked_paths` (the
/// same declared input the producer reads), so a dangling evidence pointer, a
/// ref at a retired (absorbed / archived-with-provenance) surface, or a
/// verified 'done' claim without a merged commit / merged-PR record /
/// tracked product-completion packet anywhere in the live masterplan turns
/// this test RED.
#[test]
fn masterplan_plan_evidence_crosscheck_gate_is_green_on_live_tree() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let scm_facts =
        load_json(&root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json"));
    let tracked_paths = scm_facts
        .get("tracked_paths")
        .cloned()
        .expect("committed scm-facts face must carry tracked_paths");
    let corpus = serde_json::json!({ "tracked_paths": tracked_paths });

    let findings = evaluate_masterplan_plan_evidence_crosscheck(&masterplan, &corpus);
    assert!(
        findings.is_empty(),
        "masterplan v2 plan-vs-evidence cross-check must stay green on the live tree: {findings:?}"
    );

    let crosscheck =
        &masterplan["masterplan_v2"]["evidence_state_policy"]["plan_evidence_crosscheck"];
    assert_eq!(
        crosscheck["validator"].as_str(),
        Some("cloud-ci-cross-artifact-agreement/masterplan-v2-plan-evidence-crosscheck"),
        "masterplan v2 must declare the plan-evidence cross-check validator"
    );
    assert_eq!(
        crosscheck["violation_code"].as_str(),
        Some("masterplan_plan_evidence_unrecorded"),
        "masterplan v2 must pin the cross-check violation code"
    );
    assert_eq!(
        crosscheck["resolution_universe"].as_str(),
        Some("ci/facade/artifact-inventory-registry/scm-facts.generated.json#tracked_paths"),
        "masterplan v2 must pin the tracked-tree resolution universe this test reads"
    );
}

/// Sub-AC 4.3 fail-closed pins: the frozen fixture corpus must keep one
/// ISOLATED RED fixture per plan-vs-evidence cross-check failure class — an
/// unevidenced verified-'done' claim, evidence pointing at a retired surface,
/// and a dangling evidence pointer — each emitting exactly
/// `masterplan_plan_evidence_unrecorded`.
#[test]
fn masterplan_plan_evidence_crosscheck_fixtures_fail_closed() {
    for fixture_name in [
        "tc-XA-bad-masterplan-evidence-unrecorded-done-claim.json",
        "tc-XA-bad-masterplan-evidence-retired-surface.json",
        "tc-XA-bad-masterplan-evidence-dangling-ref.json",
    ] {
        let path = fixture_dir().join(fixture_name);
        assert!(
            path.is_file(),
            "plan-evidence cross-check failure-mode fixture must exist: {}",
            path.display()
        );
        let fixture = load_json(&path);
        let report = evaluate(&fixture);
        assert_eq!(
            report.verdict,
            Verdict::Red,
            "{fixture_name} must fail closed (RED)"
        );
        let expected: BTreeSet<String> =
            std::iter::once("masterplan_plan_evidence_unrecorded".to_owned()).collect();
        assert_eq!(
            report.violations, expected,
            "{fixture_name} must emit exactly the unrecorded-evidence violation"
        );
        assert_eq!(
            expected_violations(&fixture),
            expected,
            "{fixture_name} expected_violations must stay in sync with the pinned set"
        );
    }
}
#[test]
fn masterplan_v2_program_coverage_contract_is_green() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let manifest_index = load_json(&root.join("specs/microservices/manifests-index.json"));
    let findings = evaluate_masterplan_v2_program_coverage(&masterplan, &manifest_index);
    assert!(
        findings.is_empty(),
        "masterplan v2 program coverage must cover every manifest-index microservice: {findings:?}"
    );

    // Machine-checked coverage audit: exact set equality between the manifest
    // index enumerated at consolidation time and the program-sharded coverage
    // rows — no enumerated surface may be missing and no phantom row may exist.
    let coverage = &masterplan["masterplan_v2"]["program_coverage"];
    let covered: BTreeSet<&str> = coverage["microservices"]
        .as_array()
        .expect("program_coverage.microservices must be an array")
        .iter()
        .filter_map(|entry| entry["microservice"].as_str())
        .collect();
    let indexed: BTreeSet<&str> = manifest_index["microservices"]
        .as_array()
        .expect("manifest index microservices must be an array")
        .iter()
        .filter_map(|entry| entry["name"].as_str())
        .collect();
    assert_eq!(
        covered, indexed,
        "program coverage must be exact set coverage over /specs/microservices/manifests-index.json at consolidation time"
    );

    // The ADR-0537 owned-stack ladder must be covered rung-for-rung in order.
    let rung_layers: Vec<&str> = coverage["owned_stack_ladder"]["rungs"]
        .as_array()
        .expect("program_coverage.owned_stack_ladder.rungs must be an array")
        .iter()
        .filter_map(|rung| rung["layer"].as_str())
        .collect();
    assert_eq!(
        rung_layers,
        [
            "cloud-kernel",
            "cloud-os",
            "cloud-k8s",
            "cloud-services",
            "products"
        ],
        "owned-stack ladder coverage must enumerate every ADR-0537 rung in ladder order"
    );

    // Pillar and program shards the consolidation must explicitly carry.
    let program_ids: BTreeSet<&str> = masterplan["masterplan_v2"]["programs"]
        .as_array()
        .expect("masterplan_v2.programs must be an array")
        .iter()
        .filter_map(|program| program["id"].as_str())
        .collect();
    for required in [
        "P-FD001-PRODUCT-SURFACES",
        "P-ONTOLOGY",
        "P-WORKFLOW-ENGINE",
        "P-WORKFLOW-STUDIO",
        "P-INTELLIGENCE",
        "P-OWNED-STACK-KERNEL",
        "P-OWNED-STACK-OS",
        "P-OWNED-STACK-K8S",
        "P-OWNED-STACK-CLOUD",
        "P-OWNED-STACK-DURABILITY",
        "P-OWNED-STACK-GOVERNANCE-IAM-CONSOLE",
        "P-REORG",
        "P-AST-CODE-GRAPH",
        "P-FABRIC",
    ] {
        assert!(
            program_ids.contains(required),
            "missing required program shard {required}"
        );
    }
}

#[test]
fn masterplan_v2_projection_freshness_contract_is_green() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let generated_artifacts =
        load_json(&root.join("registry/generated-artifact-control-plane.json"));
    let findings =
        evaluate_masterplan_v2_projection_freshness(&masterplan, Some(&generated_artifacts));
    assert!(
        findings.is_empty(),
        "masterplan v2 projection freshness must cover every generated/read projection: {findings:?}"
    );

    let projections = masterplan["masterplan_v2"]["projection_freshness"]["projections"]
        .as_array()
        .expect("projection_freshness.projections must be an array");
    let covered_paths: BTreeSet<&str> = projections
        .iter()
        .filter_map(|projection| projection["path"].as_str())
        .collect();
    let expected_paths = expected_masterplan_projection_paths(&masterplan, &generated_artifacts);
    assert_eq!(
        covered_paths, expected_paths,
        "projection_freshness.projections must be exact set coverage over every generated/read projection derived from specs/masterplan.json"
    );
}
#[test]
fn masterplan_v2_read_contract_archive_gate_is_green() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let findings = evaluate_masterplan_v2_read_contract_archives(&masterplan);
    assert!(
        findings.is_empty(),
        "archived stale read paths must only be referenced as provenance archives: {findings:?}"
    );
}

#[test]
fn masterplan_v2_entry_surface_allowlist_gate_is_green() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let root_hub = load_json(&root.join("specs/root-hub-pointers.json"));
    let findings = evaluate_masterplan_v2_entry_surfaces(&masterplan, &root_hub);
    assert!(
        findings.is_empty(),
        "entry-surface read contracts must exactly match the bounded root-hub allowlist and exclude superseded entrypoints: {findings:?}"
    );
}

/// Sub-AC 4.4 fail-closed pins: the frozen fixture corpus must keep one
/// ISOLATED RED fixture per read-contract/entry-surface failure class — a
/// superseded plan authority resurrected/re-exposed outside the archive
/// (docs/ROADMAP.md with its archive markers stripped plus a non-archive
/// read-path reference), a superseded entrypoint revived into the mandatory
/// entry surface, and an entry surface unbounded beyond the root-hub
/// allowlist (docs/MASTERPLAN.md promoted) — each pinned to its exact
/// violation set so none can be silently dropped or diluted. A GREEN
/// companion fixture keeps the full lane exercisable end to end.
#[test]
fn masterplan_read_contract_entry_surface_fixtures_fail_closed() {
    let cases: [(&str, &[&str]); 3] = [
        (
            "tc-XA-bad-masterplan-read-contract-resurrected-roadmap.json",
            &["masterplan_read_contract_invalid"],
        ),
        (
            "tc-XA-bad-masterplan-entry-surface-resurrected-superseded.json",
            &["masterplan_entry_surface_invalid"],
        ),
        (
            "tc-XA-bad-masterplan-entry-surface-unbounded.json",
            &["masterplan_entry_surface_invalid"],
        ),
    ];

    for (fixture_name, expected_codes) in cases {
        let path = fixture_dir().join(fixture_name);
        assert!(
            path.is_file(),
            "read-contract/entry-surface failure-mode fixture must exist: {}",
            path.display()
        );
        let fixture = load_json(&path);
        let report = evaluate(&fixture);
        assert_eq!(
            report.verdict,
            Verdict::Red,
            "{fixture_name} must fail closed (RED)"
        );
        let expected: BTreeSet<String> = expected_codes
            .iter()
            .map(|code| (*code).to_owned())
            .collect();
        assert_eq!(
            report.violations, expected,
            "{fixture_name} must emit exactly the pinned read-contract/entry-surface violation set"
        );
        assert_eq!(
            expected_violations(&fixture),
            expected,
            "{fixture_name} expected_violations must stay in sync with the pinned set"
        );
    }

    let green = fixture_dir().join("tc-XA-good-masterplan-read-surface-archive-clean.json");
    assert!(
        green.is_file(),
        "read-contract/entry-surface GREEN fixture must exist: {}",
        green.display()
    );
    let report = evaluate(&load_json(&green));
    assert_eq!(
        report.verdict,
        Verdict::Green,
        "the GREEN read-surface fixture must stay green, got {:?}",
        report.violations
    );
}

/// Sub-AC 4.4 resurrection sweep, born-blocking over the live tree: every
/// governed on-disk read surface (each `surface_dispositions` repo-file row
/// dispositioned absorbed / archived-with-provenance / generated-projection —
/// docs/MASTERPLAN.md, docs/ROADMAP.md, the retired planning specs, the
/// repo-local provenance stores) must still carry its archive markers on
/// disk. Stripping the archive front-matter from docs/ROADMAP.md, deleting
/// the absorbed status from a retired spec, or re-filling any superseded
/// authority with live-looking plan content turns this test RED. Tracked-tree
/// membership comes from the committed scm-facts face, the same declared
/// input the plan-evidence cross-check lane reads.
#[test]
fn masterplan_read_surface_resurrection_gate_is_green_on_live_tree() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let corpus = live_read_surface_corpus(&root, &masterplan);

    let findings = evaluate_masterplan_read_surface_resurrections(&masterplan, &corpus);
    assert!(
        findings.is_empty(),
        "superseded/stale plan authorities must stay archived on disk (no resurrection outside the archive): {findings:?}"
    );

    // The sweep must actually have surfaces to police: an empty corpus here
    // would mean the disposition ledger lost its governed read surfaces.
    let swept = corpus["surfaces"].as_array().expect("surfaces").len();
    assert!(
        swept >= 5,
        "live resurrection sweep must cover the governed read surfaces, swept only {swept}"
    );
}

/// Assemble the live read-surface corpus from the repo tree: one row per
/// governed `surface_dispositions` repo-file path, carrying tracked-tree
/// existence plus the on-disk facts (Markdown front-matter block, parsed
/// JSON document, or an opaque-data marker for non-document provenance
/// files).
fn live_read_surface_corpus(root: &Path, masterplan: &Value) -> Value {
    let scm_facts =
        load_json(&root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json"));
    let tracked: BTreeSet<&str> = scm_facts["tracked_paths"]
        .as_array()
        .expect("committed scm-facts face must carry tracked_paths")
        .iter()
        .filter_map(Value::as_str)
        .collect();

    let mut surfaces = Vec::new();
    let dispositions = masterplan["masterplan_v2"]["surface_dispositions"]
        .as_array()
        .expect("masterplan v2 must carry surface_dispositions");
    for surface in dispositions {
        let Some(disposition) = surface.get("disposition").and_then(Value::as_str) else {
            continue;
        };
        if !matches!(
            disposition,
            "absorbed" | "archived-with-provenance" | "generated-projection"
        ) {
            continue;
        }
        let Some(path) = surface.get("path").and_then(Value::as_str) else {
            continue;
        };
        if path.contains('#') || path.contains('*') || path.starts_with('~') {
            continue;
        }
        let rel_path = path.trim_start_matches('/');
        let on_disk = root.join(rel_path);
        let exists = tracked.contains(rel_path) && on_disk.is_file();
        let mut row = serde_json::json!({ "path": path, "exists": exists });
        if exists {
            if rel_path.ends_with(".md") {
                let content = fs::read_to_string(&on_disk).expect("read governed markdown surface");
                row["front_matter"] = Value::String(markdown_front_matter(&content));
            } else if rel_path.ends_with(".json") {
                row["document"] = load_json(&on_disk);
            } else {
                row["opaque_data"] = Value::Bool(true);
            }
        }
        surfaces.push(row);
    }

    serde_json::json!({ "surfaces": surfaces })
}

/// Extract the leading `---` front-matter block from a Markdown file; when a
/// file carries no front-matter fence, the (bounded) head of the file is the
/// scanned surface, so a marker-free live document still fails the sweep.
fn markdown_front_matter(content: &str) -> String {
    if let Some(rest) = content.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---")
    {
        return rest[..end].to_owned();
    }
    content.chars().take(4096).collect()
}

/// Sub-AC 4.2 mechanical re-derivation lane, born-blocking over the live tree:
/// every derived/generated masterplan projection that exists on disk must be
/// mechanically re-derivable from /specs/masterplan.json and byte-identical to
/// its re-derivation. The corpus is assembled from the ACTUAL tree (the human
/// projection, the fabric-loop flow-metrics ledger, the loop-card shard views,
/// and every on-disk generated planning face), so a stale or hand-edited
/// projection anywhere in the tree turns this test RED.
#[test]
fn masterplan_projection_rederivation_gate_is_green_on_live_tree() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let corpus = live_projection_rederivation_corpus(&root);

    // The lanes must be exercised, not vacuous: the live tree carries the
    // generated human projection, at least one recorded flow-metrics pass, and
    // at least one loop-card shard view.
    assert!(
        corpus["masterplan_md"]
            .as_str()
            .is_some_and(|md| !md.is_empty()),
        "docs/MASTERPLAN.md must exist as the generated human projection"
    );
    assert!(
        corpus["flow_metrics_passes"]
            .as_array()
            .is_some_and(|passes| !passes.is_empty()),
        "the flow-metrics ledger must carry at least one recorded pass"
    );
    assert!(
        corpus["loop_cards"]
            .as_array()
            .is_some_and(|cards| !cards.is_empty()),
        "the coordination plane must carry at least one loop-card shard view"
    );

    let findings = evaluate_masterplan_projection_rederivation(&masterplan, &corpus);
    assert!(
        findings.is_empty(),
        "every derived/generated masterplan projection must re-derive byte-identically from /specs/masterplan.json: {findings:?}"
    );

    // The derivation itself must reproduce the committed projection exactly.
    let derived = derive_masterplan_md_projection(&masterplan)
        .expect("docs/MASTERPLAN.md must be derivable from masterplan v2");
    let on_disk = fs::read_to_string(root.join("docs/MASTERPLAN.md")).expect("read MASTERPLAN.md");
    assert_eq!(
        derived, on_disk,
        "docs/MASTERPLAN.md must be byte-identical to its mechanical re-derivation"
    );
}

/// Assemble the live projection-rederivation corpus from the repo tree.
fn live_projection_rederivation_corpus(root: &Path) -> Value {
    let masterplan_md = fs::read_to_string(root.join("docs/MASTERPLAN.md")).unwrap_or_default();
    let flow_metrics_passes =
        read_projection_files(&root.join("plan/fabric-loop/flow-metrics/passes"), ".json");
    let loop_cards = read_projection_files(&root.join("plan/fabric-loop/cards"), ".json");
    let generated_projections_on_disk: Vec<Value> =
        list_file_names(&root.join("docs/machine-readable"), ".generated.json")
            .into_iter()
            .map(|name| Value::String(format!("docs/machine-readable/{name}")))
            .collect();
    let control_plane = load_json(&root.join("registry/generated-artifact-control-plane.json"));

    serde_json::json!({
        "masterplan_md": masterplan_md,
        "flow_metrics_passes": flow_metrics_passes,
        "loop_cards": loop_cards,
        "generated_projections_on_disk": generated_projections_on_disk,
        "generated_artifact_control_plane": control_plane,
    })
}

fn list_file_names(dir: &Path, suffix: &str) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
        .filter(|name| name.ends_with(suffix))
        .collect();
    names.sort();
    names
}

fn read_projection_files(dir: &Path, suffix: &str) -> Vec<Value> {
    list_file_names(dir, suffix)
        .into_iter()
        .map(|name| {
            let content = fs::read_to_string(dir.join(&name))
                .unwrap_or_else(|e| panic!("read {}/{name}: {e}", dir.display()));
            serde_json::json!({"file_name": name, "content": content})
        })
        .collect()
}

/// Sub-AC 4.2 fail-closed pins: the frozen fixture corpus must keep one
/// ISOLATED RED fixture for a hand-edited generated projection and one for a
/// stale derived ledger, each emitting exactly `masterplan_projection_stale`.
#[test]
fn masterplan_projection_rederivation_fixtures_fail_closed() {
    for fixture_name in [
        "tc-XA-bad-masterplan-projection-hand-edited.json",
        "tc-XA-bad-masterplan-projection-stale-ledger.json",
    ] {
        let path = fixture_dir().join(fixture_name);
        assert!(
            path.is_file(),
            "projection-rederivation failure-mode fixture must exist: {}",
            path.display()
        );
        let fixture = load_json(&path);
        let report = evaluate(&fixture);
        assert_eq!(
            report.verdict,
            Verdict::Red,
            "{fixture_name} must fail closed (RED)"
        );
        let expected: BTreeSet<String> =
            std::iter::once("masterplan_projection_stale".to_owned()).collect();
        assert_eq!(
            report.violations, expected,
            "{fixture_name} must emit exactly the stale-projection violation"
        );
        assert_eq!(
            expected_violations(&fixture),
            expected,
            "{fixture_name} expected_violations must stay in sync with the pinned set"
        );
    }
}

fn expected_masterplan_projection_paths<'a>(
    masterplan: &'a Value,
    generated_artifacts: &'a Value,
) -> BTreeSet<&'a str> {
    let v2 = &masterplan["masterplan_v2"];
    let mut expected = BTreeSet::new();

    for contract in v2["read_contracts"]
        .as_array()
        .expect("masterplan_v2.read_contracts must be an array")
    {
        let path = contract["path"]
            .as_str()
            .expect("read contract path must be a string");
        if path != "/specs/masterplan.json" {
            expected.insert(path);
        }
    }

    for surface in v2["surface_dispositions"]
        .as_array()
        .expect("masterplan_v2.surface_dispositions must be an array")
    {
        if surface["disposition"].as_str() == Some("generated-projection") {
            expected.insert(
                surface["path"]
                    .as_str()
                    .expect("generated projection surface path must be a string"),
            );
        }
    }

    for artifact in generated_artifacts["artifacts"]
        .as_array()
        .expect("generated_artifact_control_plane.artifacts must be an array")
    {
        if artifact_source_inputs_include_masterplan(artifact) {
            expected.insert(
                artifact["path"]
                    .as_str()
                    .expect("generated artifact path must be a string"),
            );
        }
    }

    expected
}

fn artifact_source_inputs_include_masterplan(artifact: &Value) -> bool {
    artifact["source_inputs"].as_array().is_some_and(|inputs| {
        inputs.iter().any(|input| {
            input
                .as_str()
                .is_some_and(source_input_refers_to_masterplan)
        })
    })
}

fn source_input_refers_to_masterplan(path: &str) -> bool {
    let path = path.trim();
    let without_fragment = path.split_once('#').map_or(path, |(path, _)| path);
    let mut normalized = without_fragment.trim_start_matches('/');
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped;
    }
    normalized == "specs/masterplan.json"
}

#[test]
fn masterplan_v2_sequencing_is_zero_based_and_founder_ratification_recorded() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let mut findings = evaluate_masterplan_v2_sequencing(&masterplan);

    // The ratification decision must be durable evidence, not a bare boolean: the
    // decision_ref must resolve to a committed evidence record.
    let ratification = &masterplan["masterplan_v2"]["sequencing"]["founder_ratification"];
    assert_eq!(
        ratification["decision_recorded"].as_bool(),
        Some(true),
        "founder-ratification decision must be recorded before execution-wave dispatch"
    );
    let decision_ref = ratification["decision_ref"]
        .as_str()
        .expect("founder_ratification.decision_ref must be a string");
    assert!(
        root.join(decision_ref).is_file(),
        "founder_ratification.decision_ref must resolve to a durable evidence record: {decision_ref}"
    );
    let evidence = load_json(&root.join(decision_ref));
    findings.extend(evaluate_masterplan_v2_ratification_digest(
        &masterplan,
        &evidence,
    ));
    assert!(
        findings.is_empty(),
        "masterplan v2 sequencing must stay zero-based, DAG-derived, and match its durable \
         founder-ratification digest before any execution-wave dispatch: {findings:?}"
    );

    // Fail-closed dispatch contract survives ratification: dispatch without a founder
    // decision stays structurally forbidden even after this decision is recorded.
    let dispatch = &masterplan["masterplan_v2"]["sequencing"]["execution_wave_dispatch"];
    assert_eq!(
        dispatch["requires_founder_ratification"].as_bool(),
        Some(true),
        "execution-wave dispatch must keep requiring founder ratification"
    );
    assert_eq!(
        dispatch["allowed_without_founder_ratification"].as_bool(),
        Some(false),
        "execution-wave dispatch must never be allowed without founder ratification"
    );
}

#[test]
fn adr_0624_is_explicitly_nonbinding_and_preserves_preplanning_hold() {
    let root = repo_root();
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let sequencing = load_json(&root.join("specs/master-plan-sequencing.json"));
    let control_plane = load_json(&root.join("registry/adr-census-epoch/control-plane.json"));
    let planning = &masterplan["planning_authority"];
    let contract = &masterplan["masterplan_v2"]["planning_entry_contract"];
    let dispatch = &masterplan["masterplan_v2"]["sequencing"]["execution_wave_dispatch"];
    let dispositions =
        &masterplan["masterplan_v2"]["accepted_decision_propagation_dispositions"]["decisions"];

    let adr_0624 = dispositions
        .as_array()
        .expect("accepted decision propagation dispositions must be an array")
        .iter()
        .find(|decision| decision["id"] == "ADR-0624")
        .expect("ADR-0624 must have an explicit masterplan nonbinding disposition");
    let sequencing_adr_0624 = sequencing["_metadata"]["accepted_decision_propagation_dispositions"]
        ["decisions"]
        .as_array()
        .expect("sequencing decision propagation dispositions must be an array")
        .iter()
        .find(|decision| decision["id"] == "ADR-0624")
        .expect("ADR-0624 must have an explicit historical-sidecar disposition");

    assert!(
        !planning["bound_adrs"]
            .as_array()
            .expect("planning_authority.bound_adrs must be an array")
            .iter()
            .any(|id| id == "ADR-0624"),
        "ADR-0624 has planning_impact:false and must not become bound planning authority"
    );
    assert_eq!(adr_0624["lifecycle_state"].as_str(), Some("Accepted"));
    assert_eq!(adr_0624["planning_impact"].as_bool(), Some(false));
    assert_eq!(adr_0624["sequencing_effect"].as_str(), Some("none"));
    assert_eq!(
        adr_0624["binding_plan_approval_effect"].as_str(),
        Some("none")
    );
    assert_eq!(adr_0624["execution_dispatch_effect"].as_str(), Some("none"));
    assert_eq!(adr_0624["hold_state"].as_str(), Some("HOLD(Planning)"));
    assert_eq!(
        adr_0624["disposition_ref"].as_str(),
        Some(
            "/specs/master-plan-sequencing.json#_metadata.accepted_decision_propagation_dispositions"
        )
    );
    assert_eq!(
        sequencing_adr_0624["disposition_ref"].as_str(),
        Some("/specs/masterplan.json#masterplan_v2.accepted_decision_propagation_dispositions")
    );
    assert_eq!(
        sequencing_adr_0624["hold_state"].as_str(),
        Some("HOLD(Planning)")
    );

    assert_eq!(control_plane["active_epoch"].as_str(), Some("P2"));
    assert_lawful_planning_entry_state(contract, dispatch);
}

/// The planning-entry contract has exactly TWO lawful shapes:
/// - the OPEN hold: both authority flags false, dispatch structurally blocked on
///   `preplanning_authority_closure`, zero dispatched waves; or
/// - the fully-evidenced CLOSED transition: both flags true plus a
///   `closure_evidence` chain whose refs resolve to durable, parseable `evidence/**`
///   records that the pure evaluator accepts.
///
/// Anything else is a hard failure. The live tree stays green only in the open
/// hold; a live closed planning contract panics here until a trusted
/// exact-pull-request/head review-admission packet is supplied.
fn assert_lawful_planning_entry_state(contract: &Value, dispatch: &Value) {
    match contract["state"].as_str() {
        Some("open") => {
            assert_eq!(
                contract["binding_plan_approval_allowed"].as_bool(),
                Some(false),
                "open hold must keep binding plan approval locked"
            );
            assert_eq!(
                contract["dispatch_allowed"].as_bool(),
                Some(false),
                "open hold must keep dispatch locked"
            );
            assert_eq!(dispatch["state"].as_str(), Some("blocked"));
            assert_eq!(
                dispatch["blocked_reason"].as_str(),
                Some("preplanning_authority_closure")
            );
            assert!(
                dispatch["dispatched_waves"]
                    .as_array()
                    .is_some_and(|waves| waves.is_empty()),
                "no execution wave may dispatch while HOLD(Planning) remains open"
            );
        }
        Some("closed") => {
            panic!(
                "the live planning-entry contract cannot close until the blocking gate receives \
                 a trusted review-admission packet bound to the exact pull request and head"
            );
        }
        other => {
            panic!("planning_entry_contract.state must be exactly open or closed, got {other:?}")
        }
    }
}

#[test]
fn closure_evidence_refs_use_one_canonical_path_for_validation_and_loading() {
    let normalized = normalize_closure_evidence_ref(Some(&serde_json::json!(
        "  ././evidence/goals/receipt.json  "
    )));
    assert_eq!(normalized.as_deref(), Some("evidence/goals/receipt.json"));

    for rejected in [
        "/evidence/goals/receipt.json",
        "~/evidence/goals/receipt.json",
        "evidence/../goals/receipt.json",
        ".gjc/receipt.json",
        ".omc/receipt.json",
        ".omx/receipt.json",
        "specs/receipt.json",
    ] {
        assert_eq!(
            normalize_closure_evidence_ref(Some(&serde_json::json!(rejected))),
            None,
            "{rejected:?} must not carry closure authority"
        );
    }
}

/// Productized false-green guard: planning closure is an architecture authority, so the
/// cloud-ci cross-artifact gate must fail whenever the contract, masterplan, or sequencing
/// sidecar carry different first-deliverable architecture rule sets. This is a pure
/// data-over-data check over JSON artifacts; it does not shell out to the legacy dev-cli gate.
#[test]
fn planning_closure_architecture_rules_agree_across_authority_artifacts() {
    let root = repo_root();
    let contract = load_json(&root.join("specs/planning-closure-contract.json"));
    let masterplan = load_json(&root.join("specs/masterplan.json"));
    let sequencing = load_json(&root.join("specs/master-plan-sequencing.json"));

    let contract_rules = architecture_rule_set(
        &contract["first_deliverable"]["architecture_exit_bar"],
        "specs/planning-closure-contract.json:first_deliverable.architecture_exit_bar",
    );
    let masterplan_rules = architecture_rule_set(
        &masterplan["planning_closure"]["first_deliverable"]["architecture_exit_bar"],
        "specs/masterplan.json:planning_closure.first_deliverable.architecture_exit_bar",
    );
    let sequencing_rules = architecture_rule_set(
        &sequencing["first_deliverable_ordering"]["architecture_exit_bar"],
        "specs/master-plan-sequencing.json:first_deliverable_ordering.architecture_exit_bar",
    );

    assert_same_rule_set(&contract_rules, &masterplan_rules, "contract", "masterplan");
    assert_same_rule_set(&contract_rules, &sequencing_rules, "contract", "sequencing");
}

/// Born-blocking self-test: GATE-1 must go RED on TODAY's real corpus. Per the firewall
/// doctrine, "a firewall that doesn't block today is the facade we're killing." This runs
/// the producer's decision-crosswalk face over the live tree and asserts the gate flags the
/// real defects:
/// - `generated_face_drift` — catalog.json axes_count:6 vs contracts.json axes_count:7
/// - `supersession_half_edge` — ADR-0511 supersedes ADR-0359 while ADR-0359 omits it
///
/// Plus two lanes that must stay CLEAN on the live corpus:
/// - `dual_decision_collision` — the historical two-ADR-0377-files exhibit was resolved
///   2026-06-12 by renumbering the newer file to ADR-0557 (FRIC-1781390000); the live
///   corpus must stay duplicate-free (the RED shape stays pinned by the frozen
///   tc-XA-bad-dup-adr-number fixture).
/// - the frozen-empty `decision_id_mismatch` lane: zero filename/front-matter id
///   disagreements today, asserted with the renumber remediation in the FAIL output.
///
/// Counts are MEASURED, not hardcoded.
#[test]
fn gate1_is_born_blocking_on_the_live_corpus() {
    let root = repo_root();
    let crosswalk = run_producer_face(&root, "decision-crosswalk");

    let report = evaluate(&crosswalk);

    assert_eq!(
        report.verdict,
        Verdict::Red,
        "GATE-1 MUST go RED on today's corpus (the firewall must block today)"
    );
    assert!(
        report.violations.contains("generated_face_drift"),
        "catalog.json axes_count:6 vs contracts.json axes_count:7 -> generated_face_drift must fire"
    );
    assert!(
        !report.violations.contains("dual_decision_collision"),
        "duplicate decision ids must stay resolved (the ADR-0377 pair was renumbered to \
         ADR-0557 per FRIC-1781390000; allocate via the accounting-registry producer's \
         --next-adr): {:?}",
        crosswalk["duplicate_ids"]
    );
    // Disposition may heal supersession half-edges; born-blocking still requires RED
    // via remaining live defects (generated_face_drift is the durable exhibit).
    if !report.violations.contains("supersession_half_edge") {
        assert!(
            report.violations.contains("generated_face_drift"),
            "when supersession_half_edge is healed, generated_face_drift must still keep GATE-1 RED: {:?}",
            report.violations
        );
    }

    // decision_id_mismatch is frozen-empty (born-blocking): the live corpus carries no
    // filename/front-matter id disagreement today, and any future occurrence is the
    // mask vector for a duplicate-numbered ADR pair (FRIC-1781320000). The remediation
    // is named here so the FAIL output alone is actionable.
    let id_mismatches = crosswalk["id_mismatches"]
        .as_array()
        .expect("id_mismatches");
    let next_free_id = crosswalk["next_free_id"].as_str().expect("next_free_id");
    assert!(
        id_mismatches.is_empty(),
        "decision_id_mismatch must stay frozen-empty: {id_mismatches:?} — renumber the newer \
         decision (filename AND front-matter id) to the next free number {next_free_id} \
         (allocate via the accounting-registry producer's --next-adr)"
    );
    assert!(
        next_free_id
            .strip_prefix("ADR-")
            .is_some_and(|digits| digits.len() == 4 && digits.chars().all(|c| c.is_ascii_digit())),
        "next_free_id must be an ADR-NNNN allocator output, got {next_free_id:?}"
    );

    // phantom_decision_citation is frozen-empty (born-blocking, FRIC-1781430000): the
    // phantom-0397 exhibit (seven governed surfaces citing "ADR-0397 Pulsar 4.x + Oxia
    // canonical event-bus" with no file at the number — audit register H-19) was healed
    // 2026-06-12 by MINTING docs/decisions/ADR-0709-general-live-apex.md,
    // and the pre-existing phantom inventory is grandfathered shrink-only DATA in the
    // producer (each id ledgered with its citation sites). Any edge here is NEW debt.
    let phantom_citations = crosswalk["phantom_citations"]
        .as_array()
        .expect("phantom_citations");
    assert!(
        phantom_citations.is_empty(),
        "phantom_decision_citation must stay frozen-empty: {phantom_citations:?} — mint the \
         record at the cited number (status Proposed, reconstruction banner; allocate NEW \
         numbers via the accounting-registry producer's --next-adr) or retarget the citation"
    );
    assert!(
        !report.violations.contains("phantom_decision_citation"),
        "the live corpus must carry no phantom decision citation"
    );
    // ADR-0397 was reconstructed; under the apex disposition it lives in docs/adr-archive/
    // (Superseded) while live crosswalk rows are apex-only. Phantom resolution still
    // knows the archive id (known_ids), so citations must not reappear as phantoms.
    assert!(
        root.join("docs/adr-archive/ADR-0397-pulsar-oxia-canonical-event-bus.md")
            .is_file(),
        "ADR-0397 reconstruction record must remain on disk under the historical archive"
    );
    assert!(
        !phantom_citations.iter().any(|citation| {
            citation
                .as_str()
                .is_some_and(|value| value.starts_with("ADR-0397@"))
        }),
        "ADR-0397 must not reappear as a phantom citation: {phantom_citations:?}"
    );

    // The grandfathered inventory is mechanically guarded (review MEDIUM, 2026-06-12):
    // (1) ANTI-PADDING — every grandfathered id must STILL resolve to no decision file;
    //     a healed id (one that now has a crosswalk row) must leave the inventory in the
    //     healing PR, so the carve-out can never shadow a real decision.
    // (2) ANTI-GROWTH — a decrease-only ceiling (the #676 baseline+independent-ceiling
    //     pattern): laundering a NEW phantom citation by adding its id to the inventory
    //     in the same PR forces a loud edit of this pinned ceiling, which may only ever
    //     go DOWN as ids are healed (mint-or-retarget per their ledger rows).
    const GRANDFATHERED_PHANTOM_CEILING: usize = 63; // decrease-only; never raise
    let grandfathered: Vec<&str> = crosswalk["grandfathered_phantom_ids"]
        .as_array()
        .expect("grandfathered_phantom_ids")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        grandfathered.len() <= GRANDFATHERED_PHANTOM_CEILING,
        "the grandfathered phantom inventory may only SHRINK (got {}, ceiling {}): a new \
         phantom citation is never grandfathered — mint the record at the cited number or \
         retarget the citation (FRIC-1781430000)",
        grandfathered.len(),
        GRANDFATHERED_PHANTOM_CEILING
    );
    let decision_ids: BTreeSet<&str> = crosswalk["decisions"]
        .as_array()
        .expect("decisions")
        .iter()
        .filter_map(|d| d["id"].as_str())
        .collect();
    let padded: Vec<&str> = grandfathered
        .iter()
        .copied()
        .filter(|id| decision_ids.contains(id))
        .collect();
    assert!(
        padded.is_empty(),
        "grandfathered ids that now resolve to a real decision must leave the inventory \
         (remove from GRANDFATHERED_PHANTOM_DECISION_IDS + lower the ceiling): {padded:?}"
    );
    // (3) ANTI-INERT — every grandfathered id must still be CITED somewhere in the
    //     governed surfaces; an entry whose citations were all retargeted away protects
    //     nothing and is a standing silent-reintroduction ticket (the FRIC-1781280001
    //     inert-door class) — retire it (remove + lower the ceiling). Together with
    //     (1), (2) and the frozen-empty phantom lane this pins the inventory to be
    //     EXACTLY the live cited-but-missing set.
    let governed_corpus = read_governed_citation_corpus(&root);
    let inert: Vec<&str> = grandfathered
        .iter()
        .copied()
        .filter(|id| !governed_corpus.contains(*id))
        .collect();
    assert!(
        inert.is_empty(),
        "grandfathered ids no longer cited by any governed surface protect nothing — \
         retire them (remove from GRANDFATHERED_PHANTOM_DECISION_IDS + lower the \
         ceiling): {inert:?}"
    );

    // Count the real exhibits for the evidence digest.
    let decisions = crosswalk["decisions"].as_array().expect("decisions");
    let dup_ids = crosswalk["duplicate_ids"]
        .as_array()
        .expect("duplicate_ids");
    let axes = crosswalk["generated_face_axes"]
        .as_object()
        .expect("generated_face_axes");
    let unpropagated = decisions
        .iter()
        .filter(|d| {
            d["status"]
                .as_str()
                .is_some_and(|s| s.eq_ignore_ascii_case("accepted"))
                && (d["in_spec"].as_bool().unwrap_or(false)
                    || d["in_masterplan"].as_bool().unwrap_or(false)
                    || d["in_roadmap"].as_bool().unwrap_or(false))
                && !(d["in_spec"].as_bool().unwrap_or(false)
                    && d["in_masterplan"].as_bool().unwrap_or(false)
                    && d["in_roadmap"].as_bool().unwrap_or(false))
        })
        .count();

    eprintln!(
        "BORN-BLOCKING live-corpus counts: decisions={} duplicate_ids={:?} id_mismatches={:?} phantom_citations={} next_free_id={next_free_id} axes={:?} unpropagated_decision={} violations={:?}",
        decisions.len(),
        dup_ids,
        id_mismatches,
        phantom_citations.len(),
        axes,
        unpropagated,
        report.violations
    );
}

/// Concatenate the governed citation surfaces (every decision body + the
/// roadmap/sequencing artifact + the masterplan) into one corpus string for the
/// anti-inert containment check. A plain substring probe over-approximates the
/// producer's token scan in the conservative direction: an id mentioned in ANY form
/// counts as still-cited, so an entry is only called inert when no governed surface
/// mentions it at all.
fn read_governed_citation_corpus(root: &Path) -> String {
    let mut corpus = String::new();
    let mut paths: Vec<PathBuf> = Vec::new();
    for rel in ["docs/decisions", "docs/adr-archive"] {
        let dir = root.join(rel);
        if !dir.is_dir() {
            continue;
        }
        paths.extend(
            fs::read_dir(&dir)
                .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "md")),
        );
    }
    paths.push(root.join("specs/master-plan-sequencing.json"));
    paths.push(root.join("specs/masterplan.json"));
    for path in paths {
        corpus.push_str(&fs::read_to_string(&path).unwrap_or_default());
        corpus.push('\n');
    }
    corpus
}

/// Run the producer to emit a single face to stdout, HERMETICALLY. The producer binary must be
/// provided by `OYA_CI_PRODUCER_BIN`; missing env fails closed so tests cannot silently fall back to
/// Cargo. The producer reads the materialized scm-facts face (a declared input); it never calls git.
fn run_producer_face(root: &Path, face: &str) -> Value {
    let scm_facts = root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json");
    let producer_bin = std::env::var("OYA_CI_PRODUCER_BIN").ok();
    let bin = producer_binary(root, producer_bin.as_deref()).unwrap_or_else(|e| panic!("{e}"));
    let output = Command::new(bin)
        .arg("--repo-root")
        .arg(root)
        .arg("--scm-facts")
        .arg(&scm_facts)
        .arg("--stdout")
        .arg("--face")
        .arg(face)
        .current_dir(root)
        .output()
        .expect("run producer binary");
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("producer face stdout is valid JSON")
}

// ===========================================================================
// Gate-coverage-gap advisory checks (born-advisory vs a frozen baseline).
//
// These three lanes close the #1327 review class that no born-blocking §5.2 code
// keys on: the defects lived in prose / derived-policy / generated-projection
// surfaces. Each check is BORN-ADVISORY — it does not join `evaluate`'s blocking
// verdict; it enforces NO-REGRESSION against the committed frozen baseline
// `gate-coverage-baseline.json`. Each live test asserts the ratchet is CLEAN: the
// live advisory finding set equals the frozen baseline exactly (zero NEW
// regressions AND zero stale burned-down rows). The baseline is born empty.
// ===========================================================================

fn gate_coverage_baseline(root: &Path) -> GateCoverageBaseline {
    let doc =
        load_json(&root.join("ci/facade/cross-artifact-agreement/gate-coverage-baseline.json"));
    GateCoverageBaseline::from_value(&doc)
}

fn assert_ratchet_clean(report: &RatchetReport, lane: &str) {
    assert!(
        report.regressions.is_empty(),
        "{lane}: NEW advisory regression(s) not in gate-coverage-baseline.json — either fix the \
         divergence or record it in the frozen baseline with a justification: {:?}",
        report.regressions
    );
    assert!(
        report.burned_down.is_empty(),
        "{lane}: a gate-coverage-baseline.json row no longer reproduces on the live corpus — \
         remove it and re-freeze (a stale phantom baseline row must never rot the ratchet): {:?}",
        report.burned_down
    );
}

/// Enumerate `docs/decisions/*.md` file names (ADR-NNNN…md), newest amendment
/// dedup applied — matching the ADR-index producer's `read_adr_decision_records`
/// dedup so the id set is apples-to-apples with the projection records.
fn decision_md_file_names(root: &Path) -> Vec<String> {
    decision_md_paths_under(root, &["docs/decisions"])
}

/// Live apex + historical archive ADR markdown paths (repo-relative).
fn all_decision_md_file_names(root: &Path) -> Vec<String> {
    decision_md_paths_under(root, &["docs/decisions", "docs/adr-archive"])
}

fn decision_md_paths_under(root: &Path, rel_dirs: &[&str]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for rel in rel_dirs {
        let dir = root.join(rel);
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
            .filter_map(Result::ok)
        {
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if name.starts_with("ADR-") && name.ends_with(".md") {
                names.push(format!("{rel}/{name}"));
            }
        }
    }
    names.sort();
    let base_ids: BTreeSet<String> = names
        .iter()
        .filter(|path| !path.contains("-amendment-"))
        .filter_map(|path| adr_id_prefix_from_path(path))
        .collect();
    names.retain(|path| {
        if !path.contains("-amendment-") {
            return true;
        }
        adr_id_prefix_from_path(path).is_none_or(|id| !base_ids.contains(&id))
    });
    names
}

fn adr_id_prefix_from_path(path: &str) -> Option<String> {
    let name = path.rsplit_once('/').map(|(_, n)| n).unwrap_or(path);
    name.get(0..8).map(str::to_owned)
}

// --- Check 1/3: prose ⇄ front-matter status agreement -----------------------

/// Extract the front-matter `status:` value and the body of an ADR markdown file.
fn adr_frontmatter_status_and_body(contents: &str) -> (Option<String>, &str) {
    let Some(rest) = contents.strip_prefix("---\n") else {
        return (None, contents);
    };
    let Some(end) = rest.find("\n---") else {
        return (None, contents);
    };
    let frontmatter = &rest[..end];
    let body = &rest[end + "\n---".len()..];
    let status = frontmatter.lines().find_map(|line| {
        let line = line.trim();
        line.strip_prefix("status:")
            .map(|value| value.trim().trim_matches('"').trim_matches('\'').to_owned())
    });
    (status, body)
}

fn live_prose_status_corpus(root: &Path) -> Value {
    let mut adrs = Vec::new();
    for rel_path in all_decision_md_file_names(root) {
        let file_name = rel_path
            .rsplit_once('/')
            .map(|(_, name)| name)
            .unwrap_or(rel_path.as_str());
        let id = file_name.get(0..8).unwrap_or_default().to_owned();
        let contents = fs::read_to_string(root.join(&rel_path))
            .unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
        let (status, body) = adr_frontmatter_status_and_body(&contents);
        let Some(status) = status else { continue };
        adrs.push(serde_json::json!({
            "id": id,
            "frontmatter_status": status,
            "body": body,
        }));
    }
    serde_json::json!({ "adrs": adrs })
}

/// Sub-check 1/3 born-advisory over the live tree: no ADR body prose contradicts
/// its own front-matter status (#1327 defect class (a): "stays Proposed" in an
/// Accepted ADR). Enforces no-regression vs the frozen baseline.
#[test]
fn adr_prose_frontmatter_status_agreement_is_advisory_clean_on_live_tree() {
    let root = repo_root();
    let policy = load_json(
        &root.join("ci/facade/cross-artifact-agreement/prose-status-agreement-policy.json"),
    );
    let corpus = live_prose_status_corpus(&root);

    let scanned = corpus["adrs"].as_array().expect("adrs").len();
    assert!(
        scanned > 100,
        "the prose⇄front-matter sweep must cover the real ADR corpus, scanned only {scanned}"
    );

    let findings = evaluate_adr_prose_frontmatter_status(&corpus, &policy);
    let report = ratchet(&findings, &gate_coverage_baseline(&root));
    assert_ratchet_clean(&report, "adr_prose_status_contradiction");
}

// --- Check 2/3: capability-registry ⇄ derived gate-policy sync ---------------

fn live_registry_policy_corpus(root: &Path) -> Value {
    serde_json::json!({
        "registry": load_json(&root.join("governance/capability-registry.json")),
        "policies": {
            "module_membership": {
                "path": "ci/facade/module-membership/capability-membership-policy.json",
                "document": load_json(
                    &root.join("ci/facade/module-membership/capability-membership-policy.json"),
                ),
            },
            "root_hygiene": {
                "path": "ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json",
                "document": load_json(
                    &root.join("ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json"),
                ),
            },
            "tier_dependency": {
                "path": "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json",
                "document": load_json(&root.join(
                    "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json",
                )),
            },
        },
    })
}

/// Sub-check 2/3 born-advisory over the live tree: every capability root in
/// governance/capability-registry.json is present in the three derived gate policies
/// (#1327 defect class (c): a registered capability root missing from a derived
/// policy). Enforces no-regression vs the frozen baseline.
#[test]
fn registry_derived_policy_sync_is_advisory_clean_on_live_tree() {
    let root = repo_root();
    let corpus = live_registry_policy_corpus(&root);

    let capabilities = corpus["registry"]["capabilities"]
        .as_array()
        .expect("capabilities")
        .len();
    assert!(
        capabilities >= 20,
        "the registry sync check must cover the real closed capability set, saw only {capabilities}"
    );

    let findings = evaluate_registry_derived_policy_sync(&corpus);
    let report = ratchet(&findings, &gate_coverage_baseline(&root));
    assert_ratchet_clean(&report, "registry_derived_policy_desync");
}

// --- Check 3/3: generated ADR-index projection parity -----------------------

fn adr_records_from_decisions_json(decisions: &Value) -> Vec<AdrDecisionRecord> {
    let mut records = Vec::new();
    for entry in decisions["decisions"].as_array().expect("decisions array") {
        let str_field = |field: &str| -> String {
            entry[field]
                .as_str()
                .unwrap_or_else(|| panic!("decisions.json entry missing string field {field}"))
                .to_owned()
        };
        let str_list = |field: &str| -> Vec<String> {
            entry[field]
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default()
        };
        records.push(AdrDecisionRecord {
            number: u16::try_from(entry["number"].as_u64().expect("number")).expect("number u16"),
            id: str_field("adr"),
            title: str_field("title"),
            status: str_field("status"),
            owner: str_field("owner"),
            date: str_field("date"),
            path: str_field("path"),
            supersedes: str_list("supersedes"),
            superseded_by: str_list("superseded_by"),
            related: str_list("related"),
        });
    }
    records
}

fn source_derived_adr_records(root: &Path) -> Vec<AdrDecisionRecord> {
    static RECORDS: OnceLock<Vec<AdrDecisionRecord>> = OnceLock::new();
    RECORDS
        .get_or_init(|| {
            let producer = std::env::var("OYA_ADR_INDEX_PRODUCER_BIN")
                .expect("Buck2 must provide the sanctioned ADR-index producer binary");
            let temp = tempfile::tempdir().expect("create ADR-index projection tempdir");
            let index = temp.path().join("ADR-INDEX.md");
            let machine = temp.path().join("decisions.json");
            let output =
                Command::new(producer_binary(root, Some(&producer)).expect("ADR producer path"))
                    .current_dir(root)
                    .args([
                        "doc",
                        "adr-index",
                        "--decisions-dir",
                        root.join("docs/decisions")
                            .to_str()
                            .expect("UTF-8 decisions path"),
                        "--index",
                        index.to_str().expect("UTF-8 index path"),
                        "--machine",
                        machine.to_str().expect("UTF-8 machine path"),
                        "--write",
                        "--format",
                        "json",
                    ])
                    .output()
                    .expect("run sanctioned ADR-index producer");
            assert!(
                output.status.success(),
                "ADR-index producer failed: stdout={} stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            adr_records_from_decisions_json(&load_json(&machine))
        })
        .clone()
}

fn disk_adr_records_for_relation_guards(root: &Path) -> Vec<AdrDecisionRecord> {
    let mut records = Vec::new();
    for rel_path in all_decision_md_file_names(root) {
        let contents = fs::read_to_string(root.join(&rel_path))
            .unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
        let file_name = rel_path
            .rsplit_once('/')
            .map(|(_, n)| n)
            .unwrap_or(rel_path.as_str());
        let Some(id) = file_name.get(0..8).map(str::to_owned) else {
            continue;
        };
        let number = id
            .strip_prefix("ADR-")
            .and_then(|digits| digits.parse::<u16>().ok())
            .unwrap_or(0);
        let related = front_matter_id_list(&contents, "related");
        records.push(AdrDecisionRecord {
            number,
            id,
            title: String::new(),
            status: front_matter_field_value(&contents, "status").unwrap_or_default(),
            owner: String::new(),
            date: String::new(),
            path: rel_path,
            supersedes: front_matter_id_list(&contents, "supersedes"),
            superseded_by: front_matter_id_list(&contents, "superseded_by"),
            related,
        });
    }
    records
}

fn front_matter_field_value(contents: &str, field: &str) -> Option<String> {
    let rest = contents.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let frontmatter = &rest[..end];
    frontmatter.lines().find_map(|line| {
        let line = line.trim();
        line.strip_prefix(&format!("{field}:"))
            .map(|value| value.trim().trim_matches('"').trim_matches('\'').to_owned())
    })
}

fn front_matter_id_list(contents: &str, field: &str) -> Vec<String> {
    let Some(rest) = contents.strip_prefix("---\n") else {
        return Vec::new();
    };
    let Some(end) = rest.find("\n---") else {
        return Vec::new();
    };
    let frontmatter = &rest[..end];
    let mut lines = frontmatter.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        let Some(value) = trimmed.strip_prefix(&format!("{field}:")) else {
            continue;
        };
        let value = value.trim();
        if let Some(inner) = value.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            return inner
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .filter_map(normalize_related_adr_token)
                .collect();
        }
        if value.is_empty() {
            let mut out = Vec::new();
            while let Some(next) = lines.peek().copied() {
                let n = next.trim();
                if n.starts_with('-') {
                    let item = n.trim_start_matches('-').trim();
                    if let Some(id) = normalize_related_adr_token(item) {
                        out.push(id);
                    }
                    lines.next();
                } else if n.is_empty() {
                    lines.next();
                } else {
                    break;
                }
            }
            return out;
        }
    }
    Vec::new()
}

fn normalize_related_adr_token(token: &str) -> Option<String> {
    let token = token.trim().trim_matches('"').trim_matches('\'');
    let rest = token.strip_prefix("ADR-")?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.len() == 4 {
        Some(format!("ADR-{digits}"))
    } else {
        None
    }
}

fn public_protocol_required_relation_edges() -> Vec<(&'static str, &'static str)> {
    const SOURCES: [&str; 4] = ["ADR-0157", "ADR-0167", "ADR-0176", "ADR-0182"];
    let mut required = Vec::new();
    for source in SOURCES {
        required.push((source, "ADR-0203"));
        required.push((source, "ADR-0258"));
        required.push(("ADR-0203", source));
        required.push(("ADR-0258", source));
    }
    required.push(("ADR-0203", "ADR-0258"));
    required.push(("ADR-0258", "ADR-0203"));
    required
}

fn public_protocol_reciprocal_relation_error(records: &[AdrDecisionRecord]) -> Option<String> {
    let relations = records
        .iter()
        .map(|record| (record.id.as_str(), record.related.as_slice()))
        .collect::<BTreeMap<_, _>>();
    public_protocol_required_relation_edges()
        .into_iter()
        .find_map(|(source, target)| {
            let Some(related) = relations.get(source) else {
                return Some(format!("missing source metadata for {source}"));
            };
            (!related.iter().any(|relation| relation == target))
                .then(|| format!("{source} missing reciprocal related edge to {target}"))
        })
}

#[test]
fn public_protocol_source_metadata_has_every_reciprocal_relation() {
    let records = disk_adr_records_for_relation_guards(&repo_root());
    assert_eq!(public_protocol_reciprocal_relation_error(&records), None);
}

#[test]
fn public_protocol_reciprocal_relation_guard_rejects_every_removed_edge() {
    let records = disk_adr_records_for_relation_guards(&repo_root());
    let expected_edges = public_protocol_required_relation_edges();
    assert_eq!(
        expected_edges.len(),
        18,
        "full directed reciprocal edge set"
    );
    for (source, target) in expected_edges {
        let mut mutated = records.clone();
        let record = mutated
            .iter_mut()
            .find(|record| record.id == source)
            .expect("mutation source ADR");
        record.related.retain(|relation| relation != target);
        assert_eq!(
            public_protocol_reciprocal_relation_error(&mutated),
            Some(format!(
                "{source} missing reciprocal related edge to {target}"
            )),
            "removing {source} -> {target} must fail closed"
        );
    }
}

#[test]
fn source_relation_change_with_stale_controller_projection_fails_closed() {
    let root = repo_root();
    let mut source_records = disk_adr_records_for_relation_guards(&root);
    let record = source_records
        .iter_mut()
        .find(|record| record.id == "ADR-0157")
        .expect("ADR-0157 source record");
    record.related.retain(|relation| relation != "ADR-0203");

    let findings = evaluate_adr_index_projection_parity(
        &source_records,
        &fs::read_to_string(root.join("docs/ADR-INDEX.md")).expect("read docs/ADR-INDEX.md"),
        &fs::read_to_string(root.join("docs/machine-readable/decisions.json"))
            .expect("read docs/machine-readable/decisions.json"),
        &decision_md_file_names(&root)
            .iter()
            .filter_map(|path| adr_id_prefix_from_path(path))
            .collect(),
    );
    assert!(
        findings.iter().any(|finding| finding
            .key
            .starts_with("docs/machine-readable/decisions.json#")),
        "a source relation mutation with an unchanged controller projection must fail: {findings:?}"
    );
}

/// Sub-check 3/3 born-advisory over the live tree: docs/ADR-INDEX.md and
/// docs/machine-readable/decisions.json are byte-parity with their producer's
/// re-render (via the sanctioned Buck2-built `oya doc adr-index` producer) AND cover exactly
/// the docs/decisions/*.md corpus (#1327 defect class (d): projections not
/// regenerated through their producer; implements the adr-index-pipeline.md
/// promise). Enforces no-regression vs the frozen baseline.
#[test]
fn adr_index_projection_parity_is_advisory_clean_on_live_tree() {
    let root = repo_root();
    let records = source_derived_adr_records(&root);
    let on_disk_markdown =
        fs::read_to_string(root.join("docs/ADR-INDEX.md")).expect("read docs/ADR-INDEX.md");
    let on_disk_json = fs::read_to_string(root.join("docs/machine-readable/decisions.json"))
        .expect("read docs/machine-readable/decisions.json");
    let source_adr_ids: BTreeSet<String> = decision_md_file_names(&root)
        .iter()
        .filter_map(|path| adr_id_prefix_from_path(path))
        .collect();

    assert!(
        records.len() >= 10 && source_adr_ids.len() >= 10,
        "the ADR-index parity check must cover the live apex corpus: {} records, {} source ids",
        records.len(),
        source_adr_ids.len()
    );

    let findings = evaluate_adr_index_projection_parity(
        &records,
        &on_disk_markdown,
        &on_disk_json,
        &source_adr_ids,
    );
    let report = ratchet(&findings, &gate_coverage_baseline(&root));
    assert_ratchet_clean(&report, "adr_index_projection_stale");
}

/// The frozen baseline must stay well-formed and, at birth, EMPTY — the three
/// checks are born-advisory-green on the live corpus after #1327. Growth is only
/// ever a reviewed, justified pre-existing divergence.
#[test]
fn gate_coverage_baseline_is_born_empty_and_wellformed() {
    let root = repo_root();
    let doc =
        load_json(&root.join("ci/facade/cross-artifact-agreement/gate-coverage-baseline.json"));
    assert_eq!(
        doc["gate_id"].as_str(),
        Some("cloud-ci-cross-artifact-agreement"),
        "the baseline must name the gate it ratchets"
    );
    let baseline = GateCoverageBaseline::from_value(&doc);
    assert!(
        baseline.keys().is_empty(),
        "the gate-coverage baseline is born empty (born-advisory-green): {:?}",
        baseline.keys()
    );
}
