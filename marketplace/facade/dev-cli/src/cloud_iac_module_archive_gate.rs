//! `oya gate validate cloud-iac-module-archive` runner.
//!
//! This gate materializes deterministic repo-local `.zip` archives for the
//! Cloud IaC reusable OpenTofu modules and verifies the archive manifest against
//! catalog, provenance, and release-index metadata. It intentionally proves only
//! local package bytes and SHA-256 coherence: no private module registry service,
//! service discovery document, live download endpoint, cosign/Sigstore signing,
//! SLSA attestation, provider runtime, tofu plan/apply, or cloud provisioning is
//! implemented by this gate.

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
const DEFAULT_OUT_DIR: &str = "target/oya-cloud-iac/module-archives";
const GATE_NAME: &str = "cloud-iac-module-archive";
const GATE_FILE: &str = "crates/oya-dev-cli/src/cloud_iac_module_archive_gate.rs";
const CHANGESET_ID: &str = "CS-CLOUD-IAC-MODULE-ARCHIVE-GATE-001";
const RUNTIME_MODE: &str = "local-deterministic-zip-module-archive-gate";
const ARCHIVE_STATUS: &str = "deterministic-local-module-archives-no-private-registry-api";
const RELEASE_STATUS: &str = "local-foundation-skeleton";
const ARCHIVE_FORMAT: &str = "zip";
const ARCHIVE_MEDIA_TYPE: &str = "archive/zip";
const COMPRESSION_METHOD: &str = "store";
const DETERMINISTIC_TIMESTAMP: &str = "1980-01-01T00:00:00Z";
const MODULE_SIGNATURE_STATUS: &str = "unsigned-no-cosign";
const SLSA_STATUS: &str = "not-generated";
const REQUIRED_OFFICIAL_SOURCES: &[&str] = &[
    "https://opentofu.org/docs/internals/module-registry-protocol/",
    "https://opentofu.org/docs/language/modules/sources/",
    "https://opentofu.org/docs/cli/oci_registries/module-package/",
    "https://slsa.dev/spec/v1.2/",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacModuleArchiveArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) catalog: PathBuf,
    pub(crate) provenance: PathBuf,
    pub(crate) release_index: PathBuf,
    pub(crate) archive_manifest: PathBuf,
    pub(crate) out_dir: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacModuleArchiveReport {
    pub(crate) manifest_path: String,
    pub(crate) catalog_path: String,
    pub(crate) provenance_path: String,
    pub(crate) release_index_path: String,
    pub(crate) archive_manifest_path: String,
    pub(crate) output_dir: String,
    pub(crate) modules_checked: usize,
    pub(crate) files_archived: usize,
    pub(crate) archives_built: usize,
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
    module_package_built: bool,
    archive_manifest_ref: String,
    archive_file: String,
    archive_sha256: String,
    archive_media_type: String,
    archive_format: String,
    archive_compression_method: String,
    archive_deterministic_timestamp: String,
    module_signature_status: String,
    slsa_provenance_status: String,
    files: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ArchiveModule {
    catalog: CatalogModule,
    address: String,
    release_index_ref: String,
    archive_file: String,
    archive_sha256: String,
    archive_format: String,
    archive_media_type: String,
    compression_method: String,
    deterministic_timestamp: String,
    module_signature_status: String,
    slsa_provenance_status: String,
    files: BTreeMap<String, String>,
    archive_entries: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BuiltArchive {
    archive_file: String,
    archive_sha256: String,
    files_archived: usize,
    archive_entries: Vec<String>,
    bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ZipEntryInput {
    name: String,
    bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ZipCentralDirectoryEntry {
    name: String,
    crc32: u32,
    size: u32,
    local_header_offset: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ArchiveModuleSetRefs<'a> {
    release_index_rel: &'a str,
    archive_manifest_rel: &'a str,
    out_dir_rel: &'a str,
}

pub(crate) fn parse_cloud_iac_module_archive_args(
    args: Vec<String>,
) -> Result<CloudIacModuleArchiveArgs, String> {
    let mut parsed = CloudIacModuleArchiveArgs {
        repo_root: PathBuf::from(DEFAULT_REPO_ROOT),
        manifest: PathBuf::from(DEFAULT_MANIFEST),
        catalog: PathBuf::from(DEFAULT_CATALOG),
        provenance: PathBuf::from(DEFAULT_PROVENANCE),
        release_index: PathBuf::from(DEFAULT_RELEASE_INDEX),
        archive_manifest: PathBuf::from(DEFAULT_ARCHIVE_MANIFEST),
        out_dir: PathBuf::from(DEFAULT_OUT_DIR),
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
            "--out-dir" => parsed.out_dir = take_path_arg(&mut args, "--out-dir")?,
            other => {
                return Err(format!(
                    "cloud-iac-module-archive: unknown flag {other:?}; usage: \
                     oya gate validate cloud-iac-module-archive \
                     [--repo-root <.>] \
                     [--manifest <iac/manifest.json>] \
                     [--catalog <iac/tofu/modules/catalog.json>] \
                     [--provenance <iac/tofu/modules/provenance.json>] \
                     [--release-index <iac/tofu/modules/release-index.json>] \
                     [--archive-manifest <iac/tofu/modules/archive-manifest.json>] \
                     [--out-dir <target/oya-cloud-iac/module-archives>]"
                ));
            }
        }
    }

    Ok(parsed)
}

fn take_path_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("cloud-iac-module-archive: {flag} requires a path argument"))
}

pub(crate) fn validate_cloud_iac_module_archive_gate(
    args: CloudIacModuleArchiveArgs,
) -> Result<CloudIacModuleArchiveReport, String> {
    let manifest_path = resolve_repo_path(&args.repo_root, &args.manifest);
    let catalog_path = resolve_repo_path(&args.repo_root, &args.catalog);
    let provenance_path = resolve_repo_path(&args.repo_root, &args.provenance);
    let release_index_path = resolve_repo_path(&args.repo_root, &args.release_index);
    let archive_manifest_path = resolve_repo_path(&args.repo_root, &args.archive_manifest);
    let out_dir_path = resolve_repo_path(&args.repo_root, &args.out_dir);

    let manifest_rel = repo_relative_argument(&args.repo_root, &args.manifest)?;
    let catalog_rel = repo_relative_argument(&args.repo_root, &args.catalog)?;
    let provenance_rel = repo_relative_argument(&args.repo_root, &args.provenance)?;
    let release_index_rel = repo_relative_argument(&args.repo_root, &args.release_index)?;
    let archive_manifest_rel = repo_relative_argument(&args.repo_root, &args.archive_manifest)?;
    let out_dir_rel = repo_relative_argument(&args.repo_root, &args.out_dir)?;

    let manifest = read_json(&manifest_path, "manifest")?;
    let catalog = read_json(&catalog_path, "module catalog")?;
    let provenance = read_json(&provenance_path, "module provenance")?;
    let release_index = read_json(&release_index_path, "module release index")?;
    let archive_manifest = read_json(&archive_manifest_path, "module archive manifest")?;

    let mut diagnostics = Vec::new();
    validate_manifest_scope(
        &manifest,
        &catalog_rel,
        &provenance_rel,
        &release_index_rel,
        &archive_manifest_rel,
        &out_dir_rel,
        &mut diagnostics,
    );
    validate_archive_manifest_header(
        &archive_manifest,
        &catalog_rel,
        &provenance_rel,
        &release_index_rel,
        &out_dir_rel,
        &mut diagnostics,
    );
    validate_archive_policy(&archive_manifest, &mut diagnostics);
    validate_release_index_archive_policy(&release_index, &archive_manifest_rel, &mut diagnostics);
    validate_no_secret_markers(&archive_manifest, &mut diagnostics);

    let catalog_modules = parse_catalog_modules(&catalog, &mut diagnostics);
    let provenance_modules = parse_provenance_modules(&provenance, &mut diagnostics);
    let release_modules = parse_release_modules(&release_index, &mut diagnostics);
    let archive_modules = parse_archive_modules(&archive_manifest, &mut diagnostics);

    validate_module_sets(
        &catalog_modules,
        &provenance_modules,
        &release_modules,
        &archive_modules,
        ArchiveModuleSetRefs {
            release_index_rel: &release_index_rel,
            archive_manifest_rel: &archive_manifest_rel,
            out_dir_rel: &out_dir_rel,
        },
        &mut diagnostics,
    );
    validate_manifest_summary(&manifest, &archive_modules, &mut diagnostics);

    let mut files_archived = 0;
    let mut archives_built = 0;
    fs::create_dir_all(&out_dir_path).map_err(|error| {
        format!(
            "cloud-iac-module-archive: unable to create output directory {}: {error}",
            out_dir_path.display()
        )
    })?;
    for (key, provenance_module) in &provenance_modules {
        let Some(archive_module) = archive_modules.get(key) else {
            continue;
        };
        let built = build_archive(
            &args.repo_root,
            &out_dir_rel,
            &provenance_module.catalog,
            &provenance_module.files,
            &mut diagnostics,
        );
        let Some(built) = built else {
            continue;
        };
        if built.archive_file != archive_module.archive_file {
            diagnostics.push(format!(
                "archive module {key} archive_file must be {:?}",
                built.archive_file
            ));
        }
        if built.archive_sha256 != archive_module.archive_sha256 {
            diagnostics.push(format!(
                "archive module {key} archive_sha256 must match deterministic archive bytes; expected {:?} found {:?}",
                built.archive_sha256, archive_module.archive_sha256
            ));
        }
        if built.archive_entries != archive_module.archive_entries {
            diagnostics.push(format!(
                "archive module {key} archive_entries must equal deterministic archive entry list"
            ));
        }
        let output_path = resolve_repo_path(&args.repo_root, Path::new(&built.archive_file));
        if let Err(error) = write_if_changed(&output_path, &built.bytes) {
            diagnostics.push(format!(
                "unable to write archive {}: {error}",
                output_path.display()
            ));
        } else {
            files_archived += built.files_archived;
            archives_built += 1;
        }
    }

    if diagnostics.is_empty() {
        Ok(CloudIacModuleArchiveReport {
            manifest_path: manifest_rel,
            catalog_path: catalog_rel,
            provenance_path: provenance_rel,
            release_index_path: release_index_rel,
            archive_manifest_path: archive_manifest_rel,
            output_dir: out_dir_rel,
            modules_checked: archive_modules.len(),
            files_archived,
            archives_built,
        })
    } else {
        Err(format!(
            "cloud-iac-module-archive validation failed:\n- {}",
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
                "cloud-iac-module-archive: unable to canonicalize repo root {}: {error}",
                repo_root.display()
            )
        })?;
        let path = fs::canonicalize(path).map_err(|error| {
            format!(
                "cloud-iac-module-archive: unable to canonicalize path {}: {error}",
                path.display()
            )
        })?;
        let relative = path.strip_prefix(&repo_root).map_err(|_| {
            format!(
                "cloud-iac-module-archive: path {} is outside repo root {}",
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
            "cloud-iac-module-archive: unable to read {label} {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "cloud-iac-module-archive: unable to parse {label} JSON {}: {error}",
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

fn validate_manifest_scope(
    manifest: &Value,
    catalog_rel: &str,
    provenance_rel: &str,
    release_index_rel: &str,
    archive_manifest_rel: &str,
    out_dir_rel: &str,
    diagnostics: &mut Vec<String>,
) {
    for (pointer, expected) in [
        ("/module_archive_scope/catalog", catalog_rel),
        ("/module_archive_scope/provenance", provenance_rel),
        ("/module_archive_scope/release_index", release_index_rel),
        (
            "/module_archive_scope/archive_manifest",
            archive_manifest_rel,
        ),
        ("/module_archive_scope/output_root", out_dir_rel),
    ] {
        if required_repo_relative_string(manifest, pointer, diagnostics).as_deref()
            != Some(expected)
        {
            diagnostics.push(format!("manifest {pointer} must equal {expected:?}"));
        }
    }
    for (pointer, expected) in [
        ("/module_archive_scope/status", ARCHIVE_STATUS),
        ("/module_archive_scope/runtime_mode", RUNTIME_MODE),
        (
            "/module_archive_scope/coherence_guard/changeset",
            CHANGESET_ID,
        ),
        ("/module_archive_scope/coherence_guard/gate", GATE_NAME),
        ("/module_archive_scope/coherence_guard/gate_file", GATE_FILE),
        (
            "/module_archive_scope/coherence_guard/runtime_mode",
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
        "/module_archive_scope/official_sources_consulted",
        diagnostics,
    );
    validate_nonclaims(manifest, "/module_archive_scope/non_claims", diagnostics);
}

fn validate_manifest_capability(manifest: &Value, diagnostics: &mut Vec<String>) {
    let Some(capabilities) = manifest.pointer("/capabilities").and_then(Value::as_array) else {
        diagnostics.push("manifest /capabilities must be an array".to_string());
        return;
    };
    let Some(capability) = capabilities.iter().find(|entry| {
        entry.pointer("/name").and_then(Value::as_str) == Some("cloud-iac-module-archive-gate")
    }) else {
        diagnostics
            .push("manifest /capabilities must include cloud-iac-module-archive-gate".to_string());
        return;
    };
    if capability.pointer("/file").and_then(Value::as_str) != Some(GATE_FILE) {
        diagnostics.push(format!(
            "manifest cloud-iac-module-archive-gate /file must be {GATE_FILE:?}"
        ));
    }
    if capability.pointer("/tier").and_then(Value::as_str) != Some("T1") {
        diagnostics.push("manifest module-archive capability /tier must be \"T1\"".to_string());
    }
    if capability.pointer("/risk_class").and_then(Value::as_str) != Some("high") {
        diagnostics
            .push("manifest module-archive capability /risk_class must be \"high\"".to_string());
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

fn validate_archive_manifest_header(
    archive_manifest: &Value,
    catalog_rel: &str,
    provenance_rel: &str,
    release_index_rel: &str,
    out_dir_rel: &str,
    diagnostics: &mut Vec<String>,
) {
    if required_string(archive_manifest, "/generated_by_changeset", diagnostics).as_deref()
        != Some(CHANGESET_ID)
    {
        diagnostics.push(format!(
            "archive manifest /generated_by_changeset must be {CHANGESET_ID:?}"
        ));
    }
    for (pointer, expected) in [
        ("/authority/source_catalog", catalog_rel),
        ("/authority/source_provenance", provenance_rel),
        ("/authority/source_release_index", release_index_rel),
        ("/authority/output_root", out_dir_rel),
    ] {
        if required_repo_relative_string(archive_manifest, pointer, diagnostics).as_deref()
            != Some(expected)
        {
            diagnostics.push(format!(
                "archive manifest {pointer} must equal {expected:?}"
            ));
        }
    }
    if required_string(archive_manifest, "/authority/runtime_mode", diagnostics).as_deref()
        != Some(RUNTIME_MODE)
    {
        diagnostics.push(format!(
            "archive manifest /authority/runtime_mode must be {RUNTIME_MODE:?}"
        ));
    }
    validate_required_source_array(
        archive_manifest,
        "/authority/official_sources_consulted",
        diagnostics,
    );
    validate_nonclaims(archive_manifest, "/authority/non_claims", diagnostics);
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

fn validate_archive_policy(archive_manifest: &Value, diagnostics: &mut Vec<String>) {
    for (pointer, expected) in [
        ("/policy/status", ARCHIVE_STATUS),
        ("/policy/archive_format", ARCHIVE_FORMAT),
        ("/policy/archive_media_type", ARCHIVE_MEDIA_TYPE),
        ("/policy/compression_method", COMPRESSION_METHOD),
        ("/policy/deterministic_timestamp", DETERMINISTIC_TIMESTAMP),
    ] {
        if required_string(archive_manifest, pointer, diagnostics).as_deref() != Some(expected) {
            diagnostics.push(format!("archive manifest {pointer} must be {expected:?}"));
        }
    }
    if required_bool(
        archive_manifest,
        "/policy/module_archives_built",
        diagnostics,
    ) != Some(true)
    {
        diagnostics.push("archive manifest /policy/module_archives_built must be true".to_string());
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
        if required_bool(archive_manifest, pointer, diagnostics) != Some(false) {
            diagnostics.push(format!("archive manifest {pointer} must remain false"));
        }
    }
}

fn validate_release_index_archive_policy(
    release_index: &Value,
    archive_manifest_rel: &str,
    diagnostics: &mut Vec<String>,
) {
    if required_repo_relative_string(
        release_index,
        "/authority/source_archive_manifest",
        diagnostics,
    )
    .as_deref()
        != Some(archive_manifest_rel)
    {
        diagnostics.push(format!(
            "release index /authority/source_archive_manifest must equal {archive_manifest_rel:?}"
        ));
    }
    if required_bool(release_index, "/policy/module_archives_built", diagnostics) != Some(true) {
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
        if required_bool(release_index, pointer, diagnostics) != Some(false) {
            diagnostics.push(format!("release index {pointer} must remain false"));
        }
    }
}

fn validate_no_secret_markers(value: &Value, diagnostics: &mut Vec<String>) {
    let Ok(serialized) = serde_json::to_string(value) else {
        diagnostics
            .push("archive manifest could not be serialized for secret-marker scan".to_string());
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
                "archive manifest must not contain credential-like marker {marker:?}"
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
        let catalog = parse_catalog_like_module(module, &label, diagnostics);
        let files = parse_files(module, &format!("{label}/files"), diagnostics);
        if !catalog.key.is_empty() {
            out.insert(
                catalog.key.clone(),
                ReleaseModule {
                    catalog,
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
                    archive_media_type: required_string(module, "/archive_media_type", diagnostics)
                        .unwrap_or_default(),
                    archive_format: required_string(module, "/archive_format", diagnostics)
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
        let archive_entries = required_string_array(module, "/archive_entries", diagnostics)
            .unwrap_or_default()
            .into_iter()
            .map(|entry| normalize_zip_entry_name(&entry, &label, diagnostics))
            .collect::<Option<Vec<_>>>()
            .unwrap_or_default();
        if !catalog.key.is_empty() {
            out.insert(
                catalog.key.clone(),
                ArchiveModule {
                    catalog,
                    address: required_string(module, "/address", diagnostics).unwrap_or_default(),
                    release_index_ref: required_repo_relative_string(
                        module,
                        "/release_index_ref",
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
                    archive_entries,
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
    refs: ArchiveModuleSetRefs<'_>,
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
        if archive_module.address != expected_address {
            diagnostics.push(format!(
                "archive module {key} address must be {expected_address:?}"
            ));
        }
        let expected_archive_file = archive_file_for(refs.out_dir_rel, catalog_module);
        if archive_module.archive_file != expected_archive_file {
            diagnostics.push(format!(
                "archive module {key} archive_file must be {expected_archive_file:?}"
            ));
        }
        if release_module.archive_file != archive_module.archive_file
            || release_module.archive_sha256 != archive_module.archive_sha256
        {
            diagnostics.push(format!(
                "release index module {key} archive file and sha256 must mirror archive manifest"
            ));
        }
        if release_module.archive_manifest_ref != refs.archive_manifest_rel {
            diagnostics.push(format!(
                "release index module {key} archive_manifest_ref must be {:?}",
                refs.archive_manifest_rel
            ));
        }
        if archive_module.release_index_ref != refs.release_index_rel {
            diagnostics.push(format!(
                "archive module {key} release_index_ref must be {:?}",
                refs.release_index_rel
            ));
        }
        if !release_module.module_package_built {
            diagnostics.push(format!(
                "release index module {key} module_package_built must be true"
            ));
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
                COMPRESSION_METHOD,
            ),
            (
                "deterministic_timestamp",
                archive_module.deterministic_timestamp.as_str(),
                DETERMINISTIC_TIMESTAMP,
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
                COMPRESSION_METHOD,
            ),
            (
                "archive_deterministic_timestamp",
                release_module.archive_deterministic_timestamp.as_str(),
                DETERMINISTIC_TIMESTAMP,
            ),
            (
                "module_signature_status",
                release_module.module_signature_status.as_str(),
                MODULE_SIGNATURE_STATUS,
            ),
            (
                "slsa_provenance_status",
                release_module.slsa_provenance_status.as_str(),
                SLSA_STATUS,
            ),
        ] {
            if found != expected {
                diagnostics.push(format!(
                    "release index module {key} {field} must be {expected:?}"
                ));
            }
        }
        if !is_sha256_hex(&archive_module.archive_sha256) {
            diagnostics.push(format!(
                "archive module {key} archive_sha256 must be lowercase SHA-256 hex"
            ));
        }
        if archive_module.catalog.release_status != RELEASE_STATUS {
            diagnostics.push(format!(
                "archive module {key} release_status must be {RELEASE_STATUS:?}"
            ));
        }
        if archive_module.files != provenance_module.files {
            diagnostics.push(format!(
                "archive module {key} files must mirror provenance files"
            ));
        }
        if release_module.files != provenance_module.files {
            diagnostics.push(format!(
                "release index module {key} files must mirror provenance files"
            ));
        }
    }
}

fn validate_manifest_summary(
    manifest: &Value,
    archive: &BTreeMap<String, ArchiveModule>,
    diagnostics: &mut Vec<String>,
) {
    let expected_names: Vec<_> = archive
        .values()
        .map(|module| module.catalog.name.clone())
        .collect();
    let found_names =
        required_string_array(manifest, "/module_archive_scope/module_names", diagnostics)
            .unwrap_or_default();
    if found_names != expected_names {
        diagnostics.push(format!(
            "manifest /module_archive_scope/module_names must equal archive module names; expected={expected_names:?} found={found_names:?}"
        ));
    }
    let module_count = manifest
        .pointer("/module_archive_scope/module_count")
        .and_then(Value::as_u64);
    if module_count != Some(archive.len() as u64) {
        diagnostics.push(format!(
            "manifest /module_archive_scope/module_count must equal {}; found={module_count:?}",
            archive.len()
        ));
    }
    let archive_count = manifest
        .pointer("/module_archive_scope/archive_count")
        .and_then(Value::as_u64);
    if archive_count != Some(archive.len() as u64) {
        diagnostics.push(format!(
            "manifest /module_archive_scope/archive_count must equal {}; found={archive_count:?}",
            archive.len()
        ));
    }
}

fn build_archive(
    repo_root: &Path,
    out_dir_rel: &str,
    module: &CatalogModule,
    files: &BTreeMap<String, String>,
    diagnostics: &mut Vec<String>,
) -> Option<BuiltArchive> {
    let source_prefix = format!("{}/", module.source_path.trim_end_matches('/'));
    let mut entries = Vec::new();
    for (path, expected_sha) in files {
        if !path.starts_with(&source_prefix) {
            diagnostics.push(format!(
                "module {} file {path} must be under source_path {}",
                module.key, module.source_path
            ));
            return None;
        }
        let entry_name = normalize_zip_entry_name(&path[source_prefix.len()..], path, diagnostics)?;
        let full_path = repo_root.join(path);
        let bytes = match fs::read(&full_path) {
            Ok(bytes) => bytes,
            Err(error) => {
                diagnostics.push(format!("unable to read archive input {path}: {error}"));
                return None;
            }
        };
        let found_sha = hex_lower(&Sha256::digest(&bytes));
        if &found_sha != expected_sha {
            diagnostics.push(format!(
                "module {} file {path} sha256 must match local file bytes",
                module.key
            ));
            return None;
        }
        entries.push(ZipEntryInput {
            name: entry_name,
            bytes,
        });
    }
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    let archive_entries = entries.iter().map(|entry| entry.name.clone()).collect();
    let bytes = match build_zip_bytes(&entries) {
        Ok(bytes) => bytes,
        Err(error) => {
            diagnostics.push(format!(
                "unable to build archive for {}: {error}",
                module.key
            ));
            return None;
        }
    };
    let archive_sha256 = hex_lower(&Sha256::digest(&bytes));
    Some(BuiltArchive {
        archive_file: archive_file_for(out_dir_rel, module),
        archive_sha256,
        files_archived: entries.len(),
        archive_entries,
        bytes,
    })
}

fn normalize_zip_entry_name(
    raw: &str,
    label: &str,
    diagnostics: &mut Vec<String>,
) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        diagnostics.push(format!("{label} archive entry name must not be empty"));
        return None;
    }
    if raw.starts_with('/') || raw.ends_with('/') {
        diagnostics.push(format!(
            "{label} archive entry name must be relative file path: {raw:?}"
        ));
        return None;
    }
    let mut parts = Vec::new();
    for component in Path::new(raw).components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                diagnostics.push(format!(
                    "{label} archive entry name must not contain '..': {raw:?}"
                ));
                return None;
            }
            Component::RootDir | Component::Prefix(_) => {
                diagnostics.push(format!(
                    "{label} archive entry name must be relative: {raw:?}"
                ));
                return None;
            }
        }
    }
    if parts.is_empty() {
        diagnostics.push(format!("{label} archive entry name must identify a file"));
        None
    } else {
        Some(parts.join("/"))
    }
}

fn build_zip_bytes(entries: &[ZipEntryInput]) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut central_entries = Vec::new();
    for entry in entries {
        let local_header_offset = checked_u32(bytes.len(), "zip local header offset")?;
        let crc32 = crc32(&entry.bytes);
        let size = checked_u32(entry.bytes.len(), "zip entry size")?;
        let name_bytes = entry.name.as_bytes();
        let name_len = checked_u16(name_bytes.len(), "zip entry name length")?;
        push_u32(&mut bytes, 0x0403_4b50);
        push_u16(&mut bytes, 20);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0x0021);
        push_u32(&mut bytes, crc32);
        push_u32(&mut bytes, size);
        push_u32(&mut bytes, size);
        push_u16(&mut bytes, name_len);
        push_u16(&mut bytes, 0);
        bytes.extend_from_slice(name_bytes);
        bytes.extend_from_slice(&entry.bytes);
        central_entries.push(ZipCentralDirectoryEntry {
            name: entry.name.clone(),
            crc32,
            size,
            local_header_offset,
        });
    }
    let central_directory_offset = checked_u32(bytes.len(), "zip central directory offset")?;
    for entry in &central_entries {
        let name_bytes = entry.name.as_bytes();
        let name_len = checked_u16(name_bytes.len(), "zip central entry name length")?;
        push_u32(&mut bytes, 0x0201_4b50);
        push_u16(&mut bytes, 20);
        push_u16(&mut bytes, 20);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0x0021);
        push_u32(&mut bytes, entry.crc32);
        push_u32(&mut bytes, entry.size);
        push_u32(&mut bytes, entry.size);
        push_u16(&mut bytes, name_len);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);
        push_u16(&mut bytes, 0);
        push_u32(&mut bytes, 0);
        push_u32(&mut bytes, entry.local_header_offset);
        bytes.extend_from_slice(name_bytes);
    }
    let central_directory_size = checked_u32(
        bytes
            .len()
            .checked_sub(central_directory_offset as usize)
            .ok_or_else(|| "zip central directory size underflow".to_string())?,
        "zip central directory size",
    )?;
    let entry_count = checked_u16(central_entries.len(), "zip entry count")?;
    push_u32(&mut bytes, 0x0605_4b50);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, 0);
    push_u16(&mut bytes, entry_count);
    push_u16(&mut bytes, entry_count);
    push_u32(&mut bytes, central_directory_size);
    push_u32(&mut bytes, central_directory_offset);
    push_u16(&mut bytes, 0);
    Ok(bytes)
}

fn checked_u16(value: usize, label: &str) -> Result<u16, String> {
    u16::try_from(value).map_err(|_| format!("{label} exceeds ZIP32 limit"))
}

fn checked_u32(value: usize, label: &str) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("{label} exceeds ZIP32 limit"))
}

fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    match fs::read(path) {
        Ok(existing) if existing == bytes => Ok(()),
        Ok(_) | Err(_) => fs::write(path, bytes).map_err(|error| error.to_string()),
    }
}

fn archive_file_for(out_dir_rel: &str, module: &CatalogModule) -> String {
    format!(
        "{}/{}-{}-{}-{}.zip",
        out_dir_rel.trim_end_matches('/'),
        module.namespace,
        module.name,
        module.system,
        module.version
    )
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
        CloudIacModuleArchiveArgs, ZipEntryInput, build_zip_bytes,
        parse_cloud_iac_module_archive_args, validate_cloud_iac_module_archive_gate,
    };

    #[test]
    fn parse_cloud_iac_module_archive_rejects_unknown_flag() {
        let error = parse_cloud_iac_module_archive_args(vec!["--bogus".into()])
            .expect_err("unknown flag should fail");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn cloud_iac_module_archive_accepts_valid_fixture_and_writes_archives() {
        let temp = TempRepo::new("cloud-iac-archive-valid");
        write_fixture(temp.path(), FixtureDrift::None);

        let report = validate_cloud_iac_module_archive_gate(fixture_args(temp.path()))
            .expect("valid fixture should pass");

        assert_eq!(report.modules_checked, 2);
        assert_eq!(report.files_archived, 4);
        assert_eq!(report.archives_built, 2);
        assert!(
            temp.path()
                .join(
                    "target/oya-cloud-iac/module-archives/oyatie-cloud-account-opentofu-0.1.0.zip"
                )
                .is_file()
        );
    }

    #[test]
    fn cloud_iac_module_archive_rejects_archive_digest_drift() {
        let temp = TempRepo::new("cloud-iac-archive-digest");
        write_fixture(temp.path(), FixtureDrift::ArchiveDigestDrift);

        let error = validate_cloud_iac_module_archive_gate(fixture_args(temp.path()))
            .expect_err("archive digest drift should fail");

        assert!(error.contains("archive_sha256 must match deterministic archive bytes"));
    }

    #[test]
    fn cloud_iac_module_archive_rejects_signed_overclaim() {
        let temp = TempRepo::new("cloud-iac-archive-signed");
        write_fixture(temp.path(), FixtureDrift::SignedOverclaim);

        let error = validate_cloud_iac_module_archive_gate(fixture_args(temp.path()))
            .expect_err("signature overclaim should fail");

        assert!(error.contains("module_signature_status"));
    }

    #[test]
    fn cloud_iac_module_archive_rejects_missing_module() {
        let temp = TempRepo::new("cloud-iac-archive-missing");
        write_fixture(temp.path(), FixtureDrift::MissingModule);

        let error = validate_cloud_iac_module_archive_gate(fixture_args(temp.path()))
            .expect_err("missing module should fail");

        assert!(error.contains("archive manifest module keys must match catalog"));
    }

    fn fixture_args(repo_root: &Path) -> CloudIacModuleArchiveArgs {
        CloudIacModuleArchiveArgs {
            repo_root: repo_root.to_path_buf(),
            manifest: PathBuf::from("iac/manifest.json"),
            catalog: PathBuf::from("iac/tofu/modules/catalog.json"),
            provenance: PathBuf::from("iac/tofu/modules/provenance.json"),
            release_index: PathBuf::from("iac/tofu/modules/release-index.json"),
            archive_manifest: PathBuf::from(
                "iac/tofu/modules/archive-manifest.json",
            ),
            out_dir: PathBuf::from("target/oya-cloud-iac/module-archives"),
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum FixtureDrift {
        None,
        ArchiveDigestDrift,
        SignedOverclaim,
        MissingModule,
    }

    fn write_fixture(root: &Path, drift: FixtureDrift) {
        for module in ["cloud-account", "dns"] {
            let module_root = root.join(format!("iac/tofu/modules/{module}"));
            fs::create_dir_all(&module_root).expect("module dir");
            fs::write(module_root.join("main.tofu"), format!("# {module}\n")).expect("main");
            fs::write(module_root.join("README.md"), format!("# {module}\n")).expect("readme");
        }
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
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": "1.0",
            "generated_by_changeset": "CS-CLOUD-IAC-MODULE-RELEASE-INDEX-GATE-001",
            "authority": {
                "source_catalog": "iac/tofu/modules/catalog.json",
                "source_provenance": "iac/tofu/modules/provenance.json",
                "source_archive_manifest": "iac/tofu/modules/archive-manifest.json",
                "non_claims": required_nonclaims()
            },
            "policy": {
                "module_archives_built": true,
                "private_registry_api_implemented": false,
                "service_discovery_implemented": false,
                "download_endpoint_implemented": false,
                "module_signing_executed": false,
                "slsa_provenance_generated": false,
                "tofu_plan_apply_executed": false,
                "provider_resource_complete_modules": false
            },
            "modules": [release_row(root, "cloud-account", drift), release_row(root, "dns", drift)]
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
            "archive_manifest_id": "cloud-iac-opentofu-modules-deterministic-local-archives",
            "generated_by_changeset": "CS-CLOUD-IAC-MODULE-ARCHIVE-GATE-001",
            "authority": {
                "source_catalog": "iac/tofu/modules/catalog.json",
                "source_provenance": "iac/tofu/modules/provenance.json",
                "source_release_index": "iac/tofu/modules/release-index.json",
                "output_root": "target/oya-cloud-iac/module-archives",
                "runtime_mode": "local-deterministic-zip-module-archive-gate",
                "official_sources_consulted": super::REQUIRED_OFFICIAL_SOURCES,
                "non_claims": required_nonclaims()
            },
            "policy": {
                "status": "deterministic-local-module-archives-no-private-registry-api",
                "archive_format": "zip",
                "archive_media_type": "archive/zip",
                "compression_method": "store",
                "deterministic_timestamp": "1980-01-01T00:00:00Z",
                "module_archives_built": true,
                "private_registry_api_implemented": false,
                "service_discovery_implemented": false,
                "download_endpoint_implemented": false,
                "module_signing_executed": false,
                "slsa_provenance_generated": false,
                "tofu_plan_apply_executed": false,
                "provider_resource_complete_modules": false
            },
            "modules": modules
        }))
        .expect("archive manifest")
    }

    fn fixture_manifest() -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "capabilities": [{
                "tier": "T1",
                "name": "cloud-iac-module-archive-gate",
                "file": "crates/oya-dev-cli/src/cloud_iac_module_archive_gate.rs",
                "risk_class": "high"
            }],
            "foundation_non_claims": ["CS-CLOUD-IAC-MODULE-ARCHIVE-GATE-001 builds deterministic local module archives only; no private registry service, live download endpoint, signing, SLSA, plan/apply, or cloud runtime is claimed."],
            "module_archive_scope": {
                "catalog": "iac/tofu/modules/catalog.json",
                "provenance": "iac/tofu/modules/provenance.json",
                "release_index": "iac/tofu/modules/release-index.json",
                "archive_manifest": "iac/tofu/modules/archive-manifest.json",
                "output_root": "target/oya-cloud-iac/module-archives",
                "status": "deterministic-local-module-archives-no-private-registry-api",
                "runtime_mode": "local-deterministic-zip-module-archive-gate",
                "module_count": 2,
                "archive_count": 2,
                "module_names": ["cloud-account", "dns"],
                "official_sources_consulted": super::REQUIRED_OFFICIAL_SOURCES,
                "coherence_guard": {
                    "changeset": "CS-CLOUD-IAC-MODULE-ARCHIVE-GATE-001",
                    "gate": "cloud-iac-module-archive",
                    "gate_file": "crates/oya-dev-cli/src/cloud_iac_module_archive_gate.rs",
                    "runtime_mode": "local-deterministic-zip-module-archive-gate"
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
        row["module_package_built"] = serde_json::json!(true);
        row["archive_manifest_ref"] =
            serde_json::json!("iac/tofu/modules/archive-manifest.json");
        row["archive_file"] = serde_json::json!(archive_file(name));
        row["archive_sha256"] = serde_json::json!(archive_sha(root, name, drift));
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
        row
    }

    fn archive_row(root: &Path, name: &str, drift: FixtureDrift) -> serde_json::Value {
        let mut row = provenance_row(root, name);
        row["address"] = serde_json::json!(format!("oyatie/{name}/opentofu"));
        row["release_index_ref"] =
            serde_json::json!("iac/tofu/modules/release-index.json");
        row["archive_file"] = serde_json::json!(archive_file(name));
        row["archive_sha256"] = serde_json::json!(archive_sha(root, name, drift));
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
        row["archive_entries"] = serde_json::json!(["README.md", "main.tofu"]);
        row
    }

    fn archive_file(name: &str) -> String {
        format!("target/oya-cloud-iac/module-archives/oyatie-{name}-opentofu-0.1.0.zip")
    }

    fn archive_sha(root: &Path, name: &str, drift: FixtureDrift) -> String {
        if drift == FixtureDrift::ArchiveDigestDrift && name == "dns" {
            return "0".repeat(64);
        }
        let entries = [
            ZipEntryInput {
                name: "README.md".to_string(),
                bytes: fs::read(root.join(format!(
                    "iac/tofu/modules/{name}/README.md"
                )))
                .expect("readme"),
            },
            ZipEntryInput {
                name: "main.tofu".to_string(),
                bytes: fs::read(root.join(format!(
                    "iac/tofu/modules/{name}/main.tofu"
                )))
                .expect("main"),
            },
        ];
        let bytes = build_zip_bytes(&entries).expect("zip bytes");
        super::hex_lower(&Sha256::digest(bytes))
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
