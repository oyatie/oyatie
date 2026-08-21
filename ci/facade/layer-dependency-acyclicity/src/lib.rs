//! # cloud-ci-tier-dependency-acyclicity (Phase-0 capability-first reorg; ADR-0245/0280/0562)
//!
//! Phase-0 of the ratified capability-first repo organization (ADR-0562) introduces a clean
//! mandatory dependency-class triple on every service `manifest.json` (`tier`, `tier_subtype`,
//! `substrate_dag_position.stratum` for substrates). This gate is the ENFORCEMENT surface of the
//! ADR-0245 tier dependency rules + the ADR-0280 substrate-of-substrate DAG, evaluated over the
//! REAL crate dependency graph — both the cargo surface (path-deps + workspace membership) AND the
//! buck surface (`deps`/visibility targets) — rather than the §D-1 DAG document alone (which the
//! sibling `oya-cloud-ci-substrate-dependency-dag-acyclicity-app` already validates).
//!
//! ## The two dependency surfaces it reads (the contract)
//! 1. **cargo** — each first-party crate's `Cargo.toml` `path = "…"` dependencies (the authoritative
//!    path-dep surface) plus workspace membership (the set of governed crates). The DAG-JSON-only
//!    check MISSES these (they are how the inversions are actually wired).
//! 2. **buck** — each crate's `BUCK`/`BUCK.v2` first-party `deps`/visibility targets (`//path:name`).
//!
//! The two surfaces are UNIONED into one crate→crate edge set, then projected service→service via
//! each crate's owning-service `manifest.json` tier metadata (ADR-0245 tier-as-facet) and the
//! ADR-0562 capability registry's `absorbs_current_dirs` (service dir → capability).
//!
//! ## The rule set (ADR-0245 + ADR-0280)
//! Let `from` DEPEND ON `to` (an edge `from -> to`):
//! - **R1 substrate-no-upward** — a `substrate` crate MUST NOT depend on a `product` or
//!   `service-cell` crate (substrates sit beneath the product/cell rings). Code `TDA-SUBSTRATE-UPWARD`.
//! - **R2 no product↔service-cell** — a `product` crate MUST NOT depend on a `service-cell` crate and
//!   vice-versa (the two top rings never cross). Code `TDA-PRODUCT-CELL-CROSS`.
//! - **R3 service-cell-no-product** — a `service-cell` crate MUST NOT depend on a `product` crate
//!   (subsumed direction of R2, surfaced distinctly for clarity). Code `TDA-CELL-PRODUCT`.
//! - **R4 intra-substrate S-rank** — a `substrate -> substrate` edge may only point to an
//!   EQUAL-OR-LOWER ADR-0280 S-rank (S0 is the lowest/leaf). An edge to a HIGHER S-rank is an
//!   inversion. Code `TDA-S-RANK-INVERSION`. (`forward-declared` substrates carry no S-rank yet and
//!   are exempt from the rank comparison — they are seed placeholders, not live edges. That
//!   justification holds for a SINGLE forward-declared service and NOT for a capability root, which
//!   would inherit repo-wide rank-exemption for its whole tree from one surviving placeholder; R6c
//!   therefore requires a capability's projected stratum to be RANKABLE.)
//! - **R5 acyclicity backstop** — Tarjan SCC over the crate graph; any SCC of size > 1 (or a
//!   self-loop) is a cycle. Code `TDA-CYCLE`.
//!
//! Lateral product↔product (ADR-0145) and substrate→substrate within rank, plus any edge to/from an
//! unclassified crate (`libs/`, `tools/`, the CI gates, `oya/office/`), are ALLOWED.
//!
//! ## Born-ADVISORY with a frozen baseline (does NOT break the live tree)
//! The PRE-MOVE tree carries pre-existing tier violations — the very substrate-inversions the reorg
//! exists to fix (e.g. `oya/intelligence -> oya/community`, `cloud-kms(S0) -> cloud-network(S1)`).
//! So the gate CANNOT be born-blocking-green. Instead it is born-ADVISORY against a FROZEN BASELINE
//! ([`Baseline`], the committed known-debt set, like the build-health / accounting baselines):
//! - Every current violation is split BASELINED (advisory, known debt) vs REGRESSION (a NEW
//!   violation absent from the baseline). The gate is RED iff there is at least one REGRESSION, so a
//!   `reorg-codemod` move PR (or any PR) MUST NOT ADD a new violation; the baseline burns DOWN as
//!   the strangler moves fix inversions (removing a baselined violation is always allowed).
//! - **Flip-to-fully-blocking trigger**: when the baseline reaches 0 (post-strangler), set the
//!   policy `enforcement` to `blocking`; the gate then treats EVERY violation (not just regressions)
//!   as RED. Until then it is `advisory-baseline`.
//!
//! The gate is GREEN on the live tree at birth: the live corpus == the freshly-frozen baseline, so
//! there are zero regressions.
//!
//! ## Born pack-shaped
//! The governed roots, tier-rule config, stratum ranks, and the meta/unclassified roots are DATA in
//! `tier-dependency-acyclicity-policy.json`; the frozen known-debt is DATA in
//! `tier-dependency-acyclicity-baseline.json`. The kernel hardcodes no repo fact — a different repo
//! adopts the gate by repointing the policy + re-freezing the baseline.
//!
//! ## Violation codes (the contract — literal strings the gate emits)
//! - `TDA-SUBSTRATE-UPWARD`   — a substrate crate depends on a product/service-cell crate (R1).
//! - `TDA-PRODUCT-CELL-CROSS` — a product crate depends on a service-cell crate (R2).
//! - `TDA-CELL-PRODUCT`       — a service-cell crate depends on a product crate (R3).
//! - `TDA-S-RANK-INVERSION`   — a substrate->substrate edge points to a higher S-rank (R4).
//! - `TDA-CYCLE`              — a strongly-connected component of size > 1 / self-loop (R5).
//! - `TDA-EMPTY-SCAN`         — fewer crates than the policy floor (false-green guard).
//! - `TDA-POLICY-MALFORMED`   — the policy is missing/wrong-typed a required field (fail-closed).
//! - `TDA-BASELINE-MALFORMED` — the baseline document is malformed (fail-closed).
//! - `TDA-STALE-BASELINE`     — a committed baseline subject names a crate ABSENT from the live corpus
//!   (a phantom row; B3 hardening). See below.
//! - `TDA-UNDECLARED-ROOT`    — a governed crate sits under a top-level root that is declared
//!   in none of `service_roots` / `capability_roots` (tier-classified) or `unclassified_roots`
//!   (deliberately exempt). Such a crate is silently unenforced: `owning_service` returns `None`,
//!   so every edge touching it is skipped. `unclassified_roots` was previously parsed and NEVER
//!   READ — inert config that looked like it governed the exemption. This code makes the
//!   declaration live, so a new crate-bearing root must be declared before its crates can be
//!   silently skipped.
//! - `TDA-UNCLASSIFIED-ROOT-NOT-META` — a root declared in `unclassified_roots` is NOT a
//!   `meta_directories` entry of the ADR-0562 CLOSED capability registry. See R6b below.
//! - `TDA-CAPABILITY-TIER-UNRESOLVED` — a root declared in `capability_roots` carries no tier the
//!   rules can ACT on: none declared or projected, or a substrate whose stratum has no S-rank. See
//!   R6c below.
//!
//! ## R6b/R6c — `unclassified` is reserved for registry-declared META dirs (the exemption floor)
//! `TDA-UNDECLARED-ROOT` alone left a self-service escape hatch: its prescribed remedy is "declare
//! the root", and declaring it in `unclassified_roots` silences R6 while leaving the tier-comparison
//! SKIP fully intact. The detector was therefore satisfiable by disabling the thing it protects, and
//! `unclassified_roots` grew 3 -> 27 entries, one per capability move — 24 of the 27 being registered
//! CAPABILITIES, i.e. exactly the tier-bearing units ADR-0562 defines, not meta trees.
//!
//! The closed capability registry (`governance/capability-registry.json`, `closed: true`) already carries
//! both halves of the fix, so neither is a new hand-maintained list:
//! - `capabilities[].name` — the units that MUST be tier-classified (`capability_roots`);
//! - `meta_directories[].dir` — the CLOSED allowlist of trees that may legitimately carry no tier.
//!
//! **R6b** (`TDA-UNCLASSIFIED-ROOT-NOT-META`): every `unclassified_roots` entry must be a registry
//! `meta_directories` dir. A registered capability declared unclassified REDs with the remedy "move
//! it to `capability_roots`"; a root that is neither capability nor meta REDs with "register it in
//! the closed registry". Since the registry is CLOSED, a future capability move can no longer buy
//! silence by appending to `unclassified_roots` — the only green paths are tier-classifying the root
//! or amending the ADR-0562 authority, both reviewable acts.
//!
//! **R6c** (`TDA-CAPABILITY-TIER-UNRESOLVED`): opting a root INTO `capability_roots` must not become
//! the replacement silent exemption. A capability root that carries no tier the rules can ACT on is
//! RED and non-baselineable, so a root cannot be parked in `capability_roots` to look enforced while
//! comparing nothing. Two ways to fail it, with distinct remedies: (i) UNRESOLVED — no `tier` in
//! `<capability>/manifest.json` and no unanimous projection from the registry-absorbed services;
//! (ii) UNRANKABLE — a `substrate` whose stratum is outside `stratum_rank_order` (`forward-declared`),
//! which R4 looks up, misses, and skips. (ii) is the one the PRESENCE test (`contains_key`) missed,
//! and it applies to a DECLARED `forward-declared` exactly as it does to a derived one — otherwise
//! "it was declared" becomes the next silent exemption.
//!
//! **A capability DECLARES its tier; nothing derives it.** ADR-0562 makes the capability the
//! tier-bearing unit, and `governance/capability-registry.json` already defines the capabilities and is
//! already the authority R6b trusts for `meta_directories`, so the `tier` +
//! `substrate_dag_position.stratum` facets live there beside the definition. The tier was previously
//! PROJECTED from the services in `absorbs_current_dirs` — a derivation standing in for a missing
//! declaration, reading the wrong authority. It failed two ways that patching could not fix: it
//! resolved only while the absorbed dirs still EXISTED (a completed move orphaned the tier, so the
//! gate failed permanently exactly when the reorg SUCCEEDED), and its unanimity was computed over
//! whichever services had not moved YET, so MIGRATION ORDER decided the answer. Both failure modes
//! are deleted with the derivation. There is ONE mechanism: an undeclared capability is RED, never
//! projected — two mechanisms where one silently covers for the other is how `unclassified_roots`
//! became a silent exemption in the first place.
//!
//! ## Baseline-liveness backstop (B3 hardening — phantom rows made impossible)
//! The frozen baseline is a SUBSET-semantics ratchet: it blocks only on a NEW regression (a
//! `code|subject` NOT in the baseline) and never REDs on a row it merely *contains*. That is sound
//! for known-debt, but the repo runs in-flight strangler crate MOVES: when a crate MOVES paths, its
//! OLD-path edge in the baseline becomes a PHANTOM — a row whose `from`/`to` names a crate dir that no
//! longer exists, which subset semantics can never fire on, so the baseline silently diverges from
//! reality (the same defect class as the firewall gate-baseline staleness fix). The gate asserts
//! every committed baseline subject is still ANCHORED — each crate dir it names must exist in the live
//! crate set — and emits `TDA-STALE-BASELINE` (a blocking regression) for any phantom, with the remedy
//! being a re-emit (`--emit-baseline`) that drops it. This is ADDITIVE: it does NOT touch the subset
//! regression check, and a baselined row whose endpoints still exist but whose EDGE was removed stays
//! a legitimate BURN-DOWN (green), never a phantom.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

/// The gate id, matching the buck2 target stem + the policy `gate_id`.
pub const GATE_ID: &str = "cloud-ci-tier-dependency-acyclicity";

/// The policy DATA path, relative to the repo root.
pub const POLICY_PATH: &str =
    "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json";

/// The frozen known-debt baseline path, relative to the repo root.
pub const BASELINE_PATH: &str =
    "ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-baseline.json";

/// The violation codes, in canonical order.
pub const VIOLATION_CODES: [&str; 12] = [
    "TDA-SUBSTRATE-UPWARD",
    "TDA-PRODUCT-CELL-CROSS",
    "TDA-CELL-PRODUCT",
    "TDA-S-RANK-INVERSION",
    "TDA-CYCLE",
    "TDA-EMPTY-SCAN",
    "TDA-POLICY-MALFORMED",
    "TDA-BASELINE-MALFORMED",
    "TDA-STALE-BASELINE",
    "TDA-UNDECLARED-ROOT",
    "TDA-UNCLASSIFIED-ROOT-NOT-META",
    "TDA-CAPABILITY-TIER-UNRESOLVED",
];

/// The codes whose `subject` is a top-level ROOT name, not a crate edge or an SCC member list.
/// Two evaluator behaviours key off this: the B3 baseline-liveness backstop must not treat a root
/// subject as a phantom crate dir (`iam` is never a crate dir, so every such row would RED), and
/// `--emit-baseline` uses it to keep the non-baselineable structural codes out of a re-freeze.
pub const ROOT_SUBJECT_CODES: [&str; 3] = [
    "TDA-UNDECLARED-ROOT",
    "TDA-UNCLASSIFIED-ROOT-NOT-META",
    "TDA-CAPABILITY-TIER-UNRESOLVED",
];

/// Sentinel key for policy/baseline-level (non-per-edge) findings.
const POLICY_KEY: &str = "<policy>";

// ───────────────────────────── errors ─────────────────────────────

/// Errors collecting the observed crate/dep/tier corpus. Returned (never panicked) so the caller
/// decides how to surface them — an unreadable governed root is a fail-closed error, never a silent
/// skip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectError {
    Io(String),
    Parse { path: String, message: String },
}

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectError::Io(message) => write!(f, "tier-dependency-acyclicity io: {message}"),
            CollectError::Parse { path, message } => {
                write!(f, "{path} is not valid JSON/TOML: {message}")
            }
        }
    }
}

impl std::error::Error for CollectError {}

// ───────────────────────────── findings + verdict ─────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Green,
    Red,
}

/// Whether a finding is pre-existing known debt (baselined, advisory) or a new regression (blocking).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    /// Present in the frozen baseline — known debt, advisory only.
    Baselined,
    /// Absent from the baseline — a NEW violation. Blocks under `advisory-baseline` enforcement.
    Regression,
}

/// A single tier-dependency violation, keyed by code + a stable subject (the crate edge).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    /// One of [`VIOLATION_CODES`].
    pub code: String,
    /// A stable subject (the crate edge `from -> to`, or `<policy>` for policy/baseline findings).
    pub subject: String,
    /// Human-readable detail.
    pub detail: String,
    /// Baselined (advisory) or a regression (blocking).
    pub status: Status,
}

impl Finding {
    fn new(code: &str, subject: &str, detail: impl Into<String>, status: Status) -> Self {
        Self {
            code: code.to_owned(),
            subject: subject.to_owned(),
            detail: detail.into(),
            status,
        }
    }
}

/// The evaluation report: the ordered findings + summary counts + the verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub findings: Vec<Finding>,
    /// How many crates the projection covered.
    pub crates_checked: usize,
    /// How many crate→crate edges (union of both surfaces) were evaluated.
    pub edges_checked: usize,
    /// Current pre-existing (baselined, advisory) violation count.
    pub baselined: usize,
    /// New violations absent from the baseline (the blocking set).
    pub regressions: usize,
    /// Baselined violations no longer present (burn-down progress; informational, never blocking).
    pub burned_down: usize,
    pub verdict: Verdict,
}

// ───────────────────────────── frozen baseline ─────────────────────────────

/// The committed frozen known-debt set: the canonical `code|subject` keys of every pre-existing
/// violation at the moment the gate was born. A NEW violation (a key NOT in this set) is a
/// regression. A move PR that FIXES a baselined inversion shrinks the live set below the baseline —
/// always allowed (burn-down).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Baseline {
    /// Canonical `code|subject` keys (sorted, de-duplicated).
    pub keys: BTreeSet<String>,
}

impl Baseline {
    /// The canonical key for a finding: `code|subject`. The single membership identity.
    #[must_use]
    pub fn key_of(code: &str, subject: &str) -> String {
        format!("{code}|{subject}")
    }

    fn contains(&self, code: &str, subject: &str) -> bool {
        self.keys.contains(&Self::key_of(code, subject))
    }
}

/// Parse a baseline document `{ "gate_id", "violations": [ { "code", "subject" }, .. ] }`. Returns
/// `Err` on a malformed shape so the gate emits `TDA-BASELINE-MALFORMED` and fails CLOSED rather
/// than treating an unreadable baseline as "everything is a regression" or "nothing is".
pub fn parse_baseline(value: &Value) -> Result<Baseline, String> {
    let arr = value
        .get("violations")
        .and_then(Value::as_array)
        .ok_or_else(|| "baseline `violations` must be an array".to_owned())?;
    let mut keys = BTreeSet::new();
    for (i, v) in arr.iter().enumerate() {
        let code = v
            .get("code")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("baseline violations[{i}] missing string `code`"))?;
        let subject = v
            .get("subject")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("baseline violations[{i}] missing string `subject`"))?;
        keys.insert(Baseline::key_of(code, subject));
    }
    Ok(Baseline { keys })
}

// ───────────────────────────── policy ─────────────────────────────

/// Parsed policy DATA. Returns an Err string on any malformed required field so the evaluator emits
/// `TDA-POLICY-MALFORMED` and fails CLOSED rather than silently dropping a check.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedPolicy {
    /// Governed crate roots (member globs) — the set of first-party crate dirs to scan.
    crate_root_globs: Vec<String>,
    /// Governed SERVICE roots whose `manifest.json` carries tier metadata (`cloud/`, `oya/`,
    /// `app/`). Shape: `<root>/<service>/**`, so the tier unit is the 2-component service prefix.
    service_roots: Vec<String>,
    /// Governed CAPABILITY roots (ADR-0562 `<capability>/<face>/<crate>`). Shape-distinct from
    /// `service_roots`: the tier unit is the ROOT ITSELF (the capability), because the second path
    /// component is an ADR-0562 FACE (`core`/`ports`/`adapters`/`facade`/`observability`), not a
    /// service. The tier is derived from the capability's registry-absorbed `cloud/`+`oya/` services.
    capability_roots: Vec<String>,
    /// Top-level dirs treated as UNCLASSIFIED (no tier; exempt from cross-tier rules): meta crates.
    /// R6b constrains this to the closed registry's `meta_directories`.
    unclassified_roots: BTreeSet<String>,
    /// Repo-relative path to the ADR-0562 closed capability registry (the R6b/R6c authority).
    capability_registry_path: String,
    /// The ordered S-rank enum (`S0` is lowest). Strata outside it (e.g. `forward-declared`) are
    /// rank-exempt.
    stratum_rank: BTreeMap<String, usize>,
    /// Floor on observed crates (false-green guard).
    min_expected_crates: u64,
    /// `advisory-baseline` (born) or `blocking` (post-burn-down flip).
    enforcement: String,
}

fn string_array(policy: &Value, key: &str) -> Result<Vec<String>, String> {
    let arr = policy
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("policy `{key}` must be a string array"))?;
    let mut out = Vec::with_capacity(arr.len());
    for (i, v) in arr.iter().enumerate() {
        let s = v
            .as_str()
            .ok_or_else(|| format!("policy `{key}`[{i}] must be a string"))?;
        out.push(s.to_owned());
    }
    if out.is_empty() {
        return Err(format!("policy `{key}` must be non-empty"));
    }
    Ok(out)
}

fn parse_policy(policy: &Value) -> Result<ParsedPolicy, String> {
    let stratum_order = string_array(policy, "stratum_rank_order")?;
    let stratum_rank: BTreeMap<String, usize> = stratum_order
        .into_iter()
        .enumerate()
        .map(|(i, s)| (s, i))
        .collect();
    let unclassified_roots: BTreeSet<String> = string_array(policy, "unclassified_roots")?
        .into_iter()
        .collect();
    let enforcement = policy
        .get("enforcement")
        .and_then(Value::as_str)
        .unwrap_or("advisory-baseline")
        .to_owned();
    if enforcement != "advisory-baseline" && enforcement != "blocking" {
        return Err(format!(
            "policy `enforcement` must be \"advisory-baseline\" or \"blocking\", got {enforcement:?}"
        ));
    }
    // `capability_roots` is OPTIONAL-but-typed: absent or `[]` is legal (a repo mid-migration, or
    // one with no capability trees), but a present non-array / non-string entry fails CLOSED rather
    // than silently degrading to "no capability roots" — the exact false-green this gate exists to
    // prevent.
    let capability_roots = match policy.get("capability_roots") {
        None => Vec::new(),
        Some(Value::Array(arr)) => {
            let mut out = Vec::with_capacity(arr.len());
            for (i, v) in arr.iter().enumerate() {
                out.push(
                    v.as_str()
                        .ok_or_else(|| format!("policy `capability_roots`[{i}] must be a string"))?
                        .to_owned(),
                );
            }
            out
        }
        Some(_) => return Err("policy `capability_roots` must be a string array".to_owned()),
    };
    if let Some(dup) = capability_roots
        .iter()
        .find(|r| unclassified_roots.contains(*r))
    {
        return Err(format!(
            "policy declares root `{dup}` in BOTH `capability_roots` and `unclassified_roots`; a \
             root is either tier-enforced or exempt, never both"
        ));
    }
    let capability_registry_path = policy
        .get("capability_registry_path")
        .and_then(Value::as_str)
        .ok_or_else(|| "policy `capability_registry_path` must be a string".to_owned())?
        .to_owned();

    Ok(ParsedPolicy {
        crate_root_globs: string_array(policy, "crate_root_globs")?,
        service_roots: string_array(policy, "service_roots")?,
        capability_roots,
        unclassified_roots,
        capability_registry_path,
        stratum_rank,
        min_expected_crates: policy
            .get("min_expected_crates")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        enforcement,
    })
}

// ───────────────────────────── projected tier ─────────────────────────────

/// The dependency tier-class of a service (ADR-0245). `Unclassified` covers meta crates with no
/// service `manifest.json` tier (`libs/`, `tools/`, the CI gates, `oya/office/`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier {
    /// `substrate` / `product` / `service-cell` / `reserved`.
    pub class: String,
    /// The ADR-0280 substrate S-rank (`S0`..`S5` / `forward-declared`), only for substrates.
    pub stratum: Option<String>,
}

// ───────────────────────────── collection (the only I/O; read-only) ─────────────────────────────

/// Collect the live observed corpus into the pure-evaluator shape:
/// ```json
/// {
///   "crate_count": <usize>,
///   "edge_count": <usize>,
///   "crates": [ { "dir", "service" } ],
///   "service_tiers": { "<service-dir>": { "tier", "stratum" } },
///   "edges": [ { "from", "to" } ]   // crate-dir -> crate-dir, union of cargo + buck surfaces
/// }
/// ```
/// Read-only; writes no temp files. A malformed manifest/Cargo.toml/BUCK is a fail-closed
/// `CollectError`, never a silently skipped file.
pub fn collect_corpus(root: &Path, policy: &Value) -> Result<Value, CollectError> {
    let parsed = parse_policy(policy).map_err(|m| CollectError::Parse {
        path: POLICY_PATH.to_owned(),
        message: m,
    })?;

    // 1. Service-dir -> tier metadata, sourced from each service-root manifest.json (authoritative).
    let mut service_tiers = serde_json::Map::new();
    for service_root in &parsed.service_roots {
        let dir = root.join(service_root);
        collect_service_tiers(&dir, root, &mut service_tiers)?;
    }

    // 1b. Capability-root -> tier, DECLARED in the ADR-0562 closed registry beside the capability's
    // own definition, in the same `tier` + `substrate_dag_position.stratum` facets a service manifest
    // uses (one extractor, [`tier_facets`], reads both).
    //
    // There is deliberately NO derivation here. A capability's tier used to be PROJECTED from the
    // `cloud/`+`oya/` services its `absorbs_current_dirs` names — a derivation standing in for a
    // declaration nobody had written. ADR-0562 makes the CAPABILITY the tier-bearing unit, so the
    // projection was reading the wrong authority, and it broke in two ways that no amount of
    // patching fixes: it resolves only while the absorbed dirs still EXIST, so a completed move
    // orphaned the tier (the gate failed permanently exactly when the reorg SUCCEEDED), and its
    // unanimity was computed over whichever services had not moved YET, so MIGRATION ORDER decided
    // the answer — `iam` spans S1+S1+S3, and moving the S3 service first silently resolved it S1.
    // Declaring the tier deletes both failure modes rather than guarding them. An undeclared
    // capability is RED (R6c), never projected: a capability whose absorbed services genuinely
    // disagree has no defensible tier, and inventing one is the under-enforcement this gate exists
    // to catch, with a plausible-looking number attached.
    let registry = load_json(root, &parsed.capability_registry_path)?;
    let (registry_capabilities, registry_meta_dirs) = registry_facts(&registry);
    for cap in &parsed.capability_roots {
        if let Some(record) = registry_entry(&registry, cap).and_then(tier_facets) {
            service_tiers.insert(cap.clone(), record);
        }
    }

    // 2. The governed first-party crate dirs (resolve the member globs against the live tree).
    let mut crate_dirs: BTreeSet<String> = BTreeSet::new();
    for glob in &parsed.crate_root_globs {
        resolve_one_star_glob(root, glob, &mut crate_dirs)?;
    }

    // 3. Per crate, the owning service + the union of cargo + buck dependency edges (first-party).
    let mut crates = Vec::with_capacity(crate_dirs.len());
    let mut edges: BTreeSet<(String, String)> = BTreeSet::new();
    for cdir in &crate_dirs {
        let service = owning_service(cdir, &parsed.service_roots, &parsed.capability_roots);
        crates.push(json!({ "dir": cdir, "service": service }));

        let mut deps: BTreeSet<String> = BTreeSet::new();
        cargo_path_deps(root, cdir, &mut deps)?;
        buck_deps(root, cdir, &mut deps)?;
        for dep in deps {
            if crate_dirs.contains(&dep) && &dep != cdir {
                edges.insert((cdir.clone(), dep));
            }
        }
    }

    let edge_vec: Vec<Value> = edges
        .iter()
        .map(|(f, t)| json!({ "from": f, "to": t }))
        .collect();

    Ok(json!({
        "crate_count": crates.len(),
        "edge_count": edge_vec.len(),
        "crates": crates,
        "service_tiers": Value::Object(service_tiers),
        "edges": edge_vec,
        // R6b/R6c inputs, carried as DATA so the evaluator stays pure (no registry I/O of its own).
        "registry_capabilities": registry_capabilities,
        "registry_meta_dirs": registry_meta_dirs,
    }))
}

/// The two closed-registry fact sets R6b/R6c evaluate against: the registered capability names and
/// the `meta_directories` dirs (trailing `/` stripped, matching the top-level root spelling).
fn registry_facts(registry: &Value) -> (Vec<String>, Vec<String>) {
    let names = |key: &str, field: &str| -> Vec<String> {
        registry
            .get(key)
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|e| e.get(field).and_then(Value::as_str))
                    .map(|d| d.trim_end_matches('/').to_owned())
                    .collect()
            })
            .unwrap_or_default()
    };
    (
        names("capabilities", "name"),
        names("meta_directories", "dir"),
    )
}

/// The `capabilities[]` entry named `capability` in the ADR-0562 closed registry.
fn registry_entry<'a>(registry: &'a Value, capability: &str) -> Option<&'a Value> {
    registry
        .get("capabilities")
        .and_then(Value::as_array)?
        .iter()
        .find(|c| c.get("name").and_then(Value::as_str) == Some(capability))
}

/// Read the service-root `manifest.json` directly under each governed service root and record its
/// `(tier, substrate_dag_position.stratum)`. A service root is `<governed-root>/<name>` (or
/// `<governed-root>/<a>/<name>` for nested governed roots like `oya/office`), identified by a
/// `manifest.json` that carries a `tier`. Read-only.
fn collect_service_tiers(
    dir: &Path,
    repo_root: &Path,
    out: &mut serde_json::Map<String, Value>,
) -> Result<(), CollectError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(CollectError::Io(format!("read dir {}: {e}", dir.display()))),
    };
    for entry in entries {
        let entry =
            entry.map_err(|e| CollectError::Io(format!("entry in {}: {e}", dir.display())))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| CollectError::Io(format!("file_type {}: {e}", path.display())))?;
        if !file_type.is_dir() {
            continue;
        }
        let rel = match path.strip_prefix(repo_root) {
            Ok(p) => p.to_string_lossy().replace('\\', "/"),
            Err(_) => path.to_string_lossy().into_owned(),
        };
        if let Some(record) = tier_record(repo_root, &rel)? {
            out.insert(rel, record);
        }
    }
    Ok(())
}

/// The `{tier, stratum}` record a document declares via the `tier` +
/// `substrate_dag_position.stratum` facets, or `None` if it carries no `tier`. The SINGLE extractor
/// for both tier authorities — a service `manifest.json` and a capability's registry entry — so the
/// two cannot drift into reading the same facets differently, and so a capability's declaration is
/// spelled exactly like the service declarations it replaced.
fn tier_facets(value: &Value) -> Option<Value> {
    let tier = value.get("tier").and_then(Value::as_str)?;
    let mut record = serde_json::Map::new();
    record.insert("tier".to_owned(), json!(tier));
    if let Some(stratum) = value
        .get("substrate_dag_position")
        .and_then(|p| p.get("stratum"))
        .and_then(Value::as_str)
    {
        record.insert("stratum".to_owned(), json!(stratum));
    }
    Some(Value::Object(record))
}

/// [`tier_facets`] of `<repo_root>/<rel>/manifest.json`, or `None` when there is no manifest.
/// A malformed manifest is a fail-closed `CollectError`, never a silent skip.
fn tier_record(repo_root: &Path, rel: &str) -> Result<Option<Value>, CollectError> {
    let manifest = repo_root.join(rel).join("manifest.json");
    if !manifest.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(&manifest)
        .map_err(|e| CollectError::Io(format!("read {rel}/manifest.json: {e}")))?;
    let value: Value = serde_json::from_str(&text).map_err(|e| CollectError::Parse {
        path: format!("{rel}/manifest.json"),
        message: e.to_string(),
    })?;
    Ok(tier_facets(&value))
}

/// Resolve a member glob with single-star path segments (e.g. `cloud/*/crates/oya-*`) against the
/// live tree, recording each matched dir that holds a `Cargo.toml`. A missing prefix dir is not an
/// error (repo-portable). Deterministic (BTreeSet output).
fn resolve_one_star_glob(
    root: &Path,
    glob: &str,
    out: &mut BTreeSet<String>,
) -> Result<(), CollectError> {
    let segments: Vec<&str> = glob.split('/').collect();
    let mut frontier: Vec<Vec<String>> = vec![Vec::new()];
    for seg in segments {
        let mut next: Vec<Vec<String>> = Vec::new();
        for prefix in &frontier {
            let dir = {
                let mut p = root.to_path_buf();
                for c in prefix {
                    p.push(c);
                }
                p
            };
            if seg.contains('*') {
                let entries = match fs::read_dir(&dir) {
                    Ok(entries) => entries,
                    Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(e) => {
                        return Err(CollectError::Io(format!("read dir {}: {e}", dir.display())));
                    }
                };
                for entry in entries {
                    let entry = entry.map_err(|e| {
                        CollectError::Io(format!("entry in {}: {e}", dir.display()))
                    })?;
                    let ft = entry.file_type().map_err(|e| {
                        CollectError::Io(format!("file_type {}: {e}", entry.path().display()))
                    })?;
                    if !ft.is_dir() {
                        continue;
                    }
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if segment_matches(seg, &name) {
                        let mut p = prefix.clone();
                        p.push(name);
                        next.push(p);
                    }
                }
            } else {
                let mut p = prefix.clone();
                p.push(seg.to_owned());
                next.push(p);
            }
        }
        frontier = next;
    }
    for prefix in frontier {
        let rel = prefix.join("/");
        if root.join(&rel).join("Cargo.toml").is_file() {
            out.insert(rel);
        }
    }
    Ok(())
}

/// Match a single glob segment with at most one trailing `*` (the member-glob shape, e.g. `oya-*`).
fn segment_matches(pattern: &str, name: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == name,
        Some((prefix, suffix)) => {
            name.len() >= prefix.len() + suffix.len()
                && name.starts_with(prefix)
                && name.ends_with(suffix)
        }
    }
}

/// The owning TIER UNIT of a crate dir. Two governed shapes, deliberately distinct:
/// - a `capability_roots` root (ADR-0562 `<capability>/<face>/<crate>`) → the ROOT ITSELF. The
///   second component is a FACE (`core`/`ports`/`adapters`/`facade`/`observability`), so a
///   2-component prefix would name `iam/adapters` — a face, not a tier-bearing unit. This is why
///   capability roots cannot simply be appended to `service_roots`.
/// - a `service_roots` root (`cloud/<svc>/crates/…`, `oya/<svc>/crates/…`,
///   `app/<product>/<face>/…`) → the 2-component `<root>/<svc>` prefix, which is
///   where the tier'd `manifest.json` lives (`app/community`, `oya/community`).
///
/// Returns `None` for crates outside both (the meta trees), which the evaluator treats as
/// unclassified.
///
/// This CONSUMES `service_roots`. It previously hardcoded two product roots and took the
/// parameter as `_service_roots`, so the policy field appeared to govern the projection and did
/// not — a reader (and an earlier audit) reasonably concluded the root set was configurable when
/// it was not. Behaviour is unchanged for every path the collector can actually produce (verified:
/// 0 disagreements across all 905 live crate dirs). It DOES differ on degenerate shapes the
/// collector cannot emit — a root with an empty remainder, and a root with an empty first
/// segment, previously yielded a bogus trailing-slash prefix that could never match
/// `service_tiers`, landing the crate in the unclassified bucket by accident rather than by rule;
/// both now correctly yield `None`. That change is pinned by test.
fn owning_service(
    crate_dir: &str,
    service_roots: &[String],
    capability_roots: &[String],
) -> Option<String> {
    let (root, rest) = crate_dir.split_once('/')?;
    if rest.is_empty() {
        return None;
    }
    if capability_roots.iter().any(|r| r == root) {
        return Some(root.to_owned());
    }
    if !service_roots.iter().any(|r| r == root) {
        return None;
    }
    let svc = rest.split('/').next().filter(|s| !s.is_empty())?;
    Some(format!("{root}/{svc}"))
}

/// Parse a crate `Cargo.toml` for first-party `path = "…"` dependencies, resolving each to a
/// repo-relative crate dir that holds a `Cargo.toml`. The `[lib] path`/`[[bin]] path` `src/…`
/// entries are skipped. Fail-closed on unreadable.
fn cargo_path_deps(
    root: &Path,
    crate_dir: &str,
    out: &mut BTreeSet<String>,
) -> Result<(), CollectError> {
    let manifest = root.join(crate_dir).join("Cargo.toml");
    let text = match fs::read_to_string(&manifest) {
        Ok(t) => t,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(CollectError::Io(format!(
                "read {crate_dir}/Cargo.toml: {e}"
            )));
        }
    };
    for rel in extract_cargo_path_values(&text) {
        // Skip the `[lib]`/`[[bin]]` source paths (they point at src/*, not a sibling crate).
        if rel.ends_with(".rs") || rel.starts_with("src/") || rel.starts_with("src\\") {
            continue;
        }
        let joined = root.join(crate_dir).join(&rel);
        let normalized = normalize_rel(&joined, root);
        if let Some(normalized) = normalized
            && root.join(&normalized).join("Cargo.toml").is_file()
        {
            out.insert(normalized);
        }
    }
    Ok(())
}

/// Extract every `path = "…"` string value from a Cargo.toml's text (the path-dep surface) that
/// participates in the BUILD/LINK dependency graph. A small purpose-built scan (no toml crate
/// dependency, keeping the gate's dep set to serde_json only): each non-comment line of the form
/// `… path = "VALUE" …` yields VALUE. Inline-table deps (`x = { path = "…" }`) and `[deps.x] path =
/// "…"` both match.
///
/// `[dev-dependencies]` (and its `[dev-dependencies.x]` / `[target.'…'.dev-dependencies]` table
/// forms) are SKIPPED: a dev-dep is a test-only edge, not a runtime/link tier ordering, and including
/// them produces phantom cycles (e.g. a producer crate that a gate's TESTS exercise). The tier
/// dependency rule is about the build graph, so only `[dependencies]` + `[build-dependencies]` count.
fn extract_cargo_path_values(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_dev_section = false;
    for raw_line in text.lines() {
        let line = strip_toml_comment(raw_line);
        let trimmed_line = line.trim();
        // Track entry into / out of a `[…]` table header. A dev-dependencies table (top-level or
        // target-scoped or sub-keyed) toggles skipping until the next table header.
        if trimmed_line.starts_with('[') && trimmed_line.contains(']') {
            in_dev_section = trimmed_line.contains("dev-dependencies");
            continue;
        }
        if in_dev_section {
            continue;
        }
        let mut rest = line;
        while let Some(idx) = rest.find("path") {
            let after = &rest[idx + "path".len()..];
            let trimmed = after.trim_start();
            if let Some(eq) = trimmed.strip_prefix('=') {
                let v = eq.trim_start();
                if let Some(stripped) = v.strip_prefix('"')
                    && let Some(end) = stripped.find('"')
                {
                    out.push(stripped[..end].to_owned());
                }
            }
            rest = &rest[idx + "path".len()..];
        }
    }
    out
}

/// Strip a `#` line comment from a TOML line (ignoring `#` inside a double-quoted string).
fn strip_toml_comment(line: &str) -> &str {
    let mut in_str = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_str = !in_str,
            '#' if !in_str => return &line[..i],
            _ => {}
        }
    }
    line
}

/// Parse a crate `BUCK`/`BUCK.v2` for first-party `//path:name` dependency/visibility targets,
/// resolving each `//path` to a repo-relative crate dir. `third-party//…` and bare relative targets
/// are skipped (only first-party `//…` absolute targets are graph edges here). Fail-closed.
fn buck_deps(root: &Path, crate_dir: &str, out: &mut BTreeSet<String>) -> Result<(), CollectError> {
    let mut text = None;
    for name in ["BUCK.v2", "BUCK"] {
        let p = root.join(crate_dir).join(name);
        match fs::read_to_string(&p) {
            Ok(t) => {
                text = Some(t);
                break;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(CollectError::Io(format!("read {crate_dir}/{name}: {e}"))),
        }
    }
    let Some(text) = text else { return Ok(()) };
    for target in extract_buck_first_party_targets(&text) {
        if root.join(&target).join("Cargo.toml").is_file() {
            out.insert(target);
        }
    }
    Ok(())
}

/// Extract the `//path` of every first-party `"//path:name"` BUILD/LINK dependency target in a BUCK
/// file. Scans rule-block by rule-block and collects targets ONLY from `rust_library` / `rust_binary`
/// blocks (the build-graph edges). It SKIPS:
/// - `rust_test` blocks (test-only edges = dev-deps; not a runtime/link tier ordering),
/// - `genrule` / `command_alias` / any non-rust_{library,binary} block (build-tool wiring),
/// - `$(exe …)` / `$(location …)` macro references (a build-tool INVOCATION, not a link dep),
/// - `third-party//…` (vendored) and any non-`//`-rooted target.
///
/// This mirrors the cargo side (which skips `[dev-dependencies]`), keeping both surfaces to the real
/// build dependency graph and avoiding phantom cycles from test/producer wiring.
fn extract_buck_first_party_targets(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (rule, body) in buck_rule_blocks(text) {
        if rule != "rust_library" && rule != "rust_binary" {
            continue;
        }
        // Within a build-target block, drop `$(…)` macro substitutions before harvesting targets so a
        // `$(exe //x:y)` reference is not counted as a link dep.
        let cleaned = strip_buck_macros(&body);
        for lit in buck_string_literals(&cleaned) {
            if let Some(rest) = lit.strip_prefix("//")
                && let Some((path, _name)) = rest.split_once(':')
                && !path.is_empty()
            {
                out.push(path.to_owned());
            }
        }
    }
    out
}

/// Split a BUCK file into `(rule_name, body)` pairs for each top-level `rule_name( … )` call. A small
/// paren-depth scan (BUCK is Starlark; calls are balanced). Bodies are the text between the outermost
/// parens. Comment lines are stripped first so a `#`-commented rule never registers.
fn buck_rule_blocks(text: &str) -> Vec<(String, String)> {
    // Strip `#` line comments (Starlark has no block comments in our BUCK files).
    let mut no_comments = String::with_capacity(text.len());
    for line in text.lines() {
        no_comments.push_str(strip_toml_comment(line));
        no_comments.push('\n');
    }
    let bytes = no_comments.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'(' {
            // Walk back over the identifier immediately preceding `(`.
            let mut k = i;
            while k > 0 && (no_comments.as_bytes()[k - 1] as char).is_whitespace() {
                k -= 1;
            }
            let end = k;
            while k > 0 {
                let ch = no_comments.as_bytes()[k - 1] as char;
                if ch.is_alphanumeric() || ch == '_' {
                    k -= 1;
                } else {
                    break;
                }
            }
            let name = &no_comments[k..end];
            if !name.is_empty() {
                // Find the matching close paren (string-aware).
                if let Some(close) = match_close_paren(&no_comments, i) {
                    out.push((name.to_owned(), no_comments[i + 1..close].to_owned()));
                    i = close + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

/// Index of the `)` matching the `(` at `open`, accounting for double-quoted strings. `None` if
/// unbalanced (a malformed BUCK; the caller harvests nothing from it rather than panicking).
fn match_close_paren(text: &str, open: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    let mut in_str = false;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => in_str = !in_str,
            b'(' if !in_str => depth += 1,
            b')' if !in_str => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Remove `$(…)` Buck macro substitutions (e.g. `$(exe //x:y)`) from a rule body, so the targets they
/// name are not harvested as link deps. Balanced-paren removal after the `$(`.
fn strip_buck_macros(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'(' {
            // Skip to the matching close paren.
            if let Some(close) = match_close_paren(body, i + 1) {
                i = close + 1;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Yield every double-quoted string literal in `text` (used to harvest `"//path:name"` targets).
fn buck_string_literals(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() && bytes[j] != b'"' {
                j += 1;
            }
            out.push(text[start..j.min(text.len())].to_owned());
            i = j + 1;
        } else {
            i += 1;
        }
    }
    out
}

/// Normalize a joined relative path (which may contain `..`) into a clean repo-relative `/`-path.
/// Returns `None` if it escapes the repo root.
fn normalize_rel(joined: &Path, root: &Path) -> Option<String> {
    // Lexically resolve `.`/`..` over the joined path's components, then strip the root prefix.
    let mut stack: Vec<std::ffi::OsString> = Vec::new();
    for comp in joined.components() {
        match comp {
            std::path::Component::ParentDir => {
                stack.pop()?;
            }
            std::path::Component::CurDir => {}
            std::path::Component::Normal(c) => stack.push(c.to_owned()),
            other => stack.push(other.as_os_str().to_owned()),
        }
    }
    let mut resolved = std::path::PathBuf::new();
    for c in stack {
        resolved.push(c);
    }
    resolved
        .strip_prefix(root)
        .ok()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .filter(|s| !s.is_empty())
}

// ───────────────────────────── pure evaluation ─────────────────────────────

/// Pure evaluator. `policy` + `baseline` + `observed` are DATA. Surface-all: every violation is
/// reported (split baselined vs regression), not just the first. The verdict is RED iff there is at
/// least one regression (under `advisory-baseline`) or at least one violation at all (under
/// `blocking`).
#[must_use]
pub fn evaluate(policy: &Value, baseline: &Value, observed: &Value) -> Report {
    let mut findings: Vec<Finding> = Vec::new();

    let parsed = match parse_policy(policy) {
        Ok(parsed) => parsed,
        Err(message) => {
            findings.push(Finding::new(
                "TDA-POLICY-MALFORMED",
                POLICY_KEY,
                format!("{message}; correct the policy before the gate can evaluate"),
                Status::Regression,
            ));
            return finalize(findings, 0, 0, &parsed_or_blocking(policy));
        }
    };

    let baseline = match parse_baseline(baseline) {
        Ok(b) => b,
        Err(message) => {
            findings.push(Finding::new(
                "TDA-BASELINE-MALFORMED",
                POLICY_KEY,
                format!("{message}; correct the frozen baseline before the gate can evaluate"),
                Status::Regression,
            ));
            return finalize(findings, 0, 0, &parsed.enforcement);
        }
    };

    let crates = observed
        .get("crates")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let edges = observed
        .get("edges")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let crate_count = observed
        .get("crate_count")
        .and_then(Value::as_u64)
        .unwrap_or(crates.len() as u64);

    // False-green guard: a broken scan (wrong CWD, empty governed roots) must fail loudly.
    let scan_is_broken = crate_count < parsed.min_expected_crates;
    if scan_is_broken {
        findings.push(Finding::new(
            "TDA-EMPTY-SCAN",
            POLICY_KEY,
            format!(
                "scan found {crate_count} crate(s), below the policy floor of {}; the governed roots, CWD, or collection is likely broken (fail-closed against a silent false-green)",
                parsed.min_expected_crates
            ),
            Status::Regression,
        ));
    }

    // crate-dir -> owning service.
    let mut crate_service: BTreeMap<String, Option<String>> = BTreeMap::new();
    for c in &crates {
        if let Some(dir) = c.get("dir").and_then(Value::as_str) {
            let svc = c.get("service").and_then(Value::as_str).map(str::to_owned);
            crate_service.insert(dir.to_owned(), svc);
        }
    }

    // service -> tier metadata (the projected dependency class).
    let service_tiers = observed
        .get("service_tiers")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let tier_of = |service: &Option<String>| -> Option<Tier> {
        let svc = service.as_ref()?;
        let record = service_tiers.get(svc)?;
        let class = record.get("tier").and_then(Value::as_str)?.to_owned();
        let stratum = record
            .get("stratum")
            .and_then(Value::as_str)
            .map(str::to_owned);
        Some(Tier { class, stratum })
    };

    let edge_count = edges.len();
    for e in &edges {
        let (Some(from), Some(to)) = (
            e.get("from").and_then(Value::as_str),
            e.get("to").and_then(Value::as_str),
        ) else {
            continue;
        };
        let from_svc = crate_service.get(from).cloned().unwrap_or(None);
        let to_svc = crate_service.get(to).cloned().unwrap_or(None);
        // Intra-service edges (same owning service) carry no cross-tier semantics — skip.
        if from_svc.is_some() && from_svc == to_svc {
            continue;
        }
        let (Some(src), Some(dst)) = (tier_of(&from_svc), tier_of(&to_svc)) else {
            // An edge to/from an unclassified (meta) crate is allowed; no tier to compare.
            continue;
        };
        if let Some((code, detail)) = classify_edge(&parsed, &src, &dst, &from_svc, &to_svc) {
            let subject = format!("{from} -> {to}");
            let status = if baseline.contains(&code, &subject) {
                Status::Baselined
            } else {
                Status::Regression
            };
            findings.push(Finding::new(&code, &subject, detail, status));
        }
    }

    // R6: UNDECLARED-ROOT backstop. A governed crate whose top-level root is declared neither in
    // `service_roots` nor in `unclassified_roots` is silently unenforced — `owning_service` gives
    // it no service, so `tier_of` yields None and EVERY edge touching it is skipped above.
    //
    // `unclassified_roots` was previously parsed into the policy struct and never read by any
    // predicate. It therefore looked like the control for that exemption while controlling
    // nothing: the exemption came entirely from `owning_service`'s hardcode. This makes the
    // declaration load-bearing, so adding a crate-bearing top-level root is a deliberate,
    // reviewable act rather than an automatic silent exemption.
    //
    // Currently ZERO on the live corpus (every unclassified root is declared), so this ships as a
    // born-blocking floor with no baselined debt — a new undeclared root REDs immediately and
    // cannot be laundered into the frozen baseline.
    if !scan_is_broken {
        // One finding per ROOT, not per crate: the violation is a property of the root, and a
        // per-crate subject would emit N identical findings for an N-crate root and churn its
        // baseline key every time a crate is added or removed under it.
        let mut undeclared: BTreeMap<&str, &str> = BTreeMap::new();
        for (dir, svc) in &crate_service {
            if svc.is_some() {
                continue;
            }
            let Some((root, _)) = dir.split_once('/') else {
                continue;
            };
            let declared = parsed.unclassified_roots.contains(root)
                || parsed.service_roots.iter().any(|r| r == root)
                || parsed.capability_roots.iter().any(|r| r == root);
            if !declared {
                undeclared.entry(root).or_insert(dir.as_str());
            }
        }
        for (root, example) in undeclared {
            findings.push(Finding::new(
                "TDA-UNDECLARED-ROOT",
                root,
                format!(
                    "crate root `{root}` is declared in none of `service_roots`/`capability_roots`/`unclassified_roots`, so its crates (e.g. `{example}`) carry no tier and EVERY dependency edge touching them is SKIPPED; declare the root (or give it a tier'd manifest) before landing crates under it"
                ),
                Status::Regression,
            ));
        }
    }

    // R6b/R6c are evaluated over POLICY DATA (policy roots + registry facts), never the live crate
    // set, so they sit OUTSIDE the `scan_is_broken` guard that R6 needs. They used to be inside it,
    // contradicting their own comment: a scan broken enough to trip the floor silenced exactly the
    // two structural rules that do not depend on the scan, so the run that most needs them reported
    // neither. TDA-EMPTY-SCAN still REDs the gate on its own; these add their findings alongside it.
    {
        // R6b: `unclassified` is reserved for registry-declared META dirs. A capability root cannot
        // go quiet merely by holding zero crates today — the exemption is the defect, whether or not
        // it is currently load-bearing.
        let registry_capabilities = string_list(observed, "registry_capabilities");
        let registry_meta_dirs = string_list(observed, "registry_meta_dirs");
        for root in &parsed.unclassified_roots {
            if registry_meta_dirs.contains(root.as_str()) {
                continue;
            }
            let detail = if registry_capabilities.contains(root.as_str()) {
                format!(
                    "`{root}` is a REGISTERED CAPABILITY (governance/capability-registry.json) declared in `unclassified_roots`, so every dependency edge touching its crates is SKIPPED — the tier rules do not run on it at all. A capability is a tier-bearing unit (ADR-0562), not a meta tree: move `{root}` to `capability_roots`. Declaring it exempt is what turns the TDA-UNDECLARED-ROOT remedy into a permanent silent exemption"
                )
            } else {
                format!(
                    "`{root}` is declared in `unclassified_roots` but is neither a registered capability nor a `meta_directories` entry of the CLOSED ADR-0562 capability registry, so its exemption rests on nothing reviewable; register it as a meta directory (or give it a tier)"
                )
            };
            findings.push(Finding::new(
                "TDA-UNCLASSIFIED-ROOT-NOT-META",
                root,
                detail,
                status_for(&baseline, "TDA-UNCLASSIFIED-ROOT-NOT-META", root),
            ));
        }

        // R6c: opting INTO `capability_roots` must not become the replacement silent exemption. A
        // declared capability root that compares nothing is exactly as unenforced as an unclassified
        // one — but now with the appearance of enforcement. Always blocking, never baselineable.
        //
        // The bar is a RANKABLE tier, not a PRESENT one. Testing `contains_key` accepted a record
        // that no rule can act on: `classify_edge`'s R4 arm looks the stratum up in `stratum_rank`,
        // and `stratum_rank_order` is S0..S5, so a substrate projecting `forward-declared` returns
        // None there and the `(Some, Some)` arm never matches. Such a root passes `contains_key`, is
        // out of `unclassified_roots` so R6b is quiet, and compares NOTHING — four live capabilities
        // (ci/billing/storage/flags) project exactly that, because every surviving absorbed manifest
        // happens to be forward-declared. The module doc's rank-exemption justification (line ~31,
        // "seed placeholders, not live edges") is valid for a SINGLE forward-declared service; it
        // does not carry over to a 53-crate capability tree inheriting the exemption from one
        // surviving placeholder.
        for root in &parsed.capability_roots {
            let detail = match capability_tier_defect(service_tiers.get(root), &parsed.stratum_rank)
            {
                None => continue,
                Some(TierDefect::Unresolved) => format!(
                    "capability root `{root}` is declared in `capability_roots` but its entry in the ADR-0562 closed capability registry declares no `tier`, so every edge touching its crates is STILL skipped while the policy claims it is enforced. ADR-0562 makes the capability the tier-bearing unit: add the `tier` facet (and `substrate_dag_position.stratum` for a substrate) to `{root}`'s registry entry, or remove `{root}` from `capability_roots` until its tier has been ruled on. It is NOT inferred from the services `{root}` absorbs — a capability whose absorbed services disagree has no defensible tier, and picking one silently is the under-enforcement this gate exists to catch"
                ),
                Some(TierDefect::UnenforceableClass(class)) => format!(
                    "capability root `{root}` declares tier class `{class}`, which no ADR-0245 rule acts on (the rules match `substrate`/`product`/`service-cell`), so every edge touching its crates falls through and is silently allowed while the policy claims the root is enforced. `reserved` is a placeholder for a µservice that ships no crates (ADR-0245), not a class for a crate-bearing capability tree; if that is genuinely the intent, remove `{root}` from `capability_roots` rather than declaring a class that enforces nothing"
                ),
                Some(TierDefect::UnrankableStratum(stratum)) => format!(
                    "capability root `{root}` declares stratum `{stratum}`, which carries no ADR-0280 rank (`stratum_rank_order` is S0..S5), so R4 compares nothing: every intra-substrate edge touching its crates is silently allowed while the policy claims the root is enforced. A DECLARED tier is not automatically a USABLE one — `forward-declared` is exactly as unenforced whether it was declared or derived. Give `{root}` a ranked stratum in its registry entry, or remove it from `capability_roots` until the rank is decided"
                ),
            };
            findings.push(Finding::new(
                "TDA-CAPABILITY-TIER-UNRESOLVED",
                root,
                detail,
                Status::Regression,
            ));
        }
    }

    // R5: acyclicity backstop over the crate graph (cycles always block — a cycle is never debt the
    // baseline can excuse; flag the SCC member edges as a single finding).
    detect_cycles(&edges, &baseline, &mut findings);

    // Baseline-liveness backstop (B3 hardening): every committed baseline subject must still be
    // ANCHORED in the live crate set. A subset baseline never REDs on a stale row, so a row whose
    // subject names a crate that no longer exists (an in-flight strangler MOVE leaves the OLD-path
    // edge as a phantom) silently diverges the baseline from reality — surface it as a blocking
    // regression whose remedy is re-emitting the baseline. A row whose endpoints still exist but whose
    // EDGE was removed is a legitimate burn-down (untouched here). Do not run this on a known-broken
    // scan; TDA-EMPTY-SCAN is already the actionable root-cause finding.
    if !scan_is_broken {
        // The roots a rule can still re-derive a ROOT_SUBJECT_CODES row from (see the root-shaped
        // anchor in `detect_stale_baseline`).
        let declared_roots: BTreeSet<&str> = parsed
            .unclassified_roots
            .iter()
            .map(String::as_str)
            .chain(parsed.capability_roots.iter().map(String::as_str))
            .collect();
        detect_stale_baseline(&baseline, &crate_service, &declared_roots, &mut findings);
    }

    let burned_down = if scan_is_broken {
        0
    } else {
        count_burned_down(&baseline, &findings)
    };
    let mut report = finalize(
        findings,
        crate_count as usize,
        edge_count,
        &parsed.enforcement,
    );
    report.burned_down = burned_down;
    report
}

/// Read a `[String]` field from the observed corpus, defaulting to empty. Absent/malformed is
/// treated as "no registry facts", which makes R6b report EVERY unclassified root rather than
/// exempting them — the fail-closed direction.
fn string_list<'a>(observed: &'a Value, key: &str) -> BTreeSet<&'a str> {
    observed
        .get(key)
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

/// The tier classes [`classify_edge`] actually matches on. A class outside this set reaches its
/// `_ => None` arm, so a root carrying one compares NOTHING — `reserved` (a legal
/// `tier_field_coverage` enum value) and any typo both land there. Kept beside the R6c predicate so
/// adding a rule arm without widening this list is a visible omission rather than a silent one.
const ENFORCED_TIER_CLASSES: [&str; 3] = ["substrate", "product", "service-cell"];

/// Why a capability root's tier cannot enforce anything (R6c). Three distinct causes, reported with
/// distinct remedies because they need different fixes.
enum TierDefect {
    /// No tier declared at all.
    Unresolved,
    /// A class no rule acts on (`reserved`, or a misspelling), so every edge is skipped.
    UnenforceableClass(String),
    /// A `substrate` whose stratum carries no ADR-0280 rank, so R4 compares nothing.
    UnrankableStratum(String),
}

/// The R6c predicate: is this capability root's tier record ACTIONABLE by the rules? `None` means
/// it is (or the root is a non-substrate class, for which R1/R2/R3 need no stratum).
fn capability_tier_defect(
    record: Option<&Value>,
    stratum_rank: &BTreeMap<String, usize>,
) -> Option<TierDefect> {
    // A record with no `tier` string is as inert as no record: `tier_of` returns None on it and the
    // edge is skipped, so treat it as unresolved rather than trusting the shape.
    let Some(class) = record.and_then(|r| r.get("tier")).and_then(Value::as_str) else {
        return Some(TierDefect::Unresolved);
    };
    if !ENFORCED_TIER_CLASSES.contains(&class) {
        return Some(TierDefect::UnenforceableClass(class.to_owned()));
    }
    if class != "substrate" {
        return None;
    }
    match record
        .and_then(|r| r.get("stratum"))
        .and_then(Value::as_str)
    {
        Some(stratum) if stratum_rank.contains_key(stratum) => None,
        Some(stratum) => Some(TierDefect::UnrankableStratum(stratum.to_owned())),
        // A substrate with no stratum at all compares nothing under R4 for the same reason.
        None => Some(TierDefect::UnrankableStratum("<absent>".to_owned())),
    }
}

/// Baselined (known debt, advisory) if the frozen baseline carries the key, else a blocking
/// regression.
fn status_for(baseline: &Baseline, code: &str, subject: &str) -> Status {
    if baseline.contains(code, subject) {
        Status::Baselined
    } else {
        Status::Regression
    }
}

/// Classify a single cross-service edge against the ADR-0245 rules. Returns `Some((code, detail))`
/// for a violation, `None` for an allowed edge.
fn classify_edge(
    parsed: &ParsedPolicy,
    src: &Tier,
    dst: &Tier,
    from_svc: &Option<String>,
    to_svc: &Option<String>,
) -> Option<(String, String)> {
    let sf = from_svc.as_deref().unwrap_or("?");
    let st = to_svc.as_deref().unwrap_or("?");
    match (src.class.as_str(), dst.class.as_str()) {
        // R1: substrate must not depend on product / service-cell.
        ("substrate", "product") | ("substrate", "service-cell") => Some((
            "TDA-SUBSTRATE-UPWARD".to_owned(),
            format!(
                "substrate `{sf}` depends on {} `{st}`; a substrate must sit BENEATH the product/service-cell rings (ADR-0245 R1)",
                dst.class
            ),
        )),
        // R2: product must not depend on service-cell.
        ("product", "service-cell") => Some((
            "TDA-PRODUCT-CELL-CROSS".to_owned(),
            format!(
                "product `{sf}` depends on service-cell `{st}`; product and service-cell are sibling rings that never cross (ADR-0245 R2)"
            ),
        )),
        // R3: service-cell must not depend on product.
        ("service-cell", "product") => Some((
            "TDA-CELL-PRODUCT".to_owned(),
            format!(
                "service-cell `{sf}` depends on product `{st}`; product and service-cell are sibling rings that never cross (ADR-0245 R2/R3)"
            ),
        )),
        // R4: intra-substrate S-rank — dep may only point to an equal-or-lower S-rank.
        ("substrate", "substrate") => {
            let sr = src
                .stratum
                .as_deref()
                .and_then(|s| parsed.stratum_rank.get(s));
            let dr = dst
                .stratum
                .as_deref()
                .and_then(|s| parsed.stratum_rank.get(s));
            match (sr, dr) {
                (Some(&sr), Some(&dr)) if dr > sr => Some((
                    "TDA-S-RANK-INVERSION".to_owned(),
                    format!(
                        "substrate `{sf}` ({}) depends on higher-S-rank substrate `{st}` ({}); an intra-substrate edge may only point to an equal-or-lower S-rank (ADR-0280 §D-1)",
                        src.stratum.as_deref().unwrap_or("?"),
                        dst.stratum.as_deref().unwrap_or("?")
                    ),
                )),
                // Either side is rank-exempt (`forward-declared` / unranked) — allowed.
                _ => None,
            }
        }
        _ => None,
    }
}

/// Tarjan SCC over the crate edge list; any SCC of size > 1 (or a self-loop) is a cycle (R5). A
/// cycle is ALWAYS a regression (never baselineable debt). One finding per SCC, keyed by its sorted
/// member list.
fn detect_cycles(edges: &[Value], baseline: &Baseline, findings: &mut Vec<Finding>) {
    let mut nodes: BTreeSet<String> = BTreeSet::new();
    let mut adj: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut self_loops: BTreeSet<String> = BTreeSet::new();
    for e in edges {
        let (Some(from), Some(to)) = (
            e.get("from").and_then(Value::as_str),
            e.get("to").and_then(Value::as_str),
        ) else {
            continue;
        };
        nodes.insert(from.to_owned());
        nodes.insert(to.to_owned());
        if from == to {
            self_loops.insert(from.to_owned());
        } else {
            adj.entry(from.to_owned())
                .or_default()
                .insert(to.to_owned());
        }
    }
    for node in &self_loops {
        let subject = format!("{node} -> {node}");
        let status = cycle_status(baseline, &subject);
        findings.push(Finding::new(
            "TDA-CYCLE",
            &subject,
            format!(
                "self-loop on `{node}` (a 1-cycle; the crate graph must be acyclic, ADR-0280 R5)"
            ),
            status,
        ));
    }
    for scc in tarjan_sccs(&nodes, &adj) {
        if scc.len() > 1 {
            let subject = scc.join(",");
            let status = cycle_status(baseline, &subject);
            findings.push(Finding::new(
                "TDA-CYCLE",
                &subject,
                format!(
                    "strongly-connected component of size {} is a cycle: {}",
                    scc.len(),
                    scc.join(" -> ")
                ),
                status,
            ));
        }
    }
}

/// A cycle is a regression unless explicitly carried in the baseline (cycles SHOULD never be
/// baselined; the lookup exists so a pre-existing cycle, were one present, is treated consistently).
fn cycle_status(baseline: &Baseline, subject: &str) -> Status {
    if baseline.contains("TDA-CYCLE", subject) {
        Status::Baselined
    } else {
        Status::Regression
    }
}

/// Iterative Tarjan SCC (no recursion → no stack overflow on adversarial input, honouring the
/// no-panic doctrine). Pure, O(V+E). Each returned component is a sorted node list.
pub fn tarjan_sccs(
    nodes: &BTreeSet<String>,
    adj: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<Vec<String>> {
    let mut index_counter: usize = 0;
    let mut indices: BTreeMap<String, usize> = BTreeMap::new();
    let mut lowlink: BTreeMap<String, usize> = BTreeMap::new();
    let mut on_stack: BTreeSet<String> = BTreeSet::new();
    let mut stack: Vec<String> = Vec::new();
    let mut sccs: Vec<Vec<String>> = Vec::new();

    enum Step {
        Enter(String),
        Resume(String, usize),
    }

    let succs: BTreeMap<String, Vec<String>> = adj
        .iter()
        .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
        .collect();

    for start in nodes {
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
                        work.push(Step::Resume(v.clone(), cursor + 1));
                        if !indices.contains_key(&w) {
                            work.push(Step::Enter(w));
                        } else if on_stack.contains(&w) {
                            let vi = *lowlink.get(&v).unwrap_or(&0);
                            let wi = *indices.get(&w).unwrap_or(&0);
                            lowlink.insert(v.clone(), vi.min(wi));
                        }
                    } else {
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

/// Count baselined violations that are NO LONGER present in the live findings (burn-down progress).
fn count_burned_down(baseline: &Baseline, findings: &[Finding]) -> usize {
    let stale_subjects: BTreeSet<&str> = findings
        .iter()
        .filter(|f| f.code == "TDA-STALE-BASELINE")
        .map(|f| f.subject.as_str())
        .collect();
    let live: BTreeSet<String> = findings
        .iter()
        .filter(|f| f.code != "TDA-POLICY-MALFORMED" && f.code != "TDA-BASELINE-MALFORMED")
        .map(|f| Baseline::key_of(&f.code, &f.subject))
        .collect();
    baseline
        .keys
        .iter()
        .filter(|k| {
            let subject = k.split_once('|').map_or(k.as_str(), |(_, subject)| subject);
            !stale_subjects.contains(subject) && !live.contains(*k)
        })
        .count()
}

/// Baseline-liveness backstop (B3 hardening). A subset-semantics baseline blocks only on NEW
/// regressions, so a STALE row silently rots: a crate MOVED by an in-flight strangler leaves its
/// OLD-path edge in the baseline as a PHANTOM (`from`/`to` names a crate dir that no longer exists),
/// and subset semantics never RED on a stale row. This asserts every committed baseline subject is
/// still ANCHORED — each crate dir it names must exist in the live crate set (`live_crate_dirs` are the
/// collected crate dirs, keyed by owning service). A phantom row is a blocking regression whose remedy
/// is re-emitting the baseline (`--emit-baseline`), which drops it. A row whose endpoints still exist
/// but whose EDGE was removed is a legitimate burn-down (the inversion was fixed) and is NOT flagged.
fn detect_stale_baseline(
    baseline: &Baseline,
    live_crate_dirs: &BTreeMap<String, Option<String>>,
    declared_roots: &BTreeSet<&str>,
    findings: &mut Vec<Finding>,
) {
    for key in &baseline.keys {
        let Some((code, subject)) = key.split_once('|') else {
            continue;
        };
        // Policy/scan sentinels carry no crate dir — never a phantom.
        if subject == POLICY_KEY {
            continue;
        }
        // Root-subject codes name a top-level ROOT (`iam`), not a crate dir (`iam/core/api`). A root
        // is never a key in the live crate set, so anchoring them against it would fire a bogus
        // TDA-STALE-BASELINE on every such row — a false RED. They get the ROOT-SHAPED anchor
        // instead: R6b re-derives its subjects from `unclassified_roots` and R6c from
        // `capability_roots`, so a row naming a root in NEITHER list can never be re-derived by any
        // rule and is permanently inert — the same phantom class, one level up. Without this the
        // root family (now 21 of the 38 committed rows, 55% of the baseline) had no staleness
        // detector at all, which is exactly the hole B3 closed for edges.
        if ROOT_SUBJECT_CODES.contains(&code) {
            if !declared_roots.contains(subject) {
                findings.push(Finding::new(
                    "TDA-STALE-BASELINE",
                    subject,
                    format!(
                        "baseline `{code}` row names root `{subject}`, declared in neither \
                         `unclassified_roots` nor `capability_roots`, so no rule can re-derive it — \
                         a phantom ROOT row (the root was renamed, removed, or tier-classified into \
                         `service_roots`). It is permanently inert and inflates the burn-down count; \
                         delete the row"
                    ),
                    Status::Regression,
                ));
            }
            continue;
        }
        if let Some(missing) =
            subject_crate_dirs(subject).find(|cdir| !live_crate_dirs.contains_key(*cdir))
        {
            findings.push(Finding::new(
                "TDA-STALE-BASELINE",
                subject,
                format!(
                    "baseline `{code}` entry names crate `{missing}`, absent from the live corpus — a \
                     phantom row (the crate was moved/renamed/removed). A subset baseline never REDs on \
                     a stale row, so it silently diverges from reality; delete the row, or re-emit the \
                     baseline (`--emit-baseline`, which drops phantom EDGE rows while carrying the \
                     still-live hand-added ROOT rows forward)"
                ),
                Status::Regression,
            ));
        }
    }
}

/// The crate dirs a baseline subject references. An edge subject is `from -> to`; a multi-node cycle
/// subject is a comma-joined node list; both split into the endpoint crate dirs (trimmed, non-empty).
fn subject_crate_dirs(subject: &str) -> impl Iterator<Item = &str> {
    subject
        .split(" -> ")
        .flat_map(|part| part.split(','))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

/// Sort findings, tally baselined/regression counts, and decide the verdict per enforcement mode.
fn finalize(
    mut findings: Vec<Finding>,
    crates_checked: usize,
    edges_checked: usize,
    enforcement: &str,
) -> Report {
    findings.sort();
    findings.dedup();
    let baselined = findings
        .iter()
        .filter(|f| f.status == Status::Baselined)
        .count();
    let regressions = findings
        .iter()
        .filter(|f| f.status == Status::Regression)
        .count();
    // advisory-baseline: RED iff there is at least one regression.
    // blocking (post-burn-down flip): RED iff there is ANY violation at all.
    let verdict = match enforcement {
        "blocking" if !findings.is_empty() => Verdict::Red,
        "blocking" => Verdict::Green,
        _ if regressions > 0 => Verdict::Red,
        _ => Verdict::Green,
    };
    Report {
        findings,
        crates_checked,
        edges_checked,
        baselined,
        regressions,
        burned_down: 0,
        verdict,
    }
}

/// Best-effort enforcement read for the policy-malformed early return (default advisory-baseline).
fn parsed_or_blocking(policy: &Value) -> String {
    policy
        .get("enforcement")
        .and_then(Value::as_str)
        .unwrap_or("advisory-baseline")
        .to_owned()
}

// ───────────────────────────── re-freeze (--emit-baseline) ─────────────────────────────

/// The `--emit-baseline` re-freeze document: the live violation set in baseline shape. Pure (the
/// binary only prints it), because what it drops is the whole risk — the documented remedy for
/// TDA-STALE-BASELINE and the baseline `_comment`'s own regenerate instruction both say to run it
/// and overwrite the file, so anything not reproduced here is DESTROYED by following the docs.
///
/// Three families, three treatments:
/// - EDGE rows — re-derived from `report.findings` (the point of a re-freeze).
/// - Diagnostics (`TDA-POLICY-MALFORMED` / `TDA-BASELINE-MALFORMED` / `TDA-STALE-BASELINE`) —
///   dropped. `evaluate` never consults the baseline for them, so a row would be permanently inert:
///   it inflates `burned_down` forever, and if its subject crate later moves TDA-STALE-BASELINE
///   fires on a row that should never have existed.
/// - ROOT rows ([`ROOT_SUBJECT_CODES`], the structural R6/R6b/R6c findings) — never MINTED from the
///   live findings, but CARRIED FORWARD from `committed`, filtered to rows still live. Minting them
///   would make "re-run --emit-baseline" the remedy for "you exempted a capability root from tier
///   enforcement" — the self-service laundering that let `unclassified_roots` grow 3 -> 27. Dropping
///   them outright emitted 17 rows against 38 committed, deleting the 21 hand-added root rows and
///   turning them into 21 blocking regressions on the next run. Carry-forward-filtered is the only
///   treatment that is neither: a burned-down root row drops (its finding is gone) and an uncommitted
///   one can never enter (nothing here is sourced from the findings).
///
/// `_comment` / `frozen_at_ref` are carried forward for the same reason.
#[must_use]
pub fn emit_baseline_doc(report: &Report, committed: &Value) -> Value {
    let mut rows: BTreeSet<(&str, &str)> = report
        .findings
        .iter()
        .filter(|f| {
            f.code != "TDA-POLICY-MALFORMED"
                && f.code != "TDA-BASELINE-MALFORMED"
                && f.code != "TDA-STALE-BASELINE"
                && !ROOT_SUBJECT_CODES.contains(&f.code.as_str())
        })
        .map(|f| (f.code.as_str(), f.subject.as_str()))
        .collect();

    let live: BTreeSet<(&str, &str)> = report
        .findings
        .iter()
        .map(|f| (f.code.as_str(), f.subject.as_str()))
        .collect();
    for row in committed
        .get("violations")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let (Some(code), Some(subject)) = (
            row.get("code").and_then(Value::as_str),
            row.get("subject").and_then(Value::as_str),
        ) else {
            continue;
        };
        if ROOT_SUBJECT_CODES.contains(&code) && live.contains(&(code, subject)) {
            rows.insert((code, subject));
        }
    }

    let mut doc = serde_json::Map::new();
    for field in ["_comment", "frozen_at_ref"] {
        if let Some(v) = committed.get(field) {
            doc.insert(field.to_owned(), v.clone());
        }
    }
    doc.insert("gate_id".to_owned(), json!(GATE_ID));
    doc.insert("burn_down_target".to_owned(), json!(0));
    doc.insert(
        "violations".to_owned(),
        Value::Array(
            rows.into_iter()
                .map(|(code, subject)| json!({ "code": code, "subject": subject }))
                .collect(),
        ),
    );
    Value::Object(doc)
}

// ───────────────────────────── rendering ─────────────────────────────

/// Render a deterministic multi-line report for the binary / CI logs.
#[must_use]
pub fn render(report: &Report) -> String {
    let mut lines = Vec::new();
    let head = match report.verdict {
        Verdict::Green => format!(
            "{GATE_ID}: GREEN — {} crate(s), {} edge(s); {} baselined (advisory) violation(s), {} regression(s), {} burned down",
            report.crates_checked,
            report.edges_checked,
            report.baselined,
            report.regressions,
            report.burned_down
        ),
        Verdict::Red => format!(
            "{GATE_ID}: RED — {} regression(s) over the frozen baseline ({} baselined advisory, {} burned down) across {} crate(s)/{} edge(s)",
            report.regressions,
            report.baselined,
            report.burned_down,
            report.crates_checked,
            report.edges_checked
        ),
    };
    lines.push(head);
    for f in &report.findings {
        let tag = match f.status {
            Status::Baselined => "BASELINED",
            Status::Regression => "REGRESSION",
        };
        lines.push(format!("  [{tag}] {} {}: {}", f.code, f.subject, f.detail));
    }
    lines.join("\n")
}

// ───────────────────────────── load helpers (I/O) ─────────────────────────────

/// Read + parse a JSON document at `<root>/<path>`. The only JSON I/O helper.
pub fn load_json(root: &Path, path: &str) -> Result<Value, CollectError> {
    let full = root.join(path);
    let text = fs::read_to_string(&full)
        .map_err(|e| CollectError::Io(format!("{}: {e}", full.display())))?;
    serde_json::from_str(&text).map_err(|e| CollectError::Parse {
        path: path.to_owned(),
        message: e.to_string(),
    })
}

#[cfg(test)]
mod tests;
