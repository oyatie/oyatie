use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use crate::usage;

const DESIGN_SPEC_ALLOWED_CLAIM: &str = "Oyatie\u{2019}s architecture, platform, and system design are specified to a hyperscaler-grade design maturity bar.";
const DESIGN_CLAIM_STATUS: &str = "allowed_when_required_design_surfaces_are_green";
const OPERATIONAL_CLAIM_STATUS: &str = "blocked_until_operational_evidence_is_green";
const DEFAULT_DEFERRED_SURFACES: &str =
    "registry/design-spec-maturity/wave-3-i-deferred-surfaces.tsv";

const REQUIRED_SURFACE_IDS: &[&str] = &[
    "prd",
    "manifest",
    "implementation_plans",
    "adr_links_or_local_adrs",
    "openapi",
    "asyncapi",
    "proto3",
    "capabilities",
    "cedar_policy",
    "slos",
    "runbooks",
    "threat_model",
    "failure_modes",
    "data_residency",
    "cost_finops",
    "audit_evidence_emission",
    "tenant_isolation",
    "operational_boundaries",
    "implementation_ready_acceptance_criteria",
];

const REQUIRED_FORBIDDEN_PATTERNS: &[&str] = &[
    "deployed hyperscaler scale",
    "slo achievement",
    "compliance certification",
    "production readiness",
    "operational maturity",
    "runtime hyperscaler maturity",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DesignSpecMaturityClaimsValidateArgs {
    standard_path: PathBuf,
    service_roots: Vec<PathBuf>,
    deferred_surfaces_path: PathBuf,
    evidence_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DesignSpecMaturityClaimsReport {
    pub service_count: usize,
    pub surface_count: usize,
    pub missing_count: usize,
    pub design_claim_status: String,
    pub operational_claim_status: String,
    pub allowed_design_claim: String,
    pub evidence_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StandardRules {
    forbidden_claim_patterns: Vec<String>,
    operational_blockers: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ManifestFacts {
    has_adr_refs: bool,
    has_audit_evidence_emission: bool,
    has_data_residency_refs: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ServiceReport {
    service_id: String,
    surfaces: BTreeMap<String, Vec<String>>,
    missing: Vec<String>,
}

struct ServiceContext {
    files: Vec<PathBuf>,
    manifest_path: PathBuf,
    manifest_facts: ManifestFacts,
    cwd: Option<PathBuf>,
}

pub(crate) fn parse_design_spec_maturity_claims_validate_args(
    args: Vec<String>,
) -> Result<DesignSpecMaturityClaimsValidateArgs, String> {
    let mut parsed = DesignSpecMaturityClaimsValidateArgs {
        standard_path: PathBuf::from("specs/design-spec-maturity-claims.json"),
        service_roots: vec![
            PathBuf::from("cloud"),
            PathBuf::from("oya"),
            PathBuf::from("microservices"),
        ],
        deferred_surfaces_path: PathBuf::from(DEFAULT_DEFERRED_SURFACES),
        evidence_path: None,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--standard" => parsed.standard_path = PathBuf::from(path),
            "--microservices-root" => parsed.service_roots = vec![PathBuf::from(path)],
            "--deferred-surfaces" => parsed.deferred_surfaces_path = PathBuf::from(path),
            "--emit-evidence" => parsed.evidence_path = Some(PathBuf::from(path)),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_design_spec_maturity_claims_gate(
    args: DesignSpecMaturityClaimsValidateArgs,
) -> Result<DesignSpecMaturityClaimsReport, String> {
    let standard = read_json(&args.standard_path, "design/spec maturity standard")?;
    let rules = validate_standard(&standard)?;
    let services = discover_services_from_roots(&args.service_roots)?;

    let cwd = std::env::current_dir().ok();
    let mut reports = Vec::new();
    for service_root in services {
        let service_id = service_root
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("service path is not utf8: {}", service_root.display()))?
            .to_owned();
        let files = collect_files(&service_root)?;
        let manifest_path = service_root.join("manifest.json");
        let manifest_facts = read_manifest_facts(&manifest_path)?;
        let context = ServiceContext {
            files,
            manifest_path,
            manifest_facts,
            cwd: cwd.clone(),
        };
        reports.push(validate_service(&service_id, &context)?);
    }

    let deferred_surfaces = read_deferred_surface_records(&args.deferred_surfaces_path)?;
    let mut deferred_count = 0usize;
    for report in &mut reports {
        report.missing.retain(|surface_id| {
            let is_deferred =
                deferred_surfaces.contains(&(report.service_id.clone(), surface_id.clone()));
            if is_deferred {
                deferred_count += 1;
            }
            !is_deferred
        });
    }

    let missing_count = reports
        .iter()
        .map(|report| report.missing.len())
        .sum::<usize>();
    if let Some(evidence_path) = &args.evidence_path {
        let roots_display = args
            .service_roots
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(",");
        let roots_display_path = std::path::PathBuf::from(&roots_display);
        let evidence = build_evidence(
            &args.standard_path,
            &roots_display_path,
            &args.deferred_surfaces_path,
            &rules,
            &reports,
            missing_count,
            deferred_count,
        );
        write_json(evidence_path, &evidence)?;
    }

    if missing_count > 0 {
        return Err(format!(
            "design/spec maturity coverage missing {} required service surfaces: {}",
            missing_count,
            summarize_missing(&reports)
        ));
    }

    Ok(DesignSpecMaturityClaimsReport {
        service_count: reports.len(),
        surface_count: REQUIRED_SURFACE_IDS.len(),
        missing_count,
        design_claim_status: DESIGN_CLAIM_STATUS.to_owned(),
        operational_claim_status: OPERATIONAL_CLAIM_STATUS.to_owned(),
        allowed_design_claim: DESIGN_SPEC_ALLOWED_CLAIM.to_owned(),
        evidence_path: args.evidence_path,
    })
}

fn validate_standard(standard: &Value) -> Result<StandardRules, String> {
    let root = object(standard, "design/spec maturity standard root")?;
    let claim_rule = object_field(root, "design_spec_maturity_claim_rule")?;
    let exact_claim = string_field(claim_rule, "exact_claim")?;
    if exact_claim != DESIGN_SPEC_ALLOWED_CLAIM {
        return Err(format!(
            "design/spec maturity exact_claim must be {DESIGN_SPEC_ALLOWED_CLAIM:?}, got {exact_claim:?}"
        ));
    }
    let claim_status = string_field(claim_rule, "claim_status")?;
    if claim_status != DESIGN_CLAIM_STATUS {
        return Err(format!(
            "design/spec maturity claim_status must be {DESIGN_CLAIM_STATUS:?}, got {claim_status:?}"
        ));
    }
    let claim_scope = string_field(claim_rule, "claim_scope")?;
    if !claim_scope.contains("design/spec") {
        return Err("design/spec maturity claim_scope must explicitly say design/spec".into());
    }
    let forbidden_claim_patterns = string_array_field(claim_rule, "forbidden_claim_patterns")?;
    for required in REQUIRED_FORBIDDEN_PATTERNS {
        if !forbidden_claim_patterns
            .iter()
            .any(|pattern| pattern.to_ascii_lowercase().contains(required))
        {
            return Err(format!(
                "design/spec maturity forbidden_claim_patterns must include {required:?}"
            ));
        }
    }

    let operational_rule = object_field(root, "operational_maturity_claim_rule")?;
    let operational_status = string_field(operational_rule, "claim_status")?;
    if operational_status != OPERATIONAL_CLAIM_STATUS {
        return Err(format!(
            "operational maturity claim_status must remain {OPERATIONAL_CLAIM_STATUS:?}, got {operational_status:?}"
        ));
    }
    let operational_blockers = string_array_field(operational_rule, "current_blockers")?;
    if operational_blockers.is_empty() {
        return Err(
            "operational maturity current_blockers must stay non-empty while implementation evidence is incomplete"
                .into(),
        );
    }

    validate_required_surfaces(root)?;

    Ok(StandardRules {
        forbidden_claim_patterns,
        operational_blockers,
    })
}

fn validate_required_surfaces(root: &Map<String, Value>) -> Result<(), String> {
    let surfaces = array_field(root, "required_surfaces")?;
    let mut ids = BTreeSet::new();
    let required_ids = REQUIRED_SURFACE_IDS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    for (index, row) in surfaces.iter().enumerate() {
        let surface = object(row, &format!("required_surfaces[{index}]"))?;
        let id = string_field(surface, "id")?;
        if !ids.insert(id.to_owned()) {
            return Err(format!("duplicate required design/spec surface id {id:?}"));
        }
        require_non_empty_string(surface, "name")?;
        require_non_empty_string(surface, "evidence_policy")?;
    }
    let unexpected = ids
        .iter()
        .filter(|id| !required_ids.contains(id.as_str()))
        .map(String::as_str)
        .collect::<Vec<_>>();
    if !unexpected.is_empty() {
        return Err(format!(
            "unknown required design/spec surface ids: {}",
            unexpected.join(", ")
        ));
    }
    for required_id in REQUIRED_SURFACE_IDS {
        if !ids.contains(*required_id) {
            return Err(format!(
                "missing required design/spec surface {required_id:?}"
            ));
        }
    }
    Ok(())
}

fn validate_service(service_id: &str, context: &ServiceContext) -> Result<ServiceReport, String> {
    let mut surfaces = BTreeMap::new();
    let mut missing = Vec::new();
    for surface_id in REQUIRED_SURFACE_IDS {
        let evidence = surface_evidence(surface_id, context)?;
        if evidence.is_empty() {
            missing.push((*surface_id).to_owned());
        }
        surfaces.insert((*surface_id).to_owned(), evidence);
    }
    Ok(ServiceReport {
        service_id: service_id.to_owned(),
        surfaces,
        missing,
    })
}

fn surface_evidence(surface_id: &str, context: &ServiceContext) -> Result<Vec<String>, String> {
    match surface_id {
        "prd" => Ok(existing_paths(context, |path| {
            file_name_lower(path) == "prd.md"
        })),
        "manifest" => Ok(existing_paths(context, |path| {
            path == context.manifest_path
        })),
        "implementation_plans" => Ok(existing_paths(context, |path| {
            file_name_lower(path).starts_with("ip-") && has_extension(path, &["md"])
        })),
        "adr_links_or_local_adrs" => {
            let mut evidence = existing_paths(context, |path| {
                path_has_component(path, "decisions")
                    && file_name_lower(path).contains("adr-")
                    && has_extension(path, &["md"])
            });
            if context.manifest_facts.has_adr_refs {
                evidence.push(format!(
                    "{}#adrs",
                    display_path(&context.manifest_path, &context.cwd)
                ));
            }
            evidence.sort();
            evidence.dedup();
            Ok(evidence)
        }
        "openapi" => Ok(existing_paths(context, |path| {
            path_lower(path).contains("openapi") && has_extension(path, &["yaml", "yml", "json"])
        })),
        "asyncapi" => Ok(existing_paths(context, |path| {
            path_lower(path).contains("asyncapi") && has_extension(path, &["yaml", "yml", "json"])
        })),
        "proto3" => evidence_by_content(context, &["proto"], |text| {
            text.contains("syntax = \"proto3\"") || text.contains("syntax=\"proto3\"")
        }),
        "capabilities" => Ok(existing_paths(context, |path| {
            path_has_component(path, "capabilities") && path.is_file()
        })),
        "cedar_policy" => Ok(existing_paths(context, |path| {
            has_extension(path, &["cedar"]) || path_has_component(path, "policy")
        })),
        "slos" => Ok(existing_paths(context, |path| {
            path_has_component(path, "slos") && has_extension(path, &["yaml", "yml", "json"])
        })),
        "runbooks" => Ok(existing_paths(context, |path| {
            path_has_component(path, "runbooks") && has_extension(path, &["md"])
        })),
        "threat_model" => Ok(existing_paths(context, |path| {
            file_name_lower(path).contains("threat-model")
                || file_name_lower(path).contains("threat_model")
        })),
        "failure_modes" => Ok(existing_paths(context, |path| {
            file_name_lower(path).contains("failure-mode")
                || file_name_lower(path).contains("failure_mode")
        })),
        "data_residency" => {
            let mut evidence = existing_paths(context, |path| {
                let text = path_lower(path);
                text.contains("residency")
                    || text.contains("sovereign")
                    || text.contains("multi-region")
                    || text.contains("regional-pack")
            });
            if context.manifest_facts.has_data_residency_refs {
                evidence.push(format!(
                    "{}#regulatory_packs",
                    display_path(&context.manifest_path, &context.cwd)
                ));
            }
            evidence.sort();
            evidence.dedup();
            Ok(evidence)
        }
        "cost_finops" => Ok(existing_paths(context, |path| {
            let text = path_lower(path);
            text.contains("cost") || text.contains("finops")
        })),
        "audit_evidence_emission" => {
            let mut evidence = existing_paths(context, |path| {
                let text = path_lower(path);
                text.contains("evidence-emission")
                    || text.contains("evidence_emission")
                    || text.contains("audit")
            });
            if context.manifest_facts.has_audit_evidence_emission {
                evidence.push(format!(
                    "{}#audit_chain",
                    display_path(&context.manifest_path, &context.cwd)
                ));
            }
            evidence.sort();
            evidence.dedup();
            Ok(evidence)
        }
        "tenant_isolation" => Ok(existing_paths(context, |path| {
            let text = path_lower(path);
            text.contains("tenant") || text.contains("isolation") || text.contains("rls")
        })),
        "operational_boundaries" => Ok(existing_paths(context, |path| {
            let text = path_lower(path);
            text.contains("operational-boundaries")
                || text.contains("incident")
                || text.contains("capacity")
                || text.contains("backfill")
                || text.contains("multi-region")
        })),
        "implementation_ready_acceptance_criteria" => {
            evidence_by_content(context, &["md"], |text| text.contains("acceptance"))
        }
        _ => Err(format!(
            "unknown required design/spec surface {surface_id:?}"
        )),
    }
}

fn existing_paths(context: &ServiceContext, predicate: impl Fn(&Path) -> bool) -> Vec<String> {
    let mut paths = context
        .files
        .iter()
        .filter(|path| predicate(path))
        .map(|path| display_path(path, &context.cwd))
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
}

fn evidence_by_content(
    context: &ServiceContext,
    extensions: &[&str],
    predicate: impl Fn(&str) -> bool,
) -> Result<Vec<String>, String> {
    let mut evidence = Vec::new();
    for path in &context.files {
        if !has_extension(path, extensions) {
            continue;
        }
        let text = fs::read_to_string(path)
            .map_err(|error| format!("{} unreadable: {error}", path.display()))?;
        if predicate(&text.to_ascii_lowercase()) {
            evidence.push(display_path(path, &context.cwd));
        }
    }
    evidence.sort();
    evidence.dedup();
    Ok(evidence)
}

fn read_manifest_facts(path: &Path) -> Result<ManifestFacts, String> {
    if !path.exists() {
        return Ok(ManifestFacts::default());
    }
    let manifest = read_json(path, "microservice manifest")?;
    let root = object(&manifest, "microservice manifest root")?;
    let has_adr_refs = root
        .get("adrs")
        .and_then(Value::as_array)
        .or_else(|| root.get("adr_authority_chain").and_then(Value::as_array))
        .is_some_and(|values| !values.is_empty());
    let has_data_residency_refs = root
        .get("regulatory_packs")
        .and_then(Value::as_array)
        .or_else(|| root.get("sovereign_packs").and_then(Value::as_array))
        .is_some_and(|values| !values.is_empty());
    let has_audit_evidence_emission = root
        .get("audit_chain")
        .and_then(Value::as_object)
        .is_some_and(|audit| {
            audit.get("enabled").and_then(Value::as_bool) == Some(true)
                && audit
                    .get("seal_events")
                    .and_then(Value::as_array)
                    .is_some_and(|values| !values.is_empty())
        });
    Ok(ManifestFacts {
        has_adr_refs,
        has_audit_evidence_emission,
        has_data_residency_refs,
    })
}

fn read_deferred_surface_records(path: &Path) -> Result<BTreeSet<(String, String)>, String> {
    if !path.exists() {
        return Ok(BTreeSet::new());
    }
    let text = fs::read_to_string(path)
        .map_err(|error| format!("design/spec maturity deferral registry unreadable: {error}"))?;
    let mut records = BTreeSet::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let fields = trimmed.split('\t').collect::<Vec<_>>();
        if fields.len() < 3 {
            return Err(format!(
                "{}:{} deferral row must be <service_id><tab><surface_id><tab><reason>",
                path.display(),
                index + 1
            ));
        }
        let service_id = fields[0].trim();
        let surface_id = fields[1].trim();
        let reason = fields[2].trim();
        if service_id.is_empty() || surface_id.is_empty() || reason.is_empty() {
            return Err(format!(
                "{}:{} deferral row fields must be non-empty",
                path.display(),
                index + 1
            ));
        }
        if !REQUIRED_SURFACE_IDS.contains(&surface_id) {
            return Err(format!(
                "{}:{} unknown deferred design/spec surface {surface_id:?}",
                path.display(),
                index + 1
            ));
        }
        if !records.insert((service_id.to_owned(), surface_id.to_owned())) {
            return Err(format!(
                "{}:{} duplicate deferral for {service_id}/{surface_id}",
                path.display(),
                index + 1
            ));
        }
    }
    Ok(records)
}

fn discover_services_from_roots(roots: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut services = Vec::new();
    for root in roots {
        if !root.exists() {
            continue;
        }
        let entries = fs::read_dir(root)
            .map_err(|error| format!("service root unreadable {}: {error}", root.display()))?;
        for entry in entries {
            let path = entry
                .map_err(|error| format!("service root entry unreadable: {error}"))?
                .path();
            if path.is_dir() {
                services.push(path);
            }
        }
    }
    services.sort();
    Ok(services)
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    collect_files_inner(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_inner(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(root)
        .map_err(|error| format!("directory unreadable {}: {error}", root.display()))?;
    let mut paths = Vec::new();
    for entry in entries {
        paths.push(
            entry
                .map_err(|error| format!("directory entry unreadable: {error}"))?
                .path(),
        );
    }
    paths.sort();
    for path in paths {
        if path.is_dir() {
            if is_excluded_scan_dir(&path) {
                continue;
            }
            collect_files_inner(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

/// Build/vendor directories are skipped during surface discovery: their
/// artifact paths (cargo `target/` hashes, `node_modules/`, build output) can
/// contain substrings like "rls"/"isolation"/"tenant" and FALSELY satisfy a
/// surface predicate, masking a genuinely-missing design surface on a clean
/// checkout. Excluding them keeps the gate honest across local-build state.
fn is_excluded_scan_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("target" | "node_modules" | ".git" | "dist" | ".vinxi" | ".output")
    )
}

fn build_evidence(
    standard_path: &Path,
    microservices_root: &Path,
    deferred_surfaces_path: &Path,
    rules: &StandardRules,
    reports: &[ServiceReport],
    missing_count: usize,
    deferred_count: usize,
) -> Value {
    let service_values = reports
        .iter()
        .map(|report| {
            json!({
                "service_id": report.service_id,
                "surfaces": report.surfaces,
                "missing": report.missing,
            })
        })
        .collect::<Vec<_>>();
    let missing_services = reports
        .iter()
        .filter(|report| !report.missing.is_empty())
        .map(|report| {
            json!({
                "service_id": report.service_id,
                "missing": report.missing,
            })
        })
        .collect::<Vec<_>>();

    json!({
        "schema_version": "oyatie.design-spec-maturity.evidence.v1",
        "generated_by": "oya gate validate design-spec-maturity-claims",
        "standard_path": normalize_path(standard_path),
        "microservices_root": normalize_path(microservices_root),
        "deferred_surfaces_path": normalize_path(deferred_surfaces_path),
        "service_count": reports.len(),
        "surface_count": REQUIRED_SURFACE_IDS.len(),
        "missing_count": missing_count,
        "deferred_count": deferred_count,
        "claims": {
            "allowed_design_claim": DESIGN_SPEC_ALLOWED_CLAIM,
            "design_claim_status": DESIGN_CLAIM_STATUS,
            "operational_claim_status": OPERATIONAL_CLAIM_STATUS,
            "forbidden_claim_patterns": rules.forbidden_claim_patterns,
            "operational_blockers": rules.operational_blockers,
        },
        "required_surface_ids": REQUIRED_SURFACE_IDS,
        "missing_services": missing_services,
        "services": service_values,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "evidence directory creation failed {}: {error}",
                parent.display()
            )
        })?;
    }
    let text = serde_json::to_string_pretty(value)
        .map_err(|error| format!("design/spec maturity evidence serialization failed: {error}"))?;
    fs::write(path, format!("{text}\n")).map_err(|error| {
        format!(
            "design/spec maturity evidence write failed {}: {error}",
            path.display()
        )
    })
}

fn summarize_missing(reports: &[ServiceReport]) -> String {
    let mut parts = reports
        .iter()
        .filter(|report| !report.missing.is_empty())
        .map(|report| {
            format!(
                "{} missing {}",
                report.service_id,
                report.missing.join(", ")
            )
        })
        .collect::<Vec<_>>();
    parts.truncate(8);
    if parts.is_empty() {
        "none".to_owned()
    } else {
        parts.join("; ")
    }
}

fn read_json(path: &Path, label: &str) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{label} unreadable: {error}"))?;
    serde_json::from_str(&text).map_err(|error| format!("{label} invalid JSON: {error}"))
}

fn object<'a>(value: &'a Value, label: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{label} must be a JSON object"))
}

fn object_field<'a>(
    object: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a Map<String, Value>, String> {
    object
        .get(field)
        .ok_or_else(|| format!("missing JSON object field {field:?}"))
        .and_then(|value| {
            value
                .as_object()
                .ok_or_else(|| format!("JSON field {field:?} must be an object"))
        })
}

fn array_field<'a>(object: &'a Map<String, Value>, field: &str) -> Result<&'a Vec<Value>, String> {
    object
        .get(field)
        .ok_or_else(|| format!("missing JSON array field {field:?}"))
        .and_then(|value| {
            value
                .as_array()
                .ok_or_else(|| format!("JSON field {field:?} must be an array"))
        })
}

fn string_field<'a>(object: &'a Map<String, Value>, field: &str) -> Result<&'a str, String> {
    object
        .get(field)
        .ok_or_else(|| format!("missing JSON string field {field:?}"))
        .and_then(|value| {
            value
                .as_str()
                .ok_or_else(|| format!("JSON field {field:?} must be a string"))
        })
}

fn require_non_empty_string(object: &Map<String, Value>, field: &str) -> Result<(), String> {
    let value = string_field(object, field)?;
    if value.trim().is_empty() {
        return Err(format!("JSON field {field:?} must not be empty"));
    }
    Ok(())
}

fn string_array_field(object: &Map<String, Value>, field: &str) -> Result<Vec<String>, String> {
    let values = array_field(object, field)?;
    let mut strings = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let Some(text) = value.as_str() else {
            return Err(format!("JSON field {field:?}[{index}] must be a string"));
        };
        if text.trim().is_empty() {
            return Err(format!("JSON field {field:?}[{index}] must not be empty"));
        }
        strings.push(text.to_owned());
    }
    Ok(strings)
}

fn path_has_component(path: &Path, component: &str) -> bool {
    path.components()
        .any(|path_component| path_component.as_os_str().to_str() == Some(component))
}

fn has_extension(path: &Path, allowed_extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            let extension = extension.to_ascii_lowercase();
            allowed_extensions.contains(&extension.as_str())
        })
        .unwrap_or(false)
}

fn file_name_lower(path: &Path) -> String {
    path.file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
}

fn path_lower(path: &Path) -> String {
    normalize_path(path).to_ascii_lowercase()
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn display_path(path: &Path, cwd: &Option<PathBuf>) -> String {
    if let Some(cwd) = cwd
        && let Ok(stripped) = path.strip_prefix(cwd)
    {
        return normalize_path(stripped);
    }
    normalize_path(path)
}
