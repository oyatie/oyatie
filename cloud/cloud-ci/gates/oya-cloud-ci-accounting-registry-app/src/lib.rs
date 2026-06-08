//! # oya-cloud-ci-accounting-registry-app
//!
//! Generates `accounting-registry.generated.json` — one record per `git ls-files`
//! path (the tracked-truth discipline; PHASE-0-FIREWALL-PLAN §5.1) — plus the three
//! companion generated faces (`ttl-policy.generated.json`, `decision-crosswalk.generated.json`,
//! `enforcement-inventory.generated.json`). The producer is the buck2 `rust_binary`
//! that GATE-2 `cloud-ci-total-accounting` owns; it is NOT an `oya` CLI command
//! (register #20 — `oya gen`/`oya gate` authority is retired).
//!
//! ## Invariants (10-gates-registry §A.2)
//! 1. `committed == regenerated` — the output is fully deterministic (no wall-clock in
//!    the row data; `_provenance` carries a content digest, not a timestamp), so the
//!    `registry-drift` test can byte-diff a fresh run against the committed face.
//! 2. Total coverage — `set(rows.path) == set(git ls-files) − ephemeral` (ephemeral
//!    carve-out rows are excluded by CLASS, resolved from the DATA table, never by row).
//! 3. Carve-outs (vendor/generated/ephemeral/...) live as DATA in `unit-class-policy.json`
//!    and `ttl-policy.json`, never as scanner branches (Linus: the exception lives in the
//!    table). The classifier walks the table; it has zero hard-coded special cases.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The carve-out classification policy (DATA, not code).
pub const UNIT_CLASS_POLICY_JSON: &str = include_str!("unit-class-policy.json");
/// The TTL policy (DATA, not code).
pub const TTL_POLICY_JSON: &str = include_str!("ttl-policy.json");

/// The buck2 target that produces the registry — recorded in `_provenance`.
pub const PRODUCER_TARGET: &str = "//cloud/cloud-ci/gates:oya-cloud-ci-accounting-registry-app";

/// A producer error. No panics escape the production path.
#[derive(Debug)]
pub enum ProducerError {
    Policy(String),
    Serialize(String),
}

impl std::fmt::Display for ProducerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProducerError::Policy(message) => write!(f, "policy error: {message}"),
            ProducerError::Serialize(message) => write!(f, "serialize error: {message}"),
        }
    }
}

impl std::error::Error for ProducerError {}

/// One carve-out classification rule (a row in `unit-class-policy.json`).
#[derive(Debug, Clone, Deserialize)]
struct ClassRule {
    kind: String,
    value: String,
    unit_class: String,
}

/// The TTL record for a unit_class (a row in `ttl-policy.json`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TtlRecord {
    pub ttl_class: String,
    pub budget_days: Option<u64>,
    pub action: String,
    pub protected: bool,
}

/// The repo facts the producer needs, supplied by the binary (git plumbing) or by tests.
/// Keeping I/O out of the library makes the producer logic deterministic + unit-testable.
#[derive(Debug, Clone, Default)]
pub struct RepoInputs {
    /// Every `git ls-files` path, repo-relative.
    pub tracked_paths: Vec<String>,
    /// path -> last commit SHA touching it (`git log`).
    pub last_touch: BTreeMap<String, String>,
    /// path -> nearest up-tree OWNERS-resolved owner. Absent ⇒ unowned (RED).
    pub owners: BTreeMap<String, String>,
    /// path -> justification ref (ADR-####/spec $id/need:<ticket>). Absent ⇒ unjustified.
    pub justifications: BTreeMap<String, String>,
    /// path -> the registries that reach it (masterplan|root-hub|cargo-members|doc-catalog|crosswalk).
    pub reachability: BTreeMap<String, Vec<String>>,
    /// path -> canonical path it duplicates (drives the MERGE verdict). Absent ⇒ not a dup.
    pub dup_of: BTreeMap<String, String>,
}

/// The carve-out + TTL policy, parsed once from the DATA tables.
pub struct Policy {
    rules: Vec<ClassRule>,
    ttl_by_class: BTreeMap<String, TtlRecord>,
}

impl Policy {
    /// Parse the bundled DATA tables. Returns an error rather than panicking on malformed data.
    pub fn from_bundled() -> Result<Self, ProducerError> {
        Self::from_strs(UNIT_CLASS_POLICY_JSON, TTL_POLICY_JSON)
    }

    pub fn from_strs(unit_class_json: &str, ttl_json: &str) -> Result<Self, ProducerError> {
        let unit_value: Value = serde_json::from_str(unit_class_json)
            .map_err(|e| ProducerError::Policy(format!("unit-class-policy.json: {e}")))?;
        let rules_value = unit_value
            .get("rules")
            .ok_or_else(|| ProducerError::Policy("unit-class-policy.json missing 'rules'".into()))?;
        let rules: Vec<ClassRule> = serde_json::from_value(rules_value.clone())
            .map_err(|e| ProducerError::Policy(format!("rules: {e}")))?;

        let ttl_value: Value = serde_json::from_str(ttl_json)
            .map_err(|e| ProducerError::Policy(format!("ttl-policy.json: {e}")))?;
        let by_class_value = ttl_value
            .get("by_unit_class")
            .ok_or_else(|| ProducerError::Policy("ttl-policy.json missing 'by_unit_class'".into()))?;
        let ttl_by_class: BTreeMap<String, TtlRecord> =
            serde_json::from_value(by_class_value.clone())
                .map_err(|e| ProducerError::Policy(format!("by_unit_class: {e}")))?;

        Ok(Self {
            rules,
            ttl_by_class,
        })
    }

    /// Classify a path by walking the DATA table top-to-bottom (first match wins).
    /// There are NO hard-coded carve-outs here — every exception is a row above.
    pub fn classify(&self, path: &str) -> &str {
        for rule in &self.rules {
            let hit = match rule.kind.as_str() {
                "prefix" => path.starts_with(&rule.value),
                "suffix" => path.ends_with(&rule.value),
                "contains" => path.contains(&rule.value),
                "exact" => path == rule.value,
                _ => false,
            };
            if hit {
                return &rule.unit_class;
            }
        }
        // The DATA table's last rule (prefix "") is the husk catch-all; reaching here
        // only happens if that row is removed — treat as husk to stay total.
        "husk"
    }

    pub fn ttl_for(&self, unit_class: &str) -> Option<&TtlRecord> {
        self.ttl_by_class.get(unit_class)
    }

    /// The emitted `ttl-policy.generated.json` body (Gate-3 companion face).
    pub fn ttl_policy_face(&self) -> Value {
        let mut by_class = Map::new();
        for (class, record) in &self.ttl_by_class {
            by_class.insert(
                class.clone(),
                serde_json::to_value(record).unwrap_or(Value::Null),
            );
        }
        let mut root = Map::new();
        root.insert(
            "_comment".into(),
            Value::String(
                "GENERATED by oya-cloud-ci-accounting-registry-app from ttl-policy.json. committed==regenerated."
                    .into(),
            ),
        );
        root.insert("by_unit_class".into(), Value::Object(by_class));
        Value::Object(root)
    }
}

/// A single accounting record (the 11 fields of PHASE-0-FIREWALL-PLAN §5.1).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AccountingRecord {
    pub path: String,
    pub unit_class: String,
    pub owner: Option<String>,
    pub justification_ref: Option<String>,
    pub reachable_from: Vec<String>,
    pub ttl: TtlRecord,
    pub last_touch_commit: Option<String>,
    pub tracked: bool,
    pub verdict: String,
    pub dup_of: Option<String>,
    #[serde(rename = "_provenance")]
    pub provenance: RecordProvenance,
}

/// Per-record provenance — proves the row was generated, NOT hand-written.
/// Deliberately carries NO wall-clock so `committed == regenerated` holds byte-for-byte.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RecordProvenance {
    pub producer_target: String,
    pub source: String,
}

impl TtlRecord {
    fn unaccounted_placeholder() -> Self {
        Self {
            ttl_class: "husk".into(),
            budget_days: Some(14),
            action: "archive".into(),
            protected: false,
        }
    }
}

/// Derive the verdict for a record from its accounting facts (the rules over the
/// 11 fields; 10-gates-registry §A.1). Order matters: RED dominates.
fn derive_verdict(
    owner: &Option<String>,
    justification: &Option<String>,
    reachable: &[String],
    ttl: &TtlRecord,
    dup_of: &Option<String>,
) -> String {
    if dup_of.is_some() {
        return "MERGE".into();
    }
    // unjustified or unreachable ⇒ RED (the firewall blocks).
    if justification.is_none() || reachable.is_empty() {
        return "RED".into();
    }
    if owner.is_none() {
        return "NEEDS-OWNER".into();
    }
    // Over-TTL, unprotected, archive-action class ⇒ ARCHIVE candidate (REPORTED, not deleted).
    if !ttl.protected && ttl.action == "archive" {
        return "ARCHIVE".into();
    }
    "KEEP".into()
}

/// Build the full registry (rows + provenance) from repo inputs + policy.
/// Pure + deterministic: same inputs ⇒ byte-identical output.
pub fn build_registry(inputs: &RepoInputs, policy: &Policy) -> Result<Value, ProducerError> {
    let mut records: Vec<AccountingRecord> = Vec::new();

    for path in &inputs.tracked_paths {
        let unit_class = policy.classify(path).to_owned();

        // Coverage invariant #2: ephemeral is carved out by CLASS (from the DATA table),
        // excluded from the registry rows — never by an ad-hoc ignore list.
        if unit_class == "ephemeral" {
            continue;
        }

        let ttl = policy
            .ttl_for(&unit_class)
            .cloned()
            .unwrap_or_else(TtlRecord::unaccounted_placeholder);

        let owner = inputs.owners.get(path).cloned();
        let justification_ref = inputs.justifications.get(path).cloned();
        let reachable_from = inputs.reachability.get(path).cloned().unwrap_or_default();
        // A `generated` artifact has no meaningful last-touch: it is rewritten by THIS
        // producer, so recording `git log -1` of a generated face is self-referential —
        // committing the regenerated face changes its own last-touch, which no single
        // regen+commit can converge (registry-drift fixed-point). Emit None for the
        // generated class so the face is invariant to which commit holds it; the row is
        // still present (total-accounting stays whole). Non-generated last-touch unchanged.
        let last_touch_commit = if unit_class == "generated" {
            None
        } else {
            inputs.last_touch.get(path).cloned()
        };
        let dup_of = inputs.dup_of.get(path).cloned();

        let verdict = derive_verdict(&owner, &justification_ref, &reachable_from, &ttl, &dup_of);

        records.push(AccountingRecord {
            path: path.clone(),
            unit_class,
            owner,
            justification_ref,
            reachable_from,
            ttl,
            last_touch_commit,
            tracked: true,
            verdict,
            dup_of,
            provenance: RecordProvenance {
                producer_target: PRODUCER_TARGET.into(),
                source: "git ls-files × OWNERS × ADR-front-matter × specs × masterplan".into(),
            },
        });
    }

    // Deterministic row order (path-sorted) so committed==regenerated holds.
    records.sort_by(|a, b| a.path.cmp(&b.path));

    let rows = serde_json::to_value(&records)
        .map_err(|e| ProducerError::Serialize(format!("rows: {e}")))?;
    let source_inputs_digest = digest_rows(&rows);

    let mut root = Map::new();
    root.insert(
        "_comment".into(),
        Value::String(
            "GENERATED by oya-cloud-ci-accounting-registry-app. DO NOT HAND-EDIT — the registry-drift gate makes any hand-edit RED (committed==regenerated)."
                .into(),
        ),
    );
    root.insert(
        "_provenance".into(),
        serde_json::json!({
            "producer_target": PRODUCER_TARGET,
            "source_inputs_digest": source_inputs_digest,
            "row_count": records.len(),
        }),
    );
    root.insert("rows".into(), rows);
    Ok(Value::Object(root))
}

/// A stable, dependency-free FNV-1a 64-bit digest of the canonical row JSON.
/// Used as `_provenance.source_inputs_digest` so the face proves regeneration without
/// a wall-clock (which would break committed==regenerated).
fn digest_rows(rows: &Value) -> String {
    let canonical = serde_json::to_string(rows).unwrap_or_default();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in canonical.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

/// One decision's cross-artifact facts (GATE-1 `decision-crosswalk.generated.json`).
/// The binary fills these from the ADR front-matter + the spec/masterplan/roadmap faces.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct DecisionCrosswalkRow {
    pub id: String,
    pub status: String,
    pub in_spec: bool,
    pub in_masterplan: bool,
    pub in_roadmap: bool,
    pub supersedes: Vec<String>,
    pub superseded_by: Vec<String>,
}

/// The repo facts the GATE-1 face needs, supplied by the binary or by tests.
#[derive(Debug, Clone, Default)]
pub struct CrosswalkInputs {
    /// One row per decision id (path-sorted by the builder for determinism).
    pub decisions: Vec<DecisionCrosswalkRow>,
    /// Decision ids carried by more than one decision file (the dup-0377 exhibit).
    pub duplicate_ids: Vec<String>,
    /// Shared values two generated faces must agree on (the catalog/contracts axes_count
    /// drift exhibit), face-name -> value.
    pub generated_face_axes: BTreeMap<String, i64>,
}

/// Build the GATE-1 `decision-crosswalk.generated.json` face. Pure + deterministic:
/// the rows are sorted by id; the maps are BTreeMaps. The shape is exactly what the
/// GATE-1 evaluator consumes (`decisions` / `duplicate_ids` / `generated_face_axes`).
pub fn build_decision_crosswalk(inputs: &CrosswalkInputs) -> Result<Value, ProducerError> {
    let mut decisions = inputs.decisions.clone();
    decisions.sort_by(|a, b| a.id.cmp(&b.id));
    let decisions_value = serde_json::to_value(&decisions)
        .map_err(|e| ProducerError::Serialize(format!("decisions: {e}")))?;

    let mut duplicate_ids = inputs.duplicate_ids.clone();
    duplicate_ids.sort();
    duplicate_ids.dedup();

    let mut axes = Map::new();
    for (face, value) in &inputs.generated_face_axes {
        axes.insert(face.clone(), Value::from(*value));
    }

    let mut root = Map::new();
    root.insert(
        "_comment".into(),
        Value::String(
            "GENERATED by oya-cloud-ci-accounting-registry-app for GATE-1 cloud-ci-cross-artifact-agreement. \
             committed==regenerated (registry-drift byte-diffs it). DO NOT HAND-EDIT."
                .into(),
        ),
    );
    root.insert(
        "_provenance".into(),
        serde_json::json!({
            "producer_target": PRODUCER_TARGET,
            "owning_gate": "cloud-ci-cross-artifact-agreement",
            "decision_count": decisions.len(),
        }),
    );
    root.insert("duplicate_ids".into(), Value::Array(
        duplicate_ids.into_iter().map(Value::String).collect(),
    ));
    root.insert("generated_face_axes".into(), Value::Object(axes));
    root.insert("decisions".into(), decisions_value);
    Ok(Value::Object(root))
}

/// One enforcement surface's facts (GATE-4 `enforcement-inventory.generated.json`).
/// The binary fills these from the gate crates + governance lanes + ADR `verified_by` lines.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct EnforcementRow {
    pub id: String,
    pub source_artifact: String,
    /// Whether the surface CLAIMS to enforce/verify/gate.
    pub claims_enforced: bool,
    /// Whether a wired buck2 gate target backs the claim.
    pub has_wired_buck2_target: bool,
    /// Whether the surface routes a blocking invariant through an `oya` CLI invocation.
    pub maps_to_oya_cli: bool,
}

/// The repo facts the GATE-4 face needs, supplied by the binary or by tests.
#[derive(Debug, Clone, Default)]
pub struct EnforcementInputs {
    pub rows: Vec<EnforcementRow>,
}

/// Build the GATE-4 `enforcement-inventory.generated.json` face. Pure + deterministic:
/// the rows are sorted by id.
pub fn build_enforcement_inventory(inputs: &EnforcementInputs) -> Result<Value, ProducerError> {
    let mut rows = inputs.rows.clone();
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    let rows_value = serde_json::to_value(&rows)
        .map_err(|e| ProducerError::Serialize(format!("rows: {e}")))?;

    let mut root = Map::new();
    root.insert(
        "_comment".into(),
        Value::String(
            "GENERATED by oya-cloud-ci-accounting-registry-app for GATE-4 cloud-ci-automation-ratchet. \
             committed==regenerated (registry-drift byte-diffs it). DO NOT HAND-EDIT."
                .into(),
        ),
    );
    root.insert(
        "_provenance".into(),
        serde_json::json!({
            "producer_target": PRODUCER_TARGET,
            "owning_gate": "cloud-ci-automation-ratchet",
            "surface_count": rows.len(),
        }),
    );
    root.insert("rows".into(), rows_value);
    Ok(Value::Object(root))
}

// ---------------------------------------------------------------------------
// Fifth face: gate-baseline.generated.json (the GO-LIVE readiness ratchet)
// ---------------------------------------------------------------------------

/// The go-live disposition table (DATA, not code): per (gate, code) it carries the mode,
/// the infra_prereq, and the frozen_empty flag. Flipping advisory-until-infra to
/// baseline-block-on-new is a DATA edit here, never a code change.
pub const GATE_DISPOSITION_JSON: &str = include_str!("gate-disposition.json");

/// The buck2 target that runs the firewall ratchet — recorded in the baseline `_provenance`.
pub const FIREWALL_TARGET: &str = "//cloud/cloud-ci/gates:oya-cloud-ci-firewall-app";

/// The firewall gate ids, in canonical (baseline) order. The fifth gate
/// `cloud-ci-brand-residue` is the forbidden-vocab shrink-only ratchet (register #25,
/// boundary enforcement): its per-stem keys are the live residue files, frozen so any NEW
/// occurrence is RED while the historical residue ages out without churning history.
pub const GATE_IDS: [&str; 6] = [
    "cloud-ci-total-accounting",
    "cloud-ci-cross-artifact-agreement",
    "cloud-ci-automation-ratchet",
    "cloud-ci-staleness-reaper",
    "cloud-ci-bnf-layer-suffix",
    "cloud-ci-brand-residue",
];

/// The four live gate-input faces the baseline is captured over. Each is the exact
/// `Value` shape that the matching gate's `evaluate_keyed` consumes:
/// - `total_accounting`: the accounting registry (`rows` with path/owner/justification/…)
/// - `cross_artifact`: the decision crosswalk (`decisions`/`duplicate_ids`/`generated_face_axes`)
/// - `automation_ratchet`: the automation matrix (`rows`) joined with the enforcement face
/// - `staleness`: the registry rows aged with `age_days` (the binary supplies the aging)
pub struct GateInputs<'a> {
    pub total_accounting: &'a Value,
    pub cross_artifact: &'a Value,
    pub automation_ratchet: &'a Value,
    pub staleness: &'a Value,
    /// The §2.5#4 BNF layer-suffix gate input: `{"rows":[{"crate_name": "oya-..."}]}` —
    /// the first-party `oya-*` crate names the binary enumerates from the tracked Cargo.toml
    /// manifests. The gate's `evaluate_keyed` resolves the role carve-out-aware and reuses
    /// `oya_governance_predictable_naming_kernel::check`. Empty in unit tests.
    pub bnf_layer_suffix: &'a Value,
    /// The forbidden-vocab shrink-only ratchet's pre-grouped `code -> keys` (the live residue
    /// files per stem), captured by the binary via `oya_check_brand_residue::forbidden_vocab`
    /// over the live corpus. Unlike the four face gates this is computed from the raw tracked
    /// files (not a generated face), so it is supplied already grouped rather than re-derived
    /// here. Empty in unit tests that do not exercise the brand gate.
    pub brand_residue: &'a BTreeMap<String, BTreeSet<String>>,
}

/// Run the matching gate's pure `evaluate_keyed` over each live face and group the keys
/// per (gate, code). Returns `gate_id -> code -> sorted+deduped keys` (BTreeMaps/BTreeSets
/// keep it deterministic so committed==regenerated holds byte-for-byte).
fn current_keys_per_gate(
    inputs: &GateInputs<'_>,
) -> BTreeMap<&'static str, BTreeMap<String, BTreeSet<String>>> {
    let mut out: BTreeMap<&'static str, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();

    out.insert(
        "cloud-ci-total-accounting",
        group_findings(
            oya_cloud_ci_total_accounting_app::evaluate_keyed(inputs.total_accounting)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
    );
    out.insert(
        "cloud-ci-bnf-layer-suffix",
        group_findings(
            oya_cloud_ci_bnf_layer_suffix_app::evaluate_keyed(inputs.bnf_layer_suffix)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
    );
    out.insert(
        "cloud-ci-cross-artifact-agreement",
        group_findings(
            oya_cloud_ci_cross_artifact_agreement_app::evaluate_keyed(inputs.cross_artifact)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
    );
    out.insert(
        "cloud-ci-automation-ratchet",
        group_findings(
            oya_cloud_ci_automation_ratchet_app::evaluate_keyed(inputs.automation_ratchet)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
    );
    out.insert(
        "cloud-ci-staleness-reaper",
        group_findings(
            oya_cloud_ci_staleness_reaper_app::evaluate_keyed(inputs.staleness)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
    );
    // The brand-residue gate's keys are computed by the binary from the raw corpus (not a
    // generated face), so they arrive already grouped — fold them in verbatim.
    out.insert("cloud-ci-brand-residue", inputs.brand_residue.clone());
    out
}

/// Group `(code, key)` pairs into `code -> sorted+deduped keys`.
fn group_findings<I>(findings: I) -> BTreeMap<String, BTreeSet<String>>
where
    I: IntoIterator<Item = (String, String)>,
{
    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (code, key) in findings {
        map.entry(code).or_default().insert(key);
    }
    map
}

/// Build the GATE go-live baseline face (`gate-baseline.generated.json`). For every
/// (gate, code) in the disposition table it stamps `mode`/`infra_prereq`/`frozen_empty`
/// (DATA) and freezes the CURRENT keys captured by `evaluate_keyed` over the live faces.
/// Pure + deterministic: keys go through BTreeSet, faces are id/path-sorted upstream, so
/// committed==regenerated holds byte-for-byte and the registry-drift gate can byte-diff it.
///
/// The ratchet contract: a key only enters a baseline by being a CURRENT violation today
/// (auto-shrink drops fixed keys on regen); GROWTH beyond the committed baseline is a
/// `ratchet_regression` caught by the cloud-ci-firewall runner, not by this builder.
pub fn build_gate_baseline(inputs: &GateInputs<'_>) -> Result<Value, ProducerError> {
    let disposition: Value = serde_json::from_str(GATE_DISPOSITION_JSON)
        .map_err(|e| ProducerError::Policy(format!("gate-disposition.json: {e}")))?;
    let disp_gates = disposition
        .get("gates")
        .and_then(Value::as_object)
        .ok_or_else(|| ProducerError::Policy("gate-disposition.json missing 'gates'".into()))?;

    let current = current_keys_per_gate(inputs);

    // Canonical-key digest input: collect every "<gate>\x1f<code>\x1f<key>" line, sorted.
    let mut digest_lines: BTreeSet<String> = BTreeSet::new();

    let mut gates_obj = Map::new();
    for gate in GATE_IDS {
        let disp_codes = disp_gates
            .get(gate)
            .and_then(Value::as_object)
            .ok_or_else(|| ProducerError::Policy(format!("disposition missing gate {gate}")))?;
        let empty = BTreeMap::new();
        let gate_current = current.get(gate).unwrap_or(&empty);

        let mut code_obj = Map::new();
        for (code, disp) in disp_codes {
            let mode = disp
                .get("mode")
                .and_then(Value::as_str)
                .unwrap_or("baseline-block-on-new");
            let frozen_empty = disp.get("frozen_empty").and_then(Value::as_bool) == Some(true);
            let infra_prereq = disp.get("infra_prereq").and_then(Value::as_str);

            // frozen_empty codes never accumulate a baseline — their keys are forced empty
            // (the emptiness is DATA; any occurrence is NEW debt for the runner to block).
            let keys: Vec<Value> = if frozen_empty {
                Vec::new()
            } else {
                gate_current
                    .get(code)
                    .map(|set| set.iter().cloned().map(Value::String).collect())
                    .unwrap_or_default()
            };

            for key in &keys {
                if let Value::String(k) = key {
                    digest_lines.insert(format!("{gate}\u{1f}{code}\u{1f}{k}"));
                }
            }

            let mut entry = Map::new();
            entry.insert("mode".into(), Value::String(mode.to_owned()));
            if let Some(prereq) = infra_prereq {
                entry.insert("infra_prereq".into(), Value::String(prereq.to_owned()));
            }
            entry.insert("keys".into(), Value::Array(keys));
            if frozen_empty {
                entry.insert("frozen_empty".into(), Value::Bool(true));
            }
            code_obj.insert(code.clone(), Value::Object(entry));
        }
        gates_obj.insert(gate.to_owned(), Value::Object(code_obj));
    }

    let digest_input: Vec<&str> = digest_lines.iter().map(String::as_str).collect();
    let source_inputs_digest = digest_strings(&digest_input);

    let mut root = Map::new();
    root.insert(
        "_comment".into(),
        Value::String(
            "GENERATED by oya-cloud-ci-accounting-registry-app (--face baseline). DO NOT HAND-EDIT \
             except via the sign-off door (gate-baseline.signoff.json). committed==regenerated \
             (registry-drift byte-diffs it); a hand-edit to launder debt is itself registry_drift RED."
                .into(),
        ),
    );
    root.insert(
        "_provenance".into(),
        serde_json::json!({
            "producer_target": PRODUCER_TARGET,
            "firewall_target": FIREWALL_TARGET,
            "baseline_schema_version": 1,
            "source_inputs_digest": source_inputs_digest,
        }),
    );
    root.insert("gates".into(), Value::Object(gates_obj));
    Ok(Value::Object(root))
}

/// FNV-1a 64-bit digest over the canonical baseline keys (one "<gate>\x1f<code>\x1f<key>"
/// line per accepted key, sorted). Reuses the same hash family as `digest_rows` so the
/// baseline carries a content digest without a wall-clock (committed==regenerated).
fn digest_strings(lines: &[&str]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for line in lines {
        for byte in line.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash ^= u64::from(b'\n');
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

/// Serialize a face to the canonical on-disk form: 2-space pretty + trailing newline.
/// Identical formatting on every run keeps committed==regenerated byte-exact.
pub fn to_canonical_json(value: &Value) -> Result<String, ProducerError> {
    let mut text = serde_json::to_string_pretty(value)
        .map_err(|e| ProducerError::Serialize(e.to_string()))?;
    text.push('\n');
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_inputs() -> RepoInputs {
        let mut owners = BTreeMap::new();
        owners.insert("specs/masterplan.json".into(), "council-architecture".into());
        let mut justifications = BTreeMap::new();
        justifications.insert("specs/masterplan.json".into(), "ADR-0364".into());
        let mut reachability = BTreeMap::new();
        reachability.insert(
            "specs/masterplan.json".into(),
            vec!["root-hub".into(), "masterplan".into()],
        );
        let mut last_touch = BTreeMap::new();
        last_touch.insert("specs/masterplan.json".into(), "abc123".into());
        RepoInputs {
            tracked_paths: vec![
                "specs/masterplan.json".into(),
                ".omc/state/run.jsonl".into(),
                "oya/orphan/lib.rs".into(),
            ],
            last_touch,
            owners,
            justifications,
            reachability,
            dup_of: BTreeMap::new(),
        }
    }

    #[test]
    fn carve_outs_are_data_tabled_not_branched() {
        let policy = Policy::from_bundled().expect("bundled policy parses");
        // every classification comes from the table; ephemeral jsonl is carved by class
        assert_eq!(policy.classify(".omc/state/run.jsonl"), "ephemeral");
        assert_eq!(policy.classify("third-party/foo/lib.rs"), "vendor");
        assert_eq!(policy.classify("docs/foo.generated.json"), "generated");
        assert_eq!(policy.classify("specs/masterplan.json"), "spec");
        assert_eq!(policy.classify("docs/decisions/ADR-0001.md"), "doc");
        assert_eq!(policy.classify("oya/x/src/lib.rs"), "code");
        assert_eq!(policy.classify("some/unknown/blob"), "husk");
    }

    #[test]
    fn ephemeral_rows_excluded_by_class_coverage_invariant() {
        let policy = Policy::from_bundled().expect("policy");
        let registry = build_registry(&sample_inputs(), &policy).expect("registry");
        let rows = registry["rows"].as_array().expect("rows array");
        let paths: Vec<&str> = rows.iter().filter_map(|r| r["path"].as_str()).collect();
        assert!(paths.contains(&"specs/masterplan.json"));
        assert!(paths.contains(&"oya/orphan/lib.rs"));
        // ephemeral .jsonl excluded by class, not by row
        assert!(!paths.contains(&".omc/state/run.jsonl"));
    }

    #[test]
    fn verdicts_derive_from_accounting_facts() {
        let policy = Policy::from_bundled().expect("policy");
        let registry = build_registry(&sample_inputs(), &policy).expect("registry");
        let rows = registry["rows"].as_array().expect("rows");
        let masterplan = rows
            .iter()
            .find(|r| r["path"] == "specs/masterplan.json")
            .expect("masterplan row");
        // fully accounted + protected spec ⇒ KEEP
        assert_eq!(masterplan["verdict"], "KEEP");
        let orphan = rows
            .iter()
            .find(|r| r["path"] == "oya/orphan/lib.rs")
            .expect("orphan row");
        // no owner + no justification + no reachability ⇒ RED
        assert_eq!(orphan["verdict"], "RED");
    }

    #[test]
    fn gate_baseline_freezes_current_keys_and_stamps_disposition() {
        // total-accounting: one row with an unowned + unjustified + unreachable + no_ttl_class
        // exhibit; cross-artifact: a dual id; the others empty.
        let registry = serde_json::json!({"rows": [
            {"path": "oya/x/lib.rs", "owner": null, "justification_ref": null,
             "reachable_from": [], "ttl": {}}
        ]});
        let crosswalk = serde_json::json!({"decisions": [], "duplicate_ids": ["ADR-0377"]});
        let automation = serde_json::json!({"rows": []});
        let staleness = serde_json::json!({"rows": []});
        let mut brand_residue: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        brand_residue
            .entry("forbidden_foundry".to_owned())
            .or_default()
            .insert("docs/products/foundry/PRD.md".to_owned());
        let inputs = GateInputs {
            total_accounting: &registry,
            cross_artifact: &crosswalk,
            automation_ratchet: &automation,
            staleness: &staleness,
            brand_residue: &brand_residue,
        };
        let baseline = build_gate_baseline(&inputs).expect("baseline");
        let ta = &baseline["gates"]["cloud-ci-total-accounting"];

        // unjustified is baseline-block-on-new and freezes the live key (the row path).
        assert_eq!(ta["unjustified"]["mode"], "baseline-block-on-new");
        assert_eq!(ta["unjustified"]["keys"][0], "oya/x/lib.rs");
        // unowned is advisory-until-infra and STILL freezes the key (advisory reports, the
        // runner just never fails on it until the disposition flips).
        assert_eq!(ta["unowned"]["mode"], "advisory-until-infra");
        assert_eq!(ta["unowned"]["infra_prereq"], "owners-files-tree-wide");
        // registry_drift is frozen_empty: never accumulates a key even if one were present.
        assert_eq!(ta["registry_drift"]["frozen_empty"], true);
        assert_eq!(ta["registry_drift"]["keys"].as_array().unwrap().len(), 0);

        let xa = &baseline["gates"]["cloud-ci-cross-artifact-agreement"];
        assert_eq!(xa["dual_decision_collision"]["keys"][0], "ADR-0377");

        // brand-residue freezes the live per-(stem,file) key under its per-stem code.
        let br = &baseline["gates"]["cloud-ci-brand-residue"];
        assert_eq!(br["forbidden_foundry"]["mode"], "baseline-block-on-new");
        assert_eq!(br["forbidden_foundry"]["keys"][0], "docs/products/foundry/PRD.md");
        // a stem with zero live residue freezes an empty (but present) key set.
        assert_eq!(br["forbidden_forgejo"]["keys"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn gate_baseline_is_idempotent_byte_for_byte() {
        let registry = serde_json::json!({"rows": []});
        let crosswalk = serde_json::json!({"decisions": []});
        let automation = serde_json::json!({"rows": []});
        let staleness = serde_json::json!({"rows": []});
        let brand_residue: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let inputs = GateInputs {
            total_accounting: &registry,
            cross_artifact: &crosswalk,
            automation_ratchet: &automation,
            staleness: &staleness,
            brand_residue: &brand_residue,
        };
        let a = to_canonical_json(&build_gate_baseline(&inputs).expect("a")).expect("ja");
        let b = to_canonical_json(&build_gate_baseline(&inputs).expect("b")).expect("jb");
        assert_eq!(a, b, "baseline must be byte-deterministic");
        assert!(a.contains("source_inputs_digest"));
        assert!(!a.contains("generated_at"), "no wall-clock in the baseline face");
    }

    #[test]
    fn build_is_idempotent_byte_for_byte() {
        let policy = Policy::from_bundled().expect("policy");
        let inputs = sample_inputs();
        let a = to_canonical_json(&build_registry(&inputs, &policy).expect("a")).expect("ja");
        let b = to_canonical_json(&build_registry(&inputs, &policy).expect("b")).expect("jb");
        assert_eq!(a, b, "producer must be byte-deterministic");
        // and the provenance digest must be present (proves generation)
        assert!(a.contains("source_inputs_digest"));
        assert!(!a.contains("generated_at"), "no wall-clock in the face");
    }
}
