//! # oya-ci-config-kernel — the oya-ci CONFORMANCE-FLOOR config kernel
//!
//! Pure, I/O-free parse + CLOSED-schema validation of the portable oya-ci policy into
//! typed structs (OYA-CI-CONFORMANCE-FLOOR-PLAN §3.1, §3.3). This crate holds NO scanner
//! logic and touches NO filesystem at runtime: the producer does the git/FS I/O and feeds
//! it; this kernel only parses + validates DATA, exactly like the existing strict
//! `Policy::from_strs`.
//!
//! ## What it carries (every section maps to a surveyed oyatie literal, plan §2/§3.1)
//! - `[repo]` roots + path-filter exclusions (replaces `discover_repo_root` marker +
//!   `collect_*` `third-party/` exclusion).
//! - `[naming]` required-prefix / allowed-roles / check-family-prefix / backend-suffixes /
//!   doctrinal-carve-outs (replaces the naming-kernel consts, §2.1).
//! - `[vocab]` forbidden-stems + carve-outs (replaces the brand consts, §2.2).
//! - `[manifest]` the §2.5#7 required-flag field-set (replaces `ManifestFlags`, §2.3).
//! - `[reachability]` / `[justification]` / `[owners]` / `[enforcement]` source paths
//!   (replaces the producer-embedded literals, §2.3).
//! - `[ttl]` + `[unit_class]` (subsumes ttl-policy.json + unit-class-policy.json — already
//!   DATA, carried over VERBATIM so the 48k+ accounting keys reproduce byte-for-byte).
//! - `[gates]` enabled list + per-gate `input_kind` + per-(gate,code) dispositions
//!   (subsumes + relocates `GATE_IDS` + gate-disposition.json).
//!
//! ## The BUNDLED DEFAULT == today's hardcoded policy
//! [`OyaCiConfig::bundled_default`] reproduces oyatie's CURRENT `const`/JSON values exactly,
//! so the producer reading the (defaulted) config produces BYTE-IDENTICAL findings. The
//! `unit_class` + `ttl` tables are embedded as the existing JSON (`include_str!`) so the
//! classification is bit-exact; the rest are Rust literals matching the surveyed consts.
//!
//! ## CLOSED schema
//! Every struct is `#[serde(deny_unknown_fields)]`: an unknown key in `oya-ci.toml` is a
//! hard error (PM-2 mitigation — the schema is the shared spine, not free-form). Sections
//! are `#[serde(default)]` so a partial/absent file materializes the bundled defaults.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// The two carry-over DATA tables, embedded verbatim from the producer's bundled JSON so the
/// classification (and thus the 48k+ total-accounting keys) reproduces byte-for-byte. These
/// are compile-time `include_str!` constants — NOT runtime I/O.
const BUNDLED_UNIT_CLASS_JSON: &str = include_str!("bundled/unit-class-policy.json");
const BUNDLED_TTL_JSON: &str = include_str!("bundled/ttl-policy.json");

/// A config load/validation error. No panics escape the parse path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// The TOML text was malformed or violated the closed schema (unknown key, wrong type).
    Parse(String),
    /// An embedded bundled DATA table failed to parse (should be impossible; guards the const).
    Bundled(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Parse(m) => write!(f, "oya-ci config parse error: {m}"),
            ConfigError::Bundled(m) => write!(f, "oya-ci bundled-default error: {m}"),
        }
    }
}

impl std::error::Error for ConfigError {}

// ---------------------------------------------------------------------------
// Top-level config
// ---------------------------------------------------------------------------

/// The full, validated oya-ci policy. Parsed from `oya-ci.toml`, or materialized from the
/// bundled default when no file is present.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OyaCiConfig {
    #[serde(default)]
    pub repo: RepoConfig,
    #[serde(default)]
    pub naming: NamingConfig,
    #[serde(default)]
    pub vocab: VocabConfig,
    #[serde(default)]
    pub manifest: ManifestConfig,
    #[serde(default)]
    pub reachability: ReachabilityConfig,
    #[serde(default)]
    pub justification: JustificationConfig,
    #[serde(default)]
    pub owners: OwnersConfig,
    #[serde(default)]
    pub enforcement: EnforcementConfig,
    #[serde(default)]
    pub slo_coverage: SloCoverageConfig,
    #[serde(default)]
    pub ttl: TtlConfig,
    #[serde(default)]
    pub unit_class: UnitClassConfig,
    #[serde(default)]
    pub gates: GatesConfig,
}

impl Default for OyaCiConfig {
    fn default() -> Self {
        Self::bundled_default()
    }
}

impl OyaCiConfig {
    /// Materialize oyatie's CURRENT policy as the bundled default (no file required). This is
    /// the byte-for-byte equivalent of today's hardcoded `const`s + embedded JSON.
    pub fn bundled_default() -> Self {
        Self {
            repo: RepoConfig::default(),
            naming: NamingConfig::default(),
            vocab: VocabConfig::default(),
            manifest: ManifestConfig::default(),
            reachability: ReachabilityConfig::default(),
            justification: JustificationConfig::default(),
            owners: OwnersConfig::default(),
            enforcement: EnforcementConfig::default(),
            slo_coverage: SloCoverageConfig::default(),
            ttl: TtlConfig::default(),
            unit_class: UnitClassConfig::default(),
            gates: GatesConfig::default(),
        }
    }

    /// Parse + closed-schema-validate an `oya-ci.toml`. Absent sections fall back to the
    /// bundled default for that section (via each struct's `Default`). Unknown keys error.
    pub fn from_toml_str(text: &str) -> Result<Self, ConfigError> {
        toml::from_str(text).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    /// A stable, dependency-free FNV-1a 64-bit digest of the config's canonical TOML
    /// serialization. Stamped into the gate-baseline `_provenance` so a config change is
    /// visible to registry-drift (the byte-diff) and the firewall (provenance audit). The
    /// digest is deterministic (the structs serialize in a fixed field order), so
    /// committed==regenerated holds without a wall-clock.
    pub fn digest(&self) -> String {
        let canonical = toml::to_string(self).unwrap_or_default();
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in canonical.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        format!("fnv1a64:{hash:016x}")
    }

    /// The bundled `unit-class-policy.json` body (DATA the producer hands to its classifier).
    /// Carried over verbatim so the classification is bit-exact.
    pub fn unit_class_policy_json(&self) -> &str {
        match &self.unit_class.inline_json {
            Some(json) => json.as_str(),
            None => BUNDLED_UNIT_CLASS_JSON,
        }
    }

    /// The bundled `ttl-policy.json` body (DATA the producer hands to its classifier).
    pub fn ttl_policy_json(&self) -> &str {
        match &self.ttl.inline_json {
            Some(json) => json.as_str(),
            None => BUNDLED_TTL_JSON,
        }
    }
}

// ---------------------------------------------------------------------------
// [repo]
// ---------------------------------------------------------------------------

/// `[repo]` — repo-root marker(s) + tracked-path exclusions (plan §3.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepoConfig {
    /// Repo-root marker file(s): the producer walks up-tree until one is present. Today's
    /// marker is `specs/root-hub-pointers.json` (`discover_repo_root`).
    #[serde(default = "default_root_markers")]
    pub root_markers: Vec<String>,
    /// Tracked-path prefixes excluded from the `collect_*` scans (today: `third-party/`).
    #[serde(default = "default_path_excludes")]
    pub path_excludes: Vec<String>,
}

fn default_root_markers() -> Vec<String> {
    vec!["specs/root-hub-pointers.json".to_owned()]
}

fn default_path_excludes() -> Vec<String> {
    vec!["third-party/".to_owned()]
}

impl Default for RepoConfig {
    fn default() -> Self {
        Self {
            root_markers: default_root_markers(),
            path_excludes: default_path_excludes(),
        }
    }
}

// ---------------------------------------------------------------------------
// [naming]  (replaces the naming-kernel consts, plan §2.1)
// ---------------------------------------------------------------------------

/// `[naming]` — the predictable-naming policy (replaces the naming-kernel `const`s, §2.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamingConfig {
    #[serde(default = "default_required_prefix")]
    pub required_prefix: String,
    #[serde(default = "default_allowed_roles")]
    pub allowed_roles: Vec<String>,
    #[serde(default = "default_check_family_prefix")]
    pub check_family_prefix: String,
    #[serde(default = "default_backend_suffixes")]
    pub backend_suffixes: Vec<String>,
    #[serde(default = "default_doctrinal_carve_outs")]
    pub doctrinal_carve_outs: Vec<String>,
}

fn default_required_prefix() -> String {
    "oya-".to_owned()
}

fn default_allowed_roles() -> Vec<String> {
    [
        "kernel",
        "domain",
        "usecase",
        "app",
        "adapter",
        "infrastructure",
        "cli",
        "rest",
        "grpc",
        "graphql",
        "worker",
        "sdk",
        "api",
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect()
}

fn default_check_family_prefix() -> String {
    "oya-check-".to_owned()
}

fn default_backend_suffixes() -> Vec<String> {
    [
        "fake", "inmemory", "aws", "oci", "gcp", "azure", "postgres", "redis", "sqlite",
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect()
}

fn default_doctrinal_carve_outs() -> Vec<String> {
    vec!["oya-tooling-agent-read".to_owned()]
}

impl Default for NamingConfig {
    fn default() -> Self {
        Self {
            required_prefix: default_required_prefix(),
            allowed_roles: default_allowed_roles(),
            check_family_prefix: default_check_family_prefix(),
            backend_suffixes: default_backend_suffixes(),
            doctrinal_carve_outs: default_doctrinal_carve_outs(),
        }
    }
}

// ---------------------------------------------------------------------------
// [vocab]  (replaces the brand-residue consts, plan §2.2)
// ---------------------------------------------------------------------------

/// One forbidden-vocab stem (a row of `FORBIDDEN_VOCAB_STEMS`, §2.2). NOTE the hyphenated
/// `code` value (`forbidden_oya-vcs`) — a TOML string value, round-trips unchanged (MF-7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForbiddenStem {
    pub stem: String,
    pub code: String,
}

/// The kind of a vocab carve-out (mirrors `CarveOutKind` in the brand crate, §2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VocabCarveOutKind {
    PathPrefix,
    PathExact,
    PathSuffix,
    LineContainsCi,
}

/// One vocab carve-out rule (a row of `CARVE_OUT_RULES`, §2.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VocabCarveOut {
    pub kind: VocabCarveOutKind,
    pub value: String,
    #[serde(default)]
    pub reason: String,
}

/// `[vocab]` — the forbidden-vocab shrink-only-ratchet policy (replaces the brand `const`s).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VocabConfig {
    #[serde(default = "default_forbidden_stems")]
    pub forbidden_stems: Vec<ForbiddenStem>,
    #[serde(default = "default_vocab_carve_outs")]
    pub carve_outs: Vec<VocabCarveOut>,
}

fn default_forbidden_stems() -> Vec<ForbiddenStem> {
    [
        ("foundry", "forbidden_foundry"),
        ("forgejo", "forbidden_forgejo"),
        ("jenkins", "forbidden_jenkins"),
        ("oya-vcs", "forbidden_oya-vcs"),
    ]
    .iter()
    .map(|(stem, code)| ForbiddenStem {
        stem: (*stem).to_owned(),
        code: (*code).to_owned(),
    })
    .collect()
}

fn default_vocab_carve_outs() -> Vec<VocabCarveOut> {
    [
        (
            VocabCarveOutKind::PathPrefix,
            "libs/oya-check-brand-residue/",
            "the deny-list patterns themselves are not residue",
        ),
        (
            VocabCarveOutKind::PathPrefix,
            "libs/oya-ci-config/",
            "the config-era deny-list SSOT (forbidden-stem table + bundled disposition) — naming a stem here is the deny-list, not residue (same rationale as oya-check-brand-residue)",
        ),
        (
            VocabCarveOutKind::PathExact,
            "oya-ci.toml",
            "the repo-root oya-ci config IS the deny-list (it declares the forbidden-stem table) — naming a stem here is the deny-list, not residue",
        ),
        (
            VocabCarveOutKind::PathExact,
            "registry/catalog/oya-check-brand-residue.yaml",
            "the catalog deny-list spec is not residue",
        ),
        (
            VocabCarveOutKind::PathPrefix,
            "oya/intelligence/_legacy-foundry/",
            "intentional historical archive of the dropped work",
        ),
        (
            VocabCarveOutKind::PathExact,
            "evidence/audit-chain.jsonl",
            "append-only audit chain — NEVER rewritten",
        ),
        (
            VocabCarveOutKind::PathSuffix,
            ".generated.json",
            "producer-generated faces record the tokens the gates track; a hand-edit is its own registry_drift RED",
        ),
        (
            VocabCarveOutKind::LineContainsCi,
            "palantir",
            "Palantir-Foundry is a competitor proper noun, not brand residue",
        ),
    ]
    .iter()
    .map(|(kind, value, reason)| VocabCarveOut {
        kind: *kind,
        value: (*value).to_owned(),
        reason: (*reason).to_owned(),
    })
    .collect()
}

impl Default for VocabConfig {
    fn default() -> Self {
        Self {
            forbidden_stems: default_forbidden_stems(),
            carve_outs: default_vocab_carve_outs(),
        }
    }
}

// ---------------------------------------------------------------------------
// [manifest]  (replaces the §2.5#7 ManifestFlags field-set, plan §2.3)
// ---------------------------------------------------------------------------

/// `[manifest]` — the rust-cargo per-crate Cargo.toml hygiene field-set (replaces the
/// hardcoded `ManifestFlags` requirement set, §2.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestConfig {
    #[serde(default = "default_required_flags")]
    pub required_flags: Vec<String>,
}

fn default_required_flags() -> Vec<String> {
    [
        "version_workspace",
        "rust_version_workspace",
        "publish_false",
        "license",
        "lints_workspace",
        "lib_doctest_false",
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect()
}

impl Default for ManifestConfig {
    fn default() -> Self {
        Self {
            required_flags: default_required_flags(),
        }
    }
}

// ---------------------------------------------------------------------------
// [reachability] / [justification] / [owners] / [enforcement]  (plan §2.3)
// ---------------------------------------------------------------------------

/// `[reachability]` — the registry sources a path can be reachable from (`resolve_reachability`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReachabilityConfig {
    #[serde(default = "default_masterplan")]
    pub masterplan: String,
    #[serde(default = "default_root_hub")]
    pub root_hub: String,
    #[serde(default = "default_doc_catalog")]
    pub doc_catalog: String,
}

fn default_masterplan() -> String {
    "specs/masterplan.json".to_owned()
}

fn default_root_hub() -> String {
    "specs/root-hub-pointers.json".to_owned()
}

fn default_doc_catalog() -> String {
    "docs/DOC-CATALOG.md".to_owned()
}

impl Default for ReachabilityConfig {
    fn default() -> Self {
        Self {
            masterplan: default_masterplan(),
            root_hub: default_root_hub(),
            doc_catalog: default_doc_catalog(),
        }
    }
}

/// `[justification]` — the ADR corpus dir + crosswalk specs (`resolve_justifications`,
/// `collect_crosswalk_inputs`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JustificationConfig {
    #[serde(default = "default_adr_dir")]
    pub adr_dir: String,
    #[serde(default = "default_roadmap")]
    pub roadmap: String,
}

fn default_adr_dir() -> String {
    "docs/decisions".to_owned()
}

fn default_roadmap() -> String {
    "specs/master-plan-sequencing.json".to_owned()
}

impl Default for JustificationConfig {
    fn default() -> Self {
        Self {
            adr_dir: default_adr_dir(),
            roadmap: default_roadmap(),
        }
    }
}

/// `[owners]` — the nearest-up-tree OWNERS marker file name (`resolve_owners`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnersConfig {
    #[serde(default = "default_owners_file")]
    pub file_name: String,
}

fn default_owners_file() -> String {
    "OWNERS".to_owned()
}

impl Default for OwnersConfig {
    fn default() -> Self {
        Self {
            file_name: default_owners_file(),
        }
    }
}

/// `[enforcement]` — the enforcement-surface sources (`collect_enforcement_inputs`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnforcementConfig {
    #[serde(default = "default_governance_substr")]
    pub governance_crate_substr: String,
    #[serde(default = "default_governance_lanes")]
    pub governance_lanes: Vec<String>,
}

fn default_governance_substr() -> String {
    "oya-governance".to_owned()
}

fn default_governance_lanes() -> Vec<String> {
    [
        "docs/governance-lanes/diataxis-doc-class.md",
        "docs/governance-lanes/prd-axis-coverage.md",
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect()
}

impl Default for EnforcementConfig {
    fn default() -> Self {
        Self {
            governance_crate_substr: default_governance_substr(),
            governance_lanes: default_governance_lanes(),
        }
    }
}

// ---------------------------------------------------------------------------
// [slo_coverage]  (portable input contract for cloud-ci-slo-coverage)
// ---------------------------------------------------------------------------

/// `[slo_coverage]` — declared catalog input globs for the SLO coverage gate.
///
/// The legacy dev-cli default was an implicit `registry/catalog` directory walk. The cloud-ci
/// product boundary makes that repo shape DATA instead: adopters keep the same pure gate engine
/// and point `catalog_record_globs` at their own catalog-row source. The producer expands these
/// globs against the declared tracked-path universe; the gate itself remains I/O-free.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SloCoverageConfig {
    #[serde(default = "default_slo_catalog_record_globs")]
    pub catalog_record_globs: Vec<String>,
}

fn default_slo_catalog_record_globs() -> Vec<String> {
    vec!["registry/catalog/*.yaml".to_owned()]
}

impl Default for SloCoverageConfig {
    fn default() -> Self {
        Self {
            catalog_record_globs: default_slo_catalog_record_globs(),
        }
    }
}

// ---------------------------------------------------------------------------
// [ttl] / [unit_class]  (subsumes the already-DATA JSON tables, plan §3.1)
// ---------------------------------------------------------------------------

/// `[ttl]` — the TTL budget table. Carried over as the existing JSON body (already DATA) so
/// the classification is bit-exact; `inline_json = None` ⇒ use the bundled JSON verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TtlConfig {
    /// An explicit inline override of the `ttl-policy.json` body (the full JSON document).
    /// `None` ⇒ the bundled default JSON (today's table). Authoring it inline is the
    /// "full-config does everything" path; absent ⇒ "zero-config does something useful".
    #[serde(default)]
    pub inline_json: Option<String>,
}

/// `[unit_class]` — the carve-out classification table, carried as the existing JSON body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct UnitClassConfig {
    /// An explicit inline override of the `unit-class-policy.json` body. `None` ⇒ bundled.
    #[serde(default)]
    pub inline_json: Option<String>,
}

// ---------------------------------------------------------------------------
// [gates]  (subsumes GATE_IDS + gate-disposition.json, plan §3.1 + §3.5)
// ---------------------------------------------------------------------------

/// The §3.5 gate INPUT-BINDING kind: how a gate's CURRENT keys are sourced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GateInputKind {
    /// Keys come from running the gate's pure `evaluate_keyed` over a producer-built face.
    ProducerFace,
    /// Keys arrive ALREADY GROUPED `code -> keys` from a raw-corpus collector (brand-residue).
    RawCorpusCollector,
    /// The gate contributes no CURRENT keys; its codes are stamped-empty by the disposition
    /// join (the `frozen_empty` codes). (Reserved KIND for a wholly meta gate; today every
    /// such code lives UNDER an existing gate via its disposition `frozen_empty: true`.)
    FrozenEmptyMeta,
}

/// Which producer face a `producer-face` gate binds (§3.5). The producer maps this to the
/// matching `GateInputs` field + `evaluate_keyed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateFace {
    TotalAccounting,
    CrossArtifact,
    AutomationRatchet,
    Staleness,
    BnfLayerSuffix,
    ManifestHygiene,
    CargoPrefix,
    SloCoverage,
    WorkspaceGlobCoverage,
}

/// One enabled gate: its id, its input KIND, and (for `producer-face`) which face it binds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateSpec {
    pub id: String,
    pub input_kind: GateInputKind,
    /// The bound face for `producer-face` gates; `None` for the other KINDs.
    #[serde(default)]
    pub face: Option<GateFace>,
}

/// `[gates]` — the enabled gate set (replaces `GATE_IDS`) + each gate's input KIND, plus the
/// per-(gate,code) disposition table (subsumes + relocates gate-disposition.json).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatesConfig {
    #[serde(default = "default_enabled_gates")]
    pub enabled: Vec<GateSpec>,
    /// The disposition table body (the gate-disposition.json document), carried verbatim so
    /// the per-code `mode`/`infra_prereq`/`frozen_empty` stamping is bit-exact. `None` ⇒
    /// the bundled default JSON (today's table).
    #[serde(default)]
    pub disposition_json: Option<String>,
}

const BUNDLED_DISPOSITION_JSON: &str = include_str!("bundled/gate-disposition.json");

impl GatesConfig {
    /// The disposition table body the producer joins against. Carried verbatim (DATA).
    pub fn disposition_json(&self) -> &str {
        match &self.disposition_json {
            Some(json) => json.as_str(),
            None => BUNDLED_DISPOSITION_JSON,
        }
    }
}

fn default_enabled_gates() -> Vec<GateSpec> {
    // The canonical enabled set, in GATE_IDS order (the on-disk baseline is BTreeMap-sorted on
    // serialization, so this order is for readability; the byte-output is order-independent).
    vec![
        GateSpec {
            id: "cloud-ci-total-accounting".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::TotalAccounting),
        },
        GateSpec {
            id: "cloud-ci-cross-artifact-agreement".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::CrossArtifact),
        },
        GateSpec {
            id: "cloud-ci-automation-ratchet".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::AutomationRatchet),
        },
        GateSpec {
            id: "cloud-ci-staleness-reaper".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::Staleness),
        },
        GateSpec {
            id: "cloud-ci-bnf-layer-suffix".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::BnfLayerSuffix),
        },
        GateSpec {
            id: "cloud-ci-manifest-hygiene".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::ManifestHygiene),
        },
        GateSpec {
            id: "cloud-ci-cargo-prefix".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::CargoPrefix),
        },
        GateSpec {
            id: "cloud-ci-slo-coverage".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::SloCoverage),
        },
        GateSpec {
            id: "cloud-ci-workspace-glob-coverage".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::WorkspaceGlobCoverage),
        },
        GateSpec {
            id: "cloud-ci-brand-residue".to_owned(),
            input_kind: GateInputKind::RawCorpusCollector,
            face: None,
        },
    ]
}

impl Default for GatesConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled_gates(),
            disposition_json: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_default_matches_todays_naming_consts() {
        let cfg = OyaCiConfig::bundled_default();
        assert_eq!(cfg.naming.required_prefix, "oya-");
        assert_eq!(cfg.naming.allowed_roles.len(), 13);
        assert!(cfg.naming.allowed_roles.contains(&"api".to_owned()));
        assert!(cfg.naming.allowed_roles.contains(&"usecase".to_owned()));
        assert!(!cfg.naming.allowed_roles.contains(&"runtime".to_owned()));
        assert_eq!(cfg.naming.check_family_prefix, "oya-check-");
        assert_eq!(cfg.naming.backend_suffixes.len(), 9);
        assert_eq!(
            cfg.naming.doctrinal_carve_outs,
            vec!["oya-tooling-agent-read".to_owned()]
        );
    }

    #[test]
    fn bundled_default_matches_todays_vocab_consts() {
        let cfg = OyaCiConfig::bundled_default();
        let codes: Vec<&str> = cfg
            .vocab
            .forbidden_stems
            .iter()
            .map(|s| s.code.as_str())
            .collect();
        assert_eq!(
            codes,
            vec![
                "forbidden_foundry",
                "forbidden_forgejo",
                "forbidden_jenkins",
                "forbidden_oya-vcs",
            ]
        );
        // 8 carve-out rules, including the line-level palantir exemption + the oya-ci-config
        // deny-list SSOT path carve-out + the repo-root oya-ci.toml deny-list carve-out.
        assert_eq!(cfg.vocab.carve_outs.len(), 8);
        assert!(cfg
            .vocab
            .carve_outs
            .iter()
            .any(|c| c.kind == VocabCarveOutKind::LineContainsCi && c.value == "palantir"));
    }

    #[test]
    fn bundled_default_carries_the_data_tables_verbatim() {
        let cfg = OyaCiConfig::bundled_default();
        // The unit-class + ttl + disposition tables parse as JSON and carry the canonical keys.
        let uc: serde_json::Value =
            serde_json::from_str(cfg.unit_class_policy_json()).expect("unit-class json");
        assert!(uc.get("rules").and_then(|r| r.as_array()).is_some());
        let ttl: serde_json::Value =
            serde_json::from_str(cfg.ttl_policy_json()).expect("ttl json");
        assert!(ttl.get("by_unit_class").is_some());
        let disp: serde_json::Value =
            serde_json::from_str(cfg.gates.disposition_json()).expect("disposition json");
        assert!(disp.get("gates").is_some());
        assert_eq!(
            cfg.slo_coverage.catalog_record_globs,
            vec!["registry/catalog/*.yaml".to_owned()]
        );
    }

    #[test]
    fn bundled_default_enables_all_ten_gates_with_input_kinds() {
        let cfg = OyaCiConfig::bundled_default();
        assert_eq!(cfg.gates.enabled.len(), 10);
        let brand = cfg
            .gates
            .enabled
            .iter()
            .find(|g| g.id == "cloud-ci-brand-residue")
            .expect("brand gate enabled");
        assert_eq!(brand.input_kind, GateInputKind::RawCorpusCollector);
        assert!(brand.face.is_none());
        let bnf = cfg
            .gates
            .enabled
            .iter()
            .find(|g| g.id == "cloud-ci-bnf-layer-suffix")
            .expect("bnf gate enabled");
        assert_eq!(bnf.input_kind, GateInputKind::ProducerFace);
        assert_eq!(bnf.face, Some(GateFace::BnfLayerSuffix));
        let cargo_prefix = cfg
            .gates
            .enabled
            .iter()
            .find(|g| g.id == "cloud-ci-cargo-prefix")
            .expect("cargo-prefix gate enabled");
        assert_eq!(cargo_prefix.input_kind, GateInputKind::ProducerFace);
        assert_eq!(cargo_prefix.face, Some(GateFace::CargoPrefix));
        let slo_coverage = cfg
            .gates
            .enabled
            .iter()
            .find(|g| g.id == "cloud-ci-slo-coverage")
            .expect("slo-coverage gate enabled");
        assert_eq!(slo_coverage.input_kind, GateInputKind::ProducerFace);
        assert_eq!(slo_coverage.face, Some(GateFace::SloCoverage));
        let workspace_glob_coverage = cfg
            .gates
            .enabled
            .iter()
            .find(|g| g.id == "cloud-ci-workspace-glob-coverage")
            .expect("workspace-glob-coverage gate enabled");
        assert_eq!(
            workspace_glob_coverage.input_kind,
            GateInputKind::ProducerFace
        );
        assert_eq!(
            workspace_glob_coverage.face,
            Some(GateFace::WorkspaceGlobCoverage)
        );
    }

    #[test]
    fn empty_toml_materializes_the_bundled_default() {
        let cfg = OyaCiConfig::from_toml_str("").expect("empty toml parses");
        assert_eq!(cfg, OyaCiConfig::bundled_default());
    }

    #[test]
    fn partial_toml_overrides_only_the_named_section() {
        // Only [naming] is given; every other section falls back to the bundled default.
        let toml = r#"
[naming]
required_prefix = "acme-"
allowed_roles = ["kernel", "app"]
check_family_prefix = "acme-check-"
backend_suffixes = ["aws"]
doctrinal_carve_outs = []
"#;
        let cfg = OyaCiConfig::from_toml_str(toml).expect("partial toml parses");
        assert_eq!(cfg.naming.required_prefix, "acme-");
        assert_eq!(cfg.naming.allowed_roles, vec!["kernel", "app"]);
        // unspecified sections are the bundled default
        assert_eq!(cfg.vocab, VocabConfig::default());
        assert_eq!(cfg.gates, GatesConfig::default());
    }

    #[test]
    fn closed_schema_rejects_unknown_top_level_key() {
        let err = OyaCiConfig::from_toml_str("bogus_section = 1").unwrap_err();
        assert!(matches!(err, ConfigError::Parse(_)), "got {err:?}");
    }

    #[test]
    fn closed_schema_rejects_unknown_nested_key() {
        let toml = "[naming]\nrequired_prefix = \"oya-\"\nbogus_field = true\n";
        let err = OyaCiConfig::from_toml_str(toml).unwrap_err();
        assert!(matches!(err, ConfigError::Parse(_)), "got {err:?}");
    }

    #[test]
    fn hyphenated_code_round_trips_through_the_closed_schema() {
        // MF-7: the `forbidden_oya-vcs` code (a hyphenated slug) must survive a parse round-trip
        // unchanged, both as a default and when authored explicitly in TOML.
        let toml = r#"
[[vocab.forbidden_stems]]
stem = "oya-vcs"
code = "forbidden_oya-vcs"
"#;
        let cfg = OyaCiConfig::from_toml_str(toml).expect("hyphenated-code toml parses");
        assert_eq!(cfg.vocab.forbidden_stems.len(), 1);
        assert_eq!(cfg.vocab.forbidden_stems[0].code, "forbidden_oya-vcs");
        // And it is present in the bundled default verbatim.
        let cfg2 = OyaCiConfig::bundled_default();
        assert!(cfg2
            .vocab
            .forbidden_stems
            .iter()
            .any(|s| s.code == "forbidden_oya-vcs"));
    }

    #[test]
    fn gate_input_kind_round_trips_kebab_case() {
        let toml = r#"
[[gates.enabled]]
id = "cloud-ci-brand-residue"
input_kind = "raw-corpus-collector"

[[gates.enabled]]
id = "cloud-ci-total-accounting"
input_kind = "producer-face"
face = "total_accounting"
"#;
        let cfg = OyaCiConfig::from_toml_str(toml).expect("gate toml parses");
        assert_eq!(cfg.gates.enabled.len(), 2);
        assert_eq!(
            cfg.gates.enabled[0].input_kind,
            GateInputKind::RawCorpusCollector
        );
        assert_eq!(
            cfg.gates.enabled[1].input_kind,
            GateInputKind::ProducerFace
        );
        assert_eq!(cfg.gates.enabled[1].face, Some(GateFace::TotalAccounting));
    }

    #[test]
    fn config_serializes_back_to_toml() {
        // Round-trip: bundled default -> TOML -> parse == bundled default (closed-schema stable).
        let cfg = OyaCiConfig::bundled_default();
        let text = toml::to_string(&cfg).expect("serialize");
        let reparsed = OyaCiConfig::from_toml_str(&text).expect("reparse");
        assert_eq!(reparsed, cfg);
    }
}
