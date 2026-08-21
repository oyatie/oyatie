//! FRIC-1781370000 incident fixtures.
//!
//! Each of the three merge-train leader incidents is reproduced as a fixture: RED under the naive
//! union semantics that produced the incident (cross-validated against the REAL ADR-0544
//! friction-accounting fold + the live policy, not a reimplementation), GREEN under the driver.
//! Plus the contract tests the deliverable pins: idempotence, commutativity modulo the append-order
//! rule, fail-closed on garbage, canonical-output stability, and live-ledger fold-equivalence
//! (canonicalizing the real ledger must not change the gate's verdict).
//!
//! ADR-0083 Tier-3: integration tests use unwrap/expect to assert invariants.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use ci_action_item_accounting::evaluate_keyed;
use friction_ledger_merge_driver_app::{MergeErrorKind, merge_ledgers, parse_ledger};
use serde_json::{Value, json};

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

fn live_policy() -> Value {
    let path = repo_root().join("ci/facade/action-item-accounting/friction-accounting-policy.json");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// Shape a JSONL document the way the ADR-0544 collector does: `{ "rows": [ <row>, .. ] }`.
fn observed(document: &str) -> Value {
    let rows: Vec<Value> = document
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).expect("fixture row parses"))
        .collect();
    json!({ "rows": rows })
}

fn gate_codes(policy: &Value, document: &str) -> BTreeSet<String> {
    evaluate_keyed(policy, &observed(document))
        .into_iter()
        .map(|finding| finding.code)
        .collect()
}

fn join(rows: &[&str]) -> String {
    let mut out = String::new();
    for row in rows {
        out.push_str(row);
        out.push('\n');
    }
    out
}

/// The naive union that produced incident 1: base + every line either side appended beyond base
/// (what `merge=union` / a hand union does — line-level, no id awareness, no logical identity).
fn naive_union(base: &str, ours: &str, theirs: &str) -> String {
    let base_lines: BTreeSet<&str> = base.lines().collect();
    let mut out = String::from(base);
    for side in [ours, theirs] {
        for line in side.lines() {
            if !base_lines.contains(line) {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    out
}

fn primary(id: &str, seen_at: &str, friction: &str) -> String {
    format!(
        "{{\"id\": \"{id}\", \"seen_at\": \"{seen_at}\", \"friction\": \"{friction}\", \
         \"enforcement_fix\": \"wire a gate for {id}\", \"status\": \"open\", \"goal\": \"G011\"}}"
    )
}

// ───────────────────────── incident 1: duplicate primary after a union ─────────────────────────

#[test]
fn incident_1_two_lanes_authored_primaries_red_under_union_green_under_driver() {
    let policy = live_policy();
    let base = join(&[&primary("FRIC-A", "2026-06-10", "base friction")]);
    let lane_1 = format!(
        "{base}{}",
        join(&[&primary(
            "FRIC-NEW",
            "2026-06-11",
            "lane one logged it first"
        )])
    );
    let lane_2 = format!(
        "{base}{}",
        join(&[&primary(
            "FRIC-NEW",
            "2026-06-12",
            "lane two logged it again"
        )])
    );

    // RED: the naive union keeps both primaries and the REAL gate fold fails closed — the exact
    // incident (correct gate behavior, wrong union).
    let red = naive_union(&base, &lane_1, &lane_2);
    assert!(
        gate_codes(&policy, &red).contains("friction_duplicate_primary_row"),
        "the incident must reproduce RED under naive union"
    );

    // GREEN: the driver auto-converts the second author to an event-sourced update row.
    let green = merge_ledgers(&base, &lane_1, &lane_2).expect("driver merges");
    let codes = gate_codes(&policy, &green);
    assert!(
        !codes.contains("friction_duplicate_primary_row"),
        "second author must be auto-converted: {green}"
    );
    assert!(
        codes.is_empty(),
        "the merged fixture ledger is fully green under the live policy: {codes:?}\n{green}"
    );
    let rows = parse_ledger("green", &green).expect("driver output reparses");
    let fric_new: Vec<_> = rows.iter().filter(|row| row.id() == "FRIC-NEW").collect();
    assert_eq!(fric_new.len(), 2, "one primary + one converted update");
    assert!(
        green.contains("\"status_update\": \"open\"")
            && green.contains("enforcement_fix: wire a gate for FRIC-NEW"),
        "conversion carries status -> status_update and folds enforcement_fix into evidence: {green}"
    );
}

#[test]
fn incident_1_conversion_is_side_symmetric_modulo_append_order() {
    let base = join(&[&primary("FRIC-A", "2026-06-10", "base friction")]);
    let lane_1 = format!(
        "{base}{}",
        join(&[&primary(
            "FRIC-NEW",
            "2026-06-11",
            "lane one logged it first"
        )])
    );
    let lane_2 = format!(
        "{base}{}",
        join(&[&primary(
            "FRIC-NEW",
            "2026-06-12",
            "lane two logged it again"
        )])
    );
    let ab = merge_ledgers(&base, &lane_1, &lane_2).expect("merge(a,b)");
    let ba = merge_ledgers(&base, &lane_2, &lane_1).expect("merge(b,a)");
    let mut ab_lines: Vec<&str> = ab.lines().collect();
    let mut ba_lines: Vec<&str> = ba.lines().collect();
    ab_lines.sort_unstable();
    ba_lines.sort_unstable();
    assert_eq!(
        ab_lines, ba_lines,
        "the same logical rows and the same conversion decision, whichever side is ours"
    );
    assert!(
        ab.contains("lane one logged it first") && ba.contains("lane one logged it first"),
        "the earliest author stays primary on both orientations"
    );
}

// ───────────────────────── incident 2: conflict markers / garbage ─────────────────────────

#[test]
fn incident_2_conflict_markers_and_garbage_refuse_the_merge() {
    let base = join(&[&primary("FRIC-A", "2026-06-10", "base friction")]);
    let crashed_union = format!(
        "<<<<<<< HEAD\n{}=======\n{}>>>>>>> other-lane\n",
        join(&[&primary("FRIC-B", "2026-06-11", "ours")]),
        join(&[&primary("FRIC-B", "2026-06-11", "theirs")]),
    );
    let err = merge_ledgers(&base, &crashed_union, &base)
        .expect_err("committed conflict markers must never merge");
    assert_eq!(err.kind(), MergeErrorKind::Parse);

    for garbage in [
        "total garbage\n",
        "{\"id\": \"X\", \"truncated\n",
        "{\"id\": \"X\", \"id\": \"Y\", \"status_update\": \"x\"}\n", // duplicate key
        "[\"a\", \"json\", \"array\"]\n",
    ] {
        let err = merge_ledgers(&base, garbage, &base).expect_err(garbage);
        assert_eq!(err.kind(), MergeErrorKind::Parse, "{garbage}");
        let err = merge_ledgers(garbage, &base, &base).expect_err("garbage base");
        assert_eq!(err.kind(), MergeErrorKind::Parse, "base side: {garbage}");
        let err = merge_ledgers(&base, &base, garbage).expect_err("garbage theirs");
        assert_eq!(err.kind(), MergeErrorKind::Parse, "theirs side: {garbage}");
    }
}

// ───────────────────────── incident 3: byte-divergent logical twins ─────────────────────────

#[test]
fn incident_3_exact_line_dedup_red_logical_dedup_green() {
    let policy = live_policy();
    let base = join(&[&primary("FRIC-A", "2026-06-10", "base friction")]);
    // Same logical primary row, two serializations: compact + ensure_ascii=true escapes on one
    // side, spaced + literal UTF-8 + different key order on the other.
    let ours_bytes = "{\"id\":\"FRIC-TWIN\",\"seen_at\":\"2026-06-12\",\"friction\":\"dash \\u2014 twin\",\"enforcement_fix\":\"same fix\",\"status\":\"open\",\"goal\":\"G011\"}";
    let theirs_bytes = "{\"goal\": \"G011\", \"status\": \"open\", \"enforcement_fix\": \"same fix\", \"friction\": \"dash \u{2014} twin\", \"seen_at\": \"2026-06-12\", \"id\": \"FRIC-TWIN\"}";
    assert_ne!(ours_bytes, theirs_bytes, "byte-divergent");
    assert_eq!(
        serde_json::from_str::<Value>(ours_bytes).unwrap(),
        serde_json::from_str::<Value>(theirs_bytes).unwrap(),
        "logically identical"
    );
    let ours = format!("{base}{ours_bytes}\n");
    let theirs = format!("{base}{theirs_bytes}\n");

    // RED: exact-line dedup (what the incident tooling did) keeps both physical rows, and the
    // REAL gate fold fails closed on the duplicate primary.
    let red = naive_union(&base, &ours, &theirs);
    assert!(
        gate_codes(&policy, &red).contains("friction_duplicate_primary_row"),
        "byte-divergent twins defeat exact-line dedup: {red}"
    );

    // GREEN: parsed-JSON identity collapses the twins to one canonical row.
    let green = merge_ledgers(&base, &ours, &theirs).expect("driver merges");
    let codes = gate_codes(&policy, &green);
    assert!(
        codes.is_empty(),
        "twins collapse to one accounted row: {codes:?}\n{green}"
    );
    let rows = parse_ledger("green", &green).expect("reparses");
    assert_eq!(
        rows.iter().filter(|row| row.id() == "FRIC-TWIN").count(),
        1,
        "exactly one twin survives: {green}"
    );
}

// ───────────────────────── contract: idempotence + canonical stability ─────────────────────────

#[test]
fn merged_output_is_idempotent_and_a_canonical_fixed_point() {
    let base = join(&[&primary("FRIC-A", "2026-06-10", "base friction")]);
    let ours = format!(
        "{base}{}",
        join(&[
            &primary("FRIC-X", "2026-06-11", "ours friction"),
            "{\"id\": \"FRIC-A\", \"seen_at\": \"2026-06-12\", \"status_update\": \"fix-in-flight\"}",
        ])
    );
    let theirs = format!(
        "{base}{}",
        join(&[&primary("FRIC-Y", "2026-06-12", "theirs friction")])
    );
    let out = merge_ledgers(&base, &ours, &theirs).expect("merges");
    assert_eq!(
        merge_ledgers(&out, &out, &out).expect("self-merge"),
        out,
        "merge(out,out,out) == out"
    );
    // Fast-forward stability: merging the result against one unchanged side changes nothing.
    assert_eq!(merge_ledgers(&base, &out, &base).expect("ff"), out);
    // Canonical fixed point: every line is one JSON object and the document re-merges to itself
    // byte-identically, so a driver-merged ledger never produces serialization churn again.
    assert!(
        out.lines()
            .all(|line| line.starts_with('{') && line.ends_with('}'))
    );
}

#[test]
fn commutativity_holds_modulo_the_pinned_append_order_rule() {
    let base = join(&[&primary("FRIC-A", "2026-06-10", "base friction")]);
    let ours = format!(
        "{base}{}",
        join(&[
            &primary("FRIC-X", "2026-06-11", "ours friction"),
            &primary("FRIC-N", "2026-06-12", "second author"),
        ])
    );
    let theirs = format!(
        "{base}{}",
        join(&[
            &primary("FRIC-Y", "2026-06-12", "theirs friction"),
            &primary("FRIC-N", "2026-06-11", "first author"),
        ])
    );
    let ab = merge_ledgers(&base, &ours, &theirs).expect("merge(a,b)");
    let ba = merge_ledgers(&base, &theirs, &ours).expect("merge(b,a)");
    let mut ab_lines: Vec<&str> = ab.lines().collect();
    let mut ba_lines: Vec<&str> = ba.lines().collect();
    ab_lines.sort_unstable();
    ba_lines.sort_unstable();
    assert_eq!(ab_lines, ba_lines, "same logical row set either way");
    // The append-order rule is pinned: base block first, then ours' additions, then theirs'.
    let ab_rows = parse_ledger("ab", &ab).expect("reparses");
    let ids: Vec<&str> = ab_rows.iter().map(|row| row.id()).collect();
    assert_eq!(ids, vec!["FRIC-A", "FRIC-X", "FRIC-N", "FRIC-Y", "FRIC-N"]);
}

// ───────────────────────── live-ledger conformance ─────────────────────────

#[test]
fn live_ledger_is_modeled_and_canonicalization_preserves_the_gate_verdict() {
    let root = repo_root();
    let policy = live_policy();
    let ledger_path = root.join("ci/facade/action-item-accounting/friction-ledger.jsonl");
    let live = std::fs::read_to_string(&ledger_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", ledger_path.display()));

    // The real ledger must be fully modeled — otherwise the driver refuses the exact file it was
    // built for and silently degrades to manual unions again.
    let rows = parse_ledger("live", &live).expect("the live friction ledger is a modeled ledger");
    assert!(rows.len() >= 100, "live census sanity: got {}", rows.len());

    // Self-merge = whole-file canonicalization. The fold the gate computes must be IDENTICAL
    // before and after (canonical serialization is byte policy, never accounting policy).
    let merged = merge_ledgers(&live, &live, &live).expect("live self-merge");
    let before = evaluate_keyed(&policy, &observed(&live));
    let after = evaluate_keyed(&policy, &observed(&merged));
    assert_eq!(
        before, after,
        "canonicalizing the live ledger must not change the friction-accounting verdict"
    );

    // And the canonical form is a fixed point on the live corpus too.
    assert_eq!(
        merge_ledgers(&merged, &merged, &merged).expect("again"),
        merged
    );
}
