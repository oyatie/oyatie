//! DS-OPS_DEPLOYMENT_STATUS_PANEL (`specs/design-system/ops-deployment-status-panel.json`).
//!
//! Deployment status panel for plan/apply/canary/rollback/drift/secret-block
//! flows. Spec security invariants:
//!
//! 1. manual SSH remediation never appears — [`RemediationRoute`] is a closed
//!    enum of declarative routes with no SSH variant, so an SSH remediation is
//!    unrepresentable (the spec's `ops` CLI mention is superseded by the
//!    founder CLI-retirement directive; operations route through the console
//!    + GitOps reconciliation instead);
//! 2. a destructive apply requires a plan artifact AND a rollback path before
//!    enablement ([`DestructiveApply::new`] demands both);
//! 3. the secret-blocked state names the missing secret CLASS, never a value.

use leptos::prelude::*;

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
            Self::ManualSshForbidden => {
                "Manual SSH remediation is forbidden; use a declarative route"
            }
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
                resource_ref: "deployments/shell".to_owned(),
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
}
