//! AC-0.5/AC-0.10 D1 seam contract seed gate.
//!
//! This checker is local/static evidence only. It verifies that the shape-only
//! A2a/A2b proto3 contracts exist, keep `consistency_token` proto-optional
//! while requiring token presence in Phase-0 fixtures, and preserve explicit
//! non-claim boundaries. It never runs live D1 conformance, posts statuses,
//! mutates branch protection, or proves Phase-0 completion.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_REGISTRY: &str = "specs/d1-seam-contracts-registry.json";
const A2A_PROTO: &str = "contracts/proto/d1/a2a/mutation/v1/entity_mutation.proto";
const A2B_PROTO: &str = "contracts/proto/d1/a2b/workflow/v1/workflow_ai_step_invocation.proto";

const FIXTURES: &[&str] = &[
    "specs/fixtures/phase0-d1-seam-contracts/tc-0.5-good-d1-seam-contracts.json",
    "specs/fixtures/phase0-d1-seam-contracts/tc-0.5-bad-missing-consistency-token.json",
    "specs/fixtures/phase0-d1-seam-contracts/tc-0.5-bad-proto-required-or-frozen-topology.json",
];

const FALSE_CLAIMS: &[&str] = &[
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
];

const TRUE_REGISTRY_FLAGS: &[&str] = &[
    "d1_seam_contracts_measured",
    "a2a_payload_contract_present",
    "a2b_invocation_signature_present",
    "mutation_result_consistency_token_optional",
    "phase0_consistency_token_presence_required",
    "topology_fields_conformance_gated",
    "d1_conformance_reference_present",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtoField {
    pub qualifier: Option<String>,
    pub field_type: String,
    pub name: String,
    pub number: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResult {
    pub path: String,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureResult {
    pub path: String,
    pub expected: String,
    pub observed_violations: Vec<String>,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    pub verdict: String,
    pub registry: String,
    pub contract_results: Vec<FileResult>,
    pub fixture_results: Vec<FixtureResult>,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub repo_root: PathBuf,
    pub registry: String,
    pub json: bool,
}

fn json_escape(input: &str) -> String {
    input
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn compact_json_text(input: &str) -> String {
    input.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn bool_token(key: &str, value: bool) -> String {
    format!("\"{}\":{}", key, if value { "true" } else { "false" })
}

fn has_bool(text: &str, key: &str, value: bool) -> bool {
    compact_json_text(text).contains(&bool_token(key, value))
}

fn value_after_key(chunk: &str, key: &str) -> Option<String> {
    let key_token = format!("\"{}\"", key);
    let after_key = chunk.split_once(&key_token)?.1;
    let after_colon = after_key.split_once(':')?.1.trim_start();
    let after_quote = after_colon.strip_prefix('"')?;
    let mut value = String::new();
    let mut escaped = false;
    for ch in after_quote.chars() {
        if escaped {
            value.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(value),
            _ => value.push(ch),
        }
    }
    None
}

fn read_repo_file(repo_root: &Path, path: &str) -> Result<String, String> {
    fs::read_to_string(repo_root.join(path)).map_err(|error| format!("{path}: {error}"))
}

fn has_syntax_proto3(source: &str) -> bool {
    source.lines().any(|line| {
        let compact = line.split_whitespace().collect::<String>();
        compact == "syntax=\"proto3\";"
    })
}

fn has_package(source: &str, package: &str) -> bool {
    source.lines().any(|line| {
        let compact = line.split_whitespace().collect::<String>();
        compact == format!("package{package};")
    })
}

fn skip_whitespace(bytes: &[u8], mut idx: usize) -> usize {
    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }
    idx
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0_u32;
    for (idx, character) in source[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(open + idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn is_identifier_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

pub fn find_message_body(source: &str, message_name: &str) -> Result<String, String> {
    let bytes = source.as_bytes();
    let mut cursor = 0;
    while let Some(relative) = source[cursor..].find("message") {
        let start = cursor + relative;
        let before_ok = start == 0
            || !source[..start]
                .chars()
                .next_back()
                .is_some_and(is_identifier_char);
        let mut idx = start + "message".len();
        idx = skip_whitespace(bytes, idx);
        if before_ok && source[idx..].starts_with(message_name) {
            let after_name = idx + message_name.len();
            let after_ok = source[after_name..]
                .chars()
                .next()
                .is_none_or(|character| !is_identifier_char(character));
            if after_ok {
                let brace = source[after_name..]
                    .find('{')
                    .map(|offset| after_name + offset)
                    .ok_or_else(|| format!("{message_name} message missing brace"))?;
                let end = matching_brace(source, brace)
                    .ok_or_else(|| format!("{message_name} message has unbalanced braces"))?;
                return Ok(source[brace + 1..end].to_owned());
            }
        }
        cursor = start + "message".len();
    }
    Err(format!("{message_name} message missing"))
}

pub fn parse_fields(source: &str) -> Vec<ProtoField> {
    let mut fields = Vec::new();
    for raw_line in source.lines() {
        let line = raw_line
            .split_once("//")
            .map_or(raw_line, |(left, _)| left)
            .trim()
            .trim_end_matches(';')
            .trim();
        let Some((left, right)) = line.split_once('=') else {
            continue;
        };
        let number = right
            .trim()
            .chars()
            .take_while(|ch| ch.is_ascii_digit())
            .collect::<String>()
            .parse::<u32>()
            .ok();
        let tokens = left.split_whitespace().collect::<Vec<_>>();
        let (qualifier, field_type, name) = match tokens.as_slice() {
            ["optional", field_type, name] => (Some("optional"), *field_type, *name),
            ["repeated", field_type, name] => (Some("repeated"), *field_type, *name),
            ["required", field_type, name] => (Some("required"), *field_type, *name),
            [field_type, name] => (None, *field_type, *name),
            _ => continue,
        };
        if let Some(number) = number {
            fields.push(ProtoField {
                qualifier: qualifier.map(str::to_owned),
                field_type: field_type.to_owned(),
                name: name.to_owned(),
                number,
            });
        }
    }
    fields
}

fn require_field(
    prefix: &str,
    fields: &[ProtoField],
    name: &str,
    field_type: &str,
    number: u32,
    qualifier: Option<&str>,
) -> Result<(), String> {
    let Some(field) = fields.iter().find(|field| field.name == name) else {
        return Err(format!("{prefix}:missing_field:{name}"));
    };
    if field.field_type != field_type
        || field.number != number
        || field.qualifier.as_deref() != qualifier
    {
        let actual = format!(
            "{}{} {} = {}",
            field
                .qualifier
                .as_ref()
                .map(|qualifier| format!("{qualifier} "))
                .unwrap_or_default(),
            field.field_type,
            name,
            field.number
        );
        let expected = format!(
            "{}{} {} = {}",
            qualifier
                .map(|qualifier| format!("{qualifier} "))
                .unwrap_or_default(),
            field_type,
            name,
            number
        );
        return Err(format!(
            "{prefix}:field_mismatch:{name}:expected {expected}:got {actual}"
        ));
    }
    Ok(())
}

fn reject_duplicate_tags(
    prefix: &str,
    message_name: &str,
    fields: &[ProtoField],
) -> Result<(), String> {
    let mut seen = Vec::new();
    for field in fields {
        if seen.contains(&field.number) {
            return Err(format!(
                "{prefix}:duplicate_tag:{message_name}:{}",
                field.number
            ));
        }
        seen.push(field.number);
    }
    Ok(())
}

fn validate_entity_payload(source: &str, message_name: &str, failures: &mut Vec<String>) {
    match find_message_body(source, message_name) {
        Ok(body) => {
            let fields = parse_fields(&body);
            for result in [
                require_field(message_name, &fields, "entity_id", "string", 1, None),
                require_field(message_name, &fields, "entity_type", "string", 2, None),
                require_field(
                    message_name,
                    &fields,
                    "mutation_kind",
                    "MutationKind",
                    3,
                    None,
                ),
                require_field(
                    message_name,
                    &fields,
                    "effective_dating_coords",
                    "EffectiveDatingCoords",
                    4,
                    None,
                ),
                require_field(message_name, &fields, "audit_link", "AuditLink", 5, None),
                reject_duplicate_tags(message_name, message_name, &fields),
            ] {
                if let Err(error) = result {
                    failures.push(error);
                }
            }
        }
        Err(error) => failures.push(format!("{message_name}:{error}")),
    }
}

pub fn a2a_proto_failures(source: &str) -> Vec<String> {
    let mut failures = Vec::new();
    if !has_syntax_proto3(source) {
        failures.push("a2a:syntax_must_be_proto3".to_owned());
    }
    if !has_package(source, "oyatie.d1.a2a.mutation.v1") {
        failures.push("a2a:package_mismatch".to_owned());
    }
    for token in [
        "enum MutationKind",
        "MUTATION_KIND_CREATED",
        "MUTATION_KIND_MUTATED",
        "MUTATION_KIND_DELETED",
        "AC-0.10b",
        "Phase-2",
    ] {
        if !source.contains(token) {
            failures.push(format!("a2a:missing_anchor:{token}"));
        }
    }
    if source.contains("required string consistency_token") {
        failures.push("a2a:proto_required_consistency_token".to_owned());
    }

    validate_entity_payload(source, "EntityCreated", &mut failures);
    validate_entity_payload(source, "EntityMutated", &mut failures);
    validate_entity_payload(source, "EntityDeleted", &mut failures);

    match find_message_body(source, "MutationResult") {
        Ok(body) => {
            let fields = parse_fields(&body);
            for result in [
                require_field("MutationResult", &fields, "entity_id", "string", 1, None),
                require_field("MutationResult", &fields, "entity_type", "string", 2, None),
                require_field(
                    "MutationResult",
                    &fields,
                    "mutation_kind",
                    "MutationKind",
                    3,
                    None,
                ),
                require_field(
                    "MutationResult",
                    &fields,
                    "consistency_token",
                    "string",
                    4,
                    Some("optional"),
                ),
                require_field(
                    "MutationResult",
                    &fields,
                    "audit_link",
                    "AuditLink",
                    5,
                    None,
                ),
                require_field(
                    "MutationResult",
                    &fields,
                    "topology",
                    "TopologyConformanceGate",
                    6,
                    None,
                ),
                require_field(
                    "MutationResult",
                    &fields,
                    "d1_conformance_reference",
                    "string",
                    7,
                    None,
                ),
                reject_duplicate_tags("MutationResult", "MutationResult", &fields),
            ] {
                if let Err(error) = result {
                    failures.push(error);
                }
            }
        }
        Err(error) => failures.push(format!("MutationResult:{error}")),
    }

    match find_message_body(source, "TopologyConformanceGate") {
        Ok(body) => {
            let fields = parse_fields(&body);
            for result in [
                require_field(
                    "TopologyConformanceGate",
                    &fields,
                    "execution_locality",
                    "string",
                    1,
                    None,
                ),
                require_field(
                    "TopologyConformanceGate",
                    &fields,
                    "outbox_offset",
                    "string",
                    2,
                    None,
                ),
                require_field(
                    "TopologyConformanceGate",
                    &fields,
                    "saga_correlation_id",
                    "string",
                    3,
                    None,
                ),
                require_field(
                    "TopologyConformanceGate",
                    &fields,
                    "conformance_gate",
                    "string",
                    4,
                    None,
                ),
                reject_duplicate_tags(
                    "TopologyConformanceGate",
                    "TopologyConformanceGate",
                    &fields,
                ),
            ] {
                if let Err(error) = result {
                    failures.push(error);
                }
            }
        }
        Err(error) => failures.push(format!("TopologyConformanceGate:{error}")),
    }

    failures
}

pub fn a2b_proto_failures(source: &str) -> Vec<String> {
    let mut failures = Vec::new();
    if !has_syntax_proto3(source) {
        failures.push("a2b:syntax_must_be_proto3".to_owned());
    }
    if !has_package(source, "oyatie.d1.a2b.workflow.v1") {
        failures.push("a2b:package_mismatch".to_owned());
    }
    for token in [
        "service WorkflowAiStepInvoker",
        "rpc InvokeWorkflowAiStep(WorkflowAiStepInvocationRequest) returns (WorkflowAiStepInvocationResponse)",
        "distinct from studio-authoring LlmAssistDraft RPCs",
    ] {
        if !source.contains(token) {
            failures.push(format!("a2b:missing_anchor:{token}"));
        }
    }

    match find_message_body(source, "TypedToolRef") {
        Ok(body) => {
            let fields = parse_fields(&body);
            for result in [
                require_field("TypedToolRef", &fields, "tool_namespace", "string", 1, None),
                require_field("TypedToolRef", &fields, "tool_name", "string", 2, None),
                require_field("TypedToolRef", &fields, "tool_version", "string", 3, None),
                reject_duplicate_tags("TypedToolRef", "TypedToolRef", &fields),
            ] {
                if let Err(error) = result {
                    failures.push(error);
                }
            }
        }
        Err(error) => failures.push(format!("TypedToolRef:{error}")),
    }

    match find_message_body(source, "CedarPrincipal") {
        Ok(body) => {
            let fields = parse_fields(&body);
            for result in [
                require_field("CedarPrincipal", &fields, "tenant_id", "string", 1, None),
                require_field("CedarPrincipal", &fields, "principal_id", "string", 2, None),
                require_field(
                    "CedarPrincipal",
                    &fields,
                    "cedar_entity_type",
                    "string",
                    3,
                    None,
                ),
                require_field(
                    "CedarPrincipal",
                    &fields,
                    "policy_pack_ref",
                    "string",
                    4,
                    None,
                ),
                reject_duplicate_tags("CedarPrincipal", "CedarPrincipal", &fields),
            ] {
                if let Err(error) = result {
                    failures.push(error);
                }
            }
        }
        Err(error) => failures.push(format!("CedarPrincipal:{error}")),
    }

    match find_message_body(source, "WorkflowAiStepInvocationRequest") {
        Ok(body) => {
            let fields = parse_fields(&body);
            for result in [
                require_field(
                    "WorkflowAiStepInvocationRequest",
                    &fields,
                    "workflow_run_id",
                    "string",
                    1,
                    None,
                ),
                require_field(
                    "WorkflowAiStepInvocationRequest",
                    &fields,
                    "workflow_step_id",
                    "string",
                    2,
                    None,
                ),
                require_field(
                    "WorkflowAiStepInvocationRequest",
                    &fields,
                    "tool_ref",
                    "TypedToolRef",
                    3,
                    None,
                ),
                require_field(
                    "WorkflowAiStepInvocationRequest",
                    &fields,
                    "args_json",
                    "bytes",
                    4,
                    None,
                ),
                require_field(
                    "WorkflowAiStepInvocationRequest",
                    &fields,
                    "cedar_principal",
                    "CedarPrincipal",
                    5,
                    None,
                ),
                require_field(
                    "WorkflowAiStepInvocationRequest",
                    &fields,
                    "idempotency_key",
                    "string",
                    6,
                    None,
                ),
                require_field(
                    "WorkflowAiStepInvocationRequest",
                    &fields,
                    "d1_conformance_reference",
                    "string",
                    7,
                    None,
                ),
                require_field(
                    "WorkflowAiStepInvocationRequest",
                    &fields,
                    "execution_locality",
                    "string",
                    8,
                    None,
                ),
                require_field(
                    "WorkflowAiStepInvocationRequest",
                    &fields,
                    "topology_conformance_gate",
                    "string",
                    9,
                    None,
                ),
                reject_duplicate_tags(
                    "WorkflowAiStepInvocationRequest",
                    "WorkflowAiStepInvocationRequest",
                    &fields,
                ),
            ] {
                if let Err(error) = result {
                    failures.push(error);
                }
            }
        }
        Err(error) => failures.push(format!("WorkflowAiStepInvocationRequest:{error}")),
    }

    match find_message_body(source, "WorkflowAiStepInvocationResponse") {
        Ok(body) => {
            let fields = parse_fields(&body);
            for result in [
                require_field(
                    "WorkflowAiStepInvocationResponse",
                    &fields,
                    "invocation_id",
                    "string",
                    1,
                    None,
                ),
                require_field(
                    "WorkflowAiStepInvocationResponse",
                    &fields,
                    "workflow_run_id",
                    "string",
                    2,
                    None,
                ),
                require_field(
                    "WorkflowAiStepInvocationResponse",
                    &fields,
                    "workflow_step_id",
                    "string",
                    3,
                    None,
                ),
                require_field(
                    "WorkflowAiStepInvocationResponse",
                    &fields,
                    "accepted",
                    "bool",
                    4,
                    None,
                ),
                require_field(
                    "WorkflowAiStepInvocationResponse",
                    &fields,
                    "idempotency_key",
                    "string",
                    5,
                    None,
                ),
                require_field(
                    "WorkflowAiStepInvocationResponse",
                    &fields,
                    "consistency_token",
                    "string",
                    6,
                    Some("optional"),
                ),
                require_field(
                    "WorkflowAiStepInvocationResponse",
                    &fields,
                    "d1_conformance_reference",
                    "string",
                    7,
                    None,
                ),
                reject_duplicate_tags(
                    "WorkflowAiStepInvocationResponse",
                    "WorkflowAiStepInvocationResponse",
                    &fields,
                ),
            ] {
                if let Err(error) = result {
                    failures.push(error);
                }
            }
        }
        Err(error) => failures.push(format!("WorkflowAiStepInvocationResponse:{error}")),
    }

    failures
}

pub fn registry_failures(text: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for claim in FALSE_CLAIMS {
        if !has_bool(text, claim, false) {
            failures.push(format!("forbidden_true_or_missing_claim_{claim}"));
        }
    }
    for flag in TRUE_REGISTRY_FLAGS {
        if !has_bool(text, flag, true) {
            failures.push(format!("missing_registry_flag_{flag}"));
        }
    }
    for token in [
        "//:d1-seam-contracts-check",
        "scripts/ci/assert-d1-seam-contracts.rs",
        "scripts/tests/d1_seam_contracts_check.rs",
        A2A_PROTO,
        A2B_PROTO,
        "optional string consistency_token",
        "phase0_consistency_token_presence_required",
        "AC-0.10b",
        "Protocol Buffers — Proto3 Language Guide",
        "https://protobuf.dev/programming-guides/proto3/",
        "https://protobuf.dev/programming-guides/field_presence/",
        "Buck2-native LLVM source-based coverage",
        "Tarpaulin is not the canonical coverage surface",
        "Dual Cargo.toml + Buck2/Reindeer",
        "trusted cloud-ci/oya-ci",
        "hyperscaler-oriented",
    ] {
        if !text.contains(token) {
            failures.push(format!("missing_required_registry_anchor:{token}"));
        }
    }
    if !compact_json_text(text).contains("\"new_oya_cli_surface_added\":false") {
        failures.push("missing_no_new_oya_cli_surface_anchor".to_owned());
    }
    for fixture in FIXTURES {
        if !text.contains(fixture) {
            failures.push(format!("missing_registered_fixture:{fixture}"));
        }
    }
    failures
}

fn expected_verdict(text: &str) -> String {
    value_after_key(text, "expected_verdict").unwrap_or_else(|| "<missing>".to_owned())
}

fn expected_violations(text: &str) -> Vec<String> {
    let mut expected = Vec::new();
    if let Some(single) = value_after_key(text, "expected_violation") {
        expected.push(single);
    }
    for token in [
        "missing_consistency_token",
        "proto_required_consistency_token",
        "topology_frozen_without_conformance_gate",
    ] {
        if text.contains(&format!("\"{token}\"")) && !expected.iter().any(|item| item == token) {
            expected.push(token.to_owned());
        }
    }
    expected
}

pub fn fixture_policy_failures(text: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for claim in FALSE_CLAIMS {
        if !has_bool(text, claim, false) {
            failures.push(format!("fixture_forbidden_true_or_missing_claim_{claim}"));
        }
    }

    let compact = compact_json_text(text);
    let expected = expected_verdict(text);
    if expected == "GREEN" {
        if !compact.contains("\"consistency_token\":\"d1ctok_fixture_good_read_your_writes\"") {
            failures.push("missing_consistency_token".to_owned());
        }
        for token in [
            A2A_PROTO,
            A2B_PROTO,
            "AC-0.10b-read-your-writes-xfail",
            ".omx/plans/test-spec-phase0.md#TC-0.10b.1",
        ] {
            if !text.contains(token) {
                failures.push(format!("good_fixture_missing_anchor:{token}"));
            }
        }
    }
    if expected == "RED" {
        if text.contains("TC-0.5-BAD-missing-consistency-token")
            && compact.contains("\"mutation_result_fixture\":{")
            && !compact.contains("\"consistency_token\":")
        {
            failures.push("missing_consistency_token".to_owned());
        }
        if text.contains("required string consistency_token") {
            failures.push("proto_required_consistency_token".to_owned());
        }
        if has_bool(text, "topology_frozen_without_conformance_gate", true) {
            failures.push("topology_frozen_without_conformance_gate".to_owned());
        }
    }

    failures
}

fn evaluate_fixture(path: &str, text: &str) -> FixtureResult {
    let expected = expected_verdict(text);
    let observed_violations = fixture_policy_failures(text);
    let mut failures = Vec::new();
    match expected.as_str() {
        "GREEN" => {
            if !observed_violations.is_empty() {
                failures.push(format!("green_fixture_reported_violations:{path}"));
            }
        }
        "RED" => {
            let expected_items = expected_violations(text);
            if expected_items.is_empty() {
                failures.push(format!("red_fixture_missing_expected_violation:{path}"));
            }
            for expected_item in expected_items {
                if !observed_violations
                    .iter()
                    .any(|violation| violation == &expected_item)
                {
                    failures.push(format!(
                        "red_fixture_missing_observed_violation:{path}:{expected_item}"
                    ));
                }
            }
        }
        _ => failures.push(format!("fixture_missing_expected_verdict:{path}")),
    }
    FixtureResult {
        path: path.to_owned(),
        expected,
        observed_violations,
        failures,
    }
}

pub fn evaluate(repo_root: &Path, registry_path: &str) -> Result<Evaluation, String> {
    let registry_text = read_repo_file(repo_root, registry_path)?;
    let mut failures = registry_failures(&registry_text);

    let a2a_text = read_repo_file(repo_root, A2A_PROTO)?;
    let a2a_failures = a2a_proto_failures(&a2a_text);
    failures.extend(
        a2a_failures
            .iter()
            .map(|failure| format!("{A2A_PROTO}:{failure}")),
    );

    let a2b_text = read_repo_file(repo_root, A2B_PROTO)?;
    let a2b_failures = a2b_proto_failures(&a2b_text);
    failures.extend(
        a2b_failures
            .iter()
            .map(|failure| format!("{A2B_PROTO}:{failure}")),
    );

    let mut fixture_results = Vec::new();
    for fixture in FIXTURES {
        let text = read_repo_file(repo_root, fixture)?;
        let result = evaluate_fixture(fixture, &text);
        failures.extend(result.failures.iter().cloned());
        fixture_results.push(result);
    }

    failures.sort();
    failures.dedup();
    Ok(Evaluation {
        verdict: if failures.is_empty() { "PASS" } else { "FAIL" }.to_owned(),
        registry: registry_path.to_owned(),
        contract_results: vec![
            FileResult {
                path: A2A_PROTO.to_owned(),
                failures: a2a_failures,
            },
            FileResult {
                path: A2B_PROTO.to_owned(),
                failures: a2b_failures,
            },
        ],
        fixture_results,
        failures,
    })
}

fn json_array(items: &[String]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(|item| format!("\"{}\"", json_escape(item)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn render_json(evaluation: &Evaluation) -> String {
    let contract_results = evaluation
        .contract_results
        .iter()
        .map(|result| {
            format!(
                "{{\"path\":\"{}\",\"failures\":{}}}",
                json_escape(&result.path),
                json_array(&result.failures)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let fixture_results = evaluation
        .fixture_results
        .iter()
        .map(|result| {
            format!(
                "{{\"path\":\"{}\",\"expected\":\"{}\",\"observed_violations\":{},\"failures\":{}}}",
                json_escape(&result.path),
                json_escape(&result.expected),
                json_array(&result.observed_violations),
                json_array(&result.failures)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"authority_boundary\":\"AC-0.5/AC-0.10 local/static D1 seam contract evidence only; no live conformance, status mutation, required-context authority, P0.0 green, Phase-0 completion, production readiness, or hyperscaler-grade readiness proven\",\"d1_seam_contracts_measured\":{},\"status_mutation_performed\":false,\"protected_branch_authority_proven\":false,\"live_required_context_execution_proven\":false,\"p0_0_green\":false,\"phase0_complete\":false,\"production_ready\":false,\"hyperscaler_grade\":false,\"registry\":\"{}\",\"contract_count\":{},\"fixture_count\":{},\"contract_results\":[{}],\"fixture_results\":[{}],\"verdict\":\"{}\",\"failures\":{}}}",
        evaluation.verdict == "PASS",
        json_escape(&evaluation.registry),
        evaluation.contract_results.len(),
        evaluation.fixture_results.len(),
        contract_results,
        fixture_results,
        json_escape(&evaluation.verdict),
        json_array(&evaluation.failures)
    )
}

fn parse_args() -> Result<Config, String> {
    let mut repo_root = PathBuf::from(".");
    let mut registry = DEFAULT_REGISTRY.to_owned();
    let mut json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = args.next() else {
                    return Err("missing value for --repo-root".to_owned());
                };
                repo_root = PathBuf::from(value);
            }
            "--registry" => {
                let Some(value) = args.next() else {
                    return Err("missing value for --registry".to_owned());
                };
                registry = value;
            }
            "--json" => json = true,
            "--help" | "-h" => {
                return Err(
                    "usage: assert-d1-seam-contracts [--repo-root DIR] [--registry PATH] [--json]"
                        .to_owned(),
                );
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(Config {
        repo_root,
        registry,
        json,
    })
}

fn main() -> std::process::ExitCode {
    let config = match parse_args() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            return std::process::ExitCode::from(2);
        }
    };
    let evaluation = match evaluate(&config.repo_root, &config.registry) {
        Ok(evaluation) => evaluation,
        Err(error) => {
            eprintln!("d1-seam-contracts: RED\n{error}");
            return std::process::ExitCode::FAILURE;
        }
    };
    let rendered = render_json(&evaluation);
    if config.json || evaluation.verdict == "PASS" {
        println!("{rendered}");
    } else {
        eprintln!("{rendered}");
    }
    if evaluation.verdict == "PASS" {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}
