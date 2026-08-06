//! `oya gate validate cloud-iac-module-registry-protocol` runner.
//!
//! This gate validates repo-local OpenTofu module registry protocol fixtures for
//! service discovery, versions, and download response shapes. It intentionally
//! proves only protocol-shape metadata bound to the local release index and
//! deterministic archive manifest: no private registry API service, live service
//! discovery endpoint, live download endpoint, registry publish path, signing,
//! SLSA/VSA attestation, provider runtime, tofu plan/apply, state backend, or
//! cloud provisioning is implemented by this gate.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use crate::slash_path;

const DEFAULT_REPO_ROOT: &str = ".";
const DEFAULT_MANIFEST: &str = "iac/manifest.json";
const DEFAULT_RELEASE_INDEX: &str = "iac/tofu/modules/release-index.json";
const DEFAULT_ARCHIVE_MANIFEST: &str = "iac/tofu/modules/archive-manifest.json";
const DEFAULT_PROTOCOL_FIXTURES: &str =
    "iac/tofu/module-registry/protocol-fixtures.json";
const GATE_NAME: &str = "cloud-iac-module-registry-protocol";
const GATE_FILE: &str = "crates/oya-dev-cli/src/cloud_iac_module_registry_protocol_gate.rs";
const CHANGESET_ID: &str = "CS-CLOUD-IAC-MODULE-REGISTRY-PROTOCOL-GATE-001";
const RUNTIME_MODE: &str = "local-opentofu-module-registry-protocol-fixture-gate";
const PROTOCOL_STATUS: &str = "local-registry-protocol-fixtures-no-service-runtime";
const DISCOVERY_PATH: &str = "/.well-known/terraform.json";
const MODULES_V1_BASE_PATH: &str = "/v1/modules/";
const ARTIFACT_BASE_PATH: &str = "/artifacts/modules/";
const JSON_MEDIA_TYPE: &str = "application/json";
const ARCHIVE_MEDIA_TYPE: &str = "archive/zip";
const HTTP_METHOD_GET: &str = "GET";
const MODULE_SIGNATURE_STATUS: &str = "unsigned-no-cosign";
const SLSA_STATUS: &str = "not-generated";
const HTTP_ARCHIVE_FIXTURE_SOURCE_KIND: &str = "http-archive-fixture-no-live-endpoint";
const OBJECT_SOURCE_FIXTURE_SOURCE_KIND: &str = "object-source-fixture-no-live-endpoint";
const REQUIRED_OFFICIAL_SOURCES: &[&str] = &[
    "https://opentofu.org/docs/internals/remote-service-discovery/",
    "https://opentofu.org/docs/internals/module-registry-protocol/",
    "https://opentofu.org/docs/language/modules/sources/",
    "https://opentofu.org/docs/cli/oci_registries/module-package/",
    "https://slsa.dev/spec/v1.2/",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacModuleRegistryProtocolArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) release_index: PathBuf,
    pub(crate) archive_manifest: PathBuf,
    pub(crate) protocol_fixtures: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacModuleRegistryProtocolReport {
    pub(crate) manifest_path: String,
    pub(crate) release_index_path: String,
    pub(crate) archive_manifest_path: String,
    pub(crate) protocol_fixtures_path: String,
    pub(crate) modules_checked: usize,
    pub(crate) versions_responses_checked: usize,
    pub(crate) download_responses_checked: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ModuleIdentity {
    key: String,
    address: String,
    namespace: String,
    name: String,
    system: String,
    version: String,
    release_status: String,
    evidence_ref: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ReleaseModule {
    identity: ModuleIdentity,
    versions_endpoint_path: String,
    download_endpoint_path: String,
    archive_manifest_ref: String,
    archive_file: String,
    archive_sha256: String,
    archive_media_type: String,
    archive_source_location: Option<String>,
    archive_source_integrity_sha256: Option<String>,
    archive_source_version_id: Option<String>,
    archive_source_generation: Option<String>,
    module_package_built: bool,
    module_signature_status: String,
    slsa_provenance_status: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ArchiveModule {
    identity: ModuleIdentity,
    archive_file: String,
    archive_sha256: String,
    archive_media_type: String,
    module_signature_status: String,
    slsa_provenance_status: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ProtocolModule {
    identity: ModuleIdentity,
    versions_method: String,
    versions_path: String,
    versions_status: u64,
    versions_media_type: String,
    versions_body_version: String,
    download_method: String,
    download_path: String,
    download_status: u64,
    download_media_type: String,
    download_location: String,
    artifact_archive_file: String,
    artifact_archive_sha256: String,
    artifact_url_path: String,
    artifact_media_type: String,
    artifact_source_kind: String,
    module_signature_status: String,
    slsa_provenance_status: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PathRefs<'a> {
    release_index_rel: &'a str,
    archive_manifest_rel: &'a str,
    protocol_fixtures_rel: &'a str,
}

pub(crate) fn parse_cloud_iac_module_registry_protocol_args(
    args: Vec<String>,
) -> Result<CloudIacModuleRegistryProtocolArgs, String> {
    let mut parsed = CloudIacModuleRegistryProtocolArgs {
        repo_root: PathBuf::from(DEFAULT_REPO_ROOT),
        manifest: PathBuf::from(DEFAULT_MANIFEST),
        release_index: PathBuf::from(DEFAULT_RELEASE_INDEX),
        archive_manifest: PathBuf::from(DEFAULT_ARCHIVE_MANIFEST),
        protocol_fixtures: PathBuf::from(DEFAULT_PROTOCOL_FIXTURES),
    };

    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--repo-root" => parsed.repo_root = take_path_arg(&mut args, "--repo-root")?,
            "--manifest" => parsed.manifest = take_path_arg(&mut args, "--manifest")?,
            "--release-index" => {
                parsed.release_index = take_path_arg(&mut args, "--release-index")?;
            }
            "--archive-manifest" => {
                parsed.archive_manifest = take_path_arg(&mut args, "--archive-manifest")?;
            }
            "--protocol-fixtures" => {
                parsed.protocol_fixtures = take_path_arg(&mut args, "--protocol-fixtures")?;
            }
            other => {
                return Err(format!(
                    "cloud-iac-module-registry-protocol: unknown flag {other:?}; usage: \
                     oya gate validate cloud-iac-module-registry-protocol \
                     [--repo-root <.>] \
                     [--manifest <iac/manifest.json>] \
                     [--release-index <iac/tofu/modules/release-index.json>] \
                     [--archive-manifest <iac/tofu/modules/archive-manifest.json>] \
                     [--protocol-fixtures <iac/tofu/module-registry/protocol-fixtures.json>]"
                ));
            }
        }
    }

    Ok(parsed)
}

fn take_path_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next().map(PathBuf::from).ok_or_else(|| {
        format!("cloud-iac-module-registry-protocol: {flag} requires a path argument")
    })
}

pub(crate) fn validate_cloud_iac_module_registry_protocol_gate(
    args: CloudIacModuleRegistryProtocolArgs,
) -> Result<CloudIacModuleRegistryProtocolReport, String> {
    let manifest_path = resolve_repo_path(&args.repo_root, &args.manifest);
    let release_index_path = resolve_repo_path(&args.repo_root, &args.release_index);
    let archive_manifest_path = resolve_repo_path(&args.repo_root, &args.archive_manifest);
    let protocol_fixtures_path = resolve_repo_path(&args.repo_root, &args.protocol_fixtures);

    let manifest_rel = repo_relative_argument(&args.repo_root, &args.manifest)?;
    let release_index_rel = repo_relative_argument(&args.repo_root, &args.release_index)?;
    let archive_manifest_rel = repo_relative_argument(&args.repo_root, &args.archive_manifest)?;
    let protocol_fixtures_rel = repo_relative_argument(&args.repo_root, &args.protocol_fixtures)?;
    let refs = PathRefs {
        release_index_rel: &release_index_rel,
        archive_manifest_rel: &archive_manifest_rel,
        protocol_fixtures_rel: &protocol_fixtures_rel,
    };

    let manifest = read_json(&manifest_path, "manifest")?;
    let release_index = read_json(&release_index_path, "module release index")?;
    let archive_manifest = read_json(&archive_manifest_path, "module archive manifest")?;
    let protocol_fixtures =
        read_json(&protocol_fixtures_path, "module registry protocol fixtures")?;

    let mut diagnostics = Vec::new();
    validate_manifest_scope(&manifest, refs, &mut diagnostics);
    validate_fixture_header(&protocol_fixtures, refs, &mut diagnostics);
    validate_fixture_policy(&protocol_fixtures, &mut diagnostics);
    validate_service_discovery(&protocol_fixtures, &mut diagnostics);
    validate_no_secret_markers(&protocol_fixtures, &mut diagnostics);

    let release_modules = parse_release_modules(&release_index, &mut diagnostics);
    let archive_modules = parse_archive_modules(&archive_manifest, &mut diagnostics);
    let protocol_modules = parse_protocol_modules(&protocol_fixtures, &mut diagnostics);
    validate_module_sets(
        &release_modules,
        &archive_modules,
        &protocol_modules,
        refs,
        &mut diagnostics,
    );
    validate_manifest_summary(&manifest, &protocol_modules, &mut diagnostics);

    if diagnostics.is_empty() {
        Ok(CloudIacModuleRegistryProtocolReport {
            manifest_path: manifest_rel,
            release_index_path: release_index_rel,
            archive_manifest_path: archive_manifest_rel,
            protocol_fixtures_path: protocol_fixtures_rel,
            modules_checked: protocol_modules.len(),
            versions_responses_checked: protocol_modules.len(),
            download_responses_checked: protocol_modules.len(),
        })
    } else {
        Err(format!(
            "cloud-iac-module-registry-protocol validation failed:\n- {}",
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
                "cloud-iac-module-registry-protocol: unable to canonicalize repo root {}: {error}",
                repo_root.display()
            )
        })?;
        let path = fs::canonicalize(path).map_err(|error| {
            format!(
                "cloud-iac-module-registry-protocol: unable to canonicalize path {}: {error}",
                path.display()
            )
        })?;
        let relative = path.strip_prefix(&repo_root).map_err(|_| {
            format!(
                "cloud-iac-module-registry-protocol: path {} is outside repo root {}",
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

fn normalize_url_path(raw: &str, label: &str, diagnostics: &mut Vec<String>) -> Option<String> {
    let raw = raw.trim();
    if !raw.starts_with('/') {
        diagnostics.push(format!("{label} must be an absolute URL path: {raw:?}"));
        return None;
    }
    if raw.contains("//") || raw.contains("/../") || raw.ends_with("/..") {
        diagnostics.push(format!("{label} must be normalized: {raw:?}"));
        return None;
    }
    if raw.chars().any(char::is_whitespace) {
        diagnostics.push(format!("{label} must not contain whitespace: {raw:?}"));
        return None;
    }
    Some(raw.to_string())
}

fn normalize_module_source_location(
    raw: &str,
    label: &str,
    diagnostics: &mut Vec<String>,
) -> Option<String> {
    let raw = raw.trim();
    if raw.starts_with('/') {
        return normalize_url_path(raw, label, diagnostics);
    }
    if object_source_provider(raw).is_some() && !has_unsafe_object_source_text(raw) {
        return Some(raw.to_string());
    }
    diagnostics.push(format!(
        "{label} must be an absolute URL path or safe s3::https:// / gcs::https:// object source: {raw:?}"
    ));
    None
}

fn read_json(path: &Path, label: &str) -> Result<Value, String> {
    let contents = fs::read_to_string(path).map_err(|error| {
        format!(
            "cloud-iac-module-registry-protocol: unable to read {label} {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "cloud-iac-module-registry-protocol: unable to parse {label} JSON {}: {error}",
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

fn required_u64(value: &Value, pointer: &str, diagnostics: &mut Vec<String>) -> Option<u64> {
    match value.pointer(pointer) {
        Some(Value::Number(found)) => found.as_u64().or_else(|| {
            diagnostics.push(format!("{pointer} must be a non-negative integer"));
            None
        }),
        Some(_) => {
            diagnostics.push(format!("{pointer} must be a non-negative integer"));
            None
        }
        None => {
            diagnostics.push(format!("missing required integer {pointer}"));
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

fn required_url_path_string(
    value: &Value,
    pointer: &str,
    diagnostics: &mut Vec<String>,
) -> Option<String> {
    required_string(value, pointer, diagnostics)
        .and_then(|raw| normalize_url_path(&raw, &format!("JSON {pointer}"), diagnostics))
}

fn required_module_source_location_string(
    value: &Value,
    pointer: &str,
    diagnostics: &mut Vec<String>,
) -> Option<String> {
    required_string(value, pointer, diagnostics).and_then(|raw| {
        normalize_module_source_location(&raw, &format!("JSON {pointer}"), diagnostics)
    })
}

fn optional_string(value: &Value, pointer: &str, diagnostics: &mut Vec<String>) -> Option<String> {
    match value.pointer(pointer) {
        Some(Value::String(found)) if !found.trim().is_empty() => Some(found.trim().to_string()),
        Some(_) => {
            diagnostics.push(format!("{pointer} must be a non-empty string when present"));
            None
        }
        None => None,
    }
}

fn validate_manifest_scope(manifest: &Value, refs: PathRefs<'_>, diagnostics: &mut Vec<String>) {
    for (pointer, expected) in [
        (
            "/module_registry_protocol_scope/release_index",
            refs.release_index_rel,
        ),
        (
            "/module_registry_protocol_scope/archive_manifest",
            refs.archive_manifest_rel,
        ),
        (
            "/module_registry_protocol_scope/protocol_fixtures",
            refs.protocol_fixtures_rel,
        ),
    ] {
        if required_repo_relative_string(manifest, pointer, diagnostics).as_deref()
            != Some(expected)
        {
            diagnostics.push(format!("manifest {pointer} must equal {expected:?}"));
        }
    }
    for (pointer, expected) in [
        ("/module_registry_protocol_scope/status", PROTOCOL_STATUS),
        ("/module_registry_protocol_scope/runtime_mode", RUNTIME_MODE),
        (
            "/module_registry_protocol_scope/service_discovery_path",
            DISCOVERY_PATH,
        ),
        (
            "/module_registry_protocol_scope/modules_v1_base_path",
            MODULES_V1_BASE_PATH,
        ),
        (
            "/module_registry_protocol_scope/artifact_base_path",
            ARTIFACT_BASE_PATH,
        ),
        (
            "/module_registry_protocol_scope/coherence_guard/changeset",
            CHANGESET_ID,
        ),
        (
            "/module_registry_protocol_scope/coherence_guard/gate",
            GATE_NAME,
        ),
        (
            "/module_registry_protocol_scope/coherence_guard/gate_file",
            GATE_FILE,
        ),
        (
            "/module_registry_protocol_scope/coherence_guard/runtime_mode",
            RUNTIME_MODE,
        ),
    ] {
        if required_string(manifest, pointer, diagnostics).as_deref() != Some(expected) {
            diagnostics.push(format!("manifest {pointer} must be {expected:?}"));
        }
    }
    validate_manifest_capability(manifest, diagnostics);
    validate_foundation_nonclaim(manifest, diagnostics);
    validate_required_source_array(
        manifest,
        "/module_registry_protocol_scope/official_sources_consulted",
        diagnostics,
    );
    validate_nonclaims(
        manifest,
        "/module_registry_protocol_scope/non_claims",
        diagnostics,
    );
}

fn validate_manifest_capability(manifest: &Value, diagnostics: &mut Vec<String>) {
    let Some(capabilities) = manifest.pointer("/capabilities").and_then(Value::as_array) else {
        diagnostics.push("manifest /capabilities must be an array".to_string());
        return;
    };
    let Some(capability) = capabilities.iter().find(|entry| {
        entry.pointer("/name").and_then(Value::as_str)
            == Some("cloud-iac-module-registry-protocol-gate")
    }) else {
        diagnostics.push(
            "manifest /capabilities must include cloud-iac-module-registry-protocol-gate"
                .to_string(),
        );
        return;
    };
    if capability.pointer("/file").and_then(Value::as_str) != Some(GATE_FILE) {
        diagnostics.push(format!(
            "manifest cloud-iac-module-registry-protocol-gate /file must be {GATE_FILE:?}"
        ));
    }
    if capability.pointer("/tier").and_then(Value::as_str) != Some("T1") {
        diagnostics
            .push("manifest module-registry-protocol capability /tier must be T1".to_string());
    }
    if capability.pointer("/risk_class").and_then(Value::as_str) != Some("high") {
        diagnostics.push(
            "manifest module-registry-protocol capability /risk_class must be high".to_string(),
        );
    }
}

fn validate_foundation_nonclaim(manifest: &Value, diagnostics: &mut Vec<String>) {
    let nonclaims =
        required_string_array(manifest, "/foundation_non_claims", diagnostics).unwrap_or_default();
    if !nonclaims.iter().any(|claim| claim.contains(CHANGESET_ID)) {
        diagnostics.push(format!(
            "manifest /foundation_non_claims must include the {CHANGESET_ID} nonclaim"
        ));
    }
}

fn validate_fixture_header(fixtures: &Value, refs: PathRefs<'_>, diagnostics: &mut Vec<String>) {
    if required_string(fixtures, "/generated_by_changeset", diagnostics).as_deref()
        != Some(CHANGESET_ID)
    {
        diagnostics.push(format!(
            "protocol fixtures /generated_by_changeset must be {CHANGESET_ID:?}"
        ));
    }
    for (pointer, expected) in [
        ("/authority/source_release_index", refs.release_index_rel),
        (
            "/authority/source_archive_manifest",
            refs.archive_manifest_rel,
        ),
        ("/authority/runtime_mode", RUNTIME_MODE),
        ("/authority/service_discovery_path", DISCOVERY_PATH),
        ("/authority/modules_v1_base_path", MODULES_V1_BASE_PATH),
        ("/authority/artifact_base_path", ARTIFACT_BASE_PATH),
    ] {
        if required_string(fixtures, pointer, diagnostics).as_deref() != Some(expected) {
            diagnostics.push(format!("protocol fixtures {pointer} must be {expected:?}"));
        }
    }
    validate_required_source_array(
        fixtures,
        "/authority/official_sources_consulted",
        diagnostics,
    );
    validate_nonclaims(fixtures, "/authority/non_claims", diagnostics);
}

fn validate_required_source_array(value: &Value, pointer: &str, diagnostics: &mut Vec<String>) {
    let sources = required_string_array(value, pointer, diagnostics).unwrap_or_default();
    for required in REQUIRED_OFFICIAL_SOURCES {
        if !sources.iter().any(|source| source == required) {
            diagnostics.push(format!("{pointer} must include {required:?}"));
        }
    }
}

fn validate_nonclaims(value: &Value, pointer: &str, diagnostics: &mut Vec<String>) {
    let claims = required_string_array(value, pointer, diagnostics).unwrap_or_default();
    for required in [
        "no private module registry API",
        "no live service discovery endpoint",
        "no live module download endpoint",
        "no registry publish path",
        "no module signing or Sigstore execution",
        "no SLSA or VSA attestation generation",
        "no tofu test/plan/apply evidence",
        "no cloud resource provisioning",
    ] {
        if !claims.iter().any(|claim| claim.contains(required)) {
            diagnostics.push(format!("{pointer} must include {required:?}"));
        }
    }
}

fn validate_fixture_policy(fixtures: &Value, diagnostics: &mut Vec<String>) {
    if required_string(fixtures, "/policy/status", diagnostics).as_deref() != Some(PROTOCOL_STATUS)
    {
        diagnostics.push(format!(
            "protocol fixtures /policy/status must be {PROTOCOL_STATUS:?}"
        ));
    }
    for pointer in [
        "/policy/service_discovery_fixture_materialized",
        "/policy/versions_response_fixtures_materialized",
        "/policy/download_response_fixtures_materialized",
    ] {
        if required_bool(fixtures, pointer, diagnostics) != Some(true) {
            diagnostics.push(format!("protocol fixtures {pointer} must be true"));
        }
    }
    for pointer in [
        "/policy/private_registry_api_implemented",
        "/policy/service_discovery_endpoint_implemented",
        "/policy/download_endpoint_implemented",
        "/policy/registry_service_runtime_implemented",
        "/policy/registry_publish_path_implemented",
        "/policy/module_signing_executed",
        "/policy/slsa_provenance_generated",
        "/policy/tofu_plan_apply_executed",
        "/policy/provider_runtime_implemented",
        "/policy/cloud_resource_provisioning",
    ] {
        if required_bool(fixtures, pointer, diagnostics) != Some(false) {
            diagnostics.push(format!("protocol fixtures {pointer} must remain false"));
        }
    }
}

fn validate_service_discovery(fixtures: &Value, diagnostics: &mut Vec<String>) {
    for (pointer, expected) in [
        ("/service_discovery/path", DISCOVERY_PATH),
        ("/service_discovery/method", HTTP_METHOD_GET),
        ("/service_discovery/media_type", JSON_MEDIA_TYPE),
        ("/service_discovery/body/modules.v1", MODULES_V1_BASE_PATH),
    ] {
        if required_string(fixtures, pointer, diagnostics).as_deref() != Some(expected) {
            diagnostics.push(format!("protocol fixtures {pointer} must be {expected:?}"));
        }
    }
    if required_u64(fixtures, "/service_discovery/status", diagnostics) != Some(200) {
        diagnostics.push("protocol fixtures /service_discovery/status must be 200".to_string());
    }
}

fn validate_no_secret_markers(value: &Value, diagnostics: &mut Vec<String>) {
    let Ok(serialized) = serde_json::to_string(value) else {
        diagnostics
            .push("protocol fixtures could not be serialized for secret-marker scan".to_string());
        return;
    };
    let lower = serialized.to_ascii_lowercase();
    for marker in [
        "aws_access_key_id",
        "aws_secret_access_key",
        "secret_access_key",
        "client_secret",
        "access_token",
        "vault_token",
        "oci_private_key",
        "private_key_pem",
        "authorization",
        "bearer ",
    ] {
        if lower.contains(marker) {
            diagnostics.push(format!(
                "protocol fixtures must not contain credential-like marker {marker:?}"
            ));
        }
    }
}

fn parse_release_modules(
    release_index: &Value,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, ReleaseModule> {
    let Some(modules) = release_index.pointer("/modules").and_then(Value::as_array) else {
        diagnostics.push("release index /modules must be an array".to_string());
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    for (idx, module) in modules.iter().enumerate() {
        let label = format!("release index /modules/{idx}");
        let identity = parse_identity(module, &label, diagnostics);
        if !identity.key.is_empty() {
            out.insert(
                identity.key.clone(),
                ReleaseModule {
                    identity,
                    versions_endpoint_path: required_url_path_string(
                        module,
                        "/versions_endpoint_path",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    download_endpoint_path: required_url_path_string(
                        module,
                        "/download_endpoint_path",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    archive_manifest_ref: required_repo_relative_string(
                        module,
                        "/archive_manifest_ref",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    archive_file: required_repo_relative_string(
                        module,
                        "/archive_file",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    archive_sha256: required_string(module, "/archive_sha256", diagnostics)
                        .unwrap_or_default(),
                    archive_media_type: required_string(module, "/archive_media_type", diagnostics)
                        .unwrap_or_default(),
                    archive_source_location: optional_string(
                        module,
                        "/archive_source_location",
                        diagnostics,
                    ),
                    archive_source_integrity_sha256: optional_string(
                        module,
                        "/archive_source_integrity_sha256",
                        diagnostics,
                    ),
                    archive_source_version_id: optional_string(
                        module,
                        "/archive_source_version_id",
                        diagnostics,
                    ),
                    archive_source_generation: optional_string(
                        module,
                        "/archive_source_generation",
                        diagnostics,
                    ),
                    module_package_built: required_bool(
                        module,
                        "/module_package_built",
                        diagnostics,
                    )
                    .unwrap_or(false),
                    module_signature_status: required_string(
                        module,
                        "/module_signature_status",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    slsa_provenance_status: required_string(
                        module,
                        "/slsa_provenance_status",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                },
            );
        }
    }
    out
}

fn parse_archive_modules(
    archive_manifest: &Value,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, ArchiveModule> {
    let Some(modules) = archive_manifest
        .pointer("/modules")
        .and_then(Value::as_array)
    else {
        diagnostics.push("archive manifest /modules must be an array".to_string());
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    for (idx, module) in modules.iter().enumerate() {
        let label = format!("archive manifest /modules/{idx}");
        let identity = parse_identity(module, &label, diagnostics);
        if !identity.key.is_empty() {
            out.insert(
                identity.key.clone(),
                ArchiveModule {
                    identity,
                    archive_file: required_repo_relative_string(
                        module,
                        "/archive_file",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    archive_sha256: required_string(module, "/archive_sha256", diagnostics)
                        .unwrap_or_default(),
                    archive_media_type: required_string(module, "/archive_media_type", diagnostics)
                        .unwrap_or_default(),
                    module_signature_status: required_string(
                        module,
                        "/module_signature_status",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    slsa_provenance_status: required_string(
                        module,
                        "/slsa_provenance_status",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                },
            );
        }
    }
    out
}

fn parse_protocol_modules(
    fixtures: &Value,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, ProtocolModule> {
    let Some(modules) = fixtures.pointer("/modules").and_then(Value::as_array) else {
        diagnostics.push("protocol fixtures /modules must be an array".to_string());
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    for (idx, module) in modules.iter().enumerate() {
        let label = format!("protocol fixtures /modules/{idx}");
        let identity = parse_identity(module, &label, diagnostics);
        validate_response_array_shape(module, &label, diagnostics);
        if !identity.key.is_empty() {
            out.insert(
                identity.key.clone(),
                ProtocolModule {
                    identity,
                    versions_method: required_string(
                        module,
                        "/versions_response/method",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    versions_path: required_url_path_string(
                        module,
                        "/versions_response/path",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    versions_status: required_u64(module, "/versions_response/status", diagnostics)
                        .unwrap_or_default(),
                    versions_media_type: required_string(
                        module,
                        "/versions_response/media_type",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    versions_body_version: required_string(
                        module,
                        "/versions_response/body/modules/0/versions/0/version",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    download_method: required_string(
                        module,
                        "/download_response/method",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    download_path: required_url_path_string(
                        module,
                        "/download_response/path",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    download_status: required_u64(module, "/download_response/status", diagnostics)
                        .unwrap_or_default(),
                    download_media_type: required_string(
                        module,
                        "/download_response/media_type",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    download_location: required_module_source_location_string(
                        module,
                        "/download_response/body/location",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    artifact_archive_file: required_repo_relative_string(
                        module,
                        "/artifact/archive_file",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    artifact_archive_sha256: required_string(
                        module,
                        "/artifact/archive_sha256",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    artifact_url_path: required_url_path_string(
                        module,
                        "/artifact/artifact_url_path",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    artifact_media_type: required_string(
                        module,
                        "/artifact/media_type",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    artifact_source_kind: required_string(
                        module,
                        "/artifact/source_kind",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    module_signature_status: required_string(
                        module,
                        "/module_signature_status",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    slsa_provenance_status: required_string(
                        module,
                        "/slsa_provenance_status",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                },
            );
        }
    }
    out
}

fn validate_response_array_shape(module: &Value, label: &str, diagnostics: &mut Vec<String>) {
    let Some(modules) = module
        .pointer("/versions_response/body/modules")
        .and_then(Value::as_array)
    else {
        diagnostics.push(format!(
            "{label} versions_response body modules must be an array"
        ));
        return;
    };
    if modules.len() != 1 {
        diagnostics.push(format!(
            "{label} versions_response body modules must contain exactly one module"
        ));
    }
    let Some(versions) = module
        .pointer("/versions_response/body/modules/0/versions")
        .and_then(Value::as_array)
    else {
        diagnostics.push(format!(
            "{label} versions_response body modules[0].versions must be an array"
        ));
        return;
    };
    if versions.len() != 1 {
        diagnostics.push(format!(
            "{label} versions_response must expose exactly one local foundation version"
        ));
    }
}

fn parse_identity(module: &Value, label: &str, diagnostics: &mut Vec<String>) -> ModuleIdentity {
    let namespace = required_string(module, "/namespace", diagnostics).unwrap_or_default();
    let name = required_string(module, "/name", diagnostics).unwrap_or_default();
    let system = required_string(module, "/system", diagnostics).unwrap_or_default();
    let version = required_string(module, "/version", diagnostics).unwrap_or_default();
    for (field, value) in [
        ("namespace", namespace.as_str()),
        ("name", name.as_str()),
        ("system", system.as_str()),
    ] {
        if !is_registry_slug(value) {
            diagnostics.push(format!("{label} {field} must be a registry slug"));
        }
    }
    if !is_semver_like(&version) {
        diagnostics.push(format!("{label} version must look like x.y.z"));
    }
    let expected_address = if namespace.is_empty() || name.is_empty() || system.is_empty() {
        String::new()
    } else {
        format!("{namespace}/{name}/{system}")
    };
    let address = required_string(module, "/address", diagnostics).unwrap_or_default();
    if address != expected_address {
        diagnostics.push(format!("{label} address must be {expected_address:?}"));
    }
    let release_status =
        required_string(module, "/release_status", diagnostics).unwrap_or_default();
    let evidence_ref = required_string(module, "/evidence_ref", diagnostics).unwrap_or_default();
    let key = if expected_address.is_empty() || version.is_empty() {
        String::new()
    } else {
        format!("{expected_address}/{version}")
    };
    ModuleIdentity {
        key,
        address,
        namespace,
        name,
        system,
        version,
        release_status,
        evidence_ref,
    }
}

fn validate_module_sets(
    release: &BTreeMap<String, ReleaseModule>,
    archive: &BTreeMap<String, ArchiveModule>,
    protocol: &BTreeMap<String, ProtocolModule>,
    refs: PathRefs<'_>,
    diagnostics: &mut Vec<String>,
) {
    let release_keys: BTreeSet<_> = release.keys().map(String::as_str).collect();
    let archive_keys: BTreeSet<_> = archive.keys().map(String::as_str).collect();
    let protocol_keys: BTreeSet<_> = protocol.keys().map(String::as_str).collect();
    if release_keys != archive_keys {
        diagnostics.push(format!(
            "archive manifest module keys must match release index; missing={:?} extra={:?}",
            release_keys.difference(&archive_keys).collect::<Vec<_>>(),
            archive_keys.difference(&release_keys).collect::<Vec<_>>()
        ));
    }
    if release_keys != protocol_keys {
        diagnostics.push(format!(
            "protocol fixture module keys must match release index; missing={:?} extra={:?}",
            release_keys.difference(&protocol_keys).collect::<Vec<_>>(),
            protocol_keys.difference(&release_keys).collect::<Vec<_>>()
        ));
    }
    for (key, release_module) in release {
        let Some(archive_module) = archive.get(key) else {
            continue;
        };
        let Some(protocol_module) = protocol.get(key) else {
            continue;
        };
        if protocol_module.identity != release_module.identity {
            diagnostics.push(format!(
                "protocol module {key} must preserve release-index identity metadata"
            ));
        }
        if archive_module.identity != release_module.identity {
            diagnostics.push(format!(
                "archive module {key} must preserve release-index identity metadata"
            ));
        }
        if release_module.archive_manifest_ref != refs.archive_manifest_rel {
            diagnostics.push(format!(
                "release module {key} archive_manifest_ref must be {:?}",
                refs.archive_manifest_rel
            ));
        }
        if !release_module.module_package_built {
            diagnostics.push(format!(
                "release module {key} module_package_built must be true before protocol fixtures"
            ));
        }
        validate_release_object_source_pin(key, release_module, diagnostics);
        validate_unsigned_status(
            key,
            &release_module.module_signature_status,
            &release_module.slsa_provenance_status,
            "release module",
            diagnostics,
        );
        validate_unsigned_status(
            key,
            &archive_module.module_signature_status,
            &archive_module.slsa_provenance_status,
            "archive module",
            diagnostics,
        );
        validate_unsigned_status(
            key,
            &protocol_module.module_signature_status,
            &protocol_module.slsa_provenance_status,
            "protocol module",
            diagnostics,
        );
        if release_module.archive_file != archive_module.archive_file
            || release_module.archive_sha256 != archive_module.archive_sha256
            || release_module.archive_media_type != archive_module.archive_media_type
        {
            diagnostics.push(format!(
                "release module {key} archive metadata must mirror archive manifest"
            ));
        }
        if protocol_module.artifact_archive_file != archive_module.archive_file
            || protocol_module.artifact_archive_sha256 != archive_module.archive_sha256
        {
            diagnostics.push(format!(
                "protocol module {key} artifact archive_file/archive_sha256 must mirror archive manifest"
            ));
        }
        if protocol_module.artifact_media_type != ARCHIVE_MEDIA_TYPE {
            diagnostics.push(format!(
                "protocol module {key} artifact media_type must be {ARCHIVE_MEDIA_TYPE:?}"
            ));
        }
        let expected_source_kind = if release_module.archive_source_location.is_some() {
            OBJECT_SOURCE_FIXTURE_SOURCE_KIND
        } else {
            HTTP_ARCHIVE_FIXTURE_SOURCE_KIND
        };
        if protocol_module.artifact_source_kind != expected_source_kind {
            diagnostics.push(format!(
                "protocol module {key} artifact source_kind must be {expected_source_kind}"
            ));
        }
        let expected_artifact_path = artifact_url_path(&archive_module.archive_file);
        if protocol_module.artifact_url_path != expected_artifact_path {
            diagnostics.push(format!(
                "protocol module {key} artifact_url_path must be {expected_artifact_path:?}"
            ));
        }
        validate_versions_response(key, release_module, protocol_module, diagnostics);
        validate_download_response(key, release_module, protocol_module, diagnostics);
    }
}

fn validate_unsigned_status(
    key: &str,
    signature_status: &str,
    slsa_status: &str,
    label: &str,
    diagnostics: &mut Vec<String>,
) {
    if signature_status != MODULE_SIGNATURE_STATUS {
        diagnostics.push(format!(
            "{label} {key} module_signature_status must be {MODULE_SIGNATURE_STATUS:?}"
        ));
    }
    if slsa_status != SLSA_STATUS {
        diagnostics.push(format!(
            "{label} {key} slsa_provenance_status must be {SLSA_STATUS:?}"
        ));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ObjectSourceProvider {
    S3,
    Gcs,
}

fn validate_release_object_source_pin(
    key: &str,
    release_module: &ReleaseModule,
    diagnostics: &mut Vec<String>,
) {
    let Some(location) = release_module.archive_source_location.as_deref() else {
        for (field, value) in [
            (
                "archive_source_integrity_sha256",
                release_module.archive_source_integrity_sha256.as_ref(),
            ),
            (
                "archive_source_version_id",
                release_module.archive_source_version_id.as_ref(),
            ),
            (
                "archive_source_generation",
                release_module.archive_source_generation.as_ref(),
            ),
        ] {
            if value.is_some() {
                diagnostics.push(format!(
                    "release module {key} {field} requires archive_source_location"
                ));
            }
        }
        return;
    };

    let provider = object_source_provider(location);
    if provider.is_none() {
        diagnostics.push(format!(
            "release module {key} archive_source_location must use s3::https:// or gcs::https://"
        ));
    }
    if has_unsafe_object_source_text(location) {
        diagnostics.push(format!(
            "release module {key} archive_source_location must not contain whitespace, control characters, userinfo, query strings, fragments, or credential-like markers"
        ));
    }
    let archive_name = Path::new(&release_module.archive_file)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if archive_name.is_empty() || !location.ends_with(archive_name) {
        diagnostics.push(format!(
            "release module {key} archive_source_location must end with archive filename {archive_name:?}"
        ));
    }

    match release_module.archive_source_integrity_sha256.as_deref() {
        Some(integrity)
            if is_sha256_hex(integrity) && integrity == release_module.archive_sha256 => {}
        Some(_) => diagnostics.push(format!(
            "release module {key} archive_source_integrity_sha256 must be lowercase SHA-256 hex matching archive_sha256"
        )),
        None => diagnostics.push(format!(
            "release module {key} archive_source_integrity_sha256 is required when archive_source_location is set"
        )),
    }

    match provider {
        Some(ObjectSourceProvider::S3) => {
            match release_module.archive_source_version_id.as_deref() {
                Some(version_id) if is_safe_pin_token(version_id) => {}
                Some(_) => diagnostics.push(format!(
                    "release module {key} archive_source_version_id must be a safe non-secret S3 version ID"
                )),
                None => diagnostics.push(format!(
                    "release module {key} archive_source_version_id is required for S3 object sources"
                )),
            }
            if release_module.archive_source_generation.is_some() {
                diagnostics.push(format!(
                    "release module {key} archive_source_generation must not be set for S3 object sources"
                ));
            }
        }
        Some(ObjectSourceProvider::Gcs) => {
            match release_module.archive_source_generation.as_deref() {
                Some(generation) if is_nonzero_decimal(generation) => {}
                Some(_) => diagnostics.push(format!(
                    "release module {key} archive_source_generation must be a non-zero decimal GCS generation"
                )),
                None => diagnostics.push(format!(
                    "release module {key} archive_source_generation is required for GCS object sources"
                )),
            }
            if release_module.archive_source_version_id.is_some() {
                diagnostics.push(format!(
                    "release module {key} archive_source_version_id must not be set for GCS object sources"
                ));
            }
        }
        None => {}
    }
}

fn object_source_provider(location: &str) -> Option<ObjectSourceProvider> {
    if location.starts_with("s3::https://") {
        Some(ObjectSourceProvider::S3)
    } else if location.starts_with("gcs::https://") {
        Some(ObjectSourceProvider::Gcs)
    } else {
        None
    }
}

fn has_unsafe_object_source_text(value: &str) -> bool {
    value
        .chars()
        .any(|ch| ch.is_ascii_whitespace() || ch.is_control())
        || value.contains('@')
        || value.contains('?')
        || value.contains('#')
        || contains_secret_like_marker(value)
}

fn is_safe_pin_token(value: &str) -> bool {
    !value.trim().is_empty()
        && !value
            .chars()
            .any(|ch| ch.is_ascii_whitespace() || ch.is_control())
        && !value.contains('\\')
        && !value.contains('"')
        && !value.contains('@')
        && !value.contains('?')
        && !value.contains('#')
        && !contains_secret_like_marker(value)
}

fn is_nonzero_decimal(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_digit()) && value.bytes().any(|byte| byte != b'0')
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn contains_secret_like_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("token=")
        || lower.contains("password=")
        || lower.contains("secret=")
        || lower.contains("aws_access_key_id")
        || lower.contains("aws_secret_access_key")
        || lower.contains("access_token")
        || lower.contains("client_secret")
        || lower.contains("private_key")
}

fn validate_versions_response(
    key: &str,
    release_module: &ReleaseModule,
    protocol_module: &ProtocolModule,
    diagnostics: &mut Vec<String>,
) {
    for (field, found, expected) in [
        (
            "versions method",
            protocol_module.versions_method.as_str(),
            HTTP_METHOD_GET,
        ),
        (
            "versions media_type",
            protocol_module.versions_media_type.as_str(),
            JSON_MEDIA_TYPE,
        ),
    ] {
        if found != expected {
            diagnostics.push(format!(
                "protocol module {key} {field} must be {expected:?}"
            ));
        }
    }
    if protocol_module.versions_status != 200 {
        diagnostics.push(format!("protocol module {key} versions status must be 200"));
    }
    if protocol_module.versions_path != release_module.versions_endpoint_path {
        diagnostics.push(format!(
            "protocol module {key} versions path must mirror release-index versions_endpoint_path"
        ));
    }
    if protocol_module.versions_body_version != release_module.identity.version {
        diagnostics.push(format!(
            "protocol module {key} versions response body must expose release-index version"
        ));
    }
}

fn validate_download_response(
    key: &str,
    release_module: &ReleaseModule,
    protocol_module: &ProtocolModule,
    diagnostics: &mut Vec<String>,
) {
    for (field, found, expected) in [
        (
            "download method",
            protocol_module.download_method.as_str(),
            HTTP_METHOD_GET,
        ),
        (
            "download media_type",
            protocol_module.download_media_type.as_str(),
            JSON_MEDIA_TYPE,
        ),
    ] {
        if found != expected {
            diagnostics.push(format!(
                "protocol module {key} {field} must be {expected:?}"
            ));
        }
    }
    if protocol_module.download_status != 200 {
        diagnostics.push(format!("protocol module {key} download status must be 200"));
    }
    if protocol_module.download_path != release_module.download_endpoint_path {
        diagnostics.push(format!(
            "protocol module {key} download path must mirror release-index download_endpoint_path"
        ));
    }
    match release_module.archive_source_location.as_deref() {
        Some(expected_source_location)
            if protocol_module.download_location == expected_source_location => {}
        Some(_) => diagnostics.push(format!(
            "protocol module {key} download location must mirror release-index archive_source_location"
        )),
        None if protocol_module.download_location == protocol_module.artifact_url_path => {}
        None => diagnostics.push(format!(
            "protocol module {key} download location must equal artifact_url_path"
        )),
    }
}

fn validate_manifest_summary(
    manifest: &Value,
    protocol: &BTreeMap<String, ProtocolModule>,
    diagnostics: &mut Vec<String>,
) {
    let expected_names: Vec<_> = protocol
        .values()
        .map(|module| module.identity.name.clone())
        .collect();
    let found_names = required_string_array(
        manifest,
        "/module_registry_protocol_scope/module_names",
        diagnostics,
    )
    .unwrap_or_default();
    if found_names != expected_names {
        diagnostics.push(format!(
            "manifest /module_registry_protocol_scope/module_names must equal protocol fixture module names; expected={expected_names:?} found={found_names:?}"
        ));
    }
    let module_count = manifest
        .pointer("/module_registry_protocol_scope/module_count")
        .and_then(Value::as_u64);
    if module_count != Some(protocol.len() as u64) {
        diagnostics.push(format!(
            "manifest /module_registry_protocol_scope/module_count must equal {}; found={module_count:?}",
            protocol.len()
        ));
    }
}

fn artifact_url_path(archive_file: &str) -> String {
    let filename = archive_file
        .rsplit('/')
        .next()
        .filter(|part| !part.is_empty())
        .unwrap_or(archive_file);
    format!("{ARTIFACT_BASE_PATH}{filename}")
}

fn is_registry_slug(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_semver_like(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::{
        CloudIacModuleRegistryProtocolArgs, parse_cloud_iac_module_registry_protocol_args,
        validate_cloud_iac_module_registry_protocol_gate,
    };

    #[test]
    fn parse_cloud_iac_module_registry_protocol_rejects_unknown_flag() {
        let error = parse_cloud_iac_module_registry_protocol_args(vec!["--bogus".into()])
            .expect_err("unknown flag should fail");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn cloud_iac_module_registry_protocol_accepts_valid_fixture() {
        let temp = TempRepo::new("cloud-iac-registry-protocol-valid");
        write_fixture(temp.path(), FixtureDrift::None);

        let report = validate_cloud_iac_module_registry_protocol_gate(fixture_args(temp.path()))
            .expect("valid fixture should pass");

        assert_eq!(report.modules_checked, 2);
        assert_eq!(report.versions_responses_checked, 2);
        assert_eq!(report.download_responses_checked, 2);
    }

    #[test]
    fn cloud_iac_module_registry_protocol_rejects_runtime_overclaim() {
        let temp = TempRepo::new("cloud-iac-registry-protocol-runtime");
        write_fixture(temp.path(), FixtureDrift::RuntimeOverclaim);

        let error = validate_cloud_iac_module_registry_protocol_gate(fixture_args(temp.path()))
            .expect_err("runtime overclaim should fail");

        assert!(error.contains("private_registry_api_implemented"));
    }

    #[test]
    fn cloud_iac_module_registry_protocol_rejects_download_location_drift() {
        let temp = TempRepo::new("cloud-iac-registry-protocol-location");
        write_fixture(temp.path(), FixtureDrift::LocationDrift);

        let error = validate_cloud_iac_module_registry_protocol_gate(fixture_args(temp.path()))
            .expect_err("download location drift should fail");

        assert!(error.contains("download location"));
    }

    #[test]
    fn cloud_iac_module_registry_protocol_accepts_object_source_download_location() {
        for (fixture_name, drift) in [
            (
                "cloud-iac-registry-protocol-s3-object-source",
                FixtureDrift::PinnedObjectSource,
            ),
            (
                "cloud-iac-registry-protocol-gcs-object-source",
                FixtureDrift::PinnedGcsObjectSource,
            ),
        ] {
            let temp = TempRepo::new(fixture_name);
            write_fixture(temp.path(), drift);

            let report =
                validate_cloud_iac_module_registry_protocol_gate(fixture_args(temp.path()))
                    .expect("pinned object-source protocol fixture should pass");

            assert_eq!(report.modules_checked, 2);
        }
    }

    #[test]
    fn cloud_iac_module_registry_protocol_rejects_object_source_download_location_drift() {
        let temp = TempRepo::new("cloud-iac-registry-protocol-object-source-drift");
        write_fixture(temp.path(), FixtureDrift::ObjectSourceLocalLocationDrift);

        let error = validate_cloud_iac_module_registry_protocol_gate(fixture_args(temp.path()))
            .expect_err(
                "object-source release rows must not point protocol downloads at local artifacts",
            );

        assert!(error.contains("archive_source_location"));
    }

    #[test]
    fn cloud_iac_module_registry_protocol_rejects_missing_module() {
        let temp = TempRepo::new("cloud-iac-registry-protocol-missing");
        write_fixture(temp.path(), FixtureDrift::MissingModule);

        let error = validate_cloud_iac_module_registry_protocol_gate(fixture_args(temp.path()))
            .expect_err("missing module should fail");

        assert!(error.contains("protocol fixture module keys must match release index"));
    }

    fn fixture_args(repo_root: &Path) -> CloudIacModuleRegistryProtocolArgs {
        CloudIacModuleRegistryProtocolArgs {
            repo_root: repo_root.to_path_buf(),
            manifest: PathBuf::from("iac/manifest.json"),
            release_index: PathBuf::from("iac/tofu/modules/release-index.json"),
            archive_manifest: PathBuf::from(
                "iac/tofu/modules/archive-manifest.json",
            ),
            protocol_fixtures: PathBuf::from(
                "iac/tofu/module-registry/protocol-fixtures.json",
            ),
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum FixtureDrift {
        None,
        RuntimeOverclaim,
        LocationDrift,
        PinnedObjectSource,
        PinnedGcsObjectSource,
        ObjectSourceLocalLocationDrift,
        MissingModule,
    }

    fn write_fixture(root: &Path, drift: FixtureDrift) {
        fs::create_dir_all(root.join("iac/tofu/modules")).expect("modules");
        fs::create_dir_all(root.join("iac/tofu/module-registry"))
            .expect("registry dir");
        fs::write(
            root.join("iac/tofu/modules/release-index.json"),
            fixture_release_index(drift),
        )
        .expect("release index");
        fs::write(
            root.join("iac/tofu/modules/archive-manifest.json"),
            fixture_archive_manifest(),
        )
        .expect("archive manifest");
        fs::write(
            root.join("iac/tofu/module-registry/protocol-fixtures.json"),
            fixture_protocol_fixtures(drift),
        )
        .expect("protocol fixtures");
        fs::create_dir_all(root.join("iac")).expect("manifest dir");
        fs::write(
            root.join("iac/manifest.json"),
            fixture_manifest(),
        )
        .expect("manifest");
    }

    fn fixture_release_index(drift: FixtureDrift) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "modules": [release_row("cloud-account", drift), release_row("dns", drift)]
        }))
        .expect("release index")
    }

    fn fixture_archive_manifest() -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "modules": [archive_row("cloud-account"), archive_row("dns")]
        }))
        .expect("archive manifest")
    }

    fn fixture_protocol_fixtures(drift: FixtureDrift) -> String {
        let mut modules = vec![
            protocol_row("cloud-account", drift),
            protocol_row("dns", drift),
        ];
        if drift == FixtureDrift::MissingModule {
            modules.pop();
        }
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": "1.0",
            "generated_by_changeset": "CS-CLOUD-IAC-MODULE-REGISTRY-PROTOCOL-GATE-001",
            "authority": {
                "source_release_index": "iac/tofu/modules/release-index.json",
                "source_archive_manifest": "iac/tofu/modules/archive-manifest.json",
                "runtime_mode": "local-opentofu-module-registry-protocol-fixture-gate",
                "service_discovery_path": "/.well-known/terraform.json",
                "modules_v1_base_path": "/v1/modules/",
                "artifact_base_path": "/artifacts/modules/",
                "official_sources_consulted": super::REQUIRED_OFFICIAL_SOURCES,
                "non_claims": required_nonclaims()
            },
            "policy": {
                "status": "local-registry-protocol-fixtures-no-service-runtime",
                "service_discovery_fixture_materialized": true,
                "versions_response_fixtures_materialized": true,
                "download_response_fixtures_materialized": true,
                "private_registry_api_implemented": drift == FixtureDrift::RuntimeOverclaim,
                "service_discovery_endpoint_implemented": false,
                "download_endpoint_implemented": false,
                "registry_service_runtime_implemented": false,
                "registry_publish_path_implemented": false,
                "module_signing_executed": false,
                "slsa_provenance_generated": false,
                "tofu_plan_apply_executed": false,
                "provider_runtime_implemented": false,
                "cloud_resource_provisioning": false
            },
            "service_discovery": {
                "method": "GET",
                "path": "/.well-known/terraform.json",
                "status": 200,
                "media_type": "application/json",
                "body": {"modules.v1": "/v1/modules/"}
            },
            "modules": modules
        }))
        .expect("protocol fixtures")
    }

    fn fixture_manifest() -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "capabilities": [{
                "tier": "T1",
                "name": "cloud-iac-module-registry-protocol-gate",
                "file": "crates/oya-dev-cli/src/cloud_iac_module_registry_protocol_gate.rs",
                "risk_class": "high"
            }],
            "foundation_non_claims": ["CS-CLOUD-IAC-MODULE-REGISTRY-PROTOCOL-GATE-001 materializes local protocol fixtures only; no private registry API, live service discovery endpoint, live download endpoint, registry publish path, signing, SLSA/VSA, plan/apply, provider runtime, or cloud provisioning is claimed."],
            "module_registry_protocol_scope": {
                "release_index": "iac/tofu/modules/release-index.json",
                "archive_manifest": "iac/tofu/modules/archive-manifest.json",
                "protocol_fixtures": "iac/tofu/module-registry/protocol-fixtures.json",
                "status": "local-registry-protocol-fixtures-no-service-runtime",
                "runtime_mode": "local-opentofu-module-registry-protocol-fixture-gate",
                "service_discovery_path": "/.well-known/terraform.json",
                "modules_v1_base_path": "/v1/modules/",
                "artifact_base_path": "/artifacts/modules/",
                "module_count": 2,
                "module_names": ["cloud-account", "dns"],
                "official_sources_consulted": super::REQUIRED_OFFICIAL_SOURCES,
                "coherence_guard": {
                    "changeset": "CS-CLOUD-IAC-MODULE-REGISTRY-PROTOCOL-GATE-001",
                    "gate": "cloud-iac-module-registry-protocol",
                    "gate_file": "crates/oya-dev-cli/src/cloud_iac_module_registry_protocol_gate.rs",
                    "runtime_mode": "local-opentofu-module-registry-protocol-fixture-gate"
                },
                "non_claims": required_nonclaims()
            }
        }))
        .expect("manifest")
    }

    fn release_row(name: &str, drift: FixtureDrift) -> serde_json::Value {
        let mut row = base_row(name);
        row["versions_endpoint_path"] =
            serde_json::json!(format!("/v1/modules/oyatie/{name}/opentofu/versions"));
        row["download_endpoint_path"] =
            serde_json::json!(format!("/v1/modules/oyatie/{name}/opentofu/0.1.0/download"));
        row["archive_manifest_ref"] =
            serde_json::json!("iac/tofu/modules/archive-manifest.json");
        row["archive_file"] = serde_json::json!(archive_file(name));
        row["archive_sha256"] = serde_json::json!(archive_sha(name));
        row["archive_media_type"] = serde_json::json!("archive/zip");
        row["module_package_built"] = serde_json::json!(true);
        row["module_signature_status"] = serde_json::json!("unsigned-no-cosign");
        row["slsa_provenance_status"] = serde_json::json!("not-generated");
        if matches!(
            drift,
            FixtureDrift::PinnedObjectSource
                | FixtureDrift::PinnedGcsObjectSource
                | FixtureDrift::ObjectSourceLocalLocationDrift
        ) && name == "dns"
        {
            row["archive_source_location"] = serde_json::json!(object_source_location(name, drift));
            row["archive_source_integrity_sha256"] = serde_json::json!(archive_sha(name));
            if drift == FixtureDrift::PinnedGcsObjectSource {
                row["archive_source_generation"] = serde_json::json!("1700000000000001");
            } else {
                row["archive_source_version_id"] = serde_json::json!("s3v-local-foundation-0001");
            }
        }
        row
    }

    fn archive_row(name: &str) -> serde_json::Value {
        let mut row = base_row(name);
        row["archive_file"] = serde_json::json!(archive_file(name));
        row["archive_sha256"] = serde_json::json!(archive_sha(name));
        row["archive_media_type"] = serde_json::json!("archive/zip");
        row["module_signature_status"] = serde_json::json!("unsigned-no-cosign");
        row["slsa_provenance_status"] = serde_json::json!("not-generated");
        row
    }

    fn protocol_row(name: &str, drift: FixtureDrift) -> serde_json::Value {
        let mut row = base_row(name);
        let artifact_url = artifact_url(name);
        row["versions_response"] = serde_json::json!({
            "method": "GET",
            "path": format!("/v1/modules/oyatie/{name}/opentofu/versions"),
            "status": 200,
            "media_type": "application/json",
            "body": {"modules": [{"versions": [{"version": "0.1.0"}]}]}
        });
        let download_location = if matches!(
            drift,
            FixtureDrift::PinnedObjectSource | FixtureDrift::PinnedGcsObjectSource
        ) && name == "dns"
        {
            object_source_location(name, drift)
        } else if drift == FixtureDrift::LocationDrift && name == "dns" {
            "/artifacts/modules/drift.zip".to_string()
        } else {
            artifact_url.clone()
        };
        row["download_response"] = serde_json::json!({
            "method": "GET",
            "path": format!("/v1/modules/oyatie/{name}/opentofu/0.1.0/download"),
            "status": 200,
            "media_type": "application/json",
            "body": {"location": download_location}
        });
        row["artifact"] = serde_json::json!({
            "archive_file": archive_file(name),
            "archive_sha256": archive_sha(name),
            "artifact_url_path": artifact_url,
            "media_type": "archive/zip",
            "source_kind": if matches!(drift, FixtureDrift::PinnedObjectSource | FixtureDrift::PinnedGcsObjectSource) && name == "dns" {
                "object-source-fixture-no-live-endpoint"
            } else {
                "http-archive-fixture-no-live-endpoint"
            }
        });
        row["module_signature_status"] = serde_json::json!("unsigned-no-cosign");
        row["slsa_provenance_status"] = serde_json::json!("not-generated");
        row
    }

    fn base_row(name: &str) -> serde_json::Value {
        serde_json::json!({
            "address": format!("oyatie/{name}/opentofu"),
            "namespace": "oyatie",
            "name": name,
            "system": "opentofu",
            "version": "0.1.0",
            "release_status": "local-foundation-skeleton",
            "evidence_ref": format!("evidence://cloud-iac/modules/{name}/0.1.0/local-foundation")
        })
    }

    fn archive_file(name: &str) -> String {
        format!("target/oya-cloud-iac/module-archives/oyatie-{name}-opentofu-0.1.0.zip")
    }

    fn archive_sha(name: &str) -> String {
        match name {
            "cloud-account" => "1".repeat(64),
            "dns" => "2".repeat(64),
            _ => "3".repeat(64),
        }
    }

    fn artifact_url(name: &str) -> String {
        format!("/artifacts/modules/oyatie-{name}-opentofu-0.1.0.zip")
    }

    fn object_source_location(name: &str, drift: FixtureDrift) -> String {
        match drift {
            FixtureDrift::PinnedGcsObjectSource => format!(
                "gcs::https://www.googleapis.com/storage/v1/oyatie-cloud-iac-modules/oyatie/{name}/0.1.0/oyatie-{name}-opentofu-0.1.0.zip"
            ),
            _ => format!(
                "s3::https://s3.amazonaws.com/oyatie-cloud-iac-modules/oyatie/{name}/0.1.0/oyatie-{name}-opentofu-0.1.0.zip"
            ),
        }
    }

    fn required_nonclaims() -> Vec<&'static str> {
        vec![
            "no private module registry API",
            "no live service discovery endpoint",
            "no live module download endpoint",
            "no registry publish path",
            "no module signing or Sigstore execution",
            "no SLSA or VSA attestation generation",
            "no tofu test/plan/apply evidence",
            "no cloud resource provisioning",
        ]
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
