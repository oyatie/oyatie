use super::*;

pub(super) fn identity_command_board() -> impl IntoView {
    view! {
        <div class="identity-command-board" aria-label="Identity command center">
            <section class="identity-command-card identity-spine-card">
                <div class="identity-command-card-head">
                    <div>
                        <p class="screen-anchor">"ACCESS SPINE"</p>
                        <h5>"One governed identity path from auth to payroll close"</h5>
                    </div>
                    <span class="status-chip success">"PIPA-safe"</span>
                </div>
                <div class="identity-spine-flow" aria-label="Identity access lineage">
                    <span class="active"><em>"01"</em><strong>"Passkey"</strong><small>"verified"</small></span>
                    <i></i>
                    <span><em>"02"</em><strong>"Session"</strong><small>"current device"</small></span>
                    <i></i>
                    <span class="review"><em>"03"</em><strong>"Role"</strong><small>"payroll review"</small></span>
                    <i></i>
                    <span><em>"04"</em><strong>"Employee"</strong><small>"118 records"</small></span>
                    <i></i>
                    <span class="review"><em>"05"</em><strong>"Workflow"</strong><small>"2-person gate"</small></span>
                    <i></i>
                    <span class="sealed"><em>"06"</em><strong>"Audit"</strong><small>"REC-ID-2026-05"</small></span>
                </div>
                <dl class="identity-command-kv">
                    <div><dt>"Autonomy ceiling"</dt><dd>"No auth mutation · local preview only"</dd></div>
                    <div><dt>"Primary risk"</dt><dd>"Payroll approver role expires before close"</dd></div>
                    <div><dt>"Connected route"</dt><dd>"Workflow → Mail → Evidence Spine"</dd></div>
                </dl>
            </section>

            <section class="identity-command-card identity-risk-card">
                <div class="identity-command-card-head">
                    <div>
                        <p class="screen-anchor">"RISK QUEUE"</p>
                        <h5>"Access work that affects today’s operations"</h5>
                    </div>
                    <button type="button" data-identity-route-action="evidence">"Evidence"</button>
                </div>
                <div class="identity-risk-list" role="list" aria-label="Identity risk queue">
                    <article role="listitem" data-identity-risk-row="review">
                        <span class="status-chip warning">"review"</span>
                        <strong>"Payroll approver recertification"</strong>
                        <p>"CFO role grants payroll close and HomeTax transport; 2-person review due today."</p>
                        <small>"Owner CFO · SLA 4.0h · REC-ID-2026-05"</small>
                    </article>
                    <article role="listitem" data-identity-risk-row="blocking">
                        <span class="status-chip danger">"blocking"</span>
                        <strong>"Vendor guest cannot view employee PII"</strong>
                        <p>"Stripe renewal route needs procurement context without exposing workforce records."</p>
                        <small>"Owner Security reviewer · policy POL-PII-014"</small>
                    </article>
                    <article role="listitem" data-identity-risk-row="sealed">
                        <span class="status-chip success">"sealed"</span>
                        <strong>"Passkey challenge evidence sealed"</strong>
                        <p>"MacBook and iPhone passkey state available to audit, not external auth writes."</p>
                        <small>"Source local island · 09:14 KST"</small>
                    </article>
                </div>
            </section>

            <section class="identity-command-card identity-lifecycle-card">
                <div class="identity-command-card-head">
                    <div>
                        <p class="screen-anchor">"WORKFORCE LIFECYCLE"</p>
                        <h5>"Employees, onboarding, roles, and payroll impact"</h5>
                    </div>
                    <button type="button" data-identity-route-action="employees">"Open people"</button>
                </div>
                // A-6: role=progressbar + aria-value*
                <div class="identity-lifecycle-grid" aria-label="Workforce lifecycle state">
                    <span role="progressbar" aria-valuenow="77" aria-valuemin="0" aria-valuemax="100" aria-label="Onboarding: 77%" style="--bar: 77%"><strong aria-hidden="true">"Onboarding"</strong><em aria-hidden="true">"6 active · 77%"</em></span>
                    <span role="progressbar" aria-valuenow="64" aria-valuemin="0" aria-valuemax="100" aria-label="Role review: 64%" style="--bar: 64%"><strong aria-hidden="true">"Role review"</strong><em aria-hidden="true">"14 grants · 64%"</em></span>
                    <span role="progressbar" aria-valuenow="48" aria-valuemin="0" aria-valuemax="100" aria-label="Session hygiene: 48%" style="--bar: 48%"><strong aria-hidden="true">"Session hygiene"</strong><em aria-hidden="true">"3 stale · 48%"</em></span>
                    <span role="progressbar" aria-valuenow="83" aria-valuemin="0" aria-valuemax="100" aria-label="Payroll readiness: 83%" style="--bar: 83%"><strong aria-hidden="true">"Payroll readiness"</strong><em aria-hidden="true">"109 active · 83%"</em></span>
                </div>
            </section>

            <section class="identity-command-card identity-route-card">
                <div class="identity-command-card-head">
                    <div>
                        <p class="screen-anchor">"ROUTE MATRIX"</p>
                        <h5>"Every identity action lands inside the same service graph"</h5>
                    </div>
                </div>
                <div class="identity-route-grid" aria-label="Identity local routes">
                    <button type="button" data-identity-route-action="workflow">"Workflow gate"</button>
                    <button type="button" data-identity-route-action="mail">"Mail reviewer"</button>
                    <button type="button" data-identity-route-action="sessions">"Session audit"</button>
                    <button type="button" data-identity-route-action="onboarding">"Setup checklist"</button>
                    <button type="button" data-identity-route-action="finance">"Payroll close"</button>
                    <button type="button" data-identity-route-action="evidence">"Evidence spine"</button>
                </div>
                <p class="identity-command-note">"Routes change local visual state only; no SSO, HRIS, payroll, or directory backend is wired."</p>
            </section>
        </div>
    }
}
