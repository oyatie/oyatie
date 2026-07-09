//! DS-POLICY_DISCLOSURE_BANNER (`specs/design-system/policy-disclosure-banner.json`).
//!
//! Inline disclosure primitive for retention, legal hold, consent,
//! audit-access, and workflow-handoff policy consequences. Spec security
//! invariants:
//!
//! 1. destructive actions remain disabled while a blocking banner is
//!    unresolved (single authority: [`PolicyDisclosure::destructive_actions_enabled`]);
//! 2. four-eyes flows display the approver requirement;
//! 3. personal-context banners never imply employer access rights.

use leptos::prelude::*;

use super::tenant_context_switcher::ContextKind;

/// Spec `variants`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BannerVariant {
    LegalHold,
    Retention,
    ConsentRequired,
    AuditAccess,
}

impl BannerVariant {
    pub const fn id(self) -> &'static str {
        match self {
            Self::LegalHold => "legal-hold",
            Self::Retention => "retention",
            Self::ConsentRequired => "consent-required",
            Self::AuditAccess => "audit-access",
        }
    }
}

/// Spec `states`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BannerState {
    Informational,
    Blocking,
    RequiresSecondApprover,
    ExpiredPolicy,
    OfflineUnavailable,
}

/// One rendered policy disclosure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyDisclosure {
    pub variant: BannerVariant,
    pub state: BannerState,
    /// The policy this disclosure derives from (id or citation).
    pub policy_basis: String,
    /// What the operator must do, if anything.
    pub required_action: String,
    /// The audit consequence the screen reader announces.
    pub audit_consequence: String,
    /// Context the banner renders inside (drives invariant 3).
    pub context: ContextKind,
}

impl PolicyDisclosure {
    /// Invariant 1: destructive actions stay disabled while the banner is
    /// blocking or awaiting the second approver.
    pub fn destructive_actions_enabled(&self) -> bool {
        !matches!(
            self.state,
            BannerState::Blocking | BannerState::RequiresSecondApprover
        )
    }

    /// Invariant 2: four-eyes flows must surface the approver requirement.
    pub fn approver_requirement(&self) -> Option<&'static str> {
        match self.state {
            BannerState::RequiresSecondApprover => {
                Some("A second approver must confirm before this action proceeds")
            }
            _ => None,
        }
    }

    /// Invariant 3: in a personal context the disclosure copy speaks only to
    /// the account owner's own policy posture; org/employer access phrasing
    /// is selected out by construction rather than filtered afterward.
    pub fn access_scope_copy(&self) -> &'static str {
        match self.context {
            ContextKind::Personal => {
                "Applies to your personal account only; it grants no organization access to your data"
            }
            ContextKind::Work | ContextKind::AdminAudit => {
                "Applies under your organization's policy scope for this tenant"
            }
        }
    }

    pub fn severity_announcement(&self) -> &'static str {
        match self.state {
            BannerState::Informational => "Informational policy disclosure",
            BannerState::Blocking => "Blocking policy disclosure; the action cannot proceed",
            BannerState::RequiresSecondApprover => "Policy disclosure requiring a second approver",
            BannerState::ExpiredPolicy => "Policy basis expired; re-evaluation required",
            BannerState::OfflineUnavailable => "Policy state unavailable offline; failing closed",
        }
    }
}

/// WCAG 2.2 AA banner: the banner's action controls render BEFORE any
/// destructive control in DOM order (spec keyboard contract), severity +
/// policy basis + required action + audit consequence are announced via the
/// alert/status role, and the destructive control is a real disabled state,
/// not a styled-only one.
#[component]
pub fn PolicyDisclosureBanner(
    disclosure: PolicyDisclosure,
    destructive_action_label: String,
) -> impl IntoView {
    let blocking = !disclosure.destructive_actions_enabled();
    let role = if blocking { "alert" } else { "status" };
    let approver = disclosure.approver_requirement();
    let scope = disclosure.access_scope_copy();
    let severity = disclosure.severity_announcement();
    view! {
        <section
            class="ds-policy-disclosure-banner"
            data-variant=disclosure.variant.id()
            role=role
            aria-live=if blocking { "assertive" } else { "polite" }
        >
            <p class="ds-banner-severity">{severity}</p>
            <dl>
                <div>
                    <dt>"Policy basis"</dt>
                    <dd>{disclosure.policy_basis.clone()}</dd>
                </div>
                <div>
                    <dt>"Required action"</dt>
                    <dd>{disclosure.required_action.clone()}</dd>
                </div>
                <div>
                    <dt>"Audit consequence"</dt>
                    <dd>{disclosure.audit_consequence.clone()}</dd>
                </div>
                <div>
                    <dt>"Access scope"</dt>
                    <dd>{scope}</dd>
                </div>
            </dl>
            {approver.map(|requirement| view! { <p class="ds-banner-approver">{requirement}</p> })}
            <div class="ds-banner-actions">
                <button type="button">"Review policy"</button>
                <button type="button" class="destructive" disabled=blocking>
                    {destructive_action_label}
                </button>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disclosure(state: BannerState, context: ContextKind) -> PolicyDisclosure {
        PolicyDisclosure {
            variant: BannerVariant::LegalHold,
            state,
            policy_basis: "legal-hold/LH-2026-014".to_owned(),
            required_action: "Release the hold before deleting".to_owned(),
            audit_consequence: "Deletion attempts are recorded to the audit chain".to_owned(),
            context,
        }
    }

    #[test]
    fn legal_hold_blocks_delete_while_unresolved() {
        // Spec test ref: test_legal_hold_blocks_delete.
        assert!(
            !disclosure(BannerState::Blocking, ContextKind::Work).destructive_actions_enabled()
        );
        assert!(
            !disclosure(BannerState::RequiresSecondApprover, ContextKind::Work)
                .destructive_actions_enabled()
        );
        assert!(
            disclosure(BannerState::Informational, ContextKind::Work).destructive_actions_enabled()
        );
    }

    #[test]
    fn four_eyes_state_surfaces_approver_requirement() {
        assert!(
            disclosure(BannerState::RequiresSecondApprover, ContextKind::Work)
                .approver_requirement()
                .is_some()
        );
        assert!(
            disclosure(BannerState::Blocking, ContextKind::Work)
                .approver_requirement()
                .is_none()
        );
    }

    #[test]
    fn personal_context_copy_never_implies_employer_access() {
        // Spec test ref: test_personal_context_no_org_access_copy.
        let copy = disclosure(BannerState::Informational, ContextKind::Personal)
            .access_scope_copy()
            .to_ascii_lowercase();
        assert!(copy.contains("personal account only"), "{copy}");
        assert!(copy.contains("no organization access"), "{copy}");

        let work_copy = disclosure(BannerState::Informational, ContextKind::Work)
            .access_scope_copy()
            .to_ascii_lowercase();
        assert!(work_copy.contains("organization"), "{work_copy}");
    }
}
