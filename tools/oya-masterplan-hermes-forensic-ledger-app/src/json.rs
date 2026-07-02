//! Minimal owned JSON value model + deterministic pretty emitter.
//!
//! Object member order is insertion order (Vec-backed), so emitted ledgers and
//! masterplan splices are byte-deterministic for identical inputs. Only the
//! shapes this tool emits are modeled; there is intentionally no parser here —
//! the masterplan splice is a bracket-aware textual replacement that leaves
//! every byte outside the owned keys untouched.

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn str(value: impl Into<String>) -> Json {
        Json::Str(value.into())
    }
}

pub fn escape_json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

/// Render `value` pretty-printed with 2-space steps, where the value itself
/// starts at `base_indent` spaces of surrounding indentation (continuation
/// lines are indented relative to it; the first line carries no leading pad).
pub fn to_pretty(value: &Json, base_indent: usize) -> String {
    let mut out = String::new();
    write_value(value, base_indent, &mut out);
    out
}

fn write_value(value: &Json, indent: usize, out: &mut String) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Json::Int(i) => out.push_str(&i.to_string()),
        Json::Str(s) => {
            out.push('"');
            out.push_str(&escape_json_string(s));
            out.push('"');
        }
        Json::Arr(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                out.push_str(&" ".repeat(indent + 2));
                write_value(item, indent + 2, out);
                if i + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&" ".repeat(indent));
            out.push(']');
        }
        Json::Obj(members) => {
            if members.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{\n");
            for (i, (key, member)) in members.iter().enumerate() {
                out.push_str(&" ".repeat(indent + 2));
                out.push('"');
                out.push_str(&escape_json_string(key));
                out.push_str("\": ");
                write_value(member, indent + 2, out);
                if i + 1 < members.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&" ".repeat(indent));
            out.push('}');
        }
    }
}
