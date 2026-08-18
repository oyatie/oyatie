use std::collections::BTreeSet;

use toml::Value;

use crate::CONFIG_PATH;
use crate::GATE_ID;
use crate::report::Finding;
use crate::schema::string_at;

const EXPECTED_SCHEMA_VERSION: &str = "1.0.0";
const EXPECTED_ENGINE: &str = "owned-rust-bump-bot";
const EXPECTED_CHANGESET_TRANSPORT: &str = "scm-facts";
const EXPECTED_EXTERNAL_BOTS: &str = "disabled";
const EXPECTED_RUST_POLICY: &str = "latest-stable";

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
