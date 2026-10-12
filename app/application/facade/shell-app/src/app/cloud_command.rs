use super::*;

pub(super) fn cloud_ops_command_matrix(envelope: &TenantRenderEnvelope) -> impl IntoView + use<> {
    view! {
        <div class="ops-command-matrix" aria-label="Illustrative Cloud controls">
            <section class="ops-command-card ops-cell-card"><div class="ops-command-card-head"><div><p class="screen-anchor">"REGION EXAMPLES"</p><h5>"Three illustrative regions"</h5></div></div>
                <div class="ops-cell-grid" role="group" aria-label="Select a sample region">{CLOUD_REGIONS.into_iter().enumerate().map(|(index,(region,state,capacity,_))| view! {
                    <button type="button" data-cloud-select=format!("region:{region}") data-cloud-record=cloud_resources_record(&format!("region:{region}")) data-cloud-focus=format!("region:{region}") aria-pressed=(index==0).to_string() aria-expanded="false" aria-controls="cloud-topology-inspection" aria-label=format!("Inspect illustrative region {region}")>
                        <strong data-cloud-value="Region">{region}</strong><span data-cloud-value="State">{state}</span><em>"Sample capacity "<span data-cloud-value="Capacity">{capacity}</span></em>
                    </button>
                }).collect_view()}</div>
            </section>
            <section class="ops-command-card ops-workload-card ops-tenant-plane" data-cloud-workload-plane="true"><div class="ops-command-card-head"><div><p class="screen-anchor">"WORKLOAD EXAMPLES"</p><h5>"Four illustrative workload records"</h5></div></div>
                <div class="ops-plane-summary" aria-label="Derived example workload counts"><span><strong>{CLOUD_WORKLOADS.len()}</strong><small>"workload examples"</small></span><span><strong>{CLOUD_REGIONS.len()}</strong><small>"region examples"</small></span></div>
                <div class="ops-workload-list" aria-label="Select a sample workload">{CLOUD_WORKLOADS.into_iter().enumerate().map(|(index,(key,title,service,cell,state,_,_,_))| view! {
                    <button type="button" data-cloud-select=format!("workload:{key}") data-cloud-record=cloud_resources_record(&format!("workload:{key}")) data-cloud-focus=format!("workload:{key}") aria-pressed=(index==0).to_string() aria-expanded="false" aria-controls="cloud-topology-inspection" aria-label=format!("Inspect illustrative workload {title}")>
                        <span data-cloud-value="Title">{title}</span><strong data-cloud-value="Service">{service}</strong><em><span data-cloud-value="Cell">{cell}</span>" · "<span data-cloud-value="State">{state}</span>" example"</em>
                    </button>
                }).collect_view()}</div>
                <div class="ops-workload-detail" aria-label="Selected sample workload detail"><dl>{[("Service",CLOUD_WORKLOADS[0].2),("Cell",CLOUD_WORKLOADS[0].3),("Route",CLOUD_WORKLOADS[0].5),("Receipt",CLOUD_WORKLOADS[0].6)].into_iter().map(|(name,value)| view! {<div><dt>{name}</dt><dd data-cloud-workload-detail=name>{value}</dd></div>}).collect_view()}</dl>
                    <div class="ops-workload-routes" aria-label="Sample workload destinations">{cloud_route(envelope,"workload:workflow","Workflow")}{cloud_route(envelope,"workload:mail","Mail sample")}{cloud_route(envelope,"workload:community","Community sample")}{cloud_route(envelope,"workload:evidence","Evidence")}{cloud_route(envelope,"workload:gates","Gates")}</div>
                </div>
            </section>
            <section class="ops-command-card ops-release-card"><div class="ops-command-card-head"><div><p class="screen-anchor">"SAMPLE RELEASE GATES"</p><h5>"Four illustrative gate values"</h5></div>{cloud_route(envelope,"cloud:gates","Gates")}</div>
                <div class="ops-release-lanes" aria-label="Illustrative gate progress; no qualification measured">{deployment_gates().into_iter().map(|gate| view! {
                    <span role="progressbar" aria-valuenow=gate.progress.trim_end_matches('%') aria-valuemin="0" aria-valuemax="100" aria-label=format!("Sample {}: {}; not measured readiness",gate.label,gate.progress) style=format!("--bar: {}",gate.progress)><strong aria-hidden="true">{gate.label}</strong><em aria-hidden="true">{gate.progress}</em></span>
                }).collect_view()}</div>
            </section>
            <section class="ops-command-card ops-route-card"><p class="screen-anchor">"SAMPLE VIEWS"</p><div class="ops-route-grid">{cloud_route(envelope,"cloud:workflow","Workflow")}{cloud_route(envelope,"cloud:mail","Mail sample")}{cloud_route(envelope,"cloud:evidence","Evidence")}{cloud_route(envelope,"cloud:finops","FinOps")}</div><p>"Open existing sample views. Live Cloud operations and external delivery are unavailable."</p></section>
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
                        <button type="button" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Stage budget"</button>
                        <a href="#finance-panel-ledger">"Finance close"</a>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="finops-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD SUBSTRATE"</p>
                    <h5>"Costs prove real tenant hosting"</h5>
                    <p>
                        "Compute, network, storage, audit, residency, and release gates expose hyperscaler-grade posture before FD-001 workloads claim production readiness."
                    </p>
                    <div class="trust-anchor-actions">
                        <a href="#cloud-ops-cockpit">"Cloud topology"</a>
                        <a href="#cockpit-panel-policy">"Policy gate"</a>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="finops-local">
                    <p class="screen-anchor">"LOCAL-ONLY FINOPS"</p>
                    <h5>"Interactive budget controls, no spend mutation"</h5>
                    <p>
                        "Operators can stage commitments, tag anomalies, and brief reviewers visually; no billing, procurement, deploy, DNS, or cloud mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <a href="#resource-panel-audit">"Audit receipt"</a>
                        <a href="#evidence-spine">"Evidence spine"</a>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Sample FinOps; live spend unavailable · FD-001 microservices dogfood Oyatie Cloud as tenant workloads with local-only controls."
                </span>
                <div class="trust-anchor-routes" aria-label="FinOps connected routes">
                    <button type="button" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Resources"</button>
                    <button type="button" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Gates"</button>
                    <a href="#work-hub" data-finance-channel="Mail">"Reviewer Mail"</a>
                    <a href="#work-hub" data-finance-channel="Community">"Community"</a>
                </div>
            </div>
        </div>
    }
}

pub(super) fn resource_inventory_anchor_board(
    envelope: &TenantRenderEnvelope,
) -> impl IntoView + use<> {
    view! {
        <div class="trust-anchor-board" aria-label="Illustrative inventory destinations"><p>"Six fixed resource examples. State, cost, owner and descriptions are illustrative; live inventory and ownership verification are unavailable."</p>
            <div class="trust-anchor-actions">{cloud_unavailable("inventory:link","Link resource")}{cloud_route(envelope,"inventory:catalog","Service catalog")}{cloud_route(envelope,"inventory:finops","FinOps sample")}{cloud_route(envelope,"inventory:gates","Gate samples")}{cloud_route(envelope,"inventory:audit","Audit samples")}{cloud_route(envelope,"inventory:graph","Object graph")}{cloud_unavailable("inventory:lineage","Trace live lineage")}</div>
            <div class="trust-anchor-routes" aria-label="Existing sample destinations">{cloud_route(envelope,"inventory:workflow","Workflow")}{cloud_route(envelope,"inventory:evidence","Evidence")}{cloud_route(envelope,"inventory:mail","Mail sample")}{cloud_route(envelope,"inventory:community","Community sample")}</div>
        </div>
    }
}
