use super::*;

pub(super) fn evidence_ledger_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board" aria-label="FD-001 evidence ledger and Oyatie Cloud substrate proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected" data-trust-proof-card="ledger-fd001">
                    <p class="screen-anchor">"FD-001 RECEIPT SPINE"</p>
                    <h5>"Tenant workload delivery remains the master-plan goal"</h5>
                    <p>
                        "Messenger, Mail, Community, Workflow, Finance, and Daily Work receipts stay one FD-001 tenant workload packet rather than disconnected modules."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="seal-receipt">"Seal packet"</button>
                        <button type="button" data-trust-proof-action="route-workflow">"Workflow proof"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="ledger-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD ADMISSION"</p>
                    <h5>"Substrate proves real tenant hosting"</h5>
                    <p>
                        "Every cloud cell, policy grant, release gate, and FinOps signal attaches evidence before FD-001 services can claim production readiness."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-cloud">"Cloud cells"</button>
                        <button type="button" data-trust-proof-action="route-gates">"Release gates"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="ledger-local">
                    <p class="screen-anchor">"LOCAL-ONLY RECEIPT VAULT"</p>
                    <h5>"Interactive trust without mutation"</h5>
                    <p>
                        "Operators can inspect, seal, brief, and route receipts visually; no backend write, deploy, billing, mail, or cloud mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-mail">"Mail brief"</button>
                        <button type="button" data-trust-proof-action="route-community">"Community note"</button>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Evidence ledger ready · FD-001 tenant workload receipts dogfood Oyatie Cloud locally."
                </span>
                <div class="trust-anchor-routes" aria-label="Evidence ledger connected routes">
                    <button type="button" data-trust-proof-action="route-finops">"FinOps"</button>
                    <button type="button" data-trust-proof-action="route-inventory">"Inventory"</button>
                    <button type="button" data-trust-proof-action="route-graph">"Object graph"</button>
                    <button type="button" data-trust-proof-action="route-policy">"Policy"</button>
                </div>
            </div>
        </div>
    }
}

pub(super) fn object_graph_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board" aria-label="FD-001 object graph and Oyatie Cloud substrate proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected" data-trust-proof-card="graph-fd001">
                    <p class="screen-anchor">"FD-001 OBJECT MODEL"</p>
                    <h5>"One service graph spans every surface"</h5>
                    <p>
                        "Workflow, approvals, Messenger, Mail, Community, Finance, Daily Work, and audit nodes resolve to a single tenant operation lineage."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="trace-lineage">"Trace lineage"</button>
                        <button type="button" data-trust-proof-action="route-catalog">"Catalog objects"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="graph-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD GRAPH"</p>
                    <h5>"Substrate nodes join product nodes"</h5>
                    <p>
                        "Cells, resources, policies, deployment gates, FinOps, and receipts prove FD-001 microservices can run as production tenant workloads."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-cloud">"Cloud topology"</button>
                        <button type="button" data-trust-proof-action="route-policy">"Policy edge"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="graph-local">
                    <p class="screen-anchor">"LOCAL-ONLY GRAPH OPS"</p>
                    <h5>"Selectable lineage, no side effects"</h5>
                    <p>
                        "Operators can traverse graph edges, stage evidence, and open communications visually; no database, workflow, deploy, or cloud mutation occurs."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-evidence">"Evidence spine"</button>
                        <button type="button" data-trust-proof-action="route-mail">"Reviewer Mail"</button>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Object graph ready · FD-001 services and Oyatie Cloud substrate stay one local visual lineage."
                </span>
                <div class="trust-anchor-routes" aria-label="Object graph connected routes">
                    <button type="button" data-trust-proof-action="route-workflow">"Workflow"</button>
                    <button type="button" data-trust-proof-action="route-inventory">"Resources"</button>
                    <button type="button" data-trust-proof-action="route-community">"Community"</button>
                    <button type="button" data-trust-proof-action="route-finops">"FinOps"</button>
                </div>
            </div>
        </div>
    }
}
