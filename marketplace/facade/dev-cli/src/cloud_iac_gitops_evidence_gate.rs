//! `oya gate validate cloud-iac-gitops-evidence` runner.
//!
//! This gate makes the Cloud IaC Argo CD Application templates a permanent
//! fail-closed local evidence surface. It parses the Cloud IaC manifest, scans
//! repo-local YAML templates as text, and intentionally performs no Argo CD API,
//! Kubernetes API, Git, provider, cosign, or OpenTofu calls.

use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

use crate::slash_path;

const DEFAULT_REPO_ROOT: &str = ".";
const DEFAULT_MANIFEST: &str = "iac/manifest.json";
const DEFAULT_TEMPLATES_ROOT: &str = "iac/iac";
const GATE_NAME: &str = "cloud-iac-gitops-evidence";
const GATE_FILE: &str = "crates/oya-dev-cli/src/cloud_iac_gitops_evidence_gate.rs";
const RUNTIME_MODE: &str = "local-filesystem-yaml-template-gate";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacGitOpsEvidenceValidateArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) templates_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacGitOpsEvidenceReport {
    pub(crate) manifest_path: String,
    pub(crate) templates_root: String,
    pub(crate) contexts_checked: usize,
    pub(crate) templates_checked: usize,
}

pub(crate) fn parse_cloud_iac_gitops_evidence_validate_args(
    args: Vec<String>,
) -> Result<CloudIacGitOpsEvidenceValidateArgs, String> {
    let mut parsed = CloudIacGitOpsEvidenceValidateArgs {
        repo_root: PathBuf::from(DEFAULT_REPO_ROOT),
        manifest: PathBuf::from(DEFAULT_MANIFEST),
        templates_root: PathBuf::from(DEFAULT_TEMPLATES_ROOT),
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
            "--templates-root" => {
                parsed.templates_root = take_path_arg(&mut args, "--templates-root")?;
            }
            other => {
                return Err(format!(
                    "cloud-iac-gitops-evidence: unknown flag {other:?}; usage: \
                     oya gate validate cloud-iac-gitops-evidence \
                     [--repo-root <.>] \
                     [--manifest <iac/manifest.json>] \
                     [--templates-root <iac/iac>]"
                ));
            }
        }
    }

    Ok(parsed)
}

fn take_path_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf, String> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("cloud-iac-gitops-evidence: {flag} requires a path argument"))
}

pub(crate) fn validate_cloud_iac_gitops_evidence_gate(
    args: CloudIacGitOpsEvidenceValidateArgs,
) -> Result<CloudIacGitOpsEvidenceReport, String> {
    let manifest_path = resolve_repo_path(&args.repo_root, &args.manifest);
    let templates_root_path = resolve_repo_path(&args.repo_root, &args.templates_root);
    let manifest_rel = repo_relative_argument(&args.repo_root, &args.manifest)?;
    let templates_root_rel = repo_relative_argument(&args.repo_root, &args.templates_root)?;

    let manifest = read_json(&manifest_path, "manifest")?;

    let mut diagnostics = Vec::new();
    require_manifest_capability(&manifest, &mut diagnostics);
    require_manifest_gate_guard(&manifest, &mut diagnostics);

    let manifest_templates_root = required_string(
        &manifest,
        "/gitops_evidence_scope/templates_root",
        &mut diagnostics,
    )
    .and_then(|raw| {
        normalize_repo_relative(
            &raw,
            "manifest /gitops_evidence_scope/templates_root",
            &mut diagnostics,
        )
    });
    if let Some(manifest_templates_root) = manifest_templates_root.as_deref()
        && manifest_templates_root != templates_root_rel
    {
        diagnostics.push(format!(
            "manifest /gitops_evidence_scope/templates_root must equal {templates_root_rel:?}; found {manifest_templates_root:?}"
        ));
    }

    let contexts = required_string_array(
        &manifest,
        "/gitops_evidence_scope/contexts",
        &mut diagnostics,
    )
    .unwrap_or_default();
    validate_manifest_application_kind(&manifest, &mut diagnostics);
    validate_manifest_context_summary(&manifest, &contexts, &mut diagnostics);
    validate_manifest_modeled_fields(&manifest, &mut diagnostics);
    validate_manifest_metadata_only_posture(&manifest, &mut diagnostics);
    validate_manifest_non_claims(&manifest, &mut diagnostics);

    let actual_contexts = discover_template_contexts(&templates_root_path, &mut diagnostics);
    if !contexts.is_empty() && contexts != actual_contexts {
        diagnostics.push(format!(
            "manifest /gitops_evidence_scope/contexts must exactly match template contexts {:?}; found {:?}",
            actual_contexts, contexts
        ));
    }

    let mut templates_checked = 0usize;
    for context in &contexts {
        if !is_context_slug(context) {
            diagnostics.push(format!(
                "gitops context {context:?} must be lowercase/digit/hyphen"
            ));
            continue;
        }
        let template_rel = format!("{templates_root_rel}/{context}/argocd/apps/template.yaml");
        let template_path = args.repo_root.join(&template_rel);
        templates_checked += 1;
        validate_application_template(context, &template_rel, &template_path, &mut diagnostics);
    }

    if diagnostics.is_empty() {
        Ok(CloudIacGitOpsEvidenceReport {
            manifest_path: manifest_rel,
            templates_root: templates_root_rel,
            contexts_checked: contexts.len(),
            templates_checked,
        })
    } else {
        Err(format!(
            "cloud-iac-gitops-evidence validation failed:\n- {}",
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
                "cloud-iac-gitops-evidence: unable to canonicalize repo root {}: {error}",
                repo_root.display()
            )
        })?;
        let path = fs::canonicalize(path).map_err(|error| {
            format!(
                "cloud-iac-gitops-evidence: unable to canonicalize path {}: {error}",
                path.display()
            )
        })?;
        let relative = path.strip_prefix(&repo_root).map_err(|_| {
            format!(
                "cloud-iac-gitops-evidence: path {} is outside repo root {}",
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
            "cloud-iac-gitops-evidence: unable to read {label} {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        format!(
            "cloud-iac-gitops-evidence: unable to parse {label} JSON {}: {error}",
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
        capability.get("name").and_then(Value::as_str) == Some("cloud-iac-gitops-evidence-gate")
            && capability.get("file").and_then(Value::as_str) == Some(GATE_FILE)
    });
    if !has_gate_capability {
        diagnostics.push(format!(
            "manifest capabilities must declare cloud-iac-gitops-evidence-gate backed by {GATE_FILE}"
        ));
    }
}

fn require_manifest_gate_guard(manifest: &Value, diagnostics: &mut Vec<String>) {
    let gate = required_string(
        manifest,
        "/gitops_evidence_scope/coherence_guard/gate",
        diagnostics,
    );
    if gate.as_deref() != Some(GATE_NAME) {
        diagnostics.push(format!(
            "manifest /gitops_evidence_scope/coherence_guard/gate must be {GATE_NAME:?}"
        ));
    }
    let mode = required_string(
        manifest,
        "/gitops_evidence_scope/coherence_guard/runtime_mode",
        diagnostics,
    );
    if mode.as_deref() != Some(RUNTIME_MODE) {
        diagnostics.push(format!(
            "manifest /gitops_evidence_scope/coherence_guard/runtime_mode must be {RUNTIME_MODE:?}"
        ));
    }
}

fn validate_manifest_context_summary(
    manifest: &Value,
    contexts: &[String],
    diagnostics: &mut Vec<String>,
) {
    match manifest
        .pointer("/gitops_evidence_scope/template_count")
        .and_then(Value::as_u64)
    {
        Some(count) if count == contexts.len() as u64 => {}
        Some(count) => diagnostics.push(format!(
            "manifest /gitops_evidence_scope/template_count must equal context count {}; found {count}",
            contexts.len()
        )),
        None => diagnostics.push(
            "manifest /gitops_evidence_scope/template_count must be an unsigned integer"
                .to_string(),
        ),
    }
    if contexts.is_empty() {
        diagnostics.push("manifest /gitops_evidence_scope/contexts must not be empty".to_string());
    }
}

fn validate_manifest_application_kind(manifest: &Value, diagnostics: &mut Vec<String>) {
    let application_kind = required_string(
        manifest,
        "/gitops_evidence_scope/application_kind",
        diagnostics,
    );
    if application_kind.as_deref() != Some("argoproj.io/v1alpha1/Application") {
        diagnostics.push(
            "manifest /gitops_evidence_scope/application_kind must be \
             \"argoproj.io/v1alpha1/Application\""
                .to_string(),
        );
    }
}

fn validate_manifest_modeled_fields(manifest: &Value, diagnostics: &mut Vec<String>) {
    let Some(modeled_fields) = required_string_array(
        manifest,
        "/gitops_evidence_scope/application_fields_modeled",
        diagnostics,
    ) else {
        return;
    };

    for required in [
        "apiVersion",
        "kind",
        "metadata.namespace",
        "metadata.labels.oyatie.com/adr",
        "metadata.labels.oyatie.com/context",
        "metadata.annotations.cosign.sigstore.dev/required",
        "metadata.annotations.oyatie.com/audit-chain-event",
        "metadata.annotations.oyatie.com/fail-open",
        "spec.project",
        "spec.source.repoURL",
        "spec.source.targetRevision",
        "spec.source.path",
        "spec.source.helm.parameters.image.digest",
        "spec.source.helm.parameters.image.cosign.required",
        "spec.destination.server",
        "spec.destination.namespace",
        "spec.syncPolicy.automated.prune",
        "spec.syncPolicy.automated.selfHeal",
        "spec.syncPolicy.syncOptions",
    ] {
        if !modeled_fields.iter().any(|field| field == required) {
            diagnostics.push(format!(
                "manifest /gitops_evidence_scope/application_fields_modeled must include {required:?}"
            ));
        }
    }
}

fn validate_manifest_metadata_only_posture(manifest: &Value, diagnostics: &mut Vec<String>) {
    for (pointer, expected) in [
        (
            "/gitops_evidence_scope/metadata_only_posture/repo_url",
            "placeholder-only: {{repo_url}}",
        ),
        (
            "/gitops_evidence_scope/metadata_only_posture/target_revision",
            "placeholder-only: {{target_revision}}",
        ),
        (
            "/gitops_evidence_scope/metadata_only_posture/cluster_api_server",
            "placeholder-only: {{cluster_api_server}}",
        ),
        (
            "/gitops_evidence_scope/metadata_only_posture/tenant_namespace",
            "placeholder-only: {{tenant_namespace}}",
        ),
    ] {
        let actual = required_string(manifest, pointer, diagnostics);
        if actual.as_deref() != Some(expected) {
            diagnostics.push(format!("{pointer} must be {expected:?}"));
        }
    }

    let credential_storage = required_string(
        manifest,
        "/gitops_evidence_scope/metadata_only_posture/credential_storage",
        diagnostics,
    );
    if let Some(credential_storage) = credential_storage
        && contains_secret_like_marker(&credential_storage)
    {
        diagnostics.push(
            "manifest /gitops_evidence_scope/metadata_only_posture/credential_storage must not contain credential-like material"
                .to_string(),
        );
    }
}

fn validate_manifest_non_claims(manifest: &Value, diagnostics: &mut Vec<String>) {
    let Some(non_claims) = manifest
        .pointer("/gitops_evidence_scope/non_claims")
        .and_then(Value::as_array)
    else {
        diagnostics.push("manifest /gitops_evidence_scope/non_claims must be an array".to_string());
        return;
    };
    if non_claims.is_empty() {
        diagnostics.push(
            "manifest /gitops_evidence_scope/non_claims must be a non-empty array".to_string(),
        );
    }
    let joined = non_claims
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join("\n")
        .to_ascii_lowercase();
    for required in ["no argocd api integration", "no repository credentials"] {
        if !joined.contains(required) {
            diagnostics.push(format!(
                "manifest /gitops_evidence_scope/non_claims must include {required:?}"
            ));
        }
    }
}

fn discover_template_contexts(templates_root: &Path, diagnostics: &mut Vec<String>) -> Vec<String> {
    let mut contexts = Vec::new();
    let Ok(entries) = fs::read_dir(templates_root) else {
        diagnostics.push(format!(
            "templates root is not readable: {}",
            templates_root.display()
        ));
        return contexts;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if path.join("argocd/apps/template.yaml").is_file() {
            contexts.push(name.to_string());
        }
    }
    contexts.sort();
    if contexts.is_empty() {
        diagnostics.push(format!(
            "templates root contains no */argocd/apps/template.yaml files: {}",
            templates_root.display()
        ));
    }
    contexts
}

fn validate_application_template(
    context: &str,
    template_rel: &str,
    template_path: &Path,
    diagnostics: &mut Vec<String>,
) {
    let contents = match fs::read_to_string(template_path) {
        Ok(contents) => contents,
        Err(error) => {
            diagnostics.push(format!(
                "{template_rel}: unable to read Argo CD Application template: {error}"
            ));
            return;
        }
    };

    if contains_secret_like_marker(&contents) {
        diagnostics.push(format!(
            "{template_rel}: contains credential-like material marker"
        ));
    }
    validate_repo_url_placeholders(template_rel, &contents, diagnostics);
    validate_helm_parameter_value_pair(
        template_rel,
        &contents,
        "image.digest",
        "value: \"{{signed_image_digest}}\"",
        diagnostics,
    );
    validate_helm_parameter_value_pair(
        template_rel,
        &contents,
        "image.cosign.required",
        "value: \"true\"",
        diagnostics,
    );

    for required_line in required_template_lines(context) {
        if !contains_trimmed_line(&contents, &required_line) {
            diagnostics.push(format!(
                "{template_rel}: missing required Argo CD Application line {required_line:?}"
            ));
        }
    }
}

fn required_template_lines(context: &str) -> Vec<String> {
    vec![
        "apiVersion: argoproj.io/v1alpha1".to_string(),
        "kind: Application".to_string(),
        "namespace: oya-cd-argocd".to_string(),
        "app.kubernetes.io/part-of: oya-ci-cd-substrate".to_string(),
        "oyatie.com/adr: ADR-0349".to_string(),
        "oyatie.com/image-promotion: ADR-0181".to_string(),
        format!("oyatie.com/context: \"{context}\""),
        "cosign.sigstore.dev/required: \"true\"".to_string(),
        "oyatie.com/audit-chain-event: argocd-application-sync".to_string(),
        "oyatie.com/fail-open: \"false\"".to_string(),
        format!("project: \"oya-{context}-tenants\""),
        "repoURL: \"{{repo_url}}\"".to_string(),
        "targetRevision: \"{{target_revision}}\"".to_string(),
        "path: \"microservices/{{microservice}}/iac/k8s/helm\"".to_string(),
        "- name: image.digest".to_string(),
        "value: \"{{signed_image_digest}}\"".to_string(),
        "- name: image.cosign.required".to_string(),
        "value: \"true\"".to_string(),
        "server: \"{{cluster_api_server}}\"".to_string(),
        "namespace: \"{{tenant_namespace}}\"".to_string(),
        "prune: true".to_string(),
        "selfHeal: true".to_string(),
        "- CreateNamespace=true".to_string(),
        "- ServerSideApply=true".to_string(),
    ]
}

fn validate_helm_parameter_value_pair(
    template_rel: &str,
    contents: &str,
    parameter_name: &str,
    expected_value_line: &str,
    diagnostics: &mut Vec<String>,
) {
    let expected_name_line = format!("- name: {parameter_name}");
    let lines = contents.lines().map(str::trim).collect::<Vec<_>>();
    let mut saw_name = false;
    for (idx, line) in lines.iter().enumerate() {
        if *line != expected_name_line {
            continue;
        }
        saw_name = true;
        let next_value = lines
            .iter()
            .skip(idx + 1)
            .find(|candidate| !candidate.is_empty());
        if next_value == Some(&expected_value_line) {
            return;
        }
    }

    if saw_name {
        diagnostics.push(format!(
            "{template_rel}: Helm parameter {parameter_name:?} must be immediately paired with {expected_value_line:?}"
        ));
    } else {
        diagnostics.push(format!(
            "{template_rel}: missing Helm parameter {parameter_name:?}"
        ));
    }
}

fn contains_trimmed_line(contents: &str, needle: &str) -> bool {
    contents.lines().any(|line| line.trim() == needle)
}

fn validate_repo_url_placeholders(
    template_rel: &str,
    contents: &str,
    diagnostics: &mut Vec<String>,
) {
    for (idx, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("repoURL:") && trimmed != "repoURL: \"{{repo_url}}\"" {
            diagnostics.push(format!(
                "{template_rel}:{} spec.source.repoURL must remain the metadata-only {{repo_url}} placeholder",
                idx + 1
            ));
        }
        if trimmed.starts_with("targetRevision:")
            && trimmed != "targetRevision: \"{{target_revision}}\""
        {
            diagnostics.push(format!(
                "{template_rel}:{} spec.source.targetRevision must remain the metadata-only {{target_revision}} placeholder",
                idx + 1
            ));
        }
    }
}

fn contains_secret_like_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "password:",
        "password=",
        "token:",
        "token=",
        "private_key",
        "private-key",
        "sshprivatekey",
        "clientsecret",
        "kubeconfig",
        "-----begin",
        "bearer ",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn is_context_slug(value: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parse_cloud_iac_gitops_evidence_defaults_to_live_paths() {
        let parsed = parse_cloud_iac_gitops_evidence_validate_args(Vec::new()).expect("defaults");
        assert_eq!(parsed.repo_root, PathBuf::from(DEFAULT_REPO_ROOT));
        assert_eq!(parsed.manifest, PathBuf::from(DEFAULT_MANIFEST));
        assert_eq!(parsed.templates_root, PathBuf::from(DEFAULT_TEMPLATES_ROOT));
    }

    #[test]
    fn parse_cloud_iac_gitops_evidence_rejects_unknown_flag() {
        let error = parse_cloud_iac_gitops_evidence_validate_args(vec!["--bogus".into()])
            .expect_err("unknown flag rejected");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn cloud_iac_gitops_evidence_gate_accepts_coherent_fixture() {
        let temp = TempRepo::new("cloud-iac-gitops-valid");
        write_fixture(
            temp.path(),
            &["aws-guest", "oci-guest"],
            TemplateDrift::None,
        );

        let report = validate_cloud_iac_gitops_evidence_gate(fixture_args(temp.path()))
            .expect("coherent fixture passes");

        assert_eq!(report.contexts_checked, 2);
        assert_eq!(report.templates_checked, 2);
    }

    #[test]
    fn cloud_iac_gitops_evidence_gate_rejects_manifest_context_drift() {
        let temp = TempRepo::new("cloud-iac-gitops-context-drift");
        write_fixture(temp.path(), &["aws-guest"], TemplateDrift::None);
        fs::create_dir_all(
            temp.path()
                .join("iac/iac/oci-guest/argocd/apps"),
        )
        .expect("extra context dir");
        fs::write(
            temp.path()
                .join("iac/iac/oci-guest/argocd/apps/template.yaml"),
            fixture_template("oci-guest", TemplateDrift::None),
        )
        .expect("extra template written");

        let error = validate_cloud_iac_gitops_evidence_gate(fixture_args(temp.path()))
            .expect_err("context drift fails");

        assert!(error.contains("must exactly match template contexts"));
    }

    #[test]
    fn cloud_iac_gitops_evidence_gate_rejects_manifest_application_kind_drift() {
        let temp = TempRepo::new("cloud-iac-gitops-kind-drift");
        write_fixture(temp.path(), &["aws-guest"], TemplateDrift::None);
        let manifest_path = temp.path().join(DEFAULT_MANIFEST);
        let mut manifest: Value =
            serde_json::from_str(&fs::read_to_string(&manifest_path).expect("manifest readable"))
                .expect("manifest parses");
        manifest["gitops_evidence_scope"]["application_kind"] =
            Value::String("argoproj.io/v1alpha1/NotApplication".to_string());
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).expect("manifest serializes"),
        )
        .expect("manifest written");

        let error = validate_cloud_iac_gitops_evidence_gate(fixture_args(temp.path()))
            .expect_err("application kind drift fails");

        assert!(error.contains("application_kind"));
    }

    #[test]
    fn cloud_iac_gitops_evidence_gate_rejects_non_placeholder_repo_url() {
        let temp = TempRepo::new("cloud-iac-gitops-repo-url");
        write_fixture(temp.path(), &["aws-guest"], TemplateDrift::ConcreteRepoUrl);

        let error = validate_cloud_iac_gitops_evidence_gate(fixture_args(temp.path()))
            .expect_err("repo URL overclaim fails");

        assert!(error.contains("repoURL must remain"));
    }

    #[test]
    fn cloud_iac_gitops_evidence_gate_rejects_missing_sync_policy() {
        let temp = TempRepo::new("cloud-iac-gitops-sync-drift");
        write_fixture(
            temp.path(),
            &["aws-guest"],
            TemplateDrift::MissingServerSideApply,
        );

        let error = validate_cloud_iac_gitops_evidence_gate(fixture_args(temp.path()))
            .expect_err("sync option drift fails");

        assert!(error.contains("ServerSideApply"));
    }

    #[test]
    fn cloud_iac_gitops_evidence_gate_rejects_credential_markers() {
        let temp = TempRepo::new("cloud-iac-gitops-secret");
        write_fixture(temp.path(), &["aws-guest"], TemplateDrift::CredentialMarker);

        let error = validate_cloud_iac_gitops_evidence_gate(fixture_args(temp.path()))
            .expect_err("credential marker fails");

        assert!(error.contains("credential-like material"));
    }

    #[test]
    fn cloud_iac_gitops_evidence_gate_rejects_missing_signed_image_parameter() {
        let temp = TempRepo::new("cloud-iac-gitops-signed-image-param");
        write_fixture(
            temp.path(),
            &["aws-guest"],
            TemplateDrift::MissingSignedImageDigestParameter,
        );

        let error = validate_cloud_iac_gitops_evidence_gate(fixture_args(temp.path()))
            .expect_err("missing signed image digest parameter fails");

        assert!(error.contains("image.digest"));
    }

    #[test]
    fn cloud_iac_gitops_evidence_gate_rejects_signed_image_parameter_value_pair_drift() {
        let temp = TempRepo::new("cloud-iac-gitops-signed-image-param-pair");
        write_fixture(
            temp.path(),
            &["aws-guest"],
            TemplateDrift::SignedImageDigestValuePairDrift,
        );

        let error = validate_cloud_iac_gitops_evidence_gate(fixture_args(temp.path()))
            .expect_err("signed image digest parameter value pair drift fails");

        assert!(error.contains("image.digest"));
        assert!(error.contains("{{signed_image_digest}}"));
    }

    fn fixture_args(root: &Path) -> CloudIacGitOpsEvidenceValidateArgs {
        CloudIacGitOpsEvidenceValidateArgs {
            repo_root: root.to_path_buf(),
            manifest: PathBuf::from(DEFAULT_MANIFEST),
            templates_root: PathBuf::from(DEFAULT_TEMPLATES_ROOT),
        }
    }

    fn fixture_manifest(contexts: &[&str]) -> Value {
        serde_json::json!({
            "capabilities": [
                {
                    "tier": "T1",
                    "name": "cloud-iac-gitops-evidence-gate",
                    "file": GATE_FILE,
                    "risk_class": "high"
                }
            ],
            "gitops_evidence_scope": {
                "templates_root": DEFAULT_TEMPLATES_ROOT,
                "template_count": contexts.len(),
                "contexts": contexts,
                "application_kind": "argoproj.io/v1alpha1/Application",
                "application_fields_modeled": [
                    "apiVersion",
                    "kind",
                    "metadata.namespace",
                    "metadata.labels.oyatie.com/adr",
                    "metadata.labels.oyatie.com/context",
                    "metadata.annotations.cosign.sigstore.dev/required",
                    "metadata.annotations.oyatie.com/audit-chain-event",
                    "metadata.annotations.oyatie.com/fail-open",
                    "spec.project",
                    "spec.source.repoURL",
                    "spec.source.targetRevision",
                    "spec.source.path",
                    "spec.source.helm.parameters.image.digest",
                    "spec.source.helm.parameters.image.cosign.required",
                    "spec.destination.server",
                    "spec.destination.namespace",
                    "spec.syncPolicy.automated.prune",
                    "spec.syncPolicy.automated.selfHeal",
                    "spec.syncPolicy.syncOptions"
                ],
                "metadata_only_posture": {
                    "repo_url": "placeholder-only: {{repo_url}}",
                    "target_revision": "placeholder-only: {{target_revision}}",
                    "cluster_api_server": "placeholder-only: {{cluster_api_server}}",
                    "tenant_namespace": "placeholder-only: {{tenant_namespace}}",
                    "credential_storage": "forbidden in templates; repository credentials remain external SecretReference/OpenBao metadata only"
                },
                "non_claims": [
                    "no ArgoCD API integration",
                    "no repository credentials"
                ],
                "coherence_guard": {
                    "gate": GATE_NAME,
                    "runtime_mode": RUNTIME_MODE
                }
            }
        })
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum TemplateDrift {
        None,
        ConcreteRepoUrl,
        MissingServerSideApply,
        CredentialMarker,
        MissingSignedImageDigestParameter,
        SignedImageDigestValuePairDrift,
    }

    fn write_fixture(root: &Path, contexts: &[&str], drift: TemplateDrift) {
        fs::create_dir_all(root.join("iac")).expect("manifest dir");
        fs::write(
            root.join(DEFAULT_MANIFEST),
            serde_json::to_string_pretty(&fixture_manifest(contexts)).expect("manifest serializes"),
        )
        .expect("manifest written");
        for context in contexts {
            let dir = root.join(format!("iac/iac/{context}/argocd/apps"));
            fs::create_dir_all(&dir).expect("template dir");
            fs::write(dir.join("template.yaml"), fixture_template(context, drift))
                .expect("template written");
        }
    }

    fn fixture_template(context: &str, drift: TemplateDrift) -> String {
        let repo_url = if matches!(drift, TemplateDrift::ConcreteRepoUrl) {
            "https://token:secret@example.invalid/repo.git"
        } else {
            "{{repo_url}}"
        };
        let sync_options = if matches!(drift, TemplateDrift::MissingServerSideApply) {
            "      - CreateNamespace=true\n".to_string()
        } else {
            "      - CreateNamespace=true\n      - ServerSideApply=true\n".to_string()
        };
        let credential_line = if matches!(drift, TemplateDrift::CredentialMarker) {
            "    password: \"do-not-store\"\n"
        } else {
            ""
        };
        let helm_parameters = if matches!(drift, TemplateDrift::MissingSignedImageDigestParameter) {
            r#"    helm:
      parameters:
        - name: image.cosign.required
          value: "true"
"#
        } else if matches!(drift, TemplateDrift::SignedImageDigestValuePairDrift) {
            r#"    helm:
      parameters:
        - name: image.digest
          value: "sha256:drift"
        - name: image.cosign.required
          value: "{{signed_image_digest}}"
        - name: image.other
          value: "true"
"#
        } else {
            r#"    helm:
      parameters:
        - name: image.digest
          value: "{{signed_image_digest}}"
        - name: image.cosign.required
          value: "true"
"#
        };
        format!(
            r#"apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: "{{{{microservice}}}}-{{{{cluster_id}}}}"
  namespace: oya-cd-argocd
  labels:
    app.kubernetes.io/part-of: oya-ci-cd-substrate
    oyatie.com/adr: ADR-0349
    oyatie.com/image-promotion: ADR-0181
    oyatie.com/context: "{context}"
  annotations:
    cosign.sigstore.dev/required: "true"
    oyatie.com/audit-chain-event: argocd-application-sync
    oyatie.com/fail-open: "false"
{credential_line}spec:
  project: "oya-{context}-tenants"
  source:
    repoURL: "{repo_url}"
    targetRevision: "{{{{target_revision}}}}"
    path: "microservices/{{{{microservice}}}}/iac/k8s/helm"
{helm_parameters}  destination:
    server: "{{{{cluster_api_server}}}}"
    namespace: "{{{{tenant_namespace}}}}"
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
{sync_options}  ignoreDifferences: []
"#
        )
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
