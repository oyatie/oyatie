//! Validate AC-0.1/P0.6/AC-0.7 service-root classifier seed evidence.
//!
//! This checker is local/static Buck2 evidence only. It proves that the
//! checked-in service inventory, structural packet catalog, and RED/GREEN
//! fixtures fail closed for closed-world root drift, legacy service sprawl,
//! retired REAL status tokens, duplicate services across roots, and underscore
//! crate names. It never posts statuses, mutates branch protection, proves
//! post-migration pure split, proves a full nested crate inventory, or claims
//! P0.0/Phase-0 completion.

#[allow(dead_code)]
#[path = "assert-result-bundle-output.rs"]
mod json_support;

pub use json_support::Json;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_INVENTORY: &str = "specs/service-inventory.json";
const DEFAULT_PACKETS: &str = "specs/phase0-structural-packets.json";
const DEFAULT_FIXTURE_DIR: &str = "specs/fixtures/phase0-service-root-classifier";
const AUTHORITY_BOUNDARY: &str = "AC-0.1/P0.6/AC-0.7 local/static service-root classifier evidence only; no status mutation, live required-context authority, post-migration pure split, P0.0 green, Phase-0 completion, production readiness, or hyperscaler-grade readiness proven";

const CLOSED_WORLD_ROOTS: [&str; 8] = [
    "oya",
    "cloud",
    "services",
    "platforms",
    "packs",
    "regional-packs",
    "libs",
    "microservices",
];
const CANONICAL_SERVICE_ROOTS: [&str; 2] = ["oya", "cloud"];
const LEGACY_SERVICE_ROOTS: [&str; 3] = ["services", "platforms", "microservices"];
const PACK_ROOTS: [&str; 2] = ["packs", "regional-packs"];
const TARGET_TREES: [&str; 3] = ["oya", "cloud", "libs"];
const FALSE_CLAIMS: [&str; 10] = [
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "full_service_inventory_coverage_proven",
    "post_migration_pure_split_proven",
    "structural_shards_executed",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];
const FIXTURE_FALSE_CLAIMS: [&str; 7] = [
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];

pub const REQUIRED_ENTRY_FIELDS: [&str; 12] = [
    "non_test_loc",
    "has_main_rs",
    "has_real_storage_adapter",
    "authoritative_toolchain",
    "builds_green",
    "tests_pass",
    "orphan_test_count",
    "cargo_red_set",
    "source_path",
    "target_path",
    "target_tree",
    "migration_class",
];

pub const REQUIRED_PACKET_FIELDS: [&str; 13] = [
    "packet_id",
    "sub_item",
    "owner_lane",
    "source_paths",
    "target_paths",
    "structural_path_set",
    "depends_on",
    "max_scope",
    "acceptance",
    "verification_commands",
    "rollback_inverse",
    "evidence_bundle",
    "trunk_checkpoint",
];

const REQUIRED_PACKET_FAMILIES: [&str; 6] = [
    "P0.6a-GC-",
    "P0.6b-SPLIT-oya-",
    "P0.6b-SPLIT-cloud-",
    "P0.6b-SPLIT-libs-",
    "P0.6c-ADR0131-",
    "P0.6d-BNF-",
];
const SKIP_DIRS: [&str; 8] = [
    ".git",
    "buck-out",
    "target",
    "node_modules",
    ".next",
    "dist",
    "build",
    "__pycache__",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventorySummary {
    pub closed_world_root_count: usize,
    pub inventory_entry_count: usize,
    pub observed_direct_child_dir_count: usize,
    pub missing_inventory_entry_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketSummary {
    pub structural_packet_count: usize,
    pub required_packet_family_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureValidation {
    pub path: String,
    pub fixture_id: String,
    pub expected_verdict: String,
    pub expected_violations: Vec<String>,
    pub observed_violations: Vec<String>,
    pub fixture_passed: bool,
    pub failures: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub authority_boundary: String,
    pub service_inventory_published: bool,
    pub service_root_classifier_measured: bool,
    pub closed_world_root_classifier_measured: bool,
    pub structural_packet_catalog_published: bool,
    pub inventory_summary: InventorySummary,
    pub packet_summary: PacketSummary,
    pub fixture_count: usize,
    pub expected_green_fixture_count: usize,
    pub expected_red_fixture_count: usize,
    pub fixture_results: Vec<FixtureValidation>,
    pub status_mutation_performed: bool,
    pub protected_branch_authority_proven: bool,
    pub live_required_context_execution_proven: bool,
    pub full_service_inventory_coverage_proven: bool,
    pub post_migration_pure_split_proven: bool,
    pub structural_shards_executed: bool,
    pub p0_0_green: bool,
    pub phase0_complete: bool,
    pub production_ready: bool,
    pub hyperscaler_grade: bool,
    pub verdict: String,
    pub failures: Vec<String>,
}

pub fn parse_json(text: &str) -> Result<Json, String> {
    json_support::parse_json(text)
}

pub fn load_json(path: &Path) -> Result<Json, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("read {} failed: {error}", path.display()))?;
    parse_json(&text).map_err(|error| format!("parse {} failed: {error}", path.display()))
}

fn object_field<'a>(object: &'a BTreeMap<String, Json>, key: &str) -> Option<&'a Json> {
    object.get(key)
}

fn string_field(object: &BTreeMap<String, Json>, key: &str) -> Option<String> {
    object_field(object, key)
        .and_then(Json::as_str)
        .map(str::to_string)
}

fn bool_field(object: &BTreeMap<String, Json>, key: &str) -> Option<bool> {
    object_field(object, key).and_then(Json::as_bool)
}

fn int_field(object: &BTreeMap<String, Json>, key: &str) -> Option<i64> {
    object_field(object, key).and_then(Json::as_i64)
}

fn string_list(value: Option<&Json>) -> Vec<String> {
    value
        .and_then(Json::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Json::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn object_list(value: Option<&Json>) -> Vec<&BTreeMap<String, Json>> {
    value
        .and_then(Json::as_array)
        .map(|items| items.iter().filter_map(Json::as_object).collect())
        .unwrap_or_default()
}

fn set_from(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn sorted_unique(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn display_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

fn root_of(path_value: &str) -> &str {
    path_value
        .split_once('/')
        .map_or(path_value, |(root, _)| root)
}

fn skipped(path: &Path) -> bool {
    path.components().any(|part| {
        let text = part.as_os_str().to_string_lossy();
        SKIP_DIRS.iter().any(|skip| *skip == text)
    })
}

fn observed_direct_child_dirs(root: &Path) -> Vec<String> {
    let mut observed = Vec::new();
    for root_name in CLOSED_WORLD_ROOTS {
        let base = root.join(root_name);
        let Ok(children) = fs::read_dir(&base) else {
            continue;
        };
        let mut children = children
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        children.sort();
        for child in children {
            let Some(name) = child.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if child.is_dir() && !name.starts_with('.') && !skipped(&child) {
                observed.push(display_path(&child, root));
            }
        }
    }
    observed.sort();
    observed
}

fn validate_false_claims(
    mapping: Option<&BTreeMap<String, Json>>,
    failures: &mut Vec<String>,
    claims: &[&str],
    prefix: &str,
) {
    let empty = BTreeMap::new();
    let mapping = mapping.unwrap_or(&empty);
    for claim in claims {
        if bool_field(mapping, claim) != Some(false) {
            failures.push(format!("{prefix}forbidden_true_or_missing_claim_{claim}"));
        }
    }
}

fn contains_real_token(text: &str) -> bool {
    let chars = text.chars().collect::<Vec<_>>();
    for start in 0..chars.len() {
        if chars.get(start..start + 4) == Some(&['R', 'E', 'A', 'L']) {
            let before = start
                .checked_sub(1)
                .and_then(|index| chars.get(index))
                .copied();
            let after = chars.get(start + 4).copied();
            let boundary = |ch: Option<char>| {
                !ch.is_some_and(|value| value.is_ascii_alphanumeric() || value == '_')
            };
            if boundary(before) && boundary(after) {
                return true;
            }
        }
    }
    false
}

fn iter_status_maturity_fields(
    value: &Json,
    key_path: &mut Vec<String>,
    out: &mut Vec<(String, String)>,
) {
    match value {
        Json::Object(object) => {
            for (key, child) in object {
                key_path.push(key.clone());
                let lowered = key.to_lowercase();
                if let Json::String(text) = child {
                    if lowered.contains("status") || lowered.contains("maturity") {
                        out.push((key_path.join("."), text.clone()));
                    }
                }
                iter_status_maturity_fields(child, key_path, out);
                key_path.pop();
            }
        }
        Json::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                key_path.push(index.to_string());
                iter_status_maturity_fields(child, key_path, out);
                key_path.pop();
            }
        }
        _ => {}
    }
}

fn validate_real_status_tokens(value: Option<&Json>, failures: &mut Vec<String>, prefix: &str) {
    let Some(value) = value else {
        return;
    };
    let mut found = Vec::new();
    iter_status_maturity_fields(value, &mut Vec::new(), &mut found);
    for (key_path, text) in found {
        if contains_real_token(&text) {
            failures.push(format!("{prefix}retired_real_token_live_field"));
            failures.push(format!("{prefix}retired_real_token_live_field:{key_path}"));
        }
    }
}

fn validate_inventory(root: &Path, inventory: &Json) -> (Vec<String>, InventorySummary) {
    let mut failures = Vec::new();
    let Some(inventory_object) = inventory.as_object() else {
        failures.push("service_inventory_must_be_object".to_string());
        return (
            failures,
            InventorySummary {
                closed_world_root_count: 0,
                inventory_entry_count: 0,
                observed_direct_child_dir_count: 0,
                missing_inventory_entry_count: 0,
            },
        );
    };
    let boundary = object_field(inventory_object, "claim_boundary").and_then(Json::as_object);
    if boundary.and_then(|boundary| bool_field(boundary, "service_inventory_published"))
        != Some(true)
    {
        failures.push("service_inventory_not_published".to_string());
    }
    if boundary.and_then(|boundary| bool_field(boundary, "service_root_classifier_measured"))
        != Some(true)
    {
        failures.push("service_root_classifier_not_measured".to_string());
    }
    if boundary.and_then(|boundary| bool_field(boundary, "closed_world_root_classifier_measured"))
        != Some(true)
    {
        failures.push("closed_world_root_classifier_not_measured".to_string());
    }
    validate_false_claims(boundary, &mut failures, &FALSE_CLAIMS, "");
    validate_real_status_tokens(Some(inventory), &mut failures, "");

    let required_fields = string_list(object_field(inventory_object, "required_entry_fields"));
    let required_fields = if required_fields.is_empty() {
        set_from(&REQUIRED_ENTRY_FIELDS)
    } else {
        required_fields.into_iter().collect()
    };
    if required_fields != set_from(&REQUIRED_ENTRY_FIELDS) {
        failures.push("required_entry_fields_drift".to_string());
    }

    let roots = object_list(object_field(inventory_object, "closed_world_roots"));
    let root_names = roots
        .iter()
        .filter_map(|item| string_field(item, "root"))
        .collect::<BTreeSet<_>>();
    for missing in set_from(&CLOSED_WORLD_ROOTS).difference(&root_names) {
        failures.push(format!("closed_world_root_missing:{missing}"));
    }
    for extra in root_names.difference(&set_from(&CLOSED_WORLD_ROOTS)) {
        failures.push(format!("closed_world_root_unexpected:{extra}"));
    }
    for item in &roots {
        let Some(root_name) = string_field(item, "root") else {
            failures.push("closed_world_root_missing_name".to_string());
            continue;
        };
        if !TARGET_TREES
            .iter()
            .any(|target| Some(*target) == string_field(item, "target_tree").as_deref())
        {
            failures.push(format!("closed_world_root_invalid_target_tree:{root_name}"));
        }
        if CANONICAL_SERVICE_ROOTS.contains(&root_name.as_str())
            && bool_field(item, "allows_new_service_dirs") != Some(true)
        {
            failures.push(format!("canonical_service_root_not_allowed:{root_name}"));
        }
        if LEGACY_SERVICE_ROOTS.contains(&root_name.as_str())
            && !matches!(
                int_field(item, "observed_direct_child_dir_count"),
                Some(0) | None
            )
        {
            failures.push(format!("legacy_service_root_not_empty:{root_name}"));
        }
        if PACK_ROOTS.contains(&root_name.as_str())
            && bool_field(item, "allows_new_service_dirs") != Some(false)
        {
            failures.push(format!("pack_root_allows_service_dirs:{root_name}"));
        }
    }

    let entries = object_list(object_field(inventory_object, "inventory_entries"));
    let mut entry_paths = BTreeSet::new();
    let mut counts = BTreeMap::<String, usize>::new();
    for entry in &entries {
        let source_path = string_field(entry, "source_path")
            .unwrap_or_else(|| "<missing-source-path>".to_string());
        *counts.entry(source_path.clone()).or_default() += 1;
        for field in REQUIRED_ENTRY_FIELDS {
            if !entry.contains_key(field) {
                failures.push(format!(
                    "{source_path}:missing_required_entry_field:{field}"
                ));
            }
        }
        if source_path == "<missing-source-path>" {
            failures.push("entry_source_path_missing".to_string());
            continue;
        }
        entry_paths.insert(source_path.clone());
        let entry_root = root_of(&source_path);
        if !CLOSED_WORLD_ROOTS.contains(&entry_root) {
            failures.push(format!("service_root_outside_closed_world:{source_path}"));
        }
        if LEGACY_SERVICE_ROOTS.contains(&entry_root) {
            failures.push(format!("service_layout_sprawl:{source_path}"));
        }
        if !root.join(&source_path).is_dir() {
            failures.push(format!("inventory_source_path_missing:{source_path}"));
        }
        if !TARGET_TREES
            .iter()
            .any(|target| Some(*target) == string_field(entry, "target_tree").as_deref())
        {
            failures.push(format!("{source_path}:invalid_target_tree"));
        }
        if !matches!(int_field(entry, "non_test_loc"), Some(value) if value >= 0) {
            failures.push(format!("{source_path}:invalid_non_test_loc"));
        }
        for bool_name in [
            "has_main_rs",
            "has_real_storage_adapter",
            "builds_green",
            "tests_pass",
        ] {
            if bool_field(entry, bool_name).is_none() {
                failures.push(format!("{source_path}:invalid_bool_field:{bool_name}"));
            }
        }
        if bool_field(entry, "builds_green") == Some(true) {
            failures.push(format!(
                "{source_path}:entry_claims_builds_green_without_live_context"
            ));
        }
        if bool_field(entry, "tests_pass") == Some(true) {
            failures.push(format!(
                "{source_path}:entry_claims_tests_pass_without_live_context"
            ));
        }
        if !matches!(int_field(entry, "orphan_test_count"), Some(value) if value >= 0) {
            failures.push(format!("{source_path}:invalid_orphan_test_count"));
        }
        if !matches!(object_field(entry, "cargo_red_set"), Some(Json::Array(_))) {
            failures.push(format!("{source_path}:invalid_cargo_red_set"));
        }
        if Path::new(&source_path)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.contains('_'))
        {
            failures.push(format!("underscore_crate_name:{source_path}"));
        }
    }
    for (path, count) in counts {
        if path != "<missing-source-path>" && count > 1 {
            failures.push(format!("duplicate_inventory_source_path:{path}"));
        }
    }

    let observed = observed_direct_child_dirs(root)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let missing_entries = observed
        .difference(&entry_paths)
        .cloned()
        .collect::<Vec<_>>();
    let extra_entries = entry_paths
        .difference(&observed)
        .cloned()
        .collect::<Vec<_>>();
    for path in &missing_entries {
        failures.push(format!("service_inventory_entry_missing:{path}"));
    }
    for path in extra_entries {
        failures.push(format!(
            "service_inventory_entry_without_observed_path:{path}"
        ));
    }

    (
        failures,
        InventorySummary {
            closed_world_root_count: root_names.len(),
            inventory_entry_count: entries.len(),
            observed_direct_child_dir_count: observed.len(),
            missing_inventory_entry_count: missing_entries.len(),
        },
    )
}

fn validate_packets(
    packets_spec: Option<&Json>,
    failures: &mut Vec<String>,
    packets_override: Option<Vec<&BTreeMap<String, Json>>>,
    prefix: &str,
) -> PacketSummary {
    let (required_fields, required_families, packets) = if let Some(packets) = packets_override {
        (
            set_from(&REQUIRED_PACKET_FIELDS),
            set_from(&REQUIRED_PACKET_FAMILIES),
            packets,
        )
    } else {
        let packets_spec = packets_spec.and_then(Json::as_object);
        let boundary = packets_spec
            .and_then(|object| object_field(object, "claim_boundary"))
            .and_then(Json::as_object);
        if boundary.and_then(|boundary| bool_field(boundary, "structural_packet_catalog_published"))
            != Some(true)
        {
            failures.push(format!("{prefix}structural_packet_catalog_not_published"));
        }
        if boundary.and_then(|boundary| bool_field(boundary, "service_root_classifier_measured"))
            != Some(true)
        {
            failures.push(format!(
                "{prefix}service_root_classifier_not_measured_in_packets"
            ));
        }
        validate_false_claims(boundary, failures, &FALSE_CLAIMS, prefix);
        let required_fields = packets_spec
            .map(|object| string_list(object_field(object, "required_packet_fields")))
            .unwrap_or_default();
        let required_families = packets_spec
            .map(|object| string_list(object_field(object, "required_packet_families")))
            .unwrap_or_default();
        let packets = packets_spec
            .map(|object| object_list(object_field(object, "structural_packets")))
            .unwrap_or_default();
        (
            if required_fields.is_empty() {
                set_from(&REQUIRED_PACKET_FIELDS)
            } else {
                required_fields.into_iter().collect()
            },
            if required_families.is_empty() {
                set_from(&REQUIRED_PACKET_FAMILIES)
            } else {
                required_families.into_iter().collect()
            },
            packets,
        )
    };

    if required_fields != set_from(&REQUIRED_PACKET_FIELDS) {
        failures.push(format!("{prefix}required_packet_fields_drift"));
    }

    let mut packet_ids = Vec::new();
    for packet in &packets {
        let packet_id =
            string_field(packet, "packet_id").unwrap_or_else(|| "<missing-packet-id>".to_string());
        packet_ids.push(packet_id.clone());
        for field in REQUIRED_PACKET_FIELDS {
            if !packet.contains_key(field) {
                failures.push(format!(
                    "{prefix}structural_packet_missing_required_field:{packet_id}:{field}"
                ));
            }
        }
        for field in [
            "source_paths",
            "target_paths",
            "structural_path_set",
            "depends_on",
            "verification_commands",
        ] {
            if string_list(object_field(packet, field)).is_empty() {
                failures.push(format!(
                    "{prefix}structural_packet_empty_list_field:{packet_id}:{field}"
                ));
            }
        }
        let commands = string_list(object_field(packet, "verification_commands")).join("\n");
        if !commands.contains("//:service-root-classifier-check") {
            failures.push(format!(
                "{prefix}structural_packet_missing_classifier_command:{packet_id}"
            ));
        }
        let commands_lower = commands.to_lowercase();
        if commands_lower.contains("oya verify")
            || commands_lower.contains("oya gate")
            || commands_lower.contains("bin/oya verify")
            || commands_lower.contains("bin/oya gate")
        {
            failures.push(format!(
                "{prefix}structural_packet_maps_to_oya_cli:{packet_id}"
            ));
        }
        for text_field in [
            "rollback_inverse",
            "trunk_checkpoint",
            "acceptance",
            "max_scope",
            "evidence_bundle",
        ] {
            if !string_field(packet, text_field).is_some_and(|text| !text.trim().is_empty()) {
                failures.push(format!(
                    "{prefix}structural_packet_missing_text_field:{packet_id}:{text_field}"
                ));
            }
        }
    }

    let mut id_counts = BTreeMap::<String, usize>::new();
    for packet_id in &packet_ids {
        *id_counts.entry(packet_id.clone()).or_default() += 1;
    }
    for (packet_id, count) in id_counts {
        if packet_id != "<missing-packet-id>" && count > 1 {
            failures.push(format!(
                "{prefix}duplicate_structural_packet_id:{packet_id}"
            ));
        }
    }
    for family in &required_families {
        if !packet_ids
            .iter()
            .any(|packet_id| packet_id.starts_with(family))
        {
            failures.push(format!("{prefix}structural_packet_missing_required_family"));
            failures.push(format!(
                "{prefix}structural_packet_missing_required_family:{family}"
            ));
        }
    }

    PacketSummary {
        structural_packet_count: packets.len(),
        required_packet_family_count: required_families.len(),
    }
}

fn validate_fixture(
    path: &str,
    fixture: &Json,
    inventory_roots: &BTreeSet<String>,
) -> FixtureValidation {
    let Some(fixture_object) = fixture.as_object() else {
        return FixtureValidation {
            path: path.to_string(),
            fixture_id: "<missing-fixture-id>".to_string(),
            expected_verdict: "RED".to_string(),
            expected_violations: Vec::new(),
            observed_violations: vec!["fixture_must_be_json_object".to_string()],
            fixture_passed: false,
            failures: vec![format!("{path}: fixture must be a JSON object")],
        };
    };
    let fixture_id = string_field(fixture_object, "fixture_id")
        .unwrap_or_else(|| "<missing-fixture-id>".to_string());
    let expected_verdict = match string_field(fixture_object, "expected_verdict").as_deref() {
        Some("GREEN") => "GREEN".to_string(),
        Some("RED") => "RED".to_string(),
        _ => "RED".to_string(),
    };
    let expected_violations = string_list(object_field(fixture_object, "expected_violations"))
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut observed = Vec::new();
    let boundary = object_field(fixture_object, "claim_boundary").and_then(Json::as_object);
    validate_false_claims(boundary, &mut observed, &FIXTURE_FALSE_CLAIMS, "");
    validate_real_status_tokens(
        object_field(fixture_object, "live_status_fields"),
        &mut observed,
        "",
    );

    let candidates = object_list(object_field(fixture_object, "candidate_paths"));
    let inventory_entry_paths = string_list(object_field(fixture_object, "inventory_entry_paths"))
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut service_roots_by_id = BTreeMap::<String, BTreeSet<String>>::new();
    for candidate in candidates {
        let path_value =
            string_field(candidate, "path").unwrap_or_else(|| "<missing-path>".to_string());
        let kind = string_field(candidate, "kind").unwrap_or_else(|| "service".to_string());
        let candidate_root = root_of(&path_value).to_string();
        if !inventory_roots.contains(&candidate_root) {
            observed.push("service_root_outside_closed_world".to_string());
        }
        if !inventory_entry_paths.contains(&path_value) {
            observed.push("service_inventory_entry_missing".to_string());
        }
        if kind == "service" && !CANONICAL_SERVICE_ROOTS.contains(&candidate_root.as_str()) {
            observed.push("service_layout_sprawl".to_string());
        }
        if kind == "pack" && !PACK_ROOTS.contains(&candidate_root.as_str()) {
            observed.push("service_layout_sprawl".to_string());
        }
        let crate_name = string_field(candidate, "crate_name").unwrap_or_else(|| {
            Path::new(&path_value)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("<missing-path>")
                .to_string()
        });
        if crate_name.contains('_')
            || Path::new(&path_value)
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains('_'))
        {
            observed.push("underscore_crate_name".to_string());
        }
        let service_id = string_field(candidate, "service_id").unwrap_or_else(|| {
            Path::new(&path_value)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("<missing-path>")
                .to_string()
        });
        service_roots_by_id
            .entry(service_id)
            .or_default()
            .insert(candidate_root);
    }

    for roots in service_roots_by_id.values() {
        if roots.len() > 1 {
            observed.push("duplicate_service_across_roots".to_string());
        }
    }

    if fixture_object.contains_key("structural_packets") {
        validate_packets(
            None,
            &mut observed,
            Some(object_list(object_field(
                fixture_object,
                "structural_packets",
            ))),
            "",
        );
    }

    let observed_set = observed.into_iter().collect::<BTreeSet<_>>();
    let mut fixture_failures = Vec::new();
    if expected_verdict == "GREEN" {
        if !observed_set.is_empty() {
            fixture_failures.push(format!(
                "{fixture_id}: GREEN service-root fixture produced violations {:?}",
                observed_set.iter().collect::<Vec<_>>()
            ));
        }
        if !expected_violations.is_empty() {
            fixture_failures.push(format!(
                "{fixture_id}: GREEN fixture must not list expected_violations"
            ));
        }
    } else {
        if observed_set.is_empty() {
            fixture_failures.push(format!(
                "{fixture_id}: RED service-root fixture must produce violations"
            ));
        }
        let missing_expected = expected_violations
            .difference(&observed_set)
            .cloned()
            .collect::<Vec<_>>();
        if !missing_expected.is_empty() {
            fixture_failures.push(format!(
                "{fixture_id}: expected violations were not observed {missing_expected:?}"
            ));
        }
    }

    FixtureValidation {
        path: path.to_string(),
        fixture_id,
        expected_verdict,
        expected_violations: expected_violations.into_iter().collect(),
        observed_violations: observed_set.into_iter().collect(),
        fixture_passed: fixture_failures.is_empty(),
        failures: fixture_failures,
    }
}

fn default_fixture_paths(root: &Path) -> Result<Vec<PathBuf>, String> {
    let dir = root.join(DEFAULT_FIXTURE_DIR);
    let mut paths = fs::read_dir(&dir)
        .map_err(|error| format!("read {} failed: {error}", dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn fixture_paths(root: &Path, explicit: &[String]) -> Result<Vec<PathBuf>, String> {
    if explicit.is_empty() {
        return default_fixture_paths(root);
    }
    Ok(explicit
        .iter()
        .map(|item| {
            let path = PathBuf::from(item);
            if path.is_absolute() {
                path
            } else {
                root.join(path)
            }
        })
        .collect())
}

pub fn evaluate_sources(
    root: &Path,
    inventory: &Json,
    packets: &Json,
    fixtures: &[(String, Json)],
) -> Report {
    let mut failures = Vec::new();
    let (inventory_failures, inventory_summary) = validate_inventory(root, inventory);
    failures.extend(inventory_failures);
    let packet_summary = validate_packets(Some(packets), &mut failures, None, "");

    let inventory_roots = inventory
        .as_object()
        .map(|inventory| object_list(object_field(inventory, "closed_world_roots")))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| string_field(item, "root"))
        .collect::<BTreeSet<_>>();
    let inventory_roots = if inventory_roots.is_empty() {
        set_from(&CLOSED_WORLD_ROOTS)
    } else {
        inventory_roots
    };

    let fixture_results = fixtures
        .iter()
        .map(|(path, fixture)| validate_fixture(path, fixture, &inventory_roots))
        .collect::<Vec<_>>();
    for fixture in &fixture_results {
        failures.extend(fixture.failures.iter().cloned());
    }
    let expected_green_fixture_count = fixture_results
        .iter()
        .filter(|item| item.expected_verdict == "GREEN")
        .count();
    let expected_red_fixture_count = fixture_results
        .iter()
        .filter(|item| item.expected_verdict == "RED")
        .count();
    let service_inventory_published = inventory
        .as_object()
        .and_then(|object| object_field(object, "claim_boundary"))
        .and_then(Json::as_object)
        .and_then(|boundary| bool_field(boundary, "service_inventory_published"))
        == Some(true);
    let closed_world_root_classifier_measured = inventory
        .as_object()
        .and_then(|object| object_field(object, "claim_boundary"))
        .and_then(Json::as_object)
        .and_then(|boundary| bool_field(boundary, "closed_world_root_classifier_measured"))
        == Some(true);
    let structural_packet_catalog_published = packets
        .as_object()
        .and_then(|object| object_field(object, "claim_boundary"))
        .and_then(Json::as_object)
        .and_then(|boundary| bool_field(boundary, "structural_packet_catalog_published"))
        == Some(true);
    let failures = sorted_unique(failures);
    let measured = failures.is_empty();

    Report {
        authority_boundary: AUTHORITY_BOUNDARY.to_string(),
        service_inventory_published,
        service_root_classifier_measured: measured,
        closed_world_root_classifier_measured,
        structural_packet_catalog_published,
        inventory_summary,
        packet_summary,
        fixture_count: fixture_results.len(),
        expected_green_fixture_count,
        expected_red_fixture_count,
        fixture_results,
        status_mutation_performed: false,
        protected_branch_authority_proven: false,
        live_required_context_execution_proven: false,
        full_service_inventory_coverage_proven: false,
        post_migration_pure_split_proven: false,
        structural_shards_executed: false,
        p0_0_green: false,
        phase0_complete: false,
        production_ready: false,
        hyperscaler_grade: false,
        verdict: if measured { "PASS" } else { "FAIL" }.to_string(),
        failures,
    }
}

pub fn evaluate_paths(
    root: &Path,
    inventory_path: &Path,
    packets_path: &Path,
    explicit_fixtures: &[String],
) -> Result<Report, String> {
    let inventory = load_json(inventory_path)?;
    let packets = load_json(packets_path)?;
    let fixtures = fixture_paths(root, explicit_fixtures)?
        .into_iter()
        .map(|path| {
            if !path.is_file() {
                return Err(format!(
                    "fixture_path_missing:{}",
                    display_path(&path, root)
                ));
            }
            let display = display_path(&path, root);
            load_json(&path).map(|fixture| (display, fixture))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(evaluate_sources(root, &inventory, &packets, &fixtures))
}

pub fn to_json(report: &Report) -> String {
    format!(
        concat!(
            "{{",
            "\"authority_boundary\":{},",
            "\"closed_world_root_classifier_measured\":{},",
            "\"closed_world_root_count\":{},",
            "\"expected_green_fixture_count\":{},",
            "\"expected_red_fixture_count\":{},",
            "\"failures\":{},",
            "\"fixture_count\":{},",
            "\"fixture_results\":{},",
            "\"full_service_inventory_coverage_proven\":false,",
            "\"hyperscaler_grade\":false,",
            "\"inventory_entry_count\":{},",
            "\"live_required_context_execution_proven\":false,",
            "\"missing_inventory_entry_count\":{},",
            "\"observed_direct_child_dir_count\":{},",
            "\"p0_0_green\":false,",
            "\"phase0_complete\":false,",
            "\"post_migration_pure_split_proven\":false,",
            "\"production_ready\":false,",
            "\"protected_branch_authority_proven\":false,",
            "\"required_packet_family_count\":{},",
            "\"service_inventory_published\":{},",
            "\"service_root_classifier_measured\":{},",
            "\"status_mutation_performed\":false,",
            "\"structural_packet_catalog_published\":{},",
            "\"structural_packet_count\":{},",
            "\"structural_shards_executed\":false,",
            "\"verdict\":{}",
            "}}"
        ),
        json_string(&report.authority_boundary),
        bool_json(report.closed_world_root_classifier_measured),
        report.inventory_summary.closed_world_root_count,
        report.expected_green_fixture_count,
        report.expected_red_fixture_count,
        string_array_json(&report.failures),
        report.fixture_count,
        fixture_results_json(&report.fixture_results),
        report.inventory_summary.inventory_entry_count,
        report.inventory_summary.missing_inventory_entry_count,
        report.inventory_summary.observed_direct_child_dir_count,
        report.packet_summary.required_packet_family_count,
        bool_json(report.service_inventory_published),
        bool_json(report.service_root_classifier_measured),
        bool_json(report.structural_packet_catalog_published),
        report.packet_summary.structural_packet_count,
        json_string(&report.verdict),
    )
}

fn fixture_results_json(results: &[FixtureValidation]) -> String {
    format!(
        "[{}]",
        results
            .iter()
            .map(fixture_validation_json)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn fixture_validation_json(result: &FixtureValidation) -> String {
    format!(
        concat!(
            "{{",
            "\"expected_violations\":{},",
            "\"expected_verdict\":{},",
            "\"failures\":{},",
            "\"fixture_id\":{},",
            "\"fixture_passed\":{},",
            "\"observed_violations\":{},",
            "\"path\":{}",
            "}}"
        ),
        string_array_json(&result.expected_violations),
        json_string(&result.expected_verdict),
        string_array_json(&result.failures),
        json_string(&result.fixture_id),
        bool_json(result.fixture_passed),
        string_array_json(&result.observed_violations),
        json_string(&result.path),
    )
}

fn string_array_json(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn bool_json(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('\"');
    out
}

fn absolute_under_root(root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn main() {
    let mut repo_root = PathBuf::from(".");
    let mut inventory = DEFAULT_INVENTORY.to_string();
    let mut packets = DEFAULT_PACKETS.to_string();
    let mut fixtures = Vec::new();
    let mut emit_json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(args.next().expect("--repo-root requires a value"))
            }
            "--inventory" => inventory = args.next().expect("--inventory requires a value"),
            "--packets" => packets = args.next().expect("--packets requires a value"),
            "--fixture" => fixtures.push(args.next().expect("--fixture requires a value")),
            "--json" => emit_json = true,
            other => panic!("unknown argument {other}"),
        }
    }
    let root = repo_root
        .canonicalize()
        .unwrap_or_else(|error| panic!("canonicalize {} failed: {error}", repo_root.display()));
    let inventory_path = absolute_under_root(&root, &inventory);
    let packets_path = absolute_under_root(&root, &packets);
    let report = evaluate_paths(&root, &inventory_path, &packets_path, &fixtures)
        .unwrap_or_else(|error| panic!("{error}"));
    let rendered = to_json(&report);
    if emit_json || report.verdict == "PASS" {
        println!("{rendered}");
    } else {
        eprintln!("{rendered}");
    }
    if report.verdict != "PASS" {
        std::process::exit(1);
    }
}
