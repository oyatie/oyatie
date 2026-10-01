use super::*;

pub(super) fn tenant_rbac_board(envelope: TenantRenderEnvelope) -> impl IntoView {
    view! {
        <section
            class="tenant-rbac-board"
            aria-labelledby="service-board-title"
            id="business-logics"
        >
            <div class="service-board-head">
                <div>
                    <p class="screen-anchor">"TENANT RBAC SERVICE GRAPH"</p>
                    <h3 id="service-board-title">"Corporate operations graph"</h3>
                </div>
                <span class="status-chip success">{format!("{} permitted services", envelope.modules.len())}</span>
            </div>

            {business_logic_os_panel()}
            {governance_ops_command_board(envelope.workflow.name.clone())}

            <article id="payroll-cockpit" class="service-card payroll-card">
                <p class="screen-anchor">"PAYROLL CLOSE"</p>
                <h4>"2026-04 payroll close"</h4>
                <p class="service-card-brief">"FD-001 payroll workload dogfooded on Oyatie Cloud with workflow, Mail, and evidence return paths."</p>
                <div class="service-metric-row">
                    <span><strong>"73%"</strong><small>"close progress"</small></span>
                    <span><strong>"5.4d"</strong><small>"cycle time"</small></span>
                    <span><strong>"₩2.18M"</strong><small>"cost of delay"</small></span>
                </div>
                <ol class="service-checklist">
                    <li><span class="status-chip danger">"blocking"</span>"4대보험 변동 확인 필요"</li>
                    <li><span class="status-chip warning">"review"</span>"Payroll reminder mail draft ready"</li>
                    <li><span class="status-chip success">"sealed"</span>"Receipt REC-PAY-2026-04 staged"</li>
                </ol>
                <div class="service-card-actions">
                    <button type="button" data-service-action="payroll-finance">"Finance cockpit"</button>
                    <button type="button" data-service-action="payroll-workflow">"Workflow gate"</button>
                    <button type="button" data-service-action="payroll-mail">"Mail brief"</button>
                    <button type="button" data-service-action="payroll-evidence">"Evidence"</button>
                </div>
            </article>

            <article id="filing-readiness" class="service-card filing-card">
                <p class="screen-anchor">"FILING READINESS"</p>
                <h4>"Withholding return"</h4>
                <p class="service-card-brief">"Korea localization workload joins the same substrate proof loop: reviewer attestation, transport, and receipt."</p>
                // A-6: added role=progressbar + aria-value* attributes
                <div class="readiness-bars compact" aria-label="Filing readiness">
                    <span role="progressbar" aria-valuenow="86" aria-valuemin="0" aria-valuemax="100" aria-label="Employee validation: 86%" style="--bar: 86%"><em aria-hidden="true">"Employee validation"</em></span>
                    <span role="progressbar" aria-valuenow="64" aria-valuemin="0" aria-valuemax="100" aria-label="HomeTax transport: 64%" style="--bar: 64%"><em aria-hidden="true">"HomeTax transport"</em></span>
                    <span role="progressbar" aria-valuenow="52" aria-valuemin="0" aria-valuemax="100" aria-label="Reviewer attestation: 52%" style="--bar: 52%"><em aria-hidden="true">"Reviewer attestation"</em></span>
                </div>
                <div class="service-card-actions">
                    <button type="button" data-sidepeek-trigger="filing" data-sidepeek-title="Withholding return" data-sidepeek-id="FILE-KR-2026-04" data-sidepeek-desc="Filing readiness is staged locally and never submitted before live integration." data-sidepeek-owner="Finance close" data-sidepeek-risk="2 review" data-sidepeek-sla="Due 2026-05-10">"Inspect"</button>
                    <button type="button" data-service-action="filing-billing">"Billing · tax"</button>
                    <button type="button" data-service-action="filing-community">"Council note"</button>
                    <button type="button" data-service-action="filing-evidence">"Receipt"</button>
                </div>
                {filing_readiness_anchor_board()}
            </article>

            <article id="employee-directory" class="service-card employee-card">
                <p class="screen-anchor">"EMPLOYEES"</p>
                <h4>"Employee directory"</h4>
                <p class="service-card-brief">"Identity and workforce data remain role-visible while FD-001 tenant workloads prove policy envelopes."</p>
                <div class="service-people-stats">
                    <span><strong>"118"</strong><small>"employees"</small></span>
                    <span><strong>"109"</strong><small>"active"</small></span>
                    <span><strong>"5"</strong><small>"probation watch"</small></span>
                </div>
                <table class="employee-mini-table">
                    <thead><tr><th>"Name"</th><th>"Role"</th><th>"Team"</th><th>"Status"</th></tr></thead>
                    <tbody>
                        <tr><td>"이재현 Jaehyun Lee"</td><td>"CEO"</td><td>"Office"</td><td>"활성"</td></tr>
                        <tr><td>"최유나 Yuna Choi"</td><td>"CFO"</td><td>"Finance"</td><td>"활성"</td></tr>
                        <tr><td>"박서준 Seojun Park"</td><td>"VP Engineering"</td><td>"Infrastructure"</td><td>"활성"</td></tr>
                        <tr><td>"김지영 Jiyoung Kim"</td><td>"Manager"</td><td>"Infrastructure"</td><td>"수습"</td></tr>
                    </tbody>
                </table>
                <div class="service-card-actions">
                    <button type="button" data-service-action="employee-identity">"Identity service"</button>
                    <button type="button" data-service-action="employee-onboarding">"Onboarding"</button>
                    <button type="button" data-service-action="employee-policy">"Policy"</button>
                    <button type="button" data-service-action="employee-mail">"Mail reviewer"</button>
                </div>
            </article>

            <article id="governance-analytics-summary" class="service-card governance-card">
                <p class="screen-anchor">"GOVERNANCE ANALYTICS"</p>
                <h4>"Policy, receipts, workflow health"</h4>
                <p class="service-card-brief">"Executive posture rolls FD-001 workload proof, cloud cell evidence, and built-in surface routes into one council view."</p>
                <div class="service-graph" aria-hidden="true">
                    <span style="--bar: 78%"></span>
                    <span style="--bar: 48%"></span>
                    <span style="--bar: 92%"></span>
                    <span style="--bar: 61%"></span>
                    <span style="--bar: 72%"></span>
                </div>
                <dl class="service-kv">
                    <div><dt>"Risk"</dt><dd>"3 high-risk approvals"</dd></div>
                    <div><dt>"Evidence"</dt><dd>"12 sealed draft receipts"</dd></div>
                    <div><dt>"Workflow"</dt><dd>{envelope.workflow.name}</dd></div>
                </dl>
                <div class="service-card-actions">
                    <button type="button" data-service-action="governance-command">"Command board"</button>
                    <button type="button" data-service-action="governance-risk">"Risk heatmap"</button>
                    <button type="button" data-service-action="governance-community">"Community"</button>
                    <button type="button" data-service-action="governance-evidence">"Evidence"</button>
                </div>
            </article>
        </section>
    }
}

pub(super) fn filing_readiness_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board filing-trust-board" aria-label="FD-001 filing readiness and Oyatie Cloud localization proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected" data-trust-proof-card="filing-fd001">
                    <p class="screen-anchor">"FD-001 LOCALIZATION WORKLOAD"</p>
                    <h5>"Korea filing is tenant workload delivery, not a side widget"</h5>
                    <p>
                        "Withholding return, employee validation, HomeTax transport, billing, Mail, Community, and evidence receipts stay in the FD-001 tenant workload graph."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="stage-filing">"Stage filing"</button>
                        <button type="button" data-trust-proof-action="route-billing">"Billing · tax"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="filing-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD RESIDENCY"</p>
                    <h5>"Substrate proves regional pack posture"</h5>
                    <p>
                        "Oyatie Cloud shows PIPA-aware residency, policy envelope, release gates, audit freshness, and rollback posture before any tax workload readiness claim."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-policy">"PIPA policy"</button>
                        <button type="button" data-trust-proof-action="route-cloud">"Cloud cells"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="filing-local">
                    <p class="screen-anchor">"LOCAL-ONLY FILING RAIL"</p>
                    <h5>"Reviewer-ready, never submitted"</h5>
                    <p>
                        "Operators can inspect readiness, stage a reviewer packet, and route council notes visually; no HomeTax, bank, payroll, billing, mail, or cloud mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-mail">"Reviewer Mail"</button>
                        <button type="button" data-trust-proof-action="route-evidence">"Receipt spine"</button>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Filing readiness ready · FD-001 localization workload dogfoods Oyatie Cloud with local-only submission controls."
                </span>
                <div class="trust-anchor-routes" aria-label="Filing readiness connected routes">
                    <button type="button" data-trust-proof-action="route-finance">"Finance close"</button>
                    <button type="button" data-trust-proof-action="route-community">"Community note"</button>
                    <button type="button" data-trust-proof-action="route-audit">"Audit ledger"</button>
                    <button type="button" data-trust-proof-action="route-catalog">"Catalog"</button>
                </div>
            </div>
        </div>
    }
}
