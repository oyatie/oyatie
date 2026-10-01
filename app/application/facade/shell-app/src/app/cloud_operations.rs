use super::*;

pub(super) fn tenant_operations_cockpit(envelope: TenantRenderEnvelope) -> impl IntoView {
    let healthcare_gate = if envelope.accreditation.healthcare_enabled {
        "Healthcare surfaces enabled"
    } else {
        "Healthcare surfaces gated"
    };
    let visible_modules = envelope.modules.len();

    view! {
        <section id="cloud-ops-cockpit" class="ops-cockpit panel" aria-labelledby="ops-cockpit-title">
            <div class="panel-header cockpit-header">
                <div>
                    <p class="eyebrow">"Operate"</p>
                    <h3 id="ops-cockpit-title">"Cloud, policy, and FinOps cockpit"</h3>
                </div>
                // A-2: cockpit tablist — aria-orientation + id/aria-controls + role=tabpanel
                <div class="cockpit-tabs" role="tablist" aria-label="Operations cockpit views" aria-orientation="horizontal">
                    <button type="button" id="cockpit-tab-topology" class="active" data-cockpit-tab="topology" role="tab" aria-selected="true" aria-controls="cockpit-panel-topology">"Topology"</button>
                    <button type="button" id="cockpit-tab-policy" data-cockpit-tab="policy" role="tab" aria-selected="false" aria-controls="cockpit-panel-policy">"Policy"</button>
                    <button type="button" id="cockpit-tab-finops" data-cockpit-tab="finops" role="tab" aria-selected="false" aria-controls="cockpit-panel-finops">"FinOps"</button>
                </div>
            </div>

            <div class="cockpit-panels">
                <article id="cockpit-panel-topology" class="cockpit-panel active" data-cockpit-panel="topology" role="tabpanel" aria-labelledby="cockpit-tab-topology">
                    <div class="cockpit-column-head">
                        <p class="screen-anchor">"CLOUD TOPOLOGY"</p>
                        <h4 id="cloud-topology-title">"Tenant runtime map"</h4>
                    </div>
                    <div class="topology-map" aria-hidden="true">
                        <span class="region primary">"us-east-2"<em>"cell active"</em></span>
                        <span class="region">"eu-west-1"<em>"warm standby"</em></span>
                        <span class="region">"kr-seoul"<em>"pack gated"</em></span>
                        <span class="service compute">"Compute"</span>
                        <span class="service network">"Network"</span>
                        <span class="service storage">"Storage"</span>
                        <span class="service audit">"Audit chain"</span>
                    </div>
                    {cloud_ops_command_matrix()}
                    <div class="ops-metrics-strip" aria-label="Cloud operations live posture">
                        <span><small>"Availability"</small><strong>"99.96%"</strong><em>"+0.01 vs SLO"</em></span>
                        <span><small>"Pending rollbacks"</small><strong>"2"</strong><em>"1 network · 1 key"</em></span>
                        <span><small>"Run-rate"</small><strong>"$48.2k"</strong><em>"4% under commit"</em></span>
                        <span><small>"Evidence age"</small><strong>"12m"</strong><em>"fresh"</em></span>
                    </div>
                    <div class="topology-detail-grid">
                        <article>
                            <p class="screen-anchor">"INCIDENT THREAD"</p>
                            <ol class="ops-timeline">
                                <li><time>"09:18"</time><strong>"Mesh split detected"</strong><span>"northwind-prod-mesh · rollback evidence requested"</span></li>
                                <li><time>"09:42"</time><strong>"DNS policy verified"</strong><span>"tenant-control-plane routes stay global"</span></li>
                                <li><time>"10:05"</time><strong>"Audit sidecar healthy"</strong><span>"receipt vault sealed draft attached"</span></li>
                            </ol>
                        </article>
                        <article>
                            <p class="screen-anchor">"RUNBOOK QUEUE"</p>
                            <div class="runbook-list">
                                <button type="button" data-cockpit-action="reconcile-cell">"Reconcile cell evidence"</button>
                                <button type="button" data-cockpit-action="simulate-failover">"Simulate failover"</button>
                                <button type="button" data-cockpit-action="queue-runbook">"Queue rollback runbook"</button>
                            </div>
                        </article>
                        <article>
                            <p class="screen-anchor">"REGIONAL CAPACITY"</p>
                            <div class="capacity-bars">
                                <span role="progressbar" aria-valuenow="72" aria-valuemin="0" aria-valuemax="100" aria-label="us-east-2 capacity: 72%" style="--bar: 72%"><em aria-hidden="true">"us-east-2"</em></span>
                                <span role="progressbar" aria-valuenow="44" aria-valuemin="0" aria-valuemax="100" aria-label="eu-west-1 capacity: 44%" style="--bar: 44%"><em aria-hidden="true">"eu-west-1"</em></span>
                                <span role="progressbar" aria-valuenow="28" aria-valuemin="0" aria-valuemax="100" aria-label="kr-seoul capacity: 28%" style="--bar: 28%"><em aria-hidden="true">"kr-seoul"</em></span>
                            </div>
                        </article>
                    </div>
                    <div class="cockpit-actions">
                        <button
                            type="button"
                            data-sidepeek-trigger="topology"
                            data-sidepeek-title="Tenant runtime map"
                            data-sidepeek-id="CELL-US-EAST-2"
                            data-sidepeek-desc="Primary cell running compute, network, storage, and audit-chain staged surfaces."
                            data-sidepeek-owner="Cloud infrastructure"
                            data-sidepeek-risk="Medium"
                            data-sidepeek-sla="99.95% target · local data"
                        >
                            "Inspect cell"
                        </button>
                        <button type="button" data-command-trigger="true">"Search resources"</button>
                        <span class="cockpit-status" data-cockpit-status="true">"Topology ready · local runbooks only."</span>
                    </div>
                </article>

                <article id="cockpit-panel-policy" class="cockpit-panel" data-cockpit-panel="policy" role="tabpanel" aria-labelledby="cockpit-tab-policy">
                    <div class="cockpit-column-head">
                        <p class="screen-anchor">"POLICY & ACCESS"</p>
                        <h4 id="policy-access-title">"Policy envelope command board"</h4>
                    </div>
                    <div class="policy-command-grid" aria-label="FD-001 and Oyatie Cloud policy proof">
                        <article class="policy-command-card selected" data-policy-card="fd001">
                            <div>
                                <p class="screen-anchor">"FD-001 TENANT"</p>
                                <h5>"Product delivery stays the goal"</h5>
                                <p>
                                    "Messenger, Mail, Community, Workflow, Ontology, and Intelligence run as tenant workloads; "
                                    "Oyatie Cloud proves they can be hosted without moving the tenant workload north star."
                                </p>
                            </div>
                            <div class="policy-command-actions" aria-label="FD-001 policy routes">
                                <button type="button" data-policy-anchor-action="role-review">"Review role grants"</button>
                                <button type="button" data-policy-anchor-action="open-identity">"Open identity"</button>
                                <button type="button" data-policy-anchor-action="route-evidence">"Evidence spine"</button>
                            </div>
                        </article>
                        <article class="policy-command-card" data-policy-card="substrate">
                            <div>
                                <p class="screen-anchor">"OYATIE CLOUD"</p>
                                <h5>"Dogfood substrate boundary"</h5>
                                <p>
                                    "Cloud controls stay tenant-scoped, PIPA-aware, auditable, and local-only until "
                                    "real FD-001 services are admitted through release gates."
                                </p>
                            </div>
                            <div class="policy-command-actions" aria-label="Oyatie Cloud policy routes">
                                <button type="button" data-policy-anchor-action="route-cloud">"Cloud topology"</button>
                                <button type="button" data-policy-anchor-action="pipa-boundary">"PIPA boundary"</button>
                                <button type="button" data-policy-anchor-action="open-audit">"Audit trail"</button>
                            </div>
                        </article>
                        <article class="policy-command-card" data-policy-card="autonomy">
                            <div>
                                <p class="screen-anchor">"AUTONOMY CEILING"</p>
                                <h5>"Interactive, never wired"</h5>
                                <p>
                                    "Policy can preview allow, gate, deny, rollback, and reviewer paths, but every action is "
                                    "visual state with no cloud, billing, DNS, or workflow mutation."
                                </p>
                            </div>
                            <div class="policy-command-actions" aria-label="Autonomy policy routes">
                                <button type="button" data-policy-anchor-action="autonomy-ceiling">"Show ceiling"</button>
                                <button type="button" data-policy-anchor-action="residency">"Residency pack"</button>
                                <button type="button" data-policy-anchor-action="route-mail">"Mail brief"</button>
                            </div>
                        </article>
                    </div>
                    <table class="policy-table">
                        <thead>
                            <tr><th>"Subject"</th><th>"Scope"</th><th>"Decision"</th><th>"Reason"</th></tr>
                        </thead>
                        <tbody>
                            <tr><td>"Tenant admin"</td><td>"Cloud controls"</td><td><span class="status-chip success">"Allow"</span></td><td>"Owner role"</td></tr>
                            <tr><td>{envelope.role_name.clone()}</td><td>"Healthcare"</td><td><span class="status-chip warning">{healthcare_gate}</span></td><td>"Accreditation"</td></tr>
                            <tr><td>"Workflow builder"</td><td>"Execution"</td><td><span class="status-chip danger">"Deny"</span></td><td>"Autonomy ceiling"</td></tr>
                        </tbody>
                    </table>
                    <div class="policy-evidence-grid">
                        <span><strong>"12"</strong><small>"Cedar rules mirrored"</small></span>
                        <span><strong>"7"</strong><small>"tenant pack grants"</small></span>
                        <span><strong>"3"</strong><small>"human review stops"</small></span>
                    </div>
                    <div class="policy-decision-strip" aria-label="Policy decision proof path">
                        <article class="policy-decision-card" data-policy-card="allow">
                            <span class="status-chip success">"Allow"</span>
                            <strong>"Tenant admin → Cloud controls"</strong>
                            <p>"Owner-scoped controls stay inside the dogfood substrate and attach receipt IDs before promotion."</p>
                            <button type="button" data-policy-anchor-action="route-cloud">"Inspect controls"</button>
                        </article>
                        <article class="policy-decision-card" data-policy-card="gate">
                            <span class="status-chip warning">"Gate"</span>
                            <strong>{format!("{} → regulated data", envelope.role_name.clone())}</strong>
                            <p>{format!("{healthcare_gate} · reviewer evidence and residency pack required before any FD-001 workload placement.")}</p>
                            <button type="button" data-policy-anchor-action="residency">"Review gate"</button>
                        </article>
                        <article class="policy-decision-card" data-policy-card="deny">
                            <span class="status-chip danger">"Deny"</span>
                            <strong>"Workflow builder → execution"</strong>
                            <p>"The autonomy ceiling blocks real execution; visual routing proves the UX without wiring side effects."</p>
                            <button type="button" data-policy-anchor-action="autonomy-ceiling">"Trace denial"</button>
                        </article>
                    </div>
                    <div class="policy-anchor-footer">
                        <span class="cockpit-status" data-policy-anchor-status="true">
                            "Policy board ready · FD-001 workloads dogfood Oyatie Cloud as tenant surfaces."
                        </span>
                        <div class="policy-anchor-routes" aria-label="Connected policy routes">
                            <button type="button" data-policy-anchor-action="route-community">"Community review"</button>
                            <button type="button" data-policy-anchor-action="open-audit">"Audit ledger"</button>
                            <button type="button" data-policy-anchor-action="route-evidence">"Evidence graph"</button>
                        </div>
                    </div>
                </article>

                <article id="cockpit-panel-finops" class="cockpit-panel" data-cockpit-panel="finops" role="tabpanel" aria-labelledby="cockpit-tab-finops">
                    <div class="cockpit-column-head">
                        <p class="screen-anchor">"FINOPS"</p>
                        <h4 id="finops-title">"Run-rate and sustainability"</h4>
                    </div>
                    <div class="finops-bars" aria-label="FinOps breakdown">
                        <span role="progressbar" aria-valuenow="72" aria-valuemin="0" aria-valuemax="100" aria-label="Compute · $21.4k: 72% of run-rate" style="--bar: 72%"><em aria-hidden="true">"Compute · $21.4k"</em></span>
                        <span role="progressbar" aria-valuenow="51" aria-valuemin="0" aria-valuemax="100" aria-label="Network · $9.8k: 51% of run-rate" style="--bar: 51%"><em aria-hidden="true">"Network · $9.8k"</em></span>
                        <span role="progressbar" aria-valuenow="43" aria-valuemin="0" aria-valuemax="100" aria-label="Storage · $7.2k: 43% of run-rate" style="--bar: 43%"><em aria-hidden="true">"Storage · $7.2k"</em></span>
                        <span role="progressbar" aria-valuenow="26" aria-valuemin="0" aria-valuemax="100" aria-label="Audit · $3.1k: 26% of run-rate" style="--bar: 26%"><em aria-hidden="true">"Audit · $3.1k"</em></span>
                    </div>
                    <div class="finops-action-grid">
                        <button type="button" data-cockpit-action="open-commit">"Open commit plan"</button>
                        <button type="button" data-cockpit-action="tag-anomaly">"Tag anomaly"</button>
                        <button type="button" data-cockpit-action="draft-budget-note">"Draft budget note"</button>
                    </div>
                    <span class="cockpit-status" data-cockpit-status="true">"FinOps ready · local budget actions only."</span>
                    {finops_anchor_board()}
                    <p class="cockpit-note">{format!("{visible_modules} services visible in this envelope · backend wiring remains disabled")}</p>
                </article>
            </div>
        </section>
    }
}
