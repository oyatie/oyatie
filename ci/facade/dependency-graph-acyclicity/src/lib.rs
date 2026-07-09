//! # cloud-ci-substrate-dependency-dag-acyclicity (ADR-0280 §D-3)
//!
//! The principal enforcement surface for the substrate-of-substrate dependency doctrine
//! (ADR-0280, amended by ADR-0520 + ADR-0562). It loads the policy-declared substrate DAG and
//! proves the substrate dependency graph is a DAG:
//!
//! 1. **Schema shape** — every node carries the §D-1 `required` fields; every edge carries the
//!    §D-1 `required` fields with a `cascade_rule` in the allowed enum; endpoints reference
//!    declared nodes.
//! 2. **Acyclicity (Tarjan)** — strongly-connected-components; any SCC of size > 1 is a cycle
//!    and fails closed (ADR-0280 R-12: cycles are BLOCKER, no exception path).
//! 3. **Forbidden-edge honouring** — every `forbidden_edges_assertion {from,to,reason}` MUST NOT
//!    appear in `edges` (the negative-space invariant, e.g. the cell-leaf assertions and the
//!    cloud-secrets->identity bootstrap-only seam).
//! 4. **Topological-sort coherence (Kahn)** — `bootstrap_order` MUST equal Kahn's deterministic
//!    topo-sort (alphabetical tie-break on equal in-degree). The bootstrap order is DERIVED by
//!    querying the DAG, never hard-coded (ADR-0280 §D-4).
//!
//! ## Born pack-shaped (R0)
//! The crate is a NEUTRAL graph engine. Repo-specific adoption points live in
//! `substrate-dependency-dag-policy.json`; the Tarjan / Kahn / forbidden-edge / schema-shape
//! logic is pure and runs on any DAG document of this schema. The kernel fixes only the algorithm,
//! not the data.
//!
//! ## Kernel contract
//! - [`parse_policy`] `(bytes) -> Result<Policy, DagError>` reads the data-pack boundary.
//! - [`parse_dag`] `(bytes) -> Result<Dag, DagError>` is the DAG parse boundary (pure; no I/O).
//! - [`load_dag`] `(root, path) -> Result<Dag, DagError>` is the only I/O (read-only file read).
//! - [`evaluate`] `(&Dag) -> Report` is PURE and unit-testable without a filesystem; it keys
//!   every finding by a stable code.
//! - [`tarjan_sccs`] / [`kahn_topo_sort`] are pure graph primitives, directly testable.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `dag_parse_error`        — the document is not valid JSON / not the DAG schema shape.
//! - `dag_node_missing_field` — a node is missing a §D-1 `required` field.
//! - `dag_edge_missing_field` — an edge is missing a §D-1 `required` field.
//! - `dag_edge_bad_cascade`   — an edge `cascade_rule` is outside {FULL,DEGRADED,BROWNOUT,INDEPENDENT}.
//! - `dag_edge_unknown_node`  — an edge endpoint references an undeclared node.
//! - `dag_cycle`              — a strongly-connected component of size > 1 (a cycle).
//! - `dag_forbidden_edge`     — an edge present that a `forbidden_edges_assertion` forbids.
//! - `dag_bootstrap_drift`    — `bootstrap_order` != Kahn deterministic topological sort.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};

use serde_json::Value;

/// The gate id, matching the buck2 target stem + the doctrine.
pub const GATE_ID: &str = "cloud-ci-substrate-dependency-dag-acyclicity";

/// Default policy data-pack path, relative to the repo root.
pub const DEFAULT_POLICY_PATH: &str = "ci/facade/dependency-graph-acyclicity/substrate-dependency-dag-policy.json";

/// The §D-1 node `required` fields.
pub const NODE_REQUIRED_FIELDS: [&str; 6] = [
    "name",
    "tier_subtype",
    "dr_tier",
    "slo_floor",
    "brownout_protocol_version",
    "chaos_drill_cadence_days",
];

/// The §D-1 edge `required` fields.
pub const EDGE_REQUIRED_FIELDS: [&str; 6] = [
    "from",
    "to",
    "dependency_weight",
    "cascade_rule",
    "version_compatibility_range",
    "cedar_permit_fragment",
];

/// The §D-1 `cascade_rule` enum.
pub const CASCADE_RULES: [&str; 4] = ["FULL", "DEGRADED", "BROWNOUT", "INDEPENDENT"];

/// The blocking violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 8] = [
    "dag_parse_error",
    "dag_node_missing_field",
    "dag_edge_missing_field",
    "dag_edge_bad_cascade",
    "dag_edge_unknown_node",
    "dag_cycle",
    "dag_forbidden_edge",
    "dag_bootstrap_drift",
];

// ───────────────────────────── parsed DAG ─────────────────────────────

/// Policy data for this gate. Repo-specific adoption points belong here, not in Rust constants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    /// The gate id this policy configures; must equal [`GATE_ID`].
    pub gate_id: String,
    /// Repo-relative path to the substrate dependency DAG document.
    pub dag_path: String,
}

/// A parsed substrate dependency DAG: the node set (in declared order), the directed edges, the
/// declared bootstrap order, and the forbidden-edge negative-space assertions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dag {
    /// Node names in declared order.
    pub nodes: Vec<String>,
    /// Directed edges `(from, to)` in declared order. A `from` DEPENDS ON `to`.
    pub edges: Vec<(String, String)>,
    /// The declared bootstrap order.
    pub bootstrap_order: Vec<String>,
    /// The forbidden-edge assertions `(from, to)`.
    pub forbidden_edges: Vec<(String, String)>,
}

/// Why a document cannot be parsed into a [`Dag`]. Returned instead of panicking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DagError {
    /// Not valid JSON, or not the DAG schema shape (with a human-readable reason).
    Parse(String),
    /// The file could not be read (with a human-readable reason).
    Io(String),
}

impl std::fmt::Display for DagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DagError::Parse(reason) => write!(f, "dag parse error: {reason}"),
            DagError::Io(reason) => write!(f, "dag io error: {reason}"),
        }
    }
}

impl std::error::Error for DagError {}

// ───────────────────────────── findings + report ─────────────────────────────

/// A single coherence violation, keyed by code + a stable subject.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    /// One of [`VIOLATION_CODES`].
    pub code: String,
    /// A stable subject (e.g. an edge `from->to` or a node name).
    pub subject: String,
    /// Human-readable detail.
    pub detail: String,
}

/// The Green/Red verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

/// The evaluation report: ordered findings + the derived Kahn topo-sort + the verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub findings: Vec<Finding>,
    /// The Kahn deterministic topological sort (the DERIVED bootstrap order). `None` when a cycle
    /// makes a total order impossible.
    pub derived_bootstrap_order: Option<Vec<String>>,
    pub verdict: Verdict,
}

impl Report {
    fn from_findings(findings: Vec<Finding>, derived: Option<Vec<String>>) -> Self {
        let verdict = if findings.is_empty() {
            Verdict::Green
        } else {
            Verdict::Red
        };
        Report {
            findings,
            derived_bootstrap_order: derived,
            verdict,
        }
    }
}

// ───────────────────────────── parse boundary ─────────────────────────────

/// Read the policy document from `<root>/<path>` and parse it. Read-only I/O.
pub fn load_policy(root: &Path, path: &str) -> Result<Policy, DagError> {
    let full = root.join(path);
    let bytes =
        fs::read_to_string(&full).map_err(|e| DagError::Io(format!("{}: {e}", full.display())))?;
    parse_policy(&bytes)
}

/// Parse the gate policy from JSON bytes. PURE — no I/O. Fails closed if the policy points outside
/// the repo-relative data-pack boundary.
pub fn parse_policy(bytes: &str) -> Result<Policy, DagError> {
    let value: Value = serde_json::from_str(bytes)
        .map_err(|e| DagError::Parse(format!("invalid policy json: {e}")))?;
    let gate_id = required_string(&value, "gate_id")?;
    if gate_id != GATE_ID {
        return Err(DagError::Parse(format!(
            "policy gate_id `{gate_id}` does not match `{GATE_ID}`"
        )));
    }
    let dag_path = required_string(&value, "dag_path")?;
    validate_repo_relative_path("dag_path", dag_path)?;
    Ok(Policy {
        gate_id: gate_id.to_owned(),
        dag_path: dag_path.to_owned(),
    })
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, DagError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| DagError::Parse(format!("policy missing string `{key}`")))
}

fn validate_repo_relative_path(field: &str, path: &str) -> Result<(), DagError> {
    if path.trim().is_empty() {
        return Err(DagError::Parse(format!(
            "policy `{field}` must not be empty"
        )));
    }
    let parsed = Path::new(path);
    for component in parsed.components() {
        match component {
            Component::ParentDir => {
                return Err(DagError::Parse(format!(
                    "policy `{field}` must stay repo-relative and must not contain `..`: {path}"
                )));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(DagError::Parse(format!(
                    "policy `{field}` must be repo-relative, got {path}"
                )));
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }
    Ok(())
}

/// Read the DAG document from `<root>/<path>` and parse it. Read-only I/O.
pub fn load_dag(root: &Path, path: &str) -> Result<Dag, DagError> {
    let full = root.join(path);
    let bytes =
        fs::read_to_string(&full).map_err(|e| DagError::Io(format!("{}: {e}", full.display())))?;
    parse_dag(&bytes)
}

/// Parse a DAG document from JSON bytes. PURE — no I/O. Validates only the SHAPE needed to build
/// the graph; the field-completeness + cascade-enum + endpoint checks are findings in [`evaluate`]
/// so the gate surfaces ALL coherence problems rather than aborting on the first.
pub fn parse_dag(bytes: &str) -> Result<Dag, DagError> {
    let value: Value =
        serde_json::from_str(bytes).map_err(|e| DagError::Parse(format!("invalid json: {e}")))?;

    let nodes = parse_node_names(&value)?;
    let edges = parse_edge_endpoints(&value)?;
    let bootstrap_order = parse_string_array(&value, "bootstrap_order")?;
    let forbidden_edges = parse_forbidden_edges(&value)?;

    Ok(Dag {
        nodes,
        edges,
        bootstrap_order,
        forbidden_edges,
    })
}

fn parse_node_names(value: &Value) -> Result<Vec<String>, DagError> {
    let arr = value
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| DagError::Parse("missing `nodes` array".to_owned()))?;
    let mut out = Vec::with_capacity(arr.len());
    for (i, n) in arr.iter().enumerate() {
        let name = n
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| DagError::Parse(format!("node[{i}] missing string `name`")))?;
        out.push(name.to_owned());
    }
    Ok(out)
}

fn parse_edge_endpoints(value: &Value) -> Result<Vec<(String, String)>, DagError> {
    let arr = value
        .get("edges")
        .and_then(Value::as_array)
        .ok_or_else(|| DagError::Parse("missing `edges` array".to_owned()))?;
    let mut out = Vec::with_capacity(arr.len());
    for (i, e) in arr.iter().enumerate() {
        let from = e
            .get("from")
            .and_then(Value::as_str)
            .ok_or_else(|| DagError::Parse(format!("edge[{i}] missing string `from`")))?;
        let to = e
            .get("to")
            .and_then(Value::as_str)
            .ok_or_else(|| DagError::Parse(format!("edge[{i}] missing string `to`")))?;
        out.push((from.to_owned(), to.to_owned()));
    }
    Ok(out)
}

fn parse_string_array(value: &Value, key: &str) -> Result<Vec<String>, DagError> {
    let arr = value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| DagError::Parse(format!("missing `{key}` array")))?;
    let mut out = Vec::with_capacity(arr.len());
    for (i, v) in arr.iter().enumerate() {
        let s = v
            .as_str()
            .ok_or_else(|| DagError::Parse(format!("{key}[{i}] is not a string")))?;
        out.push(s.to_owned());
    }
    Ok(out)
}

fn parse_forbidden_edges(value: &Value) -> Result<Vec<(String, String)>, DagError> {
    // forbidden_edges_assertion is optional in the schema; treat absence as an empty set.
    let Some(arr) = value.get("forbidden_edges_assertion") else {
        return Ok(Vec::new());
    };
    let arr = arr
        .as_array()
        .ok_or_else(|| DagError::Parse("`forbidden_edges_assertion` is not an array".to_owned()))?;
    let mut out = Vec::with_capacity(arr.len());
    for (i, e) in arr.iter().enumerate() {
        let from = e.get("from").and_then(Value::as_str).ok_or_else(|| {
            DagError::Parse(format!(
                "forbidden_edges_assertion[{i}] missing string `from`"
            ))
        })?;
        let to = e.get("to").and_then(Value::as_str).ok_or_else(|| {
            DagError::Parse(format!(
                "forbidden_edges_assertion[{i}] missing string `to`"
            ))
        })?;
        out.push((from.to_owned(), to.to_owned()));
    }
    Ok(out)
}

// ───────────────────────────── schema-completeness checks ─────────────────────────────

/// Push a finding for every node/edge that omits a §D-1 `required` field, every edge whose
/// `cascade_rule` is outside the enum, and every edge endpoint that names an undeclared node.
/// PURE: re-parses the document `Value` (the caller already proved it parses) to inspect fields the
/// graph [`Dag`] discards. Keeps [`Dag`] minimal while still surfacing schema drift.
fn check_schema_completeness(value: &Value, dag: &Dag, findings: &mut Vec<Finding>) {
    let node_set: BTreeSet<&str> = dag.nodes.iter().map(String::as_str).collect();

    if let Some(nodes) = value.get("nodes").and_then(Value::as_array) {
        for n in nodes {
            let name = n.get("name").and_then(Value::as_str).unwrap_or("<unnamed>");
            for field in NODE_REQUIRED_FIELDS {
                if n.get(field).is_none() {
                    findings.push(Finding {
                        code: "dag_node_missing_field".to_owned(),
                        subject: name.to_owned(),
                        detail: format!("node `{name}` missing required field `{field}`"),
                    });
                }
            }
        }
    }

    if let Some(edges) = value.get("edges").and_then(Value::as_array) {
        for e in edges {
            let from = e.get("from").and_then(Value::as_str).unwrap_or("?");
            let to = e.get("to").and_then(Value::as_str).unwrap_or("?");
            let subject = format!("{from}->{to}");
            for field in EDGE_REQUIRED_FIELDS {
                if e.get(field).is_none() {
                    findings.push(Finding {
                        code: "dag_edge_missing_field".to_owned(),
                        subject: subject.clone(),
                        detail: format!("edge `{subject}` missing required field `{field}`"),
                    });
                }
            }
            match e.get("cascade_rule").and_then(Value::as_str) {
                Some(rule) if CASCADE_RULES.contains(&rule) => {}
                Some(rule) => findings.push(Finding {
                    code: "dag_edge_bad_cascade".to_owned(),
                    subject: subject.clone(),
                    detail: format!(
                        "edge `{subject}` cascade_rule `{rule}` not in {CASCADE_RULES:?}"
                    ),
                }),
                None => { /* already reported by the missing-field pass */ }
            }
            for (role, ep) in [("from", from), ("to", to)] {
                if !node_set.contains(ep) {
                    findings.push(Finding {
                        code: "dag_edge_unknown_node".to_owned(),
                        subject: subject.clone(),
                        detail: format!(
                            "edge `{subject}` `{role}` endpoint `{ep}` is not a declared node"
                        ),
                    });
                }
            }
        }
    }
}

// ───────────────────────────── graph primitives ─────────────────────────────

/// Build the adjacency map `from -> sorted unique tos`, restricted to declared nodes. Edges whose
/// endpoints are not declared nodes are dropped here (they are reported separately as
/// `dag_edge_unknown_node`) so the graph algorithms operate on a well-formed node set.
fn adjacency(dag: &Dag) -> BTreeMap<String, BTreeSet<String>> {
    let node_set: BTreeSet<&str> = dag.nodes.iter().map(String::as_str).collect();
    let mut adj: BTreeMap<String, BTreeSet<String>> = dag
        .nodes
        .iter()
        .map(|n| (n.clone(), BTreeSet::new()))
        .collect();
    for (from, to) in &dag.edges {
        if node_set.contains(from.as_str()) && node_set.contains(to.as_str()) {
            adj.entry(from.clone()).or_default().insert(to.clone());
        }
    }
    adj
}

/// Tarjan's strongly-connected-components. Returns the SCCs (each a sorted node set). Any SCC of
/// size > 1 is a cycle; a self-loop (a node with an edge to itself) is also returned as a size-1
/// SCC flagged via [`has_self_loop`]. Pure, O(V+E), iterative (no recursion → no stack overflow on
/// adversarial input, honouring the no-panic doctrine).
pub fn tarjan_sccs(dag: &Dag) -> Vec<Vec<String>> {
    let adj = adjacency(dag);
    let mut index_counter: usize = 0;
    let mut indices: BTreeMap<String, usize> = BTreeMap::new();
    let mut lowlink: BTreeMap<String, usize> = BTreeMap::new();
    let mut on_stack: BTreeSet<String> = BTreeSet::new();
    let mut stack: Vec<String> = Vec::new();
    let mut sccs: Vec<Vec<String>> = Vec::new();

    // Iterative Tarjan. Each work-stack frame tracks a node and a cursor into its successor list.
    enum Step {
        Enter(String),
        // Resume `node` after visiting child at successor-cursor `cursor`.
        Resume(String, usize),
    }

    // Deterministic node iteration order (BTreeMap keys are sorted).
    let node_order: Vec<String> = adj.keys().cloned().collect();
    let succs: BTreeMap<String, Vec<String>> = adj
        .iter()
        .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
        .collect();

    for start in &node_order {
        if indices.contains_key(start) {
            continue;
        }
        let mut work: Vec<Step> = vec![Step::Enter(start.clone())];
        while let Some(step) = work.pop() {
            match step {
                Step::Enter(v) => {
                    indices.insert(v.clone(), index_counter);
                    lowlink.insert(v.clone(), index_counter);
                    index_counter += 1;
                    stack.push(v.clone());
                    on_stack.insert(v.clone());
                    work.push(Step::Resume(v, 0));
                }
                Step::Resume(v, cursor) => {
                    let children = succs.get(&v).map(Vec::as_slice).unwrap_or(&[]);
                    if cursor < children.len() {
                        let w = children[cursor].clone();
                        // Schedule resumption at the next cursor, then descend (or fold-up).
                        work.push(Step::Resume(v.clone(), cursor + 1));
                        if !indices.contains_key(&w) {
                            work.push(Step::Enter(w));
                        } else if on_stack.contains(&w) {
                            let vi = *lowlink.get(&v).unwrap_or(&0);
                            let wi = *indices.get(&w).unwrap_or(&0);
                            lowlink.insert(v.clone(), vi.min(wi));
                        }
                    } else {
                        // All children done: if v is an SCC root, pop the component.
                        let vlow = *lowlink.get(&v).unwrap_or(&0);
                        let vidx = *indices.get(&v).unwrap_or(&0);
                        if vlow == vidx {
                            let mut component: Vec<String> = Vec::new();
                            while let Some(w) = stack.pop() {
                                on_stack.remove(&w);
                                component.push(w.clone());
                                if w == v {
                                    break;
                                }
                            }
                            component.sort();
                            sccs.push(component);
                        }
                        // Propagate lowlink to the parent (the frame just below, if it is our parent).
                        if let Some(Step::Resume(parent, _)) = work.last() {
                            let pi = *lowlink.get(parent).unwrap_or(&0);
                            let vi = *lowlink.get(&v).unwrap_or(&0);
                            let parent = parent.clone();
                            lowlink.insert(parent, pi.min(vi));
                        }
                    }
                }
            }
        }
    }
    sccs
}

/// True iff any declared node carries an edge to itself (a 1-cycle Tarjan returns as a size-1 SCC).
pub fn has_self_loop(dag: &Dag) -> Option<String> {
    let node_set: BTreeSet<&str> = dag.nodes.iter().map(String::as_str).collect();
    dag.edges
        .iter()
        .find(|(from, to)| from == to && node_set.contains(from.as_str()))
        .map(|(from, _)| from.clone())
}

/// Kahn's algorithm topological sort with an alphabetical tie-break on equal in-degree (the
/// ADR-0280 §D-4 deterministic rule). Returns `None` if the graph has a cycle (some nodes never
/// reach in-degree 0). Pure, O(V+E).
///
/// NOTE ON DIRECTION: an edge `from -> to` means `from` DEPENDS ON `to`, so `to` must bootstrap
/// BEFORE `from`. Kahn therefore peels nodes with zero OUT-degree-into-unbootstrapped first; we
/// model this as in-degree over the REVERSED edge set (a node is "ready" when all its dependencies
/// are already emitted), which yields the bootstrap order (leaf `cell` first).
pub fn kahn_topo_sort(dag: &Dag) -> Option<Vec<String>> {
    let adj = adjacency(dag); // from -> {to} (dependencies)
    // remaining[from] = set of not-yet-emitted dependencies of `from`.
    let mut remaining: BTreeMap<String, BTreeSet<String>> = adj.clone();
    let mut emitted: BTreeSet<String> = BTreeSet::new();
    let mut order: Vec<String> = Vec::new();
    let total = dag.nodes.len();

    while order.len() < total {
        // Ready = declared nodes, not yet emitted, with all dependencies already emitted.
        // Alphabetical tie-break: BTreeMap iteration is sorted; pick the FIRST ready node.
        let ready: Option<String> = remaining
            .iter()
            .filter(|(name, deps)| !emitted.contains(name.as_str()) && deps.is_empty())
            .map(|(name, _)| name.clone())
            .next();
        match ready {
            Some(node) => {
                emitted.insert(node.clone());
                order.push(node.clone());
                // Remove the just-emitted node from every other node's remaining deps.
                for deps in remaining.values_mut() {
                    deps.remove(&node);
                }
            }
            // No ready node but not all emitted => a cycle blocks a total order.
            None => return None,
        }
    }
    Some(order)
}

/// Why a declared `bootstrap_order` is not a valid topological order of the DAG. Used by the
/// bootstrap-coherence check (ADR-0280 §D-3 step 4). Returned as a human-readable reason so the
/// gate finding names the precise defect.
///
/// IMPORTANT (ADR-0280 R-5 / §D-4 bootstrap-seam subtlety): the §D-1 `bootstrap_order` is a
/// HAND-AUTHORED valid topological order that does NOT match a pure alphabetical-tie-break Kahn
/// sort — `cloud-secrets` depends only on `cell` (so alphabetically it would sort 2nd, before
/// `identity`) yet §D-1 places it at bootstrap step 5 because cloud-secrets is provisioned AFTER
/// identity via the Shamir-genesis BOOTSTRAP-ONLY SEAM (a non-runtime edge captured as a
/// forbidden_edges_assertion, not a DAG edge). The doctrine's load-bearing invariant is that the
/// declared order is *a* valid topological order (every dependency precedes its dependent), NOT
/// that it equals the alphabetical one. So the gate validates VALID-TOPO-ORDER, not equality; the
/// alphabetical Kahn sort is still derived and surfaced as the canonical suggestion.
pub fn validate_bootstrap_order(dag: &Dag) -> Option<String> {
    // 1. The declared order must be exactly the declared node set (no missing / extra / dup).
    let node_set: BTreeSet<&str> = dag.nodes.iter().map(String::as_str).collect();
    let order_set: BTreeSet<&str> = dag.bootstrap_order.iter().map(String::as_str).collect();
    if order_set.len() != dag.bootstrap_order.len() {
        return Some("bootstrap_order contains a duplicate node".to_owned());
    }
    if order_set != node_set {
        let missing: Vec<&str> = node_set.difference(&order_set).copied().collect();
        let extra: Vec<&str> = order_set.difference(&node_set).copied().collect();
        return Some(format!(
            "bootstrap_order node set != DAG node set (missing={missing:?}, extra={extra:?})"
        ));
    }
    // 2. For every edge `from -> to` (from DEPENDS ON to), `to` must appear BEFORE `from`.
    let position: BTreeMap<&str, usize> = dag
        .bootstrap_order
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();
    for (from, to) in &dag.edges {
        // Skip edges with undeclared endpoints (reported separately as dag_edge_unknown_node).
        let (Some(&fp), Some(&tp)) = (position.get(from.as_str()), position.get(to.as_str()))
        else {
            continue;
        };
        if tp >= fp {
            return Some(format!(
                "edge `{from}->{to}` (dependency) violates bootstrap_order: dependency `{to}` (step {}) must precede dependent `{from}` (step {})",
                tp + 1,
                fp + 1
            ));
        }
    }
    None
}

// ───────────────────────────── evaluation ─────────────────────────────

/// Evaluate a parsed [`Dag`] for every coherence invariant. PURE — no I/O. Surfaces ALL findings
/// (does not stop at the first) so a single run reports every problem.
pub fn evaluate(dag: &Dag) -> Report {
    // Re-serialize the structural Dag back into a Value for the field-completeness pass would lose
    // the discarded fields; instead the caller passes the raw document via evaluate_with_raw when a
    // full schema check is wanted. evaluate() runs the graph-level invariants over the Dag alone.
    evaluate_inner(dag, None)
}

/// Evaluate including the §D-1 field-completeness + cascade-enum + endpoint schema checks against
/// the raw parsed `Value` (the document the [`Dag`] was parsed from). This is what the live gate
/// runs; [`evaluate`] is the graph-only projection for tests that construct a [`Dag`] directly.
pub fn evaluate_with_raw(dag: &Dag, raw: &Value) -> Report {
    evaluate_inner(dag, Some(raw))
}

fn evaluate_inner(dag: &Dag, raw: Option<&Value>) -> Report {
    let mut findings: Vec<Finding> = Vec::new();

    if let Some(raw) = raw {
        check_schema_completeness(raw, dag, &mut findings);
    }

    // ── Acyclicity (Tarjan). Any SCC of size > 1 is a cycle; a self-loop is a 1-cycle.
    if let Some(node) = has_self_loop(dag) {
        findings.push(Finding {
            code: "dag_cycle".to_owned(),
            subject: format!("{node}->{node}"),
            detail: format!("self-loop on `{node}` (a 1-cycle; the graph must be acyclic)"),
        });
    }
    for scc in tarjan_sccs(dag) {
        if scc.len() > 1 {
            findings.push(Finding {
                code: "dag_cycle".to_owned(),
                subject: scc.join(","),
                detail: format!(
                    "strongly-connected component of size {} is a cycle: {}",
                    scc.len(),
                    scc.join(" -> ")
                ),
            });
        }
    }

    // ── Forbidden-edge honouring.
    let edge_set: BTreeSet<(&str, &str)> = dag
        .edges
        .iter()
        .map(|(f, t)| (f.as_str(), t.as_str()))
        .collect();
    for (from, to) in &dag.forbidden_edges {
        if edge_set.contains(&(from.as_str(), to.as_str())) {
            findings.push(Finding {
                code: "dag_forbidden_edge".to_owned(),
                subject: format!("{from}->{to}"),
                detail: format!(
                    "edge `{from}->{to}` is present but is asserted forbidden (negative-space invariant)"
                ),
            });
        }
    }

    // ── Topological-sort coherence (Kahn). The declared bootstrap_order MUST be a VALID
    //    topological order of the DAG (every dependency precedes its dependent), per ADR-0280 §D-3
    //    step 4. It need NOT equal the alphabetical-tie-break sort — §D-1's hand-authored order
    //    encodes the R-5 cloud-secrets bootstrap-seam subtlety that pure alphabetical sorting
    //    cannot (see `validate_bootstrap_order`). The alphabetical Kahn sort is still derived and
    //    surfaced as the canonical suggestion.
    let derived = kahn_topo_sort(dag);
    if derived.is_some() {
        if let Some(reason) = validate_bootstrap_order(dag) {
            findings.push(Finding {
                code: "dag_bootstrap_drift".to_owned(),
                subject: "bootstrap_order".to_owned(),
                detail: reason,
            });
        }
    } else {
        // A cycle already produced a dag_cycle finding; record the bootstrap consequence too.
        findings.push(Finding {
            code: "dag_bootstrap_drift".to_owned(),
            subject: "bootstrap_order".to_owned(),
            detail: "no topological sort exists (the graph has a cycle)".to_owned(),
        });
    }

    findings.sort();
    Report::from_findings(findings, derived)
}

/// Render findings as a deterministic multi-line report for the binary / CI logs.
pub fn render_findings(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return format!(
            "{GATE_ID}: GREEN — DAG acyclic (Tarjan); forbidden edges honoured; bootstrap_order is a valid topological order (Kahn)"
        );
    }
    let mut out = format!("{GATE_ID}: RED — {} finding(s):", findings.len());
    for f in findings {
        out.push_str(&format!("\n  [{}] {}: {}", f.code, f.subject, f.detail));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dag(
        nodes: &[&str],
        edges: &[(&str, &str)],
        bootstrap: &[&str],
        forbidden: &[(&str, &str)],
    ) -> Dag {
        Dag {
            nodes: nodes.iter().map(|s| s.to_string()).collect(),
            edges: edges
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
            bootstrap_order: bootstrap.iter().map(|s| s.to_string()).collect(),
            forbidden_edges: forbidden
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
        }
    }

    #[test]
    fn acyclic_chain_is_green() {
        // a -> b -> c (a depends on b depends on c). Bootstrap: c, b, a.
        let d = dag(
            &["a", "b", "c"],
            &[("a", "b"), ("b", "c")],
            &["c", "b", "a"],
            &[],
        );
        let report = evaluate(&d);
        assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
        assert_eq!(
            report.derived_bootstrap_order.as_deref(),
            Some(["c", "b", "a"].map(String::from).as_slice())
        );
    }

    #[test]
    fn two_node_cycle_is_red() {
        let d = dag(&["a", "b"], &[("a", "b"), ("b", "a")], &["a", "b"], &[]);
        let report = evaluate(&d);
        assert_eq!(report.verdict, Verdict::Red);
        assert!(report.findings.iter().any(|f| f.code == "dag_cycle"));
        assert!(report.derived_bootstrap_order.is_none());
    }

    #[test]
    fn three_node_cycle_is_red() {
        let d = dag(
            &["a", "b", "c"],
            &[("a", "b"), ("b", "c"), ("c", "a")],
            &["a", "b", "c"],
            &[],
        );
        let report = evaluate(&d);
        assert_eq!(report.verdict, Verdict::Red);
        assert!(report.findings.iter().any(|f| f.code == "dag_cycle"));
    }

    #[test]
    fn self_loop_is_red() {
        let d = dag(&["a"], &[("a", "a")], &["a"], &[]);
        let report = evaluate(&d);
        assert_eq!(report.verdict, Verdict::Red);
        assert!(report.findings.iter().any(|f| f.code == "dag_cycle"));
        assert_eq!(has_self_loop(&d).as_deref(), Some("a"));
    }

    #[test]
    fn six_node_buried_cycle_is_red() {
        // A long acyclic spine with one buried back-edge d -> b forming b->c->d->b.
        let d = dag(
            &["a", "b", "c", "d", "e", "f"],
            &[
                ("a", "b"),
                ("b", "c"),
                ("c", "d"),
                ("d", "b"), // buried back-edge
                ("d", "e"),
                ("e", "f"),
            ],
            &["f", "e", "d", "c", "b", "a"],
            &[],
        );
        let report = evaluate(&d);
        assert_eq!(report.verdict, Verdict::Red);
        let cycle = report
            .findings
            .iter()
            .find(|f| f.code == "dag_cycle")
            .expect("a cycle finding");
        // The SCC must contain exactly the buried 3-cycle members.
        assert_eq!(cycle.subject, "b,c,d", "{}", cycle.detail);
    }

    #[test]
    fn forbidden_edge_present_is_red() {
        let d = dag(
            &["a", "b"],
            &[("a", "b")],
            &["b", "a"],
            &[("a", "b")], // forbidding an edge that exists
        );
        let report = evaluate(&d);
        assert_eq!(report.verdict, Verdict::Red);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.code == "dag_forbidden_edge")
        );
    }

    #[test]
    fn bootstrap_drift_is_red() {
        // Acyclic, but the declared order is NOT a valid topological order: a depends on b (edge
        // a->b) yet a is placed BEFORE b, so a dependency follows its dependent.
        let d = dag(
            &["a", "b", "c"],
            &[("a", "b"), ("b", "c")],
            &["a", "b", "c"],
            &[],
        );
        let report = evaluate(&d);
        assert_eq!(report.verdict, Verdict::Red);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.code == "dag_bootstrap_drift")
        );
    }

    #[test]
    fn valid_but_non_alphabetical_bootstrap_order_is_green() {
        // The ADR-0280 R-5 / §D-4 bootstrap-seam scenario in miniature. Nodes c,i,t,s with deps
        // i->c, t->i, s->c. The declared order [c,i,t,s] places `s` LAST — a VALID topological
        // order (s's only dependency c precedes it). The pure alphabetical-tie-break Kahn sort
        // instead yields [c,i,s,t] (after c, both i and s are ready; i<s; then s, then t). The two
        // differ at the s/t positions, yet the declared order is still valid. The gate must accept
        // it (GREEN): valid-topo-order, not equality-to-alphabetical, is the invariant.
        let d = dag(
            &["c", "i", "t", "s"],
            &[("i", "c"), ("t", "i"), ("s", "c")],
            &["c", "i", "t", "s"], // valid, but not the alphabetical sort
            &[],
        );
        let report = evaluate(&d);
        assert_eq!(report.verdict, Verdict::Green, "{:?}", report.findings);
        // The alphabetical Kahn sort differs from the (valid) declared order — and that is fine.
        let derived = kahn_topo_sort(&d).expect("acyclic => topo-sort");
        assert_eq!(
            derived,
            ["c", "i", "s", "t"].map(String::from),
            "alphabetical Kahn sort"
        );
        assert_ne!(
            derived, d.bootstrap_order,
            "derived alphabetical sort diverges from the valid declared order"
        );
    }

    #[test]
    fn kahn_alphabetical_tiebreak_is_deterministic() {
        // Two independent leaves b and c, both dependencies of a. Tie-break is alphabetical.
        let d = dag(
            &["a", "b", "c"],
            &[("a", "b"), ("a", "c")],
            &["b", "c", "a"],
            &[],
        );
        assert_eq!(
            kahn_topo_sort(&d).as_deref(),
            Some(["b", "c", "a"].map(String::from).as_slice())
        );
    }

    #[test]
    fn parse_round_trips_minimal_document() {
        let doc = r#"{
          "version": "1.0.0",
          "nodes": [{"name": "a"}, {"name": "b"}],
          "edges": [{"from": "a", "to": "b"}],
          "bootstrap_order": ["b", "a"],
          "forbidden_edges_assertion": [{"from": "b", "to": "a", "reason": "x"}]
        }"#;
        let d = parse_dag(doc).expect("parse");
        assert_eq!(d.nodes, vec!["a", "b"]);
        assert_eq!(d.edges, vec![("a".to_string(), "b".to_string())]);
        assert_eq!(d.bootstrap_order, vec!["b", "a"]);
        assert_eq!(d.forbidden_edges, vec![("b".to_string(), "a".to_string())]);
    }

    #[test]
    fn policy_accepts_neutral_repo_relative_dag_path() {
        let doc = r#"{
          "gate_id": "cloud-ci-substrate-dependency-dag-acyclicity",
          "dag_path": "config/substrate/dag.json"
        }"#;
        let policy = parse_policy(doc).expect("neutral policy parses");
        assert_eq!(policy.gate_id, GATE_ID);
        assert_eq!(policy.dag_path, "config/substrate/dag.json");
    }

    #[test]
    fn policy_rejects_wrong_gate_id() {
        let doc = r#"{
          "gate_id": "other-gate",
          "dag_path": "config/substrate/dag.json"
        }"#;
        let error = parse_policy(doc).expect_err("wrong gate id must fail closed");
        assert!(
            error.to_string().contains("does not match"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn policy_rejects_path_escape() {
        let doc = r#"{
          "gate_id": "cloud-ci-substrate-dependency-dag-acyclicity",
          "dag_path": "../outside.json"
        }"#;
        let error = parse_policy(doc).expect_err("path escape must fail closed");
        assert!(
            error.to_string().contains("must not contain `..`"),
            "unexpected error: {error}"
        );
    }
}
