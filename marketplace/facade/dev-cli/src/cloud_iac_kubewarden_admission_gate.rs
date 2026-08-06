//! `oya gate validate cloud-iac-kubewarden-admission-policy` runner.
//!
//! This is a local filesystem source gate for the Kubewarden-default Cloud IaC
//! admission policy materialization. It proves that the repo carries a
//! Kubewarden PolicyServer, a Kubewarden ClusterAdmissionPolicy for signed image
//! verification, a local verification-config source, and Kyverno first-class
//! adapter parity metadata. It intentionally does not install Kubewarden, call
//! Kubernetes, execute cosign, contact Rekor, render Helm, call Argo CD, or prove
//! live admission execution.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

const GATE_NAME: &str = "cloud-iac-kubewarden-admission-policy";
const GATE_FILE: &str = "marketplace/facade/dev-cli/src/cloud_iac_kubewarden_admission_gate.rs";
const DEFAULT_MANIFEST: &str = "iac/manifest.json";
const DEFAULT_KUBEWARDEN_ROOT: &str = "iac/iac/k8s/kubewarden";
const DEFAULT_KYVERNO_POLICY: &str = "infra/kyverno/policies/require-signed-images.yaml";
const RUNTIME_MODE: &str = "local-filesystem-admission-policy-source-gate-no-controller-execution";

const POLICY_FILES: &[&str] = &[
    "policy-server.yaml",
    "verify-image-signatures-policy.yaml",
    "verification-config.yaml",
];

const POLICY_SERVER_REQUIRED_LINES: &[(&str, &str)] = &[
    (
        "policy-server.yaml",
        "apiVersion: policies.kubewarden.io/v1",
    ),
    ("policy-server.yaml", "kind: PolicyServer"),
    ("policy-server.yaml", "name: oya-kubewarden-default"),
    (
        "policy-server.yaml",
        "image: ghcr.io/kubewarden/policy-server:",
    ),
    (
        "policy-server.yaml",
        "oyatie.com/admission-default: kubewarden",
    ),
    (
        "policy-server.yaml",
        "oyatie.com/runtime-mode: local-filesystem-admission-policy-source-gate-no-controller-execution",
    ),
    ("policy-server.yaml", "replicas:"),
];

const CLUSTER_POLICY_REQUIRED_LINES: &[(&str, &str)] = &[
    (
        "verify-image-signatures-policy.yaml",
        "apiVersion: policies.kubewarden.io/v1",
    ),
    (
        "verify-image-signatures-policy.yaml",
        "kind: ClusterAdmissionPolicy",
    ),
    (
        "verify-image-signatures-policy.yaml",
        "name: oya-verify-signed-images",
    ),
    (
        "verify-image-signatures-policy.yaml",
        "module: registry://ghcr.io/kubewarden/policies/verify-image-signatures:",
    ),
    (
        "verify-image-signatures-policy.yaml",
        "policyServer: oya-kubewarden-default",
    ),
    ("verify-image-signatures-policy.yaml", "mutating: true"),
    ("verify-image-signatures-policy.yaml", "rules:"),
    (
        "verify-image-signatures-policy.yaml",
        "operations: [\"CREATE\", \"UPDATE\"]",
    ),
    (
        "verify-image-signatures-policy.yaml",
        "resources: [\"pods\"]",
    ),
    ("verify-image-signatures-policy.yaml", "settings:"),
    (
        "verify-image-signatures-policy.yaml",
        "modifyImagesWithDigest: true",
    ),
    ("verify-image-signatures-policy.yaml", "ghcr.io/oyatie/*"),
    (
        "verify-image-signatures-policy.yaml",
        "owner: jason931225",
    ),
    ("verify-image-signatures-policy.yaml", "repo: oyatie"),
    (
        "verify-image-signatures-policy.yaml",
        "issuer: https://token.actions.githubusercontent.com",
    ),
    (
        "verify-image-signatures-policy.yaml",
        "rekor: https://rekor.sigstore.dev",
    ),
    (
        "verify-image-signatures-policy.yaml",
        "oyatie.com/admission-default: kubewarden",
    ),
    (
        "verify-image-signatures-policy.yaml",
        "oyatie.com/kyverno-adapter-parity: first-class",
    ),
];

const VERIFICATION_CONFIG_REQUIRED_LINES: &[(&str, &str)] = &[
    ("verification-config.yaml", "apiVersion: v1"),
    ("verification-config.yaml", "kind: ConfigMap"),
    (
        "verification-config.yaml",
        "name: oya-kubewarden-image-verification-config",
    ),
    ("verification-config.yaml", "verification-config.yaml: |"),
    ("verification-config.yaml", "githubAction"),
    ("verification-config.yaml", "owner: jason931225"),
    ("verification-config.yaml", "repo: oyatie"),
    (
        "verification-config.yaml",
        "issuer: https://token.actions.githubusercontent.com",
    ),
    (
        "verification-config.yaml",
        "rekor: https://rekor.sigstore.dev",
    ),
    (
        "verification-config.yaml",
        "oyatie.com/admission-default: kubewarden",
    ),
];

const KYVERNO_REQUIRED_LINES: &[(&str, &str)] = &[
    ("kyverno policy", "apiVersion: kyverno.io/v1"),
    ("kyverno policy", "kind: ClusterPolicy"),
    ("kyverno policy", "verifyImages:"),
    ("kyverno policy", "keyless:"),
    ("kyverno policy", "rekor:"),
    ("kyverno policy", "oyatie.com/admission-adapter: kyverno"),
    ("kyverno policy", "oyatie.com/default-admission: \"false\""),
    ("kyverno policy", "oyatie.com/first-class-adapter: \"true\""),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacKubewardenAdmissionArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) kubewarden_root: PathBuf,
    pub(crate) kyverno_policy: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CloudIacKubewardenAdmissionReport {
    pub(crate) manifest_path: String,
    pub(crate) kubewarden_root_path: String,
    pub(crate) kyverno_policy_path: String,
    pub(crate) policy_files_checked: usize,
    pub(crate) required_markers_checked: usize,
}

pub(crate) fn parse_cloud_iac_kubewarden_admission_args(
    args: Vec<String>,
) -> Result<CloudIacKubewardenAdmissionArgs, String> {
    let mut parsed = CloudIacKubewardenAdmissionArgs {
        repo_root: PathBuf::from("."),
        manifest: PathBuf::from(DEFAULT_MANIFEST),
        kubewarden_root: PathBuf::from(DEFAULT_KUBEWARDEN_ROOT),
        kyverno_policy: PathBuf::from(DEFAULT_KYVERNO_POLICY),
    };

    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => parsed.repo_root = take_path("--repo-root", &mut iter)?,
            "--manifest" => parsed.manifest = take_path("--manifest", &mut iter)?,
            "--kubewarden-root" => {
                parsed.kubewarden_root = take_path("--kubewarden-root", &mut iter)?
            }
            "--kyverno-policy" => parsed.kyverno_policy = take_path("--kyverno-policy", &mut iter)?,
            other => {
                return Err(format!(
                    "{GATE_NAME}: unknown flag {other:?}; usage: \
                     oya gate validate {GATE_NAME} \
                     [--repo-root <.>] [--manifest <{DEFAULT_MANIFEST}>] \
                     [--kubewarden-root <{DEFAULT_KUBEWARDEN_ROOT}>] \
                     [--kyverno-policy <{DEFAULT_KYVERNO_POLICY}>]"
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

pub(crate) fn validate_cloud_iac_kubewarden_admission_gate(
    args: CloudIacKubewardenAdmissionArgs,
) -> Result<CloudIacKubewardenAdmissionReport, String> {
    let repo_root = canonicalize_root(&args.repo_root)?;
    let manifest_path = canonicalize_under_repo(&repo_root, &args.manifest, "manifest")?;
    let kubewarden_root =
        canonicalize_under_repo(&repo_root, &args.kubewarden_root, "kubewarden root")?;
    let kyverno_policy =
        canonicalize_under_repo(&repo_root, &args.kyverno_policy, "kyverno policy")?;

    let mut issues = Vec::new();
    let manifest = read_json(&manifest_path, "manifest", &mut issues);
    if let Some(manifest) = manifest.as_ref() {
        validate_manifest_scope(
            manifest,
            &args.kubewarden_root,
            &args.kyverno_policy,
            &mut issues,
        );
    }

    let mut policy_files_checked = 0usize;
    for relative_file in POLICY_FILES {
        let path = kubewarden_root.join(relative_file);
        let text = read_text(&path, relative_file, &mut issues);
        policy_files_checked += 1;
        if let Some(text) = text.as_ref() {
            reject_secret_like_markers(relative_file, text, &mut issues);
            reject_unpinned_latest(relative_file, text, &mut issues);
        }
    }
    let kyverno_text = read_text(&kyverno_policy, "kyverno policy", &mut issues);
    policy_files_checked += 1;
    if let Some(text) = kyverno_text.as_ref() {
        reject_secret_like_markers("kyverno policy", text, &mut issues);
    }

    let policy_server = read_policy_file(&kubewarden_root, "policy-server.yaml", &mut issues);
    let cluster_policy = read_policy_file(
        &kubewarden_root,
        "verify-image-signatures-policy.yaml",
        &mut issues,
    );
    let verification_config =
        read_policy_file(&kubewarden_root, "verification-config.yaml", &mut issues);

    let mut required_markers_checked = 0usize;
    required_markers_checked += require_lines(
        policy_server.as_deref(),
        POLICY_SERVER_REQUIRED_LINES,
        &mut issues,
    );
    required_markers_checked += require_lines(
        cluster_policy.as_deref(),
        CLUSTER_POLICY_REQUIRED_LINES,
        &mut issues,
    );
    required_markers_checked += require_lines(
        verification_config.as_deref(),
        VERIFICATION_CONFIG_REQUIRED_LINES,
        &mut issues,
    );
    required_markers_checked +=
        require_lines(kyverno_text.as_deref(), KYVERNO_REQUIRED_LINES, &mut issues);

    if !issues.is_empty() {
        return Err(format!(
            "{GATE_NAME} validation failed:\n- {}",
            issues.join("\n- ")
        ));
    }

    Ok(CloudIacKubewardenAdmissionReport {
        manifest_path: slash_path(&args.manifest),
        kubewarden_root_path: slash_path(&args.kubewarden_root),
        kyverno_policy_path: slash_path(&args.kyverno_policy),
        policy_files_checked,
        required_markers_checked,
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

fn read_policy_file(
    kubewarden_root: &Path,
    relative_file: &str,
    issues: &mut Vec<String>,
) -> Option<String> {
    read_text(&kubewarden_root.join(relative_file), relative_file, issues)
}

fn validate_manifest_scope(
    manifest: &Value,
    kubewarden_root: &Path,
    kyverno_policy: &Path,
    issues: &mut Vec<String>,
) {
    let capability_ok = manifest
        .get("capabilities")
        .and_then(Value::as_array)
        .map(|capabilities| {
            capabilities.iter().any(|capability| {
                capability.get("name").and_then(Value::as_str)
                    == Some("cloud-iac-kubewarden-admission-policy-gate")
                    && capability.get("file").and_then(Value::as_str) == Some(GATE_FILE)
            })
        })
        .unwrap_or(false);
    if !capability_ok {
        issues.push(format!(
            "manifest capabilities must declare cloud-iac-kubewarden-admission-policy-gate backed by {GATE_FILE}"
        ));
    }

    let Some(scope) = manifest.get("kubewarden_admission_policy_scope") else {
        issues.push("manifest must declare kubewarden_admission_policy_scope".to_string());
        return;
    };

    let expected_root = slash_path(kubewarden_root);
    if scope.get("kubewarden_root").and_then(Value::as_str) != Some(expected_root.as_str()) {
        issues.push(format!(
            "manifest kubewarden_admission_policy_scope.kubewarden_root must be {expected_root}"
        ));
    }
    let expected_kyverno = slash_path(kyverno_policy);
    if scope.get("kyverno_policy").and_then(Value::as_str) != Some(expected_kyverno.as_str()) {
        issues.push(format!(
            "manifest kubewarden_admission_policy_scope.kyverno_policy must be {expected_kyverno}"
        ));
    }
    if scope.get("runtime_mode").and_then(Value::as_str) != Some(RUNTIME_MODE) {
        issues.push(format!(
            "manifest kubewarden_admission_policy_scope.runtime_mode must be {RUNTIME_MODE}"
        ));
    }
    if scope
        .get("default_admission_substrate")
        .and_then(Value::as_str)
        != Some("Kubewarden")
    {
        issues.push(
            "manifest kubewarden_admission_policy_scope.default_admission_substrate must be Kubewarden"
                .to_string(),
        );
    }
    if !json_string_array_contains(scope.get("first_class_adapters"), "Kyverno") {
        issues.push(
            "manifest kubewarden_admission_policy_scope.first_class_adapters must include Kyverno"
                .to_string(),
        );
    }
    for file in POLICY_FILES {
        if !json_string_array_contains(scope.get("policy_files_checked"), file) {
            issues.push(format!(
                "manifest kubewarden_admission_policy_scope.policy_files_checked must include {file}"
            ));
        }
    }
    for invariant in [
        "Kubewarden is the default admission substrate",
        "Kyverno remains a first-class adapter, not the default",
        "signed Oyatie images are required through Kubewarden policy source",
        "policy source is local filesystem only and does not prove live admission execution",
    ] {
        if !json_string_array_contains(scope.get("enforced_invariants"), invariant) {
            issues.push(format!(
                "manifest kubewarden_admission_policy_scope.enforced_invariants must include {invariant}"
            ));
        }
    }
    for non_claim in [
        "no Kubewarden controller installation",
        "no admission-controller execution",
        "no cosign verification execution",
        "no Kubernetes API integration",
        "no production readiness",
    ] {
        if !json_string_array_contains(scope.get("non_claims"), non_claim) {
            issues.push(format!(
                "manifest kubewarden_admission_policy_scope.non_claims must include {non_claim}"
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

fn reject_unpinned_latest(file: &str, text: &str, issues: &mut Vec<String>) {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.contains(":latest") || trimmed.ends_with("latest") {
            issues.push(format!(
                "{file} contains unpinned latest reference in line {trimmed:?}"
            ));
        }
    }
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
        "token:",
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

        fn args(&self) -> CloudIacKubewardenAdmissionArgs {
            CloudIacKubewardenAdmissionArgs {
                repo_root: self.root.clone(),
                manifest: PathBuf::from(DEFAULT_MANIFEST),
                kubewarden_root: PathBuf::from(DEFAULT_KUBEWARDEN_ROOT),
                kyverno_policy: PathBuf::from(DEFAULT_KYVERNO_POLICY),
            }
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn parse_cloud_iac_kubewarden_defaults_to_live_paths() {
        let parsed = parse_cloud_iac_kubewarden_admission_args(Vec::new()).expect("defaults");
        assert_eq!(parsed.repo_root, PathBuf::from("."));
        assert_eq!(parsed.manifest, PathBuf::from(DEFAULT_MANIFEST));
        assert_eq!(
            parsed.kubewarden_root,
            PathBuf::from(DEFAULT_KUBEWARDEN_ROOT)
        );
        assert_eq!(parsed.kyverno_policy, PathBuf::from(DEFAULT_KYVERNO_POLICY));
    }

    #[test]
    fn parse_cloud_iac_kubewarden_rejects_unknown_flag() {
        let error = parse_cloud_iac_kubewarden_admission_args(vec!["--bogus".to_string()])
            .expect_err("unknown flag rejected");
        assert!(error.contains("unknown flag"));
    }

    #[test]
    fn cloud_iac_kubewarden_admission_gate_accepts_default_and_adapter_sources() {
        let temp = valid_temp_repo("cloud-iac-kubewarden-valid");
        let report = validate_cloud_iac_kubewarden_admission_gate(temp.args())
            .expect("valid kubewarden admission source");
        assert_eq!(report.policy_files_checked, 4);
        assert!(report.required_markers_checked >= 40);
    }

    #[test]
    fn cloud_iac_kubewarden_admission_gate_rejects_missing_cluster_policy() {
        let temp = valid_temp_repo("cloud-iac-kubewarden-missing-policy");
        fs::remove_file(
            temp.root
                .join(DEFAULT_KUBEWARDEN_ROOT)
                .join("verify-image-signatures-policy.yaml"),
        )
        .expect("remove cluster policy");

        let error = validate_cloud_iac_kubewarden_admission_gate(temp.args())
            .expect_err("missing policy rejected");
        assert!(error.contains("verify-image-signatures-policy.yaml"));
    }

    #[test]
    fn cloud_iac_kubewarden_admission_gate_rejects_kyverno_without_adapter_parity() {
        let temp = valid_temp_repo("cloud-iac-kubewarden-kyverno-drift");
        temp.write(
            DEFAULT_KYVERNO_POLICY,
            valid_kyverno_policy()
                .replace("    oyatie.com/first-class-adapter: \"true\"\n", "")
                .as_str(),
        );

        let error = validate_cloud_iac_kubewarden_admission_gate(temp.args())
            .expect_err("adapter parity rejected");
        assert!(error.contains("first-class-adapter"));
    }

    fn valid_temp_repo(name: &str) -> TempRepo {
        let temp = TempRepo::new(name);
        temp.write(DEFAULT_MANIFEST, valid_manifest());
        temp.write(
            "iac/iac/k8s/kubewarden/policy-server.yaml",
            valid_policy_server(),
        );
        temp.write(
            "iac/iac/k8s/kubewarden/verify-image-signatures-policy.yaml",
            valid_cluster_policy(),
        );
        temp.write(
            "iac/iac/k8s/kubewarden/verification-config.yaml",
            valid_verification_config(),
        );
        temp.write(DEFAULT_KYVERNO_POLICY, valid_kyverno_policy());
        temp
    }

    fn valid_manifest() -> &'static str {
        r#"{
  "capabilities": [
    {
      "tier": "T1",
      "name": "cloud-iac-kubewarden-admission-policy-gate",
      "file": "marketplace/facade/dev-cli/src/cloud_iac_kubewarden_admission_gate.rs",
      "risk_class": "high"
    }
  ],
  "kubewarden_admission_policy_scope": {
    "kubewarden_root": "iac/iac/k8s/kubewarden",
    "kyverno_policy": "infra/kyverno/policies/require-signed-images.yaml",
    "runtime_mode": "local-filesystem-admission-policy-source-gate-no-controller-execution",
    "default_admission_substrate": "Kubewarden",
    "first_class_adapters": ["Kyverno"],
    "policy_files_checked": [
      "policy-server.yaml",
      "verify-image-signatures-policy.yaml",
      "verification-config.yaml"
    ],
    "enforced_invariants": [
      "Kubewarden is the default admission substrate",
      "Kyverno remains a first-class adapter, not the default",
      "signed Oyatie images are required through Kubewarden policy source",
      "policy source is local filesystem only and does not prove live admission execution"
    ],
    "non_claims": [
      "no Kubewarden controller installation",
      "no admission-controller execution",
      "no cosign verification execution",
      "no Kubernetes API integration",
      "no production readiness"
    ]
  }
}"#
    }

    fn valid_policy_server() -> &'static str {
        r#"apiVersion: policies.kubewarden.io/v1
kind: PolicyServer
metadata:
  name: oya-kubewarden-default
  labels:
    oyatie.com/admission-default: kubewarden
    oyatie.com/runtime-mode: local-filesystem-admission-policy-source-gate-no-controller-execution
spec:
  replicas: 2
  image: ghcr.io/kubewarden/policy-server:v1.30.0
"#
    }

    fn valid_cluster_policy() -> &'static str {
        r#"apiVersion: policies.kubewarden.io/v1
kind: ClusterAdmissionPolicy
metadata:
  name: oya-verify-signed-images
  labels:
    oyatie.com/admission-default: kubewarden
    oyatie.com/kyverno-adapter-parity: first-class
spec:
  module: registry://ghcr.io/kubewarden/policies/verify-image-signatures:v0.3.0
  policyServer: oya-kubewarden-default
  mutating: true
  rules:
    - apiGroups: [""]
      apiVersions: ["v1"]
      resources: ["pods"]
      operations: ["CREATE", "UPDATE"]
  settings:
    modifyImagesWithDigest: true
    signatures:
      - image: ghcr.io/oyatie/*
        githubActions:
          owner: jason931225
          repo: oyatie
          issuer: https://token.actions.githubusercontent.com
          rekor: https://rekor.sigstore.dev
"#
    }

    fn valid_verification_config() -> &'static str {
        r#"apiVersion: v1
kind: ConfigMap
metadata:
  name: oya-kubewarden-image-verification-config
  labels:
    oyatie.com/admission-default: kubewarden
data:
  verification-config.yaml: |
    allOf:
      - kind: githubAction
        owner: jason931225
        repo: oyatie
        issuer: https://token.actions.githubusercontent.com
        rekor: https://rekor.sigstore.dev
"#
    }

    fn valid_kyverno_policy() -> &'static str {
        r#"apiVersion: kyverno.io/v1
kind: ClusterPolicy
metadata:
  name: require-signed-oyatie-images
  labels:
    oyatie.com/admission-adapter: kyverno
    oyatie.com/default-admission: "false"
  annotations:
    oyatie.com/first-class-adapter: "true"
spec:
  validationFailureAction: Enforce
  rules:
    - name: verify-cosign-keyless-rekor
      match:
        any:
          - resources:
              kinds: ["Pod"]
      verifyImages:
        - imageReferences:
            - "ghcr.io/oyatie/*"
          attestors:
            - entries:
                - keyless:
                    issuer: "https://token.actions.githubusercontent.com"
                    subjectRegExp: "https://github.com/jason931225/oyatie/.github/workflows/.+@refs/(heads/dev|tags/v.+)"
                    rekor:
                      url: "https://rekor.sigstore.dev"
"#
    }
}
