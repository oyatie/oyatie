//! Forensic done-card ledger construction for masterplan v2.
//!
//! Every Hermes done-card completion claim is imported as
//! `claimed-done-unverified` (never a verified status). Where evidence
//! references already exist — in-repo evidence artifacts that mention the card
//! id, or merged-PR/gate-run URLs the card's own result text recorded — they
//! are attached and the claim's `evidence_state` becomes `evidence-attached`
//! (attachment is a mechanical cross-reference, NOT verification). The
//! remainder stay flagged `claimed-done-unverified` with empty refs. The
//! cloud-ci cross-artifact-agreement gate (its `masterplan_evidence_state_invalid`
//! and `masterplan_plan_evidence_drift` findings) enforces that no claim can
//! carry a verified status without an attached evidence link.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::json::Json;
use crate::sqlite::{SqlValue, SqliteDb};

pub const CLAIMED_DONE_UNVERIFIED: &str = "claimed-done-unverified";
pub const EVIDENCE_ATTACHED: &str = "evidence-attached";

/// One done-column card extracted from the board snapshot.
#[derive(Debug, Clone)]
pub struct DoneCard {
    pub id: String,
    pub title: String,
    pub completed_at: Option<i64>,
    pub result: Option<String>,
    pub branch_name: Option<String>,
}

/// Full board extraction: per-status counts plus the done cards, id-sorted.
pub struct BoardExtract {
    pub status_counts: BTreeMap<String, usize>,
    pub done_cards: Vec<DoneCard>,
}

pub fn extract_board(db: &SqliteDb) -> Result<BoardExtract, String> {
    let (columns, rows) = db.read_table("tasks")?;
    let col = |name: &str| -> Result<usize, String> {
        columns
            .iter()
            .position(|c| c == name)
            .ok_or_else(|| format!("tasks table missing column {name}"))
    };
    let id_col = col("id")?;
    let title_col = col("title")?;
    let status_col = col("status")?;
    let completed_col = col("completed_at")?;
    let result_col = col("result")?;
    let branch_col = col("branch_name")?;

    let mut status_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut done_cards = Vec::new();
    for row in rows {
        let status = row
            .get(status_col)
            .and_then(SqlValue::as_str)
            .ok_or("task row with non-text status")?
            .to_owned();
        *status_counts.entry(status.clone()).or_insert(0) += 1;
        if status != "done" {
            continue;
        }
        let text_at = |idx: usize| row.get(idx).and_then(SqlValue::as_str).map(str::to_owned);
        done_cards.push(DoneCard {
            id: text_at(id_col).ok_or("done card with non-text id")?,
            title: text_at(title_col).unwrap_or_default(),
            completed_at: row.get(completed_col).and_then(SqlValue::as_int),
            result: text_at(result_col),
            branch_name: text_at(branch_col),
        });
    }
    done_cards.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(BoardExtract {
        status_counts,
        done_cards,
    })
}

/// Scan an evidence corpus directory for mentions of card ids of the shape
/// `t_` + 8 lowercase hex characters. Returns card id → sorted set of
/// repo-relative paths of the evidence files that reference it.
///
/// `excluded_rel` names repo-relative paths that must never count as evidence
/// — above all this tool's OWN outputs (the forensic ledger artifact mentions
/// every card id by construction; counting it would launder every claim into
/// evidence-attached on regeneration).
pub fn scan_evidence_refs(
    repo_root: &Path,
    corpus_rel: &str,
    ids: &BTreeSet<String>,
    excluded_rel: &BTreeSet<String>,
) -> Result<BTreeMap<String, BTreeSet<String>>, String> {
    let mut refs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let corpus_root = repo_root.join(corpus_rel);
    let mut stack = vec![corpus_root];
    while let Some(dir) = stack.pop() {
        let entries =
            std::fs::read_dir(&dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let mentioned = card_ids_in_bytes(&bytes, ids);
            if mentioned.is_empty() {
                continue;
            }
            let rel = path
                .strip_prefix(repo_root)
                .map_err(|_| "evidence file escapes repo root")?
                .to_string_lossy()
                .replace('\\', "/");
            if excluded_rel.contains(&rel) {
                continue;
            }
            for id in mentioned {
                refs.entry(id).or_default().insert(rel.clone());
            }
        }
    }
    Ok(refs)
}

fn is_hex_lower(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

/// Find every `t_xxxxxxxx` (8 lowercase hex) token in `bytes` that names a
/// known card id, with boundary checks so longer hex runs do not match.
fn card_ids_in_bytes(bytes: &[u8], ids: &BTreeSet<String>) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut i = 0;
    while i + 10 <= bytes.len() {
        if bytes[i] == b't' && bytes[i + 1] == b'_' {
            let hex = &bytes[i + 2..i + 10];
            let boundary_ok = bytes.get(i + 10).is_none_or(|b| !is_hex_lower(*b));
            if boundary_ok && hex.iter().all(|b| is_hex_lower(*b)) {
                if let Ok(token) = std::str::from_utf8(&bytes[i..i + 10])
                    && ids.contains(token)
                {
                    found.insert(token.to_owned());
                }
                i += 10;
                continue;
            }
        }
        i += 1;
    }
    found
}

/// Extract GitHub URLs (merged-PR, PR-comment, actions-run references) from a
/// card's result text.
pub fn extract_github_urls(text: &str) -> BTreeSet<String> {
    const PREFIX: &str = "https://github.com/";
    let mut urls = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut search_from = 0;
    while let Some(pos) = text[search_from..].find(PREFIX) {
        let start = search_from + pos;
        let mut end = start;
        while end < bytes.len() {
            let c = bytes[end] as char;
            let allowed = c.is_ascii_alphanumeric()
                || matches!(c, '/' | '#' | '-' | '_' | '.' | '?' | '=' | '&' | '%' | ':');
            if !allowed {
                break;
            }
            end += 1;
        }
        let mut url = &text[start..end];
        while let Some(stripped) = url.strip_suffix(['.', ',', ':']) {
            url = stripped;
        }
        if url.len() > PREFIX.len() {
            urls.insert(url.to_owned());
        }
        search_from = end.max(start + PREFIX.len());
    }
    urls
}

/// One masterplan claim entry per done card, id-sorted, in the exact shape the
/// cross-artifact-agreement gate audits.
pub struct BuiltClaims {
    pub claims: Vec<Json>,
    pub evidence_attached_count: usize,
    pub unverified_pending_count: usize,
}

pub fn build_claims(
    cards: &[DoneCard],
    corpus_refs: &BTreeMap<String, BTreeSet<String>>,
) -> BuiltClaims {
    let mut claims = Vec::with_capacity(cards.len());
    let mut evidence_attached_count = 0;
    let mut unverified_pending_count = 0;
    for card in cards {
        let mut refs: BTreeSet<String> = corpus_refs.get(&card.id).cloned().unwrap_or_default();
        if let Some(result) = &card.result {
            refs.extend(extract_github_urls(result));
        }
        let evidence_state = if refs.is_empty() {
            unverified_pending_count += 1;
            CLAIMED_DONE_UNVERIFIED
        } else {
            evidence_attached_count += 1;
            EVIDENCE_ATTACHED
        };
        claims.push(Json::Obj(vec![
            ("source_system".into(), Json::str("hermes")),
            ("source_card_id".into(), Json::str(card.id.clone())),
            ("source_status".into(), Json::str("done")),
            ("completion_claim".into(), Json::str("hermes-done-card")),
            (
                "masterplan_status".into(),
                Json::str(CLAIMED_DONE_UNVERIFIED),
            ),
            ("evidence_state".into(), Json::str(evidence_state)),
            (
                "evidence_refs".into(),
                Json::Arr(refs.into_iter().map(Json::Str).collect()),
            ),
        ]));
    }
    BuiltClaims {
        claims,
        evidence_attached_count,
        unverified_pending_count,
    }
}

/// Convert unix seconds to an ISO-8601 UTC timestamp (owned civil-date math).
pub fn iso_utc(secs: i64) -> String {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// Locate the value span of a top-level occurrence of `"key":` in JSON text.
/// Returns (value_start, value_end_exclusive). The occurrence count must be
/// exactly one, so the splice can never rewrite an unintended region.
pub fn locate_key_value(text: &str, key: &str) -> Result<(usize, usize), String> {
    let needle = format!("\"{key}\"");
    let mut matches = text.match_indices(&needle);
    let Some((key_pos, _)) = matches.next() else {
        return Err(format!("key {key} not found"));
    };
    if matches.next().is_some() {
        return Err(format!(
            "key {key} occurs more than once; refusing to splice"
        ));
    }
    let after_key = key_pos + needle.len();
    let bytes = text.as_bytes();
    let mut i = after_key;
    while i < bytes.len() && (bytes[i] as char).is_whitespace() {
        i += 1;
    }
    if bytes.get(i) != Some(&b':') {
        return Err(format!("key {key} is not followed by a colon"));
    }
    i += 1;
    while i < bytes.len() && (bytes[i] as char).is_whitespace() {
        i += 1;
    }
    let open = *bytes.get(i).ok_or("unexpected end after key")?;
    let close = match open {
        b'[' => b']',
        b'{' => b'}',
        _ => return Err(format!("key {key} value is not an array/object")),
    };
    let value_start = i;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
        } else if b == b'"' {
            in_string = true;
        } else if b == open {
            depth += 1;
        } else if b == close {
            depth -= 1;
            if depth == 0 {
                return Ok((value_start, i + 1));
            }
        }
        i += 1;
    }
    Err(format!("unterminated value for key {key}"))
}

fn key_line_indent(text: &str, key: &str) -> Result<usize, String> {
    let needle = format!("\"{key}\"");
    let key_pos = text
        .find(&needle)
        .ok_or_else(|| format!("key {key} not found"))?;
    let line_start = text[..key_pos].rfind('\n').map_or(0, |p| p + 1);
    Ok(text[line_start..key_pos]
        .chars()
        .take_while(|c| *c == ' ')
        .count())
}

/// Replace the value of `"key"` in `text` with `value`, preserving every byte
/// outside the value span. Fails closed unless the key occurs exactly once.
pub fn replace_key_value(text: &str, key: &str, value: &Json) -> Result<String, String> {
    let (start, end) = locate_key_value(text, key)?;
    let indent = key_line_indent(text, key)?;
    let rendered = crate::json::to_pretty(value, indent);
    let mut out = String::with_capacity(text.len() + rendered.len());
    out.push_str(&text[..start]);
    out.push_str(&rendered);
    out.push_str(&text[end..]);
    Ok(out)
}

/// Insert `"key": value,` on its own line directly above `before_key`'s line.
pub fn insert_key_before(
    text: &str,
    key: &str,
    value: &Json,
    before_key: &str,
) -> Result<String, String> {
    if text.contains(&format!("\"{key}\"")) {
        return replace_key_value(text, key, value);
    }
    let needle = format!("\"{before_key}\"");
    let mut matches = text.match_indices(&needle);
    let Some((anchor_pos, _)) = matches.next() else {
        return Err(format!("anchor key {before_key} not found"));
    };
    if matches.next().is_some() {
        return Err(format!(
            "anchor key {before_key} occurs more than once; refusing to splice"
        ));
    }
    let line_start = text[..anchor_pos].rfind('\n').map_or(0, |p| p + 1);
    let indent = key_line_indent(text, before_key)?;
    let rendered = crate::json::to_pretty(value, indent);
    let mut out = String::with_capacity(text.len() + rendered.len());
    out.push_str(&text[..line_start]);
    out.push_str(&" ".repeat(indent));
    out.push('"');
    out.push_str(key);
    out.push_str("\": ");
    out.push_str(&rendered);
    out.push_str(",\n");
    out.push_str(&text[line_start..]);
    Ok(out)
}
