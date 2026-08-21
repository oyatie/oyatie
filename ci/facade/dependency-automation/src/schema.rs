use std::collections::{BTreeSet, HashSet};

use toml::Value;

use crate::CONFIG_PATH;
use crate::report::Finding;

pub(crate) fn validate_closed_schema(config: &Value, findings: &mut BTreeSet<Finding>) {
    let Some(root) = config.as_table() else {
        findings.insert(Finding::new(
            "DEP-AUTO-MALFORMED-CONFIG",
            CONFIG_PATH,
            "top-level TOML value must be a table",
        ));
        return;
    };

    check_keys(
        "",
        root.keys(),
        [
            "schema_version",
            "metadata",
            "automation",
            "rust",
            "supply_chain",
            "managed_file",
            "freshness",
        ],
        findings,
    );
    check_table(
        config,
        &["metadata"],
        ["purpose", "owner", "decision", "status"],
        findings,
    );
    check_table(
        config,
        &["automation"],
        [
            "engine",
            "changeset_transport",
            "github_actions",
            "external_bots",
            "merge_authority",
        ],
        findings,
    );
    check_table(
        config,
        &["rust"],
        [
            "channel",
            "pin",
            "update_policy",
            "drift_guard",
            "exclusions",
        ],
        findings,
    );
    check_table(
        config,
        &["supply_chain"],
        [
            "license_policy",
            "advisory_policy",
            "audit_policy",
            "stewardship_registry",
            "bot_gate",
        ],
        findings,
    );

    // Crate-dependency freshness (oyatie-gr1n): the sibling of the `[rust]` toolchain pin above.
    // `deps.toml` is a CLOSED schema by design, so a new section must be declared here before
    // it may appear in the file — the gate refused this section until this entry existed, which is
    // the contract working as intended.
    check_table(
        config,
        &["freshness"],
        [
            "mirror",
            "manifest",
            "producer",
            "kernel",
            "stale_after_days",
            "enforcement",
            "blocking_exception",
            "signals",
        ],
        findings,
    );

    if let Some(entries) = config.get("managed_file").and_then(Value::as_array) {
        for (idx, entry) in entries.iter().enumerate() {
            if let Some(table) = entry.as_table() {
                check_keys(
                    &format!("managed_file[{idx}]"),
                    table.keys(),
                    ["path", "role", "update", "reason"],
                    findings,
                );
            } else {
                findings.insert(Finding::new(
                    "DEP-AUTO-MALFORMED-CONFIG",
                    format!("{CONFIG_PATH}:managed_file[{idx}]"),
                    "managed_file entries must be TOML tables",
                ));
            }
        }
    }
}

fn check_table<const N: usize>(
    config: &Value,
    path: &[&str],
    allowed: [&'static str; N],
    findings: &mut BTreeSet<Finding>,
) {
    match value_at(config, path).and_then(Value::as_table) {
        Some(table) => check_keys(&path.join("."), table.keys(), allowed, findings),
        None => {
            findings.insert(Finding::new(
                "DEP-AUTO-MISSING-KEY",
                format!("{CONFIG_PATH}:{}", path.join(".")),
                "required table is missing or not a table",
            ));
        }
    };
}

fn check_keys<'a, I, const N: usize>(
    scope: &str,
    keys: I,
    allowed: [&'static str; N],
    findings: &mut BTreeSet<Finding>,
) where
    I: Iterator<Item = &'a String>,
{
    let allowed: HashSet<&str> = allowed.into_iter().collect();
    for key in keys {
        if !allowed.contains(key.as_str()) {
            let key_path = if scope.is_empty() {
                key.to_owned()
            } else {
                format!("{scope}.{key}")
            };
            findings.insert(Finding::new(
                "DEP-AUTO-UNKNOWN-KEY",
                format!("{CONFIG_PATH}:{key_path}"),
                "deps.toml is a closed-schema contract; add schema support before adding keys",
            ));
        }
    }
}

pub(crate) fn string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    value_at(value, path).and_then(Value::as_str)
}

pub(crate) fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_schema_rejects_unknown_top_level_key() {
        let config = r#"
schema_version = "1.0.0"
unexpected = true
[metadata]
purpose = "x"
owner = "x"
decision = "ADR-0535"
status = "accepted"
[automation]
engine = "owned-rust-bump-bot"
changeset_transport = "scm-facts"
github_actions = "adapter-only"
external_bots = "disabled"
merge_authority = "oya-ci-required"
[rust]
channel = "stable"
pin = "1.96.0"
update_policy = "latest-stable"
drift_guard = "x"
exclusions = []
[supply_chain]
license_policy = "deny.toml"
advisory_policy = "cargo-deny"
audit_policy = "cargo-vet"
stewardship_registry = "specs/oss-stewardship-registry.json"
bot_gate = "cloud-ci-dependency-automation"
"#
        .parse::<Value>()
        .unwrap();
        let mut findings = BTreeSet::new();
        validate_closed_schema(&config, &mut findings);
        assert!(
            findings
                .iter()
                .any(|finding| finding.code == "DEP-AUTO-UNKNOWN-KEY")
        );
    }
}
