//! Mechanical re-derivation lane of the masterplan projection-freshness gate
//! (masterplan v2 consolidation, Sub-AC 4.2).
//!
//! `evaluate_masterplan_v2_projection_freshness` (lib.rs) proves freshness
//! COVERAGE: every projection derived from `/specs/masterplan.json` carries a
//! complete freshness/no-authority contract. This module proves freshness
//! CONTENT: every derived/generated masterplan projection that exists on disk
//! is mechanically re-derivable from `/specs/masterplan.json#masterplan_v2`
//! and fails closed on stale or hand-edited output bytes.
//!
//! Covered projection classes (the `projection_rederivation` corpus, assembled
//! from the tree by the caller — the evaluator itself is pure and does no I/O):
//!
//! - `docs/MASTERPLAN.md` — the generated human compatibility projection.
//!   [`derive_masterplan_md_projection`] re-derives the FULL byte content from
//!   masterplan v2 (canonical authority path, MPV2 id-namespace shape, and the
//!   absorbed/archived plan-surface dispositions); any byte difference means
//!   the projection is stale (masterplan moved, projection did not regenerate)
//!   or hand-edited (projection moved without its source of truth).
//! - flow-metrics ledger (`plan/fabric-loop/flow-metrics/passes/pass-*.json`)
//!   — the ADR-0516 closed-loop per-pass ledger must re-serialize
//!   byte-for-byte through the pinned canonical wire shape, keep a strictly
//!   monotonic contiguous 1-based `pass_seq` under canonical
//!   `pass-{seq:020}.json` filenames, and reference only cards that exist on
//!   the durable coordination plane.
//! - loop-card shard views (`plan/fabric-loop/cards/*.json`) — the item-level
//!   Definition/Seed shard of the plan DAG must re-serialize byte-for-byte,
//!   resolve its `card_id` to a live MPV2 work item, its `program_id` to a
//!   declared masterplan program shard, its `depends_on` refs to existing
//!   cards, and its completion statuses to attached evidence refs.
//! - generated faces — every on-disk generated planning projection named by
//!   the corpus must be declared by a generated-artifact-control-plane row
//!   whose `source_inputs` include `specs/masterplan.json` AND covered by a
//!   `masterplan_v2.projection_freshness` row; an undeclared on-disk generated
//!   planning output is a hand-made artifact and fails closed.
//!
//! The canonical wire writers below intentionally DUPLICATE (never import) the
//! owned writer in `tools/fabric-loop-state-app`: the gate re-derives the
//! bytes independently, so a silent wire-format change on either side fails
//! closed here instead of drifting unnoticed. Metric VALUES are integrity-
//! guarded at write time by the fabric ports (`validate_pass_record` derives
//! them mechanically from raw card timelines); this gate guards everything the
//! committed tree can prove: byte-canonical serialization, ledger ordering,
//! filename agreement, and referential agreement with the plan DAG.
//!
//! Every violation emits the single blocking code
//! [`STALE_PROJECTION_CODE`] (`masterplan_projection_stale`). Carve-outs live
//! as DATA in the corpus, never as evaluator branches. ADR-0083 Tier-3:
//! production code carries no unwrap/expect/panic.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::Finding;

/// Validator id recorded by `masterplan_v2.projection_freshness.rederivation`.
pub const PROJECTION_REDERIVATION_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/masterplan-v2-projection-rederivation";

/// The blocking violation code this lane emits.
pub const STALE_PROJECTION_CODE: &str = "masterplan_projection_stale";

/// Repo-relative path of the generated human compatibility projection.
pub const MASTERPLAN_MD_PATH: &str = "docs/MASTERPLAN.md";

const DISPOSITION_ABSORBED: &str = "absorbed";
const DISPOSITION_ARCHIVED_WITH_PROVENANCE: &str = "archived-with-provenance";
const CARD_STATUSES: [&str; 5] = [
    "defined",
    "ready",
    "blocked",
    "claimed-done-unverified",
    "done-verified",
];
const COMPLETION_STATUSES: [&str; 2] = ["claimed-done-unverified", "done-verified"];

fn stale(key: &str) -> Finding {
    Finding::new(STALE_PROJECTION_CODE, key)
}

/// Evaluate the mechanical re-derivation corpus for every derived/generated
/// masterplan projection. `masterplan` is the `/specs/masterplan.json`
/// document (or a fixture mirroring it); `corpus` is the on-disk projection
/// snapshot assembled by the caller:
///
/// ```jsonc
/// {
///   "masterplan_md": "<full bytes of docs/MASTERPLAN.md>",
///   "flow_metrics_passes": [ { "file_name": "pass-…json", "content": "<bytes>" } ],
///   "loop_cards": [ { "file_name": "MPV2-….json", "content": "<bytes>" } ],
///   "generated_projections_on_disk": [ "docs/machine-readable/….generated.json" ],
///   "generated_artifact_control_plane": { "artifacts": [ … ] }   // optional
/// }
/// ```
///
/// Missing or malformed corpus sections fail closed: a projection surface the
/// gate cannot re-derive is never admitted as fresh.
pub fn evaluate_masterplan_projection_rederivation(
    masterplan: &Value,
    corpus: &Value,
) -> BTreeSet<Finding> {
    let mut findings = BTreeSet::new();

    if !corpus.is_object() {
        findings.insert(stale("<malformed-projection-rederivation-corpus>"));
        return findings;
    }
    let Some(v2) = masterplan.get("masterplan_v2") else {
        findings.insert(stale("<missing-masterplan_v2>"));
        return findings;
    };
    if !v2.is_object() {
        findings.insert(stale("<malformed-masterplan_v2>"));
        return findings;
    }

    evaluate_masterplan_md(masterplan, corpus, &mut findings);
    let card_ids = evaluate_loop_cards(v2, corpus, &mut findings);
    evaluate_flow_metrics_ledger(corpus, &card_ids, &mut findings);
    evaluate_on_disk_generated_projections(v2, corpus, &mut findings);

    findings
}

// ---------------------------------------------------------------------------
// docs/MASTERPLAN.md — full-content re-derivation
// ---------------------------------------------------------------------------

/// Re-derive the full canonical byte content of `docs/MASTERPLAN.md` from the
/// masterplan v2 contract. Deterministic: template constants plus values read
/// from `masterplan_v2` (canonical authority path, live work-item id-namespace
/// shape, and the absorbed/archived former plan surfaces). Returns `Err` with
/// the offending fragment when the masterplan lacks a required input — an
/// underivable projection fails closed.
pub fn derive_masterplan_md_projection(masterplan: &Value) -> Result<String, String> {
    let v2 = masterplan
        .get("masterplan_v2")
        .ok_or_else(|| "masterplan_v2".to_owned())?;
    let authority = v2
        .get("canonical_plan_authority")
        .ok_or_else(|| "masterplan_v2.canonical_plan_authority".to_owned())?;
    let canonical = authority
        .get("path")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .ok_or_else(|| "masterplan_v2.canonical_plan_authority.path".to_owned())?;
    let id_space = authority.get("live_work_item_id_space").ok_or_else(|| {
        "masterplan_v2.canonical_plan_authority.live_work_item_id_space".to_owned()
    })?;
    let id_prefix = id_space
        .get("id_prefix")
        .and_then(Value::as_str)
        .filter(|prefix| !prefix.is_empty())
        .ok_or_else(|| "live_work_item_id_space.id_prefix".to_owned())?;
    let numeric_width = id_space
        .get("numeric_width")
        .and_then(Value::as_u64)
        .filter(|width| (1..=16).contains(width))
        .ok_or_else(|| "live_work_item_id_space.numeric_width".to_owned())?;
    let archived = archived_plan_surfaces(v2);
    if archived.is_empty() {
        return Err("masterplan_v2.surface_dispositions#archived-plan-surfaces".to_owned());
    }

    let fragment = format!("{canonical}#masterplan_v2");
    let namespace = format!(
        "{id_prefix}{}",
        "#".repeat(usize::try_from(numeric_width).unwrap_or(4))
    );
    let archived_list = archived
        .iter()
        .map(|path| format!("`{path}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let companion_specs = archived
        .iter()
        .filter(|path| path.starts_with("/specs/"))
        .map(|path| format!("- {path}\n"))
        .collect::<String>();

    Ok(format!(
        "---\n\
doc_class: MasterPlan\n\
shape: compatibility_projection_non_authoritative\n\
length_cap: 800\n\
authority_tier: 4\n\
status: Accepted\n\
date: 2026-05-19\n\
owners:\n\
- council-architecture\n\
canonical_authority: {canonical}\n\
live_plan_authority: false\n\
read_contract:\n\
\x20 audience:\n\
\x20   - humans\n\
\x20 read_timing_class: on-demand\n\
\x20 freshness_rule: \"Projection only; conflicts resolve to {fragment}.\"\n\
companion_docs:\n\
- /specs/root-hub-pointers.json\n\
{companion_specs}\
- docs/decisions/ADR-0709-general-live-apex.md\n\
authority_chain_declaration: |\n\
\x20 system / developer / user instructions\n\
\x20   > /specs/root-hub-pointers.json\n\
\x20   > docs/AGENTS.md (operating contract until explicit /specs/agent-operating-contract.json PHASE-5 promotion evidence)\n\
\x20   > installed agent-runtime skill and role catalog (for Codex: ~/.codex/skills + ~/.codex/agents; project .codex overlays only when intentionally checked in)\n\
\x20   > {fragment} (sole live plan authority and work-item ID namespace)\n\
\x20   > machine-readable specs and registries under /specs, /registry, /evidence, and /templates (supporting evidence/provenance only unless directly cited by masterplan v2)\n\
\x20   > external/upstream skill documentation (informational only; not vendored into this repo)\n\
\x20   > repo-root Redirect-class files (non-authoritative; lane-thin)\n\
\x20   > working drafts (never authoritative)\n\
purpose: \"Human compatibility projection for the machine-readable Oyatie master plan.\"\n\
doc_status: published\n\
---\n\
# Oyatie Master Plan\n\
\n\
This file is a human compatibility projection only. It is not a live plan authority, does not mint work-item IDs, and does not carry status claims. The canonical master plan, live work-item ID space, dependency DAG, surface dispositions, and read contracts live in `{fragment}`.\n\
\n\
## Current Authority\n\
\n\
- Canonical plan authority: `{canonical}`\n\
- Canonical fragment for this consolidation: `{fragment}`\n\
- Live work-item ID namespace: `{namespace}`, validated by the cloud-ci cross-artifact agreement masterplan-v2 authority check.\n\
- Former plan surfaces ({archived_list}, and legacy agent-harness runtime artifacts) are absorbed provenance or runtime data, not live plan authorities.\n\
\n\
Historical `.omc`/`.omx` planning prompts and local runtime stores may be forensically read only when a gate or masterplan v2 evidence reference asks for them. They never override `{canonical}`.\n\
\n\
## Projection Contract\n\
\n\
This projection intentionally avoids duplicating sequence, scope, status, or dependency detail. Humans use it as a pointer; agents and gates read `{fragment}` directly.\n\
\n\
Any update that adds roadmap content, work-item IDs, readiness status, or sequencing here without a generated-projection freshness gate is stale on arrival and must be rejected."
    ))
}

/// The absorbed/archived former plan surfaces that the human projection must
/// enumerate: `surface_dispositions` rows dispositioned `absorbed` or
/// `archived-with-provenance` whose path is a concrete spec/doc plan surface
/// (no `#` fragment rows, no dot-directory harness stores). Sorted.
fn archived_plan_surfaces(v2: &Value) -> Vec<String> {
    let Some(surfaces) = v2.get("surface_dispositions").and_then(Value::as_array) else {
        return Vec::new();
    };
    let set: BTreeSet<String> = surfaces
        .iter()
        .filter(|surface| {
            surface
                .get("disposition")
                .and_then(Value::as_str)
                .is_some_and(|disposition| {
                    disposition == DISPOSITION_ABSORBED
                        || disposition == DISPOSITION_ARCHIVED_WITH_PROVENANCE
                })
        })
        .filter_map(|surface| surface.get("path").and_then(Value::as_str))
        .map(str::trim)
        .filter(|path| {
            !path.is_empty()
                && !path.contains('#')
                && (path.starts_with("/specs/") || path.starts_with("docs/"))
        })
        .map(str::to_owned)
        .collect();
    set.into_iter().collect()
}

fn evaluate_masterplan_md(masterplan: &Value, corpus: &Value, findings: &mut BTreeSet<Finding>) {
    let Some(on_disk) = corpus.get("masterplan_md").and_then(Value::as_str) else {
        findings.insert(stale("<missing-masterplan-md>"));
        return;
    };
    match derive_masterplan_md_projection(masterplan) {
        Ok(derived) => {
            if derived != on_disk {
                findings.insert(stale(MASTERPLAN_MD_PATH));
            }
        }
        Err(fragment) => {
            findings.insert(stale(&format!(
                "<underivable-{MASTERPLAN_MD_PATH}>@{fragment}"
            )));
        }
    }
}

// ---------------------------------------------------------------------------
// Canonical wire writers (independent re-derivation of the fabric wire shape)
// ---------------------------------------------------------------------------

/// String escaping of the owned loop-state wire format (duplicated by design;
/// see the module doc).
fn push_wire_string(text: &str, out: &mut String) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn push_wire_string_array(values: &[String], indent: usize, out: &mut String) {
    if values.is_empty() {
        out.push_str("[]");
        return;
    }
    out.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('\n');
        out.push_str(&"  ".repeat(indent + 1));
        push_wire_string(value, out);
    }
    out.push('\n');
    out.push_str(&"  ".repeat(indent));
    out.push(']');
}

/// Extract a required string field, or record a stale finding keyed by `key`.
fn wire_str(
    value: &Value,
    field: &str,
    key: &str,
    findings: &mut BTreeSet<Finding>,
) -> Option<String> {
    match value.get(field).and_then(Value::as_str) {
        Some(text) => Some(text.to_owned()),
        None => {
            findings.insert(stale(&format!("<malformed-{key}.{field}>")));
            None
        }
    }
}

/// Extract a required unsigned-integer field, or record a stale finding.
fn wire_num(
    value: &Value,
    field: &str,
    key: &str,
    findings: &mut BTreeSet<Finding>,
) -> Option<u64> {
    match value.get(field).and_then(Value::as_u64) {
        Some(number) => Some(number),
        None => {
            findings.insert(stale(&format!("<malformed-{key}.{field}>")));
            None
        }
    }
}

/// Extract a required string-array field, or record a stale finding.
fn wire_str_array(
    value: &Value,
    field: &str,
    key: &str,
    findings: &mut BTreeSet<Finding>,
) -> Option<Vec<String>> {
    let Some(items) = value.get(field).and_then(Value::as_array) else {
        findings.insert(stale(&format!("<malformed-{key}.{field}>")));
        return None;
    };
    let mut out = Vec::new();
    for item in items {
        let Some(text) = item.as_str() else {
            findings.insert(stale(&format!("<malformed-{key}.{field}>")));
            return None;
        };
        out.push(text.to_owned());
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// Loop-card shard views
// ---------------------------------------------------------------------------

fn evaluate_loop_cards(
    v2: &Value,
    corpus: &Value,
    findings: &mut BTreeSet<Finding>,
) -> BTreeSet<String> {
    let mut card_ids = BTreeSet::new();
    let Some(cards) = corpus.get("loop_cards").and_then(Value::as_array) else {
        findings.insert(stale("<malformed-loop-card-shard>"));
        return card_ids;
    };

    let work_item_ids = id_set(v2.get("work_items"));
    let program_ids = id_set(v2.get("programs"));

    // First pass: collect the declared card-id universe so depends_on edges can
    // be resolved against the WHOLE shard, not just earlier files.
    for card in cards {
        if let Some(id) = card
            .get("content")
            .and_then(Value::as_str)
            .and_then(|content| serde_json::from_str::<Value>(content).ok())
            .and_then(|parsed| {
                parsed
                    .get("card_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
        {
            card_ids.insert(id);
        }
    }

    for (index, card) in cards.iter().enumerate() {
        let slot = format!("loop_cards[{index}]");
        let Some(file_name) = card.get("file_name").and_then(Value::as_str) else {
            findings.insert(stale(&format!("<malformed-{slot}.file_name>")));
            continue;
        };
        let Some(content) = card.get("content").and_then(Value::as_str) else {
            findings.insert(stale(&format!("<malformed-{slot}.content>")));
            continue;
        };
        let Ok(parsed) = serde_json::from_str::<Value>(content) else {
            findings.insert(stale(&format!("<unparseable-card>@{file_name}")));
            continue;
        };

        let Some(card_id) = wire_str(&parsed, "card_id", file_name, findings) else {
            continue;
        };
        let (Some(title), Some(program_id), Some(status)) = (
            wire_str(&parsed, "title", file_name, findings),
            wire_str(&parsed, "program_id", file_name, findings),
            wire_str(&parsed, "status", file_name, findings),
        ) else {
            continue;
        };
        let (Some(depends_on), Some(evidence_refs)) = (
            wire_str_array(&parsed, "depends_on", file_name, findings),
            wire_str_array(&parsed, "evidence_refs", file_name, findings),
        ) else {
            continue;
        };

        // Canonical filename: `{card_id}.json` (the coordination-plane store key).
        if file_name != format!("{card_id}.json") {
            findings.insert(stale(&format!(
                "{card_id}.non_canonical_card_filename@{file_name}"
            )));
        }

        // Byte-canonical re-serialization through the pinned wire shape.
        let mut canonical = String::from("{\n  \"card_id\": ");
        push_wire_string(&card_id, &mut canonical);
        canonical.push_str(",\n  \"title\": ");
        push_wire_string(&title, &mut canonical);
        canonical.push_str(",\n  \"program_id\": ");
        push_wire_string(&program_id, &mut canonical);
        canonical.push_str(",\n  \"depends_on\": ");
        push_wire_string_array(&depends_on, 1, &mut canonical);
        canonical.push_str(",\n  \"status\": ");
        push_wire_string(&status, &mut canonical);
        canonical.push_str(",\n  \"evidence_refs\": ");
        push_wire_string_array(&evidence_refs, 1, &mut canonical);
        canonical.push_str("\n}\n");
        if canonical != content {
            findings.insert(stale(&format!("{card_id}.hand_edited_card_bytes")));
        }

        // Status vocabulary + evidence rule: completion claims require evidence.
        if !CARD_STATUSES.contains(&status.as_str()) {
            findings.insert(stale(&format!("{card_id}.unknown_card_status")));
        } else if COMPLETION_STATUSES.contains(&status.as_str()) && evidence_refs.is_empty() {
            findings.insert(stale(&format!("{card_id}.completion_without_evidence")));
        }

        // The shard derives from the plan DAG: the card id must extend a live
        // MPV2 work item (`<work-item-id>.C<digits>`).
        match card_id.rsplit_once(".C") {
            Some((work_item_id, ordinal))
                if !ordinal.is_empty() && ordinal.bytes().all(|b| b.is_ascii_digit()) =>
            {
                if !work_item_ids.contains(work_item_id) {
                    findings.insert(stale(&format!("{card_id}.dangling_work_item_ref")));
                }
            }
            _ => {
                findings.insert(stale(&format!("{card_id}.non_canonical_card_id")));
            }
        }
        if !program_ids.contains(&program_id) {
            findings.insert(stale(&format!("{card_id}.dangling_program_ref")));
        }
        for dependency in &depends_on {
            if !card_ids.contains(dependency) {
                findings.insert(stale(&format!(
                    "{card_id}.dangling_depends_on@{dependency}"
                )));
            }
        }
    }

    card_ids
}

fn id_set(items: Option<&Value>) -> BTreeSet<String> {
    items
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|item| item.get("id").and_then(Value::as_str))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Flow-metrics ledger
// ---------------------------------------------------------------------------

fn evaluate_flow_metrics_ledger(
    corpus: &Value,
    card_ids: &BTreeSet<String>,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(passes) = corpus.get("flow_metrics_passes").and_then(Value::as_array) else {
        findings.insert(stale("<malformed-flow-metrics-ledger>"));
        return;
    };

    let mut entries: Vec<(&str, &str)> = Vec::new();
    for (index, pass) in passes.iter().enumerate() {
        let slot = format!("flow_metrics_passes[{index}]");
        let (Some(file_name), Some(content)) = (
            pass.get("file_name").and_then(Value::as_str),
            pass.get("content").and_then(Value::as_str),
        ) else {
            findings.insert(stale(&format!("<malformed-{slot}>")));
            continue;
        };
        entries.push((file_name, content));
    }
    entries.sort_by_key(|(file_name, _)| *file_name);

    for (position, (file_name, content)) in entries.iter().enumerate() {
        let expected_seq = position as u64 + 1;
        let Ok(parsed) = serde_json::from_str::<Value>(content) else {
            findings.insert(stale(&format!("<unparseable-pass>@{file_name}")));
            continue;
        };
        let (Some(pass_seq), Some(recorded_at)) = (
            wire_num(&parsed, "pass_seq", file_name, findings),
            wire_num(&parsed, "recorded_at_epoch_s", file_name, findings),
        ) else {
            continue;
        };

        // Append-only, strictly monotonic, contiguous, 1-based ledger under
        // canonical filenames — replays, gaps, and renames fail closed.
        if pass_seq != expected_seq {
            findings.insert(stale(&format!("<non-contiguous-pass-seq>@{file_name}")));
        }
        if *file_name != format!("pass-{pass_seq:020}.json") {
            findings.insert(stale(&format!("<non-canonical-pass-filename>@{file_name}")));
        }

        let Some(cards) = parsed.get("cards").and_then(Value::as_array) else {
            findings.insert(stale(&format!("<malformed-{file_name}.cards>")));
            continue;
        };

        let mut canonical = format!(
            "{{\n  \"pass_seq\": {pass_seq},\n  \"recorded_at_epoch_s\": {recorded_at},\n  \"cards\": "
        );
        let mut card_bodies = Vec::new();
        let mut card_ok = true;
        for (card_index, card) in cards.iter().enumerate() {
            let card_slot = format!("{file_name}.cards[{card_index}]");
            let (Some(card_id), Some(lane_id)) = (
                wire_str(card, "card_id", &card_slot, findings),
                wire_str(card, "lane_id", &card_slot, findings),
            ) else {
                card_ok = false;
                continue;
            };
            let (Some(cycle), Some(latency), Some(rework)) = (
                wire_num(card, "cycle_time_s", &card_slot, findings),
                wire_num(card, "review_latency_s", &card_slot, findings),
                wire_num(card, "rework_count", &card_slot, findings),
            ) else {
                card_ok = false;
                continue;
            };
            // A metric row must measure a card that exists on the durable
            // coordination plane — a phantom card id is not re-derivable.
            if !card_ids.contains(&card_id) {
                findings.insert(stale(&format!(
                    "{card_id}.metric_for_unknown_card@{file_name}"
                )));
            }
            let mut body = String::from("    {\n      \"card_id\": ");
            push_wire_string(&card_id, &mut body);
            body.push_str(",\n      \"lane_id\": ");
            push_wire_string(&lane_id, &mut body);
            body.push_str(&format!(
                ",\n      \"cycle_time_s\": {cycle},\n      \"review_latency_s\": {latency},\n      \"rework_count\": {rework}\n    }}"
            ));
            card_bodies.push(body);
        }
        if !card_ok {
            continue;
        }
        if card_bodies.is_empty() {
            canonical.push_str("[]");
        } else {
            canonical.push_str("[\n");
            canonical.push_str(&card_bodies.join(",\n"));
            canonical.push_str("\n  ]");
        }
        canonical.push_str("\n}\n");
        if canonical != **content {
            findings.insert(stale(&format!("<hand-edited-pass-bytes>@{file_name}")));
        }
    }
}

// ---------------------------------------------------------------------------
// On-disk generated planning projections (generated faces)
// ---------------------------------------------------------------------------

fn evaluate_on_disk_generated_projections(
    v2: &Value,
    corpus: &Value,
    findings: &mut BTreeSet<Finding>,
) {
    let Some(paths) = corpus
        .get("generated_projections_on_disk")
        .and_then(Value::as_array)
    else {
        findings.insert(stale("<malformed-generated-projection-inventory>"));
        return;
    };
    if paths.is_empty() {
        return;
    }

    let freshness_paths: BTreeSet<String> = v2
        .get("projection_freshness")
        .and_then(|freshness| freshness.get("projections"))
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| row.get("path").and_then(Value::as_str))
                .map(normalize_projection_path)
                .collect()
        })
        .unwrap_or_default();
    let declared_faces: BTreeSet<String> = corpus
        .get("generated_artifact_control_plane")
        .and_then(|plane| plane.get("artifacts"))
        .and_then(Value::as_array)
        .map(|artifacts| {
            artifacts
                .iter()
                .filter(|artifact| artifact_sources_include_masterplan(artifact))
                .filter_map(|artifact| artifact.get("path").and_then(Value::as_str))
                .map(normalize_projection_path)
                .collect()
        })
        .unwrap_or_default();

    for path in paths {
        let Some(path) = path.as_str().map(str::trim).filter(|path| !path.is_empty()) else {
            findings.insert(stale("<malformed-generated-projection-inventory>"));
            continue;
        };
        let normalized = normalize_projection_path(path);
        if !declared_faces.contains(&normalized) {
            findings.insert(stale(&format!("{path}.undeclared_generated_projection")));
        }
        if !freshness_paths.contains(&normalized) {
            findings.insert(stale(&format!("{path}.uncovered_generated_projection")));
        }
    }
}

fn artifact_sources_include_masterplan(artifact: &Value) -> bool {
    artifact
        .get("source_inputs")
        .and_then(Value::as_array)
        .is_some_and(|inputs| {
            inputs.iter().any(|input| {
                input.as_str().is_some_and(|path| {
                    let without_fragment = path.split_once('#').map_or(path, |(base, _)| base);
                    normalize_projection_path(without_fragment) == "specs/masterplan.json"
                })
            })
        })
}

fn normalize_projection_path(path: &str) -> String {
    let mut normalized = path.trim();
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped;
    }
    normalized.trim_start_matches('/').to_owned()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use serde_json::{Value, json};

    use super::*;

    fn green_masterplan() -> Value {
        json!({
            "masterplan_v2": {
                "canonical_plan_authority": {
                    "path": "/specs/masterplan.json",
                    "live_work_item_id_space": {
                        "id_prefix": "MPV2-",
                        "numeric_width": 4
                    }
                },
                "surface_dispositions": [
                    {"path": "/specs/masterplan.json", "disposition": "canonical-authority"},
                    {"path": "/specs/masterplan.json#v1-legacy-fragments", "disposition": "absorbed"},
                    {"path": "/specs/master-plan-sequencing.json", "disposition": "absorbed"},
                    {"path": "/specs/planning-closure-contract.json", "disposition": "absorbed"},
                    {"path": "/specs/planning-closure-status-closure-ledger.json", "disposition": "absorbed"},
                    {"path": "docs/MASTERPLAN.md", "disposition": "generated-projection"},
                    {"path": "docs/ROADMAP.md", "disposition": "archived-with-provenance"},
                    {"path": ".omc/**", "disposition": "archived-with-provenance"}
                ],
                "projection_freshness": {
                    "projections": [
                        {"path": "docs/machine-readable/masterplan.generated.json"}
                    ]
                },
                "programs": [{"id": "P-FABRIC"}],
                "work_items": [{"id": "MPV2-0000"}]
            }
        })
    }

    fn canonical_card() -> String {
        "{\n  \"card_id\": \"MPV2-0000.C001\",\n  \"title\": \"Fixture card\",\n  \"program_id\": \"P-FABRIC\",\n  \"depends_on\": [],\n  \"status\": \"defined\",\n  \"evidence_refs\": []\n}\n".to_owned()
    }

    fn canonical_pass() -> String {
        "{\n  \"pass_seq\": 1,\n  \"recorded_at_epoch_s\": 1782989737,\n  \"cards\": [\n    {\n      \"card_id\": \"MPV2-0000.C001\",\n      \"lane_id\": \"lane-a\",\n      \"cycle_time_s\": 10,\n      \"review_latency_s\": 5,\n      \"rework_count\": 0\n    }\n  ]\n}\n".to_owned()
    }

    fn green_corpus(masterplan: &Value) -> Value {
        json!({
            "masterplan_md": derive_masterplan_md_projection(masterplan).unwrap(),
            "flow_metrics_passes": [
                {"file_name": "pass-00000000000000000001.json", "content": canonical_pass()}
            ],
            "loop_cards": [
                {"file_name": "MPV2-0000.C001.json", "content": canonical_card()}
            ],
            "generated_projections_on_disk": []
        })
    }

    fn finding_keys(findings: &std::collections::BTreeSet<Finding>) -> Vec<String> {
        findings.iter().map(|f| f.key.clone()).collect()
    }

    #[test]
    fn green_corpus_rederives_every_projection() {
        let masterplan = green_masterplan();
        let corpus = green_corpus(&masterplan);
        let findings = evaluate_masterplan_projection_rederivation(&masterplan, &corpus);
        assert!(
            findings.is_empty(),
            "green corpus must be green: {findings:?}"
        );
    }

    #[test]
    fn masterplan_md_derivation_is_deterministic_and_underivable_fails_closed() {
        let masterplan = green_masterplan();
        let first = derive_masterplan_md_projection(&masterplan).unwrap();
        let second = derive_masterplan_md_projection(&masterplan).unwrap();
        assert_eq!(first, second, "derivation must be deterministic");
        assert!(first.contains("`MPV2-####`"));
        assert!(first.contains("`/specs/master-plan-sequencing.json`, `/specs/planning-closure-contract.json`, `/specs/planning-closure-status-closure-ledger.json`, `docs/ROADMAP.md`"));
        assert!(
            !first.ends_with('\n'),
            "the on-disk projection carries no trailing newline"
        );

        let underivable = json!({"masterplan_v2": {"canonical_plan_authority": {}}});
        assert!(derive_masterplan_md_projection(&underivable).is_err());
        let corpus = json!({
            "masterplan_md": "anything",
            "flow_metrics_passes": [],
            "loop_cards": [],
            "generated_projections_on_disk": []
        });
        let findings = evaluate_masterplan_projection_rederivation(&underivable, &corpus);
        assert!(
            findings
                .iter()
                .any(|f| f.key.starts_with("<underivable-docs/MASTERPLAN.md>")),
            "underivable projection must fail closed: {findings:?}"
        );
    }

    #[test]
    fn hand_edited_masterplan_md_fails_closed() {
        let masterplan = green_masterplan();
        let mut corpus = green_corpus(&masterplan);
        let edited = format!(
            "{}\n\n## Roadmap\n\n- MPV2-9999 ship everything",
            corpus["masterplan_md"].as_str().unwrap()
        );
        corpus["masterplan_md"] = Value::String(edited);
        let findings = evaluate_masterplan_projection_rederivation(&masterplan, &corpus);
        assert_eq!(finding_keys(&findings), vec![MASTERPLAN_MD_PATH.to_owned()]);
    }

    #[test]
    fn stale_masterplan_md_fails_closed_when_the_plan_moves() {
        // The projection bytes were derived from the OLD masterplan; the plan
        // then changed its id namespace width without regenerating the
        // projection — the stale output must fail closed.
        let old_masterplan = green_masterplan();
        let corpus = green_corpus(&old_masterplan);
        let mut moved = green_masterplan();
        moved["masterplan_v2"]["canonical_plan_authority"]["live_work_item_id_space"]["numeric_width"] =
            json!(5);
        let findings = evaluate_masterplan_projection_rederivation(&moved, &corpus);
        assert_eq!(finding_keys(&findings), vec![MASTERPLAN_MD_PATH.to_owned()]);
    }

    #[test]
    fn hand_edited_card_bytes_and_dangling_refs_fail_closed() {
        let masterplan = green_masterplan();
        let mut corpus = green_corpus(&masterplan);
        // Non-canonical bytes (4-space indent) with a dangling program ref.
        corpus["loop_cards"] = json!([
            {
                "file_name": "MPV2-0000.C001.json",
                "content": "{\n    \"card_id\": \"MPV2-0000.C001\",\n    \"title\": \"Fixture card\",\n    \"program_id\": \"P-GHOST\",\n    \"depends_on\": [\"MPV2-0000.C009\"],\n    \"status\": \"defined\",\n    \"evidence_refs\": []\n}\n"
            }
        ]);
        let findings = evaluate_masterplan_projection_rederivation(&masterplan, &corpus);
        let keys = finding_keys(&findings);
        assert!(
            keys.contains(&"MPV2-0000.C001.hand_edited_card_bytes".to_owned()),
            "{keys:?}"
        );
        assert!(
            keys.contains(&"MPV2-0000.C001.dangling_program_ref".to_owned()),
            "{keys:?}"
        );
        assert!(
            keys.contains(&"MPV2-0000.C001.dangling_depends_on@MPV2-0000.C009".to_owned()),
            "{keys:?}"
        );
    }

    #[test]
    fn completion_status_without_evidence_and_unknown_status_fail_closed() {
        let masterplan = green_masterplan();
        let mut corpus = green_corpus(&masterplan);
        corpus["loop_cards"] = json!([
            {
                "file_name": "MPV2-0000.C001.json",
                "content": "{\n  \"card_id\": \"MPV2-0000.C001\",\n  \"title\": \"Fixture card\",\n  \"program_id\": \"P-FABRIC\",\n  \"depends_on\": [],\n  \"status\": \"done-verified\",\n  \"evidence_refs\": []\n}\n"
            },
            {
                "file_name": "MPV2-0000.C002.json",
                "content": "{\n  \"card_id\": \"MPV2-0000.C002\",\n  \"title\": \"Fixture card\",\n  \"program_id\": \"P-FABRIC\",\n  \"depends_on\": [],\n  \"status\": \"shipped\",\n  \"evidence_refs\": []\n}\n"
            }
        ]);
        // Keep the ledger consistent with the surviving card set.
        corpus["flow_metrics_passes"] = json!([]);
        let findings = evaluate_masterplan_projection_rederivation(&masterplan, &corpus);
        let keys = finding_keys(&findings);
        assert!(
            keys.contains(&"MPV2-0000.C001.completion_without_evidence".to_owned()),
            "{keys:?}"
        );
        assert!(
            keys.contains(&"MPV2-0000.C002.unknown_card_status".to_owned()),
            "{keys:?}"
        );
    }

    #[test]
    fn card_id_outside_the_live_work_item_space_fails_closed() {
        let masterplan = green_masterplan();
        let mut corpus = green_corpus(&masterplan);
        corpus["loop_cards"] = json!([
            {
                "file_name": "MPV2-9999.C001.json",
                "content": "{\n  \"card_id\": \"MPV2-9999.C001\",\n  \"title\": \"Fixture card\",\n  \"program_id\": \"P-FABRIC\",\n  \"depends_on\": [],\n  \"status\": \"defined\",\n  \"evidence_refs\": []\n}\n"
            }
        ]);
        corpus["flow_metrics_passes"] = json!([]);
        let findings = evaluate_masterplan_projection_rederivation(&masterplan, &corpus);
        assert!(
            finding_keys(&findings).contains(&"MPV2-9999.C001.dangling_work_item_ref".to_owned()),
            "{findings:?}"
        );
    }

    #[test]
    fn non_contiguous_or_renamed_ledger_passes_fail_closed() {
        let masterplan = green_masterplan();
        let mut corpus = green_corpus(&masterplan);
        corpus["flow_metrics_passes"] = json!([
            {
                "file_name": "pass-00000000000000000002.json",
                "content": "{\n  \"pass_seq\": 2,\n  \"recorded_at_epoch_s\": 1,\n  \"cards\": []\n}\n"
            }
        ]);
        let findings = evaluate_masterplan_projection_rederivation(&masterplan, &corpus);
        assert!(
            finding_keys(&findings)
                .contains(&"<non-contiguous-pass-seq>@pass-00000000000000000002.json".to_owned()),
            "{findings:?}"
        );

        corpus["flow_metrics_passes"] = json!([
            {
                "file_name": "pass-1.json",
                "content": "{\n  \"pass_seq\": 1,\n  \"recorded_at_epoch_s\": 1,\n  \"cards\": []\n}\n"
            }
        ]);
        let findings = evaluate_masterplan_projection_rederivation(&masterplan, &corpus);
        assert!(
            finding_keys(&findings)
                .contains(&"<non-canonical-pass-filename>@pass-1.json".to_owned()),
            "{findings:?}"
        );
    }

    #[test]
    fn hand_edited_pass_bytes_and_phantom_metric_cards_fail_closed() {
        let masterplan = green_masterplan();
        let mut corpus = green_corpus(&masterplan);
        // Value smuggled in through a hand edit: extra field survives parsing
        // but cannot survive canonical re-serialization.
        corpus["flow_metrics_passes"] = json!([
            {
                "file_name": "pass-00000000000000000001.json",
                "content": "{\n  \"pass_seq\": 1,\n  \"recorded_at_epoch_s\": 1782989737,\n  \"note\": \"hand edit\",\n  \"cards\": [\n    {\n      \"card_id\": \"MPV2-0000.C777\",\n      \"lane_id\": \"lane-a\",\n      \"cycle_time_s\": 10,\n      \"review_latency_s\": 5,\n      \"rework_count\": 0\n    }\n  ]\n}\n"
            }
        ]);
        let findings = evaluate_masterplan_projection_rederivation(&masterplan, &corpus);
        let keys = finding_keys(&findings);
        assert!(
            keys.contains(&"<hand-edited-pass-bytes>@pass-00000000000000000001.json".to_owned()),
            "{keys:?}"
        );
        assert!(
            keys.contains(
                &"MPV2-0000.C777.metric_for_unknown_card@pass-00000000000000000001.json".to_owned()
            ),
            "{keys:?}"
        );
    }

    #[test]
    fn undeclared_on_disk_generated_projection_fails_closed() {
        let masterplan = green_masterplan();
        let mut corpus = green_corpus(&masterplan);
        corpus["generated_projections_on_disk"] =
            json!(["docs/machine-readable/hand-made.generated.json"]);
        let findings = evaluate_masterplan_projection_rederivation(&masterplan, &corpus);
        let keys = finding_keys(&findings);
        assert!(
            keys.contains(
                &"docs/machine-readable/hand-made.generated.json.undeclared_generated_projection"
                    .to_owned()
            ),
            "{keys:?}"
        );
        assert!(
            keys.contains(
                &"docs/machine-readable/hand-made.generated.json.uncovered_generated_projection"
                    .to_owned()
            ),
            "{keys:?}"
        );
    }

    #[test]
    fn declared_and_covered_on_disk_generated_projection_is_green() {
        let masterplan = green_masterplan();
        let mut corpus = green_corpus(&masterplan);
        corpus["generated_projections_on_disk"] =
            json!(["docs/machine-readable/masterplan.generated.json"]);
        corpus["generated_artifact_control_plane"] = json!({
            "artifacts": [
                {
                    "path": "docs/machine-readable/masterplan.generated.json",
                    "source_inputs": ["specs/masterplan.json"]
                }
            ]
        });
        let findings = evaluate_masterplan_projection_rederivation(&masterplan, &corpus);
        assert!(
            findings.is_empty(),
            "declared+covered projection must be green: {findings:?}"
        );
    }

    #[test]
    fn missing_or_malformed_corpus_sections_fail_closed() {
        let masterplan = green_masterplan();
        let findings =
            evaluate_masterplan_projection_rederivation(&masterplan, &Value::String("x".into()));
        assert_eq!(
            finding_keys(&findings),
            vec!["<malformed-projection-rederivation-corpus>".to_owned()]
        );

        let findings = evaluate_masterplan_projection_rederivation(&masterplan, &json!({}));
        let keys = finding_keys(&findings);
        for expected in [
            "<missing-masterplan-md>",
            "<malformed-loop-card-shard>",
            "<malformed-flow-metrics-ledger>",
            "<malformed-generated-projection-inventory>",
        ] {
            assert!(
                keys.contains(&expected.to_owned()),
                "missing {expected} in {keys:?}"
            );
        }

        let corpus = green_corpus(&masterplan);
        let findings =
            evaluate_masterplan_projection_rederivation(&json!({"decisions": []}), &corpus);
        assert_eq!(
            finding_keys(&findings),
            vec!["<missing-masterplan_v2>".to_owned()]
        );
    }

    #[test]
    fn every_finding_uses_the_public_stale_code() {
        let masterplan = green_masterplan();
        let findings = evaluate_masterplan_projection_rederivation(&masterplan, &json!({}));
        assert!(!findings.is_empty());
        for finding in &findings {
            assert_eq!(finding.code, STALE_PROJECTION_CODE);
        }
    }
}
