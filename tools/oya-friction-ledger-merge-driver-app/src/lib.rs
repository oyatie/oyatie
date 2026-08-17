//! # oya-friction-ledger-merge-driver (FRIC-1781370000)
//!
//! Structural three-way Git merge driver for `.omc/ultragoal/friction-ledger.jsonl`.
//!
//! The friction ledger is an append-only, event-sourced JSONL surface: PRIMARY rows (carrying
//! `friction` + `status`) anchor a friction id, UPDATE rows (carrying `status_update`) append
//! disposition transitions onto it, and the `cloud-ci-friction-accounting` gate (ADR-0544) folds
//! the physical rows per id. With parallel agent lanes each appending rows, git's text merge
//! re-conflicts the trailing-line region on every advance of `dev`, and each manual resolution is
//! a fresh corruption opportunity. Three leader incidents in one session are this driver's RED
//! corpus (FRIC-1781370000):
//!
//! 1. **Duplicate primary after a union** — two lanes both authored PRIMARY rows for one FRIC id;
//!    the gate failed closed on `friction_duplicate_primary_row` (correct), but the union should
//!    have auto-converted the second author into an update row.
//! 2. **Conflict markers committed** — a hand-rolled union crashed mid-resolution and committed
//!    raw `<<<<<<<` markers into the ledger.
//! 3. **Exact-line dedup mangling** — byte-divergent but logically identical rows (same parsed
//!    JSON, different serialization bytes across branches) defeated an exact-line dedup pass.
//!
//! ## Merge semantics (the pinned union rule)
//! - Rows are keyed by **logical identity**: parsed-JSON equality, realized as canonical-byte
//!   equality (never raw-byte equality). Identical logical rows dedup to one. Number lexemes are
//!   identity-significant (`1.50` != `1.5` — strictly finer than value equality, so divergent
//!   spellings are kept, never silently collapsed).
//! - **Base rows are preserved in base order.** The ledger is append-only doctrine; a side that
//!   deleted a base row does not delete it from the merge (same trade-off `.gitattributes`
//!   documents for `evidence/audit-chain.jsonl merge=union`: a legitimate redact must be a single
//!   linearised commit on `dev`, not a parallel PR).
//! - **Additions append ours-then-theirs** (the pinned append-order rule): rows present on a side
//!   but not in base append after the base block, ours' additions first, theirs' additions next,
//!   each side in its own order, cross-side logical duplicates emitted once.
//! - **Second-author conversion:** if an id ends up with more than one PRIMARY row, the
//!   established primary wins — the base primary if one exists, else the earliest author decided
//!   by content (`(seen_at, canonical bytes)` minimum), never by merge side, so
//!   `merge(a,b)` and `merge(b,a)` agree on which row stays primary. Every losing primary is
//!   auto-converted to the event-sourced update row the second-author rule prescribes:
//!   `{id, seen_at, status_update: <its status>, evidence: <its evidence + enforcement_fix>,
//!   story/goal carried}`. ALL loser fields outside that pinned shape are dropped —
//!   `friction`, `pipeline_defect`, and any other extras — same id means same friction by
//!   ledger contract.
//! - **Updates never dedup against primaries** (their field shapes are disjoint by the row model),
//!   and distinct update rows for one id all survive: appends are the point of the ledger.
//! - **Known property:** concurrent DIVERGENT `status_update` rows for one id have no total
//!   order across merge orientations — `merge(a,b)` and `merge(b,a)` carry the same row set, but
//!   the ADR-0544 fold takes the physically LAST update as effective status, which then depends
//!   on which side was ours. Inherent to physical-order folding of parallel appends, and no
//!   worse than the text union this replaces; the duplicate-primary guarantee is unaffected.
//!
//! ## Canonical serialization (single-owner coupling, ADR-0546)
//! Output rows are canonically serialized: the ADR-0546 canonical-json kernel
//! (`ci_canonical_json::canonicalize`) is the single owner of escaping, key order,
//! and number-lexeme preservation. The ledger row form pins `sort_keys=true` (stable field order),
//! `ensure_ascii=false` (literal UTF-8 — the repo's settled ADR-0546 dialect), LF. The kernel only
//! emits a pretty (multi-line) form, so this crate adds exactly one documented projection,
//! [`project_single_line`]: in kernel output every raw `0x0A` byte is structural (string
//! literals escape `\n`), so `",\n" -> ", "` then dropping the remaining newlines is a sound
//! bijection onto the single-line JSONL row form. If the kernel ever grows a single-line mode,
//! this projection is the code to delete. Numbers never pass through `serde_json` on the byte
//! path (`serde_json` is used for semantic field access only; conversion rows contain only
//! strings), so the kernel's verbatim number lexemes survive feature unions
//! (`arbitrary_precision`/`preserve_order` — the exact drift class ADR-0546 polices).
//!
//! ## Fail-closed + self-validation (ADR-0548 D7)
//! Any side that does not parse as a modeled ledger — invalid JSON, a non-object row, a blank or
//! missing `id`, duplicate object keys, or an unmodeled row shape — refuses the merge with a
//! nonzero exit so git falls back to a normal conflict; the driver never writes garbage. Before
//! the merged bytes stand, [`merge_ledgers`] re-validates its own output (D7 fixer
//! self-validation): syntactic reparse as a modeled ledger, canonical-byte idempotence, at most
//! one primary per id (the `friction_duplicate_primary_row` shape), conservation (every input
//! logical row is present or has its converted form present), and no orphan-update ids introduced
//! beyond those already present in the inputs.
//!
//! Enforcement layering (honest scope): this driver is the LOCAL automation layer — it only helps
//! actors who configured it via `git config`. Merge authority stays with the cloud-ci gates behind
//! `oya-ci-required` (ADR-0515); the friction-accounting gate remains the canonical backstop that
//! fails closed on whatever an unconfigured actor merges by hand.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

use ci_canonical_json::{CanonError, CanonicalForm, Newline, canonicalize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeErrorKind {
    /// An input is not a modeled ledger (invalid JSON / non-object / unmodeled shape). Exit 2.
    Parse,
    /// The driver declines to merge (e.g. base already violates single-primary). Exit 1.
    Conflict,
    /// The driver's own output failed D7 self-validation — a driver bug, never written. Exit 2.
    Validate,
    Io,
    Usage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeError {
    kind: MergeErrorKind,
    message: String,
}

impl MergeError {
    pub fn new(kind: MergeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> MergeErrorKind {
        self.kind
    }
}

impl Display for MergeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for MergeError {}

/// The row kind under the ledger's event-sourced model (mirrors the ADR-0544 gate's fold).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    /// Anchoring record: non-blank `friction` + `status`, no `status_update`.
    Primary,
    /// Event-sourced append: non-blank `status_update`, no `friction`.
    Update,
}

/// One modeled ledger row: canonical bytes (logical identity AND output bytes), the parsed value
/// for semantic field access, its kind, and its friction id.
#[derive(Debug, Clone)]
pub struct Row {
    canon: String,
    value: Value,
    kind: RowKind,
    id: String,
}

impl Row {
    pub fn canon(&self) -> &str {
        &self.canon
    }

    pub fn kind(&self) -> RowKind {
        self.kind
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

/// The pinned canonical form for one ledger row (see crate docs): stable key order, literal
/// UTF-8, LF, no trailing newline (the renderer owns row terminators), no BOM. `indent_width: 0`
/// because [`project_single_line`] removes structural newlines anyway.
fn ledger_row_form() -> CanonicalForm {
    CanonicalForm {
        ensure_ascii: false,
        indent_width: 0,
        sort_keys: true,
        trailing_newline: false,
        newline: Newline::Lf,
        utf8_bom: false,
    }
}

/// Project the kernel's pretty canonical form onto the single-line JSONL row form.
///
/// Soundness: the ADR-0546 formatter escapes every control character inside string literals
/// (`\n` -> `\\n`), so every raw `0x0A` byte in its output is structural. Rewriting `",\n"` to
/// `", "` and removing the remaining newlines therefore touches only structure, yielding the
/// Python-`json.dumps`-style separators (`", "` / `": "`) the ledger's dominant rows already use.
fn project_single_line(pretty: &str) -> String {
    pretty.replace(",\n", ", ").replace('\n', "")
}

/// Canonicalize one row's raw JSON bytes to the single-line canonical row form.
fn canonical_row(raw: &str) -> Result<String, CanonError> {
    Ok(project_single_line(&canonicalize(raw, &ledger_row_form())?))
}

fn non_blank<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
}

fn classify(label: &str, line_no: usize, id: &str, value: &Value) -> Result<RowKind, MergeError> {
    // The ADR-0544 fold counts a row as PRIMARY whenever `friction` AND `status` are both strings
    // (blank included). The driver's kind assignment must be BIJECTIVE with that fold on every row
    // it accepts — a row the gate would fold as primary but the driver treats as update could
    // launder a duplicate primary past self-validation. So the modeled shapes are strict, and the
    // whole divergence zone (blank/non-string kind fields, a `status_update` KEY on a primary, a
    // `friction` KEY on an update) is refused per ADR-0548 D7 refusal-on-unmodeled-input.
    let has_friction_key = value.get("friction").is_some();
    let has_update_key = value.get("status_update").is_some();
    if non_blank(value, "friction").is_some()
        && non_blank(value, "status").is_some()
        && !has_update_key
    {
        return Ok(RowKind::Primary);
    }
    if non_blank(value, "status_update").is_some() && !has_friction_key {
        return Ok(RowKind::Update);
    }
    Err(MergeError::new(
        MergeErrorKind::Parse,
        format!(
            "{label}: line {line_no}: unmodeled row shape for id `{id}` \
             (primary = non-blank friction+status with no status_update key; update = non-blank \
             status_update with no friction key); refusing to merge"
        ),
    ))
}

/// Parse one ledger document into modeled rows. Blank lines are skipped (matching the ADR-0544
/// collector); everything else must be a modeled row or the whole merge is refused (fail-closed:
/// conflict markers, truncated lines, duplicate object keys, and unmodeled shapes all land here).
pub fn parse_ledger(label: &str, text: &str) -> Result<Vec<Row>, MergeError> {
    let mut rows = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line_no = index + 1;
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let canon = canonical_row(line).map_err(|err| {
            MergeError::new(
                MergeErrorKind::Parse,
                format!("{label}: line {line_no}: {err}"),
            )
        })?;
        // serde parses the ORIGINAL line (not `canon`): both parsers must accept the same bytes
        // or the merge refuses. Keep it that way — e.g. the kernel tolerates a leading BOM that
        // serde rejects; parsing the original keeps the stricter union of the two fail-closed.
        let value: Value = serde_json::from_str(line).map_err(|err| {
            MergeError::new(
                MergeErrorKind::Parse,
                format!("{label}: line {line_no}: not valid JSON: {err}"),
            )
        })?;
        if !value.is_object() {
            return Err(MergeError::new(
                MergeErrorKind::Parse,
                format!("{label}: line {line_no}: ledger row must be a JSON object"),
            ));
        }
        let id = non_blank(&value, "id")
            .ok_or_else(|| {
                MergeError::new(
                    MergeErrorKind::Parse,
                    format!("{label}: line {line_no}: ledger row carries no non-blank `id`"),
                )
            })?
            .to_owned();
        let kind = classify(label, line_no, &id, &value)?;
        rows.push(Row {
            canon,
            value,
            kind,
            id,
        });
    }
    Ok(rows)
}

/// Render modeled rows to the canonical JSONL document (one canonical row per line, LF).
fn render(rows: &[Row]) -> String {
    let mut out = String::new();
    for row in rows {
        out.push_str(&row.canon);
        out.push('\n');
    }
    out
}

/// Content-deterministic ordering for competing added primaries: earliest `seen_at` first, then
/// canonical bytes. Never the merge side, so `merge(a,b)` and `merge(b,a)` pick the same winner.
fn author_sort_key(row: &Row) -> (String, String) {
    (
        non_blank(&row.value, "seen_at").unwrap_or("").to_owned(),
        row.canon.clone(),
    )
}

fn select_primary_winner(merged: &[Row], positions: &[usize], base_kept: usize) -> Option<usize> {
    if let Some(&position) = positions.iter().find(|&&position| position < base_kept) {
        return Some(position);
    }
    positions.iter().copied().min_by(|&a, &b| {
        let left = merged.get(a).map(author_sort_key);
        let right = merged.get(b).map(author_sort_key);
        left.cmp(&right)
    })
}

/// Convert a losing second-author PRIMARY into the event-sourced update row the second-author
/// rule prescribes: `{id, seen_at, status_update: <its status>, evidence: <its evidence +
/// enforcement_fix>, story/goal carried}`. Conversion rows contain only strings, so the byte path
/// stays kernel-owned (no serde_json number formatting can leak into output bytes).
fn convert_second_primary(row: &Row) -> Result<Row, MergeError> {
    let Some(status) = non_blank(&row.value, "status") else {
        return Err(MergeError::new(
            MergeErrorKind::Validate,
            format!(
                "second-author conversion for id `{}` found no non-blank `status` (classifier \
                 invariant violated)",
                row.id
            ),
        ));
    };
    let mut object = serde_json::Map::new();
    object.insert("id".to_owned(), Value::String(row.id.clone()));
    if let Some(seen_at) = non_blank(&row.value, "seen_at") {
        object.insert("seen_at".to_owned(), Value::String(seen_at.to_owned()));
    }
    object.insert("status_update".to_owned(), Value::String(status.to_owned()));
    let evidence = non_blank(&row.value, "evidence");
    let enforcement_fix = non_blank(&row.value, "enforcement_fix");
    let combined = match (evidence, enforcement_fix) {
        (Some(evidence), Some(fix)) => Some(format!("{evidence} | enforcement_fix: {fix}")),
        (Some(evidence), None) => Some(evidence.to_owned()),
        (None, Some(fix)) => Some(format!("enforcement_fix: {fix}")),
        (None, None) => None,
    };
    if let Some(combined) = combined {
        object.insert("evidence".to_owned(), Value::String(combined));
    }
    for carry in ["story", "goal"] {
        if let Some(value) = non_blank(&row.value, carry) {
            object.insert(carry.to_owned(), Value::String(value.to_owned()));
        }
    }
    let value = Value::Object(object);
    let canon = canonical_row(&value.to_string()).map_err(|err| {
        MergeError::new(
            MergeErrorKind::Validate,
            format!(
                "second-author conversion for id `{}` produced non-canonicalizable bytes: {err}",
                row.id
            ),
        )
    })?;
    Ok(Row {
        canon,
        value,
        kind: RowKind::Update,
        id: row.id.clone(),
    })
}

fn orphan_update_ids(rows: &[Row]) -> BTreeSet<String> {
    let mut has_primary: BTreeSet<&str> = BTreeSet::new();
    let mut has_update: BTreeSet<&str> = BTreeSet::new();
    for row in rows {
        match row.kind {
            RowKind::Primary => {
                has_primary.insert(row.id.as_str());
            }
            RowKind::Update => {
                has_update.insert(row.id.as_str());
            }
        }
    }
    has_update
        .difference(&has_primary)
        .map(|id| (*id).to_owned())
        .collect()
}

/// ADR-0548 D7 fixer self-validation: the merged bytes must satisfy the invariants the driver
/// exists to maintain BEFORE they stand. Any failure here is a driver bug surfaced loudly — the
/// caller refuses the merge and git falls back to a normal conflict.
fn self_validate(
    rendered: &str,
    base: &[Row],
    ours: &[Row],
    theirs: &[Row],
    converted: &[(String, String)],
) -> Result<(), MergeError> {
    // D7.1 — syntactic reparse: the output must itself be a modeled ledger.
    let output = parse_ledger("merged-output", rendered).map_err(|err| {
        MergeError::new(
            MergeErrorKind::Validate,
            format!("self-validation: merged output failed to reparse: {err}"),
        )
    })?;
    // Canonical idempotence: re-rendering the reparse must reproduce the exact bytes to be written.
    if render(&output) != rendered {
        return Err(MergeError::new(
            MergeErrorKind::Validate,
            "self-validation: merged output is not canonical-stable (render(parse(out)) != out)",
        ));
    }
    // The friction-accounting duplicate-primary shape: at most one PRIMARY row per id.
    let mut primary_ids: BTreeSet<&str> = BTreeSet::new();
    for row in &output {
        if row.kind == RowKind::Primary && !primary_ids.insert(row.id.as_str()) {
            return Err(MergeError::new(
                MergeErrorKind::Validate,
                format!(
                    "self-validation: merged output carries duplicate primary rows for id `{}`",
                    row.id
                ),
            ));
        }
    }
    // Conservation: every input logical row is present, or its second-author conversion is.
    let output_canons: BTreeSet<&str> = output.iter().map(|row| row.canon.as_str()).collect();
    let converted_sources: BTreeMap<&str, &str> = converted
        .iter()
        .map(|(source, into)| (source.as_str(), into.as_str()))
        .collect();
    for (label, rows) in [("base", base), ("ours", ours), ("theirs", theirs)] {
        for row in rows {
            if output_canons.contains(row.canon.as_str()) {
                continue;
            }
            if let Some(into) = converted_sources.get(row.canon.as_str()) {
                if output_canons.contains(into) {
                    continue;
                }
            }
            return Err(MergeError::new(
                MergeErrorKind::Validate,
                format!(
                    "self-validation: {label} row for id `{}` is missing from the merged output",
                    row.id
                ),
            ));
        }
    }
    // No NEW orphan-update ids: conversion must never strand an update without its primary.
    let mut allowed = orphan_update_ids(base);
    allowed.extend(orphan_update_ids(ours));
    allowed.extend(orphan_update_ids(theirs));
    for id in orphan_update_ids(&output).difference(&allowed) {
        return Err(MergeError::new(
            MergeErrorKind::Validate,
            format!("self-validation: merged output introduces orphan update rows for id `{id}`"),
        ));
    }
    Ok(())
}

/// Merge three friction-ledger snapshots (`%O %A %B`) under the pinned union semantics. Returns
/// the canonical merged document, already D7 self-validated; the caller writes it to `%A`.
pub fn merge_ledgers(base: &str, ours: &str, theirs: &str) -> Result<String, MergeError> {
    let base_rows = parse_ledger("base", base)?;
    let ours_rows = parse_ledger("ours", ours)?;
    let theirs_rows = parse_ledger("theirs", theirs)?;

    // Refuse a base that already violates single-primary: the driver cannot know which base
    // primary is authoritative, and base rows are preserved verbatim by the union rule.
    let mut base_primary_ids: BTreeSet<&str> = BTreeSet::new();
    for row in &base_rows {
        if row.kind == RowKind::Primary && !base_primary_ids.insert(row.id.as_str()) {
            return Err(MergeError::new(
                MergeErrorKind::Conflict,
                format!(
                    "base ledger already carries duplicate primary rows for id `{}`; refusing to \
                     merge (repair base on a linearised commit first)",
                    row.id
                ),
            ));
        }
    }

    // Phase 1 — logical-set union: base rows in base order, then ours' additions, then theirs'
    // additions (the pinned append-order rule). Identity is canonical bytes, so byte-divergent
    // twins (incident 3) collapse to one row here.
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut merged: Vec<Row> = Vec::new();
    for row in &base_rows {
        if seen.insert(row.canon.clone()) {
            merged.push(row.clone());
        }
    }
    let base_kept = merged.len();
    for row in ours_rows.iter().chain(theirs_rows.iter()) {
        if seen.insert(row.canon.clone()) {
            merged.push(row.clone());
        }
    }

    // Phase 2 — second-author conversion (incident 1): every id keeps exactly one PRIMARY; the
    // established/earliest primary wins by content, all others convert to update rows in place.
    let mut primaries_by_id: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (position, row) in merged.iter().enumerate() {
        if row.kind == RowKind::Primary {
            primaries_by_id
                .entry(row.id.clone())
                .or_default()
                .push(position);
        }
    }
    let mut converted: Vec<(String, String)> = Vec::new();
    for positions in primaries_by_id.values() {
        if positions.len() < 2 {
            continue;
        }
        let Some(winner) = select_primary_winner(&merged, positions, base_kept) else {
            continue; // unreachable: positions is non-empty; min_by over non-empty yields Some
        };
        for &position in positions {
            if position == winner {
                continue;
            }
            let Some(source) = merged.get(position) else {
                continue; // unreachable: positions derive from enumerate over `merged`
            };
            let update = convert_second_primary(source)?;
            converted.push((source.canon.clone(), update.canon.clone()));
            if let Some(slot) = merged.get_mut(position) {
                *slot = update;
            }
        }
    }

    // Phase 3 — post-conversion dedup: a conversion may collide with an update row that already
    // exists (e.g. the second author also logged the same transition); identical logical rows
    // still dedup to one. Updates never dedup against primaries: the shapes are disjoint.
    let mut emitted: BTreeSet<String> = BTreeSet::new();
    let mut output_rows: Vec<Row> = Vec::new();
    for row in merged {
        if emitted.insert(row.canon.clone()) {
            output_rows.push(row);
        }
    }

    let rendered = render(&output_rows);
    self_validate(&rendered, &base_rows, &ours_rows, &theirs_rows, &converted)?;
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn primary(id: &str, seen_at: &str, friction: &str) -> String {
        format!(
            "{{\"id\": \"{id}\", \"seen_at\": \"{seen_at}\", \"friction\": \"{friction}\", \
             \"enforcement_fix\": \"wire a gate\", \"status\": \"open\", \"goal\": \"G011\"}}"
        )
    }

    fn update(id: &str, seen_at: &str, status_update: &str) -> String {
        format!(
            "{{\"id\": \"{id}\", \"seen_at\": \"{seen_at}\", \
             \"status_update\": \"{status_update}\", \"evidence\": \"PR #1 merged\"}}"
        )
    }

    fn merge(base: &[&str], ours: &[&str], theirs: &[&str]) -> Result<String, MergeError> {
        let join = |rows: &[&str]| {
            let mut out = String::new();
            for row in rows {
                out.push_str(row);
                out.push('\n');
            }
            out
        };
        merge_ledgers(&join(base), &join(ours), &join(theirs))
    }

    fn kinds_for(output: &str, id: &str) -> Vec<RowKind> {
        parse_ledger("test", output)
            .expect("output reparses")
            .iter()
            .filter(|row| row.id == id)
            .map(|row| row.kind)
            .collect()
    }

    #[test]
    fn disjoint_appends_union_cleanly() {
        let a = primary("FRIC-A", "2026-06-10", "base friction");
        let x = primary("FRIC-X", "2026-06-11", "ours friction");
        let y = primary("FRIC-Y", "2026-06-11", "theirs friction");
        let out = merge(&[&a], &[&a, &x], &[&a, &y]).expect("merges");
        let rows = parse_ledger("out", &out).expect("reparses");
        let ids: Vec<&str> = rows.iter().map(Row::id).collect();
        assert_eq!(
            ids,
            vec!["FRIC-A", "FRIC-X", "FRIC-Y"],
            "base order then ours then theirs"
        );
    }

    #[test]
    fn second_primary_converts_to_update_when_base_holds_the_primary() {
        let a = primary("FRIC-A", "2026-06-10", "base friction");
        let second = primary("FRIC-A", "2026-06-12", "second author re-logged it");
        let out = merge(&[&a], &[&a], &[&a, &second]).expect("merges");
        assert_eq!(
            kinds_for(&out, "FRIC-A"),
            vec![RowKind::Primary, RowKind::Update]
        );
        assert!(
            out.contains("\"status_update\": \"open\""),
            "converted row carries the second primary's status as status_update: {out}"
        );
        assert!(
            out.contains("enforcement_fix: wire a gate"),
            "converted row folds enforcement_fix into evidence: {out}"
        );
    }

    #[test]
    fn both_sides_new_primary_keeps_earliest_author_by_content_not_side() {
        let a = primary("FRIC-A", "2026-06-10", "base friction");
        let early = primary("FRIC-N", "2026-06-11", "first author");
        let late = primary("FRIC-N", "2026-06-12", "second author");
        let ab = merge(&[&a], &[&a, &early], &[&a, &late]).expect("merges");
        let ba = merge(&[&a], &[&a, &late], &[&a, &early]).expect("merges");
        for out in [&ab, &ba] {
            let kinds = kinds_for(out, "FRIC-N");
            assert_eq!(kinds.iter().filter(|k| **k == RowKind::Primary).count(), 1);
            assert!(
                out.contains("\"friction\": \"first author\""),
                "the earliest seen_at stays primary regardless of side: {out}"
            );
            assert!(
                !out.contains("\"friction\": \"second author\""),
                "the later author's narrative is dropped by the pinned conversion shape: {out}"
            );
        }
        let mut ab_lines: Vec<&str> = ab.lines().collect();
        let mut ba_lines: Vec<&str> = ba.lines().collect();
        ab_lines.sort_unstable();
        ba_lines.sort_unstable();
        assert_eq!(
            ab_lines, ba_lines,
            "merge(a,b) == merge(b,a) modulo append order"
        );
    }

    #[test]
    fn logically_identical_rows_dedup_across_byte_divergence() {
        let a = primary("FRIC-A", "2026-06-10", "base friction");
        // Same logical row, three serializations: key order, spacing, — escape vs literal.
        let ours_bytes = "{\"id\":\"FRIC-T\",\"friction\":\"dash \\u2014 here\",\"status\":\"open\",\"enforcement_fix\":\"f\",\"seen_at\":\"2026-06-12\"}";
        let theirs_bytes = "{\"seen_at\": \"2026-06-12\", \"id\": \"FRIC-T\", \"enforcement_fix\": \"f\", \"status\": \"open\", \"friction\": \"dash \u{2014} here\"}";
        assert_ne!(ours_bytes, theirs_bytes, "the twins are byte-divergent");
        let out = merge(&[&a], &[&a, ours_bytes], &[&a, theirs_bytes]).expect("merges");
        assert_eq!(
            kinds_for(&out, "FRIC-T"),
            vec![RowKind::Primary],
            "exactly one copy survives"
        );
    }

    #[test]
    fn updates_never_dedup_against_primaries_and_distinct_updates_survive() {
        let a = primary("FRIC-A", "2026-06-10", "base friction");
        let u1 = update("FRIC-A", "2026-06-11", "fix-in-flight");
        let u2 = update("FRIC-A", "2026-06-12", "RESOLVED");
        let out = merge(&[&a], &[&a, &u1], &[&a, &u2]).expect("merges");
        assert_eq!(
            kinds_for(&out, "FRIC-A"),
            vec![RowKind::Primary, RowKind::Update, RowKind::Update],
            "primary survives and both distinct updates append"
        );
        // Identical logical updates on both sides dedup to one.
        let out = merge(&[&a], &[&a, &u1], &[&a, &u1]).expect("merges");
        assert_eq!(
            kinds_for(&out, "FRIC-A"),
            vec![RowKind::Primary, RowKind::Update]
        );
    }

    #[test]
    fn conflict_markers_fail_closed() {
        let a = primary("FRIC-A", "2026-06-10", "base friction");
        let garbage = format!("<<<<<<< HEAD\n{a}\n=======\n{a}\n>>>>>>> theirs\n");
        let err = merge_ledgers(&format!("{a}\n"), &garbage, &format!("{a}\n"))
            .expect_err("conflict markers must refuse the merge");
        assert_eq!(err.kind(), MergeErrorKind::Parse);
    }

    #[test]
    fn unmodeled_shapes_fail_closed() {
        let a = primary("FRIC-A", "2026-06-10", "base friction");
        for bad in [
            "[1, 2, 3]",                                           // non-object
            "{\"seen_at\": \"2026-06-12\", \"status\": \"open\"}", // no id, no kind
            "{\"id\": \"X\", \"friction\": \"f\"}",                // friction without status
            "{\"id\": \"X\", \"friction\": \"f\", \"status\": \"open\", \"status_update\": \"x\"}", // both kinds
            "{\"id\": \"  \", \"status_update\": \"x\"}", // blank id
            "{\"id\": \"X\", \"id\": \"Y\", \"status_update\": \"x\"}", // duplicate key
            "not json at all",
            // The gate-fold divergence zone (ADR-0544 is_primary counts blank strings): every row
            // where the driver's kind could differ from the gate's fold is refused, not guessed.
            "{\"id\": \"X\", \"friction\": \" \", \"status\": \"open\", \"status_update\": \"y\"}",
            "{\"id\": \"X\", \"friction\": 123, \"status_update\": \"y\"}", // non-string friction key
            "{\"id\": \"X\", \"friction\": \"f\", \"status\": \"  \"}",     // blank status
            "{\"id\": \"X\", \"friction\": \"f\", \"status\": \"open\", \"status_update\": \"\"}", // blank update KEY on a primary
            "{\"id\": 42, \"status_update\": \"y\"}", // non-string id
        ] {
            let err = merge(&[&a], &[&a, bad], &[&a]).expect_err(bad);
            assert_eq!(err.kind(), MergeErrorKind::Parse, "{bad}");
        }
    }

    #[test]
    fn live_row_61_shape_status_key_plus_status_update_is_a_modeled_update() {
        // The live ledger carries one row with BOTH `status` and `status_update` and no `friction`
        // key — the gate folds it as an update (is_primary needs a friction string), and so do we.
        let a = primary("FRIC-A", "2026-06-10", "base friction");
        let mixed = "{\"id\": \"FRIC-A\", \"seen_at\": \"2026-06-11\", \"status_update\": \"escalated\", \"status\": \"open\"}";
        let out = merge(&[&a], &[&a, mixed], &[&a]).expect("modeled update merges");
        assert_eq!(
            kinds_for(&out, "FRIC-A"),
            vec![RowKind::Primary, RowKind::Update]
        );
    }

    #[test]
    fn crlf_input_normalizes_to_lf_canonical_rows() {
        let a = primary("FRIC-A", "2026-06-10", "base friction");
        let x = primary("FRIC-X", "2026-06-11", "ours friction");
        let crlf_side = format!("{a}\r\n{x}\r\n");
        let out = merge_ledgers(&format!("{a}\n"), &crlf_side, &format!("{a}\n")).expect("merges");
        assert!(!out.contains('\r'), "output is LF-only: {out:?}");
        assert_eq!(parse_ledger("out", &out).expect("reparses").len(), 2);
    }

    #[test]
    fn duplicate_primary_in_base_is_refused_as_conflict() {
        let one = primary("FRIC-A", "2026-06-10", "first");
        let two = primary("FRIC-A", "2026-06-11", "second");
        let err = merge(&[&one, &two], &[&one, &two], &[&one, &two])
            .expect_err("corrupt base must be repaired on a linearised commit");
        assert_eq!(err.kind(), MergeErrorKind::Conflict);
    }

    #[test]
    fn merge_is_idempotent_and_output_is_canonical_stable() {
        let a = primary("FRIC-A", "2026-06-10", "base friction");
        let x = primary("FRIC-X", "2026-06-11", "ours friction");
        let y = update("FRIC-A", "2026-06-12", "RESOLVED");
        let out = merge(&[&a], &[&a, &x], &[&a, &y]).expect("merges");
        let again = merge_ledgers(&out, &out, &out).expect("self-merge");
        assert_eq!(again, out, "merge(out,out,out) == out");
        let rows = parse_ledger("out", &out).expect("reparses");
        assert_eq!(render(&rows), out, "render(parse(out)) == out");
        assert!(
            out.lines().all(|line| line.starts_with('{')),
            "every line is one JSON object"
        );
    }

    #[test]
    fn empty_base_unions_both_sides_for_add_add() {
        let x = primary("FRIC-X", "2026-06-11", "ours friction");
        let y = primary("FRIC-Y", "2026-06-11", "theirs friction");
        let out = merge(&[], &[&x], &[&y]).expect("add/add unions");
        let rows = parse_ledger("out", &out).expect("reparses");
        assert_eq!(rows.len(), 2);
        assert_eq!(merge(&[], &[], &[]).expect("empty"), "");
    }

    #[test]
    fn conversion_without_evidence_or_fix_omits_evidence() {
        let base = "{\"id\": \"FRIC-B\", \"seen_at\": \"2026-06-10\", \"friction\": \"f\", \"status\": \"open\", \"enforcement_fix\": \"g\"}";
        let bare = "{\"id\": \"FRIC-B\", \"seen_at\": \"2026-06-12\", \"friction\": \"again\", \"status\": \"queued\"}";
        let out = merge(&[base], &[base], &[base, bare]).expect("merges");
        let rows = parse_ledger("out", &out).expect("reparses");
        let converted = rows
            .iter()
            .find(|row| row.kind == RowKind::Update)
            .expect("conversion happened");
        assert_eq!(non_blank(&converted.value, "status_update"), Some("queued"));
        assert_eq!(
            non_blank(&converted.value, "evidence"),
            None,
            "no fabricated evidence"
        );
    }
}
