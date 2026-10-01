use super::*;

pub(super) const CATALOG_FILTERS: [(&str, &str); 9] = [
    ("all", "All"),
    ("control", "Control"),
    ("cloud", "Cloud"),
    ("operations", "Ops"),
    ("trust", "Trust"),
    ("corporate", "Corporate"),
    ("daily", "Daily"),
    ("workflow", "Workflow"),
    ("no-code", "No-code"),
];

pub(super) fn service_catalog_panel(
    modules: Vec<ModuleCard>,
    omitted_note: String,
) -> impl IntoView {
    let total_modules = modules.len();
    let attention_count = modules
        .iter()
        .filter(|module| catalog_state_for(&module.group, &module.name) == "attention")
        .count();
    let cloud_count = modules
        .iter()
        .filter(|module| catalog_group_slug(&module.group) == "cloud")
        .count();
    let trust_count = modules
        .iter()
        .filter(|module| matches!(catalog_group_slug(&module.group), "trust" | "control"))
        .count();

    view! {
        <div class="catalog-kpi-strip" aria-label="Service catalog summary">
            <div class="catalog-kpi accent"><span>"Permitted modules"</span><strong>{total_modules}</strong><small>"from server envelope"</small></div>
            <div class="catalog-kpi"><span>"Cloud dependencies"</span><strong>{cloud_count}</strong><small>"compute · network · cells"</small></div>
            <div class="catalog-kpi warn"><span>"Need attention"</span><strong>{attention_count}</strong><small>"review before promote"</small></div>
            <div class="catalog-kpi"><span>"Trust surfaces"</span><strong>{trust_count}</strong><small>"roles · audit · policy"</small></div>
            <div class="catalog-kpi"><span>"Cross-service routes"</span><strong>"7"</strong><small>"workflow → mail/community/ops"</small></div>
        </div>

        <div class="catalog-toolbar" aria-label="Catalog search and filters">
            <label class="catalog-search">
                <span aria-hidden="true">"⌕"</span>
                <input data-catalog-search="true" type="search" aria-label="Search service catalog" placeholder="Search modules, owners, dependencies..." />
            </label>
            <div class="filter-pills catalog-filters" role="toolbar" aria-label="Catalog filters">
                {CATALOG_FILTERS.into_iter().map(|(slug, label)| view! {
                    <button
                        type="button"
                        class=if slug == "all" { "fp active" } else { "fp" }
                        data-catalog-filter=slug
                    >
                        <span class="fp-dot" aria-hidden="true"></span>{label}
                    </button>
                }).collect_view()}
                <button type="button" class="fp" data-catalog-filter="attention">
                    <span class="fp-dot danger" aria-hidden="true"></span>"Attention"
                </button>
            </div>
            <span class="catalog-status" data-catalog-status="true">
                <strong data-catalog-visible-count="true">{total_modules}</strong>
                " visible · all filter · local catalog only"
            </span>
        </div>

        <div class="catalog-workspace">
            <div class="catalog-table-shell" role="region" aria-label="Permitted modules table">
                <div class="catalog-table-head" aria-hidden="true">
                    <span>"Health"</span>
                    <span>"Module"</span>
                    <span>"Category"</span>
                    <span>"Owner"</span>
                    <span>"Downstream graph"</span>
                    <span>"Actions"</span>
                </div>
                <div class="catalog-module-list">
                    {modules.into_iter().map(service_catalog_module).collect_view()}
                </div>
            </div>

            <aside id="service-graph" class="catalog-service-graph" aria-label="Service graph and module lineage">
                <div class="graph-head">
                    <p class="screen-anchor">"SERVICE GRAPH"</p>
                    <strong>"One cohesive Oyatie nervous system"</strong>
                    <span>"Workflow events fan out to built-in surfaces and return audit evidence."</span>
                </div>
                <ol class="lineage-list">
                    <li class="root"><span>"Workflow"</span><strong>"Tenant change approval"</strong><em>"root event"</em></li>
                    <li><span>"Messenger"</span><strong>"Ops room draft"</strong><em>"delivered"</em></li>
                    <li><span>"Mail"</span><strong>"Formal approval brief"</strong><em>"pending"</em></li>
                    <li><span>"Community"</span><strong>"Governance council note"</strong><em>"review"</em></li>
                    <li><span>"Cloud Ops"</span><strong>"Runbook + FinOps"</strong><em>"guarded"</em></li>
                    <li><span>"Audit"</span><strong>"Receipt spine"</strong><em>"sealed"</em></li>
                </ol>
                <div class="catalog-graph-actions" aria-label="Service graph actions">
                    <button type="button" data-catalog-graph-action="workflow">"Open workflow"</button>
                    <button type="button" data-catalog-graph-action="mail">"Mail route"</button>
                    <button type="button" data-catalog-graph-action="community">"Community route"</button>
                    <button type="button" data-catalog-graph-action="evidence">"Evidence spine"</button>
                </div>
            </aside>
        </div>

        {service_catalog_anchor_board()}

        <p class="catalog-footer-hint omitted-note">
            <span aria-hidden="true">"✦"</span>
            {omitted_note}
        </p>
    }
}

pub(super) fn service_catalog_anchor_board() -> impl IntoView {
    view! {
        <div class="trust-anchor-board catalog-trust-board" aria-label="FD-001 service catalog and Oyatie Cloud tenant workload proof">
            <div class="trust-anchor-grid">
                <article class="trust-anchor-card selected" data-trust-proof-card="catalog-fd001">
                    <p class="screen-anchor">"FD-001 MODULE CONTRACT"</p>
                    <h5>"Catalog is the service graph manifest"</h5>
                    <p>
                        "Core, Workflow, Messenger, Mail, Community, Finance, Identity, Ontology, Intelligence, and Daily Work are presented as one permitted FD-001 tenant workload graph."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="stage-catalog">"Stage manifest"</button>
                        <button type="button" data-trust-proof-action="route-workflow">"Workflow proof"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="catalog-cloud">
                    <p class="screen-anchor">"OYATIE CLOUD ADMISSION"</p>
                    <h5>"Substrate dependencies are visible first"</h5>
                    <p>
                        "Cloud cells, policy gates, resource inventory, deployment gates, FinOps, and audit receipts make hosting readiness explicit before service claims."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-cloud">"Cloud substrate"</button>
                        <button type="button" data-trust-proof-action="route-gates">"Deployment gates"</button>
                    </div>
                </article>
                <article class="trust-anchor-card" data-trust-proof-card="catalog-local">
                    <p class="screen-anchor">"LOCAL-ONLY CATALOG OPS"</p>
                    <h5>"Request, pin, and route without provisioning"</h5>
                    <p>
                        "Operators can filter modules, inspect dependencies, pin rows, and route reviewers visually; no service admission, IAM, deploy, billing, or cloud mutation executes."
                    </p>
                    <div class="trust-anchor-actions">
                        <button type="button" data-trust-proof-action="route-policy">"Policy"</button>
                        <button type="button" data-trust-proof-action="route-audit">"Audit ledger"</button>
                    </div>
                </article>
            </div>
            <div class="trust-anchor-footer">
                <span data-trust-proof-status="true">
                    "Service catalog ready · FD-001 module graph dogfoods Oyatie Cloud as local tenant workload proof."
                </span>
                <div class="trust-anchor-routes" aria-label="Service catalog connected routes">
                    <button type="button" data-trust-proof-action="route-finance">"Finance"</button>
                    <button type="button" data-trust-proof-action="route-identity">"Identity"</button>
                    <button type="button" data-trust-proof-action="route-daily">"Daily Work"</button>
                    <button type="button" data-trust-proof-action="route-evidence">"Evidence"</button>
                </div>
            </div>
        </div>
    }
}

pub(super) fn service_catalog_module(module: ModuleCard) -> impl IntoView {
    let group_slug = catalog_group_slug(&module.group);
    let state = catalog_state_for(&module.group, &module.name);
    let state_label = catalog_state_label(state);
    let health_class = format!("health-dot health-{state}");
    let owner = catalog_owner_for(&module.group, &module.name);
    let avatar = catalog_owner_avatar(owner);
    let criticality = catalog_criticality_for(&module.group, &module.name);
    let criticality_class = format!("crit crit-{criticality}");
    let route = catalog_route_for(&module.name);
    let dependency = catalog_dependency_for(&module.group, &module.name);
    let sidepeek_desc = format!(
        "{} · {} · {}",
        module.description, owner, "local visual-only catalog module"
    );
    let sidepeek_id = format!("CAT-{}", catalog_code_for(&module.name));

    view! {
        <article
            class="catalog-module-row module-card"
            data-catalog-module="true"
            data-catalog-group=group_slug
            data-catalog-state=state
        >
            <span class=health_class aria-label=state_label></span>
            <div class="catalog-module-main">
                <button
                    type="button"
                    class="catalog-module-title"
                    data-sidepeek-trigger="catalog-module"
                    data-sidepeek-title=module.name.clone()
                    data-sidepeek-id=sidepeek_id
                    data-sidepeek-desc=sidepeek_desc
                    data-sidepeek-owner=owner
                    data-sidepeek-risk=state_label
                    data-sidepeek-sla="4.0h review window"
                >
                    {module.name.clone()}
                </button>
                <p>{module.description.clone()}</p>
                <code>{catalog_code_for(&module.name)}</code>
            </div>
            <span class="cat-tag">{module.group.clone()}</span>
            <span class="owner-cell">
                <span class="avatar-xs" aria-hidden="true">{avatar}</span>
                <span>{owner}</span>
            </span>
            <span class="catalog-dependency-chain">
                <em>"Workflow"</em>
                <i aria-hidden="true">"→"</i>
                <em>{dependency}</em>
                <i aria-hidden="true">"→"</i>
                <em>"Audit"</em>
                <span class=criticality_class>{criticality}</span>
            </span>
            <span class="catalog-row-actions">
                <button type="button" data-catalog-action="open" data-catalog-target=route>
                    {module.action_label}
                </button>
                <button type="button" data-catalog-action="pin">"Pin"</button>
                <button type="button" data-catalog-action="request">"Request access"</button>
            </span>
        </article>
    }
}
