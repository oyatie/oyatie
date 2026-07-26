//! Print columns for `talosctl get`.
//!
//! Mirrors COSI's `meta.PrintColumn` (modeled on Kubernetes
//! `additionalPrinterColumns`). Each column has a header name and a JSONPath
//! expression evaluated against the resource spec to produce the cell value.
//!
//! This module keeps a tiny self-contained JSONPath evaluator over an
//! in-memory [`SpecValue`] tree so tests can exercise column rendering without
//! pulling in a JSON crate (the workspace forbids external dependencies).

use std::collections::BTreeMap;
use os_kernel::error::{Error, Result};

/// A single output column for `talosctl get`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintColumn {
    name: String,
    json_path: String,
}

impl PrintColumn {
    /// Construct a column with a header `name` and a `json_path` like
    /// `{.typeURL}` or `{.addresses[0]}`.
    pub fn new(name: impl Into<String>, json_path: impl Into<String>) -> Self {
        PrintColumn {
            name: name.into(),
            json_path: json_path.into(),
        }
    }

    /// The column header.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The raw JSONPath expression.
    pub fn json_path(&self) -> &str {
        &self.json_path
    }

    /// Render this column against a spec tree, returning the cell text.
    /// A missing path renders as an empty string (matching `talosctl`).
    pub fn render(&self, spec: &SpecValue) -> String {
        match eval_path(&self.json_path, spec) {
            Ok(Some(v)) => v.to_display(),
            _ => String::new(),
        }
    }
}

/// A minimal dynamic spec value used to render print columns in tests, standing
/// in for the JSON encoding of a concrete resource spec.
#[derive(Debug, Clone, PartialEq)]
pub enum SpecValue {
    /// Absence of a value.
    Null,
    /// A boolean.
    Bool(bool),
    /// An integer.
    Int(i64),
    /// A string.
    Str(String),
    /// An ordered list.
    List(Vec<SpecValue>),
    /// A keyed object (ordered for deterministic rendering).
    Map(BTreeMap<String, SpecValue>),
}

impl SpecValue {
    /// Render a leaf value as the text shown in a table cell.
    pub fn to_display(&self) -> String {
        match self {
            SpecValue::Null => String::new(),
            SpecValue::Bool(b) => b.to_string(),
            SpecValue::Int(i) => i.to_string(),
            SpecValue::Str(s) => s.clone(),
            SpecValue::List(items) => {
                let parts: Vec<String> = items.iter().map(Self::to_display).collect();
                parts.join(",")
            }
            SpecValue::Map(_) => String::from("<map>"),
        }
    }

    /// Convenience constructor for a map from key/value pairs.
    pub fn map<I, K>(entries: I) -> Self
    where
        I: IntoIterator<Item = (K, SpecValue)>,
        K: Into<String>,
    {
        SpecValue::Map(entries.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
}

/// A parsed JSONPath segment.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    /// `.field` access.
    Field(String),
    /// `[n]` index access.
    Index(usize),
}

/// Parse a `talosctl`-style JSONPath expression of the form `{.a.b[0].c}`.
///
/// The surrounding braces and leading dot are optional. Only field access and
/// numeric indexing are supported (the subset Talos RDs actually use).
fn parse_path(path: &str) -> Result<Vec<Segment>> {
    let trimmed = path.trim();
    let inner = trimmed
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .unwrap_or(trimmed);
    let inner = inner.trim();
    let inner = inner.strip_prefix('.').unwrap_or(inner);
    if inner.is_empty() {
        return Ok(Vec::new());
    }

    let mut segments = Vec::new();
    let mut chars = inner.chars().peekable();
    let mut field = String::new();

    let flush_field = |field: &mut String, segments: &mut Vec<Segment>| {
        if !field.is_empty() {
            segments.push(Segment::Field(std::mem::take(field)));
        }
    };

    while let Some(&c) = chars.peek() {
        match c {
            '.' => {
                chars.next();
                flush_field(&mut field, &mut segments);
            }
            '[' => {
                chars.next();
                flush_field(&mut field, &mut segments);
                let mut idx = String::new();
                for ic in chars.by_ref() {
                    if ic == ']' {
                        break;
                    }
                    idx.push(ic);
                }
                let n: usize = idx
                    .trim()
                    .parse()
                    .map_err(|_| Error::parse(format!("invalid array index '{idx}'")))?;
                segments.push(Segment::Index(n));
            }
            _ => {
                field.push(c);
                chars.next();
            }
        }
    }
    flush_field(&mut field, &mut segments);
    Ok(segments)
}

/// Evaluate a JSONPath against a spec, returning the located value if any.
fn eval_path<'a>(path: &str, root: &'a SpecValue) -> Result<Option<&'a SpecValue>> {
    let segments = parse_path(path)?;
    let mut cur = root;
    for seg in &segments {
        match (seg, cur) {
            (Segment::Field(name), SpecValue::Map(m)) => match m.get(name) {
                Some(v) => cur = v,
                None => return Ok(None),
            },
            (Segment::Index(i), SpecValue::List(items)) => match items.get(*i) {
                Some(v) => cur = v,
                None => return Ok(None),
            },
            _ => return Ok(None),
        }
    }
    Ok(Some(cur))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> SpecValue {
        SpecValue::map([
            ("hostname", SpecValue::Str("worker-1".into())),
            (
                "addresses",
                SpecValue::List(vec![
                    SpecValue::Str("10.0.0.5".into()),
                    SpecValue::Str("10.0.0.6".into()),
                ]),
            ),
            ("ready", SpecValue::Bool(true)),
            (
                "router",
                SpecValue::map([("priority", SpecValue::Int(100))]),
            ),
        ])
    }

    #[test]
    fn renders_simple_field() {
        let col = PrintColumn::new("HOSTNAME", "{.hostname}");
        assert_eq!(col.render(&sample()), "worker-1");
        assert_eq!(col.name(), "HOSTNAME");
    }

    #[test]
    fn renders_index_and_nested() {
        assert_eq!(
            PrintColumn::new("ADDR", "{.addresses[0]}").render(&sample()),
            "10.0.0.5"
        );
        assert_eq!(
            PrintColumn::new("PRIO", "{.router.priority}").render(&sample()),
            "100"
        );
        assert_eq!(
            PrintColumn::new("READY", "{.ready}").render(&sample()),
            "true"
        );
    }

    #[test]
    fn missing_path_renders_empty() {
        assert_eq!(PrintColumn::new("X", "{.nope}").render(&sample()), "");
        assert_eq!(
            PrintColumn::new("X", "{.addresses[9]}").render(&sample()),
            ""
        );
    }

    #[test]
    fn list_joins_with_comma() {
        assert_eq!(
            PrintColumn::new("ADDRS", "{.addresses}").render(&sample()),
            "10.0.0.5,10.0.0.6"
        );
    }

    #[test]
    fn path_parsing_handles_braces_and_dots() {
        assert_eq!(
            parse_path("{.a.b[2].c}").unwrap(),
            vec![
                Segment::Field("a".into()),
                Segment::Field("b".into()),
                Segment::Index(2),
                Segment::Field("c".into()),
            ]
        );
        assert_eq!(parse_path("{}").unwrap(), vec![]);
        assert!(parse_path("{.a[x]}").is_err());
    }
}
