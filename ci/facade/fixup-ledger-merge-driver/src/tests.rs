//! Behaviour tests for the fixup-ledger three-way merge.
//!
//! The regression that motivated this crate is [`header_is_carried_by_position_not_by_id`]: an
//! id-keyed resolver silently dropped the id-less schema header. Every other test exists to keep
//! the kernel conservative — it must decline rather than guess.

use super::*;

const HEADER: &str = r#"{"_meta": "FixupTask registry — JSONL append-only. Schema: {id, title, blocker_for}."}"#;

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

fn ids(text: &str) -> Vec<String> {
    text.split('\n')
        .filter(|l| !l.trim().is_empty())
        .skip(1)
        .map(|l| {
            serde_json::from_str::<Value>(l).unwrap()["id"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect()
}

#[test]
fn header_is_carried_by_position_not_by_id() {
    // THE regression. The header has no `id`. A resolver that indexes rows by id and skips
    // falsy ids drops it silently — that deleted it from four branches before anyone noticed.
    let base = ledger(&[row("A", "a")]);
    let ours = ledger(&[row("A", "a"), row("B", "b")]);
    let theirs = ledger(&[row("A", "a"), row("C", "c")]);

    let merged = merge_ledgers(&base, &ours, &theirs).expect("disjoint rows must merge");

    assert!(
        merged.starts_with(HEADER),
        "schema header must survive the merge, verbatim and first:\n{merged}"
    );
    assert_eq!(merged.matches(HEADER).count(), 1, "header must appear once");
}

#[test]
fn two_lanes_each_filing_a_row_merge_without_conflict() {
    // The whole point: this is the case that conflicts by hand on every pair of PRs (GH #1412).
    let base = ledger(&[row("A", "a")]);
    let ours = ledger(&[row("A", "a"), row("B", "b")]);
    let theirs = ledger(&[row("A", "a"), row("C", "c")]);

    let merged = merge_ledgers(&base, &ours, &theirs).unwrap();
    assert_eq!(ids(&merged), vec!["A", "B", "C"]);
}

#[test]
fn a_row_edited_on_one_side_only_takes_the_edit() {
    let base = ledger(&[row("A", "old")]);
    let ours = ledger(&[row("A", "old")]);
    let theirs = ledger(&[row("A", "new")]);
    assert!(merge_ledgers(&base, &ours, &theirs).unwrap().contains("new"));

    // ...and symmetrically.
    let ours = ledger(&[row("A", "new")]);
    let theirs = ledger(&[row("A", "old")]);
    assert!(merge_ledgers(&base, &ours, &theirs).unwrap().contains("new"));
}

#[test]
fn a_row_edited_differently_on_both_sides_conflicts() {
    // union would keep BOTH, producing two rows with one id — silent corruption, since every
    // consumer keys on id. Declining is the only safe answer.
    let base = ledger(&[row("A", "old")]);
    let ours = ledger(&[row("A", "mine")]);
    let theirs = ledger(&[row("A", "yours")]);

    let err = merge_ledgers(&base, &ours, &theirs).expect_err("must decline");
    assert_eq!(err.kind(), MergeErrorKind::Conflict);
}

#[test]
fn the_same_row_added_on_both_sides_is_emitted_once() {
    let base = ledger(&[]);
    let ours = ledger(&[row("A", "a")]);
    let theirs = ledger(&[row("A", "a")]);
    assert_eq!(ids(&merge_ledgers(&base, &ours, &theirs).unwrap()), vec!["A"]);
}

#[test]
fn the_same_id_added_with_different_content_conflicts() {
    let base = ledger(&[]);
    let ours = ledger(&[row("A", "mine")]);
    let theirs = ledger(&[row("A", "yours")]);

    let err = merge_ledgers(&base, &ours, &theirs).expect_err("must decline");
    assert_eq!(err.kind(), MergeErrorKind::Conflict);
}

#[test]
fn deletion_never_wins() {
    // The registry declares itself append-only. A row vanishing in a merge is the exact failure
    // this kernel exists to stop, so a one-sided delete is carried, not honoured.
    let base = ledger(&[row("A", "a"), row("B", "b")]);
    let ours = ledger(&[row("A", "a")]);
    let theirs = ledger(&[row("A", "a"), row("B", "b")]);

    let merged = merge_ledgers(&base, &ours, &theirs).unwrap();
    assert_eq!(ids(&merged), vec!["A", "B"], "the base row must survive");
}

#[test]
fn rows_are_copied_verbatim_never_reserialised() {
    // Byte preservation matters: re-dumping this file has twice produced enormous phantom diffs
    // by reordering keys and re-escaping em-dashes.
    let fancy = r#"{"id":"A","title":"em—dash and \"quotes\"","extra":  1}"#;
    let base = ledger(&[]);
    let ours = ledger(&[fancy.to_owned()]);
    let theirs = ledger(&[]);

    let merged = merge_ledgers(&base, &ours, &theirs).unwrap();
    assert!(
        merged.contains(fancy),
        "row must survive byte-for-byte:\n{merged}"
    );
}

#[test]
fn unparseable_input_is_refused_rather_than_guessed_at() {
    let base = ledger(&[]);
    let ours = format!("{HEADER}\nnot json\n");
    let theirs = ledger(&[]);

    let err = merge_ledgers(&base, &ours, &theirs).expect_err("must refuse");
    assert_eq!(err.kind(), MergeErrorKind::Parse);
}

#[test]
fn a_row_without_a_string_id_is_refused() {
    let base = ledger(&[]);
    let ours = format!("{HEADER}\n{{\"title\":\"no id\"}}\n");
    let theirs = ledger(&[]);

    let err = merge_ledgers(&base, &ours, &theirs).expect_err("must refuse");
    assert_eq!(err.kind(), MergeErrorKind::Parse);
}

#[test]
fn a_first_line_that_carries_an_id_is_refused() {
    // If the header is ever lost upstream, row 1 would be a task row. Silently treating it as a
    // header would bake the loss in; refuse instead.
    let headerless = format!("{}\n", row("A", "a"));
    let err = merge_ledgers(&headerless, &headerless, &headerless).expect_err("must refuse");
    assert_eq!(err.kind(), MergeErrorKind::Parse);
}

#[test]
fn a_duplicate_id_on_one_side_is_refused() {
    let base = ledger(&[]);
    let ours = ledger(&[row("A", "one"), row("A", "two")]);
    let theirs = ledger(&[]);

    let err = merge_ledgers(&base, &ours, &theirs).expect_err("must refuse");
    assert_eq!(err.kind(), MergeErrorKind::Parse);
}

#[test]
fn divergent_header_edits_conflict() {
    let base = ledger(&[]);
    let ours = format!("{{\"_meta\": \"ours\"}}\n");
    let theirs = format!("{{\"_meta\": \"theirs\"}}\n");

    let err = merge_ledgers(&base, &ours, &theirs).expect_err("must decline");
    assert_eq!(err.kind(), MergeErrorKind::Conflict);
}

#[test]
fn merging_is_order_stable_and_idempotent() {
    let base = ledger(&[row("A", "a")]);
    let ours = ledger(&[row("A", "a"), row("B", "b")]);
    let theirs = ledger(&[row("A", "a"), row("C", "c")]);

    let once = merge_ledgers(&base, &ours, &theirs).unwrap();
    let twice = merge_ledgers(&once, &once, &once).unwrap();
    assert_eq!(once, twice, "merging a merged ledger must be a no-op");
}

#[test]
fn no_row_present_on_any_side_is_ever_lost() {
    // The post-condition the validator enforces, exercised directly.
    let base = ledger(&[row("A", "a"), row("B", "b")]);
    let ours = ledger(&[row("A", "a"), row("B", "b"), row("C", "c")]);
    let theirs = ledger(&[row("A", "a"), row("B", "b"), row("D", "d")]);

    let merged = merge_ledgers(&base, &ours, &theirs).unwrap();
    for id in ["A", "B", "C", "D"] {
        assert!(ids(&merged).contains(&id.to_owned()), "lost {id}");
    }
}
