//! Metal config URL variable expansion.
//!
//! Mirrors the pure URL-shaping part of Talos'
//! `internal/app/machined/pkg/runtime/v1alpha1/platform/metal/url`: metal config
//! download URLs may contain query variables such as `${uuid}` or legacy
//! `?uuid=`. Upstream waits for the corresponding runtime resources, replaces
//! the variables, then serializes the query with Go's `url.Values.Encode`
//! semantics (sorted keys, repeated values preserved, query escaping).
//!
//! This module keeps the same substitution rules as a no-std, data-only helper:
//! callers provide the already-discovered values, and the URL is expanded
//! deterministically without networking or state watches.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

/// Supported Talos metal URL variable keys.
pub const UUID_KEY: &str = "uuid";
/// Supported Talos metal URL variable keys.
pub const SERIAL_NUMBER_KEY: &str = "serial";
/// Supported Talos metal URL variable keys.
pub const MAC_KEY: &str = "mac";
/// Supported Talos metal URL variable keys.
pub const HOSTNAME_KEY: &str = "hostname";
/// Supported Talos metal URL variable keys.
pub const CODE_KEY: &str = "code";

const VARIABLES: [VariableSpec; 5] = [
    VariableSpec {
        key: UUID_KEY,
        match_on_arg: true,
    },
    VariableSpec {
        key: SERIAL_NUMBER_KEY,
        match_on_arg: false,
    },
    VariableSpec {
        key: MAC_KEY,
        match_on_arg: false,
    },
    VariableSpec {
        key: HOSTNAME_KEY,
        match_on_arg: false,
    },
    VariableSpec {
        key: CODE_KEY,
        match_on_arg: false,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VariableSpec {
    key: &'static str,
    match_on_arg: bool,
}

/// Concrete values for Talos metal URL variables.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UrlVariableValues {
    uuid: Option<String>,
    serial: Option<String>,
    mac: Option<String>,
    hostname: Option<String>,
    code: Option<String>,
}

impl UrlVariableValues {
    /// Empty variable map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the `uuid` variable.
    pub fn with_uuid(mut self, value: impl Into<String>) -> Self {
        self.uuid = Some(value.into());
        self
    }

    /// Set the `serial` variable.
    pub fn with_serial(mut self, value: impl Into<String>) -> Self {
        self.serial = Some(value.into());
        self
    }

    /// Set the `mac` variable.
    pub fn with_mac(mut self, value: impl Into<String>) -> Self {
        self.mac = Some(value.into());
        self
    }

    /// Set the `hostname` variable.
    pub fn with_hostname(mut self, value: impl Into<String>) -> Self {
        self.hostname = Some(value.into());
        self
    }

    /// Set the `code` variable.
    pub fn with_code(mut self, value: impl Into<String>) -> Self {
        self.code = Some(value.into());
        self
    }

    fn get(&self, key: &str) -> Option<&str> {
        match key {
            UUID_KEY => self.uuid.as_deref(),
            SERIAL_NUMBER_KEY => self.serial.as_deref(),
            MAC_KEY => self.mac.as_deref(),
            HOSTNAME_KEY => self.hostname.as_deref(),
            CODE_KEY => self.code.as_deref(),
            _ => None,
        }
    }
}

/// URL expansion failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlPopulateError {
    /// A URL references a known Talos variable for which the caller supplied no
    /// value. Upstream would wait for the runtime resource; this pure helper
    /// surfaces the missing value explicitly.
    MissingVariable(&'static str),
}

impl fmt::Display for UrlPopulateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlPopulateError::MissingVariable(key) => {
                write!(f, "missing metal URL variable: {key}")
            }
        }
    }
}

/// Return the Talos variable keys required by `download_url`, in canonical
/// upstream order (`uuid`, `serial`, `mac`, `hostname`, `code`).
pub fn required_variables(download_url: &str) -> Vec<&'static str> {
    let Some(query) = split_url(download_url).1 else {
        return Vec::new();
    };
    let params = parse_query(query);
    VARIABLES
        .iter()
        .filter(|spec| variable_matches(spec, &params))
        .map(|spec| spec.key)
        .collect()
}

/// Populate known Talos metal variables in `download_url`.
pub fn populate_url(
    download_url: &str,
    values: &UrlVariableValues,
) -> Result<String, UrlPopulateError> {
    let (base, Some(query), fragment) = split_url(download_url) else {
        return Ok(download_url.to_string());
    };

    let mut params = parse_query(query);
    let required = required_variables(download_url);
    if required.is_empty() {
        return Ok(download_url.to_string());
    }

    for key in &required {
        if values.get(key).is_none() {
            return Err(UrlPopulateError::MissingVariable(key));
        }
    }

    for spec in &VARIABLES {
        if !required.contains(&spec.key) {
            continue;
        }
        replace_variable(spec, values.get(spec.key).unwrap_or_default(), &mut params);
    }

    let encoded = encode_query(&params);
    if encoded.is_empty() {
        Ok([base, fragment].concat())
    } else {
        Ok([base, "?", &encoded, fragment].concat())
    }
}

fn split_url(input: &str) -> (&str, Option<&str>, &str) {
    let hash = input.find('#').unwrap_or(input.len());
    let before_fragment = &input[..hash];
    let fragment = if hash < input.len() {
        &input[hash..]
    } else {
        ""
    };

    match before_fragment.find('?') {
        Some(q) => (&input[..q], Some(&input[q + 1..hash]), fragment),
        None => (before_fragment, None, fragment),
    }
}

fn parse_query(query: &str) -> BTreeMap<String, Vec<String>> {
    let mut params = BTreeMap::new();
    if query.is_empty() {
        return params;
    }

    for part in query.split('&') {
        let (key, value) = part.split_once('=').unwrap_or((part, ""));
        params
            .entry(percent_decode_query(key))
            .or_insert_with(Vec::new)
            .push(percent_decode_query(value));
    }

    params
}

fn variable_matches(spec: &VariableSpec, params: &BTreeMap<String, Vec<String>>) -> bool {
    let needle = key_to_var(spec.key);
    params.iter().any(|(arg, values)| {
        if spec.match_on_arg
            && arg == spec.key
            && (values.len() != 1 || values[0].trim().is_empty())
        {
            return true;
        }

        values
            .iter()
            .any(|value| contains_ignore_ascii_case(value, &needle))
    })
}

fn replace_variable(
    spec: &VariableSpec,
    replacement: &str,
    params: &mut BTreeMap<String, Vec<String>>,
) {
    let needle = key_to_var(spec.key);

    for (arg, values) in params.iter_mut() {
        if spec.match_on_arg
            && arg == spec.key
            && (values.len() != 1 || values[0].trim().is_empty())
        {
            *values = alloc::vec![replacement.to_string()];
            continue;
        }

        for value in values.iter_mut() {
            *value = replace_ignore_ascii_case(value, &needle, replacement);
        }
    }
}

fn key_to_var(key: &str) -> String {
    ["${", key, "}"].concat()
}

fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    haystack
        .to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

fn replace_ignore_ascii_case(haystack: &str, needle: &str, replacement: &str) -> String {
    let lower = haystack.to_ascii_lowercase();
    let needle_lower = needle.to_ascii_lowercase();
    let mut out = String::new();
    let mut start = 0usize;

    while let Some(pos) = lower[start..].find(&needle_lower) {
        let absolute = start + pos;
        out.push_str(&haystack[start..absolute]);
        out.push_str(replacement);
        start = absolute + needle.len();
    }

    out.push_str(&haystack[start..]);
    out
}

fn encode_query(params: &BTreeMap<String, Vec<String>>) -> String {
    let mut out = String::new();
    let mut first = true;

    for (key, values) in params {
        if values.is_empty() {
            if !first {
                out.push('&');
            }
            first = false;
            out.push_str(&query_escape(key));
            out.push('=');
            continue;
        }

        for value in values {
            if !first {
                out.push('&');
            }
            first = false;
            out.push_str(&query_escape(key));
            out.push('=');
            out.push_str(&query_escape(value));
        }
    }

    out
}

fn percent_decode_query(input: &str) -> String {
    let mut out = String::new();
    let bytes = input.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let (Some(hi), Some(lo)) = (hex_value(bytes[i + 1]), hex_value(bytes[i + 2])) {
                    out.push((hi << 4 | lo) as char);
                    i += 3;
                } else {
                    out.push('%');
                    i += 1;
                }
            }
            b => {
                out.push(b as char);
                i += 1;
            }
        }
    }

    out
}

fn query_escape(input: &str) -> String {
    let mut out = String::new();
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    for &b in input.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                out.push(HEX[(b >> 4) as usize] as char);
                out.push(HEX[(b & 0x0f) as usize] as char);
            }
        }
    }

    out
}

fn hex_value(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_values() -> UrlVariableValues {
        UrlVariableValues::new()
            .with_uuid("0000-0000")
            .with_serial("12345")
            .with_mac("12:34:56:78:90:ab")
            .with_hostname("example-node")
            .with_code("top-secret")
    }

    #[test]
    fn detects_required_variables_like_talos() {
        assert!(required_variables("https://example.com?foo=bar").is_empty());
        assert_eq!(required_variables("https://example.com?uuid="), ["uuid"]);
        assert!(required_variables("https://example.com?uuid=0000-0000").is_empty());
        assert_eq!(
            required_variables("https://example.com?uuid=${UUId}&foo=bar&serial=${SeRiaL}",),
            ["uuid", "serial"]
        );
    }

    #[test]
    fn expands_legacy_uuid_arg() {
        assert_eq!(
            populate_url(
                "https://example.com?uuid=",
                &UrlVariableValues::new().with_uuid("0000-0000")
            )
            .unwrap(),
            "https://example.com?uuid=0000-0000"
        );
    }

    #[test]
    fn expands_query_variables_and_go_encodes_query() {
        assert_eq!(
            populate_url(
                "https://example.com?uuid=${uuid}&mac=${mac}&hostname=${hostname}&code=${code}",
                &full_values()
            )
            .unwrap(),
            "https://example.com?code=top-secret&hostname=example-node&mac=12%3A34%3A56%3A78%3A90%3Aab&uuid=0000-0000"
        );
    }

    #[test]
    fn expands_variables_case_insensitively() {
        assert_eq!(
            populate_url(
                "https://example.com?uuid=${UUId}&foo=bar&serial=${SeRiaL}",
                &full_values()
            )
            .unwrap(),
            "https://example.com?foo=bar&serial=12345&uuid=0000-0000"
        );
    }

    #[test]
    fn reports_missing_value_instead_of_guessing() {
        assert_eq!(
            populate_url("https://example.com?mac=${mac}", &UrlVariableValues::new()).unwrap_err(),
            UrlPopulateError::MissingVariable("mac")
        );
    }
}
