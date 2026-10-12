use super::*;

pub(super) fn catalog_available(available: Signal<bool>) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        available.try_get().unwrap_or(false)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        available.try_get_untracked().unwrap_or(false)
    }
}

pub(super) fn service_catalog_panel(
    envelope: TenantRenderEnvelope,
    available: Signal<bool>,
) -> impl IntoView {
    let source = catalog_source(&envelope);
    view! {
        <For each=move || [(source.clone(), envelope.clone())]
            key=|(source, _)| source.clone()
            children=move |(source, envelope)| catalog_panel(envelope, available, source) />
    }
}

fn catalog_source(envelope: &TenantRenderEnvelope) -> String {
    let records: Vec<_> = envelope.modules.iter().filter_map(catalog_record).collect();
    let label = daily_sample_label(envelope);
    let hub_source = HubSample::envelope_source(envelope);
    serde_json::to_string(&(
        envelope.context.id(),
        &label,
        &hub_source,
        records
            .iter()
            .map(|record| (record.id, record.fields()))
            .collect::<Vec<_>>(),
    ))
    .expect("native visible Catalog sample source serializes")
}

fn catalog_panel(
    envelope: TenantRenderEnvelope,
    available: Signal<bool>,
    source: String,
) -> impl IntoView {
    let records: Vec<_> = envelope.modules.iter().filter_map(catalog_record).collect();
    let identities: std::collections::BTreeSet<_> =
        records.iter().map(|record| record.id).collect();
    let admitted = envelope.modules
        == crate::shell_capability_registry::permitted_module_cards(envelope.context)
        && records.len() == envelope.modules.len()
        && identities.len() == records.len();
    let enabled = move || admitted && catalog_available(available);
    let local_available = Signal::derive(enabled);
    let groups: std::collections::BTreeMap<_, _> = records
        .iter()
        .map(|record| {
            (
                catalog_group_slug(&record.module.group),
                record.module.group.clone(),
            )
        })
        .collect();
    let total = records.len();
    let attention = records
        .iter()
        .filter(|record| record.state == "attention")
        .count();
    let cloud = records
        .iter()
        .filter(|record| record.module.group == "Cloud")
        .count();
    let trust = records
        .iter()
        .filter(|record| matches!(record.module.group.as_str(), "Trust" | "Control"))
        .count();
    let group_count = groups.len();
    let label = daily_sample_label(&envelope);
    let hub_source = HubSample::envelope_source(&envelope);
    let templates = catalog_hub_templates(&envelope);

    view! {
        <section id="service-catalog" class="panel modules-panel catalog-workbench" aria-labelledby="modules-title"
            data-catalog-root="true" data-catalog-context=envelope.context.id() data-catalog-label=label.clone()
            data-catalog-source=source data-catalog-hub-source=hub_source data-catalog-admitted=admitted.to_string()
            data-catalog-mode=if cfg!(target_arch="wasm32") { "island" } else { "ssr" }
            data-catalog-available=move || enabled().to_string()>
            <div class="panel-header catalog-header"><div><p class="eyebrow">"Service catalog"</p>
                <h3 id="modules-title">"Illustrative service samples"</h3></div>
                <span class="catalog-live-chip">"Anonymous samples · live authority unavailable"</span>
            </div>
            <p class="sample-context-notice">{format!("Observed anonymous sample: {label}. No tenant or role verified; signing in does not grant live Catalog authority.")}</p>
            <p>"Cards, health labels, owners, priorities, dependencies and lineage are illustrative examples, not current service status or entitlements."</p>
            <p id="catalog-availability" data-catalog-availability="true" role="status">{move || if enabled() {
                "Search, pin and inspect these browser-local samples. Live Catalog operations are unavailable."
            } else { "Catalog sample unavailable while the requested context is loading, failed, mismatched or unrecognized. Local controls are disabled." }}</p>
            <p id="catalog-unavailable-effects">"Request access and Stage manifest are unavailable: no live grant, admission or provisioning contract. No request, receipt, file or durable pin is created."</p>
            <div class="catalog-kpi-strip" aria-label="Illustrative Catalog sample summary">
                <div class="catalog-kpi accent"><span>"Sample cards"</span><strong>{total}</strong><small>"from the observed sample envelope"</small></div>
                <div class="catalog-kpi"><span>"Cloud examples"</span><strong>{cloud}</strong><small>"illustrative dependencies"</small></div>
                <div class="catalog-kpi warn"><span>"Attention examples"</span><strong>{attention}</strong><small>"sample labels; no live health"</small></div>
                <div class="catalog-kpi"><span>"Trust examples"</span><strong>{trust}</strong><small>"no authorization inferred"</small></div>
                <div class="catalog-kpi"><span>"Visible categories"</span><strong>{group_count}</strong><small>"derived from these cards"</small></div>
            </div>
            <div class="catalog-toolbar" aria-label="Catalog search and filters">
                <label class="catalog-search"><span aria-hidden="true">"⌕"</span>
                    <input id="catalog-sample-search" data-catalog-search="true" data-catalog-focus="search" type="search" aria-label="Search Catalog samples"
                        placeholder="Search samples, owners, dependencies" disabled=move || !enabled() aria-describedby="catalog-availability" />
                </label>
                <div class="filter-pills catalog-filters" role="toolbar" aria-label="Sample categories">
                    <button type="button" class="fp active" data-catalog-filter="all" data-catalog-focus="filter:all" aria-pressed="true"
                        disabled=move || !enabled()>"All"</button>
                    {groups.into_iter().map(|(slug, group)| view! {
                        <button type="button" class="fp" data-catalog-filter=slug data-catalog-focus=format!("filter:{slug}")
                            aria-pressed="false" disabled=move || !enabled()>{group}</button>
                    }).collect_view()}
                    <button type="button" class="fp" data-catalog-attention="true" data-catalog-focus="attention"
                        aria-pressed="false" disabled=move || !enabled()>"Attention examples"</button>
                    <button type="button" data-catalog-reset="true" data-catalog-focus="reset" disabled=move || !enabled()>"Reset local view"</button>
                </div>
                <span class="catalog-status" data-catalog-status="true" role="status" aria-live="polite">{format!("{total} of {total} sample cards · all categories")}</span>
            </div>
            <p data-catalog-empty="true" hidden>"No sample cards match. Reset the local view to see the observed sample cards."</p>
            <section class="catalog-detail" data-catalog-inspection="true" id="catalog-sample-detail" aria-label="Selected Catalog sample" hidden>
                <h4 data-catalog-detail-title="true">"Sample record"</h4><p>"Illustrative fields only; live verification and operations unavailable."</p>
                <dl>{(0..12).map(|index| view! {
                    <div data-catalog-inspect-field=index hidden><dt data-catalog-inspect-label="true"></dt><dd data-catalog-inspect-value="true"></dd></div>
                }).collect_view()}</dl>
                <button type="button" data-catalog-close="true" data-catalog-focus="close" disabled=move || !enabled()>"Close sample inspection"</button>
            </section>
            <div class="catalog-workspace">
                <div class="catalog-table-shell" role="region" aria-label="Illustrative sample cards; scroll horizontally for all fields and actions"
                    tabindex="0" data-catalog-focus="table">
                    <div class="catalog-table-head" aria-hidden="true"><span>"Sample health"</span><span>"Module"</span><span>"Category"</span>
                        <span>"Sample owner"</span><span>"Illustrative dependency"</span><span>"Local actions"</span></div>
                    <div class="catalog-module-list">{records.into_iter().map(|record| catalog_module(record, local_available)).collect_view()}</div>
                </div>
                <aside id="service-graph" class="catalog-service-graph" aria-label="Illustrative sample lineage">
                    <div class="graph-head"><p class="screen-anchor">"SAMPLE GRAPH"</p><strong>"Illustrative service lineage"</strong>
                        <span>"Examples only. Live relationships, delivery and verified receipts are unavailable."</span></div>
                    <ol class="lineage-list">
                        <li class="root"><span>"Workflow"</span><strong>"Tenant change approval"</strong><em>"example root"</em></li>
                        <li><span>"Messenger"</span><strong>"Ops room sample"</strong><em>"no message created"</em></li>
                        <li><span>"Mail"</span><strong>"Formal approval sample"</strong><em>"no draft created"</em></li>
                        <li><span>"Community"</span><strong>"Governance council sample"</strong><em>"no publication"</em></li>
                        <li><span>"Cloud Ops"</span><strong>"Runbook + FinOps examples"</strong><em>"no live operation"</em></li>
                        <li><span>"Audit"</span><strong>"Receipt example"</strong><em>"unverified"</em></li>
                    </ol>
                    <div class="catalog-graph-actions" aria-label="Existing sample graph routes">
                        {catalog_link("graph-workflow", "workflow-studio", "", "Workflow sample")}
                        {catalog_link("graph-mail", "work-hub", "Mail", "Mail sample")}
                        {catalog_link("graph-community", "work-hub", "Community", "Community sample")}
                        {catalog_link("graph-evidence", "evidence-spine", "", "Evidence sample")}
                    </div>
                </aside>
            </div>
            {service_catalog_anchor_board()}
            <p class="catalog-footer-hint omitted-note">{envelope.omitted_capability_note}</p>
            <p data-catalog-route-status="true" role="status">"Existing sample routes only; no operation or output performed."</p>
            {templates}
        </section>
    }
}

pub(super) fn catalog_module(record: CatalogRecord, available: Signal<bool>) -> impl IntoView {
    let fields =
        serde_json::to_string(&record.fields()).expect("visible Catalog record serializes");
    let health = catalog_state_label(record.state);
    let group_slug = catalog_group_slug(&record.module.group);
    let unavailable_id = format!("catalog-unavailable-{}", record.id);
    let title_marker = format!("CAT-{}", catalog_code_for(&record.module.name));
    let open = if record.destination.is_empty() {
        view! { <button type="button" disabled aria-describedby=unavailable_id.clone()>"Open unavailable"</button> }.into_any()
    } else if record.id == "foundry-ontology" {
        view! { <button type="button" data-catalog-action="open" data-catalog-target=record.destination data-catalog-focus="open:foundry-ontology"
            disabled=move || !catalog_available(available)>"Open ontology sample"</button> }.into_any()
    } else {
        view! { <a href=format!("#{}", record.destination) data-catalog-route=record.id data-catalog-channel=record.channel
            data-catalog-focus=format!("open:{}", record.id) aria-describedby=format!("catalog-availability {unavailable_id}")>
            <span data-catalog-field="Action">{record.module.action_label.clone()}</span>" sample"
        </a> }.into_any()
    };
    view! {
        <article class="catalog-module-row module-card" data-catalog-module=record.id data-catalog-record=fields
            data-catalog-group=group_slug data-catalog-state=record.state>
            <span class="catalog-health"><span class=format!("health-dot health-{}", record.state) aria-hidden="true"></span>
                <span data-catalog-field="Sample health">{health}</span></span>
            <div class="catalog-module-main">
                <button type="button" class="catalog-module-title" data-catalog-detail=record.id data-sidepeek-id=title_marker
                    data-catalog-field="Module" data-catalog-focus=format!("detail:{}", record.id)
                    aria-controls="catalog-sample-detail" aria-expanded="false" disabled=move || !catalog_available(available)>{record.module.name.clone()}</button>
                <p data-catalog-field="Description">{record.module.description.clone()}</p><code>{catalog_code_for(&record.module.name)}</code>
            </div>
            <span class="cat-tag" data-catalog-field="Category">{record.module.group}</span>
            <span class="owner-cell"><span class="avatar-xs" aria-hidden="true">{catalog_owner_avatar(record.owner)}</span>
                <span data-catalog-field="Sample owner">{record.owner}</span></span>
            <span class="catalog-dependency-chain"><em>"Workflow example"</em><i aria-hidden="true">"→"</i>
                <em data-catalog-field="Sample dependency">{record.dependency}</em><i aria-hidden="true">"→"</i><em>"Audit example"</em>
                <span class="crit" data-catalog-field="Sample priority">{record.priority}</span></span>
            <span class="catalog-row-actions">{open}
                <button type="button" data-catalog-pin=record.id data-catalog-focus=format!("pin:{}", record.id) aria-pressed="false"
                    disabled=move || !catalog_available(available)>"Pin locally"</button>
                <button type="button" disabled data-catalog-unavailable="request" aria-describedby="catalog-unavailable-effects">"Request access unavailable"</button>
            </span>
            <small id=unavailable_id class="catalog-open-reason" hidden=!record.destination.is_empty()>"Dedicated product unavailable; this card is an illustrative sample only."</small>
        </article>
    }
}

pub(super) fn catalog_link(
    key: &'static str,
    destination: &'static str,
    channel: &'static str,
    label: &'static str,
) -> impl IntoView {
    let unavailable_id = format!("catalog-link-unavailable-{key}");
    view! {
        <span class="catalog-connected-link">
            <a href=format!("#{destination}") data-catalog-link=key data-catalog-channel=channel
                data-catalog-focus=format!("route:{key}") aria-describedby=format!("catalog-availability {unavailable_id}")>{label}</a>
            <small id=unavailable_id class="catalog-open-reason" data-catalog-link-reason=key hidden></small>
        </span>
    }
}

pub(super) fn service_catalog_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board catalog-trust-board" aria-label="Illustrative Catalog dependencies; no live admission proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected"><p class="screen-anchor">"SAMPLE MODULE GRAPH"</p><h5>"Inspect the visible sample cards"</h5>
                    <p>"This anonymous sample does not establish service admission, health or entitlement."</p><div class="trust-anchor-actions">
                        <button type="button" disabled data-catalog-unavailable="stage" aria-describedby="catalog-unavailable-effects">"Stage manifest unavailable"</button>
                        {catalog_link("workflow", "workflow-studio", "", "Workflow sample")}
                    </div></article>
                <article class="trust-anchor-card"><p class="screen-anchor">"ILLUSTRATIVE CLOUD"</p><h5>"Example dependencies only"</h5>
                    <p>"No live hosting, deployment or production readiness is verified."</p><div class="trust-anchor-actions">
                        {catalog_link("cloud", "cockpit-panel-topology", "", "Cloud sample")}
                        {catalog_link("gates", "resource-panel-gates", "", "Gate examples")}
                    </div></article>
                <article class="trust-anchor-card"><p class="screen-anchor">"BROWSER-LOCAL CATALOG"</p><h5>"Filter, pin and inspect samples"</h5>
                    <p>"Local pins last only for this page. No provisioning or durable changes."</p><div class="trust-anchor-actions">
                        {catalog_link("policy", "cockpit-panel-policy", "", "Policy sample")}
                        {catalog_link("audit", "resource-panel-audit", "", "Audit sample")}
                    </div></article>
            </div>
            <div class="trust-anchor-footer"><span>"Illustrative Catalog lineage; live authority unavailable."</span>
                <div class="trust-anchor-routes" aria-label="Existing connected samples">
                    {catalog_link("finance", "finance-panel-ledger", "", "Finance sample")}
                    {catalog_link("identity", "identity-panel-auth", "", "Identity sample")}
                    {catalog_link("daily", "daily-execution", "", "Daily sample")}
                    {catalog_link("evidence", "evidence-spine", "", "Evidence sample")}
                </div></div>
        </div>
    }
}

fn catalog_hub_templates(envelope: &TenantRenderEnvelope) -> impl IntoView + use<> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        view! { {HubChannel::ALL.into_iter().map(|channel| view! {
            <template data-catalog-hub-template=channel.label() inner_html=communication_hub_html(envelope, channel)></template>
        }).collect_view()} }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = envelope;
        view! {}
    }
}

#[cfg(test)]
#[path = "catalog_identity_test.rs"]
mod tests;
