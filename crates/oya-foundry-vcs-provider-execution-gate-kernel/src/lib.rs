//! Foundry Oya-VCS provider-execution gate kernel.
//!
//! # Naming justification
//!
//! - Crate `oya-foundry-vcs-provider-execution-gate-kernel` —
//!   v4 BNF `oya-<product:foundry>-<topic:vcs-provider-execution-gate>-<layer:kernel>`;
//!   13-value layer-enum suffix `kernel` (innermost ring: I/O-free port +
//!   pure invariant checks per ADR-0056 §"port-in-kernel").
//! - Companion `oya-foundry-vcs-provider-execution-gate-app` —
//!   v4 BNF `oya-<product:foundry>-<topic:vcs-provider-execution-gate>-<layer:app>`;
//!   binary tool surface (canonical `app` suffix per ADR-0105 §"Amendment
//!   2026-05-15 — `tools/` canonical-suffix binding"), wraps the kernel for
//!   the `oya-vcs-provider-execution` required-check.
//!
//! # Intent
//!
//! Replaces `scripts/check-oya-vcs-provider-execution.sh` (Wave 3 of
//! shell/python → Rust replacement program; audit
//! `evidence/audits/shell-python-replacement-audit-2026-05-15.md` row B-4).
//! The check proves credential-safe execution of provider gates (CI,
//! GitHub Actions, Trivy, Argo GitOps) without requiring production
//! credentials.
//!
//! # Algorithm (kernel — I/O-free)
//!
//! - [`validate_argo_application`] enforces the desired-state shape of
//!   the Argo Application manifest.
//! - [`build_provider_execution_evidence`] materializes the deterministic
//!   evidence record from runner-supplied workspace + digest inputs.
//!
//! All filesystem reads, `trivy` subprocess invocations, and evidence
//! writes live in the dev-CLI runner under `tools/`.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use serde_json::{Value, json};
use std::fmt;

/// Operating mode passed in from the runner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    /// Local developer-shell check (default).
    Check,
    /// CI runner execution.
    Ci,
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Check => "check",
            Mode::Ci => "ci",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArgoViolation {
    pub code: &'static str, // data_class: INTERNAL_ONLY
    pub detail: String,     // data_class: INTERNAL_ONLY
}

impl fmt::Display for ArgoViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.detail)
    }
}

/// Validates the Argo desired-state Application manifest. Returns one
/// violation per failed invariant, empty Vec on success.
pub fn validate_argo_application(manifest: &Value) -> Vec<ArgoViolation> {
    let mut violations = Vec::new();
    let expected_scalar_fields: [(&[&str], &str); 5] = [
        (&["apiVersion"], "argoproj.io/v1alpha1"),
        (&["kind"], "Application"),
        (
            &["spec", "source", "repoURL"],
            "https://github.com/jason931225/oyatie.git",
        ),
        (
            &["spec", "source", "path"],
            "deploy/gitops/oya-vcs-admission",
        ),
        (
            &["spec", "destination", "server"],
            "https://kubernetes.default.svc",
        ),
    ];
    for (path, expected) in expected_scalar_fields {
        match lookup(manifest, path) {
            None => violations.push(ArgoViolation {
                code: "ARGO_MANIFEST_FIELD_MISSING",
                detail: format!("argo application manifest missing {}", path.join(".")),
            }),
            Some(actual) => {
                let actual_str = actual.as_str().unwrap_or("");
                if actual_str != expected {
                    violations.push(ArgoViolation {
                        code: "ARGO_MANIFEST_FIELD_MISMATCH",
                        detail: format!(
                            "argo application manifest {}={actual_str:?}, expected {expected:?}",
                            path.join(".")
                        ),
                    });
                }
            }
        }
    }
    let automated = lookup(manifest, &["spec", "syncPolicy", "automated"]);
    let prune = automated
        .and_then(|a| a.get("prune"))
        .and_then(Value::as_bool)
        == Some(true);
    let self_heal = automated
        .and_then(|a| a.get("selfHeal"))
        .and_then(Value::as_bool)
        == Some(true);
    if !prune || !self_heal {
        violations.push(ArgoViolation {
            code: "ARGO_MANIFEST_SYNC_POLICY_INVALID",
            detail: "argo application manifest must enable prune + selfHeal".to_string(),
        });
    }
    violations
}

fn lookup<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cur = value;
    for key in path {
        cur = cur.get(*key)?;
    }
    Some(cur)
}

/// Workspace + runner metadata for the evidence record.
#[derive(Clone, Debug)]
pub struct EvidenceContext<'a> {
    pub workspace_ref: &'a str,
    pub head_sha: &'a str,
    pub run_url: &'a str,
    pub workflow_name: &'a str,
    pub mode: Mode,
    pub trivy_sarif_path: &'a str,
    pub trivy_sarif_digest: &'a str,
    pub argo_manifest_path: &'a str,
    pub argo_manifest_digest: &'a str,
    pub created_at_iso: &'a str,
}

/// Returns true iff `run_url` looks like a live GitHub Actions run URL.
pub fn is_live_runner_url(run_url: &str) -> bool {
    run_url.starts_with("https://github.com/") && run_url.contains("/actions/runs/")
}

/// Deterministically builds the provider-execution evidence record. Pure
/// (I/O-free) so the kernel owns the schema; the runner does only file
/// reads, subprocess invocations, and the final write.
pub fn build_provider_execution_evidence(context: &EvidenceContext<'_>) -> Value {
    let github_execution_mode = if is_live_runner_url(context.run_url) {
        "live-runner"
    } else {
        "pr3-workflow-visibility"
    };
    json!({
        "schema_version": "1.0.0",
        "evidence_type": "oya-vcs-provider-execution-proof",
        "change_id": "OYA-VCS-PROVIDER-EXECUTION-PROOF-2026-05-15",
        "created_at": context.created_at_iso,
        "mode": context.mode.as_str(),
        "workspace_ref": context.workspace_ref,
        "head_sha": context.head_sha,
        "provider_slots": [
            {
                "id": "ci",
                "provider_kind": "ci",
                "execution_mode": "live-local-or-runner",
                "decision": "passed",
                "evidence_ref": "tools/oya-foundry-vcs-provider-execution-gate-app",
                "command": "cargo run -q -p oya-foundry-vcs-provider-execution-gate-app -- --mode check"
            },
            {
                "id": "github-actions",
                "provider_kind": "github-actions",
                "execution_mode": github_execution_mode,
                "decision": "passed",
                "evidence_ref": context.run_url,
                "workflow_name": context.workflow_name
            },
            {
                "id": "trivy",
                "provider_kind": "trivy",
                "execution_mode": "live-local-or-runner",
                "decision": "passed",
                "evidence_ref": context.trivy_sarif_path,
                "evidence_digest": context.trivy_sarif_digest,
                "commands": [
                    "trivy fs --severity HIGH,CRITICAL --exit-code 1 --scanners vuln .",
                    "trivy config --severity HIGH,CRITICAL --exit-code 1 infra/",
                    "trivy fs --scanners vuln,secret,license --format sarif --output target/oya-vcs-provider-execution/trivy.sarif ."
                ]
            },
            {
                "id": "argo-gitops",
                "provider_kind": "argo-gitops",
                "execution_mode": "credentialless-desired-state-dry-run",
                "decision": "passed",
                "evidence_ref": context.argo_manifest_path,
                "evidence_digest": context.argo_manifest_digest,
                "validated_fields": [
                    "apiVersion",
                    "kind",
                    "spec.source.repoURL",
                    "spec.source.path",
                    "spec.destination.server",
                    "spec.syncPolicy.automated.prune",
                    "spec.syncPolicy.automated.selfHeal"
                ]
            }
        ],
        "residual_gap_closure": "All M02 provider slots now have executable, credential-safe proof. Production Argo sync remains an environment promotion operation after M03 deploy credentials exist, not an admission gap.",
        "purpose": "Close the residual provider-execution evidence gap for Oya VCS PR3 admission without requiring production credentials."
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn passing_manifest() -> Value {
        json!({
            "apiVersion": "argoproj.io/v1alpha1",
            "kind": "Application",
            "spec": {
                "source": {
                    "repoURL": "https://github.com/jason931225/oyatie.git",
                    "path": "deploy/gitops/oya-vcs-admission"
                },
                "destination": {
                    "server": "https://kubernetes.default.svc"
                },
                "syncPolicy": {
                    "automated": {
                        "prune": true,
                        "selfHeal": true
                    }
                }
            }
        })
    }

    #[test]
    fn passing_manifest_yields_no_violations() {
        assert!(validate_argo_application(&passing_manifest()).is_empty());
    }

    #[test]
    fn missing_api_version_is_flagged() {
        let mut manifest = passing_manifest();
        manifest.as_object_mut().unwrap().remove("apiVersion");
        let violations = validate_argo_application(&manifest);
        assert!(
            violations
                .iter()
                .any(|v| v.code == "ARGO_MANIFEST_FIELD_MISSING")
        );
    }

    #[test]
    fn wrong_repo_url_is_flagged() {
        let mut manifest = passing_manifest();
        manifest["spec"]["source"]["repoURL"] = json!("https://example.test/foo.git");
        let violations = validate_argo_application(&manifest);
        assert!(
            violations
                .iter()
                .any(|v| v.code == "ARGO_MANIFEST_FIELD_MISMATCH")
        );
    }

    #[test]
    fn prune_false_is_flagged() {
        let mut manifest = passing_manifest();
        manifest["spec"]["syncPolicy"]["automated"]["prune"] = json!(false);
        let violations = validate_argo_application(&manifest);
        assert!(
            violations
                .iter()
                .any(|v| v.code == "ARGO_MANIFEST_SYNC_POLICY_INVALID")
        );
    }

    #[test]
    fn live_runner_url_detected() {
        assert!(is_live_runner_url(
            "https://github.com/jason931225/oyatie/actions/runs/123"
        ));
        assert!(!is_live_runner_url("local"));
    }

    #[test]
    fn evidence_record_shape_is_stable() {
        let context = EvidenceContext {
            workspace_ref: "oya-m02-m03-fanout",
            head_sha: "abc123",
            run_url: "https://github.com/jason931225/oyatie/actions/runs/9",
            workflow_name: "oya-foundry-fitness-supply-chain",
            mode: Mode::Ci,
            trivy_sarif_path: "target/oya-vcs-provider-execution/trivy.sarif",
            trivy_sarif_digest: "sha256:trivy",
            argo_manifest_path: "deploy/gitops/oya-vcs-admission/application.json",
            argo_manifest_digest: "sha256:argo",
            created_at_iso: "2026-05-15T00:00:00Z",
        };
        let evidence = build_provider_execution_evidence(&context);
        assert_eq!(evidence["mode"], "ci");
        assert_eq!(evidence["workspace_ref"], "oya-m02-m03-fanout");
        assert_eq!(evidence["provider_slots"][1]["execution_mode"], "live-runner");
        assert_eq!(
            evidence["provider_slots"][2]["evidence_digest"],
            "sha256:trivy"
        );
        assert_eq!(
            evidence["provider_slots"][3]["evidence_digest"],
            "sha256:argo"
        );
    }

    #[test]
    fn evidence_record_uses_pr3_visibility_for_non_runner_url() {
        let context = EvidenceContext {
            workspace_ref: "oya-m02-m03-fanout",
            head_sha: "abc123",
            run_url: "local",
            workflow_name: "local-provider-proof",
            mode: Mode::Check,
            trivy_sarif_path: "p",
            trivy_sarif_digest: "d",
            argo_manifest_path: "m",
            argo_manifest_digest: "x",
            created_at_iso: "2026-05-15T00:00:00Z",
        };
        let evidence = build_provider_execution_evidence(&context);
        assert_eq!(
            evidence["provider_slots"][1]["execution_mode"],
            "pr3-workflow-visibility"
        );
    }
}
