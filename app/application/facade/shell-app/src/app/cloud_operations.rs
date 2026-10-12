use super::*;

pub(super) fn tenant_operations_cockpit(envelope: TenantRenderEnvelope) -> impl IntoView {
    let finance_scope = finance_sample_scope(envelope.context);
    view! {
        <section id="cloud-ops-cockpit" class="ops-cockpit panel" aria-labelledby="ops-cockpit-title">
            <div class="panel-header cockpit-header"><div><p class="eyebrow">"Operate"</p><h3 id="ops-cockpit-title">"Cloud, policy, and FinOps cockpit"</h3></div>
                <div class="cockpit-tabs" role="tablist" aria-label="Operations cockpit views" aria-orientation="horizontal">
                    <button type="button" id="cockpit-tab-topology" class="active" data-cockpit-tab="topology" role="tab" aria-selected="true" aria-controls="cockpit-panel-topology">"Topology"</button>
                    <button type="button" id="cockpit-tab-policy" data-cockpit-tab="policy" role="tab" aria-selected="false" aria-controls="cockpit-panel-policy">"Policy"</button>
                    {finance_scope.finops_examples.then(|| view! { <button type="button" id="cockpit-tab-finops" data-cockpit-tab="finops" role="tab" aria-selected="false" aria-controls="cockpit-panel-finops">"FinOps"</button> })}
                </div>
            </div>
            <div class="cockpit-panels">{cloud_topology_panel(&envelope)}{policy_preview_panel(envelope.clone())}
                {finance_scope.finops_examples.then(|| finance_finops_panel(&envelope))}
            </div>
        </section>
    }
}

pub(super) fn cloud_topology_panel(envelope: &TenantRenderEnvelope) -> impl IntoView + use<> {
    view! {
        <article id="cockpit-panel-topology" class="cockpit-panel active" data-cockpit-panel="topology" role="tabpanel" aria-labelledby="cockpit-tab-topology"
            data-cloud-root="topology" data-cloud-source=cloud_resources_source(envelope) data-cloud-context=envelope.context.id()
            data-cloud-label=daily_sample_label(envelope) data-cloud-hub-source=HubSample::envelope_source(envelope) data-cloud-mode=if cfg!(target_arch="wasm32") {"island"} else {"ssr"}>
            <div class="cockpit-column-head"><p class="screen-anchor">"SAMPLE CLOUD TOPOLOGY"</p><h4 id="cloud-topology-title">"Illustrative regions and workloads"</h4></div>
            <p data-cloud-availability="topology">"Fixed examples only. Live hosting, uptime, capacity and release readiness are unavailable."</p>
            <div class="topology-map" aria-label="Illustrative region map">{CLOUD_REGIONS.into_iter().map(|(region,state,_,_)| view! {
                <span class="region">{region}<em>{format!("{state} example")}</em></span>
            }).collect_view()}<span class="service compute">"Compute kind"</span><span class="service network">"Network kind"</span><span class="service storage">"Storage kind"</span><span class="service audit">"Audit kind"</span></div>
            {cloud_ops_command_matrix(envelope)}
            <div class="ops-metrics-strip" aria-label="Derived sample counts">
                <span><small>"Region examples"</small><strong>{CLOUD_REGIONS.len()}</strong></span><span><small>"Workload examples"</small><strong>{CLOUD_WORKLOADS.len()}</strong></span>
                <span><small>"Resource examples"</small><strong>{resource_inventory_rows().len()}</strong></span><span><small>"Gate examples"</small><strong>{deployment_gates().len()}</strong></span>
            </div>
            <div class="topology-detail-grid"><article><p class="screen-anchor">"ILLUSTRATIVE INCIDENT THREAD"</p><ol class="ops-timeline">
                <li><time>"09:18 example"</time><strong>"Mesh split example"</strong><span>"northwind-prod-mesh · no live incident observed"</span></li>
                <li><time>"09:42 example"</time><strong>"DNS policy example"</strong><span>"tenant-control-plane · no policy verified"</span></li>
                <li><time>"10:05 example"</time><strong>"Audit sidecar example"</strong><span>"receipt vault · no health or seal verified"</span></li>
            </ol></article><article><p class="screen-anchor">"UNAVAILABLE OPERATIONS"</p><p>"These examples cannot reconcile cells, simulate failover or queue runbooks."</p><div class="runbook-list">
                {cloud_unavailable("topology:reconcile", "Reconcile cell evidence")}{cloud_unavailable("topology:failover", "Simulate failover")}{cloud_unavailable("topology:runbook", "Queue rollback runbook")}
            </div></article></div>
            <div class="cockpit-actions"><button type="button" data-cloud-inspect-region="true" aria-controls="cloud-topology-inspection" aria-expanded="false">"Inspect selected region sample"</button>
                {cloud_route(envelope,"cloud:inventory","Search inventory samples")}<span class="cockpit-status" data-cloud-status="topology" role="status">"Select a region or workload example to inspect it."</span></div>
            {cloud_inspection("topology")}
        </article>
    }
}

pub(super) fn finance_finops_panel(envelope: &TenantRenderEnvelope) -> impl IntoView + use<> {
    let visible_modules = envelope.modules.len();
    view! {
                <article id="cockpit-panel-finops" data-finance-owned="finops" data-finance-owner-source=finance_sample_source(envelope) data-finance-context=envelope.context.id() class="cockpit-panel" data-cockpit-panel="finops" role="tabpanel" aria-labelledby="cockpit-tab-finops">
                    <div class="cockpit-column-head">
                        <p class="screen-anchor">"FINOPS"</p>
                        <h4 id="finops-title">"Sample April 2026 monthly spend"</h4>
                    </div>
                    <div class="finops-bars" aria-label="Illustrative April 2026 USD monthly spend shares of the sample total" inner_html=finance_finops_bars_html()></div>
                    <p>"Illustrative April 2026 USD spend fixtures. Fixture as-of 2026-04-30; live observation unavailable. Sample monthly total USD 41,500; bars are rounded shares of that sample total. Live run-rate, spend and sustainability measurements unavailable."</p>
                    <div class="finops-action-grid">
                        <button type="button" data-cockpit-action="open-commit" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Open commit plan"</button>
                        <button type="button" data-cockpit-action="tag-anomaly" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Tag anomaly"</button>
                        <button type="button" data-cockpit-action="draft-budget-note" disabled data-finance-unavailable="true" aria-describedby="finance-unavailable-reason">"Draft budget note"</button>
                    </div>
                    <span class="cockpit-status" data-cockpit-status="true">"Sample FinOps; live spend and budget operations unavailable."</span>
                    {finops_anchor_board()}
                    <p class="cockpit-note">{format!("{visible_modules} services visible in this envelope · backend wiring remains disabled")}</p>
                </article>
    }
}

pub(super) const FINOPS_SAMPLE_COSTS: [(&str, u64); 4] = [
    ("Compute", 21_400),
    ("Network", 9_800),
    ("Storage", 7_200),
    ("Audit", 3_100),
];

pub(super) fn finance_finops_bars_html() -> String {
    let total: u64 = FINOPS_SAMPLE_COSTS.iter().map(|(_, cost)| cost).sum();
    FINOPS_SAMPLE_COSTS.into_iter().map(|(category, cost)| {
        let share = (cost * 100 + total / 2) / total;
        format!(r#"<span role="progressbar" aria-valuenow="{share}" aria-valuemin="0" aria-valuemax="100" aria-label="Sample {category}: USD {cost} for April 2026; {share}% of the USD {total} sample monthly total; live observation unavailable" data-finops-sample-usd="{cost}" data-finops-sample-total-usd="{total}" style="--bar: {share}%"><em aria-hidden="true">Sample {category} · USD {cost} / month · {share}% of sample total</em></span>"#)
    }).collect()
}

pub(super) const CLOUD_REGIONS: [(&str, &str, &str, &str); 3] = [
    (
        "us-east-2",
        "primary",
        "72%",
        "Example primary cell; live capacity unavailable.",
    ),
    (
        "eu-west-1",
        "standby",
        "44%",
        "Example standby cell; live capacity unavailable.",
    ),
    (
        "kr-seoul",
        "pack gated",
        "28%",
        "Example residency review; live capacity unavailable.",
    ),
];
pub(super) const CLOUD_WORKLOADS: [(&str, &str, &str, &str, &str, &str, &str, &str); 4] = [
    (
        "workflow",
        "Workflow runner",
        "workflow-runner",
        "us-east-2",
        "review",
        "Workflow → Messenger/Mail/Community → Evidence",
        "REC-FD001-WF-018",
        "Illustrative workflow workload; no hosting or delivery verified.",
    ),
    (
        "comms",
        "Built-in communications",
        "messenger-mail-community",
        "us-east-2 + kr-seoul",
        "drafts",
        "Messenger/Mail/Community handoff bus",
        "REC-COMMS-HANDOFF-006",
        "Illustrative communications workload; no message sent.",
    ),
    (
        "evidence",
        "Evidence spine",
        "audit-vault",
        "multi-cell",
        "sealed",
        "Audit ledger + object graph",
        "REC-FD001-CLOUD-009",
        "Illustrative evidence workload; no receipt verified or created.",
    ),
    (
        "identity",
        "Identity envelope",
        "identity-access",
        "kr-seoul gated",
        "policy",
        "Identity → Policy → Deployment gates",
        "REC-ID-2026-05",
        "Illustrative identity workload; no tenant or placement verified.",
    ),
];
pub(super) fn cloud_resources_records() -> Vec<serde_json::Value> {
    use serde_json::json;
    let mut records = Vec::new();
    for (region, state, capacity, description) in CLOUD_REGIONS {
        records.push(json!([
            format!("region:{region}"),
            [
                ["Region", region],
                ["State", state],
                ["Capacity", capacity],
                ["Description", description]
            ]
        ]));
    }
    for (key, title, service, cell, state, route, receipt, description) in CLOUD_WORKLOADS {
        records.push(json!([
            format!("workload:{key}"),
            [
                ["Key", key],
                ["Title", title],
                ["Service", service],
                ["Cell", cell],
                ["State", state],
                ["Route", route],
                ["Receipt", receipt],
                ["Description", description]
            ]
        ]));
    }
    for row in resource_inventory_rows() {
        records.push(json!([
            format!("resource:{}", row.side_id),
            [
                ["Key", row.side_id],
                ["Kind", row.kind],
                ["Name", row.name],
                ["Region", row.region],
                ["Owner", row.owner],
                ["State", row.state],
                ["Monthly", row.monthly],
                ["Risk", row.risk],
                ["Description", row.description]
            ]
        ]));
    }
    for gate in deployment_gates() {
        records.push(json!([
            format!("gate:{}", gate.label),
            [
                ["Label", gate.label],
                ["Detail", gate.detail],
                ["State", gate.state],
                ["Progress", gate.progress]
            ]
        ]));
    }
    records
}
pub(super) fn cloud_resources_record(key: &str) -> String {
    cloud_resources_records()
        .into_iter()
        .find(|record| record[0] == key)
        .expect("known Cloud sample")[1]
        .to_string()
}
pub(super) fn cloud_resources_routes(envelope: &TenantRenderEnvelope) -> Vec<serde_json::Value> {
    let mut routes = vec![
        ("cloud:inventory", "resource-panel-inventory", None),
        ("cloud:gates", "resource-panel-gates", None),
        ("cloud:workflow", "workflow-studio", None),
        ("cloud:mail", "work-hub", Some("Mail")),
        ("cloud:evidence", "evidence-spine", None),
        ("workload:workflow", "workflow-studio", None),
        ("workload:mail", "work-hub", Some("Mail")),
        ("workload:community", "work-hub", Some("Community")),
        ("workload:evidence", "evidence-spine", None),
        ("workload:gates", "resource-panel-gates", None),
        ("inventory:catalog", "service-catalog", None),
        ("inventory:gates", "resource-panel-gates", None),
        ("inventory:audit", "resource-panel-audit", None),
        ("inventory:graph", "object-graph", None),
        ("inventory:workflow", "workflow-studio", None),
        ("inventory:evidence", "evidence-spine", None),
        ("inventory:mail", "work-hub", Some("Mail")),
        ("inventory:community", "work-hub", Some("Community")),
        ("gates:workflow", "workflow-studio", None),
        ("gates:cloud", "cockpit-panel-topology", None),
        ("gates:mail", "work-hub", Some("Mail")),
        ("gates:policy", "cockpit-panel-policy", None),
        ("gates:audit", "resource-panel-audit", None),
        ("gates:community", "work-hub", Some("Community")),
    ];
    if finance_sample_scope(envelope.context).finops_examples {
        routes.extend([
            ("cloud:finops", "cockpit-panel-finops", None),
            ("inventory:finops", "cockpit-panel-finops", None),
            ("gates:finops", "cockpit-panel-finops", None),
        ]);
    }
    let mut result: Vec<_> = routes
        .into_iter()
        .map(|route| serde_json::json!(route))
        .collect();
    result.extend(deployment_gates().map(|gate| {
        serde_json::json!([
            format!("gate:{}:evidence", gate.label),
            "evidence-spine",
            null
        ])
    }));
    result
}
pub(super) fn cloud_resources_source(envelope: &TenantRenderEnvelope) -> String {
    serde_json::json!([
        envelope.context.id(),
        daily_sample_label(envelope),
        HubSample::envelope_source(envelope),
        cloud_resources_records(),
        cloud_resources_routes(envelope)
    ])
    .to_string()
}
pub(super) fn cloud_route(
    envelope: &TenantRenderEnvelope,
    key: &str,
    label: &str,
) -> impl IntoView + use<> {
    let route = cloud_resources_routes(envelope)
        .into_iter()
        .find(|route| route[0] == key);
    if let Some(route) = route {
        let channel = route[2].as_str().map(str::to_owned);
        let destination = route[1].as_str().unwrap().to_owned();
        let href = channel.is_none().then(|| format!("#{destination}"));
        let admitted = href.is_some();
        view! { <a href=href data-cloud-route=key.to_owned() data-cloud-target=destination data-cloud-channel=channel
            aria-disabled=if admitted { "false" } else { "true" } title=if admitted { "Open this sample view" } else { "Sample channel unavailable until its controls are ready" }>{label.to_owned()}</a> }.into_any()
    } else {
        cloud_unavailable(key, label).into_any()
    }
}
pub(super) fn cloud_unavailable(key: &str, label: &str) -> impl IntoView + use<> {
    view! { <button type="button" disabled aria-disabled="true" data-cloud-effect=key.to_owned() title="Sample only; this operation is unavailable.">{label.to_owned()}</button> }
}
pub(super) fn cloud_inspection(scope: &str) -> impl IntoView + use<> {
    view! { <div class="cloud-inspection" id=format!("cloud-{scope}-inspection") data-cloud-inspection=scope.to_owned() hidden>
    <h5 data-cloud-inspect-title="true">"Sample detail"</h5><dl>{(0..9).map(|index| view! {
        <div data-cloud-inspect-field=index.to_string() hidden><dt data-cloud-inspect-label="true"></dt><dd data-cloud-inspect-value="true"></dd></div>
    }).collect_view()}</dl><button type="button" data-cloud-close=scope.to_owned()>"Close sample detail"</button>
    <p>"Illustrative fields only; no live resource, receipt or release verified."</p></div> }
}
