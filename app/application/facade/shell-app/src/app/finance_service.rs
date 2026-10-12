use super::*;

pub(super) fn finance_service_markup(context: OperatorContext) -> impl IntoView {
    let scope = finance_sample_scope(context);
    view! {
            <section
                id="finance-commercial-service"
                class="finance-commercial-service panel"
                data-finance-service="true"
                aria-labelledby="finance-service-title"
            >
                <div class="finance-service-head">
                    <div>
                        <p class="screen-anchor">{if scope.commercial_examples { "MONEY · SAMPLE COMMERCIAL OPERATIONS" } else { "PEOPLE · LEAVE & TIME" }}</p>
                        <h3 id="finance-service-title">{if scope.commercial_examples { "Sample Finance and leave-time" } else { "Leave & time sample" }}</h3>
                    </div>
    {scope.commercial_examples.then(|| view! {
                    <div class="finance-service-actions">
                        <span class="status-chip warning" data-finance-status="true">"Sample only · live close readiness unknown"</span>
                        <button type="button" data-finance-action="reconcile" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Reconcile"</button>
                        <button type="button" data-finance-action="export-pack" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Export pack"</button>
                    </div>
    })}
                </div>

    {scope.commercial_examples.then(|| view! {
                <p>"All commercial figures are April 2026 fixtures. Fixture as-of 2026-04-30; live observation unavailable. KRW amounts; no bank balance, invoice or contract has been read."</p>
                <div class="finance-kpi-strip" aria-label="Illustrative April 2026 Finance summary; no live measurement">
                    <span><small>"Cash runway"</small><strong>"18.4 mo"</strong><em>"Sample KRW 12.6B balance"</em></span>
                    <span><small>"Open invoices"</small><strong>"Sample KRW 482M"</strong><em>"12 sample invoices · 2 sample overdue"</em></span>
                    <span><small>"Sample vendor risk"</small><strong>"1 high · 1 medium · 1 low"</strong><em>"Stripe · AWS Korea · Shinhan Bank"</em></span>
                    <span><small>"Leave liability"</small><strong>"Sample KRW 53M"</strong><em>"49.5h this week"</em></span>
                </div>


    })}            <div class="finance-service-shell">
    {scope.commercial_examples.then(|| view! {
                    <aside class="finance-ledger-rail" aria-label="Finance sections">
                        <p class="screen-anchor">"FINANCE VIEWS"</p>
                        <a href="#finance-panel-ledger" data-finance-side="ledger">"Ledger close"</a>
                        <a href="#finance-panel-vendors" data-finance-side="vendors">"Vendors & spend"</a>
                        <a href="#finance-panel-billing" data-finance-side="billing">"Billing & tax"</a>
                        <a href="#finance-panel-leave" data-finance-side="leave">"Leave & time"</a>
                        <div class="finance-close-card">
                            <small>"SAMPLE APRIL CLOSE"</small>
                            <strong>"73%"</strong>
                            <span class="score-bar" role="progressbar" aria-valuenow="73" aria-valuemin="0" aria-valuemax="100" aria-label="Sample April close readiness: 73%" style="--bar: 73%"><em aria-hidden="true"></em></span>
                            <p>"Sample payroll, withholding, and vendor accruals illustrate a 2-person review; no close has been performed."</p>
                        </div>
                    </aside>


    })}                <div class="finance-service-main" style=if scope.commercial_examples { "" } else { "grid-column: 1 / -1" }>
                        // A-2: finance tablist — aria-orientation + id/aria-controls + role=tabpanel
                        <div class="finance-tabs" role="tablist" aria-label="Finance service views" aria-orientation="horizontal">
    {scope.commercial_examples.then(|| view! {
                            <button type="button" id="finance-tab-ledger" class="active" data-finance-tab="ledger" role="tab" aria-selected="true" aria-controls="finance-panel-ledger">"Ledger"</button>
    })}
    {scope.commercial_examples.then(|| view! {
                            <button type="button" id="finance-tab-vendors" data-finance-tab="vendors" role="tab" aria-selected="false" aria-controls="finance-panel-vendors">"Vendors"</button>
    })}
    {scope.commercial_examples.then(|| view! {
                            <button type="button" id="finance-tab-billing" data-finance-tab="billing" role="tab" aria-selected="false" aria-controls="finance-panel-billing">"Billing"</button>
    })}
                            <button type="button" id="finance-tab-leave" class=if scope.commercial_examples { "" } else { "active" } data-finance-tab="leave" role="tab" aria-selected=(!scope.commercial_examples).to_string() aria-controls="finance-panel-leave">"Leave & time"</button>
                        </div>

    {scope.commercial_examples.then(|| view! {
                        <article id="finance-panel-ledger" class="finance-panel active" data-finance-panel="ledger" role="tabpanel" aria-labelledby="finance-tab-ledger">
                            {payroll_preview_detail(PayrollPreview::Finance)}
                            <div class="finance-panel-copy">
                                <p class="screen-anchor">"LEDGER · CLOSE PACKAGE"</p>
                                <h4 id="ledger-panel-title">"Ledger close cockpit"</h4>
                                <p>"Illustrative April 2026 journal examples. Fixture as-of 2026-04-30; live observation unavailable. No authorized journal read, reconciliation, close or evidence receipt is connected."</p>
                            </div>
                            <div class="ledger-layout">
                                // A-6: role=progressbar + aria-value* on reconciliation bars
                                <div class="ledger-reconciliation">
                                    <span role="progressbar" aria-valuenow="92" aria-valuemin="0" aria-valuemax="100" aria-label="Sample Bank feed match: 92%" style="--bar: 92%"><em aria-hidden="true">"Sample Bank feed match · 92%"</em></span>
                                    <span role="progressbar" aria-valuenow="86" aria-valuemin="0" aria-valuemax="100" aria-label="Sample payroll accrual: 86%" style="--bar: 86%"><em aria-hidden="true">"Sample payroll accrual · 86%"</em></span>
                                    <span role="progressbar" aria-valuenow="68" aria-valuemin="0" aria-valuemax="100" aria-label="Sample Vendor accrual: 68%" style="--bar: 68%"><em aria-hidden="true">"Sample Vendor accrual · 68%"</em></span>
                                    <span role="progressbar" aria-valuenow="51" aria-valuemin="0" aria-valuemax="100" aria-label="Sample Tax evidence: 51%" style="--bar: 51%"><em aria-hidden="true">"Sample Tax evidence · 51%"</em></span>
                                </div>
                                <div class="finance-table-scroll" role="region" aria-label="Scrollable sample journal table, April 2026" tabindex="0" data-finance-scroll="journal" data-finance-focus="scroll:journal"><table class="finance-table">
                                    <thead><tr><th>"Time"</th><th>"Account"</th><th>"Object"</th><th>"Amount"</th><th>"State"</th></tr></thead>
                                    <tbody inner_html=finance_journal_rows_html()></tbody>
                                </table></div>
                            </div>
                            {finance_command_board()}
                            {ledger_preview_anchor_board()}
                        </article>


    })}{scope.commercial_examples.then(|| view! {
                        <article id="finance-panel-vendors" class="finance-panel" data-finance-panel="vendors" role="tabpanel" aria-labelledby="finance-tab-vendors">
                            <div class="finance-panel-copy">
                                <p class="screen-anchor">"PROCUREMENT · VENDORS"</p>
                                <h4 id="vendors-panel-title">"Vendors & spend control"</h4>
                                <p>"Contracts, approvals, owners, and cost-of-delay are tied to the same service graph as workflow and mail."</p>
                            </div>
                            <div class="vendor-toolbar">
                                <label><span aria-hidden="true">"⌕"</span><input data-vendor-search="true" data-finance-focus="vendor-search" aria-label="Search vendors" placeholder="Search vendor, owner, contract..." /></label>
                                <button type="button" data-finance-action="add-vendor" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"+ Vendor"</button>
                            </div>
                            <div class="finance-table-scroll" role="region" aria-label="Scrollable sample vendor table, April 2026" tabindex="0" data-finance-scroll="vendors" data-finance-focus="scroll:vendors"><table class="finance-table vendor-table">
                                <thead><tr><th>"Vendor"</th><th>"Owner"</th><th>"Monthly"</th><th>"Renewal"</th><th>"Risk"</th><th>"Action"</th></tr></thead>
                                <tbody inner_html=finance_vendor_rows_html()></tbody>
                            </table></div>
                            <p data-finance-projected="vendor-status" data-finance-vendor-status="true" role="status">"3 of 3 sample vendors shown"</p>
                            <p data-finance-projected="empty" data-finance-vendor-empty="true" hidden>"No sample vendors match."</p>
                            {finance_vendors_anchor_board()}
                        </article>


    })}{scope.commercial_examples.then(|| view! {
                        <article id="finance-panel-billing" class="finance-panel" data-finance-panel="billing" role="tabpanel" aria-labelledby="finance-tab-billing">
                            <div class="finance-panel-copy">
                                <p class="screen-anchor">"BILLING · TAX"</p>
                                <h4 id="billing-panel-title">"Billing, plans, and filings"</h4>
                                <p>"Customer invoices, plan changes, HomeTax filing readiness, and payment evidence stay visible next to operations."</p>
                            </div>
                            <div class="billing-grid">
                                <article><p class="screen-anchor">"REVENUE"</p><strong>"Sample KRW 2.31B / year"</strong><span>"ARR staged · 42 active contracts"</span><button type="button" data-finance-action="send-invoice" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Stage invoice"</button></article>
                                <article><p class="screen-anchor">"TAX FILING"</p><strong>"Sample 64%"</strong><span>"HomeTax transport awaiting reviewer"</span><button type="button" data-finance-action="tax-brief" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Draft brief"</button></article>
                                <article><p class="screen-anchor">"PLAN CHANGES"</p><strong>"7 sample changes"</strong><span>"2 require billing owner review"</span><button type="button" data-sidepeek-trigger="billing-plans" data-sidepeek-title="Plan change queue" data-sidepeek-id="BILL-PLAN-7" data-sidepeek-desc="Plan changes are staged queue items with no payment or billing mutation." data-sidepeek-owner="Revenue ops" data-sidepeek-risk="Review" data-sidepeek-sla="Visual only" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Open queue"</button></article>
                            </div>
                            {finance_billing_anchor_board()}
                        </article>


    })}                    {(!scope.commercial_examples).then(|| payroll_preview_detail(PayrollPreview::Finance))}
                        {leave_time_panel(context)}
                    </div>

    {scope.commercial_examples.then(|| view! {
                    <aside class="finance-context-rail" aria-label="Finance provenance">
                        <p class="screen-anchor">"CLOSE PROVENANCE"</p>
                        <dl class="service-kv">
                            <div><dt>"Workflow"</dt><dd>"April close package"</dd></div>
                            <div><dt>"Receipts"</dt><dd>"Unavailable; examples only"</dd></div>
                            <div><dt>"Reviewers"</dt><dd>"CFO · payroll · tax"</dd></div>
                        </dl>
                        <p class="screen-anchor">"LOCAL NOTIFICATIONS"</p>
                        <ol class="notification-stack">
                            <li>"Vendor approval waiting"</li>
                            <li>"Tax brief creation unavailable"</li>
                            <li>"No real money moves"</li>
                        </ol>
                    </aside>

    })}            </div>
            </section>
        }
}

pub(super) fn finance_journal_rows_html() -> String {
    [
        ("sample-journal-payroll", "Payroll payable", "Sample APR payroll close", "894,000,000", "review"),
        ("sample-journal-tax", "Withholding tax", "HomeTax draft", "118,400,000", "sample match; not performed"),
        ("sample-journal-stripe", "Vendor accrual", "Stripe invoice", "4,820,000", "sample blocker"),
        ("sample-journal-leave", "Leave liability", "Yoon Tae-min risk", "53,000,000", "advisory"),
    ].into_iter().map(|(id, account, title, amount, state)| {
        let record = daily_escape(&serde_json::to_string(&(id, account, title, amount, state, "sample-tenant", "sample-entity-kr", "2026-04", "verified evidence unavailable")).expect("sample journal record is serializable"));
        format!(r#"<tr data-finance-record="{record}" data-finance-journal="{id}"><td>Sample April 2026</td><td>{account}</td><td><details data-finance-detail="{id}"><summary data-finance-focus="{id}">Inspect {title} example</summary><dl class="service-kv"><div><dt>Journal</dt><dd>{id}; illustrative, not a native journal ID</dd></div><div><dt>Tenant / entity</dt><dd>sample-tenant / sample-entity-kr; not verified</dd></div><div><dt>Period / fixture as-of</dt><dd>2026-04 / 2026-04-30; live observation unavailable</dd></div><div><dt>Evidence / live read</dt><dd>Unavailable; no receipt or native journal read connected</dd></div></dl></details></td><td>Sample KRW {amount} minor units</td><td>{state}</td></tr>"#)
    }).collect()
}

pub(super) fn finance_vendor_rows_html() -> String {
    [
        ("sample-vendor-stripe", "Stripe", "Payments · invoice INV-4281", "Finance", "4,820,000", "2026-05-31", "High"),
        ("sample-vendor-aws", "AWS Korea", "Cloud infrastructure · reserved capacity", "SRE", "63,400,000", "2026-06-12", "Medium"),
        ("sample-vendor-shinhan", "Shinhan Bank", "Payroll and withholding transport", "CFO", "1,200,000", "2027-01-10", "Low"),
    ].into_iter().map(|(id, name, contract, owner, amount, renewal, risk)| {
        let record = daily_escape(&serde_json::to_string(&(id, name, contract, owner, amount, renewal, risk, "2026-04", "sample-tenant", "sample-entity-kr", "verified evidence unavailable")).expect("sample vendor record is serializable"));
        format!(r#"<tr data-vendor-row="true" data-finance-record="{record}" data-finance-vendor="{id}" data-finance-search-text="{name} {contract} {owner} {risk}"><td><strong>{name}</strong><small>{contract}</small></td><td>{owner}</td><td>Sample KRW {amount} minor units / month</td><td>Sample {renewal}</td><td>{risk} sample</td><td><details data-finance-detail="{id}"><summary data-finance-focus="{id}">Inspect {name} sample</summary><dl class="service-kv"><div><dt>Vendor / tenant / entity</dt><dd>{id} / sample-tenant / sample-entity-kr; not verified</dd></div><div><dt>Period / fixture as-of</dt><dd>2026-04 / 2026-04-30; live observation unavailable</dd></div><div><dt>Contract / evidence</dt><dd>{contract}; verified evidence unavailable</dd></div><div><dt>Live review / approval</dt><dd>Unavailable; no vendor approval or spend optimization performed</dd></div></dl></details></td></tr>"#)
    }).collect()
}
