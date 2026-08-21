//! Black-box tests for the cursor codec ((b)) and pagination ((c)).
//!
//! `QueryRow` (from `audit-query-api`) derives `Clone, Debug` but not
//! `PartialEq`, so page equality below is asserted on the extracted
//! `audit_id` sequence — the actually-computed values, not a shape or count
//! (per L5).

use audit_query_domain::{
    AuditQuery, CursorScope, QueryDomainError, QueryRow, ResultSealState, decode_cursor,
    encode_cursor, paginate, validate_query,
};

fn row(audit_id: &str, period_id: &str) -> QueryRow {
    QueryRow {
        audit_id: audit_id.to_string(),
        period_id: period_id.to_string(),
        seal_state: ResultSealState::Sealed,
    }
}

fn seven_row_set() -> Vec<QueryRow> {
    (0..7)
        .map(|i| row(&format!("audit-{i}"), "period-2026-01"))
        .collect()
}

fn audit_ids(rows: &[QueryRow]) -> Vec<String> {
    rows.iter().map(|r| r.audit_id.clone()).collect()
}

// A default scope: no filters, the default page size. Most tests below are
// exercising the tenant/offset/framing legs of the codec and hold the scope
// fixed, so they use this constant rather than repeating it everywhere.
const NO_FILTERS: CursorScope<'_> = CursorScope {
    pack: None,
    period: None,
    event_type: None,
    principal: None,
    entity: None,
    limit: 3,
};

// ── cursor codec: roundtrip ─────────────────────────────────────────────

#[test]
fn cursor_roundtrips_offset_for_same_tenant_and_scope() {
    let cursor = encode_cursor("tenant-alpha", 42, NO_FILTERS);
    assert_eq!(decode_cursor(&cursor, "tenant-alpha", NO_FILTERS), Ok(42));
}

#[test]
fn cursor_roundtrips_zero_offset() {
    let cursor = encode_cursor("tenant-alpha", 0, NO_FILTERS);
    assert_eq!(decode_cursor(&cursor, "tenant-alpha", NO_FILTERS), Ok(0));
}

// ── cursor codec: the security-relevant cross-tenant case ──────────────

#[test]
fn cursor_minted_for_one_tenant_is_rejected_for_another() {
    let cursor_for_tenant_a = encode_cursor("tenant-a", 5, NO_FILTERS);
    assert_eq!(
        decode_cursor(&cursor_for_tenant_a, "tenant-b", NO_FILTERS),
        Err(QueryDomainError::InvalidCursor)
    );
    // The rightful tenant still decodes it fine — the codec itself is sound,
    // only the cross-tenant presentation is refused.
    assert_eq!(
        decode_cursor(&cursor_for_tenant_a, "tenant-a", NO_FILTERS),
        Ok(5)
    );
}

#[test]
fn validate_query_rejects_cross_tenant_cursor() {
    let cursor_for_tenant_a = encode_cursor("tenant-a", 3, NO_FILTERS);
    let query = AuditQuery {
        tenant_id: "tenant-b".to_string(),
        cursor: Some(cursor_for_tenant_a),
        ..Default::default()
    };
    assert_eq!(validate_query(&query), Err(QueryDomainError::InvalidCursor));
}

// ── cursor codec: the scope-relevant case (finding D / #2 / #10) ────────

#[test]
fn cursor_minted_under_one_pack_is_rejected_for_a_different_pack() {
    let scope_eu = CursorScope {
        pack: Some("pack-eu"),
        ..NO_FILTERS
    };
    let scope_us = CursorScope {
        pack: Some("pack-us"),
        ..NO_FILTERS
    };
    let cursor = encode_cursor("tenant-alpha", 5, scope_eu);
    // Same tenant, same offset, only `pack` differs: still refused.
    assert_eq!(
        decode_cursor(&cursor, "tenant-alpha", scope_us),
        Err(QueryDomainError::InvalidCursor)
    );
    // The scope it was actually minted under still decodes fine.
    assert_eq!(decode_cursor(&cursor, "tenant-alpha", scope_eu), Ok(5));
}

#[test]
fn cursor_minted_under_one_limit_is_rejected_for_a_different_limit() {
    let scope_3 = CursorScope {
        limit: 3,
        ..NO_FILTERS
    };
    let scope_2 = CursorScope {
        limit: 2,
        ..NO_FILTERS
    };
    let cursor = encode_cursor("tenant-alpha", 5, scope_3);
    assert_eq!(
        decode_cursor(&cursor, "tenant-alpha", scope_2),
        Err(QueryDomainError::InvalidCursor)
    );
}

#[test]
fn cursor_scope_distinguishes_none_from_present_but_empty_looking_fields() {
    // Regression for the length-prefix / presence-tag domain separation in
    // `compute_scope_fingerprint`: concatenation-adjacent fields must not
    // collide, and `None` must not collide with a field holding "".
    let scope_a = CursorScope {
        event_type: Some("ab"),
        principal: Some("c"),
        ..NO_FILTERS
    };
    let scope_b = CursorScope {
        event_type: Some("a"),
        principal: Some("bc"),
        ..NO_FILTERS
    };
    let cursor = encode_cursor("tenant-alpha", 5, scope_a);
    assert_eq!(
        decode_cursor(&cursor, "tenant-alpha", scope_b),
        Err(QueryDomainError::InvalidCursor)
    );
    assert_eq!(decode_cursor(&cursor, "tenant-alpha", scope_a), Ok(5));
}

#[test]
fn a_cursor_replayed_across_a_changed_pack_does_not_silently_skip_rows() {
    // End-to-end reproduction of the finding: page one query (`pack-eu`) to
    // get a `next_cursor`, then replay it verbatim against a DIFFERENT query
    // (`pack-us`, a different `event_type`, a different `limit`) for the
    // same tenant. Before the fix this was accepted and silently dropped
    // rows; now it must be refused as a stale/foreign cursor.
    let eu_rows: Vec<QueryRow> = (0..5).map(|i| row(&format!("eu-{i}"), "p")).collect();
    let us_rows: Vec<QueryRow> = (0..5).map(|i| row(&format!("us-{i}"), "p")).collect();

    let eu_query = AuditQuery {
        tenant_id: "tenant-alpha".to_string(),
        pack: Some("pack-eu".to_string()),
        limit: Some(3),
        ..Default::default()
    };
    let eu_validated = validate_query(&eu_query).unwrap();
    let eu_page = paginate(&eu_rows, &eu_validated).unwrap();
    assert_eq!(audit_ids(&eu_page.rows), vec!["eu-0", "eu-1", "eu-2"]);
    let eu_next_cursor = eu_page.next_cursor.expect("more eu rows remain");

    let us_query = AuditQuery {
        tenant_id: "tenant-alpha".to_string(),
        pack: Some("pack-us".to_string()),
        event_type: Some("something-else".to_string()),
        cursor: Some(eu_next_cursor),
        limit: Some(2),
        ..Default::default()
    };
    assert_eq!(
        validate_query(&us_query),
        Err(QueryDomainError::InvalidCursor)
    );
    // us-0..us-1 were never at risk of being silently skipped, because
    // validate_query refused to build a `ValidatedAuditQuery` at all.
    let _ = us_rows;
}

// ── cursor codec: non-canonical offset field (finding #9) ───────────────

#[test]
fn decode_cursor_rejects_a_non_canonical_offset_field() {
    // `encode_cursor` never emits a sign-prefixed or zero-padded offset
    // (`usize::to_string()` never produces one), so `"+7"` / `"0007"` are
    // not cursors this crate minted, even though both parse to the same
    // usize as the canonical `"7"` field this crate does mint. Build a
    // well-formed 4-field cursor frame by hand with each non-canonical
    // offset field and confirm it is rejected.
    let canonical = encode_cursor("tenant-alpha", 7, NO_FILTERS);
    let hand_built = hand_build_cursor("tenant-alpha", "+7", NO_FILTERS);
    assert_eq!(
        decode_cursor(&hand_built, "tenant-alpha", NO_FILTERS),
        Err(QueryDomainError::InvalidCursor)
    );
    let hand_built_padded = hand_build_cursor("tenant-alpha", "0007", NO_FILTERS);
    assert_eq!(
        decode_cursor(&hand_built_padded, "tenant-alpha", NO_FILTERS),
        Err(QueryDomainError::InvalidCursor)
    );
    // Sanity: the canonical field this same hand-builder produces for "7"
    // matches the crate's own encoder byte-for-byte, so the hand-builder is
    // faithfully reproducing the wire format rather than testing itself.
    assert_eq!(
        hand_build_cursor("tenant-alpha", "7", NO_FILTERS),
        canonical
    );
}

/// Re-implements `encode_cursor_raw` + `compute_scope_fingerprint` byte for
/// byte (length-prefixed fields, FNV-1a-64 checksum, lowercase hex) so tests
/// can substitute an arbitrary offset field string, including non-canonical
/// ones the real encoder would never produce. `scope` covers only the
/// no-filters/default-limit shape used by `NO_FILTERS`-based tests above.
fn hand_build_cursor(tenant_id: &str, offset_field: &str, scope: CursorScope<'_>) -> String {
    fn encode_field(buf: &mut Vec<u8>, value: &[u8]) {
        buf.extend_from_slice(&(value.len() as u64).to_be_bytes());
        buf.extend_from_slice(value);
    }
    fn fnv1a64(bytes: &[u8]) -> u64 {
        const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
        const PRIME: u64 = 0x0000_0100_0000_01b3;
        let mut hash = OFFSET_BASIS;
        for &byte in bytes {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(PRIME);
        }
        hash
    }
    fn encode_optional(buf: &mut Vec<u8>, value: Option<&str>) {
        match value {
            Some(v) => {
                buf.push(1);
                encode_field(buf, v.as_bytes());
            }
            None => buf.push(0),
        }
    }
    fn scope_fingerprint(scope: &CursorScope<'_>) -> u64 {
        let mut buf = Vec::new();
        encode_optional(&mut buf, scope.pack);
        encode_optional(&mut buf, scope.period);
        encode_optional(&mut buf, scope.event_type);
        encode_optional(&mut buf, scope.principal);
        encode_optional(&mut buf, scope.entity);
        encode_field(&mut buf, &scope.limit.to_be_bytes());
        fnv1a64(&buf)
    }

    let mut payload = Vec::new();
    encode_field(&mut payload, b"v2");
    encode_field(&mut payload, tenant_id.as_bytes());
    encode_field(&mut payload, offset_field.as_bytes());
    encode_field(&mut payload, &scope_fingerprint(&scope).to_be_bytes());
    let checksum = fnv1a64(&payload);
    let mut full = payload;
    full.extend_from_slice(&checksum.to_be_bytes());
    full.iter().map(|b| format!("{b:02x}")).collect()
}

// ── cursor codec: malformed ──────────────────────────────────────────────

#[test]
fn decode_cursor_rejects_garbage_string() {
    assert_eq!(
        decode_cursor("not-a-cursor!", "tenant-alpha", NO_FILTERS),
        Err(QueryDomainError::InvalidCursor)
    );
}

#[test]
fn decode_cursor_rejects_empty_string() {
    // Zero bytes decoded from empty hex: shorter than the mandatory 8-byte
    // checksum suffix.
    assert_eq!(
        decode_cursor("", "tenant-alpha", NO_FILTERS),
        Err(QueryDomainError::InvalidCursor)
    );
}

#[test]
fn decode_cursor_rejects_valid_hex_that_is_not_a_cursor() {
    // Well-formed hex, wrong length/shape for our framing.
    assert_eq!(
        decode_cursor("deadbeef", "tenant-alpha", NO_FILTERS),
        Err(QueryDomainError::InvalidCursor)
    );
}

// ── cursor codec: truncated ──────────────────────────────────────────────

#[test]
fn decode_cursor_rejects_truncated_cursor() {
    let cursor = encode_cursor("tenant-alpha", 100, NO_FILTERS);
    let truncated = &cursor[..cursor.len() - 4];
    assert_eq!(
        decode_cursor(truncated, "tenant-alpha", NO_FILTERS),
        Err(QueryDomainError::InvalidCursor)
    );
}

#[test]
fn decode_cursor_rejects_cursor_with_flipped_checksum_byte() {
    // A single corrupted hex character anywhere must fail the checksum.
    let cursor = encode_cursor("tenant-alpha", 7, NO_FILTERS);
    let mut chars: Vec<char> = cursor.chars().collect();
    let last = chars.len() - 1;
    chars[last] = if chars[last] == '0' { '1' } else { '0' };
    let corrupted: String = chars.into_iter().collect();
    assert_eq!(
        decode_cursor(&corrupted, "tenant-alpha", NO_FILTERS),
        Err(QueryDomainError::InvalidCursor)
    );
}

// ── pagination: end-to-end multi-page walk (L5) ──────────────────────────

#[test]
fn multi_page_walk_reconstructs_exact_row_set_no_drop_no_duplicate() {
    let rows = seven_row_set();
    let tenant_id = "tenant-alpha";
    let mut collected = Vec::new();
    let mut cursor: Option<String> = None;
    let mut pages_seen = 0;

    loop {
        let query = AuditQuery {
            tenant_id: tenant_id.to_string(),
            cursor: cursor.clone(),
            limit: Some(3),
            ..Default::default()
        };
        let validated = validate_query(&query).expect("each page's query must validate");
        let page = paginate(&rows, &validated).expect("page must build");
        collected.extend(page.rows.iter().map(|r| r.audit_id.clone()));
        pages_seen += 1;
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
        assert!(pages_seen <= 10, "pagination did not terminate");
    }

    assert_eq!(pages_seen, 3, "7 rows at limit 3 must take exactly 3 pages");
    assert_eq!(
        collected,
        audit_ids(&rows),
        "concatenated pages must equal the input exactly, in order"
    );
}

#[test]
fn first_page_has_correct_rows_and_next_cursor() {
    let rows = seven_row_set();
    let query = AuditQuery {
        tenant_id: "tenant-alpha".to_string(),
        limit: Some(3),
        ..Default::default()
    };
    let validated = validate_query(&query).unwrap();
    let page = paginate(&rows, &validated).unwrap();
    assert_eq!(audit_ids(&page.rows), vec!["audit-0", "audit-1", "audit-2"]);
    let next = page.next_cursor.expect("more rows remain");
    let scope = CursorScope {
        limit: 3,
        ..NO_FILTERS
    };
    assert_eq!(decode_cursor(&next, "tenant-alpha", scope), Ok(3));
}

#[test]
fn final_page_has_no_next_cursor() {
    let rows = seven_row_set();
    let scope = CursorScope {
        limit: 3,
        ..NO_FILTERS
    };
    let query = AuditQuery {
        tenant_id: "tenant-alpha".to_string(),
        cursor: Some(encode_cursor("tenant-alpha", 6, scope)),
        limit: Some(3),
        ..Default::default()
    };
    let validated = validate_query(&query).unwrap();
    let page = paginate(&rows, &validated).unwrap();
    assert_eq!(audit_ids(&page.rows), vec!["audit-6"]);
    assert_eq!(page.next_cursor, None);
}

#[test]
fn cursor_pointing_past_end_of_row_set_is_rejected() {
    let rows = seven_row_set();
    let scope = CursorScope {
        limit: 3,
        ..NO_FILTERS
    };
    let query = AuditQuery {
        tenant_id: "tenant-alpha".to_string(),
        cursor: Some(encode_cursor("tenant-alpha", 999, scope)),
        limit: Some(3),
        ..Default::default()
    };
    let validated = validate_query(&query).unwrap();
    assert_eq!(
        paginate(&rows, &validated).unwrap_err(),
        QueryDomainError::InvalidCursor
    );
}

#[test]
fn empty_row_set_yields_empty_page_and_no_next_cursor() {
    let rows: Vec<QueryRow> = Vec::new();
    let query = AuditQuery {
        tenant_id: "tenant-alpha".to_string(),
        limit: Some(3),
        ..Default::default()
    };
    let validated = validate_query(&query).unwrap();
    let page = paginate(&rows, &validated).unwrap();
    assert!(page.rows.is_empty());
    assert_eq!(page.next_cursor, None);
}

// ── pagination: exact-boundary stale cursor (finding #3 / #11) ──────────

#[test]
fn cursor_at_exact_boundary_of_a_shrunk_row_set_is_rejected_not_silently_empty() {
    // Page a 10-row set at limit 3 to obtain a cursor for offset 3, then
    // re-present that cursor against a row set that has since shrunk to
    // exactly 3 rows. Before the fix this returned Ok(empty page, no
    // cursor); it must now be InvalidCursor, matching the doc's promise
    // that a stale position is never confused with "no more rows".
    let ten_rows: Vec<QueryRow> = (0..10).map(|i| row(&format!("audit-{i}"), "p")).collect();
    let query = AuditQuery {
        tenant_id: "tenant-alpha".to_string(),
        limit: Some(3),
        ..Default::default()
    };
    let validated = validate_query(&query).unwrap();
    let page = paginate(&ten_rows, &validated).unwrap();
    let cursor = page.next_cursor.expect("more rows remain");

    let shrunk_rows: Vec<QueryRow> = (0..3).map(|i| row(&format!("audit-{i}"), "p")).collect();
    let follow_up = AuditQuery {
        tenant_id: "tenant-alpha".to_string(),
        cursor: Some(cursor),
        limit: Some(3),
        ..Default::default()
    };
    let follow_up_validated = validate_query(&follow_up).unwrap();
    assert_eq!(
        paginate(&shrunk_rows, &follow_up_validated).unwrap_err(),
        QueryDomainError::InvalidCursor
    );
}

#[test]
fn cursor_against_a_row_set_shrunk_to_completely_empty_is_rejected_not_a_panic() {
    // Regression for a blocking finding: page a 10-row set at limit 3 to
    // obtain a cursor for offset 3 (same tenant, same scope), then
    // re-present that SAME crate-minted cursor against a row set that has
    // since shrunk all the way to zero rows (e.g. a retention cascade
    // redacted everything in the period). Before the fix, `paginate`'s
    // guard was `offset >= rows.len() && !rows.is_empty()`, whose
    // `!rows.is_empty()` carve-out let every offset through once `rows` was
    // empty, so this fell through to `rows[3..0]` and panicked with "range
    // start index 3 out of range for slice of length 0" instead of
    // returning `Err(InvalidCursor)` as the doc promises and as the
    // non-empty exact-boundary case above already does.
    let ten_rows: Vec<QueryRow> = (0..10).map(|i| row(&format!("audit-{i}"), "p")).collect();
    let query = AuditQuery {
        tenant_id: "tenant-alpha".to_string(),
        limit: Some(3),
        ..Default::default()
    };
    let validated = validate_query(&query).unwrap();
    let page = paginate(&ten_rows, &validated).unwrap();
    let cursor = page.next_cursor.expect("more rows remain");

    let emptied_rows: Vec<QueryRow> = Vec::new();
    let follow_up = AuditQuery {
        tenant_id: "tenant-alpha".to_string(),
        cursor: Some(cursor),
        limit: Some(3),
        ..Default::default()
    };
    let follow_up_validated = validate_query(&follow_up).unwrap();
    assert_eq!(
        paginate(&emptied_rows, &follow_up_validated).unwrap_err(),
        QueryDomainError::InvalidCursor
    );
}

#[test]
fn paginate_rejects_smallest_possible_stale_offset_against_an_empty_row_set() {
    // Minimal reproduction at offset == 1 (rather than 3), to pin the
    // boundary condition itself rather than a specific page size. Two rows
    // at limit 1 so the first page leaves one row unseen and mints a
    // `next_cursor` at offset 1.
    let two_rows: Vec<QueryRow> = vec![row("audit-0", "p"), row("audit-1", "p")];
    let query = AuditQuery {
        tenant_id: "tenant-alpha".to_string(),
        limit: Some(1),
        ..Default::default()
    };
    let validated = validate_query(&query).unwrap();
    let page = paginate(&two_rows, &validated).unwrap();
    let cursor = page.next_cursor.expect("offset 1 next_cursor expected");

    let empty: Vec<QueryRow> = Vec::new();
    let follow_up = AuditQuery {
        tenant_id: "tenant-alpha".to_string(),
        cursor: Some(cursor),
        limit: Some(1),
        ..Default::default()
    };
    let follow_up_validated = validate_query(&follow_up).unwrap();
    assert_eq!(
        paginate(&empty, &follow_up_validated).unwrap_err(),
        QueryDomainError::InvalidCursor
    );
}
