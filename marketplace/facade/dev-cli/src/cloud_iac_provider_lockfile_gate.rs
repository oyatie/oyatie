//! `oya gate validate cloud-iac-provider-lockfile` runner.
//!
//! This gate verifies the first honest provider dependency lock step for Cloud
//! IaC: a repo-local OpenTofu root configuration containing only
//! `required_providers` plus a committed `.terraform.lock.hcl` populated for
//! the minimum supported platforms. It does not install providers into the
//! source tree, configure providers, run `tofu init`, run plan/apply, read
//! credentials, or provision cloud resources.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use crate::slash_path;

const DEFAULT_REPO_ROOT: &str = ".";
const DEFAULT_MANIFEST: &str = "iac/manifest.json";
const DEFAULT_READINESS: &str = "iac/tofu/modules/provider-readiness.json";
const DEFAULT_LOCK_ROOT: &str = "iac/tofu/provider-locks/foundation";
const GATE_NAME: &str = "cloud-iac-provider-lockfile";
const GATE_FILE: &str = "crates/oya-dev-cli/src/cloud_iac_provider_lockfile_gate.rs";
const CHANGESET_ID: &str = "CS-CLOUD-IAC-PROVIDER-LOCKFILE-GATE-001";
const RUNTIME_MODE: &str = "local-opentofu-provider-lockfile-gate";
const LOCKFILE_STATUS: &str = "locked-multi-platform-no-provider-install";
const PROVIDERS_FILE: &str = "providers.tofu";
const LOCKFILE_NAME: &str = ".terraform.lock.hcl";
const MODULES_ROOT: &str = "iac/tofu/modules";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacProviderLockfileArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) readiness: PathBuf,
    pub(crate) lock_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacProviderLockfileReport {
    pub(crate) manifest_path: String,
    pub(crate) readiness_path: String,
    pub(crate) lock_root_path: String,
    pub(crate) providers_checked: usize,
    pub(crate) platforms_checked: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProviderRequirement {
    local_name: String,
    family: String,
    source: String,
    constraint: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct HclProviderRequirement {
    source: String,
    version: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct LockProviderBlock {
    version: String,
    constraints: String,
    h1_hashes: usize,
    zh_hashes: usize,
}

pub(crate) fn parse_cloud_iac_provider_lockfile_args(
    args: Vec<String>,
) -> Result<CloudIacProviderLockfileArgs, String> {
    let mut parsed = CloudIacProviderLockfileArgs {
        repo_root: PathBuf::from(DEFAULT_REPO_ROOT),
        manifest: PathBuf::from(DEFAULT_MANIFEST),
        readiness: PathBuf::from(DEFAULT_READINESS),
        lock_root: PathBuf::from(DEFAULT_LOCK_ROOT),
    };

    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--repo-root" => parsed.repo_root = take_path_arg(&mut args, "--repo-root")?,
            "--manifest" => parsed.manifest = take_path_arg(&mut args, "--manifest")?,
            "--readiness" => parsed.readiness = take_path_arg(&mut args, "--readiness")?,
            "--lock-root" => parsed.lock_root = take_path_arg(&mut args, "--lock-root")?,
            other => {
                return Err(format!(
                    "cloud-iac-provider-lockfile: unknown flag {other:?}; usage: \
                     oya gate validate cloud-iac-provider-lockfile \
                     [--repo-root <.>] \
                     [--manifest <iac/manifest.json>] \
                     [--readiness <iac/tofu/modules/provider-readiness.json>] \
                     [--lock-root <iac/tofu/provider-locks/foundation>]"
                ));
            }
        }
    }

    Ok(parsed)
}

fn take_path_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("cloud-iac-provider-lockfile: {flag} requires a path argument"))
}

pub(crate) fn validate_cloud_iac_provider_lockfile_gate(
    args: CloudIacProviderLockfileArgs,
) -> Result<CloudIacProviderLockfileReport, String> {
    let manifest_path = resolve_repo_path(&args.repo_root, &args.manifest);
    let readiness_path = resolve_repo_path(&args.repo_root, &args.readiness);
    let lock_root_path = resolve_repo_path(&args.repo_root, &args.lock_root);
    let manifest_rel = repo_relative_argument(&args.repo_root, &args.manifest)?;
    let readiness_rel = repo_relative_argument(&args.repo_root, &args.readiness)?;
    let lock_root_rel = repo_relative_argument(&args.repo_root, &args.lock_root)?;

    let manifest = read_json(&manifest_path, "manifest")?;
    let readiness = read_json(&readiness_path, "provider readiness")?;

    let mut diagnostics = Vec::new();
    let platforms =
        require_manifest_scope(&manifest, &readiness_rel, &lock_root_rel, &mut diagnostics);
    validate_readiness_policy(&readiness, &mut diagnostics);
    let requirements = parse_readiness_provider_requirements(&readiness, &mut diagnostics);
    validate_lock_root_path(&lock_root_rel, &lock_root_path, &mut diagnostics);
    validate_providers_file(&lock_root_path, &requirements, &mut diagnostics);
    let locked_providers = validate_lockfile(&lock_root_path, &requirements, &mut diagnostics);
    validate_manifest_summary(
        &manifest,
        &requirements,
        &locked_providers,
        &mut diagnostics,
    );

    if diagnostics.is_empty() {
        Ok(CloudIacProviderLockfileReport {
            manifest_path: manifest_rel,
            readiness_path: readiness_rel,
            lock_root_path: lock_root_rel,
            providers_checked: requirements.len(),
            platforms_checked: platforms.len(),
        })
    } else {
        Err(format!(
            "cloud-iac-provider-lockfile validation failed:\n- {}",
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
                "cloud-iac-provider-lockfile: unable to canonicalize repo root {}: {error}",
                repo_root.display()
            )
        })?;
        let path = fs::canonicalize(path).map_err(|error| {
            format!(
                "cloud-iac-provider-lockfile: unable to canonicalize path {}: {error}",
                path.display()
            )
        })?;
        let relative = path.strip_prefix(&repo_root).map_err(|_| {
            format!(
                "cloud-iac-provider-lockfile: path {} is outside repo root {}",
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

fn read_json(path: &Path, label: &str) -> Result<Value, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "cloud-iac-provider-lockfile: unable to read {label} {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "cloud-iac-provider-lockfile: unable to parse {label} JSON {}: {error}",
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

fn required_string_object(
    value: &Value,
    pointer: &str,
    diagnostics: &mut Vec<String>,
) -> Option<BTreeMap<String, String>> {
    let Some(object) = value.pointer(pointer).and_then(Value::as_object) else {
        diagnostics.push(format!("{pointer} must be an object with string values"));
        return None;
    };
    let mut out = BTreeMap::new();
    for (key, entry) in object {
        match entry.as_str() {
            Some(found) if !found.trim().is_empty() => {
                out.insert(key.clone(), found.trim().to_string());
            }
            _ => diagnostics.push(format!("{pointer}/{key} must be a non-empty string")),
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

fn require_manifest_scope(
    manifest: &Value,
    readiness_rel: &str,
    lock_root_rel: &str,
    diagnostics: &mut Vec<String>,
) -> Vec<String> {
    let expected_providers_file = format!("{lock_root_rel}/{PROVIDERS_FILE}");
    let expected_lockfile = format!("{lock_root_rel}/{LOCKFILE_NAME}");
    if required_repo_relative_string(manifest, "/provider_lockfile_scope/readiness", diagnostics)
        .as_deref()
        != Some(readiness_rel)
    {
        diagnostics.push(format!(
            "manifest /provider_lockfile_scope/readiness must equal {readiness_rel:?}"
        ));
    }
    if required_repo_relative_string(manifest, "/provider_lockfile_scope/lock_root", diagnostics)
        .as_deref()
        != Some(lock_root_rel)
    {
        diagnostics.push(format!(
            "manifest /provider_lockfile_scope/lock_root must equal {lock_root_rel:?}"
        ));
    }
    if required_repo_relative_string(
        manifest,
        "/provider_lockfile_scope/providers_file",
        diagnostics,
    )
    .as_deref()
        != Some(expected_providers_file.as_str())
    {
        diagnostics.push(format!(
            "manifest /provider_lockfile_scope/providers_file must equal {:?}",
            expected_providers_file
        ));
    }
    if required_repo_relative_string(manifest, "/provider_lockfile_scope/lockfile", diagnostics)
        .as_deref()
        != Some(expected_lockfile.as_str())
    {
        diagnostics.push(format!(
            "manifest /provider_lockfile_scope/lockfile must equal {:?}",
            expected_lockfile
        ));
    }
    if required_string(manifest, "/provider_lockfile_scope/status", diagnostics).as_deref()
        != Some(LOCKFILE_STATUS)
    {
        diagnostics.push(format!(
            "manifest /provider_lockfile_scope/status must be {LOCKFILE_STATUS:?}"
        ));
    }
    if required_string(
        manifest,
        "/provider_lockfile_scope/runtime_mode",
        diagnostics,
    )
    .as_deref()
        != Some(RUNTIME_MODE)
    {
        diagnostics.push(format!(
            "manifest /provider_lockfile_scope/runtime_mode must be {RUNTIME_MODE:?}"
        ));
    }
    validate_manifest_capability(manifest, diagnostics);
    if required_string(
        manifest,
        "/provider_lockfile_scope/coherence_guard/gate",
        diagnostics,
    )
    .as_deref()
        != Some(GATE_NAME)
    {
        diagnostics.push(format!(
            "manifest /provider_lockfile_scope/coherence_guard/gate must be {GATE_NAME:?}"
        ));
    }
    if required_string(
        manifest,
        "/provider_lockfile_scope/coherence_guard/changeset",
        diagnostics,
    )
    .as_deref()
        != Some(CHANGESET_ID)
    {
        diagnostics.push(format!(
            "manifest /provider_lockfile_scope/coherence_guard/changeset must be {CHANGESET_ID:?}"
        ));
    }
    if required_string(
        manifest,
        "/provider_lockfile_scope/coherence_guard/runtime_mode",
        diagnostics,
    )
    .as_deref()
        != Some(RUNTIME_MODE)
    {
        diagnostics.push(format!(
            "manifest /provider_lockfile_scope/coherence_guard/runtime_mode must be {RUNTIME_MODE:?}"
        ));
    }
    if required_repo_relative_string(
        manifest,
        "/provider_lockfile_scope/coherence_guard/gate_file",
        diagnostics,
    )
    .as_deref()
        != Some(GATE_FILE)
    {
        diagnostics.push(format!(
            "manifest /provider_lockfile_scope/coherence_guard/gate_file must be {GATE_FILE:?}"
        ));
    }
    let platforms =
        required_string_array(manifest, "/provider_lockfile_scope/platforms", diagnostics)
            .unwrap_or_default();
    for required in ["darwin_arm64", "linux_amd64", "linux_arm64"] {
        if !platforms.iter().any(|platform| platform == required) {
            diagnostics.push(format!(
                "manifest /provider_lockfile_scope/platforms must include {required:?}"
            ));
        }
    }
    let non_claims =
        required_string_array(manifest, "/provider_lockfile_scope/non_claims", diagnostics)
            .unwrap_or_default();
    for required in [
        "no provider installation in source tree",
        "no provider configuration or credentials",
        "no tofu test/plan/apply evidence",
        "cloud resource provisioning",
    ] {
        if !non_claims.iter().any(|claim| claim.contains(required)) {
            diagnostics.push(format!(
                "manifest /provider_lockfile_scope/non_claims must include {required:?}"
            ));
        }
    }
    platforms
}

fn validate_manifest_capability(manifest: &Value, diagnostics: &mut Vec<String>) {
    let Some(capabilities) = manifest.pointer("/capabilities").and_then(Value::as_array) else {
        diagnostics.push("manifest /capabilities must be an array".to_string());
        return;
    };
    let Some(capability) = capabilities.iter().find(|entry| {
        entry.pointer("/name").and_then(Value::as_str) == Some("cloud-iac-provider-lockfile-gate")
    }) else {
        diagnostics.push(
            "manifest /capabilities must include cloud-iac-provider-lockfile-gate".to_string(),
        );
        return;
    };
    if capability.pointer("/file").and_then(Value::as_str) != Some(GATE_FILE) {
        diagnostics.push(format!(
            "manifest provider lockfile capability /file must be {GATE_FILE:?}"
        ));
    }
    if capability.pointer("/tier").and_then(Value::as_str) != Some("T1") {
        diagnostics.push("manifest provider lockfile capability /tier must be \"T1\"".to_string());
    }
    if capability.pointer("/risk_class").and_then(Value::as_str) != Some("high") {
        diagnostics
            .push("manifest provider lockfile capability /risk_class must be \"high\"".to_string());
    }
}

fn validate_readiness_policy(readiness: &Value, diagnostics: &mut Vec<String>) {
    if required_bool(
        readiness,
        "/policy/provider_lockfiles_materialized",
        diagnostics,
    ) != Some(false)
    {
        diagnostics.push(
            "readiness /policy/provider_lockfiles_materialized must remain false; this gate owns the separate lock-root evidence".to_string(),
        );
    }
    for pointer in [
        "/policy/provider_installation_executed",
        "/policy/provider_provenance_verified",
        "/policy/module_signing_executed",
    ] {
        if required_bool(readiness, pointer, diagnostics) != Some(false) {
            diagnostics.push(format!("readiness {pointer} must remain false"));
        }
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

fn parse_readiness_provider_requirements(
    readiness: &Value,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, ProviderRequirement> {
    let Some(modules) = readiness.pointer("/modules").and_then(Value::as_array) else {
        diagnostics.push("readiness /modules must be an array".to_string());
        return BTreeMap::new();
    };
    let mut requirements = BTreeMap::new();
    for (module_idx, module) in modules.iter().enumerate() {
        if required_bool(module, "/provider_lockfile_materialized", diagnostics) != Some(false) {
            diagnostics.push(format!(
                "readiness /modules/{module_idx}/provider_lockfile_materialized must remain false"
            ));
        }
        if required_bool(module, "/provider_resources_implemented", diagnostics) != Some(false) {
            diagnostics.push(format!(
                "readiness /modules/{module_idx}/provider_resources_implemented must remain false"
            ));
        }
        let Some(families) = module
            .pointer("/provider_families")
            .and_then(Value::as_array)
        else {
            diagnostics.push(format!(
                "readiness /modules/{module_idx}/provider_families must be an array"
            ));
            continue;
        };
        for (family_idx, family) in families.iter().enumerate() {
            let family_name = required_string(family, "/family", diagnostics).unwrap_or_default();
            let source = required_string(family, "/source", diagnostics).unwrap_or_default();
            let local_name =
                required_string(family, "/preferred_local_name", diagnostics).unwrap_or_default();
            let constraint = required_string(family, "/minimum_version_constraint", diagnostics)
                .unwrap_or_default();
            if !is_minimum_version_constraint(&constraint) {
                diagnostics.push(format!(
                    "readiness /modules/{module_idx}/provider_families/{family_idx}/minimum_version_constraint must use >= x.y.z"
                ));
            }
            for pointer in [
                "/future_lock_required",
                "/future_signature_review_required",
                "/future_provider_provenance_required",
            ] {
                if required_bool(family, pointer, diagnostics) != Some(true) {
                    diagnostics.push(format!(
                        "readiness /modules/{module_idx}/provider_families/{family_idx}{pointer} must be true"
                    ));
                }
            }
            if source.is_empty() {
                continue;
            }
            let requirement = ProviderRequirement {
                local_name,
                family: family_name,
                source: source.clone(),
                constraint,
            };
            match requirements.get(&source) {
                Some(existing) if existing != &requirement => diagnostics.push(format!(
                    "provider source {source:?} has conflicting readiness requirements"
                )),
                Some(_) => {}
                None => {
                    requirements.insert(source, requirement);
                }
            }
        }
    }
    requirements
}

fn validate_lock_root_path(lock_root_rel: &str, lock_root: &Path, diagnostics: &mut Vec<String>) {
    if lock_root_rel.starts_with(MODULES_ROOT) {
        diagnostics.push(format!(
            "provider lock root must be outside reusable module tree {MODULES_ROOT:?}; found {lock_root_rel:?}"
        ));
    }
    if !lock_root.is_dir() {
        diagnostics.push(format!(
            "provider lock root does not exist or is not a directory: {}",
            lock_root.display()
        ));
        return;
    }
    if lock_root.join(".terraform").exists() {
        diagnostics.push(format!(
            "provider lock root must not contain provider installation cache .terraform: {}",
            lock_root.join(".terraform").display()
        ));
    }
}

fn validate_providers_file(
    lock_root: &Path,
    requirements: &BTreeMap<String, ProviderRequirement>,
    diagnostics: &mut Vec<String>,
) {
    let path = lock_root.join(PROVIDERS_FILE);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) => {
            diagnostics.push(format!(
                "unable to read provider requirements root {}: {error}",
                path.display()
            ));
            return;
        }
    };
    if contains_forbidden_runtime_hcl(&contents) {
        diagnostics.push(format!(
            "{} must contain only terraform.required_providers metadata, not provider/resource/data blocks",
            path.display()
        ));
    }
    if !contents.contains("required_version") || !contents.contains("required_providers") {
        diagnostics.push(format!(
            "{} must declare required_version and required_providers",
            path.display()
        ));
    }
    let parsed = parse_required_providers(&contents);
    let expected_locals: BTreeSet<_> = requirements
        .values()
        .map(|requirement| requirement.local_name.as_str())
        .collect();
    let found_locals: BTreeSet<_> = parsed.keys().map(String::as_str).collect();
    if expected_locals != found_locals {
        diagnostics.push(format!(
            "{} required provider local names must match readiness; missing={:?} extra={:?}",
            path.display(),
            expected_locals
                .difference(&found_locals)
                .collect::<Vec<_>>(),
            found_locals
                .difference(&expected_locals)
                .collect::<Vec<_>>()
        ));
    }
    for requirement in requirements.values() {
        match parsed.get(&requirement.local_name) {
            Some(found)
                if found.source == requirement.source && found.version == requirement.constraint => {}
            Some(found) => diagnostics.push(format!(
                "{} provider {} must use source {:?} and version {:?}; found source {:?}, version {:?}",
                path.display(),
                requirement.local_name,
                requirement.source,
                requirement.constraint,
                found.source,
                found.version
            )),
            None => {}
        }
    }
}

fn validate_lockfile(
    lock_root: &Path,
    requirements: &BTreeMap<String, ProviderRequirement>,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, LockProviderBlock> {
    let path = lock_root.join(LOCKFILE_NAME);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) => {
            diagnostics.push(format!(
                "unable to read provider dependency lockfile {}: {error}",
                path.display()
            ));
            return BTreeMap::new();
        }
    };
    if contains_secret_like_marker(&contents) {
        diagnostics.push(format!(
            "{} must not contain credential-like material markers",
            path.display()
        ));
    }
    let parsed = parse_lockfile_provider_blocks(&contents);
    let expected_sources: BTreeSet<_> = requirements.keys().map(String::as_str).collect();
    let found_sources: BTreeSet<_> = parsed.keys().map(String::as_str).collect();
    if expected_sources != found_sources {
        diagnostics.push(format!(
            "{} provider sources must match readiness; missing={:?} extra={:?}",
            path.display(),
            expected_sources
                .difference(&found_sources)
                .collect::<Vec<_>>(),
            found_sources
                .difference(&expected_sources)
                .collect::<Vec<_>>()
        ));
    }
    for (source, requirement) in requirements {
        let Some(found) = parsed.get(source) else {
            continue;
        };
        if found.constraints != requirement.constraint {
            diagnostics.push(format!(
                "lockfile provider {source} constraints must be {:?}; found {:?}",
                requirement.constraint, found.constraints
            ));
        }
        if !selected_version_satisfies_minimum(&found.version, &requirement.constraint) {
            diagnostics.push(format!(
                "lockfile provider {source} selected version {:?} must satisfy {:?}",
                found.version, requirement.constraint
            ));
        }
        if found.h1_hashes == 0 || found.zh_hashes < 3 {
            diagnostics.push(format!(
                "lockfile provider {source} must include h1 and multi-platform zh checksums; found h1={}, zh={}",
                found.h1_hashes, found.zh_hashes
            ));
        }
    }
    parsed
}

fn validate_manifest_summary(
    manifest: &Value,
    requirements: &BTreeMap<String, ProviderRequirement>,
    locked_providers: &BTreeMap<String, LockProviderBlock>,
    diagnostics: &mut Vec<String>,
) {
    let expected_sources: Vec<_> = requirements.keys().cloned().collect();
    let found_sources = required_string_array(
        manifest,
        "/provider_lockfile_scope/provider_sources",
        diagnostics,
    )
    .unwrap_or_default();
    if found_sources != expected_sources {
        diagnostics.push(format!(
            "manifest /provider_lockfile_scope/provider_sources must equal sorted readiness provider sources; expected={expected_sources:?} found={found_sources:?}"
        ));
    }

    let expected_constraints: BTreeMap<String, String> = requirements
        .iter()
        .map(|(source, requirement)| (source.clone(), requirement.constraint.clone()))
        .collect();
    let found_constraints = required_string_object(
        manifest,
        "/provider_lockfile_scope/minimum_constraints",
        diagnostics,
    )
    .unwrap_or_default();
    if found_constraints != expected_constraints {
        diagnostics.push(format!(
            "manifest /provider_lockfile_scope/minimum_constraints must mirror readiness; expected={expected_constraints:?} found={found_constraints:?}"
        ));
    }

    let expected_versions: BTreeMap<String, String> = locked_providers
        .iter()
        .map(|(source, provider)| (source.clone(), provider.version.clone()))
        .collect();
    let found_versions = required_string_object(
        manifest,
        "/provider_lockfile_scope/provider_versions_selected",
        diagnostics,
    )
    .unwrap_or_default();
    if found_versions != expected_versions {
        diagnostics.push(format!(
            "manifest /provider_lockfile_scope/provider_versions_selected must mirror .terraform.lock.hcl; expected={expected_versions:?} found={found_versions:?}"
        ));
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

fn parse_lockfile_provider_blocks(contents: &str) -> BTreeMap<String, LockProviderBlock> {
    let mut parsed = BTreeMap::new();
    let mut current_source: Option<String> = None;
    let mut current = LockProviderBlock::default();
    let mut block_depth = 0i32;

    for raw_line in contents.lines() {
        let active = strip_hcl_line_comment(raw_line).trim();
        if active.is_empty() {
            continue;
        }
        if current_source.is_none() {
            if let Some(source) = provider_lock_block_source(active) {
                current_source = Some(source);
                current = LockProviderBlock::default();
                block_depth = hcl_brace_delta(active);
            }
            continue;
        }
        if let Some(version) = quoted_assignment(active, "version") {
            current.version = version;
        }
        if let Some(constraints) = quoted_assignment(active, "constraints") {
            current.constraints = constraints;
        }
        if active.contains("\"h1:") {
            current.h1_hashes += 1;
        }
        if active.contains("\"zh:") {
            current.zh_hashes += 1;
        }
        block_depth += hcl_brace_delta(active);
        if block_depth <= 0
            && let Some(source) = current_source.take()
        {
            parsed.insert(source, current.clone());
            current = LockProviderBlock::default();
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

fn provider_lock_block_source(line: &str) -> Option<String> {
    let rest = line.strip_prefix("provider ")?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let (source, tail) = rest.split_once('"')?;
    if tail.trim_start().starts_with('{') {
        Some(source.to_string())
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
        active.starts_with("provider \"")
            || active.starts_with("resource \"")
            || active.starts_with("data \"")
            || active.starts_with("backend \"")
            || active.starts_with("variable \"")
            || active.starts_with("output \"")
            || active.starts_with("module \"")
    })
}

fn contains_secret_like_marker(contents: &str) -> bool {
    let lowered = contents.to_ascii_lowercase();
    [
        "-----begin",
        "password",
        "secret_key",
        "private_key",
        "client_secret",
        "token=",
        "kubeconfig",
    ]
    .iter()
    .any(|marker| lowered.contains(marker))
}

fn is_slug(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_minimum_version_constraint(value: &str) -> bool {
    let Some(version) = value.strip_prefix(">= ") else {
        return false;
    };
    is_semver(version)
}

fn selected_version_satisfies_minimum(version: &str, constraint: &str) -> bool {
    let Some(minimum) = constraint.strip_prefix(">= ") else {
        return false;
    };
    match (parse_semver_tuple(version), parse_semver_tuple(minimum)) {
        (Some(version), Some(minimum)) => version >= minimum,
        _ => false,
    }
}

fn is_semver(value: &str) -> bool {
    parse_semver_tuple(value).is_some()
}

fn parse_semver_tuple(value: &str) -> Option<(u64, u64, u64)> {
    let parts: Vec<_> = value.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let mut parsed = [0u64; 3];
    for (idx, part) in parts.iter().enumerate() {
        if part.is_empty()
            || !part.bytes().all(|byte| byte.is_ascii_digit())
            || (part.len() > 1 && part.starts_with('0'))
        {
            return None;
        }
        parsed[idx] = part.parse().ok()?;
    }
    Some((parsed[0], parsed[1], parsed[2]))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::{
        CloudIacProviderLockfileArgs, parse_cloud_iac_provider_lockfile_args,
        validate_cloud_iac_provider_lockfile_gate,
    };

    #[test]
    fn parse_cloud_iac_provider_lockfile_rejects_unknown_flag() {
        let error = parse_cloud_iac_provider_lockfile_args(vec!["--bogus".into()])
            .expect_err("unknown flag should fail");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn cloud_iac_provider_lockfile_gate_accepts_valid_fixture() {
        let temp = TempRepo::new("cloud-iac-provider-lockfile-valid");
        write_fixture(temp.path(), FixtureDrift::None);

        let report = validate_cloud_iac_provider_lockfile_gate(fixture_args(temp.path()))
            .expect("valid fixture should pass");

        assert_eq!(report.providers_checked, 2);
        assert_eq!(report.platforms_checked, 3);
    }

    #[test]
    fn cloud_iac_provider_lockfile_gate_rejects_missing_provider_requirement() {
        let temp = TempRepo::new("cloud-iac-provider-lockfile-missing-provider");
        write_fixture(temp.path(), FixtureDrift::MissingProviderRequirement);

        let error = validate_cloud_iac_provider_lockfile_gate(fixture_args(temp.path()))
            .expect_err("missing provider requirement should fail");

        assert!(error.contains("required provider local names must match readiness"));
    }

    #[test]
    fn cloud_iac_provider_lockfile_gate_rejects_lockfile_without_hashes() {
        let temp = TempRepo::new("cloud-iac-provider-lockfile-hashes");
        write_fixture(temp.path(), FixtureDrift::MissingHashes);

        let error = validate_cloud_iac_provider_lockfile_gate(fixture_args(temp.path()))
            .expect_err("missing hashes should fail");

        assert!(error.contains("multi-platform zh checksums"));
    }

    #[test]
    fn cloud_iac_provider_lockfile_gate_rejects_provider_install_cache() {
        let temp = TempRepo::new("cloud-iac-provider-lockfile-cache");
        write_fixture(temp.path(), FixtureDrift::ProviderInstallCache);

        let error = validate_cloud_iac_provider_lockfile_gate(fixture_args(temp.path()))
            .expect_err("provider cache should fail");

        assert!(error.contains("must not contain provider installation cache"));
    }

    #[test]
    fn cloud_iac_provider_lockfile_gate_rejects_runtime_hcl_blocks() {
        let temp = TempRepo::new("cloud-iac-provider-lockfile-runtime-hcl");
        write_fixture(temp.path(), FixtureDrift::RuntimeHclBlock);

        let error = validate_cloud_iac_provider_lockfile_gate(fixture_args(temp.path()))
            .expect_err("runtime hcl should fail");

        assert!(error.contains("must contain only terraform.required_providers metadata"));
    }

    fn fixture_args(repo_root: &Path) -> CloudIacProviderLockfileArgs {
        CloudIacProviderLockfileArgs {
            repo_root: repo_root.to_path_buf(),
            manifest: PathBuf::from("iac/manifest.json"),
            readiness: PathBuf::from(
                "iac/tofu/modules/provider-readiness.json",
            ),
            lock_root: PathBuf::from("iac/tofu/provider-locks/foundation"),
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum FixtureDrift {
        None,
        MissingProviderRequirement,
        MissingHashes,
        ProviderInstallCache,
        RuntimeHclBlock,
    }

    fn write_fixture(root: &Path, drift: FixtureDrift) {
        let lock_root = root.join("iac/tofu/provider-locks/foundation");
        fs::create_dir_all(&lock_root).expect("lock root");
        if drift == FixtureDrift::ProviderInstallCache {
            fs::create_dir_all(lock_root.join(".terraform/providers")).expect("provider cache");
        }
        fs::write(lock_root.join("providers.tofu"), fixture_providers(drift)).expect("providers");
        fs::write(
            lock_root.join(".terraform.lock.hcl"),
            fixture_lockfile(drift),
        )
        .expect("lockfile");
        fs::create_dir_all(root.join("iac/tofu/modules")).expect("modules");
        fs::write(
            root.join("iac/tofu/modules/provider-readiness.json"),
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

    fn fixture_providers(drift: FixtureDrift) -> String {
        let cloudflare = if drift == FixtureDrift::MissingProviderRequirement {
            ""
        } else {
            r#"
    cloudflare = {
      source  = "registry.opentofu.org/cloudflare/cloudflare"
      version = ">= 4.0.0"
    }
"#
        };
        let runtime_block = if drift == FixtureDrift::RuntimeHclBlock {
            r#"
variable "region" {
  type = string
}
"#
        } else {
            ""
        };
        format!(
            r#"terraform {{
  required_version = ">= 1.6"
  required_providers {{
    aws = {{
      source  = "registry.opentofu.org/hashicorp/aws"
      version = ">= 5.0.0"
    }}{cloudflare}
  }}
}}
{runtime_block}
"#
        )
    }

    fn fixture_lockfile(drift: FixtureDrift) -> String {
        let hashes = if drift == FixtureDrift::MissingHashes {
            "[]"
        } else {
            r#"[
    "h1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa=",
    "zh:1111111111111111111111111111111111111111111111111111111111111111",
    "zh:2222222222222222222222222222222222222222222222222222222222222222",
    "zh:3333333333333333333333333333333333333333333333333333333333333333",
  ]"#
        };
        format!(
            r#"provider "registry.opentofu.org/hashicorp/aws" {{
  version     = "6.0.0"
  constraints = ">= 5.0.0"
  hashes = {hashes}
}}

provider "registry.opentofu.org/cloudflare/cloudflare" {{
  version     = "5.0.0"
  constraints = ">= 4.0.0"
  hashes = {hashes}
}}
"#
        )
    }

    fn fixture_readiness() -> String {
        r#"{
  "policy": {
    "provider_lockfiles_materialized": false,
    "provider_installation_executed": false,
    "provider_provenance_verified": false,
    "module_signing_executed": false,
    "minimum_future_lock_platforms": ["darwin_arm64", "linux_amd64", "linux_arm64"]
  },
  "modules": [
    {"provider_lockfile_materialized": false, "provider_resources_implemented": false, "provider_families": [
      {"family": "aws", "source": "registry.opentofu.org/hashicorp/aws", "preferred_local_name": "aws", "minimum_version_constraint": ">= 5.0.0", "future_lock_required": true, "future_signature_review_required": true, "future_provider_provenance_required": true},
      {"family": "cloudflare", "source": "registry.opentofu.org/cloudflare/cloudflare", "preferred_local_name": "cloudflare", "minimum_version_constraint": ">= 4.0.0", "future_lock_required": true, "future_signature_review_required": true, "future_provider_provenance_required": true}
    ]}
  ]
}
"#
        .to_string()
    }

    fn fixture_manifest() -> String {
        r#"{
  "capabilities": [
    {
      "tier": "T1",
      "name": "cloud-iac-provider-lockfile-gate",
      "file": "crates/oya-dev-cli/src/cloud_iac_provider_lockfile_gate.rs",
      "risk_class": "high"
    }
  ],
  "provider_lockfile_scope": {
    "readiness": "iac/tofu/modules/provider-readiness.json",
    "lock_root": "iac/tofu/provider-locks/foundation",
    "providers_file": "iac/tofu/provider-locks/foundation/providers.tofu",
    "lockfile": "iac/tofu/provider-locks/foundation/.terraform.lock.hcl",
    "status": "locked-multi-platform-no-provider-install",
    "runtime_mode": "local-opentofu-provider-lockfile-gate",
    "platforms": ["darwin_arm64", "linux_amd64", "linux_arm64"],
    "provider_sources": [
      "registry.opentofu.org/cloudflare/cloudflare",
      "registry.opentofu.org/hashicorp/aws"
    ],
    "provider_versions_selected": {
      "registry.opentofu.org/cloudflare/cloudflare": "5.0.0",
      "registry.opentofu.org/hashicorp/aws": "6.0.0"
    },
    "minimum_constraints": {
      "registry.opentofu.org/cloudflare/cloudflare": ">= 4.0.0",
      "registry.opentofu.org/hashicorp/aws": ">= 5.0.0"
    },
    "coherence_guard": {
      "changeset": "CS-CLOUD-IAC-PROVIDER-LOCKFILE-GATE-001",
      "gate": "cloud-iac-provider-lockfile",
      "gate_file": "crates/oya-dev-cli/src/cloud_iac_provider_lockfile_gate.rs",
      "runtime_mode": "local-opentofu-provider-lockfile-gate"
    },
    "non_claims": [
      "no provider installation in source tree",
      "no provider configuration or credentials",
      "no tofu test/plan/apply evidence",
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
