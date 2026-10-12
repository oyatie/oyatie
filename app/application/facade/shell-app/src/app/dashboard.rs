use super::*;

#[component]
pub fn DashboardIsland() -> impl IntoView {
    let initial_envelope = initial_envelope();
    let initial_context = workflow_initial_context();
    let initial_workflow = initial_envelope
        .as_ref()
        .map(WorkflowSample::from_envelope)
        .unwrap_or_default();
    let workflow_sample = RwSignal::new(initial_workflow);
    let initial_hub = initial_envelope
        .as_ref()
        .map(HubSample::from_envelope)
        .unwrap_or_else(|| HubSample::empty(initial_context));
    let hub_sample = RwSignal::new(initial_hub);
    let (active_context, set_active_context) = signal(initial_context);
    let (envelope, set_envelope) = signal(initial_envelope);
    let request_generation = RwSignal::new(0_u64);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (active_surface, set_active_surface) = signal(ProductSurface::Workflow);
    let hub_available = Signal::derive(move || {
        let observed = envelope.get();
        hub_context_available(
            active_context.get(),
            observed.as_ref(),
            loading.get(),
            error.get().is_some(),
        ) && observed
            .as_ref()
            .is_some_and(|source| hub_sample.get().source == HubSample::envelope_source(source))
    });

    #[cfg(target_arch = "wasm32")]
    if envelope.get_untracked().is_none() {
        request_render_envelope(
            initial_context,
            request_generation,
            set_envelope,
            workflow_sample,
            hub_sample,
            set_loading,
            set_error,
        );
    }

    // Bind navigation after the dashboard renders, including when reactive state
    // replaces its panels. Listener cleanup belongs to the island's lifetime.
    #[cfg(target_arch = "wasm32")]
    {
        let mount_generation = crate::mount_footprint::current_generation();
        on_cleanup(move || crate::mount_footprint::release(mount_generation, true));
        on_cleanup(detach_island_listeners);
        Effect::new(move |_| {
            let _ = (envelope.get(), active_surface.get(), hub_sample.get());
            // The render effect can run after this effect. Wait one microtask so
            // listener targets come from the settled dashboard DOM.
            leptos::task::spawn_local_scoped_with_cancellation(async {
                detach_island_listeners();
                wire_tablist_keyboard_navigation();
                wire_fragment_navigation();
                restore_fragment_destination();
            });
        });
        Effect::new(move |_| {
            let _ = (envelope.get(), loading.get(), error.get());
            leptos::task::spawn_local_scoped_with_cancellation(async move {
                // First settle the render, then the bridges queued by mutation delivery.
                leptos::task::tick().await;
                leptos::task::tick().await;
                let settled = crate::mount_footprint::replacement_settled(
                    loading.get_untracked(),
                    envelope.get_untracked().is_some(),
                    error.get_untracked().is_some(),
                );
                crate::mount_footprint::release(mount_generation, settled);
            });
        });
    }

    view! {
        <div
            class=move || if render_signal(loading) { "dashboard-island loading" } else { "dashboard-island" }
            data-island="render-envelope-dashboard"
            data-fragment-navigation=if cfg!(target_arch = "wasm32") { "island" } else { "ssr" }
            aria-live="polite"
            aria-busy=move || render_signal(loading).to_string()
        >
            <section
                class="context-switcher sample-context-selector island-frame"
                aria-labelledby="context-title"
                aria-describedby="sample-context-notice sample-context-status"
                data-sample-context-requested=move || render_signal(active_context).id()
                data-sample-context-displayed=move || render_signal(envelope).map(|source| source.context.id()).unwrap_or("none")
            >
                <div>
                    <h2 id="context-title">"Anonymous sample context"</h2>
                    <p id="sample-context-notice" class="sample-context-notice">{sample_context_notice_text()}</p>
                </div>
                <div class="context-grid" role="group" aria-label="Anonymous sample contexts">
                    {OperatorContext::ALL.into_iter().map(|context| view! {
                        <button
                            type="button"
                            data-sample-context=context.id()
                            disabled=!cfg!(target_arch = "wasm32")
                            class=move || if render_signal(active_context) == context { "context-card selected" } else { "context-card" }
                            aria-pressed=move || (render_signal(active_context) == context).to_string()
                            on:click=move |_| {
                                set_active_context.set(context);
                                request_render_envelope(
                                    context,
                                    request_generation,
                                    set_envelope,
                                    workflow_sample,
                                    hub_sample,
                                    set_loading,
                                    set_error,
                                );
                                set_active_surface.set(ProductSurface::Workflow);

                            }
                        >
                            <span class="context-icon" aria-hidden="true">{context_icon(context)}</span>
                            <span class="context-label">{context.label()}" sample"</span>
                        </button>
                    }).collect_view()}
                </div>
                <p id="sample-context-status" class="sample-context-status" role="status">
                    {move || sample_context_status(
                        render_signal(active_context),
                        render_signal(envelope).as_ref(),
                        render_signal(loading),
                        render_signal(error).is_some(),
                        cfg!(target_arch = "wasm32"),
                    )}
                </p>
            </section>

            {move || render_signal(error).map(|message| view! {
                <p class="fetch-error" role="alert">{message}</p>
            })}

            {move || match render_signal(envelope) {
                Some(envelope) => dashboard_view(
                    envelope,
                    workflow_sample,
                    active_surface,
                    set_active_surface,
                    hub_sample,
                    hub_available,
                ).into_any(),
                None => loading_state(render_signal(loading)).into_any(),
            }}
        </div>
    }
}

pub(super) fn sample_context_notice_text() -> &'static str {
    "Anonymous sample data. No tenant or role is verified; live service status is not queried."
}

pub(super) fn sample_context_status(
    requested: OperatorContext,
    observed: Option<&TenantRenderEnvelope>,
    loading: bool,
    failed: bool,
    bound: bool,
) -> String {
    let displayed = observed.map_or_else(
        || "No sample is displayed.".to_string(),
        |source| {
            let retained = if loading || failed {
                "Retained sample displayed"
            } else {
                "Sample displayed"
            };
            format!(
                "{retained}: {} · {} ({}).",
                source.tenant_name,
                source.role_name,
                source.context.label()
            )
        },
    );
    let state = if !bound {
        "Sample switching becomes available when the interactive preview loads."
    } else if loading {
        "Loading the requested sample."
    } else if failed {
        "Could not load the requested sample. Select a sample again to retry."
    } else if observed.is_none() {
        "The requested sample is unavailable. Select a sample again to retry."
    } else if observed.is_some_and(|source| source.context != requested) {
        "The returned sample differs from the request; Work Hub editing is unavailable. Select a sample again to retry."
    } else {
        "Select the current sample again to refresh."
    };
    format!(
        "Requested sample: {}. {displayed} {state}",
        requested.label()
    )
}

pub(super) fn loading_state(loading: bool) -> impl IntoView {
    let heading = if loading {
        "Loading anonymous sample"
    } else {
        "Anonymous sample unavailable"
    };
    view! {
        <section class="panel loading-panel" aria-label=heading>
            <p class="eyebrow">"Anonymous preview"</p>
            <h2>{heading}</h2>
            <p>{sample_context_notice_text()}</p>
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

#[cfg(test)]
#[path = "dashboard_test.rs"]
mod tests;
