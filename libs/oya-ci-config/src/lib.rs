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
//! //! - `[reachability]` / `[justification]` / `[owners]` / `[enforcement]` source paths
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

/// The closed-schema version (ADR-0533 §Decision item 5): a published `$id`/`$schema` + a
/// `schema_version` so the closed schema can evolve without silently breaking adopters. Bumped
/// when a breaking schema change ships; additive (back-compatible) keys do NOT bump it.
pub const SCHEMA_VERSION: u32 = 2;

/// The published `$id` URL for the closed `oya-ci.toml` schema (ADR-0533 §Decision item 5).
pub const SCHEMA_ID: &str = "https://oya-ci.dev/schema/oya-ci-config/v2";

/// The JSON-Schema dialect the published schema is authored against (ADR-0533 item 5).
pub const SCHEMA_DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

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
// The DE-BRAND PROFILE (ADR-0532 amends ADR-0017; ADR-0533; ADR-0562 §9)
// ---------------------------------------------------------------------------

/// The de-brand PROFILE that selects the bundled section defaults (ADR-0533 §Decision item 1).
///
/// ADR-0527 made the floor config-driven, but the verified trap is that "zero-config does NOT
/// mean policy-free; it means OYATIE-policy" — the bundled defaults ARE oyatie's brand
/// deny-list + layout. The profile resolves that trap: `Oyatie` reproduces today's values
/// VERBATIM (first-party self-host, ZERO behaviour change — the safety property), while
/// `Neutral` is policy-FREE (empty `forbidden_stems`, no `required_prefix`, generic `.git`
/// root marker, no `governance_lanes`, gates present-but-quiet, ZERO oyatie path literals) so
/// an external adopter inherits NO oyatie policy by default.
///
/// `Oyatie` is the `#[default]`, so a profile-less `oya-ci.toml` (and the compiled-in
/// `bundled_default()`) resolve to today's behaviour unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    /// Policy-FREE public boundary: zero oyatie literals (ADR-0533 §Decision item 1).
    Neutral,
    /// Today's oyatie values verbatim — the first-party self-host profile (the safety property).
    #[default]
    Oyatie,
}

impl Profile {
    /// True iff this is the (default) `Oyatie` profile. Used to SKIP serializing the `profile`
    /// key when it is the default, so today's `oya-ci.toml` (which carries no `profile` key)
    /// round-trips to the byte-IDENTICAL canonical TOML — keeping `digest()` (and thus every
    /// face's `_provenance` config digest) byte-stable for first-party (zero behaviour change).
    fn is_oyatie(&self) -> bool {
        matches!(self, Profile::Oyatie)
    }
}

// ---------------------------------------------------------------------------
// Top-level config
// ---------------------------------------------------------------------------

/// The full, validated oya-ci policy. Parsed from `oya-ci.toml`, or materialized from the
/// bundled default when no file is present.
///
/// Deserialization routes through [`OyaCiConfigShadow`] (every section `Option`) so the
/// top-level `profile` key is the SOLE selector of the section defaults: a `profile='neutral'`
/// file with no other sections materializes [`OyaCiConfig::neutral`], not the oyatie defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OyaCiConfig {
    /// The active de-brand profile (ADR-0533). Skipped from serialization when `Oyatie` (the
    /// default), so first-party output is byte-identical to today (the `digest()` invariant).
    #[serde(default, skip_serializing_if = "Profile::is_oyatie")]
    pub profile: Profile,
    pub repo: RepoConfig,
    pub naming: NamingConfig,
    pub vocab: VocabConfig,
    pub reachability: ReachabilityConfig,
    pub justification: JustificationConfig,
    pub owners: OwnersConfig,
    pub enforcement: EnforcementConfig,
    /// `[ttl]` — the TTL budget table (carried as bundled JSON DATA; the section exists for
    /// inline overrides only).
    pub ttl: TtlConfig,
    pub unit_class: UnitClassConfig,
    pub gates: GatesConfig,
    /// `[output]` — producer output layout (ADR-0533 §Decision item 2). Faces directory.
    /// Skipped from serialization when it equals the default so first-party canonical TOML (and
    /// thus `digest()`) is byte-identical to today — this section did not exist before ADR-0533.
    #[serde(default, skip_serializing_if = "OutputConfig::is_default")]
    pub output: OutputConfig,
    /// `[cross_artifact]` — the crosswalk source paths (ADR-0533 §Decision item 3). Skipped from
    /// serialization when default, same byte-identity rationale as `[output]`.
    #[serde(default, skip_serializing_if = "CrossArtifactConfig::is_default")]
    pub cross_artifact: CrossArtifactConfig,
}

/// The closed-schema DESERIALIZATION shadow (ADR-0533): every section is `Option` so an
/// OMITTED table is distinguishable from one authored with default values. After parse, the
/// top-level `profile`/`extends` selects the base ([`OyaCiConfig::oyatie`] /
/// [`OyaCiConfig::neutral`]) and each `Some(section)` overlays it. `deny_unknown_fields` lives
/// here (the input gate); each inner section keeps its OWN `deny_unknown_fields` so a bogus
/// nested key still errors inside its `Option`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OyaCiConfigShadow {
    #[serde(default)]
    profile: Profile,
    /// Optional alias for `profile` (ADR-0533 §Decision item 1: "a top-level `profile` ... (or
    /// `extends`) key"). When present it WINS over `profile` (it names the base to extend).
    #[serde(default)]
    extends: Option<Profile>,
    #[serde(default)]
    repo: Option<RepoConfig>,
    #[serde(default)]
    naming: Option<NamingConfig>,
    #[serde(default)]
    vocab: Option<VocabConfig>,
    #[serde(default)]
    reachability: Option<ReachabilityConfig>,
    #[serde(default)]
    justification: Option<JustificationConfig>,
    #[serde(default)]
    owners: Option<OwnersConfig>,
    #[serde(default)]
    enforcement: Option<EnforcementConfig>,
    #[serde(default)]
    ttl: Option<TtlConfig>,
    #[serde(default)]
    unit_class: Option<UnitClassConfig>,
    #[serde(default)]
    gates: Option<GatesConfig>,
    #[serde(default)]
    output: Option<OutputConfig>,
    #[serde(default)]
    cross_artifact: Option<CrossArtifactConfig>,
}

impl Default for OyaCiConfig {
    fn default() -> Self {
        Self::bundled_default()
    }
}

impl OyaCiConfig {
    /// Materialize oyatie's CURRENT policy as the bundled default (no file required). This is
    /// the byte-for-byte equivalent of today's hardcoded `const`s + embedded JSON. Alias for
    /// [`OyaCiConfig::oyatie`] — preserved so every existing caller is unchanged (ADR-0533).
    pub fn bundled_default() -> Self {
        Self::oyatie()
    }

    /// The `oyatie` profile: today's values VERBATIM (ADR-0533 §Decision item 1). This is the
    /// first-party self-host profile and the SAFETY PROPERTY — it reproduces the pre-profile
    /// `bundled_default()` byte-for-byte, so first-party gate outputs do not move.
    pub fn oyatie() -> Self {
        Self {
            profile: Profile::Oyatie,
            repo: RepoConfig::default(),
            naming: NamingConfig::default(),
            vocab: VocabConfig::default(),
            reachability: ReachabilityConfig::default(),
            justification: JustificationConfig::default(),
            owners: OwnersConfig::default(),
            enforcement: EnforcementConfig::default(),
            ttl: TtlConfig::default(),
            unit_class: UnitClassConfig::default(),
            gates: GatesConfig::default(),
            output: OutputConfig::default(),
            cross_artifact: CrossArtifactConfig::default(),
        }
    }

    /// The `neutral` profile: a policy-FREE public boundary with ZERO oyatie path literals
    /// (ADR-0533 §Decision item 1). Empty `forbidden_stems`, no `required_prefix`, generic
    /// `.git` root marker, no `governance_lanes`, gates present-but-quiet (the gate set is
    /// enabled so the engine dispatches, but the deny-lists/dispositions are empty). An
    /// external adopter starting from `profile='neutral'` inherits NO oyatie policy.
    pub fn neutral() -> Self {
        Self {
            profile: Profile::Neutral,
            repo: RepoConfig::neutral(),
            naming: NamingConfig::neutral(),
            vocab: VocabConfig::neutral(),
            reachability: ReachabilityConfig::neutral(),
            justification: JustificationConfig::neutral(),
            owners: OwnersConfig::neutral(),
            enforcement: EnforcementConfig::neutral(),
            ttl: TtlConfig::neutral(),
            unit_class: UnitClassConfig::neutral(),
            gates: GatesConfig::neutral(),
            output: OutputConfig::default(),
            cross_artifact: CrossArtifactConfig::neutral(),
        }
    }

    /// Parse + closed-schema-validate an `oya-ci.toml`. The top-level `profile` (or `extends`)
    /// key selects the base defaults ([`oyatie`](Self::oyatie) when absent — zero behaviour
    /// change for first-party — or [`neutral`](Self::neutral)); each section AUTHORED in the
    /// file overlays that base, and an OMITTED section keeps the base value. Unknown keys error.
    pub fn from_toml_str(text: &str) -> Result<Self, ConfigError> {
        Self::from_toml_str_with_line_scope_mode(text, LegacyLineScope::Reject)
    }

    /// Parse a v1 frozen-reference config while preserving its historical line-rule semantics.
    ///
    /// Schema v1 `line_contains_ci` rows had no `exempt_stems` field and skipped the entire
    /// matching line. Merge-base regeneration must reproduce that historical policy even after
    /// the candidate binary upgrades to schema v2; otherwise the current parser cannot regenerate
    /// an older frozen reference. Empty line-rule scopes are therefore expanded to every forbidden
    /// stem only on this explicitly named compatibility path. Candidate config loading continues
    /// through [`Self::from_toml_str`] and rejects the same input fail-closed.
    pub fn from_frozen_reference_toml_str(text: &str) -> Result<Self, ConfigError> {
        Self::from_toml_str_with_line_scope_mode(text, LegacyLineScope::ExpandToAllStems)
    }

    /// The absent-config fallback for a v1 frozen reference. This is intentionally separate from
    /// [`Self::bundled_default`]: historical line rules skipped every matching line, while the v2
    /// bundled default exempts only the explicitly named stems.
    pub fn frozen_reference_bundled_default() -> Self {
        let mut config = Self::bundled_default();
        config.vocab.set_all_line_scopes_to_all_stems();
        config
    }

    fn from_toml_str_with_line_scope_mode(
        text: &str,
        line_scope_mode: LegacyLineScope,
    ) -> Result<Self, ConfigError> {
        let shadow: OyaCiConfigShadow =
            toml::from_str(text).map_err(|e| ConfigError::Parse(e.to_string()))?;
        // `extends` (the explicit base-to-extend) wins over `profile` when both are present.
        let profile = shadow.extends.unwrap_or(shadow.profile);
        let mut base = match profile {
            Profile::Oyatie => Self::oyatie(),
            Profile::Neutral => Self::neutral(),
        };
        base.profile = profile;
        if let Some(v) = shadow.repo {
            base.repo = v;
        }
        if let Some(v) = shadow.naming {
            base.naming = v;
        }
        if let Some(v) = shadow.vocab {
            base.vocab = v;
        }
        if let Some(v) = shadow.reachability {
            base.reachability = v;
        }
        if let Some(v) = shadow.justification {
            base.justification = v;
        }
        if let Some(v) = shadow.owners {
            base.owners = v;
        }
        if let Some(v) = shadow.enforcement {
            base.enforcement = v;
        }
        if let Some(v) = shadow.ttl {
            base.ttl = v;
        }
        if let Some(v) = shadow.unit_class {
            base.unit_class = v;
        }
        if let Some(v) = shadow.gates {
            base.gates = v;
        }
        if let Some(v) = shadow.output {
            base.output = v;
        }
        if let Some(v) = shadow.cross_artifact {
            base.cross_artifact = v;
        }
        if line_scope_mode == LegacyLineScope::ExpandToAllStems {
            base.vocab.expand_legacy_line_scopes();
        }
        base.vocab.validate()?;
        Ok(base)
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

    /// The closed-schema version this config was authored against (ADR-0533 item 5).
    pub fn schema_version(&self) -> u32 {
        SCHEMA_VERSION
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

impl RepoConfig {
    /// The neutral profile's `[repo]`: a generic `.git` root marker (ADR-0533 item 1 — "generic
    /// root_markers defaulting to `.git`") and no path exclusions (zero oyatie path literals).
    fn neutral() -> Self {
        Self {
            root_markers: vec![".git".to_owned()],
            path_excludes: Vec::new(),
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
    // ADR-0565: `graphql` removed from the canonical role/layer vocabulary
    // (13 → 12). Kept byte-for-byte aligned with `oya-ci.toml [naming]` and
    // `oya_governance_predictable_naming_kernel::ALLOWED_ROLES`.
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
    vec![
        "oya-tooling-agent-read".to_owned(),
        "oya-ci-gate-contract".to_owned(),
    ]
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

impl NamingConfig {
    /// The neutral profile's `[naming]`: NO `required_prefix` (so `MissingOyaPrefix` is never
    /// raised, ADR-0533 item 1) and empty role/family/backend/carve-out tables — zero oyatie
    /// brand literals. The naming kernel sources `required_prefix` from here; an empty prefix
    /// means every crate name "starts with" it, so the de-brand is complete.
    fn neutral() -> Self {
        Self {
            required_prefix: String::new(),
            allowed_roles: Vec::new(),
            check_family_prefix: String::new(),
            backend_suffixes: Vec::new(),
            doctrinal_carve_outs: Vec::new(),
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
    /// Stems a line-level rule may exempt. Empty for path rules; required and validated for
    /// `line_contains_ci` so one structural/proper-noun marker cannot suppress other stems.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exempt_stems: Vec<String>,
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
    let rows: &[(VocabCarveOutKind, &str, &[&str], &str)] = &[
        (
            VocabCarveOutKind::PathPrefix,
            "libs/oya-check-brand-residue/",
            &[],
            "the deny-list patterns themselves are not residue",
        ),
        (
            VocabCarveOutKind::PathPrefix,
            "libs/oya-ci-config/",
            &[],
            "the config-era deny-list SSOT (forbidden-stem table + bundled disposition) — naming a stem here is the deny-list, not residue (same rationale as oya-check-brand-residue)",
        ),
        (
            VocabCarveOutKind::PathExact,
            "oya-ci.toml",
            &[],
            "the repo-root oya-ci config IS the deny-list (it declares the forbidden-stem table) — naming a stem here is the deny-list, not residue",
        ),
        (
            VocabCarveOutKind::PathExact,
            "registry/catalog/oya-check-brand-residue.yaml",
            &[],
            "the catalog deny-list spec is not residue",
        ),
        (
            VocabCarveOutKind::PathPrefix,
            "oya/intelligence/_legacy-foundry/",
            &[],
            "intentional historical archive of the dropped work",
        ),
        (
            VocabCarveOutKind::PathPrefix,
            "marketplace/facade/dev-cli/tests/",
            &[],
            "integration test fixtures that reference live repo contracts/openapi/foundry/ paths and fixture data strings — structural references to real contract paths, not brand residue; moved from oya/developer-sdk/crates/oya-dev-cli/tests/ where it was already baselined",
        ),
        (
            VocabCarveOutKind::PathExact,
            "evidence/audit-chain.jsonl",
            &[],
            "append-only audit chain — NEVER rewritten",
        ),
        (
            VocabCarveOutKind::PathSuffix,
            ".generated.json",
            &[],
            "producer-generated faces record the tokens the gates track; a hand-edit is its own ci_inventory_registry_drift RED",
        ),
        (
            VocabCarveOutKind::LineContainsCi,
            "palantir",
            &["foundry"],
            "Palantir-Foundry is a competitor proper noun, not brand residue",
        ),
    ];
    rows.iter()
        .map(|(kind, value, exempt_stems, reason)| VocabCarveOut {
            kind: *kind,
            value: (*value).to_owned(),
            exempt_stems: exempt_stems.iter().map(|stem| (*stem).to_owned()).collect(),
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

impl VocabConfig {
    fn all_stems(&self) -> Vec<String> {
        self.forbidden_stems
            .iter()
            .map(|stem| stem.stem.clone())
            .collect()
    }

    fn expand_legacy_line_scopes(&mut self) {
        let all_stems = self.all_stems();
        for carve_out in &mut self.carve_outs {
            if carve_out.kind == VocabCarveOutKind::LineContainsCi
                && carve_out.exempt_stems.is_empty()
            {
                carve_out.exempt_stems.clone_from(&all_stems);
            }
        }
    }

    fn set_all_line_scopes_to_all_stems(&mut self) {
        let all_stems = self.all_stems();
        for carve_out in &mut self.carve_outs {
            if carve_out.kind == VocabCarveOutKind::LineContainsCi {
                carve_out.exempt_stems.clone_from(&all_stems);
            }
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        for carve_out in &self.carve_outs {
            match carve_out.kind {
                VocabCarveOutKind::LineContainsCi => {
                    if carve_out.exempt_stems.is_empty() {
                        return Err(ConfigError::Parse(format!(
                            "vocab line_contains_ci carve-out {:?} must declare at least one exempt_stems entry",
                            carve_out.value
                        )));
                    }
                }
                VocabCarveOutKind::PathPrefix
                | VocabCarveOutKind::PathExact
                | VocabCarveOutKind::PathSuffix => {
                    if !carve_out.exempt_stems.is_empty() {
                        return Err(ConfigError::Parse(format!(
                            "vocab path carve-out {:?} must not declare exempt_stems",
                            carve_out.value
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    /// The neutral profile's `[vocab]`: an EMPTY forbidden-stem list (ADR-0533 item 1) — the
    /// brand-residue census finds nothing, so a neutral repo carries no oyatie deny-list. No
    /// carve-outs either (carve-outs only exist to exempt the deny-list, which is empty).
    fn neutral() -> Self {
        Self {
            forbidden_stems: Vec::new(),
            carve_outs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LegacyLineScope {
    Reject,
    ExpandToAllStems,
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
    /// The reviewed prefix-registration registry (ADR-0555): explicit `{prefix, anchor}`
    /// entries that register whole trees (dir prefixes ending `/`) or exact paths as
    /// reached, each naming WHY. Registration is a review-visible design act (the
    /// ADR-0551 trust class — same as ratchet-policy.json), NOT an exemption table.
    /// Absent file ⇒ empty registry (zero-config); malformed file ⇒ hard producer error
    /// (fail-loud).
    #[serde(default = "default_reachability_registry")]
    pub registry: String,
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

fn default_reachability_registry() -> String {
    "specs/reachability-registry.json".to_owned()
}

impl Default for ReachabilityConfig {
    fn default() -> Self {
        Self {
            masterplan: default_masterplan(),
            root_hub: default_root_hub(),
            doc_catalog: default_doc_catalog(),
            registry: default_reachability_registry(),
        }
    }
}

impl ReachabilityConfig {
    /// The neutral profile's `[reachability]`: EMPTY source paths (zero oyatie path literals,
    /// ADR-0533 item 1). An adopter declares their own reachability sources; absent ⇒ the
    /// producer treats reachability as an empty registry (zero-config, no oyatie `specs/` leak).
    fn neutral() -> Self {
        Self {
            masterplan: String::new(),
            root_hub: String::new(),
            doc_catalog: String::new(),
            registry: String::new(),
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

impl JustificationConfig {
    /// The neutral profile's `[justification]`: EMPTY ADR/roadmap source paths (zero oyatie
    /// `docs/`/`specs/` literals, ADR-0533 item 1). An adopter declares their own corpus.
    fn neutral() -> Self {
        Self {
            adr_dir: String::new(),
            roadmap: String::new(),
        }
    }
}

/// `[owners]` — the nearest-up-tree OWNERS marker file name (`resolve_owners`) plus the
/// ADR-0555 hardening policy (FRIC-1781400000): the per-file breadth bound that stops a
/// single OWNERS registration from bulk-neutering a tree's unowned accounting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnersConfig {
    #[serde(default = "default_owners_file")]
    pub file_name: String,
    /// The maximum number of tracked paths a SINGLE OWNERS file may cover via
    /// nearest-ancestor resolution (the OWNERS file itself counts as one covered path).
    /// Coverage beyond the bound stays UNOWNED and the producer names the exact fix:
    /// split the registration into narrower subtree OWNERS files. Policy-as-data
    /// (reviewed TOML, same trust class as ratchet-policy.json); the default is sized
    /// from the live-corpus distribution — max legitimate coverage 886 paths
    /// (`registry/catalog/OWNERS`) of 18,400 tracked at measurement (2026-06-12), so
    /// 2000 gives >2x headroom for legitimate tree growth while catching bulk
    /// registrations (a root/`cloud/`-level OWNERS would claim thousands).
    /// `NonZeroU64` by construction: a zero bound (which would neuter ALL ownership) is
    /// rejected by the closed-schema loader, fail-loud.
    #[serde(default = "default_max_paths_per_owners_file")]
    pub max_paths_per_owners_file: std::num::NonZeroU64,
}

fn default_owners_file() -> String {
    "OWNERS".to_owned()
}

fn default_max_paths_per_owners_file() -> std::num::NonZeroU64 {
    std::num::NonZeroU64::new(2000).expect("2000 is non-zero")
}

impl Default for OwnersConfig {
    fn default() -> Self {
        Self {
            file_name: default_owners_file(),
            max_paths_per_owners_file: default_max_paths_per_owners_file(),
        }
    }
}

impl OwnersConfig {
    /// The neutral profile's `[owners]`: the generic `OWNERS` marker (NOT an oyatie literal —
    /// `OWNERS` is the cross-ecosystem convention) and the measured breadth bound. The bound is
    /// `NonZeroU64` by construction, so neutral cannot zero it (ADR-0533; ADR-0555 hardening).
    fn neutral() -> Self {
        Self {
            file_name: default_owners_file(),
            max_paths_per_owners_file: default_max_paths_per_owners_file(),
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

impl EnforcementConfig {
    /// The neutral profile's `[enforcement]`: an EMPTY governance-crate substring (the default
    /// `oya-governance` is a brand literal) and NO `governance_lanes` (ADR-0533 item 1 — "no
    /// governance_lanes"; zero oyatie path literals). The enforcement gate is present-but-quiet.
    fn neutral() -> Self {
        Self {
            governance_crate_substr: String::new(),
            governance_lanes: Vec::new(),
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

impl TtlConfig {
    /// The neutral profile's `[ttl]`: an inline EMPTY-but-schema-valid table (`{"by_unit_class":
    /// {}}`) rather than `None`, because `None` would silently inherit the BUNDLED oyatie TTL
    /// JSON (ADR-0533 item 1 — zero oyatie policy leak). An adopter authors their own budgets.
    fn neutral() -> Self {
        Self {
            inline_json: Some("{\"by_unit_class\":{}}".to_owned()),
        }
    }
}

/// `[unit_class]` — the carve-out classification table, carried as the existing JSON body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct UnitClassConfig {
    /// An explicit inline override of the `unit-class-policy.json` body. `None` ⇒ bundled.
    #[serde(default)]
    pub inline_json: Option<String>,
}

impl UnitClassConfig {
    /// The neutral profile's `[unit_class]`: an inline EMPTY-but-schema-valid table
    /// (`{"rules": []}`) rather than `None`, because `None` would silently inherit the BUNDLED
    /// oyatie classification JSON (ADR-0533 item 1 — zero oyatie policy leak).
    fn neutral() -> Self {
        Self {
            inline_json: Some("{\"rules\":[]}".to_owned()),
        }
    }
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
    CrossArtifact,
    AutomationRatchet,
    BnfLayerSuffix,
    CargoPrefix,
    SloCoverage,
    LicensePolicy,
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
            id: "cloud-ci-bnf-layer-suffix".to_owned(),
            input_kind: GateInputKind::ProducerFace,
            face: Some(GateFace::BnfLayerSuffix),
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

impl GatesConfig {
    /// The neutral profile's `[gates]`: the SAME enabled gate set (so the portable engine still
    /// dispatches every gate KIND — "gates present", ADR-0533 item 1) but an EMPTY disposition
    /// table (`{"gates":{}}`) instead of `None`, because `None` would inherit the BUNDLED oyatie
    /// disposition JSON (a policy leak). "Present-but-quiet": the gates run, the dispositions are
    /// empty, so no oyatie-specific code disposition is stamped.
    fn neutral() -> Self {
        Self {
            enabled: default_enabled_gates(),
            disposition_json: Some("{\"gates\":{}}".to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// [output]  (ADR-0533 §Decision item 2 — faces_dir replaces the hardcoded literal)
// ---------------------------------------------------------------------------

/// `[output]` — the producer output layout (ADR-0533 item 2). `faces_dir` replaces the
/// hardcoded `.oya-ci/faces/`-class literal at the producer's `main.rs`, so the faces location
/// is DATA. The default keeps oyatie's location, so first-party output is unchanged; an adopter
/// overrides it without forking the producer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputConfig {
    /// Directory the producer writes the generated faces into. Default = oyatie's current
    /// location (the producer's `out_dir` literal) so first-party byte-output is unchanged.
    #[serde(default = "default_faces_dir")]
    pub faces_dir: String,
}

fn default_faces_dir() -> String {
    "ci/facade/artifact-inventory-registry".to_owned()
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            faces_dir: default_faces_dir(),
        }
    }
}

impl OutputConfig {
    /// True iff this is the default `[output]` — used to SKIP serialization so first-party
    /// canonical TOML (and `digest()`) is byte-identical to the pre-ADR-0533 output.
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

// ---------------------------------------------------------------------------
// [cross_artifact]  (ADR-0533 §Decision item 3 — sources replace compiled-in artifacts)
// ---------------------------------------------------------------------------

/// `[cross_artifact]` — the crosswalk SOURCE paths (ADR-0533 item 3). Replaces the compiled-in
/// oyatie artifacts the producer feeds the cross-artifact-agreement gate. The default keeps
/// oyatie's source set (so first-party is unchanged); a neutral adopter declares their own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrossArtifactConfig {
    /// The generated-face / corpus source paths the crosswalk agreement is computed over.
    #[serde(default = "default_cross_artifact_sources")]
    pub sources: Vec<String>,
}

fn default_cross_artifact_sources() -> Vec<String> {
    [
        "specs/masterplan.json",
        "specs/root-hub-pointers.json",
        "specs/master-plan-sequencing.json",
        "docs/decisions",
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect()
}

impl Default for CrossArtifactConfig {
    fn default() -> Self {
        Self {
            sources: default_cross_artifact_sources(),
        }
    }
}

impl CrossArtifactConfig {
    /// The neutral profile's `[cross_artifact]`: EMPTY sources (the defaults are oyatie
    /// `specs/`/`docs/` path literals; ADR-0533 item 1, item 3). An adopter declares their own.
    fn neutral() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// True iff this is the default `[cross_artifact]` — used to SKIP serialization so
    /// first-party canonical TOML (and `digest()`) is byte-identical to the pre-ADR-0533 output.
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_default_matches_todays_naming_consts() {
        let cfg = OyaCiConfig::bundled_default();
        assert_eq!(cfg.naming.required_prefix, "oya-");
        assert_eq!(cfg.naming.allowed_roles.len(), 12);
        assert!(cfg.naming.allowed_roles.contains(&"api".to_owned()));
        assert!(cfg.naming.allowed_roles.contains(&"usecase".to_owned()));
        assert!(!cfg.naming.allowed_roles.contains(&"runtime".to_owned()));
        // ADR-0565: graphql is de-blessed from the canonical vocabulary.
        assert!(!cfg.naming.allowed_roles.contains(&"graphql".to_owned()));
        assert_eq!(cfg.naming.check_family_prefix, "oya-check-");
        assert_eq!(cfg.naming.backend_suffixes.len(), 9);
        assert_eq!(
            cfg.naming.doctrinal_carve_outs,
            vec![
                "oya-tooling-agent-read".to_owned(),
                "oya-ci-gate-contract".to_owned()
            ]
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
        // 9 carve-out rules, including the line-level palantir exemption + the oya-ci-config
        // deny-list SSOT path carve-out + the repo-root oya-ci.toml deny-list carve-out.
        assert_eq!(cfg.vocab.carve_outs.len(), 9);
        assert!(
            cfg.vocab
                .carve_outs
                .iter()
                .any(|c| c.kind == VocabCarveOutKind::LineContainsCi
                    && c.value == "palantir"
                    && c.exempt_stems == ["foundry"])
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
    }

    #[test]
    fn bundled_default_enables_all_eleven_gates_with_input_kinds() {
        let cfg = OyaCiConfig::bundled_default();
        assert_eq!(cfg.gates.enabled.len(), 11);
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
        let cross_artifact = cfg
            .gates
            .enabled
            .iter()
            .find(|g| g.id == "cloud-ci-cross-artifact-agreement")
            .expect("cross-artifact gate enabled");
        assert_eq!(cross_artifact.input_kind, GateInputKind::ProducerFace);
        assert_eq!(cross_artifact.face, Some(GateFace::CrossArtifact));
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
        // The phantom-citation lane (FRIC-1781430000) is born-blocking frozen-empty: the
        // pre-existing inventory is producer-side shrink-only DATA, never a baseline.
        let phantom = disp
            .get("gates")
            .and_then(|gates| gates.get("cloud-ci-cross-artifact-agreement"))
            .and_then(|codes| codes.get("phantom_decision_citation"))
            .expect("phantom_decision_citation disposition");
        assert_eq!(
            phantom.get("mode").and_then(serde_json::Value::as_str),
            Some("baseline-block-on-new")
        );
        assert_eq!(
            phantom
                .get("frozen_empty")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert!(
            phantom
                .get("remediation")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|text| text.contains("--next-adr")),
            "the phantom remediation must name the allocator"
        );
        let liveness_codes = disp
            .get("gates")
            .and_then(|gates| gates.get("cloud-ci-enforcement-liveness"))
            .and_then(serde_json::Value::as_object)
            .expect("enforcement-liveness disposition");
        for code in [
            "malformed_enforcement_liveness_face",
            "malformed_enforcement_liveness_row",
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
            if code.starts_with("malformed_enforcement_liveness") {
                assert!(
                    disposition
                        .get("remediation")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|text| text.contains("producer output")
                            && text.contains("governance PR")),
                    "malformed enforcement-liveness dispositions must carry actionable remediation"
                );
            }
        }
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

    /// ADR-0555 hardening (FRIC-1781400000): the OWNERS breadth bound is policy-as-data
    /// with a measured default (2000, >2x the live-corpus max of 886), overridable in
    /// TOML, and a ZERO bound — which would neuter all ownership — is structurally
    /// impossible (NonZeroU64 rejected by the closed-schema loader, fail-loud).
    #[test]
    fn owners_breadth_bound_defaults_overrides_and_rejects_zero() {
        let cfg = OyaCiConfig::bundled_default();
        assert_eq!(cfg.owners.max_paths_per_owners_file.get(), 2000);

        let cfg = OyaCiConfig::from_toml_str("[owners]\nmax_paths_per_owners_file = 50\n")
            .expect("explicit bound parses");
        assert_eq!(cfg.owners.max_paths_per_owners_file.get(), 50);
        // an unspecified bound falls back to the measured default
        assert_eq!(
            OyaCiConfig::from_toml_str("[owners]\nfile_name = \"OWNERS\"\n")
                .expect("partial owners section parses")
                .owners
                .max_paths_per_owners_file
                .get(),
            2000
        );

        let err =
            OyaCiConfig::from_toml_str("[owners]\nmax_paths_per_owners_file = 0\n").unwrap_err();
        assert!(matches!(err, ConfigError::Parse(_)), "got {err:?}");
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
    fn line_carve_out_stem_scope_round_trips_through_the_closed_schema() {
        let text = r#"
[[vocab.carve_outs]]
kind = "line_contains_ci"
value = "//contracts/openapi/foundry:capability-v1.yaml"
exempt_stems = ["foundry"]
reason = "structural reference"
"#;
        let cfg = OyaCiConfig::from_toml_str(text).expect("scoped line carve-out parses");
        assert_eq!(cfg.vocab.carve_outs[0].exempt_stems, vec!["foundry"]);

        let serialized = toml::to_string(&cfg).expect("serialize scoped carve-out");
        assert!(serialized.contains("exempt_stems = [\"foundry\"]"));
        assert_eq!(
            OyaCiConfig::from_toml_str(&serialized).expect("reparse scoped carve-out"),
            cfg
        );
    }

    #[test]
    fn line_carve_out_requires_explicit_stem_scope() {
        for text in [
            r#"
[[vocab.carve_outs]]
kind = "line_contains_ci"
value = "palantir"
"#,
            r#"
[[vocab.carve_outs]]
kind = "path_exact"
value = "docs/example.md"
exempt_stems = ["foundry"]
"#,
        ] {
            let err = OyaCiConfig::from_toml_str(text).unwrap_err();
            assert!(matches!(err, ConfigError::Parse(_)), "got {err:?}");
        }
    }

    #[test]
    fn frozen_reference_v1_line_scope_expands_to_all_configured_stems() {
        let text = r#"
[[vocab.forbidden_stems]]
stem = "alpha"
code = "forbidden_alpha"

[[vocab.forbidden_stems]]
stem = "beta"
code = "forbidden_beta"

[[vocab.carve_outs]]
kind = "line_contains_ci"
value = "legacy-marker"
"#;

        assert!(
            OyaCiConfig::from_toml_str(text).is_err(),
            "candidate configs must declare an explicit v2 line scope"
        );
        let frozen = OyaCiConfig::from_frozen_reference_toml_str(text)
            .expect("v1 frozen-reference config migrates in memory");
        assert_eq!(
            frozen.vocab.carve_outs[0].exempt_stems,
            vec!["alpha", "beta"]
        );
    }

    #[test]
    fn frozen_reference_compatibility_preserves_explicit_v2_scope() {
        let text = r#"
[[vocab.forbidden_stems]]
stem = "alpha"
code = "forbidden_alpha"

[[vocab.forbidden_stems]]
stem = "beta"
code = "forbidden_beta"

[[vocab.carve_outs]]
kind = "line_contains_ci"
value = "scoped-marker"
exempt_stems = ["alpha"]
"#;

        let frozen = OyaCiConfig::from_frozen_reference_toml_str(text)
            .expect("v2 frozen-reference config remains valid");
        assert_eq!(frozen.vocab.carve_outs[0].exempt_stems, vec!["alpha"]);
    }

    #[test]
    fn absent_v1_frozen_reference_uses_legacy_all_stem_line_scope() {
        let frozen = OyaCiConfig::frozen_reference_bundled_default();
        let expected: Vec<String> = frozen
            .vocab
            .forbidden_stems
            .iter()
            .map(|stem| stem.stem.clone())
            .collect();
        let palantir = frozen
            .vocab
            .carve_outs
            .iter()
            .find(|rule| rule.value == "palantir")
            .expect("bundled legacy line rule");
        assert_eq!(palantir.exempt_stems, expected);
    }

    #[test]
    fn gate_input_kind_round_trips_kebab_case() {
        let toml = r#"
[[gates.enabled]]
id = "cloud-ci-brand-residue"
input_kind = "raw-corpus-collector"

[[gates.enabled]]
id = "cloud-ci-cross-artifact-agreement"
input_kind = "producer-face"
face = "cross_artifact"
"#;
        let cfg = OyaCiConfig::from_toml_str(toml).expect("gate toml parses");
        assert_eq!(cfg.gates.enabled.len(), 2);
        assert_eq!(
            cfg.gates.enabled[0].input_kind,
            GateInputKind::RawCorpusCollector
        );
        assert_eq!(cfg.gates.enabled[1].input_kind, GateInputKind::ProducerFace);
        assert_eq!(cfg.gates.enabled[1].face, Some(GateFace::CrossArtifact));
    }

    #[test]
    fn config_serializes_back_to_toml() {
        // Round-trip: bundled default -> TOML -> parse == bundled default (closed-schema stable).
        let cfg = OyaCiConfig::bundled_default();
        let text = toml::to_string(&cfg).expect("serialize");
        let reparsed = OyaCiConfig::from_toml_str(&text).expect("reparse");
        assert_eq!(reparsed, cfg);
    }

    // -----------------------------------------------------------------------
    // ADR-0532 (amends ADR-0017) + ADR-0533 + ADR-0562 §9: the de-brand PROFILE.
    // -----------------------------------------------------------------------

    /// SAFETY PROPERTY: `bundled_default()` is exactly the `oyatie()` profile, and `oyatie()` is
    /// the (default) profile a profile-less config resolves to — so first-party is unchanged.
    #[test]
    fn bundled_default_is_the_oyatie_profile_verbatim() {
        assert_eq!(OyaCiConfig::bundled_default(), OyaCiConfig::oyatie());
        assert_eq!(OyaCiConfig::oyatie().profile, Profile::Oyatie);
        // a profile-less config IS the oyatie profile (default profile = oyatie).
        let cfg = OyaCiConfig::from_toml_str("").expect("empty toml parses");
        assert_eq!(cfg.profile, Profile::Oyatie);
        assert_eq!(cfg, OyaCiConfig::oyatie());
    }

    /// SAFETY PROPERTY: serializing the oyatie profile produces BYTE-IDENTICAL canonical TOML to
    /// today (no `profile`/`[output]`/`[cross_artifact]` keys leak in), so `digest()` — stamped
    /// into every face's `_provenance` — does not move. `skip_serializing_if` guards this.
    #[test]
    fn oyatie_profile_serialization_is_byte_identical_to_pre_profile() {
        let oyatie = OyaCiConfig::oyatie();
        let text = toml::to_string(&oyatie).expect("serialize");
        // The de-brand additions must NOT appear in the first-party canonical serialization.
        assert!(!text.contains("profile"), "profile leaked:\n{text}");
        assert!(!text.contains("[output]"), "[output] leaked:\n{text}");
        assert!(
            !text.contains("[cross_artifact]"),
            "[cross_artifact] leaked:\n{text}"
        );
        // And it still round-trips.
        assert_eq!(OyaCiConfig::from_toml_str(&text).expect("reparse"), oyatie);
    }

    /// The `neutral()` profile is policy-FREE: empty deny-list, no required prefix, generic
    /// `.git` root marker, no governance lanes, present-but-quiet gates (ADR-0533 item 1).
    #[test]
    fn neutral_profile_is_policy_free() {
        let n = OyaCiConfig::neutral();
        assert_eq!(n.profile, Profile::Neutral);
        assert!(
            n.vocab.forbidden_stems.is_empty(),
            "neutral has no deny-list"
        );
        assert!(n.vocab.carve_outs.is_empty());
        assert_eq!(
            n.naming.required_prefix, "",
            "neutral has no required prefix"
        );
        assert!(n.naming.allowed_roles.is_empty());
        assert_eq!(n.repo.root_markers, vec![".git".to_owned()]);
        assert!(n.repo.path_excludes.is_empty());
        assert!(n.enforcement.governance_lanes.is_empty());
        assert_eq!(n.enforcement.governance_crate_substr, "");

        assert!(n.cross_artifact.sources.is_empty());
        // gates present (engine still dispatches) but disposition is empty (quiet).
        assert_eq!(n.gates.enabled.len(), 11, "gates present");
        let disp: serde_json::Value =
            serde_json::from_str(n.gates.disposition_json()).expect("neutral disposition json");
        assert_eq!(
            disp.get("gates")
                .and_then(serde_json::Value::as_object)
                .map(|m| m.len()),
            Some(0),
            "neutral disposition is quiet (empty)"
        );
        // ttl/unit_class do NOT silently inherit the oyatie bundled JSON.
        let uc: serde_json::Value =
            serde_json::from_str(n.unit_class_policy_json()).expect("neutral unit-class json");
        assert_eq!(
            uc.get("rules")
                .and_then(serde_json::Value::as_array)
                .map(Vec::len),
            Some(0)
        );
    }

    /// THE DE-BRAND PROOF: the neutral profile emits ZERO oyatie/oya- brand literals across the
    /// whole serialized config (no `oya-` prefix, no `oyatie`, no `forbidden_foundry`, no oyatie
    /// `specs/`/`docs/`/`cloud/` path literals). This is the capability ADR-0533 adds.
    #[test]
    fn neutral_profile_serializes_zero_brand_literals() {
        let text = toml::to_string(&OyaCiConfig::neutral()).expect("serialize neutral");
        for needle in [
            "oya-",
            "oyatie",
            "oya-governance",
            "oya-check-",
            "forbidden_foundry",
            "registry/catalog",
            "specs/masterplan.json",
            "docs/decisions",
            "oya-cloud-ci",
        ] {
            assert!(
                !text.contains(needle),
                "neutral config leaked brand literal {needle:?}:\n{text}"
            );
        }
        // It still round-trips through the closed schema.
        assert_eq!(
            OyaCiConfig::from_toml_str(&text).expect("reparse neutral"),
            OyaCiConfig::neutral()
        );
    }

    /// `profile = 'neutral'` with NO other sections materializes the neutral defaults — the
    /// top-level profile key is the SOLE selector (the serde-default trap ADR-0533 resolves).
    #[test]
    fn profile_neutral_key_alone_materializes_neutral_defaults() {
        let cfg = OyaCiConfig::from_toml_str("profile = 'neutral'\n").expect("parses");
        assert_eq!(cfg, OyaCiConfig::neutral());
        // explicit oyatie key materializes oyatie.
        let cfg = OyaCiConfig::from_toml_str("profile = 'oyatie'\n").expect("parses");
        assert_eq!(cfg, OyaCiConfig::oyatie());
    }

    /// A neutral config may still AUTHOR a section, which overlays the neutral base (the public
    /// adoption path: start neutral, declare only your own policy).
    #[test]
    fn neutral_base_overlays_authored_sections() {
        let toml = "profile = 'neutral'\n[naming]\nrequired_prefix = \"acme-\"\nallowed_roles = [\"kernel\"]\ncheck_family_prefix = \"\"\nbackend_suffixes = []\ndoctrinal_carve_outs = []\n";
        let cfg = OyaCiConfig::from_toml_str(toml).expect("parses");
        assert_eq!(cfg.profile, Profile::Neutral);
        assert_eq!(cfg.naming.required_prefix, "acme-");
        // the unspecified sections stay at the NEUTRAL base, not oyatie.
        assert!(cfg.vocab.forbidden_stems.is_empty());
        assert_eq!(cfg.repo.root_markers, vec![".git".to_owned()]);
    }

    /// `extends` is the explicit alias for the base profile and wins over `profile` when both
    /// are present (ADR-0533 item 1: "a top-level `profile` ... (or `extends`) key").
    #[test]
    fn extends_selects_and_overrides_profile() {
        let cfg = OyaCiConfig::from_toml_str("extends = 'neutral'\n").expect("parses");
        assert_eq!(cfg, OyaCiConfig::neutral());
        // extends wins over a conflicting profile key.
        let cfg = OyaCiConfig::from_toml_str("profile = 'oyatie'\nextends = 'neutral'\n")
            .expect("parses");
        assert_eq!(cfg.profile, Profile::Neutral);
    }

    /// The closed schema still rejects an unknown top-level key AND the de-brand additions are
    /// the only new accepted keys (`profile`, `extends`, `[output]`, `[cross_artifact]`).
    #[test]
    fn closed_schema_still_rejects_unknown_key_after_profile_addition() {
        assert!(matches!(
            OyaCiConfig::from_toml_str("bogus = 1").unwrap_err(),
            ConfigError::Parse(_)
        ));
        // an unknown profile value is rejected (closed enum).
        assert!(matches!(
            OyaCiConfig::from_toml_str("profile = 'acme'").unwrap_err(),
            ConfigError::Parse(_)
        ));
    }

    /// The new `[output]`/`[cross_artifact]` sections parse + carry oyatie defaults under the
    /// oyatie profile (faces_dir + crosswalk sources are now DATA, ADR-0533 items 2 + 3).
    #[test]
    fn output_and_cross_artifact_sections_default_under_oyatie() {
        let cfg = OyaCiConfig::oyatie();
        assert_eq!(
            cfg.output.faces_dir,
            "ci/facade/artifact-inventory-registry"
        );
        assert!(
            cfg.cross_artifact
                .sources
                .contains(&"specs/masterplan.json".to_owned())
        );
        // explicitly authoring them overlays.
        let toml =
            "[output]\nfaces_dir = \".oya-ci/faces\"\n[cross_artifact]\nsources = [\"x.json\"]\n";
        let cfg = OyaCiConfig::from_toml_str(toml).expect("parses");
        assert_eq!(cfg.output.faces_dir, ".oya-ci/faces");
        assert_eq!(cfg.cross_artifact.sources, vec!["x.json".to_owned()]);
    }

    /// The published closed-schema version + `$id`/`$schema` are exposed (ADR-0533 item 5).
    #[test]
    fn schema_version_and_id_are_published() {
        assert_eq!(OyaCiConfig::oyatie().schema_version(), SCHEMA_VERSION);
        assert_eq!(SCHEMA_VERSION, 2);
        assert!(SCHEMA_ID.ends_with("/v2"));
        assert!(SCHEMA_ID.starts_with("https://"));
        assert!(SCHEMA_DIALECT.contains("json-schema.org"));
    }
}
