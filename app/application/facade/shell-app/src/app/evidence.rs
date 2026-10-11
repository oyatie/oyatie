use super::*;

pub(super) fn operator_intelligence_strip(envelope: TenantRenderEnvelope) -> impl IntoView {
    let evidence_count = envelope.approvals.len() + envelope.workflow.nodes.len() + 6;
    let readiness = if envelope.accreditation.healthcare_enabled {
        "Accredited"
    } else {
        "Gated"
    };
    let object_count =
        envelope.modules.len() + envelope.daily_tasks.len() + envelope.community.len() + 7;
    let signal_count =
        envelope.messages.len() + envelope.community.len() + envelope.approvals.len();
    let workflow_name = envelope.workflow.name.clone();
    let workflow_goal = envelope.workflow.goal.clone();

    let evidence_events = [
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
            "118 employees validated; 사업자등록번호 confirmation remains before send preview.",
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
    ];

    let object_nodes = [
        (
            "tenant",
            "Tenant",
            "Northwind Corp.",
            "Authoritative tenant envelope and pack gates",
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
            "Immutable local evidence preview",
        ),
    ];

    view! {
        <section
            id="evidence-spine"
            class="operator-intelligence evidence-intelligence-console panel"
            aria-label="Evidence spine, object graph, and governed copilot intelligence"
        >
            <div class="evidence-console-head">
                <div>
                    <p class="screen-anchor">"EVIDENCE SPINE · OBJECT GRAPH"</p>
                    <h3>"Operational intelligence console"</h3>
                    <p>
                        "Workflow, approvals, Messenger, Mail, Community, cloud operations, and audit receipts are shown as one cohesive local service graph."
                    </p>
                </div>
                <div class="evidence-head-actions" aria-label="Evidence console actions">
                    <span class="status-chip success">"sealed draft"</span>
                    <button type="button" data-evidence-action="run-review">"Run review"</button>
                    <button type="button" data-evidence-action="export">"Export packet"</button>
                </div>
            </div>

            <div class="evidence-kpi-strip" aria-label="Operational intelligence summary">
                <span><strong>{evidence_count}</strong><small>"evidence leaves"</small></span>
                <span><strong>{object_count}</strong><small>"graph objects"</small></span>
                <span><strong>{signal_count}</strong><small>"cross-service signals"</small></span>
                <span><strong>{readiness}</strong><small>"tenant readiness"</small></span>
                <span><strong>"0"</strong><small>"backend writes"</small></span>
            </div>

            <div class="evidence-console-toolbar" aria-label="Evidence ledger filters">
                <label class="evidence-search">
                    <span aria-hidden="true">"⌕"</span>
                    <input data-evidence-search="true" type="search" aria-label="Search evidence ledger" placeholder="Search evidence, owner, object, route..." />
                </label>
                <div class="evidence-filter-pills" role="toolbar" aria-label="Evidence state filters">
                    <button type="button" class="active" data-evidence-filter="all">"All"</button>
                    <button type="button" data-evidence-filter="blocking">"Blocking"</button>
                    <button type="button" data-evidence-filter="review">"Review"</button>
                    <button type="button" data-evidence-filter="sealed">"Sealed"</button>
                    <button type="button" data-evidence-filter="watch">"Watch"</button>
                </div>
                <span class="evidence-console-status" data-evidence-status="true">
                    {format!("{} visible · all states · local evidence only", evidence_events.len())}
                </span>
            </div>

            <div class="evidence-layout">
                {evidence_ledger_panel(evidence_events)}

                <article id="object-graph" class="object-graph-panel" aria-labelledby="object-graph-title">
                    <div class="evidence-panel-head">
                        <div>
                            <p class="screen-anchor">"OBJECT GRAPH"</p>
                            <h4 id="object-graph-title">"Tenant operation lineage"</h4>
                        </div>
                        <span data-object-status="true">"Tenant selected · 8 linked objects"</span>
                    </div>
                    <div class="object-graph-canvas" aria-label="Selectable object graph preview">
                        {object_nodes.into_iter().enumerate().map(|(index, (key, label, value, desc))| {
                            let node_class = if index == 0 { "object-node active" } else { "object-node" };
                            view! {
                                <button
                                    type="button"
                                    class=node_class
                                    data-object-node=key
                                    data-object-label=label
                                    data-sidepeek-trigger="object-graph"
                                    data-sidepeek-title=label
                                    data-sidepeek-id=format!("OBJ-{}", key.to_ascii_uppercase())
                                    data-sidepeek-desc=desc
                                    data-sidepeek-owner="Object graph"
                                    data-sidepeek-risk="Read-only"
                                    data-sidepeek-sla="Local data"
                                    aria-label=format!("Open object node {label}")
                                >
                                    <span>{label}</span>
                                    <strong>{value}</strong>
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
                            <div><dt>"Graph root"</dt><dd>{workflow_name.clone()}</dd></div>
                            <div><dt>"Primary output"</dt><dd>"Task · message · evidence draft"</dd></div>
                            <div><dt>"Autonomy ceiling"</dt><dd>"No auto-execution"</dd></div>
                            <div><dt>"Region"</dt><dd>"us-east-2 active · kr-seoul pack gated"</dd></div>
                        </dl>
                    </div>
                    {object_graph_anchor_board()}
                </article>

                <aside class="copilot-rail-panel" aria-labelledby="intel-copilot-title">
                    <div class="evidence-panel-head compact">
                        <div>
                            <p class="screen-anchor">"COPILOT RAIL"</p>
                            <h4 id="intel-copilot-title">"Governed next moves"</h4>
                        </div>
                        <span class="status-chip ai">"PIPA-safe"</span>
                    </div>
                    <div class="intel-action-stack">
                        <button type="button" data-intel-action="audit">
                            <strong>"Draft audit brief"</strong>
                            <span>"Bundle payroll, HomeTax, workflow, and cloud evidence into one reviewer packet."</span>
                        </button>
                        <button type="button" data-intel-action="workflow">
                            <strong>"Simulate critical path"</strong>
                            <span>"Preview CFO escalation and Mail/Messenger/Community outputs before any execution."</span>
                        </button>
                        <button type="button" data-intel-action="mail">
                            <strong>"Compose approval mail"</strong>
                            <span>"Open the built-in Mail surface with receipt links and owner context."</span>
                        </button>
                        <button type="button" data-intel-action="community">
                            <strong>"Post council note"</strong>
                            <span>"Route a governance update to Community without leaving the console."</span>
                        </button>
                    </div>
                    <p class="copilot-rail-status" data-copilot-rail-status="true">
                        "Read-only recommendations; every action changes local visual state only."
                    </p>
                </aside>

                <article class="signal-lineage-panel" aria-labelledby="signal-lineage-title">
                    <div class="evidence-panel-head">
                        <div>
                            <p class="screen-anchor">"CROSS-MODULE SIGNAL"</p>
                            <h4 id="signal-lineage-title">{workflow_name}</h4>
                        </div>
                        <span class="status-chip warning">"review path"</span>
                    </div>
                    <p>{workflow_goal}</p>
                    <ol class="signal-lineage">
                        <li class="root"><span>"Workflow"</span><strong>"Tenant change approval"</strong><em>"root event"</em></li>
                        <li><span>"Messenger"</span><strong>"Ops room update"</strong><em>"drafted"</em></li>
                        <li><span>"Mail"</span><strong>"Finance approval brief"</strong><em>"ready"</em></li>
                        <li><span>"Community"</span><strong>"Governance council note"</strong><em>"review"</em></li>
                        <li><span>"Cloud Ops"</span><strong>"Rollback evidence"</strong><em>"blocking"</em></li>
                        <li><span>"Audit"</span><strong>"Receipt spine"</strong><em>"sealed draft"</em></li>
                    </ol>
                    <div class="lineage-actions">
                        <button type="button" data-intel-action="messenger">"Messenger"</button>
                        <button type="button" data-intel-action="mail">"Mail"</button>
                        <button type="button" data-intel-action="community">"Community"</button>
                        <button type="button" data-intel-action="audit">"Audit ledger"</button>
                    </div>
                </article>
            </div>
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
