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
//! 3. Carve-outs (vendor/generated/ephemeral/...) live as DATA in the bundled
//!    oya-ci-config unit-class + ttl tables (`Policy::from_config`), never as scanner
//!    branches (Linus: the exception lives in the table). The classifier walks the
//!    table; it has zero hard-coded special cases.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The buck2 target that produces the registry — recorded in `_provenance`.
pub const PRODUCER_TARGET: &str =
    "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin";

/// A producer error. No panics escape the production path.
///
/// The first two variants cover the face-builders (policy parse + serialization). The last
/// three cover the registration BRIDGES moved into this library (slice 2.5 — `fix_owners`,
/// `fix_reachability`, `allocate_next_adr_id`): the bridges express filesystem failures
/// ([`ProducerError::Io`]), input/shape/parse rejections ([`ProducerError::Validation`]),
/// and refusal of an unsafe or already-applied registration ([`ProducerError::Refused`]).
/// The thin CLI adapter maps these back to its own `CliError` so the binary's messages and
/// exit semantics are unchanged (CLI surfaces are retirement-marked: the logic lives here,
/// the binary is a transitional adapter).
#[derive(Debug)]
pub enum ProducerError {
    Policy(String),
    Serialize(String),
    /// A filesystem read/write failed (the message already names the path + cause).
    Io(String),
    /// An input or on-disk artifact failed validation/shape/parse (bad `<dir>=<owner>`
    /// spec, invalid OWNERS principal, malformed reachability registry, path traversal).
    Validation(String),
    /// A registration was REFUSED for a safety reason (OWNERS already exists, coverage over
    /// the breadth bound, prefix already registered, self-validation did not take). The
    /// message names the exact fix; refusal leaves NO residue (the bridges revert on refusal).
    Refused(String),
}

impl ProducerError {
    /// The bare message carried by a bridge-originated variant (`Io`/`Validation`/`Refused`),
    /// WITHOUT the variant's Display prefix. The CLI adapter uses this to re-wrap the error in
    /// its own `CliError::Io` so the binary's stderr stays byte-identical to before the
    /// extraction. Returns `None` for the face-builder variants (`Policy`/`Serialize`), which
    /// the adapter already routes through `CliError::Producer`.
    pub fn bridge_message(&self) -> Option<&str> {
        match self {
            ProducerError::Io(message)
            | ProducerError::Validation(message)
            | ProducerError::Refused(message) => Some(message.as_str()),
            ProducerError::Policy(_) | ProducerError::Serialize(_) => None,
        }
    }
}

impl std::fmt::Display for ProducerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProducerError::Policy(message) => write!(f, "policy error: {message}"),
            ProducerError::Serialize(message) => write!(f, "serialize error: {message}"),
            ProducerError::Io(message) => write!(f, "{message}"),
            ProducerError::Validation(message) => write!(f, "{message}"),
            ProducerError::Refused(message) => write!(f, "{message}"),
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
    /// path -> nearest up-tree OWNERS-resolved owner. Absent ⇒ unowned (RED). Ownership
    /// requires existence AND schema-valid content under the per-file breadth bound
    /// (ADR-0555 hardening, FRIC-1781400000) — the binary's `resolve_owners` enforces
    /// both before a path ever lands in this map.
    pub owners: BTreeMap<String, String>,
    /// path -> justification ref (ADR-####/spec $id/need:<ticket>). Absent AND unreached ⇒
    /// unjustified: `build_registry` falls back to `reached:<source>` for any path a live
    /// reachability source reaches (reached ⇒ justified).
    pub justifications: BTreeMap<String, String>,
    /// path -> the registries that reach it (masterplan|root-hub|cargo-members|doc-catalog|crosswalk).
    pub reachability: BTreeMap<String, Vec<String>>,
    /// path -> canonical path it duplicates (drives the MERGE verdict). Absent ⇒ not a dup.
    pub dup_of: BTreeMap<String, String>,
    /// Every `old_path` named by a COMMITTED `specs/reorg/<capability>-move-plan.json`, as
    /// discovered by the codemod's own `discover_committed_move_plans`. Empty ⇒ no capability
    /// has a committed plan, so every derived `move` is unplanned — which is the honest reading,
    /// not a reason to stay silent.
    pub planned_move_paths: BTreeSet<String>,
    /// The DECLARED placement authority, parsed from `governance/capability-registry.json`.
    /// Empty ⇒ every destination is `None` ⇒ every disposition is `unclassified` (fail-closed).
    pub placement: CapabilityPlacement,
    /// The OWNERS files (repo-relative) that PARSE against the ADR-0555 schema — the
    /// `valid_files` half of [`resolve_owners`]'s outcome. Membership here is the ONLY
    /// signal `build_registry` accepts for the [`OWNERS_SCHEMA_ANCHOR`] accounting floor,
    /// which is why the set carries the parse VERDICT rather than the name shape: a file
    /// named `OWNERS` that fails `parse_owners_content` is absent from this set and stays
    /// fully RED. Empty ⇒ no path gets the floor (the pre-hardening behaviour).
    pub valid_owners_files: BTreeSet<String>,
}

/// The carve-out + TTL policy, parsed once from the DATA tables.
pub struct Policy {
    rules: Vec<ClassRule>,
    ttl_by_class: BTreeMap<String, TtlRecord>,
}

impl Policy {
    /// Parse the bundled DATA tables from oya-ci-config (SSOT). Returns an error rather
    /// than panicking on malformed data.
    pub fn from_bundled() -> Result<Self, ProducerError> {
        Self::from_config(&oya_ci_config_kernel::OyaCiConfig::bundled_default())
    }

    /// Parse the carve-out + TTL tables from the oya-ci config (OYA-CI-CONFORMANCE-FLOOR-PLAN
    /// §3.3). The config carries these two tables as DATA (the `[unit_class]` + `[ttl]`
    /// sections); the bundled default is the SSOT, so this is equivalent to
    /// [`Policy::from_bundled`] under the default config.
    pub fn from_config(cfg: &oya_ci_config_kernel::OyaCiConfig) -> Result<Self, ProducerError> {
        Self::from_strs(cfg.unit_class_policy_json(), cfg.ttl_policy_json())
    }

    pub fn from_strs(unit_class_json: &str, ttl_json: &str) -> Result<Self, ProducerError> {
        let unit_value: Value = serde_json::from_str(unit_class_json)
            .map_err(|e| ProducerError::Policy(format!("unit-class-policy.json: {e}")))?;
        let rules_value = unit_value.get("rules").ok_or_else(|| {
            ProducerError::Policy("unit-class-policy.json missing 'rules'".into())
        })?;
        let rules: Vec<ClassRule> = serde_json::from_value(rules_value.clone())
            .map_err(|e| ProducerError::Policy(format!("rules: {e}")))?;

        let ttl_value: Value = serde_json::from_str(ttl_json)
            .map_err(|e| ProducerError::Policy(format!("ttl-policy.json: {e}")))?;
        let by_class_value = ttl_value.get("by_unit_class").ok_or_else(|| {
            ProducerError::Policy("ttl-policy.json missing 'by_unit_class'".into())
        })?;
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
            // `root_*` kinds are generic predicates (NOT scratch special-cases): a path is
            // "at repo root" iff it carries no `/` separator. They let the DATA table express
            // root-anchored carve-outs (e.g. the scratch-artifact class) without a glob engine,
            // keeping the classifier branch-per-KIND, never branch-per-path.
            let is_root = !path.contains('/');
            let hit = match rule.kind.as_str() {
                "prefix" => path.starts_with(&rule.value),
                "suffix" => path.ends_with(&rule.value),
                "contains" => path.contains(&rule.value),
                "exact" => path == rule.value,
                "root_suffix" => is_root && path.ends_with(&rule.value),
                "root_exact" => is_root && path == rule.value,
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

/// The `justification_ref` AND `reachable_from` source stamped on a schema-valid OWNERS
/// file (ADR-0555 §ownership-registration). It is deliberately NOT an `ADR-####` id: the
/// value is derived from the OWNERS SCHEMA rather than from any prose mention, and a
/// reviewer must be able to tell the two apart at a glance (`justification_ref ==
/// "owners-schema"` enumerates every by-construction row). It is also portable — a repo
/// adopting this producer inherits the derivation, not an Oyatie decision id.
///
/// WHY the derivation exists (FRIC: PR #1473 turned `dev` RED with a one-line `os/OWNERS`).
/// An OWNERS file is the ownership PRIMITIVE this very producer resolves rows against
/// (`resolve_owners`), so demanding that each one ALSO be named in ADR prose and listed in
/// `specs/reachability-registry.json` made the accounting system self-referential: 49 of the
/// registry's 124 rows existed only to permit an OWNERS file, each carrying hundreds of
/// characters of hand-written anchor prose, and every capability move had to hand-edit that
/// one global file or turn the branch RED. ADR-0562 §10.29 wrote the obligation down ("the
/// OWNERS file rides the `git mv`; its registry entry does not") and the very next PR forgot
/// it — a written-down manual obligation that gets forgotten needs a detector, not more
/// discipline. Accounting an OWNERS file by CONSTRUCTION removes the obligation entirely.
pub const OWNERS_SCHEMA_ANCHOR: &str = "owners-schema";

/// A single accounting record (the 11 fields of PHASE-0-FIREWALL-PLAN §5.1).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AccountingRecord {
    pub path: String,
    pub unit_class: String,
    pub owner: Option<String>,
    pub justification_ref: Option<String>,
    pub reachable_from: Vec<String>,
    pub ttl: TtlRecord,
    pub tracked: bool,
    pub verdict: String,
    /// Where this path BELONGS, from the closed capability registry (or the owner root).
    /// `None` ⇒ no declared home; the disposition is then `unclassified` and blocks.
    pub destination: Option<String>,
    /// How this path TERMINATES. Derivable TODAY from row facts alone:
    /// `retain | move | generate | externalize | delete | unclassified`.
    ///
    /// `refactor` and `rewrite` are deliberately NOT emitted: distinguishing "move it" from
    /// "rewrite it on arrival" needs a CONTENT signal (e.g. the 216 `docs/runbooks/` files that
    /// are `Stub` templates carrying `TODO — fill at …`), and [`derive_disposition`] is pure
    /// over the row. Emitting a variant no rule can produce would be a lie in the schema, so
    /// they wait for the content pass rather than being declared and never reached.
    pub disposition: String,
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

/// The DECLARED placement authority: `governance/capability-registry.json` (ADR-0562 as
/// amended by ADR-0615). This is a closed registry — capability roots, the five faces, and the
/// meta directories are enumerated there, so destination is a LOOKUP over declared data and
/// never a heuristic over path shape.
///
/// Deliberately NOT a new DATA table. An earlier draft of this work proposed seeding a
/// destination table from `specs/integ-branch-envelopes.json#reorg_debt_freeze`; that would
/// have added a file under `libs/` (which ADR-0562 dissolves) to restate a mapping the
/// capability registry already declares — and `reorg_debt_freeze` still carries 90 of 171 rows
/// at `judgment_status: pending`, so it is a worse source than the registry it was derived from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilityPlacement {
    /// current dir prefix -> capability name. From `capabilities[].absorbs_current_dirs`.
    /// Longest-prefix wins, so `iam/cloud-iam` beats `iam`.
    pub absorbs: BTreeMap<String, String>,
    /// Meta directories that are their own destination (`kernel/`, `governance/`, `app/`, ...).
    pub meta_dirs: Vec<String>,
}

impl CapabilityPlacement {
    /// Parse the closed registry. A malformed or absent registry yields an EMPTY placement,
    /// which makes every destination `None` and therefore every disposition `unclassified` —
    /// fail-closed, never a silent fallback to "looks fine where it is".
    pub fn from_registry_value(value: &Value) -> Self {
        let mut absorbs = BTreeMap::new();
        if let Some(caps) = value.get("capabilities").and_then(Value::as_array) {
            for cap in caps {
                let Some(name) = cap.get("name").and_then(Value::as_str) else {
                    continue;
                };
                for dir in cap
                    .get("absorbs_current_dirs")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    let key = dir.trim_end_matches('/').to_owned();
                    if !key.is_empty() {
                        absorbs.insert(key, name.to_owned());
                    }
                }
            }
        }
        let meta_dirs = value
            .get("meta_directories")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|m| m.get("dir").and_then(Value::as_str))
            .map(|d| d.trim_end_matches('/').to_owned())
            .filter(|d| !d.is_empty())
            .collect();
        Self { absorbs, meta_dirs }
    }

    /// Longest declared prefix that owns `path`, if any.
    fn capability_for(&self, path: &str) -> Option<&str> {
        let mut best: Option<(usize, &str)> = None;
        for (dir, name) in &self.absorbs {
            let owns = path == dir || path.starts_with(&format!("{dir}/"));
            if owns && best.is_none_or(|(len, _)| dir.len() > len) {
                best = Some((dir.len(), name.as_str()));
            }
        }
        best.map(|(_, name)| name)
    }

    fn meta_dir_for(&self, path: &str) -> Option<&str> {
        self.meta_dirs
            .iter()
            .find(|d| path.starts_with(&format!("{d}/")))
            .map(String::as_str)
    }
}

/// Where this path BELONGS, derived from declared authority only.
///
/// Order is load-bearing and mirrors the authority chain: a meta directory is its own
/// destination (it is enumerated in the registry as such); otherwise the capability that
/// declares it absorbed; otherwise the owner's root, because an owned tree has a defined home
/// even when the capability registry has not yet absorbed it. A path with none of the three has
/// NO derivable destination and must not be given one by guessing.
fn derive_destination(
    path: &str,
    placement: &CapabilityPlacement,
    owner: &Option<String>,
) -> Option<String> {
    if let Some(meta) = placement.meta_dir_for(path) {
        return Some(format!("{meta}/"));
    }
    if let Some(capability) = placement.capability_for(path) {
        return Some(format!("{capability}/"));
    }
    // The owner root is a weaker but still DECLARED signal: `resolve_owners` resolved it from a
    // schema-valid OWNERS file, so the tree has an accountable home.
    owner
        .as_deref()
        .and_then(|o| o.strip_prefix("OWNERS:"))
        .filter(|root| !root.is_empty())
        .map(|root| format!("{}/", root.trim_end_matches('/')))
}

/// How this path TERMINATES. Total and order-sensitive, exactly like [`derive_verdict`], and
/// PURE over facts the row already carries — no file reads, no clock.
///
/// `unclassified` is the fail-closed outcome and BLOCKS its domain's cutover. It is never a
/// silent default: every other arm requires a positive signal.
///
/// Measured on the live tree at the time of writing (16,643 rows):
/// `retain 10,276 · unclassified 5,401 · move 940 · generate 26`. `delete` and `externalize`
/// have rules here but do not fire on this corpus — `dup_of` is empty in the producer path and
/// `third-party/` is excluded from the registry by config — so they are reachable by
/// construction, not dead. `refactor`/`rewrite` have NO rule and are not emitted; see the
/// `disposition` field doc for why.
fn derive_disposition(
    verdict: &str,
    unit_class: &str,
    destination: &Option<String>,
    path: &str,
    dup_of: &Option<String>,
) -> String {
    // A duplicate terminates by merging into its canonical twin, whatever else is true of it.
    if dup_of.is_some() {
        return "delete".into();
    }
    // Generated output is not authored and not moved: it terminates by being regenerated at its
    // declared path. Its class already proves a producer owns it.
    if unit_class == "generated" {
        return "generate".into();
    }
    // Vendored code is owned elsewhere by definition.
    if unit_class == "vendor" {
        return "externalize".into();
    }
    // Scratch has a delete TTL by class; nothing is extracted from it.
    if unit_class == "scratch" {
        return "delete".into();
    }
    // Reached by nothing and justified by nothing. The tree does not know why this exists, so
    // no destination can be honest about where it should go.
    if verdict == "RED" {
        return "unclassified".into();
    }
    let Some(destination) = destination else {
        // Owned or reached, but no declared home. This is the NEEDS-OWNER / unabsorbed
        // population; it blocks rather than defaulting to "leave it".
        return "unclassified".into();
    };
    // Already inside its declared destination ⇒ it stays.
    if path.starts_with(destination.as_str()) {
        return "retain".into();
    }
    "move".into()
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
        let mut justification_ref = inputs.justifications.get(path).cloned();
        let mut reachable_from = inputs.reachability.get(path).cloned().unwrap_or_default();

        // The OWNERS accounting FLOOR (see [`OWNERS_SCHEMA_ANCHOR`]). A file that parses
        // against the ADR-0555 OWNERS schema is categorically justified (the schema is the
        // decision that says it exists) and categorically reachable (this producer's own
        // `resolve_owners` reaches it on every run) — no prose mention, no registry row.
        //
        // Two properties make this a floor rather than a laundry:
        //  1. It is keyed on the PARSE VERDICT, never the filename. `valid_owners_files`
        //     holds only files `parse_owners_content` accepted, so a comment-only / garbage
        //     / non-UTF-8 OWNERS file gets nothing here and stays unjustified + unreachable
        //     + unowned. Invalid ownership markers must stay RED, or the fail-closed
        //     resolution ADR-0555 hardened would be reachable-by-renaming-a-file.
        //  2. It only ever FILLS AN ABSENCE. A path that already resolved a justification
        //     or a reachability source keeps exactly what it resolved, so the derivation
        //     provably cannot alter any row that was already accounted — it can only turn
        //     an otherwise-RED valid OWNERS file GREEN.
        if inputs.valid_owners_files.contains(path) {
            justification_ref.get_or_insert_with(|| OWNERS_SCHEMA_ANCHOR.to_owned());
            if reachable_from.is_empty() {
                reachable_from.push(OWNERS_SCHEMA_ANCHOR.to_owned());
            }
        }

        // REACHED ⇒ JUSTIFIED. Every reachability source is itself a reviewed design act — a
        // masterplan/root-hub/DOC-CATALOG entry, a workspace Cargo member registration, or a
        // reachability-registry entry that MUST carry a non-empty `anchor` naming why the tree
        // is reached (`load_reachability_registry`). Demanding a SECOND, prose restatement of
        // that decision in an ADR body added no signal and blocked real work: the ADR-mention
        // resolver is a context-free token match, so it credits a path named in a prohibition,
        // an allowlist, or a Rejected ADR exactly as it credits a decision, and it collides on
        // bare basenames (73 ADRs contain the token `Cargo.toml`).
        //
        // The fallback is a RULE, never a per-path exemption list, and it is strictly weaker
        // than reachability: a path reached by NOTHING leaves `reachable_from` empty, so this
        // yields `None` and the row still raises BOTH `unjustified` and `unreachable`.
        // `resolve_reachability` pushes sources in a fixed order, so `first()` is deterministic
        // and the face stays byte-stable. The ADR corpus still wins when it names the path, so
        // no existing `justification_ref` changes value.
        //
        // It runs AFTER the OWNERS floor, and the ORDER IS LOAD-BEARING — not because the
        // floor's `reachable_from` push feeds `first()` (it cannot: the floor fills
        // `justification_ref` for every valid OWNERS file, so this fallback is always a no-op
        // on them, and `reached:owners-schema` is unreachable by construction), but because a
        // valid OWNERS file that ALSO resolves a reachability source would otherwise be stamped
        // `reached:cargo-members` instead of `owners-schema`. That would silently empty the
        // `justification_ref == "owners-schema"` census [`OWNERS_SCHEMA_ANCHOR`] documents as
        // the way a reviewer enumerates every by-construction row.
        if justification_ref.is_none() {
            justification_ref = reachable_from.first().map(|src| format!("reached:{src}"));
        }
        // No last-touch column (ADR-0552, FRIC-1781234047): per-path last-touch revision ids
        // are HISTORY-derived volatile facts — a squash-merge rewrites them for every path
        // the PR touched, so embedding them here made the committed face un-settle on every
        // merge to the base branch. They live in the untracked scm-volatile-facts snapshot;
        // the staleness gate joins rows to ages at evaluation time.
        let dup_of = inputs.dup_of.get(path).cloned();

        let verdict = derive_verdict(&owner, &justification_ref, &reachable_from, &ttl, &dup_of);
        let destination = derive_destination(path, &inputs.placement, &owner);
        let disposition = derive_disposition(&verdict, &unit_class, &destination, path, &dup_of);

        records.push(AccountingRecord {
            path: path.clone(),
            unit_class,
            owner,
            justification_ref,
            reachable_from,
            ttl,
            tracked: true,
            verdict,
            destination,
            disposition,
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
    // The committed move plans, carried as DATA so the conformance gate stays a pure evaluator
    // (it must never glob the tree itself). Sorted, so the face is byte-stable.
    root.insert(
        "planned_move_paths".into(),
        Value::Array(
            inputs
                .planned_move_paths
                .iter()
                .map(|p| Value::String(p.clone()))
                .collect(),
        ),
    );
    root.insert("rows".into(), rows);
    Ok(Value::Object(root))
}

/// Recursively rebuild a `Value` with every object's keys in sorted (BTreeMap) order, so the
/// serialized form is canonical INDEPENDENT of how serde_json was built. serde_json's `Value`
/// map is a `BTreeMap` (sorted) by default but an insertion-ordered `IndexMap` when the
/// `preserve_order` feature is enabled. Under buck2, reindeer unions features across the whole
/// workspace, so the single generated `third-party//:serde_json` has `preserve_order` ON (pulled
/// in by a few unrelated crates) even though the producer's own cargo closure does not — which
/// would make the faces serialize in insertion order under buck2 and sorted order under cargo,
/// breaking the committed==regenerated byte-parity invariant. Canonicalizing here makes the
/// on-disk faces sorted-by-construction under BOTH build systems (hermetic, feature-independent).
fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted: std::collections::BTreeMap<String, Value> =
                std::collections::BTreeMap::new();
            for (key, val) in map {
                sorted.insert(key.clone(), canonicalize_value(val));
            }
            let mut out = Map::new();
            for (key, val) in sorted {
                out.insert(key, val);
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonicalize_value).collect()),
        other => other.clone(),
    }
}

/// A stable, dependency-free FNV-1a 64-bit digest of the canonical row JSON.
/// Used as `_provenance.source_inputs_digest` so the face proves regeneration without
/// a wall-clock (which would break committed==regenerated). Canonicalized so the digest is
/// independent of the serde_json map-ordering feature (see `canonicalize_value`).
fn digest_rows(rows: &Value) -> String {
    let canonical = serde_json::to_string(&canonicalize_value(rows)).unwrap_or_default();
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
    /// Decision files whose front-matter id disagrees with their filename number
    /// (`<file>:<filename-id>!=<front-matter-id>`). A mismatched front-matter id silently
    /// re-keys the file in the dup map, so this is the mask vector for
    /// `dual_decision_collision` (FRIC-1781320000) and a violation in its own right.
    pub id_mismatches: Vec<String>,
    /// `ADR-NNNN` citation edges from governed surfaces (decision bodies, the
    /// roadmap/sequencing artifact, the masterplan `bound_adrs`) that resolve to NO
    /// on-disk decision id (`<cited-id>@<source-path>`), excluding the grandfathered
    /// historical inventory (reviewed shrink-only DATA in the binary; each grandfathered
    /// id is ledgered with its citation sites — FRIC-1781430000). The GATE-1
    /// `phantom_decision_citation` lane is frozen-empty over these: the phantom-0397
    /// exhibit was healed by MINTING the record at the cited number, so any entry here
    /// is NEW debt and born-blocking.
    pub phantom_citations: Vec<String>,
    /// The grandfathered historical phantom inventory itself (the binary's reviewed
    /// shrink-only carve-out DATA), emitted into the face as `grandfathered_phantom_ids`
    /// so the carve-out is AUDIT-VISIBLE and mechanically guarded: the live gate test
    /// asserts anti-padding (every listed id must still resolve to NO decision file — a
    /// healed id must leave the list) and a decrease-only size ceiling (anti-growth:
    /// laundering a NEW phantom by same-PR list addition forces a loud ceiling edit).
    pub grandfathered_phantom_ids: Vec<String>,
    /// The next unallocated decision number derived from the whole tree
    /// (max over filename AND front-matter ids, plus one). The allocator output:
    /// lanes allocate by reading this (or `--next-adr`), never by convention.
    pub next_free_id: String,
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

    let mut id_mismatches = inputs.id_mismatches.clone();
    id_mismatches.sort();
    id_mismatches.dedup();

    let mut phantom_citations = inputs.phantom_citations.clone();
    phantom_citations.sort();
    phantom_citations.dedup();

    let mut grandfathered_phantom_ids = inputs.grandfathered_phantom_ids.clone();
    grandfathered_phantom_ids.sort();
    grandfathered_phantom_ids.dedup();

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
    root.insert(
        "duplicate_ids".into(),
        Value::Array(duplicate_ids.into_iter().map(Value::String).collect()),
    );
    root.insert(
        "id_mismatches".into(),
        Value::Array(id_mismatches.into_iter().map(Value::String).collect()),
    );
    root.insert(
        "phantom_citations".into(),
        Value::Array(phantom_citations.into_iter().map(Value::String).collect()),
    );
    root.insert(
        "grandfathered_phantom_ids".into(),
        Value::Array(
            grandfathered_phantom_ids
                .into_iter()
                .map(Value::String)
                .collect(),
        ),
    );
    root.insert(
        "next_free_id".into(),
        Value::String(inputs.next_free_id.clone()),
    );
    root.insert("generated_face_axes".into(), Value::Object(axes));
    root.insert("decisions".into(), decisions_value);
    Ok(Value::Object(root))
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_empty(value: &str) -> bool {
    value.is_empty()
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
    /// Whether merge admission requires a distinct pre-merge review authority.
    #[serde(skip_serializing_if = "is_false")]
    pub requires_pre_merge_review_authority: bool,
    /// Whether the review-authority evidence is from live merge-admission state, not target-only
    /// shadow config or aspirational docs.
    #[serde(skip_serializing_if = "is_false")]
    pub review_authority_live: bool,
    /// Provenance of the review-authority evidence source.
    #[serde(skip_serializing_if = "is_empty")]
    pub review_authority_source: String,
    /// Whether durable review evidence is present in the admission packet.
    #[serde(skip_serializing_if = "is_false")]
    pub has_durable_review_evidence: bool,
    /// Whether a machine-verifiable review status is a required merge context.
    #[serde(skip_serializing_if = "is_false")]
    pub has_machine_verifiable_review_status: bool,
    /// Whether the admission packet binds the forge PR number.
    #[serde(skip_serializing_if = "is_false")]
    pub binds_pr_number: bool,
    /// Whether the admission packet binds the exact candidate head SHA.
    #[serde(skip_serializing_if = "is_false")]
    pub binds_head_sha: bool,
    /// Whether the admission packet binds the forge-reported PR author identity.
    #[serde(skip_serializing_if = "is_false")]
    pub binds_author_identity: bool,
    /// Whether the admission packet binds the forge-reported reviewer identity.
    #[serde(skip_serializing_if = "is_false")]
    pub binds_reviewer_identity: bool,
    /// Whether the admission packet binds the forge review verdict.
    #[serde(skip_serializing_if = "is_false")]
    pub binds_review_verdict: bool,
    /// Whether the review authority blocks merge admission.
    #[serde(skip_serializing_if = "is_false")]
    pub review_blocks_merge: bool,
    /// Whether the review authority proves reviewer identity is distinct from author.
    #[serde(skip_serializing_if = "is_false")]
    pub reviewer_identity_distinct_from_author: bool,
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
    let rows_value =
        serde_json::to_value(&rows).map_err(|e| ProducerError::Serialize(format!("rows: {e}")))?;

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

/// The buck2 target that runs the firewall ratchet — recorded in the baseline `_provenance`.
pub const FIREWALL_TARGET: &str = "//ci/facade/baseline-ratchet:ci-baseline-ratchet-gate";

// The hardcoded `GATE_IDS: [&str; 7]` array and the `include_str!`-embedded
// `GATE_DISPOSITION_JSON` const were RETIRED in the config-driven floor (Stage 3): the enabled
// gate set + each gate's input KIND + the per-(gate,code) disposition table now come from
// `OyaCiConfig` (`cfg.gates.enabled` + `cfg.gates.disposition_json()`), so adding/removing a
// gate is a `oya-ci.toml` DATA edit, not a producer code change. `build_gate_baseline` +
// `current_keys_per_gate` dispatch on the config-declared `input_kind` (§3.5).

/// The live producer-face inputs the baseline is captured over. Each is the exact
/// `Value` shape that the matching gate's `evaluate_keyed` consumes:
/// - `total_accounting`: the accounting registry (`rows` with path/owner/justification/…)
/// - `cross_artifact`: the decision crosswalk (`decisions`/`duplicate_ids`/`generated_face_axes`)
/// - `automation_ratchet`: the automation matrix (`rows`) joined with the enforcement face
/// - `slo_coverage`: the catalog SLO face (`rows` with crate_id/slo)
/// - `license_policy`: workspace package-license rows (`package_name`/`manifest_path`/`license`)
/// - `enforcement_liveness`: tracked hook/wiring rows for the FRIC-012 liveness gate
pub struct GateInputs<'a> {
    pub total_accounting: &'a Value,
    pub cross_artifact: &'a Value,
    pub automation_ratchet: &'a Value,
    /// The §2.5#4 BNF layer-suffix gate input: `{"rows":[{"crate_name": "oya-..."}]}` —
    /// the first-party `oya-*` crate names the binary enumerates from the tracked Cargo.toml
    /// manifests. The gate's `evaluate_keyed` resolves the role carve-out-aware and reuses
    /// `oya_governance_predictable_naming_kernel::check`. Empty in unit tests.
    pub bnf_layer_suffix: &'a Value,
    /// The §2.5#7 manifest-hygiene gate input: `{"rows":[{"crate_name", "has_version_workspace",
    /// "has_publish_false", "has_license", "has_rust_version_workspace", "has_lints_workspace",
    /// "has_lib", "has_lib_doctest_false"}]}` — per-crate manifest flags the binary parses from
    /// each first-party `oya-*` Cargo.toml. The gate's `evaluate_keyed` is a pure flag→Finding
    /// policy. Empty in unit tests.
    pub manifest_hygiene: &'a Value,
    /// The ADR-0017 cargo-prefix gate input: `{"rows":[{"member_path", "package_name"}]}` — the
    /// first-party `oya-*` workspace members the binary enumerates from the tracked Cargo.toml
    /// manifests (member-path = the dir holding the manifest; package_name = its `[package].name`).
    /// The gate's `evaluate_keyed` reuses `intelligence_cargo_prefix_domain::validate_cargo_prefix`
    /// per crate (surface-all). Empty in unit tests.
    pub cargo_prefix: &'a Value,
    /// The SLO coverage gate input: `{"rows":[{"crate_id", "slo"}]}`. The producer expands the
    /// config-declared `[slo_coverage].catalog_record_globs` against tracked paths, derives the
    /// catalog identity from each file stem, and parses the top-level `slo:` value. The gate's
    /// `evaluate_keyed` reuses `oya_check_slo_coverage::validate_slo_coverage` per row.
    pub slo_coverage: &'a Value,
    /// The license-policy gate input: `{"rows":[{"package_name","manifest_path","license"}]}`.
    /// The producer resolves workspace members via `oya-workspace-members-kernel`, reads each
    /// member manifest, and the gate reuses `oya_check_license_policy::LicensePolicy` per row.
    pub license_policy: &'a Value,
    /// The catalog-liveness gate input: `{"rows":[{"crate_id", "source_path", "is_live",
    /// "marker"}]}`. The producer expands the config-declared `[catalog_liveness]
    /// .catalog_record_globs` against tracked paths, derives the catalog identity from each file
    /// stem, resolves whether that identity is a LIVE workspace crate-id IN-PROCESS (via
    /// `oya-workspace-members-kernel` + each member's `[package].name` — NO shell-out), and parses
    /// the explicit non-live marker (`status:` value / `non_claims` no-crate). The gate's
    /// `evaluate_keyed` is pure live-OR-marked policy.
    pub catalog_liveness: &'a Value,
    /// The ADR-0538 workspace-glob-coverage gate input:
    /// `{"rows":[{"member_entry","is_glob"},{"member_match","has_manifest"},
    /// {"crate_dir","covered","excluded"}]}`. The producer reads the root workspace entries and
    /// scans concrete matches via `oya-workspace-members-kernel`; the gate's `evaluate_keyed` is
    /// pure boolean policy.
    pub workspace_glob_coverage: &'a Value,
    /// The ADR-0540 target-parity gate input:
    /// `{"rows":[{"member_path","has_buck","has_rust_test_target","has_test_code"}]}`. The
    /// producer resolves workspace members via `oya-workspace-members-kernel` and inspects the
    /// declared tracked files; the gate is pure policy over those booleans.
    pub target_parity: &'a Value,
    /// The FRIC-012 enforcement-liveness gate input:
    /// `{"rows":[{"row_type":"hook","hook_path","wired_in_claude","wired_in_codex",
    /// "stub_marked"},{"row_type":"command_reference","wiring_file","command_path",
    /// "target_exists"}]}`. The producer enumerates tracked project hooks and hook-command
    /// references from `.claude/settings.json` + `.codex/hooks.json`.
    pub enforcement_liveness: &'a Value,
    /// The forbidden-vocab shrink-only ratchet's pre-grouped `code -> keys` (the live residue
    /// files per stem), captured by the binary via `oya_check_brand_residue::forbidden_vocab`
    /// over the live corpus. Unlike the four face gates this is computed from the raw tracked
    /// files (not a generated face), so it is supplied already grouped rather than re-derived
    /// here. Empty in unit tests that do not exercise the brand gate.
    pub brand_residue: &'a BTreeMap<String, BTreeSet<String>>,
}

/// Resolve a `producer-face` gate's CURRENT keys: run the bound gate's pure `evaluate_keyed`
/// over the matching `GateInputs` face and group `(code, key)` (§3.5 KIND 1). The
/// face↔evaluator binding is the single per-gate coupling that cannot be data-driven in Rust
/// (no reflection); everything else (which gates, their dispositions, their KIND) is config.
fn producer_face_keys(
    face: oya_ci_config_kernel::GateFace,
    inputs: &GateInputs<'_>,
    naming: &oya_ci_config_kernel::NamingConfig,
) -> BTreeMap<String, BTreeSet<String>> {
    use oya_ci_config_kernel::GateFace;
    match face {
        GateFace::TotalAccounting => group_findings(
            ci_artifact_accountability::evaluate_keyed(inputs.total_accounting)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
        GateFace::CrossArtifact => group_findings(
            ci_cross_artifact_agreement::evaluate_keyed(inputs.cross_artifact)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
        GateFace::AutomationRatchet => group_findings(
            ci_automation_coverage::evaluate_keyed(inputs.automation_ratchet)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
        // Staleness keys are deliberately EMPTY in the committed baseline (ADR-0552,
        // FRIC-1781234047): they derive from history-volatile aging data (last-touch
        // timestamps), so freezing them in a committed face would re-create the
        // squash-merge un-settle defect. The staleness gate ages registry rows from the
        // untracked scm-volatile-facts snapshot at evaluation time; its blocking authority
        // is its own gate lane, not the firewall ratchet. The disposition rows (modes)
        // remain declared so a future flip is still a reviewed DATA edit.
        GateFace::Staleness => BTreeMap::new(),
        // ADR-0533 items 1/2: route the PROFILE-RESOLVED `[naming]` config (oyatie default ==
        // today's consts, byte-identical; neutral == empty prefix, de-branded).
        GateFace::BnfLayerSuffix => group_findings(
            ci_crate_layer_suffix::evaluate_keyed_with(inputs.bnf_layer_suffix, naming)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
        GateFace::ManifestHygiene => group_findings(
            ci_package_manifest_hygiene::evaluate_keyed(inputs.manifest_hygiene)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
        // ADR-0533 item 3: route the PROFILE-RESOLVED `[naming].required_prefix` (oyatie default
        // == `oya-`, byte-identical; neutral == empty prefix, no cargo_prefix_violation).
        GateFace::CargoPrefix => group_findings(
            ci_crate_name_prefix::evaluate_keyed_with(inputs.cargo_prefix, naming)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
        GateFace::SloCoverage => group_findings(
            ci_slo_coverage::evaluate_keyed(inputs.slo_coverage)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
        GateFace::LicensePolicy => group_findings(
            ci_license_policy::evaluate_keyed(inputs.license_policy)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
        GateFace::CatalogLiveness => group_findings(
            ci_service_catalog_parity::evaluate_keyed(inputs.catalog_liveness)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
        GateFace::WorkspaceGlobCoverage => group_findings(
            ci_workspace_member_coverage::evaluate_keyed(inputs.workspace_glob_coverage)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
        GateFace::TargetParity => group_findings(
            ci_build_target_parity::evaluate_keyed(inputs.target_parity)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
        GateFace::EnforcementLiveness => group_findings(
            ci_hook_wiring::evaluate_keyed(inputs.enforcement_liveness)
                .into_iter()
                .map(|f| (f.code, f.key)),
        ),
    }
}

/// Capture each enabled gate's CURRENT `code -> keys` by DISPATCHING on its declared
/// `input_kind` (OYA-CI-CONFORMANCE-FLOOR-PLAN §3.5 — the gate INPUT-BINDING abstraction; the
/// one engine touch-point of this floor). Three KINDs:
/// - `ProducerFace`  → run the bound gate's pure `evaluate_keyed` over its `GateInputs` face;
/// - `RawCorpusCollector` → the keys arrive ALREADY GROUPED from the binary's raw-corpus
///   census (brand-residue) — folded in verbatim (NOT a face, NOT `evaluate_keyed`);
/// - `FrozenEmptyMeta` → contributes NO current keys (its codes are stamped-empty by the
///   disposition join in `build_gate_baseline`).
///
/// Returns `gate_id -> code -> sorted+deduped keys`; BTreeMaps/BTreeSets keep it deterministic
/// so committed==regenerated holds byte-for-byte. (Iteration order over `cfg.gates.enabled` is
/// irrelevant to the on-disk bytes: the baseline `gates` object is BTreeMap-sorted on
/// serialization — but the disposition join in `build_gate_baseline` still walks this map.)
fn current_keys_per_gate(
    cfg: &oya_ci_config_kernel::OyaCiConfig,
    inputs: &GateInputs<'_>,
) -> BTreeMap<String, BTreeMap<String, BTreeSet<String>>> {
    use oya_ci_config_kernel::GateInputKind;
    let mut out: BTreeMap<String, BTreeMap<String, BTreeSet<String>>> = BTreeMap::new();
    for gate in &cfg.gates.enabled {
        let keys = match gate.input_kind {
            GateInputKind::ProducerFace => match gate.face {
                Some(face) => producer_face_keys(face, inputs, &cfg.naming),
                // A producer-face gate with no bound face contributes nothing (mis-config-safe).
                None => BTreeMap::new(),
            },
            GateInputKind::RawCorpusCollector => inputs.brand_residue.clone(),
            GateInputKind::FrozenEmptyMeta => BTreeMap::new(),
        };
        out.insert(gate.id.clone(), keys);
    }
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
pub fn build_gate_baseline(
    cfg: &oya_ci_config_kernel::OyaCiConfig,
    inputs: &GateInputs<'_>,
    config_digest: &str,
) -> Result<Value, ProducerError> {
    let disposition: Value = serde_json::from_str(cfg.gates.disposition_json())
        .map_err(|e| ProducerError::Policy(format!("gate-disposition.json: {e}")))?;
    let disp_gates = disposition
        .get("gates")
        .and_then(Value::as_object)
        .ok_or_else(|| ProducerError::Policy("gate-disposition.json missing 'gates'".into()))?;

    let current = current_keys_per_gate(cfg, inputs);

    // Canonical-key digest input: collect every "<gate>\x1f<code>\x1f<key>" line, sorted.
    let mut digest_lines: BTreeSet<String> = BTreeSet::new();

    // Iterate the CONFIG-DECLARED enabled gates (replacing the hardcoded GATE_IDS, §3.5):
    // each gate's codes + dispositions come from the disposition table, its CURRENT keys from
    // the KIND-dispatched `current` map. The on-disk gate order is BTreeMap-sorted regardless.
    // ADR-0533 "gates present-but-quiet": under the NEUTRAL profile a gate may be ENABLED (so the
    // engine still dispatches its KIND) while the disposition table is empty — that gate then
    // contributes NO codes (quiet). Under the OYATIE profile a gate enabled but absent from the
    // disposition table stays a HARD error (a real misconfig must fail loud — the safety property
    // keeps first-party behaviour identical).
    let quiet_missing_disposition = cfg.profile == oya_ci_config_kernel::Profile::Neutral;
    let empty_disp = Map::new();
    let mut gates_obj = Map::new();
    for spec in &cfg.gates.enabled {
        let gate = spec.id.as_str();
        let disp_codes = match disp_gates.get(gate).and_then(Value::as_object) {
            Some(codes) => codes,
            None if quiet_missing_disposition => &empty_disp,
            None => {
                return Err(ProducerError::Policy(format!(
                    "disposition missing gate {gate}"
                )));
            }
        };
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
            // The exact registration edit (or the precise design decision needed) the
            // firewall prints when this code FAILs — DATA from the disposition table
            // (ADR-0555: an unaccounted artifact is never a bare flag).
            let remediation = disp.get("remediation").and_then(Value::as_str);

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
            if let Some(text) = remediation {
                entry.insert("remediation".into(), Value::String(text.to_owned()));
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
             (registry-drift byte-diffs it); a hand-edit to launder debt is itself ci_inventory_registry_drift RED."
                .into(),
        ),
    );
    root.insert(
        "_provenance".into(),
        serde_json::json!({
            "producer_target": PRODUCER_TARGET,
            "firewall_target": FIREWALL_TARGET,
            "baseline_schema_version": 1,
            "config_digest": config_digest,
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

/// Serialize a face to the canonical on-disk form: 2-space pretty + trailing newline, with
/// every object's keys in sorted order. Identical formatting on every run keeps
/// committed==regenerated byte-exact. The explicit key-sort (via `canonicalize_value`) makes the
/// on-disk bytes independent of the serde_json `preserve_order` feature, which reindeer unions ON
/// under buck2 — so cargo and buck2 emit byte-identical faces (hermetic byte-parity).
pub fn to_canonical_json(value: &Value) -> Result<String, ProducerError> {
    let mut text = serde_json::to_string_pretty(&canonicalize_value(value))
        .map_err(|e| ProducerError::Serialize(e.to_string()))?;
    text.push('\n');
    Ok(text)
}

// ===========================================================================
// Registration BRIDGES (slice 2.5: extracted from the binary as a LIBRARY API)
//
// These three functions were the producer's TRANSITIONAL registration bridges
// (ADR-0555; cli_surface_policy: local bridge only, never merge authority — their
// successors are the ADR-0548 D3 reconcilers). They were private in the binary, reachable
// only via the retirement-marked `--fix-owners` / `--fix-reachability` / `--next-adr` CLI
// flags. The future `register_crate` app (slice 3) calls them as LIBRARY functions (no
// subprocess), so they live here as PUBLIC API. The logic is preserved EXACTLY; the only
// change is the error type (`CliError` → [`ProducerError`]) and the bridges read the
// tracked-paths universe as a `&[String]` slice (the binary's `ScmFacts` is binary-only).
// ===========================================================================

/// Read a file to a String, or the empty string on any error (the producer's tolerant
/// corpus read — a missing ADR/registry sibling is the zero-config default, not a hard
/// error). Mirrors the binary's `read_text` so the moved logic is byte-for-byte identical.
fn read_text(path: &std::path::Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

/// The lines inside the leading `---` front-matter block.
fn front_matter_lines(body: &str) -> Vec<&str> {
    let mut lines = body.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Vec::new();
    }
    let mut out = Vec::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        out.push(line.trim_start());
    }
    out
}

/// Read the value of a top-level YAML-ish front-matter scalar field (`key: value`).
pub fn front_matter_field(body: &str, key: &str) -> Option<String> {
    for line in front_matter_lines(body) {
        if let Some(rest) = line.strip_prefix(&format!("{key}:")) {
            let value = rest.trim().trim_matches('"').trim_matches('\'').trim();
            if !value.is_empty() {
                return Some(value.to_owned());
            }
        }
    }
    None
}

/// The `ADR-NNNN` decision id encoded by a decision FILENAME (`ADR-0001-title.md` ⇒
/// `ADR-0001`). Names in any other shape contribute nothing.
pub fn adr_id_from_filename(name: &str) -> Option<String> {
    let rest = name.strip_prefix("ADR-")?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.len() == 4 {
        Some(format!("ADR-{digits}"))
    } else {
        None
    }
}

/// The numeric component of an `ADR-NNNN` decision id (allocator input). Ids in any
/// other shape contribute nothing to the next-free-number derivation.
pub fn adr_number(id: &str) -> Option<u32> {
    let rest = id.strip_prefix("ADR-")?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.len() == 4 && digits.len() == rest.len() {
        digits.parse().ok()
    } else {
        None
    }
}

/// The next unallocated decision id, derived from the decisions directory (`adr_dir`).
///
/// STANDALONE allocator (slice 2.5): the highest decision number seen across BOTH the
/// filename AND the front-matter `id:` of every decision file, plus one, formatted
/// `ADR-{:04}`. This is the SINGLE allocator — the binary's `collect_crosswalk_inputs`
/// calls it for `next_free_id`, and the `--next-adr` flag + `register_crate` (slice 3) call
/// it directly. Read-only: it never writes. A missing/unreadable directory yields `ADR-0001`
/// (max 0 + 1), the zero-config default. Lanes allocate by running this, never by convention.
pub fn allocate_next_adr_id(adr_dir: &std::path::Path) -> Result<String, ProducerError> {
    let mut max_decision_number: u32 = 0;
    if let Ok(entries) = std::fs::read_dir(adr_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(filename_id) = adr_id_from_filename(&name) {
                if let Some(number) = adr_number(&filename_id) {
                    max_decision_number = max_decision_number.max(number);
                }
                let body = read_text(&entry.path());
                if let Some(front_id) = front_matter_field(&body, "id")
                    && let Some(number) = adr_number(&front_id)
                {
                    max_decision_number = max_decision_number.max(number);
                }
            }
        }
    }
    Ok(format!("ADR-{:04}", max_decision_number + 1))
}

/// An owner principal (one OWNERS line): a lowercase DNS-1123-label-shaped team
/// identifier — `[a-z0-9]` plus interior `-`, 1..=63 chars (the K8s name shape; matches
/// every live principal: `cloud-ci-platform`, `council-architecture`,
/// `axis-cloud-platform`).
pub fn is_valid_owner_principal(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes.len() > 63 {
        return false;
    }
    let alnum = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    alnum(bytes[0]) && alnum(bytes[bytes.len() - 1]) && bytes.iter().all(|&b| alnum(b) || b == b'-')
}

/// Parse OWNERS content against the minimal codified schema (ADR-0555 hardening,
/// FRIC-1781400000 — codifies what the live corpus already does): each line, after
/// trimming, is empty (ignored), a `#` comment (ignored), or an owner principal. A VALID
/// file carries at least one principal and zero unparseable lines. Anything else —
/// empty, comment-only, garbage, non-UTF-8 — is NOT ownership (fail-closed).
fn parse_owners_content(text: &str) -> Result<Vec<String>, String> {
    let mut principals = Vec::new();
    for (idx, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !is_valid_owner_principal(line) {
            return Err(format!(
                "line {}: {line:?} is not a valid owner principal (schema: one owner \
                 principal per line — lowercase alphanumeric + interior hyphens, 1..=63 \
                 chars, e.g. `cloud-ci-platform`; `#` comments and blank lines allowed)",
                idx + 1
            ));
        }
        principals.push(line.to_owned());
    }
    if principals.is_empty() {
        return Err(
            "zero owner principals (an empty or comment-only OWNERS file is NOT \
             ownership — name at least one owning team, one principal per line)"
                .to_owned(),
        );
    }
    Ok(principals)
}

/// OWNERS-resolution integrity diagnostics (ADR-0555 hardening, FRIC-1781400000).
/// These never grant or carry ownership themselves — they name the exact fix for each
/// OWNERS file that failed the content schema or the breadth bound, so a FAIL is never
/// a bare flag (founder directive: flagging/red-gating isn't enough).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OwnersIntegrity {
    /// OWNERS file (repo-relative) -> the schema defect. Fail-closed: an invalid file is
    /// NOT an ownership marker AND still poisons resolution at its directory (no
    /// fall-through to a broader ancestor) — invalid content can never yield owned rows.
    pub invalid: BTreeMap<String, String>,
    /// OWNERS file (repo-relative) -> raw nearest-ancestor coverage, for files whose
    /// coverage exceeds `[owners] max_paths_per_owners_file`. The first <bound> covered
    /// paths (path-sorted) keep ownership; the excess stays UNOWNED.
    pub over_broad: BTreeMap<String, usize>,
}

/// The outcome of OWNERS resolution: the per-path owner map plus integrity diagnostics.
pub struct OwnersResolution {
    pub by_path: BTreeMap<String, String>,
    pub integrity: OwnersIntegrity,
    /// The OWNERS files (repo-relative) that PARSED against the schema — exactly the
    /// complement of `integrity.invalid` over the tracked OWNERS files. This is the one
    /// place in the producer that reads and parses OWNERS content, so it is the only place
    /// the per-file validity verdict exists; it feeds `RepoInputs::valid_owners_files` and
    /// hence the `build_registry` accounting floor. NOT the same as "appears in `by_path`":
    /// the per-file breadth bound (`max_paths_per_owners_file`) can truncate a valid file's
    /// own row out of the coverage set, and truncation is a BREADTH verdict, not a schema
    /// verdict.
    pub valid_files: BTreeSet<String>,
}

fn nearest_ancestor(path: &str, dirs: &BTreeSet<String>) -> Option<String> {
    let mut cursor = path;
    while let Some((parent, _)) = cursor.rsplit_once('/') {
        if dirs.contains(parent) {
            return Some(parent.to_owned());
        }
        cursor = parent;
    }
    if dirs.contains("") {
        return Some(String::new());
    }
    None
}

fn is_top_level_buck_metadata(path: &str) -> bool {
    let mut components = path.split('/');
    components
        .next()
        .is_some_and(|directory| !directory.is_empty())
        && components.next() == Some("BUCK")
        && components.next().is_none()
}

/// Resolve the nearest up-tree `OWNERS` file for each path. Ownership requires BOTH
/// existence and valid content (ADR-0555 hardening, FRIC-1781400000): the file must
/// parse to >=1 owner principal under `parse_owners_content`, and a single file's
/// coverage is capped by `[owners] max_paths_per_owners_file` (excess stays unowned).
/// With zero valid OWNERS files this returns an empty map (every row ⇒ unowned) — the
/// gap is DATA (no OWNERS rows), not scanner code.
pub fn resolve_owners(
    repo_root: &std::path::Path,
    paths: &[String],
    cfg: &oya_ci_config_kernel::OyaCiConfig,
) -> OwnersResolution {
    let owners_file = cfg.owners.file_name.as_str();
    let bound = usize::try_from(cfg.owners.max_paths_per_owners_file.get()).unwrap_or(usize::MAX);
    // Every tracked OWNERS file is a resolution BOUNDARY (dir -> the file's repo-relative
    // path); only the ones with schema-valid content GRANT ownership. An invalid file
    // poisons its directory rather than falling through to a broader ancestor — fail-
    // closed, so invalid content can never yield owned rows.
    let mut owners_paths: BTreeMap<String, String> = BTreeMap::new();
    for p in paths {
        if p.as_str() == owners_file {
            owners_paths.insert(String::new(), p.clone());
        } else if p.ends_with(&format!("/{owners_file}"))
            && let Some((dir, _)) = p.rsplit_once('/')
        {
            owners_paths.insert(dir.to_owned(), p.clone());
        }
    }

    let mut integrity = OwnersIntegrity::default();
    let mut valid_dirs: BTreeSet<String> = BTreeSet::new();
    let mut valid_files: BTreeSet<String> = BTreeSet::new();
    for (dir, rel) in &owners_paths {
        let defect = match std::fs::read(repo_root.join(rel)) {
            Err(e) => Some(format!("unreadable: {e}")),
            Ok(bytes) => match String::from_utf8(bytes) {
                Err(_) => Some("not UTF-8 text".to_owned()),
                Ok(text) => parse_owners_content(&text).err(),
            },
        };
        match defect {
            Some(defect) => {
                integrity.invalid.insert(rel.clone(), defect);
            }
            None => {
                valid_dirs.insert(dir.clone());
                valid_files.insert(rel.clone());
            }
        }
    }

    let mut by_path = BTreeMap::new();
    if owners_paths.is_empty() {
        return OwnersResolution {
            by_path,
            integrity,
            valid_files,
        };
    }

    let all_dirs: BTreeSet<String> = owners_paths.keys().cloned().collect();
    let mut covered: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in paths {
        if let Some(owner_dir) = nearest_ancestor(path, &all_dirs)
            && valid_dirs.contains(&owner_dir)
        {
            // A root-level OWNERS file is the narrow registration for root DATA files and
            // direct top-level BUCK package aggregators. Those BUCK files shape the
            // repository-wide hermetic build graph and belong to the root build owner; the
            // rule is structural rather than a path allowlist. Letting root OWNERS claim any
            // other otherwise-unowned subtree would bulk-neuter accounting and immediately
            // trip the breadth bound on large repos. Subtrees keep their own nearest-ancestor
            // OWNERS registrations.
            if owner_dir.is_empty() && path.contains('/') && !is_top_level_buck_metadata(path) {
                continue;
            }
            covered.entry(owner_dir).or_default().push(path.clone());
        }
    }

    for (dir, mut dir_paths) in covered {
        // Deterministic breadth accounting: path-sorted, so the SAME paths keep
        // ownership on every regeneration (committed==regenerated holds).
        dir_paths.sort();
        if dir_paths.len() > bound {
            integrity
                .over_broad
                .insert(owners_paths[&dir].clone(), dir_paths.len());
            dir_paths.truncate(bound);
        }
        for p in dir_paths {
            by_path.insert(p, format!("OWNERS:{dir}"));
        }
    }

    OwnersResolution {
        by_path,
        integrity,
        valid_files,
    }
}

/// One reviewed reachability registration (ADR-0555): a dir prefix (MUST end with `/`) or
/// an exact path, plus the non-empty `anchor` naming WHY the tree is reached. Registration
/// is a review-visible design act recorded as DATA — the ADR-0551 trust class (same as
/// ratchet-policy.json / gate-baseline.signoff.json) — never a silent exemption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachabilityRegistration {
    pub prefix: String,
    pub anchor: String,
}

/// Load + validate `specs/reachability-registry.json`. Fail-loud: a malformed file or an
/// invalid entry (empty prefix, empty anchor) is a hard error naming the defect — never a
/// silent empty registry. A MISSING file is the declared zero-config default (empty).
pub fn load_reachability_registry(
    path: &std::path::Path,
) -> Result<Vec<ReachabilityRegistration>, ProducerError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(ProducerError::Io(format!("{}: {e}", path.display()))),
    };
    let value: Value = serde_json::from_str(&text)
        .map_err(|e| ProducerError::Validation(format!("{}: parse: {e}", path.display())))?;
    let entries = value
        .get("registered")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            ProducerError::Validation(format!(
                "{}: missing 'registered' array (fail-loud: the reachability registry must \
                 declare its entries explicitly)",
                path.display()
            ))
        })?;
    let mut out: Vec<ReachabilityRegistration> = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let prefix = entry
            .get("prefix")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let anchor = entry
            .get("anchor")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if prefix.is_empty() || anchor.trim().is_empty() {
            return Err(ProducerError::Validation(format!(
                "{}: registered[{index}] must carry a non-empty prefix AND a non-empty \
                 anchor naming WHY the tree is reached (registration is a reviewed design \
                 act, never a bare exemption)",
                path.display()
            )));
        }
        out.push(ReachabilityRegistration { prefix, anchor });
    }
    Ok(out)
}

/// Whether a tracked path is covered by a registration entry: dir prefixes (ending `/`)
/// cover the subtree; any other entry is an EXACT path match. The trailing-`/` requirement
/// prevents `docs/dec` from silently covering `docs/decisions-evil/...`.
pub fn registration_matches(path: &str, prefix: &str) -> bool {
    if prefix.ends_with('/') {
        path.starts_with(prefix)
    } else {
        path == prefix
    }
}

/// Envelope policy SSOT (cite; do not re-list roots in this crate).
/// Authority: `specs/integ-branch-envelopes.json#path_ownership` + `#roots.*.envelope_globs`.
pub const ENVELOPES_RELPATH: &str = "specs/integ-branch-envelopes.json";

/// Reachability source tag when a path is covered by an envelope `envelope_globs` prefix.
/// Forever admission.policy consumes these so in-domain adds need no per-file tip-free row.
pub const ENVELOPE_PREFIX_OWNERSHIP_SOURCE: &str = "envelope-prefix-ownership";

/// Convert one envelope glob (`compute/**`) into a reachability prefix (`compute/`).
///
/// Only the live envelope shape `dir/**` is accepted (measured: all 74 roots use it).
/// Other glob shapes return `None` (fail-closed — never invent a silent broad allow).
pub fn envelope_glob_to_prefix(glob: &str) -> Option<String> {
    let glob = glob.trim();
    if let Some(stem) = glob.strip_suffix("/**") {
        if stem.is_empty() || stem.contains('*') || stem.starts_with('/') || stem.contains("..") {
            return None;
        }
        return Some(format!("{stem}/"));
    }
    None
}

/// Load prefix allows from `roots.*.envelope_globs` (path ownership law).
///
/// Missing file ⇒ empty (zero-config fixtures). Present but missing/non-object `roots`
/// ⇒ hard error (fail-loud). Duplicate prefixes collapse.
pub fn load_envelope_prefix_allows(
    path: &std::path::Path,
) -> Result<Vec<ReachabilityRegistration>, ProducerError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(ProducerError::Io(format!("{}: {e}", path.display()))),
    };
    let value: Value = serde_json::from_str(&text)
        .map_err(|e| ProducerError::Validation(format!("{}: parse: {e}", path.display())))?;
    let roots = value
        .get("roots")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            ProducerError::Validation(format!(
                "{}: missing object 'roots' (fail-loud: envelope prefix allow requires \
             roots.*.envelope_globs)",
                path.display()
            ))
        })?;

    let mut by_prefix: BTreeMap<String, String> = BTreeMap::new();
    for (root_id, root) in roots {
        let Some(globs) = root.get("envelope_globs").and_then(Value::as_array) else {
            continue;
        };
        let branch = root
            .get("branch")
            .and_then(Value::as_str)
            .unwrap_or(root_id);
        for (index, glob_value) in globs.iter().enumerate() {
            let Some(glob) = glob_value.as_str() else {
                return Err(ProducerError::Validation(format!(
                    "{}: roots.{root_id}.envelope_globs[{index}] must be a string",
                    path.display()
                )));
            };
            let Some(prefix) = envelope_glob_to_prefix(glob) else {
                return Err(ProducerError::Validation(format!(
                    "{}: roots.{root_id}.envelope_globs[{index}]={glob:?} is not a supported \
                     envelope prefix glob (expected 'dir/**')",
                    path.display()
                )));
            };
            by_prefix.entry(prefix).or_insert_with(|| {
                format!(
                    "Envelope prefix ownership ({branch} → {glob}): in-domain path allow from \
                     {ENVELOPES_RELPATH}#roots.{root_id}.envelope_globs (path_ownership law). \
                     Per-file tip-free / reachability-registry rows are NOT required for paths \
                     under this prefix."
                )
            });
        }
    }

    Ok(by_prefix
        .into_iter()
        .map(|(prefix, anchor)| ReachabilityRegistration { prefix, anchor })
        .collect())
}

/// `fix-owners <dir>=<owner>` — the TRANSITIONAL ownership-registration bridge
/// (ADR-0555; cli_surface_policy: local bridge only, never merge authority; successor =
/// the ADR-0548 D3 reconcilers). The OWNER is the human design decision supplied as input;
/// the bridge applies the exact edit (write `<dir>/OWNERS`) and SELF-VALIDATES by
/// re-running the ownership derivation over the tracked universe (`tracked_paths`) plus the
/// new file. Returns the success message; refusal leaves NO residue.
pub fn fix_owners(
    repo_root: &std::path::Path,
    cfg: &oya_ci_config_kernel::OyaCiConfig,
    tracked_paths: &[String],
    spec: &str,
) -> Result<String, ProducerError> {
    let (dir, owner) = spec.split_once('=').ok_or_else(|| {
        ProducerError::Validation(format!(
            "--fix-owners expects <dir>=<owner> (the owner is YOUR design decision — \
             this bridge applies it, it never invents one), got {spec:?}"
        ))
    })?;
    let dir = dir.trim_end_matches('/');
    let owner = owner.trim();
    if dir.is_empty() || owner.is_empty() {
        return Err(ProducerError::Validation(
            "--fix-owners: both <dir> and <owner> must be non-empty".to_owned(),
        ));
    }
    // ADR-0555 hardening (FRIC-1781400000): the bridge must EMIT valid schema content —
    // an OWNERS file it writes that the resolver would reject is a self-defeating
    // registration. Validate the principal before writing anything.
    if !is_valid_owner_principal(owner) {
        return Err(ProducerError::Validation(format!(
            "--fix-owners: {owner:?} is not a valid owner principal (OWNERS schema: \
             lowercase alphanumeric + interior hyphens, 1..=63 chars, e.g. \
             `cloud-ci-platform`)"
        )));
    }
    // <dir> must be a repo-relative path; reject absolute paths and `..` traversal so the
    // local bridge cannot write an OWNERS file outside the repo (defence-in-depth — this is
    // a local feedback bridge, never merge authority).
    if dir.starts_with('/') || dir.split('/').any(|seg| seg == "..") {
        return Err(ProducerError::Validation(format!(
            "--fix-owners: <dir> must be a repo-relative path (no leading '/' or '..'): {dir:?}"
        )));
    }
    if !repo_root.join(dir).is_dir() {
        return Err(ProducerError::Validation(format!(
            "--fix-owners: {dir} is not a directory under the repo root"
        )));
    }
    let owners_file = cfg.owners.file_name.as_str();
    let owners_rel = format!("{dir}/{owners_file}");
    let owners_abs = repo_root.join(&owners_rel);
    if owners_abs.exists() {
        return Err(ProducerError::Refused(format!(
            "--fix-owners: {owners_rel} already exists — extend it by hand (a reviewed \
             edit), this bridge only seeds missing registrations"
        )));
    }
    std::fs::write(&owners_abs, format!("{owner}\n"))
        .map_err(|e| ProducerError::Io(format!("{owners_rel}: {e}")))?;

    // SELF-VALIDATION: re-run the derivation over tracked ∪ {the new OWNERS file} and
    // count the tracked paths that now ownership-resolve to this registration. The
    // derivation is content-aware (ADR-0555 hardening), so this also proves the written
    // file parses to the schema.
    let mut universe = tracked_paths.to_vec();
    if !universe.contains(&owners_rel) {
        universe.push(owners_rel.clone());
    }
    let resolution = resolve_owners(repo_root, &universe, cfg);
    // Breadth bound (FRIC-1781400000): refuse a registration whose coverage exceeds
    // [owners] max_paths_per_owners_file — a single bulk OWNERS must not neuter a
    // tree's unowned accounting. No residue on refusal.
    if let Some(coverage) = resolution.integrity.over_broad.get(&owners_rel) {
        let bound = cfg.owners.max_paths_per_owners_file;
        let _ = std::fs::remove_file(&owners_abs);
        return Err(ProducerError::Refused(format!(
            "--fix-owners: {owners_rel} would cover {coverage} tracked paths, over the \
             [owners] max_paths_per_owners_file bound ({bound}) — a single bulk \
             registration cannot neuter a tree's unowned accounting (ADR-0555); reverted \
             the written {owners_rel}. Exact fix: split the registration — add OWNERS \
             files in child subtrees so no single file covers more than {bound} paths"
        )));
    }
    let owners = resolution.by_path;
    // Count only the PRE-EXISTING tracked paths the registration now covers (the new
    // OWNERS file covering itself is not evidence of coverage).
    let covered = owners
        .iter()
        .filter(|(path, resolved)| {
            *path != &owners_rel
                && path.starts_with(&format!("{dir}/"))
                && *resolved == &format!("OWNERS:{dir}")
        })
        .count();
    if covered == 0 {
        // Self-validating-bridge contract: a registration that does not take leaves NO
        // residue. Remove the file we just wrote before failing (best-effort; the error
        // names the defect regardless).
        let _ = std::fs::remove_file(&owners_abs);
        return Err(ProducerError::Refused(format!(
            "--fix-owners: self-validation FAILED — no tracked path under {dir}/ resolves \
             to OWNERS:{dir} after the edit (the registration did not take; reverted the \
             written {owners_rel})"
        )));
    }
    Ok(format!(
        "fix-owners: wrote {owners_rel} (owner: {owner}); self-validation: {covered} tracked \
         path(s) under {dir}/ now ownership-resolve to OWNERS:{dir}. Next: git add \
         {owners_rel}, then re-run buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root . and \
         settle the regenerated faces (the settle protocol)."
    ))
}

/// `fix-reachability <prefix>=<anchor>` — the TRANSITIONAL reachability-registration
/// bridge (ADR-0555; cli_surface_policy: local bridge only, never merge authority;
/// successor = the ADR-0548 D3 reconcilers). The ANCHOR (why this tree is reached) is the
/// human design decision supplied as input; the bridge appends the reviewed registry entry
/// and SELF-VALIDATES by re-loading the registry fail-loud and re-running the match. The
/// coverage count is taken over `tracked_paths`. Returns the success message.
pub fn fix_reachability(
    repo_root: &std::path::Path,
    cfg: &oya_ci_config_kernel::OyaCiConfig,
    tracked_paths: &[String],
    spec: &str,
) -> Result<String, ProducerError> {
    let (prefix, anchor) = spec.split_once('=').ok_or_else(|| {
        ProducerError::Validation(format!(
            "--fix-reachability expects <prefix>=<anchor> (the anchor names WHY the tree \
             is reached — YOUR design decision; this bridge applies it), got {spec:?}"
        ))
    })?;
    let prefix = prefix.trim();
    let anchor = anchor.trim();
    if prefix.is_empty() || anchor.is_empty() {
        return Err(ProducerError::Validation(
            "--fix-reachability: both <prefix> and <anchor> must be non-empty".to_owned(),
        ));
    }
    // An OWNERS file is accounted BY CONSTRUCTION (see [`OWNERS_SCHEMA_ANCHOR`]), so a row
    // for one is dead weight the next capability move has to carry. Refusing here keeps the
    // paved road and the gate in agreement: without it the bridge would happily write a row
    // that `owners_files_are_never_registered_in_the_reachability_registry` then rejects in
    // CI — a trap that reads as a gate bug rather than as a no-op registration. The remedy
    // for an unaccounted OWNERS file is to make it PARSE, never to register it.
    let owners_file = cfg.owners.file_name.as_str();
    if prefix == owners_file || prefix.ends_with(&format!("/{owners_file}")) {
        return Err(ProducerError::Refused(format!(
            "--fix-reachability: {prefix} is an {owners_file} file, which is accounted by \
             CONSTRUCTION — a schema-valid {owners_file} file is justified + reachable with \
             no registry row, so this registration would be a no-op that every future move \
             has to maintain. If {prefix} is still RED, its CONTENT fails the {owners_file} \
             schema (>=1 owner principal, no unparseable lines): fix the file, not the registry."
        )));
    }
    let registry_rel = cfg.reachability.registry.as_str();
    let registry_abs = repo_root.join(registry_rel);
    let mut entries = load_reachability_registry(&registry_abs)?;
    if entries.iter().any(|entry| entry.prefix == prefix) {
        return Err(ProducerError::Refused(format!(
            "--fix-reachability: {prefix} is already registered in {registry_rel}"
        )));
    }
    entries.push(ReachabilityRegistration {
        prefix: prefix.to_owned(),
        anchor: anchor.to_owned(),
    });
    entries.sort_by(|a, b| a.prefix.cmp(&b.prefix));

    let registered: Vec<Value> = entries
        .iter()
        .map(|entry| serde_json::json!({ "prefix": entry.prefix, "anchor": entry.anchor }))
        .collect();
    let body = serde_json::json!({
        "_comment": "Reviewed reachability registrations (ADR-0555). Each entry registers a tree (dir prefix ending '/') or an exact path as reached, and MUST carry an anchor naming WHY. Registration is a review-visible design act — the ADR-0551 trust class (same as ratchet-policy.json) — never a silent exemption. Hand-edited (or via the transitional --fix-reachability bridge); the producer fails LOUD on a malformed entry.",
        "registered": registered,
    });
    let text = to_canonical_json(&body)?;
    std::fs::write(&registry_abs, text)
        .map_err(|e| ProducerError::Io(format!("{registry_rel}: {e}")))?;

    // SELF-VALIDATION: the written registry must round-trip the fail-loud loader, and the
    // new prefix is reported with its tracked coverage (0 is legal for a not-yet-committed
    // artifact — the registration covers it the moment it is tracked).
    let reloaded = load_reachability_registry(&registry_abs)?;
    if !reloaded.iter().any(|entry| entry.prefix == prefix) {
        return Err(ProducerError::Refused(format!(
            "--fix-reachability: self-validation FAILED — {prefix} did not round-trip \
             {registry_rel} (do not commit)"
        )));
    }
    let covered = tracked_paths
        .iter()
        .filter(|path| registration_matches(path, prefix))
        .count();
    Ok(format!(
        "fix-reachability: registered {prefix} in {registry_rel} (anchor: {anchor}); \
         self-validation: round-trip OK, {covered} tracked path(s) currently covered. \
         Next: git add {registry_rel}, then re-run \
         buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root . and settle the regenerated \
         faces (the settle protocol)."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A placement fixture mirroring the real registry's shape: one capability that absorbs a
    /// legacy dir, and one meta directory.
    fn sample_placement() -> CapabilityPlacement {
        CapabilityPlacement::from_registry_value(&serde_json::json!({
            "capabilities": [
                {"name": "iam", "absorbs_current_dirs": ["iam", "iam/cloud-iam", "oya/identity"]},
                {"name": "data", "absorbs_current_dirs": ["data"]}
            ],
            "meta_directories": [{"dir": "governance/"}, {"dir": "app/"}]
        }))
    }

    #[test]
    fn destination_prefers_the_longest_declared_absorb() {
        let placement = sample_placement();
        // `iam/cloud-iam` and `iam` both match; the longer declaration wins, so a path is never
        // credited to a broader capability than the one that actually declared it.
        assert_eq!(
            derive_destination("iam/cloud-iam/src/lib.rs", &placement, &None).as_deref(),
            Some("iam/")
        );
        assert_eq!(
            derive_destination("oya/identity/src/lib.rs", &placement, &None).as_deref(),
            Some("iam/"),
            "a legacy dir absorbed by a capability resolves to that capability, not to oya/"
        );
    }

    #[test]
    fn destination_falls_back_to_the_owner_root_then_to_none() {
        let placement = sample_placement();
        assert_eq!(
            derive_destination(
                "tools/thing/src/lib.rs",
                &placement,
                &Some("OWNERS:tools".into())
            )
            .as_deref(),
            Some("tools/"),
            "an owned tree has a declared home even before its capability absorbs it"
        );
        assert_eq!(
            derive_destination("nowhere/thing.md", &placement, &None),
            None,
            "no meta dir, no absorb, no owner ⇒ NO destination may be invented"
        );
    }

    #[test]
    fn disposition_is_unclassified_without_a_declared_home() {
        // This is the fail-closed property the whole design rests on: a path the tree cannot
        // place must BLOCK its domain, never default to `retain` (leave it) or `delete`.
        assert_eq!(
            derive_disposition("KEEP", "doc", &None, "nowhere/thing.md", &None),
            "unclassified"
        );
        assert_eq!(
            derive_disposition("RED", "doc", &Some("docs/".into()), "docs/x.md", &None),
            "unclassified",
            "RED means the tree does not know why this exists; no destination can be honest"
        );
    }

    #[test]
    fn disposition_separates_staying_from_moving() {
        assert_eq!(
            derive_disposition("KEEP", "doc", &Some("iam/".into()), "iam/docs/x.md", &None),
            "retain"
        );
        assert_eq!(
            derive_disposition(
                "KEEP",
                "doc",
                &Some("iam/".into()),
                "oya/identity/x.md",
                &None
            ),
            "move"
        );
    }

    #[test]
    fn disposition_terminals_come_from_class_and_duplication() {
        assert_eq!(
            derive_disposition(
                "KEEP",
                "generated",
                &Some("ci/".into()),
                "ci/x.generated.json",
                &None
            ),
            "generate",
            "generated output terminates by regeneration, not by being moved"
        );
        assert_eq!(
            derive_disposition(
                "KEEP",
                "vendor",
                &Some("third-party/".into()),
                "third-party/x.rs",
                &None
            ),
            "externalize"
        );
        assert_eq!(
            derive_disposition("KEEP", "scratch", &Some("x/".into()), "x/tmp.txt", &None),
            "delete"
        );
        assert_eq!(
            derive_disposition(
                "MERGE",
                "doc",
                &Some("iam/".into()),
                "iam/dup.md",
                &Some("iam/canon.md".into())
            ),
            "delete",
            "a duplicate terminates into its canonical twin whatever else is true of it"
        );
    }

    #[test]
    fn an_absent_or_malformed_registry_fails_closed() {
        // The registry cannot be silently ignored into a permissive result: an empty placement
        // must yield NO destinations, hence `unclassified`, hence a blocked domain.
        let empty = CapabilityPlacement::default();
        assert_eq!(derive_destination("iam/src/lib.rs", &empty, &None), None);
        let garbage = CapabilityPlacement::from_registry_value(&serde_json::json!({"nope": 1}));
        assert_eq!(
            garbage, empty,
            "a registry missing its keys yields an empty placement"
        );
    }

    fn sample_inputs() -> RepoInputs {
        let mut owners = BTreeMap::new();
        owners.insert(
            "specs/masterplan.json".into(),
            "council-architecture".into(),
        );
        let mut justifications = BTreeMap::new();
        justifications.insert("specs/masterplan.json".into(), "ADR-0364".into());
        let mut reachability = BTreeMap::new();
        reachability.insert(
            "specs/masterplan.json".into(),
            vec!["root-hub".into(), "masterplan".into()],
        );
        RepoInputs {
            tracked_paths: vec![
                "specs/masterplan.json".into(),
                ".omc/state/run.jsonl".into(),
                "oya/orphan/lib.rs".into(),
            ],
            owners,
            justifications,
            reachability,
            dup_of: BTreeMap::new(),
            valid_owners_files: BTreeSet::new(),
            placement: CapabilityPlacement::default(),
            planned_move_paths: BTreeSet::new(),
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
        assert_eq!(
            policy
                .classify("docs/adr-archive/ADR-0001-cohesion-thesis-one-product-flat-catalog.md"),
            "doc"
        );
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

    /// REACHED ⇒ JUSTIFIED, as a rule over every reachability source — and the safety floor
    /// that makes the rule admissible: a path reached by NOTHING still raises BOTH codes.
    /// This is the whole safety argument for collapsing `unjustified` into `unreachable`.
    #[test]
    fn reached_paths_are_justified_by_the_reaching_source_and_unreached_paths_are_not() {
        let policy = Policy::from_bundled().expect("policy");
        let sources = [
            "masterplan",
            "root-hub",
            "doc-catalog",
            "cargo-members",
            "reachability-registry",
        ];
        let mut inputs = RepoInputs {
            tracked_paths: vec!["oya/unreached/src/lib.rs".into()],
            ..RepoInputs::default()
        };
        for source in sources {
            let path = format!("oya/{source}/src/lib.rs");
            inputs.tracked_paths.push(path.clone());
            inputs.reachability.insert(path, vec![source.to_owned()]);
        }
        let registry = build_registry(&inputs, &policy).expect("registry");
        let rows = registry["rows"].as_array().expect("rows");
        let row = |path: &str| {
            rows.iter()
                .find(|r| r["path"] == path)
                .unwrap_or_else(|| panic!("row for {path}"))
                .clone()
        };

        // Every reaching source justifies, naming WHICH source did it.
        for source in sources {
            let record = row(&format!("oya/{source}/src/lib.rs"));
            assert_eq!(
                record["justification_ref"],
                serde_json::json!(format!("reached:{source}")),
                "{source} must justify the paths it reaches"
            );
            let codes = ci_artifact_accountability::evaluate_keyed(&registry);
            assert!(
                !codes.iter().any(|f| f.code == "unjustified"
                    && f.key == format!("oya/{source}/src/lib.rs")),
                "a path reached by {source} must not be unjustified"
            );
        }

        // The floor: reached by NOTHING ⇒ no justification laundered in, BOTH codes raised.
        let unreached = row("oya/unreached/src/lib.rs");
        assert_eq!(unreached["justification_ref"], serde_json::Value::Null);
        assert_eq!(unreached["verdict"], "RED");
        let codes: BTreeSet<String> = ci_artifact_accountability::evaluate_keyed(&registry)
            .into_iter()
            .filter(|f| f.key == "oya/unreached/src/lib.rs")
            .map(|f| f.code)
            .collect();
        assert!(
            codes.contains("unjustified") && codes.contains("unreachable"),
            "an unregistered artifact must still raise BOTH codes, got {codes:?}"
        );
    }

    /// The ADR corpus still wins when it names the path: the fallback never overwrites a real
    /// decision ref, so no existing `justification_ref` changes value.
    #[test]
    fn an_adr_justification_is_not_overwritten_by_the_reaching_source() {
        let policy = Policy::from_bundled().expect("policy");
        let registry = build_registry(&sample_inputs(), &policy).expect("registry");
        let rows = registry["rows"].as_array().expect("rows");
        let masterplan = rows
            .iter()
            .find(|r| r["path"] == "specs/masterplan.json")
            .expect("masterplan row");
        assert_eq!(masterplan["justification_ref"], "ADR-0364");
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
        let empty_face = serde_json::json!({"rows": []});
        let mut brand_residue: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        brand_residue
            .entry("forbidden_foundry".to_owned())
            .or_default()
            .insert("docs/products/foundry/PRD.md".to_owned());
        let inputs = GateInputs {
            total_accounting: &registry,
            cross_artifact: &crosswalk,
            automation_ratchet: &automation,
            bnf_layer_suffix: &empty_face,
            manifest_hygiene: &empty_face,
            cargo_prefix: &empty_face,
            slo_coverage: &empty_face,
            license_policy: &empty_face,
            catalog_liveness: &empty_face,
            workspace_glob_coverage: &empty_face,
            target_parity: &empty_face,
            enforcement_liveness: &empty_face,
            brand_residue: &brand_residue,
        };
        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let baseline = build_gate_baseline(&cfg, &inputs, "fnv1a64:test").expect("baseline");
        let ta = &baseline["gates"]["cloud-ci-total-accounting"];

        // unjustified is baseline-block-on-new and freezes the live key (the row path).
        assert_eq!(ta["unjustified"]["mode"], "baseline-block-on-new");
        assert_eq!(ta["unjustified"]["keys"][0], "oya/x/lib.rs");
        // ADR-0555 conversion: the exists-but-unaccounted codes are BLOCKING and still
        // freeze the live keys (the pre-existing debt is grandfathered by the ADR-0551
        // merge-base frozen baseline, not by an advisory mode). Each carries the exact
        // registration remediation as DATA — never a bare flag.
        assert_eq!(ta["unowned"]["mode"], "baseline-block-on-new");
        assert_eq!(ta["unowned"]["keys"][0], "oya/x/lib.rs");
        assert!(
            ta["unowned"]["remediation"]
                .as_str()
                .is_some_and(|t| t.contains("OWNERS") && t.contains("design act")),
            "unowned must stamp the exact ownership-registration remediation"
        );
        assert_eq!(ta["unreachable"]["mode"], "baseline-block-on-new");
        assert!(
            ta["unreachable"]["remediation"]
                .as_str()
                .is_some_and(|t| t.contains("specs/reachability-registry.json")),
            "unreachable must stamp the exact reachability-registration remediation"
        );
        assert_eq!(ta["no_ttl_class"]["mode"], "baseline-block-on-new");
        // stale_over_budget_unreachable stays advisory BY DESIGN (time-driven decay —
        // its convergence surface is the reaper reconciler, not admission).
        let sr = &baseline["gates"]["cloud-ci-staleness-reaper"];
        assert_eq!(
            sr["stale_over_budget_unreachable"]["mode"],
            "advisory-until-infra"
        );
        assert_eq!(sr["untyped_staleness"]["mode"], "baseline-block-on-new");
        // ci_inventory_registry_drift is frozen_empty: never accumulates a key even if one were present.
        assert_eq!(ta["ci_inventory_registry_drift"]["frozen_empty"], true);
        assert_eq!(
            ta["ci_inventory_registry_drift"]["keys"]
                .as_array()
                .unwrap()
                .len(),
            0
        );

        let xa = &baseline["gates"]["cloud-ci-cross-artifact-agreement"];
        assert_eq!(xa["dual_decision_collision"]["keys"][0], "ADR-0377");

        // brand-residue freezes the live per-(stem,file) key under its per-stem code.
        let br = &baseline["gates"]["cloud-ci-brand-residue"];
        assert_eq!(br["forbidden_foundry"]["mode"], "baseline-block-on-new");
        assert_eq!(
            br["forbidden_foundry"]["keys"][0],
            "docs/products/foundry/PRD.md"
        );
        // a stem with zero live residue freezes an empty (but present) key set.
        assert_eq!(br["forbidden_forgejo"]["keys"].as_array().unwrap().len(), 0);

        let ar = &baseline["gates"]["cloud-ci-automation-ratchet"];
        assert!(
            ar["advisory_claiming_enforced"]["remediation"]
                .as_str()
                .is_some_and(|t| t.contains("hermetic cloud-ci/Buck2 gate target")),
            "advisory enforcement claims must stamp an actionable cloud-ci remediation"
        );
        assert!(
            ar["blocking_invariant_mapped_to_oya_cli"]["remediation"]
                .as_str()
                .is_some_and(|t| t.contains("replace retired oya CLI authority")),
            "retired CLI authority findings must stamp the cloud-native replacement remediation"
        );
    }

    #[test]
    fn gate_baseline_is_idempotent_byte_for_byte() {
        let registry = serde_json::json!({"rows": []});
        let crosswalk = serde_json::json!({"decisions": []});
        let automation = serde_json::json!({"rows": []});
        let empty_face = serde_json::json!({"rows": []});
        let brand_residue: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let inputs = GateInputs {
            total_accounting: &registry,
            cross_artifact: &crosswalk,
            automation_ratchet: &automation,
            bnf_layer_suffix: &empty_face,
            manifest_hygiene: &empty_face,
            cargo_prefix: &empty_face,
            slo_coverage: &empty_face,
            license_policy: &empty_face,
            catalog_liveness: &empty_face,
            workspace_glob_coverage: &empty_face,
            target_parity: &empty_face,
            enforcement_liveness: &empty_face,
            brand_residue: &brand_residue,
        };
        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let a = to_canonical_json(&build_gate_baseline(&cfg, &inputs, "fnv1a64:test").expect("a"))
            .expect("ja");
        let b = to_canonical_json(&build_gate_baseline(&cfg, &inputs, "fnv1a64:test").expect("b"))
            .expect("jb");
        assert_eq!(a, b, "baseline must be byte-deterministic");
        assert!(a.contains("source_inputs_digest"));
        assert!(
            !a.contains("generated_at"),
            "no wall-clock in the baseline face"
        );
        assert!(
            a.contains(
                "\"producer_target\": \"//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin\""
            ),
            "baseline provenance must name the moved ci/facade producer target"
        );
        assert!(
            a.contains(
                "\"firewall_target\": \"//ci/facade/baseline-ratchet:ci-baseline-ratchet-gate\""
            ),
            "baseline provenance must name the moved ci/facade firewall target"
        );
        assert!(
            !a.contains("//cloud/cloud-ci/gates:"),
            "baseline provenance must not regress to retired cloud/cloud-ci targets"
        );
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

    /// SYNTHETIC NEUTRAL-FIXTURE (ADR-0533 item 6 / ADR-0532 amends ADR-0017): under the NEUTRAL
    /// profile the gate pipeline produces a baseline carrying ZERO oyatie/`oya-` brand literals,
    /// PROVING the neutral profile actually de-brands. The SAME corpus under the oyatie profile
    /// DOES carry brand findings (the safety property — oyatie is unchanged).
    ///
    /// Inputs deliberately include corpus that WOULD trip the brand-specific gates under oyatie:
    /// an unprefixed crate (`acme-bad` → cargo-prefix / bnf would flag `oya-` violations under
    /// oyatie), and a `foundry` residue file (brand-residue would flag `forbidden_foundry`).
    #[test]
    fn neutral_profile_baseline_emits_zero_brand_literals() {
        let registry = serde_json::json!({"rows": []});
        let crosswalk = serde_json::json!({"decisions": [], "duplicate_ids": []});
        let automation = serde_json::json!({"rows": []});
        let empty_face = serde_json::json!({"rows": []});
        // Corpus that trips the brand gates under OYATIE: an unprefixed crate + a foundry file.
        let cargo_face = serde_json::json!({"rows": [
            {"member_path": "crates/acme-bad", "package_name": "acme-bad"}
        ]});
        let bnf_face = serde_json::json!({"rows": [ {"crate_name": "acme-bad"} ]});
        let mut brand_residue: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        brand_residue
            .entry("forbidden_foundry".to_owned())
            .or_default()
            .insert("docs/products/foundry/PRD.md".to_owned());

        // --- NEUTRAL: brand_residue is supplied EMPTY (the producer's collector uses the
        // neutral [vocab] policy = empty deny-list, so it would find nothing); cargo/bnf faces
        // carry the unprefixed crate but the neutral [naming] prefix is empty ⇒ no violation. ---
        let neutral_residue: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let neutral_inputs = GateInputs {
            total_accounting: &registry,
            cross_artifact: &crosswalk,
            automation_ratchet: &automation,
            bnf_layer_suffix: &bnf_face,
            manifest_hygiene: &empty_face,
            cargo_prefix: &cargo_face,
            slo_coverage: &empty_face,
            license_policy: &empty_face,
            catalog_liveness: &empty_face,
            workspace_glob_coverage: &empty_face,
            target_parity: &empty_face,
            enforcement_liveness: &empty_face,
            brand_residue: &neutral_residue,
        };
        let neutral_cfg = oya_ci_config_kernel::OyaCiConfig::neutral();
        let neutral_baseline =
            build_gate_baseline(&neutral_cfg, &neutral_inputs, "fnv1a64:neutral").expect("neutral");
        // THE DE-BRAND PROOF: the GATE FINDINGS (the `gates` object — codes + keys the gates
        // emit) carry ZERO oyatie/oya- brand literals under neutral. (The face's `_comment` /
        // `_provenance` legitimately name the producer crate itself — that is producer
        // self-identification, not a policy literal — so the proof is scoped to `gates`.)
        let neutral_gates_json = to_canonical_json(&neutral_baseline["gates"])
            .expect("neutral gates findings serialize");
        for needle in [
            "oya-",
            "oyatie",
            "forbidden_foundry",
            "foundry",
            "oya-governance",
            "oya-check-",
            "cargo_prefix_violation",
            "bnf_missing_oya_prefix",
        ] {
            assert!(
                !neutral_gates_json.contains(needle),
                "neutral gate findings leaked brand literal {needle:?}:\n{neutral_gates_json}"
            );
        }
        // "Gates present-but-quiet" (ADR-0533 item 1): the gates are PRESENT (every enabled
        // gate still appears so the engine dispatches every KIND) but QUIET (each gate's code
        // object is empty — the neutral disposition stamps no codes).
        let neutral_gates = neutral_baseline["gates"]
            .as_object()
            .expect("neutral gates object");
        let enabled_gate_count = neutral_cfg.gates.enabled.len();
        assert_eq!(
            neutral_gates.len(),
            enabled_gate_count,
            "gates present (all enabled gates)"
        );
        for (gate_id, codes) in neutral_gates {
            assert_eq!(
                codes.as_object().map(|m| m.len()),
                Some(0),
                "gate {gate_id} must be quiet (no codes) under neutral"
            );
        }

        // SAFETY PROPERTY: the SAME unprefixed/foundry corpus under the OYATIE profile DOES
        // surface the brand findings — oyatie behaviour is unchanged.
        let oyatie_inputs = GateInputs {
            total_accounting: &registry,
            cross_artifact: &crosswalk,
            automation_ratchet: &automation,
            bnf_layer_suffix: &bnf_face,
            manifest_hygiene: &empty_face,
            cargo_prefix: &cargo_face,
            slo_coverage: &empty_face,
            license_policy: &empty_face,
            catalog_liveness: &empty_face,
            workspace_glob_coverage: &empty_face,
            target_parity: &empty_face,
            enforcement_liveness: &empty_face,
            brand_residue: &brand_residue,
        };
        let oyatie_cfg = oya_ci_config_kernel::OyaCiConfig::oyatie();
        let oyatie_baseline =
            build_gate_baseline(&oyatie_cfg, &oyatie_inputs, "fnv1a64:oyatie").expect("oyatie");
        let cp = &oyatie_baseline["gates"]["cloud-ci-cargo-prefix"]["cargo_prefix_violation"];
        assert_eq!(
            cp["keys"][0], "acme-bad",
            "oyatie profile must flag the unprefixed crate (safety: unchanged)"
        );
        let br = &oyatie_baseline["gates"]["cloud-ci-brand-residue"]["forbidden_foundry"];
        assert_eq!(
            br["keys"][0], "docs/products/foundry/PRD.md",
            "oyatie profile must flag the foundry residue (safety: unchanged)"
        );
    }

    // -----------------------------------------------------------------------
    // Registration BRIDGE tests (slice 2.5): the three functions extracted from the binary
    // were previously reachable only via the CLI; these exercise them as the LIBRARY API the
    // slice-3 register_crate app will call. std-only, each on a fresh tmpdir.
    // -----------------------------------------------------------------------

    /// A unique throwaway repo dir under the system temp dir (std-only; no tempfile dep).
    fn unique_temp_repo() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "oya-bornaccount-libtest-{}-{nanos}-{seq}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("create temp repo");
        dir
    }

    fn tracked(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|p| (*p).to_owned()).collect()
    }

    #[test]
    fn fix_owners_applies_the_decided_edit_and_self_validates() {
        let root = unique_temp_repo();
        std::fs::create_dir_all(root.join("docs/adr-archive")).expect("create dir");
        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let scm =
            tracked(&["docs/adr-archive/ADR-0001-cohesion-thesis-one-product-flat-catalog.md"]);
        let message = fix_owners(&root, &cfg, &scm, "docs/adr-archive=council-architecture")
            .expect("fix applies");
        assert!(message.contains("1 tracked path(s)"), "{message}");
        assert_eq!(
            std::fs::read_to_string(root.join("docs/adr-archive/OWNERS")).expect("read"),
            "council-architecture\n"
        );
        // re-application refuses: an existing registration is extended by hand (reviewed).
        assert!(fix_owners(&root, &cfg, &scm, "docs/adr-archive=council-architecture").is_err());
        // the owner is a required DESIGN DECISION — a bare dir is refused.
        assert!(fix_owners(&root, &cfg, &scm, "docs/adr-archive=").is_err());
        // path traversal / absolute dirs are refused (the bridge cannot escape the repo).
        assert!(fix_owners(&root, &cfg, &scm, "/etc=evil").is_err());
        assert!(fix_owners(&root, &cfg, &scm, "../outside=evil").is_err());
        // a self-validation failure (no tracked path under the dir) leaves no OWNERS residue.
        std::fs::create_dir_all(root.join("empty-dir")).expect("create empty dir");
        assert!(fix_owners(&root, &cfg, &scm, "empty-dir=team").is_err());
        assert!(
            !root.join("empty-dir/OWNERS").exists(),
            "failed self-validation must remove the written OWNERS file"
        );
        std::fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn root_owners_covers_root_data_and_top_level_buck_metadata_not_subtrees() {
        let root = unique_temp_repo();
        std::fs::write(root.join("OWNERS"), "cloud-ci-platform\n").expect("write root owners");
        std::fs::create_dir_all(root.join("docs")).expect("create docs dir");
        std::fs::write(root.join("docs/doc.md"), "doc\n").expect("write nested doc");
        std::fs::create_dir_all(root.join("cloud/gate")).expect("create gate dir");
        std::fs::write(root.join("cloud/gate/app.rs"), "fn main() {}\n")
            .expect("write nested code");

        let cfg = oya_ci_config_kernel::OyaCiConfig::from_toml_str(
            "[owners]\nmax_paths_per_owners_file = 3\n",
        )
        .expect("bound parses");
        let resolution = resolve_owners(
            &root,
            &tracked(&[
                "OWNERS",
                "oya-deps.toml",
                "oya/BUCK",
                "docs/doc.md",
                "cloud/gate/app.rs",
            ]),
            &cfg,
        );

        assert_eq!(
            resolution.by_path,
            BTreeMap::from([
                ("OWNERS".to_owned(), "OWNERS:".to_owned()),
                ("oya-deps.toml".to_owned(), "OWNERS:".to_owned()),
                ("oya/BUCK".to_owned(), "OWNERS:".to_owned())
            ]),
            "root OWNERS should cover root DATA and direct top-level BUCK metadata, not blanket subtrees"
        );
        assert!(
            resolution.integrity.over_broad.is_empty(),
            "nested paths skipped by root OWNERS must not count against the breadth bound"
        );
        std::fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn fix_owners_refuses_schema_invalid_and_over_broad_registrations() {
        let root = unique_temp_repo();
        std::fs::create_dir_all(root.join("docs/adr-archive")).expect("create dir");
        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let scm =
            tracked(&["docs/adr-archive/ADR-0001-cohesion-thesis-one-product-flat-catalog.md"]);

        // A principal the resolver would reject must be refused BEFORE writing.
        for hostile in ["Team Evil", "EVIL", "evil!", "a@b.example", "-x"] {
            let err = fix_owners(&root, &cfg, &scm, &format!("docs/adr-archive={hostile}"))
                .expect_err("schema-invalid principal must be refused");
            assert!(
                format!("{err:?}").contains("not a valid owner principal"),
                "refusal must name the schema defect, got {err:?}"
            );
            assert!(
                !root.join("docs/adr-archive/OWNERS").exists(),
                "a refused registration must leave no OWNERS residue"
            );
        }

        // The bulk-neuter shape: a registration covering more tracked paths than the
        // bound is refused with the split-the-registration fix, and the written file is
        // reverted.
        let small_bound_cfg = oya_ci_config_kernel::OyaCiConfig::from_toml_str(
            "[owners]\nmax_paths_per_owners_file = 3\n",
        )
        .expect("bound parses");
        let bulk = tracked(&[
            "docs/adr-archive/ADR-0001-cohesion-thesis-one-product-flat-catalog.md",
            "docs/adr-archive/ADR-0002-tenant-and-identity-kernel.md",
            "docs/adr-archive/ADR-0003-audit-chain-and-evidence-emission.md",
            "docs/adr-archive/ADR-0004-plane-separation-control-data-analytics.md",
        ]);
        let err = fix_owners(
            &root,
            &small_bound_cfg,
            &bulk,
            "docs/adr-archive=council-architecture",
        )
        .expect_err("an over-broad registration must be refused");
        let message = format!("{err:?}");
        assert!(
            message.contains("max_paths_per_owners_file") && message.contains("split"),
            "refusal must name the bound and the split fix, got {message}"
        );
        assert!(
            !root.join("docs/adr-archive/OWNERS").exists(),
            "the over-broad OWNERS must be reverted (no residue)"
        );

        // Under the bound the same registration applies cleanly (the bound only catches
        // bulk shapes, never legitimate trees).
        let ok = fix_owners(
            &root,
            &small_bound_cfg,
            &scm,
            "docs/adr-archive=council-architecture",
        )
        .expect("a within-bound registration applies");
        assert!(ok.contains("1 tracked path(s)"), "{ok}");

        std::fs::remove_dir_all(root).expect("remove temp repo");
    }

    /// THE SAFETY ARGUMENT, end to end on a real tree: a schema-VALID OWNERS file is
    /// accounted by construction; a schema-INVALID one stays fully RED. Both files are the
    /// same name in the same shape — only the CONTENT differs — so this pins that the floor
    /// is keyed on the parse verdict and cannot be reached by naming a file `OWNERS`.
    ///
    /// The tree is deliberately barren: no ADRs, no masterplan, no reachability registry, no
    /// Cargo workspace. So NOTHING except the derivation can justify or reach either file —
    /// which is exactly the `os/OWNERS` situation that turned `dev` RED in PR #1473.
    #[test]
    fn valid_owners_file_is_accounted_by_construction_and_invalid_one_stays_red() {
        let root = unique_temp_repo();
        std::fs::create_dir_all(root.join("good")).expect("create good dir");
        std::fs::create_dir_all(root.join("bad")).expect("create bad dir");
        std::fs::write(root.join("good/OWNERS"), "cloud-ci-platform\n").expect("write valid");
        // Comment-only: the ADR-0555 schema's canonical NOT-ownership case.
        std::fs::write(root.join("bad/OWNERS"), "# owner: TBD\n").expect("write invalid");
        std::fs::write(root.join("good/thing.rs"), "fn main() {}\n").expect("write covered");
        std::fs::write(root.join("bad/thing.rs"), "fn main() {}\n").expect("write covered");

        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let paths = tracked(&["bad/OWNERS", "bad/thing.rs", "good/OWNERS", "good/thing.rs"]);
        let resolution = resolve_owners(&root, &paths, &cfg);
        assert_eq!(
            resolution.valid_files,
            BTreeSet::from(["good/OWNERS".to_owned()]),
            "only the schema-valid OWNERS file may carry the accounting floor"
        );
        assert!(
            resolution.integrity.invalid.contains_key("bad/OWNERS"),
            "the comment-only file must be reported as a schema defect"
        );

        let policy = Policy::from_bundled().expect("policy");
        let inputs = RepoInputs {
            tracked_paths: paths,
            owners: resolution.by_path,
            // Barren tree: nothing else accounts anything.
            justifications: BTreeMap::new(),
            reachability: BTreeMap::new(),
            dup_of: BTreeMap::new(),
            valid_owners_files: resolution.valid_files,
            placement: CapabilityPlacement::default(),
            planned_move_paths: BTreeSet::new(),
        };
        let registry = build_registry(&inputs, &policy).expect("registry");
        let row = |path: &str| -> Value {
            registry["rows"]
                .as_array()
                .expect("rows")
                .iter()
                .find(|r| r["path"] == path)
                .expect("row")
                .clone()
        };

        // GREEN: the valid OWNERS file, with ZERO hand-written registrations anywhere.
        let good = row("good/OWNERS");
        assert_eq!(good["justification_ref"], OWNERS_SCHEMA_ANCHOR);
        assert_eq!(
            good["reachable_from"],
            serde_json::json!([OWNERS_SCHEMA_ANCHOR])
        );
        let good_findings: BTreeSet<String> = ci_artifact_accountability::evaluate_keyed(&registry)
            .into_iter()
            .filter(|f| f.key == "good/OWNERS")
            .map(|f| f.code)
            .collect();
        assert!(
            good_findings.is_empty(),
            "a schema-valid OWNERS file must raise NO accounting violation, got {good_findings:?}"
        );

        // RED: the invalid one is not laundered — it stays unjustified AND unreachable AND
        // unowned (invalid content poisons its own directory, ADR-0555 fail-closed).
        let bad = row("bad/OWNERS");
        assert_eq!(bad["justification_ref"], Value::Null);
        assert_eq!(bad["reachable_from"], serde_json::json!([]));
        assert_eq!(bad["verdict"], "RED");
        let bad_findings: BTreeSet<String> = ci_artifact_accountability::evaluate_keyed(&registry)
            .into_iter()
            .filter(|f| f.key == "bad/OWNERS")
            .map(|f| f.code)
            .collect();
        for code in ["unjustified", "unreachable", "unowned"] {
            assert!(
                bad_findings.contains(code),
                "a schema-INVALID OWNERS file must keep firing {code}, got {bad_findings:?}"
            );
        }

        // The floor NEVER reaches a non-OWNERS path, even one the valid file owns.
        let covered = row("good/thing.rs");
        assert_eq!(covered["justification_ref"], Value::Null);
        assert_eq!(covered["reachable_from"], serde_json::json!([]));
        assert_eq!(covered["verdict"], "RED");

        std::fs::remove_dir_all(root).expect("remove temp repo");
    }

    /// The floor only ever FILLS AN ABSENCE: a valid OWNERS file that already resolved a
    /// justification / reachability source keeps exactly what it resolved. This is what makes
    /// "no other row changes" a property of the code rather than of the corpus.
    #[test]
    fn owners_floor_never_overrides_a_resolved_justification_or_reachability() {
        let policy = Policy::from_bundled().expect("policy");
        let inputs = RepoInputs {
            tracked_paths: tracked(&["cloud/x/OWNERS"]),
            owners: BTreeMap::from([("cloud/x/OWNERS".to_owned(), "OWNERS:cloud/x".to_owned())]),
            justifications: BTreeMap::from([("cloud/x/OWNERS".to_owned(), "ADR-0543".to_owned())]),
            reachability: BTreeMap::from([(
                "cloud/x/OWNERS".to_owned(),
                vec!["cargo-members".to_owned()],
            )]),
            dup_of: BTreeMap::new(),
            valid_owners_files: BTreeSet::from(["cloud/x/OWNERS".to_owned()]),
            placement: CapabilityPlacement::default(),
            planned_move_paths: BTreeSet::new(),
        };
        let registry = build_registry(&inputs, &policy).expect("registry");
        let row = &registry["rows"].as_array().expect("rows")[0];
        assert_eq!(row["justification_ref"], "ADR-0543");
        assert_eq!(row["reachable_from"], serde_json::json!(["cargo-members"]));
    }

    /// ORDERING: the OWNERS floor runs BEFORE the reached ⇒ justified fallback. Both rules
    /// fill an absent `justification_ref`, so on a valid OWNERS file that ALSO resolves a
    /// reachability source the two race — and only the floor's answer keeps the row findable
    /// by the `justification_ref == "owners-schema"` census [`OWNERS_SCHEMA_ANCHOR`] documents.
    ///
    /// TODAY's corpus does not exercise this: all 108 tracked OWNERS files either carry an ADR
    /// justification already or resolve no reachability but the floor's own. It bites as the
    /// reorg lands new OWNERS files under cargo members with no ADR prose — exactly the case
    /// the floor exists to serve — so the ordering needs a test, not a comment.
    #[test]
    fn owners_floor_wins_over_the_reached_fallback_for_an_unjustified_valid_owners_file() {
        let policy = Policy::from_bundled().expect("policy");
        let inputs = RepoInputs {
            tracked_paths: tracked(&["cloud/x/OWNERS"]),
            owners: BTreeMap::new(),
            justifications: BTreeMap::new(),
            reachability: BTreeMap::from([(
                "cloud/x/OWNERS".to_owned(),
                vec!["cargo-members".to_owned()],
            )]),
            dup_of: BTreeMap::new(),
            valid_owners_files: BTreeSet::from(["cloud/x/OWNERS".to_owned()]),
            placement: CapabilityPlacement::default(),
            planned_move_paths: BTreeSet::new(),
        };
        let registry = build_registry(&inputs, &policy).expect("registry");
        let row = &registry["rows"].as_array().expect("rows")[0];
        assert_eq!(
            row["justification_ref"], OWNERS_SCHEMA_ANCHOR,
            "the floor must stamp the by-construction anchor, not `reached:cargo-members`"
        );
        // The floor never displaces a reachability source it did not need to supply.
        assert_eq!(row["reachable_from"], serde_json::json!(["cargo-members"]));
    }

    /// The bridge cannot re-create the rows this change deleted: registering an OWNERS path
    /// is refused with the real remedy (fix the file's CONTENT), and leaves the registry
    /// byte-identical. Without this the paved road would write a row the gate then rejects.
    #[test]
    fn fix_reachability_refuses_owners_paths_as_accounted_by_construction() {
        let root = unique_temp_repo();
        std::fs::create_dir_all(root.join("specs")).expect("create specs");
        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let registry_rel = cfg.reachability.registry.clone();
        let seed = "{\n  \"registered\": []\n}\n";
        std::fs::write(root.join(&registry_rel), seed).expect("seed registry");

        for prefix in ["OWNERS", "os/OWNERS", "cloud/cloud-kernel/OWNERS"] {
            let err = fix_reachability(
                &root,
                &cfg,
                &tracked(&[prefix]),
                &format!("{prefix}=some hand-written anchor prose"),
            )
            .expect_err("registering an OWNERS path must be refused");
            let message = format!("{err:?}");
            assert!(
                message.contains("accounted by") && message.contains("schema"),
                "the refusal must name the construction + the real remedy, got {message}"
            );
            assert_eq!(
                std::fs::read_to_string(root.join(&registry_rel)).expect("read"),
                seed,
                "a refused registration must leave the registry byte-identical"
            );
        }

        // Control: a NON-OWNERS path still registers, so the refusal is scoped, not a
        // blanket break of the bridge.
        fix_reachability(
            &root,
            &cfg,
            &tracked(&["specs/thing.json"]),
            "specs/thing.json=reviewed anchor",
        )
        .expect("a non-OWNERS registration still applies");

        std::fs::remove_dir_all(root).expect("remove temp repo");
    }

    #[test]
    fn fix_reachability_appends_registers_sorts_and_round_trips() {
        let root = unique_temp_repo();
        std::fs::create_dir_all(root.join("specs")).expect("create specs");
        let cfg = oya_ci_config_kernel::OyaCiConfig::bundled_default();
        let scm = tracked(&["specs/fixtures/x/tc-1.json"]);
        let message = fix_reachability(
            &root,
            &cfg,
            &scm,
            "specs/fixtures/=gates' data-under-test, dir-loaded by gate tests",
        )
        .expect("fix applies");
        assert!(message.contains("1 tracked path(s)"), "{message}");
        // duplicate registration refused (idempotent-ish: a second identical prefix is a no-op error).
        assert!(fix_reachability(&root, &cfg, &scm, "specs/fixtures/=again").is_err());
        // the anchor is a required DESIGN DECISION — a bare prefix is refused.
        assert!(fix_reachability(&root, &cfg, &scm, "third-party/=").is_err());

        // Appending a SECOND, alphabetically-earlier prefix must sort it ahead in the
        // canonical on-disk registry (the bridge sorts before writing).
        fix_reachability(&root, &cfg, &scm, "evidence/=gate evidence corpus").expect("second");
        let registry_abs = root.join(cfg.reachability.registry.as_str());
        let entries = load_reachability_registry(&registry_abs).expect("registry round-trips");
        let prefixes: Vec<&str> = entries.iter().map(|e| e.prefix.as_str()).collect();
        assert_eq!(
            prefixes,
            vec!["evidence/", "specs/fixtures/"],
            "registry entries must be prefix-sorted canonically"
        );
        std::fs::remove_dir_all(root).expect("remove temp repo");
    }

    /// The standalone allocator must pick max+1 across BOTH the filename number AND the
    /// front-matter `id:` — including the case where the front-matter id EXCEEDS the
    /// filename number (the FRIC-1781320000 re-keying shape that must not be undercounted).
    #[test]
    fn allocate_next_adr_id_takes_max_over_filename_and_front_matter() {
        let root = unique_temp_repo();
        let decisions = root.join("docs/decisions");
        std::fs::create_dir_all(&decisions).expect("create decisions dir");
        // filename number 2, but front-matter id 9 (front-matter EXCEEDS filename) ⇒ max 9.
        std::fs::write(
            decisions.join("ADR-0002-mismatch.md"),
            "---\nid: ADR-0009\nstatus: Proposed\n---\n",
        )
        .expect("write mismatched ADR");
        // a plain filename-only ADR below the front-matter max.
        std::fs::write(
            decisions.join("ADR-0005-plain.md"),
            "---\nstatus: Accepted\n---\n",
        )
        .expect("write plain ADR");
        // a non-ADR file is ignored entirely.
        std::fs::write(decisions.join("README.md"), "not an ADR\n").expect("write readme");

        let next = allocate_next_adr_id(&decisions).expect("allocate");
        assert_eq!(
            next, "ADR-0010",
            "max(filename 2,5; front-matter 9) + 1 = 10"
        );

        // An empty / missing decisions dir is the zero-config default (ADR-0001).
        let empty = unique_temp_repo();
        assert_eq!(
            allocate_next_adr_id(&empty.join("docs/decisions")).expect("missing dir"),
            "ADR-0001"
        );

        std::fs::remove_dir_all(root).expect("remove temp repo");
        std::fs::remove_dir_all(empty).expect("remove empty");
    }

    /// The owner-principal grammar: every live principal shape is accepted; the obvious
    /// hostile/garbage shapes are rejected (ADR-0555 hardening, FRIC-1781400000).
    #[test]
    fn owner_principal_schema_accepts_live_shapes_and_rejects_garbage() {
        for valid in [
            "cloud-ci-platform",
            "council-architecture",
            "axis-cloud-platform",
            "team0",
            "a",
        ] {
            assert!(is_valid_owner_principal(valid), "{valid:?} must be valid");
        }
        let too_long = "a".repeat(64);
        for invalid in [
            "",
            "Team-Evil",
            "EVIL",
            "team evil",
            "-leading-hyphen",
            "trailing-hyphen-",
            "dot.separated",
            "email@example.com",
            "tab\tseparated",
            too_long.as_str(),
        ] {
            assert!(
                !is_valid_owner_principal(invalid),
                "{invalid:?} must be rejected"
            );
        }
    }

    #[test]
    fn envelope_glob_to_prefix_accepts_dir_star_star_only() {
        assert_eq!(
            envelope_glob_to_prefix("compute/**").as_deref(),
            Some("compute/")
        );
        assert_eq!(
            envelope_glob_to_prefix("app/payments/**").as_deref(),
            Some("app/payments/")
        );
        assert_eq!(envelope_glob_to_prefix("compute/"), None);
        assert_eq!(envelope_glob_to_prefix("compute/*"), None);
        assert_eq!(envelope_glob_to_prefix("**/evil"), None);
        assert_eq!(envelope_glob_to_prefix("/**"), None);
        assert_eq!(envelope_glob_to_prefix("../escape/**"), None);
    }

    #[test]
    fn load_envelope_prefix_allows_covers_owned_prefix_without_tip_free() {
        let root = std::env::temp_dir().join(format!(
            "oya-envelope-prefix-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("specs")).expect("specs dir");
        let envelopes = root.join(ENVELOPES_RELPATH);
        std::fs::write(
            &envelopes,
            r#"{
              "roots": {
                "compute": {
                  "branch": "integ/compute",
                  "envelope_globs": ["compute/**"]
                },
                "iac": {
                  "branch": "integ/iac",
                  "envelope_globs": ["iac/**"]
                }
              }
            }"#,
        )
        .expect("write envelopes");

        let allows = load_envelope_prefix_allows(&envelopes).expect("load");
        assert_eq!(allows.len(), 2);
        assert!(allows.iter().any(|e| e.prefix == "compute/"));
        assert!(allows.iter().any(|e| e.prefix == "iac/"));
        assert!(registration_matches(
            "compute/manifest.json",
            &allows
                .iter()
                .find(|e| e.prefix == "compute/")
                .unwrap()
                .prefix
        ));
        assert!(registration_matches(
            "iac/governance/note.md",
            &allows.iter().find(|e| e.prefix == "iac/").unwrap().prefix
        ));
        assert!(!registration_matches("compute-evil/x.rs", "compute/"));

        // missing file ⇒ empty (fixture zero-config)
        std::fs::remove_file(&envelopes).expect("remove");
        assert!(
            load_envelope_prefix_allows(&envelopes)
                .expect("missing ok")
                .is_empty()
        );
        // present without roots ⇒ fail-loud
        std::fs::write(&envelopes, "{}").expect("write empty");
        assert!(load_envelope_prefix_allows(&envelopes).is_err());

        let _ = std::fs::remove_dir_all(root);
    }
}
