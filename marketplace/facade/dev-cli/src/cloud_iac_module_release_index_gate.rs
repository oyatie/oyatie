//! `oya gate validate cloud-iac-module-release-index` runner.
//!
//! This gate verifies a local OpenTofu module-registry-shaped release index for
//! the Cloud IaC foundation modules. It intentionally proves only repo-local
//! metadata/index coherence: no private module registry API, service discovery,
//! download endpoint, cosign/Sigstore signature, SLSA provenance, provider
//! runtime, tofu plan/apply, or cloud provisioning is implemented by this gate.

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
const DEFAULT_RELEASE_INDEX: &str = "iac/tofu/modules/release-index.json";
const DEFAULT_ARCHIVE_MANIFEST: &str = "iac/tofu/modules/archive-manifest.json";
const DEFAULT_PROVIDER_LOCK_ROOT: &str = "iac/tofu/provider-locks/foundation";
const DEFAULT_PROVIDER_SIGNATURE_REVIEW: &str =
    "iac/tofu/provider-locks/foundation/provider-signature-review.json";
const GATE_NAME: &str = "cloud-iac-module-release-index";
const GATE_FILE: &str = "crates/oya-dev-cli/src/cloud_iac_module_release_index_gate.rs";
const CHANGESET_ID: &str = "CS-CLOUD-IAC-MODULE-RELEASE-INDEX-GATE-001";
const RUNTIME_MODE: &str = "local-opentofu-module-release-index-gate";
const INDEX_STATUS: &str = "local-release-index-no-private-registry-api";
const RELEASE_STATUS: &str = "local-foundation-skeleton";
const REGISTRY_PUBLISH_STATUS: &str = "local-index-only-no-service";
const MODULE_SIGNATURE_STATUS: &str = "unsigned-no-cosign";
const SLSA_STATUS: &str = "not-generated";
const ARCHIVE_FORMAT: &str = "zip";
const ARCHIVE_MEDIA_TYPE: &str = "archive/zip";
const ARCHIVE_COMPRESSION_METHOD: &str = "store";
const ARCHIVE_DETERMINISTIC_TIMESTAMP: &str = "1980-01-01T00:00:00Z";
const REQUIRED_OFFICIAL_SOURCES: &[&str] = &[
    "https://opentofu.org/docs/internals/module-registry-protocol/",
    "https://opentofu.org/docs/language/modules/sources/",
    "https://opentofu.org/docs/cli/oci_registries/module-package/",
    "https://opentofu.org/docs/language/modules/develop/providers/",
    "https://slsa.dev/spec/v1.2/",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacModuleReleaseIndexArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) catalog: PathBuf,
    pub(crate) provenance: PathBuf,
    pub(crate) release_index: PathBuf,
    pub(crate) archive_manifest: PathBuf,
    pub(crate) provider_lock_root: PathBuf,
    pub(crate) provider_signature_review: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacModuleReleaseIndexReport {
    pub(crate) manifest_path: String,
    pub(crate) catalog_path: String,
    pub(crate) provenance_path: String,
    pub(crate) release_index_path: String,
    pub(crate) archive_manifest_path: String,
    pub(crate) modules_checked: usize,
    pub(crate) files_checked: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct CatalogModule {
    key: String,
    namespace: String,
    name: String,
    system: String,
    version: String,
    source_path: String,
    main_file: String,
    release_status: String,
    evidence_ref: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ProvenanceModule {
    catalog: CatalogModule,
    files: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ReleaseModule {
    catalog: CatalogModule,
    address: String,
    versions_endpoint_path: String,
    download_endpoint_path: String,
    registry_publish_status: String,
    module_source_kind: String,
    module_package_built: bool,
    archive_manifest_ref: String,
    archive_file: String,
    archive_sha256: String,
    archive_format: String,
    archive_media_type: String,
    archive_compression_method: String,
    archive_deterministic_timestamp: String,
    archive_source_location: Option<String>,
    archive_source_integrity_sha256: Option<String>,
    archive_source_version_id: Option<String>,
    archive_source_generation: Option<String>,
    module_signature_status: String,
    slsa_provenance_status: String,
    provider_lock_scope: String,
    provider_signature_review_ref: String,
    files: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ArchiveModule {
    catalog: CatalogModule,
    archive_file: String,
    archive_sha256: String,
    archive_format: String,
    archive_media_type: String,
    compression_method: String,
    deterministic_timestamp: String,
    module_signature_status: String,
    slsa_provenance_status: String,
    files: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReleaseIndexPathRefs<'a> {
    catalog_rel: &'a str,
    provenance_rel: &'a str,
    release_index_rel: &'a str,
    archive_manifest_rel: &'a str,
    provider_lock_root_rel: &'a str,
    provider_signature_review_rel: &'a str,
}

pub(crate) fn parse_cloud_iac_module_release_index_args(
    args: Vec<String>,
) -> Result<CloudIacModuleReleaseIndexArgs, String> {
    let mut parsed = CloudIacModuleReleaseIndexArgs {
        repo_root: PathBuf::from(DEFAULT_REPO_ROOT),
        manifest: PathBuf::from(DEFAULT_MANIFEST),
        catalog: PathBuf::from(DEFAULT_CATALOG),
        provenance: PathBuf::from(DEFAULT_PROVENANCE),
        release_index: PathBuf::from(DEFAULT_RELEASE_INDEX),
        archive_manifest: PathBuf::from(DEFAULT_ARCHIVE_MANIFEST),
        provider_lock_root: PathBuf::from(DEFAULT_PROVIDER_LOCK_ROOT),
        provider_signature_review: PathBuf::from(DEFAULT_PROVIDER_SIGNATURE_REVIEW),
    };

    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--repo-root" => parsed.repo_root = take_path_arg(&mut args, "--repo-root")?,
            "--manifest" => parsed.manifest = take_path_arg(&mut args, "--manifest")?,
            "--catalog" => parsed.catalog = take_path_arg(&mut args, "--catalog")?,
            "--provenance" => parsed.provenance = take_path_arg(&mut args, "--provenance")?,
            "--release-index" => {
                parsed.release_index = take_path_arg(&mut args, "--release-index")?;
            }
            "--archive-manifest" => {
                parsed.archive_manifest = take_path_arg(&mut args, "--archive-manifest")?;
            }
            "--provider-lock-root" => {
                parsed.provider_lock_root = take_path_arg(&mut args, "--provider-lock-root")?;
            }
            "--provider-signature-review" => {
                parsed.provider_signature_review =
                    take_path_arg(&mut args, "--provider-signature-review")?;
            }
            other => {
                return Err(format!(
                    "cloud-iac-module-release-index: unknown flag {other:?}; usage: \
                     oya gate validate cloud-iac-module-release-index \
                     [--repo-root <.>] \
                     [--manifest <iac/manifest.json>] \
                     [--catalog <iac/tofu/modules/catalog.json>] \
                     [--provenance <iac/tofu/modules/provenance.json>] \
                     [--release-index <iac/tofu/modules/release-index.json>] \
                     [--archive-manifest <iac/tofu/modules/archive-manifest.json>] \
                     [--provider-lock-root <iac/tofu/provider-locks/foundation>] \
                     [--provider-signature-review <iac/tofu/provider-locks/foundation/provider-signature-review.json>]"
                ));
            }
        }
    }
    Ok(parsed)
}

fn take_path_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("cloud-iac-module-release-index: {flag} requires a path argument"))
}

pub(crate) fn validate_cloud_iac_module_release_index_gate(
    args: CloudIacModuleReleaseIndexArgs,
) -> Result<CloudIacModuleReleaseIndexReport, String> {
    let manifest_path = resolve_repo_path(&args.repo_root, &args.manifest);
    let catalog_path = resolve_repo_path(&args.repo_root, &args.catalog);
    let provenance_path = resolve_repo_path(&args.repo_root, &args.provenance);
    let release_index_path = resolve_repo_path(&args.repo_root, &args.release_index);
    let archive_manifest_path = resolve_repo_path(&args.repo_root, &args.archive_manifest);
    let provider_lock_root_path = resolve_repo_path(&args.repo_root, &args.provider_lock_root);
    let provider_signature_review_path =
        resolve_repo_path(&args.repo_root, &args.provider_signature_review);

    let manifest_rel = repo_relative_argument(&args.repo_root, &args.manifest)?;
    let catalog_rel = repo_relative_argument(&args.repo_root, &args.catalog)?;
    let provenance_rel = repo_relative_argument(&args.repo_root, &args.provenance)?;
    let release_index_rel = repo_relative_argument(&args.repo_root, &args.release_index)?;
    let archive_manifest_rel = repo_relative_argument(&args.repo_root, &args.archive_manifest)?;
    let provider_lock_root_rel = repo_relative_argument(&args.repo_root, &args.provider_lock_root)?;
    let provider_signature_review_rel =
        repo_relative_argument(&args.repo_root, &args.provider_signature_review)?;

    let manifest = read_json(&manifest_path, "manifest")?;
    let catalog = read_json(&catalog_path, "module catalog")?;
    let provenance = read_json(&provenance_path, "module provenance")?;
    let release_index = read_json(&release_index_path, "module release index")?;
    let archive_manifest = read_json(&archive_manifest_path, "module archive manifest")?;

    let mut diagnostics = Vec::new();
    let refs = ReleaseIndexPathRefs {
        catalog_rel: &catalog_rel,
        provenance_rel: &provenance_rel,
        release_index_rel: &release_index_rel,
        archive_manifest_rel: &archive_manifest_rel,
        provider_lock_root_rel: &provider_lock_root_rel,
        provider_signature_review_rel: &provider_signature_review_rel,
    };
    validate_manifest_scope(&manifest, refs, &mut diagnostics);
    validate_index_header(
        &release_index,
        refs,
        &provider_signature_review_path,
        &mut diagnostics,
    );
    validate_index_policy(&release_index, &mut diagnostics);
    validate_no_secret_markers(&release_index, &mut diagnostics);

    let catalog_modules = parse_catalog_modules(&catalog, &mut diagnostics);
    let provenance_modules = parse_provenance_modules(&provenance, &mut diagnostics);
    let release_modules = parse_release_modules(&release_index, &mut diagnostics);
    let archive_modules = parse_archive_modules(&archive_manifest, &mut diagnostics);

    validate_module_sets(
        &catalog_modules,
        &provenance_modules,
        &release_modules,
        &archive_modules,
        &archive_manifest_rel,
        &provider_signature_review_rel,
        &mut diagnostics,
    );
    validate_manifest_summary(&manifest, &release_modules, &mut diagnostics);
    let files_checked = validate_file_digests(
        &args.repo_root,
        &provenance_modules,
        &release_modules,
        &mut diagnostics,
    );
    validate_provider_evidence_paths(
        &provider_lock_root_path,
        &provider_signature_review_path,
        &mut diagnostics,
    );

    if diagnostics.is_empty() {
        Ok(CloudIacModuleReleaseIndexReport {
            manifest_path: manifest_rel,
            catalog_path: catalog_rel,
            provenance_path: provenance_rel,
            release_index_path: release_index_rel,
            archive_manifest_path: archive_manifest_rel,
            modules_checked: release_modules.len(),
            files_checked,
        })
    } else {
        Err(format!(
            "cloud-iac-module-release-index validation failed:\n- {}",
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
                "cloud-iac-module-release-index: unable to canonicalize repo root {}: {error}",
                repo_root.display()
            )
        })?;
        let path = fs::canonicalize(path).map_err(|error| {
            format!(
                "cloud-iac-module-release-index: unable to canonicalize path {}: {error}",
                path.display()
            )
        })?;
        let relative = path.strip_prefix(&repo_root).map_err(|_| {
            format!(
                "cloud-iac-module-release-index: path {} is outside repo root {}",
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
            "cloud-iac-module-release-index: unable to read {label} {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "cloud-iac-module-release-index: unable to parse {label} JSON {}: {error}",
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

fn validate_manifest_scope(
    manifest: &Value,
    refs: ReleaseIndexPathRefs<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (pointer, expected) in [
        ("/module_release_index_scope/catalog", refs.catalog_rel),
        (
            "/module_release_index_scope/provenance",
            refs.provenance_rel,
        ),
        (
            "/module_release_index_scope/release_index",
            refs.release_index_rel,
        ),
        (
            "/module_release_index_scope/archive_manifest",
            refs.archive_manifest_rel,
        ),
        (
            "/module_release_index_scope/provider_lock_root",
            refs.provider_lock_root_rel,
        ),
        (
            "/module_release_index_scope/provider_signature_review",
            refs.provider_signature_review_rel,
        ),
    ] {
        if required_repo_relative_string(manifest, pointer, diagnostics).as_deref()
            != Some(expected)
        {
            diagnostics.push(format!("manifest {pointer} must equal {expected:?}"));
        }
    }
    for (pointer, expected) in [
        ("/module_release_index_scope/status", INDEX_STATUS),
        ("/module_release_index_scope/runtime_mode", RUNTIME_MODE),
        (
            "/module_release_index_scope/coherence_guard/changeset",
            CHANGESET_ID,
        ),
        (
            "/module_release_index_scope/coherence_guard/gate",
            GATE_NAME,
        ),
        (
            "/module_release_index_scope/coherence_guard/gate_file",
            GATE_FILE,
        ),
        (
            "/module_release_index_scope/coherence_guard/runtime_mode",
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
        "/module_release_index_scope/official_sources_consulted",
        diagnostics,
    );
    validate_nonclaims(
        manifest,
        "/module_release_index_scope/non_claims",
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
            == Some("cloud-iac-module-release-index-gate")
    }) else {
        diagnostics.push(
            "manifest /capabilities must include cloud-iac-module-release-index-gate".to_string(),
        );
        return;
    };
    if capability.pointer("/file").and_then(Value::as_str) != Some(GATE_FILE) {
        diagnostics.push(format!(
            "manifest cloud-iac-module-release-index-gate /file must be {GATE_FILE:?}"
        ));
    }
    if capability.pointer("/tier").and_then(Value::as_str) != Some("T1") {
        diagnostics
            .push("manifest module-release-index capability /tier must be \"T1\"".to_string());
    }
    if capability.pointer("/risk_class").and_then(Value::as_str) != Some("high") {
        diagnostics.push(
            "manifest module-release-index capability /risk_class must be \"high\"".to_string(),
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
        "no module service discovery",
        "no live module download endpoint",
        "no module signing or Sigstore execution",
        "no SLSA attestation generation",
        "no tofu test/plan/apply evidence",
        "no cloud resource provisioning",
    ] {
        if !claims.iter().any(|claim| claim.contains(required)) {
            diagnostics.push(format!("{pointer} must include {required:?}"));
        }
    }
}

fn validate_index_header(
    index: &Value,
    refs: ReleaseIndexPathRefs<'_>,
    provider_signature_review_path: &Path,
    diagnostics: &mut Vec<String>,
) {
    if required_string(index, "/generated_by_changeset", diagnostics).as_deref()
        != Some(CHANGESET_ID)
    {
        diagnostics.push(format!(
            "release index /generated_by_changeset must be {CHANGESET_ID:?}"
        ));
    }
    for (pointer, expected) in [
        ("/authority/source_catalog", refs.catalog_rel),
        ("/authority/source_provenance", refs.provenance_rel),
        (
            "/authority/source_archive_manifest",
            refs.archive_manifest_rel,
        ),
        (
            "/authority/source_provider_lock_root",
            refs.provider_lock_root_rel,
        ),
        (
            "/authority/source_provider_signature_review",
            refs.provider_signature_review_rel,
        ),
        ("/provider_evidence/lock_root", refs.provider_lock_root_rel),
        (
            "/provider_evidence/signature_review",
            refs.provider_signature_review_rel,
        ),
    ] {
        if required_repo_relative_string(index, pointer, diagnostics).as_deref() != Some(expected) {
            diagnostics.push(format!("release index {pointer} must equal {expected:?}"));
        }
    }
    if required_string(index, "/authority/runtime_mode", diagnostics).as_deref()
        != Some(RUNTIME_MODE)
    {
        diagnostics.push(format!(
            "release index /authority/runtime_mode must be {RUNTIME_MODE:?}"
        ));
    }
    validate_required_source_array(index, "/authority/official_sources_consulted", diagnostics);
    validate_nonclaims(index, "/authority/non_claims", diagnostics);
    if sha256_file(provider_signature_review_path, diagnostics).as_deref()
        != required_string(
            index,
            "/provider_evidence/signature_review_sha256",
            diagnostics,
        )
        .as_deref()
    {
        diagnostics.push(
            "release index /provider_evidence/signature_review_sha256 must match provider-signature-review.json bytes"
                .to_string(),
        );
    }
}

fn validate_index_policy(index: &Value, diagnostics: &mut Vec<String>) {
    if required_string(index, "/policy/status", diagnostics).as_deref() != Some(INDEX_STATUS) {
        diagnostics.push(format!(
            "release index /policy/status must be {INDEX_STATUS:?}"
        ));
    }
    if required_string(index, "/policy/required_release_status", diagnostics).as_deref()
        != Some(RELEASE_STATUS)
    {
        diagnostics.push(format!(
            "release index /policy/required_release_status must be {RELEASE_STATUS:?}"
        ));
    }
    if required_bool(index, "/policy/module_archives_built", diagnostics) != Some(true) {
        diagnostics.push("release index /policy/module_archives_built must be true".to_string());
    }
    for pointer in [
        "/policy/private_registry_api_implemented",
        "/policy/service_discovery_implemented",
        "/policy/download_endpoint_implemented",
        "/policy/module_signing_executed",
        "/policy/slsa_provenance_generated",
        "/policy/tofu_plan_apply_executed",
        "/policy/provider_resource_complete_modules",
    ] {
        if required_bool(index, pointer, diagnostics) != Some(false) {
            diagnostics.push(format!("release index {pointer} must remain false"));
        }
    }
}

fn validate_no_secret_markers(index: &Value, diagnostics: &mut Vec<String>) {
    let Ok(serialized) = serde_json::to_string(index) else {
        diagnostics
            .push("release index could not be serialized for secret-marker scan".to_string());
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
    ] {
        if lower.contains(marker) {
            diagnostics.push(format!(
                "release index must not contain credential-like marker {marker:?}"
            ));
        }
    }
}

fn parse_catalog_modules(
    catalog: &Value,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, CatalogModule> {
    let Some(modules) = catalog.pointer("/modules").and_then(Value::as_array) else {
        diagnostics.push("catalog /modules must be an array".to_string());
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    for (idx, module) in modules.iter().enumerate() {
        let parsed =
            parse_catalog_like_module(module, &format!("catalog /modules/{idx}"), diagnostics);
        if !parsed.key.is_empty() && out.insert(parsed.key.clone(), parsed).is_some() {
            diagnostics.push(format!("duplicate catalog module key at /modules/{idx}"));
        }
    }
    out
}

fn parse_provenance_modules(
    provenance: &Value,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, ProvenanceModule> {
    let Some(modules) = provenance.pointer("/modules").and_then(Value::as_array) else {
        diagnostics.push("provenance /modules must be an array".to_string());
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    for (idx, module) in modules.iter().enumerate() {
        let catalog =
            parse_catalog_like_module(module, &format!("provenance /modules/{idx}"), diagnostics);
        let files = parse_files(
            module,
            &format!("provenance /modules/{idx}/files"),
            diagnostics,
        );
        if !catalog.key.is_empty() {
            out.insert(catalog.key.clone(), ProvenanceModule { catalog, files });
        }
    }
    out
}

fn parse_release_modules(
    index: &Value,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, ReleaseModule> {
    let Some(modules) = index.pointer("/modules").and_then(Value::as_array) else {
        diagnostics.push("release index /modules must be an array".to_string());
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    for (idx, module) in modules.iter().enumerate() {
        let label = format!("release index /modules/{idx}");
        let catalog = parse_catalog_like_module(module, &label, diagnostics);
        let files = parse_files(module, &format!("{label}/files"), diagnostics);
        if !catalog.key.is_empty() {
            out.insert(
                catalog.key.clone(),
                ReleaseModule {
                    catalog,
                    address: required_string(module, "/address", diagnostics).unwrap_or_default(),
                    versions_endpoint_path: required_string(
                        module,
                        "/versions_endpoint_path",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    download_endpoint_path: required_string(
                        module,
                        "/download_endpoint_path",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    registry_publish_status: required_string(
                        module,
                        "/registry_publish_status",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    module_source_kind: required_string(module, "/module_source_kind", diagnostics)
                        .unwrap_or_default(),
                    module_package_built: required_bool(
                        module,
                        "/module_package_built",
                        diagnostics,
                    )
                    .unwrap_or(false),
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
                    archive_format: required_string(module, "/archive_format", diagnostics)
                        .unwrap_or_default(),
                    archive_media_type: required_string(module, "/archive_media_type", diagnostics)
                        .unwrap_or_default(),
                    archive_compression_method: required_string(
                        module,
                        "/archive_compression_method",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    archive_deterministic_timestamp: required_string(
                        module,
                        "/archive_deterministic_timestamp",
                        diagnostics,
                    )
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
                    provider_lock_scope: required_string(
                        module,
                        "/provider_lock_scope",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    provider_signature_review_ref: required_string(
                        module,
                        "/provider_signature_review_ref",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    files,
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
        let catalog = parse_catalog_like_module(module, &label, diagnostics);
        let files = parse_files(module, &format!("{label}/files"), diagnostics);
        if !catalog.key.is_empty() {
            out.insert(
                catalog.key.clone(),
                ArchiveModule {
                    catalog,
                    archive_file: required_repo_relative_string(
                        module,
                        "/archive_file",
                        diagnostics,
                    )
                    .unwrap_or_default(),
                    archive_sha256: required_string(module, "/archive_sha256", diagnostics)
                        .unwrap_or_default(),
                    archive_format: required_string(module, "/archive_format", diagnostics)
                        .unwrap_or_default(),
                    archive_media_type: required_string(module, "/archive_media_type", diagnostics)
                        .unwrap_or_default(),
                    compression_method: required_string(module, "/compression_method", diagnostics)
                        .unwrap_or_default(),
                    deterministic_timestamp: required_string(
                        module,
                        "/deterministic_timestamp",
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
                    files,
                },
            );
        }
    }
    out
}

fn parse_catalog_like_module(
    module: &Value,
    label: &str,
    diagnostics: &mut Vec<String>,
) -> CatalogModule {
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
    let source_path =
        required_repo_relative_string(module, "/source_path", diagnostics).unwrap_or_default();
    let main_file =
        required_repo_relative_string(module, "/main_file", diagnostics).unwrap_or_default();
    let release_status =
        required_string(module, "/release_status", diagnostics).unwrap_or_default();
    let evidence_ref = required_string(module, "/evidence_ref", diagnostics).unwrap_or_default();
    let key = if namespace.is_empty() || name.is_empty() || system.is_empty() || version.is_empty()
    {
        String::new()
    } else {
        format!("{namespace}/{name}/{system}/{version}")
    };
    CatalogModule {
        key,
        namespace,
        name,
        system,
        version,
        source_path,
        main_file,
        release_status,
        evidence_ref,
    }
}

fn parse_files(
    module: &Value,
    label: &str,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, String> {
    let Some(files) = module.pointer("/files").and_then(Value::as_array) else {
        diagnostics.push(format!("{label} must be an array"));
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    for (idx, file) in files.iter().enumerate() {
        let path = required_repo_relative_string(file, "/path", diagnostics).unwrap_or_default();
        let sha256 = required_string(file, "/sha256", diagnostics).unwrap_or_default();
        if !is_sha256_hex(&sha256) {
            diagnostics.push(format!(
                "{label}/{idx}/sha256 must be lowercase SHA-256 hex"
            ));
        }
        if !path.is_empty() {
            out.insert(path, sha256);
        }
    }
    out
}

fn validate_module_sets(
    catalog: &BTreeMap<String, CatalogModule>,
    provenance: &BTreeMap<String, ProvenanceModule>,
    release: &BTreeMap<String, ReleaseModule>,
    archive: &BTreeMap<String, ArchiveModule>,
    archive_manifest_rel: &str,
    provider_signature_review_rel: &str,
    diagnostics: &mut Vec<String>,
) {
    let catalog_keys: BTreeSet<_> = catalog.keys().map(String::as_str).collect();
    let provenance_keys: BTreeSet<_> = provenance.keys().map(String::as_str).collect();
    let release_keys: BTreeSet<_> = release.keys().map(String::as_str).collect();
    let archive_keys: BTreeSet<_> = archive.keys().map(String::as_str).collect();
    if catalog_keys != provenance_keys {
        diagnostics.push(format!(
            "provenance module keys must match catalog; missing={:?} extra={:?}",
            catalog_keys
                .difference(&provenance_keys)
                .collect::<Vec<_>>(),
            provenance_keys
                .difference(&catalog_keys)
                .collect::<Vec<_>>()
        ));
    }
    if catalog_keys != release_keys {
        diagnostics.push(format!(
            "release index module keys must match catalog; missing={:?} extra={:?}",
            catalog_keys.difference(&release_keys).collect::<Vec<_>>(),
            release_keys.difference(&catalog_keys).collect::<Vec<_>>()
        ));
    }
    if catalog_keys != archive_keys {
        diagnostics.push(format!(
            "archive manifest module keys must match catalog; missing={:?} extra={:?}",
            catalog_keys.difference(&archive_keys).collect::<Vec<_>>(),
            archive_keys.difference(&catalog_keys).collect::<Vec<_>>()
        ));
    }
    for (key, catalog_module) in catalog {
        let Some(provenance_module) = provenance.get(key) else {
            continue;
        };
        let Some(release_module) = release.get(key) else {
            continue;
        };
        let Some(archive_module) = archive.get(key) else {
            continue;
        };
        if &provenance_module.catalog != catalog_module {
            diagnostics.push(format!(
                "provenance module {key} must preserve catalog metadata"
            ));
        }
        if &release_module.catalog != catalog_module {
            diagnostics.push(format!(
                "release index module {key} must preserve catalog metadata"
            ));
        }
        if &archive_module.catalog != catalog_module {
            diagnostics.push(format!(
                "archive manifest module {key} must preserve catalog metadata"
            ));
        }
        let expected_address = format!(
            "{}/{}/{}",
            catalog_module.namespace, catalog_module.name, catalog_module.system
        );
        if release_module.address != expected_address {
            diagnostics.push(format!(
                "release module {key} address must be {expected_address:?}"
            ));
        }
        let expected_versions_endpoint = format!("/v1/modules/{expected_address}/versions");
        let expected_download_endpoint = format!(
            "/v1/modules/{expected_address}/{}/download",
            catalog_module.version
        );
        if release_module.versions_endpoint_path != expected_versions_endpoint {
            diagnostics.push(format!(
                "release module {key} versions_endpoint_path must be {expected_versions_endpoint:?}"
            ));
        }
        if release_module.download_endpoint_path != expected_download_endpoint {
            diagnostics.push(format!(
                "release module {key} download_endpoint_path must be {expected_download_endpoint:?}"
            ));
        }
        if release_module.registry_publish_status != REGISTRY_PUBLISH_STATUS {
            diagnostics.push(format!(
                "release module {key} registry_publish_status must be {REGISTRY_PUBLISH_STATUS:?}"
            ));
        }
        if release_module.module_source_kind != "local-path" {
            diagnostics.push(format!(
                "release module {key} module_source_kind must be \"local-path\""
            ));
        }
        if !release_module.module_package_built {
            diagnostics.push(format!(
                "release module {key} module_package_built must be true"
            ));
        }
        if release_module.archive_manifest_ref != archive_manifest_rel {
            diagnostics.push(format!(
                "release module {key} archive_manifest_ref must be {archive_manifest_rel:?}"
            ));
        }
        if release_module.archive_file != archive_module.archive_file
            || release_module.archive_sha256 != archive_module.archive_sha256
        {
            diagnostics.push(format!(
                "release module {key} archive_file and archive_sha256 must mirror archive manifest"
            ));
        }
        for (field, found, expected) in [
            (
                "archive_format",
                release_module.archive_format.as_str(),
                ARCHIVE_FORMAT,
            ),
            (
                "archive_media_type",
                release_module.archive_media_type.as_str(),
                ARCHIVE_MEDIA_TYPE,
            ),
            (
                "archive_compression_method",
                release_module.archive_compression_method.as_str(),
                ARCHIVE_COMPRESSION_METHOD,
            ),
            (
                "archive_deterministic_timestamp",
                release_module.archive_deterministic_timestamp.as_str(),
                ARCHIVE_DETERMINISTIC_TIMESTAMP,
            ),
        ] {
            if found != expected {
                diagnostics.push(format!("release module {key} {field} must be {expected:?}"));
            }
        }
        for (field, found, expected) in [
            (
                "archive_format",
                archive_module.archive_format.as_str(),
                ARCHIVE_FORMAT,
            ),
            (
                "archive_media_type",
                archive_module.archive_media_type.as_str(),
                ARCHIVE_MEDIA_TYPE,
            ),
            (
                "compression_method",
                archive_module.compression_method.as_str(),
                ARCHIVE_COMPRESSION_METHOD,
            ),
            (
                "deterministic_timestamp",
                archive_module.deterministic_timestamp.as_str(),
                ARCHIVE_DETERMINISTIC_TIMESTAMP,
            ),
            (
                "module_signature_status",
                archive_module.module_signature_status.as_str(),
                MODULE_SIGNATURE_STATUS,
            ),
            (
                "slsa_provenance_status",
                archive_module.slsa_provenance_status.as_str(),
                SLSA_STATUS,
            ),
        ] {
            if found != expected {
                diagnostics.push(format!("archive module {key} {field} must be {expected:?}"));
            }
        }
        if !is_sha256_hex(&release_module.archive_sha256) {
            diagnostics.push(format!(
                "release module {key} archive_sha256 must be lowercase SHA-256 hex"
            ));
        }
        validate_release_object_source_pin(key, release_module, diagnostics);
        if release_module.module_signature_status != MODULE_SIGNATURE_STATUS {
            diagnostics.push(format!(
                "release module {key} module_signature_status must be {MODULE_SIGNATURE_STATUS:?}"
            ));
        }
        if release_module.slsa_provenance_status != SLSA_STATUS {
            diagnostics.push(format!(
                "release module {key} slsa_provenance_status must be {SLSA_STATUS:?}"
            ));
        }
        if release_module.provider_lock_scope != "foundation" {
            diagnostics.push(format!(
                "release module {key} provider_lock_scope must be \"foundation\""
            ));
        }
        if release_module.provider_signature_review_ref != provider_signature_review_rel {
            diagnostics.push(format!(
                "release module {key} provider_signature_review_ref must be {provider_signature_review_rel:?}"
            ));
        }
        if release_module.catalog.release_status != RELEASE_STATUS {
            diagnostics.push(format!(
                "release module {key} release_status must be {RELEASE_STATUS:?}"
            ));
        }
        if release_module.files != provenance_module.files {
            diagnostics.push(format!(
                "release module {key} files must mirror provenance files"
            ));
        }
        if archive_module.files != provenance_module.files {
            diagnostics.push(format!(
                "archive module {key} files must mirror provenance files"
            ));
        }
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
        for field in [
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
            if field.1.is_some() {
                diagnostics.push(format!(
                    "release module {key} {} requires archive_source_location",
                    field.0
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
            if is_sha256_hex(integrity) && integrity == release_module.archive_sha256 =>
        {
        }
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

fn validate_manifest_summary(
    manifest: &Value,
    release: &BTreeMap<String, ReleaseModule>,
    diagnostics: &mut Vec<String>,
) {
    let expected_names: Vec<_> = release
        .values()
        .map(|module| module.catalog.name.clone())
        .collect();
    let found_names = required_string_array(
        manifest,
        "/module_release_index_scope/module_names",
        diagnostics,
    )
    .unwrap_or_default();
    if found_names != expected_names {
        diagnostics.push(format!(
            "manifest /module_release_index_scope/module_names must equal release index module names; expected={expected_names:?} found={found_names:?}"
        ));
    }
    let module_count = manifest
        .pointer("/module_release_index_scope/module_count")
        .and_then(Value::as_u64);
    if module_count != Some(release.len() as u64) {
        diagnostics.push(format!(
            "manifest /module_release_index_scope/module_count must equal {}; found={module_count:?}",
            release.len()
        ));
    }
}

fn validate_file_digests(
    repo_root: &Path,
    provenance: &BTreeMap<String, ProvenanceModule>,
    release: &BTreeMap<String, ReleaseModule>,
    diagnostics: &mut Vec<String>,
) -> usize {
    let mut files_checked = 0;
    for (key, provenance_module) in provenance {
        let Some(release_module) = release.get(key) else {
            continue;
        };
        for (path, expected_sha) in &release_module.files {
            let Some(provenance_sha) = provenance_module.files.get(path) else {
                continue;
            };
            if provenance_sha != expected_sha {
                diagnostics.push(format!(
                    "release module {key} file {path} sha256 must match provenance"
                ));
            }
            let full_path = repo_root.join(path);
            if sha256_file(&full_path, diagnostics).as_deref() != Some(expected_sha.as_str()) {
                diagnostics.push(format!(
                    "release module {key} file {path} sha256 must match local file bytes"
                ));
            } else {
                files_checked += 1;
            }
        }
    }
    files_checked
}

fn validate_provider_evidence_paths(
    provider_lock_root: &Path,
    provider_signature_review: &Path,
    diagnostics: &mut Vec<String>,
) {
    if !provider_lock_root.is_dir() {
        diagnostics.push(format!(
            "provider lock root does not exist or is not a directory: {}",
            provider_lock_root.display()
        ));
    }
    if !provider_signature_review.is_file() {
        diagnostics.push(format!(
            "provider signature review does not exist or is not a file: {}",
            provider_signature_review.display()
        ));
    }
    if provider_lock_root.join(".terraform").exists() {
        diagnostics.push(format!(
            "release index provider lock root must not contain .terraform provider cache: {}",
            provider_lock_root.join(".terraform").display()
        ));
    }
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

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_file(path: &Path, diagnostics: &mut Vec<String>) -> Option<String> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            diagnostics.push(format!("unable to hash {}: {error}", path.display()));
            return None;
        }
    };
    Some(hex_lower(&Sha256::digest(bytes)))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use sha2::{Digest, Sha256};

    use super::{
        CloudIacModuleReleaseIndexArgs, parse_cloud_iac_module_release_index_args,
        validate_cloud_iac_module_release_index_gate,
    };

    #[test]
    fn parse_cloud_iac_module_release_index_rejects_unknown_flag() {
        let error = parse_cloud_iac_module_release_index_args(vec!["--bogus".into()])
            .expect_err("unknown flag should fail");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn cloud_iac_module_release_index_accepts_valid_fixture() {
        let temp = TempRepo::new("cloud-iac-release-index-valid");
        write_fixture(temp.path(), FixtureDrift::None);

        let report = validate_cloud_iac_module_release_index_gate(fixture_args(temp.path()))
            .expect("valid fixture should pass");

        assert_eq!(report.modules_checked, 2);
        assert_eq!(report.files_checked, 4);
    }

    #[test]
    fn cloud_iac_module_release_index_rejects_signed_overclaim() {
        let temp = TempRepo::new("cloud-iac-release-index-signed");
        write_fixture(temp.path(), FixtureDrift::SignedOverclaim);

        let error = validate_cloud_iac_module_release_index_gate(fixture_args(temp.path()))
            .expect_err("signing overclaim should fail");

        assert!(error.contains("module_signature_status"));
    }

    #[test]
    fn cloud_iac_module_release_index_rejects_digest_drift() {
        let temp = TempRepo::new("cloud-iac-release-index-digest");
        write_fixture(temp.path(), FixtureDrift::DigestDrift);

        let error = validate_cloud_iac_module_release_index_gate(fixture_args(temp.path()))
            .expect_err("digest drift should fail");

        assert!(error.contains("sha256 must match local file bytes"));
    }

    #[test]
    fn cloud_iac_module_release_index_rejects_missing_module() {
        let temp = TempRepo::new("cloud-iac-release-index-missing-module");
        write_fixture(temp.path(), FixtureDrift::MissingModule);

        let error = validate_cloud_iac_module_release_index_gate(fixture_args(temp.path()))
            .expect_err("missing module should fail");

        assert!(error.contains("release index module keys must match catalog"));
    }

    #[test]
    fn cloud_iac_module_release_index_accepts_pinned_object_source_metadata() {
        for (fixture_name, drift) in [
            (
                "cloud-iac-release-index-pinned-s3-object-source",
                FixtureDrift::PinnedS3ObjectSource,
            ),
            (
                "cloud-iac-release-index-pinned-gcs-object-source",
                FixtureDrift::PinnedGcsObjectSource,
            ),
        ] {
            let temp = TempRepo::new(fixture_name);
            write_fixture(temp.path(), drift);

            let report = validate_cloud_iac_module_release_index_gate(fixture_args(temp.path()))
                .expect("pinned object-source metadata should pass");

            assert_eq!(report.modules_checked, 2);
        }
    }

    #[test]
    fn cloud_iac_module_release_index_rejects_unpinned_object_source_metadata() {
        let temp = TempRepo::new("cloud-iac-release-index-object-source-pin");
        write_fixture(temp.path(), FixtureDrift::UnpinnedObjectSource);

        let error = validate_cloud_iac_module_release_index_gate(fixture_args(temp.path()))
            .expect_err("unpinned object-source metadata should fail");

        assert!(error.contains("archive_source_version_id"));
    }

    #[test]
    fn cloud_iac_module_release_index_rejects_mismatched_object_source_integrity() {
        let temp = TempRepo::new("cloud-iac-release-index-object-source-integrity");
        write_fixture(temp.path(), FixtureDrift::MismatchedObjectSourceIntegrity);

        let error = validate_cloud_iac_module_release_index_gate(fixture_args(temp.path()))
            .expect_err("mismatched object-source integrity should fail");

        assert!(error.contains("archive_source_integrity_sha256"));
    }

    fn fixture_args(repo_root: &Path) -> CloudIacModuleReleaseIndexArgs {
        CloudIacModuleReleaseIndexArgs {
            repo_root: repo_root.to_path_buf(),
            manifest: PathBuf::from("iac/manifest.json"),
            catalog: PathBuf::from("iac/tofu/modules/catalog.json"),
            provenance: PathBuf::from("iac/tofu/modules/provenance.json"),
            release_index: PathBuf::from("iac/tofu/modules/release-index.json"),
            archive_manifest: PathBuf::from(
                "iac/tofu/modules/archive-manifest.json",
            ),
            provider_lock_root: PathBuf::from(
                "iac/tofu/provider-locks/foundation",
            ),
            provider_signature_review: PathBuf::from(
                "iac/tofu/provider-locks/foundation/provider-signature-review.json",
            ),
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum FixtureDrift {
        None,
        SignedOverclaim,
        DigestDrift,
        MissingModule,
        PinnedS3ObjectSource,
        PinnedGcsObjectSource,
        UnpinnedObjectSource,
        MismatchedObjectSourceIntegrity,
    }

    fn write_fixture(root: &Path, drift: FixtureDrift) {
        for module in ["cloud-account", "dns"] {
            let module_root = root.join(format!("iac/tofu/modules/{module}"));
            fs::create_dir_all(&module_root).expect("module dir");
            fs::write(module_root.join("main.tofu"), format!("# {module}\n")).expect("main");
            fs::write(module_root.join("README.md"), format!("# {module}\n")).expect("readme");
        }
        let lock_root = root.join("iac/tofu/provider-locks/foundation");
        fs::create_dir_all(&lock_root).expect("lock root");
        fs::write(lock_root.join("provider-signature-review.json"), "{}\n").expect("sig review");
        fs::create_dir_all(root.join("iac/tofu/modules")).expect("modules");
        fs::write(
            root.join("iac/tofu/modules/catalog.json"),
            fixture_catalog(),
        )
        .expect("catalog");
        fs::write(
            root.join("iac/tofu/modules/provenance.json"),
            fixture_provenance(root),
        )
        .expect("provenance");
        fs::write(
            root.join("iac/tofu/modules/release-index.json"),
            fixture_release_index(root, drift),
        )
        .expect("release index");
        fs::write(
            root.join("iac/tofu/modules/archive-manifest.json"),
            fixture_archive_manifest(root, drift),
        )
        .expect("archive manifest");
        fs::create_dir_all(root.join("iac")).expect("manifest dir");
        fs::write(
            root.join("iac/manifest.json"),
            fixture_manifest(),
        )
        .expect("manifest");
    }

    fn fixture_catalog() -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "modules": [module_row("cloud-account"), module_row("dns")]
        }))
        .expect("catalog")
    }

    fn fixture_provenance(root: &Path) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "modules": [provenance_row(root, "cloud-account"), provenance_row(root, "dns")]
        }))
        .expect("provenance")
    }

    fn fixture_release_index(root: &Path, drift: FixtureDrift) -> String {
        let mut modules = vec![
            release_row(root, "cloud-account", drift),
            release_row(root, "dns", drift),
        ];
        if drift == FixtureDrift::MissingModule {
            modules.pop();
        }
        let sig_hash = sha256_for_test(&root.join(
            "iac/tofu/provider-locks/foundation/provider-signature-review.json",
        ));
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": "1.0",
            "release_index_id": "cloud-iac-opentofu-modules-local-release-index",
            "generated_by_changeset": "CS-CLOUD-IAC-MODULE-RELEASE-INDEX-GATE-001",
            "authority": {
                "source_catalog": "iac/tofu/modules/catalog.json",
                "source_provenance": "iac/tofu/modules/provenance.json",
                "source_archive_manifest": "iac/tofu/modules/archive-manifest.json",
                "source_provider_lock_root": "iac/tofu/provider-locks/foundation",
                "source_provider_signature_review": "iac/tofu/provider-locks/foundation/provider-signature-review.json",
                "runtime_mode": "local-opentofu-module-release-index-gate",
                "official_sources_consulted": super::REQUIRED_OFFICIAL_SOURCES,
                "non_claims": required_nonclaims()
            },
            "policy": {
                "status": "local-release-index-no-private-registry-api",
                "required_release_status": "local-foundation-skeleton",
                "private_registry_api_implemented": false,
                "service_discovery_implemented": false,
                "download_endpoint_implemented": false,
                "module_archives_built": true,
                "module_signing_executed": false,
                "slsa_provenance_generated": false,
                "tofu_plan_apply_executed": false,
                "provider_resource_complete_modules": false
            },
            "provider_evidence": {
                "lock_root": "iac/tofu/provider-locks/foundation",
                "signature_review": "iac/tofu/provider-locks/foundation/provider-signature-review.json",
                "signature_review_sha256": sig_hash
            },
            "modules": modules
        }))
        .expect("release index")
    }

    fn fixture_archive_manifest(root: &Path, drift: FixtureDrift) -> String {
        let mut modules = vec![
            archive_row(root, "cloud-account", drift),
            archive_row(root, "dns", drift),
        ];
        if drift == FixtureDrift::MissingModule {
            modules.pop();
        }
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": "1.0",
            "generated_by_changeset": "CS-CLOUD-IAC-MODULE-ARCHIVE-GATE-001",
            "modules": modules
        }))
        .expect("archive manifest")
    }

    fn fixture_manifest() -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "capabilities": [{
                "tier": "T1",
                "name": "cloud-iac-module-release-index-gate",
                "file": "crates/oya-dev-cli/src/cloud_iac_module_release_index_gate.rs",
                "risk_class": "high"
            }],
            "foundation_non_claims": ["CS-CLOUD-IAC-MODULE-RELEASE-INDEX-GATE-001 adds a local release index only; no private registry API, signing, SLSA, plan/apply, or cloud runtime is claimed."],
            "module_release_index_scope": {
                "catalog": "iac/tofu/modules/catalog.json",
                "provenance": "iac/tofu/modules/provenance.json",
                "release_index": "iac/tofu/modules/release-index.json",
                "archive_manifest": "iac/tofu/modules/archive-manifest.json",
                "provider_lock_root": "iac/tofu/provider-locks/foundation",
                "provider_signature_review": "iac/tofu/provider-locks/foundation/provider-signature-review.json",
                "status": "local-release-index-no-private-registry-api",
                "runtime_mode": "local-opentofu-module-release-index-gate",
                "module_count": 2,
                "module_names": ["cloud-account", "dns"],
                "official_sources_consulted": super::REQUIRED_OFFICIAL_SOURCES,
                "coherence_guard": {
                    "changeset": "CS-CLOUD-IAC-MODULE-RELEASE-INDEX-GATE-001",
                    "gate": "cloud-iac-module-release-index",
                    "gate_file": "crates/oya-dev-cli/src/cloud_iac_module_release_index_gate.rs",
                    "runtime_mode": "local-opentofu-module-release-index-gate"
                },
                "non_claims": required_nonclaims()
            }
        }))
        .expect("manifest")
    }

    fn module_row(name: &str) -> serde_json::Value {
        serde_json::json!({
            "namespace": "oyatie",
            "name": name,
            "system": "opentofu",
            "version": "0.1.0",
            "source_path": format!("iac/tofu/modules/{name}"),
            "main_file": format!("iac/tofu/modules/{name}/main.tofu"),
            "release_status": "local-foundation-skeleton",
            "evidence_ref": format!("evidence://cloud-iac/modules/{name}/0.1.0/local-foundation")
        })
    }

    fn provenance_row(root: &Path, name: &str) -> serde_json::Value {
        let mut row = module_row(name);
        row["files"] = serde_json::json!([
            {"path": format!("iac/tofu/modules/{name}/main.tofu"), "sha256": sha256_for_test(&root.join(format!("iac/tofu/modules/{name}/main.tofu")))},
            {"path": format!("iac/tofu/modules/{name}/README.md"), "sha256": sha256_for_test(&root.join(format!("iac/tofu/modules/{name}/README.md")))}
        ]);
        row
    }

    fn release_row(root: &Path, name: &str, drift: FixtureDrift) -> serde_json::Value {
        let mut row = provenance_row(root, name);
        let address = format!("oyatie/{name}/opentofu");
        row["address"] = serde_json::json!(address);
        row["versions_endpoint_path"] =
            serde_json::json!(format!("/v1/modules/oyatie/{name}/opentofu/versions"));
        row["download_endpoint_path"] =
            serde_json::json!(format!("/v1/modules/oyatie/{name}/opentofu/0.1.0/download"));
        row["registry_publish_status"] = serde_json::json!("local-index-only-no-service");
        row["module_source_kind"] = serde_json::json!("local-path");
        row["module_package_built"] = serde_json::json!(true);
        row["archive_manifest_ref"] =
            serde_json::json!("iac/tofu/modules/archive-manifest.json");
        row["archive_file"] = serde_json::json!(archive_file(name));
        row["archive_sha256"] = serde_json::json!(archive_sha(root, name));
        row["archive_format"] = serde_json::json!("zip");
        row["archive_media_type"] = serde_json::json!("archive/zip");
        row["archive_compression_method"] = serde_json::json!("store");
        row["archive_deterministic_timestamp"] = serde_json::json!("1980-01-01T00:00:00Z");
        row["module_signature_status"] =
            serde_json::json!(if drift == FixtureDrift::SignedOverclaim {
                "signed-cosign"
            } else {
                "unsigned-no-cosign"
            });
        row["slsa_provenance_status"] = serde_json::json!("not-generated");
        row["provider_lock_scope"] = serde_json::json!("foundation");
        row["provider_signature_review_ref"] = serde_json::json!(
            "iac/tofu/provider-locks/foundation/provider-signature-review.json"
        );
        if drift == FixtureDrift::DigestDrift && name == "dns" {
            row["files"][0]["sha256"] = serde_json::json!("0".repeat(64));
        }
        if drift == FixtureDrift::UnpinnedObjectSource && name == "dns" {
            row["archive_source_location"] = serde_json::json!(format!(
                "s3::https://s3.amazonaws.com/oyatie-cloud-iac-modules/oyatie/{name}/0.1.0/oyatie-{name}-opentofu-0.1.0.zip"
            ));
            row["archive_source_integrity_sha256"] = row["archive_sha256"].clone();
        }
        if drift == FixtureDrift::PinnedS3ObjectSource && name == "dns" {
            row["archive_source_location"] = serde_json::json!(format!(
                "s3::https://s3.amazonaws.com/oyatie-cloud-iac-modules/oyatie/{name}/0.1.0/oyatie-{name}-opentofu-0.1.0.zip"
            ));
            row["archive_source_integrity_sha256"] = row["archive_sha256"].clone();
            row["archive_source_version_id"] = serde_json::json!("s3v-local-foundation-0001");
        }
        if drift == FixtureDrift::PinnedGcsObjectSource && name == "dns" {
            row["archive_source_location"] = serde_json::json!(format!(
                "gcs::https://www.googleapis.com/storage/v1/oyatie-cloud-iac-modules/oyatie/{name}/0.1.0/oyatie-{name}-opentofu-0.1.0.zip"
            ));
            row["archive_source_integrity_sha256"] = row["archive_sha256"].clone();
            row["archive_source_generation"] = serde_json::json!("1700000000000001");
        }
        if drift == FixtureDrift::MismatchedObjectSourceIntegrity && name == "dns" {
            row["archive_source_location"] = serde_json::json!(format!(
                "s3::https://s3.amazonaws.com/oyatie-cloud-iac-modules/oyatie/{name}/0.1.0/oyatie-{name}-opentofu-0.1.0.zip"
            ));
            row["archive_source_integrity_sha256"] = serde_json::json!(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            );
            row["archive_source_version_id"] = serde_json::json!("s3v-local-foundation-0001");
        }
        row
    }

    fn archive_row(root: &Path, name: &str, drift: FixtureDrift) -> serde_json::Value {
        let mut row = provenance_row(root, name);
        row["archive_file"] = serde_json::json!(archive_file(name));
        row["archive_sha256"] = serde_json::json!(archive_sha(root, name));
        row["archive_format"] = serde_json::json!("zip");
        row["archive_media_type"] = serde_json::json!("archive/zip");
        row["compression_method"] = serde_json::json!("store");
        row["deterministic_timestamp"] = serde_json::json!("1980-01-01T00:00:00Z");
        row["module_signature_status"] =
            serde_json::json!(if drift == FixtureDrift::SignedOverclaim {
                "signed-cosign"
            } else {
                "unsigned-no-cosign"
            });
        row["slsa_provenance_status"] = serde_json::json!("not-generated");
        row
    }

    fn archive_file(name: &str) -> String {
        format!("target/oya-cloud-iac/module-archives/oyatie-{name}-opentofu-0.1.0.zip")
    }

    fn archive_sha(root: &Path, name: &str) -> String {
        sha256_for_test(&root.join(format!(
            "iac/tofu/modules/{name}/main.tofu"
        )))
    }

    fn required_nonclaims() -> Vec<&'static str> {
        vec![
            "no private module registry API",
            "no module service discovery",
            "no live module download endpoint",
            "no module signing or Sigstore execution",
            "no SLSA attestation generation",
            "no tofu test/plan/apply evidence",
            "no cloud resource provisioning",
        ]
    }

    fn sha256_for_test(path: &Path) -> String {
        let bytes = fs::read(path).expect("hash input");
        super::hex_lower(&Sha256::digest(bytes))
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
