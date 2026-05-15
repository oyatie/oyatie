//! `oya-json-kernel` — std-only minimal JSON value-kind classifier + top-
//! level-object parser. Reusable across workspace.
//!
//! Scope: identify the kind of each top-level value in a JSON object. Skips
//! nested object/array contents correctly. Does NOT parse numbers as f64,
//! does NOT decode escape sequences in strings (just walks past them safely).
//!
//! Originated as inline code in `oya-check-dependency-seam` resolving CONV-1
//! of TG2 11-facet debate synthesis (6-lens convergence: substring-grep JSON
//! inspection is approximate-when-exact-feasible + trivially-bypassed +
//! copy-introduced + n-source-of-truth). User directive 2026-05-15 promoted
//! to its own kernel crate: "shouldn't we make a reusable json parser?"
//!
//! Future consumers:
//!   - oya-check-dependency-seam (current; substring-grep replacement)
//!   - A-family adherence sub-checks (A6_schema_adherence, etc.)
//!   - any tooling needing JSON inspection without pulling serde_json

use std::collections::BTreeMap;

/// Classifier for top-level JSON value kinds. No number parsing (strings
/// stay as raw bytes); no escape decoding (parser walks past escapes
/// safely). Std-only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsonValueKind {
    BoolTrue,
    BoolFalse,
    Null,
    Number,
    String,
    Object,
    Array,
}

/// Parse a JSON document and return its top-level keys mapped to value-kind
/// classifications. Skips nested object/array contents correctly. Returns
/// an empty map if `raw` is not a valid top-level JSON object or parsing
/// fails partway through.
///
/// Safe-by-design:
///   - whitespace-tolerant (matches `"key": value`, `"key":value`,
///     `"key" :    value`, multi-line with newlines/tabs)
///   - bypass-safe: literal occurrences of `"key"` inside nested string
///     values DO NOT register as top-level keys
///   - case-sensitive (per JSON spec)
///
/// Replaces brittle `raw.contains("\"key\"")` substring grep flagged by
/// 6 facet lenses in TG2 11-facet debate synthesis CONV-1.
pub fn parse_top_level_object(raw: &str) -> BTreeMap<String, JsonValueKind> {
    let mut out: BTreeMap<String, JsonValueKind> = BTreeMap::new();
    let bytes = raw.as_bytes();
    let mut i = 0usize;
    // Skip leading whitespace.
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'{' {
        return out;
    }
    i += 1;
    loop {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            return out;
        }
        if bytes[i] == b'}' {
            return out;
        }
        if bytes[i] == b',' {
            i += 1;
            continue;
        }
        if bytes[i] != b'"' {
            return out;
        }
        let key_start = i + 1;
        i += 1;
        while i < bytes.len() && bytes[i] != b'"' {
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            i += 1;
        }
        if i >= bytes.len() {
            return out;
        }
        let key = String::from_utf8_lossy(&bytes[key_start..i]).to_string();
        i += 1; // past close-quote of key
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b':' {
            return out;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            return out;
        }
        let kind = match bytes[i] {
            b't' if i + 4 <= bytes.len() && &bytes[i..i + 4] == b"true" => {
                i += 4;
                JsonValueKind::BoolTrue
            }
            b't' => return out,
            b'f' if i + 5 <= bytes.len() && &bytes[i..i + 5] == b"false" => {
                i += 5;
                JsonValueKind::BoolFalse
            }
            b'f' => return out,
            b'n' if i + 4 <= bytes.len() && &bytes[i..i + 4] == b"null" => {
                i += 4;
                JsonValueKind::Null
            }
            b'n' => return out,
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 2;
                        continue;
                    }
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
                JsonValueKind::String
            }
            b'{' => {
                let mut depth = 1usize;
                i += 1;
                while i < bytes.len() && depth > 0 {
                    match bytes[i] {
                        b'"' => {
                            i += 1;
                            while i < bytes.len() && bytes[i] != b'"' {
                                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                                    i += 2;
                                    continue;
                                }
                                i += 1;
                            }
                            if i < bytes.len() {
                                i += 1;
                            }
                        }
                        b'{' => {
                            depth += 1;
                            i += 1;
                        }
                        b'}' => {
                            depth -= 1;
                            i += 1;
                        }
                        _ => i += 1,
                    }
                }
                JsonValueKind::Object
            }
            b'[' => {
                let mut depth = 1usize;
                i += 1;
                while i < bytes.len() && depth > 0 {
                    match bytes[i] {
                        b'"' => {
                            i += 1;
                            while i < bytes.len() && bytes[i] != b'"' {
                                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                                    i += 2;
                                    continue;
                                }
                                i += 1;
                            }
                            if i < bytes.len() {
                                i += 1;
                            }
                        }
                        b'[' => {
                            depth += 1;
                            i += 1;
                        }
                        b']' => {
                            depth -= 1;
                            i += 1;
                        }
                        _ => i += 1,
                    }
                }
                JsonValueKind::Array
            }
            b'-' | b'0'..=b'9' => {
                while i < bytes.len()
                    && !matches!(bytes[i], b',' | b'}' | b' ' | b'\t' | b'\n' | b'\r')
                {
                    i += 1;
                }
                JsonValueKind::Number
            }
            _ => return out,
        };
        out.insert(key, kind);
    }
}

/// Return the byte slice of the value associated with `top_level_key` in
/// `raw`, INCLUDING the outer delimiter (e.g., `{...}` for objects,
/// `[...]` for arrays, `"..."` for strings, or the raw token for
/// numbers/booleans/null). Returns `None` if `raw` is not a top-level
/// object, the key is not found at depth 1, or parsing fails.
///
/// Std-only. Skips nested object/array/string contents correctly. Used by
/// `extract_object_keys` and downstream build-time codegen consumers.
pub fn extract_top_level_value_slice<'a>(raw: &'a str, top_level_key: &str) -> Option<&'a str> {
    let bytes = raw.as_bytes();
    let mut i = 0usize;
    // Outer { open
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'{' {
        return None;
    }
    i += 1;
    // Now we're at depth=1 inside the outer object. Walk keys.
    loop {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }
        if bytes[i] == b'}' {
            return None;
        }
        if bytes[i] == b',' {
            i += 1;
            continue;
        }
        if bytes[i] != b'"' {
            return None;
        }
        let key_start = i + 1;
        i += 1;
        while i < bytes.len() && bytes[i] != b'"' {
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }
        let this_key = &bytes[key_start..i];
        let key_matches = this_key == top_level_key.as_bytes();
        i += 1; // past close-quote of key
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b':' {
            return None;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }
        let value_start = i;
        // Advance past the value, tracking depth for objects/arrays + string boundaries.
        match bytes[i] {
            b't' => i += 4,
            b'f' => i += 5,
            b'n' => i += 4,
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 2;
                        continue;
                    }
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
            }
            b'{' => {
                let mut depth = 1usize;
                i += 1;
                while i < bytes.len() && depth > 0 {
                    match bytes[i] {
                        b'"' => {
                            i += 1;
                            while i < bytes.len() && bytes[i] != b'"' {
                                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                                    i += 2;
                                    continue;
                                }
                                i += 1;
                            }
                            if i < bytes.len() {
                                i += 1;
                            }
                        }
                        b'{' => {
                            depth += 1;
                            i += 1;
                        }
                        b'}' => {
                            depth -= 1;
                            i += 1;
                        }
                        _ => i += 1,
                    }
                }
            }
            b'[' => {
                let mut depth = 1usize;
                i += 1;
                while i < bytes.len() && depth > 0 {
                    match bytes[i] {
                        b'"' => {
                            i += 1;
                            while i < bytes.len() && bytes[i] != b'"' {
                                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                                    i += 2;
                                    continue;
                                }
                                i += 1;
                            }
                            if i < bytes.len() {
                                i += 1;
                            }
                        }
                        b'[' => {
                            depth += 1;
                            i += 1;
                        }
                        b']' => {
                            depth -= 1;
                            i += 1;
                        }
                        _ => i += 1,
                    }
                }
            }
            b'-' | b'0'..=b'9' => {
                while i < bytes.len()
                    && !matches!(bytes[i], b',' | b'}' | b' ' | b'\t' | b'\n' | b'\r')
                {
                    i += 1;
                }
            }
            _ => return None,
        }
        let value_end = i;
        if key_matches {
            return std::str::from_utf8(&bytes[value_start..value_end]).ok();
        }
    }
}

/// Return the top-level keys of the JSON object stored at `parent_key`
/// inside `raw`. Returns an empty Vec if the parent value is not an object
/// or the key is absent. Std-only; composed from
/// `extract_top_level_value_slice` + `parse_top_level_object`.
pub fn extract_object_keys(raw: &str, parent_key: &str) -> Vec<String> {
    match extract_top_level_value_slice(raw, parent_key) {
        Some(slice) => parse_top_level_object(slice).into_keys().collect(),
        None => Vec::new(),
    }
}

/// Return the string elements of an array stored at `parent_key`. Numbers,
/// nested objects/arrays, booleans, null are skipped (not included in the
/// returned Vec). Useful when the JSON declares an array of identifier
/// strings (e.g., enum value lists). Std-only.
pub fn extract_string_array(raw: &str, parent_key: &str) -> Vec<String> {
    let slice = match extract_top_level_value_slice(raw, parent_key) {
        Some(s) => s,
        None => return Vec::new(),
    };
    let bytes = slice.as_bytes();
    let mut i = 0usize;
    let mut out: Vec<String> = Vec::new();
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() || bytes[i] != b'[' {
        return out;
    }
    i += 1;
    loop {
        while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b',') {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b']' {
            return out;
        }
        if bytes[i] != b'"' {
            // skip non-string element (number/object/array/bool/null)
            let mut depth_obj = 0usize;
            let mut depth_arr = 0usize;
            while i < bytes.len() {
                match bytes[i] {
                    b',' | b']' if depth_obj == 0 && depth_arr == 0 => break,
                    b'{' => depth_obj += 1,
                    b'}' => depth_obj = depth_obj.saturating_sub(1),
                    b'[' => depth_arr += 1,
                    b']' if depth_arr > 0 => depth_arr -= 1,
                    b']' => break,
                    b'"' => {
                        // skip nested string
                        i += 1;
                        while i < bytes.len() && bytes[i] != b'"' {
                            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                                i += 2;
                                continue;
                            }
                            i += 1;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
            continue;
        }
        // string element
        let s_start = i + 1;
        i += 1;
        while i < bytes.len() && bytes[i] != b'"' {
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            i += 1;
        }
        if i >= bytes.len() {
            return out;
        }
        out.push(String::from_utf8_lossy(&bytes[s_start..i]).to_string());
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_all_kinds() {
        let raw = r#"{"a": true, "b": false, "c": null, "d": 42, "e": "str", "f": {}, "g": [1,2]}"#;
        let m = parse_top_level_object(raw);
        assert_eq!(m.get("a"), Some(&JsonValueKind::BoolTrue));
        assert_eq!(m.get("b"), Some(&JsonValueKind::BoolFalse));
        assert_eq!(m.get("c"), Some(&JsonValueKind::Null));
        assert_eq!(m.get("d"), Some(&JsonValueKind::Number));
        assert_eq!(m.get("e"), Some(&JsonValueKind::String));
        assert_eq!(m.get("f"), Some(&JsonValueKind::Object));
        assert_eq!(m.get("g"), Some(&JsonValueKind::Array));
    }

    #[test]
    fn whitespace_tolerant() {
        for raw in [
            r#"{"meta_review_triggered" :  true}"#,
            r#"{"meta_review_triggered":true}"#,
            "{\n  \"meta_review_triggered\":\n    true\n}",
            "{ \t\"meta_review_triggered\" \t: \t true \t}",
        ] {
            let m = parse_top_level_object(raw);
            assert_eq!(
                m.get("meta_review_triggered"),
                Some(&JsonValueKind::BoolTrue),
                "raw: {:?}",
                raw
            );
        }
    }

    #[test]
    fn rejects_substring_bypass() {
        let raw = r#"{
          "note": "we discussed meta_review_triggered: true in round 1",
          "change_id": "CC-X"
        }"#;
        let m = parse_top_level_object(raw);
        assert_eq!(m.get("meta_review_triggered"), None);
        assert_eq!(m.get("note"), Some(&JsonValueKind::String));
        assert_eq!(m.get("change_id"), Some(&JsonValueKind::String));
    }

    #[test]
    fn skips_nested_object_contents() {
        let raw = r#"{
          "facets": {"F1_linus": {"considered": true}, "F2": {"considered": false}},
          "change_class_id": "CC-7"
        }"#;
        let m = parse_top_level_object(raw);
        assert_eq!(m.get("facets"), Some(&JsonValueKind::Object));
        assert_eq!(m.get("change_class_id"), Some(&JsonValueKind::String));
        assert_eq!(m.get("F1_linus"), None);
        assert_eq!(m.get("considered"), None);
    }

    #[test]
    fn returns_empty_on_malformed() {
        assert!(parse_top_level_object("not json").is_empty());
        assert!(parse_top_level_object("").is_empty());
        assert!(parse_top_level_object("[1, 2, 3]").is_empty());
    }

    #[test]
    fn handles_escaped_quote_in_string() {
        let raw = r#"{"key": "value with \"quote\" inside", "other": 1}"#;
        let m = parse_top_level_object(raw);
        assert_eq!(m.get("key"), Some(&JsonValueKind::String));
        assert_eq!(m.get("other"), Some(&JsonValueKind::Number));
    }

    #[test]
    fn handles_negative_number() {
        let raw = r#"{"n": -42, "m": 0.5}"#;
        let m = parse_top_level_object(raw);
        assert_eq!(m.get("n"), Some(&JsonValueKind::Number));
        assert_eq!(m.get("m"), Some(&JsonValueKind::Number));
    }

    #[test]
    fn extract_value_slice_object() {
        let raw = r#"{"facets":{"F1":{},"F2":{}},"other":42}"#;
        let v = extract_top_level_value_slice(raw, "facets").unwrap();
        assert_eq!(v, r#"{"F1":{},"F2":{}}"#);
    }

    #[test]
    fn extract_value_slice_missing_returns_none() {
        let raw = r#"{"a":1}"#;
        assert_eq!(extract_top_level_value_slice(raw, "missing"), None);
    }

    #[test]
    fn extract_object_keys_returns_nested_keys() {
        let raw = r#"{"facets":{"F1_linus":{"id":"F1"},"F2_hyperscaler":{"id":"F2"}}}"#;
        let mut keys = extract_object_keys(raw, "facets");
        keys.sort();
        assert_eq!(
            keys,
            vec!["F1_linus".to_string(), "F2_hyperscaler".to_string()]
        );
    }

    #[test]
    fn extract_object_keys_missing_returns_empty() {
        let raw = r#"{"a":1}"#;
        assert!(extract_object_keys(raw, "missing").is_empty());
    }

    #[test]
    fn extract_string_array_returns_strings_only() {
        let raw = r#"{"items":["a","b","c"]}"#;
        let v = extract_string_array(raw, "items");
        assert_eq!(v, vec!["a", "b", "c"]);
    }

    #[test]
    fn extract_string_array_filters_non_string_elements() {
        let raw = r#"{"mixed":["a",1,"b",{"x":1},"c"]}"#;
        let v = extract_string_array(raw, "mixed");
        assert_eq!(v, vec!["a", "b", "c"]);
    }
}
