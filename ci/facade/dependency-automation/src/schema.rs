//! Closed-schema and policy-value checks for root `oya-deps.toml`.

use std::collections::{BTreeSet, HashSet};

use toml::Value;

use crate::{CONFIG_PATH, Finding, GATE_ID, string_at, value_at};

const EXPECTED_SCHEMA_VERSION: &str = "1.0.0";
const EXPECTED_ENGINE: &str = "owned-rust-bump-bot";
const EXPECTED_CHANGESET_TRANSPORT: &str = "scm-facts";
const EXPECTED_EXTERNAL_BOTS: &str = "disabled";
const EXPECTED_RUST_POLICY: &str = "latest-stable";

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
    // `oya-deps.toml` is a CLOSED schema by design, so a new section must be declared here before
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
                "oya-deps.toml is a closed-schema contract; add schema support before adding keys",
            ));
        }
    }
}

pub(crate) fn validate_policy_values(config: &Value, findings: &mut BTreeSet<Finding>) {
    expect_string(
        config,
        &["schema_version"],
        EXPECTED_SCHEMA_VERSION,
        "DEP-AUTO-SCHEMA-VERSION",
        "schema_version must match the gate contract",
        findings,
    );
    expect_string(
        config,
        &["metadata", "decision"],
        "ADR-0535",
        "DEP-AUTO-MISSING-ADR",
        "owned dependency automation must cite ADR-0535",
        findings,
    );
    expect_string(
        config,
        &["automation", "engine"],
        EXPECTED_ENGINE,
        "DEP-AUTO-NONOWNED-ENGINE",
        "dependency automation must use the owned Rust bump-bot engine",
        findings,
    );
    expect_string(
        config,
        &["automation", "changeset_transport"],
        EXPECTED_CHANGESET_TRANSPORT,
        "DEP-AUTO-NONOWNED-TRANSPORT",
        "dependency automation must emit provider-neutral scm-facts ChangeSets",
        findings,
    );
    expect_string(
        config,
        &["automation", "external_bots"],
        EXPECTED_EXTERNAL_BOTS,
        "DEP-AUTO-EXTERNAL-BOTS-ENABLED",
        "external dependency bots stay disabled for the owned stack",
        findings,
    );
    expect_string(
        config,
        &["automation", "merge_authority"],
        "oya-ci-required",
        "DEP-AUTO-MERGE-AUTHORITY",
        "dependency updates must still merge through the single required context",
        findings,
    );
    expect_string(
        config,
        &["rust", "channel"],
        "stable",
        "DEP-AUTO-RUST-CHANNEL",
        "root workspace follows the stable Rust channel",
        findings,
    );
    expect_string(
        config,
        &["rust", "update_policy"],
        EXPECTED_RUST_POLICY,
        "DEP-AUTO-RUST-UPDATE-POLICY",
        "Rust updates should track the latest stable release",
        findings,
    );
    expect_string(
        config,
        &["supply_chain", "bot_gate"],
        GATE_ID,
        "DEP-AUTO-BOT-GATE",
        "supply-chain policy must name this enforcement gate",
        findings,
    );
}

fn expect_string(
    config: &Value,
    path: &[&str],
    expected: &str,
    code: &'static str,
    detail: &str,
    findings: &mut BTreeSet<Finding>,
) {
    match string_at(config, path) {
        Some(actual) if actual == expected => {}
        Some(actual) => {
            findings.insert(Finding::new(
                code,
                format!("{CONFIG_PATH}:{}", path.join(".")),
                format!("{detail}: expected {expected:?}, got {actual:?}"),
            ));
        }
        None => {
            findings.insert(Finding::new(
                "DEP-AUTO-MISSING-KEY",
                format!("{CONFIG_PATH}:{}", path.join(".")),
                format!("missing required string; {detail}"),
            ));
        }
    }
}
