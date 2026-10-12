use super::*;

pub(super) fn evidence_events() -> [EvidenceEvent; 6] {
    [
        (
            "blocking",
            "REC-PAY-2026-04-PARK",
            "Payroll delta needs four-insurance approval",
            "NHIS tier increase detected; owner must approve before April close package seals.",
            "Payroll",
            "Finance close",
            "4.0h",
        ),
        (
            "review",
            "REC-TAX-HOMETAX-118",
            "HomeTax withholding transport waiting",
            "118 sample employees; not validated; 사업자등록번호 confirmation remains before send preview.",
            "Tax",
            "CFO desk",
            "1d",
        ),
        (
            "sealed",
            "REC-WF-7741",
            "Workflow receipt staged from Command Center",
            "Tenant change approval produced Messenger, Mail, Community, and audit drafts.",
            "Workflow",
            "Tenant admin",
            "sealed",
        ),
        (
            "review",
            "REC-CLOUD-MESH-4182",
            "Network split rollback evidence requested",
            "us-east-2 mesh split requires regional capacity and rollback runbook attestation.",
            "Cloud Ops",
            "Infrastructure SRE",
            "2.1h",
        ),
        (
            "sealed",
            "REC-COMM-GOV-221",
            "Governance council broadcast prepared",
            "Community note links policy rationale, approval owner, and object graph lineage.",
            "Community",
            "Governance",
            "sealed",
        ),
        (
            "watch",
            "REC-VND-STRIPE-4820",
            "Vendor renewal route can be shortened",
            "Stripe 청구서 approval can move from three-stage to one-stage below policy threshold.",
            "Procurement",
            "AP owner",
            "next run",
        ),
    ]
}

pub(super) type ObjectNode = (&'static str, &'static str, &'static str, &'static str);

pub(super) fn object_nodes() -> [ObjectNode; 8] {
    [
        (
            "tenant",
            "Tenant",
            "Northwind Corp.",
            "Illustrative tenant envelope and pack gates; not authoritative",
        ),
        (
            "workflow",
            "Workflow",
            "Tenant change approval",
            "No-code approval path and run preview",
        ),
        (
            "approval",
            "Approval",
            "APR-274",
            "Human reviewer checkpoint before action",
        ),
        (
            "mail",
            "Mail",
            "Finance close brief",
            "Formal approval route draft",
        ),
        (
            "messenger",
            "Messenger",
            "Ops room",
            "Fast coordination thread",
        ),
        (
            "community",
            "Community",
            "Governance council",
            "Role-aware broadcast",
        ),
        (
            "cloud",
            "Cloud cell",
            "us-east-2",
            "Runtime, network, and FinOps posture",
        ),
        (
            "audit",
            "Receipt",
            "REC-WF-7741",
            "Illustrative local receipt; verification unavailable",
        ),
    ]
}

pub(super) fn operator_intelligence_strip(
    envelope: TenantRenderEnvelope,
    available: Signal<bool>,
) -> impl IntoView {
    let evidence_events = evidence_events();
    let object_nodes = object_nodes();
    let evidence_count = evidence_events.len();
    let object_count = object_nodes.len();
    let signal_count = receipt_sources().len();
    let workflow_name = envelope.workflow.name.clone();
    let workflow_goal = envelope.workflow.goal.clone();

    view! {
        <section
            id="evidence-spine"
            class="operator-intelligence evidence-intelligence-console panel"
            aria-label="Anonymous Evidence and object graph samples"
            data-ae-source=ae_source(&envelope)
            data-ae-context=envelope.context.id()
            data-ae-label=daily_sample_label(&envelope)
            data-ae-hub-source=HubSample::envelope_source(&envelope)
            data-ae-source-available=move || ae_available(available).to_string()
            data-ae-available=move || ae_available(available).to_string()
            data-ae-mode=if cfg!(target_arch = "wasm32") { "island" } else { "ssr" }
        >
            {ae_notice()}
            <div class="evidence-console-head">
                <div>
                    <p class="screen-anchor">"EVIDENCE SPINE · OBJECT GRAPH"</p>
                    <h3>"Sample evidence and object graph"</h3>
                    <p>
                        "Workflow, approvals, Messenger, Mail, Community, cloud operations, and audit receipts are shown as one cohesive local service graph."
                    </p>
                </div>
                <div class="evidence-head-actions" aria-label="Evidence console actions">
                    <span class="status-chip success">"illustrative examples; not sealed"</span>
                    {ae_action("evidence", "run-review", "Run review")}
                    {ae_action("evidence", "export", "Export packet")}
                </div>
            </div>

            <div class="evidence-kpi-strip" aria-label="Operational intelligence summary">
                <span><strong>{evidence_count}</strong><small>"sample events"</small></span>
                <span><strong>{object_count}</strong><small>"sample objects"</small></span>
                <span><strong>{signal_count}</strong><small>"sample sources"</small></span>
                <span><strong>"Unavailable"</strong><small>"live readiness"</small></span>
                <span><strong>"0"</strong><small>"backend writes"</small></span>
            </div>

            <div class="evidence-console-toolbar" aria-label="Evidence ledger filters">
                <label class="evidence-search">
                    <span aria-hidden="true">"⌕"</span>
                    <input data-evidence-search="true" data-ae-focus="evidence-search" type="search" aria-label="Search evidence ledger" placeholder="Search evidence, owner, object, route..." />
                </label>
                <div class="evidence-filter-pills" role="toolbar" aria-label="Evidence state filters">
                    <button type="button" class="active" data-evidence-filter="all" data-ae-focus="filter:all" aria-pressed="true">"All"</button>
                    <button type="button" data-evidence-filter="blocking" data-ae-focus="filter:blocking" aria-pressed="false">"Blocking"</button>
                    <button type="button" data-evidence-filter="review" data-ae-focus="filter:review" aria-pressed="false">"Review"</button>
                    <button type="button" data-evidence-filter="sealed" data-ae-focus="filter:sealed" aria-pressed="false">"Sealed example"</button>
                    <button type="button" data-evidence-filter="watch" data-ae-focus="filter:watch" aria-pressed="false">"Watch"</button>
                </div>
                <button type="button" data-ae-clear="true" data-ae-focus="evidence-clear">"Clear search and filter"</button>
                <span class="evidence-console-status" data-evidence-status="true" role="status">
                    {format!("{} of {} sample events · all states", evidence_events.len(), evidence_events.len())}
                </span>
            </div>

            <p data-ae-empty="true" role="status" hidden>"No sample events match this search and filter."</p>
            {ae_inspection()}
            <div class="evidence-layout">
                {evidence_ledger_panel(evidence_events, envelope.context)}

                <article id="object-graph" class="object-graph-panel" aria-labelledby="object-graph-title">
                    <div class="evidence-panel-head">
                        <div>
                            <p class="screen-anchor">"OBJECT GRAPH"</p>
                            <h4 id="object-graph-title">"Tenant operation lineage"</h4>
                        </div>
                        <span data-object-status="true">"8 sample objects · no live tenant verified"</span>
                    </div>
                    <div class="object-graph-canvas" aria-label="Selectable object graph preview">
                        {object_nodes.into_iter().map(|(key, label, value, desc)| {
                            view! {
                                <button
                                    type="button"
                                    class="object-node"
                                    data-object-node=key
                                    data-object-label=label
                                    data-ae-select=format!("node:{key}")
                                    data-ae-record=ae_record(&format!("node:{key}"))
                                    data-ae-focus=format!("node:{key}")
                                    aria-pressed="false"
                                    data-sidepeek-title=label
                                    data-sidepeek-id=format!("OBJ-{}", key.to_ascii_uppercase())
                                    data-sidepeek-desc=desc
                                    data-sidepeek-owner="Object graph"
                                    data-sidepeek-risk="Read-only"
                                    data-sidepeek-sla="Local data"
                                    aria-label=format!("Open object node {label}")
                                >
                                    <span data-ae-value="Label">{label}</span>
                                    <strong data-ae-value="Value">{value}</strong>
                                </button>
                            }
                        }).collect_view()}
                        <svg viewBox="0 0 640 280" aria-hidden="true" class="object-graph-links">
                            <path d="M92 60 C210 40 250 112 324 116 S484 88 552 64" />
                            <path d="M96 118 C190 168 266 158 326 116" />
                            <path d="M324 116 C388 146 452 168 550 166" />
                            <path d="M322 116 C302 198 374 230 548 226" />
                            <path d="M92 222 C190 232 252 204 324 116" />
                        </svg>
                    </div>
                    <div class="object-graph-table" aria-label="Object graph properties">
                        <dl>
                            <div><dt>"Graph root"</dt><dd data-ae-workflow="name">{workflow_name.clone()}</dd></div>
                            <div><dt>"Primary output"</dt><dd>"Task · message · evidence draft"</dd></div>
                            <div><dt>"Autonomy ceiling"</dt><dd>"No auto-execution"</dd></div>
                            <div><dt>"Region"</dt><dd>"us-east-2 active · kr-seoul pack gated"</dd></div>
                        </dl>
                    </div>
                    {object_graph_anchor_board(envelope.context)}
                </article>

                <aside class="copilot-rail-panel" aria-labelledby="intel-copilot-title">
                    <div class="evidence-panel-head compact">
                        <div>
                            <p class="screen-anchor">"COPILOT RAIL"</p>
                            <h4 id="intel-copilot-title">"Governed next moves"</h4>
                        </div>
                        <span class="status-chip ai">"Privacy assurance unavailable"</span>
                    </div>
                    <div class="intel-action-stack">
                        {ae_action("intel", "audit", "Audit ledger")}
                        {ae_action("intel", "workflow", "Workflow sample")}
                        {ae_action("intel", "mail", "Open sample Mail")}
                        {ae_action("intel", "community", "Open sample Community")}
                    </div>
                    <p class="copilot-rail-status" data-copilot-rail-status="true">
                        "Read-only recommendations; every action changes local visual state only."
                    </p>
                </aside>

                <article class="signal-lineage-panel" aria-labelledby="signal-lineage-title">
                    <div class="evidence-panel-head">
                        <div>
                            <p class="screen-anchor">"CROSS-MODULE SIGNAL"</p>
                            <h4 id="signal-lineage-title" data-ae-workflow="name">{workflow_name}</h4>
                        </div>
                        <span class="status-chip warning">"review path"</span>
                    </div>
                    <p data-ae-workflow="goal">{workflow_goal}</p>
                    <ol class="signal-lineage">
                        <li class="root"><span>"Workflow"</span><strong>"Tenant change approval"</strong><em>"root event"</em></li>
                        <li><span>"Messenger"</span><strong>"Ops room update"</strong><em>"drafted"</em></li>
                        <li><span>"Mail"</span><strong>"Finance approval brief"</strong><em>"ready"</em></li>
                        <li><span>"Community"</span><strong>"Governance council note"</strong><em>"review"</em></li>
                        <li><span>"Cloud Ops"</span><strong>"Rollback evidence"</strong><em>"blocking"</em></li>
                        <li><span>"Audit"</span><strong>"Receipt spine"</strong><em>"illustrative examples; not sealed"</em></li>
                    </ol>
                    <div class="lineage-actions">
                        {ae_action("intel", "messenger", "Open sample Messenger")}
                        {ae_action_at("intel", "mail", "Open sample Mail", "lineage")}
                        {ae_action_at("intel", "community", "Open sample Community", "lineage")}
                        {ae_action_at("intel", "audit", "Audit ledger", "lineage")}
                    </div>
                </article>
            </div>
            {ae_hub_templates(&envelope)}
        </section>
    }
}

pub(super) type EvidenceEvent = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);
