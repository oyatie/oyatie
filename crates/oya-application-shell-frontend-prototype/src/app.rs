use leptos::prelude::*;

#[cfg(any(feature = "ssr", test))]
use crate::render_envelope::server_derived_envelope;
use crate::render_envelope::{
    ApprovalItem, CommunityItem, DemoContext, IntelligenceSuggestion, MessageItem, MetricCard,
    ModuleCard, OntologyFact, ScheduleItem, TenantRenderEnvelope, WorkItem, WorkflowNode,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProductSurface {
    Workflow,
    Messenger,
    Mail,
    Community,
}

impl ProductSurface {
    const ALL: [Self; 4] = [Self::Workflow, Self::Messenger, Self::Mail, Self::Community];

    const fn label(self) -> &'static str {
        match self {
            Self::Workflow => "Workflow Studio",
            Self::Messenger => "Messenger",
            Self::Mail => "Mail",
            Self::Community => "Community",
        }
    }

    const fn summary(self) -> &'static str {
        match self {
            Self::Workflow => "Build governed no-code flows",
            Self::Messenger => "Discuss operational threads",
            Self::Mail => "Draft formal work messages",
            Self::Community => "Coordinate role-aware spaces",
        }
    }

    const fn href(self) -> &'static str {
        match self {
            Self::Workflow => "#workflow-studio",
            Self::Messenger | Self::Mail | Self::Community => "#work-hub",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkflowTool {
    Select,
    Connect,
    Simulate,
}

impl WorkflowTool {
    const ALL: [Self; 3] = [Self::Select, Self::Connect, Self::Simulate];

    const fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Connect => "Connect",
            Self::Simulate => "Simulate",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalDraft {
    surface: ProductSurface,
    title: String,
    body: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HubItem {
    surface: ProductSurface,
    source: String,
    title: String,
    body: String,
    meta: String,
}

pub fn prototype_notice_text() -> &'static str {
    "Prototype/demo only: mock Leptos shell, no backend, no real auth, no PHI/PII, and no workflow execution."
}

pub fn shell_landmark_label() -> &'static str {
    "Oyatie Cloud/Tenant Control Center"
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div class="oya-prototype-app">
            <a class="skip-link" href="#prototype-shell">"Skip to dashboard"</a>
            <ShellRail />
            <ShellHeader />
            <main
                id="prototype-shell"
                class="control-center"
                aria-labelledby="prototype-title"
                aria-describedby="prototype-notice"
            >
                <HeroPanel />
                <DashboardIsland />
            </main>
        </div>
    }
}

#[component]
fn ShellRail() -> impl IntoView {
    view! {
        <aside class="app-rail" aria-label="Product navigation">
            <div class="rail-brand">
                <span class="rail-mark" aria-hidden="true">"O"</span>
                <div>
                    <strong>"Oyatie"</strong>
                    <span>"Control Center"</span>
                </div>
                <code>"v0.1"</code>
            </div>
            <p class="rail-group">"Run the tenant"</p>
            <a class="rail-nav active" href="#prototype-shell"><span aria-hidden="true">"⌂"</span>"Command center"<em>"12"</em></a>
            <a class="rail-nav" href="#work-hub"><span aria-hidden="true">"✉"</span>"Messenger · Mail"<em>"18"</em></a>
            <a class="rail-nav" href="#workflow-studio"><span aria-hidden="true">"⌘"</span>"Workflow Studio"</a>
            <p class="rail-group">"Operate"</p>
            <a class="rail-nav" href="#modules-title"><span aria-hidden="true">"▦"</span>"Service catalog"</a>
            <a class="rail-nav" href="#tasks-title"><span aria-hidden="true">"☑"</span>"Action inbox"<em>"5"</em></a>
            <a class="rail-nav" href="#schedule-title"><span aria-hidden="true">"◷"</span>"Schedule"</a>
            <p class="rail-group">"Trust"</p>
            <a class="rail-nav" href="#ontology-title"><span aria-hidden="true">"◎"</span>"Object graph"</a>
            <a class="rail-nav" href="#intelligence-title"><span aria-hidden="true">"✦"</span>"Copilot rail"</a>
            <div class="workspace-switch">
                <span class="workspace-avatar" aria-hidden="true">"N"</span>
                <div>
                    <strong>"Northwind"</strong>
                    <span>"Enterprise · US/EU/KR"</span>
                </div>
            </div>
        </aside>
    }
}

#[component]
fn ShellHeader() -> impl IntoView {
    view! {
        <header class="app-header" role="banner">
            <div class="top-breadcrumb" aria-label="Breadcrumb">
                <span>"Oyatie Cloud"</span>
                <span class="sep">"/"</span>
                <span>"Operations"</span>
                <span class="sep">"/"</span>
                <strong>"Control Center"</strong>
            </div>
            <button class="command-trigger" type="button" data-command-trigger="true" aria-haspopup="dialog">
                <span aria-hidden="true">"⌕"</span>
                <span>"Search actions, objects, workflows"</span>
                <kbd>"⌘K"</kbd>
            </button>
            <div class="header-actions" aria-label="Prototype status">
                <button type="button" class="header-status">"SSR shell"</button>
                <button type="button" class="header-status muted">"Selective WASM islands"</button>
                <button type="button" class="header-icon" aria-label="Open notifications">"◔"</button>
                <button type="button" class="header-icon" aria-label="Open settings">"⚙"</button>
            </div>
        </header>
    }
}

#[component]
fn HeroPanel() -> impl IntoView {
    view! {
        <section class="hero-panel" aria-labelledby="prototype-title">
            <div class="page-title-copy">
                <p class="screen-anchor">"01 / Command Center"</p>
                <h1 id="prototype-title">{shell_landmark_label()}</h1>
                <p id="prototype-notice" class="demo-notice" role="note">{prototype_notice_text()}</p>
            </div>
            <div class="hero-copy page-actions">
                <button type="button">"New action"</button>
                <button type="button" class="primary">"Close May →"</button>
            </div>
        </section>
    }
}

#[component]
pub fn DashboardIsland() -> impl IntoView {
    let initial_envelope = initial_envelope();
    let initial_node_id = initial_envelope
        .as_ref()
        .and_then(|envelope| envelope.workflow.nodes.first())
        .map(|node| node.id.clone())
        .unwrap_or_default();

    let (active_context, set_active_context) = signal(DemoContext::TenantAdmin);
    let (selected_node_id, set_selected_node_id) = signal(initial_node_id);
    let (envelope, set_envelope) = signal(initial_envelope);
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
            DemoContext::TenantAdmin,
            set_envelope,
            set_selected_node_id,
            set_loading,
            set_error,
        );
    }

    view! {
        <div
            class=move || if loading.get() { "dashboard-island loading" } else { "dashboard-island" }
            data-island="render-envelope-dashboard"
            aria-live="polite"
            aria-busy=move || loading.get()
        >
            <section class="context-switcher island-frame" aria-labelledby="context-title">
                <div>
                    <p class="eyebrow">"Context"</p>
                    <h2 id="context-title">"Switch render envelope"</h2>
                    <span class="island-label">"interactive island"</span>
                </div>
                <div class="context-grid" role="list" aria-label="Demo tenant and role contexts">
                    {DemoContext::ALL.into_iter().map(|context| view! {
                        <button
                            type="button"
                            class=move || if active_context.get() == context { "context-card selected" } else { "context-card" }
                            aria-pressed=move || active_context.get() == context
                            on:click=move |_| {
                                set_active_context.set(context);
                                request_render_envelope(
                                    context,
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

            {move || error.get().map(|message| view! {
                <p class="fetch-error" role="alert">{message}</p>
            })}

            {move || match envelope.get() {
                Some(envelope) => dashboard_view(
                    envelope,
                    selected_node_id.get(),
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

fn initial_envelope() -> Option<TenantRenderEnvelope> {
    #[cfg(any(feature = "ssr", test))]
    {
        Some(server_derived_envelope(DemoContext::TenantAdmin))
    }

    #[cfg(not(any(feature = "ssr", test)))]
    {
        None
    }
}

#[cfg(target_arch = "wasm32")]
fn request_render_envelope(
    context: DemoContext,
    set_envelope: WriteSignal<Option<TenantRenderEnvelope>>,
    set_selected_node_id: WriteSignal<String>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::{JsFuture, spawn_local};

    set_loading.set(true);
    set_error.set(None);

    spawn_local(async move {
        let context_id = context.id().to_string();
        let result = async {
            let window = web_sys::window().ok_or_else(|| "window unavailable".to_string())?;
            let response_value = JsFuture::from(
                window.fetch_with_str(&format!("/api/render-envelope/{context_id}")),
            )
            .await
            .map_err(|_| "render-envelope request failed".to_string())?;
            let response = response_value
                .dyn_into::<web_sys::Response>()
                .map_err(|_| "render-envelope response was invalid".to_string())?;

            if !response.ok() {
                return Err(format!(
                    "render-envelope request returned HTTP {}",
                    response.status()
                ));
            }

            let text_promise = response
                .text()
                .map_err(|_| "render-envelope body was unavailable".to_string())?;
            let text_value = JsFuture::from(text_promise)
                .await
                .map_err(|_| "render-envelope body could not be read".to_string())?;
            let text = text_value
                .as_string()
                .ok_or_else(|| "render-envelope body was not text".to_string())?;

            serde_json::from_str::<TenantRenderEnvelope>(&text)
                .map_err(|error| format!("render-envelope JSON was invalid: {error}"))
        }
        .await;

        match result {
            Ok(envelope) => {
                let node_id = envelope
                    .workflow
                    .nodes
                    .first()
                    .map(|node| node.id.clone())
                    .unwrap_or_default();
                set_selected_node_id.set(node_id);
                set_envelope.set(Some(envelope));
            }
            Err(message) => set_error.set(Some(message)),
        }

        set_loading.set(false);
    });
}

#[cfg(all(not(target_arch = "wasm32"), any(feature = "ssr", test)))]
fn request_render_envelope(
    context: DemoContext,
    set_envelope: WriteSignal<Option<TenantRenderEnvelope>>,
    set_selected_node_id: WriteSignal<String>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    set_loading.set(true);
    set_error.set(None);

    let envelope = server_derived_envelope(context);
    let node_id = envelope
        .workflow
        .nodes
        .first()
        .map(|node| node.id.clone())
        .unwrap_or_default();
    set_selected_node_id.set(node_id);
    set_envelope.set(Some(envelope));
    set_loading.set(false);
}

#[cfg(all(not(target_arch = "wasm32"), not(any(feature = "ssr", test))))]
fn request_render_envelope(
    _context: DemoContext,
    _set_envelope: WriteSignal<Option<TenantRenderEnvelope>>,
    _set_selected_node_id: WriteSignal<String>,
    _set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    set_error.set(Some(
        "render envelope refresh requires the SSR dev server or the WASM island fetch path"
            .to_string(),
    ));
}

#[expect(
    clippy::too_many_arguments,
    reason = "Leptos prototype view composes several reactive signals at the island boundary; refactoring into state bags would obscure the explicit demo data flow."
)]
fn dashboard_view(
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
    let display_nodes = workflow_display_nodes(&envelope.workflow.nodes, draft_node_count.get());
    let selected_node = selected_workflow_node(&display_nodes, &selected_node_id)
        .cloned()
        .or_else(|| display_nodes.first().cloned());

    view! {
        {surface_command_bar(active_surface, set_active_surface)}

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

        <section class="dashboard-grid" aria-label="Personalized dashboard">
            <section class="panel daily-panel" aria-labelledby="tasks-title">
                <PanelHeader eyebrow="Daily work" title={"Tasks and approvals".to_string()} />
                <div class="work-columns">
                    {work_list(envelope.daily_tasks.clone())}
                    {approval_list(envelope.approvals.clone())}
                </div>
            </section>

            <section class="panel schedule-panel" aria-labelledby="schedule-title">
                <PanelHeader eyebrow="Calendar" title={"Today’s schedule".to_string()} />
                {schedule_list(envelope.schedule.clone())}
            </section>

            <section id="work-hub" class="panel communications-panel" aria-labelledby="messages-title">
                <PanelHeader eyebrow="Messenger · Mail · Community" title={"Work hub".to_string()} />
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

            <section class="panel modules-panel" aria-labelledby="modules-title">
                <PanelHeader eyebrow="Modules" title={"Permitted service catalog".to_string()} />
                {module_grid(envelope.modules.clone())}
                <p class="omitted-note">{envelope.omitted_capability_note.clone()}</p>
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

            <section class="panel" aria-labelledby="ontology-title">
                <PanelHeader eyebrow="Ontology" title={"Why this user can see it".to_string()} />
                {ontology_list(envelope.ontology.clone())}
            </section>

            <section class="panel" aria-labelledby="intelligence-title">
                <PanelHeader eyebrow="Intelligence" title={"Advisory assistant".to_string()} />
                {suggestion_list(envelope.intelligence.clone())}
            </section>
        </section>
    }
}

fn loading_state() -> impl IntoView {
    view! {
        <section class="panel loading-panel" aria-label="Loading permitted dashboard">
            <p class="eyebrow">"Server render envelope"</p>
            <h2>"Loading permitted dashboard"</h2>
            <p>"Fetching only the modules and workflow state allowed for this tenant and role."</p>
        </section>
    }
}

#[component]
fn PanelHeader(eyebrow: &'static str, title: String) -> impl IntoView {
    view! {
        <div class="panel-header">
            <p class="eyebrow">{eyebrow}</p>
            <h3>{title}</h3>
        </div>
    }
}

fn surface_command_bar(
    active_surface: ReadSignal<ProductSurface>,
    set_active_surface: WriteSignal<ProductSurface>,
) -> impl IntoView {
    view! {
        <nav class="surface-command-bar" aria-label="Open built-in product surface">
            {ProductSurface::ALL.into_iter().map(|surface| view! {
                <a
                    href=surface.href()
                    class=move || {
                        if active_surface.get() == surface {
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

fn workflow_display_nodes(nodes: &[WorkflowNode], draft_node_count: usize) -> Vec<WorkflowNode> {
    let mut display_nodes = nodes.to_vec();
    for index in 0..draft_node_count {
        display_nodes.push(WorkflowNode {
            id: format!("draft-block-{}", index + 1),
            label: format!("Draft block {}", index + 1),
            kind: "Local".to_string(),
            x: 110 + ((index as i32 % 4) * 165),
            y: 164 + ((index as i32 / 4) * 74),
            explanation: "Local visual-only block added in the prototype. It is not wired to a backend or workflow engine.".to_string(),
        });
    }
    display_nodes
}

fn metric_grid(metrics: Vec<MetricCard>) -> impl IntoView {
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

fn work_list(items: Vec<WorkItem>) -> impl IntoView {
    view! {
        <div>
            <h4 id="tasks-title">"Task queue"</h4>
            <ul class="item-list">
                {items.into_iter().map(|item| view! {
                    <li>
                        <span class="priority">{item.priority}</span>
                        <strong>{item.title}</strong>
                        <p>{item.detail}</p>
                    </li>
                }).collect_view()}
            </ul>
        </div>
    }
}

fn approval_list(items: Vec<ApprovalItem>) -> impl IntoView {
    view! {
        <div>
            <h4>"Approval queue"</h4>
            <ul class="item-list compact">
                {items.into_iter().map(|item| view! {
                    <li>
                        <strong>{item.title}</strong>
                        <p>{item.requester}</p>
                        <span>{item.risk_note}</span>
                    </li>
                }).collect_view()}
            </ul>
        </div>
    }
}

fn schedule_list(items: Vec<ScheduleItem>) -> impl IntoView {
    view! {
        <ol class="timeline-list">
            {items.into_iter().map(|item| view! {
                <li>
                    <time>{item.time}</time>
                    <div>
                        <strong>{item.title}</strong>
                        <p>{item.detail}</p>
                    </div>
                </li>
            }).collect_view()}
        </ol>
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Prototype communication surface keeps signal ownership explicit so mock-only local drafts cannot be confused with server persistence."
)]
fn communication_hub(
    messages: Vec<MessageItem>,
    communities: Vec<CommunityItem>,
    active_surface: ReadSignal<ProductSurface>,
    set_active_surface: WriteSignal<ProductSurface>,
    selected_hub_index: ReadSignal<usize>,
    set_selected_hub_index: WriteSignal<usize>,
    draft_body: ReadSignal<String>,
    set_draft_body: WriteSignal<String>,
    local_drafts: ReadSignal<Vec<LocalDraft>>,
    set_local_drafts: WriteSignal<Vec<LocalDraft>>,
) -> impl IntoView {
    let tabs = [
        ProductSurface::Messenger,
        ProductSurface::Mail,
        ProductSurface::Community,
    ];
    let list_messages = messages.clone();
    let list_communities = communities.clone();
    let detail_messages = messages.clone();
    let detail_communities = communities.clone();

    view! {
        <div class="communications-hub interactive-hub">
            <div class="hub-tabs" role="tablist" aria-label="Work hub channels">
                {tabs.into_iter().map(|surface| view! {
                    <button
                        type="button"
                        role="tab"
                        aria-selected=move || active_surface.get() == surface
                        class=move || if active_surface.get() == surface { "hub-tab active" } else { "hub-tab" }
                        on:click=move |_| {
                            set_active_surface.set(surface);
                            set_selected_hub_index.set(0);
                        }
                    >
                        {surface.label()}
                    </button>
                }).collect_view()}
            </div>

            <div class="hub-workspace">
                <div class="hub-list" role="list" aria-label="Channel items">
                    {move || {
                        let items = hub_items(
                            &list_messages,
                            &list_communities,
                            &local_drafts.get(),
                            active_surface.get(),
                        );
                        let active_index = selected_hub_index.get();
                        items.into_iter().enumerate().map(|(index, item)| view! {
                            <button
                                type="button"
                                class=if index == active_index { "hub-item active" } else { "hub-item" }
                                on:click=move |_| set_selected_hub_index.set(index)
                            >
                                <span>{item.source}</span>
                                <strong>{item.title}</strong>
                                <p>{item.body}</p>
                            </button>
                        }).collect_view()
                    }}
                </div>

                <div class="hub-detail" aria-live="polite">
                    {move || {
                        let items = hub_items(
                            &detail_messages,
                            &detail_communities,
                            &local_drafts.get(),
                            active_surface.get(),
                        );
                        match selected_hub_item(&items, selected_hub_index.get()) {
                            Some(item) => view! {
                                <article>
                                    <p class="eyebrow">{item.surface.label()}</p>
                                    <h4>{item.title}</h4>
                                    <p>{item.body}</p>
                                    <span class="hub-meta">{item.meta}</span>
                                </article>
                            }.into_any(),
                            None => view! {
                                <article>
                                    <p class="eyebrow">"Empty channel"</p>
                                    <h4>"No visible items"</h4>
                                    <p>"This permitted envelope has no items for the selected channel."</p>
                                </article>
                            }.into_any(),
                        }
                    }}

                    <div class="hub-composer" aria-label="Local prototype composer">
                        <label for="hub-composer-input">"Draft a local prototype response"</label>
                        <textarea
                            id="hub-composer-input"
                            rows="3"
                            prop:value=move || draft_body.get()
                            placeholder="Type here; Queue draft keeps it local to this browser island."
                            on:input=move |event| set_draft_body.set(event_target_value(&event))
                        ></textarea>
                        <div class="composer-actions">
                            <button
                                type="button"
                                on:click=move |_| {
                                    let body = draft_body.get().trim().to_string();
                                    if body.is_empty() {
                                        return;
                                    }
                                    let mut drafts = local_drafts.get();
                                    drafts.insert(0, LocalDraft {
                                        surface: active_surface.get(),
                                        title: "Local draft queued".to_string(),
                                        body,
                                    });
                                    set_local_drafts.set(drafts);
                                    set_draft_body.set(String::new());
                                    set_selected_hub_index.set(0);
                                }
                            >
                                "Queue draft"
                            </button>
                            <button
                                type="button"
                                class="secondary"
                                on:click=move |_| set_draft_body.set(String::new())
                            >
                                "Clear"
                            </button>
                        </div>
                        <p>"Visual-only: drafts, sends, posts, and replies stay in local island state."</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn hub_items(
    messages: &[MessageItem],
    communities: &[CommunityItem],
    drafts: &[LocalDraft],
    surface: ProductSurface,
) -> Vec<HubItem> {
    let mut items = drafts
        .iter()
        .filter(|draft| draft.surface == surface)
        .map(|draft| HubItem {
            surface,
            source: "Local draft".to_string(),
            title: draft.title.clone(),
            body: draft.body.clone(),
            meta: "Stored in WASM island state only".to_string(),
        })
        .collect::<Vec<_>>();

    match surface {
        ProductSurface::Messenger => {
            items.extend(
                messages
                    .iter()
                    .filter(|item| !item.channel.to_ascii_lowercase().contains("mail"))
                    .map(|item| HubItem {
                        surface,
                        source: item.channel.clone(),
                        title: item.from.clone(),
                        body: item.preview.clone(),
                        meta: "Thread preview; no external message sent".to_string(),
                    }),
            );
        }
        ProductSurface::Mail => {
            items.extend(
                messages
                    .iter()
                    .filter(|item| item.channel.to_ascii_lowercase().contains("mail"))
                    .map(|item| HubItem {
                        surface,
                        source: item.channel.clone(),
                        title: item.from.clone(),
                        body: item.preview.clone(),
                        meta: "Mail preview; compose is local only".to_string(),
                    }),
            );
        }
        ProductSurface::Community => {
            items.extend(communities.iter().map(|item| HubItem {
                surface,
                source: item.space.clone(),
                title: item.topic.clone(),
                body: item.activity.clone(),
                meta: "Community post preview; no backend write".to_string(),
            }));
        }
        ProductSurface::Workflow => {}
    }

    items
}

fn selected_hub_item(items: &[HubItem], selected_index: usize) -> Option<HubItem> {
    items.get(selected_index).or_else(|| items.first()).cloned()
}

fn module_grid(modules: Vec<ModuleCard>) -> impl IntoView {
    view! {
        <div class="module-grid">
            {modules.into_iter().map(|module| view! {
                <article class="module-card">
                    <span>{module.group}</span>
                    <h4>{module.name}</h4>
                    <p>{module.description}</p>
                    <button type="button">{module.action_label}</button>
                </article>
            }).collect_view()}
        </div>
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Prototype workflow panel keeps each reactive control explicit pending a production component boundary."
)]
fn workflow_studio_panel(
    name: String,
    goal: String,
    nodes: Vec<WorkflowNode>,
    selected_node: Option<WorkflowNode>,
    set_selected_node_id: WriteSignal<String>,
    workflow_tool: ReadSignal<WorkflowTool>,
    set_workflow_tool: WriteSignal<WorkflowTool>,
    draft_node_count: ReadSignal<usize>,
    set_draft_node_count: WriteSignal<usize>,
    set_active_surface: WriteSignal<ProductSurface>,
) -> impl IntoView {
    view! {
        <section id="workflow-studio" class="panel workflow-panel cohesive-workflow" aria-labelledby="workflow-title">
            <div class="workflow-topbar">
                <div>
                    <p class="eyebrow">"Workflow Studio"</p>
                    <h3 id="workflow-title">{name}</h3>
                </div>
                <div class="workflow-run-chip" aria-label="Mock run state">
                    <span></span>
                    {move || match workflow_tool.get() {
                        WorkflowTool::Select => "draft · select mode",
                        WorkflowTool::Connect => "draft · connect mode",
                        WorkflowTool::Simulate => "simulation preview",
                    }}
                </div>
                <div class="workflow-actions" aria-label="Workflow mock actions">
                    <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Select)>"Validate"</button>
                    <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Simulate)>"Preview run"</button>
                    <button type="button" on:click=move |_| set_draft_node_count.set(draft_node_count.get() + 1)>"Add block"</button>
                </div>
            </div>
            <p class="panel-intro">{goal}</p>

            <div class="workflow-modebar" role="toolbar" aria-label="Workflow editor modes">
                {WorkflowTool::ALL.into_iter().map(|tool| view! {
                    <button
                        type="button"
                        class=move || if workflow_tool.get() == tool { "active" } else { "" }
                        aria-pressed=move || workflow_tool.get() == tool
                        on:click=move |_| set_workflow_tool.set(tool)
                    >
                        {tool.label()}
                    </button>
                }).collect_view()}
            </div>

            <div class="workflow-ide">
                <aside class="workflow-palette" aria-label="Workflow building blocks">
                    <h4>"Blocks"</h4>
                    {["Trigger", "Policy check", "Approval", "Evidence note"].into_iter().map(|label| view! {
                        <button
                            type="button"
                            on:click=move |_| set_draft_node_count.set(draft_node_count.get() + 1)
                        >
                            {label}
                        </button>
                    }).collect_view()}
                    <h4>"Built-in surfaces"</h4>
                    <button type="button" on:click=move |_| set_active_surface.set(ProductSurface::Messenger)>"Messenger post"</button>
                    <button type="button" on:click=move |_| set_active_surface.set(ProductSurface::Mail)>"Mail draft"</button>
                    <button type="button" on:click=move |_| set_active_surface.set(ProductSurface::Community)>"Community note"</button>
                </aside>

                {workflow_canvas(nodes.clone(), set_selected_node_id, workflow_tool, set_workflow_tool)}

                <aside class="workflow-inspector" aria-label="Selected workflow node inspector">
                    <div class="inspector-tabs" aria-hidden="true">
                        <span class="active">"Inspector"</span>
                        <span>"Run log"</span>
                    </div>
                    {selected_node_view(selected_node)}
                    <dl class="inspector-fields">
                        <div><dt>"Guardrail"</dt><dd>"Human review before action"</dd></div>
                        <div><dt>"Output"</dt><dd>"Task · message · evidence draft"</dd></div>
                        <div><dt>"Execution"</dt><dd>"Disabled in prototype"</dd></div>
                    </dl>
                </aside>
            </div>

            <div class="workflow-statusbar" aria-label="Workflow editor status">
                <span>{move || format!("Nodes: {}", nodes.len())}</span>
                <span>{move || format!("Local blocks: {}", draft_node_count.get())}</span>
                <span>"Messenger/Mail/Community outputs are drafts"</span>
                <span>{move || match workflow_tool.get() {
                    WorkflowTool::Select => "Ready · mock",
                    WorkflowTool::Connect => "Click nodes to visualize links",
                    WorkflowTool::Simulate => "Previewing run path only",
                }}</span>
            </div>
        </section>
    }
}

fn workflow_canvas(
    nodes: Vec<WorkflowNode>,
    set_selected_node_id: WriteSignal<String>,
    workflow_tool: ReadSignal<WorkflowTool>,
    set_workflow_tool: WriteSignal<WorkflowTool>,
) -> impl IntoView {
    let svg_nodes = nodes.clone();

    view! {
        <div class="workflow-canvas island-frame" role="img" aria-label="Interactive mock workflow canvas">
            <div class="workflow-toolbar" aria-label="Workflow studio tools">
                <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Select)>"Select"</button>
                <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Connect)>"Connect"</button>
                <button type="button" on:click=move |_| set_workflow_tool.set(WorkflowTool::Simulate)>"Simulate"</button>
                <span class="island-label">"interactive island"</span>
            </div>
            <svg viewBox="0 0 820 310" aria-hidden="true" focusable="false">
                <defs>
                    <marker id="workflow-arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
                        <path d="M0,0 L0,6 L9,3 z" class="workflow-arrow" />
                    </marker>
                </defs>
                <line x1="140" y1="120" x2="690" y2="120" class="workflow-edge" />
                {svg_nodes.into_iter().map(|node| view! {
                    <g
                        class=move || match workflow_tool.get() {
                            WorkflowTool::Select => "workflow-node-group selectable",
                            WorkflowTool::Connect => "workflow-node-group connectable",
                            WorkflowTool::Simulate => "workflow-node-group simulating",
                        }
                        on:click={
                            let id = node.id.clone();
                            move |_| set_selected_node_id.set(id.clone())
                        }
                    >
                        <rect x=node.x y=node.y width="130" height="56" rx="10" class="workflow-node" />
                        <circle cx={node.x + 8} cy={node.y + 28} r="4" class="port in" />
                        <circle cx={node.x + 122} cy={node.y + 28} r="4" class="port out" />
                        <text x={node.x + 16} y={node.y + 24}>{node.label.clone()}</text>
                        <text x={node.x + 16} y={node.y + 43} class="node-kind">{node.kind.clone()}</text>
                    </g>
                }).collect_view()}
            </svg>
            <div class="canvas-footer">
                <div class="zoom-controls" aria-label="Visual zoom controls">
                    <button type="button">"−"</button>
                    <span>"82%"</span>
                    <button type="button">"+"</button>
                </div>
                <div class="mini-map" aria-hidden="true">
                    <span></span><span></span><span></span><span></span>
                </div>
            </div>
            <div class="node-toolbar" aria-label="Select workflow node to inspect">
                {nodes.into_iter().map(|node| {
                    let id = node.id.clone();
                    view! {
                        <button type="button" on:click=move |_| set_selected_node_id.set(id.clone())>
                            {node.label}
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

fn selected_node_view(node: Option<WorkflowNode>) -> impl IntoView {
    match node {
        Some(node) => view! {
            <aside class="node-inspector" aria-live="polite">
                <p class="eyebrow">"Selected node"</p>
                <h4>{node.label}</h4>
                <p><strong>{node.kind}</strong>" · "{node.explanation}</p>
            </aside>
        }
        .into_any(),
        None => view! {
            <aside class="node-inspector" aria-live="polite">
                <p>"Select a node to inspect its workflow, ontology, and guardrail meaning."</p>
            </aside>
        }
        .into_any(),
    }
}

fn ontology_list(items: Vec<OntologyFact>) -> impl IntoView {
    view! {
        <ul class="ontology-list">
            {items.into_iter().map(|item| view! {
                <li>
                    <strong>{item.entity}</strong>
                    <span>{item.relation}</span>
                    <p>{item.access_reason}</p>
                </li>
            }).collect_view()}
        </ul>
    }
}

fn suggestion_list(items: Vec<IntelligenceSuggestion>) -> impl IntoView {
    view! {
        <ul class="suggestion-list">
            {items.into_iter().map(|item| view! {
                <li>
                    <strong>{item.title}</strong>
                    <p>{item.body}</p>
                    <span>{item.guardrail}</span>
                </li>
            }).collect_view()}
        </ul>
    }
}

fn selected_workflow_node<'a>(
    nodes: &'a [WorkflowNode],
    selected_node_id: &str,
) -> Option<&'a WorkflowNode> {
    nodes.iter().find(|node| node.id == selected_node_id)
}

fn context_icon(context: DemoContext) -> &'static str {
    match context {
        DemoContext::TenantAdmin => "◇",
        DemoContext::CorporateOffice => "▣",
        DemoContext::HealthcareClinician => "✚",
    }
}

#[cfg(any(feature = "ssr", test))]
pub fn render_envelope_json(context_id: &str) -> Option<String> {
    DemoContext::from_id(context_id)
        .and_then(|context| serde_json::to_string(&server_derived_envelope(context)).ok())
}

#[cfg(any(feature = "ssr", test))]
pub fn static_dashboard_html() -> String {
    let envelope = server_derived_envelope(DemoContext::TenantAdmin);
    format!(
        r##"<div class="oya-prototype-app">
  <a class="skip-link" href="#prototype-shell">Skip to dashboard</a>
  {rail}
  {header}
  <main id="prototype-shell" class="control-center" aria-labelledby="prototype-title" aria-describedby="prototype-notice">
    {hero}
    <div id="oya-dashboard-island-root" class="dashboard-island" data-island="render-envelope-dashboard">
      {dashboard}
    </div>
  </main>
  {command_palette}
</div>"##,
        rail = static_rail_html(),
        header = static_header_html(),
        hero = static_hero_html(),
        dashboard = static_dashboard_content(&envelope),
        command_palette = static_command_palette_html(),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_rail_html() -> String {
    r##"<aside class="app-rail" aria-label="Product navigation">
    <div class="rail-brand"><span class="rail-mark" aria-hidden="true">O</span><div><strong>Oyatie</strong><span>Control Center</span></div><code>v0.1</code></div>
    <p class="rail-group">Run the tenant</p>
    <a class="rail-nav active" href="#prototype-shell"><span aria-hidden="true">⌂</span>Command center<em>12</em></a>
    <a class="rail-nav" href="#work-hub"><span aria-hidden="true">✉</span>Messenger · Mail<em>18</em></a>
    <a class="rail-nav" href="#workflow-studio"><span aria-hidden="true">⌘</span>Workflow Studio</a>
    <p class="rail-group">Operate</p>
    <a class="rail-nav" href="#modules-title"><span aria-hidden="true">▦</span>Service catalog</a>
    <a class="rail-nav" href="#tasks-title"><span aria-hidden="true">☑</span>Action inbox<em>5</em></a>
    <a class="rail-nav" href="#schedule-title"><span aria-hidden="true">◷</span>Schedule</a>
    <p class="rail-group">Trust</p>
    <a class="rail-nav" href="#ontology-title"><span aria-hidden="true">◎</span>Object graph</a>
    <a class="rail-nav" href="#intelligence-title"><span aria-hidden="true">✦</span>Copilot rail</a>
    <div class="workspace-switch"><span class="workspace-avatar" aria-hidden="true">N</span><div><strong>Northwind</strong><span>Enterprise · US/EU/KR</span></div></div>
  </aside>"##
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
fn static_header_html() -> String {
    r#"<header class="app-header" role="banner">
    <div class="top-breadcrumb" aria-label="Breadcrumb"><span>Oyatie Cloud</span><span class="sep">/</span><span>Operations</span><span class="sep">/</span><strong>Control Center</strong></div>
    <button class="command-trigger" type="button" data-command-trigger="true" aria-haspopup="dialog"><span aria-hidden="true">⌕</span><span>Search actions, objects, workflows</span><kbd>⌘K</kbd></button>
    <div class="header-actions" aria-label="Prototype status"><button type="button" class="header-status">SSR shell</button><button type="button" class="header-status muted">Selective WASM islands</button><button type="button" class="header-icon" aria-label="Open notifications">◔</button><button type="button" class="header-icon" aria-label="Open settings">⚙</button></div>
  </header>"#
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
fn static_hero_html() -> String {
    format!(
        r#"<section class="hero-panel" aria-labelledby="prototype-title"><div class="page-title-copy"><p class="screen-anchor">01 / Command Center</p><h1 id="prototype-title">{title}</h1><p id="prototype-notice" class="demo-notice" role="note">{notice}</p></div><div class="hero-copy page-actions"><button type="button">New action</button><button type="button" class="primary">Close May →</button></div></section>"#,
        title = escape(shell_landmark_label()),
        notice = escape(prototype_notice_text())
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_command_palette_html() -> String {
    r#"<div class="command-palette-backdrop" data-command-backdrop hidden>
    <section class="command-palette" role="dialog" aria-modal="true" aria-label="Command palette">
      <div class="command-input-row"><span aria-hidden="true">⌕</span><input aria-label="Search command palette" placeholder="Search actions, objects, workflows…" value="" /><kbd>ESC</kbd></div>
      <div class="command-results" role="listbox">
        <button type="button"><strong>Open Workflow Studio</strong><span>Build governed no-code flows</span><kbd>W</kbd></button>
        <button type="button"><strong>Compose mail</strong><span>Draft formal work messages locally</span><kbd>M</kbd></button>
        <button type="button"><strong>Post to community</strong><span>Coordinate role-aware spaces</span><kbd>C</kbd></button>
        <button type="button"><strong>Inspect audit chain</strong><span>Open object graph and evidence spine</span><kbd>A</kbd></button>
      </div>
    </section>
  </div>"#
        .to_string()
}

#[cfg(any(feature = "ssr", test))]
fn static_dashboard_content(envelope: &TenantRenderEnvelope) -> String {
    format!(
        r#"{context_switcher}
{surface_commands}
{envelope_banner}
<section class="metric-grid" aria-label="Dashboard metrics">{metrics}</section>
<section class="dashboard-grid" aria-label="Personalized dashboard"><section class="panel daily-panel"><div class="panel-header"><p class="eyebrow">Daily work</p><h3>Tasks and approvals</h3></div><div class="work-columns"><div><h4>Task queue</h4><ul class="item-list">{tasks}</ul></div><div><h4>Approval queue</h4><ul class="item-list compact">{approvals}</ul></div></div></section><section class="panel schedule-panel"><div class="panel-header"><p class="eyebrow">Calendar</p><h3>Today’s schedule</h3></div><ol class="timeline-list">{schedule}</ol></section><section id="work-hub" class="panel communications-panel"><div class="panel-header"><p class="eyebrow">Messenger · Mail · Community</p><h3>Work hub</h3></div>{communication_hub}</section><section class="panel modules-panel"><div class="panel-header"><p class="eyebrow">Modules</p><h3>Permitted service catalog</h3></div><div class="module-grid">{modules}</div><p class="omitted-note">{omitted}</p></section></section>
<section class="studio-grid" aria-label="Workflow, ontology, and intelligence">{workflow_studio}<section class="panel"><div class="panel-header"><p class="eyebrow">Ontology</p><h3>Why this user can see it</h3></div><ul class="ontology-list">{ontology}</ul></section><section class="panel"><div class="panel-header"><p class="eyebrow">Intelligence</p><h3>Advisory assistant</h3></div><ul class="suggestion-list">{suggestions}</ul></section></section>"#,
        context_switcher = static_context_switcher(envelope.context),
        surface_commands = static_surface_commands(),
        envelope_banner = static_envelope_banner(envelope),
        metrics = envelope
            .metrics
            .iter()
            .map(static_metric)
            .collect::<String>(),
        tasks = envelope
            .daily_tasks
            .iter()
            .map(static_task)
            .collect::<String>(),
        approvals = envelope
            .approvals
            .iter()
            .map(static_approval)
            .collect::<String>(),
        schedule = envelope
            .schedule
            .iter()
            .map(static_schedule)
            .collect::<String>(),
        communication_hub = static_communication_hub(&envelope.messages, &envelope.community),
        modules = envelope
            .modules
            .iter()
            .map(static_module)
            .collect::<String>(),
        omitted = escape(&envelope.omitted_capability_note),
        workflow_studio = static_workflow_studio_panel(envelope),
        ontology = envelope
            .ontology
            .iter()
            .map(static_ontology)
            .collect::<String>(),
        suggestions = envelope
            .intelligence
            .iter()
            .map(static_suggestion)
            .collect::<String>(),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_surface_commands() -> String {
    let commands = ProductSurface::ALL
        .iter()
        .map(|surface| {
            let class = if *surface == ProductSurface::Workflow {
                "surface-command active"
            } else {
                "surface-command"
            };
            format!(
                r#"<a class="{class}" href="{href}"><span>{label}</span><small>{summary}</small></a>"#,
                class = class,
                href = surface.href(),
                label = escape(surface.label()),
                summary = escape(surface.summary())
            )
        })
        .collect::<String>();

    format!(
        r#"<nav class="surface-command-bar" aria-label="Open built-in product surface">{commands}</nav>"#
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_communication_hub(messages: &[MessageItem], communities: &[CommunityItem]) -> String {
    let messenger = messages
        .iter()
        .filter(|message| !message.channel.to_ascii_lowercase().contains("mail"))
        .map(static_message)
        .collect::<String>();
    let mail = messages
        .iter()
        .filter(|message| message.channel.to_ascii_lowercase().contains("mail"))
        .map(static_message)
        .collect::<String>();
    let community = communities.iter().map(static_community).collect::<String>();

    format!(
        r#"<div class="communications-hub interactive-hub"><div class="hub-tabs" role="tablist" aria-label="Work hub channels"><button type="button" role="tab" aria-selected="true" class="hub-tab active">Messenger</button><button type="button" role="tab" aria-selected="false" class="hub-tab">Mail</button><button type="button" role="tab" aria-selected="false" class="hub-tab">Community</button></div><div class="hub-workspace"><div class="hub-list" role="list" aria-label="Channel items">{messenger}</div><div class="hub-detail"><article><p class="eyebrow">Messenger</p><h4>Local work hub</h4><p>Use the WASM island to switch channels, inspect items, and queue local drafts.</p><span class="hub-meta">Visual-only; no external send</span></article><div class="hub-composer"><label for="static-hub-composer">Draft a local prototype response</label><textarea id="static-hub-composer" rows="3" placeholder="Hydration enables local queueing."></textarea><div class="composer-actions"><button type="button">Queue draft</button><button type="button" class="secondary">Clear</button></div><p>Mail previews: {mail_count}. Community spaces: {community_count}.</p></div></div></div><template data-mail-preview="{mail}"></template><template data-community-preview="{community}"></template></div>"#,
        messenger = messenger,
        mail = escape(&mail),
        community = escape(&community),
        mail_count = messages
            .iter()
            .filter(|message| message.channel.to_ascii_lowercase().contains("mail"))
            .count(),
        community_count = communities.len(),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_workflow_studio_panel(envelope: &TenantRenderEnvelope) -> String {
    let nodes = envelope
        .workflow
        .nodes
        .iter()
        .map(|node| format!("<button type=\"button\">{}</button>", escape(&node.label)))
        .collect::<String>();
    let selected_node = envelope
        .workflow
        .nodes
        .first()
        .map(static_selected_node)
        .unwrap_or_default();

    format!(
        r#"<section id="workflow-studio" class="panel workflow-panel cohesive-workflow" aria-labelledby="workflow-title"><div class="workflow-topbar"><div><p class="eyebrow">Workflow Studio</p><h3 id="workflow-title">{name}</h3></div><div class="workflow-run-chip"><span></span>draft · select mode</div><div class="workflow-actions"><button type="button">Validate</button><button type="button">Preview run</button><button type="button">Add block</button></div></div><p class="panel-intro">{goal}</p><div class="workflow-modebar" role="toolbar" aria-label="Workflow editor modes"><button type="button" class="active">Select</button><button type="button">Connect</button><button type="button">Simulate</button></div><div class="workflow-ide"><aside class="workflow-palette" aria-label="Workflow building blocks"><h4>Blocks</h4><button type="button">Trigger</button><button type="button">Policy check</button><button type="button">Approval</button><button type="button">Evidence note</button><h4>Built-in surfaces</h4><button type="button">Messenger post</button><button type="button">Mail draft</button><button type="button">Community note</button></aside><div class="workflow-canvas island-frame"><div class="workflow-toolbar"><button type="button">Select</button><button type="button">Connect</button><button type="button">Simulate</button><span class="island-label">interactive island</span></div>{workflow_svg}<div class="canvas-footer"><div class="zoom-controls"><button type="button">−</button><span>82%</span><button type="button">+</button></div><div class="mini-map" aria-hidden="true"><span></span><span></span><span></span><span></span></div></div><div class="node-toolbar">{nodes}</div></div><aside class="workflow-inspector"><div class="inspector-tabs" aria-hidden="true"><span class="active">Inspector</span><span>Run log</span></div>{selected_node}<dl class="inspector-fields"><div><dt>Guardrail</dt><dd>Human review before action</dd></div><div><dt>Output</dt><dd>Task · message · evidence draft</dd></div><div><dt>Execution</dt><dd>Disabled in prototype</dd></div></dl></aside></div><div class="workflow-statusbar"><span>Nodes: {node_count}</span><span>Local blocks: 0</span><span>Messenger/Mail/Community outputs are drafts</span><span>Ready · mock</span></div></section>"#,
        name = escape(&envelope.workflow.name),
        goal = escape(&envelope.workflow.goal),
        workflow_svg = static_workflow_svg(&envelope.workflow.nodes),
        nodes = nodes,
        selected_node = selected_node,
        node_count = envelope.workflow.nodes.len(),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_context_switcher(active: DemoContext) -> String {
    let cards = DemoContext::ALL
        .iter()
        .map(|context| {
            let selected = if *context == active { " context-card selected" } else { " context-card" };
            let pressed = if *context == active { "true" } else { "false" };
            format!(
                "<button type=\"button\" class=\"{class}\" aria-pressed=\"{pressed}\"><span class=\"context-icon\" aria-hidden=\"true\">{icon}</span><span class=\"context-label\">{label}</span><span class=\"context-role\">{role}</span></button>",
                class = selected.trim(),
                pressed = pressed,
                icon = escape(context_icon(*context)),
                label = escape(context.label()),
                role = escape(context.role())
            )
        })
        .collect::<String>();

    format!(
        r#"<section class="context-switcher island-frame" aria-labelledby="context-title"><div><p class="eyebrow">Context</p><h2 id="context-title">Switch render envelope</h2><span class="island-label">interactive island</span></div><div class="context-grid" role="list" aria-label="Demo tenant and role contexts">{cards}</div></section>"#
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_envelope_banner(envelope: &TenantRenderEnvelope) -> String {
    let accreditation_class = if envelope.accreditation.healthcare_enabled {
        "badge success"
    } else {
        "badge warning"
    };

    format!(
        r#"<section class="envelope-banner" aria-labelledby="envelope-title"><div><p class="eyebrow">Active context</p><h2 id="envelope-title">{tenant}</h2><p>{role}</p></div><div class="envelope-detail"><span class="badge">{tenant_class}</span><span class="{accreditation_class}">{accreditation}</span><p>{derivation}</p></div></section>"#,
        tenant = escape(&envelope.tenant_name),
        role = escape(&envelope.role_name),
        tenant_class = escape(&envelope.tenant_class),
        accreditation_class = accreditation_class,
        accreditation = escape(&envelope.accreditation.label),
        derivation = escape(&envelope.server_derivation_note),
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_metric(metric: &MetricCard) -> String {
    format!(
        "<article class=\"metric-card\"><p>{}</p><strong>{}</strong><span>{}</span></article>",
        escape(&metric.label),
        escape(&metric.value),
        escape(&metric.detail)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_module(module: &ModuleCard) -> String {
    format!(
        "<article class=\"module-card\"><span>{}</span><h4>{}</h4><p>{}</p><button type=\"button\">{}</button></article>",
        escape(&module.group),
        escape(&module.name),
        escape(&module.description),
        escape(&module.action_label)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_task(task: &WorkItem) -> String {
    format!(
        "<li><span class=\"priority\">{}</span><strong>{}</strong><p>{}</p></li>",
        escape(&task.priority),
        escape(&task.title),
        escape(&task.detail)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_approval(approval: &ApprovalItem) -> String {
    format!(
        "<li><strong>{}</strong><p>{}</p><span>{}</span></li>",
        escape(&approval.title),
        escape(&approval.requester),
        escape(&approval.risk_note)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_schedule(item: &ScheduleItem) -> String {
    format!(
        "<li><time>{}</time><div><strong>{}</strong><p>{}</p></div></li>",
        escape(&item.time),
        escape(&item.title),
        escape(&item.detail)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_message(message: &MessageItem) -> String {
    format!(
        "<button type=\"button\" class=\"hub-item\"><span>{}</span><strong>{}</strong><p>{}</p></button>",
        escape(&message.channel),
        escape(&message.from),
        escape(&message.preview)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_community(item: &CommunityItem) -> String {
    format!(
        "<button type=\"button\" class=\"hub-item\"><span>{}</span><strong>{}</strong><p>{}</p></button>",
        escape(&item.space),
        escape(&item.topic),
        escape(&item.activity)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_selected_node(node: &WorkflowNode) -> String {
    format!(
        "<aside class=\"node-inspector\"><p class=\"eyebrow\">Selected node</p><h4>{}</h4><p><strong>{}</strong> · {}</p></aside>",
        escape(&node.label),
        escape(&node.kind),
        escape(&node.explanation)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_workflow_svg(nodes: &[WorkflowNode]) -> String {
    let svg_nodes = nodes
        .iter()
        .map(|node| {
            format!(
                "<g class=\"workflow-node-group selectable\"><rect x=\"{x}\" y=\"{y}\" width=\"130\" height=\"56\" rx=\"10\" class=\"workflow-node\"></rect><circle cx=\"{in_x}\" cy=\"{port_y}\" r=\"4\" class=\"port in\"></circle><circle cx=\"{out_x}\" cy=\"{port_y}\" r=\"4\" class=\"port out\"></circle><text x=\"{label_x}\" y=\"{label_y}\">{label}</text><text x=\"{kind_x}\" y=\"{kind_y}\" class=\"node-kind\">{kind}</text></g>",
                x = node.x,
                y = node.y,
                in_x = node.x + 8,
                out_x = node.x + 122,
                port_y = node.y + 28,
                label_x = node.x + 16,
                label_y = node.y + 24,
                kind_x = node.x + 16,
                kind_y = node.y + 43,
                label = escape(&node.label),
                kind = escape(&node.kind)
            )
        })
        .collect::<String>();

    format!(
        "<svg viewBox=\"0 0 820 310\" aria-hidden=\"true\"><line x1=\"140\" y1=\"120\" x2=\"690\" y2=\"120\" class=\"workflow-edge\"></line>{svg_nodes}</svg>"
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_ontology(fact: &OntologyFact) -> String {
    format!(
        "<li><strong>{}</strong><span>{}</span><p>{}</p></li>",
        escape(&fact.entity),
        escape(&fact.relation),
        escape(&fact.access_reason)
    )
}

#[cfg(any(feature = "ssr", test))]
fn static_suggestion(suggestion: &IntelligenceSuggestion) -> String {
    format!(
        "<li><strong>{}</strong><p>{}</p><span>{}</span></li>",
        escape(&suggestion.title),
        escape(&suggestion.body),
        escape(&suggestion.guardrail)
    )
}

#[cfg(any(feature = "ssr", test))]
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
