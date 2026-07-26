//! A tiny, dependency-free YAML-*subset* parser.
//!
//! This is **not** a general YAML implementation. It supports exactly the shape
//! the Talos machine-config corpus in `difftest/configs/` uses, which the real
//! Talos loader (`configloader.NewFromBytes`) accepts:
//!
//! - 2-space-indent nested mappings (`key:` then deeper-indented children);
//! - scalar values: bare strings, integers, booleans (`true`/`false`), and the
//!   double-quoted empty string `""`;
//! - simple block lists of scalars (`- item` lines);
//! - the inline empty mapping `{}`;
//! - `#` line comments and blank lines;
//! - the top-level keys `version` / `machine` / `cluster`.
//!
//! Anything outside this subset (flow mappings other than `{}`, anchors,
//! multi-line/literal scalars, tabs for indentation) is rejected with a parse
//! error rather than silently mis-parsed.
//!
//! The parser deliberately mirrors one structural rule of real YAML that the
//! corpus exercises in its malformed case: a key whose value is a scalar on the
//! same line cannot also have deeper-indented child lines. That combination
//! (`machine: scalar` followed by an indented `type:`) is a decode error, which
//! is how the oracle classifies `11-invalid-malformed.yaml`.

use std::collections::BTreeMap;
use std::fmt;

/// A parsed YAML value from the supported subset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Yaml {
    /// A scalar leaf (string / int / bool), stored as its raw decoded string.
    ///
    /// Booleans and integers are kept as their textual form; callers that need
    /// typed access use [`Yaml::as_bool`] / [`Yaml::as_str`].
    Scalar(String),
    /// A mapping, preserving insertion order is not required by the corpus, so
    /// a sorted map is used for deterministic lookups.
    Mapping(BTreeMap<String, Yaml>),
    /// A block sequence of scalars.
    Sequence(Vec<Yaml>),
}

impl Yaml {
    /// Borrow this value as a string scalar, if it is one.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Yaml::Scalar(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Interpret this value as a boolean (`true`/`false`), if it is a scalar
    /// holding one of those tokens.
    pub fn as_bool(&self) -> Option<bool> {
        match self.as_str()? {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }

    /// Borrow this value as a mapping, if it is one.
    pub fn as_mapping(&self) -> Option<&BTreeMap<String, Yaml>> {
        match self {
            Yaml::Mapping(m) => Some(m),
            _ => None,
        }
    }

    /// Borrow this value as a sequence, if it is one.
    pub fn as_sequence(&self) -> Option<&[Yaml]> {
        match self {
            Yaml::Sequence(s) => Some(s.as_slice()),
            _ => None,
        }
    }

    /// Look up `key` in a mapping value (returns `None` for non-mappings).
    pub fn get(&self, key: &str) -> Option<&Yaml> {
        self.as_mapping()?.get(key)
    }

    /// Convenience: the string at a single mapping key, or `None`.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_str()
    }
}

/// An error produced while parsing the YAML subset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// 1-based source line number where the problem was detected.
    pub line: usize,
    /// Human-readable description.
    pub message: String,
}

impl ParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        ParseError {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "yaml parse error at line {}: {}",
            self.line, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// A logical, comment/blank-stripped source line with its indentation depth.
struct Line {
    /// 1-based source line number (for diagnostics).
    number: usize,
    /// Indentation in spaces.
    indent: usize,
    /// The trimmed content (no leading indent, no trailing whitespace).
    content: String,
}

/// Parse a YAML-subset document into a [`Yaml`] tree.
///
/// The corpus documents are always a top-level mapping; an empty document
/// yields an empty mapping.
///
/// # Errors
///
/// Returns a [`ParseError`] if the input uses a feature outside the supported
/// subset, has inconsistent indentation, uses tabs, or mixes a same-line scalar
/// value with deeper-indented children.
pub fn parse(input: &str) -> Result<Yaml, ParseError> {
    let lines = lex(input)?;
    let mut pos = 0;
    let value = parse_block(&lines, &mut pos, 0)?;
    Ok(value)
}

/// Strip comments / blanks and compute indentation for each meaningful line.
fn lex(input: &str) -> Result<Vec<Line>, ParseError> {
    let mut out = Vec::new();
    for (idx, raw) in input.lines().enumerate() {
        let number = idx + 1;
        if raw.contains('\t') {
            // Reject tabs in indentation region; the corpus is pure spaces.
            let leading_tab = raw.len() - raw.trim_start().len();
            if raw[..leading_tab].contains('\t') {
                return Err(ParseError::new(number, "tab used for indentation"));
            }
        }
        // Determine indent (count of leading spaces).
        let indent = raw.len() - raw.trim_start().len();
        let trimmed = raw.trim_start();
        // Skip whole-line comments and blank lines.
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let content = strip_inline_comment(trimmed).trim_end().to_string();
        if content.is_empty() {
            continue;
        }
        out.push(Line {
            number,
            indent,
            content,
        });
    }
    Ok(out)
}

/// Strip a trailing ` # comment`, but never inside a quoted scalar.
fn strip_inline_comment(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut in_quote = false;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'"' {
            in_quote = !in_quote;
        } else if c == b'#' && !in_quote {
            // A comment must be preceded by whitespace (or start of string) to
            // count; otherwise it is part of the token (e.g. a URL fragment).
            if i == 0 || bytes[i - 1] == b' ' {
                return &s[..i];
            }
        }
        i += 1;
    }
    s
}

/// Parse a mapping (or sequence) block at the given indentation level.
fn parse_block(lines: &[Line], pos: &mut usize, indent: usize) -> Result<Yaml, ParseError> {
    if *pos >= lines.len() {
        return Ok(Yaml::Mapping(BTreeMap::new()));
    }
    if lines[*pos].content.starts_with("- ") || lines[*pos].content == "-" {
        return parse_sequence(lines, pos, indent);
    }
    parse_mapping(lines, pos, indent)
}

/// Parse a block mapping whose entries sit at `indent`.
fn parse_mapping(lines: &[Line], pos: &mut usize, indent: usize) -> Result<Yaml, ParseError> {
    let mut map = BTreeMap::new();
    while *pos < lines.len() {
        let line = &lines[*pos];
        if line.indent < indent {
            break;
        }
        if line.indent > indent {
            return Err(ParseError::new(line.number, "unexpected indentation"));
        }
        if line.content.starts_with("- ") || line.content == "-" {
            return Err(ParseError::new(
                line.number,
                "list item where mapping key expected",
            ));
        }
        let (key, rest) = split_key(line)?;
        let key_line = line.number;
        *pos += 1;

        if let Some(rest) = rest {
            // Value present on the same line.
            if rest == "{}" {
                map.insert(key, Yaml::Mapping(BTreeMap::new()));
            } else {
                // A same-line scalar must NOT be followed by deeper-indented
                // child lines. This is the malformed-corpus decode error.
                if *pos < lines.len() && lines[*pos].indent > indent {
                    return Err(ParseError::new(
                        lines[*pos].number,
                        "block mapping entry has both a scalar value and child nodes",
                    ));
                }
                map.insert(key, Yaml::Scalar(decode_scalar(&rest)));
            }
        } else {
            // No same-line value: children (nested map or sequence) at a
            // deeper indent, or an empty value.
            if *pos < lines.len() && lines[*pos].indent > indent {
                let child_indent = lines[*pos].indent;
                let child = parse_block(lines, pos, child_indent)?;
                map.insert(key, child);
            } else if *pos < lines.len()
                && lines[*pos].indent == indent
                && (lines[*pos].content.starts_with("- ") || lines[*pos].content == "-")
            {
                // Sequence items aligned with the key (valid YAML).
                let child = parse_sequence(lines, pos, indent)?;
                map.insert(key, child);
            } else {
                let _ = key_line;
                map.insert(key, Yaml::Scalar(String::new()));
            }
        }
    }
    Ok(Yaml::Mapping(map))
}

/// Parse a block sequence at `indent`.
///
/// Two item shapes are supported, matching the corpus:
///
/// - **scalar items** — `- value` (a bare scalar after the dash); and
/// - **map items** — `- key: value` whose remaining `key: value` pairs sit on
///   the following lines indented to align past the `- ` marker, e.g.
///
///   ```text
///   - name: eth0
///     dhcp: true
///   ```
///
/// A single sequence may not mix the two shapes; an item that begins a mapping
/// continues until a line returns to (or below) the dash indentation.
fn parse_sequence(lines: &[Line], pos: &mut usize, indent: usize) -> Result<Yaml, ParseError> {
    let mut items = Vec::new();
    while *pos < lines.len() {
        let line = &lines[*pos];
        if line.indent < indent {
            break;
        }
        if line.indent > indent {
            return Err(ParseError::new(
                line.number,
                "unexpected indentation in sequence",
            ));
        }
        if line.content == "-" {
            return Err(ParseError::new(
                line.number,
                "empty sequence item not supported",
            ));
        }
        let Some(item) = line.content.strip_prefix("- ") else {
            break;
        };
        let number = line.number;
        // The content that follows the dash starts at `indent + 2` columns
        // (`- ` is two characters). A map item has that content shaped as
        // `key:` / `key: value`; a scalar item is a bare token.
        let item = item.to_string();
        let inner_indent = indent + 2;

        if looks_like_mapping_entry(&item) {
            // Map item: synthesize the dash line as the first entry of a child
            // mapping, then fold in any following deeper-indented lines.
            let mut entry_lines = vec![Line {
                number,
                indent: inner_indent,
                content: item,
            }];
            *pos += 1;
            // Fold in the following lines that belong to this map item. A
            // sequence item at exactly `inner_indent` would start a *new* item
            // of the parent sequence (the dash aligns with this item's keys),
            // so it terminates the fold; deeper sequence items are a nested
            // sequence under one of this item's keys (e.g. `addresses:`) and are
            // re-parsed by `parse_mapping`.
            while *pos < lines.len() {
                let l = &lines[*pos];
                let is_seq_item = l.content.starts_with("- ") || l.content == "-";
                if l.indent < inner_indent || (is_seq_item && l.indent == inner_indent) {
                    break;
                }
                entry_lines.push(Line {
                    number: l.number,
                    indent: l.indent,
                    content: l.content.clone(),
                });
                *pos += 1;
            }
            let mut inner_pos = 0;
            let value = parse_mapping(&entry_lines, &mut inner_pos, inner_indent)?;
            items.push(value);
        } else {
            items.push(Yaml::Scalar(decode_scalar(item.trim())));
            *pos += 1;
        }
    }
    Ok(Yaml::Sequence(items))
}

/// Heuristic: does this post-dash content begin a `key: value` mapping entry?
///
/// True when there is a `:` that is either at end-of-token (`key:`) or followed
/// by a space (`key: value`), and the key is a plain unquoted token. A bare
/// scalar such as `10.244.0.0/16` or `https://host:6443` is NOT a mapping entry
/// because its colon is not followed by a space and the token is not consumed as
/// a key here (those are list-of-scalar items).
fn looks_like_mapping_entry(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut i = 0;
    let mut in_quote = false;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => in_quote = !in_quote,
            b':' if !in_quote => {
                let key = content[..i].trim();
                let after = &content[i + 1..];
                if !key.is_empty()
                    && !key.contains(' ')
                    && (after.is_empty() || after.starts_with(' '))
                {
                    return true;
                }
                return false;
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Split a `key: value` (or `key:`) line into key and optional same-line value.
fn split_key(line: &Line) -> Result<(String, Option<String>), ParseError> {
    let content = &line.content;
    // Find the first ':' that terminates the key. Keys in the corpus are plain
    // identifiers, never quoted, so a bare scan suffices.
    let Some(colon) = content.find(':') else {
        return Err(ParseError::new(line.number, "mapping entry missing ':'"));
    };
    let key = content[..colon].trim();
    if key.is_empty() {
        return Err(ParseError::new(line.number, "empty mapping key"));
    }
    let after = content[colon + 1..].trim();
    if after.is_empty() {
        Ok((key.to_string(), None))
    } else {
        Ok((key.to_string(), Some(after.to_string())))
    }
}

/// Decode a scalar token: strip surrounding double quotes if present.
fn decode_scalar(raw: &str) -> String {
    let t = raw.trim();
    if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
        return t[1..t.len() - 1].to_string();
    }
    t.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_map() {
        let src =
            "version: v1alpha1\nmachine:\n  type: controlplane\n  install:\n    disk: /dev/sda\n";
        let y = parse(src).unwrap();
        assert_eq!(y.get_str("version"), Some("v1alpha1"));
        assert_eq!(
            y.get("machine").unwrap().get_str("type"),
            Some("controlplane")
        );
        assert_eq!(
            y.get("machine")
                .unwrap()
                .get("install")
                .unwrap()
                .get_str("disk"),
            Some("/dev/sda")
        );
    }

    #[test]
    fn parses_bool_and_int() {
        let src = "machine:\n  features:\n    hostDNS:\n      enabled: true\n      forwardKubeDNSToHost: false\n  port: 6443\n";
        let y = parse(src).unwrap();
        let hostdns = y
            .get("machine")
            .unwrap()
            .get("features")
            .unwrap()
            .get("hostDNS")
            .unwrap();
        assert_eq!(hostdns.get("enabled").unwrap().as_bool(), Some(true));
        assert_eq!(
            hostdns.get("forwardKubeDNSToHost").unwrap().as_bool(),
            Some(false)
        );
        assert_eq!(y.get("machine").unwrap().get_str("port"), Some("6443"));
    }

    #[test]
    fn parses_scalar_list() {
        let src = "machine:\n  certSANs:\n    - 10.0.0.10\n    - 192.168.0.10\n";
        let y = parse(src).unwrap();
        let sans = y
            .get("machine")
            .unwrap()
            .get("certSANs")
            .unwrap()
            .as_sequence()
            .unwrap();
        assert_eq!(sans.len(), 2);
        assert_eq!(sans[0].as_str(), Some("10.0.0.10"));
        assert_eq!(sans[1].as_str(), Some("192.168.0.10"));
    }

    #[test]
    fn comments_and_blanks_ignored() {
        let src = "# header comment\nversion: v1alpha1   # inline\n\nmachine:\n  # nested comment\n  type: worker\n";
        let y = parse(src).unwrap();
        assert_eq!(y.get_str("version"), Some("v1alpha1"));
        assert_eq!(y.get("machine").unwrap().get_str("type"), Some("worker"));
    }

    #[test]
    fn quoted_empty_string() {
        let src = "machine:\n  ca:\n    key: \"\"\n";
        let y = parse(src).unwrap();
        assert_eq!(
            y.get("machine").unwrap().get("ca").unwrap().get_str("key"),
            Some("")
        );
    }

    #[test]
    fn inline_empty_map() {
        let src = "cluster:\n  controlPlane: {}\n";
        let y = parse(src).unwrap();
        let cp = y.get("cluster").unwrap().get("controlPlane").unwrap();
        assert_eq!(cp.as_mapping().map(BTreeMap::len), Some(0));
    }

    #[test]
    fn scalar_then_child_is_error() {
        // Mirrors 11-invalid-malformed.yaml.
        let src = "version: v1alpha1\nmachine: this-is-a-scalar-not-a-map\n  type: controlplane\n";
        let err = parse(src).unwrap_err();
        assert_eq!(err.line, 3);
    }

    #[test]
    fn tab_indent_rejected() {
        let src = "machine:\n\ttype: worker\n";
        assert!(parse(src).is_err());
    }

    #[test]
    fn comment_in_url_not_stripped() {
        let src = "cluster:\n  controlPlane:\n    endpoint: https://10.0.0.1:6443\n";
        let y = parse(src).unwrap();
        assert_eq!(
            y.get("cluster")
                .unwrap()
                .get("controlPlane")
                .unwrap()
                .get_str("endpoint"),
            Some("https://10.0.0.1:6443")
        );
    }

    #[test]
    fn empty_document_is_empty_map() {
        let y = parse("# just a comment\n\n").unwrap();
        assert_eq!(y.as_mapping().map(BTreeMap::len), Some(0));
    }

    #[test]
    fn parses_list_of_maps() {
        let src = "machine:\n  network:\n    interfaces:\n      - interface: eth0\n        dhcp: true\n      - interface: eth1\n        dhcp: false\n";
        let y = parse(src).unwrap();
        let ifaces = y
            .get("machine")
            .unwrap()
            .get("network")
            .unwrap()
            .get("interfaces")
            .unwrap()
            .as_sequence()
            .unwrap();
        assert_eq!(ifaces.len(), 2);
        assert_eq!(ifaces[0].get_str("interface"), Some("eth0"));
        assert_eq!(ifaces[0].get("dhcp").unwrap().as_bool(), Some(true));
        assert_eq!(ifaces[1].get_str("interface"), Some("eth1"));
        assert_eq!(ifaces[1].get("dhcp").unwrap().as_bool(), Some(false));
    }

    #[test]
    fn list_of_maps_with_nested_map_item() {
        // A map item whose value is itself a nested mapping.
        let src = "items:\n  - name: a\n    meta:\n      kind: x\n      tier: 1\n  - name: b\n    meta:\n      kind: y\n      tier: 2\n";
        let y = parse(src).unwrap();
        let items = y.get("items").unwrap().as_sequence().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].get_str("name"), Some("a"));
        assert_eq!(items[0].get("meta").unwrap().get_str("kind"), Some("x"));
        assert_eq!(items[0].get("meta").unwrap().get_str("tier"), Some("1"));
        assert_eq!(items[1].get("meta").unwrap().get_str("kind"), Some("y"));
    }

    #[test]
    fn list_of_maps_with_nested_sequence_item() {
        // A map item whose value is itself a nested sequence (e.g. an interface
        // with an `addresses:` list). The first item carries the addresses; the
        // second is a sibling interface that must not be folded into the first.
        let src = "machine:\n  network:\n    interfaces:\n      - interface: eth0\n        addresses:\n          - 10.0.2.15/24\n          - 10.0.2.16/24\n      - interface: eth1\n        dhcp: true\n";
        let y = parse(src).unwrap();
        let ifaces = y
            .get("machine")
            .unwrap()
            .get("network")
            .unwrap()
            .get("interfaces")
            .unwrap()
            .as_sequence()
            .unwrap();
        assert_eq!(ifaces.len(), 2);
        assert_eq!(ifaces[0].get_str("interface"), Some("eth0"));
        let addrs = ifaces[0].get("addresses").unwrap().as_sequence().unwrap();
        assert_eq!(addrs.len(), 2);
        assert_eq!(addrs[0].as_str(), Some("10.0.2.15/24"));
        assert_eq!(addrs[1].as_str(), Some("10.0.2.16/24"));
        assert_eq!(ifaces[1].get_str("interface"), Some("eth1"));
        assert_eq!(ifaces[1].get("dhcp").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn scalar_list_with_colon_values_not_treated_as_maps() {
        // Subnets and URLs contain colons but are list-of-scalar items.
        let src = "cluster:\n  network:\n    podSubnets:\n      - 10.244.0.0/16\n      - fd00:10:244::/56\n";
        let y = parse(src).unwrap();
        let pods = y
            .get("cluster")
            .unwrap()
            .get("network")
            .unwrap()
            .get("podSubnets")
            .unwrap()
            .as_sequence()
            .unwrap();
        assert_eq!(pods.len(), 2);
        assert_eq!(pods[0].as_str(), Some("10.244.0.0/16"));
        assert_eq!(pods[1].as_str(), Some("fd00:10:244::/56"));
    }

    #[test]
    fn deeper_nesting_map_under_map() {
        let src = "machine:\n  sysctls:\n    net.ipv4.ip_forward: \"1\"\n    net.core.somaxconn: \"65535\"\n  features:\n    hostDNS:\n      enabled: true\n      forwardKubeDNSToHost: true\n";
        let y = parse(src).unwrap();
        let sysctls = y.get("machine").unwrap().get("sysctls").unwrap();
        assert_eq!(sysctls.get_str("net.ipv4.ip_forward"), Some("1"));
        assert_eq!(sysctls.get_str("net.core.somaxconn"), Some("65535"));
        let hostdns = y
            .get("machine")
            .unwrap()
            .get("features")
            .unwrap()
            .get("hostDNS")
            .unwrap();
        assert_eq!(hostdns.get("enabled").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn looks_like_mapping_entry_classification() {
        assert!(looks_like_mapping_entry("interface: eth0"));
        assert!(looks_like_mapping_entry("dhcp:"));
        assert!(!looks_like_mapping_entry("10.244.0.0/16"));
        assert!(!looks_like_mapping_entry("https://10.0.0.1:6443"));
        assert!(!looks_like_mapping_entry("8.8.8.8"));
    }
}
