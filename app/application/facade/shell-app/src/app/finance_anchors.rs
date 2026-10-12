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
                    <button type="button" data-finance-anchor-action="stage-contract" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Stage contract"</button>
                    <a href="#workflow-studio" data-finance-anchor-action="route-workflow" data-finance-route="workflow">"Workflow gate"</a>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="vendors-cloud">
                <p class="screen-anchor">"OYATIE CLOUD FINOPS"</p>
                <h5>"Cloud substrate proves tenant spend posture"</h5>
                <p>
                    "Oyatie Cloud hosts FD-001 services as real tenant workloads while FinOps, policy, audit, and regional gates stay visible before production claims."
                </p>
                <div class="finance-anchor-actions">
                    <a href="#cockpit-panel-finops" data-finance-anchor-action="route-cloud" data-finance-route="finops">"Open FinOps"</a>
                    <a href="#cockpit-panel-policy" data-finance-anchor-action="route-policy" data-finance-route="policy">"Policy envelope"</a>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="vendors-local">
                <p class="screen-anchor">"LOCAL-ONLY RAIL"</p>
                <h5>"Interactive procurement preview"</h5>
                <p>
                    "Operators can inspect Stripe, AWS Korea, and bank transport paths; no bank, payroll, tax, billing, vendor, or cloud mutation executes."
                </p>
                <div class="finance-anchor-actions">
                    <a href="#resource-panel-audit" data-finance-anchor-action="route-audit" data-finance-route="audit">"Audit trail"</a>
                    <a href="#work-hub" data-finance-anchor-action="route-mail" data-finance-route="mail" data-finance-channel="Mail">"Reviewer Mail"</a>
                </div>
            </article>
        </div>
        <div class="finance-anchor-footer">
            <span data-finance-anchor-status="true">"Sample vendors; live readiness unknown · FD-001 procurement workload dogfoods Oyatie Cloud locally."</span>
            <div class="finance-anchor-routes" aria-label="Vendor connected routes">
                <a href="#finance-panel-ledger" data-finance-anchor-action="route-ledger" data-finance-route="ledger">"Ledger"</a>
                <a href="#finance-panel-billing" data-finance-anchor-action="route-billing" data-finance-route="billing">"Billing"</a>
                <a href="#work-hub" data-finance-anchor-action="route-community" data-finance-route="community" data-finance-channel="Community">"Community"</a>
                <a href="#evidence-spine" data-finance-anchor-action="route-evidence" data-finance-route="evidence">"Evidence"</a>
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
                    <button type="button" data-finance-anchor-action="stage-invoice" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Stage invoice"</button>
                    <a href="#work-hub" data-finance-anchor-action="route-mail" data-finance-route="mail" data-finance-channel="Mail">"Mail customer"</a>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="billing-cloud">
                <p class="screen-anchor">"OYATIE CLOUD TENANT"</p>
                <h5>"Revenue systems run as tenant workloads"</h5>
                <p>
                    "Oyatie Cloud proves production hosting through residency, policy, rollback, and audit receipts before any FD-001 billing surface claims readiness."
                </p>
                <div class="finance-anchor-actions">
                    <a href="#cloud-ops-cockpit" data-finance-anchor-action="route-cloud" data-finance-route="cloud">"Cloud proof"</a>
                    <a href="#cockpit-panel-policy" data-finance-anchor-action="route-policy" data-finance-route="policy">"Tax policy"</a>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="billing-local">
                <p class="screen-anchor">"LOCAL-ONLY CASH CONTROL"</p>
                <h5>"Tax and invoice dry-run only"</h5>
                <p>
                    "Operators can route invoices, HomeTax briefs, and plan reviews visually; no bank, payroll, tax filing, billing send, or cloud mutation executes."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="tax-brief" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Tax brief"</button>
                    <a href="#resource-panel-audit" data-finance-anchor-action="route-audit" data-finance-route="audit">"Audit packet"</a>
                </div>
            </article>
        </div>
        <div class="finance-anchor-footer">
            <span data-finance-anchor-status="true">"Sample billing; live readiness unknown · FD-001 revenue workload dogfoods Oyatie Cloud locally."</span>
            <div class="finance-anchor-routes" aria-label="Billing connected routes">
                <a href="#finance-panel-vendors" data-finance-anchor-action="route-vendors" data-finance-route="vendors">"Vendors"</a>
                <a href="#finance-panel-leave" data-finance-anchor-action="route-leave" data-finance-route="leave">"Leave cost"</a>
                <a href="#workflow-studio" data-finance-anchor-action="route-workflow" data-finance-route="workflow">"Workflow"</a>
                <a href="#evidence-spine" data-finance-anchor-action="route-evidence" data-finance-route="evidence">"Evidence"</a>
            </div>
        </div>
    }
}

pub(super) fn finance_leave_anchor_board() -> impl IntoView {
    view! {
        <div class="finance-anchor-grid" aria-label="Sample leave and time connected previews">
            <article class="finance-anchor-card selected" data-finance-anchor-card="leave-fd001">
                <p class="screen-anchor">"FD-001 PEOPLE COST"</p>
                <h5>"Sample request and connected previews"</h5>
                <p>
                    "Inspect the May 2026 sample request and existing Workflow preview. No approval, workflow execution, or evidence is created."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="approve-leave">"Review sample request"</button>
                    <button type="button" data-finance-anchor-action="route-workflow">"Workflow route"</button>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="leave-cloud">
                <p class="screen-anchor">"OYATIE CLOUD WORKFORCE"</p>
                <h5>"Existing cloud and policy previews"</h5>
                <p>
                    "Inspect sample Cloud and Policy views. Live HR source, verified permission, and policy evidence remain unavailable."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="route-cloud">"Cloud cells"</button>
                    <button type="button" data-finance-anchor-action="route-policy">"PIPA policy"</button>
                </div>
            </article>
            <article class="finance-anchor-card" data-finance-anchor-card="leave-local">
                <p class="screen-anchor">"LOCAL-ONLY TIME RAIL"</p>
                <h5>"Local sample coverage preview"</h5>
                <p>
                    "Choose a sample backup and inspect existing Audit, Mail, Community, and Evidence previews. No assignment, send, receipt, or payroll effect occurs."
                </p>
                <div class="finance-anchor-actions">
                    <button type="button" data-finance-anchor-action="reassign-time">"Preview sample coverage"</button>
                    <button type="button" data-finance-anchor-action="route-audit">"Audit trail"</button>
                </div>
            </article>
        </div>
        <div class="finance-anchor-footer">
            <span data-finance-anchor-status="true">"Sample leave/time only · live HR source and approval readiness unavailable."</span>
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
                        <button type="button" data-trust-proof-action="stage-close" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Stage close"</button>
                        <a href="#workflow-studio" data-trust-proof-action="route-workflow">"Workflow gate"</a>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="ledger-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD FINOPS"</p>
                    <h5>"Sample substrate cost and audit context"</h5>
                    <p>
                        "Oyatie Cloud hosts commercial microservices as tenant workloads while FinOps, resource inventory, release gates, and policy guard the close."
                    </p>
                    <div class="trust-anchor-actions">
                        <a href="#cockpit-panel-finops" data-trust-proof-action="route-finops">"FinOps"</a>
                        <a href="#resource-panel-inventory" data-trust-proof-action="route-inventory">"Resources"</a>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="ledger-local">
                    <p class="screen-anchor">"LOCAL-ONLY LEDGER CONTROL"</p>
                    <h5>"Dense finance preview, no money movement"</h5>
                    <p>
                        "Operators can stage reconciliations, route reviewers, and attach evidence visually; no bank, payroll, tax, invoice, database, or cloud mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <a href="#work-hub" data-trust-proof-action="route-mail" data-finance-channel="Mail">"Reviewer Mail"</a>
                        <a href="#resource-panel-audit" data-trust-proof-action="route-audit">"Audit packet"</a>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Sample ledger; live readiness unknown · FD-001 commercial close workload dogfoods Oyatie Cloud with local visual controls only."
                </span>
                <div class="trust-anchor-routes" aria-label="Ledger preview connected routes">
                    <a href="#finance-panel-billing" data-trust-proof-action="route-billing">"Billing"</a>
                    <a href="#finance-panel-vendors" data-trust-proof-action="route-vendors">"Vendors"</a>
                    <a href="#filing-readiness" data-trust-proof-action="route-filing">"Filing"</a>
                    <a href="#evidence-spine" data-trust-proof-action="route-evidence">"Evidence"</a>
                </div>
            </div>
        </div>
    }
}

pub(super) fn finance_compatible_html(mut body: String) -> String {
    for (native, compatibility) in [
        ("finance-panel-ledger", "ledger-preview"),
        ("finance-panel-vendors", "vendors-spend"),
        ("finance-panel-billing", "billing-tax"),
        ("finance-panel-leave", "leave-time"),
    ] {
        for attribute in ["id", "aria-controls", "href"] {
            let prefix = if attribute == "href" { "#" } else { "" };
            body = body.replace(
                &format!("{attribute}=\"{prefix}{native}\""),
                &format!("{attribute}=\"{prefix}{compatibility}\""),
            );
        }
    }
    body
}
