//! IaC tier discipline gate — advisory CI lane per ADR-0202.
//!
//! # What this gate enforces
//!
//! ADR-0202 separates three concerns:
//!
//! - Tier A — ArgoCD: K8s app manifests, Helm releases, Kustomize.
//! - Tier B — OpenTofu: cloud-side resources (VPC, IAM, DNS, KMS,
//!   namespace bootstrap, ArgoCD project bootstrap).
//! - Tier C — Cluster API: K8s cluster lifecycle.
//!
//! Boundary violations:
//!
//! 1. `OpenTofuDefinesPodManifest` — a `.tofu` / `.tf` file
//!    declares a `kubernetes_deployment` / `kubernetes_pod` /
//!    `kubernetes_stateful_set` / `kubernetes_daemonset` etc.
//! 2. `ArgocdAppReferencesCloudPrimitive` — an ArgoCD Application
//!    YAML references AWS / GCP / Azure cloud primitives that
//!    must come from Tier B.
//! 3. `TerraformResidual` — `terraform { ... }` block or
//!    `*.terraform.tfstate` reference past the 90-day migration
//!    window (caller supplies window-elapsed flag).
//! 4. `ArgocdProjectBootstrappedFromTierA` — a Tier-A artifact
//!    declares an ArgoCD `AppProject`; that bootstrap belongs to
//!    Tier B (OpenTofu).
//!
//! # Layer
//!
//! `domain` (port-in-kernel per ADR-0056).
//!
//! # Naming justification
//!
//! `check-iac-tier-discipline` follows the ADR-0532/0533 de-branded grammar:
//! `<group:check>-<axis:iac-tier-discipline>`.
//!
//! # References
//!
//! - ADR-0202 — GitOps + IaC + cluster lifecycle three-tier.
//! - ADR-0171 — multi-cluster federation (ArgoCD + Cluster API).
//! - ADR-0173 — vendor lock-in avoidance (Terraform BSL → OpenTofu).

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]
#![allow(clippy::result_large_err)]

use std::fmt;

/// One IaC artifact under audit. Kind drives which boundary rules
/// apply.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IacArtifact {
    pub tier: IacTier,
    pub path: String,
    pub contents: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum IacTier {
    /// Tier A — ArgoCD: app deploy.
    TierAArgoCd,
    /// Tier B — OpenTofu: cloud-side resources.
    TierBOpenTofu,
    /// Tier C — Cluster API: cluster lifecycle.
    TierCClusterApi,
}

impl fmt::Display for IacTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IacTier::TierAArgoCd => f.write_str("tier-a-argocd"),
            IacTier::TierBOpenTofu => f.write_str("tier-b-opentofu"),
            IacTier::TierCClusterApi => f.write_str("tier-c-cluster-api"),
        }
    }
}

/// Audit configuration.
#[derive(Clone, Debug, Default)]
pub struct DisciplineConfig {
    /// Set to `true` once the 90-day Terraform → OpenTofu
    /// migration window has elapsed. When `true`, residual
    /// Terraform usage is flagged.
    pub migration_window_elapsed: bool,
}

/// Successful report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisciplineReport {
    pub artifacts_checked: usize,
}

/// Violation record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisciplineViolation {
    pub tier: IacTier,
    pub path: String,
    pub kind: ViolationKind,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ViolationKind {
    /// Tier-B artifact declares a per-pod manifest (Tier-A territory).
    OpenTofuDefinesPodManifest,
    /// Tier-A artifact references a cloud primitive (Tier-B territory).
    ArgocdAppReferencesCloudPrimitive,
    /// Tier-A artifact declares an ArgoCD `AppProject` (Tier-B bootstrap).
    ArgocdProjectBootstrappedFromTierA,
    /// Residual Terraform after migration window.
    TerraformResidual,
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ViolationKind::OpenTofuDefinesPodManifest => {
                f.write_str("opentofu-defines-pod-manifest")
            }
            ViolationKind::ArgocdAppReferencesCloudPrimitive => {
                f.write_str("argocd-app-references-cloud-primitive")
            }
            ViolationKind::ArgocdProjectBootstrappedFromTierA => {
                f.write_str("argocd-project-bootstrapped-from-tier-a")
            }
            ViolationKind::TerraformResidual => f.write_str("terraform-residual"),
        }
    }
}

// Tier-A territory markers that must NOT appear in a Tier-B artifact.
const POD_MANIFEST_KINDS: &[&str] = &[
    "kubernetes_deployment",
    "kubernetes_stateful_set",
    "kubernetes_daemon_set",
    "kubernetes_pod",
    "kubernetes_replica_set",
];

// Tier-B territory markers that must NOT appear in a Tier-A artifact.
const CLOUD_PRIMITIVE_KINDS: &[&str] = &[
    "aws_iam_role",
    "aws_iam_policy",
    "aws_vpc",
    "aws_subnet",
    "aws_kms_key",
    "aws_route53_zone",
    "aws_rds_instance",
    "aws_sesv2_email_identity",
    "google_compute_network",
    "google_kms_key_ring",
    "azurerm_virtual_network",
    "azurerm_key_vault",
];

const TERRAFORM_RESIDUAL_MARKERS: &[&str] = &[
    "terraform.tfstate",
    "terraform-state",
    "required_providers = {\n    hashicorp",
    "source = \"hashicorp/",
];

/// Audit a batch of artifacts against the tier boundary table.
#[must_use]
pub fn audit(
    config: &DisciplineConfig,
    artifacts: &[IacArtifact],
) -> (DisciplineReport, Vec<DisciplineViolation>) {
    let mut violations: Vec<DisciplineViolation> = Vec::new();

    for art in artifacts {
        match art.tier {
            IacTier::TierBOpenTofu => {
                for kind in POD_MANIFEST_KINDS {
                    let needle = format!("resource \"{kind}\"");
                    if art.contents.contains(&needle) {
                        violations.push(DisciplineViolation {
                            tier: art.tier,
                            path: art.path.clone(),
                            kind: ViolationKind::OpenTofuDefinesPodManifest,
                            summary: format!(
                                "Tier-B OpenTofu artifact declares `{kind}` — per-pod manifests belong to Tier-A (ArgoCD)"
                            ),
                        });
                    }
                }
                // Terraform residual past migration window.
                if config.migration_window_elapsed {
                    for marker in TERRAFORM_RESIDUAL_MARKERS {
                        if art.contents.contains(marker) {
                            violations.push(DisciplineViolation {
                                tier: art.tier,
                                path: art.path.clone(),
                                kind: ViolationKind::TerraformResidual,
                                summary: format!(
                                    "Tier-B artifact still references Terraform marker `{marker}` past 90-day window"
                                ),
                            });
                        }
                    }
                }
            }
            IacTier::TierAArgoCd => {
                for kind in CLOUD_PRIMITIVE_KINDS {
                    if art.contents.contains(kind) {
                        violations.push(DisciplineViolation {
                            tier: art.tier,
                            path: art.path.clone(),
                            kind: ViolationKind::ArgocdAppReferencesCloudPrimitive,
                            summary: format!(
                                "Tier-A ArgoCD artifact references cloud primitive `{kind}` — belongs to Tier-B (OpenTofu)"
                            ),
                        });
                    }
                }
                if art.contents.contains("kind: AppProject") {
                    violations.push(DisciplineViolation {
                        tier: art.tier,
                        path: art.path.clone(),
                        kind: ViolationKind::ArgocdProjectBootstrappedFromTierA,
                        summary: "Tier-A artifact declares an ArgoCD AppProject — that bootstrap belongs to Tier-B (OpenTofu)".into(),
                    });
                }
            }
            IacTier::TierCClusterApi => {
                // Tier C should not declare per-pod manifests either; reuse the same check.
                for kind in POD_MANIFEST_KINDS {
                    let needle = format!("resource \"{kind}\"");
                    if art.contents.contains(&needle) {
                        violations.push(DisciplineViolation {
                            tier: art.tier,
                            path: art.path.clone(),
                            kind: ViolationKind::OpenTofuDefinesPodManifest,
                            summary: format!(
                                "Tier-C ClusterAPI artifact declares `{kind}` — per-pod manifests belong to Tier-A"
                            ),
                        });
                    }
                }
            }
        }
    }

    violations.sort_by(|a, b| {
        a.tier
            .cmp(&b.tier)
            .then(a.kind.cmp(&b.kind))
            .then(a.path.cmp(&b.path))
    });
    (
        DisciplineReport {
            artifacts_checked: artifacts.len(),
        },
        violations,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn art(tier: IacTier, path: &str, contents: &str) -> IacArtifact {
        IacArtifact {
            tier,
            path: path.into(),
            contents: contents.into(),
        }
    }

    #[test]
    fn clean_tiers_have_no_violations() {
        let cfg = DisciplineConfig::default();
        let arts = vec![
            art(
                IacTier::TierBOpenTofu,
                "tofu/vpc.tofu",
                "resource \"aws_vpc\" \"main\" { cidr_block = \"10.0.0.0/16\" }",
            ),
            art(
                IacTier::TierAArgoCd,
                "argocd/app.yaml",
                "apiVersion: argoproj.io/v1alpha1\nkind: Application\nspec:\n  source: { repoURL: 'oyatie' }",
            ),
            art(
                IacTier::TierCClusterApi,
                "clusterapi/clusterclass.yaml",
                "kind: ClusterClass",
            ),
        ];
        let (rep, viols) = audit(&cfg, &arts);
        assert_eq!(rep.artifacts_checked, 3);
        assert!(viols.is_empty(), "expected clean, got {viols:?}");
    }

    #[test]
    fn opentofu_defining_pod_manifest_is_flagged() {
        let cfg = DisciplineConfig::default();
        let arts = vec![art(
            IacTier::TierBOpenTofu,
            "tofu/wrong.tofu",
            "resource \"kubernetes_deployment\" \"app\" { metadata { name = \"x\" } }",
        )];
        let (_, viols) = audit(&cfg, &arts);
        assert_eq!(viols.len(), 1);
        assert_eq!(viols[0].kind, ViolationKind::OpenTofuDefinesPodManifest);
    }

    #[test]
    fn argocd_app_referencing_aws_iam_role_is_flagged() {
        let cfg = DisciplineConfig::default();
        let arts = vec![art(
            IacTier::TierAArgoCd,
            "argocd/bad.yaml",
            "spec:\n  source:\n    helm:\n      values: |\n        roleArn: arn:aws:iam aws_iam_role/foo",
        )];
        let (_, viols) = audit(&cfg, &arts);
        assert_eq!(viols.len(), 1);
        assert_eq!(
            viols[0].kind,
            ViolationKind::ArgocdAppReferencesCloudPrimitive
        );
    }

    #[test]
    fn argocd_appproject_in_tier_a_is_flagged() {
        let cfg = DisciplineConfig::default();
        let arts = vec![art(
            IacTier::TierAArgoCd,
            "argocd/project.yaml",
            "apiVersion: argoproj.io/v1alpha1\nkind: AppProject\nmetadata: { name: oyatie }",
        )];
        let (_, viols) = audit(&cfg, &arts);
        assert_eq!(viols.len(), 1);
        assert_eq!(
            viols[0].kind,
            ViolationKind::ArgocdProjectBootstrappedFromTierA
        );
    }

    #[test]
    fn terraform_residual_only_flagged_after_window_elapsed() {
        let mut cfg = DisciplineConfig::default();
        let arts = vec![art(
            IacTier::TierBOpenTofu,
            "tofu/main.tofu",
            "source = \"hashicorp/aws\"\n",
        )];
        let (_, viols_before) = audit(&cfg, &arts);
        assert!(
            viols_before.is_empty(),
            "before window: expected no flag, got {viols_before:?}"
        );
        cfg.migration_window_elapsed = true;
        let (_, viols_after) = audit(&cfg, &arts);
        assert_eq!(viols_after.len(), 1);
        assert_eq!(viols_after[0].kind, ViolationKind::TerraformResidual);
    }

    #[test]
    fn tier_c_defining_pod_manifest_is_flagged() {
        let cfg = DisciplineConfig::default();
        let arts = vec![art(
            IacTier::TierCClusterApi,
            "clusterapi/wrong.yaml",
            "resource \"kubernetes_pod\" \"x\" {}",
        )];
        let (_, viols) = audit(&cfg, &arts);
        assert_eq!(viols.len(), 1);
        assert_eq!(viols[0].kind, ViolationKind::OpenTofuDefinesPodManifest);
    }

    #[test]
    fn violations_are_sorted_by_tier_kind_path() {
        let cfg = DisciplineConfig::default();
        let arts = vec![
            art(IacTier::TierAArgoCd, "z-late.yaml", "kind: AppProject"),
            art(
                IacTier::TierBOpenTofu,
                "a-early.tofu",
                "resource \"kubernetes_pod\" \"x\" {}",
            ),
            art(IacTier::TierAArgoCd, "m-middle.yaml", "kind: AppProject"),
        ];
        let (_, viols) = audit(&cfg, &arts);
        assert_eq!(viols.len(), 3);
        // TierAArgoCd < TierBOpenTofu in our enum ordering — Tier A first.
        assert_eq!(viols[0].tier, IacTier::TierAArgoCd);
        assert_eq!(viols[1].tier, IacTier::TierAArgoCd);
        assert_eq!(viols[2].tier, IacTier::TierBOpenTofu);
        assert_eq!(viols[0].path, "m-middle.yaml");
        assert_eq!(viols[1].path, "z-late.yaml");
    }
}
