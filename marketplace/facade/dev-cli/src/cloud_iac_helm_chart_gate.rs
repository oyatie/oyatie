//! `oya gate validate cloud-iac-helm-chart-signed-image-wiring` runner.
//!
//! This is intentionally a local filesystem wiring gate. It proves that the
//! repo-local Cloud IaC Helm chart preserves signed-image render inputs that
//! Argo CD parameter gates pass into the chart; it does not render Helm, call
//! Argo CD, call Kubernetes, execute cosign, or prove admission policy.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

const GATE_NAME: &str = "cloud-iac-helm-chart-signed-image-wiring";
const GATE_FILE: &str = "marketplace/facade/dev-cli/src/cloud_iac_helm_chart_gate.rs";
const DEFAULT_MANIFEST: &str = "iac/manifest.json";
const DEFAULT_CHART_ROOT: &str = "iac/iac/k8s/helm";
const RUNTIME_MODE: &str = "local-filesystem-helm-chart-wiring-gate-no-helm-render";

const CHART_FILES: &[&str] = &[
    "Chart.yaml",
    "values.yaml",
    "templates/deployment.yaml",
    "templates/configmap.yaml",
];

const CHART_REQUIRED_LINES: &[(&str, &str)] = &[
    ("Chart.yaml", "apiVersion: v2"),
    ("Chart.yaml", "type: application"),
    (
        "Chart.yaml",
        "oyatie.com/adr-0181: signed-image-promotion-required",
    ),
    (
        "Chart.yaml",
        "oyatie.com/adr-0349: argocd-application-source",
    ),
];

const VALUES_REQUIRED_LINES: &[(&str, &str)] = &[
    ("values.yaml", "required: true"),
    ("values.yaml", "policy: ADR-0181"),
    ("values.yaml", "imagePromotionPolicy: cosign-required"),
];

const DEPLOYMENT_REQUIRED_LINES: &[(&str, &str)] = &[
    (
        "templates/deployment.yaml",
        "$imageDigest := .Values.image.digest | default \"\"",
    ),
    (
        "templates/deployment.yaml",
        "regexMatch \"^sha256:[0-9a-f]{64}$\" $imageDigest",
    ),
    (
        "templates/deployment.yaml",
        "regexMatch \"[1-9a-f]\" (trimPrefix \"sha256:\" $imageDigest)",
    ),
    (
        "templates/deployment.yaml",
        "fail \"image.digest must be set to a real non-zero sha256 digest when image.cosign.required=true\"",
    ),
    ("templates/deployment.yaml", "@{{ $imageDigest }}"),
    ("templates/deployment.yaml", ".Values.image.cosign.required"),
    (
        "templates/deployment.yaml",
        "cosign.sigstore.dev/required: {{ .Values.image.cosign.required | quote }}",
    ),
    (
        "templates/deployment.yaml",
        "oyatie.com/image-digest: {{ $imageDigest | quote }}",
    ),
    (
        "templates/deployment.yaml",
        "oyatie.com/image-promotion-policy: {{ .Values.image.cosign.policy | quote }}",
    ),
    (
        "templates/deployment.yaml",
        "allowPrivilegeEscalation: false",
    ),
    ("templates/deployment.yaml", "readOnlyRootFilesystem: true"),
    ("templates/deployment.yaml", "seccompProfile:"),
    ("templates/deployment.yaml", "drop:"),
    ("templates/deployment.yaml", "- ALL"),
];

const CONFIGMAP_REQUIRED_LINES: &[(&str, &str)] = &[
    (
        "templates/configmap.yaml",
        "image_signature_policy: {{ .Values.config.imagePromotionPolicy | quote }}",
    ),
    (
        "templates/configmap.yaml",
        "cosign_required: {{ .Values.image.cosign.required | quote }}",
    ),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacHelmChartArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) chart_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacHelmChartReport {
    pub(crate) manifest_path: String,
    pub(crate) chart_root_path: String,
    pub(crate) files_checked: usize,
    pub(crate) required_lines_checked: usize,
}

pub(crate) fn parse_cloud_iac_helm_chart_args(
    args: Vec<String>,
) -> Result<CloudIacHelmChartArgs, String> {
    let mut parsed = CloudIacHelmChartArgs {
        repo_root: PathBuf::from("."),
        manifest: PathBuf::from(DEFAULT_MANIFEST),
        chart_root: PathBuf::from(DEFAULT_CHART_ROOT),
    };

    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => parsed.repo_root = take_path("--repo-root", &mut iter)?,
            "--manifest" => parsed.manifest = take_path("--manifest", &mut iter)?,
            "--chart-root" => parsed.chart_root = take_path("--chart-root", &mut iter)?,
            other => {
                return Err(format!(
                    "{GATE_NAME}: unknown flag {other:?}; usage: \
                     oya gate validate {GATE_NAME} \
                     [--repo-root <.>] [--manifest <{DEFAULT_MANIFEST}>] \
                     [--chart-root <{DEFAULT_CHART_ROOT}>]"
                ));
            }
        }
    }

    Ok(parsed)
}

fn take_path(flag: &str, iter: &mut impl Iterator<Item = String>) -> Result<PathBuf, String> {
    iter.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("{GATE_NAME}: {flag} requires a path argument"))
}

pub(crate) fn validate_cloud_iac_helm_chart_gate(
    args: CloudIacHelmChartArgs,
) -> Result<CloudIacHelmChartReport, String> {
    let repo_root = canonicalize_root(&args.repo_root)?;
    let manifest_path = canonicalize_under_repo(&repo_root, &args.manifest, "manifest")?;
    let chart_root = canonicalize_under_repo(&repo_root, &args.chart_root, "chart root")?;

    let mut issues = Vec::new();
    let manifest = read_json(&manifest_path, "manifest", &mut issues);
    if let Some(manifest) = manifest.as_ref() {
        validate_manifest_scope(manifest, &args.chart_root, &mut issues);
    }

    let mut files_checked = 0usize;
    let mut required_lines_checked = 0usize;
    for relative_file in CHART_FILES {
        let file_path =
            canonicalize_under_repo(&repo_root, &chart_root.join(relative_file), relative_file)?;
        let text = read_text(&file_path, relative_file, &mut issues);
        files_checked += 1;
        if let Some(text) = text.as_ref() {
            reject_secret_like_markers(relative_file, text, &mut issues);
        }
    }

    let chart_yaml = read_chart_file(&chart_root, "Chart.yaml", &mut issues);
    let values_yaml = read_chart_file(&chart_root, "values.yaml", &mut issues);
    let deployment_yaml = read_chart_file(&chart_root, "templates/deployment.yaml", &mut issues);
    let configmap_yaml = read_chart_file(&chart_root, "templates/configmap.yaml", &mut issues);

    required_lines_checked +=
        require_lines(chart_yaml.as_deref(), CHART_REQUIRED_LINES, &mut issues);
    required_lines_checked += require_values_yaml(values_yaml.as_deref(), &mut issues);
    required_lines_checked +=
        require_lines(values_yaml.as_deref(), VALUES_REQUIRED_LINES, &mut issues);
    required_lines_checked += require_lines(
        deployment_yaml.as_deref(),
        DEPLOYMENT_REQUIRED_LINES,
        &mut issues,
    );
    required_lines_checked += require_lines(
        configmap_yaml.as_deref(),
        CONFIGMAP_REQUIRED_LINES,
        &mut issues,
    );

    if !issues.is_empty() {
        return Err(format!(
            "{GATE_NAME} validation failed:\n- {}",
            issues.join("\n- ")
        ));
    }

    Ok(CloudIacHelmChartReport {
        manifest_path: slash_path(&args.manifest),
        chart_root_path: slash_path(&args.chart_root),
        files_checked,
        required_lines_checked,
    })
}

fn canonicalize_root(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize().map_err(|error| {
        format!(
            "{GATE_NAME}: unable to canonicalize repo root {}: {error}",
            slash_path(path)
        )
    })
}

fn canonicalize_under_repo(repo_root: &Path, path: &Path, label: &str) -> Result<PathBuf, String> {
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    };
    let canonical = candidate.canonicalize().map_err(|error| {
        format!(
            "{GATE_NAME}: unable to canonicalize {label} {}: {error}",
            slash_path(&candidate)
        )
    })?;
    if !canonical.starts_with(repo_root) {
        return Err(format!(
            "{GATE_NAME}: {label} {} is outside repo root {}",
            slash_path(&canonical),
            slash_path(repo_root)
        ));
    }
    Ok(canonical)
}

fn read_json(path: &Path, label: &str, issues: &mut Vec<String>) -> Option<Value> {
    let text = read_text(path, label, issues)?;
    match serde_json::from_str::<Value>(&text) {
        Ok(value) => Some(value),
        Err(error) => {
            issues.push(format!(
                "{label} {} is not valid JSON: {error}",
                slash_path(path)
            ));
            None
        }
    }
}

fn read_text(path: &Path, label: &str, issues: &mut Vec<String>) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(text) => Some(text),
        Err(error) => {
            issues.push(format!(
                "unable to read {label} {}: {error}",
                slash_path(path)
            ));
            None
        }
    }
}

fn read_chart_file(
    chart_root: &Path,
    relative_file: &str,
    issues: &mut Vec<String>,
) -> Option<String> {
    read_text(&chart_root.join(relative_file), relative_file, issues)
}

fn validate_manifest_scope(manifest: &Value, chart_root: &Path, issues: &mut Vec<String>) {
    let capability_ok = manifest
        .get("capabilities")
        .and_then(Value::as_array)
        .map(|capabilities| {
            capabilities.iter().any(|capability| {
                capability.get("name").and_then(Value::as_str)
                    == Some("cloud-iac-helm-chart-signed-image-wiring-gate")
                    && capability.get("file").and_then(Value::as_str) == Some(GATE_FILE)
            })
        })
        .unwrap_or(false);
    if !capability_ok {
        issues.push(format!(
            "manifest capabilities must declare cloud-iac-helm-chart-signed-image-wiring-gate backed by {GATE_FILE}"
        ));
    }

    let Some(scope) = manifest.get("helm_chart_signed_image_wiring_scope") else {
        issues.push("manifest must declare helm_chart_signed_image_wiring_scope".to_string());
        return;
    };

    let expected_chart_root = slash_path(chart_root);
    if scope.get("chart_root").and_then(Value::as_str) != Some(expected_chart_root.as_str()) {
        issues.push(format!(
            "manifest helm_chart_signed_image_wiring_scope.chart_root must be {expected_chart_root}"
        ));
    }
    if scope.get("runtime_mode").and_then(Value::as_str) != Some(RUNTIME_MODE) {
        issues.push(format!(
            "manifest helm_chart_signed_image_wiring_scope.runtime_mode must be {RUNTIME_MODE}"
        ));
    }

    for file in CHART_FILES {
        if !json_string_array_contains(scope.get("chart_files_checked"), file) {
            issues.push(format!(
                "manifest helm_chart_signed_image_wiring_scope.chart_files_checked must include {file}"
            ));
        }
    }

    for required_value in [
        "image.digest",
        "image.cosign.required",
        "image.cosign.policy",
        "config.imagePromotionPolicy",
    ] {
        if !json_string_array_contains(scope.get("required_values"), required_value) {
            issues.push(format!(
                "manifest helm_chart_signed_image_wiring_scope.required_values must include {required_value}"
            ));
        }
    }

    for required_ref in [
        "deployment.image.digest-reference",
        "deployment.cosign-required-annotation",
        "configmap.cosign-required-output",
    ] {
        if !json_string_array_contains(scope.get("required_template_refs"), required_ref) {
            issues.push(format!(
                "manifest helm_chart_signed_image_wiring_scope.required_template_refs must include {required_ref}"
            ));
        }
    }

    for non_claim in [
        "no Helm CLI rendering",
        "no cosign verification execution",
        "no Argo CD API integration",
        "no Kubernetes API integration",
    ] {
        if !json_string_array_contains(scope.get("non_claims"), non_claim) {
            issues.push(format!(
                "manifest helm_chart_signed_image_wiring_scope.non_claims must include {non_claim}"
            ));
        }
    }
}

fn json_string_array_contains(value: Option<&Value>, needle: &str) -> bool {
    value
        .and_then(Value::as_array)
        .map(|items| items.iter().any(|item| item.as_str() == Some(needle)))
        .unwrap_or(false)
}

fn require_values_yaml(text: Option<&str>, issues: &mut Vec<String>) -> usize {
    let Some(text) = text else {
        return 0;
    };
    let mut checked = 0usize;
    match find_image_digest(text) {
        Some(digest) if digest.is_empty() || valid_sha256_digest(&digest) => {
            checked += 1;
        }
        Some(digest) => {
            issues.push(format!(
                "values.yaml image.digest must be empty for release injection or sha256:<64 lowercase non-zero hex>, found {digest:?}"
            ));
        }
        None => issues.push("values.yaml must declare image.digest".to_string()),
    }
    checked
}

fn find_image_digest(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        let Some(value) = trimmed.strip_prefix("digest:") else {
            continue;
        };
        let value = value.trim().trim_matches('"').trim_matches('\'');
        return Some(value.to_string());
    }
    None
}

fn valid_sha256_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
        && hex.chars().any(|ch| ch != '0')
}

fn require_lines(text: Option<&str>, required: &[(&str, &str)], issues: &mut Vec<String>) -> usize {
    let Some(text) = text else {
        return 0;
    };
    let mut checked = 0usize;
    for (file, line) in required {
        if text.contains(line) {
            checked += 1;
        } else {
            issues.push(format!("{file} must contain {line:?}"));
        }
    }
    checked
}

fn reject_secret_like_markers(file: &str, text: &str, issues: &mut Vec<String>) {
    let lower = text.to_ascii_lowercase();
    for marker in [
        "password:",
        "passwd:",
        "private_key",
        "client_secret",
        "access_key",
        "secret_access_key",
        "bearer ",
        "kubeconfig",
    ] {
        if lower.contains(marker) {
            issues.push(format!("{file} contains credential-like marker {marker:?}"));
        }
    }
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempRepo {
        root: PathBuf,
    }

    impl TempRepo {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock after unix epoch")
                .as_nanos();
            let root =
                std::env::temp_dir().join(format!("oya-{name}-{}-{nonce}", std::process::id()));
            fs::create_dir_all(&root).expect("create temp repo");
            Self { root }
        }

        fn write(&self, path: &str, text: &str) {
            let path = self.root.join(path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create parent");
            }
            fs::write(path, text).expect("write fixture");
        }

        fn args(&self) -> CloudIacHelmChartArgs {
            CloudIacHelmChartArgs {
                repo_root: self.root.clone(),
                manifest: PathBuf::from(DEFAULT_MANIFEST),
                chart_root: PathBuf::from(DEFAULT_CHART_ROOT),
            }
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn parse_cloud_iac_helm_chart_defaults_to_live_paths() {
        let parsed = parse_cloud_iac_helm_chart_args(Vec::new()).expect("defaults");
        assert_eq!(parsed.repo_root, PathBuf::from("."));
        assert_eq!(parsed.manifest, PathBuf::from(DEFAULT_MANIFEST));
        assert_eq!(parsed.chart_root, PathBuf::from(DEFAULT_CHART_ROOT));
    }

    #[test]
    fn parse_cloud_iac_helm_chart_rejects_unknown_flag() {
        let error = parse_cloud_iac_helm_chart_args(vec!["--bogus".to_string()])
            .expect_err("unknown flag rejected");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn cloud_iac_helm_chart_gate_accepts_signed_image_wiring() {
        let temp = valid_temp_repo("cloud-iac-helm-chart-valid");
        let report = validate_cloud_iac_helm_chart_gate(temp.args()).expect("valid chart");
        assert_eq!(report.files_checked, 4);
        assert!(report.required_lines_checked >= 20);
    }

    #[test]
    fn cloud_iac_helm_chart_gate_rejects_missing_digest_wiring() {
        let temp = valid_temp_repo("cloud-iac-helm-chart-digest-drift");
        temp.write(
            "iac/iac/k8s/helm/templates/deployment.yaml",
            &valid_deployment().replace(
                "{{- if $imageDigest }}@{{ $imageDigest }}{{- else }}:",
                "{{- if $imageDigest }}:{{ .Values.image.tag }}{{- else }}:",
            ),
        );

        let error = validate_cloud_iac_helm_chart_gate(temp.args()).expect_err("drift rejected");
        assert!(error.contains("@{{ $imageDigest }}"));
    }

    #[test]
    fn cloud_iac_helm_chart_gate_rejects_zero_digest_placeholder() {
        let temp = valid_temp_repo("cloud-iac-helm-chart-zero-digest");
        let zero_digest = format!("sha256:{}", "0".repeat(64));
        temp.write(
            "iac/iac/k8s/helm/values.yaml",
            &valid_values().replace("digest: \"\"", &format!("digest: \"{zero_digest}\"")),
        );

        let error = validate_cloud_iac_helm_chart_gate(temp.args()).expect_err("zero rejected");
        assert!(error.contains("values.yaml image.digest"));
    }

    #[test]
    fn cloud_iac_helm_chart_gate_rejects_missing_manifest_scope() {
        let temp = valid_temp_repo("cloud-iac-helm-chart-manifest-drift");
        temp.write(
            DEFAULT_MANIFEST,
            r#"{"capabilities":[{"name":"cloud-iac-helm-chart-signed-image-wiring-gate","file":"marketplace/facade/dev-cli/src/cloud_iac_helm_chart_gate.rs"}]}"#,
        );

        let error = validate_cloud_iac_helm_chart_gate(temp.args()).expect_err("scope rejected");
        assert!(error.contains("helm_chart_signed_image_wiring_scope"));
    }

    fn valid_temp_repo(name: &str) -> TempRepo {
        let temp = TempRepo::new(name);
        temp.write(DEFAULT_MANIFEST, &valid_manifest());
        temp.write(
            "iac/iac/k8s/helm/Chart.yaml",
            valid_chart(),
        );
        temp.write(
            "iac/iac/k8s/helm/values.yaml",
            valid_values(),
        );
        temp.write(
            "iac/iac/k8s/helm/templates/deployment.yaml",
            &valid_deployment(),
        );
        temp.write(
            "iac/iac/k8s/helm/templates/configmap.yaml",
            valid_configmap(),
        );
        temp
    }

    fn valid_manifest() -> String {
        r#"{
  "capabilities": [
    {
      "tier": "T1",
      "name": "cloud-iac-helm-chart-signed-image-wiring-gate",
      "file": "marketplace/facade/dev-cli/src/cloud_iac_helm_chart_gate.rs",
      "risk_class": "high"
    }
  ],
  "helm_chart_signed_image_wiring_scope": {
    "chart_root": "iac/iac/k8s/helm",
    "runtime_mode": "local-filesystem-helm-chart-wiring-gate-no-helm-render",
    "chart_files_checked": [
      "Chart.yaml",
      "values.yaml",
      "templates/deployment.yaml",
      "templates/configmap.yaml"
    ],
    "required_values": [
      "image.digest",
      "image.cosign.required",
      "image.cosign.policy",
      "config.imagePromotionPolicy"
    ],
    "required_template_refs": [
      "deployment.image.digest-reference",
      "deployment.cosign-required-annotation",
      "configmap.cosign-required-output"
    ],
    "non_claims": [
      "no Helm CLI rendering",
      "no cosign verification execution",
      "no Argo CD API integration",
      "no Kubernetes API integration"
    ]
  }
}"#
        .to_string()
    }

    fn valid_chart() -> &'static str {
        r#"apiVersion: v2
name: oya-cloud-iac
type: application
annotations:
  oyatie.com/adr-0181: signed-image-promotion-required
  oyatie.com/adr-0349: argocd-application-source
"#
    }

    fn valid_values() -> &'static str {
        r#"image:
  registry: registry.oyatie.internal
  repository: oya-cloud-iac
  tag: "0.1.0"
  digest: ""
  cosign:
    required: true
    policy: ADR-0181
config:
  imagePromotionPolicy: cosign-required
"#
    }

    fn valid_deployment() -> String {
        r#"{{- $imageDigest := .Values.image.digest | default "" -}}
{{- if and .Values.image.cosign.required (not (and (regexMatch "^sha256:[0-9a-f]{64}$" $imageDigest) (regexMatch "[1-9a-f]" (trimPrefix "sha256:" $imageDigest)))) -}}
{{- fail "image.digest must be set to a real non-zero sha256 digest when image.cosign.required=true" -}}
{{- end -}}
apiVersion: apps/v1
kind: Deployment
spec:
  template:
    metadata:
      annotations:
        cosign.sigstore.dev/required: {{ .Values.image.cosign.required | quote }}
        oyatie.com/image-digest: {{ $imageDigest | quote }}
        oyatie.com/image-promotion-policy: {{ .Values.image.cosign.policy | quote }}
    spec:
      securityContext:
        seccompProfile:
          type: RuntimeDefault
      containers:
        - name: app
          image: "{{ .Values.image.registry }}/{{ .Values.image.repository }}{{- if $imageDigest }}@{{ $imageDigest }}{{- else }}:{{ .Values.image.tag }}{{- end }}"
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: true
            capabilities:
              drop:
                - ALL
"#
        .to_string()
    }

    fn valid_configmap() -> &'static str {
        r#"apiVersion: v1
kind: ConfigMap
data:
  image_signature_policy: {{ .Values.config.imagePromotionPolicy | quote }}
  cosign_required: {{ .Values.image.cosign.required | quote }}
"#
    }
}
