use super::*;

pub(super) fn governance_ops_command_board(workflow_name: String) -> impl IntoView {
    view! {
        <section id="governance-analytics" class="governance-ops-command" aria-labelledby="governance-ops-title">
            <div class="governance-command-head">
                <div>
                    <p class="screen-anchor">"00 / GOVERNANCE ANALYTICS"</p>
                    <h4 id="governance-ops-title">"Policy, evidence, risk, and business logic control"</h4>
                    <p>"Bominal-grade executive governance: posture, risk heatmap, control attestations, compliance calendar, decision queue, evidence chain, and routes into every built-in surface."</p>
                </div>
                <div class="governance-command-actions">
                    <span class="status-chip warning" data-governance-status="true">"84 posture · 5 top risks · local only"</span>
                    <button type="button" data-governance-action="run-review">"Run review"</button>
                    <button type="button" data-governance-action="seal-brief" data-governance-route="evidence">"Seal brief"</button>
                    <button type="button" data-governance-action="route-inbox" data-governance-route="inbox">"Open queue"</button>
                </div>
            </div>

            <div class="governance-posture-strip" aria-label="Governance posture overview">
                <article class="gov-posture-score">
                    <span>"Composite posture"</span>
                    <strong>"84"</strong>
                    <em>"A− · +3 vs last quarter · target 90"</em>
                </article>
                <article class="gov-posture-trend">
                    <div><span>"13 week trend"</span><strong>"78 → 84"</strong></div>
                    <svg viewBox="0 0 160 42" aria-hidden="true" class="gov-sparkline">
                        <polyline points="0,34 14,32 28,32 42,29 56,29 70,27 84,29 98,26 112,23 126,23 140,18 154,16" />
                    </svg>
                </article>
                <div class="gov-pillar-grid" aria-label="Governance pillars">
                    <button type="button" data-governance-action="pillar-compliance" data-governance-route="finance"><span>"Compliance"</span><strong>"87"</strong><em>"24 controls · 1 breach"</em></button>
                    <button type="button" data-governance-action="pillar-financial" data-governance-route="finance"><span>"Financial controls"</span><strong>"91"</strong><em>"manual JE 4.2%"</em></button>
                    <button type="button" data-governance-action="pillar-workforce" data-governance-route="identity"><span>"Workforce risk"</span><strong>"71"</strong><em>"§53 watch"</em></button>
                    <button type="button" data-governance-action="pillar-disclosure" data-governance-route="community"><span>"Board + disclosure"</span><strong>"88"</strong><em>"pack ships May 12"</em></button>
                </div>
            </div>

            <div class="governance-command-grid">
                <article class="governance-command-card policy-gate-card">
                    <div class="governance-card-head">
                        <div>
                            <p class="screen-anchor">"POLICY GATES"</p>
                            <h5>"Decision rights before risky operation"</h5>
                        </div>
                        <span class="status-chip danger">"2 blocks"</span>
                    </div>
                    <div class="policy-gate-list" aria-label="Governance policy gates">
                        <button type="button" class="active" data-governance-action="select-payroll" data-governance-route="workflow"><span>"P0"</span><strong>"Payroll close"</strong><em>"2-person CFO signoff"</em></button>
                        <button type="button" data-governance-action="select-hometax" data-governance-route="finance"><span>"P0"</span><strong>"HomeTax filing"</strong><em>"사업자등록번호 confirmation"</em></button>
                        <button type="button" data-governance-action="select-cloud" data-governance-route="cloud"><span>"P0"</span><strong>"Network split"</strong><em>"rollback evidence required"</em></button>
                        <button type="button" data-governance-action="select-pipa" data-governance-route="identity"><span>"P1"</span><strong>"PIPA boundary"</strong><em>"vendor cannot view employee PII"</em></button>
                    </div>
                </article>

                <article class="governance-command-card decision-queue-card">
                    <div class="governance-card-head">
                        <div>
                            <p class="screen-anchor">"EXEC APPROVALS"</p>
                            <h5>"Owners, SLA, origin, and route"</h5>
                        </div>
                        <button type="button" data-governance-action="open-inbox" data-governance-route="inbox">"Inbox"</button>
                    </div>
                    <table class="governance-decision-table">
                        <thead><tr><th>"Decision"</th><th>"Owner"</th><th>"SLA"</th><th>"State"</th></tr></thead>
                        <tbody>
                            <tr><td><strong>"Park 4대보험 delta"</strong><small>"REC-PAY-2026-04-PARK"</small></td><td>"CFO"</td><td>"4.0h"</td><td><span class="status-chip danger">"blocking"</span></td></tr>
                            <tr><td><strong>"Stripe approval compression"</strong><small>"policy ≤ ₩5M route"</small></td><td>"AP"</td><td>"1d"</td><td><span class="status-chip warning">"review"</span></td></tr>
                            <tr><td><strong>"Governance council note"</strong><small>"Community + Mail packet"</small></td><td>"Gov"</td><td>"today"</td><td><span class="status-chip success">"ready"</span></td></tr>
                            <tr><td><strong>"Board option pool"</strong><small>"resolution pack May 12"</small></td><td>"CEO"</td><td>"6d"</td><td><span class="status-chip">"scheduled"</span></td></tr>
                        </tbody>
                    </table>
                </article>

                <article class="governance-command-card risk-matrix-card">
                    <div class="governance-card-head">
                        <div>
                            <p class="screen-anchor">"RISK MATRIX"</p>
                            <h5>"Audit committee 5×5 heatmap"</h5>
                        </div>
                        <span class="status-chip ai">"interactive"</span>
                    </div>
                    <div class="gov-risk-workspace">
                        <div class="gov-risk-heatmap" aria-label="Selectable risk matrix">
                            <span class="tone-minimal"></span><span class="tone-minimal"></span><span class="tone-low"></span><span class="tone-low"></span><span class="tone-moderate"></span>
                            <span class="tone-minimal"></span><span class="tone-low"></span><span class="tone-low"></span><span class="tone-moderate"></span><span class="tone-high"></span>
                            <span class="tone-low"></span><span class="tone-low"></span><span class="tone-moderate"></span><span class="tone-high"></span><span class="tone-high"></span>
                            <span class="tone-low"></span><span class="tone-moderate"></span><span class="tone-high"></span><span class="tone-high"></span><span class="tone-extreme"></span>
                            <span class="tone-moderate"></span><span class="tone-high"></span><span class="tone-high"></span><span class="tone-extreme"></span><span class="tone-extreme"></span>
                            <button type="button" class="gov-risk-pin selected" style="--x: 78%; --y: 42%" data-governance-risk="RISK-042" data-risk-title="AI agent governance" data-risk-detail="Auto-delegation rollout needs change-management, rollback evidence, and human signoff." data-risk-owner="EMP-104" data-risk-score="4×3 High">"AI"</button>
                            <button type="button" class="gov-risk-pin" style="--x: 58%; --y: 30%" data-governance-risk="RISK-014" data-risk-title="LSA §53 weekly-hour breach" data-risk-detail="Yoon Tae-min projected 49.5h; automatic reassignment lowers residual risk." data-risk-owner="EMP-211" data-risk-score="3×4 Moderate">"53"</button>
                            <button type="button" class="gov-risk-pin" style="--x: 40%; --y: 34%" data-governance-risk="RISK-009" data-risk-title="PIPA retention overrun" data-risk-detail="43 medical certificates expire in 14 days; consent renewal or purge decision required." data-risk-owner="EMP-274" data-risk-score="2×4 Moderate">"PI"</button>
                            <button type="button" class="gov-risk-pin" style="--x: 57%; --y: 54%" data-governance-risk="RISK-031" data-risk-title="JE four-eyes gap" data-risk-detail="7 manual journal entries posted with a single approver; enforcement pending." data-risk-owner="EMP-188" data-risk-score="3×3 Moderate">"JE"</button>
                            <button type="button" class="gov-risk-pin" style="--x: 21%; --y: 20%" data-governance-risk="RISK-018" data-risk-title="Missed board resolution" data-risk-detail="Option pool expansion requires May 12 board resolution and quorum confirmation." data-risk-owner="EMP-274" data-risk-score="1×5 Low">"BD"</button>
                        </div>
                        <aside class="gov-risk-peek" aria-live="polite">
                            <div><span data-risk-peek-id="true">"RISK-042"</span><strong data-risk-peek-score="true">"4×3 High"</strong></div>
                            <h6 data-risk-peek-title="true">"AI agent governance"</h6>
                            <p data-risk-peek-detail="true">"Auto-delegation rollout needs change-management, rollback evidence, and human signoff."</p>
                            <dl><div><dt>"Owner"</dt><dd data-risk-peek-owner="true">"EMP-104"</dd></div><div><dt>"Next review"</dt><dd>"2026-05-08"</dd></div></dl>
                        </aside>
                    </div>
                </article>

                <article class="governance-command-card compliance-calendar-card">
                    <div class="governance-card-head">
                        <div>
                            <p class="screen-anchor">"COMPLIANCE CALENDAR"</p>
                            <h5>"12-month filing commitments"</h5>
                        </div>
                        <button type="button" data-governance-action="calendar-review" data-governance-route="finance">"Review"</button>
                    </div>
                    <div class="gov-calendar" aria-label="Compliance calendar">
                        <span class="cal-corner">"Family"</span><span class="cal-month now">"May"</span><span class="cal-month">"Jun"</span><span class="cal-month">"Jul"</span><span class="cal-month">"Aug"</span><span class="cal-month">"Sep"</span><span class="cal-month">"Oct"</span><span class="cal-month">"Nov"</span><span class="cal-month">"Dec"</span><span class="cal-month">"Jan"</span><span class="cal-month">"Feb"</span><span class="cal-month">"Mar"</span><span class="cal-month">"Apr"</span>
                        <strong>"Withholding"</strong><button type="button" class="ready" data-gov-calendar-cell="Withholding May ready">"10 ₩38.2M"</button><button type="button" class="pending" data-gov-calendar-cell="Withholding Jun pending">"10 ₩39.1M"</button><button type="button" class="pending" data-gov-calendar-cell="Withholding Jul pending">"10 ₩40.0M"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button>
                        <strong>"4대보험"</strong><button type="button" class="review" data-gov-calendar-cell="Social insurance May review">"10 ₩57.0M"</button><button type="button" class="pending" data-gov-calendar-cell="Social insurance Jun pending">"10 ₩58.2M"</button><button type="button" class="pending" data-gov-calendar-cell="Social insurance Jul pending">"10 ₩59.1M"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button>
                        <strong>"VAT"</strong><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="pending" data-gov-calendar-cell="VAT Q2 prelim pending">"25 Q2 prelim"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="pending" data-gov-calendar-cell="VAT Q3 final pending">"25 Q3 final"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="pending" data-gov-calendar-cell="VAT Q4 prelim pending">"25 Q4 prelim"</button><button type="button" class="empty">"—"</button><button type="button" class="empty">"—"</button><button type="button" class="pending" data-gov-calendar-cell="VAT Q1 final pending">"25 Q1 final"</button>
                    </div>
                </article>

                <article class="governance-command-card evidence-readiness-card">
                    <div class="governance-card-head">
                        <div>
                            <p class="screen-anchor">"EVIDENCE READINESS"</p>
                            <h5>"Receipts that prove the graph"</h5>
                        </div>
                        <span class="status-chip success">"sealed draft"</span>
                    </div>
                    // A-6: role=progressbar + aria-value*
                    <div class="evidence-readiness-lanes" aria-label="Evidence readiness lanes">
                        <span role="progressbar" aria-valuenow="92" aria-valuemin="0" aria-valuemax="100" aria-label="Workflow receipts: 92%" style="--bar: 92%"><strong aria-hidden="true">"Workflow receipts"</strong><em aria-hidden="true">"11 / 12"</em></span>
                        <span role="progressbar" aria-valuenow="78" aria-valuemin="0" aria-valuemax="100" aria-label="Mail approvals: 78%" style="--bar: 78%"><strong aria-hidden="true">"Mail approvals"</strong><em aria-hidden="true">"7 linked"</em></span>
                        <span role="progressbar" aria-valuenow="64" aria-valuemin="0" aria-valuemax="100" aria-label="Cloud runbooks: 64%" style="--bar: 64%"><strong aria-hidden="true">"Cloud runbooks"</strong><em aria-hidden="true">"2 waiting"</em></span>
                        <span role="progressbar" aria-valuenow="86" aria-valuemin="0" aria-valuemax="100" aria-label="PIPA audit: 86%" style="--bar: 86%"><strong aria-hidden="true">"PIPA audit"</strong><em aria-hidden="true">"vendor gated"</em></span>
                    </div>
                </article>

                <article class="governance-command-card graph-route-card">
                    <div class="governance-card-head">
                        <div>
                            <p class="screen-anchor">"ROUTE MATRIX"</p>
                            <h5>"Open connected product surface"</h5>
                        </div>
                    </div>
                    <div class="governance-route-grid" aria-label="Governance route matrix">
                        <button type="button" data-governance-action="route-workflow" data-governance-route="workflow">"Workflow"</button>
                        <button type="button" data-governance-action="route-mail" data-governance-route="mail">"Mail"</button>
                        <button type="button" data-governance-action="route-community" data-governance-route="community">"Community"</button>
                        <button type="button" data-governance-action="route-finance" data-governance-route="finance">"Finance"</button>
                        <button type="button" data-governance-action="route-cloud" data-governance-route="cloud">"Cloud Ops"</button>
                        <button type="button" data-governance-action="route-identity" data-governance-route="identity">"Identity"</button>
                        <button type="button" data-governance-action="route-evidence" data-governance-route="evidence">"Evidence"</button>
                        <button type="button" data-governance-action="route-catalog" data-governance-route="catalog">"Catalog"</button>
                    </div>
                    <dl class="governance-kv">
                        <div><dt>"Graph root"</dt><dd>{workflow_name}</dd></div>
                        <div><dt>"Autonomy ceiling"</dt><dd>"No auto-approval · visual review only"</dd></div>
                    </dl>
                </article>
            </div>

            <div class="governance-lower-deck">
                <article class="governance-command-card control-attestation-card">
                    <div class="governance-card-head"><div><p class="screen-anchor">"CONTROL ATTESTATION"</p><h5>"Controls with overdue evidence"</h5></div><span class="status-chip warning">"3 due"</span></div>
                    <div class="gov-attestation-list">
                        <button type="button" data-governance-action="attest-payroll" data-governance-route="workflow"><strong>"CTRL-PAY-09 · payroll 4-eyes"</strong><span style="--bar: 82%"></span><em>"82% · CFO attestation waiting"</em></button>
                        <button type="button" data-governance-action="attest-pipa" data-governance-route="identity"><strong>"CTRL-PIPA-03 · retention boundary"</strong><span style="--bar: 68%"></span><em>"68% · vendor visibility review"</em></button>
                        <button type="button" data-governance-action="attest-cloud" data-governance-route="cloud"><strong>"CTRL-CLOUD-12 · rollback runbook"</strong><span style="--bar: 74%"></span><em>"74% · regional evidence stale"</em></button>
                    </div>
                </article>
                <article class="governance-command-card audit-chain-card">
                    <div class="governance-card-head"><div><p class="screen-anchor">"AUDIT CHAIN"</p><h5>"Immutable receipt lineage"</h5></div><button type="button" data-governance-action="route-evidence" data-governance-route="evidence">"Evidence"</button></div>
                    <ol class="gov-audit-chain">
                        <li><span>"01"</span><strong>"Workflow run"</strong><em>"hash 5bf7…91"</em></li>
                        <li><span>"02"</span><strong>"Mail approval"</strong><em>"hash a81d…0c"</em></li>
                        <li><span>"03"</span><strong>"Community note"</strong><em>"hash 44c2…bf"</em></li>
                        <li><span>"04"</span><strong>"Board packet"</strong><em>"draft"</em></li>
                    </ol>
                </article>
                <article class="governance-command-card board-cycle-card">
                    <div class="governance-card-head"><div><p class="screen-anchor">"BOARD CYCLE"</p><h5>"Resolution timeline and quorum"</h5></div><span class="status-chip success">"3 / 5 ack"</span></div>
                    <div class="gov-board-timeline" aria-label="Board cycle timeline">
                        <span class="done"><strong>"Draft"</strong><em>"May 04"</em></span>
                        <span class="active"><strong>"Review"</strong><em>"May 07"</em></span>
                        <span><strong>"Send"</strong><em>"May 10"</em></span>
                        <span><strong>"Vote"</strong><em>"May 12"</em></span>
                    </div>
                </article>
            </div>
        </section>
    }
}
