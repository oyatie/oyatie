//! `oya gate validate platform-substrate-defaults` runner.
//!
//! This is a local source-of-truth guard for launch substrate vocabulary. It
//! enforces the 2026-05-25 directive that Citus, OpenSearch, Milvus,
//! ClickHouse, and Iceberg are workload-specific selections, not universal
//! defaults for every Oyatie microservice or cell.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

const GATE_NAME: &str = "platform-substrate-defaults";
const DEFAULT_ARCHITECTURE: &str = "specs/platform-architecture.json";
const CLASSIFICATION: &str = "workload-specific-not-universal-default";

const REQUIRED_SUBSTRATES: &[(&str, &str)] = &[
    ("citus", "Citus"),
    ("opensearch", "OpenSearch"),
    ("milvus", "Milvus"),
    ("clickhouse", "ClickHouse"),
    ("iceberg", "Iceberg"),
];

const FORBIDDEN_UNIVERSAL_DEFAULTS: &[(&str, &str)] = &[
    (
        "/platform/data_substrate/primary_oltp_store",
        "Postgres + Citus (sharded multi-tenant)",
    ),
    (
        "/platform/data_substrate/search_substrate",
        "OpenSearch (per cell)",
    ),
    ("/platform/data_substrate/vector_store", "Milvus"),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlatformSubstrateDefaultsArgs {
    pub(crate) architecture: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlatformSubstrateDefaultsReport {
    pub(crate) architecture_path: String,
    pub(crate) workload_specific_substrates_checked: usize,
    pub(crate) universal_default_fields_checked: usize,
}

pub(crate) fn parse_platform_substrate_defaults_args(
    args: Vec<String>,
) -> Result<PlatformSubstrateDefaultsArgs, String> {
    let mut parsed = PlatformSubstrateDefaultsArgs {
        architecture: PathBuf::from(DEFAULT_ARCHITECTURE),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--architecture" => {
                parsed.architecture = iter
                    .next()
                    .map(PathBuf::from)
                    .ok_or_else(|| format!("{GATE_NAME}: --architecture requires a path"))?;
            }
            other => {
                return Err(format!(
                    "{GATE_NAME}: unknown flag {other:?}; usage: \
                     oya gate validate {GATE_NAME} [--architecture <{DEFAULT_ARCHITECTURE}>]"
                ));
            }
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_platform_substrate_defaults_gate(
    args: PlatformSubstrateDefaultsArgs,
) -> Result<PlatformSubstrateDefaultsReport, String> {
    let architecture = read_json(&args.architecture)?;
    let mut issues = Vec::new();
    let Some(data_substrate) = architecture.pointer("/platform/data_substrate") else {
        issues.push("platform.data_substrate is required".to_string());
        return Err(format_issues(&issues));
    };

    validate_default_policy(data_substrate, &mut issues);
    validate_required_substrate_rows(data_substrate, &mut issues);
    validate_universal_default_fields(data_substrate, &mut issues);

    if !issues.is_empty() {
        return Err(format_issues(&issues));
    }

    Ok(PlatformSubstrateDefaultsReport {
        architecture_path: slash_path(&args.architecture),
        workload_specific_substrates_checked: REQUIRED_SUBSTRATES.len(),
        universal_default_fields_checked: FORBIDDEN_UNIVERSAL_DEFAULTS.len(),
    })
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("{GATE_NAME}: unable to read {}: {error}", slash_path(path)))?;
    serde_json::from_str(&text).map_err(|error| {
        format!(
            "{GATE_NAME}: {} is not valid JSON: {error}",
            slash_path(path)
        )
    })
}

fn validate_default_policy(data_substrate: &Value, issues: &mut Vec<String>) {
    let policy = data_substrate.get("default_policy");
    if json_bool(policy, "universal_data_substrate_default_forbidden") != Some(true) {
        issues.push(
            "platform.data_substrate.default_policy.universal_data_substrate_default_forbidden must be true"
                .to_string(),
        );
    }
    if json_bool(policy, "workload_selection_required") != Some(true) {
        issues.push(
            "platform.data_substrate.default_policy.workload_selection_required must be true"
                .to_string(),
        );
    }
}

fn validate_required_substrate_rows(data_substrate: &Value, issues: &mut Vec<String>) {
    let rows = data_substrate.get("workload_specific_substrates");
    for (key, display_name) in REQUIRED_SUBSTRATES {
        let Some(row) = rows.and_then(|rows| rows.get(*key)) else {
            issues.push(format!(
                "platform.data_substrate.workload_specific_substrates.{key} is required"
            ));
            continue;
        };
        if row.get("name").and_then(Value::as_str) != Some(*display_name) {
            issues.push(format!(
                "workload_specific_substrates.{key}.name must be {display_name}"
            ));
        }
        if row.get("classification").and_then(Value::as_str) != Some(CLASSIFICATION) {
            issues.push(format!(
                "workload_specific_substrates.{key}.classification must be {CLASSIFICATION}"
            ));
        }
        if row.get("universal_default").and_then(Value::as_bool) != Some(false) {
            issues.push(format!(
                "workload_specific_substrates.{key}.universal_default must be false"
            ));
        }
        if row.get("selection_required").and_then(Value::as_bool) != Some(true) {
            issues.push(format!(
                "workload_specific_substrates.{key}.selection_required must be true"
            ));
        }
        if !non_empty_array(row.get("allowed_when")) {
            issues.push(format!(
                "workload_specific_substrates.{key}.allowed_when must be a non-empty array"
            ));
        }
    }
}

fn validate_universal_default_fields(data_substrate: &Value, issues: &mut Vec<String>) {
    for (pointer, forbidden_value) in FORBIDDEN_UNIVERSAL_DEFAULTS {
        let Some(value) = data_substrate
            .pointer(pointer.trim_start_matches("/platform/data_substrate"))
            .and_then(Value::as_str)
        else {
            issues.push(format!(
                "{pointer} must remain present with workload-selected wording"
            ));
            continue;
        };
        if value == *forbidden_value {
            issues.push(format!(
                "{pointer} must not retain universal default value {forbidden_value:?}"
            ));
        }
        let lower = value.to_ascii_lowercase();
        if !(lower.contains("workload-selected") || lower.contains("workload-specific")) {
            issues.push(format!(
                "{pointer} must state workload-selected or workload-specific semantics"
            ));
        }
    }
}

fn json_bool(value: Option<&Value>, key: &str) -> Option<bool> {
    value
        .and_then(|value| value.get(key))
        .and_then(Value::as_bool)
}

fn non_empty_array(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_array)
        .map(|items| !items.is_empty() && items.iter().all(|item| item.as_str().is_some()))
        .unwrap_or(false)
}

fn format_issues(issues: &[String]) -> String {
    format!("{GATE_NAME} validation failed:\n- {}", issues.join("\n- "))
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempSpec {
        root: PathBuf,
        architecture: PathBuf,
    }

    impl TempSpec {
        fn new(name: &str, body: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock after unix epoch")
                .as_nanos();
            let root =
                std::env::temp_dir().join(format!("oya-{name}-{}-{nonce}", std::process::id()));
            fs::create_dir_all(&root).expect("create temp root");
            let architecture = root.join("platform-architecture.json");
            fs::write(&architecture, body).expect("write platform architecture");
            Self { root, architecture }
        }

        fn args(&self) -> PlatformSubstrateDefaultsArgs {
            PlatformSubstrateDefaultsArgs {
                architecture: self.architecture.clone(),
            }
        }
    }

    impl Drop for TempSpec {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn parse_platform_substrate_defaults_defaults_to_live_spec() {
        let parsed = parse_platform_substrate_defaults_args(Vec::new()).expect("defaults");
        assert_eq!(parsed.architecture, PathBuf::from(DEFAULT_ARCHITECTURE));
    }

    #[test]
    fn parse_platform_substrate_defaults_rejects_unknown_flag() {
        let error = parse_platform_substrate_defaults_args(vec!["--bogus".to_string()])
            .expect_err("unknown flag rejected");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn platform_substrate_defaults_accepts_workload_specific_rows() {
        let temp = TempSpec::new("platform-substrate-defaults-valid", &valid_architecture());
        let report = validate_platform_substrate_defaults_gate(temp.args()).expect("valid");
        assert_eq!(report.workload_specific_substrates_checked, 5);
        assert_eq!(report.universal_default_fields_checked, 3);
    }

    #[test]
    fn platform_substrate_defaults_rejects_old_universal_defaults() {
        let body = valid_architecture().replace(
            "workload-selected relational store; Postgres baseline, Citus only when tenant-sharded horizontal OLTP requires it",
            "Postgres + Citus (sharded multi-tenant)",
        );
        let temp = TempSpec::new("platform-substrate-defaults-old-default", &body);
        let error = validate_platform_substrate_defaults_gate(temp.args()).expect_err("rejected");
        assert!(error.contains("universal default value"));
    }

    #[test]
    fn platform_substrate_defaults_rejects_missing_substrate_row() {
        let body = valid_architecture().replace(
            r#",
        "iceberg": {"name":"Iceberg","classification":"workload-specific-not-universal-default","universal_default":false,"selection_required":true,"allowed_when":["OLAP table format or lakehouse write path is selected for the workload"]}"#,
            "",
        );
        let temp = TempSpec::new("platform-substrate-defaults-missing-row", &body);
        let error = validate_platform_substrate_defaults_gate(temp.args()).expect_err("rejected");
        assert!(error.contains("workload_specific_substrates.iceberg is required"));
    }

    fn valid_architecture() -> String {
        r#"{
  "platform": {
    "data_substrate": {
      "default_policy": {
        "universal_data_substrate_default_forbidden": true,
        "workload_selection_required": true
      },
      "primary_oltp_store": "workload-selected relational store; Postgres baseline, Citus only when tenant-sharded horizontal OLTP requires it",
      "search_substrate": "workload-selected search substrate; OpenSearch only when search or log-index workloads require it",
      "vector_store": "workload-selected vector substrate; Milvus only when high-scale vector or RAG workloads require it",
      "workload_specific_substrates": {
        "citus": {"name":"Citus","classification":"workload-specific-not-universal-default","universal_default":false,"selection_required":true,"allowed_when":["tenant-sharded horizontal OLTP is selected for the workload"]},
        "opensearch": {"name":"OpenSearch","classification":"workload-specific-not-universal-default","universal_default":false,"selection_required":true,"allowed_when":["search or log-index workloads require OpenSearch"]},
        "milvus": {"name":"Milvus","classification":"workload-specific-not-universal-default","universal_default":false,"selection_required":true,"allowed_when":["high-scale vector or RAG workloads require Milvus"]},
        "clickhouse": {"name":"ClickHouse","classification":"workload-specific-not-universal-default","universal_default":false,"selection_required":true,"allowed_when":["OLAP serving or analytics workloads require ClickHouse"]},
        "iceberg": {"name":"Iceberg","classification":"workload-specific-not-universal-default","universal_default":false,"selection_required":true,"allowed_when":["OLAP table format or lakehouse write path is selected for the workload"]}
      }
    }
  }
}"#
        .to_string()
    }
}
