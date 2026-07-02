// Contract tests for the owned-Rust Hermes forensic ledger. ADR-0083 Tier-3:
// integration tests use unwrap/expect to assert invariants.
//
// The SQLite fixture is built byte-by-byte in this file (schema page, table
// interior page, two leaf pages, a two-page overflow chain, and a short
// pre-migration record), so the reader's parsing of the real on-disk format is
// proven hermetically without any sqlite3/shell/python dependency.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;

use oya_masterplan_hermes_forensic_ledger_app::json::{Json, to_pretty};
use oya_masterplan_hermes_forensic_ledger_app::ledger::{
    CLAIMED_DONE_UNVERIFIED, EVIDENCE_ATTACHED, build_claims, extract_board, extract_github_urls,
    insert_key_before, iso_utc, locate_key_value, replace_key_value, scan_evidence_refs,
};
use oya_masterplan_hermes_forensic_ledger_app::sqlite::{SqliteDb, parse_create_table_columns};

const PAGE_SIZE: usize = 512;
const USABLE: usize = PAGE_SIZE; // reserved = 0

const FIXTURE_DDL: &str = "CREATE TABLE tasks (\n    id TEXT PRIMARY KEY, -- card id\n    title TEXT NOT NULL,\n    body TEXT,\n    status TEXT NOT NULL,\n    completed_at INTEGER,\n    branch_name TEXT,\n    result TEXT\n)";

#[derive(Clone)]
enum V {
    Null,
    Int(i64),
    Text(String),
}

fn enc_varint(mut value: u64) -> Vec<u8> {
    if value <= 0x7f {
        return vec![u8::try_from(value).unwrap()];
    }
    let mut groups = Vec::new();
    while value > 0 {
        groups.push(u8::try_from(value & 0x7f).unwrap());
        value >>= 7;
    }
    groups.reverse();
    let last = groups.len() - 1;
    for group in &mut groups[..last] {
        *group |= 0x80;
    }
    groups
}

fn record(values: &[V]) -> Vec<u8> {
    let mut serials: Vec<Vec<u8>> = Vec::new();
    let mut body: Vec<u8> = Vec::new();
    for value in values {
        match value {
            V::Null => serials.push(enc_varint(0)),
            V::Int(i) => {
                serials.push(enc_varint(6));
                body.extend_from_slice(&i.to_be_bytes());
            }
            V::Text(s) => {
                serials.push(enc_varint(13 + 2 * s.len() as u64));
                body.extend_from_slice(s.as_bytes());
            }
        }
    }
    let serial_len: usize = serials.iter().map(Vec::len).sum();
    let mut header_len = serial_len + 1;
    loop {
        let candidate = enc_varint(header_len as u64).len() + serial_len;
        if candidate == header_len {
            break;
        }
        header_len = candidate;
    }
    let mut out = enc_varint(header_len as u64);
    for serial in serials {
        out.extend_from_slice(&serial);
    }
    out.extend_from_slice(&body);
    out
}

/// Build one table-leaf cell; spills into `overflow_pages` (page numbers
/// starting at `overflow_start_page`) when the payload exceeds the inline max.
fn leaf_cell(
    payload: &[u8],
    rowid: u64,
    overflow_start_page: u32,
    overflow_pages: &mut Vec<Vec<u8>>,
) -> Vec<u8> {
    let p = payload.len();
    let x = USABLE - 35;
    let mut cell = enc_varint(p as u64);
    cell.extend_from_slice(&enc_varint(rowid));
    if p <= x {
        cell.extend_from_slice(payload);
        return cell;
    }
    let m = ((USABLE - 12) * 32 / 255) - 23;
    let k = m + (p - m) % (USABLE - 4);
    let inline = if k <= x { k } else { m };
    cell.extend_from_slice(&payload[..inline]);
    cell.extend_from_slice(&overflow_start_page.to_be_bytes());
    let mut rest = &payload[inline..];
    let mut page_no = overflow_start_page;
    while !rest.is_empty() {
        let take = rest.len().min(USABLE - 4);
        let mut page = vec![0u8; PAGE_SIZE];
        let next: u32 = if take < rest.len() { page_no + 1 } else { 0 };
        page[0..4].copy_from_slice(&next.to_be_bytes());
        page[4..4 + take].copy_from_slice(&rest[..take]);
        overflow_pages.push(page);
        rest = &rest[take..];
        page_no += 1;
    }
    cell
}

fn table_leaf_page(cells: &[Vec<u8>], header_off: usize) -> Vec<u8> {
    let mut page = vec![0u8; PAGE_SIZE];
    let total: usize = cells.iter().map(Vec::len).sum();
    let content_start = PAGE_SIZE - total;
    page[header_off] = 13;
    page[header_off + 3..header_off + 5]
        .copy_from_slice(&u16::try_from(cells.len()).unwrap().to_be_bytes());
    page[header_off + 5..header_off + 7]
        .copy_from_slice(&u16::try_from(content_start).unwrap().to_be_bytes());
    let mut offset = content_start;
    let mut pointer = header_off + 8;
    for cell in cells {
        page[pointer..pointer + 2].copy_from_slice(&u16::try_from(offset).unwrap().to_be_bytes());
        pointer += 2;
        page[offset..offset + cell.len()].copy_from_slice(cell);
        offset += cell.len();
    }
    page
}

fn table_interior_page(entries: &[(u32, u64)], rightmost: u32) -> Vec<u8> {
    let mut page = vec![0u8; PAGE_SIZE];
    let cells: Vec<Vec<u8>> = entries
        .iter()
        .map(|(child, key)| {
            let mut cell = child.to_be_bytes().to_vec();
            cell.extend_from_slice(&enc_varint(*key));
            cell
        })
        .collect();
    let total: usize = cells.iter().map(Vec::len).sum();
    let content_start = PAGE_SIZE - total;
    page[0] = 5;
    page[3..5].copy_from_slice(&u16::try_from(cells.len()).unwrap().to_be_bytes());
    page[5..7].copy_from_slice(&u16::try_from(content_start).unwrap().to_be_bytes());
    page[8..12].copy_from_slice(&rightmost.to_be_bytes());
    let mut offset = content_start;
    let mut pointer = 12;
    for cell in cells {
        page[pointer..pointer + 2].copy_from_slice(&u16::try_from(offset).unwrap().to_be_bytes());
        pointer += 2;
        page[offset..offset + cell.len()].copy_from_slice(&cell);
        offset += cell.len();
    }
    page
}

/// Assemble the fixture database image:
/// page 1 schema leaf, page 2 tasks interior (-> 3, rightmost 4),
/// page 3 leaf (done card with PR URL result, todo card, SHORT done record),
/// page 4 leaf (overflow done card), pages 5-6 overflow chain.
fn fixture_db() -> Vec<u8> {
    let schema_record = record(&[
        V::Text("table".into()),
        V::Text("tasks".into()),
        V::Text("tasks".into()),
        V::Int(2),
        V::Text(FIXTURE_DDL.into()),
    ]);
    let mut no_overflow = Vec::new();
    let schema_cell = leaf_cell(&schema_record, 1, 0, &mut no_overflow);
    assert!(no_overflow.is_empty(), "schema record must stay inline");
    let mut page1 = table_leaf_page(&[schema_cell], 100);
    page1[0..16].copy_from_slice(b"SQLite format 3\0");
    page1[16..18].copy_from_slice(&u16::try_from(PAGE_SIZE).unwrap().to_be_bytes());
    page1[18] = 1; // write version: legacy
    page1[19] = 1; // read version: legacy
    page1[20] = 0; // reserved space
    page1[28..32].copy_from_slice(&6u32.to_be_bytes()); // page count
    page1[56..60].copy_from_slice(&1u32.to_be_bytes()); // UTF-8

    let done_with_url = record(&[
        V::Text("t_aaaa0001".into()),
        V::Text("done card with claimed PR".into()),
        V::Null,
        V::Text("done".into()),
        V::Int(1_782_900_000),
        V::Text("work-t_aaaa0001".into()),
        V::Text(
            "Merged PR https://github.com/example/oyatie/pull/12. Packet: \
https://github.com/example/oyatie/pull/12#issuecomment-99."
                .into(),
        ),
    ]);
    let todo_card = record(&[
        V::Text("t_bbbb0002".into()),
        V::Text("todo card".into()),
        V::Null,
        V::Text("todo".into()),
        V::Null,
        V::Null,
        V::Null,
    ]);
    // Pre-migration short record: only 4 of 7 declared columns present.
    let short_done = record(&[
        V::Text("t_dddd0004".into()),
        V::Text("short pre-migration done card".into()),
        V::Null,
        V::Text("done".into()),
    ]);
    let mut overflow_pages = Vec::new();
    let cell1 = leaf_cell(&done_with_url, 1, 5, &mut overflow_pages);
    let cell2 = leaf_cell(&todo_card, 2, 5, &mut overflow_pages);
    let cell3 = leaf_cell(&short_done, 3, 5, &mut overflow_pages);
    assert!(overflow_pages.is_empty(), "page-3 records must stay inline");
    let page3 = table_leaf_page(&[cell1, cell2, cell3], 0);

    let big_done = record(&[
        V::Text("t_cccc0003".into()),
        V::Text("overflow done card without inline evidence".into()),
        V::Text("x".repeat(1200)),
        V::Text("done".into()),
        V::Int(1_782_910_000),
        V::Null,
        V::Null,
    ]);
    let cell4 = leaf_cell(&big_done, 4, 5, &mut overflow_pages);
    assert_eq!(overflow_pages.len(), 2, "big record must spill two pages");
    let page4 = table_leaf_page(&[cell4], 0);

    let page2 = table_interior_page(&[(3, 3)], 4);

    let mut db = Vec::with_capacity(PAGE_SIZE * 6);
    db.extend_from_slice(&page1);
    db.extend_from_slice(&page2);
    db.extend_from_slice(&page3);
    db.extend_from_slice(&page4);
    for page in overflow_pages {
        db.extend_from_slice(&page);
    }
    db
}

#[test]
fn sqlite_reader_walks_interior_pages_overflow_chains_and_short_records() {
    let db = SqliteDb::open(fixture_db()).expect("fixture must open");
    let (columns, rows) = db.read_table("tasks").expect("tasks must read");
    assert_eq!(
        columns,
        vec![
            "id",
            "title",
            "body",
            "status",
            "completed_at",
            "branch_name",
            "result"
        ]
    );
    assert_eq!(rows.len(), 4, "all four rows across both leaves decode");
    let big = rows
        .iter()
        .find(|r| r[0].as_str() == Some("t_cccc0003"))
        .expect("overflow row present");
    assert_eq!(
        big[2].as_str().map(str::len),
        Some(1200),
        "overflow chain reassembles the full 1200-byte body"
    );
    let short = rows
        .iter()
        .find(|r| r[0].as_str() == Some("t_dddd0004"))
        .expect("short row present");
    assert_eq!(short.len(), 7, "short record pads missing columns");
    assert!(short[4].as_int().is_none(), "padded completed_at is Null");
}

#[test]
fn board_extract_and_claims_hold_the_unverified_without_evidence_invariant() {
    let db = SqliteDb::open(fixture_db()).expect("fixture must open");
    let board = extract_board(&db).expect("board extracts");
    assert_eq!(board.status_counts.get("done"), Some(&3));
    assert_eq!(board.status_counts.get("todo"), Some(&1));
    assert_eq!(board.done_cards.len(), 3);

    // Evidence corpus: one artifact mentions the overflow card id. A
    // previously-generated ledger artifact mentions EVERY card id; it must be
    // excluded so regeneration cannot launder claims into evidence-attached.
    let corpus_root =
        std::env::temp_dir().join(format!("hermes-ledger-test-{}", std::process::id()));
    let evidence_dir = corpus_root.join("evidence/multispectrum");
    std::fs::create_dir_all(&evidence_dir).unwrap();
    std::fs::write(
        evidence_dir.join("packet.json"),
        r#"{"kanban_task":"t_cccc0003","verdict":"APPROVE"}"#,
    )
    .unwrap();
    let goals_dir = corpus_root.join("evidence/goals");
    std::fs::create_dir_all(&goals_dir).unwrap();
    std::fs::write(
        goals_dir.join("ledger.json"),
        r#"{"done_cards":["t_aaaa0001","t_cccc0003","t_dddd0004"]}"#,
    )
    .unwrap();

    let ids: BTreeSet<String> = board.done_cards.iter().map(|c| c.id.clone()).collect();
    let excluded: BTreeSet<String> = ["evidence/goals/ledger.json".to_owned()].into();
    let refs =
        scan_evidence_refs(&corpus_root, "evidence", &ids, &excluded).expect("scan succeeds");
    assert_eq!(
        refs.get("t_cccc0003")
            .map(|set| set.iter().cloned().collect::<Vec<_>>()),
        Some(vec!["evidence/multispectrum/packet.json".to_owned()])
    );
    assert!(
        !refs.contains_key("t_dddd0004"),
        "the excluded self-output must not count as evidence"
    );

    let built = build_claims(&board.done_cards, &refs);
    assert_eq!(built.claims.len(), 3);
    assert_eq!(built.evidence_attached_count, 2);
    assert_eq!(built.unverified_pending_count, 1);

    for claim in &built.claims {
        let Json::Obj(members) = claim else {
            panic!("claim must be an object")
        };
        let get = |key: &str| {
            members
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
        };
        assert_eq!(
            get("masterplan_status"),
            Some(Json::Str(CLAIMED_DONE_UNVERIFIED.into())),
            "every done-card claim stays unverified-pending-evidence"
        );
        let Some(Json::Arr(evidence_refs)) = get("evidence_refs") else {
            panic!("evidence_refs must be an array")
        };
        let Some(Json::Str(state)) = get("evidence_state") else {
            panic!("evidence_state must be a string")
        };
        // The Sub-AC verifiability clause: never a verified/attached state
        // without an attached evidence link, and never a bare unverified
        // state that silently hides existing evidence.
        if evidence_refs.is_empty() {
            assert_eq!(state, CLAIMED_DONE_UNVERIFIED);
        } else {
            assert_eq!(state, EVIDENCE_ATTACHED);
        }
    }

    // The card whose own result text carried PR URLs gets them attached.
    let Json::Obj(members) = &built.claims[0] else {
        panic!("claim must be an object")
    };
    assert_eq!(
        members.iter().find(|(k, _)| k == "source_card_id"),
        Some(&("source_card_id".into(), Json::Str("t_aaaa0001".into())))
    );
    let Some((_, Json::Arr(refs_a))) = members.iter().find(|(k, _)| k == "evidence_refs") else {
        panic!("evidence_refs must be an array")
    };
    assert_eq!(
        refs_a,
        &vec![
            Json::Str("https://github.com/example/oyatie/pull/12".into()),
            Json::Str("https://github.com/example/oyatie/pull/12#issuecomment-99".into()),
        ]
    );

    std::fs::remove_dir_all(&corpus_root).ok();
}

#[test]
fn create_table_parser_handles_the_real_board_ddl() {
    let ddl = "CREATE TABLE tasks (\n    id                   TEXT PRIMARY KEY,\n    title                TEXT NOT NULL,\n    -- Unified consecutive-failure counter. Incremented on spawn\n    consecutive_failures INTEGER NOT NULL DEFAULT 0,\n    status               TEXT NOT NULL,\n    priority             INTEGER DEFAULT 0\n)";
    let columns = parse_create_table_columns(ddl).expect("parses");
    assert_eq!(
        columns,
        vec!["id", "title", "consecutive_failures", "status", "priority"]
    );

    let with_constraint = "CREATE TABLE task_links (\n    parent_id  TEXT NOT NULL,\n    child_id   TEXT NOT NULL,\n    PRIMARY KEY (parent_id, child_id)\n)";
    assert_eq!(
        parse_create_table_columns(with_constraint).expect("parses"),
        vec!["parent_id", "child_id"]
    );
}

#[test]
fn github_url_extraction_trims_sentence_punctuation() {
    let urls = extract_github_urls(
        "Merged PR #1054 at https://github.com/j/oyatie/pull/1054, packet \
https://github.com/j/oyatie/pull/1054#issuecomment-4846059676. CI \
https://github.com/j/oyatie/actions/runs/28447071285/job/84304557825 green.",
    );
    let urls: Vec<&str> = urls.iter().map(String::as_str).collect();
    assert_eq!(
        urls,
        vec![
            "https://github.com/j/oyatie/actions/runs/28447071285/job/84304557825",
            "https://github.com/j/oyatie/pull/1054",
            "https://github.com/j/oyatie/pull/1054#issuecomment-4846059676",
        ]
    );
}

#[test]
fn iso_utc_matches_known_timestamps() {
    assert_eq!(iso_utc(0), "1970-01-01T00:00:00Z");
    assert_eq!(iso_utc(1_782_984_148), "2026-07-02T09:22:28Z");
}

const MINI_MASTERPLAN: &str = r#"{
  "masterplan_v2": {
    "evidence_state_policy": {
      "status_claims_require_evidence_refs": true
    },
    "hermes_done_card_imports": [
      {
        "source_system": "hermes",
        "source_count": 814,
        "evidence_refs": ["nested ] bracket \" in string"]
      }
    ],
    "authority_consolidation_audit": {}
  }
}
"#;

#[test]
fn masterplan_splice_is_bracket_aware_key_unique_and_idempotent() {
    let imports = Json::Arr(vec![Json::Obj(vec![
        ("source_system".into(), Json::str("hermes")),
        ("source_card_id".into(), Json::str("t_aaaa0001")),
        ("source_status".into(), Json::str("done")),
        ("completion_claim".into(), Json::str("hermes-done-card")),
        (
            "masterplan_status".into(),
            Json::str(CLAIMED_DONE_UNVERIFIED),
        ),
        ("evidence_state".into(), Json::str(CLAIMED_DONE_UNVERIFIED)),
        ("evidence_refs".into(), Json::Arr(Vec::new())),
    ])]);
    let summary = Json::Obj(vec![
        ("source_system".into(), Json::str("hermes")),
        ("extracted_done_count".into(), Json::Int(1)),
    ]);

    let pass1 = replace_key_value(MINI_MASTERPLAN, "hermes_done_card_imports", &imports)
        .expect("replace succeeds");
    let pass1 = insert_key_before(
        &pass1,
        "hermes_done_card_import_summary",
        &summary,
        "hermes_done_card_imports",
    )
    .expect("insert succeeds");
    assert!(pass1.contains("\"hermes_done_card_import_summary\": {"));
    assert!(pass1.contains("\"source_card_id\": \"t_aaaa0001\""));
    assert!(!pass1.contains("source_count"), "old value fully replaced");
    assert!(
        pass1.contains("\"authority_consolidation_audit\": {}"),
        "bytes outside the owned keys stay untouched"
    );

    // Idempotent regeneration: a second pass over already-spliced text
    // replaces (never duplicates) both owned keys.
    let pass2 = replace_key_value(&pass1, "hermes_done_card_imports", &imports)
        .expect("re-replace succeeds");
    let pass2 = insert_key_before(
        &pass2,
        "hermes_done_card_import_summary",
        &summary,
        "hermes_done_card_imports",
    )
    .expect("re-insert replaces in place");
    assert_eq!(pass1, pass2, "regeneration is byte-idempotent");

    // Fail-closed on ambiguity: a duplicated key refuses to splice.
    let duplicated = format!("{}{}", pass1, "{\"hermes_done_card_imports\": []}");
    assert!(replace_key_value(&duplicated, "hermes_done_card_imports", &imports).is_err());

    // The bracket walker honours strings containing brackets/quotes.
    let (start, end) =
        locate_key_value(MINI_MASTERPLAN, "hermes_done_card_imports").expect("locates");
    let span = &MINI_MASTERPLAN[start..end];
    assert!(span.starts_with('[') && span.ends_with(']'));
    assert!(span.contains("nested ] bracket"));
}

#[test]
fn pretty_emitter_is_deterministic_and_escapes() {
    let value = Json::Obj(vec![(
        "note".into(),
        Json::str("line1\nline2 \"quoted\" \\ tab\t"),
    )]);
    assert_eq!(
        to_pretty(&value, 2),
        "{\n    \"note\": \"line1\\nline2 \\\"quoted\\\" \\\\ tab\\t\"\n  }"
    );
}
