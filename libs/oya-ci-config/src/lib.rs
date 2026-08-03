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
pub const OYA_CI_CONFIG_SCHEMA_VERSION: u32 = 1;
const NEUTRAL_UNIT_CLASS_JSON: &str = r#"{"rules":[]}"#;
const NEUTRAL_TTL_JSON: &str = r#"{"by_unit_class":{}}"#;
const NEUTRAL_DISPOSITION_JSON: &str =
    r#"{"_provenance":{"disposition_schema_version":1},"gates":{}}"#;

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

/// The named public-boundary profile a config extends (ADR-0533).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigProfile {
    /// Policy-free public defaults: no Oyatie prefix, path, vocab, or gate-disposition literals.
    Neutral,
    /// Oyatie's self-host policy, preserving the pre-public-boundary bundled defaults verbatim.
    Oyatie,
}

/// The full, validated oya-ci policy. Parsed from `oya-ci.toml`, or materialized from a
/// selected profile plus any closed-schema section overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OyaCiConfig {
    pub schema_version: u32,
    pub profile: ConfigProfile,
    pub repo: RepoConfig,
    pub naming: NamingConfig,
    pub vocab: VocabConfig,
    pub manifest: ManifestConfig,
    pub reachability: ReachabilityConfig,
    pub justification: JustificationConfig,
    pub owners: OwnersConfig,
    pub enforcement: EnforcementConfig,
    pub slo_coverage: SloCoverageConfig,
    pub ttl: TtlConfig,
    pub unit_class: UnitClassConfig,
    pub gates: GatesConfig,
}

impl<'de> Deserialize<'de> for OyaCiConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(OyaCiConfigPatch::deserialize(deserializer)?.materialize())
    }
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
        Self::oyatie()
    }

    /// Materialize Oyatie's self-host policy profile. This preserves the pre-ADR-0533 bundled
    /// defaults and remains the compatibility default for omitted `profile` in existing configs.
    pub fn oyatie() -> Self {
        Self {
            schema_version: OYA_CI_CONFIG_SCHEMA_VERSION,
            profile: ConfigProfile::Oyatie,
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

    /// Materialize the policy-free public profile. It is intentionally quiet and carries no
    /// Oyatie path, prefix, forbidden-vocab, governance-lane, or bundled disposition literals;
    /// adopters add their repo-specific policy as DATA in `oya-ci.toml`.
    pub fn neutral() -> Self {
        Self {
            schema_version: OYA_CI_CONFIG_SCHEMA_VERSION,
            profile: ConfigProfile::Neutral,
            repo: RepoConfig {
                root_markers: vec![".git".to_owned()],
                path_excludes: Vec::new(),
            },
            naming: NamingConfig {
                required_prefix: String::new(),
                allowed_roles: Vec::new(),
                check_family_prefix: String::new(),
                backend_suffixes: Vec::new(),
                doctrinal_carve_outs: Vec::new(),
            },
            vocab: VocabConfig {
                forbidden_stems: Vec::new(),
                carve_outs: Vec::new(),
            },
            manifest: ManifestConfig {
                required_flags: Vec::new(),
            },
            reachability: ReachabilityConfig {
                masterplan: String::new(),
                root_hub: String::new(),
                doc_catalog: String::new(),
            },
            justification: JustificationConfig {
                adr_dir: String::new(),
                roadmap: String::new(),
            },
            owners: OwnersConfig {
                file_name: default_owners_file(),
            },
            enforcement: EnforcementConfig {
                governance_crate_substr: String::new(),
                governance_lanes: Vec::new(),
            },
            slo_coverage: SloCoverageConfig {
                catalog_record_globs: Vec::new(),
            },
            ttl: TtlConfig {
                inline_json: Some(NEUTRAL_TTL_JSON.to_owned()),
            },
            unit_class: UnitClassConfig {
                inline_json: Some(NEUTRAL_UNIT_CLASS_JSON.to_owned()),
            },
            gates: GatesConfig {
                enabled: Vec::new(),
                disposition_json: Some(NEUTRAL_DISPOSITION_JSON.to_owned()),
            },
        }
    }

    /// Parse + closed-schema-validate an `oya-ci.toml`. Absent sections fall back to the
    /// selected profile (`oyatie` when omitted, preserving existing configs). Unknown keys error.
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

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct OyaCiConfigPatch {
    schema_version: Option<u32>,
    profile: Option<ConfigProfile>,
    repo: Option<RepoConfigPatch>,
    naming: Option<NamingConfigPatch>,
    vocab: Option<VocabConfigPatch>,
    manifest: Option<ManifestConfigPatch>,
    reachability: Option<ReachabilityConfigPatch>,
    justification: Option<JustificationConfigPatch>,
    owners: Option<OwnersConfigPatch>,
    enforcement: Option<EnforcementConfigPatch>,
    slo_coverage: Option<SloCoverageConfigPatch>,
    ttl: Option<TtlConfigPatch>,
    unit_class: Option<UnitClassConfigPatch>,
    gates: Option<GatesConfigPatch>,
}

impl OyaCiConfigPatch {
    fn materialize(self) -> OyaCiConfig {
        let profile = self.profile.unwrap_or(ConfigProfile::Oyatie);
        let mut cfg = match profile {
            ConfigProfile::Neutral => OyaCiConfig::neutral(),
            ConfigProfile::Oyatie => OyaCiConfig::oyatie(),
        };

        if let Some(schema_version) = self.schema_version {
            cfg.schema_version = schema_version;
        }
        cfg.profile = profile;
        if let Some(repo) = self.repo {
            repo.apply(&mut cfg.repo);
        }
        if let Some(naming) = self.naming {
            naming.apply(&mut cfg.naming);
        }
        if let Some(vocab) = self.vocab {
            vocab.apply(&mut cfg.vocab);
        }
        if let Some(manifest) = self.manifest {
            manifest.apply(&mut cfg.manifest);
        }
        if let Some(reachability) = self.reachability {
            reachability.apply(&mut cfg.reachability);
        }
        if let Some(justification) = self.justification {
            justification.apply(&mut cfg.justification);
        }
        if let Some(owners) = self.owners {
            owners.apply(&mut cfg.owners);
        }
        if let Some(enforcement) = self.enforcement {
            enforcement.apply(&mut cfg.enforcement);
        }
        if let Some(slo_coverage) = self.slo_coverage {
            slo_coverage.apply(&mut cfg.slo_coverage);
        }
        if let Some(ttl) = self.ttl {
            ttl.apply(&mut cfg.ttl);
        }
        if let Some(unit_class) = self.unit_class {
            unit_class.apply(&mut cfg.unit_class);
        }
        if let Some(gates) = self.gates {
            gates.apply(&mut cfg.gates);
        }
        cfg
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RepoConfigPatch {
    root_markers: Option<Vec<String>>,
    path_excludes: Option<Vec<String>>,
}

impl RepoConfigPatch {
    fn apply(self, cfg: &mut RepoConfig) {
        if let Some(root_markers) = self.root_markers {
            cfg.root_markers = root_markers;
        }
        if let Some(path_excludes) = self.path_excludes {
            cfg.path_excludes = path_excludes;
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct NamingConfigPatch {
    required_prefix: Option<String>,
    allowed_roles: Option<Vec<String>>,
    check_family_prefix: Option<String>,
    backend_suffixes: Option<Vec<String>>,
    doctrinal_carve_outs: Option<Vec<String>>,
}

impl NamingConfigPatch {
    fn apply(self, cfg: &mut NamingConfig) {
        if let Some(required_prefix) = self.required_prefix {
            cfg.required_prefix = required_prefix;
        }
        if let Some(allowed_roles) = self.allowed_roles {
            cfg.allowed_roles = allowed_roles;
        }
        if let Some(check_family_prefix) = self.check_family_prefix {
            cfg.check_family_prefix = check_family_prefix;
        }
        if let Some(backend_suffixes) = self.backend_suffixes {
            cfg.backend_suffixes = backend_suffixes;
        }
        if let Some(doctrinal_carve_outs) = self.doctrinal_carve_outs {
            cfg.doctrinal_carve_outs = doctrinal_carve_outs;
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct VocabConfigPatch {
    forbidden_stems: Option<Vec<ForbiddenStem>>,
    carve_outs: Option<Vec<VocabCarveOut>>,
}

impl VocabConfigPatch {
    fn apply(self, cfg: &mut VocabConfig) {
        if let Some(forbidden_stems) = self.forbidden_stems {
            cfg.forbidden_stems = forbidden_stems;
        }
        if let Some(carve_outs) = self.carve_outs {
            cfg.carve_outs = carve_outs;
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestConfigPatch {
    required_flags: Option<Vec<String>>,
}

impl ManifestConfigPatch {
    fn apply(self, cfg: &mut ManifestConfig) {
        if let Some(required_flags) = self.required_flags {
            cfg.required_flags = required_flags;
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReachabilityConfigPatch {
    masterplan: Option<String>,
    root_hub: Option<String>,
    doc_catalog: Option<String>,
}

impl ReachabilityConfigPatch {
    fn apply(self, cfg: &mut ReachabilityConfig) {
        if let Some(masterplan) = self.masterplan {
            cfg.masterplan = masterplan;
        }
        if let Some(root_hub) = self.root_hub {
            cfg.root_hub = root_hub;
        }
        if let Some(doc_catalog) = self.doc_catalog {
            cfg.doc_catalog = doc_catalog;
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct JustificationConfigPatch {
    adr_dir: Option<String>,
    roadmap: Option<String>,
}

impl JustificationConfigPatch {
    fn apply(self, cfg: &mut JustificationConfig) {
        if let Some(adr_dir) = self.adr_dir {
            cfg.adr_dir = adr_dir;
        }
        if let Some(roadmap) = self.roadmap {
            cfg.roadmap = roadmap;
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnersConfigPatch {
    file_name: Option<String>,
}

impl OwnersConfigPatch {
    fn apply(self, cfg: &mut OwnersConfig) {
        if let Some(file_name) = self.file_name {
            cfg.file_name = file_name;
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnforcementConfigPatch {
    governance_crate_substr: Option<String>,
    governance_lanes: Option<Vec<String>>,
}

impl EnforcementConfigPatch {
    fn apply(self, cfg: &mut EnforcementConfig) {
        if let Some(governance_crate_substr) = self.governance_crate_substr {
            cfg.governance_crate_substr = governance_crate_substr;
        }
        if let Some(governance_lanes) = self.governance_lanes {
            cfg.governance_lanes = governance_lanes;
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct SloCoverageConfigPatch {
    catalog_record_globs: Option<Vec<String>>,
}

impl SloCoverageConfigPatch {
    fn apply(self, cfg: &mut SloCoverageConfig) {
        if let Some(catalog_record_globs) = self.catalog_record_globs {
            cfg.catalog_record_globs = catalog_record_globs;
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct TtlConfigPatch {
    inline_json: Option<String>,
}

impl TtlConfigPatch {
    fn apply(self, cfg: &mut TtlConfig) {
        if let Some(inline_json) = self.inline_json {
            cfg.inline_json = Some(inline_json);
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitClassConfigPatch {
    inline_json: Option<String>,
}

impl UnitClassConfigPatch {
    fn apply(self, cfg: &mut UnitClassConfig) {
        if let Some(inline_json) = self.inline_json {
            cfg.inline_json = Some(inline_json);
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatesConfigPatch {
    enabled: Option<Vec<GateSpec>>,
    disposition_json: Option<String>,
}

impl GatesConfigPatch {
    fn apply(self, cfg: &mut GatesConfig) {
        if let Some(enabled) = self.enabled {
            cfg.enabled = enabled;
        }
        if let Some(disposition_json) = self.disposition_json {
            cfg.disposition_json = Some(disposition_json);
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
    LicensePolicy,
    ZeroStaticSecrets,
    LoadBalancerInventory,
    MultiRegionDisposition,
    SovereignTenantPin,
    TenantEnvironmentTier,
    WorkspaceGlobCoverage,
    TargetParity,
    EnforcementLiveness,
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
            id: "cloud-ci-license-policy".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::LicensePolicy),
        },
        GateSpec {
            id: "cloud-ci-zero-static-secrets".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::ZeroStaticSecrets),
        },
        GateSpec {
            id: "cloud-ci-load-balancer-inventory".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::LoadBalancerInventory),
        },
        GateSpec {
            id: "cloud-ci-multi-region-disposition".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::MultiRegionDisposition),
        },
        GateSpec {
            id: "cloud-ci-sovereign-tenant-pin".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::SovereignTenantPin),
        },
        GateSpec {
            id: "cloud-ci-tenant-environment-tier".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::TenantEnvironmentTier),
        },
        GateSpec {
            id: "cloud-ci-workspace-glob-coverage".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::WorkspaceGlobCoverage),
        },
        GateSpec {
            id: "cloud-ci-target-parity".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::TargetParity),
        },
        GateSpec {
            id: "cloud-ci-enforcement-liveness".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::EnforcementLiveness),
        },
        GateSpec {
            id: "cloud-ci-freshness".to_owned(),
            input_kind: GateInputKind::FrozenEmptyMeta,
            face: None,
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
        assert!(
            cfg.vocab
                .carve_outs
                .iter()
                .any(|c| c.kind == VocabCarveOutKind::LineContainsCi && c.value == "palantir")
        );
    }

    #[test]
    fn bundled_default_carries_the_data_tables_verbatim() {
        let cfg = OyaCiConfig::bundled_default();
        // The unit-class + ttl + disposition tables parse as JSON and carry the canonical keys.
        let uc: serde_json::Value =
            serde_json::from_str(cfg.unit_class_policy_json()).expect("unit-class json");
        assert!(uc.get("rules").and_then(|r| r.as_array()).is_some());
        let ttl: serde_json::Value = serde_json::from_str(cfg.ttl_policy_json()).expect("ttl json");
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
    fn bundled_default_enables_all_configured_gates_with_input_kinds() {
        let cfg = OyaCiConfig::bundled_default();
        assert_eq!(cfg.gates.enabled.len(), 19);
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
        let license_policy = cfg
            .gates
            .enabled
            .iter()
            .find(|g| g.id == "cloud-ci-license-policy")
            .expect("license-policy gate enabled");
        assert_eq!(license_policy.input_kind, GateInputKind::ProducerFace);
        assert_eq!(license_policy.face, Some(GateFace::LicensePolicy));
        let zero_static_secrets = cfg
            .gates
            .enabled
            .iter()
            .find(|g| g.id == "cloud-ci-zero-static-secrets")
            .expect("zero-static-secrets gate enabled");
        assert_eq!(zero_static_secrets.input_kind, GateInputKind::ProducerFace);
        assert_eq!(zero_static_secrets.face, Some(GateFace::ZeroStaticSecrets));
        let multi_region_disposition = cfg
            .gates
            .enabled
            .iter()
            .find(|g| g.id == "cloud-ci-multi-region-disposition")
            .expect("multi-region-disposition gate enabled");
        assert_eq!(
            multi_region_disposition.input_kind,
            GateInputKind::ProducerFace
        );
        assert_eq!(
            multi_region_disposition.face,
            Some(GateFace::MultiRegionDisposition)
        );
        let sovereign_tenant_pin = cfg
            .gates
            .enabled
            .iter()
            .find(|g| g.id == "cloud-ci-sovereign-tenant-pin")
            .expect("sovereign-tenant-pin gate enabled");
        assert_eq!(sovereign_tenant_pin.input_kind, GateInputKind::ProducerFace);
        assert_eq!(
            sovereign_tenant_pin.face,
            Some(GateFace::SovereignTenantPin)
        );
        let tenant_environment_tier = cfg
            .gates
            .enabled
            .iter()
            .find(|g| g.id == "cloud-ci-tenant-environment-tier")
            .expect("tenant-environment-tier gate enabled");
        assert_eq!(
            tenant_environment_tier.input_kind,
            GateInputKind::ProducerFace
        );
        assert_eq!(
            tenant_environment_tier.face,
            Some(GateFace::TenantEnvironmentTier)
        );
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
        let target_parity = cfg
            .gates
            .enabled
            .iter()
            .find(|g| g.id == "cloud-ci-target-parity")
            .expect("target-parity gate enabled");
        assert_eq!(target_parity.input_kind, GateInputKind::ProducerFace);
        assert_eq!(target_parity.face, Some(GateFace::TargetParity));
        let enforcement_liveness = cfg
            .gates
            .enabled
            .iter()
            .find(|g| g.id == "cloud-ci-enforcement-liveness")
            .expect("enforcement-liveness gate enabled");
        assert_eq!(enforcement_liveness.input_kind, GateInputKind::ProducerFace);
        assert_eq!(
            enforcement_liveness.face,
            Some(GateFace::EnforcementLiveness)
        );
        let freshness = cfg
            .gates
            .enabled
            .iter()
            .find(|g| g.id == "cloud-ci-freshness")
            .expect("freshness gate enabled");
        assert_eq!(freshness.input_kind, GateInputKind::FrozenEmptyMeta);
        assert!(freshness.face.is_none());

        let disp: serde_json::Value =
            serde_json::from_str(cfg.gates.disposition_json()).expect("disposition json");
        let freshness_codes = disp
            .get("gates")
            .and_then(|gates| gates.get("cloud-ci-freshness"))
            .and_then(serde_json::Value::as_object)
            .expect("freshness disposition");
        for code in [
            "lock_missing_member_package",
            "lock_stale_member_version",
            "lock_orphan_path_package",
            "generated_face_stale",
        ] {
            let disposition = freshness_codes
                .get(code)
                .expect("freshness code disposition");
            assert_eq!(
                disposition.get("mode").and_then(serde_json::Value::as_str),
                Some("baseline-block-on-new")
            );
            assert_eq!(
                disposition
                    .get("frozen_empty")
                    .and_then(serde_json::Value::as_bool),
                Some(true)
            );
        }
        let liveness_codes = disp
            .get("gates")
            .and_then(|gates| gates.get("cloud-ci-enforcement-liveness"))
            .and_then(serde_json::Value::as_object)
            .expect("enforcement-liveness disposition");
        for code in [
            "hook_unwired_without_stub_marker",
            "hook_wiring_mirror_drift",
            "wired_hook_missing_file",
        ] {
            let disposition = liveness_codes
                .get(code)
                .expect("enforcement-liveness code disposition");
            assert_eq!(
                disposition.get("mode").and_then(serde_json::Value::as_str),
                Some("baseline-block-on-new")
            );
            assert_eq!(
                disposition
                    .get("frozen_empty")
                    .and_then(serde_json::Value::as_bool),
                Some(true)
            );
        }
    }

    #[test]
    fn empty_toml_materializes_the_bundled_default() {
        let cfg = OyaCiConfig::from_toml_str("").expect("empty toml parses");
        assert_eq!(cfg, OyaCiConfig::bundled_default());
    }

    #[test]
    fn explicit_oyatie_profile_materializes_the_self_host_policy() {
        let cfg = OyaCiConfig::from_toml_str(
            r#"
schema_version = 1
profile = "oyatie"
"#,
        )
        .expect("oyatie profile parses");

        assert_eq!(cfg.schema_version, 1);
        assert_eq!(cfg.profile, ConfigProfile::Oyatie);
        assert_eq!(cfg, OyaCiConfig::oyatie());
    }

    #[test]
    fn neutral_profile_adopter_fixture_has_no_oyatie_path_assumptions() {
        let cfg =
            OyaCiConfig::from_toml_str(include_str!("../fixtures/neutral-adopter-oya-ci.toml"))
                .expect("neutral adopter fixture parses");

        assert_eq!(cfg.schema_version, 1);
        assert_eq!(cfg.profile, ConfigProfile::Neutral);
        assert_eq!(cfg.repo.root_markers, vec![".git".to_owned()]);
        assert_eq!(cfg.repo.path_excludes, vec!["target/".to_owned()]);
        assert!(cfg.naming.required_prefix.is_empty());
        assert!(cfg.vocab.forbidden_stems.is_empty());
        assert!(cfg.vocab.carve_outs.is_empty());
        assert!(cfg.enforcement.governance_crate_substr.is_empty());
        assert!(cfg.enforcement.governance_lanes.is_empty());
        assert_eq!(
            cfg.slo_coverage.catalog_record_globs,
            vec!["catalog/*.yaml".to_owned()]
        );
        assert_eq!(cfg.gates.enabled.len(), 1);
        assert_eq!(cfg.gates.enabled[0].id, "cloud-ci-brand-residue");
        assert_eq!(
            cfg.gates.enabled[0].input_kind,
            GateInputKind::RawCorpusCollector
        );
        assert!(cfg.gates.enabled[0].face.is_none());
        serde_json::from_str::<serde_json::Value>(cfg.unit_class_policy_json())
            .expect("neutral unit-class json parses");
        serde_json::from_str::<serde_json::Value>(cfg.ttl_policy_json())
            .expect("neutral ttl json parses");
        serde_json::from_str::<serde_json::Value>(cfg.gates.disposition_json())
            .expect("neutral disposition json parses");

        let serialized = toml::to_string(&cfg).expect("neutral serializes");
        for oyatie_literal in [
            "oya-",
            "specs/root-hub-pointers.json",
            "docs/decisions",
            "docs/governance-lanes",
            "registry/catalog",
        ] {
            assert!(
                !serialized.contains(oyatie_literal),
                "neutral fixture leaked {oyatie_literal}:\n{serialized}"
            );
        }
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
        assert!(
            cfg2.vocab
                .forbidden_stems
                .iter()
                .any(|s| s.code == "forbidden_oya-vcs")
        );
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
        assert_eq!(cfg.gates.enabled[1].input_kind, GateInputKind::ProducerFace);
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
