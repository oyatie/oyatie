//! `oya gate validate cloud-iac-opentofu-validation` runner.
//!
//! This gate makes local OpenTofu syntax validation a permanent evidence
//! surface for the Cloud IaC module skeletons. It intentionally runs only
//! `tofu init -backend=false` and `tofu validate` in per-module temporary
//! copies, so it never writes `.terraform`, lockfiles, state, or provider
//! artifacts into the repository and never runs plan/apply/test/provider APIs.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::slash_path;

const DEFAULT_REPO_ROOT: &str = ".";
const DEFAULT_MANIFEST: &str = "iac/manifest.json";
const DEFAULT_CATALOG: &str = "iac/tofu/modules/catalog.json";
const DEFAULT_MODULES_ROOT: &str = "iac/tofu/modules";
const DEFAULT_TOFU_BIN: &str = "tofu";
const GATE_NAME: &str = "cloud-iac-opentofu-validation";
const GATE_FILE: &str = "crates/oya-dev-cli/src/cloud_iac_opentofu_validation_gate.rs";
const CHANGESET_ID: &str = "CS-CLOUD-IAC-OPENTOFU-VALIDATION-GATE-001";
const RUNTIME_MODE: &str = "temp-copy-opentofu-init-backend-false-validate";
const LOCAL_SKELETON_STATUS: &str = "local-foundation-skeleton";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacOpenTofuValidationArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) catalog: PathBuf,
    pub(crate) modules_root: PathBuf,
    pub(crate) tofu_bin: PathBuf,
    pub(crate) keep_temp: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacOpenTofuValidationReport {
    pub(crate) manifest_path: String,
    pub(crate) catalog_path: String,
    pub(crate) modules_root: String,
    pub(crate) modules_checked: usize,
    pub(crate) init_runs: usize,
    pub(crate) validate_runs: usize,
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
}

pub(crate) fn parse_cloud_iac_opentofu_validation_args(
    args: Vec<String>,
) -> Result<CloudIacOpenTofuValidationArgs, String> {
    let mut parsed = CloudIacOpenTofuValidationArgs {
        repo_root: PathBuf::from(DEFAULT_REPO_ROOT),
        manifest: PathBuf::from(DEFAULT_MANIFEST),
        catalog: PathBuf::from(DEFAULT_CATALOG),
        modules_root: PathBuf::from(DEFAULT_MODULES_ROOT),
        tofu_bin: PathBuf::from(DEFAULT_TOFU_BIN),
        keep_temp: false,
    };

    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--repo-root" => parsed.repo_root = take_path_arg(&mut args, "--repo-root")?,
            "--manifest" => parsed.manifest = take_path_arg(&mut args, "--manifest")?,
            "--catalog" => parsed.catalog = take_path_arg(&mut args, "--catalog")?,
            "--modules-root" => {
                parsed.modules_root = take_path_arg(&mut args, "--modules-root")?;
            }
            "--tofu-bin" => parsed.tofu_bin = take_path_arg(&mut args, "--tofu-bin")?,
            "--keep-temp" => parsed.keep_temp = true,
            other => {
                return Err(format!(
                    "cloud-iac-opentofu-validation: unknown flag {other:?}; usage: \
                     oya gate validate cloud-iac-opentofu-validation \
                     [--repo-root <.>] \
                     [--manifest <iac/manifest.json>] \
                     [--catalog <iac/tofu/modules/catalog.json>] \
                     [--modules-root <iac/tofu/modules>] \
                     [--tofu-bin <tofu>] [--keep-temp]"
                ));
            }
        }
    }

    Ok(parsed)
}

fn take_path_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("cloud-iac-opentofu-validation: {flag} requires a path argument"))
}

pub(crate) fn validate_cloud_iac_opentofu_validation_gate(
    args: CloudIacOpenTofuValidationArgs,
) -> Result<CloudIacOpenTofuValidationReport, String> {
    let manifest_path = resolve_repo_path(&args.repo_root, &args.manifest);
    let catalog_path = resolve_repo_path(&args.repo_root, &args.catalog);
    let modules_root_path = resolve_repo_path(&args.repo_root, &args.modules_root);
    let manifest_rel = repo_relative_argument(&args.repo_root, &args.manifest)?;
    let catalog_rel = repo_relative_argument(&args.repo_root, &args.catalog)?;
    let modules_root_rel = repo_relative_argument(&args.repo_root, &args.modules_root)?;

    let manifest = read_json(&manifest_path, "manifest")?;
    let catalog = read_json(&catalog_path, "catalog")?;

    let mut diagnostics = Vec::new();
    require_manifest_scope(&manifest, &catalog_rel, &modules_root_rel, &mut diagnostics);
    let modules = parse_catalog_modules(&catalog, &modules_root_rel, &mut diagnostics);
    validate_manifest_module_summary(&manifest, &modules, &mut diagnostics);

    if !modules_root_path.is_dir() {
        diagnostics.push(format!(
            "modules root does not exist or is not a directory: {}",
            modules_root_path.display()
        ));
    }

    let temp = TempDir::new("cloud-iac-opentofu-validation", args.keep_temp)?;
    let mut init_runs = 0usize;
    let mut validate_runs = 0usize;

    for module in &modules {
        let module_source = args.repo_root.join(&module.source_path);
        let main_file = args.repo_root.join(&module.main_file);
        validate_module_metadata(
            module,
            &modules_root_rel,
            &module_source,
            &main_file,
            &mut diagnostics,
        );
        validate_source_hygiene(module, &module_source, &mut diagnostics);
        if diagnostics.is_empty() {
            let module_temp = temp.path().join(&module.name);
            if let Err(error) = copy_dir_all(&module_source, &module_temp) {
                diagnostics.push(format!(
                    "module {}: unable to copy {} to temp dir {}: {error}",
                    module.name,
                    module_source.display(),
                    module_temp.display()
                ));
                continue;
            }
            match run_tofu_command(
                &args.tofu_bin,
                &module_temp,
                &["init", "-backend=false", "-input=false", "-no-color"],
            ) {
                Ok(()) => init_runs += 1,
                Err(error) => {
                    diagnostics.push(format!("module {}: {error}", module.name));
                    continue;
                }
            }
            match run_tofu_command(&args.tofu_bin, &module_temp, &["validate", "-no-color"]) {
                Ok(()) => validate_runs += 1,
                Err(error) => diagnostics.push(format!("module {}: {error}", module.name)),
            }
        }
    }

    if diagnostics.is_empty() {
        Ok(CloudIacOpenTofuValidationReport {
            manifest_path: manifest_rel,
            catalog_path: catalog_rel,
            modules_root: modules_root_rel,
            modules_checked: modules.len(),
            init_runs,
            validate_runs,
        })
    } else {
        Err(format!(
            "cloud-iac-opentofu-validation validation failed:\n- {}",
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
                "cloud-iac-opentofu-validation: unable to canonicalize repo root {}: {error}",
                repo_root.display()
            )
        })?;
        let path = fs::canonicalize(path).map_err(|error| {
            format!(
                "cloud-iac-opentofu-validation: unable to canonicalize path {}: {error}",
                path.display()
            )
        })?;
        let relative = path.strip_prefix(&repo_root).map_err(|_| {
            format!(
                "cloud-iac-opentofu-validation: path {} is outside repo root {}",
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
            "cloud-iac-opentofu-validation: unable to read {label} {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "cloud-iac-opentofu-validation: unable to parse {label} JSON {}: {error}",
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
    if raw.starts_with('/') {
        diagnostics.push(format!(
            "{label} must be repo-relative, found absolute path {raw:?}"
        ));
        return None;
    }
    let path = Path::new(raw);
    let mut parts = Vec::new();
    for component in path.components() {
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
        return None;
    }
    Some(parts.join("/"))
}

fn require_manifest_scope(
    manifest: &Value,
    catalog_rel: &str,
    modules_root_rel: &str,
    diagnostics: &mut Vec<String>,
) {
    let catalog = required_string(manifest, "/opentofu_validation_scope/catalog", diagnostics)
        .and_then(|raw| {
            normalize_repo_relative(
                &raw,
                "manifest /opentofu_validation_scope/catalog",
                diagnostics,
            )
        });
    if catalog.as_deref() != Some(catalog_rel) {
        diagnostics.push(format!(
            "manifest /opentofu_validation_scope/catalog must equal {catalog_rel:?}"
        ));
    }

    let modules_root = required_string(
        manifest,
        "/opentofu_validation_scope/modules_root",
        diagnostics,
    )
    .and_then(|raw| {
        normalize_repo_relative(
            &raw,
            "manifest /opentofu_validation_scope/modules_root",
            diagnostics,
        )
    });
    if modules_root.as_deref() != Some(modules_root_rel) {
        diagnostics.push(format!(
            "manifest /opentofu_validation_scope/modules_root must equal {modules_root_rel:?}"
        ));
    }

    let gate = required_string(
        manifest,
        "/opentofu_validation_scope/coherence_guard/gate",
        diagnostics,
    );
    if gate.as_deref() != Some(GATE_NAME) {
        diagnostics.push(format!(
            "manifest /opentofu_validation_scope/coherence_guard/gate must be {GATE_NAME:?}"
        ));
    }
    let changeset = required_string(
        manifest,
        "/opentofu_validation_scope/coherence_guard/changeset",
        diagnostics,
    );
    if changeset.as_deref() != Some(CHANGESET_ID) {
        diagnostics.push(format!(
            "manifest /opentofu_validation_scope/coherence_guard/changeset must be {CHANGESET_ID:?}"
        ));
    }
    let runtime_mode = required_string(
        manifest,
        "/opentofu_validation_scope/coherence_guard/runtime_mode",
        diagnostics,
    );
    if runtime_mode.as_deref() != Some(RUNTIME_MODE) {
        diagnostics.push(format!(
            "manifest /opentofu_validation_scope/coherence_guard/runtime_mode must be {RUNTIME_MODE:?}"
        ));
    }
    let gate_file = required_string(
        manifest,
        "/opentofu_validation_scope/coherence_guard/gate_file",
        diagnostics,
    )
    .and_then(|raw| {
        normalize_repo_relative(
            &raw,
            "manifest /opentofu_validation_scope/coherence_guard/gate_file",
            diagnostics,
        )
    });
    if gate_file.as_deref() != Some(GATE_FILE) {
        diagnostics.push(format!(
            "manifest /opentofu_validation_scope/coherence_guard/gate_file must be {GATE_FILE:?}"
        ));
    }

    let init = required_string(
        manifest,
        "/opentofu_validation_scope/commands/init",
        diagnostics,
    );
    if init.as_deref() != Some("tofu init -backend=false -input=false -no-color") {
        diagnostics.push(
            "manifest /opentofu_validation_scope/commands/init must record the exact backend-disabled init command".to_string(),
        );
    }
    let validate = required_string(
        manifest,
        "/opentofu_validation_scope/commands/validate",
        diagnostics,
    );
    if validate.as_deref() != Some("tofu validate -no-color") {
        diagnostics.push(
            "manifest /opentofu_validation_scope/commands/validate must record the exact validate command".to_string(),
        );
    }
    if required_bool(
        manifest,
        "/opentofu_validation_scope/temp_copy_required",
        diagnostics,
    ) != Some(true)
    {
        diagnostics.push(
            "manifest /opentofu_validation_scope/temp_copy_required must be true".to_string(),
        );
    }

    let non_claims = required_string_array(
        manifest,
        "/opentofu_validation_scope/non_claims",
        diagnostics,
    )
    .unwrap_or_default();
    for required in [
        "no tofu test evidence",
        "no tofu plan/apply evidence",
        "no provider API calls, provider credentials, or state backend access",
        "no provider dependency lockfile or provider provenance evidence",
        "no provider-resource-complete module bodies",
    ] {
        if !non_claims.iter().any(|claim| claim.contains(required)) {
            diagnostics.push(format!(
                "manifest /opentofu_validation_scope/non_claims must include {required:?}"
            ));
        }
    }
}

fn parse_catalog_modules(
    catalog: &Value,
    modules_root_rel: &str,
    diagnostics: &mut Vec<String>,
) -> Vec<CatalogModuleRow> {
    let authority_root = required_string(catalog, "/authority/source_path_root", diagnostics)
        .and_then(|raw| {
            normalize_repo_relative(&raw, "catalog /authority/source_path_root", diagnostics)
        });
    if authority_root.as_deref() != Some(modules_root_rel) {
        diagnostics.push(format!(
            "catalog /authority/source_path_root must equal modules root {modules_root_rel:?}"
        ));
    }

    let Some(entries) = catalog.pointer("/modules").and_then(Value::as_array) else {
        diagnostics.push("catalog /modules must be an array".to_string());
        return Vec::new();
    };

    let mut seen = BTreeSet::new();
    let mut modules = Vec::with_capacity(entries.len());
    for (idx, entry) in entries.iter().enumerate() {
        let prefix = format!("/modules/{idx}");
        let namespace = required_string(entry, "/namespace", diagnostics);
        let name = required_string(entry, "/name", diagnostics);
        let system = required_string(entry, "/system", diagnostics);
        let version = required_string(entry, "/version", diagnostics);
        let source_path = required_string(entry, "/source_path", diagnostics).and_then(|raw| {
            normalize_repo_relative(&raw, &format!("catalog {prefix}/source_path"), diagnostics)
        });
        let main_file = required_string(entry, "/main_file", diagnostics).and_then(|raw| {
            normalize_repo_relative(&raw, &format!("catalog {prefix}/main_file"), diagnostics)
        });
        let release_status = required_string(entry, "/release_status", diagnostics);
        let provider_resources_implemented =
            required_bool(entry, "/provider_resources_implemented", diagnostics);
        let outputs_materialized = required_bool(entry, "/outputs_materialized", diagnostics);
        let tests_present = required_bool(entry, "/tests_present", diagnostics);

        if let (
            Some(namespace),
            Some(name),
            Some(system),
            Some(version),
            Some(source_path),
            Some(main_file),
            Some(release_status),
            Some(provider_resources_implemented),
            Some(outputs_materialized),
            Some(tests_present),
        ) = (
            namespace,
            name,
            system,
            version,
            source_path,
            main_file,
            release_status,
            provider_resources_implemented,
            outputs_materialized,
            tests_present,
        ) {
            let key = format!("{namespace}/{name}/{system}/{version}");
            if !seen.insert(key.clone()) {
                diagnostics.push(format!("duplicate catalog module key {key}"));
            }
            modules.push(CatalogModuleRow {
                namespace,
                name,
                system,
                version,
                source_path,
                main_file,
                release_status,
                provider_resources_implemented,
                outputs_materialized,
                tests_present,
            });
        }
    }
    modules
}

fn validate_manifest_module_summary(
    manifest: &Value,
    modules: &[CatalogModuleRow],
    diagnostics: &mut Vec<String>,
) {
    if let Some(found) = manifest
        .pointer("/opentofu_validation_scope/module_count")
        .and_then(Value::as_u64)
    {
        if found as usize != modules.len() {
            diagnostics.push(format!(
                "manifest /opentofu_validation_scope/module_count must equal {}; found {found}",
                modules.len()
            ));
        }
    } else {
        diagnostics
            .push("manifest /opentofu_validation_scope/module_count must be a number".to_string());
    }

    let expected_names: Vec<String> = modules.iter().map(|module| module.name.clone()).collect();
    let found_names = required_string_array(
        manifest,
        "/opentofu_validation_scope/module_names",
        diagnostics,
    )
    .unwrap_or_default();
    if found_names != expected_names {
        diagnostics.push(format!(
            "manifest /opentofu_validation_scope/module_names must equal {:?}; found {:?}",
            expected_names, found_names
        ));
    }
}

fn validate_module_metadata(
    module: &CatalogModuleRow,
    modules_root_rel: &str,
    module_source: &Path,
    main_file: &Path,
    diagnostics: &mut Vec<String>,
) {
    let expected_prefix = format!("{modules_root_rel}/");
    if !module.source_path.starts_with(&expected_prefix) {
        diagnostics.push(format!(
            "module {} source_path must stay under {modules_root_rel:?}; found {:?}",
            module.name, module.source_path
        ));
    }
    let expected_main = format!("{}/main.tofu", module.source_path);
    if module.main_file != expected_main {
        diagnostics.push(format!(
            "module {} main_file must be {expected_main:?}; found {:?}",
            module.name, module.main_file
        ));
    }
    if module.release_status != LOCAL_SKELETON_STATUS {
        diagnostics.push(format!(
            "module {} release_status must remain {LOCAL_SKELETON_STATUS:?}; found {:?}",
            module.name, module.release_status
        ));
    }
    if module.provider_resources_implemented {
        diagnostics.push(format!(
            "module {} cannot claim provider_resources_implemented while validation scope is skeleton-only",
            module.name
        ));
    }
    if module.outputs_materialized {
        diagnostics.push(format!(
            "module {} cannot claim outputs_materialized while validation scope is skeleton-only",
            module.name
        ));
    }
    if module.tests_present {
        diagnostics.push(format!(
            "module {} cannot claim tests_present because this gate does not run tofu test",
            module.name
        ));
    }
    if !module_source.is_dir() {
        diagnostics.push(format!(
            "module {} source_path does not exist or is not a directory: {}",
            module.name,
            module_source.display()
        ));
    }
    if !main_file.is_file() {
        diagnostics.push(format!(
            "module {} main_file does not exist: {}",
            module.name,
            main_file.display()
        ));
    }
}

fn validate_source_hygiene(
    module: &CatalogModuleRow,
    module_source: &Path,
    diagnostics: &mut Vec<String>,
) {
    if !module_source.is_dir() {
        return;
    }
    let mut stack = vec![module_source.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) => {
                diagnostics.push(format!(
                    "module {}: unable to read {}: {error}",
                    module.name,
                    dir.display()
                ));
                continue;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    diagnostics.push(format!(
                        "module {}: unable to read directory entry under {}: {error}",
                        module.name,
                        dir.display()
                    ));
                    continue;
                }
            };
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if name == ".terraform" {
                    diagnostics.push(format!(
                        "module {} contains generated OpenTofu directory {}; run validation in temp copies only",
                        module.name,
                        path.display()
                    ));
                } else {
                    stack.push(path);
                }
                continue;
            }
            if is_forbidden_generated_artifact(&name) {
                diagnostics.push(format!(
                    "module {} contains generated/sensitive OpenTofu artifact {}; validation must not commit state, lock, tfvars, or plans",
                    module.name,
                    path.display()
                ));
                continue;
            }
            if matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("tofu" | "tf")
            ) {
                validate_configuration_text(module, &path, diagnostics);
            }
        }
    }
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

fn validate_configuration_text(
    module: &CatalogModuleRow,
    path: &Path,
    diagnostics: &mut Vec<String>,
) {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            diagnostics.push(format!(
                "module {}: unable to read configuration {}: {error}",
                module.name,
                path.display()
            ));
            return;
        }
    };
    for (idx, line) in contents.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("backend ") || trimmed.starts_with("backend\"") {
            diagnostics.push(format!(
                "module {} contains backend block at {}:{}; validation scope forbids state backend access",
                module.name,
                path.display(),
                idx + 1
            ));
        }
        if trimmed.starts_with("resource ") || trimmed.starts_with("resource\"") {
            diagnostics.push(format!(
                "module {} contains provider resource block at {}:{} while catalog marks provider_resources_implemented=false",
                module.name,
                path.display(),
                idx + 1
            ));
        }
        if trimmed.starts_with("data ") || trimmed.starts_with("data\"") {
            diagnostics.push(format!(
                "module {} contains provider data source at {}:{} while validation scope forbids provider API access",
                module.name,
                path.display(),
                idx + 1
            ));
        }
        if trimmed.starts_with("provider ") || trimmed.starts_with("provider\"") {
            diagnostics.push(format!(
                "module {} contains provider configuration at {}:{} while validation scope forbids provider credentials/API access",
                module.name,
                path.display(),
                idx + 1
            ));
        }
    }
    if contains_forbidden_secret_marker(&contents) {
        diagnostics.push(format!(
            "module {} contains high-confidence secret/credential marker in {}; remove raw credentials from IaC sources",
            module.name,
            path.display()
        ));
    }
}

fn contains_forbidden_secret_marker(contents: &str) -> bool {
    let lower = contents.to_ascii_lowercase();
    contents.contains("-----BEGIN PRIVATE KEY-----")
        || lower.contains("aws_secret_access_key")
        || lower.contains("secret_access_key")
        || lower.contains("client_secret = \"")
        || lower.contains("password = \"")
        || lower.contains("private_key = \"")
        || lower.contains("token = \"")
        || lower.contains("kubeconfig = \"")
}

fn run_tofu_command(tofu_bin: &Path, cwd: &Path, args: &[&str]) -> Result<(), String> {
    let output = Command::new(tofu_bin)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| {
            format!(
                "unable to execute {} {} in {}: {error}",
                tofu_bin.display(),
                args.join(" "),
                cwd.display()
            )
        })?;
    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "{} {} failed with exit {:?}: {}{}{}",
            tofu_bin.display(),
            args.join(" "),
            output.status.code(),
            trim_command_output(&stdout),
            if stdout.trim().is_empty() || stderr.trim().is_empty() {
                ""
            } else {
                "\n"
            },
            trim_command_output(&stderr)
        ))
    }
}

fn trim_command_output(output: &str) -> String {
    let mut lines = output.trim().lines();
    let mut selected = Vec::new();
    for _ in 0..18 {
        let Some(line) = lines.next() else {
            break;
        };
        selected.push(line.trim_end());
    }
    selected.join("\n")
}

fn copy_dir_all(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

#[derive(Debug)]
struct TempDir {
    path: PathBuf,
    keep: bool,
}

impl TempDir {
    fn new(prefix: &str, keep: bool) -> Result<Self, String> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("system clock before UNIX_EPOCH: {error}"))?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{nonce}-{}", std::process::id()));
        fs::create_dir_all(&path).map_err(|error| {
            format!(
                "cloud-iac-opentofu-validation: unable to create temp dir {}: {error}",
                path.display()
            )
        })?;
        Ok(Self { path, keep })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    use super::{
        CloudIacOpenTofuValidationArgs, parse_cloud_iac_opentofu_validation_args,
        validate_cloud_iac_opentofu_validation_gate,
    };

    #[test]
    fn parse_cloud_iac_opentofu_validation_rejects_unknown_flag() {
        let error = parse_cloud_iac_opentofu_validation_args(vec!["--bogus".into()])
            .expect_err("unknown flag should fail");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn cloud_iac_opentofu_validation_gate_accepts_valid_temp_copy_validation() {
        let temp = TempRepo::new("cloud-iac-opentofu-valid");
        write_fixture(temp.path(), FixtureDrift::None);
        let tofu = write_fake_tofu(temp.path(), FakeTofuMode::Pass);

        let report = validate_cloud_iac_opentofu_validation_gate(fixture_args(temp.path(), tofu))
            .expect("valid fixture should pass");

        assert_eq!(report.modules_checked, 2);
        assert_eq!(report.init_runs, 2);
        assert_eq!(report.validate_runs, 2);
    }

    #[test]
    fn cloud_iac_opentofu_validation_gate_rejects_validate_failure() {
        let temp = TempRepo::new("cloud-iac-opentofu-validate-fail");
        write_fixture(temp.path(), FixtureDrift::None);
        let tofu = write_fake_tofu(temp.path(), FakeTofuMode::ValidateFails);

        let error = validate_cloud_iac_opentofu_validation_gate(fixture_args(temp.path(), tofu))
            .expect_err("tofu validate failure should fail gate");

        assert!(error.contains("validate"));
        assert!(error.contains("failed with exit"));
    }

    #[test]
    fn cloud_iac_opentofu_validation_gate_rejects_generated_state_artifact() {
        let temp = TempRepo::new("cloud-iac-opentofu-state-artifact");
        write_fixture(temp.path(), FixtureDrift::GeneratedStateArtifact);
        let tofu = write_fake_tofu(temp.path(), FakeTofuMode::Pass);

        let error = validate_cloud_iac_opentofu_validation_gate(fixture_args(temp.path(), tofu))
            .expect_err("state artifact should fail gate");

        assert!(error.contains("generated/sensitive OpenTofu artifact"));
    }

    #[test]
    fn cloud_iac_opentofu_validation_gate_rejects_provider_resource_overclaim() {
        let temp = TempRepo::new("cloud-iac-opentofu-resource-overclaim");
        write_fixture(temp.path(), FixtureDrift::ProviderResourceBlock);
        let tofu = write_fake_tofu(temp.path(), FakeTofuMode::Pass);

        let error = validate_cloud_iac_opentofu_validation_gate(fixture_args(temp.path(), tofu))
            .expect_err("resource block should fail skeleton validation");

        assert!(error.contains("provider resource block"));
    }

    #[test]
    fn cloud_iac_opentofu_validation_gate_rejects_missing_manifest_scope() {
        let temp = TempRepo::new("cloud-iac-opentofu-missing-scope");
        write_fixture(temp.path(), FixtureDrift::MissingManifestScope);
        let tofu = write_fake_tofu(temp.path(), FakeTofuMode::Pass);

        let error = validate_cloud_iac_opentofu_validation_gate(fixture_args(temp.path(), tofu))
            .expect_err("missing manifest scope should fail gate");

        assert!(error.contains("opentofu_validation_scope"));
    }

    fn fixture_args(repo_root: &Path, tofu_bin: PathBuf) -> CloudIacOpenTofuValidationArgs {
        CloudIacOpenTofuValidationArgs {
            repo_root: repo_root.to_path_buf(),
            manifest: PathBuf::from("iac/manifest.json"),
            catalog: PathBuf::from("iac/tofu/modules/catalog.json"),
            modules_root: PathBuf::from("iac/tofu/modules"),
            tofu_bin,
            keep_temp: false,
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum FixtureDrift {
        None,
        GeneratedStateArtifact,
        ProviderResourceBlock,
        MissingManifestScope,
    }

    fn write_fixture(root: &Path, drift: FixtureDrift) {
        let modules_root = root.join("iac/tofu/modules");
        for module in ["cloud-account", "dns"] {
            let module_dir = modules_root.join(module);
            fs::create_dir_all(&module_dir).expect("module dir");
            fs::write(
                module_dir.join("main.tofu"),
                format!(
                    r#"terraform {{
  required_version = ">= 1.6"
}}

variable "name" {{
  type = string
}}

output "name" {{
  value = var.name
}}
{}
"#,
                    if drift == FixtureDrift::ProviderResourceBlock && module == "dns" {
                        "resource \"example\" \"bad\" {}"
                    } else {
                        ""
                    }
                ),
            )
            .expect("module main");
        }
        if drift == FixtureDrift::GeneratedStateArtifact {
            fs::write(modules_root.join("dns").join("terraform.tfstate"), "{}")
                .expect("state artifact");
        }

        fs::create_dir_all(modules_root.parent().expect("tofu dir")).expect("tofu dir");
        fs::write(
            modules_root.join("catalog.json"),
            r#"{
  "authority": { "source_path_root": "iac/tofu/modules" },
  "modules": [
    {
      "namespace": "oyatie",
      "name": "cloud-account",
      "system": "opentofu",
      "version": "0.1.0",
      "source_path": "iac/tofu/modules/cloud-account",
      "main_file": "iac/tofu/modules/cloud-account/main.tofu",
      "release_status": "local-foundation-skeleton",
      "provider_resources_implemented": false,
      "outputs_materialized": false,
      "tests_present": false
    },
    {
      "namespace": "oyatie",
      "name": "dns",
      "system": "opentofu",
      "version": "0.1.0",
      "source_path": "iac/tofu/modules/dns",
      "main_file": "iac/tofu/modules/dns/main.tofu",
      "release_status": "local-foundation-skeleton",
      "provider_resources_implemented": false,
      "outputs_materialized": false,
      "tests_present": false
    }
  ]
}
"#,
        )
        .expect("catalog");

        fs::create_dir_all(root.join("iac")).expect("manifest dir");
        fs::write(
            root.join("iac/manifest.json"),
            fixture_manifest(drift),
        )
        .expect("manifest");
    }

    fn fixture_manifest(drift: FixtureDrift) -> String {
        if drift == FixtureDrift::MissingManifestScope {
            return "{\n  \"schema_version\": \"test\"\n}\n".to_string();
        }
        r#"{
  "opentofu_validation_scope": {
    "catalog": "iac/tofu/modules/catalog.json",
    "modules_root": "iac/tofu/modules",
    "module_count": 2,
    "module_names": ["cloud-account", "dns"],
    "temp_copy_required": true,
    "commands": {
      "init": "tofu init -backend=false -input=false -no-color",
      "validate": "tofu validate -no-color"
    },
    "coherence_guard": {
      "changeset": "CS-CLOUD-IAC-OPENTOFU-VALIDATION-GATE-001",
      "crate": "oya-dev-cli",
      "gate": "cloud-iac-opentofu-validation",
      "gate_file": "crates/oya-dev-cli/src/cloud_iac_opentofu_validation_gate.rs",
      "runtime_mode": "temp-copy-opentofu-init-backend-false-validate"
    },
    "non_claims": [
      "no tofu test evidence",
      "no tofu plan/apply evidence",
      "no provider API calls, provider credentials, or state backend access",
      "no provider dependency lockfile or provider provenance evidence",
      "no provider-resource-complete module bodies"
    ]
  }
}
"#
        .to_string()
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum FakeTofuMode {
        Pass,
        ValidateFails,
    }

    fn write_fake_tofu(root: &Path, mode: FakeTofuMode) -> PathBuf {
        let script = root.join("fake-tofu.sh");
        let body = match mode {
            FakeTofuMode::Pass => {
                r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "init" ]]; then
  mkdir -p .terraform
  exit 0
fi
if [[ "${1:-}" == "validate" ]]; then
  echo 'Success! The configuration is valid.'
  exit 0
fi
echo "unexpected fake tofu args: $*" >&2
exit 2
"#
            }
            FakeTofuMode::ValidateFails => {
                r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "init" ]]; then
  mkdir -p .terraform
  exit 0
fi
if [[ "${1:-}" == "validate" ]]; then
  echo 'Error: Invalid single-argument block definition' >&2
  exit 1
fi
echo "unexpected fake tofu args: $*" >&2
exit 2
"#
            }
        };
        fs::write(&script, body).expect("fake tofu script");
        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(&script)
                .expect("script metadata")
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&script, permissions).expect("script executable");
        }
        script
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
