//! Mechanical founder-ratification gate on execution-wave dispatch surfaces
//! (masterplan v2 Sub-AC 3).
//!
//! What is proven here, against the LIVE `/specs/masterplan.json`:
//! 1. The recorded founder-ratification decision binds to the digest
//!    recomputed from the live sequencing content, so dispatch is granted —
//!    and the owned dependency-free SHA-256 + canonicalization in this crate
//!    byte-agrees with the sha2-backed cross-artifact-agreement gate's
//!    recorded `sequencing_hash`.
//! 2. Failure injection — ABSENT: stripping the ratification record from the
//!    live document makes the gate refuse dispatch (`RecordAbsent`).
//! 3. Failure injection — STALE: mutating the live sequencing content
//!    (dependency edges / work-item order) without re-ratifying voids the
//!    recorded digest (`StaleDigest`); bumping the sequencing version without
//!    re-ratifying voids the version binding (`StaleVersion`).
//! 4. A missing masterplan fails closed (`MasterplanUnreadable`) — a dispatch
//!    surface with no reachable plan authority never opens.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use oya_fabric_loop_state_app::{
    JsonValue, MASTERPLAN_REPO_PATH, RatificationRefusal, check_dispatch_ratification,
    check_dispatch_ratification_at, compute_sequencing_digest,
};

/// Walk up from the test's working directory to the repo root (the dir holding
/// the canonical `specs/root-hub-pointers.json`). Mirrors the sibling gates.
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
    panic!("repo root marker specs/root-hub-pointers.json not found");
}

fn live_masterplan_text() -> String {
    let path = repo_root().join(MASTERPLAN_REPO_PATH);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn live_masterplan() -> JsonValue {
    JsonValue::parse(&live_masterplan_text()).expect("live masterplan parses as canonical JSON")
}

fn field_mut<'a>(value: &'a mut JsonValue, key: &str) -> &'a mut JsonValue {
    let JsonValue::Obj(fields) = value else {
        panic!("expected object while descending to {key}");
    };
    &mut fields
        .iter_mut()
        .find(|(k, _)| k == key)
        .unwrap_or_else(|| panic!("missing key {key}"))
        .1
}

fn sequencing_mut(doc: &mut JsonValue) -> &mut JsonValue {
    field_mut(field_mut(doc, "masterplan_v2"), "sequencing")
}

// ---------------------------------------------------------------------------
// 1. Grant path: the live ratification record binds to the live content
// ---------------------------------------------------------------------------

#[test]
fn live_masterplan_ratification_binds_and_grants_dispatch() {
    let text = live_masterplan_text();
    let grant = check_dispatch_ratification(&text)
        .expect("live masterplan must carry a founder-ratification record bound to live content");

    // The grant digest is the digest recomputed from live content by the
    // OWNED hasher; it must byte-agree with the embedded sequencing_hash and
    // the ratified digest, both of which were produced by the sha2-backed
    // cross-artifact gate. This pins the two implementations together.
    let doc = live_masterplan();
    let sequencing = doc
        .get("masterplan_v2")
        .and_then(|v2| v2.get("sequencing"))
        .expect("masterplan_v2.sequencing");
    let embedded_hash = sequencing
        .get("sequencing_identity")
        .and_then(|identity| identity.get("sequencing_hash"))
        .and_then(JsonValue::as_str)
        .expect("sequencing_identity.sequencing_hash");
    assert_eq!(grant.sequencing_digest, embedded_hash);
    let ratified_digest = sequencing
        .get("founder_ratification")
        .and_then(|record| record.get("ratified_sequencing_digest"))
        .and_then(JsonValue::as_str)
        .expect("founder_ratification.ratified_sequencing_digest");
    assert_eq!(grant.sequencing_digest, ratified_digest);
    let recomputed = compute_sequencing_digest(doc.get("masterplan_v2").unwrap())
        .expect("live sequencing content must hash");
    assert_eq!(grant.sequencing_digest, recomputed);

    // The decision reference resolves to a durable committed evidence record.
    let decision_ref = grant
        .decision_ref
        .expect("live ratification record must carry a decision_ref");
    assert!(
        repo_root().join(&decision_ref).is_file(),
        "decision_ref must resolve to a durable evidence record: {decision_ref}"
    );

    // The filesystem entrypoint the dispatch surfaces call grants too.
    let grant_at = check_dispatch_ratification_at(&repo_root())
        .expect("check_dispatch_ratification_at must grant on the live repo");
    assert_eq!(grant_at.sequencing_digest, recomputed);
}

// ---------------------------------------------------------------------------
// 2. Failure injection — ABSENT record refusal path
// ---------------------------------------------------------------------------

#[test]
fn dispatch_is_refused_when_ratification_record_is_injected_absent() {
    // Strip the record entirely.
    let mut doc = live_masterplan();
    let JsonValue::Obj(sequencing_fields) = sequencing_mut(&mut doc) else {
        panic!("sequencing must be an object");
    };
    sequencing_fields.retain(|(k, _)| k != "founder_ratification");
    let refusal = check_dispatch_ratification(&doc.to_canonical_string())
        .expect_err("dispatch must be refused when the ratification record is absent");
    assert!(
        matches!(refusal, RatificationRefusal::RecordAbsent(_)),
        "expected RecordAbsent, got {refusal:?}"
    );
    assert!(
        refusal.to_string().contains("fail-closed"),
        "refusal message must state the fail-closed posture: {refusal}"
    );

    // Un-record the decision: decision_recorded=false is equally absent.
    let mut doc = live_masterplan();
    let record = field_mut(sequencing_mut(&mut doc), "founder_ratification");
    *field_mut(record, "decision_recorded") = JsonValue::Bool(false);
    assert!(matches!(
        check_dispatch_ratification(&doc.to_canonical_string()),
        Err(RatificationRefusal::RecordAbsent(_))
    ));

    // Downgrade the decision status: a non-ratified status is not a decision.
    let mut doc = live_masterplan();
    let record = field_mut(sequencing_mut(&mut doc), "founder_ratification");
    *field_mut(record, "decision_status") = JsonValue::Str("proposed".into());
    assert!(matches!(
        check_dispatch_ratification(&doc.to_canonical_string()),
        Err(RatificationRefusal::RecordAbsent(_))
    ));
}

// ---------------------------------------------------------------------------
// 3. Failure injection — STALE digest / version refusal paths
// ---------------------------------------------------------------------------

#[test]
fn dispatch_is_refused_when_sequencing_content_mutates_after_ratification() {
    // Mutate the ratifiable content: drop the last work_item_order entry.
    // The recomputed digest changes; the recorded ratification is void.
    let mut doc = live_masterplan();
    let JsonValue::Arr(order) = field_mut(sequencing_mut(&mut doc), "work_item_order") else {
        panic!("work_item_order must be an array");
    };
    assert!(!order.is_empty(), "live work_item_order must be non-empty");
    order.pop();
    let refusal = check_dispatch_ratification(&doc.to_canonical_string())
        .expect_err("dispatch must be refused when the ratified digest is stale");
    let RatificationRefusal::StaleDigest {
        ratified_sequencing_digest,
        computed_sequencing_digest,
    } = &refusal
    else {
        panic!("expected StaleDigest, got {refusal:?}");
    };
    assert_ne!(ratified_sequencing_digest, computed_sequencing_digest);
    assert!(refusal.to_string().contains("fail-closed"));

    // Mutate the DAG: reverse one dependency edge. Same void-ratification rule.
    let mut doc = live_masterplan();
    let JsonValue::Arr(edges) = field_mut(field_mut(&mut doc, "masterplan_v2"), "dependency_edges")
    else {
        panic!("dependency_edges must be an array");
    };
    let JsonValue::Obj(edge_fields) = edges.first_mut().expect("live DAG must have edges") else {
        panic!("edge must be an object");
    };
    for (key, value) in edge_fields.iter_mut() {
        if key == "from" {
            *value = JsonValue::Str("MPV2-INJECTED".into());
        }
    }
    assert!(matches!(
        check_dispatch_ratification(&doc.to_canonical_string()),
        Err(RatificationRefusal::StaleDigest { .. })
    ));
}

#[test]
fn dispatch_is_refused_when_ratified_version_is_stale() {
    // A fresh re-derivation bumps sequencing_version but the digest scope is
    // unchanged; the surviving old record no longer binds by version.
    let mut doc = live_masterplan();
    let identity = field_mut(sequencing_mut(&mut doc), "sequencing_identity");
    let current = identity
        .get("sequencing_version")
        .and_then(JsonValue::as_num)
        .expect("sequencing_version");
    *field_mut(identity, "sequencing_version") = JsonValue::Num(current + 1);
    let refusal = check_dispatch_ratification(&doc.to_canonical_string())
        .expect_err("dispatch must be refused when the ratified version is stale");
    assert!(
        matches!(
            refusal,
            RatificationRefusal::StaleVersion {
                ratified_sequencing_version: Some(ratified),
                current_sequencing_version: current_live,
            } if ratified == current && current_live == current + 1
        ),
        "expected StaleVersion, got {refusal:?}"
    );
}

// ---------------------------------------------------------------------------
// 4. Missing plan authority fails closed
// ---------------------------------------------------------------------------

#[test]
fn dispatch_is_refused_when_masterplan_is_missing() {
    let missing_root = std::env::temp_dir().join(format!(
        "oya-ratification-gate-int-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    assert!(matches!(
        check_dispatch_ratification_at(&missing_root),
        Err(RatificationRefusal::MasterplanUnreadable(_))
    ));
}
