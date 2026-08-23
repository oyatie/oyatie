//! Regional-pack manifest schema and local gate checks.
//!
//! ADR-0010 is planning context for the `packs/<code>/` pack-authoring root,
//! while accepted ADR-0064 requires canonical-base neutrality and pack
//! isolation. This module is deliberately fixture-scoped: it parses one
//! declared regional-pack manifest and evaluates the two local gates against
//! caller-supplied source text, without claiming repo-wide production
//! enforcement.
//!
//! The API is path-parameterized: callers own I/O and pass the manifest
//! contents (or a path they choose) plus the source text to scan. Nothing here
//! reaches for repo-root files, so tests load crate-local fixtures via
//! `include_str!` and stay hermetic under buck2's sandbox.

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;

pub const REGIONAL_PACK_MANIFEST_SCHEMA_VERSION: u16 = 1;
const CANONICAL_BASE_NEUTRALITY_GATE: &str = "canonical-base-neutrality";
const CROSS_PACK_REFUSAL_GATE: &str = "cross-pack-refusal";
const ACCEPTED_AUTHORITY_ADR: &str = "ADR-0064";
const PLANNING_CONTEXT_ADR: &str = "ADR-0010";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RegionalPackManifest {
    pub manifest_schema_version: u16,
    pub pack: RegionalPackManifestIdentity,
    pub source_authority: RegionalPackManifestAuthority,
    pub regional_pack: RegionalPackManifestJurisdiction,
    pub canonical_base: RegionalPackCanonicalBaseGate,
    pub pack_impl: RegionalPackImplGate,
    pub ci_lanes_required: Vec<String>,
    pub claim_ceiling: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RegionalPackManifestIdentity {
    pub id: String,
    pub code: String,
    pub name: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RegionalPackManifestAuthority {
    pub accepted_adrs: Vec<String>,
    #[serde(default)]
    pub planning_context_adrs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RegionalPackManifestJurisdiction {
    pub jurisdiction: String,
    pub region_codes: Vec<String>,
    pub home_region: String,
    pub residency_classes: Vec<String>,
    pub regulatory_controls: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RegionalPackCanonicalBaseGate {
    pub neutrality_gate: String,
    pub canonical_base_paths: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RegionalPackImplGate {
    pub cross_pack_refusal_gate: String,
    pub pack_paths: Vec<String>,
    #[serde(default)]
    pub allowed_pack_dependencies: Vec<String>,
}

/// A named unit of source text handed to a gate check. `label` is the
/// caller-facing path (e.g. the manifest's declared canonical-base path) that
/// appears in any violation; `content` is the text the check scans.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionalPackSource {
    pub label: String,
    pub content: String,
}

impl RegionalPackSource {
    pub fn new(label: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            content: content.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionalPackManifestGateReport {
    pub pack_id: String,
    pub pack_code: String,
    pub checks: Vec<GateCheck>,
}

impl RegionalPackManifestGateReport {
    pub fn passed(&self) -> bool {
        self.checks.iter().all(|check| check.passed)
    }

    pub fn check(&self, gate_id: &str) -> Option<&GateCheck> {
        self.checks.iter().find(|check| check.gate_id == gate_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GateCheck {
    pub gate_id: String,
    pub passed: bool,
    pub violations: Vec<GateViolation>,
}

impl GateCheck {
    fn from_violations(gate_id: &str, violations: Vec<GateViolation>) -> Self {
        Self {
            gate_id: gate_id.to_string(),
            passed: violations.is_empty(),
            violations,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GateViolation {
    pub path: String,
    pub line: Option<usize>,
    pub reason: String,
}

impl GateViolation {
    fn new(path: impl Into<String>, line: Option<usize>, reason: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            line,
            reason: reason.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegionalPackManifestError {
    ReadFailed { path: PathBuf, reason: String },
    ParseFailed { reason: String },
    Shape { field: &'static str, reason: String },
    PathNotRelative { field: &'static str, path: String },
    PathEscapesRepo { field: &'static str, path: String },
}

/// Parse and shape-validate a manifest from its JSON contents. This is the
/// path-parameterized entry point: the caller owns how the bytes were obtained.
pub fn parse_regional_pack_manifest(
    manifest_json: &str,
) -> Result<RegionalPackManifest, RegionalPackManifestError> {
    let manifest: RegionalPackManifest = serde_json::from_str(manifest_json).map_err(|error| {
        RegionalPackManifestError::ParseFailed {
            reason: error.to_string(),
        }
    })?;
    validate_manifest_shape(&manifest)?;
    Ok(manifest)
}

/// Read a manifest from a caller-chosen path and parse it. No repo-root
/// assumptions: the caller passes whatever path locates the manifest.
pub fn load_regional_pack_manifest(
    manifest_path: impl AsRef<Path>,
) -> Result<RegionalPackManifest, RegionalPackManifestError> {
    let manifest_path = manifest_path.as_ref();
    let manifest_text = fs::read_to_string(manifest_path).map_err(|error| {
        RegionalPackManifestError::ReadFailed {
            path: manifest_path.to_path_buf(),
            reason: error.to_string(),
        }
    })?;
    parse_regional_pack_manifest(&manifest_text)
}

/// Evaluate both local pack gates against caller-supplied source text. The
/// caller resolves the manifest's declared paths (canonical-base and pack-impl)
/// and passes the loaded contents; this keeps the gate logic pure and testable.
pub fn evaluate_regional_pack_gates(
    manifest: &RegionalPackManifest,
    canonical_base_sources: &[RegionalPackSource],
    pack_impl_sources: &[RegionalPackSource],
) -> RegionalPackManifestGateReport {
    let checks = vec![
        canonical_base_neutrality_check(manifest, canonical_base_sources),
        cross_pack_refusal_check(manifest, pack_impl_sources),
    ];

    RegionalPackManifestGateReport {
        pack_id: manifest.pack.id.clone(),
        pack_code: manifest.pack.code.clone(),
        checks,
    }
}

fn validate_manifest_shape(
    manifest: &RegionalPackManifest,
) -> Result<(), RegionalPackManifestError> {
    if manifest.manifest_schema_version != REGIONAL_PACK_MANIFEST_SCHEMA_VERSION {
        return shape_error(
            "manifest_schema_version",
            format!("must equal {}", REGIONAL_PACK_MANIFEST_SCHEMA_VERSION),
        );
    }
    require_non_empty("pack.id", &manifest.pack.id)?;
    require_non_empty("pack.code", &manifest.pack.code)?;
    require_non_empty("pack.name", &manifest.pack.name)?;
    // De-branded id scheme: the id is the jurisdiction code itself or a
    // `<code>-<region>` id (e.g. `kr-seoul`), consistent with the canonical
    // `packs/<code>/` manifests. This also rejects any legacy `pack-`
    // prefix, which no longer starts with the pack code.
    let code = manifest.pack.code.as_str();
    if manifest.pack.id != code && !manifest.pack.id.starts_with(&format!("{code}-")) {
        return shape_error(
            "pack.id",
            format!("must be `{code}` or start with `{code}-` (unbranded pack id)"),
        );
    }
    if !matches!(
        manifest.pack.status.as_str(),
        "fixture" | "planned" | "preview" | "active" | "maintained" | "retired"
    ) {
        return shape_error(
            "pack.status",
            "must be fixture, planned, preview, active, maintained, or retired",
        );
    }
    require_contains(
        "source_authority.accepted_adrs",
        &manifest.source_authority.accepted_adrs,
        ACCEPTED_AUTHORITY_ADR,
    )?;
    require_contains(
        "source_authority.planning_context_adrs",
        &manifest.source_authority.planning_context_adrs,
        PLANNING_CONTEXT_ADR,
    )?;
    require_non_empty(
        "regional_pack.jurisdiction",
        &manifest.regional_pack.jurisdiction,
    )?;
    if manifest.regional_pack.jurisdiction != code.to_ascii_uppercase() {
        return shape_error(
            "regional_pack.jurisdiction",
            format!("must be {}", code.to_ascii_uppercase()),
        );
    }
    require_non_empty(
        "regional_pack.home_region",
        &manifest.regional_pack.home_region,
    )?;
    require_non_empty_vec(
        "regional_pack.region_codes",
        &manifest.regional_pack.region_codes,
    )?;
    require_non_empty_vec(
        "regional_pack.residency_classes",
        &manifest.regional_pack.residency_classes,
    )?;
    require_non_empty_vec(
        "regional_pack.regulatory_controls",
        &manifest.regional_pack.regulatory_controls,
    )?;
    require_exact(
        "canonical_base.neutrality_gate",
        &manifest.canonical_base.neutrality_gate,
        CANONICAL_BASE_NEUTRALITY_GATE,
    )?;
    require_non_empty_vec(
        "canonical_base.canonical_base_paths",
        &manifest.canonical_base.canonical_base_paths,
    )?;
    require_exact(
        "pack_impl.cross_pack_refusal_gate",
        &manifest.pack_impl.cross_pack_refusal_gate,
        CROSS_PACK_REFUSAL_GATE,
    )?;
    require_non_empty_vec("pack_impl.pack_paths", &manifest.pack_impl.pack_paths)?;
    require_contains(
        "ci_lanes_required",
        &manifest.ci_lanes_required,
        CANONICAL_BASE_NEUTRALITY_GATE,
    )?;
    require_contains(
        "ci_lanes_required",
        &manifest.ci_lanes_required,
        CROSS_PACK_REFUSAL_GATE,
    )?;
    require_non_empty("claim_ceiling", &manifest.claim_ceiling)?;
    for path in &manifest.canonical_base.canonical_base_paths {
        ensure_repo_relative("canonical_base.canonical_base_paths", path)?;
    }
    for path in &manifest.pack_impl.pack_paths {
        ensure_repo_relative("pack_impl.pack_paths", path)?;
    }
    Ok(())
}

/// Canonical-base neutrality: the shared base must not carry jurisdiction- or
/// pack-specific markers (ADR-0064). Any scanned line containing such a marker
/// is a violation.
pub fn canonical_base_neutrality_check(
    manifest: &RegionalPackManifest,
    sources: &[RegionalPackSource],
) -> GateCheck {
    let markers = forbidden_canonical_base_markers(manifest);
    let mut violations = Vec::new();

    for source in sources {
        for (line_index, line) in source.content.lines().enumerate() {
            if let Some(marker) = markers.iter().find(|marker| contains_marker(line, marker)) {
                violations.push(GateViolation::new(
                    &source.label,
                    Some(line_index + 1),
                    format!(
                        "canonical base contains jurisdiction marker `{marker}`; move locale-specific logic to the pack"
                    ),
                ));
            }
        }
    }

    GateCheck::from_violations(CANONICAL_BASE_NEUTRALITY_GATE, violations)
}

/// Cross-pack refusal: a pack implementation must not depend on another pack.
/// The manifest may not declare `allowed_pack_dependencies`, and no scanned
/// line may reference another `packs/<code>/` tree.
pub fn cross_pack_refusal_check(
    manifest: &RegionalPackManifest,
    sources: &[RegionalPackSource],
) -> GateCheck {
    let mut violations = Vec::new();
    if !manifest.pack_impl.allowed_pack_dependencies.is_empty() {
        violations.push(GateViolation::new(
            format!("packs/{}/manifest.json", manifest.pack.code),
            None,
            "regional pack must not declare pack-to-pack dependencies",
        ));
    }

    for source in sources {
        for (line_index, line) in source.content.lines().enumerate() {
            for other_pack in referenced_other_packs(line, &manifest.pack.code) {
                violations.push(GateViolation::new(
                    &source.label,
                    Some(line_index + 1),
                    format!(
                        "pack implementation references `{other_pack}`; cross-pack imports must flow through Workflow + Ontology"
                    ),
                ));
            }
        }
    }

    GateCheck::from_violations(CROSS_PACK_REFUSAL_GATE, violations)
}

fn forbidden_canonical_base_markers(manifest: &RegionalPackManifest) -> Vec<String> {
    let mut markers = vec![
        manifest.regional_pack.jurisdiction.clone(),
        title_case_ascii(&manifest.pack.code),
        manifest.pack.id.clone(),
        manifest.pack.name.clone(),
    ];
    markers.extend(manifest.regional_pack.regulatory_controls.iter().cloned());
    markers.sort();
    markers.dedup();
    markers
}

fn referenced_other_packs(line: &str, own_code: &str) -> Vec<String> {
    let needle = "packs/";
    let mut refs: Vec<String> = line
        .match_indices(needle)
        .filter_map(|(index, _)| {
            let after_prefix = &line[index + needle.len()..];
            let code: String = after_prefix
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
                .collect();
            if code.is_empty() || code == own_code {
                None
            } else {
                Some(format!("packs/{code}"))
            }
        })
        .collect();
    refs.sort();
    refs.dedup();
    refs
}

fn contains_marker(line: &str, marker: &str) -> bool {
    if marker.is_empty() {
        return false;
    }
    if marker
        .chars()
        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '-')
    {
        return contains_delimited(line, marker);
    }
    if marker.len() <= 2 && marker.chars().all(|ch| ch.is_ascii_lowercase()) {
        return contains_delimited(line, marker);
    }
    line.contains(marker)
}

fn contains_delimited(line: &str, needle: &str) -> bool {
    line.match_indices(needle).any(|(index, _)| {
        let before = line[..index].chars().next_back();
        let after = line[index + needle.len()..].chars().next();
        !is_identifier_char(before) && !is_identifier_char(after)
    })
}

fn is_identifier_char(ch: Option<char>) -> bool {
    ch.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn title_case_ascii(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

fn ensure_repo_relative(field: &'static str, path: &str) -> Result<(), RegionalPackManifestError> {
    let path_ref = Path::new(path);
    if path.trim().is_empty() || path_ref.is_absolute() {
        return Err(RegionalPackManifestError::PathNotRelative {
            field,
            path: path.to_string(),
        });
    }
    if path_ref.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(RegionalPackManifestError::PathEscapesRepo {
            field,
            path: path.to_string(),
        });
    }
    Ok(())
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), RegionalPackManifestError> {
    if value.trim().is_empty() {
        return shape_error(field, "must be non-empty");
    }
    Ok(())
}

fn require_non_empty_vec(
    field: &'static str,
    value: &[String],
) -> Result<(), RegionalPackManifestError> {
    if value.is_empty() || value.iter().any(|entry| entry.trim().is_empty()) {
        return shape_error(field, "must contain at least one non-empty value");
    }
    Ok(())
}

fn require_contains(
    field: &'static str,
    values: &[String],
    expected: &str,
) -> Result<(), RegionalPackManifestError> {
    if !values.iter().any(|value| value == expected) {
        return shape_error(field, format!("must include {expected}"));
    }
    Ok(())
}

fn require_exact(
    field: &'static str,
    value: &str,
    expected: &str,
) -> Result<(), RegionalPackManifestError> {
    if value != expected {
        return shape_error(field, format!("must equal {expected}"));
    }
    Ok(())
}

fn shape_error<T>(
    field: &'static str,
    reason: impl Into<String>,
) -> Result<T, RegionalPackManifestError> {
    Err(RegionalPackManifestError::Shape {
        field,
        reason: reason.into(),
    })
}
