use super::*;

pub(super) fn tenant_rbac_board(
    envelope: TenantRenderEnvelope,
    available: Signal<bool>,
) -> impl IntoView {
    view! {
        <section
            class="tenant-rbac-board"
            aria-labelledby="service-board-title"
            id="business-logics"
        >
            <div class="service-board-head">
                <div>
                    <p class="screen-anchor">"SAMPLE SERVICE GRAPH"</p>
                    <h3 id="service-board-title">"Sample corporate operations graph"</h3>
                </div>
                <span class="status-chip success">{format!("{} sample service cards", envelope.modules.len())}</span>
            </div>

            {governance_preview(envelope.clone(), available)}

            {payroll_preview_card()}

            {finance_sample_scope(envelope.context).filing_examples.then(|| view! {
                {finance_filing_card(&envelope)}
            })}

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
                        <button type="button" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Stage filing"</button>
                        <a href="#finance-panel-billing">"Billing · tax"</a>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="filing-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD RESIDENCY"</p>
                    <h5>"Substrate proves regional pack posture"</h5>
                    <p>
                        "Oyatie Cloud shows PIPA-aware residency, policy envelope, release gates, audit freshness, and rollback posture before any tax workload readiness claim."
                    </p>
                    <div class="trust-anchor-actions">
                        <a href="#cockpit-panel-policy">"PIPA policy"</a>
                        <a href="#cloud-ops-cockpit">"Cloud cells"</a>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="filing-local">
                    <p class="screen-anchor">"LOCAL-ONLY FILING RAIL"</p>
                    <h5>"Sample reviewer context; not submitted"</h5>
                    <p>
                        "Operators can inspect readiness, stage a reviewer packet, and route council notes visually; no HomeTax, bank, payroll, billing, mail, or cloud mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <a href="#work-hub" data-finance-channel="Mail">"Reviewer Mail"</a>
                        <a href="#evidence-spine">"Receipt spine"</a>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Sample filing; agency acceptance unavailable · FD-001 localization workload dogfoods Oyatie Cloud with local-only submission controls."
                </span>
                <div class="trust-anchor-routes" aria-label="Filing readiness connected routes">
                    <a href="#finance-panel-ledger">"Finance close"</a>
                    <a href="#work-hub" data-finance-channel="Community">"Community note"</a>
                    <a href="#resource-panel-audit">"Audit ledger"</a>
                    <a href="#modules-title">"Catalog"</a>
                </div>
            </div>
        </div>
    }
}

pub(super) fn governance_sample_visible(context: OperatorContext) -> bool {
    matches!(
        context,
        OperatorContext::TenantAdmin | OperatorContext::CorporateOffice
    )
}

pub(super) fn governance_sample_source(envelope: &TenantRenderEnvelope) -> String {
    serde_json::to_string(&(
        envelope.context,
        daily_sample_label(envelope),
        HubSample::envelope_source(envelope),
        governance_body_html(envelope, true),
    ))
    .expect("complete native Governance sample source is serializable")
}

fn governance_available(available: Signal<bool>) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        available.try_get().unwrap_or(false)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        available.try_get_untracked().unwrap_or(false)
    }
}

fn governance_preview(envelope: TenantRenderEnvelope, available: Signal<bool>) -> impl IntoView {
    let visible = governance_sample_visible(envelope.context);
    view! {
        <section id="governance-preview" aria-label="Anonymous Governance sample"
            data-governance-source=governance_sample_source(&envelope)
            data-governance-context=envelope.context.id() data-governance-label=daily_sample_label(&envelope)
            data-governance-hub-source=HubSample::envelope_source(&envelope)
            data-governance-visible=visible.to_string()
            data-governance-source-available=move || governance_available(available).to_string()
            data-governance-mode=if cfg!(target_arch = "wasm32") { "island" } else { "ssr" }
            data-governance-available=move || (visible && governance_available(available)).to_string()
            inner_html=governance_native_body_html(&envelope)></section>
    }
}

pub(super) fn governance_body_html(envelope: &TenantRenderEnvelope, native: bool) -> String {
    let label = daily_escape(&daily_sample_label(envelope));
    if !governance_sample_visible(envelope.context) {
        return format!(
            r#"<p data-governance-availability="true" role="status">Observed anonymous sample: {label}. Executive Governance examples are omitted for the clinician sample. No tenant or role is verified; live Governance remains unavailable.</p>"#
        );
    }
    format!(
        r##"<p data-governance-availability="true" role="status">Observed anonymous sample: {label}. Executive examples only; no tenant or role is verified.</p><p data-governance-reset="true" role="status" hidden>Sample source changed; prior local search, filter, selection and inspection cleared.</p>{logic}{command}
<article id="governance-analytics-summary" class="service-card governance-card" data-governance-workflow-source="{workflow_source}"><p class="screen-anchor">SAMPLE GOVERNANCE SUMMARY</p><h4>Example policy, risk and evidence</h4><p class="service-card-brief">May 2026–April 2027 illustrative fixtures. Live posture, freshness, attestations and durable signed receipts are unavailable.</p><dl class="service-kv"><div><dt>Sample risks</dt><dd>5 illustrated records</dd></div><div><dt>Verified receipts</dt><dd>Unavailable</dd></div><div><dt>Sample workflow</dt><dd>{workflow}</dd></div></dl><nav class="service-card-actions" aria-label="Existing Governance sample views"><a href="#governance-analytics" data-governance-route="command" data-service-action="governance-command">Open command sample</a><a href="#governance-risk-matrix" data-governance-route="risk" data-service-action="governance-risk">Open risk examples</a>{community}{evidence}</nav></article>
<p data-governance-route-status="true" role="status">Existing sample views only; no action, draft, message or receipt is created.</p>"##,
        logic = business_logic_os_html(native),
        command = governance_command_html(&envelope.workflow.name, native),
        workflow = daily_escape(&envelope.workflow.name),
        workflow_source = daily_escape(
            &serde_json::to_string(&envelope.workflow)
                .expect("native sample workflow is serializable")
        ),
        community = governance_route_html("community", "Community", native),
        evidence = governance_route_html("evidence", "Evidence", native)
    )
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_governance_preview(envelope: &TenantRenderEnvelope) -> String {
    let templates = if governance_sample_visible(envelope.context) {
        [HubChannel::Mail, HubChannel::Community]
            .into_iter()
            .map(|channel| {
                format!(
                    r#"<template data-governance-hub-template="{}">{}</template>"#,
                    channel.label(),
                    communication_hub_html(envelope, channel)
                )
            })
            .collect::<String>()
    } else {
        String::new()
    };
    format!(
        r#"<section id="governance-preview" aria-label="Anonymous Governance sample" data-governance-source="{}" data-governance-context="{}" data-governance-label="{}" data-governance-hub-source="{}" data-governance-visible="{}" data-governance-mode="ssr" data-governance-source-available="true" data-governance-available="{}">{}{templates}</section>"#,
        daily_escape(&governance_sample_source(envelope)),
        envelope.context.id(),
        daily_escape(&daily_sample_label(envelope)),
        daily_escape(&HubSample::envelope_source(envelope)),
        governance_sample_visible(envelope.context),
        governance_sample_visible(envelope.context),
        governance_body_html(envelope, false)
    )
}

fn governance_native_body_html(envelope: &TenantRenderEnvelope) -> String {
    let body = governance_body_html(envelope, true);
    #[cfg(not(target_arch = "wasm32"))]
    {
        if governance_sample_visible(envelope.context) {
            return body
                + &[HubChannel::Mail, HubChannel::Community]
                    .into_iter()
                    .map(|channel| {
                        format!(
                            r#"<template data-governance-hub-template="{}">{}</template>"#,
                            channel.label(),
                            communication_hub_html(envelope, channel)
                        )
                    })
                    .collect::<String>();
        }
    }
    body
}

pub(super) fn governance_rail_html(context: OperatorContext, logic: bool) -> String {
    if !governance_sample_visible(context) {
        return String::new();
    }
    if logic {
        format!(
            r##"<a class="rail-nav" data-governance-rail="true" href="#business-logics"><span aria-hidden="true">⌬</span>Sample Logics<em>{}</em></a>"##,
            BUSINESS_LOGIC_ROWS.len()
        )
    } else {
        r##"<a class="rail-nav" data-governance-rail="true" href="#governance-analytics"><span aria-hidden="true">↟</span>Governance sample</a>"##.to_string()
    }
}

pub(super) fn finance_filing_card(envelope: &TenantRenderEnvelope) -> impl IntoView + use<> {
    view! {
            <article id="filing-readiness" data-finance-owned="filing" data-finance-owner-source=finance_sample_source(envelope) data-finance-context=envelope.context.id() class="service-card filing-card">
                <p class="screen-anchor">"FILING READINESS"</p>
                <h4>"Withholding return preview"</h4>
                <p class="service-card-brief">"Sample readiness figures only. Workforce validation, HomeTax transport, and reviewer attestation have not been performed."</p>
                // A-6: added role=progressbar + aria-value* attributes
                <div class="readiness-bars compact" aria-label="Sample filing readiness">
                    <span role="progressbar" aria-valuenow="86" aria-valuemin="0" aria-valuemax="100" aria-label="Sample employee validation: 86%; not performed" style="--bar: 86%"><em aria-hidden="true">"Sample employee validation"</em></span>
                    <span role="progressbar" aria-valuenow="64" aria-valuemin="0" aria-valuemax="100" aria-label="Sample HomeTax transport: 64%; not performed" style="--bar: 64%"><em aria-hidden="true">"Sample HomeTax transport"</em></span>
                    <span role="progressbar" aria-valuenow="52" aria-valuemin="0" aria-valuemax="100" aria-label="Sample reviewer attestation: 52%; not performed" style="--bar: 52%"><em aria-hidden="true">"Sample reviewer attestation"</em></span>
                </div>
                <div class="service-card-actions">
                    <details><summary>"Inspect filing example"</summary><p>"Sample FILE-KR-2026-04, April 2026. Workforce validation, HomeTax transport, attestation and agency acceptance unavailable; nothing submitted."</p></details>
                    <a href="#finance-panel-billing">"Billing · tax"</a>
                    <a href="#work-hub" data-finance-channel="Community">"Council note"</a>
                    <a href="#evidence-spine">"Receipt"</a>
                </div>
                {filing_readiness_anchor_board()}
            </article>
    }
}
