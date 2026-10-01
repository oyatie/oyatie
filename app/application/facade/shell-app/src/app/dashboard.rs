use super::*;

#[component]
pub fn DashboardIsland() -> impl IntoView {
    let initial_envelope = initial_envelope();
    let initial_node_id = initial_envelope
        .as_ref()
        .and_then(|envelope| envelope.workflow.nodes.first())
        .map(|node| node.id.clone())
        .unwrap_or_default();

    let (active_context, set_active_context) = signal(OperatorContext::TenantAdmin);
    let (selected_node_id, set_selected_node_id) = signal(initial_node_id);
    let (envelope, set_envelope) = signal(initial_envelope);
    let request_generation = RwSignal::new(0_u64);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (active_surface, set_active_surface) = signal(ProductSurface::Workflow);
    let (workflow_tool, set_workflow_tool) = signal(WorkflowTool::Select);
    let (draft_node_count, set_draft_node_count) = signal(0_usize);
    let (selected_hub_index, set_selected_hub_index) = signal(0_usize);
    let (draft_body, set_draft_body) = signal(String::new());
    let (local_drafts, set_local_drafts) = signal(Vec::<LocalDraft>::new());

    #[cfg(target_arch = "wasm32")]
    if envelope.get_untracked().is_none() {
        request_render_envelope(
            OperatorContext::TenantAdmin,
            request_generation,
            set_envelope,
            set_selected_node_id,
            set_loading,
            set_error,
        );
    }

    // Bind navigation after the dashboard renders, including when reactive state
    // replaces its panels. Listener cleanup belongs to the island's lifetime.
    #[cfg(target_arch = "wasm32")]
    {
        on_cleanup(detach_island_listeners);
        Effect::new(move |_| {
            let _ = (
                envelope.get(),
                selected_node_id.get(),
                active_surface.get(),
                workflow_tool.get(),
                draft_node_count.get(),
                selected_hub_index.get(),
                draft_body.get(),
                local_drafts.get(),
            );
            // The render effect can run after this effect. Wait one microtask so
            // listener targets come from the settled dashboard DOM.
            leptos::task::spawn_local_scoped_with_cancellation(async {
                detach_island_listeners();
                wire_tablist_keyboard_navigation();
                wire_fragment_navigation();
                restore_fragment_destination();
            });
        });
    }

    view! {
        <div
            class=move || if render_signal(loading) { "dashboard-island loading" } else { "dashboard-island" }
            data-island="render-envelope-dashboard"
            aria-live="polite"
            aria-busy=move || render_signal(loading).to_string()
        >
            <section class="context-switcher island-frame" aria-labelledby="context-title">
                <div>
                    <p class="eyebrow">"Context"</p>
                    <h2 id="context-title">"Switch render envelope"</h2>
                    <span class="island-label">"interactive island"</span>
                </div>
                <div class="context-grid" role="list" aria-label="Tenant and role contexts">
                    {OperatorContext::ALL.into_iter().map(|context| view! {
                        <button
                            type="button"
                            class=move || if render_signal(active_context) == context { "context-card selected" } else { "context-card" }
                            aria-pressed=move || (render_signal(active_context) == context).to_string()
                            on:click=move |_| {
                                set_active_context.set(context);
                                request_render_envelope(
                                    context,
                                    request_generation,
                                    set_envelope,
                                    set_selected_node_id,
                                    set_loading,
                                    set_error,
                                );
                                set_active_surface.set(ProductSurface::Workflow);
                                set_workflow_tool.set(WorkflowTool::Select);
                                set_draft_node_count.set(0);
                                set_selected_hub_index.set(0);
                                set_draft_body.set(String::new());
                                set_local_drafts.set(Vec::new());
                            }
                        >
                            <span class="context-icon" aria-hidden="true">{context_icon(context)}</span>
                            <span class="context-label">{context.label()}</span>
                            <span class="context-role">{context.role()}</span>
                        </button>
                    }).collect_view()}
                </div>
            </section>

            {move || render_signal(error).map(|message| view! {
                <p class="fetch-error" role="alert">{message}</p>
            })}

            {move || match render_signal(envelope) {
                Some(envelope) => dashboard_view(
                    envelope,
                    render_signal(selected_node_id),
                    set_selected_node_id,
                    active_surface,
                    set_active_surface,
                    workflow_tool,
                    set_workflow_tool,
                    draft_node_count,
                    set_draft_node_count,
                    selected_hub_index,
                    set_selected_hub_index,
                    draft_body,
                    set_draft_body,
                    local_drafts,
                    set_local_drafts,
                ).into_any(),
                None => loading_state().into_any(),
            }}
        </div>
    }
}

pub(super) fn loading_state() -> impl IntoView {
    view! {
        <section class="panel loading-panel" aria-label="Loading permitted dashboard">
            <p class="eyebrow">"Server render envelope"</p>
            <h2>"Loading permitted dashboard"</h2>
            <p>"Fetching only the modules and workflow state allowed for this tenant and role."</p>
        </section>
    }
}

pub(super) fn context_icon(context: OperatorContext) -> &'static str {
    match context {
        OperatorContext::TenantAdmin => "◇",
        OperatorContext::CorporateOffice => "▣",
        OperatorContext::HealthcareClinician => "✚",
    }
}
