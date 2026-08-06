//! `oya gate validate cloud-iac-module-provenance` runner.
//!
//! This gate binds the local Cloud IaC module catalog to SHA-256 digests of
//! repo-local module source files. It is intentionally local and read-only: it
//! does not sign modules, resolve provider dependencies, call registries, run
//! OpenTofu, read provider credentials, or create provider lockfiles.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::slash_path;

const DEFAULT_REPO_ROOT: &str = ".";
const DEFAULT_MANIFEST: &str = "iac/manifest.json";
const DEFAULT_CATALOG: &str = "iac/tofu/modules/catalog.json";
const DEFAULT_PROVENANCE: &str = "iac/tofu/modules/provenance.json";
const GATE_NAME: &str = "cloud-iac-module-provenance";
const GATE_FILE: &str = "crates/oya-dev-cli/src/cloud_iac_module_provenance_gate.rs";
const CHANGESET_ID: &str = "CS-CLOUD-IAC-MODULE-PROVENANCE-GATE-001";
const RUNTIME_MODE: &str = "local-filesystem-sha256-module-provenance-gate";
const DIGEST_ALGORITHM: &str = "sha256";
const LOCAL_SKELETON_STATUS: &str = "local-foundation-skeleton";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacModuleProvenanceArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) catalog: PathBuf,
    pub(crate) provenance: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacModuleProvenanceReport {
    pub(crate) manifest_path: String,
    pub(crate) catalog_path: String,
    pub(crate) provenance_path: String,
    pub(crate) modules_checked: usize,
    pub(crate) files_checked: usize,
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
struct ProvenanceFileDigest {
    path: String,
    sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProvenanceModule {
    key: ModuleKey,
    source_path: String,
    main_file: String,
    release_status: String,
    evidence_ref: String,
    files: Vec<ProvenanceFileDigest>,
}

pub(crate) fn parse_cloud_iac_module_provenance_args(
    args: Vec<String>,
) -> Result<CloudIacModuleProvenanceArgs, String> {
    let mut parsed = CloudIacModuleProvenanceArgs {
        repo_root: PathBuf::from(DEFAULT_REPO_ROOT),
        manifest: PathBuf::from(DEFAULT_MANIFEST),
        catalog: PathBuf::from(DEFAULT_CATALOG),
        provenance: PathBuf::from(DEFAULT_PROVENANCE),
    };

    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--repo-root" => parsed.repo_root = take_path_arg(&mut args, "--repo-root")?,
            "--manifest" => parsed.manifest = take_path_arg(&mut args, "--manifest")?,
            "--catalog" => parsed.catalog = take_path_arg(&mut args, "--catalog")?,
            "--provenance" => parsed.provenance = take_path_arg(&mut args, "--provenance")?,
            other => {
                return Err(format!(
                    "cloud-iac-module-provenance: unknown flag {other:?}; usage: \
                     oya gate validate cloud-iac-module-provenance \
                     [--repo-root <.>] \
                     [--manifest <iac/manifest.json>] \
                     [--catalog <iac/tofu/modules/catalog.json>] \
                     [--provenance <iac/tofu/modules/provenance.json>]"
                ));
            }
        }
    }

    Ok(parsed)
}

fn take_path_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("cloud-iac-module-provenance: {flag} requires a path argument"))
}

pub(crate) fn validate_cloud_iac_module_provenance_gate(
    args: CloudIacModuleProvenanceArgs,
) -> Result<CloudIacModuleProvenanceReport, String> {
    let manifest_path = resolve_repo_path(&args.repo_root, &args.manifest);
    let catalog_path = resolve_repo_path(&args.repo_root, &args.catalog);
    let provenance_path = resolve_repo_path(&args.repo_root, &args.provenance);
    let manifest_rel = repo_relative_argument(&args.repo_root, &args.manifest)?;
    let catalog_rel = repo_relative_argument(&args.repo_root, &args.catalog)?;
    let provenance_rel = repo_relative_argument(&args.repo_root, &args.provenance)?;

    let manifest = read_json(&manifest_path, "manifest")?;
    let catalog = read_json(&catalog_path, "catalog")?;
    let provenance = read_json(&provenance_path, "provenance")?;

    let mut diagnostics = Vec::new();
    require_manifest_scope(&manifest, &catalog_rel, &provenance_rel, &mut diagnostics);
    let modules_root =
        required_repo_relative_string(&provenance, "/authority/source_path_root", &mut diagnostics)
            .unwrap_or_else(|| "iac/tofu/modules".to_string());
    let catalog_modules = parse_catalog_modules(&catalog, &modules_root, &mut diagnostics);
    let provenance_modules = parse_provenance_modules(&provenance, &modules_root, &mut diagnostics);

    validate_provenance_header(&provenance, &catalog_rel, &modules_root, &mut diagnostics);
    validate_manifest_summary(&manifest, &catalog_modules, &mut diagnostics);
    let files_checked = validate_module_provenance(
        &args.repo_root,
        &catalog_modules,
        &provenance_modules,
        &mut diagnostics,
    );

    if diagnostics.is_empty() {
        Ok(CloudIacModuleProvenanceReport {
            manifest_path: manifest_rel,
            catalog_path: catalog_rel,
            provenance_path: provenance_rel,
            modules_checked: catalog_modules.len(),
            files_checked,
        })
    } else {
        Err(format!(
            "cloud-iac-module-provenance validation failed:\n- {}",
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
                "cloud-iac-module-provenance: unable to canonicalize repo root {}: {error}",
                repo_root.display()
            )
        })?;
        let path = fs::canonicalize(path).map_err(|error| {
            format!(
                "cloud-iac-module-provenance: unable to canonicalize path {}: {error}",
                path.display()
            )
        })?;
        let relative = path.strip_prefix(&repo_root).map_err(|_| {
            format!(
                "cloud-iac-module-provenance: path {} is outside repo root {}",
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
            "cloud-iac-module-provenance: unable to read {label} {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "cloud-iac-module-provenance: unable to parse {label} JSON {}: {error}",
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
    provenance_rel: &str,
    diagnostics: &mut Vec<String>,
) {
    let manifest_catalog =
        required_repo_relative_string(manifest, "/module_provenance_scope/catalog", diagnostics);
    if manifest_catalog.as_deref() != Some(catalog_rel) {
        diagnostics.push(format!(
            "manifest /module_provenance_scope/catalog must equal {catalog_rel:?}"
        ));
    }
    let manifest_provenance =
        required_repo_relative_string(manifest, "/module_provenance_scope/provenance", diagnostics);
    if manifest_provenance.as_deref() != Some(provenance_rel) {
        diagnostics.push(format!(
            "manifest /module_provenance_scope/provenance must equal {provenance_rel:?}"
        ));
    }
    if required_string(
        manifest,
        "/module_provenance_scope/digest_algorithm",
        diagnostics,
    )
    .as_deref()
        != Some(DIGEST_ALGORITHM)
    {
        diagnostics
            .push("manifest /module_provenance_scope/digest_algorithm must be sha256".to_string());
    }
    if required_string(
        manifest,
        "/module_provenance_scope/coherence_guard/gate",
        diagnostics,
    )
    .as_deref()
        != Some(GATE_NAME)
    {
        diagnostics.push(format!(
            "manifest /module_provenance_scope/coherence_guard/gate must be {GATE_NAME:?}"
        ));
    }
    if required_string(
        manifest,
        "/module_provenance_scope/coherence_guard/changeset",
        diagnostics,
    )
    .as_deref()
        != Some(CHANGESET_ID)
    {
        diagnostics.push(format!(
            "manifest /module_provenance_scope/coherence_guard/changeset must be {CHANGESET_ID:?}"
        ));
    }
    if required_string(
        manifest,
        "/module_provenance_scope/coherence_guard/runtime_mode",
        diagnostics,
    )
    .as_deref()
        != Some(RUNTIME_MODE)
    {
        diagnostics.push(format!(
            "manifest /module_provenance_scope/coherence_guard/runtime_mode must be {RUNTIME_MODE:?}"
        ));
    }
    if required_repo_relative_string(
        manifest,
        "/module_provenance_scope/coherence_guard/gate_file",
        diagnostics,
    )
    .as_deref()
        != Some(GATE_FILE)
    {
        diagnostics.push(format!(
            "manifest /module_provenance_scope/coherence_guard/gate_file must be {GATE_FILE:?}"
        ));
    }
    let non_claims =
        required_string_array(manifest, "/module_provenance_scope/non_claims", diagnostics)
            .unwrap_or_default();
    for required in [
        "no cosign or Sigstore signing execution",
        "no provider dependency lockfile or provider provenance evidence",
        "no live private module registry API",
        "no tofu test/plan/apply evidence",
    ] {
        if !non_claims.iter().any(|claim| claim.contains(required)) {
            diagnostics.push(format!(
                "manifest /module_provenance_scope/non_claims must include {required:?}"
            ));
        }
    }
}

fn validate_provenance_header(
    provenance: &Value,
    catalog_rel: &str,
    modules_root: &str,
    diagnostics: &mut Vec<String>,
) {
    if required_string(provenance, "/digest_algorithm", diagnostics).as_deref()
        != Some(DIGEST_ALGORITHM)
    {
        diagnostics.push("provenance /digest_algorithm must be sha256".to_string());
    }
    if required_string(provenance, "/generated_by_changeset", diagnostics).as_deref()
        != Some(CHANGESET_ID)
    {
        diagnostics.push(format!(
            "provenance /generated_by_changeset must be {CHANGESET_ID:?}"
        ));
    }
    if required_repo_relative_string(provenance, "/authority/source_catalog", diagnostics)
        .as_deref()
        != Some(catalog_rel)
    {
        diagnostics.push(format!(
            "provenance /authority/source_catalog must equal {catalog_rel:?}"
        ));
    }
    if required_repo_relative_string(provenance, "/authority/source_path_root", diagnostics)
        .as_deref()
        != Some(modules_root)
    {
        diagnostics.push(format!(
            "provenance /authority/source_path_root must equal {modules_root:?}"
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

fn parse_provenance_modules(
    provenance: &Value,
    modules_root: &str,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<ModuleKey, ProvenanceModule> {
    let Some(entries) = provenance.pointer("/modules").and_then(Value::as_array) else {
        diagnostics.push("provenance /modules must be an array".to_string());
        return BTreeMap::new();
    };
    let mut modules = BTreeMap::new();
    for (idx, entry) in entries.iter().enumerate() {
        let Some(module) = parse_provenance_module(entry, idx, modules_root, diagnostics) else {
            continue;
        };
        if modules.insert(module.key.clone(), module).is_some() {
            diagnostics.push(format!("duplicate provenance module at /modules/{idx}"));
        }
    }
    modules
}

fn parse_provenance_module(
    entry: &Value,
    idx: usize,
    modules_root: &str,
    diagnostics: &mut Vec<String>,
) -> Option<ProvenanceModule> {
    let prefix = format!("provenance /modules/{idx}");
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
    let Some(files) = entry.pointer("/files").and_then(Value::as_array) else {
        diagnostics.push(format!("{prefix}/files must be an array"));
        return None;
    };
    let mut parsed_files = Vec::new();
    for (file_idx, file) in files.iter().enumerate() {
        let file_prefix = format!("{prefix}/files/{file_idx}");
        let Some(path) = required_repo_relative_string(file, "/path", diagnostics) else {
            continue;
        };
        if !path.starts_with(&format!("{source_path}/")) {
            diagnostics.push(format!(
                "{file_prefix}/path must stay under source_path {source_path:?}; found {path:?}"
            ));
        }
        let Some(sha256) = required_string(file, "/sha256", diagnostics) else {
            continue;
        };
        if !is_lower_hex_64(&sha256) {
            diagnostics.push(format!(
                "{file_prefix}/sha256 must be a 64-character lowercase hex SHA-256 digest"
            ));
        }
        parsed_files.push(ProvenanceFileDigest { path, sha256 });
    }
    Some(ProvenanceModule {
        key,
        source_path,
        main_file,
        release_status,
        evidence_ref,
        files: parsed_files,
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
        .pointer("/module_provenance_scope/module_count")
        .and_then(Value::as_u64)
    {
        if found as usize != catalog_modules.len() {
            diagnostics.push(format!(
                "manifest /module_provenance_scope/module_count must equal {}; found {found}",
                catalog_modules.len()
            ));
        }
    } else {
        diagnostics
            .push("manifest /module_provenance_scope/module_count must be a number".to_string());
    }
    let expected: Vec<String> = catalog_modules
        .values()
        .map(|module| module.key.name.clone())
        .collect();
    let found = required_string_array(
        manifest,
        "/module_provenance_scope/module_names",
        diagnostics,
    )
    .unwrap_or_default();
    if found != expected {
        diagnostics.push(format!(
            "manifest /module_provenance_scope/module_names must equal {:?}; found {:?}",
            expected, found
        ));
    }
}

fn validate_module_provenance(
    repo_root: &Path,
    catalog_modules: &BTreeMap<ModuleKey, CatalogModule>,
    provenance_modules: &BTreeMap<ModuleKey, ProvenanceModule>,
    diagnostics: &mut Vec<String>,
) -> usize {
    let catalog_keys: BTreeSet<_> = catalog_modules.keys().cloned().collect();
    let provenance_keys: BTreeSet<_> = provenance_modules.keys().cloned().collect();
    if catalog_keys != provenance_keys {
        diagnostics.push(format!(
            "provenance module keys must match catalog keys; missing={:?} extra={:?}",
            catalog_keys
                .difference(&provenance_keys)
                .map(ModuleKey::display)
                .collect::<Vec<_>>(),
            provenance_keys
                .difference(&catalog_keys)
                .map(ModuleKey::display)
                .collect::<Vec<_>>()
        ));
    }

    let mut files_checked = 0usize;
    for (key, catalog) in catalog_modules {
        let Some(provenance) = provenance_modules.get(key) else {
            continue;
        };
        validate_module_pair(catalog, provenance, diagnostics);
        files_checked += validate_file_digests(repo_root, provenance, diagnostics);
    }
    files_checked
}

fn validate_module_pair(
    catalog: &CatalogModule,
    provenance: &ProvenanceModule,
    diagnostics: &mut Vec<String>,
) {
    let key = catalog.key.display();
    if catalog.source_path != provenance.source_path {
        diagnostics.push(format!(
            "module {key} source_path drift: catalog {:?}, provenance {:?}",
            catalog.source_path, provenance.source_path
        ));
    }
    if catalog.main_file != provenance.main_file {
        diagnostics.push(format!(
            "module {key} main_file drift: catalog {:?}, provenance {:?}",
            catalog.main_file, provenance.main_file
        ));
    }
    if catalog.release_status != LOCAL_SKELETON_STATUS
        || provenance.release_status != LOCAL_SKELETON_STATUS
    {
        diagnostics.push(format!(
            "module {key} release_status must remain {LOCAL_SKELETON_STATUS:?} in catalog and provenance"
        ));
    }
    if catalog.provider_resources_implemented
        || catalog.outputs_materialized
        || catalog.tests_present
    {
        diagnostics.push(format!(
            "module {key} catalog must not claim provider resources, materialized outputs, or tests for local provenance-only skeletons"
        ));
    }
    if catalog.evidence_ref != provenance.evidence_ref {
        diagnostics.push(format!(
            "module {key} evidence_ref drift: catalog {:?}, provenance {:?}",
            catalog.evidence_ref, provenance.evidence_ref
        ));
    }
    let mut paths = BTreeSet::new();
    for file in &provenance.files {
        if !paths.insert(file.path.clone()) {
            diagnostics.push(format!(
                "module {key} duplicate provenance file {:?}",
                file.path
            ));
        }
    }
    if !paths.contains(&catalog.main_file) {
        diagnostics.push(format!(
            "module {key} provenance files must include main_file {:?}",
            catalog.main_file
        ));
    }
    let readme = format!("{}/README.md", catalog.source_path);
    if !paths.contains(&readme) {
        diagnostics.push(format!(
            "module {key} provenance files must include README.md {:?}",
            readme
        ));
    }
}

fn validate_file_digests(
    repo_root: &Path,
    provenance: &ProvenanceModule,
    diagnostics: &mut Vec<String>,
) -> usize {
    let mut checked = 0usize;
    for file in &provenance.files {
        let path = repo_root.join(&file.path);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                diagnostics.push(format!(
                    "module {} unable to read provenance file {}: {error}",
                    provenance.key.display(),
                    path.display()
                ));
                continue;
            }
        };
        let actual = sha256_hex(&bytes);
        checked += 1;
        if actual != file.sha256 {
            diagnostics.push(format!(
                "module {} digest drift for {}: expected {}, actual {}",
                provenance.key.display(),
                file.path,
                file.sha256,
                actual
            ));
        }
    }
    checked
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::{
        CloudIacModuleProvenanceArgs, parse_cloud_iac_module_provenance_args, sha256_hex,
        validate_cloud_iac_module_provenance_gate,
    };

    #[test]
    fn parse_cloud_iac_module_provenance_rejects_unknown_flag() {
        let error = parse_cloud_iac_module_provenance_args(vec!["--bogus".into()])
            .expect_err("unknown flag should fail");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn cloud_iac_module_provenance_gate_accepts_valid_fixture() {
        let temp = TempRepo::new("cloud-iac-module-provenance-valid");
        write_fixture(temp.path(), FixtureDrift::None);

        let report = validate_cloud_iac_module_provenance_gate(fixture_args(temp.path()))
            .expect("valid fixture should pass");

        assert_eq!(report.modules_checked, 2);
        assert_eq!(report.files_checked, 4);
    }

    #[test]
    fn cloud_iac_module_provenance_gate_rejects_digest_drift() {
        let temp = TempRepo::new("cloud-iac-module-provenance-digest-drift");
        write_fixture(temp.path(), FixtureDrift::DigestDrift);

        let error = validate_cloud_iac_module_provenance_gate(fixture_args(temp.path()))
            .expect_err("digest drift should fail");

        assert!(error.contains("digest drift"));
    }

    #[test]
    fn cloud_iac_module_provenance_gate_rejects_missing_main_file_digest() {
        let temp = TempRepo::new("cloud-iac-module-provenance-missing-main");
        write_fixture(temp.path(), FixtureDrift::MissingMainFileDigest);

        let error = validate_cloud_iac_module_provenance_gate(fixture_args(temp.path()))
            .expect_err("missing main digest should fail");

        assert!(error.contains("must include main_file"));
    }

    #[test]
    fn cloud_iac_module_provenance_gate_rejects_manifest_scope_drift() {
        let temp = TempRepo::new("cloud-iac-module-provenance-manifest-drift");
        write_fixture(temp.path(), FixtureDrift::ManifestScopeDrift);

        let error = validate_cloud_iac_module_provenance_gate(fixture_args(temp.path()))
            .expect_err("manifest drift should fail");

        assert!(error.contains("module_provenance_scope"));
    }

    fn fixture_args(repo_root: &Path) -> CloudIacModuleProvenanceArgs {
        CloudIacModuleProvenanceArgs {
            repo_root: repo_root.to_path_buf(),
            manifest: PathBuf::from("iac/manifest.json"),
            catalog: PathBuf::from("iac/tofu/modules/catalog.json"),
            provenance: PathBuf::from("iac/tofu/modules/provenance.json"),
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum FixtureDrift {
        None,
        DigestDrift,
        MissingMainFileDigest,
        ManifestScopeDrift,
    }

    fn write_fixture(root: &Path, drift: FixtureDrift) {
        let modules_root = root.join("iac/tofu/modules");
        for module in ["cloud-account", "dns"] {
            let module_dir = modules_root.join(module);
            fs::create_dir_all(&module_dir).expect("module dir");
            fs::write(
                module_dir.join("main.tofu"),
                format!("terraform {{\n  required_version = \">= 1.6\"\n}}\n\noutput \"name\" {{ value = \"{module}\" }}\n"),
            )
            .expect("main.tofu");
            fs::write(module_dir.join("README.md"), format!("# {module}\n")).expect("readme");
        }
        fs::write(modules_root.join("catalog.json"), fixture_catalog()).expect("catalog");
        fs::write(
            modules_root.join("provenance.json"),
            fixture_provenance(root, drift),
        )
        .expect("provenance");
        fs::create_dir_all(root.join("iac")).expect("manifest dir");
        fs::write(
            root.join("iac/manifest.json"),
            fixture_manifest(drift),
        )
        .expect("manifest");
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

    fn fixture_provenance(root: &Path, drift: FixtureDrift) -> String {
        let rows = ["cloud-account", "dns"].map(|module| {
            let source = format!("iac/tofu/modules/{module}");
            let main = format!("{source}/main.tofu");
            let readme = format!("{source}/README.md");
            let mut files = vec![(main.clone(), digest(root, &main)), (readme.clone(), digest(root, &readme))];
            if drift == FixtureDrift::DigestDrift && module == "dns" {
                files[0].1 = "0".repeat(64);
            }
            if drift == FixtureDrift::MissingMainFileDigest && module == "dns" {
                files.remove(0);
            }
            let files_json = files
                .into_iter()
                .map(|(path, sha)| format!(r#"{{"path":"{path}","sha256":"{sha}"}}"#))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"namespace":"oyatie","name":"{module}","system":"opentofu","version":"0.1.0","source_path":"{source}","main_file":"{main}","release_status":"local-foundation-skeleton","evidence_ref":"evidence://cloud-iac/modules/{module}/0.1.0/local-foundation","files":[{files_json}]}}"#
            )
        });
        format!(
            r#"{{
  "schema_version":"1.0",
  "provenance_id":"fixture",
  "generated_by_changeset":"CS-CLOUD-IAC-MODULE-PROVENANCE-GATE-001",
  "digest_algorithm":"sha256",
  "authority":{{"source_catalog":"iac/tofu/modules/catalog.json","source_path_root":"iac/tofu/modules"}},
  "modules":[{},{}]
}}
"#,
            rows[0], rows[1]
        )
    }

    fn digest(root: &Path, rel: &str) -> String {
        sha256_hex(&fs::read(root.join(rel)).expect("digest fixture file"))
    }

    fn fixture_manifest(drift: FixtureDrift) -> String {
        let provenance = if drift == FixtureDrift::ManifestScopeDrift {
            "iac/tofu/modules/wrong.json"
        } else {
            "iac/tofu/modules/provenance.json"
        };
        format!(
            r#"{{
  "module_provenance_scope": {{
    "catalog": "iac/tofu/modules/catalog.json",
    "provenance": "{provenance}",
    "digest_algorithm": "sha256",
    "module_count": 2,
    "module_names": ["cloud-account", "dns"],
    "coherence_guard": {{
      "changeset": "CS-CLOUD-IAC-MODULE-PROVENANCE-GATE-001",
      "gate": "cloud-iac-module-provenance",
      "gate_file": "crates/oya-dev-cli/src/cloud_iac_module_provenance_gate.rs",
      "runtime_mode": "local-filesystem-sha256-module-provenance-gate"
    }},
    "non_claims": [
      "no cosign or Sigstore signing execution",
      "no provider dependency lockfile or provider provenance evidence",
      "no live private module registry API",
      "no tofu test/plan/apply evidence"
    ]
  }}
}}
"#
        )
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
