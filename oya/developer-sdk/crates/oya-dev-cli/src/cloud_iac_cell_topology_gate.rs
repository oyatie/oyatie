//! `oya gate validate cloud-iac-cell-topology` runner.
//!
//! This gate makes the Cloud IaC cell-topology metadata a permanent local
//! fail-closed evidence surface. It parses JSON, checks repo-relative files,
//! ties cells back to the local OpenTofu module catalog and Argo CD template
//! contexts, and intentionally performs no Argo CD API, Kubernetes API,
//! provider API, OpenTofu CLI, Git, cosign, sharding, or mesh-runtime calls.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use crate::slash_path;

const DEFAULT_REPO_ROOT: &str = ".";
const DEFAULT_MANIFEST: &str = "cloud/cloud-iac/manifest.json";
const DEFAULT_TOPOLOGY: &str = "cloud/cloud-iac/cell-topology/foundation.json";
const DEFAULT_CATALOG: &str = "cloud/cloud-iac/tofu/modules/catalog.json";
const DEFAULT_TEMPLATES_ROOT: &str = "cloud/cloud-iac/iac";
const GATE_NAME: &str = "cloud-iac-cell-topology";
const GATE_FILE: &str = "cloud/cloud-ci/gates/cloud-iac-cell-topology-gate/src/main.rs";
const RUNTIME_MODE: &str = "local-filesystem-json-cell-topology-gate";
const CHANGESET_ID: &str = "CS-CLOUD-IAC-CELL-TOPOLOGY-GATE-001";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacCellTopologyValidateArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) topology: PathBuf,
    pub(crate) catalog: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacCellTopologyReport {
    pub(crate) manifest_path: String,
    pub(crate) topology_path: String,
    pub(crate) catalog_path: String,
    pub(crate) contexts_checked: usize,
    pub(crate) cells_checked: usize,
    pub(crate) service_tenant_fixtures_checked: usize,
    pub(crate) module_refs_checked: usize,
    pub(crate) files_checked: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct CatalogModuleKey {
    namespace: String,
    name: String,
    system: String,
    version: String,
}

impl CatalogModuleKey {
    fn display_key(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.namespace, self.name, self.system, self.version
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CellModuleRef {
    namespace: String,
    name: String,
    system: String,
    version: String,
}

impl CellModuleRef {
    fn key(&self) -> CatalogModuleKey {
        CatalogModuleKey {
            namespace: self.namespace.clone(),
            name: self.name.clone(),
            system: self.system.clone(),
            version: self.version.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CellRow {
    context: String,
    region: String,
    cell_id: String,
    tenant_id: String,
    isolation_tier: String,
    default_cross_cell_traffic_allowed: bool,
    gitops_template: String,
    evidence_ref: String,
    module_refs: Vec<CellModuleRef>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ServiceTenantTopologyFixture {
    microservice: String,
    tenant_id: String,
    cell_id: String,
    cell_tier: String,
    residency_class: String,
    region_disposition: String,
    storage_class: String,
    quarterly_isolation_evidence: QuarterlyIsolationEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct QuarterlyIsolationEvidence {
    quarter: String,
    network: String,
    storage: String,
    crypto: String,
    compute: String,
    audit: String,
}

pub(crate) fn parse_cloud_iac_cell_topology_validate_args(
    args: Vec<String>,
) -> Result<CloudIacCellTopologyValidateArgs, String> {
    let mut parsed = CloudIacCellTopologyValidateArgs {
        repo_root: PathBuf::from(DEFAULT_REPO_ROOT),
        manifest: PathBuf::from(DEFAULT_MANIFEST),
        topology: PathBuf::from(DEFAULT_TOPOLOGY),
        catalog: PathBuf::from(DEFAULT_CATALOG),
    };

    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--repo-root" => parsed.repo_root = take_path_arg(&mut args, "--repo-root")?,
            "--manifest" => parsed.manifest = take_path_arg(&mut args, "--manifest")?,
            "--topology" => parsed.topology = take_path_arg(&mut args, "--topology")?,
            "--catalog" => parsed.catalog = take_path_arg(&mut args, "--catalog")?,
            other => {
                return Err(format!(
                    "cloud-iac-cell-topology: unknown flag {other:?}; usage: \
                     oya gate validate cloud-iac-cell-topology \
                     [--repo-root <.>] \
                     [--manifest <cloud/cloud-iac/manifest.json>] \
                     [--topology <cloud/cloud-iac/cell-topology/foundation.json>] \
                     [--catalog <cloud/cloud-iac/tofu/modules/catalog.json>]"
                ));
            }
        }
    }

    Ok(parsed)
}

fn take_path_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("cloud-iac-cell-topology: {flag} requires a path argument"))
}

pub(crate) fn validate_cloud_iac_cell_topology_gate(
    args: CloudIacCellTopologyValidateArgs,
) -> Result<CloudIacCellTopologyReport, String> {
    let manifest_path = resolve_repo_path(&args.repo_root, &args.manifest);
    let topology_path = resolve_repo_path(&args.repo_root, &args.topology);
    let catalog_path = resolve_repo_path(&args.repo_root, &args.catalog);
    let manifest_rel = repo_relative_argument(&args.repo_root, &args.manifest)?;
    let topology_rel = repo_relative_argument(&args.repo_root, &args.topology)?;
    let catalog_rel = repo_relative_argument(&args.repo_root, &args.catalog)?;

    let manifest = read_json(&manifest_path, "manifest")?;
    let topology = read_json(&topology_path, "topology")?;
    let catalog = read_json(&catalog_path, "catalog")?;

    let mut diagnostics = Vec::new();
    require_manifest_capability(&manifest, &mut diagnostics);
    require_manifest_gate_guard(&manifest, &mut diagnostics);

    let manifest_topology =
        required_repo_relative_string(&manifest, "/cell_topology_scope/topology", &mut diagnostics);
    if let Some(manifest_topology) = manifest_topology.as_deref()
        && manifest_topology != topology_rel
    {
        diagnostics.push(format!(
            "manifest /cell_topology_scope/topology must equal {topology_rel:?}; found {manifest_topology:?}"
        ));
    }

    let manifest_catalog = required_repo_relative_string(
        &manifest,
        "/cell_topology_scope/module_catalog",
        &mut diagnostics,
    );
    if let Some(manifest_catalog) = manifest_catalog.as_deref()
        && manifest_catalog != catalog_rel
    {
        diagnostics.push(format!(
            "manifest /cell_topology_scope/module_catalog must equal {catalog_rel:?}; found {manifest_catalog:?}"
        ));
    }

    let templates_root = required_repo_relative_string(
        &manifest,
        "/cell_topology_scope/gitops_templates_root",
        &mut diagnostics,
    )
    .unwrap_or_else(|| DEFAULT_TEMPLATES_ROOT.to_string());

    let manifest_contexts =
        required_string_array(&manifest, "/cell_topology_scope/contexts", &mut diagnostics)
            .unwrap_or_default();
    let manifest_regions =
        required_string_array(&manifest, "/cell_topology_scope/regions", &mut diagnostics)
            .unwrap_or_default();
    validate_manifest_summary(
        &manifest,
        &manifest_contexts,
        &manifest_regions,
        &mut diagnostics,
    );
    validate_manifest_modeled_fields(&manifest, &mut diagnostics);
    validate_non_claims(
        &manifest,
        "/cell_topology_scope/non_claims",
        "manifest /cell_topology_scope/non_claims",
        &mut diagnostics,
    );

    let catalog_modules = parse_catalog_module_keys(&catalog, &mut diagnostics);
    validate_topology_header(
        &topology,
        &manifest_rel,
        &catalog_rel,
        &templates_root,
        &mut diagnostics,
    );
    let cells = parse_topology_cells(&topology, &mut diagnostics);
    let service_tenant_fixtures = parse_service_tenant_fixtures(&topology, &mut diagnostics);
    let module_refs_checked = validate_cells(
        &args.repo_root,
        &templates_root,
        &manifest_contexts,
        &catalog_modules,
        &cells,
        &mut diagnostics,
    );
    let service_tenant_fixtures_checked = validate_service_tenant_fixtures(
        &cells,
        &service_tenant_fixtures,
        &mut diagnostics,
    );
    validate_topology_summary(
        &topology,
        &manifest_contexts,
        &manifest_regions,
        &catalog_rel,
        &cells,
        &service_tenant_fixtures,
        &mut diagnostics,
    );
    validate_manifest_topology_counts(&manifest, &cells, &mut diagnostics);
    validate_manifest_service_tenant_fixture_count(
        &manifest,
        &service_tenant_fixtures,
        &mut diagnostics,
    );

    if diagnostics.is_empty() {
        Ok(CloudIacCellTopologyReport {
            manifest_path: manifest_rel,
            topology_path: topology_rel,
            catalog_path: catalog_rel,
            contexts_checked: manifest_contexts.len(),
            cells_checked: cells.len(),
            service_tenant_fixtures_checked,
            module_refs_checked,
            files_checked: cells.len() + service_tenant_fixtures_checked + 3,
        })
    } else {
        Err(format!(
            "cloud-iac-cell-topology validation failed:\n- {}",
            diagnostics.join("\n- ")
        ))
    }
}

fn resolve_repo_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn repo_relative_argument(repo_root: &Path, path: &Path) -> Result<String, String> {
    if path.is_absolute() {
        let repo_root = fs::canonicalize(repo_root).map_err(|error| {
            format!(
                "cloud-iac-cell-topology: unable to canonicalize repo root {}: {error}",
                repo_root.display()
            )
        })?;
        let path = fs::canonicalize(path).map_err(|error| {
            format!(
                "cloud-iac-cell-topology: unable to canonicalize path {}: {error}",
                path.display()
            )
        })?;
        let relative = path.strip_prefix(&repo_root).map_err(|_| {
            format!(
                "cloud-iac-cell-topology: path {} is outside repo root {}",
                path.display(),
                repo_root.display()
            )
        })?;
        strict_repo_relative_path(relative, "absolute CLI path")
    } else {
        strict_repo_relative_path(path, "relative CLI path")
    }
}

fn strict_repo_relative_path(path: &Path, label: &str) -> Result<String, String> {
    let raw = slash_path(path);
    let mut diagnostics = Vec::new();
    let Some(normalized) = normalize_repo_relative(&raw, label, &mut diagnostics) else {
        return Err(diagnostics.join("; "));
    };
    Ok(normalized)
}

fn read_json(path: &Path, label: &str) -> Result<Value, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "cloud-iac-cell-topology: unable to read {label} {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "cloud-iac-cell-topology: unable to parse {label} JSON {}: {error}",
            path.display()
        )
    })
}

fn required_string(value: &Value, pointer: &str, diagnostics: &mut Vec<String>) -> Option<String> {
    match value.pointer(pointer) {
        Some(Value::String(found)) if !found.trim().is_empty() => Some(found.trim().to_string()),
        Some(_) => {
            diagnostics.push(format!("{pointer} must be a non-empty string"));
            None
        }
        None => {
            diagnostics.push(format!("missing required string {pointer}"));
            None
        }
    }
}

fn required_repo_relative_string(
    value: &Value,
    pointer: &str,
    diagnostics: &mut Vec<String>,
) -> Option<String> {
    required_string(value, pointer, diagnostics)
        .and_then(|raw| normalize_repo_relative(&raw, pointer, diagnostics))
}

fn required_bool(value: &Value, pointer: &str, diagnostics: &mut Vec<String>) -> Option<bool> {
    match value.pointer(pointer) {
        Some(Value::Bool(found)) => Some(*found),
        Some(_) => {
            diagnostics.push(format!("{pointer} must be a boolean"));
            None
        }
        None => {
            diagnostics.push(format!("missing required boolean {pointer}"));
            None
        }
    }
}

fn required_u64(value: &Value, pointer: &str, diagnostics: &mut Vec<String>) -> Option<u64> {
    match value.pointer(pointer).and_then(Value::as_u64) {
        Some(found) => Some(found),
        None => {
            diagnostics.push(format!("{pointer} must be an unsigned integer"));
            None
        }
    }
}

fn required_string_array(
    value: &Value,
    pointer: &str,
    diagnostics: &mut Vec<String>,
) -> Option<Vec<String>> {
    let Some(array) = value.pointer(pointer).and_then(Value::as_array) else {
        diagnostics.push(format!("{pointer} must be an array of strings"));
        return None;
    };
    let mut out = Vec::with_capacity(array.len());
    for (idx, entry) in array.iter().enumerate() {
        match entry.as_str() {
            Some(found) if !found.trim().is_empty() => out.push(found.trim().to_string()),
            _ => diagnostics.push(format!("{pointer}/{idx} must be a non-empty string")),
        }
    }
    Some(out)
}

fn normalize_repo_relative(
    raw: &str,
    label: &str,
    diagnostics: &mut Vec<String>,
) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        diagnostics.push(format!("{label} must not be empty"));
        return None;
    }
    if raw.contains('\\') {
        diagnostics.push(format!(
            "{label} must use slash-separated repo-relative paths"
        ));
        return None;
    }

    let path = Path::new(raw);
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                let Some(part) = part.to_str() else {
                    diagnostics.push(format!("{label} contains non-UTF-8 path component"));
                    return None;
                };
                if part.is_empty() {
                    diagnostics.push(format!("{label} contains an empty path component"));
                    return None;
                }
                parts.push(part.to_string());
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                diagnostics.push(format!(
                    "{label} must be repo-relative and must not contain .."
                ));
                return None;
            }
        }
    }

    if parts.is_empty() {
        diagnostics.push(format!("{label} must include at least one path component"));
        None
    } else {
        Some(parts.join("/"))
    }
}

fn require_manifest_capability(manifest: &Value, diagnostics: &mut Vec<String>) {
    let Some(capabilities) = manifest.pointer("/capabilities").and_then(Value::as_array) else {
        diagnostics.push("manifest /capabilities must be an array".to_string());
        return;
    };
    let has_gate_capability = capabilities.iter().any(|capability| {
        capability.get("name").and_then(Value::as_str) == Some("cloud-iac-cell-topology-gate")
            && capability.get("file").and_then(Value::as_str) == Some(GATE_FILE)
    });
    if !has_gate_capability {
        diagnostics.push(format!(
            "manifest capabilities must declare cloud-iac-cell-topology-gate backed by {GATE_FILE}"
        ));
    }
}

fn require_manifest_gate_guard(manifest: &Value, diagnostics: &mut Vec<String>) {
    let gate = required_string(
        manifest,
        "/cell_topology_scope/coherence_guard/gate",
        diagnostics,
    );
    if gate.as_deref() != Some(GATE_NAME) {
        diagnostics.push(format!(
            "manifest /cell_topology_scope/coherence_guard/gate must be {GATE_NAME:?}"
        ));
    }
    let mode = required_string(
        manifest,
        "/cell_topology_scope/coherence_guard/runtime_mode",
        diagnostics,
    );
    if mode.as_deref() != Some(RUNTIME_MODE) {
        diagnostics.push(format!(
            "manifest /cell_topology_scope/coherence_guard/runtime_mode must be {RUNTIME_MODE:?}"
        ));
    }
}

fn validate_manifest_summary(
    manifest: &Value,
    contexts: &[String],
    regions: &[String],
    diagnostics: &mut Vec<String>,
) {
    validate_sorted_array("manifest contexts", contexts, diagnostics);
    validate_sorted_array("manifest regions", regions, diagnostics);
    for context in contexts {
        if !is_slug(context) {
            diagnostics.push(format!(
                "manifest /cell_topology_scope/contexts entry {context:?} must be lowercase/digit/hyphen"
            ));
        }
    }
    for region in regions {
        if !is_slug(region) {
            diagnostics.push(format!(
                "manifest /cell_topology_scope/regions entry {region:?} must be lowercase/digit/hyphen"
            ));
        }
    }
    if contexts.is_empty() {
        diagnostics.push("manifest /cell_topology_scope/contexts must not be empty".to_string());
    }
    if regions.is_empty() {
        diagnostics.push("manifest /cell_topology_scope/regions must not be empty".to_string());
    }

    if let Some(count) = required_u64(manifest, "/cell_topology_scope/context_count", diagnostics)
        && count != contexts.len() as u64
    {
        diagnostics.push(format!(
            "manifest /cell_topology_scope/context_count must equal contexts length {}; found {count}",
            contexts.len()
        ));
    }
}

fn validate_manifest_topology_counts(
    manifest: &Value,
    cells: &[CellRow],
    diagnostics: &mut Vec<String>,
) {
    if let Some(count) = required_u64(manifest, "/cell_topology_scope/cell_count", diagnostics)
        && count != cells.len() as u64
    {
        diagnostics.push(format!(
            "manifest /cell_topology_scope/cell_count must equal topology cell count {}; found {count}",
            cells.len()
        ));
    }
}

fn validate_manifest_service_tenant_fixture_count(
    manifest: &Value,
    fixtures: &[ServiceTenantTopologyFixture],
    diagnostics: &mut Vec<String>,
) {
    if let Some(count) = required_u64(
        manifest,
        "/cell_topology_scope/service_tenant_fixture_count",
        diagnostics,
    ) && count != fixtures.len() as u64
    {
        diagnostics.push(format!(
            "manifest /cell_topology_scope/service_tenant_fixture_count must equal topology service_tenant_fixtures count {}; found {count}",
            fixtures.len()
        ));
    }
}

fn validate_manifest_modeled_fields(manifest: &Value, diagnostics: &mut Vec<String>) {
    let Some(modeled_fields) = required_string_array(
        manifest,
        "/cell_topology_scope/topology_fields_modeled",
        diagnostics,
    ) else {
        return;
    };

    for required in [
        "topology_id",
        "runtime_mode",
        "contexts",
        "regions",
        "cells.context",
        "cells.region",
        "cells.cell_id",
        "cells.tenant_id",
        "cells.isolation_tier",
        "cells.default_cross_cell_traffic_allowed",
        "cells.module_refs",
        "cells.gitops_template",
        "cells.evidence_ref",
        "service_tenant_fixtures.microservice",
        "service_tenant_fixtures.tenant_id",
        "service_tenant_fixtures.cell_id",
        "service_tenant_fixtures.cell_tier",
        "service_tenant_fixtures.residency_class",
        "service_tenant_fixtures.region_disposition",
        "service_tenant_fixtures.storage_class",
        "service_tenant_fixtures.quarterly_isolation_evidence.quarter",
        "service_tenant_fixtures.quarterly_isolation_evidence.network",
        "service_tenant_fixtures.quarterly_isolation_evidence.storage",
        "service_tenant_fixtures.quarterly_isolation_evidence.crypto",
        "service_tenant_fixtures.quarterly_isolation_evidence.compute",
        "service_tenant_fixtures.quarterly_isolation_evidence.audit",
    ] {
        if !modeled_fields.iter().any(|field| field == required) {
            diagnostics.push(format!(
                "manifest /cell_topology_scope/topology_fields_modeled must include {required:?}"
            ));
        }
    }
}

fn validate_topology_header(
    topology: &Value,
    manifest_rel: &str,
    catalog_rel: &str,
    templates_root: &str,
    diagnostics: &mut Vec<String>,
) {
    let schema_version = required_string(topology, "/schema_version", diagnostics);
    if schema_version.as_deref() != Some("1.0") {
        diagnostics.push("topology /schema_version must be \"1.0\"".to_string());
    }
    let topology_id = required_string(topology, "/topology_id", diagnostics);
    if let Some(topology_id) = topology_id.as_deref()
        && !is_slug(topology_id)
    {
        diagnostics.push(format!(
            "topology_id {topology_id:?} must be a lowercase slug"
        ));
    }
    let runtime_mode = required_string(topology, "/runtime_mode", diagnostics);
    if runtime_mode.as_deref() != Some(RUNTIME_MODE) {
        diagnostics.push(format!("topology /runtime_mode must be {RUNTIME_MODE:?}"));
    }
    let generated_by = required_string(topology, "/generated_by_changeset", diagnostics);
    if generated_by.as_deref() != Some(CHANGESET_ID) {
        diagnostics.push(format!(
            "topology /generated_by_changeset must be {CHANGESET_ID:?}"
        ));
    }

    let manifest = required_repo_relative_string(topology, "/authority/manifest", diagnostics);
    if manifest.as_deref() != Some(manifest_rel) {
        diagnostics.push(format!(
            "topology /authority/manifest must equal {manifest_rel:?}"
        ));
    }
    let catalog = required_repo_relative_string(topology, "/authority/module_catalog", diagnostics);
    if catalog.as_deref() != Some(catalog_rel) {
        diagnostics.push(format!(
            "topology /authority/module_catalog must equal {catalog_rel:?}"
        ));
    }
    let topology_templates =
        required_repo_relative_string(topology, "/authority/gitops_templates_root", diagnostics);
    if topology_templates.as_deref() != Some(templates_root) {
        diagnostics.push(format!(
            "topology /authority/gitops_templates_root must equal {templates_root:?}"
        ));
    }

    validate_non_claims(
        topology,
        "/authority/non_claims",
        "topology /authority/non_claims",
        diagnostics,
    );
    validate_non_claims(topology, "/non_claims", "topology /non_claims", diagnostics);
}

fn validate_non_claims(value: &Value, pointer: &str, label: &str, diagnostics: &mut Vec<String>) {
    let Some(non_claims) = value.pointer(pointer).and_then(Value::as_array) else {
        diagnostics.push(format!("{label} must be an array"));
        return;
    };
    if non_claims.is_empty() {
        diagnostics.push(format!("{label} must be a non-empty array"));
        return;
    }
    let joined = non_claims
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join("\n")
        .to_ascii_lowercase();
    for required in [
        "no autosharding",
        "no argocd api",
        "no opentofu cli",
        "no provider api",
    ] {
        if !joined.contains(required) {
            diagnostics.push(format!("{label} must include {required:?}"));
        }
    }
}

fn parse_catalog_module_keys(
    catalog: &Value,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<CatalogModuleKey, String> {
    let Some(modules) = catalog.pointer("/modules").and_then(Value::as_array) else {
        diagnostics.push("catalog /modules must be a non-empty array".to_string());
        return BTreeMap::new();
    };
    if modules.is_empty() {
        diagnostics.push("catalog /modules must be a non-empty array".to_string());
        return BTreeMap::new();
    }

    let mut keys = BTreeMap::new();
    for (idx, module) in modules.iter().enumerate() {
        let prefix = format!("/modules/{idx}");
        let namespace = required_string(module, "/namespace", diagnostics);
        let name = required_string(module, "/name", diagnostics);
        let system = required_string(module, "/system", diagnostics);
        let version = required_string(module, "/version", diagnostics);
        let source_path = required_repo_relative_string(module, "/source_path", diagnostics);
        let Some(key) = namespace.zip(name).zip(system).zip(version).map(
            |(((namespace, name), system), version)| CatalogModuleKey {
                namespace,
                name,
                system,
                version,
            },
        ) else {
            continue;
        };
        if !is_slug(&key.namespace) {
            diagnostics.push(format!(
                "catalog {prefix}/namespace must be a lowercase slug"
            ));
        }
        if !is_slug(&key.name) {
            diagnostics.push(format!("catalog {prefix}/name must be a lowercase slug"));
        }
        if !is_slug(&key.system) {
            diagnostics.push(format!("catalog {prefix}/system must be a lowercase slug"));
        }
        if !is_exact_semver(&key.version) {
            diagnostics.push(format!(
                "catalog {prefix}/version {:?} must be exact MAJOR.MINOR.PATCH semver",
                key.version
            ));
        }
        if let Some(previous) = keys.insert(
            key.clone(),
            source_path.unwrap_or_else(|| "<missing-source-path>".to_string()),
        ) {
            diagnostics.push(format!(
                "catalog {prefix} duplicates module key {:?}; previous source {previous}",
                key.display_key()
            ));
        }
    }
    keys
}

fn parse_topology_cells(topology: &Value, diagnostics: &mut Vec<String>) -> Vec<CellRow> {
    let Some(cells) = topology.pointer("/cells").and_then(Value::as_array) else {
        diagnostics.push("topology /cells must be a non-empty array".to_string());
        return Vec::new();
    };
    if cells.is_empty() {
        diagnostics.push("topology /cells must be a non-empty array".to_string());
        return Vec::new();
    }

    let mut rows = Vec::with_capacity(cells.len());
    for (idx, cell) in cells.iter().enumerate() {
        let prefix = format!("/cells/{idx}");
        let context = required_string(cell, "/context", diagnostics);
        let region = required_string(cell, "/region", diagnostics);
        let cell_id = required_string(cell, "/cell_id", diagnostics);
        let tenant_id = required_string(cell, "/tenant_id", diagnostics);
        let isolation_tier = required_string(cell, "/isolation_tier", diagnostics);
        let default_cross_cell_traffic_allowed =
            required_bool(cell, "/default_cross_cell_traffic_allowed", diagnostics);
        let gitops_template = required_repo_relative_string(cell, "/gitops_template", diagnostics);
        let evidence_ref = required_string(cell, "/evidence_ref", diagnostics);
        let module_refs = parse_cell_module_refs(cell, &prefix, diagnostics);

        let Some(row) = context
            .zip(region)
            .zip(cell_id)
            .zip(tenant_id)
            .zip(isolation_tier)
            .zip(default_cross_cell_traffic_allowed)
            .zip(gitops_template)
            .zip(evidence_ref)
            .map(
                |(
                    (
                        (
                            ((((context, region), cell_id), tenant_id), isolation_tier),
                            default_cross_cell_traffic_allowed,
                        ),
                        gitops_template,
                    ),
                    evidence_ref,
                )| CellRow {
                    context,
                    region,
                    cell_id,
                    tenant_id,
                    isolation_tier,
                    default_cross_cell_traffic_allowed,
                    gitops_template,
                    evidence_ref,
                    module_refs,
                },
            )
        else {
            continue;
        };
        rows.push(row);
    }
    rows
}

fn parse_cell_module_refs(
    cell: &Value,
    prefix: &str,
    diagnostics: &mut Vec<String>,
) -> Vec<CellModuleRef> {
    let Some(module_refs) = cell.pointer("/module_refs").and_then(Value::as_array) else {
        diagnostics.push(format!("{prefix}/module_refs must be a non-empty array"));
        return Vec::new();
    };
    if module_refs.is_empty() {
        diagnostics.push(format!("{prefix}/module_refs must be a non-empty array"));
        return Vec::new();
    }

    let mut refs = Vec::with_capacity(module_refs.len());
    let mut seen = BTreeSet::new();
    for (idx, module_ref) in module_refs.iter().enumerate() {
        let ref_prefix = format!("{prefix}/module_refs/{idx}");
        let namespace = required_string(module_ref, "/namespace", diagnostics);
        let name = required_string(module_ref, "/name", diagnostics);
        let system = required_string(module_ref, "/system", diagnostics);
        let version = required_string(module_ref, "/version", diagnostics);
        let Some(row) = namespace.zip(name).zip(system).zip(version).map(
            |(((namespace, name), system), version)| CellModuleRef {
                namespace,
                name,
                system,
                version,
            },
        ) else {
            continue;
        };
        let key = row.key().display_key();
        if !seen.insert(key.clone()) {
            diagnostics.push(format!("{ref_prefix} duplicates module ref {key:?}"));
        }
        refs.push(row);
    }
    refs
}

fn parse_service_tenant_fixtures(
    topology: &Value,
    diagnostics: &mut Vec<String>,
) -> Vec<ServiceTenantTopologyFixture> {
    let Some(fixtures) = topology
        .pointer("/service_tenant_fixtures")
        .and_then(Value::as_array)
    else {
        diagnostics.push("topology /service_tenant_fixtures must be a non-empty array".to_string());
        return Vec::new();
    };
    if fixtures.is_empty() {
        diagnostics.push("topology /service_tenant_fixtures must be a non-empty array".to_string());
        return Vec::new();
    }

    let mut rows = Vec::with_capacity(fixtures.len());
    for (idx, fixture) in fixtures.iter().enumerate() {
        let prefix = format!("/service_tenant_fixtures/{idx}");
        let microservice = required_string(fixture, "/microservice", diagnostics);
        let tenant_id = required_string(fixture, "/tenant_id", diagnostics);
        let cell_id = required_string(fixture, "/cell_id", diagnostics);
        let cell_tier = required_string(fixture, "/cell_tier", diagnostics);
        let residency_class = required_string(fixture, "/residency_class", diagnostics);
        let region_disposition = required_string(fixture, "/region_disposition", diagnostics);
        let storage_class = required_string(fixture, "/storage_class", diagnostics);
        let quarterly_isolation_evidence =
            parse_quarterly_isolation_evidence(fixture, &prefix, diagnostics);

        if let (
            Some(microservice),
            Some(tenant_id),
            Some(cell_id),
            Some(cell_tier),
            Some(residency_class),
            Some(region_disposition),
            Some(storage_class),
            Some(quarterly_isolation_evidence),
        ) = (
            microservice,
            tenant_id,
            cell_id,
            cell_tier,
            residency_class,
            region_disposition,
            storage_class,
            quarterly_isolation_evidence,
        ) {
            rows.push(ServiceTenantTopologyFixture {
                microservice,
                tenant_id,
                cell_id,
                cell_tier,
                residency_class,
                region_disposition,
                storage_class,
                quarterly_isolation_evidence,
            });
        }
    }
    rows
}

fn parse_quarterly_isolation_evidence(
    fixture: &Value,
    prefix: &str,
    diagnostics: &mut Vec<String>,
) -> Option<QuarterlyIsolationEvidence> {
    let Some(evidence) = fixture.pointer("/quarterly_isolation_evidence") else {
        diagnostics.push(format!(
            "{prefix}/quarterly_isolation_evidence must be an object"
        ));
        return None;
    };

    let quarter = required_string(evidence, "/quarter", diagnostics);
    let network = required_string(evidence, "/network", diagnostics);
    let storage = required_string(evidence, "/storage", diagnostics);
    let crypto = required_string(evidence, "/crypto", diagnostics);
    let compute = required_string(evidence, "/compute", diagnostics);
    let audit = required_string(evidence, "/audit", diagnostics);

    if let (Some(quarter), Some(network), Some(storage), Some(crypto), Some(compute), Some(audit)) =
        (quarter, network, storage, crypto, compute, audit)
    {
        Some(QuarterlyIsolationEvidence {
            quarter,
            network,
            storage,
            crypto,
            compute,
            audit,
        })
    } else {
        None
    }
}

fn validate_cells(
    repo_root: &Path,
    templates_root: &str,
    allowed_contexts: &[String],
    catalog_modules: &BTreeMap<CatalogModuleKey, String>,
    cells: &[CellRow],
    diagnostics: &mut Vec<String>,
) -> usize {
    let allowed_contexts = allowed_contexts.iter().cloned().collect::<BTreeSet<_>>();
    let mut seen_cells = BTreeSet::new();
    let mut seen_contexts = BTreeSet::new();
    let mut seen_module_keys = BTreeSet::new();
    let mut module_refs_checked = 0usize;

    for (idx, cell) in cells.iter().enumerate() {
        let prefix = format!("/cells/{idx}");
        if !allowed_contexts.contains(&cell.context) {
            diagnostics.push(format!(
                "{prefix}/context {:?} is not declared by manifest /cell_topology_scope/contexts",
                cell.context
            ));
        }
        if !is_slug(&cell.context) {
            diagnostics.push(format!("{prefix}/context must be lowercase/digit/hyphen"));
        }
        if !is_slug(&cell.region) {
            diagnostics.push(format!("{prefix}/region must be lowercase/digit/hyphen"));
        }
        if !is_slug(&cell.cell_id) {
            diagnostics.push(format!("{prefix}/cell_id must be lowercase/digit/hyphen"));
        }
        if !is_tenant_id(&cell.tenant_id) {
            diagnostics.push(format!(
                "{prefix}/tenant_id must be lowercase/digit/hyphen/underscore"
            ));
        }
        if !is_isolation_tier(&cell.isolation_tier) {
            diagnostics.push(format!(
                "{prefix}/isolation_tier {:?} is not one of foundation/substrate/capability/application/edge",
                cell.isolation_tier
            ));
        }
        if cell.default_cross_cell_traffic_allowed {
            diagnostics.push(format!(
                "{prefix}/default_cross_cell_traffic_allowed must remain false for local topology evidence"
            ));
        }
        if !seen_cells.insert(cell.cell_id.clone()) {
            diagnostics.push(format!(
                "{prefix}/cell_id {:?} duplicates another cell",
                cell.cell_id
            ));
        }
        seen_contexts.insert(cell.context.clone());

        let expected_template = format!(
            "{templates_root}/{}/argocd/apps/template.yaml",
            cell.context
        );
        if cell.gitops_template != expected_template {
            diagnostics.push(format!(
                "{prefix}/gitops_template must be {expected_template:?}; found {:?}",
                cell.gitops_template
            ));
        }
        if !repo_root.join(&cell.gitops_template).is_file() {
            diagnostics.push(format!(
                "{prefix}/gitops_template does not exist: {}",
                repo_root.join(&cell.gitops_template).display()
            ));
        } else {
            validate_gitops_template_cell_identity(repo_root, &prefix, cell, diagnostics);
        }

        let evidence_prefix = format!(
            "evidence://cloud-iac/cell-topology/{}/{}/{}",
            cell.context, cell.region, cell.cell_id
        );
        if !cell.evidence_ref.starts_with(&evidence_prefix) {
            diagnostics.push(format!(
                "{prefix}/evidence_ref must start with {evidence_prefix:?}"
            ));
        }
        if contains_secret_like_marker(&cell.evidence_ref) {
            diagnostics.push(format!(
                "{prefix}/evidence_ref contains secret-like material marker"
            ));
        }

        for (module_idx, module_ref) in cell.module_refs.iter().enumerate() {
            let module_prefix = format!("{prefix}/module_refs/{module_idx}");
            module_refs_checked += 1;
            if !is_slug(&module_ref.namespace) {
                diagnostics.push(format!("{module_prefix}/namespace must be lowercase slug"));
            }
            if !is_slug(&module_ref.name) {
                diagnostics.push(format!("{module_prefix}/name must be lowercase slug"));
            }
            if !is_slug(&module_ref.system) {
                diagnostics.push(format!("{module_prefix}/system must be lowercase slug"));
            }
            if !is_exact_semver(&module_ref.version) {
                diagnostics.push(format!(
                    "{module_prefix}/version {:?} must be exact MAJOR.MINOR.PATCH semver",
                    module_ref.version
                ));
            }
            let key = module_ref.key();
            if !catalog_modules.contains_key(&key) {
                diagnostics.push(format!(
                    "{module_prefix} references module {:?} not present in catalog",
                    key.display_key()
                ));
            }
            seen_module_keys.insert(key);
        }
    }

    let declared_contexts = allowed_contexts;
    if !declared_contexts.is_empty() && seen_contexts != declared_contexts {
        diagnostics.push(format!(
            "topology cells must cover each manifest context exactly once-or-more; expected {:?}, found {:?}",
            declared_contexts, seen_contexts
        ));
    }
    let catalog_keys = catalog_modules.keys().cloned().collect::<BTreeSet<_>>();
    if !catalog_keys.is_empty() && seen_module_keys != catalog_keys {
        diagnostics.push(format!(
            "topology module_refs must collectively cover catalog modules {:?}; found {:?}",
            catalog_keys
                .iter()
                .map(CatalogModuleKey::display_key)
                .collect::<Vec<_>>(),
            seen_module_keys
                .iter()
                .map(CatalogModuleKey::display_key)
                .collect::<Vec<_>>()
        ));
    }

    module_refs_checked
}

fn validate_service_tenant_fixtures(
    cells: &[CellRow],
    fixtures: &[ServiceTenantTopologyFixture],
    diagnostics: &mut Vec<String>,
) -> usize {
    let mut seen = BTreeSet::new();
    for (idx, fixture) in fixtures.iter().enumerate() {
        let prefix = format!("/service_tenant_fixtures/{idx}");
        if !seen.insert((
            fixture.microservice.clone(),
            fixture.tenant_id.clone(),
            fixture.cell_id.clone(),
        )) {
            diagnostics.push(format!(
                "{prefix} duplicates a microservice/tenant_id/cell_id contract fixture"
            ));
        }
        if !is_slug(&fixture.microservice) {
            diagnostics.push(format!("{prefix}/microservice must be lowercase/digit/hyphen"));
        }
        if !is_tenant_id(&fixture.tenant_id) {
            diagnostics.push(format!(
                "{prefix}/tenant_id must be lowercase/digit/hyphen/underscore"
            ));
        }
        if !is_slug(&fixture.cell_id) {
            diagnostics.push(format!("{prefix}/cell_id must be lowercase/digit/hyphen"));
        }
        let matched_cell = cells
            .iter()
            .find(|cell| cell.tenant_id == fixture.tenant_id && cell.cell_id == fixture.cell_id);
        if matched_cell.is_none() {
            diagnostics.push(format!(
                "{prefix} must reference an existing topology cell by tenant_id {:?} and cell_id {:?}",
                fixture.tenant_id, fixture.cell_id
            ));
        }
        if !is_cell_tier(&fixture.cell_tier) {
            diagnostics.push(format!(
                "{prefix}/cell_tier {:?} is not one of dedicated/shared-small/shared-medium/shared-large/foundry-runtime/public-corpus",
                fixture.cell_tier
            ));
        }
        if !is_residency_class(&fixture.residency_class) {
            diagnostics.push(format!(
                "{prefix}/residency_class {:?} is not a canonical ADR-0049 residency class",
                fixture.residency_class
            ));
        }
        if !is_region_disposition(&fixture.region_disposition) {
            diagnostics.push(format!(
                "{prefix}/region_disposition {:?} is not one of active_active/active_passive/single_region",
                fixture.region_disposition
            ));
        }
        if !is_storage_class(&fixture.storage_class) {
            diagnostics.push(format!(
                "{prefix}/storage_class {:?} must follow oya-<pg|s3|redis|object>-<hot|warm|cold>",
                fixture.storage_class
            ));
        }
        validate_quarterly_isolation_evidence(
            &prefix,
            &fixture.quarterly_isolation_evidence,
            matched_cell,
            diagnostics,
        );
    }
    fixtures.len()
}

fn validate_quarterly_isolation_evidence(
    prefix: &str,
    evidence: &QuarterlyIsolationEvidence,
    matched_cell: Option<&CellRow>,
    diagnostics: &mut Vec<String>,
) {
    if !is_quarter_slug(&evidence.quarter) {
        diagnostics.push(format!(
            "{prefix}/quarterly_isolation_evidence/quarter {:?} must use yyyynq form such as 2026q2",
            evidence.quarter
        ));
    }

    let expected_cell_prefix = matched_cell.map(|cell| {
        format!(
            "evidence://cloud-iac/cell-topology/{}/{}/{}",
            cell.context, cell.region, cell.cell_id
        )
    });
    for (field, value) in [
        ("network", &evidence.network),
        ("storage", &evidence.storage),
        ("crypto", &evidence.crypto),
        ("compute", &evidence.compute),
        ("audit", &evidence.audit),
    ] {
        if !value.starts_with("evidence://") {
            diagnostics.push(format!(
                "{prefix}/quarterly_isolation_evidence/{field} must be an evidence:// reference"
            ));
        }
        if !value.contains("/quarterly-isolation/") {
            diagnostics.push(format!(
                "{prefix}/quarterly_isolation_evidence/{field} must reference quarterly-isolation evidence"
            ));
        }
        if !value.contains(&evidence.quarter) {
            diagnostics.push(format!(
                "{prefix}/quarterly_isolation_evidence/{field} must include quarter {:?}",
                evidence.quarter
            ));
        }
        if let Some(expected_cell_prefix) = expected_cell_prefix.as_deref()
            && !value.starts_with(expected_cell_prefix)
        {
            diagnostics.push(format!(
                "{prefix}/quarterly_isolation_evidence/{field} must start with {expected_cell_prefix:?}"
            ));
        }
        if contains_secret_like_marker(value) {
            diagnostics.push(format!(
                "{prefix}/quarterly_isolation_evidence/{field} contains secret-like material marker"
            ));
        }
    }
}

fn validate_gitops_template_cell_identity(
    repo_root: &Path,
    prefix: &str,
    cell: &CellRow,
    diagnostics: &mut Vec<String>,
) {
    let template_path = repo_root.join(&cell.gitops_template);
    let contents = match fs::read_to_string(&template_path) {
        Ok(contents) => contents,
        Err(error) => {
            diagnostics.push(format!(
                "{prefix}/gitops_template unable to read {}: {error}",
                template_path.display()
            ));
            return;
        }
    };

    for (field, required_line) in [
        ("region", format!("oyatie.com/region: \"{}\"", cell.region)),
        (
            "cell_id",
            format!("oyatie.com/cell-id: \"{}\"", cell.cell_id),
        ),
        (
            "tenant_id",
            format!("oyatie.com/tenant-id: \"{}\"", cell.tenant_id),
        ),
        (
            "isolation_tier",
            format!("oyatie.com/isolation-tier: \"{}\"", cell.isolation_tier),
        ),
        (
            "default_cross_cell_traffic_allowed",
            format!(
                "oyatie.com/default-cross-cell-traffic-allowed: \"{}\"",
                cell.default_cross_cell_traffic_allowed
            ),
        ),
    ] {
        if !contains_trimmed_line(&contents, &required_line) {
            diagnostics.push(format!(
                "{prefix}/gitops_template must carry topology cell {field} metadata line {required_line:?}"
            ));
        }
    }
}

fn validate_topology_summary(
    topology: &Value,
    manifest_contexts: &[String],
    manifest_regions: &[String],
    catalog_rel: &str,
    cells: &[CellRow],
    service_tenant_fixtures: &[ServiceTenantTopologyFixture],
    diagnostics: &mut Vec<String>,
) {
    if let Some(count) = required_u64(topology, "/summary/context_count", diagnostics)
        && count != manifest_contexts.len() as u64
    {
        diagnostics.push(format!(
            "topology /summary/context_count must equal manifest context count {}; found {count}",
            manifest_contexts.len()
        ));
    }
    if let Some(count) = required_u64(topology, "/summary/cell_count", diagnostics)
        && count != cells.len() as u64
    {
        diagnostics.push(format!(
            "topology /summary/cell_count must equal cells length {}; found {count}",
            cells.len()
        ));
    }
    if let Some(count) = required_u64(
        topology,
        "/summary/service_tenant_fixture_count",
        diagnostics,
    ) && count != service_tenant_fixtures.len() as u64
    {
        diagnostics.push(format!(
            "topology /summary/service_tenant_fixture_count must equal service_tenant_fixtures length {}; found {count}",
            service_tenant_fixtures.len()
        ));
    }

    let topology_contexts =
        required_string_array(topology, "/summary/contexts", diagnostics).unwrap_or_default();
    if topology_contexts != manifest_contexts {
        diagnostics.push(format!(
            "topology /summary/contexts must equal manifest contexts {:?}; found {:?}",
            manifest_contexts, topology_contexts
        ));
    }

    let actual_contexts = sorted_unique(cells.iter().map(|cell| cell.context.clone()));
    if actual_contexts != manifest_contexts {
        diagnostics.push(format!(
            "topology cells contexts must equal manifest contexts {:?}; found {:?}",
            manifest_contexts, actual_contexts
        ));
    }

    let topology_regions =
        required_string_array(topology, "/summary/regions", diagnostics).unwrap_or_default();
    if topology_regions != manifest_regions {
        diagnostics.push(format!(
            "topology /summary/regions must equal manifest regions {:?}; found {:?}",
            manifest_regions, topology_regions
        ));
    }

    let actual_regions = sorted_unique(cells.iter().map(|cell| cell.region.clone()));
    if actual_regions != manifest_regions {
        diagnostics.push(format!(
            "topology cells regions must equal manifest regions {:?}; found {:?}",
            manifest_regions, actual_regions
        ));
    }

    let summary_catalog =
        required_repo_relative_string(topology, "/summary/module_catalog", diagnostics);
    if summary_catalog.as_deref() != Some(catalog_rel) {
        diagnostics.push(format!(
            "topology /summary/module_catalog must equal {catalog_rel:?}"
        ));
    }
}

fn sorted_unique(values: impl Iterator<Item = String>) -> Vec<String> {
    values.collect::<BTreeSet<_>>().into_iter().collect()
}

fn contains_trimmed_line(contents: &str, needle: &str) -> bool {
    contents.lines().any(|line| line.trim() == needle)
}

fn validate_sorted_array(label: &str, values: &[String], diagnostics: &mut Vec<String>) {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted.dedup();
    if values != sorted {
        diagnostics.push(format!(
            "{label} must be sorted ascending with no duplicates; expected {:?}, found {:?}",
            sorted, values
        ));
    }
}

fn is_slug(value: &str) -> bool {
    let mut previous_dash = false;
    let mut saw_char = false;
    for ch in value.chars() {
        let valid = ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-';
        if !valid {
            return false;
        }
        if ch == '-' && (!saw_char || previous_dash) {
            return false;
        }
        previous_dash = ch == '-';
        saw_char = true;
    }
    saw_char && !previous_dash
}

fn is_tenant_id(value: &str) -> bool {
    let mut saw_char = false;
    for ch in value.chars() {
        let valid = ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_';
        if !valid {
            return false;
        }
        saw_char = true;
    }
    saw_char
}

fn is_isolation_tier(value: &str) -> bool {
    matches!(
        value,
        "foundation" | "substrate" | "capability" | "application" | "edge"
    )
}

fn is_cell_tier(value: &str) -> bool {
    matches!(
        value,
        "dedicated"
            | "shared-small"
            | "shared-medium"
            | "shared-large"
            | "foundry-runtime"
            | "public-corpus"
    )
}

fn is_residency_class(value: &str) -> bool {
    matches!(value, "strict_kr" | "kr_with_us_failover" | "global")
        || value
            .strip_prefix("per_pack_")
            .is_some_and(|suffix| is_tenant_id(suffix) && suffix.contains('_'))
}

fn is_region_disposition(value: &str) -> bool {
    matches!(value, "active_active" | "active_passive" | "single_region")
}

fn is_storage_class(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("oya-") else {
        return false;
    };
    let parts = rest.split('-').collect::<Vec<_>>();
    parts.len() == 2
        && matches!(parts[0], "pg" | "s3" | "redis" | "object")
        && matches!(parts[1], "hot" | "warm" | "cold")
}

fn is_quarter_slug(value: &str) -> bool {
    let mut chars = value.chars();
    chars.by_ref().take(4).all(|ch| ch.is_ascii_digit())
        && chars.next() == Some('q')
        && matches!(chars.next(), Some('1' | '2' | '3' | '4'))
        && chars.next().is_none()
}

fn is_exact_semver(version: &str) -> bool {
    let parts = version.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
}

fn contains_secret_like_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "password=",
        "password:",
        "token=",
        "token:",
        "private_key",
        "private-key",
        "kubeconfig:",
        "bearer ",
        "-----begin",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parse_cloud_iac_cell_topology_defaults_to_live_paths() {
        let parsed = parse_cloud_iac_cell_topology_validate_args(Vec::new()).expect("defaults");
        assert_eq!(parsed.repo_root, PathBuf::from(DEFAULT_REPO_ROOT));
        assert_eq!(parsed.manifest, PathBuf::from(DEFAULT_MANIFEST));
        assert_eq!(parsed.topology, PathBuf::from(DEFAULT_TOPOLOGY));
        assert_eq!(parsed.catalog, PathBuf::from(DEFAULT_CATALOG));
    }

    #[test]
    fn parse_cloud_iac_cell_topology_rejects_unknown_flag() {
        let error = parse_cloud_iac_cell_topology_validate_args(vec!["--bogus".into()])
            .expect_err("unknown flag rejected");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn cloud_iac_cell_topology_gate_accepts_coherent_fixture() {
        let temp = TempRepo::new("cloud-iac-cell-topology-valid");
        write_fixture(temp.path(), FixtureDrift::None);

        let report = validate_cloud_iac_cell_topology_gate(fixture_args(temp.path()))
            .expect("coherent fixture passes");

        assert_eq!(report.contexts_checked, 2);
        assert_eq!(report.cells_checked, 2);
        assert_eq!(report.module_refs_checked, 4);
    }

    #[test]
    fn cloud_iac_cell_topology_gate_rejects_cross_cell_traffic_overclaim() {
        let temp = TempRepo::new("cloud-iac-cell-topology-cross-cell");
        write_fixture(temp.path(), FixtureDrift::CrossCellTrafficAllowed);

        let error = validate_cloud_iac_cell_topology_gate(fixture_args(temp.path()))
            .expect_err("cross-cell traffic overclaim fails");

        assert!(error.contains("default_cross_cell_traffic_allowed"));
    }

    #[test]
    fn cloud_iac_cell_topology_gate_rejects_missing_catalog_module_ref() {
        let temp = TempRepo::new("cloud-iac-cell-topology-missing-module");
        write_fixture(temp.path(), FixtureDrift::MissingCatalogModuleRef);

        let error = validate_cloud_iac_cell_topology_gate(fixture_args(temp.path()))
            .expect_err("missing catalog module fails");

        assert!(error.contains("not present in catalog"));
    }

    #[test]
    fn cloud_iac_cell_topology_gate_rejects_manifest_context_drift() {
        let temp = TempRepo::new("cloud-iac-cell-topology-context-drift");
        write_fixture(temp.path(), FixtureDrift::ManifestContextDrift);

        let error = validate_cloud_iac_cell_topology_gate(fixture_args(temp.path()))
            .expect_err("manifest context drift fails");

        assert!(error.contains("contexts"));
    }

    #[test]
    fn cloud_iac_cell_topology_gate_rejects_secret_marker() {
        let temp = TempRepo::new("cloud-iac-cell-topology-secret");
        write_fixture(temp.path(), FixtureDrift::SecretMarker);

        let error = validate_cloud_iac_cell_topology_gate(fixture_args(temp.path()))
            .expect_err("secret marker fails");

        assert!(error.contains("secret-like material"));
    }

    #[test]
    fn cloud_iac_cell_topology_gate_rejects_gitops_template_identity_drift() {
        let temp = TempRepo::new("cloud-iac-cell-topology-gitops-identity");
        write_fixture(temp.path(), FixtureDrift::GitOpsTemplateIdentityDrift);

        let error = validate_cloud_iac_cell_topology_gate(fixture_args(temp.path()))
            .expect_err("gitops template cell identity drift fails");

        assert!(error.contains("gitops_template"));
        assert!(error.contains("cell_id"));
    }

    #[test]
    fn cloud_iac_cell_topology_gate_rejects_missing_service_tenant_contract_fixture() {
        let temp = TempRepo::new("cloud-iac-cell-topology-service-tenant-fixture");
        write_fixture(temp.path(), FixtureDrift::MissingServiceTenantFixture);

        let error = validate_cloud_iac_cell_topology_gate(fixture_args(temp.path()))
            .expect_err("missing service/tenant topology contract fixture fails");

        assert!(error.contains("service_tenant_fixtures"));
    }

    fn fixture_args(root: &Path) -> CloudIacCellTopologyValidateArgs {
        CloudIacCellTopologyValidateArgs {
            repo_root: root.to_path_buf(),
            manifest: PathBuf::from(DEFAULT_MANIFEST),
            topology: PathBuf::from(DEFAULT_TOPOLOGY),
            catalog: PathBuf::from(DEFAULT_CATALOG),
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum FixtureDrift {
        None,
        CrossCellTrafficAllowed,
        MissingCatalogModuleRef,
        ManifestContextDrift,
        SecretMarker,
        GitOpsTemplateIdentityDrift,
        MissingServiceTenantFixture,
    }

    fn write_fixture(root: &Path, drift: FixtureDrift) {
        let contexts = if drift == FixtureDrift::ManifestContextDrift {
            vec!["aws-guest"]
        } else {
            vec!["aws-guest", "oci-guest"]
        };
        let regions = vec!["us-ashburn-1", "us-east-1"];
        fs::create_dir_all(root.join("cloud/cloud-iac/cell-topology"))
            .expect("topology dir");
        fs::create_dir_all(root.join("cloud/cloud-iac/tofu/modules/dns")).expect("dns dir");
        fs::create_dir_all(root.join("cloud/cloud-iac/tofu/modules/vpc")).expect("vpc dir");
        for (context, region, cell_id) in [
            ("aws-guest", "us-east-1", "aws-guest-us-east-1-a-001"),
            ("oci-guest", "us-ashburn-1", "oci-guest-us-ashburn-1-a-001"),
        ] {
            fs::create_dir_all(
                root.join(format!("cloud/cloud-iac/iac/{context}/argocd/apps")),
            )
            .expect("argocd app dir");
            fs::write(
                root.join(format!(
                    "cloud/cloud-iac/iac/{context}/argocd/apps/template.yaml"
                )),
                fixture_template(context, region, cell_id, drift),
            )
            .expect("template written");
        }
        for name in ["dns", "vpc"] {
            fs::write(
                root.join(format!(
                    "cloud/cloud-iac/tofu/modules/{name}/main.tofu"
                )),
                "# local foundation skeleton\n",
            )
            .expect("module file written");
        }

        let manifest = fixture_manifest(&contexts, &regions);
        fs::write(
            root.join(DEFAULT_MANIFEST),
            serde_json::to_string_pretty(&manifest).expect("manifest serializes"),
        )
        .expect("manifest written");

        let catalog = fixture_catalog();
        fs::write(
            root.join(DEFAULT_CATALOG),
            serde_json::to_string_pretty(&catalog).expect("catalog serializes"),
        )
        .expect("catalog written");

        let topology = fixture_topology(drift);
        fs::write(
            root.join(DEFAULT_TOPOLOGY),
            serde_json::to_string_pretty(&topology).expect("topology serializes"),
        )
        .expect("topology written");
    }

    fn fixture_manifest(contexts: &[&str], regions: &[&str]) -> Value {
        serde_json::json!({
            "capabilities": [
                {
                    "tier": "T1",
                    "name": "cloud-iac-cell-topology-gate",
                    "file": GATE_FILE,
                    "risk_class": "high"
                }
            ],
            "cell_topology_scope": {
                "topology": DEFAULT_TOPOLOGY,
                "module_catalog": DEFAULT_CATALOG,
                "gitops_templates_root": DEFAULT_TEMPLATES_ROOT,
                "runtime_mode": RUNTIME_MODE,
                "context_count": contexts.len(),
                "cell_count": 2,
                "service_tenant_fixture_count": 1,
                "contexts": contexts,
                "regions": regions,
                "topology_fields_modeled": [
                    "topology_id",
                    "runtime_mode",
                    "contexts",
                    "regions",
                    "cells.context",
                    "cells.region",
                    "cells.cell_id",
                    "cells.tenant_id",
                    "cells.isolation_tier",
                    "cells.default_cross_cell_traffic_allowed",
                    "cells.module_refs",
                    "cells.gitops_template",
                    "cells.evidence_ref",
                    "service_tenant_fixtures.microservice",
                    "service_tenant_fixtures.tenant_id",
                    "service_tenant_fixtures.cell_id",
                    "service_tenant_fixtures.cell_tier",
                    "service_tenant_fixtures.residency_class",
                    "service_tenant_fixtures.region_disposition",
                    "service_tenant_fixtures.storage_class",
                    "service_tenant_fixtures.quarterly_isolation_evidence.quarter",
                    "service_tenant_fixtures.quarterly_isolation_evidence.network",
                    "service_tenant_fixtures.quarterly_isolation_evidence.storage",
                    "service_tenant_fixtures.quarterly_isolation_evidence.crypto",
                    "service_tenant_fixtures.quarterly_isolation_evidence.compute",
                    "service_tenant_fixtures.quarterly_isolation_evidence.audit"
                ],
                "non_claims": [
                    "no autosharding runtime",
                    "no ArgoCD API integration",
                    "no OpenTofu CLI execution",
                    "no provider API integration"
                ],
                "coherence_guard": {
                    "gate": GATE_NAME,
                    "runtime_mode": RUNTIME_MODE
                }
            }
        })
    }

    fn fixture_catalog() -> Value {
        serde_json::json!({
            "schema_version": "1.0",
            "modules": [
                fixture_catalog_module("dns"),
                fixture_catalog_module("vpc")
            ]
        })
    }

    fn fixture_catalog_module(name: &str) -> Value {
        serde_json::json!({
            "namespace": "oyatie",
            "name": name,
            "system": "opentofu",
            "version": "0.1.0",
            "source_path": format!("cloud/cloud-iac/tofu/modules/{name}")
        })
    }

    fn fixture_topology(drift: FixtureDrift) -> Value {
        let missing_module = drift == FixtureDrift::MissingCatalogModuleRef;
        let secret_marker = drift == FixtureDrift::SecretMarker;
        serde_json::json!({
            "schema_version": "1.0",
            "topology_id": "cloud-iac-foundation-cell-topology",
            "generated_by_changeset": CHANGESET_ID,
            "runtime_mode": RUNTIME_MODE,
            "authority": {
                "manifest": DEFAULT_MANIFEST,
                "module_catalog": DEFAULT_CATALOG,
                "gitops_templates_root": DEFAULT_TEMPLATES_ROOT,
                "non_claims": [
                    "no autosharding runtime",
                    "no ArgoCD API integration",
                    "no OpenTofu CLI execution",
                    "no provider API integration"
                ]
            },
            "summary": {
                "context_count": 2,
                "cell_count": 2,
                "service_tenant_fixture_count": if drift == FixtureDrift::MissingServiceTenantFixture { 0 } else { 1 },
                "contexts": ["aws-guest", "oci-guest"],
                "regions": ["us-ashburn-1", "us-east-1"],
                "module_catalog": DEFAULT_CATALOG
            },
            "cells": [
                fixture_cell(
                    "aws-guest",
                    "us-east-1",
                    "aws-guest-us-east-1-a-001",
                    drift == FixtureDrift::CrossCellTrafficAllowed,
                    missing_module,
                    secret_marker
                ),
                fixture_cell(
                    "oci-guest",
                    "us-ashburn-1",
                    "oci-guest-us-ashburn-1-a-001",
                    false,
                    false,
                    false
                )
            ],
            "service_tenant_fixtures": if drift == FixtureDrift::MissingServiceTenantFixture {
                Vec::<Value>::new()
            } else {
                vec![fixture_service_tenant_fixture(
                    "aws-guest",
                    "us-east-1",
                    "aws-guest-us-east-1-a-001"
                )]
            },
            "non_claims": [
                "no autosharding runtime",
                "no ArgoCD API integration",
                "no OpenTofu CLI execution",
                "no provider API integration"
            ]
        })
    }

    fn fixture_service_tenant_fixture(context: &str, region: &str, cell_id: &str) -> Value {
        let tenant_id = format!("ten_cloud_iac_{}", context.replace('-', "_"));
        let evidence_prefix = format!(
            "evidence://cloud-iac/cell-topology/{context}/{region}/{cell_id}/quarterly-isolation/2026q2"
        );
        serde_json::json!({
            "microservice": "cloud-iac",
            "tenant_id": tenant_id,
            "cell_id": cell_id,
            "cell_tier": "shared-small",
            "residency_class": "global",
            "region_disposition": "active_active",
            "storage_class": "oya-pg-hot",
            "quarterly_isolation_evidence": {
                "quarter": "2026q2",
                "network": format!("{evidence_prefix}/network"),
                "storage": format!("{evidence_prefix}/storage"),
                "crypto": format!("{evidence_prefix}/crypto"),
                "compute": format!("{evidence_prefix}/compute"),
                "audit": format!("{evidence_prefix}/audit")
            }
        })
    }

    fn fixture_cell(
        context: &str,
        region: &str,
        cell_id: &str,
        cross_cell_traffic: bool,
        missing_module: bool,
        secret_marker: bool,
    ) -> Value {
        let second_module = if missing_module { "kms" } else { "vpc" };
        let evidence_ref = if secret_marker {
            format!("evidence://cloud-iac/cell-topology/{context}/{region}/{cell_id}/token=bad")
        } else {
            format!(
                "evidence://cloud-iac/cell-topology/{context}/{region}/{cell_id}/local-foundation"
            )
        };
        serde_json::json!({
            "context": context,
            "region": region,
            "cell_id": cell_id,
            "tenant_id": format!("ten_cloud_iac_{}", context.replace('-', "_")),
            "isolation_tier": "foundation",
            "default_cross_cell_traffic_allowed": cross_cell_traffic,
            "gitops_template": format!("cloud/cloud-iac/iac/{context}/argocd/apps/template.yaml"),
            "evidence_ref": evidence_ref,
            "module_refs": [
                fixture_module_ref("dns"),
                fixture_module_ref(second_module)
            ]
        })
    }

    fn fixture_template(context: &str, region: &str, cell_id: &str, drift: FixtureDrift) -> String {
        let rendered_cell_id =
            if drift == FixtureDrift::GitOpsTemplateIdentityDrift && context == "aws-guest" {
                "aws-guest-us-east-1-a-drift"
            } else {
                cell_id
            };
        format!(
            r#"apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  labels:
    oyatie.com/context: "{context}"
    oyatie.com/region: "{region}"
    oyatie.com/cell-id: "{rendered_cell_id}"
    oyatie.com/tenant-id: "ten_cloud_iac_{tenant_suffix}"
    oyatie.com/isolation-tier: "foundation"
    oyatie.com/default-cross-cell-traffic-allowed: "false"
"#,
            tenant_suffix = context.replace('-', "_")
        )
    }

    fn fixture_module_ref(name: &str) -> Value {
        serde_json::json!({
            "namespace": "oyatie",
            "name": name,
            "system": "opentofu",
            "version": "0.1.0"
        })
    }

    struct TempRepo {
        path: PathBuf,
    }

    impl TempRepo {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos();
            Self {
                path: std::env::temp_dir()
                    .join(format!("oya-{label}-{}-{nanos}", std::process::id())),
            }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.path).ok();
        }
    }
}
