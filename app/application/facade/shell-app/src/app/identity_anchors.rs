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
            <span data-workforce-anchor-status="true">"Sample workforce preview · employee readiness unavailable."</span>
            <div class="workforce-anchor-routes" aria-label="Workforce connected routes">
                <button type="button" data-workforce-anchor-action="route-mail">"Reviewer Mail"</button>
                <button type="button" data-workforce-anchor-action="route-community">"Community update"</button>
                <button type="button" data-workforce-anchor-action="route-evidence">"Evidence graph"</button>
                <button type="button" data-workforce-anchor-action="route-cloud">"Cloud cells"</button>
            </div>
        </div>
    }
}

pub(super) fn onboarding_anchor_html() -> String {
    r##"<div class="onboarding-anchor-grid" aria-label="Sample setup destinations"><article class="onboarding-anchor-card"><p class="screen-anchor">SAMPLE SETUP</p><h5>Inspect product workload examples</h5><p>Legal, calendar, people and policy examples are not a tenant admission packet.</p><div class="onboarding-anchor-actions"><a href="#tasks-title" data-account-route="tasks" data-account-focus="setup-tasks">Tasks sample</a><button type="button" data-onboarding-anchor-action="import-employees" data-account-focus="setup-import">Inspect HR sample; import unavailable</button></div></article><article class="onboarding-anchor-card"><p class="screen-anchor">CLOUD AND POLICY EXAMPLES</p><h5>Inspect existing sample destinations</h5><p>Live hosting, privacy, deployment and readiness remain Unknown / unavailable.</p><div class="onboarding-anchor-actions"><button type="button" data-onboarding-anchor-action="route-cloud" data-account-focus="setup-cloud">Cloud sample</button><button type="button" data-onboarding-anchor-action="route-policy" data-account-focus="setup-policy">Policy sample</button></div></article><article class="onboarding-anchor-card"><p class="screen-anchor">PAGE MEMORY ONLY</p><h5>Complete illustrative steps locally</h5><p>No reviewer draft, community publication, evidence receipt, import or go-live is created.</p><div class="onboarding-anchor-actions"><button type="button" data-onboarding-anchor-action="advance-setup" data-account-focus="setup-anchor-advance">Complete sample step</button><button type="button" data-onboarding-anchor-action="route-evidence" data-account-focus="setup-evidence">Evidence sample</button></div></article></div><div class="onboarding-anchor-footer"><span data-onboarding-anchor-status="true" data-account-local="anchor-status" role="status">Sample setup only; no tenant admission or readiness occurred.</span><div class="onboarding-anchor-routes" aria-label="Sample setup routes"><button type="button" data-onboarding-anchor-action="route-payroll" data-account-focus="setup-payroll">Payroll sample</button><button type="button" data-onboarding-anchor-action="route-mail" data-account-focus="setup-mail">Mail sample</button><button type="button" data-onboarding-anchor-action="route-community" data-account-focus="setup-community">Community sample</button><a href="#schedule-title" data-account-route="schedule" data-account-focus="setup-schedule">Schedule sample</a></div></div>"##.to_owned()
}

pub(super) fn identity_anchor_html(panel: &str) -> String {
    let inspect = match panel {
        "sessions" => {
            r#"<button type="button" data-identity-anchor-action="route-roles" data-account-focus="device-roles">Inspect sample roles</button>"#
        }
        "roles" => {
            r#"<button type="button" data-identity-anchor-action="review-roles" data-account-focus="review-grants">Review sample grants</button><button type="button" data-identity-anchor-action="route-workflow" data-account-focus="roles-workflow">Workflow sample</button>"#
        }
        "org" => {
            r#"<button type="button" data-identity-anchor-action="route-onboarding" data-account-focus="org-setup">Inspect sample setup</button>"#
        }
        _ => panic!("unsupported account anchor"),
    };
    format!(
        r#"<div class="identity-anchor-grid identity-{panel}-anchor" aria-label="Sample {panel} connected destinations"><article class="identity-anchor-card"><p class="screen-anchor">SAMPLE ACCOUNT CONTEXT</p><h5>Illustrative {panel} facts</h5><p>These fixed examples supply no verified account, privacy, hosting or policy result.</p><div class="identity-anchor-actions">{inspect}<button type="button" data-identity-anchor-action="route-evidence" data-account-focus="{panel}-evidence">Evidence sample</button></div></article><article class="identity-anchor-card"><p class="screen-anchor">EXISTING SAMPLE DESTINATIONS</p><h5>Inspect cloud and policy examples</h5><p>Live authority is Unknown / unavailable.</p><div class="identity-anchor-actions"><button type="button" data-identity-anchor-action="route-cloud" data-account-focus="{panel}-cloud">Cloud sample</button><button type="button" data-identity-anchor-action="route-policy" data-account-focus="{panel}-policy">Policy sample</button></div></article></div><div class="identity-anchor-footer"><span data-identity-anchor-status="true" data-account-local="{panel}-status" role="status">Sample inspection only; no live authority supplied.</span><div class="identity-anchor-routes" aria-label="Sample {panel} routes"><button type="button" data-identity-anchor-action="route-mail" data-account-focus="{panel}-mail">Mail sample</button><button type="button" data-identity-anchor-action="route-community" data-account-focus="{panel}-community">Community sample</button><button type="button" data-identity-anchor-action="route-payroll" data-account-focus="{panel}-payroll">Payroll sample</button><button type="button" data-identity-anchor-action="route-audit" data-account-focus="{panel}-audit">Audit sample</button></div></div>"#
    )
}
