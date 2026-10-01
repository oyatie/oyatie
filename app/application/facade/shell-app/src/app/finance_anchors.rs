use super::*;

pub(super) fn finance_vendors_anchor_board() -> impl IntoView {
    view! {
        <div class="finance-anchor-grid" aria-label="FD-001 vendor spend and Oyatie Cloud tenant proof">
            <article class="finance-anchor-card selected" data-finance-anchor-card="vendors-fd001">
                <p class="screen-anchor">"FD-001 VENDOR WORKLOAD"</p>
                <h5>"Procurement is part of product delivery"</h5>
                <p>
                    "Vendor approvals, spend controls, Workflow tasks, Mail briefs, and Community notes are FD-001 tenant workloads sharing one commercial graph."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="stage-contract">"Stage contract"</button>
                    <button type="button" data-finance-anchor-action="route-workflow">"Workflow gate"</button>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="vendors-cloud">
                <p class="screen-anchor">"OYATIE CLOUD FINOPS"</p>
                <h5>"Cloud substrate proves tenant spend posture"</h5>
                <p>
                    "Oyatie Cloud hosts FD-001 services as real tenant workloads while FinOps, policy, audit, and regional gates stay visible before production claims."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="route-cloud">"Open FinOps"</button>
                    <button type="button" data-finance-anchor-action="route-policy">"Policy envelope"</button>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="vendors-local">
                <p class="screen-anchor">"LOCAL-ONLY RAIL"</p>
                <h5>"Interactive procurement preview"</h5>
                <p>
                    "Operators can inspect Stripe, AWS Korea, and bank transport paths; no bank, payroll, tax, billing, vendor, or cloud mutation executes."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="route-audit">"Audit trail"</button>
                    <button type="button" data-finance-anchor-action="route-mail">"Reviewer Mail"</button>
                </div>
            </article>
        </div>
        <div class="finance-anchor-footer">
            <span data-finance-anchor-status="true">"Vendors ready · FD-001 procurement workload dogfoods Oyatie Cloud locally."</span>
            <div class="finance-anchor-routes" aria-label="Vendor connected routes">
                <button type="button" data-finance-anchor-action="route-ledger">"Ledger"</button>
                <button type="button" data-finance-anchor-action="route-billing">"Billing"</button>
                <button type="button" data-finance-anchor-action="route-community">"Community"</button>
                <button type="button" data-finance-anchor-action="route-evidence">"Evidence"</button>
            </div>
        </div>
    }
}

pub(super) fn finance_billing_anchor_board() -> impl IntoView {
    view! {
        <div class="finance-anchor-grid" aria-label="FD-001 billing tax and Oyatie Cloud tenant proof">
            <article class="finance-anchor-card selected" data-finance-anchor-card="billing-fd001">
                <p class="screen-anchor">"FD-001 REVENUE WORKLOAD"</p>
                <h5>"Billing supports master-plan product delivery"</h5>
                <p>
                    "Invoices, plan changes, tax briefs, customer Mail, and evidence receipts stay inside FD-001 so product delivery remains the master-plan goal."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="stage-invoice">"Stage invoice"</button>
                    <button type="button" data-finance-anchor-action="route-mail">"Mail customer"</button>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="billing-cloud">
                <p class="screen-anchor">"OYATIE CLOUD TENANT"</p>
                <h5>"Revenue systems run as tenant workloads"</h5>
                <p>
                    "Oyatie Cloud proves production hosting through residency, policy, rollback, and audit receipts before any FD-001 billing surface claims readiness."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="route-cloud">"Cloud proof"</button>
                    <button type="button" data-finance-anchor-action="route-policy">"Tax policy"</button>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="billing-local">
                <p class="screen-anchor">"LOCAL-ONLY CASH CONTROL"</p>
                <h5>"Tax and invoice dry-run only"</h5>
                <p>
                    "Operators can route invoices, HomeTax briefs, and plan reviews visually; no bank, payroll, tax filing, billing send, or cloud mutation executes."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="tax-brief">"Tax brief"</button>
                    <button type="button" data-finance-anchor-action="route-audit">"Audit packet"</button>
                </div>
            </article>
        </div>
        <div class="finance-anchor-footer">
            <span data-finance-anchor-status="true">"Billing ready · FD-001 revenue workload dogfoods Oyatie Cloud locally."</span>
            <div class="finance-anchor-routes" aria-label="Billing connected routes">
                <button type="button" data-finance-anchor-action="route-vendors">"Vendors"</button>
                <button type="button" data-finance-anchor-action="route-leave">"Leave cost"</button>
                <button type="button" data-finance-anchor-action="route-workflow">"Workflow"</button>
                <button type="button" data-finance-anchor-action="route-evidence">"Evidence"</button>
            </div>
        </div>
    }
}

pub(super) fn finance_leave_anchor_board() -> impl IntoView {
    view! {
        <div class="finance-anchor-grid" aria-label="FD-001 leave time and Oyatie Cloud tenant proof">
            <article class="finance-anchor-card selected" data-finance-anchor-card="leave-fd001">
                <p class="screen-anchor">"FD-001 PEOPLE COST"</p>
                <h5>"Leave and time feed payroll, schedule, and tenant workload delivery"</h5>
                <p>
                    "Leave approvals, overtime risk, payroll cutoff, Workflow routes, Mail, and Community updates are FD-001 tenant workload evidence, not a side module."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="approve-leave">"Approve preview"</button>
                    <button type="button" data-finance-anchor-action="route-workflow">"Workflow route"</button>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="leave-cloud">
                <p class="screen-anchor">"OYATIE CLOUD WORKFORCE"</p>
                <h5>"Substrate hosts workforce-cost tenant surfaces"</h5>
                <p>
                    "Oyatie Cloud proves the workforce substrate with regional policy, audit receipts, and deployment gates before leave or payroll workloads claim readiness."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="route-cloud">"Cloud cells"</button>
                    <button type="button" data-finance-anchor-action="route-policy">"PIPA policy"</button>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="leave-local">
                <p class="screen-anchor">"LOCAL-ONLY TIME RAIL"</p>
                <h5>"Interactive liability preview"</h5>
                <p>
                    "Operators can reassign coverage, preview timesheet locks, and brief reviewers; no bank, payroll, tax, billing, HRIS, or cloud mutation executes."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="reassign-time">"Reassign time"</button>
                    <button type="button" data-finance-anchor-action="route-audit">"Audit trail"</button>
                </div>
            </article>
        </div>
        <div class="finance-anchor-footer">
            <span data-finance-anchor-status="true">"Leave/time ready · FD-001 people-cost workload dogfoods Oyatie Cloud locally."</span>
            <div class="finance-anchor-routes" aria-label="Leave time connected routes">
                <button type="button" data-finance-anchor-action="route-ledger">"Ledger"</button>
                <button type="button" data-finance-anchor-action="route-mail">"Reviewer Mail"</button>
                <button type="button" data-finance-anchor-action="route-community">"Community"</button>
                <button type="button" data-finance-anchor-action="route-evidence">"Evidence"</button>
            </div>
        </div>
    }
}

pub(super) fn ledger_preview_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board ledger-trust-board" aria-label="FD-001 ledger close and Oyatie Cloud commercial proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected" data-trust-proof-card="ledger-fd001">
                    <p class="screen-anchor">"FD-001 CLOSE PACKAGE"</p>
                    <h5>"Ledger is the commercial product spine"</h5>
                    <p>
                        "Payroll, filing, vendors, billing, leave/time, Workflow, Mail, Community, and audit receipts resolve into one FD-001 tenant workload close packet."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="stage-close">"Stage close"</button>
                        <button type="button" data-trust-proof-action="route-workflow">"Workflow gate"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="ledger-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD FINOPS"</p>
                    <h5>"Substrate cost and audit prove readiness"</h5>
                    <p>
                        "Oyatie Cloud hosts commercial microservices as tenant workloads while FinOps, resource inventory, release gates, and policy guard the close."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-finops">"FinOps"</button>
                        <button type="button" data-trust-proof-action="route-inventory">"Resources"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="ledger-local">
                    <p class="screen-anchor">"LOCAL-ONLY LEDGER CONTROL"</p>
                    <h5>"Dense finance preview, no money movement"</h5>
                    <p>
                        "Operators can stage reconciliations, route reviewers, and attach evidence visually; no bank, payroll, tax, invoice, database, or cloud mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-mail">"Reviewer Mail"</button>
                        <button type="button" data-trust-proof-action="route-audit">"Audit packet"</button>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Ledger ready · FD-001 commercial close workload dogfoods Oyatie Cloud with local visual controls only."
                </span>
                <div class="trust-anchor-routes" aria-label="Ledger preview connected routes">
                    <button type="button" data-trust-proof-action="route-billing">"Billing"</button>
                    <button type="button" data-trust-proof-action="route-vendors">"Vendors"</button>
                    <button type="button" data-trust-proof-action="route-filing">"Filing"</button>
                    <button type="button" data-trust-proof-action="route-evidence">"Evidence"</button>
                </div>
            </div>
        </div>
    }
}
