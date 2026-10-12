use super::*;

mod body;
use body::finance_render_body_view;

pub(super) fn finance_command_board() -> impl IntoView {
    view! {
        <section class="finance-command-board" aria-labelledby="finance-command-board-title">
            <div class="finance-command-head">
                <div>
                    <p class="screen-anchor">"CLOSE COMMAND · OBJECT GRAPH"</p>
                    <h4 id="finance-command-board-title">"Sample April 2026 commercial command center"</h4>
                    <p>
                        "April 2026 fixtures show KRW amounts and sample review states. No financial, filing, message or receipt operation is performed."
                    </p>
                </div>
                <div class="finance-command-actions">
                    <span class="status-chip warning" data-finance-command-status="true">"6 spine examples · 3 risk examples · live readiness unknown"</span>
                    <button type="button" data-finance-command-action="run-close" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Run close dry-run"</button>
                    <button type="button" data-finance-command-action="attach-proof" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Attach proof"</button>
                </div>
            </div>

            <div class="finance-command-grid">
                <article class="finance-command-card finance-close-spine-card">
                    <div class="finance-command-card-head">
                        <div>
                            <p class="screen-anchor">"CLOSE SPINE"</p>
                            <h5>"Every commercial object joins the same package"</h5>
                        </div>
                        <span class="status-chip danger">"3 sample risk examples"</span>
                    </div>
                    <div class="finance-spine-flow" aria-label="Commercial close object spine">
                        <a href="#finance-panel-ledger" class="active" data-finance-command-action="open-payroll" data-finance-route="ledger">
                            <span>"Sample payroll"</span><strong>"₩894M"</strong><em>"review"</em>
                        </a>
                        <a href="#finance-panel-billing" data-finance-command-action="open-tax" data-finance-route="billing">
                            <span>"Tax"</span><strong>"₩118M"</strong><em>"HomeTax"</em>
                        </a>
                        <a href="#finance-panel-vendors" data-finance-command-action="open-vendor" data-finance-route="vendors">
                            <span>"Vendors"</span><strong>"₩69M"</strong><em>"Stripe · AWS"</em>
                        </a>
                        <a href="#finance-panel-billing" data-finance-command-action="open-billing" data-finance-route="billing">
                            <span>"Billing"</span><strong>"₩482M"</strong><em>"12 invoices"</em>
                        </a>
                        <a href="#finance-panel-leave" data-finance-command-action="open-leave" data-finance-route="leave">
                            <span>"Leave"</span><strong>"49.5h"</strong><em>"liability"</em>
                        </a>
                        <a href="#evidence-spine" data-finance-command-action="open-evidence" data-finance-route="evidence">
                            <span>"Evidence"</span><strong>"Unavailable"</strong><em>"verified receipts"</em>
                        </a>
                    </div>
                    <dl class="finance-command-kv">
                        <div><dt>"Critical path"</dt><dd>"Payroll delta → vendor approval → HomeTax filing → CFO signoff"</dd></div>
                        <div><dt>"Autonomy ceiling"</dt><dd>"Dry-run unavailable without an authorized accounting read and evidence contract. No banking, payroll, tax, invoice or receipt operation executes."</dd></div>
                    </dl>
                </article>

                <article class="finance-command-card finance-cash-pipeline-card">
                    <div class="finance-command-card-head">
                        <div>
                            <p class="screen-anchor">"CASH PIPELINE"</p>
                            <h5>"Invoices, plans, and filing readiness"</h5>
                        </div>
                        <a href="#finance-panel-billing" data-finance-command-action="open-billing" data-finance-route="billing">"Billing"</a>
                    </div>
                    <div class="finance-cash-lanes" aria-label="Commercial cash pipeline">
                        <a href="#finance-panel-billing" class="finance-cash-row" data-finance-command-action="stage-invoice" data-finance-route="billing">
                            <span>"Invoice"</span><strong>"Northwind annual plan"</strong><em>"₩184M · due May 27"</em><i class="status-chip success">"sample only"</i>
                        </a>
                        <a href="#finance-panel-billing" class="finance-cash-row" data-finance-command-action="review-plan" data-finance-route="billing">
                            <span>"Plan"</span><strong>"7 contract changes"</strong><em>"2 owner reviews"</em><i class="status-chip warning">"review"</i>
                        </a>
                        <a href="#finance-panel-billing" class="finance-cash-row" data-finance-command-action="tax-transport" data-finance-route="billing">
                            <span>"Tax"</span><strong>"HomeTax withholding"</strong><em>"118 sample employees; not validated"</em><i class="status-chip warning">"draft"</i>
                        </a>
                        <a href="#finance-panel-ledger" class="finance-cash-row" data-finance-command-action="bank-match" data-finance-route="ledger">
                            <span>"Bank"</span><strong>"Shinhan feed match"</strong><em>"Sample 92%; no match performed"</em><i class="status-chip success">"matched"</i>
                        </a>
                    </div>
                </article>

                <article class="finance-command-card finance-vendor-risk-card">
                    <div class="finance-command-card-head">
                        <div>
                            <p class="screen-anchor">"RISK QUEUE"</p>
                            <h5>"Spend, renewal, and approval compression"</h5>
                        </div>
                        <a href="#finance-panel-vendors" data-finance-command-action="open-vendor" data-finance-route="vendors">"Vendors"</a>
                    </div>
                    <table class="finance-risk-table">
                        <thead><tr><th>"Object"</th><th>"Owner"</th><th>"SLA"</th><th>"Next"</th></tr></thead>
                        <tbody>
                            <tr data-finance-risk-row="true"><td><strong>"Stripe invoice"</strong><small>"approval can collapse to 1-stage"</small></td><td>"AP"</td><td><span class="status-chip danger">"4.0h"</span></td><td><a href="#finance-panel-vendors" data-finance-command-action="route-stripe" data-finance-route="vendors">"Route"</a></td></tr>
                            <tr data-finance-risk-row="true"><td><strong>"AWS reserved capacity"</strong><small>"commit under-run vs kr-seoul gate"</small></td><td>"SRE"</td><td><span class="status-chip warning">"1d"</span></td><td><a href="#cockpit-panel-finops" data-finance-command-action="route-aws" data-finance-route="cloud">"FinOps"</a></td></tr>
                            <tr data-finance-risk-row="true"><td><strong>"Payroll bureau"</strong><small>"NHIS tier delta requires reviewer"</small></td><td>"CFO"</td><td><span class="status-chip danger">"today"</span></td><td><a href="#workflow-studio" data-finance-command-action="route-payroll" data-finance-route="workflow">"Workflow"</a></td></tr>
                        </tbody>
                    </table>
                </article>

                <article class="finance-command-card finance-evidence-lane-card">
                    <div class="finance-command-card-head">
                        <div>
                            <p class="screen-anchor">"EVIDENCE LANE"</p>
                            <h5>"Reviewer packet and communication routes"</h5>
                        </div>
                        <span class="status-chip">"sample receipt examples"</span>
                    </div>
                    <ol class="finance-evidence-lane">
                        <li><span>"REC-PAY-2026-04-PARK"</span><strong>"Payroll delta"</strong><em>"blocking · Finance close"</em></li>
                        <li><span>"REC-TAX-HOMETAX-118"</span><strong>"Withholding transport"</strong><em>"review · CFO desk"</em></li>
                        <li><span>"REC-COMM-GOV-221"</span><strong>"Council note"</strong><em>"illustrative · not published"</em></li>
                    </ol>
                    <div class="finance-mini-actions">
                        <button type="button" data-finance-command-action="mail-brief" data-finance-route="mail" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Mail brief"</button>
                        <button type="button" data-finance-command-action="messenger-room" data-finance-route="messenger" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Messenger room"</button>
                        <button type="button" data-finance-command-action="council-note" data-finance-route="community" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Community note"</button>
                    </div>
                </article>
            </div>

            <div class="finance-route-matrix" aria-label="Commercial operations route matrix">
                <a href="#finance-panel-ledger" data-finance-command-action="route-ledger" data-finance-route="ledger">"Ledger"</a>
                <a href="#finance-panel-vendors" data-finance-command-action="route-vendors" data-finance-route="vendors">"Vendors"</a>
                <a href="#finance-panel-billing" data-finance-command-action="route-billing" data-finance-route="billing">"Billing · Tax"</a>
                <a href="#finance-panel-leave" data-finance-command-action="route-leave" data-finance-route="leave">"Leave · Time"</a>
                <a href="#workflow-studio" data-finance-command-action="route-workflow" data-finance-route="workflow">"Workflow"</a>
                <a href="#work-hub" data-finance-command-action="route-mail" data-finance-route="mail" data-finance-channel="Mail">"Mail"</a>
                <a href="#work-hub" data-finance-command-action="route-community" data-finance-route="community" data-finance-channel="Community">"Community"</a>
                <a href="#evidence-spine" data-finance-command-action="route-evidence" data-finance-route="evidence">"Evidence"</a>
            </div>
        </section>
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub(super) struct FinanceSampleScope {
    pub commercial_examples: bool,
    pub finops_examples: bool,
    pub filing_examples: bool,
}

pub(super) fn finance_sample_scope(context: OperatorContext) -> FinanceSampleScope {
    let grants = crate::shell_context_grants::context_action_grants(context);
    let commercial = grants.contains("accounting.close") || grants.contains("tenancy.administer");
    FinanceSampleScope {
        commercial_examples: commercial,
        finops_examples: grants.contains("finops.review") || commercial,
        filing_examples: commercial,
    }
}

pub(super) fn finance_body_html(envelope: &TenantRenderEnvelope, native: bool) -> String {
    let body = envelope
        .finance_body_projection
        .as_ref()
        .filter(|projection| projection.context == envelope.context && !projection.body.is_empty())
        .map(|projection| projection.body.clone())
        .unwrap_or_default();
    if native {
        body
    } else {
        finance_compatible_html(body)
    }
}

pub(super) fn finance_sample_source(envelope: &TenantRenderEnvelope) -> String {
    serde_json::to_string(&(
        envelope.context,
        daily_sample_label(envelope),
        finance_sample_scope(envelope.context),
        HubSample::envelope_source(envelope),
        finance_body_html(envelope, true),
    ))
    .expect("complete native Finance sample source is serializable")
}

fn finance_available(available: Signal<bool>) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        available.try_get().unwrap_or(false)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        available.try_get_untracked().unwrap_or(false)
    }
}

pub(super) fn finance_commercial_service(
    envelope: TenantRenderEnvelope,
    available: Signal<bool>,
) -> impl IntoView {
    finance_preview_view(envelope, move || finance_available(available), true)
}

fn finance_preview_view(
    envelope: TenantRenderEnvelope,
    available: impl Fn() -> bool + Copy + Send + Sync + 'static,
    native: bool,
) -> impl IntoView {
    view! {
        <section id="finance-preview" data-finance-source=finance_sample_source(&envelope)
            data-finance-context=envelope.context.id() data-finance-label=daily_sample_label(&envelope)
            data-finance-hub-source=HubSample::envelope_source(&envelope)
            data-finance-scope=serde_json::to_string(&finance_sample_scope(envelope.context)).expect("sample scope is serializable")
            data-finance-source-available=move || available().to_string()
            data-finance-native=native.to_string()
            data-finance-available=move || available().to_string()
            data-finance-mode=if cfg!(target_arch = "wasm32") { "island" } else { "ssr" }
            aria-label="Anonymous Finance and Leave samples">
            <p data-finance-availability="true" role="status">{format!("Observed anonymous sample: {}. Financial examples follow explicit sample grants, not verified tenant or role permissions. Live financial readiness is unknown.", daily_sample_label(&envelope))}</p>
            <p id="finance-unavailable-reason">"Reconciliation, export, close dry-run, vendor changes, invoice, tax brief, communications and evidence attachment are unavailable: no authenticated accounting read, mutation or verified evidence contract is bound. No financial operation or receipt was performed."</p>
            <p data-finance-route-status="true" role="status">"Existing sample routes only; no draft, message or receipt is created."</p>
            <p data-finance-reset="true" role="status" hidden>"Sample source changed; prior Finance search, detail and scroll state cleared."</p>
            {finance_render_body_view(&envelope, native)}
        </section>
    }
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_finance_preview(envelope: &TenantRenderEnvelope) -> String {
    finance_preview_view(envelope.clone(), || true, false)
        .into_view()
        .to_html()
}

pub(super) fn finance_rail_html(scope: FinanceSampleScope, group: &str, native: bool) -> String {
    let entries: &[(&str, &str, &str)] = match group {
        "commercial" if scope.commercial_examples => &[
            ("finance-panel-ledger", "ledger-preview", "Ledger"),
            (
                "finance-panel-vendors",
                "vendors-spend",
                "Vendors &amp; spend",
            ),
            ("finance-panel-billing", "billing-tax", "Billing &amp; tax"),
        ],
        "finops" if scope.finops_examples => {
            &[("cockpit-panel-finops", "cockpit-panel-finops", "FinOps")]
        }
        "filing" if scope.filing_examples => {
            &[("filing-readiness", "filing-readiness", "Filing readiness")]
        }
        _ => &[],
    };
    entries
        .iter()
        .map(|(native_id, compatibility_id, label)| {
            format!(
                r##"<a class="rail-nav" href="#{id}" data-finance-rail="{group}">{label}</a>"##,
                id = if native { native_id } else { compatibility_id }
            )
        })
        .collect()
}
