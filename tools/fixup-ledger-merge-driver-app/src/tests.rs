//! Behaviour tests for the fixup-ledger three-way merge.
//!
//! Two regressions anchor this file:
//!
//! * [`header_is_carried_by_position_not_by_id`] — an id-keyed resolver silently dropped the
//!   id-less schema header from four branches.
//! * [`a_conflict_still_carries_every_row_from_both_sides`] — the first cut of this driver exited
//!   nonzero WITHOUT writing, on the false belief that git would then re-run its own text merge.
//!   It does not; it takes whatever is in `%A`. That left `ours` alone, unmarked, with theirs'
//!   rows absent — a clean-looking file that loses data on a reflexive `git add`.
//!
//! Both are the same failure class: silent loss that looks like success.

use super::*;

const HEADER: &str = r#"{"_meta": "FixupTask registry — JSONL append-only. Schema: {id, title}."}"#;

fn row(id: &str, title: &str) -> String {
    format!(r#"{{"id":"{id}","title":"{title}"}}"#)
}

fn ledger(rows: &[String]) -> String {
    let mut out = String::from(HEADER);
    for r in rows {
        out.push('\n');
        out.push_str(r);
    }
    out.push('\n');
    out
}

fn ledger_with(header: &str, rows: &[String]) -> String {
    let mut out = String::from(header);
    for r in rows {
        out.push('\n');
        out.push_str(r);
    }
    out.push('\n');
    out
}

fn clean(base: &str, ours: &str, theirs: &str) -> String {
    let merged = merge_ledgers(base, ours, theirs).expect("must merge");
    assert!(!merged.conflicted, "expected a clean merge:\n{}", merged.content);
    merged.content
}

fn ids(text: &str) -> Vec<String> {
    text.split('\n')
        .filter(|l| !l.trim().is_empty())
        .skip(1)
        .filter_map(|l| {
            serde_json::from_str::<Value>(l)
                .ok()?
                .get("id")?
                .as_str()
                .map(str::to_owned)
        })
        .collect()
}

#[test]
fn header_is_carried_by_position_not_by_id() {
    // The header has no `id`. A resolver that indexes rows by id and skips falsy ids drops it
    // silently — that deleted it from four branches before anyone noticed.
    let merged = clean(
        &ledger(&[row("A", "a")]),
        &ledger(&[row("A", "a"), row("B", "b")]),
        &ledger(&[row("A", "a"), row("C", "c")]),
    );
    assert!(merged.starts_with(HEADER), "header must survive, first:\n{merged}");
    assert_eq!(merged.matches(HEADER).count(), 1, "header must appear once");
}

#[test]
fn a_conflict_still_carries_every_row_from_both_sides() {
    // THE regression. Git does not re-run its text merge when a driver exits nonzero; it takes
    // whatever is in `%A`. Declining to write loses theirs' rows with no marker to show it.
    let base = ledger(&[row("A", "base")]);
    let ours = ledger(&[row("A", "MINE")]);
    let theirs = ledger(&[row("A", "THEIRS"), row("NEW", "important new row")]);

    let merged = merge_ledgers(&base, &ours, &theirs).expect("conflict must still produce content");
    assert!(merged.conflicted, "must report a conflict");

    assert!(merged.content.contains("MINE"), "ours' row must survive");
    assert!(merged.content.contains("THEIRS"), "theirs' row must survive");
    assert!(
        merged.content.contains("important new row"),
        "theirs' UNRELATED new row must survive a conflict elsewhere:\n{}",
        merged.content
    );
    assert!(merged.content.contains("base"), "the base side must be shown");

    for marker in [OURS_MARKER, BASE_MARKER, SPLIT_MARKER, THEIRS_MARKER] {
        assert!(
            merged.content.contains(marker),
            "missing {marker} — an unmarked conflict reads as a clean file:\n{}",
            merged.content
        );
    }
    assert_sides_are_under_their_own_markers(&merged.content, "MINE", "THEIRS");
}

/// Presence is not enough: a block that files theirs' content under `<<<<<<< ours` contains every
/// marker and every side, so it passes a presence-only check — and a human resolving "keep ours"
/// then silently keeps theirs. Same looks-right-is-wrong family as everything else this crate
/// guards against, so the ORDER is asserted, not just the contents.
fn assert_sides_are_under_their_own_markers(content: &str, ours_needle: &str, theirs_needle: &str) {
    let at = |needle: &str| {
        content
            .find(needle)
            .unwrap_or_else(|| panic!("{needle} absent from:\n{content}"))
    };
    let (ours_marker, base_marker) = (at(OURS_MARKER), at(BASE_MARKER));
    let (split, theirs_marker) = (at(SPLIT_MARKER), at(THEIRS_MARKER));
    assert!(
        ours_marker < at(ours_needle) && at(ours_needle) < base_marker,
        "ours content must sit between the ours and base markers:\n{content}"
    );
    assert!(
        split < at(theirs_needle) && at(theirs_needle) < theirs_marker,
        "theirs content must sit between the split and theirs markers:\n{content}"
    );
}

#[test]
fn two_lanes_each_filing_a_row_merge_without_conflict() {
    // The case that conflicts by hand on every pair of PRs (GH #1412).
    let merged = clean(
        &ledger(&[row("A", "a")]),
        &ledger(&[row("A", "a"), row("B", "b")]),
        &ledger(&[row("A", "a"), row("C", "c")]),
    );
    assert_eq!(ids(&merged), vec!["A", "B", "C"]);
}

#[test]
fn a_row_edited_on_one_side_only_takes_the_edit() {
    let base = ledger(&[row("A", "old")]);
    assert!(clean(&base, &base, &ledger(&[row("A", "new")])).contains("new"));
    assert!(clean(&base, &ledger(&[row("A", "new")]), &base).contains("new"));
}

#[test]
fn a_header_edited_on_one_side_only_is_not_a_conflict() {
    // Comparing only ours-vs-theirs called a ONE-sided header edit a conflict. A `_meta` schema
    // bump is not hypothetical (ADR-0622 proposes one), and under the old behaviour the first
    // bump would conflict with every concurrent lane — the exact problem this driver removes.
    let new_header = r#"{"_meta": "v2 schema"}"#;
    let merged = clean(
        &ledger(&[row("A", "a")]),
        &ledger(&[row("A", "a")]),
        &ledger_with(new_header, &[row("A", "a"), row("B", "b")]),
    );
    assert!(merged.starts_with(new_header), "the one-sided header edit must win");
    assert_eq!(ids(&merged), vec!["A", "B"], "and the row must come with it");
}

#[test]
fn a_header_edited_differently_on_both_sides_conflicts_but_keeps_both() {
    let merged = merge_ledgers(
        &ledger(&[]),
        &ledger_with(r#"{"_meta":"ours"}"#, &[]),
        &ledger_with(r#"{"_meta":"theirs"}"#, &[]),
    )
    .expect("must still produce content");
    assert!(merged.conflicted);
    assert!(merged.content.contains(r#"{"_meta":"ours"}"#));
    assert!(merged.content.contains(r#"{"_meta":"theirs"}"#));
}

#[test]
fn a_row_edited_differently_on_both_sides_conflicts() {
    // union would keep BOTH as real rows, producing two rows with one id — silent corruption,
    // since every consumer keys on id. Markers make it visible instead.
    let merged = merge_ledgers(
        &ledger(&[row("A", "old")]),
        &ledger(&[row("A", "mine")]),
        &ledger(&[row("A", "yours")]),
    )
    .unwrap();
    assert!(merged.conflicted);
}

#[test]
fn the_same_row_added_on_both_sides_is_emitted_once() {
    let merged = clean(&ledger(&[]), &ledger(&[row("A", "a")]), &ledger(&[row("A", "a")]));
    assert_eq!(ids(&merged), vec!["A"]);
}

#[test]
fn the_same_id_added_with_different_content_conflicts() {
    let merged = merge_ledgers(&ledger(&[]), &ledger(&[row("A", "mine")]), &ledger(&[row("A", "yours")]))
        .unwrap();
    assert!(merged.conflicted);
    assert!(merged.content.contains("mine") && merged.content.contains("yours"));
}

#[test]
fn a_one_sided_deletion_does_not_win() {
    let base = ledger(&[row("A", "a"), row("B", "b")]);
    let merged = clean(&base, &ledger(&[row("A", "a")]), &base);
    assert_eq!(ids(&merged), vec!["A", "B"], "the base row must survive");
}

#[test]
fn a_two_sided_deletion_also_does_not_win() {
    // Only this case reaches the final base pass. It is deliberate — the registry declares itself
    // append-only, so a legitimate redaction is a linearised commit on `dev`, not a merge outcome.
    // Without this test, deleting that pass leaves the whole suite green.
    let base = ledger(&[row("A", "a"), row("B", "b")]);
    let one_sided = ledger(&[row("A", "a")]);
    let merged = clean(&base, &one_sided, &one_sided);
    assert_eq!(ids(&merged), vec!["A", "B"], "a row deleted by BOTH sides is still carried");
}

#[test]
fn rows_are_copied_verbatim_never_reserialised() {
    // Re-dumping this file has twice produced enormous phantom diffs by reordering keys and
    // re-escaping em-dashes.
    let fancy = r#"{"id":"A","title":"em—dash and \"quotes\"","extra":  1}"#;
    let merged = clean(&ledger(&[]), &ledger(&[fancy.to_owned()]), &ledger(&[]));
    assert!(merged.contains(fancy), "row must survive byte-for-byte:\n{merged}");
}

#[test]
fn unparseable_input_is_refused_rather_than_guessed_at() {
    let err = merge_ledgers(&ledger(&[]), &format!("{HEADER}\nnot json\n"), &ledger(&[]))
        .expect_err("must refuse");
    assert_eq!(err.kind(), MergeErrorKind::Parse);
}

#[test]
fn a_row_without_a_string_id_is_refused() {
    let err = merge_ledgers(&ledger(&[]), &format!("{HEADER}\n{{\"title\":\"no id\"}}\n"), &ledger(&[]))
        .expect_err("must refuse");
    assert_eq!(err.kind(), MergeErrorKind::Parse);
}

#[test]
fn a_first_line_that_carries_an_id_is_refused() {
    // If the header were ever lost upstream, row 1 would be a task row. Treating it as a header
    // would bake the loss in.
    let headerless = format!("{}\n", row("A", "a"));
    let err = merge_ledgers(&headerless, &headerless, &headerless).expect_err("must refuse");
    assert_eq!(err.kind(), MergeErrorKind::Parse);
}

#[test]
fn a_duplicate_id_on_one_side_is_refused() {
    let err = merge_ledgers(&ledger(&[]), &ledger(&[row("A", "one"), row("A", "two")]), &ledger(&[]))
        .expect_err("must refuse");
    assert_eq!(err.kind(), MergeErrorKind::Parse);
}

#[test]
fn merging_is_order_stable_and_idempotent() {
    let once = clean(
        &ledger(&[row("A", "a")]),
        &ledger(&[row("A", "a"), row("B", "b")]),
        &ledger(&[row("A", "a"), row("C", "c")]),
    );
    assert_eq!(once, clean(&once, &once, &once), "merging a merged ledger must be a no-op");
}

#[test]
fn no_row_present_on_any_side_is_ever_lost() {
    let merged = clean(
        &ledger(&[row("A", "a"), row("B", "b")]),
        &ledger(&[row("A", "a"), row("B", "b"), row("C", "c")]),
        &ledger(&[row("A", "a"), row("B", "b"), row("D", "d")]),
    );
    for id in ["A", "B", "C", "D"] {
        assert!(ids(&merged).contains(&id.to_owned()), "lost {id}");
    }
}

#[test]
fn validate_fires_when_a_row_would_be_dropped() {
    // `validate` cannot trip on the unmutated kernel — every id is emitted unconditionally. It is
    // a REGRESSION guard, not a runtime safety net, and this pins that it actually guards. Without
    // this test, deleting the validate call leaves the suite green.
    let base_text = ledger(&[row("A", "a")]);
    let ours_text = ledger(&[row("A", "a"), row("B", "b")]);
    let theirs_text = ledger(&[]);
    let base = parse(&base_text, "base").unwrap();
    let ours = parse(&ours_text, "ours").unwrap();
    let theirs = parse(&theirs_text, "theirs").unwrap();

    let dropped = ledger(&[row("A", "a")]); // B missing
    let err = validate(&dropped, &base, &ours, &theirs).expect_err("must catch the dropped row");
    assert_eq!(err.kind(), MergeErrorKind::Validate);
    assert!(err.to_string().contains('B'), "must name the lost row: {err}");
}

#[test]
fn unmodelled_input_still_yields_a_file_carrying_every_side() {
    // The exit-2 twin of `a_conflict_still_carries_every_row_from_both_sides`. The kernel cannot
    // merge a malformed side, but the CALLER must not respond by leaving `%A` alone: git takes
    // `%A` as the conflicted tree, so abstaining hands back `ours` looking clean and complete
    // while the other side's rows vanish. Verified with a real `git merge` before this test.
    //
    // Reachable in practice: the ledger has no schema validator, so one lane appending a row with
    // no `id` would otherwise make EVERY later merge of that file silently present `ours`.
    let base = ledger(&[row("A", "base")]);
    let ours = ledger(&[row("A", "MINE")]);
    let theirs = format!(
        "{HEADER}\n{}\n{}\n{{\"no_id_field\":true}}\n",
        row("A", "base"),
        row("IMPORTANT", "theirs only")
    );

    assert_eq!(
        merge_ledgers(&base, &ours, &theirs).expect_err("kernel cannot model it").kind(),
        MergeErrorKind::Parse
    );

    let fallback = whole_file_conflict(&base, &ours, &theirs);
    assert!(fallback.contains("MINE"), "ours must survive");
    assert!(fallback.contains("IMPORTANT"), "theirs' unrelated row must survive:\n{fallback}");
    assert!(fallback.contains("no_id_field"), "the offending row must be visible to fix");
    for marker in [OURS_MARKER, BASE_MARKER, SPLIT_MARKER, THEIRS_MARKER] {
        assert!(fallback.contains(marker), "missing {marker}");
    }
    assert_sides_are_under_their_own_markers(&fallback, "MINE", "IMPORTANT");
}

#[test]
fn a_clean_merge_is_actually_validated_not_just_validatable() {
    // Scope, stated honestly because this claim has been falsified three times: deleting the
    // `validate(...)` CALL SITE still leaves this suite green, and no black-box assertion can
    // change that. `validate` cannot fire on today's kernel — every id is emitted unconditionally
    // — so "ran" and "did not run" are observationally identical from outside. This test pins the
    // post-condition through the real entry point, which is worth having; it does NOT pin that
    // validate is wired in. Only a mutation harness could.
    let base = ledger(&[row("A", "a")]);
    let ours = ledger(&[row("A", "a"), row("B", "b")]);
    let theirs = ledger(&[row("A", "a"), row("C", "c")]);
    let merged = merge_ledgers(&base, &ours, &theirs).expect("clean merge");
    assert!(!merged.conflicted);

    // Every id from every side is present in the emitted content — the post-condition validate
    // enforces, asserted through `merge_ledgers` rather than by calling `validate` directly.
    let present = ids(&merged.content);
    for id in ["A", "B", "C"] {
        assert!(present.contains(&id.to_owned()), "merge_ledgers dropped {id}");
    }
    assert_eq!(present.len(), 3, "and emitted nothing extra: {present:?}");
}
