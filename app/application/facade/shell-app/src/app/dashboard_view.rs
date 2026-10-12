use super::*;

#[expect(
    clippy::too_many_arguments,
    reason = "Leptos shell view composes several reactive signals at the island boundary; refactoring into state bags would obscure the explicit contract-envelope flow."
)]
pub(super) fn dashboard_view(
    envelope: TenantRenderEnvelope,
    workflow_sample: RwSignal<WorkflowSample>,
    active_surface: ReadSignal<ProductSurface>,
    set_active_surface: WriteSignal<ProductSurface>,
    hub_sample: RwSignal<HubSample>,
    hub_available: Signal<bool>,
) -> impl IntoView {
    view! {
        {surface_command_bar(active_surface, set_active_surface, hub_sample, hub_available)}

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

        {command_center_workbench(envelope.clone(), hub_available)}

        {tenant_rbac_board(envelope.clone(), hub_available)}

        {identity_workforce_service(envelope.clone(), hub_available)}

        {finance_commercial_service(envelope.clone(), hub_available)}

        {operator_intelligence_strip(envelope.clone(), hub_available)}

        {tenant_operations_cockpit(envelope.clone())}

        {resource_audit_console(envelope.clone(), hub_available)}

        <section class="dashboard-grid" aria-label="Personalized dashboard">
            {daily_execution_console(envelope.clone(), hub_available)}

            <section id="work-hub" class="panel communications-panel" aria-labelledby="messages-title">
                <PanelHeader heading_id="messages-title" eyebrow="Messenger · Mail · Community" title={"Work hub".to_string()} />
                {payroll_preview_detail(PayrollPreview::Mail)}
                {communication_hub(hub_sample, set_active_surface, hub_available)}
            </section>

            {service_catalog_panel(envelope.clone(), hub_available)}
        </section>

        <section class="studio-grid" aria-label="Workflow, ontology, and intelligence">
            {workflow_studio_panel(
                envelope.clone(),
                workflow_sample,
            )}

            <section id="ontology-command-console" class="panel ontology-command-shell" aria-labelledby="ontology-title">
                <PanelHeader heading_id="ontology-title" eyebrow="Ontology" title={"Tenant workload graph".to_string()} />
                {ontology_list(envelope.ontology.clone(), envelope.context)}
            </section>

            {intelligence_command_console(envelope.clone(), hub_available)}
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
    hub_sample: RwSignal<HubSample>,
    hub_available: Signal<bool>,
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
                    on:click=move |_| {
                        if surface.hub_channel().is_none() || hub_editing_available(hub_available) {
                            select_product_surface(surface, set_active_surface, hub_sample);
                        }
                    }
                >
                    <span>{surface.label()}</span>
                    <small>{surface.summary()}</small>
                </a>
            }).collect_view()}
        </nav>
    }
}

pub(super) fn select_product_surface(
    surface: ProductSurface,
    set_active_surface: WriteSignal<ProductSurface>,
    hub_sample: RwSignal<HubSample>,
) {
    set_active_surface.set(surface);
    if let Some(channel) = surface.hub_channel() {
        edit_hub_sample(hub_sample, |sample| sample.select_channel(channel));
    }
}

#[cfg(test)]
#[test]
fn workflow_selection_keeps_a_full_valid_communication_channel() {
    let owner = Owner::new();
    owner.with(|| {
        let html = DashboardIsland().into_view().to_html();
        assert!(html.contains("href=\"#workflow-studio\" class=\"surface-command active\""));
        let hub = html.split_once("id=\"work-hub\"").expect("work hub").1;
        let tabs = hub
            .split_once("class=\"hub-tabs\"")
            .unwrap()
            .1
            .split_once("</div>")
            .unwrap()
            .0;
        assert_eq!(tabs.matches("aria-selected=\"true\"").count(), 1);
        let messenger = tabs
            .split_once("id=\"hub-tab-messenger\"")
            .unwrap()
            .1
            .split_once("</button>")
            .unwrap()
            .0;
        assert!(messenger.contains("aria-selected=\"true\""));
        assert!(messenger.contains("aria-controls=\"hub-panel-messenger\""));
        assert!(messenger.contains(">Messenger"));
        let list = hub
            .split_once("class=\"hub-list\"")
            .unwrap()
            .1
            .split_once("class=\"hub-detail\"")
            .unwrap()
            .0;
        assert!(list.matches("data-comms-item=\"true\"").count() >= 2);
        assert!(hub.contains("class=\"comms-detail-card\""));
        assert!(hub.contains("id=\"hub-composer-input\""));
        let (active, set_active) = signal(ProductSurface::Workflow);
        let envelope = server_derived_envelope(OperatorContext::TenantAdmin);
        let sample = RwSignal::new(HubSample::from_envelope(&envelope));
        for target in [HubChannel::Mail, HubChannel::Community] {
            select_product_surface(target.surface(), set_active, sample);
            assert_eq!(render_signal(active), target.surface());
            assert_eq!(render_hub_sample(sample).channel, target);
            edit_hub_sample(sample, |state| {
                state.composer = "Keeps its communication channel".into();
                state.queue();
            });
            let before = render_hub_sample(sample);
            select_product_surface(ProductSurface::Workflow, set_active, sample);
            assert_eq!(render_signal(active), ProductSurface::Workflow);
            assert_eq!(render_hub_sample(sample), before);
            let html = communication_hub(sample, set_active, Signal::derive(|| true))
                .into_view()
                .to_html();
            assert!(html.contains(&format!("data-comms-board-surface=\"{}\"", target.label())));
            assert!(html.contains("<strong>Local draft queued</strong>"));
            assert!(html.contains("id=\"hub-composer-input\""));
        }
    });
    owner.cleanup();
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
