use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::usage;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkspaceHygieneValidateArgs {
    pub(crate) policy_path: PathBuf,
    pub(crate) scan: bool,
    pub(crate) strict: bool,
    pub(crate) clean_build_artifacts: bool,
    pub(crate) clean_temp_artifacts: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorkspaceHygieneReport {
    pub surfaces_checked: usize,
    pub roots_scanned: usize,
    pub findings: usize,
    pub strict: bool,
    pub cleaned: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScanSurface {
    id: String,
    roots: Vec<String>,
    missing_ok: bool,
    patterns: Vec<String>,
    max_depth: usize,
    audit_finding_budget: usize,
    strict_finding_budget: usize,
    action: String,
    cleanup_patterns: Vec<String>,
    exempt_patterns: Vec<String>,
    exemption_evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ScanOutcome {
    findings: usize,
    cleaned: usize,
    examples: Vec<String>,
}

pub(crate) fn parse_workspace_hygiene_validate_args(
    args: Vec<String>,
) -> Result<WorkspaceHygieneValidateArgs, String> {
    let mut parsed = WorkspaceHygieneValidateArgs {
        policy_path: PathBuf::from("specs/workspace-hygiene.json"),
        scan: true,
        strict: false,
        clean_build_artifacts: false,
        clean_temp_artifacts: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--policy" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.policy_path = PathBuf::from(path);
            }
            "--no-scan" => parsed.scan = false,
            "--strict" => parsed.strict = true,
            "--clean-build-artifacts" => parsed.clean_build_artifacts = true,
            "--clean-temp-artifacts" => parsed.clean_temp_artifacts = true,
            _ => return Err(usage()),
        }
    }
    if (parsed.clean_build_artifacts || parsed.clean_temp_artifacts) && !parsed.scan {
        return Err("workspace hygiene cleanup requires scanning; remove --no-scan".into());
    }
    Ok(parsed)
}

pub(crate) fn validate_workspace_hygiene_gate(
    args: WorkspaceHygieneValidateArgs,
) -> Result<WorkspaceHygieneReport, String> {
    let policy = read_json(&args.policy_path)?;
    let root = object(&policy, "workspace hygiene policy root")?;
    require_non_empty_string(root, "schema_version")?;
    require_non_empty_string(root, "id")?;
    require_non_empty_string(root, "purpose")?;
    validate_gate_contract(root)?;
    validate_pipeline_contract(root)?;

    let required_surface_ids = string_array_field(root, "required_scan_surfaces")?;
    for expected in REQUIRED_SURFACES {
        if !required_surface_ids.iter().any(|value| value == expected) {
            return Err(format!(
                "workspace hygiene policy required_scan_surfaces missing {expected}"
            ));
        }
    }

    let surfaces = read_scan_surfaces(root)?;
    for expected in REQUIRED_SURFACES {
        if !surfaces.contains_key(*expected) {
            return Err(format!(
                "workspace hygiene policy scan_surfaces missing {expected}"
            ));
        }
    }

    let mut roots_scanned = 0usize;
    let mut findings = 0usize;
    let mut cleaned = 0usize;
    if args.scan {
        for surface in surfaces.values() {
            let mut scanned_roots = BTreeSet::<PathBuf>::new();
            let surface_outcome = scan_surface(
                surface,
                args.strict,
                args.clean_build_artifacts,
                args.clean_temp_artifacts,
                &mut scanned_roots,
                &mut roots_scanned,
            )?;
            let budget = if args.strict {
                surface.strict_finding_budget
            } else {
                surface.audit_finding_budget
            };
            if surface_outcome.findings > budget {
                let examples = if surface_outcome.examples.is_empty() {
                    String::new()
                } else {
                    format!("; examples: {}", surface_outcome.examples.join(", "))
                };
                return Err(format!(
                    "workspace hygiene surface {} has {} findings above {} budget {budget}{examples}",
                    surface.id,
                    surface_outcome.findings,
                    if args.strict { "strict" } else { "audit" }
                ));
            }
            findings += surface_outcome.findings;
            cleaned += surface_outcome.cleaned;
        }
    }

    Ok(WorkspaceHygieneReport {
        surfaces_checked: surfaces.len(),
        roots_scanned,
        findings,
        strict: args.strict,
        cleaned,
    })
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| {
        format!(
            "workspace hygiene policy unreadable {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&text)
        .map_err(|error| format!("workspace hygiene policy invalid JSON: {error}"))
}

fn validate_gate_contract(root: &serde_json::Map<String, Value>) -> Result<(), String> {
    let gate = object_field(root, "gate")?;
    require_string_field_equals(gate, "command", "oya gate validate workspace-hygiene")?;
    require_string_field_equals(
        gate,
        "side_effect_policy",
        "inventory_by_default_cleanup_requires_explicit_flag",
    )?;
    Ok(())
}

fn validate_pipeline_contract(root: &serde_json::Map<String, Value>) -> Result<(), String> {
    let contract = object_field(root, "pipeline_contract")?;
    for phase in REQUIRED_PIPELINE_PHASES {
        require_string_array_contains(contract, "required_phases", phase)?;
    }
    for action in REQUIRED_PIPELINE_ACTIONS {
        require_string_array_contains(contract, "minimum_actions", action)?;
    }
    Ok(())
}

fn read_scan_surfaces(
    root: &serde_json::Map<String, Value>,
) -> Result<BTreeMap<String, ScanSurface>, String> {
    let rows = array_field(root, "scan_surfaces")?;
    let mut surfaces = BTreeMap::new();
    for (index, value) in rows.iter().enumerate() {
        let row = object(value, &format!("scan_surfaces[{index}]"))?;
        let id = string_field(row, "id")?.to_string();
        let surface = ScanSurface {
            id: id.clone(),
            roots: string_array_field(row, "roots")?,
            missing_ok: optional_bool_field(row, "missing_ok").unwrap_or(false),
            patterns: string_array_field(row, "match_globs")?,
            max_depth: usize_field(row, "max_depth")?,
            audit_finding_budget: usize_field(row, "audit_finding_budget")?,
            strict_finding_budget: usize_field(row, "strict_finding_budget")?,
            action: string_field(row, "action")?.to_string(),
            cleanup_patterns: optional_string_array_field(row, "cleanup_globs")?
                .unwrap_or_default(),
            exempt_patterns: optional_string_array_field(row, "exempt_globs")?.unwrap_or_default(),
            exemption_evidence_refs: optional_string_array_field(row, "exemption_evidence_refs")?
                .unwrap_or_default(),
        };
        if surface.action != "inventory_only"
            && surface.action != "cleanable_build_artifacts"
            && surface.action != "cleanable_temp_artifacts"
        {
            return Err(format!(
                "workspace hygiene surface {id} action must be inventory_only, cleanable_build_artifacts, or cleanable_temp_artifacts"
            ));
        }
        if surface.action == "cleanable_build_artifacts" && surface.id != "build-artifacts" {
            return Err(format!(
                "workspace hygiene surface {id} cannot use cleanable_build_artifacts"
            ));
        }
        if surface.action == "cleanable_temp_artifacts" && surface.id != "tmp" {
            return Err(format!(
                "workspace hygiene surface {id} cannot use cleanable_temp_artifacts"
            ));
        }
        if surface.action != "inventory_only" && surface.cleanup_patterns.is_empty() {
            return Err(format!(
                "workspace hygiene surface {id} cleanup_globs must be non-empty"
            ));
        }
        if !surface.exempt_patterns.is_empty() && surface.exemption_evidence_refs.is_empty() {
            return Err(format!(
                "workspace hygiene surface {id} exemption_evidence_refs must be non-empty when exempt_globs are present"
            ));
        }
        for cleanup_pattern in &surface.cleanup_patterns {
            if !surface
                .patterns
                .iter()
                .any(|pattern| pattern == cleanup_pattern)
            {
                return Err(format!(
                    "workspace hygiene surface {id} cleanup_globs entry {cleanup_pattern:?} must also appear in match_globs"
                ));
            }
        }
        if surface.patterns.is_empty() {
            return Err(format!(
                "workspace hygiene surface {id} match_globs must be non-empty"
            ));
        }
        if surfaces.insert(id.clone(), surface).is_some() {
            return Err(format!("duplicate workspace hygiene surface {id}"));
        }
    }
    Ok(surfaces)
}

fn scan_surface(
    surface: &ScanSurface,
    strict: bool,
    clean_build_artifacts: bool,
    clean_temp_artifacts: bool,
    scanned_roots: &mut BTreeSet<PathBuf>,
    roots_scanned: &mut usize,
) -> Result<ScanOutcome, String> {
    let mut outcome = ScanOutcome::default();
    for root in &surface.roots {
        let expanded = expand_home(root)?;
        if !expanded.exists() {
            if surface.missing_ok {
                continue;
            }
            return Err(format!(
                "workspace hygiene surface {} root missing: {}",
                surface.id,
                expanded.display()
            ));
        }
        let canonical = fs::canonicalize(&expanded).map_err(|error| {
            format!(
                "workspace hygiene surface {} root canonicalize failed {}: {error}",
                surface.id,
                expanded.display()
            )
        })?;
        if !scanned_roots.insert(canonical.clone()) {
            continue;
        }
        *roots_scanned += 1;
        outcome += scan_directory_entries(
            surface,
            &canonical,
            0,
            clean_build_artifacts,
            clean_temp_artifacts,
        )?;
    }
    if strict && surface.roots.is_empty() {
        return Err(format!(
            "workspace hygiene strict mode requires at least one root for {}",
            surface.id
        ));
    }
    Ok(outcome)
}

fn scan_directory_entries(
    surface: &ScanSurface,
    root: &Path,
    depth: usize,
    clean_build_artifacts: bool,
    clean_temp_artifacts: bool,
) -> Result<ScanOutcome, String> {
    let entries = fs::read_dir(root).map_err(|error| {
        format!(
            "workspace hygiene surface {} root unreadable {}: {error}",
            surface.id,
            root.display()
        )
    })?;
    let mut outcome = ScanOutcome::default();
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "workspace hygiene surface {} read_dir entry failed: {error}",
                surface.id
            )
        })?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if is_exempt(surface, name) {
            continue;
        }
        let matched = surface
            .patterns
            .iter()
            .any(|pattern| simple_glob_matches(name, pattern));
        if should_clean_artifact(surface, clean_build_artifacts, clean_temp_artifacts, name) {
            remove_artifact(surface, &entry.path())?;
            outcome.cleaned += 1;
            continue;
        }
        if matched {
            outcome.findings += 1;
            if outcome.examples.len() < MAX_FINDING_EXAMPLES {
                outcome.examples.push(entry.path().display().to_string());
            }
        }
        if depth + 1 < surface.max_depth
            && entry
                .file_type()
                .map(|file_type| file_type.is_dir())
                .unwrap_or(false)
        {
            outcome += scan_directory_entries(
                surface,
                &entry.path(),
                depth + 1,
                clean_build_artifacts,
                clean_temp_artifacts,
            )?;
        }
    }
    Ok(outcome)
}

fn is_exempt(surface: &ScanSurface, name: &str) -> bool {
    surface
        .exempt_patterns
        .iter()
        .any(|pattern| simple_glob_matches(name, pattern))
}

fn should_clean_artifact(
    surface: &ScanSurface,
    clean_build_artifacts: bool,
    clean_temp_artifacts: bool,
    name: &str,
) -> bool {
    let clean_surface = (clean_build_artifacts && surface.action == "cleanable_build_artifacts")
        || (clean_temp_artifacts && surface.action == "cleanable_temp_artifacts");
    clean_surface
        && surface
            .cleanup_patterns
            .iter()
            .any(|pattern| simple_glob_matches(name, pattern))
}

fn remove_artifact(surface: &ScanSurface, path: &Path) -> Result<(), String> {
    let file_type = fs::symlink_metadata(path)
        .map_err(|error| {
            format!(
                "workspace hygiene cleanup metadata failed {}: {error}",
                path.display()
            )
        })?
        .file_type();
    if file_type.is_dir() {
        fs::remove_dir_all(path).map_err(|error| {
            format!(
                "workspace hygiene surface {} cleanup failed {}: {error}",
                surface.id,
                path.display()
            )
        })
    } else {
        fs::remove_file(path).map_err(|error| {
            format!(
                "workspace hygiene surface {} cleanup failed {}: {error}",
                surface.id,
                path.display()
            )
        })
    }
}

fn expand_home(raw: &str) -> Result<PathBuf, String> {
    if raw == "~" {
        let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
        return Ok(PathBuf::from(home));
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
        return Ok(PathBuf::from(home).join(rest));
    }
    Ok(PathBuf::from(raw))
}

fn simple_glob_matches(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    match (pattern.strip_prefix('*'), pattern.strip_suffix('*')) {
        (Some(_), Some(_)) if pattern.len() >= 2 => {
            let inner = &pattern[1..pattern.len() - 1];
            name.contains(inner)
        }
        (Some(suffix), None) => name.ends_with(suffix),
        (None, Some(prefix)) => name.starts_with(prefix),
        (None, None) => name == pattern,
        _ => false,
    }
}

fn object<'a>(value: &'a Value, path: &str) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{path} must be a JSON object"))
}

fn object_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    object
        .get(field)
        .ok_or_else(|| format!("missing object field {field}"))?
        .as_object()
        .ok_or_else(|| format!("{field} must be a JSON object"))
}

fn array_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a Vec<Value>, String> {
    object
        .get(field)
        .ok_or_else(|| format!("missing array field {field}"))?
        .as_array()
        .ok_or_else(|| format!("{field} must be a JSON array"))
}

fn string_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a str, String> {
    object
        .get(field)
        .ok_or_else(|| format!("missing string field {field}"))?
        .as_str()
        .ok_or_else(|| format!("{field} must be a string"))
}

fn require_non_empty_string(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<(), String> {
    if string_field(object, field)?.trim().is_empty() {
        Err(format!("{field} must be non-empty"))
    } else {
        Ok(())
    }
}

fn require_string_field_equals(
    object: &serde_json::Map<String, Value>,
    field: &str,
    expected: &str,
) -> Result<(), String> {
    let actual = string_field(object, field)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{field} must be {expected:?}, got {actual:?}"))
    }
}

fn string_array_field(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<Vec<String>, String> {
    let values = array_field(object, field)?;
    let mut strings = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let Some(string) = value.as_str() else {
            return Err(format!("{field}[{index}] must be a string"));
        };
        if string.trim().is_empty() {
            return Err(format!("{field}[{index}] must be non-empty"));
        }
        strings.push(string.to_string());
    }
    Ok(strings)
}

fn optional_string_array_field(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<Option<Vec<String>>, String> {
    if object.contains_key(field) {
        string_array_field(object, field).map(Some)
    } else {
        Ok(None)
    }
}

fn require_string_array_contains(
    object: &serde_json::Map<String, Value>,
    field: &str,
    expected: &str,
) -> Result<(), String> {
    let values = string_array_field(object, field)?;
    if values.iter().any(|value| value == expected) {
        Ok(())
    } else {
        Err(format!("{field} must include {expected:?}"))
    }
}

fn optional_bool_field(object: &serde_json::Map<String, Value>, field: &str) -> Option<bool> {
    object.get(field).and_then(Value::as_bool)
}

fn usize_field(object: &serde_json::Map<String, Value>, field: &str) -> Result<usize, String> {
    let value = object
        .get(field)
        .ok_or_else(|| format!("missing number field {field}"))?
        .as_u64()
        .ok_or_else(|| format!("{field} must be an unsigned integer"))?;
    usize::try_from(value).map_err(|_| format!("{field} is too large"))
}

impl std::ops::AddAssign for ScanOutcome {
    fn add_assign(&mut self, rhs: Self) {
        self.findings += rhs.findings;
        self.cleaned += rhs.cleaned;
        for example in rhs.examples {
            if self.examples.len() >= MAX_FINDING_EXAMPLES {
                break;
            }
            self.examples.push(example);
        }
    }
}

const REQUIRED_SURFACES: &[&str] = &["tmp", "home", "repo", "build-artifacts", "oyatie-worktrees"];
const MAX_FINDING_EXAMPLES: usize = 5;

const REQUIRED_PIPELINE_PHASES: &[&str] =
    &["session-start", "pre-pr", "post-merge", "session-close"];

const REQUIRED_PIPELINE_ACTIONS: &[&str] = &[
    "inventory_all_required_scan_surfaces",
    "classify_findings_by_hygiene_class",
    "classify_build_artifacts_by_cleanup_or_exemption",
    "clean_configured_build_artifacts_with_explicit_cleanup_flag",
    "clean_configured_temp_artifacts_with_explicit_cleanup_flag",
    "classify_owned_roots_by_exemption_evidence",
    "link_each_keep_item_to_owner_or_evidence",
    "strict_mode_zero_untriaged_findings_before_release_or_hyperscaler_claim",
];
