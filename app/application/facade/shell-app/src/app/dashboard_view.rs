use super::*;

#[expect(
    clippy::too_many_arguments,
    reason = "Leptos shell view composes several reactive signals at the island boundary; refactoring into state bags would obscure the explicit contract-envelope flow."
)]
pub(super) fn dashboard_view(
    envelope: TenantRenderEnvelope,
    selected_node_id: String,
    set_selected_node_id: WriteSignal<String>,
    active_surface: ReadSignal<ProductSurface>,
    set_active_surface: WriteSignal<ProductSurface>,
    workflow_tool: ReadSignal<WorkflowTool>,
    set_workflow_tool: WriteSignal<WorkflowTool>,
    draft_node_count: ReadSignal<usize>,
    set_draft_node_count: WriteSignal<usize>,
    selected_hub_index: ReadSignal<usize>,
    set_selected_hub_index: WriteSignal<usize>,
    draft_body: ReadSignal<String>,
    set_draft_body: WriteSignal<String>,
    local_drafts: ReadSignal<Vec<LocalDraft>>,
    set_local_drafts: WriteSignal<Vec<LocalDraft>>,
) -> impl IntoView {
    let display_nodes =
        workflow_display_nodes(&envelope.workflow.nodes, render_signal(draft_node_count));
    let selected_node = selected_workflow_node(&display_nodes, &selected_node_id)
        .cloned()
        .or_else(|| display_nodes.first().cloned());

    view! {
        {surface_command_bar(active_surface, set_active_surface)}

        {product_activity_spine(envelope.product_activity.clone())}

        <section class="envelope-banner" aria-labelledby="envelope-title">
            <div>
                <p class="eyebrow">"Active context"</p>
                <h2 id="envelope-title">{envelope.tenant_name.clone()}</h2>
                <p>{envelope.role_name.clone()}</p>
            </div>
            <div class="envelope-detail">
                <span class="badge">{envelope.tenant_class.clone()}</span>
                <span class=if envelope.accreditation.healthcare_enabled { "badge success" } else { "badge warning" }>
                    {envelope.accreditation.label.clone()}
                </span>
                <p>{envelope.server_derivation_note.clone()}</p>
            </div>
        </section>

        {metric_grid(envelope.metrics.clone())}

        {command_shell_substrate()}

        {substrate_proof_command(envelope.clone())}

        {command_center_workbench(envelope.clone())}

        {tenant_rbac_board(envelope.clone())}

        {identity_workforce_service()}

        {finance_commercial_service()}

        {operator_intelligence_strip(envelope.clone())}

        {tenant_operations_cockpit(envelope.clone())}

        {resource_audit_console(envelope.clone())}

        <section class="dashboard-grid" aria-label="Personalized dashboard">
            {daily_execution_console(envelope.clone())}

            <section id="work-hub" class="panel communications-panel" aria-labelledby="messages-title">
                <PanelHeader heading_id="messages-title" eyebrow="Messenger · Mail · Community" title={"Work hub".to_string()} />
                {communication_hub(
                    envelope.messages.clone(),
                    envelope.community.clone(),
                    active_surface,
                    set_active_surface,
                    selected_hub_index,
                    set_selected_hub_index,
                    draft_body,
                    set_draft_body,
                    local_drafts,
                    set_local_drafts,
                )}
            </section>

            <section id="service-catalog" class="panel modules-panel catalog-workbench" aria-labelledby="modules-title">
                <div class="panel-header catalog-header">
                    <div>
                        <p class="eyebrow">"Service catalog"</p>
                        <h3 id="modules-title">"Permitted service graph"</h3>
                    </div>
                    <span class="catalog-live-chip">"local · visually interactive"</span>
                </div>
                {service_catalog_panel(envelope.modules.clone(), envelope.omitted_capability_note.clone())}
            </section>
        </section>

        <section class="studio-grid" aria-label="Workflow, ontology, and intelligence">
            {workflow_studio_panel(
                envelope.workflow.name.clone(),
                envelope.workflow.goal.clone(),
                display_nodes,
                selected_node,
                set_selected_node_id,
                workflow_tool,
                set_workflow_tool,
                draft_node_count,
                set_draft_node_count,
                set_active_surface,
            )}

            <section id="ontology-command-console" class="panel ontology-command-shell" aria-labelledby="ontology-title">
                <PanelHeader heading_id="ontology-title" eyebrow="Ontology" title={"Tenant workload graph".to_string()} />
                {ontology_list(envelope.ontology.clone(), envelope.context)}
            </section>

            <section id="intelligence-command-console" class="panel intelligence-command-shell" aria-labelledby="intelligence-title">
                <PanelHeader heading_id="intelligence-title" eyebrow="Intelligence" title={"Governed AI command".to_string()} />
                {suggestion_list(envelope.intelligence.clone())}
            </section>
        </section>
    }
}

#[component]
pub(super) fn PanelHeader(
    heading_id: &'static str,
    eyebrow: &'static str,
    title: String,
) -> impl IntoView {
    view! {
        <div class="panel-header">
            <p class="eyebrow">{eyebrow}</p>
            <h3 id=heading_id>{title}</h3>
        </div>
    }
}

pub(super) fn surface_command_bar(
    active_surface: ReadSignal<ProductSurface>,
    set_active_surface: WriteSignal<ProductSurface>,
) -> impl IntoView {
    view! {
        <nav class="surface-command-bar" aria-label="Open built-in product surface">
            {ProductSurface::ALL.into_iter().map(|surface| view! {
                <a
                    href=surface.href()
                    class=move || {
                        if render_signal(active_surface) == surface {
                            "surface-command active"
                        } else {
                            "surface-command"
                        }
                    }
                    on:click=move |_| set_active_surface.set(surface)
                >
                    <span>{surface.label()}</span>
                    <small>{surface.summary()}</small>
                </a>
            }).collect_view()}
        </nav>
    }
}

pub(super) fn metric_grid(metrics: Vec<MetricCard>) -> impl IntoView {
    view! {
        <section class="metric-grid" aria-label="Dashboard metrics">
            {metrics.into_iter().map(|metric| view! {
                <article class="metric-card">
                    <p>{metric.label}</p>
                    <strong>{metric.value}</strong>
                    <span>{metric.detail}</span>
                </article>
            }).collect_view()}
        </section>
    }
}
