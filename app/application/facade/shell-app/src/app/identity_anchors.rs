use super::*;

pub(super) fn workforce_anchor_board() -> impl IntoView {
    view! {
        <div class="workforce-anchor-grid" aria-label="FD-001 and Oyatie Cloud workforce proof">
            <article class="workforce-anchor-card selected" data-workforce-card="fd001">
                <p class="screen-anchor">"FD-001 WORKFORCE"</p>
                <h5>"People data powers product delivery"</h5>
                <p>
                    "Employee directory, payroll eligibility, reviewer mail, community announcements, and onboarding "
                    "are FD-001 tenant workloads, not separate HR widgets."
                </p>
                <div class="workforce-anchor-actions">
                    <button type="button" data-workforce-anchor-action="route-payroll">"Payroll impact"</button>
                    <button type="button" data-workforce-anchor-action="route-workflow">"Workflow path"</button>
                </div>
            </article>
            <article class="workforce-anchor-card" data-workforce-card="cloud">
                <p class="screen-anchor">"OYATIE CLOUD"</p>
                <h5>"Hosted as a governed tenant surface"</h5>
                <p>
                    "PIPA boundaries, regional pack gates, role envelopes, audit receipts, and evidence routes prove "
                    "the substrate can host real workforce tenants."
                </p>
                <div class="workforce-anchor-actions">
                    <button type="button" data-workforce-anchor-action="route-policy">"Policy envelope"</button>
                    <button type="button" data-workforce-anchor-action="route-audit">"Audit trail"</button>
                </div>
            </article>
            <article class="workforce-anchor-card" data-workforce-card="lifecycle">
                <p class="screen-anchor">"LIFECYCLE OPS"</p>
                <h5>"Interactive, local-only employee command"</h5>
                <p>
                    "Operators can inspect people, stage invites, route leave/time, and brief reviewers while HRIS, "
                    "auth, payroll, and cloud mutations remain disconnected."
                </p>
                <div class="workforce-anchor-actions">
                    <button type="button" data-workforce-anchor-action="stage-invite">"Stage invite"</button>
                    <button type="button" data-workforce-anchor-action="route-leave">"Leave & time"</button>
                </div>
            </article>
        </div>
        <div class="workforce-anchor-footer">
            <span data-workforce-anchor-status="true">"Employees ready · FD-001 workforce workload dogfoods Oyatie Cloud locally."</span>
            <div class="workforce-anchor-routes" aria-label="Workforce connected routes">
                <button type="button" data-workforce-anchor-action="route-mail">"Reviewer Mail"</button>
                <button type="button" data-workforce-anchor-action="route-community">"Community update"</button>
                <button type="button" data-workforce-anchor-action="route-evidence">"Evidence graph"</button>
                <button type="button" data-workforce-anchor-action="route-cloud">"Cloud cells"</button>
            </div>
        </div>
    }
}

pub(super) fn onboarding_anchor_board() -> impl IntoView {
    view! {
        <div class="onboarding-anchor-grid" aria-label="FD-001 tenant admission setup proof">
            <article class="onboarding-anchor-card selected" data-onboarding-card="tenant">
                <p class="screen-anchor">"FD-001 TENANT ADMISSION"</p>
                <h5>"Product workload setup path"</h5>
                <p>
                    "Legal profile, payroll calendar, employee import, policy gates, Mail reviewers, Community launch notes, "
                    "and evidence receipts become one tenant setup packet."
                </p>
                <div class="onboarding-anchor-actions">
                    <button type="button" data-onboarding-anchor-action="route-tasks">"Today queue"</button>
                    <button type="button" data-onboarding-anchor-action="import-employees">"Import people"</button>
                </div>
            </article>
            <article class="onboarding-anchor-card" data-onboarding-card="cloud">
                <p class="screen-anchor">"OYATIE CLOUD"</p>
                <h5>"Substrate readiness before go-live"</h5>
                <p>
                    "Region pack, PIPA boundary, role envelope, deployment gates, audit freshness, and rollback posture "
                    "prove the tenant can be hosted safely."
                </p>
                <div class="onboarding-anchor-actions">
                    <button type="button" data-onboarding-anchor-action="route-cloud">"Cloud cells"</button>
                    <button type="button" data-onboarding-anchor-action="route-policy">"Policy gate"</button>
                </div>
            </article>
            <article class="onboarding-anchor-card" data-onboarding-card="launch">
                <p class="screen-anchor">"LAUNCH PACKET"</p>
                <h5>"Interactive, local-only setup"</h5>
                <p>
                    "Operators can advance setup, draft reviewer mail, post a community note, and attach evidence while "
                    "registries, HRIS, payroll, auth, and cloud mutations remain disconnected."
                </p>
                <div class="onboarding-anchor-actions">
                    <button type="button" data-onboarding-anchor-action="advance-setup">"Advance setup"</button>
                    <button type="button" data-onboarding-anchor-action="route-evidence">"Evidence"</button>
                </div>
            </article>
        </div>
        <div class="onboarding-anchor-footer">
            <span data-onboarding-anchor-status="true">
                "Onboarding ready · FD-001 tenant setup dogfoods Oyatie Cloud locally."
            </span>
            <div class="onboarding-anchor-routes" aria-label="Onboarding connected routes">
                <button type="button" data-onboarding-anchor-action="route-payroll">"Payroll calendar"</button>
                <button type="button" data-onboarding-anchor-action="route-mail">"Reviewer Mail"</button>
                <button type="button" data-onboarding-anchor-action="route-community">"Community launch"</button>
                <button type="button" data-onboarding-anchor-action="route-schedule">"Schedule"</button>
            </div>
        </div>
    }
}

pub(super) fn identity_sessions_anchor_board() -> impl IntoView {
    view! {
        <div class="identity-anchor-grid identity-sessions-anchor" aria-label="Session tenant proof">
            <article class="identity-anchor-card selected" data-identity-anchor-card="sessions">
                <p class="screen-anchor">"FD-001 SESSION PROOF"</p>
                <h5>"Auth sessions protect product workloads"</h5>
                <p>"Passkey, device, and payroll-role activity are evidence leaves for FD-001 tenant services."</p>
                <div class="identity-anchor-actions">
                    <button type="button" data-identity-anchor-action="route-roles">"Role envelope"</button>
                    <button type="button" data-identity-anchor-action="route-evidence">"Evidence graph"</button>
                </div>
            </article>
            <article class="identity-anchor-card" data-identity-anchor-card="cloud">
                <p class="screen-anchor">"OYATIE CLOUD"</p>
                <h5>"Oyatie Cloud tenant session posture"</h5>
                <p>"Device locality, PIPA-safe audit, and session freshness prove the substrate can host workforce tenants."</p>
                <div class="identity-anchor-actions">
                    <button type="button" data-identity-anchor-action="route-cloud">"Cloud cells"</button>
                    <button type="button" data-identity-anchor-action="route-policy">"PIPA policy"</button>
                </div>
            </article>
        </div>
        <div class="identity-anchor-footer">
            <span data-identity-anchor-status="true">"Sessions ready · local-only identity telemetry."</span>
            <div class="identity-anchor-routes">
                <button type="button" data-identity-anchor-action="route-mail">"Reviewer Mail"</button>
                <button type="button" data-identity-anchor-action="route-audit">"Audit ledger"</button>
            </div>
        </div>
    }
}

pub(super) fn identity_roles_anchor_board() -> impl IntoView {
    view! {
        <div class="identity-anchor-grid identity-roles-anchor" aria-label="Role envelope proof">
            <article class="identity-anchor-card selected" data-identity-anchor-card="roles">
                <p class="screen-anchor">"FD-001 ROLE ENVELOPE"</p>
                <h5>"Access controls every product workload"</h5>
                <p>"Payroll, filing, workflow, Mail, Community, and cloud operations share one role envelope."</p>
                <div class="identity-anchor-actions">
                    <button type="button" data-identity-anchor-action="review-roles">"Review grants"</button>
                    <button type="button" data-identity-anchor-action="route-workflow">"Workflow gate"</button>
                </div>
            </article>
            <article class="identity-anchor-card" data-identity-anchor-card="pipa">
                <p class="screen-anchor">"OYATIE CLOUD POLICY"</p>
                <h5>"Oyatie Cloud PIPA-safe tenant boundary"</h5>
                <p>"Role decisions stay auditable before any tenant workload can move through cloud admission gates."</p>
                <div class="identity-anchor-actions">
                    <button type="button" data-identity-anchor-action="route-policy">"Policy board"</button>
                    <button type="button" data-identity-anchor-action="route-cloud">"Cloud gate"</button>
                </div>
            </article>
            <article class="identity-anchor-card" data-identity-anchor-card="local">
                <p class="screen-anchor">"LOCAL ONLY"</p>
                <h5>"Interactive access preview"</h5>
                <p>"Grant reviews, denial traces, and reviewer routes update visual state only; no SSO or IAM mutation runs."</p>
                <div class="identity-anchor-actions">
                    <button type="button" data-identity-anchor-action="route-evidence">"Evidence"</button>
                    <button type="button" data-identity-anchor-action="route-community">"Community note"</button>
                </div>
            </article>
        </div>
        <div class="identity-anchor-footer">
            <span data-identity-anchor-status="true">"Roles ready · FD-001 access envelope dogfoods Oyatie Cloud locally."</span>
            <div class="identity-anchor-routes">
                <button type="button" data-identity-anchor-action="route-payroll">"Payroll close"</button>
                <button type="button" data-identity-anchor-action="route-audit">"Audit packet"</button>
            </div>
        </div>
    }
}

pub(super) fn identity_org_anchor_board() -> impl IntoView {
    view! {
        <div class="identity-anchor-grid identity-org-anchor" aria-label="Organization tenant proof">
            <article class="identity-anchor-card selected" data-identity-anchor-card="org">
                <p class="screen-anchor">"FD-001 ORG PROFILE"</p>
                <h5>"Corporate facts feed every module"</h5>
                <p>"Legal profile, payroll calendar, tax identifiers, billing, and employee facts become shared tenant context."</p>
                <div class="identity-anchor-actions">
                    <button type="button" data-identity-anchor-action="route-onboarding">"Setup packet"</button>
                    <button type="button" data-identity-anchor-action="route-payroll">"Payroll calendar"</button>
                </div>
            </article>
            <article class="identity-anchor-card" data-identity-anchor-card="cloud">
                <p class="screen-anchor">"OYATIE CLOUD TENANT"</p>
                <h5>"Oyatie Cloud hosted profile readiness"</h5>
                <p>"Region packs, audit receipts, deployment gates, and evidence spine prove this tenant can be hosted."</p>
                <div class="identity-anchor-actions">
                    <button type="button" data-identity-anchor-action="route-cloud">"Cloud cells"</button>
                    <button type="button" data-identity-anchor-action="route-evidence">"Evidence spine"</button>
                </div>
            </article>
        </div>
        <div class="identity-anchor-footer">
            <span data-identity-anchor-status="true">"Organization ready · local-only tenant profile preview."</span>
            <div class="identity-anchor-routes">
                <button type="button" data-identity-anchor-action="route-mail">"Reviewer Mail"</button>
                <button type="button" data-identity-anchor-action="route-community">"Community launch"</button>
            </div>
        </div>
    }
}
