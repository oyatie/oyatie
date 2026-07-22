//! Pure validator for the controller-materialized fixed historical P2 receipt.
#![cfg_attr(test, allow(clippy::expect_used, clippy::panic, clippy::unwrap_used))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

const PATH: &str = "ci/facade/artifact-inventory-registry/adr-census-parent-receipt.generated.json";
const SCHEMA: &str = "oya-ci/adr-census-parent-receipt/v1";
const CORPUS_COMMIT: &str = "1fa09da22be819b062881eb59252f4dd4c6b550a";
const REPOSITORY_TREE: &str = "d7b15539396db21b219d68779362850cce9afa8f";
const DOCS_TREE: &str = "fbf3f8d4b9ecf30b2272f37871e8152a616eed5a";
const DECISIONS_TREE: &str = "7c7c371697d2a7009e3d43b16235518d00ac33ea";
const PARSER_COMMIT: &str = "a2b326eebd418ae970847b5e1bca3782c61c52ab";
const PARSER_TREE: &str = "0cdece525bc54f83ec51d3ba67a4308d0ce43812";
const PARSER_SOURCE: &str = "governance/corpus/doc-parser/src/lib.rs:ab3884dbf4a657869fd87920b016cc4734a1c27f:e559419fdb11452f5d30312ce3baca6f22bd9a08b98f0e880bfe344c3420d62e";
const SELECTOR: &str = "docs-decisions-direct-adr-v1";

// Updated only when the fixed-object projection contract intentionally changes. These values
// make the receipt immutable: recomputing self-consistent hashes after changing an entry cannot
// make a different historical receipt pass.
const FIXED_OUTER_SHA256: &str = "c3c4195f440fbf7825101dcf303fea9d8aec9d2ce7a77bd3ec25d8411dfdf528";
const FIXED_CANONICAL_DIGEST: &str =
    "7a8eb3848e3b5d1dd148595b5210f2a059fac582db9e5607cf54be2f502b24d8";
const FIXED_AGGREGATE_FOLD: &str =
    "2aeb7459f61b6f216b4eee75164bcfb85e405bbe8ca74cf180e5492b09c99507";

#[derive(Clone, Copy)]
struct ExpectedDigests<'a> {
    outer: &'a str,
    canonical: &'a str,
    aggregate: &'a str,
}

const FIXED_DIGESTS: ExpectedDigests<'static> = ExpectedDigests {
    outer: FIXED_OUTER_SHA256,
    canonical: FIXED_CANONICAL_DIGEST,
    aggregate: FIXED_AGGREGATE_FOLD,
};

fn main() {
    if let Err(error) = validate_path(&repo_root_from_current_dir().join(PATH)) {
        eprintln!("adr-census-parent-receipt-gate: {error}");
        std::process::exit(1);
    }
}

fn repo_root_from_current_dir() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|error| {
        panic!("adr-census-parent-receipt-gate: resolve current directory: {error}")
    });
    for _ in 0..16 {
        if dir.join("specs/root-hub-pointers.json").is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("adr-census-parent-receipt-gate: repository root not found")
}

fn validate_path(path: &Path) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|error| format!("read fixed receipt: {error}"))?;
    validate_bytes(&bytes)
}

fn validate_bytes(bytes: &[u8]) -> Result<(), String> {
    validate_bytes_with_expected(bytes, FIXED_DIGESTS)
}

fn validate_bytes_with_expected(bytes: &[u8], expected: ExpectedDigests<'_>) -> Result<(), String> {
    let raw = std::str::from_utf8(bytes).map_err(|_| "receipt is not UTF-8")?;
    let prefix = "{\"outer_sha256\":\"";
    let rest = raw
        .strip_prefix(prefix)
        .ok_or("receipt is not in canonical outer-field order")?;
    let (outer, receipt_and_tail) = rest
        .split_once("\",\"receipt\":")
        .ok_or("receipt outer digest field malformed")?;
    require_lower_hex(outer, 64, "receipt outer digest")?;
    let tail = format!(",\"schema\":{}}}\n", json_string(SCHEMA));
    let receipt = receipt_and_tail
        .strip_suffix(&tail)
        .ok_or("receipt has newline, outer field-order, or schema drift")?;
    let actual_outer = sha256_hex(receipt.as_bytes());
    if actual_outer != outer {
        return Err("receipt raw whole-byte SHA256 mismatch".to_owned());
    }
    if outer != expected.outer {
        return Err("receipt differs from the fixed historical whole-byte digest".to_owned());
    }

    let value: Value =
        serde_json::from_str(receipt).map_err(|error| format!("receipt JSON invalid: {error}"))?;
    let object = value.as_object().ok_or("receipt must be an object")?;
    require_exact_keys(
        object,
        &[
            "aggregate_fold",
            "canonical_digest",
            "claim_ceiling",
            "decisions_tree",
            "diagnostic_policy",
            "docs_tree",
            "entries",
            "first_error_kinds",
            "parser_api",
            "parser_commit",
            "parser_parent_commit",
            "parser_source_hashes",
            "parser_tree",
            "parser_version",
            "repository_commit",
            "repository_tree",
            "selector",
            "totals",
        ],
        "receipt",
    )?;
    require_string(object, "claim_ceiling", "BLOCKED/HOLD")?;
    require_string(object, "decisions_tree", DECISIONS_TREE)?;
    require_string(object, "diagnostic_policy", "first-error-only")?;
    require_string(object, "docs_tree", DOCS_TREE)?;
    require_string(
        object,
        "parser_api",
        "corpus-doc-parser::parse_adr_decision",
    )?;
    require_string(object, "parser_commit", PARSER_COMMIT)?;
    require_string(object, "parser_parent_commit", CORPUS_COMMIT)?;
    require_string(object, "parser_tree", PARSER_TREE)?;
    require_string(object, "parser_version", "corpus-doc-parser-v1")?;
    require_string(object, "repository_commit", CORPUS_COMMIT)?;
    require_string(object, "repository_tree", REPOSITORY_TREE)?;
    require_string(object, "selector", SELECTOR)?;
    require_string(object, "aggregate_fold", expected.aggregate)?;
    require_string(object, "canonical_digest", expected.canonical)?;

    let parser_sources = object
        .get("parser_source_hashes")
        .and_then(Value::as_array)
        .ok_or("receipt parser sources absent")?;
    if parser_sources.len() != 1 || parser_sources[0].as_str() != Some(PARSER_SOURCE) {
        return Err("receipt fixed parser path, blob, or raw digest mismatch".to_owned());
    }

    let expected_errors = BTreeMap::from([
        ("InvalidAdrReference", 28_u64),
        ("InvalidFrontmatter", 4),
        ("MissingLeadingFrontmatter", 26),
        ("MissingRequiredField", 142),
        ("UnsupportedNesting", 45),
    ]);
    let errors = object
        .get("first_error_kinds")
        .and_then(Value::as_object)
        .ok_or("receipt error totals absent")?;
    require_exact_keys(
        errors,
        &expected_errors.keys().copied().collect::<Vec<_>>(),
        "first_error_kinds",
    )?;
    for (kind, expected_count) in &expected_errors {
        if errors.get(*kind).and_then(Value::as_u64) != Some(*expected_count) {
            return Err(format!("receipt error total differs for {kind}"));
        }
    }

    let totals = object
        .get("totals")
        .and_then(Value::as_object)
        .ok_or("receipt totals absent")?;
    require_exact_keys(totals, &["parsed", "rejected"], "totals")?;
    if totals.get("parsed").and_then(Value::as_u64) != Some(184)
        || totals.get("rejected").and_then(Value::as_u64) != Some(245)
    {
        return Err("receipt parsed/rejected totals differ".to_owned());
    }

    let entries = object
        .get("entries")
        .and_then(Value::as_array)
        .ok_or("receipt entries absent")?;
    validate_entries(entries, &expected_errors, expected.aggregate)?;

    let canonical_body = render_canonical_body(object)?;
    let actual_canonical = sha256_hex(canonical_body.as_bytes());
    if actual_canonical != expected.canonical {
        return Err("receipt embedded canonical digest mismatch".to_owned());
    }
    let canonical_receipt = format!(
        "{{{canonical_body},\"canonical_digest\":{}}}",
        json_string(expected.canonical)
    );
    if receipt != canonical_receipt {
        return Err(
            "receipt has whitespace, order, duplicate-key, or canonical-byte drift".to_owned(),
        );
    }
    Ok(())
}

fn validate_entries(
    entries: &[Value],
    expected_errors: &BTreeMap<&str, u64>,
    expected_aggregate: &str,
) -> Result<(), String> {
    if entries.len() != 429 {
        return Err("receipt direct ADR selection count mismatch".to_owned());
    }
    let mut previous_path: Option<&str> = None;
    let mut parsed = 0_u64;
    let mut rejected = 0_u64;
    let mut observed_errors = BTreeMap::<&str, u64>::new();
    let mut aggregate = Sha256::new();
    aggregate.update(b"oyatie:census:entry-fold:v1\\0");
    for entry in entries {
        let entry = entry.as_object().ok_or("receipt entry must be an object")?;
        require_exact_keys(
            entry,
            &["blob_oid", "first_error", "outcome", "path", "sha256"],
            "entry",
        )?;
        let path = entry
            .get("path")
            .and_then(Value::as_str)
            .ok_or("entry path must be a string")?;
        if !is_direct_adr_path(path) {
            return Err(format!(
                "entry path is outside the direct ADR selector: {path}"
            ));
        }
        if previous_path.is_some_and(|previous| previous.as_bytes() >= path.as_bytes()) {
            return Err("entry paths are duplicate or not byte-sorted".to_owned());
        }
        previous_path = Some(path);
        require_lower_hex(
            entry
                .get("blob_oid")
                .and_then(Value::as_str)
                .ok_or("entry blob_oid must be a string")?,
            40,
            "entry blob_oid",
        )?;
        require_lower_hex(
            entry
                .get("sha256")
                .and_then(Value::as_str)
                .ok_or("entry sha256 must be a string")?,
            64,
            "entry sha256",
        )?;
        match entry.get("outcome").and_then(Value::as_str) {
            Some("parsed") => {
                parsed += 1;
                if !entry.get("first_error").is_some_and(Value::is_null) {
                    return Err("parsed entry must have null first_error".to_owned());
                }
            }
            Some("rejected") => {
                rejected += 1;
                let error = entry
                    .get("first_error")
                    .and_then(Value::as_object)
                    .ok_or("rejected entry must carry first_error")?;
                require_exact_keys(error, &["kind", "raw", "span"], "first_error")?;
                let kind = error
                    .get("kind")
                    .and_then(Value::as_str)
                    .ok_or("first_error.kind must be a string")?;
                if !expected_errors.contains_key(kind) {
                    return Err(format!(
                        "first_error.kind is outside the closed set: {kind}"
                    ));
                }
                *observed_errors.entry(kind).or_default() += 1;
                if error
                    .get("raw")
                    .and_then(Value::as_str)
                    .is_none_or(str::is_empty)
                {
                    return Err("first_error.raw must be a non-empty string".to_owned());
                }
                validate_span(error.get("span").ok_or("first_error.span absent")?)?;
            }
            _ => return Err("entry outcome must be parsed or rejected".to_owned()),
        }
        let canonical_entry = render_canonical_entry(entry)?;
        aggregate.update((canonical_entry.len() as u64).to_be_bytes());
        aggregate.update(canonical_entry.as_bytes());
    }
    if parsed != 184 || rejected != 245 || observed_errors != *expected_errors {
        return Err("entry outcomes do not reproduce the closed totals".to_owned());
    }
    if format!("{:x}", aggregate.finalize()) != expected_aggregate {
        return Err("receipt aggregate_fold does not reproduce exact canonical entries".to_owned());
    }
    Ok(())
}

fn render_canonical_entry(entry: &Map<String, Value>) -> Result<String, String> {
    let field = |key: &str| {
        entry
            .get(key)
            .ok_or_else(|| format!("entry field absent: {key}"))
            .and_then(|value| {
                serde_json::to_string(value)
                    .map_err(|error| format!("serialize entry field {key}: {error}"))
            })
    };
    Ok(format!(
        "{{\"blob_oid\":{},\"first_error\":{},\"outcome\":{},\"path\":{},\"sha256\":{}}}",
        field("blob_oid")?,
        field("first_error")?,
        field("outcome")?,
        field("path")?,
        field("sha256")?,
    ))
}

fn validate_span(span: &Value) -> Result<(), String> {
    if span.is_null() {
        return Ok(());
    }
    let values = span
        .as_array()
        .ok_or("first_error.span must be null or [start,end]")?;
    if values.len() != 2 {
        return Err("first_error.span must contain exactly two offsets".to_owned());
    }
    let start = values[0]
        .as_u64()
        .ok_or("first_error.span start must be u64")?;
    let end = values[1]
        .as_u64()
        .ok_or("first_error.span end must be u64")?;
    if start > end {
        return Err("first_error.span start exceeds end".to_owned());
    }
    Ok(())
}

fn render_canonical_body(object: &Map<String, Value>) -> Result<String, String> {
    let field = |key: &str| {
        object
            .get(key)
            .ok_or_else(|| format!("receipt field absent: {key}"))
            .and_then(|value| {
                serde_json::to_string(value)
                    .map_err(|error| format!("serialize receipt field {key}: {error}"))
            })
    };
    Ok(format!(
        "\"aggregate_fold\":{},\"claim_ceiling\":{},\"decisions_tree\":{},\"diagnostic_policy\":{},\"docs_tree\":{},\"entries\":{},\"first_error_kinds\":{},\"parser_api\":{},\"parser_commit\":{},\"parser_parent_commit\":{},\"parser_source_hashes\":{},\"parser_tree\":{},\"parser_version\":{},\"repository_commit\":{},\"repository_tree\":{},\"selector\":{},\"totals\":{}",
        field("aggregate_fold")?,
        field("claim_ceiling")?,
        field("decisions_tree")?,
        field("diagnostic_policy")?,
        field("docs_tree")?,
        field("entries")?,
        field("first_error_kinds")?,
        field("parser_api")?,
        field("parser_commit")?,
        field("parser_parent_commit")?,
        field("parser_source_hashes")?,
        field("parser_tree")?,
        field("parser_version")?,
        field("repository_commit")?,
        field("repository_tree")?,
        field("selector")?,
        field("totals")?,
    ))
}

fn require_string(object: &Map<String, Value>, key: &str, expected: &str) -> Result<(), String> {
    if object.get(key).and_then(Value::as_str) != Some(expected) {
        return Err(format!("receipt fixed field differs: {key}"));
    }
    Ok(())
}

fn require_exact_keys(
    object: &Map<String, Value>,
    expected: &[&str],
    context: &str,
) -> Result<(), String> {
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(format!("{context} key set is not exact"));
    }
    Ok(())
}

fn require_lower_hex(value: &str, length: usize, context: &str) -> Result<(), String> {
    if value.len() != length
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!(
            "{context} must be {length} lowercase hex characters"
        ));
    }
    Ok(())
}

fn is_direct_adr_path(path: &str) -> bool {
    path.starts_with("docs/decisions/ADR-")
        && path.ends_with(".md")
        && !path["docs/decisions/".len()..].contains('/')
        && !path.contains("..")
        && path.is_ascii()
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("serializing a string cannot fail")
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_entry(index: usize, kind: Option<&str>) -> Value {
        let path = format!("docs/decisions/ADR-{index:04}-fixture.md");
        let first_error = kind.map_or(Value::Null, |kind| {
            serde_json::json!({
                "kind": kind,
                "raw": format!("fixture diagnostic {kind}"),
                "span": [index as u64, index as u64 + 1]
            })
        });
        serde_json::json!({
            "blob_oid": format!("{:040x}", index + 1),
            "first_error": first_error,
            "outcome": if kind.is_some() { "rejected" } else { "parsed" },
            "path": path,
            "sha256": sha256_hex(format!("fixture-{index}").as_bytes())
        })
    }

    fn valid_fixture() -> (Vec<u8>, ExpectedDigests<'static>) {
        let mut entries = Vec::new();
        for index in 0..184 {
            entries.push(fixture_entry(index, None));
        }
        let error_groups = [
            ("InvalidAdrReference", 28),
            ("InvalidFrontmatter", 4),
            ("MissingLeadingFrontmatter", 26),
            ("MissingRequiredField", 142),
            ("UnsupportedNesting", 45),
        ];
        let mut index = entries.len();
        for (kind, count) in error_groups {
            for _ in 0..count {
                entries.push(fixture_entry(index, Some(kind)));
                index += 1;
            }
        }
        let mut aggregate = Sha256::new();
        aggregate.update(b"oyatie:census:entry-fold:v1\\0");
        for entry in &entries {
            let entry = serde_json::to_string(entry).unwrap();
            aggregate.update((entry.len() as u64).to_be_bytes());
            aggregate.update(entry.as_bytes());
        }
        let aggregate = format!("{:x}", aggregate.finalize());
        let mut object = serde_json::json!({
            "aggregate_fold": aggregate,
            "claim_ceiling": "BLOCKED/HOLD",
            "decisions_tree": DECISIONS_TREE,
            "diagnostic_policy": "first-error-only",
            "docs_tree": DOCS_TREE,
            "entries": entries,
            "first_error_kinds": {
                "InvalidAdrReference": 28,
                "InvalidFrontmatter": 4,
                "MissingLeadingFrontmatter": 26,
                "MissingRequiredField": 142,
                "UnsupportedNesting": 45
            },
            "parser_api": "corpus-doc-parser::parse_adr_decision",
            "parser_commit": PARSER_COMMIT,
            "parser_parent_commit": CORPUS_COMMIT,
            "parser_source_hashes": [PARSER_SOURCE],
            "parser_tree": PARSER_TREE,
            "parser_version": "corpus-doc-parser-v1",
            "repository_commit": CORPUS_COMMIT,
            "repository_tree": REPOSITORY_TREE,
            "selector": SELECTOR,
            "totals": {"parsed": 184, "rejected": 245}
        });
        let body = render_canonical_body(object.as_object().unwrap()).unwrap();
        let canonical = sha256_hex(body.as_bytes());
        object.as_object_mut().unwrap().insert(
            "canonical_digest".to_owned(),
            Value::String(canonical.clone()),
        );
        let receipt = format!(
            "{{{body},\"canonical_digest\":{}}}",
            json_string(&canonical)
        );
        let outer = sha256_hex(receipt.as_bytes());
        let bytes = format!(
            "{{\"outer_sha256\":{},\"receipt\":{receipt},\"schema\":{}}}\n",
            json_string(&outer),
            json_string(SCHEMA)
        )
        .into_bytes();
        let expected = ExpectedDigests {
            outer: Box::leak(outer.into_boxed_str()),
            canonical: Box::leak(canonical.into_boxed_str()),
            aggregate: Box::leak(aggregate.into_boxed_str()),
        };
        (bytes, expected)
    }

    fn replace_and_rehash(bytes: &[u8], from: &str, to: &str) -> Vec<u8> {
        let raw = std::str::from_utf8(bytes).unwrap();
        let prefix = "{\"outer_sha256\":\"";
        let rest = raw.strip_prefix(prefix).unwrap();
        let (_, receipt_and_tail) = rest.split_once("\",\"receipt\":").unwrap();
        let tail = format!(",\"schema\":{}}}\n", json_string(SCHEMA));
        let receipt = receipt_and_tail.strip_suffix(&tail).unwrap();
        let mut value: Value = serde_json::from_str(receipt).unwrap();
        let object = value.as_object_mut().unwrap();
        let current = object.get(from).cloned().unwrap();
        object.remove(from);
        object.insert(to.to_owned(), current);
        let body = render_canonical_body(object).unwrap_or_default();
        let canonical = sha256_hex(body.as_bytes());
        object.insert(
            "canonical_digest".to_owned(),
            Value::String(canonical.clone()),
        );
        let receipt = format!(
            "{{{body},\"canonical_digest\":{}}}",
            json_string(&canonical)
        );
        let outer = sha256_hex(receipt.as_bytes());
        format!(
            "{{\"outer_sha256\":{},\"receipt\":{receipt},\"schema\":{}}}\n",
            json_string(&outer),
            json_string(SCHEMA)
        )
        .into_bytes()
    }

    #[test]
    fn deterministic_fixture_is_valid_without_generated_file_coupling() {
        let (bytes, expected) = valid_fixture();
        validate_bytes_with_expected(&bytes, expected)
            .expect("deterministic fixture must validate");
    }

    #[test]
    fn live_gate_validates_the_downloaded_controller_face() {
        validate_path(&repo_root_from_current_dir().join(PATH))
            .expect("downloaded/materialized fixed historical receipt must validate");
    }

    #[test]
    fn rejects_missing_receipt() {
        assert!(validate_path(Path::new("definitely-missing-receipt.json")).is_err());
    }

    #[test]
    fn rejects_newline_order_duplicate_and_whole_byte_mutations() {
        let (bytes, expected) = valid_fixture();
        let mut no_newline = bytes.clone();
        no_newline.pop();
        assert!(validate_bytes_with_expected(&no_newline, expected).is_err());

        let reordered = String::from_utf8(bytes.clone()).unwrap().replace(
            "{\"outer_sha256\":",
            "{\"schema\":\"wrong\",\"outer_sha256\":",
        );
        assert!(validate_bytes_with_expected(reordered.as_bytes(), expected).is_err());

        let duplicated = String::from_utf8(bytes.clone()).unwrap().replace(
            "\"claim_ceiling\":",
            "\"claim_ceiling\":\"BLOCKED/HOLD\",\"claim_ceiling\":",
        );
        assert!(validate_bytes_with_expected(duplicated.as_bytes(), expected).is_err());

        let mut byte_mutated = bytes;
        let index = byte_mutated.iter().position(|byte| *byte == b'B').unwrap();
        byte_mutated[index] = b'C';
        assert!(validate_bytes_with_expected(&byte_mutated, expected).is_err());
    }

    #[test]
    fn rejects_rehashed_extra_missing_or_renamed_fields() {
        let (bytes, expected) = valid_fixture();
        let renamed = replace_and_rehash(&bytes, "decisions_tree", "decisions_tree_extra");
        assert!(validate_bytes_with_expected(&renamed, expected).is_err());
    }

    #[test]
    fn rejects_entry_shape_and_closed_error_set_mutations() {
        let (bytes, expected) = valid_fixture();
        let raw = String::from_utf8(bytes).unwrap();
        let missing_entry_field = raw.replacen("\"blob_oid\":", "\"unexpected_oid\":", 1);
        assert!(validate_bytes_with_expected(missing_entry_field.as_bytes(), expected).is_err());
        let extra_error = raw.replace(
            "\"UnsupportedNesting\":45}",
            "\"UnsupportedNesting\":45,\"Unknown\":0}",
        );
        assert!(validate_bytes_with_expected(extra_error.as_bytes(), expected).is_err());
    }

    #[test]
    fn rejects_rehashed_entry_mutation_when_aggregate_is_stale() {
        let (bytes, expected) = valid_fixture();
        let mut wrapper: Value = serde_json::from_slice(&bytes).unwrap();
        let receipt = wrapper
            .as_object_mut()
            .unwrap()
            .get_mut("receipt")
            .unwrap()
            .as_object_mut()
            .unwrap();
        receipt.get_mut("entries").unwrap().as_array_mut().unwrap()[0]
            .as_object_mut()
            .unwrap()
            .insert("sha256".to_owned(), Value::String("f".repeat(64)));
        let body = render_canonical_body(receipt).unwrap();
        let canonical = sha256_hex(body.as_bytes());
        let canonical_receipt = format!(
            "{{{body},\"canonical_digest\":{}}}",
            json_string(&canonical)
        );
        let outer = sha256_hex(canonical_receipt.as_bytes());
        let mutated = format!(
            "{{\"outer_sha256\":{},\"receipt\":{canonical_receipt},\"schema\":{}}}\n",
            json_string(&outer),
            json_string(SCHEMA)
        );
        let mutated_expected = ExpectedDigests {
            outer: Box::leak(outer.into_boxed_str()),
            canonical: Box::leak(canonical.into_boxed_str()),
            aggregate: expected.aggregate,
        };
        assert_eq!(
            validate_bytes_with_expected(mutated.as_bytes(), mutated_expected).unwrap_err(),
            "receipt aggregate_fold does not reproduce exact canonical entries"
        );
    }
}
