//! `oya gate validate cloud-iac-module-provider-requirements` runner.
//!
//! This gate is the first Cloud IaC step after provider-readiness inventory:
//! reusable local OpenTofu modules must declare explicit `required_providers`
//! blocks that exactly match `provider-readiness.json`. It deliberately keeps
//! provider configuration, resources, lockfiles inside reusable modules,
//! provider installation in the source tree, plan/apply, signing, VSA/SLSA, and
//! cloud provisioning out of scope.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use crate::slash_path;

const DEFAULT_REPO_ROOT: &str = ".";
const DEFAULT_MANIFEST: &str = "iac/manifest.json";
const DEFAULT_CATALOG: &str = "iac/tofu/modules/catalog.json";
const DEFAULT_READINESS: &str = "iac/tofu/modules/provider-readiness.json";
const GATE_NAME: &str = "cloud-iac-module-provider-requirements";
const GATE_FILE: &str = "crates/oya-dev-cli/src/cloud_iac_module_provider_requirements_gate.rs";
const CHANGESET_ID: &str = "CS-CLOUD-IAC-MODULE-PROVIDER-REQUIREMENTS-GATE-001";
const RUNTIME_MODE: &str = "local-opentofu-required-providers-materialization-gate";
const READINESS_STATUS: &str = "required-providers-hcl-materialized-no-provider-lockfile";
const LOCAL_SKELETON_STATUS: &str = "local-foundation-skeleton";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacModuleProviderRequirementsArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) catalog: PathBuf,
    pub(crate) readiness: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacModuleProviderRequirementsReport {
    pub(crate) manifest_path: String,
    pub(crate) catalog_path: String,
    pub(crate) readiness_path: String,
    pub(crate) modules_checked: usize,
    pub(crate) provider_requirements_checked: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct ModuleKey {
    namespace: String,
    name: String,
    system: String,
    version: String,
}

impl ModuleKey {
    fn display(&self) -> String {
        format!(
            "{}/{}/{}/{}",
            self.namespace, self.name, self.system, self.version
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CatalogModule {
    key: ModuleKey,
    source_path: String,
    main_file: String,
    release_status: String,
    provider_resources_implemented: bool,
    outputs_materialized: bool,
    tests_present: bool,
    evidence_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProviderFamily {
    family: String,
    source: String,
    preferred_local_name: String,
    minimum_version_constraint: String,
    future_lock_required: bool,
    future_signature_review_required: bool,
    future_provider_provenance_required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReadinessModule {
    key: ModuleKey,
    source_path: String,
    main_file: String,
    release_status: String,
    evidence_ref: String,
    provider_requirements_hcl_materialized: bool,
    provider_lockfile_materialized: bool,
    provider_resources_implemented: bool,
    provider_families: Vec<ProviderFamily>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct HclProviderRequirement {
    source: String,
    version: String,
}

pub(crate) fn parse_cloud_iac_module_provider_requirements_args(
    args: Vec<String>,
) -> Result<CloudIacModuleProviderRequirementsArgs, String> {
    let mut parsed = CloudIacModuleProviderRequirementsArgs {
        repo_root: PathBuf::from(DEFAULT_REPO_ROOT),
        manifest: PathBuf::from(DEFAULT_MANIFEST),
        catalog: PathBuf::from(DEFAULT_CATALOG),
        readiness: PathBuf::from(DEFAULT_READINESS),
    };

    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--repo-root" => parsed.repo_root = take_path_arg(&mut args, "--repo-root")?,
            "--manifest" => parsed.manifest = take_path_arg(&mut args, "--manifest")?,
            "--catalog" => parsed.catalog = take_path_arg(&mut args, "--catalog")?,
            "--readiness" => parsed.readiness = take_path_arg(&mut args, "--readiness")?,
            other => {
                return Err(format!(
                    "cloud-iac-module-provider-requirements: unknown flag {other:?}; usage: \
                     oya gate validate cloud-iac-module-provider-requirements \
                     [--repo-root <.>] \
                     [--manifest <iac/manifest.json>] \
                     [--catalog <iac/tofu/modules/catalog.json>] \
                     [--readiness <iac/tofu/modules/provider-readiness.json>]"
                ));
            }
        }
    }

    Ok(parsed)
}

fn take_path_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next().map(PathBuf::from).ok_or_else(|| {
        format!("cloud-iac-module-provider-requirements: {flag} requires a path argument")
    })
}

pub(crate) fn validate_cloud_iac_module_provider_requirements_gate(
    args: CloudIacModuleProviderRequirementsArgs,
) -> Result<CloudIacModuleProviderRequirementsReport, String> {
    let manifest_path = resolve_repo_path(&args.repo_root, &args.manifest);
    let catalog_path = resolve_repo_path(&args.repo_root, &args.catalog);
    let readiness_path = resolve_repo_path(&args.repo_root, &args.readiness);
    let manifest_rel = repo_relative_argument(&args.repo_root, &args.manifest)?;
    let catalog_rel = repo_relative_argument(&args.repo_root, &args.catalog)?;
    let readiness_rel = repo_relative_argument(&args.repo_root, &args.readiness)?;

    let manifest = read_json(&manifest_path, "manifest")?;
    let catalog = read_json(&catalog_path, "catalog")?;
    let readiness = read_json(&readiness_path, "provider readiness")?;

    let mut diagnostics = Vec::new();
    require_manifest_scope(&manifest, &catalog_rel, &readiness_rel, &mut diagnostics);
    validate_manifest_capability(&manifest, &mut diagnostics);
    validate_readiness_policy(&readiness, &catalog_rel, &mut diagnostics);

    let modules_root =
        required_repo_relative_string(&readiness, "/authority/source_path_root", &mut diagnostics)
            .unwrap_or_else(|| "iac/tofu/modules".to_string());
    let catalog_modules = parse_catalog_modules(&catalog, &modules_root, &mut diagnostics);
    let readiness_modules = parse_readiness_modules(&readiness, &modules_root, &mut diagnostics);

    validate_manifest_module_summary(&manifest, &catalog_modules, &mut diagnostics);
    let provider_requirements_checked = validate_modules(
        &args.repo_root,
        &catalog_modules,
        &readiness_modules,
        &mut diagnostics,
    );

    if diagnostics.is_empty() {
        Ok(CloudIacModuleProviderRequirementsReport {
            manifest_path: manifest_rel,
            catalog_path: catalog_rel,
            readiness_path: readiness_rel,
            modules_checked: catalog_modules.len(),
            provider_requirements_checked,
        })
    } else {
        Err(format!(
            "cloud-iac-module-provider-requirements validation failed:\n- {}",
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
                "cloud-iac-module-provider-requirements: unable to canonicalize repo root {}: {error}",
                repo_root.display()
            )
        })?;
        let path = fs::canonicalize(path).map_err(|error| {
            format!(
                "cloud-iac-module-provider-requirements: unable to canonicalize path {}: {error}",
                path.display()
            )
        })?;
        let relative = path.strip_prefix(&repo_root).map_err(|_| {
            format!(
                "cloud-iac-module-provider-requirements: path {} is outside repo root {}",
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
            "cloud-iac-module-provider-requirements: unable to read {label} {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "cloud-iac-module-provider-requirements: unable to parse {label} JSON {}: {error}",
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

fn required_string_array(
    value: &Value,
    pointer: &str,
    diagnostics: &mut Vec<String>,
) -> Option<Vec<String>> {
    match value.pointer(pointer).and_then(Value::as_array) {
        Some(entries) => entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| match entry.as_str() {
                Some(found) if !found.trim().is_empty() => Some(found.trim().to_string()),
                _ => {
                    diagnostics.push(format!("{pointer}/{idx} must be a non-empty string"));
                    None
                }
            })
            .collect(),
        None => {
            diagnostics.push(format!("{pointer} must be an array of strings"));
            None
        }
    }
}

fn required_repo_relative_string(
    value: &Value,
    pointer: &str,
    diagnostics: &mut Vec<String>,
) -> Option<String> {
    let raw = required_string(value, pointer, diagnostics)?;
    normalize_repo_relative(&raw, pointer, diagnostics)
}

fn normalize_repo_relative(
    raw: &str,
    label: &str,
    diagnostics: &mut Vec<String>,
) -> Option<String> {
    if raw.contains('\\') {
        diagnostics.push(format!("{label} must use '/' separators: {raw:?}"));
        return None;
    }
    let mut parts = Vec::new();
    for component in Path::new(raw).components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                diagnostics.push(format!("{label} must not contain '..': {raw:?}"));
                return None;
            }
            Component::RootDir | Component::Prefix(_) => {
                diagnostics.push(format!("{label} must be repo-relative: {raw:?}"));
                return None;
            }
        }
    }
    if parts.is_empty() {
        diagnostics.push(format!("{label} must identify a file or directory"));
        None
    } else {
        Some(parts.join("/"))
    }
}

fn require_manifest_scope(
    manifest: &Value,
    catalog_rel: &str,
    readiness_rel: &str,
    diagnostics: &mut Vec<String>,
) {
    for (pointer, expected) in [
        ("/module_provider_requirements_scope/catalog", catalog_rel),
        (
            "/module_provider_requirements_scope/readiness",
            readiness_rel,
        ),
    ] {
        if required_repo_relative_string(manifest, pointer, diagnostics).as_deref()
            != Some(expected)
        {
            diagnostics.push(format!("manifest {pointer} must equal {expected:?}"));
        }
    }
    for (pointer, expected) in [
        (
            "/module_provider_requirements_scope/status",
            READINESS_STATUS,
        ),
        (
            "/module_provider_requirements_scope/runtime_mode",
            RUNTIME_MODE,
        ),
        (
            "/module_provider_requirements_scope/coherence_guard/changeset",
            CHANGESET_ID,
        ),
        (
            "/module_provider_requirements_scope/coherence_guard/gate",
            GATE_NAME,
        ),
        (
            "/module_provider_requirements_scope/coherence_guard/runtime_mode",
            RUNTIME_MODE,
        ),
    ] {
        if required_string(manifest, pointer, diagnostics).as_deref() != Some(expected) {
            diagnostics.push(format!("manifest {pointer} must be {expected:?}"));
        }
    }
    if required_repo_relative_string(
        manifest,
        "/module_provider_requirements_scope/coherence_guard/gate_file",
        diagnostics,
    )
    .as_deref()
        != Some(GATE_FILE)
    {
        diagnostics.push(format!(
            "manifest /module_provider_requirements_scope/coherence_guard/gate_file must be {GATE_FILE:?}"
        ));
    }
    let sources = required_string_array(
        manifest,
        "/module_provider_requirements_scope/official_sources_consulted",
        diagnostics,
    )
    .unwrap_or_default();
    for required in [
        "https://opentofu.org/docs/language/providers/requirements/",
        "https://opentofu.org/docs/language/modules/develop/providers/",
    ] {
        if !sources.iter().any(|source| source == required) {
            diagnostics.push(format!(
                "manifest /module_provider_requirements_scope/official_sources_consulted must include {required:?}"
            ));
        }
    }
    let non_claims = required_string_array(
        manifest,
        "/module_provider_requirements_scope/non_claims",
        diagnostics,
    )
    .unwrap_or_default();
    for required in [
        "no provider configuration",
        "no provider resources",
        "no provider lockfiles in reusable modules",
        "no provider installation in source tree",
        "no tofu plan/apply evidence",
        "no cloud resource provisioning",
    ] {
        if !non_claims.iter().any(|claim| claim.contains(required)) {
            diagnostics.push(format!(
                "manifest /module_provider_requirements_scope/non_claims must include {required:?}"
            ));
        }
    }
}

fn validate_manifest_capability(manifest: &Value, diagnostics: &mut Vec<String>) {
    let Some(capabilities) = manifest.pointer("/capabilities").and_then(Value::as_array) else {
        diagnostics.push("manifest /capabilities must be an array".to_string());
        return;
    };
    let Some(capability) = capabilities.iter().find(|entry| {
        entry.pointer("/name").and_then(Value::as_str)
            == Some("cloud-iac-module-provider-requirements-gate")
    }) else {
        diagnostics.push(
            "manifest /capabilities must include cloud-iac-module-provider-requirements-gate"
                .to_string(),
        );
        return;
    };
    if capability.pointer("/file").and_then(Value::as_str) != Some(GATE_FILE) {
        diagnostics.push(format!(
            "manifest module provider requirements capability /file must be {GATE_FILE:?}"
        ));
    }
    if capability.pointer("/tier").and_then(Value::as_str) != Some("T1") {
        diagnostics.push(
            "manifest module provider requirements capability /tier must be \"T1\"".to_string(),
        );
    }
}

fn validate_readiness_policy(readiness: &Value, catalog_rel: &str, diagnostics: &mut Vec<String>) {
    if required_repo_relative_string(readiness, "/authority/source_catalog", diagnostics).as_deref()
        != Some(catalog_rel)
    {
        diagnostics.push(format!(
            "readiness /authority/source_catalog must equal {catalog_rel:?}"
        ));
    }
    if required_string(readiness, "/policy/status", diagnostics).as_deref()
        != Some(READINESS_STATUS)
    {
        diagnostics.push(format!(
            "readiness /policy/status must be {READINESS_STATUS:?}"
        ));
    }
    for (pointer, expected) in [
        ("/policy/hcl_required_providers_materialized", true),
        ("/policy/provider_lockfiles_materialized", false),
        ("/policy/provider_installation_executed", false),
        ("/policy/provider_provenance_verified", false),
        ("/policy/module_signing_executed", false),
    ] {
        if required_bool(readiness, pointer, diagnostics) != Some(expected) {
            diagnostics.push(format!("readiness {pointer} must be {expected}"));
        }
    }
}

fn validate_manifest_module_summary(
    manifest: &Value,
    catalog_modules: &BTreeMap<ModuleKey, CatalogModule>,
    diagnostics: &mut Vec<String>,
) {
    if manifest
        .pointer("/module_provider_requirements_scope/module_count")
        .and_then(Value::as_u64)
        != Some(catalog_modules.len() as u64)
    {
        diagnostics.push(format!(
            "manifest /module_provider_requirements_scope/module_count must equal {}",
            catalog_modules.len()
        ));
    }
    let expected: Vec<String> = catalog_modules
        .values()
        .map(|module| module.key.name.clone())
        .collect();
    let found = required_string_array(
        manifest,
        "/module_provider_requirements_scope/module_names",
        diagnostics,
    )
    .unwrap_or_default();
    if found != expected {
        diagnostics.push(format!(
            "manifest /module_provider_requirements_scope/module_names must equal {:?}; found {:?}",
            expected, found
        ));
    }
}

fn parse_catalog_modules(
    catalog: &Value,
    modules_root: &str,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<ModuleKey, CatalogModule> {
    if required_repo_relative_string(catalog, "/authority/source_path_root", diagnostics).as_deref()
        != Some(modules_root)
    {
        diagnostics.push(format!(
            "catalog /authority/source_path_root must equal {modules_root:?}"
        ));
    }
    let Some(entries) = catalog.pointer("/modules").and_then(Value::as_array) else {
        diagnostics.push("catalog /modules must be an array".to_string());
        return BTreeMap::new();
    };
    let mut modules = BTreeMap::new();
    for (idx, entry) in entries.iter().enumerate() {
        let Some(module) = parse_catalog_module(entry, idx, diagnostics) else {
            continue;
        };
        if modules.insert(module.key.clone(), module).is_some() {
            diagnostics.push(format!("duplicate catalog module at /modules/{idx}"));
        }
    }
    modules
}

fn parse_catalog_module(
    entry: &Value,
    idx: usize,
    diagnostics: &mut Vec<String>,
) -> Option<CatalogModule> {
    let key = parse_module_key(entry, &format!("catalog /modules/{idx}"), diagnostics)?;
    Some(CatalogModule {
        key,
        source_path: required_repo_relative_string(entry, "/source_path", diagnostics)?,
        main_file: required_repo_relative_string(entry, "/main_file", diagnostics)?,
        release_status: required_string(entry, "/release_status", diagnostics)?,
        provider_resources_implemented: required_bool(
            entry,
            "/provider_resources_implemented",
            diagnostics,
        )?,
        outputs_materialized: required_bool(entry, "/outputs_materialized", diagnostics)?,
        tests_present: required_bool(entry, "/tests_present", diagnostics)?,
        evidence_ref: required_string(entry, "/evidence_ref", diagnostics)?,
    })
}

fn parse_readiness_modules(
    readiness: &Value,
    modules_root: &str,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<ModuleKey, ReadinessModule> {
    let Some(entries) = readiness.pointer("/modules").and_then(Value::as_array) else {
        diagnostics.push("readiness /modules must be an array".to_string());
        return BTreeMap::new();
    };
    let mut modules = BTreeMap::new();
    for (idx, entry) in entries.iter().enumerate() {
        let Some(module) = parse_readiness_module(entry, idx, modules_root, diagnostics) else {
            continue;
        };
        if modules.insert(module.key.clone(), module).is_some() {
            diagnostics.push(format!("duplicate readiness module at /modules/{idx}"));
        }
    }
    modules
}

fn parse_readiness_module(
    entry: &Value,
    idx: usize,
    modules_root: &str,
    diagnostics: &mut Vec<String>,
) -> Option<ReadinessModule> {
    let prefix = format!("readiness /modules/{idx}");
    let key = parse_module_key(entry, &prefix, diagnostics)?;
    let source_path = required_repo_relative_string(entry, "/source_path", diagnostics)?;
    if !source_path.starts_with(&format!("{modules_root}/")) {
        diagnostics.push(format!(
            "{prefix}/source_path must stay under {modules_root:?}; found {source_path:?}"
        ));
    }
    Some(ReadinessModule {
        key,
        source_path,
        main_file: required_repo_relative_string(entry, "/main_file", diagnostics)?,
        release_status: required_string(entry, "/release_status", diagnostics)?,
        evidence_ref: required_string(entry, "/evidence_ref", diagnostics)?,
        provider_requirements_hcl_materialized: required_bool(
            entry,
            "/provider_requirements_hcl_materialized",
            diagnostics,
        )?,
        provider_lockfile_materialized: required_bool(
            entry,
            "/provider_lockfile_materialized",
            diagnostics,
        )?,
        provider_resources_implemented: required_bool(
            entry,
            "/provider_resources_implemented",
            diagnostics,
        )?,
        provider_families: parse_provider_families(entry, &prefix, diagnostics)?,
    })
}

fn parse_provider_families(
    entry: &Value,
    prefix: &str,
    diagnostics: &mut Vec<String>,
) -> Option<Vec<ProviderFamily>> {
    let Some(entries) = entry
        .pointer("/provider_families")
        .and_then(Value::as_array)
    else {
        diagnostics.push(format!("{prefix}/provider_families must be an array"));
        return None;
    };
    if entries.is_empty() {
        diagnostics.push(format!("{prefix}/provider_families must not be empty"));
    }
    let mut families = Vec::with_capacity(entries.len());
    let mut seen = BTreeSet::new();
    for (idx, entry) in entries.iter().enumerate() {
        let family_prefix = format!("{prefix}/provider_families/{idx}");
        let Some(family) = parse_provider_family(entry, &family_prefix, diagnostics) else {
            continue;
        };
        if !seen.insert(family.preferred_local_name.clone()) {
            diagnostics.push(format!(
                "{family_prefix}/preferred_local_name must be unique per module"
            ));
        }
        families.push(family);
    }
    Some(families)
}

fn parse_provider_family(
    entry: &Value,
    prefix: &str,
    diagnostics: &mut Vec<String>,
) -> Option<ProviderFamily> {
    let family = required_string(entry, "/family", diagnostics)?;
    let source = required_string(entry, "/source", diagnostics)?;
    let preferred_local_name = required_string(entry, "/preferred_local_name", diagnostics)?;
    let minimum_version_constraint =
        required_string(entry, "/minimum_version_constraint", diagnostics)?;
    let future_lock_required = required_bool(entry, "/future_lock_required", diagnostics)?;
    let future_signature_review_required =
        required_bool(entry, "/future_signature_review_required", diagnostics)?;
    let future_provider_provenance_required =
        required_bool(entry, "/future_provider_provenance_required", diagnostics)?;

    if !is_slug(&family) {
        diagnostics.push(format!("{prefix}/family must be a lowercase slug"));
    }
    if !is_slug(&preferred_local_name) {
        diagnostics.push(format!(
            "{prefix}/preferred_local_name must be a lowercase provider local name"
        ));
    }
    if !is_fully_qualified_provider_source(&source) {
        diagnostics.push(format!(
            "{prefix}/source must be an explicit registry.opentofu.org provider source"
        ));
    }
    if !is_minimum_version_constraint(&minimum_version_constraint) {
        diagnostics.push(format!(
            "{prefix}/minimum_version_constraint must use >= x.y.z"
        ));
    }
    for (flag, value) in [
        ("future_lock_required", future_lock_required),
        (
            "future_signature_review_required",
            future_signature_review_required,
        ),
        (
            "future_provider_provenance_required",
            future_provider_provenance_required,
        ),
    ] {
        if !value {
            diagnostics.push(format!("{prefix}/{flag} must remain true"));
        }
    }

    Some(ProviderFamily {
        family,
        source,
        preferred_local_name,
        minimum_version_constraint,
        future_lock_required,
        future_signature_review_required,
        future_provider_provenance_required,
    })
}

fn parse_module_key(
    value: &Value,
    label: &str,
    diagnostics: &mut Vec<String>,
) -> Option<ModuleKey> {
    let namespace = required_string(value, "/namespace", diagnostics)?;
    let name = required_string(value, "/name", diagnostics)?;
    let system = required_string(value, "/system", diagnostics)?;
    let version = required_string(value, "/version", diagnostics)?;
    if namespace != "oyatie" {
        diagnostics.push(format!("{label}/namespace must be 'oyatie'"));
    }
    if system != "opentofu" {
        diagnostics.push(format!("{label}/system must be 'opentofu'"));
    }
    Some(ModuleKey {
        namespace,
        name,
        system,
        version,
    })
}

fn validate_modules(
    repo_root: &Path,
    catalog_modules: &BTreeMap<ModuleKey, CatalogModule>,
    readiness_modules: &BTreeMap<ModuleKey, ReadinessModule>,
    diagnostics: &mut Vec<String>,
) -> usize {
    let catalog_keys: BTreeSet<_> = catalog_modules.keys().cloned().collect();
    let readiness_keys: BTreeSet<_> = readiness_modules.keys().cloned().collect();
    if catalog_keys != readiness_keys {
        diagnostics.push(format!(
            "readiness module keys must match catalog keys; missing={:?} extra={:?}",
            catalog_keys
                .difference(&readiness_keys)
                .map(ModuleKey::display)
                .collect::<Vec<_>>(),
            readiness_keys
                .difference(&catalog_keys)
                .map(ModuleKey::display)
                .collect::<Vec<_>>()
        ));
    }

    let mut provider_requirements_checked = 0usize;
    for (key, catalog) in catalog_modules {
        let Some(readiness) = readiness_modules.get(key) else {
            continue;
        };
        validate_module_pair(catalog, readiness, diagnostics);
        validate_module_source_tree(repo_root, readiness, diagnostics);
        provider_requirements_checked += readiness.provider_families.len();
    }
    provider_requirements_checked
}

fn validate_module_pair(
    catalog: &CatalogModule,
    readiness: &ReadinessModule,
    diagnostics: &mut Vec<String>,
) {
    let key = catalog.key.display();
    if catalog.source_path != readiness.source_path {
        diagnostics.push(format!(
            "module {key} source_path drift: catalog {:?}, readiness {:?}",
            catalog.source_path, readiness.source_path
        ));
    }
    if catalog.main_file != readiness.main_file {
        diagnostics.push(format!(
            "module {key} main_file drift: catalog {:?}, readiness {:?}",
            catalog.main_file, readiness.main_file
        ));
    }
    if catalog.release_status != LOCAL_SKELETON_STATUS
        || readiness.release_status != LOCAL_SKELETON_STATUS
    {
        diagnostics.push(format!(
            "module {key} release_status must remain {LOCAL_SKELETON_STATUS:?}"
        ));
    }
    if catalog.provider_resources_implemented
        || catalog.outputs_materialized
        || catalog.tests_present
        || readiness.provider_resources_implemented
    {
        diagnostics.push(format!(
            "module {key} must not claim provider resources, materialized outputs, or tofu tests in required-provider scope"
        ));
    }
    if !readiness.provider_requirements_hcl_materialized {
        diagnostics.push(format!(
            "module {key} provider_requirements_hcl_materialized must be true"
        ));
    }
    if readiness.provider_lockfile_materialized {
        diagnostics.push(format!(
            "module {key} provider_lockfile_materialized must remain false"
        ));
    }
    if catalog.evidence_ref != readiness.evidence_ref {
        diagnostics.push(format!(
            "module {key} evidence_ref drift: catalog {:?}, readiness {:?}",
            catalog.evidence_ref, readiness.evidence_ref
        ));
    }
}

fn validate_module_source_tree(
    repo_root: &Path,
    readiness: &ReadinessModule,
    diagnostics: &mut Vec<String>,
) {
    let source = repo_root.join(&readiness.source_path);
    if !source.is_dir() {
        diagnostics.push(format!(
            "module {} source_path does not exist: {}",
            readiness.key.display(),
            source.display()
        ));
        return;
    }
    let main_file = repo_root.join(&readiness.main_file);
    let contents = match fs::read_to_string(&main_file) {
        Ok(contents) => contents,
        Err(error) => {
            diagnostics.push(format!(
                "module {} unable to read main_file {}: {error}",
                readiness.key.display(),
                main_file.display()
            ));
            return;
        }
    };
    validate_main_hcl(readiness, &contents, &main_file, diagnostics);

    let mut stack = vec![source];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) => {
                diagnostics.push(format!("unable to read {}: {error}", dir.display()));
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if name == ".terraform" {
                    diagnostics.push(format!(
                        "module {} must not contain provider installation cache {}",
                        readiness.key.display(),
                        path.display()
                    ));
                } else {
                    stack.push(path);
                }
                continue;
            }
            if is_forbidden_generated_artifact(&name) {
                diagnostics.push(format!(
                    "module {} must not contain generated OpenTofu artifact {}",
                    readiness.key.display(),
                    path.display()
                ));
            }
        }
    }
}

fn validate_main_hcl(
    readiness: &ReadinessModule,
    contents: &str,
    path: &Path,
    diagnostics: &mut Vec<String>,
) {
    if contains_forbidden_runtime_hcl(contents) {
        diagnostics.push(format!(
            "module {} {} must contain required_providers metadata only, not provider/resource/data/backend runtime blocks",
            readiness.key.display(),
            path.display()
        ));
    }
    if contains_secret_like_marker(contents) {
        diagnostics.push(format!(
            "module {} {} contains credential-like material marker",
            readiness.key.display(),
            path.display()
        ));
    }
    let parsed = parse_required_providers(contents);
    let expected_locals: BTreeSet<_> = readiness
        .provider_families
        .iter()
        .map(|family| family.preferred_local_name.as_str())
        .collect();
    let found_locals: BTreeSet<_> = parsed.keys().map(String::as_str).collect();
    if expected_locals != found_locals {
        diagnostics.push(format!(
            "module {} required provider local names must match readiness; missing={:?} extra={:?}",
            readiness.key.display(),
            expected_locals
                .difference(&found_locals)
                .collect::<Vec<_>>(),
            found_locals
                .difference(&expected_locals)
                .collect::<Vec<_>>()
        ));
    }
    for family in &readiness.provider_families {
        match parsed.get(&family.preferred_local_name) {
            Some(found)
                if found.source == family.source
                    && found.version == family.minimum_version_constraint => {}
            Some(found) => diagnostics.push(format!(
                "module {} provider {} must use source {:?} and version {:?}; found source {:?}, version {:?}",
                readiness.key.display(),
                family.preferred_local_name,
                family.source,
                family.minimum_version_constraint,
                found.source,
                found.version
            )),
            None => {}
        }
    }
}

fn parse_required_providers(contents: &str) -> BTreeMap<String, HclProviderRequirement> {
    let mut parsed = BTreeMap::new();
    let mut inside_required_providers = false;
    let mut required_depth = 0i32;
    let mut current_local: Option<String> = None;
    let mut current = HclProviderRequirement::default();

    for raw_line in contents.lines() {
        let active = strip_hcl_line_comment(raw_line).trim();
        if active.is_empty() {
            continue;
        }
        if !inside_required_providers {
            if opens_required_providers_block(active) {
                inside_required_providers = true;
                required_depth = hcl_brace_delta(active);
            }
            continue;
        }
        if current_local.is_none() {
            if let Some(local) = provider_local_block_name(active) {
                current_local = Some(local);
                current = HclProviderRequirement::default();
            }
        } else {
            if let Some(source) = quoted_assignment(active, "source") {
                current.source = source;
            }
            if let Some(version) = quoted_assignment(active, "version") {
                current.version = version;
            }
            if active.starts_with('}')
                && let Some(local) = current_local.take()
            {
                parsed.insert(local, current.clone());
                current = HclProviderRequirement::default();
            }
        }
        required_depth += hcl_brace_delta(active);
        if required_depth <= 0 {
            inside_required_providers = false;
            current_local = None;
            current = HclProviderRequirement::default();
        }
    }
    parsed
}

fn strip_hcl_line_comment(line: &str) -> &str {
    let hash = line.find('#');
    let slash = line.find("//");
    let end = match (hash, slash) {
        (Some(hash), Some(slash)) => hash.min(slash),
        (Some(hash), None) => hash,
        (None, Some(slash)) => slash,
        (None, None) => line.len(),
    };
    &line[..end]
}

fn opens_required_providers_block(line: &str) -> bool {
    line.contains("required_providers") && line.contains('{')
}

fn provider_local_block_name(line: &str) -> Option<String> {
    let (left, right) = line.split_once('=')?;
    if !right.contains('{') {
        return None;
    }
    let local = left.trim();
    if is_slug(local) {
        Some(local.to_string())
    } else {
        None
    }
}

fn quoted_assignment(line: &str, key: &str) -> Option<String> {
    let (left, right) = line.split_once('=')?;
    if left.trim() != key {
        return None;
    }
    let value = right.trim().strip_prefix('"')?;
    let (value, _) = value.split_once('"')?;
    Some(value.to_string())
}

fn hcl_brace_delta(line: &str) -> i32 {
    line.bytes().fold(0, |delta, byte| match byte {
        b'{' => delta + 1,
        b'}' => delta - 1,
        _ => delta,
    })
}

fn contains_forbidden_runtime_hcl(contents: &str) -> bool {
    contents.lines().any(|line| {
        let active = strip_hcl_line_comment(line).trim_start();
        active.starts_with("provider ")
            || active.starts_with("provider\"")
            || active.starts_with("resource ")
            || active.starts_with("resource\"")
            || active.starts_with("data ")
            || active.starts_with("data\"")
            || active.starts_with("backend ")
            || active.starts_with("backend\"")
    })
}

fn contains_secret_like_marker(contents: &str) -> bool {
    let lower = contents.to_ascii_lowercase();
    [
        "-----begin private key-----",
        "password",
        "secret_key",
        "private_key",
        "client_secret",
        "token=",
        "kubeconfig",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn is_forbidden_generated_artifact(name: &str) -> bool {
    name == ".terraform.lock.hcl"
        || name == "terraform.tfstate"
        || name == "terraform.tfstate.backup"
        || name == "crash.log"
        || name.ends_with(".tfstate")
        || name.ends_with(".tfstate.backup")
        || name.ends_with(".tfvars")
        || name.ends_with(".auto.tfvars")
        || name.ends_with(".tfplan")
        || name.ends_with(".tofutest.hcl")
        || name.ends_with(".tftest.hcl")
}

fn is_slug(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
}

fn is_fully_qualified_provider_source(value: &str) -> bool {
    let parts: Vec<_> = value.split('/').collect();
    parts.len() == 3 && parts[0] == "registry.opentofu.org" && parts[1..].iter().all(|p| is_slug(p))
}

fn is_minimum_version_constraint(value: &str) -> bool {
    let Some(version) = value.strip_prefix(">= ") else {
        return false;
    };
    is_semver(version)
}

fn is_semver(value: &str) -> bool {
    let parts: Vec<_> = value.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part == &"0" || !part.starts_with('0'))
        })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use oya_governance_gate_catalog_domain::AGGREGATED_VALIDATE_LANES;

    use super::{
        CloudIacModuleProviderRequirementsArgs, parse_cloud_iac_module_provider_requirements_args,
        validate_cloud_iac_module_provider_requirements_gate,
    };

    #[test]
    fn parse_cloud_iac_module_provider_requirements_rejects_unknown_flag() {
        let error = parse_cloud_iac_module_provider_requirements_args(vec!["--bogus".into()])
            .expect_err("unknown flag should fail");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn cloud_iac_module_provider_requirements_gate_accepts_valid_fixture() {
        let temp = TempRepo::new("cloud-iac-module-provider-requirements-valid");
        write_fixture(temp.path(), FixtureDrift::None);

        let report =
            validate_cloud_iac_module_provider_requirements_gate(fixture_args(temp.path()))
                .expect("valid fixture should pass");

        assert_eq!(report.modules_checked, 2);
        assert_eq!(report.provider_requirements_checked, 2);
    }

    #[test]
    fn cloud_iac_module_provider_requirements_gate_rejects_missing_hcl() {
        let temp = TempRepo::new("cloud-iac-module-provider-requirements-missing-hcl");
        write_fixture(temp.path(), FixtureDrift::MissingProviderHcl);

        let error = validate_cloud_iac_module_provider_requirements_gate(fixture_args(temp.path()))
            .expect_err("missing provider HCL should fail");

        assert!(error.contains("required provider local names"));
    }

    #[test]
    fn cloud_iac_module_provider_requirements_gate_rejects_source_drift() {
        let temp = TempRepo::new("cloud-iac-module-provider-requirements-source-drift");
        write_fixture(temp.path(), FixtureDrift::SourceDrift);

        let error = validate_cloud_iac_module_provider_requirements_gate(fixture_args(temp.path()))
            .expect_err("source drift should fail");

        assert!(error.contains("registry.opentofu.org/hashicorp/aws"));
    }

    #[test]
    fn cloud_iac_module_provider_requirements_gate_rejects_provider_configuration() {
        let temp = TempRepo::new("cloud-iac-module-provider-requirements-provider-block");
        write_fixture(temp.path(), FixtureDrift::ProviderBlock);

        let error = validate_cloud_iac_module_provider_requirements_gate(fixture_args(temp.path()))
            .expect_err("provider block should fail");

        assert!(error.contains("runtime blocks"));
    }

    #[test]
    fn aggregated_lane_catalog_contains_cloud_iac_module_provider_requirements() {
        assert!(AGGREGATED_VALIDATE_LANES.contains(&"cloud-iac-module-provider-requirements"));
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum FixtureDrift {
        None,
        MissingProviderHcl,
        SourceDrift,
        ProviderBlock,
    }

    fn fixture_args(repo_root: &Path) -> CloudIacModuleProviderRequirementsArgs {
        CloudIacModuleProviderRequirementsArgs {
            repo_root: repo_root.to_path_buf(),
            manifest: PathBuf::from("iac/manifest.json"),
            catalog: PathBuf::from("iac/tofu/modules/catalog.json"),
            readiness: PathBuf::from(
                "iac/tofu/modules/provider-readiness.json",
            ),
        }
    }

    fn write_fixture(root: &Path, drift: FixtureDrift) {
        let modules_root = root.join("iac/tofu/modules");
        for module in ["cloud-account", "dns"] {
            let module_dir = modules_root.join(module);
            fs::create_dir_all(&module_dir).expect("module dir");
            fs::write(
                module_dir.join("main.tofu"),
                fixture_main(module, drift, module),
            )
            .expect("main.tofu");
            fs::write(module_dir.join("README.md"), format!("# {module}\n")).expect("readme");
        }
        fs::write(modules_root.join("catalog.json"), fixture_catalog()).expect("catalog");
        fs::write(
            modules_root.join("provider-readiness.json"),
            fixture_readiness(),
        )
        .expect("readiness");
        fs::create_dir_all(root.join("iac")).expect("manifest dir");
        fs::write(
            root.join("iac/manifest.json"),
            fixture_manifest(),
        )
        .expect("manifest");
    }

    fn fixture_main(module: &str, drift: FixtureDrift, current_module: &str) -> String {
        if drift == FixtureDrift::MissingProviderHcl && current_module == "dns" {
            return format!(
                "terraform {{\n  required_version = \">= 1.6\"\n}}\n\noutput \"name\" {{ value = \"{module}\" }}\n"
            );
        }
        let source = if drift == FixtureDrift::SourceDrift && current_module == "dns" {
            "registry.opentofu.org/hashicorp/random"
        } else {
            "registry.opentofu.org/hashicorp/aws"
        };
        let provider_block = if drift == FixtureDrift::ProviderBlock && current_module == "dns" {
            "\nprovider \"aws\" {\n  region = \"us-east-1\"\n}\n"
        } else {
            ""
        };
        format!(
            "terraform {{\n  required_version = \">= 1.6\"\n  required_providers {{\n    aws = {{\n      source  = \"{source}\"\n      version = \">= 5.0.0\"\n    }}\n  }}\n}}\n{provider_block}\noutput \"name\" {{ value = \"{module}\" }}\n"
        )
    }

    fn fixture_catalog() -> String {
        r#"{
  "authority": { "source_path_root": "iac/tofu/modules" },
  "modules": [
    {
      "namespace": "oyatie", "name": "cloud-account", "system": "opentofu", "version": "0.1.0",
      "source_path": "iac/tofu/modules/cloud-account",
      "main_file": "iac/tofu/modules/cloud-account/main.tofu",
      "release_status": "local-foundation-skeleton",
      "provider_resources_implemented": false,
      "outputs_materialized": false,
      "tests_present": false,
      "evidence_ref": "evidence://cloud-iac/modules/cloud-account/0.1.0/local-foundation"
    },
    {
      "namespace": "oyatie", "name": "dns", "system": "opentofu", "version": "0.1.0",
      "source_path": "iac/tofu/modules/dns",
      "main_file": "iac/tofu/modules/dns/main.tofu",
      "release_status": "local-foundation-skeleton",
      "provider_resources_implemented": false,
      "outputs_materialized": false,
      "tests_present": false,
      "evidence_ref": "evidence://cloud-iac/modules/dns/0.1.0/local-foundation"
    }
  ]
}
"#
        .to_string()
    }

    fn fixture_readiness() -> String {
        r#"{
  "schema_version":"1.0",
  "readiness_id":"fixture",
  "generated_by_changeset":"CS-CLOUD-IAC-MODULE-PROVIDER-REQUIREMENTS-GATE-001",
  "authority":{"source_catalog":"iac/tofu/modules/catalog.json","source_path_root":"iac/tofu/modules","runtime_mode":"local-provider-readiness-inventory-gate"},
  "policy":{"status":"required-providers-hcl-materialized-no-provider-lockfile","hcl_required_providers_materialized":true,"provider_lockfiles_materialized":false,"provider_installation_executed":false,"provider_provenance_verified":false,"module_signing_executed":false,"minimum_future_lock_platforms":["darwin_arm64","linux_amd64","linux_arm64"]},
  "modules":[
    {"namespace":"oyatie","name":"cloud-account","system":"opentofu","version":"0.1.0","source_path":"iac/tofu/modules/cloud-account","main_file":"iac/tofu/modules/cloud-account/main.tofu","release_status":"local-foundation-skeleton","evidence_ref":"evidence://cloud-iac/modules/cloud-account/0.1.0/local-foundation","provider_requirements_hcl_materialized":true,"provider_lockfile_materialized":false,"provider_resources_implemented":false,"provider_families":[{"family":"aws","source":"registry.opentofu.org/hashicorp/aws","preferred_local_name":"aws","minimum_version_constraint":">= 5.0.0","future_lock_required":true,"future_signature_review_required":true,"future_provider_provenance_required":true}]},
    {"namespace":"oyatie","name":"dns","system":"opentofu","version":"0.1.0","source_path":"iac/tofu/modules/dns","main_file":"iac/tofu/modules/dns/main.tofu","release_status":"local-foundation-skeleton","evidence_ref":"evidence://cloud-iac/modules/dns/0.1.0/local-foundation","provider_requirements_hcl_materialized":true,"provider_lockfile_materialized":false,"provider_resources_implemented":false,"provider_families":[{"family":"aws","source":"registry.opentofu.org/hashicorp/aws","preferred_local_name":"aws","minimum_version_constraint":">= 5.0.0","future_lock_required":true,"future_signature_review_required":true,"future_provider_provenance_required":true}]}
  ]
}
"#
        .to_string()
    }

    fn fixture_manifest() -> String {
        r#"{
  "capabilities": [
    {
      "name": "cloud-iac-module-provider-requirements-gate",
      "file": "crates/oya-dev-cli/src/cloud_iac_module_provider_requirements_gate.rs",
      "tier": "T1"
    }
  ],
  "module_provider_requirements_scope": {
    "catalog": "iac/tofu/modules/catalog.json",
    "readiness": "iac/tofu/modules/provider-readiness.json",
    "status": "required-providers-hcl-materialized-no-provider-lockfile",
    "runtime_mode": "local-opentofu-required-providers-materialization-gate",
    "module_count": 2,
    "module_names": ["cloud-account", "dns"],
    "official_sources_consulted": [
      "https://opentofu.org/docs/language/providers/requirements/",
      "https://opentofu.org/docs/language/modules/develop/providers/"
    ],
    "coherence_guard": {
      "changeset": "CS-CLOUD-IAC-MODULE-PROVIDER-REQUIREMENTS-GATE-001",
      "gate": "cloud-iac-module-provider-requirements",
      "gate_file": "crates/oya-dev-cli/src/cloud_iac_module_provider_requirements_gate.rs",
      "runtime_mode": "local-opentofu-required-providers-materialization-gate"
    },
    "non_claims": [
      "no provider configuration",
      "no provider resources",
      "no provider lockfiles in reusable modules",
      "no provider installation in source tree",
      "no tofu plan/apply evidence",
      "no cloud resource provisioning"
    ]
  }
}
"#
        .to_string()
    }

    struct TempRepo {
        path: PathBuf,
    }

    impl TempRepo {
        fn new(prefix: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "{prefix}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("clock")
                    .as_nanos()
            ));
            fs::create_dir_all(&path).expect("temp repo dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
