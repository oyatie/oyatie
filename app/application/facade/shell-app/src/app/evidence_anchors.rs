use super::*;

pub(super) fn evidence_ledger_anchor_board(context: OperatorContext) -> impl IntoView {
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
                        {ae_action("trust", "seal-receipt", "Seal packet")}
                        {ae_action("trust", "route-workflow", "Workflow proof")}
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="ledger-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD ADMISSION"</p>
                    <h5>"Illustrative cloud admission; live hosting unknown"</h5>
                    <p>
                        "Example cloud, policy, release and FinOps relationships; no production-readiness proof is bound."
                    </p>
                    <div class="trust-anchor-actions">
                        {ae_action("trust", "route-cloud", "Cloud cells")}
                        {ae_action("trust", "route-gates", "Release gates")}
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="ledger-local">
                    <p class="screen-anchor">"LOCAL-ONLY RECEIPT VAULT"</p>
                    <h5>"Interactive trust without mutation"</h5>
                    <p>
                        "Inspect illustrative fields and open existing sample routes. Seal, brief, and all live effects remain unavailable."
                    </p>
                    <div class="trust-anchor-actions">
                        {ae_action("trust", "route-mail", "Mail brief")}
                        {ae_action("trust", "route-community", "Community note")}
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Illustrative receipt examples; none created, sealed or verified."
                </span>
                <div class="trust-anchor-routes" aria-label="Evidence ledger connected routes">
                    {ae_finops(context, "")}
                    {ae_action("trust", "route-inventory", "Inventory")}
                    {ae_action("trust", "route-graph", "Object graph")}
                    {ae_action("trust", "route-policy", "Policy")}
                </div>
            </div>
        </div>
    }
}

pub(super) fn object_graph_anchor_board(context: OperatorContext) -> impl IntoView {
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
                        {ae_action("trust", "trace-lineage", "Trace lineage")}
                        {ae_action("trust", "route-catalog", "Catalog objects")}
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="graph-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD GRAPH"</p>
                    <h5>"Substrate nodes join product nodes"</h5>
                    <p>
                        "Illustrative cells, resources, policies, gates, FinOps and receipts; production workload proof is unavailable."
                    </p>
                    <div class="trust-anchor-actions">
                        {ae_action_at("trust", "route-cloud", "Cloud topology", "graph")}
                        {ae_action_at("trust", "route-policy", "Policy edge", "graph")}
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="graph-local">
                    <p class="screen-anchor">"LOCAL-ONLY GRAPH OPS"</p>
                    <h5>"Selectable lineage, no side effects"</h5>
                    <p>
                        "Select example nodes and open existing communication samples; tracing, evidence staging and all live effects are unavailable."
                    </p>
                    <div class="trust-anchor-actions">
                        {ae_action("trust", "route-evidence", "Evidence spine")}
                        {ae_action_at("trust", "route-mail", "Reviewer Mail", "graph")}
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Illustrative object graph; live lineage is unknown."
                </span>
                <div class="trust-anchor-routes" aria-label="Object graph connected routes">
                    {ae_action_at("trust", "route-workflow", "Workflow", "graph")}
                    {ae_action_at("trust", "route-inventory", "Resources", "graph")}
                    {ae_action_at("trust", "route-community", "Community", "graph")}
                    {ae_finops(context, "graph")}
                </div>
            </div>
        </div>
    }
}
