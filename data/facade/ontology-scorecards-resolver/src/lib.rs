//! Scorecard resolver kernel (ADR-0064 canonical base + localization).
//!
//! Naming justification:
//! - Crate id `resolve-scorecards-app` — no `oya-` brand prefix (ADR-0719
//!   D-9), `resolve` verb (kernel-tier action), `scorecards`
//!   subject, `app` Layer-3 (use-case orchestrator that wires kernel +
//!   adapter IO).
//! - Library identifier `data_ontology_scorecards_resolver` — snake_case mirror
//!   (ADR-0105 v4 BNF §2.2).
//!
//! Replaces `scripts/resolve-scorecards.py`. Reads four canonical
//! framework scorecards (AWS Well-Architected, Google SRE PRR, CIS
//! Kubernetes Benchmark, SLSA L3) under
//! `specs/microservices/scorecards/canonical/` plus per-µservice override
//! files under `microservices/<ms>/scorecards/overrides.json`, then emits
//! the resolved per-µservice view at audit time.
//!
//! Tier 1 (kernel-tier) per ADR-0083: pure logic over already-parsed
//! `serde_json::Value` documents. IO + CLI dispatch live in `src/main.rs`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

use serde_json::{Map, Value, json};

/// The four canonical frameworks per ADR-0064 + SWEEP-H. Each entry is
/// `(framework slug used on disk, overrides-file key, display label)`.
pub const FRAMEWORKS: &[(&str, &str, &str)] = &[
    (
        "aws-well-architected",
        "aws_well_architected",
        "AWS Well-Architected (5 pillars)",
    ),
    (
        "google-sre-prr",
        "google_sre_prr",
        "Google SRE Production Readiness Review",
    ),
    (
        "cis-k8s-benchmark",
        "cis_k8s_benchmark",
        "CIS Kubernetes Benchmark v1.10",
    ),
    (
        "slsa-l3",
        "slsa_l3",
        "SLSA (Supply-chain Levels for Software Artifacts) v1.0",
    ),
];

/// Spec-mandated 32-µservice scope. Order matches the legacy Python script.
pub const MICROSERVICES: &[&str] = &[
    "application",
    "audit-chain",
    "cell",
    "community",
    "observability",
    "ontology",
    "tenancy",
    "workflow-engine",
    "anonymous",
    "calendar",
    "docs",
    "drive",
    "foundry",
    "forms",
    "mail",
    "meet",
    "messenger",
    "network",
    "notes",
    "recordings",
    "sheets",
    "shorts",
    "sites",
    "slides",
    "social",
    "tasks",
    "translate",
    "workflow-studio",
    "cloud-iac",
    "cloud-k8s",
    "cloud-secrets",
    "governance",
];

/// Resolve one canonical scorecard against one override file.
///
/// Semantics mirror the legacy Python `resolve_scorecard()`:
///  1. recursively substitute `<ms>` + `<chart>` placeholders,
///  2. rename `evidence_pattern` → `evidence`,
///  3. drop canonical-only metadata keys (`_placeholders`,
///     `canonical_base`, `overlay_consumers`),
///  4. stamp `microservice` + `summary`,
///  5. honor `overall_status` override,
///  6. apply per-framework deltas (evidence_suffix appends, scalar
///     field overrides per matching control id).
pub fn resolve_scorecard(
    framework: &str,
    canonical: &Value,
    overrides: &Value,
    ms: &str,
) -> Result<Value, Error> {
    let chart = overrides
        .get("chart_name")
        .and_then(|v| v.as_str())
        .unwrap_or(ms)
        .to_string();
    let mut resolved = substitute_placeholders(canonical, ms, &chart);
    resolved = rename_evidence_pattern(resolved);
    if let Value::Object(map) = &mut resolved {
        map.remove("_placeholders");
        map.remove("canonical_base");
        map.remove("overlay_consumers");
        map.insert("microservice".into(), Value::String(ms.to_string()));
        if let Some(summary) = overrides.get("summary").and_then(|v| v.as_str()) {
            map.insert("summary".into(), Value::String(summary.to_string()));
        }
    }

    let fw_key = framework_overrides_key(framework)
        .ok_or_else(|| Error::UnknownFramework(framework.to_string()))?;
    let fw_overrides = overrides.get(fw_key).cloned().unwrap_or_else(|| json!({}));
    if let Some(status) = fw_overrides.get("overall_status").and_then(|v| v.as_str())
        && let Value::Object(map) = &mut resolved
    {
        map.insert("overall_status".into(), Value::String(status.to_string()));
    }
    if let Some(deltas) = fw_overrides.get("deltas").and_then(|v| v.as_array()) {
        for delta in deltas {
            apply_delta(framework, &mut resolved, delta);
        }
    }
    Ok(resolved)
}

/// Build the aggregate rollup index referenced by
/// `registry/hyperscaler-scorecards/index.json`.
pub fn build_rollup(generated_at: &str, overrides_by_ms: &[(String, Value)]) -> Value {
    let mut microservices = Map::new();
    let mut all_green = true;
    let mut keys: Vec<&str> = overrides_by_ms.iter().map(|(k, _)| k.as_str()).collect();
    keys.sort_unstable();
    for ms in &keys {
        let Some(entry) = overrides_by_ms.iter().find(|(name, _)| name == *ms) else {
            continue;
        };
        let overrides = &entry.1;
        let mut row = Map::new();
        let mut ms_green = true;
        for (_slug, key, _label) in FRAMEWORKS {
            let status = overrides
                .get(*key)
                .and_then(|v| v.get("overall_status"))
                .and_then(|v| v.as_str())
                .unwrap_or("green")
                .to_string();
            if status != "green" {
                ms_green = false;
            }
            row.insert((*key).to_string(), Value::String(status));
        }
        let chart_name = overrides
            .get("chart_name")
            .and_then(|v| v.as_str())
            .unwrap_or(ms)
            .to_string();
        row.insert(
            "overrides_path".to_string(),
            Value::String(format!("microservices/{ms}/scorecards/overrides.json")),
        );
        row.insert("chart_name".to_string(), Value::String(chart_name));
        if !ms_green {
            all_green = false;
        }
        microservices.insert((*ms).to_string(), Value::Object(row));
    }

    let mut canonical_paths = Map::new();
    for (slug, key, _) in FRAMEWORKS {
        canonical_paths.insert(
            (*key).to_string(),
            Value::String(format!(
                "specs/microservices/scorecards/canonical/{slug}.json"
            )),
        );
    }

    json!({
        "$schema": "https://oyatie.com/schemas/hyperscaler-scorecard-rollup.json",
        "schema_version": "1.1",
        "generated_at": generated_at,
        "sweep": "SWEEP-H",
        "frameworks": FRAMEWORKS.iter().map(|(_, _, label)| *label).collect::<Vec<_>>(),
        "canonical_base_paths": canonical_paths,
        "canonical_authority": "ADR-0064",
        "aggregate_status": if all_green { "green" } else { "yellow" },
        "microservices": microservices,
    })
}

fn framework_overrides_key(framework: &str) -> Option<&'static str> {
    FRAMEWORKS
        .iter()
        .find(|(slug, _, _)| *slug == framework)
        .map(|(_, key, _)| *key)
}

fn substitute_placeholders(value: &Value, ms: &str, chart: &str) -> Value {
    match value {
        Value::String(s) => Value::String(s.replace("<ms>", ms).replace("<chart>", chart)),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|v| substitute_placeholders(v, ms, chart))
                .collect(),
        ),
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                out.insert(k.clone(), substitute_placeholders(v, ms, chart));
            }
            Value::Object(out)
        }
        other => other.clone(),
    }
}

fn rename_evidence_pattern(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                let key = if k == "evidence_pattern" {
                    "evidence".to_string()
                } else {
                    k
                };
                out.insert(key, rename_evidence_pattern(v));
            }
            Value::Object(out)
        }
        Value::Array(items) => {
            Value::Array(items.into_iter().map(rename_evidence_pattern).collect())
        }
        other => other,
    }
}

fn apply_delta(framework: &str, resolved: &mut Value, delta: &Value) {
    let Some(control) = delta.get("control").and_then(|v| v.as_str()) else {
        return;
    };
    let Some(field) = delta.get("field").and_then(|v| v.as_str()) else {
        return;
    };
    let value = delta.get("value").cloned().unwrap_or(Value::Null);

    match framework {
        "aws-well-architected" => {
            if let Some(pillars) = resolved.get_mut("pillars").and_then(|v| v.as_object_mut()) {
                for pillar in pillars.values_mut() {
                    if let Some(controls) =
                        pillar.get_mut("controls").and_then(|v| v.as_array_mut())
                    {
                        for ctrl in controls {
                            if ctrl.get("id").and_then(|v| v.as_str()) == Some(control) {
                                apply_field_update(ctrl, field, &value);
                            }
                        }
                    }
                }
            }
        }
        "cis-k8s-benchmark" => {
            if let Some(cats) = resolved
                .get_mut("categories")
                .and_then(|v| v.as_object_mut())
            {
                for cat in cats.values_mut() {
                    if let Some(controls) = cat.get_mut("controls").and_then(|v| v.as_array_mut()) {
                        for ctrl in controls {
                            if ctrl.get("id").and_then(|v| v.as_str()) == Some(control) {
                                apply_field_update(ctrl, field, &value);
                            }
                        }
                    }
                }
            }
        }
        "google-sre-prr" => {
            if let Some(item) = resolved
                .get_mut("checklist")
                .and_then(|v| v.as_object_mut())
                .and_then(|m| m.get_mut(control))
            {
                match field {
                    "slo_count" | "runbook_count" | "dashboard_count" => {
                        let existing = item
                            .get("evidence")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let count = value_to_str(&value);
                        let new = format!("{existing} count: {count}").trim().to_string();
                        if let Value::Object(map) = item {
                            map.insert("evidence".into(), Value::String(new));
                        }
                    }
                    "evidence_suffix" => apply_evidence_suffix(item, &value),
                    _ => apply_field_update(item, field, &value),
                }
            }
        }
        "slsa-l3" => {
            if let Some(reqs) = resolved
                .get_mut("requirements")
                .and_then(|v| v.as_object_mut())
            {
                for sect in reqs.values_mut() {
                    if let Value::Object(map) = sect
                        && let Some(item) = map.get_mut(control)
                    {
                        apply_field_update(item, field, &value);
                    }
                }
            }
        }
        _ => {}
    }
}

fn apply_field_update(item: &mut Value, field: &str, value: &Value) {
    if field == "evidence_suffix" {
        apply_evidence_suffix(item, value);
        return;
    }
    if let Value::Object(map) = item {
        map.insert(field.to_string(), value.clone());
    }
}

fn apply_evidence_suffix(item: &mut Value, value: &Value) {
    if let Value::Object(map) = item {
        let existing = map
            .get("evidence")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let suffix = value_to_str(value);
        let merged = format!("{existing} {suffix}").trim().to_string();
        map.insert("evidence".into(), Value::String(merged));
    }
}

fn value_to_str(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    UnknownFramework(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFramework(s) => write!(f, "unknown framework: {s}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn canonical_aws() -> Value {
        json!({
            "framework": "AWS Well-Architected (5 pillars)",
            "canonical_base": true,
            "_placeholders": {"<ms>": "slug"},
            "overlay_consumers": "<ms>",
            "pillars": {
                "operational_excellence": {
                    "status": "green",
                    "controls": [
                        {
                            "id": "OPS-01",
                            "name": "Runbooks",
                            "evidence_pattern": "microservices/<ms>/runbooks/",
                            "passing": true
                        }
                    ]
                }
            }
        })
    }

    fn overrides_ok(chart: &str, summary: &str) -> Value {
        json!({
            "chart_name": chart,
            "summary": summary,
            "aws_well_architected": {
                "overall_status": "green",
                "deltas": [
                    {"control": "OPS-01", "field": "evidence_suffix", "value": "(6 runbooks)"}
                ]
            }
        })
    }

    #[test]
    fn resolve_substitutes_ms_and_chart() {
        let resolved = resolve_scorecard(
            "aws-well-architected",
            &canonical_aws(),
            &overrides_ok("foo-chart", "demo"),
            "foo",
        )
        .expect("resolve");
        let ev = resolved["pillars"]["operational_excellence"]["controls"][0]["evidence"]
            .as_str()
            .expect("evidence");
        assert!(ev.contains("microservices/foo/runbooks/"));
        assert!(ev.contains("(6 runbooks)"));
    }

    #[test]
    fn resolve_renames_evidence_pattern_to_evidence() {
        let resolved = resolve_scorecard(
            "aws-well-architected",
            &canonical_aws(),
            &overrides_ok("c", "s"),
            "ms",
        )
        .expect("resolve");
        let ctrl = &resolved["pillars"]["operational_excellence"]["controls"][0];
        assert!(ctrl.get("evidence").is_some());
        assert!(ctrl.get("evidence_pattern").is_none());
    }

    #[test]
    fn resolve_drops_canonical_metadata() {
        let resolved = resolve_scorecard(
            "aws-well-architected",
            &canonical_aws(),
            &overrides_ok("c", "s"),
            "ms",
        )
        .expect("resolve");
        let obj = resolved.as_object().expect("object");
        assert!(!obj.contains_key("_placeholders"));
        assert!(!obj.contains_key("canonical_base"));
        assert!(!obj.contains_key("overlay_consumers"));
        assert_eq!(obj.get("microservice").and_then(|v| v.as_str()), Some("ms"));
    }

    #[test]
    fn resolve_unknown_framework_errors() {
        let err = resolve_scorecard(
            "made-up-framework",
            &canonical_aws(),
            &overrides_ok("c", "s"),
            "ms",
        )
        .expect_err("unknown framework");
        assert!(matches!(err, Error::UnknownFramework(_)));
    }

    #[test]
    fn resolve_applies_overall_status_override() {
        let mut over = overrides_ok("c", "s");
        over["aws_well_architected"]["overall_status"] = json!("yellow");
        let resolved = resolve_scorecard("aws-well-architected", &canonical_aws(), &over, "ms")
            .expect("resolve");
        assert_eq!(resolved["overall_status"], json!("yellow"));
    }

    #[test]
    fn resolve_prr_count_delta_writes_evidence_string() {
        let canonical = json!({
            "framework": "Google SRE PRR",
            "checklist": {
                "slos_defined": {"name": "x", "evidence": "see slos/"}
            }
        });
        let overrides = json!({
            "google_sre_prr": {
                "overall_status": "green",
                "deltas": [
                    {"control": "slos_defined", "field": "slo_count", "value": 5}
                ]
            }
        });
        let resolved =
            resolve_scorecard("google-sre-prr", &canonical, &overrides, "ms").expect("resolve");
        let ev = resolved["checklist"]["slos_defined"]["evidence"]
            .as_str()
            .expect("evidence string");
        assert!(ev.contains("count: 5"));
    }

    #[test]
    fn resolve_cis_categories_walks_controls() {
        let canonical = json!({
            "categories": {
                "control-plane": {
                    "controls": [
                        {"id": "5.1.1", "name": "x", "evidence_pattern": "foo", "passing": true}
                    ]
                }
            }
        });
        let overrides = json!({
            "cis_k8s_benchmark": {
                "deltas": [
                    {"control": "5.1.1", "field": "evidence_suffix", "value": "+ tighter"}
                ]
            }
        });
        let resolved =
            resolve_scorecard("cis-k8s-benchmark", &canonical, &overrides, "ms").expect("resolve");
        let ev = resolved["categories"]["control-plane"]["controls"][0]["evidence"]
            .as_str()
            .expect("ev");
        assert!(ev.contains("+ tighter"));
    }

    #[test]
    fn rollup_marks_yellow_when_any_ms_yellow() {
        let entries = vec![
            (
                "alpha".to_string(),
                json!({
                    "chart_name": "alpha-c",
                    "aws_well_architected": {"overall_status": "green"},
                    "google_sre_prr": {"overall_status": "green"},
                    "cis_k8s_benchmark": {"overall_status": "green"},
                    "slsa_l3": {"overall_status": "green"}
                }),
            ),
            (
                "beta".to_string(),
                json!({
                    "chart_name": "beta-c",
                    "aws_well_architected": {"overall_status": "yellow"},
                    "google_sre_prr": {"overall_status": "green"},
                    "cis_k8s_benchmark": {"overall_status": "green"},
                    "slsa_l3": {"overall_status": "green"}
                }),
            ),
        ];
        let rollup = build_rollup("2026-05-18", &entries);
        assert_eq!(rollup["aggregate_status"], json!("yellow"));
        assert_eq!(
            rollup["microservices"]["beta"]["aws_well_architected"],
            json!("yellow")
        );
    }

    #[test]
    fn rollup_all_green_marks_green() {
        let entries = vec![(
            "alpha".to_string(),
            json!({
                "chart_name": "alpha-c",
                "aws_well_architected": {"overall_status": "green"},
                "google_sre_prr": {"overall_status": "green"},
                "cis_k8s_benchmark": {"overall_status": "green"},
                "slsa_l3": {"overall_status": "green"}
            }),
        )];
        let rollup = build_rollup("2026-05-18", &entries);
        assert_eq!(rollup["aggregate_status"], json!("green"));
    }
}
