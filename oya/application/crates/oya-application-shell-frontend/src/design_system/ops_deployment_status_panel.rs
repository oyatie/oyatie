//! DS-OPS_DEPLOYMENT_STATUS_PANEL (`specs/design-system/ops-deployment-status-panel.json`).
//!
//! Deployment status panel for plan/apply/canary/rollback/drift/secret-block
//! flows. Spec security invariants:
//!
//! 1. manual SSH remediation never appears — [`RemediationRoute`] is a closed
//!    enum of declarative routes with no SSH variant, so an SSH remediation is
//!    unrepresentable (the spec's `oya ops` CLI mention is superseded by the
//!    founder CLI-retirement directive; operations route through the console
//!    + GitOps reconciliation instead);
//! 2. a destructive apply requires a plan artifact AND a rollback path before
//!    enablement ([`DestructiveApply::new`] demands both);
//! 3. the secret-blocked state names the missing secret CLASS, never a value.

use leptos::prelude::*;
use serde::Deserialize;

/// Spec `variants`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PanelVariant {
    PlanPreview,
    ApplyRunning,
    Canary,
    Rollback,
    DriftDetected,
    BlockedSecret,
}

impl PanelVariant {
    pub const fn id(self) -> &'static str {
        match self {
            Self::PlanPreview => "plan-preview",
            Self::ApplyRunning => "apply-running",
            Self::Canary => "canary",
            Self::Rollback => "rollback",
            Self::DriftDetected => "drift-detected",
            Self::BlockedSecret => "blocked-secret",
        }
    }
}

/// Spec `states`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PanelState {
    Ready,
    Planning,
    Applying,
    FailedWithCommand,
    RollbackRunning,
    CellHealthWarning,
    ManualSshForbidden,
}

impl PanelState {
    pub const fn announcement(self) -> &'static str {
        match self {
            Self::Ready => "Deployment ready",
            Self::Planning => "Plan in progress",
            Self::Applying => "Apply in progress",
            Self::FailedWithCommand => "Deployment failed; the failing step is shown",
            Self::RollbackRunning => "Rollback in progress",
            Self::CellHealthWarning => "Cluster health warning; inspect non-mutating panel status",
            Self::ManualSshForbidden => {
                "Manual SSH remediation is forbidden; use a declarative route"
            }
        }
    }
}

/// Typed view of `GET /ops/v1/clusters/{cluster_id}/health` from
/// `oya/ops-dashboard-control-center/contracts/openapi/ops-dashboard-control-center.yaml`.
/// It is a read-only adapter for console status; it never shells out to CLIs or
/// encodes provider mutation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClusterHealthLevel {
    Green,
    Yellow,
    Red,
}

impl ClusterHealthLevel {
    const fn label(self) -> &'static str {
        match self {
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Red => "red",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterHealthStatus {
    cluster_id: String,
    observed_at: String,
    health: ClusterHealthLevel,
    signals: Vec<String>,
}

impl ClusterHealthStatus {
    pub fn from_json(value: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(value)
    }

    pub const fn endpoint(&self) -> &'static str {
        "GET /ops/v1/clusters/{cluster_id}/health"
    }

    pub fn cluster_id(&self) -> &str {
        &self.cluster_id
    }

    pub const fn health(&self) -> ClusterHealthLevel {
        self.health
    }

    pub fn to_deployment_status(&self) -> DeploymentStatus {
        let current_step = if self.signals.is_empty() {
            "no cluster health signal".to_owned()
        } else {
            self.signals.join(" · ")
        };

        DeploymentStatus {
            variant: PanelVariant::DriftDetected,
            state: match self.health {
                ClusterHealthLevel::Green => PanelState::Ready,
                ClusterHealthLevel::Yellow | ClusterHealthLevel::Red => {
                    PanelState::CellHealthWarning
                }
            },
            target_environment: self.cluster_id.clone(),
            current_step,
            drift_status: format!(
                "{} · observed {} · typed ops API",
                self.health.label(),
                self.observed_at
            ),
            remediation: Some(RemediationRoute::GitOpsReconcile {
                resource_ref: self.cluster_id.clone(),
            }),
            destructive_apply: None,
        }
    }
}

/// Invariant 1: every remediation is declarative. No SSH variant exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemediationRoute {
    /// Infrastructure plan/apply through the OpenTofu pipeline.
    OpenTofuPlan { plan_ref: String },
    /// K8s-native reconciliation (GitOps; operators converge the state).
    GitOpsReconcile { resource_ref: String },
    /// A console-surfaced operation backed by the platform API.
    ConsoleOperation { operation_id: String },
}

impl RemediationRoute {
    pub fn label(&self) -> String {
        match self {
            Self::OpenTofuPlan { plan_ref } => format!("OpenTofu plan {plan_ref}"),
            Self::GitOpsReconcile { resource_ref } => format!("GitOps reconcile {resource_ref}"),
            Self::ConsoleOperation { operation_id } => format!("Console operation {operation_id}"),
        }
    }
}

/// Invariant 2: constructing an enabled destructive apply requires BOTH the
/// plan artifact and the rollback path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DestructiveApply {
    plan_artifact: String,
    rollback_path: String,
}

impl DestructiveApply {
    pub fn new(plan_artifact: String, rollback_path: String) -> Result<Self, &'static str> {
        if plan_artifact.trim().is_empty() {
            return Err("destructive apply requires a plan artifact");
        }
        if rollback_path.trim().is_empty() {
            return Err("destructive apply requires a rollback path");
        }
        Ok(Self {
            plan_artifact,
            rollback_path,
        })
    }

    pub fn plan_artifact(&self) -> &str {
        &self.plan_artifact
    }

    pub fn rollback_path(&self) -> &str {
        &self.rollback_path
    }
}

/// Invariant 3: the blocked-secret message names the class only; no value
/// parameter exists to leak.
pub fn secret_blocked_message(secret_class: &str) -> String {
    format!("Deployment blocked: missing secret of class `{secret_class}`")
}

/// One status card in the panel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeploymentStatus {
    pub variant: PanelVariant,
    pub state: PanelState,
    pub target_environment: String,
    pub current_step: String,
    pub drift_status: String,
    pub remediation: Option<RemediationRoute>,
    /// Present only when a destructive apply is staged (and therefore valid).
    pub destructive_apply: Option<DestructiveApply>,
}

/// WCAG 2.2 AA panel: status cards, logs, remediation links, and rollback
/// controls are native focusable elements; phase, command, target
/// environment, drift status, and rollback availability are announced via
/// `aria-live`.
#[component]
pub fn OpsDeploymentStatusPanel(status: DeploymentStatus) -> impl IntoView {
    let announcement = status.state.announcement();
    let rollback_available =
        status.destructive_apply.is_some() || matches!(status.variant, PanelVariant::Rollback);
    view! {
        <section
            class="ds-ops-deployment-status-panel"
            data-variant=status.variant.id()
            aria-label="Deployment status panel"
        >
            <p role="status" aria-live="polite">{announcement}</p>
            <dl>
                <div>
                    <dt>"Target environment"</dt>
                    <dd>{status.target_environment.clone()}</dd>
                </div>
                <div>
                    <dt>"Current step"</dt>
                    <dd>{status.current_step.clone()}</dd>
                </div>
                <div>
                    <dt>"Drift"</dt>
                    <dd>{status.drift_status.clone()}</dd>
                </div>
                <div>
                    <dt>"Rollback available"</dt>
                    <dd>{rollback_available.to_string()}</dd>
                </div>
            </dl>
            {status
                .remediation
                .as_ref()
                .map(|route| view! {
                    <button type="button" class="ds-remediation-route">{route.label()}</button>
                })}
            {status
                .destructive_apply
                .as_ref()
                .map(|apply| view! {
                    <div class="ds-destructive-apply">
                        <span>{format!("Plan artifact: {}", apply.plan_artifact())}</span>
                        <span>{format!("Rollback path: {}", apply.rollback_path())}</span>
                        <button type="button" class="destructive">"Apply with rollback staged"</button>
                    </div>
                })}
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_ssh_remediation_is_unrepresentable() {
        // Closed enum: every possible route is declarative. Guard the labels
        // so no route ever renders ssh instructions.
        let routes = [
            RemediationRoute::OpenTofuPlan {
                plan_ref: "plan/2026-06-10".to_owned(),
            },
            RemediationRoute::GitOpsReconcile {
                resource_ref: "deployments/oya-shell".to_owned(),
            },
            RemediationRoute::ConsoleOperation {
                operation_id: "op/rollback-42".to_owned(),
            },
        ];
        for route in routes {
            let label = route.label().to_ascii_lowercase();
            assert!(!label.contains("ssh"), "{label}");
        }
        assert_eq!(
            PanelState::ManualSshForbidden.announcement(),
            "Manual SSH remediation is forbidden; use a declarative route"
        );
    }

    #[test]
    fn destructive_apply_requires_plan_and_rollback() {
        assert!(DestructiveApply::new(String::new(), "rollback/1".to_owned()).is_err());
        assert!(DestructiveApply::new("plan/1".to_owned(), "  ".to_owned()).is_err());
        let staged = DestructiveApply::new("plan/1".to_owned(), "rollback/1".to_owned()).unwrap();
        assert_eq!(staged.plan_artifact(), "plan/1");
        assert_eq!(staged.rollback_path(), "rollback/1");
    }

    #[test]
    fn secret_blocked_names_class_never_value() {
        let message = secret_blocked_message("database-credentials");
        assert!(message.contains("database-credentials"));
        // The API takes only the class; there is no value to leak. Guard the
        // copy shape anyway.
        assert!(message.starts_with("Deployment blocked: missing secret of class"));
    }

    #[test]
    fn ops_cluster_health_fixture_maps_to_non_mutating_panel_status() {
        let fixture = r#"{
            "cluster_id": "cell-us-east-2",
            "observed_at": "2026-07-01T05:00:00Z",
            "health": "yellow",
            "signals": [
                "argocd-app-health degraded",
                "cosign verify ok",
                "traceparent fixture only"
            ]
        }"#;

        let api_status =
            ClusterHealthStatus::from_json(fixture).expect("typed cluster health JSON");
        let panel = api_status.to_deployment_status();

        assert_eq!(
            api_status.endpoint(),
            "GET /ops/v1/clusters/{cluster_id}/health"
        );
        assert_eq!(api_status.cluster_id(), "cell-us-east-2");
        assert_eq!(api_status.health(), ClusterHealthLevel::Yellow);
        assert_eq!(panel.variant, PanelVariant::DriftDetected);
        assert_eq!(panel.state, PanelState::CellHealthWarning);
        assert_eq!(panel.target_environment, "cell-us-east-2");
        assert_eq!(
            panel.current_step,
            "argocd-app-health degraded · cosign verify ok · traceparent fixture only"
        );
        assert_eq!(
            panel.drift_status,
            "yellow · observed 2026-07-01T05:00:00Z · typed ops API"
        );
        assert!(panel.destructive_apply.is_none());
    }

    #[test]
    fn ops_cluster_health_rejects_unknown_contract_fields() {
        let fixture = r#"{
            "cluster_id": "cell-us-east-2",
            "observed_at": "2026-07-01T05:00:00Z",
            "health": "yellow",
            "signals": ["argocd-app-health degraded"],
            "provider_mutation_hint": "ssh into the node"
        }"#;

        assert!(ClusterHealthStatus::from_json(fixture).is_err());
    }
}
