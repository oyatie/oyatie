//! `oya gate validate cloud-iac-module-catalog` runner.
//!
//! This gate makes the Cloud IaC local OpenTofu module catalog a permanent
//! fail-closed validation surface instead of a one-off evidence-time script.
//! It intentionally stays local: it parses JSON, checks repo-relative files,
//! and does not call OpenTofu, a registry API, cloud provider APIs, or cosign.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use crate::slash_path;

const DEFAULT_REPO_ROOT: &str = ".";
const DEFAULT_MANIFEST: &str = "iac/manifest.json";
const DEFAULT_CATALOG: &str = "iac/tofu/modules/catalog.json";
const GATE_NAME: &str = "cloud-iac-module-catalog";
const LOCAL_SKELETON_STATUS: &str = "local-foundation-skeleton";
const OPENTOFU_SYSTEM: &str = "opentofu";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacModuleCatalogValidateArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) catalog: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacModuleCatalogReport {
    pub(crate) manifest_path: String,
    pub(crate) catalog_path: String,
    pub(crate) modules_checked: usize,
    pub(crate) files_checked: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CatalogModuleRow {
    namespace: String,
    name: String,
    system: String,
    version: String,
    source_path: String,
    main_file: String,
    release_status: String,
    provider_resources_implemented: bool,
    outputs_materialized: bool,
    tests_present: bool,
    evidence_ref: String,
}

pub(crate) fn parse_cloud_iac_module_catalog_validate_args(
    args: Vec<String>,
) -> Result<CloudIacModuleCatalogValidateArgs, String> {
    let mut parsed = CloudIacModuleCatalogValidateArgs {
        repo_root: PathBuf::from(DEFAULT_REPO_ROOT),
        manifest: PathBuf::from(DEFAULT_MANIFEST),
        catalog: PathBuf::from(DEFAULT_CATALOG),
    };

    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--repo-root" => {
                parsed.repo_root = take_path_arg(&mut args, "--repo-root")?;
            }
            "--manifest" => {
                parsed.manifest = take_path_arg(&mut args, "--manifest")?;
            }
            "--catalog" => {
                parsed.catalog = take_path_arg(&mut args, "--catalog")?;
            }
            other => {
                return Err(format!(
                    "cloud-iac-module-catalog: unknown flag {other:?}; usage: \
                     oya gate validate cloud-iac-module-catalog \
                     [--repo-root <.>] \
                     [--manifest <iac/manifest.json>] \
                     [--catalog <iac/tofu/modules/catalog.json>]"
                ));
            }
        }
    }

    Ok(parsed)
}

fn take_path_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("cloud-iac-module-catalog: {flag} requires a path argument"))
}

pub(crate) fn validate_cloud_iac_module_catalog_gate(
    args: CloudIacModuleCatalogValidateArgs,
) -> Result<CloudIacModuleCatalogReport, String> {
    let manifest_path = resolve_repo_path(&args.repo_root, &args.manifest);
    let catalog_path = resolve_repo_path(&args.repo_root, &args.catalog);
    let manifest_rel = repo_relative_argument(&args.repo_root, &args.manifest)?;
    let catalog_rel = repo_relative_argument(&args.repo_root, &args.catalog)?;

    let manifest = read_json(&manifest_path, "manifest")?;
    let catalog = read_json(&catalog_path, "catalog")?;

    let mut diagnostics = Vec::new();
    let manifest_catalog =
        required_string(&manifest, "/module_library_scope/catalog", &mut diagnostics);
    if let Some(manifest_catalog) = manifest_catalog.as_deref()
        && manifest_catalog != catalog_rel
    {
        diagnostics.push(format!(
            "manifest /module_library_scope/catalog must equal {catalog_rel:?}; found {manifest_catalog:?}"
        ));
    }

    let manifest_root = required_string(
        &manifest,
        "/module_library_scope/actual_path_root",
        &mut diagnostics,
    )
    .and_then(|raw| {
        normalize_repo_relative(
            &raw,
            "manifest /module_library_scope/actual_path_root",
            &mut diagnostics,
        )
    });

    let catalog_root = required_string(&catalog, "/authority/source_path_root", &mut diagnostics)
        .and_then(|raw| {
            normalize_repo_relative(
                &raw,
                "catalog /authority/source_path_root",
                &mut diagnostics,
            )
        });

    if let (Some(manifest_root), Some(catalog_root)) =
        (manifest_root.as_deref(), catalog_root.as_deref())
        && manifest_root != catalog_root
    {
        diagnostics.push(format!(
            "manifest actual_path_root {manifest_root:?} must equal catalog authority.source_path_root {catalog_root:?}"
        ));
    }

    require_manifest_capability(&manifest, &mut diagnostics);
    require_manifest_gate_guard(&manifest, &mut diagnostics);
    require_catalog_header(&catalog, &mut diagnostics);

    let source_root = catalog_root.or(manifest_root).unwrap_or_else(|| {
        DEFAULT_CATALOG
            .trim_end_matches("/catalog.json")
            .to_string()
    });
    let modules = parse_catalog_modules(&catalog, &source_root, &args.repo_root, &mut diagnostics);

    validate_manifest_module_summary(&manifest, &modules, &mut diagnostics);

    let files_checked = validate_catalog_files(&args.repo_root, &modules, &mut diagnostics) + 2;

    if diagnostics.is_empty() {
        Ok(CloudIacModuleCatalogReport {
            manifest_path: manifest_rel,
            catalog_path: catalog_rel,
            modules_checked: modules.len(),
            files_checked,
        })
    } else {
        Err(format!(
            "cloud-iac-module-catalog validation failed:\n- {}",
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
                "cloud-iac-module-catalog: unable to canonicalize repo root {}: {error}",
                repo_root.display()
            )
        })?;
        let path = fs::canonicalize(path).map_err(|error| {
            format!(
                "cloud-iac-module-catalog: unable to canonicalize path {}: {error}",
                path.display()
            )
        })?;
        let relative = path.strip_prefix(&repo_root).map_err(|_| {
            format!(
                "cloud-iac-module-catalog: path {} is outside repo root {}",
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
            "cloud-iac-module-catalog: unable to read {label} {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "cloud-iac-module-catalog: unable to parse {label} JSON {}: {error}",
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
        capability.get("name").and_then(Value::as_str) == Some("cloud-iac-module-catalog-gate")
            && capability.get("file").and_then(Value::as_str)
                == Some("crates/oya-dev-cli/src/cloud_iac_module_catalog_gate.rs")
    });
    if !has_gate_capability {
        diagnostics.push(
            "manifest capabilities must declare cloud-iac-module-catalog-gate backed by crates/oya-dev-cli/src/cloud_iac_module_catalog_gate.rs".to_string(),
        );
    }
}

fn require_manifest_gate_guard(manifest: &Value, diagnostics: &mut Vec<String>) {
    let gate = required_string(
        manifest,
        "/module_library_scope/coherence_guard/gate",
        diagnostics,
    );
    if gate.as_deref() != Some(GATE_NAME) {
        diagnostics.push(format!(
            "manifest /module_library_scope/coherence_guard/gate must be {GATE_NAME:?}"
        ));
    }
    let mode = required_string(
        manifest,
        "/module_library_scope/coherence_guard/runtime_mode",
        diagnostics,
    );
    if mode.as_deref() != Some("local-filesystem-json-gate") {
        diagnostics.push(
            "manifest /module_library_scope/coherence_guard/runtime_mode must be \"local-filesystem-json-gate\"".to_string(),
        );
    }
}

fn require_catalog_header(catalog: &Value, diagnostics: &mut Vec<String>) {
    let schema_version = required_string(catalog, "/schema_version", diagnostics);
    if schema_version.as_deref() != Some("1.0") {
        diagnostics.push("catalog /schema_version must be \"1.0\"".to_string());
    }
    let catalog_id = required_string(catalog, "/catalog_id", diagnostics);
    if let Some(catalog_id) = catalog_id.as_deref()
        && !is_slug(catalog_id)
    {
        diagnostics.push(format!(
            "catalog_id {catalog_id:?} must be a lowercase slug"
        ));
    }
    let non_claims = catalog
        .pointer("/authority/non_claims")
        .and_then(Value::as_array);
    if non_claims.is_none_or(Vec::is_empty) {
        diagnostics.push("catalog /authority/non_claims must be a non-empty array".to_string());
    }
}

fn parse_catalog_modules(
    catalog: &Value,
    source_root: &str,
    repo_root: &Path,
    diagnostics: &mut Vec<String>,
) -> Vec<CatalogModuleRow> {
    let Some(modules) = catalog.pointer("/modules").and_then(Value::as_array) else {
        diagnostics.push("catalog /modules must be a non-empty array".to_string());
        return Vec::new();
    };
    if modules.is_empty() {
        diagnostics.push("catalog /modules must be a non-empty array".to_string());
        return Vec::new();
    }

    let mut rows = Vec::with_capacity(modules.len());
    let mut keys = BTreeSet::new();
    for (idx, module) in modules.iter().enumerate() {
        let prefix = format!("/modules/{idx}");
        let Some(row) = parse_catalog_module_row(module, &prefix, diagnostics) else {
            continue;
        };

        validate_catalog_module_row(&row, source_root, repo_root, &prefix, diagnostics);

        let key = format!(
            "{}/{}/{}/{}",
            row.namespace, row.name, row.system, row.version
        );
        if !keys.insert(key.clone()) {
            diagnostics.push(format!("{prefix} duplicates module release key {key:?}"));
        }
        rows.push(row);
    }
    rows
}

fn parse_catalog_module_row(
    module: &Value,
    prefix: &str,
    diagnostics: &mut Vec<String>,
) -> Option<CatalogModuleRow> {
    if !module.is_object() {
        diagnostics.push(format!("{prefix} must be an object"));
        return None;
    }
    let namespace = required_string(module, "/namespace", diagnostics);
    let name = required_string(module, "/name", diagnostics);
    let system = required_string(module, "/system", diagnostics);
    let version = required_string(module, "/version", diagnostics);
    let source_path = required_string(module, "/source_path", diagnostics);
    let main_file = required_string(module, "/main_file", diagnostics);
    let release_status = required_string(module, "/release_status", diagnostics);
    let provider_resources_implemented =
        required_bool(module, "/provider_resources_implemented", diagnostics);
    let outputs_materialized = required_bool(module, "/outputs_materialized", diagnostics);
    let tests_present = required_bool(module, "/tests_present", diagnostics);
    let evidence_ref = required_string(module, "/evidence_ref", diagnostics);

    Some(CatalogModuleRow {
        namespace: namespace?,
        name: name?,
        system: system?,
        version: version?,
        source_path: source_path?,
        main_file: main_file?,
        release_status: release_status?,
        provider_resources_implemented: provider_resources_implemented?,
        outputs_materialized: outputs_materialized?,
        tests_present: tests_present?,
        evidence_ref: evidence_ref?,
    })
}

fn validate_catalog_module_row(
    row: &CatalogModuleRow,
    source_root: &str,
    repo_root: &Path,
    prefix: &str,
    diagnostics: &mut Vec<String>,
) {
    if !is_slug(&row.namespace) {
        diagnostics.push(format!("{prefix}/namespace must be a lowercase slug"));
    }
    if !is_slug(&row.name) {
        diagnostics.push(format!("{prefix}/name must be a lowercase slug"));
    }
    if row.system != OPENTOFU_SYSTEM {
        diagnostics.push(format!("{prefix}/system must be {OPENTOFU_SYSTEM:?}"));
    }
    if !is_exact_semver(&row.version) {
        diagnostics.push(format!(
            "{prefix}/version {:?} must be exact MAJOR.MINOR.PATCH semver",
            row.version
        ));
    }

    validate_module_paths(row, source_root, repo_root, prefix, diagnostics);

    if row.release_status != LOCAL_SKELETON_STATUS {
        diagnostics.push(format!(
            "{prefix}/release_status must remain {LOCAL_SKELETON_STATUS:?} until provider-resource-complete module bodies are implemented"
        ));
    }
    if row.release_status == LOCAL_SKELETON_STATUS
        && (row.provider_resources_implemented || row.outputs_materialized || row.tests_present)
    {
        diagnostics.push(format!(
            "{prefix} local-foundation-skeleton entries must not claim provider resources, outputs, or tests"
        ));
    }

    let evidence_prefix = format!("evidence://cloud-iac/modules/{}/{}/", row.name, row.version);
    if !row.evidence_ref.starts_with(&evidence_prefix) {
        diagnostics.push(format!(
            "{prefix}/evidence_ref must start with {evidence_prefix:?}"
        ));
    }
    if contains_secret_like_marker(&row.evidence_ref) {
        diagnostics.push(format!(
            "{prefix}/evidence_ref contains secret-like material marker"
        ));
    }
}

fn validate_module_paths(
    row: &CatalogModuleRow,
    source_root: &str,
    repo_root: &Path,
    prefix: &str,
    diagnostics: &mut Vec<String>,
) {
    let Some(source_path) = normalize_repo_relative(
        &row.source_path,
        &format!("{prefix}/source_path"),
        diagnostics,
    ) else {
        return;
    };
    let Some(main_file) =
        normalize_repo_relative(&row.main_file, &format!("{prefix}/main_file"), diagnostics)
    else {
        return;
    };

    let expected_source_path = format!("{source_root}/{}", row.name);
    if source_path != expected_source_path {
        diagnostics.push(format!(
            "{prefix}/source_path must be {expected_source_path:?}; found {source_path:?}"
        ));
    }
    let expected_main_file = format!("{source_path}/main.tofu");
    if main_file != expected_main_file {
        diagnostics.push(format!(
            "{prefix}/main_file must be {expected_main_file:?}; found {main_file:?}"
        ));
    }

    let source_dir = repo_root.join(&source_path);
    if !source_dir.is_dir() {
        diagnostics.push(format!(
            "{prefix}/source_path directory does not exist: {}",
            source_dir.display()
        ));
    }
    let main_file_path = repo_root.join(&main_file);
    if !main_file_path.is_file() {
        diagnostics.push(format!(
            "{prefix}/main_file does not exist: {}",
            main_file_path.display()
        ));
    }
}

fn validate_manifest_module_summary(
    manifest: &Value,
    modules: &[CatalogModuleRow],
    diagnostics: &mut Vec<String>,
) {
    match manifest
        .pointer("/module_library_scope/module_count")
        .and_then(Value::as_u64)
    {
        Some(count) if count == modules.len() as u64 => {}
        Some(count) => diagnostics.push(format!(
            "manifest /module_library_scope/module_count must equal catalog module count {}; found {count}",
            modules.len()
        )),
        None => diagnostics.push(
            "manifest /module_library_scope/module_count must be an unsigned integer".to_string(),
        ),
    }

    if let Some(module_names) =
        required_string_array(manifest, "/module_library_scope/module_names", diagnostics)
    {
        let catalog_names = modules
            .iter()
            .map(|module| module.name.clone())
            .collect::<Vec<_>>();
        if module_names != catalog_names {
            diagnostics.push(format!(
                "manifest /module_library_scope/module_names must exactly match catalog order {:?}; found {:?}",
                catalog_names, module_names
            ));
        }
    }

    if let Some(modeled_fields) = required_string_array(
        manifest,
        "/module_library_scope/registry_protocol_fields_modeled",
        diagnostics,
    ) {
        for required in [
            "namespace",
            "name",
            "system",
            "version",
            "source_path",
            "main_file",
            "release_status",
            "evidence_ref",
        ] {
            if !modeled_fields.iter().any(|field| field == required) {
                diagnostics.push(format!(
                    "manifest registry_protocol_fields_modeled must include {required:?}"
                ));
            }
        }
    }
}

fn validate_catalog_files(
    repo_root: &Path,
    modules: &[CatalogModuleRow],
    diagnostics: &mut Vec<String>,
) -> usize {
    let mut checked = 0usize;
    for module in modules {
        checked += 1;
        if contains_secret_like_marker(&module.main_file) {
            diagnostics.push(format!(
                "module {} main_file contains secret-like material marker",
                module.name
            ));
        }
        let path = repo_root.join(&module.main_file);
        if !path.is_file() {
            diagnostics.push(format!(
                "module {} main_file missing during file pass: {}",
                module.name,
                path.display()
            ));
        }
    }
    checked
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
        "token=",
        "private_key",
        "private-key",
        "kubeconfig:",
        "bearer ",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parse_cloud_iac_module_catalog_defaults_to_live_paths() {
        let parsed = parse_cloud_iac_module_catalog_validate_args(Vec::new()).expect("defaults");
        assert_eq!(parsed.repo_root, PathBuf::from(DEFAULT_REPO_ROOT));
        assert_eq!(parsed.manifest, PathBuf::from(DEFAULT_MANIFEST));
        assert_eq!(parsed.catalog, PathBuf::from(DEFAULT_CATALOG));
    }

    #[test]
    fn parse_cloud_iac_module_catalog_rejects_unknown_flag() {
        let error = parse_cloud_iac_module_catalog_validate_args(vec!["--bogus".into()])
            .expect_err("unknown flag rejected");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn cloud_iac_module_catalog_gate_accepts_coherent_fixture() {
        let temp = TempRepo::new("cloud-iac-catalog-valid");
        write_fixture(
            temp.path(),
            fixture_manifest(&["dns", "vpc"]),
            fixture_catalog(false),
        );

        let report = validate_cloud_iac_module_catalog_gate(fixture_args(temp.path()))
            .expect("coherent fixture passes");

        assert_eq!(report.modules_checked, 2);
        assert_eq!(report.files_checked, 4);
    }

    #[test]
    fn cloud_iac_module_catalog_gate_rejects_manifest_count_drift() {
        let temp = TempRepo::new("cloud-iac-catalog-count-drift");
        let mut manifest = fixture_manifest(&["dns", "vpc"]);
        manifest["module_library_scope"]["module_count"] = Value::from(1_u64);
        write_fixture(temp.path(), manifest, fixture_catalog(false));

        let error = validate_cloud_iac_module_catalog_gate(fixture_args(temp.path()))
            .expect_err("count drift fails");

        assert!(error.contains("module_count"));
    }

    #[test]
    fn cloud_iac_module_catalog_gate_rejects_skeleton_overclaim() {
        let temp = TempRepo::new("cloud-iac-catalog-overclaim");
        write_fixture(
            temp.path(),
            fixture_manifest(&["dns", "vpc"]),
            fixture_catalog(true),
        );

        let error = validate_cloud_iac_module_catalog_gate(fixture_args(temp.path()))
            .expect_err("skeleton overclaim fails");

        assert!(error.contains("must not claim provider resources"));
    }

    #[test]
    fn cloud_iac_module_catalog_gate_rejects_missing_main_file() {
        let temp = TempRepo::new("cloud-iac-catalog-missing-main");
        write_fixture(
            temp.path(),
            fixture_manifest(&["dns", "vpc"]),
            fixture_catalog(false),
        );
        fs::remove_file(
            temp.path()
                .join("iac/tofu/modules/vpc/main.tofu"),
        )
        .expect("main file removed");

        let error = validate_cloud_iac_module_catalog_gate(fixture_args(temp.path()))
            .expect_err("missing main file fails");

        assert!(error.contains("main_file does not exist"));
    }

    fn fixture_args(root: &Path) -> CloudIacModuleCatalogValidateArgs {
        CloudIacModuleCatalogValidateArgs {
            repo_root: root.to_path_buf(),
            manifest: PathBuf::from(DEFAULT_MANIFEST),
            catalog: PathBuf::from(DEFAULT_CATALOG),
        }
    }

    fn fixture_manifest(names: &[&str]) -> Value {
        serde_json::json!({
            "capabilities": [
                {
                    "tier": "T1",
                    "name": "cloud-iac-module-catalog-gate",
                    "file": "crates/oya-dev-cli/src/cloud_iac_module_catalog_gate.rs",
                    "risk_class": "high"
                }
            ],
            "module_library_scope": {
                "catalog": DEFAULT_CATALOG,
                "actual_path_root": "iac/tofu/modules",
                "catalog_status": "local-foundation-skeleton-index",
                "module_count": names.len(),
                "module_names": names,
                "registry_protocol_fields_modeled": [
                    "namespace",
                    "name",
                    "system",
                    "version",
                    "source_path",
                    "main_file",
                    "release_status",
                    "evidence_ref"
                ],
                "coherence_guard": {
                    "gate": GATE_NAME,
                    "runtime_mode": "local-filesystem-json-gate"
                }
            }
        })
    }

    fn fixture_catalog(overclaim_dns: bool) -> Value {
        serde_json::json!({
            "schema_version": "1.0",
            "catalog_id": "cloud-iac-opentofu-modules-local-foundation",
            "authority": {
                "source_path_root": "iac/tofu/modules",
                "non_claims": ["not a live OpenTofu private registry API"]
            },
            "modules": [
                fixture_module("dns", overclaim_dns),
                fixture_module("vpc", false)
            ]
        })
    }

    fn fixture_module(name: &str, overclaim: bool) -> Value {
        serde_json::json!({
            "namespace": "oyatie",
            "name": name,
            "system": "opentofu",
            "version": "0.1.0",
            "source_path": format!("iac/tofu/modules/{name}"),
            "main_file": format!("iac/tofu/modules/{name}/main.tofu"),
            "release_status": LOCAL_SKELETON_STATUS,
            "provider_resources_implemented": overclaim,
            "outputs_materialized": false,
            "tests_present": false,
            "evidence_ref": format!("evidence://cloud-iac/modules/{name}/0.1.0/local-foundation")
        })
    }

    fn write_fixture(root: &Path, manifest: Value, catalog: Value) {
        fs::create_dir_all(root.join("iac/tofu/modules/dns")).expect("dns dir");
        fs::create_dir_all(root.join("iac/tofu/modules/vpc")).expect("vpc dir");
        fs::write(
            root.join("iac/manifest.json"),
            serde_json::to_string_pretty(&manifest).expect("manifest serializes"),
        )
        .expect("manifest written");
        fs::write(
            root.join("iac/tofu/modules/catalog.json"),
            serde_json::to_string_pretty(&catalog).expect("catalog serializes"),
        )
        .expect("catalog written");
        for name in ["dns", "vpc"] {
            fs::write(
                root.join(format!(
                    "iac/tofu/modules/{name}/main.tofu"
                )),
                "# local foundation skeleton\n",
            )
            .expect("main.tofu written");
        }
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
