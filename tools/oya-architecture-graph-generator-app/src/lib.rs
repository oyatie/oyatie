//! Architecture-graph dashboard generator (CI pipeline tool).
//!
//! This is an OWNED-Rust binary invoked by the CI pipeline — NOT an `oya`
//! developer CLI subcommand and NOT the retired `oya doc graph emit` mechanism.
//! It regenerates `docs/architecture/product-graph.html` from source-of-truth
//! inputs so the dashboard can never silently rot, and a drift gate byte-compares
//! the committed HTML against the freshly generated output.
//!
//! Honesty note on provenance (see ADR-0364 for the masterplan half):
//!   * The `masterplan` section of the dashboard IS generated — it is a
//!     deterministic projection of `docs/machine-readable/masterplan.generated.json`
//!     (itself generated from the ADR corpus by `oya gen masterplan`).
//!   * The `_meta`, `verticals`, `techstack`, and `lanes` sections are a
//!     CURATED machine-readable SSOT checked in at
//!     `docs/machine-readable/architecture-graph.json`. This is a legitimate,
//!     DECLARED input — it is NOT derived from the Cargo workspace or ADR
//!     front-matter today. Deriving the verticals/tech-stack/lanes from the
//!     Cargo workspace + ADR front-matter is a future ratchet, not a current
//!     claim.
//!
//! The merged `GRAPH` object is emitted with keys in the exact order the
//! verified dashboard uses (`_meta, verticals, techstack, masterplan, lanes`)
//! and serialized with the same 2-space pretty-print the HTML bakes in. The
//! `serde_json` `preserve_order` feature is REQUIRED so key insertion order is
//! preserved (the dashboard key order is not alphabetical).

use std::path::Path;

use serde_json::{Map, Value};

/// Default path (repo-relative) of the curated architecture SSOT.
pub const DEFAULT_GRAPH_SSOT: &str = "docs/machine-readable/architecture-graph.json";
/// Default path (repo-relative) of the generated masterplan projection.
pub const DEFAULT_MASTERPLAN: &str = "docs/machine-readable/masterplan.generated.json";
/// Default path (repo-relative) of the dashboard template.
pub const DEFAULT_TEMPLATE: &str = "docs/architecture/product-graph.template.html";
/// Default path (repo-relative) of the generated dashboard.
pub const DEFAULT_OUTPUT: &str = "docs/architecture/product-graph.html";

/// The placeholder token in the template whose VALUE is replaced by the baked
/// `GRAPH` literal. Everything outside this token is preserved byte-for-byte.
pub const PLACEHOLDER: &str = "/*__GRAPH_DATA__*/ null /*__END__*/";

/// The `source` field stamped onto every masterplan milestone + the masterplan
/// `meta` block, matching the verified dashboard dataset.
const MASTERPLAN_SOURCE: &str = "source/docs/machine-readable/masterplan.generated.json";

/// Maximum length of a derived deliverable title before it is truncated.
/// Titles longer than this are cut to `MAX_TITLE_LEN - 3` chars + `"..."` so the
/// final string is exactly `MAX_TITLE_LEN` chars — matching the verified dataset.
const MAX_TITLE_LEN: usize = 140;

/// Errors the generator can surface. Kept as a flat enum (no third-party error
/// crate) per the minimal-dependency constraint.
#[derive(Debug)]
pub enum GenError {
    /// An input file could not be read.
    Read {
        path: String,
        source: std::io::Error,
    },
    /// An input file was not valid JSON.
    ParseJson {
        path: String,
        source: serde_json::Error,
    },
    /// An input JSON document did not have the expected shape.
    Shape(String),
    /// The template did not contain the expected placeholder token.
    Placeholder(String),
    /// The output file could not be written.
    Write {
        path: String,
        source: std::io::Error,
    },
    /// Serializing the merged GRAPH failed.
    Serialize(serde_json::Error),
}

impl std::fmt::Display for GenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenError::Read { path, source } => write!(f, "cannot read {path}: {source}"),
            GenError::ParseJson { path, source } => write!(f, "invalid JSON in {path}: {source}"),
            GenError::Shape(message) => write!(f, "unexpected document shape: {message}"),
            GenError::Placeholder(message) => write!(f, "template placeholder error: {message}"),
            GenError::Write { path, source } => write!(f, "cannot write {path}: {source}"),
            GenError::Serialize(source) => write!(f, "cannot serialize GRAPH: {source}"),
        }
    }
}

impl std::error::Error for GenError {}

/// Derive a dashboard deliverable title from a masterplan deliverable
/// `description`. The verified dataset takes the text up to (and not including)
/// the first sentence break (`". "`) or em-dash separator (`" — "`), whichever
/// comes first, then truncates to [`MAX_TITLE_LEN`] chars with a `"..."` suffix.
pub fn title_from_description(description: &str) -> String {
    let sentence = description.find(". ").unwrap_or(description.len());
    let em_dash = description.find(" \u{2014} ").unwrap_or(description.len());
    let cut = sentence.min(em_dash);
    let first = &description[..cut];

    if first.chars().count() > MAX_TITLE_LEN {
        let kept: String = first.chars().take(MAX_TITLE_LEN - 3).collect();
        format!("{kept}...")
    } else {
        first.to_string()
    }
}

/// Transform `masterplan.generated.json` into the dashboard `masterplan`
/// schema: `{ milestones: [{ id, title, adr_count, deliverables: [{ id, title,
/// status }], gate?, source }], meta: { ... } }`.
///
/// The transform reproduces the verified dashboard `masterplan` section
/// byte-for-byte (enforced by a golden unit test). The `gate` key is the
/// `verified_by` of the first deliverable in a milestone and is OMITTED when a
/// milestone has no deliverables.
pub fn masterplan_from_generated(generated: &Value) -> Result<Value, GenError> {
    let gen_milestones = generated
        .get("milestones")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            GenError::Shape("masterplan.generated.json: missing `milestones` array".into())
        })?;

    let mut milestones = Vec::with_capacity(gen_milestones.len());
    for gen_milestone in gen_milestones {
        let id = gen_milestone
            .get("milestone")
            .and_then(Value::as_str)
            .ok_or_else(|| GenError::Shape("milestone entry missing string `milestone`".into()))?;
        let adrs = gen_milestone
            .get("adrs")
            .and_then(Value::as_array)
            .ok_or_else(|| GenError::Shape(format!("milestone `{id}` missing `adrs` array")))?;

        let mut deliverables: Vec<Value> = Vec::new();
        let mut gate: Option<String> = None;
        for adr in adrs {
            let Some(adr_deliverables) = adr.get("deliverables").and_then(Value::as_array) else {
                continue;
            };
            for deliverable in adr_deliverables {
                let del_id = deliverable
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| GenError::Shape("deliverable missing string `id`".into()))?;
                let description = deliverable
                    .get("description")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        GenError::Shape(format!(
                            "deliverable `{del_id}` missing string `description`"
                        ))
                    })?;
                let status = deliverable
                    .get("status")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        GenError::Shape(format!("deliverable `{del_id}` missing string `status`"))
                    })?;

                if deliverables.is_empty() {
                    gate = deliverable
                        .get("verified_by")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                }

                let mut entry = Map::new();
                entry.insert("id".into(), Value::String(del_id.to_string()));
                entry.insert(
                    "title".into(),
                    Value::String(title_from_description(description)),
                );
                entry.insert("status".into(), Value::String(status.to_string()));
                deliverables.push(Value::Object(entry));
            }
        }

        let mut milestone = Map::new();
        milestone.insert("id".into(), Value::String(id.to_string()));
        milestone.insert("title".into(), Value::String(id.to_string()));
        milestone.insert("adr_count".into(), Value::Number(adrs.len().into()));
        let has_deliverables = !deliverables.is_empty();
        milestone.insert("deliverables".into(), Value::Array(deliverables));
        if has_deliverables {
            // `gate` is only present on milestones that carry deliverables.
            milestone.insert(
                "gate".into(),
                gate.map(Value::String).unwrap_or(Value::Null),
            );
        }
        milestone.insert("source".into(), Value::String(MASTERPLAN_SOURCE.into()));
        milestones.push(Value::Object(milestone));
    }

    let mut meta = Map::new();
    meta.insert("adr_count".into(), copy_field(generated, "adr_count")?);
    meta.insert(
        "deliverable_count".into(),
        copy_field(generated, "deliverable_count")?,
    );
    meta.insert(
        "deliverable_status_model".into(),
        copy_field(generated, "deliverable_status_model")?,
    );
    meta.insert("generator".into(), copy_field(generated, "generator")?);
    meta.insert("source".into(), Value::String(MASTERPLAN_SOURCE.into()));

    let mut masterplan = Map::new();
    masterplan.insert("milestones".into(), Value::Array(milestones));
    masterplan.insert("meta".into(), Value::Object(meta));
    Ok(Value::Object(masterplan))
}

fn copy_field(source: &Value, key: &str) -> Result<Value, GenError> {
    source
        .get(key)
        .cloned()
        .ok_or_else(|| GenError::Shape(format!("masterplan.generated.json: missing `{key}`")))
}

/// Merge the curated SSOT (`_meta`, `verticals`, `techstack`, `lanes`) and the
/// transformed `masterplan` into one `GRAPH` object with keys in the exact
/// dashboard order: `_meta, verticals, techstack, masterplan, lanes`.
pub fn merge_graph(ssot: &Value, masterplan: Value) -> Result<Value, GenError> {
    let mut graph = Map::new();
    for key in ["_meta", "verticals", "techstack"] {
        let value = ssot
            .get(key)
            .cloned()
            .ok_or_else(|| GenError::Shape(format!("architecture-graph.json: missing `{key}`")))?;
        graph.insert(key.to_string(), value);
    }
    graph.insert("masterplan".into(), masterplan);
    let lanes = ssot
        .get("lanes")
        .cloned()
        .ok_or_else(|| GenError::Shape("architecture-graph.json: missing `lanes`".into()))?;
    graph.insert("lanes".into(), lanes);
    Ok(Value::Object(graph))
}

/// Serialize the merged `GRAPH` with the same 2-space pretty-print the dashboard
/// bakes in (`serde_json::to_string_pretty`, Unicode preserved — serde does not
/// escape non-ASCII).
pub fn serialize_graph(graph: &Value) -> Result<String, GenError> {
    serde_json::to_string_pretty(graph).map_err(GenError::Serialize)
}

/// Inject the serialized `GRAPH` literal into the template at the placeholder
/// token, returning the full HTML. Everything outside the placeholder is
/// preserved byte-for-byte.
pub fn inject(template: &str, graph_literal: &str) -> Result<String, GenError> {
    let count = template.matches(PLACEHOLDER).count();
    if count != 1 {
        return Err(GenError::Placeholder(format!(
            "expected exactly one `{PLACEHOLDER}` token in template, found {count}"
        )));
    }
    Ok(template.replace(PLACEHOLDER, graph_literal))
}

/// Read a file as a UTF-8 string, mapping IO errors to [`GenError::Read`].
pub fn read_to_string(path: &Path) -> Result<String, GenError> {
    std::fs::read_to_string(path).map_err(|source| GenError::Read {
        path: path.display().to_string(),
        source,
    })
}

/// Parse a JSON string with the source path attached for error context.
pub fn parse_json(path: &Path, text: &str) -> Result<Value, GenError> {
    serde_json::from_str(text).map_err(|source| GenError::ParseJson {
        path: path.display().to_string(),
        source,
    })
}

/// End-to-end render: read inputs, transform, merge, serialize, inject. Returns
/// the complete HTML document (without writing it). Pure with respect to the
/// filesystem outputs — callers decide whether to write or drift-check.
pub fn render(
    ssot_path: &Path,
    masterplan_path: &Path,
    template_path: &Path,
) -> Result<String, GenError> {
    let ssot_text = read_to_string(ssot_path)?;
    let ssot = parse_json(ssot_path, &ssot_text)?;

    let masterplan_text = read_to_string(masterplan_path)?;
    let generated = parse_json(masterplan_path, &masterplan_text)?;
    let masterplan = masterplan_from_generated(&generated)?;

    let graph = merge_graph(&ssot, masterplan)?;
    let graph_literal = serialize_graph(&graph)?;

    let template = read_to_string(template_path)?;
    inject(&template, &graph_literal)
}

/// Outcome of an end-to-end run.
pub enum RunOutcome {
    /// `--write`: the dashboard was (re)written.
    Wrote,
    /// `--check`: the committed dashboard matches the regenerated HTML.
    Clean,
    /// `--check`: the committed dashboard drifted from the regenerated HTML.
    Drifted { committed_path: String },
}

/// Generate the dashboard HTML and either write it or drift-check it against the
/// committed file.
pub fn run(
    ssot_path: &Path,
    masterplan_path: &Path,
    template_path: &Path,
    output_path: &Path,
    check: bool,
) -> Result<RunOutcome, GenError> {
    let rendered = render(ssot_path, masterplan_path, template_path)?;

    if check {
        let committed = read_to_string(output_path)?;
        if committed == rendered {
            return Ok(RunOutcome::Clean);
        }
        return Ok(RunOutcome::Drifted {
            committed_path: output_path.display().to_string(),
        });
    }

    std::fs::write(output_path, rendered.as_bytes()).map_err(|source| GenError::Write {
        path: output_path.display().to_string(),
        source,
    })?;
    Ok(RunOutcome::Wrote)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_splits_on_em_dash() {
        let desc =
            "This ADR \u{2014} the generative ADR template + front-matter schema (the contract).";
        assert_eq!(title_from_description(desc), "This ADR");
    }

    #[test]
    fn title_splits_on_sentence_break() {
        let desc = "Human input is architecture-only: agents own everything except architectural decisions (ADRs). Human-touch on non-architecture merges is an exception.";
        assert_eq!(
            title_from_description(desc),
            "Human input is architecture-only: agents own everything except architectural decisions (ADRs)"
        );
    }

    #[test]
    fn title_truncates_to_140_chars() {
        let long = "x".repeat(200);
        let title = title_from_description(&long);
        assert_eq!(title.chars().count(), 140);
        assert!(title.ends_with("..."));
    }

    #[test]
    fn inject_replaces_single_placeholder() {
        let template = "const GRAPH = /*__GRAPH_DATA__*/ null /*__END__*/;\n";
        let out = inject(template, "{\"a\":1}").unwrap();
        assert_eq!(out, "const GRAPH = {\"a\":1};\n");
    }

    #[test]
    fn inject_rejects_missing_placeholder() {
        assert!(inject("const GRAPH = null;", "{}").is_err());
    }

    #[test]
    fn merge_preserves_dashboard_key_order() {
        let ssot = serde_json::json!({
            "_meta": {"k": 1},
            "verticals": [],
            "techstack": [],
            "lanes": []
        });
        let masterplan = serde_json::json!({"milestones": [], "meta": {}});
        let graph = merge_graph(&ssot, masterplan).unwrap();
        let keys: Vec<&str> = graph
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            ["_meta", "verticals", "techstack", "masterplan", "lanes"]
        );
    }
}
