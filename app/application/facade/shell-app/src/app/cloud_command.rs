use super::*;

pub(super) fn cloud_ops_command_matrix() -> impl IntoView {
    view! {
        <div class="ops-command-matrix" aria-label="Cloud operations command matrix">
            <section class="ops-command-card ops-cell-card">
                <div class="ops-command-card-head">
                    <div>
                        <p class="screen-anchor">"CELL CONTROL"</p>
                        <h5>"Runtime cells, residency, and rollback posture"</h5>
                    </div>
                    <span class="status-chip warning">"2 guardrails"</span>
                </div>
                <div class="ops-cell-grid" role="list" aria-label="Regional cell state">
                    <button type="button" class="active" data-cockpit-action="select-us-east">
                        <strong>"us-east-2"</strong><span>"primary"</span><em>"99.96% · 72% cap"</em>
                    </button>
                    <button type="button" data-cockpit-action="select-eu-west">
                        <strong>"eu-west-1"</strong><span>"standby"</span><em>"warm · 44% cap"</em>
                    </button>
                    <button type="button" data-cockpit-action="select-kr-seoul">
                        <strong>"kr-seoul"</strong><span>"pack gated"</span><em>"residency review"</em>
                    </button>
                </div>
                <dl class="ops-command-kv">
                    <div><dt>"Residency"</dt><dd>"KR pack gated before workload placement"</dd></div>
                    <div><dt>"Rollback"</dt><dd>"Network split runbook needs reviewer evidence"</dd></div>
                    <div><dt>"Audit sidecar"</dt><dd>"Receipt vault sealed draft attached"</dd></div>
                </dl>
            </section>

            <section class="ops-command-card ops-workload-card ops-tenant-plane" data-cloud-workload-plane="true">
                <div class="ops-command-card-head">
                    <div>
                        <p class="screen-anchor">"FD-001 TENANT WORKLOAD PLANE"</p>
                        <h5>"Product microservices hosted on the dogfood substrate"</h5>
                    </div>
                    <button type="button" data-cockpit-action="open-resource-inventory">"Inventory"</button>
                </div>
                <div class="ops-plane-summary" aria-label="FD-001 tenant workload summary">
                    <span><strong>"9"</strong><small>"FD-001 services"</small></span>
                    <span><strong>"3"</strong><small>"cells"</small></span>
                    <span><strong>"0"</strong><small>"live mutations"</small></span>
                </div>
                <div class="ops-workload-list" aria-label="FD-001 microservices running as tenant workloads">
                    <button
                        type="button"
                        class="selected"
                        data-cockpit-workload="workflow"
                        data-workload-title="Workflow runner"
                        data-workload-service="workflow-runner"
                        data-workload-cell="us-east-2"
                        data-workload-state="review"
                        data-workload-route="Workflow → Messenger/Mail/Community → Evidence"
                        data-workload-receipt="REC-FD001-WF-018"
                    >
                        <span>"Workflow"</span>
                        <strong>"workflow-runner"</strong>
                        <em>"us-east-2 · review"</em>
                        <small>"Runs approvals as visual-only tenant workload previews."</small>
                    </button>
                    <button
                        type="button"
                        data-cockpit-workload="comms"
                        data-workload-title="Built-in communications"
                        data-workload-service="messenger-mail-community"
                        data-workload-cell="us-east-2 + kr-seoul"
                        data-workload-state="drafts"
                        data-workload-route="Messenger/Mail/Community handoff bus"
                        data-workload-receipt="REC-COMMS-HANDOFF-006"
                    >
                        <span>"Comms"</span>
                        <strong>"messenger-mail-community"</strong>
                        <em>"multi-surface · drafts"</em>
                        <small>"Local drafts prove FD-001 coordination without delivery."</small>
                    </button>
                    <button
                        type="button"
                        data-cockpit-workload="evidence"
                        data-workload-title="Evidence spine"
                        data-workload-service="audit-vault"
                        data-workload-cell="multi-cell"
                        data-workload-state="sealed"
                        data-workload-route="Audit ledger + object graph"
                        data-workload-receipt="REC-FD001-CLOUD-009"
                    >
                        <span>"Evidence"</span>
                        <strong>"audit-vault"</strong>
                        <em>"multi-cell · sealed"</em>
                        <small>"Receipts bind cloud posture, workflow output, and reviewers."</small>
                    </button>
                    <button
                        type="button"
                        data-cockpit-workload="identity"
                        data-workload-title="Identity envelope"
                        data-workload-service="identity-access"
                        data-workload-cell="kr-seoul gated"
                        data-workload-state="policy"
                        data-workload-route="Identity → Policy → Deployment gates"
                        data-workload-receipt="REC-ID-2026-05"
                    >
                        <span>"Identity"</span>
                        <strong>"identity-access"</strong>
                        <em>"kr pack · policy"</em>
                        <small>"Role and residency controls prove tenant placement."</small>
                    </button>
                </div>
                <div class="ops-workload-detail" aria-label="Selected tenant workload detail">
                    <span class="status-chip warning" data-cockpit-workload-status="true">
                        "Workflow runner selected · review gate open · local-only substrate proof"
                    </span>
                    <dl>
                        <div><dt>"Service"</dt><dd data-workload-detail-service="true">"workflow-runner"</dd></div>
                        <div><dt>"Cell"</dt><dd data-workload-detail-cell="true">"us-east-2"</dd></div>
                        <div><dt>"Route"</dt><dd data-workload-detail-route="true">"Workflow → Messenger/Mail/Community → Evidence"</dd></div>
                        <div><dt>"Receipt"</dt><dd data-workload-detail-receipt="true">"REC-FD001-WF-018"</dd></div>
                    </dl>
                    <div class="ops-workload-routes" aria-label="Selected workload routes">
                        <button type="button" data-cockpit-workload-route="workflow">"Workflow"</button>
                        <button type="button" data-cockpit-workload-route="mail">"Mail brief"</button>
                        <button type="button" data-cockpit-workload-route="community">"Community"</button>
                        <button type="button" data-cockpit-workload-route="evidence">"Evidence"</button>
                        <button type="button" data-cockpit-workload-route="gates">"Gates"</button>
                    </div>
                </div>
            </section>

            <section class="ops-command-card ops-release-card">
                <div class="ops-command-card-head">
                    <div>
                        <p class="screen-anchor">"RELEASE GATES"</p>
                        <h5>"Jenkins, ArgoCD, cosign, and audit evidence"</h5>
                    </div>
                    <button type="button" data-cockpit-action="open-deployment-gates">"Gates"</button>
                </div>
                <div class="ops-release-lanes" aria-label="Cloud release readiness">
                    <span role="progressbar" aria-valuenow="92" aria-valuemin="0" aria-valuemax="100" aria-label="Jenkins parity: 92%" style="--bar: 92%"><strong aria-hidden="true">"Jenkins parity"</strong><em aria-hidden="true">"92%"</em></span>
                    <span role="progressbar" aria-valuenow="74" aria-valuemin="0" aria-valuemax="100" aria-label="ArgoCD app: 74%" style="--bar: 74%"><strong aria-hidden="true">"ArgoCD app"</strong><em aria-hidden="true">"74%"</em></span>
                    <span role="progressbar" aria-valuenow="88" aria-valuemin="0" aria-valuemax="100" aria-label="Cosign verify: 88%" style="--bar: 88%"><strong aria-hidden="true">"Cosign verify"</strong><em aria-hidden="true">"88%"</em></span>
                    <span role="progressbar" aria-valuenow="69" aria-valuemin="0" aria-valuemax="100" aria-label="Audit emit: 69%" style="--bar: 69%"><strong aria-hidden="true">"Audit emit"</strong><em aria-hidden="true">"69%"</em></span>
                </div>
            </section>

            <section class="ops-command-card ops-route-card">
                <div class="ops-command-card-head">
                    <div>
                        <p class="screen-anchor">"ROUTES"</p>
                        <h5>"Open the connected product surface without leaving context"</h5>
                    </div>
                </div>
                <div class="ops-route-grid" aria-label="Cloud operations local routes">
                    <button type="button" data-cockpit-action="open-workflow">"Workflow"</button>
                    <button type="button" data-cockpit-action="open-mail">"Mail brief"</button>
                    <button type="button" data-cockpit-action="open-evidence">"Evidence"</button>
                    <button type="button" data-cockpit-action="open-finops">"FinOps"</button>
                </div>
                <p>"All actions are visual-only local state; no cloud, DNS, deploy, or billing operation is executed."</p>
            </section>
        </div>
    }
}

pub(super) fn finops_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board finops-trust-board" aria-label="FD-001 FinOps and Oyatie Cloud substrate proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected" data-trust-proof-card="finops-fd001">
                    <p class="screen-anchor">"FD-001 WORKLOAD ECONOMY"</p>
                    <h5>"Product delivery remains the north star"</h5>
                    <p>
                        "Run-rate is shown per FD-001 tenant workload: Workflow, Messenger, Mail, Community, Intelligence, and audit services stay in one delivery envelope."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="stage-budget">"Stage budget"</button>
                        <button type="button" data-trust-proof-action="route-finance">"Finance close"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="finops-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD SUBSTRATE"</p>
                    <h5>"Costs prove real tenant hosting"</h5>
                    <p>
                        "Compute, network, storage, audit, residency, and release gates expose hyperscaler-grade posture before FD-001 workloads claim production readiness."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-cloud">"Cloud topology"</button>
                        <button type="button" data-trust-proof-action="route-policy">"Policy gate"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="finops-local">
                    <p class="screen-anchor">"LOCAL-ONLY FINOPS"</p>
                    <h5>"Interactive budget controls, no spend mutation"</h5>
                    <p>
                        "Operators can stage commitments, tag anomalies, and brief reviewers visually; no billing, procurement, deploy, DNS, or cloud mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-audit">"Audit receipt"</button>
                        <button type="button" data-trust-proof-action="route-evidence">"Evidence spine"</button>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "FinOps ready · FD-001 microservices dogfood Oyatie Cloud as tenant workloads with local-only controls."
                </span>
                <div class="trust-anchor-routes" aria-label="FinOps connected routes">
                    <button type="button" data-trust-proof-action="route-inventory">"Resources"</button>
                    <button type="button" data-trust-proof-action="route-gates">"Gates"</button>
                    <button type="button" data-trust-proof-action="route-mail">"Reviewer Mail"</button>
                    <button type="button" data-trust-proof-action="route-community">"Community"</button>
                </div>
            </div>
        </div>
    }
}

pub(super) fn resource_inventory_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board" aria-label="FD-001 resource inventory and Oyatie Cloud substrate proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected" data-trust-proof-card="resource-fd001">
                    <p class="screen-anchor">"FD-001 SERVICE FLEET"</p>
                    <h5>"Microservices are tenant workloads"</h5>
                    <p>
                        "Tenant admin, workflow runner, audit vault, Mail, Messenger, Community, and Intelligence assets are tracked as one FD-001 product fleet."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="link-resource">"Link resource"</button>
                        <button type="button" data-trust-proof-action="route-catalog">"Service catalog"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="resource-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD INVENTORY"</p>
                    <h5>"Substrate owns residency and release posture"</h5>
                    <p>
                        "Each resource shows cell, owner, cost, risk, policy, deployment gate, and audit receipt so the cloud substrate can prove real tenant hosting."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-finops">"FinOps cost"</button>
                        <button type="button" data-trust-proof-action="route-gates">"Admission gates"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="resource-local">
                    <p class="screen-anchor">"LOCAL-ONLY RESOURCE OPS"</p>
                    <h5>"Inspect without provider mutation"</h5>
                    <p>
                        "Operators can inspect ownership, route evidence, and preview remediation; no cloud provider, database, deploy, billing, or audit mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-audit">"Audit ledger"</button>
                        <button type="button" data-trust-proof-action="trace-lineage">"Trace lineage"</button>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Resource inventory ready · FD-001 workload fleet is hosted-proof on Oyatie Cloud with local visual controls only."
                </span>
                <div class="trust-anchor-routes" aria-label="Resource inventory connected routes">
                    <button type="button" data-trust-proof-action="route-workflow">"Workflow"</button>
                    <button type="button" data-trust-proof-action="route-evidence">"Evidence"</button>
                    <button type="button" data-trust-proof-action="route-mail">"Mail"</button>
                    <button type="button" data-trust-proof-action="route-community">"Community"</button>
                </div>
            </div>
        </div>
    }
}
