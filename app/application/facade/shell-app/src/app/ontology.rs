use super::*;

pub(super) fn ontology_list(items: Vec<OntologyFact>, context: OperatorContext) -> impl IntoView {
    let fact_count = items.len();
    view! {
        <div class="ontology-command-console" data-ontology-console="true" data-ontology-context=context.id()>
            <div class="ontology-console-head">
                <div>
                    <p class="screen-anchor">"ONTOLOGY · FD-001 TENANT WORKLOAD MAP"</p>
                    <h4>"What exists, who can see it, and where it runs"</h4>
                    <span>"Typed entities connect FD-001 tenant workload delivery to Oyatie Cloud cells, policy envelopes, workflow outputs, and evidence receipts."</span>
                </div>
                <div class="ontology-console-actions">
                    <span class="status-chip success" data-ontology-status="true">{format!("{fact_count} facts · 7 workload nodes · local graph")}</span>
                    <button type="button" data-ontology-action="lineage">"Trace lineage"</button>
                    <button type="button" data-ontology-action="policy">"Policy view"</button>
                    <button type="button" data-ontology-action="evidence">"Evidence"</button>
                </div>
            </div>

            <div class="ontology-topology-grid" aria-label="FD-001 tenant workload ontology graph">
                <button type="button" class="ontology-node root selected" data-ontology-node="Tenant" data-node-route="workload" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Tenant" data-sidepeek-id="ONT-TENANT" data-sidepeek-desc="Tenant owns the permitted FD-001 module set and runtime envelope." data-sidepeek-owner="Tenant admin" data-sidepeek-risk="Visible" data-sidepeek-sla="Local graph only"><span>"TENANT"</span><strong>"Tenant Admin"</strong><em>"owns envelope"</em></button>
                <button type="button" class="ontology-node workload" data-ontology-node="FD-001 Workloads" data-node-route="workflow" data-sidepeek-trigger="ontology-node" data-sidepeek-title="FD-001 workload set" data-sidepeek-id="ONT-FD001" data-sidepeek-desc="Core FD-001 microservices are represented as tenant workloads for dogfood validation." data-sidepeek-owner="Product delivery" data-sidepeek-risk="P0" data-sidepeek-sla="Dogfood proving loop"><span>"FD-001"</span><strong>"Microservice workloads"</strong><em>"product goal"</em></button>
                <button type="button" class="ontology-node cloud" data-ontology-node="Oyatie Cloud" data-node-route="cloud" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Oyatie Cloud substrate" data-sidepeek-id="ONT-CLOUD" data-sidepeek-desc="Hyperscaler-grade substrate hosts dogfood tenant workloads and exposes cell posture." data-sidepeek-owner="Cloud substrate" data-sidepeek-risk="Substrate proof" data-sidepeek-sla="99.95 target staged"><span>"CLOUD"</span><strong>"Cell substrate"</strong><em>"hosts tenants"</em></button>
                <button type="button" class="ontology-node workflow" data-ontology-node="Workflow" data-node-route="workflow" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Workflow runtime" data-sidepeek-id="ONT-WORKFLOW" data-sidepeek-desc="Workflow coordinates payroll close, approval, comms outputs, and receipts." data-sidepeek-owner="Workflow Studio" data-sidepeek-risk="Governed" data-sidepeek-sla="4.0h gate"><span>"FLOW"</span><strong>"Workflow"</strong><em>"orchestrates"</em></button>
                <button type="button" class="ontology-node comms" data-ontology-node="Built-in Comms" data-node-route="mail" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Built-in communications" data-sidepeek-id="ONT-COMMS" data-sidepeek-desc="Messenger, Mail, and Community receive workflow outputs without external send." data-sidepeek-owner="Work Hub" data-sidepeek-risk="Local only" data-sidepeek-sla="No backend send"><span>"COMMS"</span><strong>"Messenger · Mail · Community"</strong><em>"outputs"</em></button>
                <button type="button" class="ontology-node evidence" data-ontology-node="Evidence" data-node-route="evidence" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Evidence spine" data-sidepeek-id="ONT-EVIDENCE" data-sidepeek-desc="Receipts bind workload state, approvals, messages, and deployment gates." data-sidepeek-owner="Audit spine" data-sidepeek-risk="Immutable staged" data-sidepeek-sla="Sealed draft"><span>"AUDIT"</span><strong>"Evidence spine"</strong><em>"proves"</em></button>
                <button type="button" class="ontology-node policy" data-ontology-node="Policy" data-node-route="identity" data-sidepeek-trigger="ontology-node" data-sidepeek-title="Policy envelope" data-sidepeek-id="ONT-POLICY" data-sidepeek-desc="Role, data-class, residency, and autonomy ceilings decide visibility and action eligibility." data-sidepeek-owner="Governance" data-sidepeek-risk="Guardrail" data-sidepeek-sla="Human review"><span>"POLICY"</span><strong>"Access envelope"</strong><em>"permits"</em></button>
            </div>

            <div class="ontology-fact-matrix" aria-label="Current permitted ontology facts">
                {items.into_iter().enumerate().map(|(index, item)| {
                    let key = format!("FACT-{:02}", index + 1);
                    view! {
                        <article class="ontology-fact-card" data-ontology-fact="true">
                            <span class="status-chip">{key}</span>
                            <strong>{item.entity}</strong>
                            <em>{item.relation}</em>
                            <p>{item.access_reason}</p>
                            <div>
                                <button type="button" data-ontology-action="inspect-fact">"Inspect"</button>
                                <button type="button" data-ontology-action="route-workflow">"Workflow"</button>
                            </div>
                        </article>
                    }
                }).collect_view()}
            </div>

            <div class="ontology-proof-rail" aria-label="Substrate proof contract">
                <article><p class="screen-anchor">"SUBSTRATE PROOF"</p><strong>"FD-001 runs as tenant workloads before service claim"</strong><span>"Cloud cells, policy envelopes, and evidence receipts prove the substrate can host real production tenants."</span></article>
                <article><p class="screen-anchor">"VISIBILITY"</p><strong>"Role + data-class gates"</strong><span>"Tenant admin can inspect posture; hidden modules remain server-derived, not client hidden."</span></article>
                <article><p class="screen-anchor">"GRAPH STATUS"</p><strong data-ontology-detail="true">"Tenant selected · workload lineage visible"</strong><span>"Click nodes or facts to stage local graph state."</span></article>
            </div>
        </div>
    }
}

pub(super) fn suggestion_list(items: Vec<IntelligenceSuggestion>) -> impl IntoView {
    let suggestion_count = items.len();
    view! {
        <div class="intelligence-command-console" data-intelligence-console="true">
            <div class="intelligence-console-head">
                <div>
                    <p class="screen-anchor">"GOVERNED AI · DOGFOOD ADVISOR"</p>
                    <h4>"Recommendations that can explain, route, and prove themselves"</h4>
                    <span>"AI suggestions stay read-only until a human routes them to Workflow, Mail, Community, or Evidence."</span>
                </div>
                <div class="intelligence-console-actions">
                    <span class="status-chip warning" data-intelligence-status="true">{format!("{suggestion_count} suggestions · human gated")}</span>
                    <button type="button" data-intelligence-action="evaluate">"Run eval"</button>
                    <button type="button" data-intelligence-action="explain">"Explain"</button>
                    <button type="button" data-intelligence-action="route-evidence">"Evidence"</button>
                </div>
            </div>

            <div class="intelligence-score-strip" aria-label="Governed AI evaluation summary">
                <span><strong>0.86</strong><small>decision confidence</small></span>
                <span><strong>14</strong><small>policy checks</small></span>
                <span><strong>0</strong><small>auto-executions</small></span>
                <span><strong>3</strong><small>FD-001 routes</small></span>
            </div>

            <div class="intelligence-layout">
                <div class="intelligence-suggestion-stack" role="list" aria-label="Governed AI suggestions">
                    {items.into_iter().enumerate().map(|(index, item)| {
                        let route = match index {
                            0 => "workflow",
                            1 => "mail",
                            _ => "community",
                        };
                        let receipt = match index {
                            0 => "AI-WF-217",
                            1 => "AI-TAX-118",
                            _ => "AI-HR-053",
                        };
                        view! {
                            <article class="intelligence-suggestion-card" data-intelligence-card="true" data-intelligence-route=route role="listitem">
                                <div><span class="status-chip ai">{receipt}</span><strong>{item.title}</strong></div>
                                <p>{item.body}</p>
                                <small>{item.guardrail}</small>
                                <div class="intelligence-card-actions">
                                    <button type="button" data-intelligence-action="preview" data-intelligence-route=route>"Preview"</button>
                                    <button type="button" data-intelligence-action="route" data-intelligence-route=route>"Route"</button>
                                    <button type="button" data-intelligence-action="dismiss">"Dismiss"</button>
                                </div>
                            </article>
                        }
                    }).collect_view()}
                </div>

                <aside class="intelligence-eval-panel" aria-label="AI guardrail evaluation harness">
                    <div><p class="screen-anchor">"EVAL HARNESS"</p><strong>"Before any tenant action"</strong><span>"Policy, data-class, autonomy ceiling, residency, and evidence checks must pass."</span></div>
                    <ol>
                        <li><span class="status-chip success">"pass"</span><strong>"No backend execution"</strong><em>"T1 advisory only"</em></li>
                        <li><span class="status-chip success">"pass"</span><strong>"Human approval required"</strong><em>"CFO / reviewer gate"</em></li>
                        <li><span class="status-chip warning">"review"</span><strong>"Tenant workload impact"</strong><em>"FD-001 cloud dogfood"</em></li>
                        <li><span class="status-chip success">"sealed"</span><strong>"Evidence receipt ready"</strong><em>"REC-AI-GUARD-009"</em></li>
                    </ol>
                    <div class="intelligence-route-grid" aria-label="Recommendation routes">
                        <button type="button" data-intelligence-action="route-workflow">"Workflow"</button>
                        <button type="button" data-intelligence-action="route-mail">"Mail"</button>
                        <button type="button" data-intelligence-action="route-community">"Community"</button>
                        <button type="button" data-intelligence-action="route-evidence">"Evidence"</button>
                    </div>
                </aside>
            </div>
        </div>
    }
}
