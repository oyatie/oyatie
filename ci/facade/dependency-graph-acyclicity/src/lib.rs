//! Runtime-face-aware substrate graph-v2 validator (ADR-0635).
//!
//! The document contains exactly five separately typed graphs. Only
//! `steady_state_request` is constrained to be acyclic. The failure graph is not authored
//! independently: it must equal the max-min reverse transitive closure of the steady-state graph.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};

use serde_json::Value;
use sha2::{Digest, Sha256};

pub const GATE_ID: &str = "cloud-ci-substrate-dependency-dag-acyclicity";
pub const DEFAULT_POLICY_PATH: &str =
    "ci/facade/dependency-graph-acyclicity/substrate-dependency-dag-policy.json";
pub const GRAPH_KINDS: [&str; 5] = [
    "genesis",
    "new_cell_provisioning",
    "steady_state_request",
    "control_data_publication",
    "failure_brownout_propagation",
];
pub const IMPACT_RULES: [&str; 4] = ["INDEPENDENT", "BROWNOUT", "DEGRADED", "FULL"];
pub const DEPENDENCY_UNIT_COUNT: usize = 19;
pub const CAPABILITY_COUNT: usize = 24;
pub const SCHEMA_CANONICAL_SHA256: &str =
    "b0a826504fac2718e264dd9c307b425e82211675cfb06d180bb9fbd8bc086ce4";
pub const GRAPH_DOCTRINE_ADRS: [&str; 5] =
    ["ADR-0245", "ADR-0280", "ADR-0562", "ADR-0615", "ADR-0635"];
const DEPENDENCY_UNIT_AUTHORITY: [(&str, &str, &str, &str); DEPENDENCY_UNIT_COUNT] = [
    ("network.bootstrap", "network", "bootstrap", "B0"),
    ("cell.envelope", "cell", "envelope", "B0"),
    ("cell.genesis", "cell", "genesis", "G"),
    ("cell.lifecycle.cp", "cell", "lifecycle.cp", "G"),
    ("cell.router.dp", "cell", "router.dp", "R"),
    ("iam.admin.cp", "iam", "admin.cp", "G"),
    ("iam.local-verifier", "iam", "local-verifier", "C0"),
    ("tenancy.directory.cp", "tenancy", "directory.cp", "G"),
    ("tenancy.local-context", "tenancy", "local-context", "C0"),
    ("policy.authoring.cp", "policy", "authoring.cp", "G"),
    ("policy.local-pdp", "policy", "local-pdp", "C0"),
    ("secrets.root-control", "secrets", "root-control", "G"),
    ("secrets.cell-issuer", "secrets", "cell-issuer", "C0"),
    (
        "audit.control-aggregation",
        "audit",
        "control-aggregation",
        "G",
    ),
    ("audit.cell-seal", "audit", "cell-seal", "C1"),
    (
        "observability.cell-runtime",
        "observability",
        "cell-runtime",
        "C1",
    ),
    ("data.ontology-runtime", "data", "ontology-runtime", "C1"),
    ("intelligence.runtime", "intelligence", "runtime", "C2"),
    ("workflow.runtime", "workflow", "runtime", "C2"),
];
const PATH_RULE: &str = "minimum severity across every steady_state_request edge on a path; a weak propagation edge bounds that path";
const MULTI_PATH_RULE: &str = "maximum severity across all paths from impacted_unit to failed_unit; the strongest propagation path wins";
const CLOSURE_DIRECTION: &str =
    "reverse_transitive_closure: request dependency A -> B yields failure propagation B -> A";
const TOPOLOGY_FOLLOW_UP_ID: &str = "W0-C-TOPOLOGY-COVERAGE";
const TOPOLOGY_TRACKING_ISSUE: &str = "https://github.com/jason931225/oyatie/issues/1537";
const NO_NEW_BASELINE_POLICY: &str = "no-new-frozen-baseline";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub gate_id: String,
    pub dag_path: String,
    pub schema_path: String,
    pub capability_registry_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dag {
    raw: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DagError {
    Parse(String),
    Io(String),
}

impl std::fmt::Display for DagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(reason) => write!(f, "dag parse error: {reason}"),
            Self::Io(reason) => write!(f, "dag io error: {reason}"),
        }
    }
}

impl std::error::Error for DagError {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: String,
    pub subject: String,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub derived_bootstrap_order: Option<Vec<String>>,
    pub verdict: Verdict,
}

fn finding(code: &str, subject: impl Into<String>, detail: impl Into<String>) -> Finding {
    Finding {
        code: code.to_owned(),
        subject: subject.into(),
        detail: detail.into(),
    }
}

pub fn load_policy(root: &Path, path: &str) -> Result<Policy, DagError> {
    let full = root.join(path);
    let bytes = fs::read_to_string(&full)
        .map_err(|error| DagError::Io(format!("{}: {error}", full.display())))?;
    parse_policy(&bytes)
}

pub fn parse_policy(bytes: &str) -> Result<Policy, DagError> {
    let value: Value = serde_json::from_str(bytes)
        .map_err(|error| DagError::Parse(format!("invalid policy json: {error}")))?;
    let gate_id = required_string(&value, "gate_id", "policy")?;
    if gate_id != GATE_ID {
        return Err(DagError::Parse(format!(
            "policy gate_id `{gate_id}` does not match `{GATE_ID}`"
        )));
    }
    let dag_path = required_string(&value, "dag_path", "policy")?;
    let schema_path = required_string(&value, "schema_path", "policy")?;
    let capability_registry_path = required_string(&value, "capability_registry_path", "policy")?;
    validate_repo_relative_path("dag_path", dag_path)?;
    validate_repo_relative_path("schema_path", schema_path)?;
    validate_repo_relative_path("capability_registry_path", capability_registry_path)?;
    let allowed: BTreeSet<&str> = [
        "_comment",
        "gate_id",
        "dag_path",
        "schema_path",
        "capability_registry_path",
    ]
    .into_iter()
    .collect();
    if let Some(object) = value.as_object()
        && let Some(extra) = object.keys().find(|key| !allowed.contains(key.as_str()))
    {
        return Err(DagError::Parse(format!(
            "policy closed schema rejects property `{extra}`"
        )));
    }
    Ok(Policy {
        gate_id: gate_id.to_owned(),
        dag_path: dag_path.to_owned(),
        schema_path: schema_path.to_owned(),
        capability_registry_path: capability_registry_path.to_owned(),
    })
}

fn required_string<'a>(value: &'a Value, key: &str, subject: &str) -> Result<&'a str, DagError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| DagError::Parse(format!("{subject} missing string `{key}`")))
}

fn validate_repo_relative_path(field: &str, path: &str) -> Result<(), DagError> {
    if path.trim().is_empty() {
        return Err(DagError::Parse(format!(
            "policy `{field}` must not be empty"
        )));
    }
    for component in Path::new(path).components() {
        match component {
            Component::ParentDir => {
                return Err(DagError::Parse(format!(
                    "policy `{field}` must not contain `..`: {path}"
                )));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(DagError::Parse(format!(
                    "policy `{field}` must be repo-relative: {path}"
                )));
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }
    Ok(())
}

pub fn load_dag(root: &Path, path: &str) -> Result<Dag, DagError> {
    let full = root.join(path);
    let bytes = fs::read_to_string(&full)
        .map_err(|error| DagError::Io(format!("{}: {error}", full.display())))?;
    parse_dag(&bytes)
}

pub fn load_json(root: &Path, path: &str) -> Result<Value, DagError> {
    let full = root.join(path);
    let bytes = fs::read_to_string(&full)
        .map_err(|error| DagError::Io(format!("{}: {error}", full.display())))?;
    serde_json::from_str(&bytes)
        .map_err(|error| DagError::Parse(format!("{}: invalid json: {error}", full.display())))
}

pub fn parse_dag(bytes: &str) -> Result<Dag, DagError> {
    let raw: Value = serde_json::from_str(bytes)
        .map_err(|error| DagError::Parse(format!("invalid json: {error}")))?;
    if !raw.is_object() {
        return Err(DagError::Parse("top level must be an object".to_owned()));
    }
    if !raw.get("dependency_units").is_some_and(Value::is_array) {
        return Err(DagError::Parse(
            "missing `dependency_units` array".to_owned(),
        ));
    }
    if !raw.get("graphs").is_some_and(Value::is_array) {
        return Err(DagError::Parse("missing `graphs` array".to_owned()));
    }
    Ok(Dag { raw })
}

pub fn evaluate(dag: &Dag, schema: &Value, capability_registry: &Value) -> Report {
    evaluate_with_raw(dag, &dag.raw, schema, capability_registry)
}

pub fn evaluate_with_raw(
    _dag: &Dag,
    raw: &Value,
    schema: &Value,
    capability_registry: &Value,
) -> Report {
    let mut findings = Vec::new();
    check_schema_authority(schema, &mut findings);
    check_top_level(raw, &mut findings);
    let capabilities = check_capability_registry(capability_registry, &mut findings);
    let units = check_dependency_units(raw, &capabilities, &mut findings);
    let external_anchors = check_external_anchors(raw, &mut findings);
    let endpoints: BTreeSet<String> = units.union(&external_anchors).cloned().collect();
    let graph_map = check_graph_set(raw, &mut findings);

    let mut steady_edges = Vec::new();
    let mut bootstrap_order = Vec::new();
    let mut forbidden_edges = Vec::new();
    let mut declared_failure = Vec::new();

    for (kind, graph) in &graph_map {
        check_graph_shape(kind, graph, &mut findings);
        let edges = graph.get("edges").and_then(Value::as_array);
        let Some(edges) = edges else {
            findings.push(finding(
                "dag_schema_violation",
                kind,
                "graph missing `edges` array",
            ));
            continue;
        };
        let mut seen_edges = BTreeSet::new();
        for (index, edge) in edges.iter().enumerate() {
            check_edge(
                kind,
                index,
                edge,
                &endpoints,
                &external_anchors,
                &mut steady_edges,
                &mut declared_failure,
                &mut seen_edges,
                &mut findings,
            );
        }
        if kind == "steady_state_request" {
            bootstrap_order = string_array(graph.get("bootstrap_order"));
            forbidden_edges = parse_forbidden_edges(graph, &units, &mut findings);
        }
    }

    let request_units: BTreeSet<String> = steady_edges
        .iter()
        .flat_map(|edge: &RequestEdge| [edge.from.clone(), edge.to.clone()])
        .collect();
    let request_pairs: Vec<(String, String)> = steady_edges
        .iter()
        .map(|edge| (edge.from.clone(), edge.to.clone()))
        .collect();

    let sccs = tarjan_sccs(&request_units, &request_pairs);
    for component in sccs {
        let self_loop = component.len() == 1
            && request_pairs
                .iter()
                .any(|(from, to)| from == &component[0] && to == &component[0]);
        if component.len() > 1 || self_loop {
            findings.push(finding(
                "dag_cycle",
                component.join(" -> "),
                "steady_state_request contains a directed cycle; other graph kinds are not subject to this invariant",
            ));
        }
    }

    let derived = kahn_dependency_first(&request_units, &request_pairs);
    if let Some(order) = &derived {
        if let Some(reason) =
            validate_bootstrap_order(&request_units, &request_pairs, &bootstrap_order)
        {
            findings.push(finding("dag_bootstrap_drift", "bootstrap_order", reason));
        }
        if order.len() != request_units.len() {
            findings.push(finding(
                "dag_bootstrap_drift",
                "bootstrap_order",
                "derived order does not cover every steady-state dependency unit",
            ));
        }
    } else {
        findings.push(finding(
            "dag_bootstrap_drift",
            "bootstrap_order",
            "no dependency-first topological order exists",
        ));
    }

    let request_set: BTreeSet<(String, String)> = request_pairs.iter().cloned().collect();
    for (from, to) in forbidden_edges {
        if request_set.contains(&(from.clone(), to.clone())) {
            findings.push(finding(
                "dag_forbidden_edge",
                format!("{from}->{to}"),
                "steady-state edge is present but explicitly forbidden",
            ));
        }
    }

    check_failure_closure(&steady_edges, &declared_failure, &mut findings);

    findings.sort();
    findings.dedup();
    Report {
        verdict: if findings.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        },
        findings,
        derived_bootstrap_order: derived,
    }
}

fn check_schema_authority(schema: &Value, findings: &mut Vec<Finding>) {
    let digest = serde_json::to_vec(schema)
        .ok()
        .map(|bytes| format!("{:x}", Sha256::digest(bytes)));
    if schema.get("$schema").and_then(Value::as_str)
        != Some("https://json-schema.org/draft/2020-12/schema")
        || digest.as_deref() != Some(SCHEMA_CANONICAL_SHA256)
    {
        findings.push(finding(
            "dag_schema_authority_mismatch",
            "substrate-dependency-dag.schema.json",
            format!(
                "schema must be the reviewed Draft 2020-12 authority with sha256 {}; got {:?}",
                SCHEMA_CANONICAL_SHA256, digest
            ),
        ));
    }
}

fn check_top_level(raw: &Value, findings: &mut Vec<Finding>) {
    let allowed: BTreeSet<&str> = [
        "_comment",
        "$schema",
        "schema",
        "version",
        "doctrine_adrs",
        "external_anchors",
        "dependency_units",
        "failure_impact_composition",
        "graphs",
        "mandatory_follow_ups",
    ]
    .into_iter()
    .collect();
    check_closed_object("document", raw, &allowed, "dag_schema_violation", findings);
    check_required_properties(
        "document",
        raw,
        &[
            "$schema",
            "schema",
            "version",
            "doctrine_adrs",
            "external_anchors",
            "dependency_units",
            "failure_impact_composition",
            "graphs",
            "mandatory_follow_ups",
        ],
        "dag_schema_violation",
        findings,
    );
    if raw.get("_comment").is_some_and(|value| !value.is_string()) {
        findings.push(finding(
            "dag_schema_violation",
            "_comment",
            "optional comment must be a string",
        ));
    }
    for (key, expected) in [
        ("$schema", "https://json-schema.org/draft/2020-12/schema"),
        ("schema", "specs/substrate-dependency-dag.schema.json"),
        ("version", "2.0.0"),
    ] {
        if raw.get(key).and_then(Value::as_str) != Some(expected) {
            findings.push(finding(
                "dag_schema_violation",
                key,
                format!("must equal `{expected}`"),
            ));
        }
    }
    let doctrine = raw.get("doctrine_adrs").and_then(Value::as_array);
    let doctrine_matches = doctrine.is_some_and(|items| {
        items.len() == GRAPH_DOCTRINE_ADRS.len()
            && items
                .iter()
                .zip(GRAPH_DOCTRINE_ADRS)
                .all(|(actual, expected)| actual.as_str() == Some(expected))
    });
    if !doctrine_matches {
        findings.push(finding(
            "dag_schema_violation",
            "doctrine_adrs",
            format!("must equal {GRAPH_DOCTRINE_ADRS:?}"),
        ));
    }

    let composition = raw.get("failure_impact_composition");
    let null = Value::Null;
    let composition_value = composition.unwrap_or(&null);
    let composition_allowed: BTreeSet<&str> = [
        "path_rule",
        "multi_path_rule",
        "severity_order",
        "closure_direction",
    ]
    .into_iter()
    .collect();
    check_closed_object(
        "failure_impact_composition",
        composition_value,
        &composition_allowed,
        "dag_schema_violation",
        findings,
    );
    check_required_properties(
        "failure_impact_composition",
        composition_value,
        &[
            "path_rule",
            "multi_path_rule",
            "severity_order",
            "closure_direction",
        ],
        "dag_schema_violation",
        findings,
    );
    for (key, expected) in [
        ("path_rule", PATH_RULE),
        ("multi_path_rule", MULTI_PATH_RULE),
        ("closure_direction", CLOSURE_DIRECTION),
    ] {
        if composition
            .and_then(|value| value.get(key))
            .and_then(Value::as_str)
            != Some(expected)
        {
            findings.push(finding(
                "dag_schema_violation",
                format!("failure_impact_composition.{key}"),
                format!("must equal `{expected}`"),
            ));
        }
    }
    let severity = composition
        .and_then(|value| value.get("severity_order"))
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>());
    if severity.as_deref() != Some(IMPACT_RULES.as_slice()) {
        findings.push(finding(
            "dag_schema_violation",
            "failure_impact_composition.severity_order",
            "must declare INDEPENDENT < BROWNOUT < DEGRADED < FULL",
        ));
    }
    let follow_up_items = raw.get("mandatory_follow_ups").and_then(Value::as_array);
    let follow_ups: BTreeSet<&str> = follow_up_items
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("id").and_then(Value::as_str))
        .collect();
    let expected: BTreeSet<&str> = [
        "W0-C-MODULE-MEMBERSHIP",
        "W0-C-LAYER-RANKS",
        "W0-C-TOPOLOGY-COVERAGE",
    ]
    .into_iter()
    .collect();
    if follow_up_items.is_none_or(|items| items.len() != 3) || follow_ups != expected {
        findings.push(finding(
            "dag_schema_violation",
            "mandatory_follow_ups",
            "must carry module-membership, layer-rank, and topology-coverage migrations without baselines",
        ));
    }
    let allowed_follow_up: BTreeSet<&str> = [
        "id",
        "status",
        "tracking_issue",
        "baseline_policy",
        "constraint",
    ]
    .into_iter()
    .collect();
    for (index, item) in follow_up_items.into_iter().flatten().enumerate() {
        let subject = format!("mandatory_follow_ups[{index}]");
        check_closed_object(
            &subject,
            item,
            &allowed_follow_up,
            "dag_schema_violation",
            findings,
        );
        check_required_properties(
            &subject,
            item,
            &["id", "status", "constraint"],
            "dag_schema_violation",
            findings,
        );
        if item.get("status").and_then(Value::as_str) != Some("required-not-in-this-slice")
            || !is_non_empty_string(item.get("constraint"))
        {
            findings.push(finding(
                "dag_schema_violation",
                &subject,
                "follow-up requires locked status and non-empty constraint",
            ));
        }
        if item.get("id").and_then(Value::as_str) == Some(TOPOLOGY_FOLLOW_UP_ID) {
            check_required_properties(
                &subject,
                item,
                &["tracking_issue", "baseline_policy"],
                "dag_follow_up_policy_drift",
                findings,
            );
            if item.get("tracking_issue").and_then(Value::as_str) != Some(TOPOLOGY_TRACKING_ISSUE)
                || item.get("baseline_policy").and_then(Value::as_str)
                    != Some(NO_NEW_BASELINE_POLICY)
            {
                findings.push(finding(
                    "dag_follow_up_policy_drift",
                    &subject,
                    format!(
                        "topology coverage must remain tracked by `{TOPOLOGY_TRACKING_ISSUE}` with `{NO_NEW_BASELINE_POLICY}`"
                    ),
                ));
            }
        } else if item.get("tracking_issue").is_some() || item.get("baseline_policy").is_some() {
            findings.push(finding(
                "dag_follow_up_policy_drift",
                &subject,
                "topology tracking and baseline policy fields belong only to W0-C-TOPOLOGY-COVERAGE",
            ));
        }
    }
}

fn check_capability_registry(registry: &Value, findings: &mut Vec<Finding>) -> BTreeSet<String> {
    let mut capabilities = BTreeSet::new();
    let rows = registry.get("capabilities").and_then(Value::as_array);
    if registry.get("closed").and_then(Value::as_bool) != Some(true)
        || registry.get("registry_kind").and_then(Value::as_str) != Some("capability")
        || rows.is_none_or(|items| items.len() != CAPABILITY_COUNT)
    {
        findings.push(finding(
            "dag_capability_registry_invalid",
            "capability_registry",
            "canonical registry must be closed, kind=capability, and contain exactly 24 rows",
        ));
    }
    for (index, row) in rows.into_iter().flatten().enumerate() {
        let Some(name) = row
            .get("name")
            .and_then(Value::as_str)
            .filter(|name| !name.is_empty())
        else {
            findings.push(finding(
                "dag_capability_registry_invalid",
                format!("capabilities[{index}]"),
                "capability row requires a non-empty name",
            ));
            continue;
        };
        if !capabilities.insert(name.to_owned()) {
            findings.push(finding(
                "dag_capability_registry_invalid",
                name,
                "canonical capability names must be unique",
            ));
        }
    }
    capabilities
}

fn check_external_anchors(raw: &Value, findings: &mut Vec<Finding>) -> BTreeSet<String> {
    let mut anchors = BTreeSet::new();
    let rows = raw.get("external_anchors").and_then(Value::as_array);
    if rows.is_none_or(|items| items.len() != 1) {
        findings.push(finding(
            "dag_schema_violation",
            "external_anchors",
            "requires exactly one E0 external anchor",
        ));
    }
    let allowed: BTreeSet<&str> = ["id", "plane", "purpose"].into_iter().collect();
    for (index, anchor) in rows.into_iter().flatten().enumerate() {
        let subject = format!("external_anchors[{index}]");
        check_closed_object(&subject, anchor, &allowed, "dag_schema_violation", findings);
        check_required_properties(
            &subject,
            anchor,
            &["id", "plane", "purpose"],
            "dag_schema_violation",
            findings,
        );
        let id = anchor.get("id").and_then(Value::as_str);
        if id.is_none_or(|id| !is_external_id(id))
            || anchor.get("plane").and_then(Value::as_str) != Some("E0")
            || !is_non_empty_string(anchor.get("purpose"))
        {
            findings.push(finding(
                "dag_schema_violation",
                &subject,
                "external anchor requires external.* id, plane E0, and non-empty purpose",
            ));
        }
        if let Some(id) = id
            && !anchors.insert(id.to_owned())
        {
            findings.push(finding(
                "dag_duplicate_unit",
                id,
                "external anchor ids must be unique",
            ));
        }
    }
    anchors
}

fn check_dependency_units(
    raw: &Value,
    capabilities: &BTreeSet<String>,
    findings: &mut Vec<Finding>,
) -> BTreeSet<String> {
    let mut units = BTreeSet::new();
    let mut declared_authority = BTreeSet::new();
    let allowed: BTreeSet<&str> = ["id", "capability", "runtime_face", "plane", "purpose"]
        .into_iter()
        .collect();
    let planes = ["B0", "C0", "C1", "C2", "G", "R"];
    let rows = raw.get("dependency_units").and_then(Value::as_array);
    if rows.is_none_or(|items| items.len() != DEPENDENCY_UNIT_COUNT) {
        findings.push(finding(
            "dag_dependency_unit_set",
            "dependency_units",
            "must contain exactly 19 unique internal dependency units",
        ));
    }
    for (index, unit) in rows.into_iter().flatten().enumerate() {
        let subject = format!("dependency_units[{index}]");
        check_closed_object(&subject, unit, &allowed, "dag_schema_violation", findings);
        check_required_properties(
            &subject,
            unit,
            &["id", "capability", "runtime_face", "plane", "purpose"],
            "dag_schema_violation",
            findings,
        );
        let id = unit.get("id").and_then(Value::as_str);
        let capability = unit.get("capability").and_then(Value::as_str);
        let runtime_face = unit.get("runtime_face").and_then(Value::as_str);
        let plane = unit.get("plane").and_then(Value::as_str);
        for field in ["id", "capability", "runtime_face", "plane", "purpose"] {
            if !is_non_empty_string(unit.get(field)) {
                findings.push(finding(
                    "dag_schema_violation",
                    &subject,
                    format!("missing non-empty string `{field}`"),
                ));
            }
        }
        if !unit
            .get("plane")
            .and_then(Value::as_str)
            .is_some_and(|plane| planes.contains(&plane))
        {
            findings.push(finding(
                "dag_schema_violation",
                &subject,
                "plane must be one of B0/C0/C1/C2/G/R; E0 belongs to external_anchors",
            ));
        }
        if let Some(capability) = capability
            && !capabilities.contains(capability)
        {
            findings.push(finding(
                "dag_unknown_capability",
                &subject,
                format!("capability `{capability}` is absent from the canonical registry"),
            ));
        }
        if id.is_some_and(|id| !is_dependency_unit_id(id)) {
            findings.push(finding(
                "dag_schema_violation",
                &subject,
                "id must be a lowercase dot-qualified dependency-unit identifier",
            ));
        }
        if let Some(id) = id
            && !units.insert(id.to_owned())
        {
            findings.push(finding(
                "dag_duplicate_unit",
                id,
                "dependency unit ids must be unique",
            ));
        }
        if let (Some(id), Some(capability), Some(runtime_face), Some(plane)) =
            (id, capability, runtime_face, plane)
        {
            declared_authority.insert((
                id.to_owned(),
                capability.to_owned(),
                runtime_face.to_owned(),
                plane.to_owned(),
            ));
        }
    }
    let expected_authority: BTreeSet<(String, String, String, String)> = DEPENDENCY_UNIT_AUTHORITY
        .iter()
        .map(|(id, capability, runtime_face, plane)| {
            (
                (*id).to_owned(),
                (*capability).to_owned(),
                (*runtime_face).to_owned(),
                (*plane).to_owned(),
            )
        })
        .collect();
    if declared_authority != expected_authority {
        findings.push(finding(
            "dag_dependency_unit_authority_mismatch",
            "dependency_units",
            "must equal the founder-authoritative closed set of 19 (id, capability, runtime_face, plane) tuples",
        ));
    }
    units
}

fn check_graph_set<'a>(raw: &'a Value, findings: &mut Vec<Finding>) -> BTreeMap<String, &'a Value> {
    let mut graphs = BTreeMap::new();
    let mut declared = Vec::new();
    for graph in raw
        .get("graphs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(kind) = graph.get("kind").and_then(Value::as_str) else {
            findings.push(finding(
                "dag_graph_kind_set",
                "graphs",
                "every graph requires a string kind",
            ));
            continue;
        };
        declared.push(kind.to_owned());
        if graphs.insert(kind.to_owned(), graph).is_some() {
            findings.push(finding(
                "dag_graph_kind_set",
                kind,
                "graph kind is duplicated",
            ));
        }
    }
    let expected: Vec<String> = GRAPH_KINDS.iter().map(|kind| (*kind).to_owned()).collect();
    if declared != expected {
        findings.push(finding(
            "dag_graph_kind_set",
            "graphs",
            format!("must contain exactly {GRAPH_KINDS:?} in canonical order; got {declared:?}"),
        ));
    }
    graphs
}

fn check_graph_shape(kind: &str, graph: &Value, findings: &mut Vec<Finding>) {
    let keys: &[&str] = match kind {
        "steady_state_request" => &[
            "kind",
            "edge_semantics",
            "bootstrap_order",
            "forbidden_edges_assertion",
            "edges",
        ],
        "failure_brownout_propagation" => &["kind", "edge_semantics", "composition", "edges"],
        _ => &["kind", "edge_semantics", "edges"],
    };
    let allowed: BTreeSet<&str> = keys.iter().copied().collect();
    check_closed_object(kind, graph, &allowed, "dag_schema_violation", findings);
    check_required_properties(kind, graph, keys, "dag_schema_violation", findings);
    if graph
        .get("edge_semantics")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        findings.push(finding(
            "dag_schema_violation",
            kind,
            "graph requires non-empty edge_semantics",
        ));
    }
    if kind == "failure_brownout_propagation"
        && graph.get("composition").and_then(Value::as_str) != Some("max_min")
    {
        findings.push(finding(
            "dag_schema_violation",
            kind,
            "failure graph composition must equal `max_min`",
        ));
    }
    if kind == "steady_state_request" {
        check_string_array(
            "steady_state_request.bootstrap_order",
            graph.get("bootstrap_order"),
            true,
            findings,
        );
        if !graph
            .get("forbidden_edges_assertion")
            .is_some_and(Value::is_array)
        {
            findings.push(finding(
                "dag_schema_violation",
                kind,
                "forbidden_edges_assertion must be an array",
            ));
        }
    }
}

#[derive(Debug, Clone)]
struct RequestEdge {
    from: String,
    to: String,
    rule: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct FailureEdge {
    failed: String,
    impacted: String,
    rule: u8,
}

#[allow(clippy::too_many_arguments)]
fn check_edge(
    kind: &str,
    index: usize,
    edge: &Value,
    endpoints: &BTreeSet<String>,
    external_anchors: &BTreeSet<String>,
    steady: &mut Vec<RequestEdge>,
    failure: &mut Vec<FailureEdge>,
    seen_edges: &mut BTreeSet<String>,
    findings: &mut Vec<Finding>,
) {
    let subject = format!("{kind}.edges[{index}]");
    let declared_kind = edge.get("graph_kind").and_then(Value::as_str);
    if declared_kind != Some(kind) {
        findings.push(finding(
            "dag_cross_kind_edge",
            &subject,
            format!("edge graph_kind {declared_kind:?} must match containing graph `{kind}`"),
        ));
    }

    if kind == "failure_brownout_propagation" {
        let allowed: BTreeSet<&str> = [
            "graph_kind",
            "failed_unit",
            "impacted_unit",
            "impact_rule",
            "derivation",
        ]
        .into_iter()
        .collect();
        check_closed_object(
            &subject,
            edge,
            &allowed,
            "dag_failure_edge_malformed",
            findings,
        );
        check_required_properties(
            &subject,
            edge,
            &[
                "graph_kind",
                "failed_unit",
                "impacted_unit",
                "impact_rule",
                "derivation",
            ],
            "dag_failure_edge_malformed",
            findings,
        );
        let failed = edge.get("failed_unit").and_then(Value::as_str);
        let impacted = edge.get("impacted_unit").and_then(Value::as_str);
        let rule = edge
            .get("impact_rule")
            .and_then(Value::as_str)
            .and_then(impact_rank);
        let derivation = edge.get("derivation").and_then(Value::as_str);
        if failed.is_none_or(str::is_empty)
            || impacted.is_none_or(str::is_empty)
            || rule.is_none()
            || derivation != Some("max_min_reverse_transitive_closure")
        {
            findings.push(finding(
                "dag_failure_edge_malformed",
                &subject,
                "failure edge requires failed_unit, impacted_unit, a valid impact_rule, and max-min derivation",
            ));
            return;
        }
        let failed = failed.unwrap_or_default();
        let impacted = impacted.unwrap_or_default();
        for endpoint in [failed, impacted] {
            if !endpoints.contains(endpoint) {
                findings.push(finding(
                    "dag_edge_unknown_unit",
                    &subject,
                    format!("endpoint `{endpoint}` is not a declared dependency unit"),
                ));
            }
        }
        let identity = format!("{failed}->{impacted}");
        if !seen_edges.insert(identity.clone()) {
            findings.push(finding(
                "dag_duplicate_edge",
                identity,
                "duplicate failure edge",
            ));
        }
        if let Some(rule) = rule {
            failure.push(FailureEdge {
                failed: failed.to_owned(),
                impacted: impacted.to_owned(),
                rule,
            });
        }
        return;
    }

    let allowed: BTreeSet<&str> = if kind == "steady_state_request" {
        [
            "graph_kind",
            "from",
            "to",
            "dependency_weight",
            "cascade_rule",
            "version_compatibility_range",
            "cedar_permit_fragment",
            "rationale",
        ]
        .into_iter()
        .collect()
    } else {
        ["graph_kind", "from", "to", "rationale"]
            .into_iter()
            .collect()
    };
    check_closed_object(&subject, edge, &allowed, "dag_edge_malformed", findings);
    check_required_properties(
        &subject,
        edge,
        if kind == "steady_state_request" {
            &[
                "graph_kind",
                "from",
                "to",
                "dependency_weight",
                "cascade_rule",
                "version_compatibility_range",
                "cedar_permit_fragment",
            ]
        } else {
            &["graph_kind", "from", "to"]
        },
        "dag_edge_malformed",
        findings,
    );
    if edge
        .get("rationale")
        .is_some_and(|value| !value.is_string())
    {
        findings.push(finding(
            "dag_edge_malformed",
            &subject,
            "optional rationale must be a string",
        ));
    }
    let from = edge.get("from").and_then(Value::as_str);
    let to = edge.get("to").and_then(Value::as_str);
    let (Some(from), Some(to)) = (
        from.filter(|value| !value.is_empty()),
        to.filter(|value| !value.is_empty()),
    ) else {
        findings.push(finding(
            "dag_edge_malformed",
            &subject,
            "edge requires string from and to endpoints",
        ));
        return;
    };
    for endpoint in [from, to] {
        if !endpoints.contains(endpoint) {
            findings.push(finding(
                "dag_edge_unknown_unit",
                &subject,
                format!("endpoint `{endpoint}` is not a declared dependency unit"),
            ));
        }
    }
    if kind != "genesis"
        && [from, to]
            .iter()
            .any(|endpoint| external_anchors.contains(*endpoint))
    {
        findings.push(finding(
            "dag_edge_unknown_unit",
            &subject,
            "external anchors are valid only in the genesis graph",
        ));
    }
    let identity = format!("{from}->{to}");
    if !seen_edges.insert(identity.clone()) {
        findings.push(finding(
            "dag_duplicate_edge",
            format!("{kind}:{identity}"),
            "duplicate edge in graph kind",
        ));
    }
    if kind == "steady_state_request" {
        let rule = edge
            .get("cascade_rule")
            .and_then(Value::as_str)
            .and_then(impact_rank);
        let weight = edge.get("dependency_weight").and_then(Value::as_f64);
        let metadata_valid = weight.is_some_and(|weight| weight > 0.0 && weight <= 1.0)
            && rule.is_some()
            && is_non_empty_string(edge.get("version_compatibility_range"))
            && is_non_empty_string(edge.get("cedar_permit_fragment"));
        if !metadata_valid {
            findings.push(finding(
                "dag_edge_malformed",
                &subject,
                "steady-state metadata requires weight number in (0,1], valid cascade_rule, and non-empty version/Cedar strings",
            ));
        }
        if let Some(rule) = rule {
            steady.push(RequestEdge {
                from: from.to_owned(),
                to: to.to_owned(),
                rule,
            });
        }
    }
}

fn parse_forbidden_edges(
    graph: &Value,
    units: &BTreeSet<String>,
    findings: &mut Vec<Finding>,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (index, edge) in graph
        .get("forbidden_edges_assertion")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let subject = format!("steady_state_request.forbidden_edges_assertion[{index}]");
        let allowed: BTreeSet<&str> = ["from", "to", "reason"].into_iter().collect();
        check_closed_object(&subject, edge, &allowed, "dag_schema_violation", findings);
        check_required_properties(
            &subject,
            edge,
            &["from", "to", "reason"],
            "dag_schema_violation",
            findings,
        );
        let from = edge.get("from").and_then(Value::as_str);
        let to = edge.get("to").and_then(Value::as_str);
        let reason_valid = is_non_empty_string(edge.get("reason"));
        if !reason_valid {
            findings.push(finding(
                "dag_schema_violation",
                &subject,
                "forbidden edge requires a non-empty string reason",
            ));
        }
        if let (Some(from), Some(to)) = (
            from.filter(|value| !value.is_empty()),
            to.filter(|value| !value.is_empty()),
        ) {
            for endpoint in [from, to] {
                if !units.contains(endpoint) {
                    findings.push(finding(
                        "dag_edge_unknown_unit",
                        &subject,
                        format!("endpoint `{endpoint}` is not declared"),
                    ));
                }
            }
            out.push((from.to_owned(), to.to_owned()));
        } else {
            findings.push(finding(
                "dag_schema_violation",
                subject,
                "forbidden edge requires from and to",
            ));
        }
    }
    out
}

fn check_closed_object(
    subject: &str,
    value: &Value,
    allowed: &BTreeSet<&str>,
    code: &str,
    findings: &mut Vec<Finding>,
) {
    let Some(object) = value.as_object() else {
        findings.push(finding(code, subject, "must be an object"));
        return;
    };
    for key in object.keys() {
        if !allowed.contains(key.as_str()) {
            findings.push(finding(
                code,
                subject,
                format!("closed schema rejects property `{key}`"),
            ));
        }
    }
}

fn check_required_properties(
    subject: &str,
    value: &Value,
    required: &[&str],
    code: &str,
    findings: &mut Vec<Finding>,
) {
    let Some(object) = value.as_object() else {
        return;
    };
    for key in required {
        if !object.contains_key(*key) {
            findings.push(finding(
                code,
                subject,
                format!("missing required property `{key}`"),
            ));
        }
    }
}

fn check_string_array(
    subject: &str,
    value: Option<&Value>,
    require_non_empty: bool,
    findings: &mut Vec<Finding>,
) {
    let Some(items) = value.and_then(Value::as_array) else {
        findings.push(finding("dag_schema_violation", subject, "must be an array"));
        return;
    };
    let strings: Vec<&str> = items.iter().filter_map(Value::as_str).collect();
    if strings.len() != items.len()
        || (require_non_empty && strings.is_empty())
        || strings.iter().any(|item| item.is_empty())
        || strings.iter().copied().collect::<BTreeSet<_>>().len() != strings.len()
    {
        findings.push(finding(
            "dag_schema_violation",
            subject,
            "must contain unique non-empty strings",
        ));
    }
}

fn is_non_empty_string(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_str)
        .is_some_and(|value| !value.is_empty())
}

fn is_dependency_unit_id(value: &str) -> bool {
    let mut segments = value.split('.');
    let first = segments.next();
    let rest: Vec<&str> = segments.collect();
    first.is_some_and(is_slug) && !rest.is_empty() && rest.iter().copied().all(is_slug)
}

fn is_external_id(value: &str) -> bool {
    value
        .strip_prefix("external.")
        .is_some_and(|suffix| !suffix.is_empty() && suffix.split('.').all(is_slug))
}

fn is_slug(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn impact_rank(rule: &str) -> Option<u8> {
    IMPACT_RULES
        .iter()
        .position(|candidate| candidate == &rule)
        .and_then(|index| u8::try_from(index).ok())
}

fn impact_name(rank: u8) -> &'static str {
    IMPACT_RULES
        .get(usize::from(rank))
        .copied()
        .unwrap_or("INDEPENDENT")
}

fn check_failure_closure(
    steady: &[RequestEdge],
    declared: &[FailureEdge],
    findings: &mut Vec<Finding>,
) {
    let expected = derive_failure_closure(steady);
    let declared_set: BTreeSet<FailureEdge> = declared.iter().cloned().collect();
    if declared.len() != declared_set.len() {
        findings.push(finding(
            "dag_failure_closure_mismatch",
            "failure_brownout_propagation",
            "failure closure contains duplicate entries",
        ));
    }
    if expected != declared_set {
        let missing: Vec<String> = expected
            .difference(&declared_set)
            .map(|edge| {
                format!(
                    "{}->{}:{}",
                    edge.failed,
                    edge.impacted,
                    impact_name(edge.rule)
                )
            })
            .collect();
        let extra: Vec<String> = declared_set
            .difference(&expected)
            .map(|edge| {
                format!(
                    "{}->{}:{}",
                    edge.failed,
                    edge.impacted,
                    impact_name(edge.rule)
                )
            })
            .collect();
        findings.push(finding(
            "dag_failure_closure_mismatch",
            "failure_brownout_propagation",
            format!("must equal max-min reverse closure; missing={missing:?}; extra={extra:?}"),
        ));
    }
}

fn derive_failure_closure(steady: &[RequestEdge]) -> BTreeSet<FailureEdge> {
    let mut units = BTreeSet::new();
    let mut widest: BTreeMap<(String, String), u8> = BTreeMap::new();
    for edge in steady {
        units.insert(edge.from.clone());
        units.insert(edge.to.clone());
        let key = (edge.from.clone(), edge.to.clone());
        widest
            .entry(key)
            .and_modify(|rank| *rank = (*rank).max(edge.rule))
            .or_insert(edge.rule);
    }
    let ordered: Vec<String> = units.into_iter().collect();
    for via in &ordered {
        for from in &ordered {
            let Some(left) = widest.get(&(from.clone(), via.clone())).copied() else {
                continue;
            };
            for to in &ordered {
                let Some(right) = widest.get(&(via.clone(), to.clone())).copied() else {
                    continue;
                };
                let candidate = left.min(right);
                widest
                    .entry((from.clone(), to.clone()))
                    .and_modify(|rank| *rank = (*rank).max(candidate))
                    .or_insert(candidate);
            }
        }
    }
    widest
        .into_iter()
        .filter(|((impacted, failed), _)| impacted != failed)
        .map(|((impacted, failed), rule)| FailureEdge {
            failed,
            impacted,
            rule,
        })
        .collect()
}

fn validate_bootstrap_order(
    units: &BTreeSet<String>,
    edges: &[(String, String)],
    order: &[String],
) -> Option<String> {
    if order.len() != units.len() {
        return Some(format!(
            "contains {} entries but steady-state graph has {} units",
            order.len(),
            units.len()
        ));
    }
    let positions: BTreeMap<&str, usize> = order
        .iter()
        .enumerate()
        .map(|(index, unit)| (unit.as_str(), index))
        .collect();
    if positions.len() != units.len()
        || !units
            .iter()
            .all(|unit| positions.contains_key(unit.as_str()))
    {
        return Some("must contain every steady-state dependency unit exactly once".to_owned());
    }
    for (from, to) in edges {
        let dependent = positions.get(from.as_str()).copied();
        let dependency = positions.get(to.as_str()).copied();
        if let (Some(dependent), Some(dependency)) = (dependent, dependency)
            && dependency >= dependent
        {
            return Some(format!("dependency `{to}` must precede dependent `{from}`"));
        }
    }
    None
}

fn adjacency(
    units: &BTreeSet<String>,
    edges: &[(String, String)],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = units
        .iter()
        .map(|unit| (unit.clone(), BTreeSet::new()))
        .collect();
    for (from, to) in edges {
        if units.contains(from) && units.contains(to) {
            out.entry(from.clone()).or_default().insert(to.clone());
        }
    }
    out
}

fn kahn_dependency_first(
    units: &BTreeSet<String>,
    edges: &[(String, String)],
) -> Option<Vec<String>> {
    let mut remaining = adjacency(units, edges);
    let mut emitted = BTreeSet::new();
    let mut order = Vec::new();
    while order.len() < units.len() {
        let ready = remaining
            .iter()
            .find(|(unit, dependencies)| !emitted.contains(*unit) && dependencies.is_empty())
            .map(|(unit, _)| unit.clone());
        let unit = ready?;
        emitted.insert(unit.clone());
        order.push(unit.clone());
        for dependencies in remaining.values_mut() {
            dependencies.remove(&unit);
        }
    }
    Some(order)
}

fn tarjan_sccs(units: &BTreeSet<String>, edges: &[(String, String)]) -> Vec<Vec<String>> {
    let adj = adjacency(units, edges);
    let mut next_index = 0usize;
    let mut indices: BTreeMap<String, usize> = BTreeMap::new();
    let mut lowlinks: BTreeMap<String, usize> = BTreeMap::new();
    let mut on_stack = BTreeSet::new();
    let mut stack = Vec::new();
    let mut components = Vec::new();

    enum Step {
        Enter(String),
        Resume(String, usize),
    }
    let successors: BTreeMap<String, Vec<String>> = adj
        .into_iter()
        .map(|(unit, next)| (unit, next.into_iter().collect()))
        .collect();

    for start in units {
        if indices.contains_key(start) {
            continue;
        }
        let mut work = vec![Step::Enter(start.clone())];
        while let Some(step) = work.pop() {
            match step {
                Step::Enter(unit) => {
                    indices.insert(unit.clone(), next_index);
                    lowlinks.insert(unit.clone(), next_index);
                    next_index += 1;
                    stack.push(unit.clone());
                    on_stack.insert(unit.clone());
                    work.push(Step::Resume(unit, 0));
                }
                Step::Resume(unit, cursor) => {
                    let next = successors.get(&unit).map(Vec::as_slice).unwrap_or(&[]);
                    if cursor < next.len() {
                        let successor = next[cursor].clone();
                        work.push(Step::Resume(unit.clone(), cursor + 1));
                        if !indices.contains_key(&successor) {
                            work.push(Step::Enter(successor));
                        } else if on_stack.contains(&successor) {
                            let current = lowlinks.get(&unit).copied().unwrap_or(0);
                            let target = indices.get(&successor).copied().unwrap_or(0);
                            lowlinks.insert(unit, current.min(target));
                        }
                    } else {
                        let low = lowlinks.get(&unit).copied().unwrap_or(0);
                        let index = indices.get(&unit).copied().unwrap_or(0);
                        if low == index {
                            let mut component = Vec::new();
                            while let Some(member) = stack.pop() {
                                on_stack.remove(&member);
                                component.push(member.clone());
                                if member == unit {
                                    break;
                                }
                            }
                            component.sort();
                            components.push(component);
                        }
                        if let Some(Step::Resume(parent, _)) = work.last() {
                            let parent_low = lowlinks.get(parent).copied().unwrap_or(0);
                            let child_low = lowlinks.get(&unit).copied().unwrap_or(0);
                            let parent = parent.clone();
                            lowlinks.insert(parent, parent_low.min(child_low));
                        }
                    }
                }
            }
        }
    }
    components
}

pub fn render_findings(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return format!(
            "{GATE_ID}: GREEN — graph-v2 closed shape valid; steady-state acyclic; failure closure exact"
        );
    }
    let mut output = format!("{GATE_ID}: RED — {} finding(s):", findings.len());
    for item in findings {
        output.push_str(&format!(
            "\n  [{}] {}: {}",
            item.code, item.subject, item.detail
        ));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn request(from: &str, to: &str, rule: &str) -> RequestEdge {
        RequestEdge {
            from: from.to_owned(),
            to: to.to_owned(),
            rule: impact_rank(rule).expect("known rule"),
        }
    }

    #[test]
    fn max_min_closure_weakest_edge_bounds_path_and_strongest_path_wins() {
        let edges = [
            request("a", "b", "FULL"),
            request("b", "c", "BROWNOUT"),
            request("a", "d", "DEGRADED"),
            request("d", "c", "FULL"),
        ];
        let closure = derive_failure_closure(&edges);
        assert!(closure.contains(&FailureEdge {
            failed: "c".to_owned(),
            impacted: "a".to_owned(),
            rule: impact_rank("DEGRADED").unwrap(),
        }));
    }

    #[test]
    fn policy_requires_all_live_contract_paths() {
        let parsed = parse_policy(
            &json!({
                "gate_id": GATE_ID,
                "dag_path": "specs/dag.json",
                "schema_path": "specs/dag.schema.json",
                "capability_registry_path": "specs/capability-registry.json"
            })
            .to_string(),
        )
        .unwrap();
        assert_eq!(parsed.schema_path, "specs/dag.schema.json");
        assert_eq!(
            parsed.capability_registry_path,
            "specs/capability-registry.json"
        );
    }
}
