//! `oya gate validate cloud-iac-provider-readiness` runner.
//!
//! This gate records and checks the next honest Cloud IaC supply-chain step:
//! local provider-readiness inventory before provider-specific OpenTofu modules,
//! provider lockfiles, provider installation, provider provenance, module signing,
//! tofu test/plan/apply, or cloud resource provisioning are claimed.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use crate::slash_path;

const DEFAULT_REPO_ROOT: &str = ".";
const DEFAULT_MANIFEST: &str = "iac/manifest.json";
const DEFAULT_CATALOG: &str = "iac/tofu/modules/catalog.json";
const DEFAULT_READINESS: &str = "iac/tofu/modules/provider-readiness.json";
const GATE_NAME: &str = "cloud-iac-provider-readiness";
const GATE_FILE: &str = "crates/oya-dev-cli/src/cloud_iac_provider_readiness_gate.rs";
const CHANGESET_ID: &str = "CS-CLOUD-IAC-PROVIDER-READINESS-GATE-001";
const MATERIALIZED_CHANGESET_ID: &str = "CS-CLOUD-IAC-MODULE-PROVIDER-REQUIREMENTS-GATE-001";
const RUNTIME_MODE: &str = "local-provider-readiness-inventory-gate";
const READINESS_STATUS: &str = "planned-inventory-no-provider-lockfile";
const MATERIALIZED_READINESS_STATUS: &str =
    "required-providers-hcl-materialized-no-provider-lockfile";
const LOCAL_SKELETON_STATUS: &str = "local-foundation-skeleton";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacProviderReadinessArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) catalog: PathBuf,
    pub(crate) readiness: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacProviderReadinessReport {
    pub(crate) manifest_path: String,
    pub(crate) catalog_path: String,
    pub(crate) readiness_path: String,
    pub(crate) modules_checked: usize,
    pub(crate) provider_families_checked: usize,
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

pub(crate) fn parse_cloud_iac_provider_readiness_args(
    args: Vec<String>,
) -> Result<CloudIacProviderReadinessArgs, String> {
    let mut parsed = CloudIacProviderReadinessArgs {
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
                    "cloud-iac-provider-readiness: unknown flag {other:?}; usage: \
                     oya gate validate cloud-iac-provider-readiness \
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
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("cloud-iac-provider-readiness: {flag} requires a path argument"))
}

pub(crate) fn validate_cloud_iac_provider_readiness_gate(
    args: CloudIacProviderReadinessArgs,
) -> Result<CloudIacProviderReadinessReport, String> {
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
    let modules_root =
        required_repo_relative_string(&readiness, "/authority/source_path_root", &mut diagnostics)
            .unwrap_or_else(|| "iac/tofu/modules".to_string());
    let catalog_modules = parse_catalog_modules(&catalog, &modules_root, &mut diagnostics);
    let readiness_modules = parse_readiness_modules(&readiness, &modules_root, &mut diagnostics);

    validate_readiness_header(&readiness, &catalog_rel, &modules_root, &mut diagnostics);
    validate_policy(&readiness, &mut diagnostics);
    validate_manifest_summary(&manifest, &catalog_modules, &mut diagnostics);
    let provider_families_checked = validate_readiness_modules(
        &args.repo_root,
        &catalog_modules,
        &readiness_modules,
        &mut diagnostics,
    );
    validate_no_lockfiles_under_modules(&args.repo_root, &modules_root, &mut diagnostics);

    if diagnostics.is_empty() {
        Ok(CloudIacProviderReadinessReport {
            manifest_path: manifest_rel,
            catalog_path: catalog_rel,
            readiness_path: readiness_rel,
            modules_checked: catalog_modules.len(),
            provider_families_checked,
        })
    } else {
        Err(format!(
            "cloud-iac-provider-readiness validation failed:\n- {}",
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
                "cloud-iac-provider-readiness: unable to canonicalize repo root {}: {error}",
                repo_root.display()
            )
        })?;
        let path = fs::canonicalize(path).map_err(|error| {
            format!(
                "cloud-iac-provider-readiness: unable to canonicalize path {}: {error}",
                path.display()
            )
        })?;
        let relative = path.strip_prefix(&repo_root).map_err(|_| {
            format!(
                "cloud-iac-provider-readiness: path {} is outside repo root {}",
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
            "cloud-iac-provider-readiness: unable to read {label} {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "cloud-iac-provider-readiness: unable to parse {label} JSON {}: {error}",
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

fn required_repo_relative_string(
    value: &Value,
    pointer: &str,
    diagnostics: &mut Vec<String>,
) -> Option<String> {
    required_string(value, pointer, diagnostics)
        .and_then(|raw| normalize_repo_relative(&raw, &format!("JSON {pointer}"), diagnostics))
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
    if raw.starts_with('/') {
        diagnostics.push(format!(
            "{label} must be repo-relative, found absolute path {raw:?}"
        ));
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
    if required_repo_relative_string(manifest, "/provider_readiness_scope/catalog", diagnostics)
        .as_deref()
        != Some(catalog_rel)
    {
        diagnostics.push(format!(
            "manifest /provider_readiness_scope/catalog must equal {catalog_rel:?}"
        ));
    }
    if required_repo_relative_string(manifest, "/provider_readiness_scope/readiness", diagnostics)
        .as_deref()
        != Some(readiness_rel)
    {
        diagnostics.push(format!(
            "manifest /provider_readiness_scope/readiness must equal {readiness_rel:?}"
        ));
    }
    let status = required_string(manifest, "/provider_readiness_scope/status", diagnostics);
    if !is_allowed_readiness_status(status.as_deref()) {
        diagnostics.push(format!(
            "manifest /provider_readiness_scope/status must be {READINESS_STATUS:?} or {MATERIALIZED_READINESS_STATUS:?}"
        ));
    }
    if required_string(
        manifest,
        "/provider_readiness_scope/coherence_guard/gate",
        diagnostics,
    )
    .as_deref()
        != Some(GATE_NAME)
    {
        diagnostics.push(format!(
            "manifest /provider_readiness_scope/coherence_guard/gate must be {GATE_NAME:?}"
        ));
    }
    if required_string(
        manifest,
        "/provider_readiness_scope/coherence_guard/changeset",
        diagnostics,
    )
    .as_deref()
        != Some(CHANGESET_ID)
    {
        diagnostics.push(format!(
            "manifest /provider_readiness_scope/coherence_guard/changeset must be {CHANGESET_ID:?}"
        ));
    }
    if required_string(
        manifest,
        "/provider_readiness_scope/coherence_guard/runtime_mode",
        diagnostics,
    )
    .as_deref()
        != Some(RUNTIME_MODE)
    {
        diagnostics.push(format!(
            "manifest /provider_readiness_scope/coherence_guard/runtime_mode must be {RUNTIME_MODE:?}"
        ));
    }
    if required_repo_relative_string(
        manifest,
        "/provider_readiness_scope/coherence_guard/gate_file",
        diagnostics,
    )
    .as_deref()
        != Some(GATE_FILE)
    {
        diagnostics.push(format!(
            "manifest /provider_readiness_scope/coherence_guard/gate_file must be {GATE_FILE:?}"
        ));
    }
    let non_claims = required_string_array(
        manifest,
        "/provider_readiness_scope/non_claims",
        diagnostics,
    )
    .unwrap_or_default();
    for required in [
        "no provider dependency lockfile",
        "no provider installation",
        "no provider provenance verification",
        "no module signing or Sigstore execution",
        "no tofu test/plan/apply evidence",
    ] {
        if !non_claims.iter().any(|claim| claim.contains(required)) {
            diagnostics.push(format!(
                "manifest /provider_readiness_scope/non_claims must include {required:?}"
            ));
        }
    }
}

fn validate_readiness_header(
    readiness: &Value,
    catalog_rel: &str,
    modules_root: &str,
    diagnostics: &mut Vec<String>,
) {
    let generated_by = required_string(readiness, "/generated_by_changeset", diagnostics);
    if !matches!(
        generated_by.as_deref(),
        Some(CHANGESET_ID | MATERIALIZED_CHANGESET_ID)
    ) {
        diagnostics.push(format!(
            "readiness /generated_by_changeset must be {CHANGESET_ID:?} or {MATERIALIZED_CHANGESET_ID:?}"
        ));
    }
    if required_string(readiness, "/authority/runtime_mode", diagnostics).as_deref()
        != Some(RUNTIME_MODE)
    {
        diagnostics.push(format!(
            "readiness /authority/runtime_mode must be {RUNTIME_MODE:?}"
        ));
    }
    if required_repo_relative_string(readiness, "/authority/source_catalog", diagnostics).as_deref()
        != Some(catalog_rel)
    {
        diagnostics.push(format!(
            "readiness /authority/source_catalog must equal {catalog_rel:?}"
        ));
    }
    if required_repo_relative_string(readiness, "/authority/source_path_root", diagnostics)
        .as_deref()
        != Some(modules_root)
    {
        diagnostics.push(format!(
            "readiness /authority/source_path_root must equal {modules_root:?}"
        ));
    }
}

fn is_allowed_readiness_status(status: Option<&str>) -> bool {
    matches!(
        status,
        Some(READINESS_STATUS | MATERIALIZED_READINESS_STATUS)
    )
}

fn validate_policy(readiness: &Value, diagnostics: &mut Vec<String>) {
    for (pointer, expected) in [
        ("/policy/provider_lockfiles_materialized", false),
        ("/policy/provider_installation_executed", false),
        ("/policy/provider_provenance_verified", false),
        ("/policy/module_signing_executed", false),
    ] {
        if required_bool(readiness, pointer, diagnostics) != Some(expected) {
            diagnostics.push(format!("readiness {pointer} must be {expected}"));
        }
    }
    let status = required_string(readiness, "/policy/status", diagnostics);
    let hcl_materialized = required_bool(
        readiness,
        "/policy/hcl_required_providers_materialized",
        diagnostics,
    );
    match status.as_deref() {
        Some(READINESS_STATUS) if hcl_materialized != Some(false) => diagnostics.push(format!(
            "readiness /policy/hcl_required_providers_materialized must be false for {READINESS_STATUS:?}"
        )),
        Some(MATERIALIZED_READINESS_STATUS) if hcl_materialized != Some(true) => diagnostics.push(format!(
            "readiness /policy/hcl_required_providers_materialized must be true for {MATERIALIZED_READINESS_STATUS:?}"
        )),
        Some(READINESS_STATUS | MATERIALIZED_READINESS_STATUS) => {}
        _ => diagnostics.push(format!(
            "readiness /policy/status must be {READINESS_STATUS:?} or {MATERIALIZED_READINESS_STATUS:?}"
        )),
    }
    let platforms = required_string_array(
        readiness,
        "/policy/minimum_future_lock_platforms",
        diagnostics,
    )
    .unwrap_or_default();
    for required in ["darwin_arm64", "linux_amd64", "linux_arm64"] {
        if !platforms.iter().any(|platform| platform == required) {
            diagnostics.push(format!(
                "readiness /policy/minimum_future_lock_platforms must include {required:?}"
            ));
        }
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
    let source_path = required_repo_relative_string(entry, "/source_path", diagnostics)?;
    let main_file = required_repo_relative_string(entry, "/main_file", diagnostics)?;
    let release_status = required_string(entry, "/release_status", diagnostics)?;
    let provider_resources_implemented =
        required_bool(entry, "/provider_resources_implemented", diagnostics)?;
    let outputs_materialized = required_bool(entry, "/outputs_materialized", diagnostics)?;
    let tests_present = required_bool(entry, "/tests_present", diagnostics)?;
    let evidence_ref = required_string(entry, "/evidence_ref", diagnostics)?;
    Some(CatalogModule {
        key,
        source_path,
        main_file,
        release_status,
        provider_resources_implemented,
        outputs_materialized,
        tests_present,
        evidence_ref,
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
    let main_file = required_repo_relative_string(entry, "/main_file", diagnostics)?;
    let release_status = required_string(entry, "/release_status", diagnostics)?;
    let evidence_ref = required_string(entry, "/evidence_ref", diagnostics)?;
    let provider_requirements_hcl_materialized = required_bool(
        entry,
        "/provider_requirements_hcl_materialized",
        diagnostics,
    )?;
    let provider_lockfile_materialized =
        required_bool(entry, "/provider_lockfile_materialized", diagnostics)?;
    let provider_resources_implemented =
        required_bool(entry, "/provider_resources_implemented", diagnostics)?;
    let provider_families = parse_provider_families(entry, &prefix, diagnostics)?;
    Some(ReadinessModule {
        key,
        source_path,
        main_file,
        release_status,
        evidence_ref,
        provider_requirements_hcl_materialized,
        provider_lockfile_materialized,
        provider_resources_implemented,
        provider_families,
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
    for (idx, family) in entries.iter().enumerate() {
        let family_prefix = format!("{prefix}/provider_families/{idx}");
        let Some(parsed) = parse_provider_family(family, &family_prefix, diagnostics) else {
            continue;
        };
        if !seen.insert((parsed.family.clone(), parsed.source.clone())) {
            diagnostics.push(format!(
                "{family_prefix} duplicates provider family/source {:?}/{:?}",
                parsed.family, parsed.source
            ));
        }
        families.push(parsed);
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
            "{prefix}/source must be an explicit provider source address like registry.opentofu.org/hashicorp/aws"
        ));
    }
    if !is_minimum_version_constraint(&minimum_version_constraint) {
        diagnostics.push(format!(
            "{prefix}/minimum_version_constraint must use a reusable-module minimum constraint such as >= 1.2.3"
        ));
    }
    if !future_lock_required {
        diagnostics.push(format!("{prefix}/future_lock_required must be true"));
    }
    if !future_signature_review_required {
        diagnostics.push(format!(
            "{prefix}/future_signature_review_required must be true"
        ));
    }
    if !future_provider_provenance_required {
        diagnostics.push(format!(
            "{prefix}/future_provider_provenance_required must be true"
        ));
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

fn validate_manifest_summary(
    manifest: &Value,
    catalog_modules: &BTreeMap<ModuleKey, CatalogModule>,
    diagnostics: &mut Vec<String>,
) {
    if let Some(found) = manifest
        .pointer("/provider_readiness_scope/module_count")
        .and_then(Value::as_u64)
    {
        if found as usize != catalog_modules.len() {
            diagnostics.push(format!(
                "manifest /provider_readiness_scope/module_count must equal {}; found {found}",
                catalog_modules.len()
            ));
        }
    } else {
        diagnostics
            .push("manifest /provider_readiness_scope/module_count must be a number".to_string());
    }
    let expected: Vec<String> = catalog_modules
        .values()
        .map(|module| module.key.name.clone())
        .collect();
    let found = required_string_array(
        manifest,
        "/provider_readiness_scope/module_names",
        diagnostics,
    )
    .unwrap_or_default();
    if found != expected {
        diagnostics.push(format!(
            "manifest /provider_readiness_scope/module_names must equal {:?}; found {:?}",
            expected, found
        ));
    }
}

fn validate_readiness_modules(
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

    let mut provider_families_checked = 0usize;
    for (key, catalog) in catalog_modules {
        let Some(readiness) = readiness_modules.get(key) else {
            continue;
        };
        validate_module_pair(catalog, readiness, diagnostics);
        validate_source_hcl_state(repo_root, readiness, diagnostics);
        provider_families_checked += readiness.provider_families.len();
    }
    provider_families_checked
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
            "module {key} must not claim provider resources, materialized outputs, or tests in provider-readiness-only scope"
        ));
    }
    if readiness.provider_lockfile_materialized {
        diagnostics.push(format!(
            "module {key} readiness flags must keep provider lockfiles materialized=false"
        ));
    }
    if catalog.evidence_ref != readiness.evidence_ref {
        diagnostics.push(format!(
            "module {key} evidence_ref drift: catalog {:?}, readiness {:?}",
            catalog.evidence_ref, readiness.evidence_ref
        ));
    }
}

fn validate_source_hcl_state(
    repo_root: &Path,
    readiness: &ReadinessModule,
    diagnostics: &mut Vec<String>,
) {
    let path = repo_root.join(&readiness.main_file);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) => {
            diagnostics.push(format!(
                "module {} unable to read main_file {}: {error}",
                readiness.key.display(),
                path.display()
            ));
            return;
        }
    };
    let active_hcl_provider_requirement =
        has_required_provider_source_or_version_assignment(&contents);
    if readiness.provider_requirements_hcl_materialized && !active_hcl_provider_requirement {
        diagnostics.push(format!(
            "module {} must have active provider source/version HCL because provider_requirements_hcl_materialized=true",
            readiness.key.display()
        ));
    }
    if !readiness.provider_requirements_hcl_materialized && active_hcl_provider_requirement {
        diagnostics.push(format!(
            "module {} has active provider source/version HCL while provider_requirements_hcl_materialized=false",
            readiness.key.display()
        ));
    }
}

fn has_required_provider_source_or_version_assignment(contents: &str) -> bool {
    let mut inside_required_providers = false;
    let mut required_provider_depth = 0i32;

    for raw_line in contents.lines() {
        let active = strip_hcl_line_comment(raw_line).trim();
        if active.is_empty() {
            continue;
        }

        if !inside_required_providers {
            if opens_required_providers_block(active) {
                inside_required_providers = true;
            } else {
                continue;
            }
        }

        if contains_hcl_assignment(active, "source") || contains_hcl_assignment(active, "version") {
            return true;
        }

        required_provider_depth += hcl_brace_delta(active);
        if required_provider_depth <= 0 {
            inside_required_providers = false;
            required_provider_depth = 0;
        }
    }

    false
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
    let key = "required_providers";
    let mut search_start = 0usize;
    while let Some(offset) = line[search_start..].find(key) {
        let token_start = search_start + offset;
        let token_end = token_start + key.len();
        let bytes = line.as_bytes();
        let previous_is_boundary =
            token_start == 0 || !is_hcl_identifier_byte(bytes[token_start - 1]);
        let next_is_boundary =
            token_end == bytes.len() || !is_hcl_identifier_byte(bytes[token_end]);
        if previous_is_boundary && next_is_boundary && line[token_end..].contains('{') {
            return true;
        }
        search_start = token_end;
    }
    false
}

fn contains_hcl_assignment(line: &str, key: &str) -> bool {
    let mut search_start = 0usize;
    while let Some(offset) = line[search_start..].find(key) {
        let token_start = search_start + offset;
        let token_end = token_start + key.len();
        let bytes = line.as_bytes();
        let previous_is_boundary =
            token_start == 0 || !is_hcl_identifier_byte(bytes[token_start - 1]);
        let mut next = token_end;
        while next < bytes.len() && bytes[next].is_ascii_whitespace() {
            next += 1;
        }
        if previous_is_boundary && bytes.get(next) == Some(&b'=') {
            return true;
        }
        search_start = token_end;
    }
    false
}

fn is_hcl_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

fn hcl_brace_delta(line: &str) -> i32 {
    line.bytes().fold(0, |delta, byte| match byte {
        b'{' => delta + 1,
        b'}' => delta - 1,
        _ => delta,
    })
}

fn validate_no_lockfiles_under_modules(
    repo_root: &Path,
    modules_root: &str,
    diagnostics: &mut Vec<String>,
) {
    let root = repo_root.join(modules_root);
    if !root.is_dir() {
        diagnostics.push(format!(
            "provider readiness modules root does not exist or is not a directory: {}",
            root.display()
        ));
        return;
    }
    visit_dirs(&root, &mut |path| {
        if path.file_name().and_then(|name| name.to_str()) == Some(".terraform.lock.hcl") {
            diagnostics.push(format!(
                "provider readiness scope forbids materialized provider lockfile under modules: {}",
                path.display()
            ));
        }
    });
}

fn visit_dirs(dir: &Path, on_file: &mut impl FnMut(&Path)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_dirs(&path, on_file);
        } else {
            on_file(&path);
        }
    }
}

fn is_slug(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_fully_qualified_provider_source(source: &str) -> bool {
    let parts: Vec<_> = source.split('/').collect();
    parts.len() == 3
        && parts[0].contains('.')
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || byte == b'-'
                        || byte == b'.'
                })
        })
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

    use super::{
        CloudIacProviderReadinessArgs, parse_cloud_iac_provider_readiness_args,
        validate_cloud_iac_provider_readiness_gate,
    };

    #[test]
    fn parse_cloud_iac_provider_readiness_rejects_unknown_flag() {
        let error = parse_cloud_iac_provider_readiness_args(vec!["--bogus".into()])
            .expect_err("unknown flag should fail");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn cloud_iac_provider_readiness_gate_accepts_valid_fixture() {
        let temp = TempRepo::new("cloud-iac-provider-readiness-valid");
        write_fixture(temp.path(), FixtureDrift::None);

        let report = validate_cloud_iac_provider_readiness_gate(fixture_args(temp.path()))
            .expect("valid fixture should pass");

        assert_eq!(report.modules_checked, 2);
        assert_eq!(report.provider_families_checked, 2);
    }

    #[test]
    fn cloud_iac_provider_readiness_gate_rejects_missing_catalog_module() {
        let temp = TempRepo::new("cloud-iac-provider-readiness-missing-module");
        write_fixture(temp.path(), FixtureDrift::MissingModule);

        let error = validate_cloud_iac_provider_readiness_gate(fixture_args(temp.path()))
            .expect_err("missing module should fail");

        assert!(error.contains("readiness module keys must match catalog keys"));
    }

    #[test]
    fn cloud_iac_provider_readiness_gate_rejects_materialized_lockfile() {
        let temp = TempRepo::new("cloud-iac-provider-readiness-lockfile");
        write_fixture(temp.path(), FixtureDrift::MaterializedLockfile);

        let error = validate_cloud_iac_provider_readiness_gate(fixture_args(temp.path()))
            .expect_err("lockfile should fail");

        assert!(error.contains("forbids materialized provider lockfile"));
    }

    #[test]
    fn cloud_iac_provider_readiness_gate_rejects_provider_requirement_hcl() {
        let temp = TempRepo::new("cloud-iac-provider-readiness-hcl");
        write_fixture(temp.path(), FixtureDrift::MaterializedProviderHcl);

        let error = validate_cloud_iac_provider_readiness_gate(fixture_args(temp.path()))
            .expect_err("active provider HCL should fail");

        assert!(error.contains("active provider source/version HCL"));
    }

    #[test]
    fn cloud_iac_provider_readiness_gate_ignores_non_provider_source_version_attributes() {
        let temp = TempRepo::new("cloud-iac-provider-readiness-non-provider-hcl");
        write_fixture(
            temp.path(),
            FixtureDrift::NonProviderSourceVersionAttributes,
        );

        let report = validate_cloud_iac_provider_readiness_gate(fixture_args(temp.path()))
            .expect("source/version attributes outside required_providers should pass");

        assert_eq!(report.modules_checked, 2);
        assert_eq!(report.provider_families_checked, 2);
    }

    #[test]
    fn cloud_iac_provider_readiness_gate_rejects_non_minimum_constraint() {
        let temp = TempRepo::new("cloud-iac-provider-readiness-constraint");
        write_fixture(temp.path(), FixtureDrift::MaximumConstraint);

        let error = validate_cloud_iac_provider_readiness_gate(fixture_args(temp.path()))
            .expect_err("maximum constraint should fail");

        assert!(error.contains("minimum constraint"));
    }

    fn fixture_args(repo_root: &Path) -> CloudIacProviderReadinessArgs {
        CloudIacProviderReadinessArgs {
            repo_root: repo_root.to_path_buf(),
            manifest: PathBuf::from("iac/manifest.json"),
            catalog: PathBuf::from("iac/tofu/modules/catalog.json"),
            readiness: PathBuf::from(
                "iac/tofu/modules/provider-readiness.json",
            ),
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum FixtureDrift {
        None,
        MissingModule,
        MaterializedLockfile,
        MaterializedProviderHcl,
        NonProviderSourceVersionAttributes,
        MaximumConstraint,
    }

    fn write_fixture(root: &Path, drift: FixtureDrift) {
        let modules_root = root.join("iac/tofu/modules");
        for module in ["cloud-account", "dns"] {
            let module_dir = modules_root.join(module);
            fs::create_dir_all(&module_dir).expect("module dir");
            fs::write(
                module_dir.join("main.tofu"),
                fixture_main(
                    module,
                    drift == FixtureDrift::MaterializedProviderHcl && module == "dns",
                    drift == FixtureDrift::NonProviderSourceVersionAttributes && module == "dns",
                ),
            )
            .expect("main.tofu");
            fs::write(module_dir.join("README.md"), format!("# {module}\n")).expect("readme");
        }
        if drift == FixtureDrift::MaterializedLockfile {
            fs::write(
                modules_root.join("dns/.terraform.lock.hcl"),
                "provider \"registry.opentofu.org/hashicorp/aws\" {}\n",
            )
            .expect("lockfile");
        }
        fs::write(modules_root.join("catalog.json"), fixture_catalog()).expect("catalog");
        fs::write(
            modules_root.join("provider-readiness.json"),
            fixture_readiness(drift),
        )
        .expect("readiness");
        fs::create_dir_all(root.join("iac")).expect("manifest dir");
        fs::write(
            root.join("iac/manifest.json"),
            fixture_manifest(),
        )
        .expect("manifest");
    }

    fn fixture_main(
        module: &str,
        materialized_provider: bool,
        non_provider_source_version_attributes: bool,
    ) -> String {
        if materialized_provider {
            format!(
                "terraform {{\n  required_version = \">= 1.6\"\n  required_providers {{\n    aws = {{\n      source = \"registry.opentofu.org/hashicorp/aws\"\n      version = \">= 5.0.0\"\n    }}\n  }}\n}}\n\noutput \"name\" {{ value = \"{module}\" }}\n"
            )
        } else if non_provider_source_version_attributes {
            format!(
                "terraform {{\n  required_version = \">= 1.6\"\n}}\n\nlocals {{\n  source = \"local-skeleton-only\"\n  version = \"draft-metadata-only\"\n}}\n\noutput \"name\" {{ value = \"{module}\" }}\n"
            )
        } else {
            format!(
                "terraform {{\n  required_version = \">= 1.6\"\n}}\n\noutput \"name\" {{ value = \"{module}\" }}\n"
            )
        }
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

    fn fixture_readiness(drift: FixtureDrift) -> String {
        let dns_constraint = if drift == FixtureDrift::MaximumConstraint {
            "~> 5.0"
        } else {
            ">= 5.0.0"
        };
        let modules = if drift == FixtureDrift::MissingModule {
            vec![readiness_module("cloud-account", ">= 5.0.0")]
        } else {
            vec![
                readiness_module("cloud-account", ">= 5.0.0"),
                readiness_module("dns", dns_constraint),
            ]
        };
        format!(
            r#"{{
  "schema_version":"1.0",
  "readiness_id":"fixture",
  "generated_by_changeset":"CS-CLOUD-IAC-PROVIDER-READINESS-GATE-001",
  "authority":{{"source_catalog":"iac/tofu/modules/catalog.json","source_path_root":"iac/tofu/modules","runtime_mode":"local-provider-readiness-inventory-gate"}},
  "policy":{{"status":"planned-inventory-no-provider-lockfile","hcl_required_providers_materialized":false,"provider_lockfiles_materialized":false,"provider_installation_executed":false,"provider_provenance_verified":false,"module_signing_executed":false,"minimum_future_lock_platforms":["darwin_arm64","linux_amd64","linux_arm64"]}},
  "modules":[{}]
}}
"#,
            modules.join(",")
        )
    }

    fn readiness_module(module: &str, constraint: &str) -> String {
        let source = format!("iac/tofu/modules/{module}");
        let main = format!("{source}/main.tofu");
        format!(
            r#"{{"namespace":"oyatie","name":"{module}","system":"opentofu","version":"0.1.0","source_path":"{source}","main_file":"{main}","release_status":"local-foundation-skeleton","evidence_ref":"evidence://cloud-iac/modules/{module}/0.1.0/local-foundation","provider_requirements_hcl_materialized":false,"provider_lockfile_materialized":false,"provider_resources_implemented":false,"provider_families":[{{"family":"aws","source":"registry.opentofu.org/hashicorp/aws","preferred_local_name":"aws","minimum_version_constraint":"{constraint}","future_lock_required":true,"future_signature_review_required":true,"future_provider_provenance_required":true}}]}}"#
        )
    }

    fn fixture_manifest() -> String {
        r#"{
  "provider_readiness_scope": {
    "catalog": "iac/tofu/modules/catalog.json",
    "readiness": "iac/tofu/modules/provider-readiness.json",
    "status": "planned-inventory-no-provider-lockfile",
    "module_count": 2,
    "module_names": ["cloud-account", "dns"],
    "coherence_guard": {
      "changeset": "CS-CLOUD-IAC-PROVIDER-READINESS-GATE-001",
      "gate": "cloud-iac-provider-readiness",
      "gate_file": "crates/oya-dev-cli/src/cloud_iac_provider_readiness_gate.rs",
      "runtime_mode": "local-provider-readiness-inventory-gate"
    },
    "non_claims": [
      "no provider dependency lockfile",
      "no provider installation",
      "no provider provenance verification",
      "no module signing or Sigstore execution",
      "no tofu test/plan/apply evidence"
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
