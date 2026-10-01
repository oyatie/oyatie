use super::*;

pub(super) fn finance_commercial_service() -> impl IntoView {
    view! {
        <section
            id="finance-commercial-service"
            class="finance-commercial-service panel"
            data-finance-service="true"
            aria-labelledby="finance-service-title"
        >
            <div class="finance-service-head">
                <div>
                    <p class="screen-anchor">"MONEY · COMMERCIAL OPERATIONS"</p>
                    <h3 id="finance-service-title">"Finance, ledger, vendor spend, billing, and leave-time"</h3>
                </div>
                <div class="finance-service-actions">
                    <span class="status-chip success" data-finance-status="true">"close package ready"</span>
                    <button type="button" data-finance-action="reconcile">"Reconcile"</button>
                    <button type="button" data-finance-action="export-pack">"Export pack"</button>
                </div>
            </div>

            <div class="finance-kpi-strip" aria-label="Finance operations summary">
                <span><small>"Cash runway"</small><strong>"18.4 mo"</strong><em>"₩12.6B available"</em></span>
                <span><small>"Open invoices"</small><strong>"₩482M"</strong><em>"12 invoices · 2 overdue"</em></span>
                <span><small>"Vendor risk"</small><strong>"3 high"</strong><em>"Stripe · AWS · payroll bureau"</em></span>
                <span><small>"Leave liability"</small><strong>"₩53M"</strong><em>"49.5h this week"</em></span>
            </div>

            <div class="finance-service-shell">
                <aside class="finance-ledger-rail" aria-label="Finance sections">
                    <p class="screen-anchor">"FINANCE VIEWS"</p>
                    <button type="button" class="active" data-finance-tab="ledger">"Ledger close"</button>
                    <button type="button" data-finance-tab="vendors">"Vendors & spend"</button>
                    <button type="button" data-finance-tab="billing">"Billing & tax"</button>
                    <button type="button" data-finance-tab="leave">"Leave & time"</button>
                    <div class="finance-close-card">
                        <small>"APRIL CLOSE"</small>
                        <strong>"73%"</strong>
                        <span class="score-bar" role="progressbar" aria-valuenow="73" aria-valuemin="0" aria-valuemax="100" aria-label="April close readiness: 73%" style="--bar: 73%"><em aria-hidden="true"></em></span>
                        <p>"Payroll, withholding, and vendor accruals are staged for 2-person review."</p>
                    </div>
                </aside>

                <div class="finance-service-main">
                    // A-2: finance tablist — aria-orientation + id/aria-controls + role=tabpanel
                    <div class="finance-tabs" role="tablist" aria-label="Finance service views" aria-orientation="horizontal">
                        <button type="button" id="finance-tab-ledger" class="active" data-finance-tab="ledger" role="tab" aria-selected="true" aria-controls="finance-panel-ledger">"Ledger"</button>
                        <button type="button" id="finance-tab-vendors" data-finance-tab="vendors" role="tab" aria-selected="false" aria-controls="finance-panel-vendors">"Vendors"</button>
                        <button type="button" id="finance-tab-billing" data-finance-tab="billing" role="tab" aria-selected="false" aria-controls="finance-panel-billing">"Billing"</button>
                        <button type="button" id="finance-tab-leave" data-finance-tab="leave" role="tab" aria-selected="false" aria-controls="finance-panel-leave">"Leave & time"</button>
                    </div>

                    <article id="finance-panel-ledger" class="finance-panel active" data-finance-panel="ledger" role="tabpanel" aria-labelledby="finance-tab-ledger">
                        <div class="finance-panel-copy">
                            <p class="screen-anchor">"LEDGER · CLOSE PACKAGE"</p>
                            <h4 id="ledger-panel-title">"Ledger close cockpit"</h4>
                            <p>"Every payroll, filing, vendor, and billing event resolves into one audit-ready close package."</p>
                        </div>
                        <div class="ledger-layout">
                            // A-6: role=progressbar + aria-value* on reconciliation bars
                            <div class="ledger-reconciliation">
                                <span role="progressbar" aria-valuenow="92" aria-valuemin="0" aria-valuemax="100" aria-label="Bank feed match: 92%" style="--bar: 92%"><em aria-hidden="true">"Bank feed match · 92%"</em></span>
                                <span role="progressbar" aria-valuenow="86" aria-valuemin="0" aria-valuemax="100" aria-label="Payroll accrual: 86%" style="--bar: 86%"><em aria-hidden="true">"Payroll accrual · 86%"</em></span>
                                <span role="progressbar" aria-valuenow="68" aria-valuemin="0" aria-valuemax="100" aria-label="Vendor accrual: 68%" style="--bar: 68%"><em aria-hidden="true">"Vendor accrual · 68%"</em></span>
                                <span role="progressbar" aria-valuenow="51" aria-valuemin="0" aria-valuemax="100" aria-label="Tax evidence: 51%" style="--bar: 51%"><em aria-hidden="true">"Tax evidence · 51%"</em></span>
                            </div>
                            <table class="finance-table">
                                <thead><tr><th>"Time"</th><th>"Account"</th><th>"Object"</th><th>"Amount"</th><th>"State"</th></tr></thead>
                                <tbody>
                                    <tr><td>"09:18"</td><td>"Payroll payable"</td><td>"APR payroll close"</td><td>"₩894,000,000"</td><td><span class="status-chip warning">"review"</span></td></tr>
                                    <tr><td>"09:42"</td><td>"Withholding tax"</td><td>"HomeTax draft"</td><td>"₩118,400,000"</td><td><span class="status-chip success">"matched"</span></td></tr>
                                    <tr><td>"10:05"</td><td>"Vendor accrual"</td><td>"Stripe invoice"</td><td>"₩4,820,000"</td><td><span class="status-chip danger">"blocking"</span></td></tr>
                                    <tr><td>"10:21"</td><td>"Leave liability"</td><td>"Yoon Tae-min risk"</td><td>"₩53,000,000"</td><td><span class="status-chip">"advisory"</span></td></tr>
                                </tbody>
                            </table>
                        </div>
                        {finance_command_board()}
                        {ledger_preview_anchor_board()}
                    </article>

                    <article id="finance-panel-vendors" class="finance-panel" data-finance-panel="vendors" role="tabpanel" aria-labelledby="finance-tab-vendors">
                        <div class="finance-panel-copy">
                            <p class="screen-anchor">"PROCUREMENT · VENDORS"</p>
                            <h4 id="vendors-panel-title">"Vendors & spend control"</h4>
                            <p>"Contracts, approvals, owners, and cost-of-delay are tied to the same service graph as workflow and mail."</p>
                        </div>
                        <div class="vendor-toolbar">
                            <label><span aria-hidden="true">"⌕"</span><input data-vendor-search="true" aria-label="Search vendors" placeholder="Search vendor, owner, contract..." /></label>
                            <button type="button" data-finance-action="add-vendor">"+ Vendor"</button>
                        </div>
                        <table class="finance-table vendor-table">
                            <thead><tr><th>"Vendor"</th><th>"Owner"</th><th>"Monthly"</th><th>"Renewal"</th><th>"Risk"</th><th>"Action"</th></tr></thead>
                            <tbody>
                                <tr data-vendor-row="true"><td><strong>"Stripe"</strong><small>"Payments · invoice INV-4281"</small></td><td>"Finance"</td><td>"₩4.82M"</td><td>"2026-05-31"</td><td><span class="status-chip danger">"High"</span></td><td><button type="button" data-finance-action="approve-vendor">"Review"</button></td></tr>
                                <tr data-vendor-row="true"><td><strong>"AWS Korea"</strong><small>"Cloud infrastructure · reserved capacity"</small></td><td>"SRE"</td><td>"₩63.4M"</td><td>"2026-06-12"</td><td><span class="status-chip warning">"Medium"</span></td><td><button type="button" data-finance-action="optimize-spend">"Optimize"</button></td></tr>
                                <tr data-vendor-row="true"><td><strong>"Shinhan Bank"</strong><small>"Payroll and withholding transport"</small></td><td>"CFO"</td><td>"₩1.2M"</td><td>"2027-01-10"</td><td><span class="status-chip success">"Low"</span></td><td><button type="button" data-sidepeek-trigger="bank-vendor" data-sidepeek-title="Shinhan Bank transport" data-sidepeek-id="VEN-SHINHAN" data-sidepeek-desc="Bank transport is staged visually and is not connected to a real payment rail." data-sidepeek-owner="CFO" data-sidepeek-risk="Low" data-sidepeek-sla="Staged only">"Inspect"</button></td></tr>
                            </tbody>
                        </table>
                        {finance_vendors_anchor_board()}
                    </article>

                    <article id="finance-panel-billing" class="finance-panel" data-finance-panel="billing" role="tabpanel" aria-labelledby="finance-tab-billing">
                        <div class="finance-panel-copy">
                            <p class="screen-anchor">"BILLING · TAX"</p>
                            <h4 id="billing-panel-title">"Billing, plans, and filings"</h4>
                            <p>"Customer invoices, plan changes, HomeTax filing readiness, and payment evidence stay visible next to operations."</p>
                        </div>
                        <div class="billing-grid">
                            <article><p class="screen-anchor">"REVENUE"</p><strong>"₩2.31B"</strong><span>"ARR staged · 42 active contracts"</span><button type="button" data-finance-action="send-invoice">"Stage invoice"</button></article>
                            <article><p class="screen-anchor">"TAX FILING"</p><strong>"64%"</strong><span>"HomeTax transport awaiting reviewer"</span><button type="button" data-finance-action="tax-brief">"Draft brief"</button></article>
                            <article><p class="screen-anchor">"PLAN CHANGES"</p><strong>"7"</strong><span>"2 require billing owner review"</span><button type="button" data-sidepeek-trigger="billing-plans" data-sidepeek-title="Plan change queue" data-sidepeek-id="BILL-PLAN-7" data-sidepeek-desc="Plan changes are staged queue items with no payment or billing mutation." data-sidepeek-owner="Revenue ops" data-sidepeek-risk="Review" data-sidepeek-sla="Visual only">"Open queue"</button></article>
                        </div>
                        {finance_billing_anchor_board()}
                    </article>

                    <article id="finance-panel-leave" class="finance-panel" data-finance-panel="leave" role="tabpanel" aria-labelledby="finance-tab-leave">
                        <div class="finance-panel-copy">
                            <p class="screen-anchor">"PEOPLE COST · LEAVE"</p>
                            <h4 id="leave-panel-title">"Leave & time liability"</h4>
                            <p>"Leave approvals are connected to workforce, payroll, schedule, and financial liability before close."</p>
                        </div>
                        <div class="leave-layout">
                            <ol class="leave-queue">
                                <li><time>"May 13–17"</time><strong>"김지영 leave request"</strong><span>"5 days · backup confirmed"</span><button type="button" data-finance-action="approve-leave">"Approve locally"</button></li>
                                <li><time>"This week"</time><strong>"윤태민 overtime risk"</strong><span>"49.5h projected · 2 backup engineers recommended"</span><button type="button" data-finance-action="reassign-time">"Reassign"</button></li>
                                <li><time>"May 22"</time><strong>"Payroll cutoff"</strong><span>"Timesheets lock 3 days before payout"</span><button type="button" data-finance-action="lock-timesheets">"Preview lock"</button></li>
                            </ol>
                            <div class="time-heatmap" aria-label="Weekly time utilization">
                                <span role="progressbar" aria-valuenow="58" aria-valuemin="0" aria-valuemax="100" aria-label="Monday utilization: 58%" style="--bar: 58%"><em aria-hidden="true">"Mon"</em></span>
                                <span role="progressbar" aria-valuenow="72" aria-valuemin="0" aria-valuemax="100" aria-label="Tuesday utilization: 72%" style="--bar: 72%"><em aria-hidden="true">"Tue"</em></span>
                                <span role="progressbar" aria-valuenow="91" aria-valuemin="0" aria-valuemax="100" aria-label="Wednesday utilization: 91%" style="--bar: 91%"><em aria-hidden="true">"Wed"</em></span>
                                <span role="progressbar" aria-valuenow="84" aria-valuemin="0" aria-valuemax="100" aria-label="Thursday utilization: 84%" style="--bar: 84%"><em aria-hidden="true">"Thu"</em></span>
                                <span role="progressbar" aria-valuenow="49" aria-valuemin="0" aria-valuemax="100" aria-label="Friday utilization: 49%" style="--bar: 49%"><em aria-hidden="true">"Fri"</em></span>
                            </div>
                        </div>
                        {finance_leave_anchor_board()}
                    </article>
                </div>

                <aside class="finance-context-rail" aria-label="Finance provenance">
                    <p class="screen-anchor">"CLOSE PROVENANCE"</p>
                    <dl class="service-kv">
                        <div><dt>"Workflow"</dt><dd>"April close package"</dd></div>
                        <div><dt>"Receipts"</dt><dd>"18 staged"</dd></div>
                        <div><dt>"Reviewers"</dt><dd>"CFO · payroll · tax"</dd></div>
                    </dl>
                    <p class="screen-anchor">"LOCAL NOTIFICATIONS"</p>
                    <ol class="notification-stack">
                        <li>"Vendor approval waiting"</li>
                        <li>"Tax brief can be drafted"</li>
                        <li>"No real money moves"</li>
                    </ol>
                </aside>
            </div>
        </section>
    }
}

pub(super) fn finance_command_board() -> impl IntoView {
    view! {
        <section class="finance-command-board" aria-labelledby="finance-command-board-title">
            <div class="finance-command-head">
                <div>
                    <p class="screen-anchor">"CLOSE COMMAND · OBJECT GRAPH"</p>
                    <h4 id="finance-command-board-title">"April commercial command center"</h4>
                    <p>
                        "A Bominal-grade operating surface: payroll, ledger, tax, vendors, billing, leave, workflow, messages, and evidence stay in one dense local workspace."
                    </p>
                </div>
                <div class="finance-command-actions">
                    <span class="status-chip warning" data-finance-command-status="true">"7 objects · 3 blockers · local dry-run"</span>
                    <button type="button" data-finance-command-action="run-close">"Run close dry-run"</button>
                    <button type="button" data-finance-command-action="attach-proof" data-finance-route="evidence">"Attach proof"</button>
                </div>
            </div>

            <div class="finance-command-grid">
                <article class="finance-command-card finance-close-spine-card">
                    <div class="finance-command-card-head">
                        <div>
                            <p class="screen-anchor">"CLOSE SPINE"</p>
                            <h5>"Every commercial object joins the same package"</h5>
                        </div>
                        <span class="status-chip danger">"3 blockers"</span>
                    </div>
                    <div class="finance-spine-flow" aria-label="Commercial close object spine">
                        <button type="button" class="active" data-finance-command-action="open-payroll" data-finance-route="ledger">
                            <span>"Payroll"</span><strong>"₩894M"</strong><em>"review"</em>
                        </button>
                        <button type="button" data-finance-command-action="open-tax" data-finance-route="billing">
                            <span>"Tax"</span><strong>"₩118M"</strong><em>"HomeTax"</em>
                        </button>
                        <button type="button" data-finance-command-action="open-vendor" data-finance-route="vendors">
                            <span>"Vendors"</span><strong>"₩69M"</strong><em>"Stripe · AWS"</em>
                        </button>
                        <button type="button" data-finance-command-action="open-billing" data-finance-route="billing">
                            <span>"Billing"</span><strong>"₩482M"</strong><em>"12 invoices"</em>
                        </button>
                        <button type="button" data-finance-command-action="open-leave" data-finance-route="leave">
                            <span>"Leave"</span><strong>"49.5h"</strong><em>"liability"</em>
                        </button>
                        <button type="button" data-finance-command-action="open-evidence" data-finance-route="evidence">
                            <span>"Evidence"</span><strong>"18"</strong><em>"receipts"</em>
                        </button>
                    </div>
                    <dl class="finance-command-kv">
                        <div><dt>"Critical path"</dt><dd>"Payroll delta → vendor approval → HomeTax filing → CFO signoff"</dd></div>
                        <div><dt>"Autonomy ceiling"</dt><dd>"Visual dry-run only; no banking, payroll, tax, or invoice rail executes."</dd></div>
                    </dl>
                </article>

                <article class="finance-command-card finance-cash-pipeline-card">
                    <div class="finance-command-card-head">
                        <div>
                            <p class="screen-anchor">"CASH PIPELINE"</p>
                            <h5>"Invoices, plans, and filing readiness"</h5>
                        </div>
                        <button type="button" data-finance-command-action="open-billing" data-finance-route="billing">"Billing"</button>
                    </div>
                    <div class="finance-cash-lanes" aria-label="Commercial cash pipeline">
                        <button type="button" class="finance-cash-row" data-finance-command-action="stage-invoice" data-finance-route="billing">
                            <span>"Invoice"</span><strong>"Northwind annual plan"</strong><em>"₩184M · due May 27"</em><i class="status-chip success">"ready"</i>
                        </button>
                        <button type="button" class="finance-cash-row" data-finance-command-action="review-plan" data-finance-route="billing">
                            <span>"Plan"</span><strong>"7 contract changes"</strong><em>"2 owner reviews"</em><i class="status-chip warning">"review"</i>
                        </button>
                        <button type="button" class="finance-cash-row" data-finance-command-action="tax-transport" data-finance-route="billing">
                            <span>"Tax"</span><strong>"HomeTax withholding"</strong><em>"118 employees validated"</em><i class="status-chip warning">"draft"</i>
                        </button>
                        <button type="button" class="finance-cash-row" data-finance-command-action="bank-match" data-finance-route="ledger">
                            <span>"Bank"</span><strong>"Shinhan feed match"</strong><em>"92% matched"</em><i class="status-chip success">"matched"</i>
                        </button>
                    </div>
                </article>

                <article class="finance-command-card finance-vendor-risk-card">
                    <div class="finance-command-card-head">
                        <div>
                            <p class="screen-anchor">"RISK QUEUE"</p>
                            <h5>"Spend, renewal, and approval compression"</h5>
                        </div>
                        <button type="button" data-finance-command-action="open-vendor" data-finance-route="vendors">"Vendors"</button>
                    </div>
                    <table class="finance-risk-table">
                        <thead><tr><th>"Object"</th><th>"Owner"</th><th>"SLA"</th><th>"Next"</th></tr></thead>
                        <tbody>
                            <tr data-finance-risk-row="true"><td><strong>"Stripe invoice"</strong><small>"approval can collapse to 1-stage"</small></td><td>"AP"</td><td><span class="status-chip danger">"4.0h"</span></td><td><button type="button" data-finance-command-action="route-stripe" data-finance-route="vendors">"Route"</button></td></tr>
                            <tr data-finance-risk-row="true"><td><strong>"AWS reserved capacity"</strong><small>"commit under-run vs kr-seoul gate"</small></td><td>"SRE"</td><td><span class="status-chip warning">"1d"</span></td><td><button type="button" data-finance-command-action="route-aws" data-finance-route="cloud">"FinOps"</button></td></tr>
                            <tr data-finance-risk-row="true"><td><strong>"Payroll bureau"</strong><small>"NHIS tier delta requires reviewer"</small></td><td>"CFO"</td><td><span class="status-chip danger">"today"</span></td><td><button type="button" data-finance-command-action="route-payroll" data-finance-route="workflow">"Workflow"</button></td></tr>
                        </tbody>
                    </table>
                </article>

                <article class="finance-command-card finance-evidence-lane-card">
                    <div class="finance-command-card-head">
                        <div>
                            <p class="screen-anchor">"EVIDENCE LANE"</p>
                            <h5>"Reviewer packet and communication routes"</h5>
                        </div>
                        <span class="status-chip success">"sealed draft"</span>
                    </div>
                    <ol class="finance-evidence-lane">
                        <li><span>"REC-PAY-2026-04-PARK"</span><strong>"Payroll delta"</strong><em>"blocking · Finance close"</em></li>
                        <li><span>"REC-TAX-HOMETAX-118"</span><strong>"Withholding transport"</strong><em>"review · CFO desk"</em></li>
                        <li><span>"REC-COMM-GOV-221"</span><strong>"Council note"</strong><em>"ready · Community"</em></li>
                    </ol>
                    <div class="finance-mini-actions">
                        <button type="button" data-finance-command-action="mail-brief" data-finance-route="mail">"Mail brief"</button>
                        <button type="button" data-finance-command-action="messenger-room" data-finance-route="messenger">"Messenger room"</button>
                        <button type="button" data-finance-command-action="council-note" data-finance-route="community">"Community note"</button>
                    </div>
                </article>
            </div>

            <div class="finance-route-matrix" aria-label="Commercial operations route matrix">
                <button type="button" data-finance-command-action="route-ledger" data-finance-route="ledger">"Ledger"</button>
                <button type="button" data-finance-command-action="route-vendors" data-finance-route="vendors">"Vendors"</button>
                <button type="button" data-finance-command-action="route-billing" data-finance-route="billing">"Billing · Tax"</button>
                <button type="button" data-finance-command-action="route-leave" data-finance-route="leave">"Leave · Time"</button>
                <button type="button" data-finance-command-action="route-workflow" data-finance-route="workflow">"Workflow"</button>
                <button type="button" data-finance-command-action="route-mail" data-finance-route="mail">"Mail"</button>
                <button type="button" data-finance-command-action="route-community" data-finance-route="community">"Community"</button>
                <button type="button" data-finance-command-action="route-evidence" data-finance-route="evidence">"Evidence"</button>
            </div>
        </section>
    }
}
