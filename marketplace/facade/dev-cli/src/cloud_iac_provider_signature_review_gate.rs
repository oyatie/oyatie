//! `oya gate validate cloud-iac-provider-signature-review` runner.
//!
//! This gate verifies the first honest provider signer-review step for Cloud
//! IaC: a repo-local JSON review that records OpenTofu `providers lock` signer
//! key output for the committed provider lockfile. It does not independently
//! verify provider provenance, emit a VSA/SLSA attestation, install providers
//! into the source tree, configure providers, run plan/apply, read credentials,
//! or provision cloud resources.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::slash_path;

const DEFAULT_REPO_ROOT: &str = ".";
const DEFAULT_MANIFEST: &str = "iac/manifest.json";
const DEFAULT_LOCK_ROOT: &str = "iac/tofu/provider-locks/foundation";
const DEFAULT_REVIEW: &str =
    "iac/tofu/provider-locks/foundation/provider-signature-review.json";
const GATE_NAME: &str = "cloud-iac-provider-signature-review";
const GATE_FILE: &str = "crates/oya-dev-cli/src/cloud_iac_provider_signature_review_gate.rs";
const CHANGESET_ID: &str = "CS-CLOUD-IAC-PROVIDER-SIGNATURE-REVIEW-GATE-001";
const RUNTIME_MODE: &str = "local-opentofu-provider-signature-review-gate";
const REVIEW_STATUS: &str = "signed-provider-lock-reviewed-no-vsa";
const PROVIDERS_FILE: &str = "providers.tofu";
const LOCKFILE_NAME: &str = ".terraform.lock.hcl";
const REVIEW_FILE: &str = "provider-signature-review.json";
const MODULES_ROOT: &str = "iac/tofu/modules";
const REQUIRED_OFFICIAL_SOURCES: &[&str] = &[
    "https://opentofu.org/docs/cli/commands/providers/lock/",
    "https://opentofu.org/docs/language/files/dependency-lock/",
    "https://opentofu.org/docs/cli/plugins/signing/",
    "https://slsa.dev/spec/v1.2/verification_summary",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacProviderSignatureReviewArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) lock_root: PathBuf,
    pub(crate) review: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacProviderSignatureReviewReport {
    pub(crate) manifest_path: String,
    pub(crate) lock_root_path: String,
    pub(crate) review_path: String,
    pub(crate) providers_checked: usize,
    pub(crate) signer_keys_checked: usize,
    pub(crate) platforms_checked: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ProviderRequirement {
    local_name: String,
    source: String,
    constraint: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct LockProviderBlock {
    version: String,
    constraints: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReviewProvider {
    local_name: String,
    source: String,
    version: String,
    constraints: String,
    signing_status: String,
    signing_key_id: String,
    platforms: BTreeMap<String, String>,
}

pub(crate) fn parse_cloud_iac_provider_signature_review_args(
    args: Vec<String>,
) -> Result<CloudIacProviderSignatureReviewArgs, String> {
    let mut parsed = CloudIacProviderSignatureReviewArgs {
        repo_root: PathBuf::from(DEFAULT_REPO_ROOT),
        manifest: PathBuf::from(DEFAULT_MANIFEST),
        lock_root: PathBuf::from(DEFAULT_LOCK_ROOT),
        review: PathBuf::from(DEFAULT_REVIEW),
    };

    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--repo-root" => parsed.repo_root = take_path_arg(&mut args, "--repo-root")?,
            "--manifest" => parsed.manifest = take_path_arg(&mut args, "--manifest")?,
            "--lock-root" => parsed.lock_root = take_path_arg(&mut args, "--lock-root")?,
            "--review" => parsed.review = take_path_arg(&mut args, "--review")?,
            other => {
                return Err(format!(
                    "cloud-iac-provider-signature-review: unknown flag {other:?}; usage: \
                     oya gate validate cloud-iac-provider-signature-review \
                     [--repo-root <.>] \
                     [--manifest <iac/manifest.json>] \
                     [--lock-root <iac/tofu/provider-locks/foundation>] \
                     [--review <iac/tofu/provider-locks/foundation/provider-signature-review.json>]"
                ));
            }
        }
    }

    Ok(parsed)
}

fn take_path_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next().map(PathBuf::from).ok_or_else(|| {
        format!("cloud-iac-provider-signature-review: {flag} requires a path argument")
    })
}

pub(crate) fn validate_cloud_iac_provider_signature_review_gate(
    args: CloudIacProviderSignatureReviewArgs,
) -> Result<CloudIacProviderSignatureReviewReport, String> {
    let manifest_path = resolve_repo_path(&args.repo_root, &args.manifest);
    let lock_root_path = resolve_repo_path(&args.repo_root, &args.lock_root);
    let review_path = resolve_repo_path(&args.repo_root, &args.review);
    let manifest_rel = repo_relative_argument(&args.repo_root, &args.manifest)?;
    let lock_root_rel = repo_relative_argument(&args.repo_root, &args.lock_root)?;
    let review_rel = repo_relative_argument(&args.repo_root, &args.review)?;

    let manifest = read_json(&manifest_path, "manifest")?;
    let review = read_json(&review_path, "provider signature review")?;

    let mut diagnostics = Vec::new();
    let platforms =
        require_manifest_scope(&manifest, &lock_root_rel, &review_rel, &mut diagnostics);
    validate_lock_root(&lock_root_rel, &lock_root_path, &mut diagnostics);

    let providers_path = lock_root_path.join(PROVIDERS_FILE);
    let lockfile_path = lock_root_path.join(LOCKFILE_NAME);
    let requirements = parse_required_providers_file(&providers_path, &mut diagnostics);
    let locked = parse_lockfile(&lockfile_path, &mut diagnostics);
    validate_review_header(
        &review,
        &lock_root_rel,
        &review_rel,
        &providers_path,
        &lockfile_path,
        &mut diagnostics,
    );
    validate_review_policy(&review, &platforms, &mut diagnostics);
    let reviewed = parse_review_providers(&review, &mut diagnostics);
    validate_manifest_summary(&manifest, &review, &reviewed, &platforms, &mut diagnostics);
    validate_reviewed_providers(
        &requirements,
        &locked,
        &reviewed,
        &platforms,
        &mut diagnostics,
    );

    if diagnostics.is_empty() {
        let signer_keys_checked = reviewed
            .values()
            .map(|provider| provider.signing_key_id.as_str())
            .collect::<BTreeSet<_>>()
            .len();
        Ok(CloudIacProviderSignatureReviewReport {
            manifest_path: manifest_rel,
            lock_root_path: lock_root_rel,
            review_path: review_rel,
            providers_checked: reviewed.len(),
            signer_keys_checked,
            platforms_checked: platforms.len(),
        })
    } else {
        Err(format!(
            "cloud-iac-provider-signature-review validation failed:\n- {}",
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
                "cloud-iac-provider-signature-review: unable to canonicalize repo root {}: {error}",
                repo_root.display()
            )
        })?;
        let path = fs::canonicalize(path).map_err(|error| {
            format!(
                "cloud-iac-provider-signature-review: unable to canonicalize path {}: {error}",
                path.display()
            )
        })?;
        let relative = path.strip_prefix(&repo_root).map_err(|_| {
            format!(
                "cloud-iac-provider-signature-review: path {} is outside repo root {}",
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
            "cloud-iac-provider-signature-review: unable to read {label} {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "cloud-iac-provider-signature-review: unable to parse {label} JSON {}: {error}",
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
    lock_root_rel: &str,
    review_rel: &str,
    diagnostics: &mut Vec<String>,
) -> Vec<String> {
    let expected_providers_file = format!("{lock_root_rel}/{PROVIDERS_FILE}");
    let expected_lockfile = format!("{lock_root_rel}/{LOCKFILE_NAME}");
    let expected_review = format!("{lock_root_rel}/{REVIEW_FILE}");
    if required_repo_relative_string(
        manifest,
        "/provider_signature_review_scope/lock_root",
        diagnostics,
    )
    .as_deref()
        != Some(lock_root_rel)
    {
        diagnostics.push(format!(
            "manifest /provider_signature_review_scope/lock_root must equal {lock_root_rel:?}"
        ));
    }
    if required_repo_relative_string(
        manifest,
        "/provider_signature_review_scope/providers_file",
        diagnostics,
    )
    .as_deref()
        != Some(expected_providers_file.as_str())
    {
        diagnostics.push(format!(
            "manifest /provider_signature_review_scope/providers_file must equal {expected_providers_file:?}"
        ));
    }
    if required_repo_relative_string(
        manifest,
        "/provider_signature_review_scope/lockfile",
        diagnostics,
    )
    .as_deref()
        != Some(expected_lockfile.as_str())
    {
        diagnostics.push(format!(
            "manifest /provider_signature_review_scope/lockfile must equal {expected_lockfile:?}"
        ));
    }
    if required_repo_relative_string(
        manifest,
        "/provider_signature_review_scope/review",
        diagnostics,
    )
    .as_deref()
        != Some(review_rel)
        || review_rel != expected_review
    {
        diagnostics.push(format!(
            "manifest /provider_signature_review_scope/review and CLI review path must equal {expected_review:?}"
        ));
    }
    if required_string(
        manifest,
        "/provider_signature_review_scope/status",
        diagnostics,
    )
    .as_deref()
        != Some(REVIEW_STATUS)
    {
        diagnostics.push(format!(
            "manifest /provider_signature_review_scope/status must be {REVIEW_STATUS:?}"
        ));
    }
    if required_string(
        manifest,
        "/provider_signature_review_scope/runtime_mode",
        diagnostics,
    )
    .as_deref()
        != Some(RUNTIME_MODE)
    {
        diagnostics.push(format!(
            "manifest /provider_signature_review_scope/runtime_mode must be {RUNTIME_MODE:?}"
        ));
    }
    validate_manifest_capability(manifest, diagnostics);
    validate_manifest_coherence_guard(manifest, diagnostics);
    validate_foundation_non_claim(manifest, diagnostics);
    validate_required_source_array(
        manifest,
        "/provider_signature_review_scope/official_sources_consulted",
        diagnostics,
    );
    let platforms = required_string_array(
        manifest,
        "/provider_signature_review_scope/platforms",
        diagnostics,
    )
    .unwrap_or_default();
    for required in ["darwin_arm64", "linux_amd64", "linux_arm64"] {
        if !platforms.iter().any(|platform| platform == required) {
            diagnostics.push(format!(
                "manifest /provider_signature_review_scope/platforms must include {required:?}"
            ));
        }
    }
    let non_claims = required_string_array(
        manifest,
        "/provider_signature_review_scope/non_claims",
        diagnostics,
    )
    .unwrap_or_default();
    for required in [
        "no provider provenance verification summary or VSA",
        "no SLSA attestation generation",
        "no module signing or Sigstore execution",
        "no tofu test/plan/apply evidence",
        "no cloud resource provisioning",
    ] {
        if !non_claims.iter().any(|claim| claim.contains(required)) {
            diagnostics.push(format!(
                "manifest /provider_signature_review_scope/non_claims must include {required:?}"
            ));
        }
    }
    platforms
}

fn validate_required_source_array(value: &Value, pointer: &str, diagnostics: &mut Vec<String>) {
    let sources = required_string_array(value, pointer, diagnostics).unwrap_or_default();
    for required in REQUIRED_OFFICIAL_SOURCES {
        if !sources.iter().any(|source| source == required) {
            diagnostics.push(format!("{pointer} must include {required:?}"));
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
            == Some("cloud-iac-provider-signature-review-gate")
    }) else {
        diagnostics.push(
            "manifest /capabilities must include cloud-iac-provider-signature-review-gate"
                .to_string(),
        );
        return;
    };
    if capability.pointer("/file").and_then(Value::as_str) != Some(GATE_FILE) {
        diagnostics.push(format!(
            "manifest provider signature review capability /file must be {GATE_FILE:?}"
        ));
    }
    if capability.pointer("/tier").and_then(Value::as_str) != Some("T1") {
        diagnostics
            .push("manifest provider signature review capability /tier must be \"T1\"".to_string());
    }
    if capability.pointer("/risk_class").and_then(Value::as_str) != Some("high") {
        diagnostics.push(
            "manifest provider signature review capability /risk_class must be \"high\""
                .to_string(),
        );
    }
}

fn validate_foundation_non_claim(manifest: &Value, diagnostics: &mut Vec<String>) {
    let non_claims =
        required_string_array(manifest, "/foundation_non_claims", diagnostics).unwrap_or_default();
    if !non_claims
        .iter()
        .any(|claim| claim.contains("CS-CLOUD-IAC-PROVIDER-SIGNATURE-REVIEW-GATE-001"))
    {
        diagnostics.push(
            "manifest /foundation_non_claims must include the provider signature review ChangeSet nonclaim"
                .to_string(),
        );
    }
}

fn validate_manifest_coherence_guard(manifest: &Value, diagnostics: &mut Vec<String>) {
    for (pointer, expected) in [
        (
            "/provider_signature_review_scope/coherence_guard/gate",
            GATE_NAME,
        ),
        (
            "/provider_signature_review_scope/coherence_guard/changeset",
            CHANGESET_ID,
        ),
        (
            "/provider_signature_review_scope/coherence_guard/runtime_mode",
            RUNTIME_MODE,
        ),
        (
            "/provider_signature_review_scope/coherence_guard/gate_file",
            GATE_FILE,
        ),
    ] {
        if required_string(manifest, pointer, diagnostics).as_deref() != Some(expected) {
            diagnostics.push(format!("manifest {pointer} must be {expected:?}"));
        }
    }
}

fn validate_lock_root(lock_root_rel: &str, lock_root: &Path, diagnostics: &mut Vec<String>) {
    if lock_root_rel == MODULES_ROOT
        || lock_root_rel
            .strip_prefix(MODULES_ROOT)
            .is_some_and(|suffix| suffix.starts_with('/'))
    {
        diagnostics.push(format!(
            "provider signature review lock root must stay outside reusable module tree {MODULES_ROOT:?}; found {lock_root_rel:?}"
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
            "provider signature review root must not contain provider installation cache .terraform: {}",
            lock_root.join(".terraform").display()
        ));
    }
}

fn validate_review_header(
    review: &Value,
    lock_root_rel: &str,
    review_rel: &str,
    providers_file: &Path,
    lockfile: &Path,
    diagnostics: &mut Vec<String>,
) {
    let expected_providers_file = format!("{lock_root_rel}/{PROVIDERS_FILE}");
    let expected_lockfile = format!("{lock_root_rel}/{LOCKFILE_NAME}");
    if required_string(review, "/generated_by_changeset", diagnostics).as_deref()
        != Some(CHANGESET_ID)
    {
        diagnostics.push(format!(
            "provider signature review /generated_by_changeset must be {CHANGESET_ID:?}"
        ));
    }
    if required_repo_relative_string(review, "/authority/lock_root", diagnostics).as_deref()
        != Some(lock_root_rel)
    {
        diagnostics.push(format!(
            "review /authority/lock_root must equal {lock_root_rel:?}"
        ));
    }
    if required_repo_relative_string(review, "/authority/providers_file", diagnostics).as_deref()
        != Some(expected_providers_file.as_str())
    {
        diagnostics.push(format!(
            "review /authority/providers_file must equal {expected_providers_file:?}"
        ));
    }
    if required_repo_relative_string(review, "/authority/lockfile", diagnostics).as_deref()
        != Some(expected_lockfile.as_str())
    {
        diagnostics.push(format!(
            "review /authority/lockfile must equal {expected_lockfile:?}"
        ));
    }
    if review_rel != format!("{lock_root_rel}/{REVIEW_FILE}") {
        diagnostics.push(format!(
            "review CLI path must equal {:?}",
            format!("{lock_root_rel}/{REVIEW_FILE}")
        ));
    }
    if required_string(review, "/authority/runtime_mode", diagnostics).as_deref()
        != Some(RUNTIME_MODE)
    {
        diagnostics.push(format!(
            "review /authority/runtime_mode must be {RUNTIME_MODE:?}"
        ));
    }
    validate_required_source_array(review, "/authority/official_sources_consulted", diagnostics);
    validate_no_secret_markers(review, diagnostics);
    if sha256_file(providers_file, diagnostics).as_deref()
        != required_string(review, "/artifacts/providers_file_sha256", diagnostics).as_deref()
    {
        diagnostics.push(
            "review /artifacts/providers_file_sha256 must match providers.tofu bytes".to_string(),
        );
    }
    if sha256_file(lockfile, diagnostics).as_deref()
        != required_string(review, "/artifacts/lockfile_sha256", diagnostics).as_deref()
    {
        diagnostics.push(
            "review /artifacts/lockfile_sha256 must match .terraform.lock.hcl bytes".to_string(),
        );
    }
}

fn validate_no_secret_markers(review: &Value, diagnostics: &mut Vec<String>) {
    let Ok(serialized) = serde_json::to_string(review) else {
        diagnostics.push(
            "provider signature review JSON could not be serialized for secret-marker scan"
                .to_string(),
        );
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
                "provider signature review must not contain credential-like marker {marker:?}"
            ));
        }
    }
}

fn validate_review_policy(review: &Value, platforms: &[String], diagnostics: &mut Vec<String>) {
    if required_string(review, "/policy/status", diagnostics).as_deref() != Some(REVIEW_STATUS) {
        diagnostics.push(format!("review /policy/status must be {REVIEW_STATUS:?}"));
    }
    if required_bool(review, "/policy/signature_review_completed", diagnostics) != Some(true) {
        diagnostics.push("review /policy/signature_review_completed must be true".to_string());
    }
    if required_bool(review, "/policy/provider_lockfile_required", diagnostics) != Some(true) {
        diagnostics.push("review /policy/provider_lockfile_required must be true".to_string());
    }
    for pointer in [
        "/policy/provider_installation_in_source_tree",
        "/policy/provider_provenance_verified",
        "/policy/verification_summary_attestation_emitted",
        "/policy/slsa_attestation_generated",
        "/policy/module_signing_executed",
        "/policy/tofu_plan_apply_executed",
    ] {
        if required_bool(review, pointer, diagnostics) != Some(false) {
            diagnostics.push(format!("review {pointer} must remain false"));
        }
    }
    let review_platforms = required_string_array(review, "/policy/required_platforms", diagnostics)
        .unwrap_or_default();
    if review_platforms != platforms {
        diagnostics.push(format!(
            "review /policy/required_platforms must match manifest platforms; expected={platforms:?} found={review_platforms:?}"
        ));
    }
}

fn validate_manifest_summary(
    manifest: &Value,
    review: &Value,
    reviewed: &BTreeMap<String, ReviewProvider>,
    platforms: &[String],
    diagnostics: &mut Vec<String>,
) {
    for (field, pointer) in [
        ("providers_file_sha256", "/artifacts/providers_file_sha256"),
        ("lockfile_sha256", "/artifacts/lockfile_sha256"),
    ] {
        let manifest_pointer = format!("/provider_signature_review_scope/artifacts/{field}");
        let found = required_string(manifest, &manifest_pointer, diagnostics).unwrap_or_default();
        let expected = required_string(review, pointer, diagnostics).unwrap_or_default();
        if found != expected {
            diagnostics.push(format!(
                "manifest {manifest_pointer} must mirror review {pointer}; expected={expected:?} found={found:?}"
            ));
        }
    }
    let expected_sources: Vec<_> = reviewed.keys().cloned().collect();
    let found_sources = required_string_array(
        manifest,
        "/provider_signature_review_scope/provider_sources",
        diagnostics,
    )
    .unwrap_or_default();
    if found_sources != expected_sources {
        diagnostics.push(format!(
            "manifest /provider_signature_review_scope/provider_sources must equal reviewed provider sources; expected={expected_sources:?} found={found_sources:?}"
        ));
    }
    let expected_versions: BTreeMap<_, _> = reviewed
        .iter()
        .map(|(source, provider)| (source.clone(), provider.version.clone()))
        .collect();
    let found_versions = required_string_object(
        manifest,
        "/provider_signature_review_scope/provider_versions_selected",
        diagnostics,
    )
    .unwrap_or_default();
    if found_versions != expected_versions {
        diagnostics.push(format!(
            "manifest /provider_signature_review_scope/provider_versions_selected must mirror review versions; expected={expected_versions:?} found={found_versions:?}"
        ));
    }
    let expected_keys: BTreeMap<_, _> = reviewed
        .iter()
        .map(|(source, provider)| (source.clone(), provider.signing_key_id.clone()))
        .collect();
    let found_keys = required_string_object(
        manifest,
        "/provider_signature_review_scope/signing_key_ids",
        diagnostics,
    )
    .unwrap_or_default();
    if found_keys != expected_keys {
        diagnostics.push(format!(
            "manifest /provider_signature_review_scope/signing_key_ids must mirror review signer keys; expected={expected_keys:?} found={found_keys:?}"
        ));
    }
    let reviewed_platforms = required_string_array(
        manifest,
        "/provider_signature_review_scope/platforms",
        diagnostics,
    )
    .unwrap_or_default();
    if reviewed_platforms != platforms {
        diagnostics.push(format!(
            "manifest /provider_signature_review_scope/platforms must stay sorted and match review platforms; expected={platforms:?} found={reviewed_platforms:?}"
        ));
    }
}

fn validate_reviewed_providers(
    requirements: &BTreeMap<String, ProviderRequirement>,
    locked: &BTreeMap<String, LockProviderBlock>,
    reviewed: &BTreeMap<String, ReviewProvider>,
    platforms: &[String],
    diagnostics: &mut Vec<String>,
) {
    let expected: BTreeSet<_> = requirements.keys().map(String::as_str).collect();
    let locked_sources: BTreeSet<_> = locked.keys().map(String::as_str).collect();
    let reviewed_sources: BTreeSet<_> = reviewed.keys().map(String::as_str).collect();
    if expected != locked_sources {
        diagnostics.push(format!(
            "lockfile provider sources must match providers.tofu; missing={:?} extra={:?}",
            expected.difference(&locked_sources).collect::<Vec<_>>(),
            locked_sources.difference(&expected).collect::<Vec<_>>()
        ));
    }
    if expected != reviewed_sources {
        diagnostics.push(format!(
            "review provider sources must match providers.tofu; missing={:?} extra={:?}",
            expected.difference(&reviewed_sources).collect::<Vec<_>>(),
            reviewed_sources.difference(&expected).collect::<Vec<_>>()
        ));
    }
    for (source, requirement) in requirements {
        let Some(lock) = locked.get(source) else {
            continue;
        };
        let Some(review) = reviewed.get(source) else {
            continue;
        };
        if review.local_name != requirement.local_name {
            diagnostics.push(format!(
                "review provider {source} local_name must be {:?}; found {:?}",
                requirement.local_name, review.local_name
            ));
        }
        if review.constraints != requirement.constraint || review.constraints != lock.constraints {
            diagnostics.push(format!(
                "review provider {source} constraints must match providers.tofu and lockfile; providers.tofu={:?} lock={:?} review={:?}",
                requirement.constraint, lock.constraints, review.constraints
            ));
        }
        if review.version != lock.version {
            diagnostics.push(format!(
                "review provider {source} version must match lockfile; lock={:?} review={:?}",
                lock.version, review.version
            ));
        }
        if review.signing_status != "signed" {
            diagnostics.push(format!(
                "review provider {source} signing_status must be \"signed\""
            ));
        }
        if !is_hex_key_id(&review.signing_key_id) {
            diagnostics.push(format!(
                "review provider {source} signing_key_id must be an uppercase hex key id"
            ));
        }
        for platform in platforms {
            match review.platforms.get(platform) {
                Some(key_id) if key_id == &review.signing_key_id => {}
                Some(key_id) => diagnostics.push(format!(
                    "review provider {source} platform {platform} signing_key_id must be {:?}; found {key_id:?}",
                    review.signing_key_id
                )),
                None => diagnostics.push(format!(
                    "review provider {source} must include signed retrieval evidence for platform {platform}"
                )),
            }
        }
    }
}

fn parse_review_providers(
    review: &Value,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, ReviewProvider> {
    let Some(providers) = review.pointer("/providers").and_then(Value::as_array) else {
        diagnostics.push("review /providers must be an array".to_string());
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    for (idx, provider) in providers.iter().enumerate() {
        let source = required_string(provider, "/source", diagnostics).unwrap_or_default();
        let mut platforms = BTreeMap::new();
        let Some(platform_rows) = provider.pointer("/platforms").and_then(Value::as_array) else {
            diagnostics.push(format!(
                "review /providers/{idx}/platforms must be an array"
            ));
            continue;
        };
        for (module_idx, row) in platform_rows.iter().enumerate() {
            let platform = required_string(row, "/platform", diagnostics).unwrap_or_default();
            let status = required_string(row, "/retrieval_status", diagnostics).unwrap_or_default();
            let key_id = required_string(row, "/signing_key_id", diagnostics).unwrap_or_default();
            if status != "signed" {
                diagnostics.push(format!(
                    "review /providers/{idx}/platforms/{module_idx}/retrieval_status must be \"signed\""
                ));
            }
            if !platform.is_empty() {
                platforms.insert(platform, key_id);
            }
        }
        if !source.is_empty() {
            out.insert(
                source.clone(),
                ReviewProvider {
                    local_name: required_string(provider, "/local_name", diagnostics)
                        .unwrap_or_default(),
                    source,
                    version: required_string(provider, "/version", diagnostics).unwrap_or_default(),
                    constraints: required_string(provider, "/constraints", diagnostics)
                        .unwrap_or_default(),
                    signing_status: required_string(provider, "/signing_status", diagnostics)
                        .unwrap_or_default(),
                    signing_key_id: required_string(provider, "/signing_key_id", diagnostics)
                        .unwrap_or_default(),
                    platforms,
                },
            );
        }
    }
    out
}

fn parse_required_providers_file(
    path: &Path,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, ProviderRequirement> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            diagnostics.push(format!(
                "unable to read providers file {}: {error}",
                path.display()
            ));
            return BTreeMap::new();
        }
    };
    let mut parsed = BTreeMap::new();
    let mut inside_required_providers = false;
    let mut required_depth = 0i32;
    let mut current_local: Option<String> = None;
    let mut current = ProviderRequirement::default();
    for raw_line in contents.lines() {
        let active = strip_hcl_line_comment(raw_line).trim();
        if active.is_empty() {
            continue;
        }
        if !inside_required_providers {
            if active.contains("required_providers") && active.contains('{') {
                inside_required_providers = true;
                required_depth = hcl_brace_delta(active);
            }
            continue;
        }
        if current_local.is_none() {
            if let Some(local) = provider_local_block_name(active) {
                current_local = Some(local.clone());
                current = ProviderRequirement {
                    local_name: local,
                    ..ProviderRequirement::default()
                };
            }
        } else {
            if let Some(source) = quoted_assignment(active, "source") {
                current.source = source;
            }
            if let Some(version) = quoted_assignment(active, "version") {
                current.constraint = version;
            }
            if active.starts_with('}')
                && let Some(_local) = current_local.take()
            {
                if !current.source.is_empty() {
                    parsed.insert(current.source.clone(), current.clone());
                }
                current = ProviderRequirement::default();
            }
        }
        required_depth += hcl_brace_delta(active);
        if required_depth <= 0 {
            inside_required_providers = false;
            current_local = None;
            current = ProviderRequirement::default();
        }
    }
    parsed
}

fn parse_lockfile(
    path: &Path,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, LockProviderBlock> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            diagnostics.push(format!(
                "unable to read lockfile {}: {error}",
                path.display()
            ));
            return BTreeMap::new();
        }
    };
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

fn is_slug(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_hex_key_id(value: &str) -> bool {
    matches!(value.len(), 16 | 40)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte))
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
        CloudIacProviderSignatureReviewArgs, parse_cloud_iac_provider_signature_review_args,
        validate_cloud_iac_provider_signature_review_gate,
    };

    #[test]
    fn parse_cloud_iac_provider_signature_review_rejects_unknown_flag() {
        let error = parse_cloud_iac_provider_signature_review_args(vec!["--bogus".into()])
            .expect_err("unknown flag should fail");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn cloud_iac_provider_signature_review_accepts_valid_fixture() {
        let temp = TempRepo::new("cloud-iac-provider-signature-review-valid");
        write_fixture(temp.path(), FixtureDrift::None);

        let report = validate_cloud_iac_provider_signature_review_gate(fixture_args(temp.path()))
            .expect("valid fixture should pass");

        assert_eq!(report.providers_checked, 2);
        assert_eq!(report.signer_keys_checked, 2);
        assert_eq!(report.platforms_checked, 3);
    }

    #[test]
    fn cloud_iac_provider_signature_review_rejects_unsigned_provider() {
        let temp = TempRepo::new("cloud-iac-provider-signature-review-unsigned");
        write_fixture(temp.path(), FixtureDrift::UnsignedProvider);

        let error = validate_cloud_iac_provider_signature_review_gate(fixture_args(temp.path()))
            .expect_err("unsigned provider should fail");

        assert!(error.contains("signing_status must be \"signed\""));
    }

    #[test]
    fn cloud_iac_provider_signature_review_rejects_missing_platform() {
        let temp = TempRepo::new("cloud-iac-provider-signature-review-platform");
        write_fixture(temp.path(), FixtureDrift::MissingPlatform);

        let error = validate_cloud_iac_provider_signature_review_gate(fixture_args(temp.path()))
            .expect_err("missing platform should fail");

        assert!(error.contains("must include signed retrieval evidence"));
    }

    #[test]
    fn cloud_iac_provider_signature_review_rejects_digest_drift() {
        let temp = TempRepo::new("cloud-iac-provider-signature-review-digest");
        write_fixture(temp.path(), FixtureDrift::DigestDrift);

        let error = validate_cloud_iac_provider_signature_review_gate(fixture_args(temp.path()))
            .expect_err("digest drift should fail");

        assert!(error.contains("lockfile_sha256 must match"));
    }

    #[test]
    fn cloud_iac_provider_signature_review_rejects_module_tree_lock_root() {
        let temp = TempRepo::new("cloud-iac-provider-signature-review-module-root");
        let module_lock_root = temp
            .path()
            .join("iac/tofu/modules/foundation-lock");
        fs::create_dir_all(&module_lock_root).expect("module lock root");
        let mut diagnostics = Vec::new();

        super::validate_lock_root(
            "iac/tofu/modules/foundation-lock",
            &module_lock_root,
            &mut diagnostics,
        );

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("outside reusable module tree"))
        );
    }

    fn fixture_args(repo_root: &Path) -> CloudIacProviderSignatureReviewArgs {
        CloudIacProviderSignatureReviewArgs {
            repo_root: repo_root.to_path_buf(),
            manifest: PathBuf::from("iac/manifest.json"),
            lock_root: PathBuf::from("iac/tofu/provider-locks/foundation"),
            review: PathBuf::from(
                "iac/tofu/provider-locks/foundation/provider-signature-review.json",
            ),
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum FixtureDrift {
        None,
        UnsignedProvider,
        MissingPlatform,
        DigestDrift,
    }

    fn write_fixture(root: &Path, drift: FixtureDrift) {
        let lock_root = root.join("iac/tofu/provider-locks/foundation");
        fs::create_dir_all(&lock_root).expect("lock root");
        fs::write(lock_root.join("providers.tofu"), fixture_providers()).expect("providers");
        fs::write(lock_root.join(".terraform.lock.hcl"), fixture_lockfile()).expect("lockfile");
        fs::write(
            lock_root.join("provider-signature-review.json"),
            fixture_review(&lock_root, drift),
        )
        .expect("review");
        fs::create_dir_all(root.join("iac")).expect("manifest dir");
        fs::write(
            root.join("iac/manifest.json"),
            fixture_manifest(&lock_root),
        )
        .expect("manifest");
    }

    fn fixture_providers() -> &'static str {
        r#"terraform {
  required_version = ">= 1.6"
  required_providers {
    aws = {
      source  = "registry.opentofu.org/hashicorp/aws"
      version = ">= 5.0.0"
    }
    cloudflare = {
      source  = "registry.opentofu.org/cloudflare/cloudflare"
      version = ">= 4.0.0"
    }
  }
}
"#
    }

    fn fixture_lockfile() -> &'static str {
        r#"provider "registry.opentofu.org/hashicorp/aws" {
  version     = "6.0.0"
  constraints = ">= 5.0.0"
  hashes = ["h1:aaa", "zh:111", "zh:222", "zh:333"]
}
provider "registry.opentofu.org/cloudflare/cloudflare" {
  version     = "5.0.0"
  constraints = ">= 4.0.0"
  hashes = ["h1:bbb", "zh:444", "zh:555", "zh:666"]
}
"#
    }

    fn fixture_review(lock_root: &Path, drift: FixtureDrift) -> String {
        let providers_hash = sha256_for_test(&lock_root.join("providers.tofu"));
        let lock_hash = if drift == FixtureDrift::DigestDrift {
            "bad".to_string()
        } else {
            sha256_for_test(&lock_root.join(".terraform.lock.hcl"))
        };
        let cloudflare_status = if drift == FixtureDrift::UnsignedProvider {
            "unsigned"
        } else {
            "signed"
        };
        let cloudflare_platforms = if drift == FixtureDrift::MissingPlatform {
            r#"[
        {"platform":"darwin_arm64","retrieval_status":"signed","signing_key_id":"C76001609EE3B136"},
        {"platform":"linux_amd64","retrieval_status":"signed","signing_key_id":"C76001609EE3B136"}
      ]"#
        } else {
            r#"[
        {"platform":"darwin_arm64","retrieval_status":"signed","signing_key_id":"C76001609EE3B136"},
        {"platform":"linux_amd64","retrieval_status":"signed","signing_key_id":"C76001609EE3B136"},
        {"platform":"linux_arm64","retrieval_status":"signed","signing_key_id":"C76001609EE3B136"}
      ]"#
        };
        format!(
            r#"{{
  "schema_version":"1.0",
  "review_id":"cloud-iac-provider-signature-review-local-inventory",
  "generated_by_changeset":"CS-CLOUD-IAC-PROVIDER-SIGNATURE-REVIEW-GATE-001",
  "authority":{{
    "lock_root":"iac/tofu/provider-locks/foundation",
    "providers_file":"iac/tofu/provider-locks/foundation/providers.tofu",
    "lockfile":"iac/tofu/provider-locks/foundation/.terraform.lock.hcl",
    "runtime_mode":"local-opentofu-provider-signature-review-gate",
    "official_sources_consulted":[
      "https://opentofu.org/docs/cli/commands/providers/lock/",
      "https://opentofu.org/docs/language/files/dependency-lock/",
      "https://opentofu.org/docs/cli/plugins/signing/",
      "https://slsa.dev/spec/v1.2/verification_summary"
    ]
  }},
  "policy":{{
    "status":"signed-provider-lock-reviewed-no-vsa",
    "signature_review_completed":true,
    "required_platforms":["darwin_arm64","linux_amd64","linux_arm64"],
    "provider_lockfile_required":true,
    "provider_installation_in_source_tree":false,
    "provider_provenance_verified":false,
    "verification_summary_attestation_emitted":false,
    "slsa_attestation_generated":false,
    "module_signing_executed":false,
    "tofu_plan_apply_executed":false
  }},
  "artifacts":{{"providers_file_sha256":"{providers_hash}","lockfile_sha256":"{lock_hash}"}},
  "providers":[
    {{"local_name":"aws","source":"registry.opentofu.org/hashicorp/aws","version":"6.0.0","constraints":">= 5.0.0","signing_status":"signed","signing_key_id":"0C0AF313E5FD9F80","platforms":[
      {{"platform":"darwin_arm64","retrieval_status":"signed","signing_key_id":"0C0AF313E5FD9F80"}},
      {{"platform":"linux_amd64","retrieval_status":"signed","signing_key_id":"0C0AF313E5FD9F80"}},
      {{"platform":"linux_arm64","retrieval_status":"signed","signing_key_id":"0C0AF313E5FD9F80"}}
    ]}},
    {{"local_name":"cloudflare","source":"registry.opentofu.org/cloudflare/cloudflare","version":"5.0.0","constraints":">= 4.0.0","signing_status":"{cloudflare_status}","signing_key_id":"C76001609EE3B136","platforms":{cloudflare_platforms}}}
  ]
}}
"#
        )
    }

    fn fixture_manifest(lock_root: &Path) -> String {
        let providers_hash = sha256_for_test(&lock_root.join("providers.tofu"));
        let lock_hash = sha256_for_test(&lock_root.join(".terraform.lock.hcl"));
        serde_json::to_string_pretty(&serde_json::json!({
            "foundation_non_claims": [
                "CS-CLOUD-IAC-PROVIDER-SIGNATURE-REVIEW-GATE-001 adds a local signer-key review gate only; no VSA/SLSA, module signing, provider installation, tofu plan/apply, or cloud runtime is claimed."
            ],
            "capabilities": [{
                "tier": "T1",
                "name": "cloud-iac-provider-signature-review-gate",
                "file": "crates/oya-dev-cli/src/cloud_iac_provider_signature_review_gate.rs",
                "risk_class": "high"
            }],
            "provider_signature_review_scope": {
                "lock_root": "iac/tofu/provider-locks/foundation",
                "providers_file": "iac/tofu/provider-locks/foundation/providers.tofu",
                "lockfile": "iac/tofu/provider-locks/foundation/.terraform.lock.hcl",
                "review": "iac/tofu/provider-locks/foundation/provider-signature-review.json",
                "status": "signed-provider-lock-reviewed-no-vsa",
                "runtime_mode": "local-opentofu-provider-signature-review-gate",
                "platforms": ["darwin_arm64", "linux_amd64", "linux_arm64"],
                "provider_sources": [
                    "registry.opentofu.org/cloudflare/cloudflare",
                    "registry.opentofu.org/hashicorp/aws"
                ],
                "provider_versions_selected": {
                    "registry.opentofu.org/cloudflare/cloudflare": "5.0.0",
                    "registry.opentofu.org/hashicorp/aws": "6.0.0"
                },
                "signing_key_ids": {
                    "registry.opentofu.org/cloudflare/cloudflare": "C76001609EE3B136",
                    "registry.opentofu.org/hashicorp/aws": "0C0AF313E5FD9F80"
                },
                "artifacts": {
                    "providers_file_sha256": providers_hash,
                    "lockfile_sha256": lock_hash
                },
                "official_sources_consulted": [
                    "https://opentofu.org/docs/cli/commands/providers/lock/",
                    "https://opentofu.org/docs/language/files/dependency-lock/",
                    "https://opentofu.org/docs/cli/plugins/signing/",
                    "https://slsa.dev/spec/v1.2/verification_summary"
                ],
                "coherence_guard": {
                    "changeset": "CS-CLOUD-IAC-PROVIDER-SIGNATURE-REVIEW-GATE-001",
                    "gate": "cloud-iac-provider-signature-review",
                    "gate_file": "crates/oya-dev-cli/src/cloud_iac_provider_signature_review_gate.rs",
                    "runtime_mode": "local-opentofu-provider-signature-review-gate"
                },
                "non_claims": [
                    "no provider provenance verification summary or VSA",
                    "no SLSA attestation generation",
                    "no module signing or Sigstore execution",
                    "no tofu test/plan/apply evidence",
                    "no cloud resource provisioning"
                ]
            }
        }))
        .expect("fixture manifest JSON")
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
