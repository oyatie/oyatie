use super::*;

#[expect(
    clippy::too_many_arguments,
    reason = "Communication surface keeps signal ownership explicit so local-only drafts cannot be confused with server persistence."
)]
pub(super) fn communication_hub(
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
                        aria-selected=move || (render_signal(active_surface) == surface).to_string()
                        class=move || if render_signal(active_surface) == surface { "hub-tab active" } else { "hub-tab" }
                        on:click=move |_| {
                            set_active_surface.set(surface);
                            set_selected_hub_index.set(0);
                        }
                    >
                        {surface.label()}
                    </button>
                }).collect_view()}
            </div>

            <div class="comms-kpi-strip" aria-label="Built-in communications summary">
                <span><strong>"18"</strong><small>"threads · drafts"</small></span>
                <span><strong>"6"</strong><small>"workflow routes"</small></span>
                <span><strong>"4"</strong><small>"evidence links"</small></span>
                <span><strong>"0"</strong><small>"external sends"</small></span>
            </div>

            <div class="hub-route-board" aria-label="Workflow output routes">
                <div>
                    <p class="screen-anchor">"OUTPUT ROUTES"</p>
                    <strong>"FD-001 tenant-workload drafts fan out to Messenger, Mail, and Community with evidence return paths"</strong>
                    <span data-comms-route-status="true">"FD-001 workload dogfood · REC-WF-7741 · no backend send"</span>
                </div>
                <button type="button" data-hub-route="Messenger" on:click=move |_| {
                    set_active_surface.set(ProductSurface::Messenger);
                    set_selected_hub_index.set(0);
                }>"Messenger post"</button>
                <button type="button" data-hub-route="Mail" on:click=move |_| {
                    set_active_surface.set(ProductSurface::Mail);
                    set_selected_hub_index.set(0);
                }>"Mail draft"</button>
                <button type="button" data-hub-route="Community" on:click=move |_| {
                    set_active_surface.set(ProductSurface::Community);
                    set_selected_hub_index.set(0);
                }>"Community note"</button>
            </div>

            {move || comms_product_board(render_signal(active_surface))}

            <section class="comms-substrate-strip" aria-label="Oyatie Cloud tenant-workload proof">
                <div>
                    <p class="screen-anchor">"SUBSTRATE PROOF"</p>
                    <strong>"Messenger, Mail, and Community are dogfood tenant workloads on Oyatie Cloud"</strong>
                    <span data-comms-substrate-status="true">
                        {move || format!(
                            "{} route pinned to FD-001 workload · cell-us-east-2 · local visual proof",
                            render_signal(active_surface).label()
                        )}
                    </span>
                </div>
                <button type="button" data-comms-action="prove-substrate">
                    <span>"Cloud cell"</span><strong>"cell-us-east-2"</strong>
                </button>
                <button type="button" data-comms-action="route-cloud">
                    <span>"Tenant workload"</span><strong>"FD-001 microservices"</strong>
                </button>
                <button type="button" data-comms-action="seal-proof">
                    <span>"Evidence"</span><strong>"REC-WF-7741"</strong>
                </button>
            </section>

            {comms_receipt_bridge()}

            <div class="comms-service-toolbar" aria-label="Communications workspace controls">
                <label>
                    <span aria-hidden="true">"⌕"</span>
                    <input data-comms-search="true" aria-label="Search communications" placeholder="Search threads, mail, spaces..." />
                </label>
                <div class="comms-filter-pills" role="toolbar" aria-label="Communication filters">
                    <button type="button" class="active" data-comms-filter="all">"All"</button>
                    <button type="button" data-comms-filter="unread">"Unread"</button>
                    <button type="button" data-comms-filter="draft">"Drafts"</button>
                    <button type="button" data-comms-filter="evidence">"Evidence"</button>
                </div>
                <button type="button" data-comms-action="new-thread">"New thread"</button>
                <button type="button" data-comms-action="attach-evidence">"Attach evidence"</button>
                <button type="button" data-comms-action="directory">"Directory"</button>
                <span data-comms-status="true">"Local service workspace ready · no external send"</span>
            </div>

            <div class="hub-workspace comms-service-shell">
                <aside class="comms-sidebar" aria-label="Communications folders and spaces">
                    <p class="screen-anchor">"WORKSPACES"</p>
                    <button type="button" class="active" data-hub-route="Messenger">
                        <strong>"Ops room"</strong><span>"Messenger · 5 items · 2 unread"</span>
                    </button>
                    <button type="button" data-hub-route="Mail">
                        <strong>"Finance close"</strong><span>"Mail · 4 drafts · 2 evidence"</span>
                    </button>
                    <button type="button" data-hub-route="Community">
                        <strong>"Governance council"</strong><span>"Community · 5 spaces · 1 publish"</span>
                    </button>
                    <button type="button" data-comms-action="notification-filter">
                        <strong>"Notifications"</strong><span>"6 local alerts · no external send"</span>
                    </button>
                </aside>
                <div class="hub-list" role="list" aria-label="Channel items">
                    {move || {
                        let items = hub_items(
                            &list_messages,
                            &list_communities,
                            &render_signal(local_drafts),
                            render_signal(active_surface),
                        );
                        let active_index = render_signal(selected_hub_index);
                        items.into_iter().enumerate().map(|(index, item)| {
                            let kind = hub_item_kind(&item, index);
                            let chip_class = hub_item_chip_class(kind);
                            view! {
                                <button
                                    type="button"
                                    class=if index == active_index { "hub-item active" } else { "hub-item" }
                                    data-comms-item="true"
                                    data-comms-kind=kind
                                    on:click=move |_| set_selected_hub_index.set(index)
                                >
                                    <span class=chip_class>{item.source}</span>
                                    <strong>{item.title}</strong>
                                    <p>{item.body}</p>
                                    <small><em>{kind}</em><b>{item.meta}</b></small>
                                </button>
                            }
                        }).collect_view()
                    }}
                </div>

                <div class="hub-detail" aria-live="polite">
                    <div class="comms-message-toolbar" aria-label="Selected communication actions">
                        <span class="status-chip success">"role-visible"</span>
                        <button type="button" data-comms-action="mark-reviewed">"Mark reviewed"</button>
                        <button type="button" data-comms-action="create-task">"Create task"</button>
                        <button type="button" data-comms-action="link-workflow">"Link workflow"</button>
                        <button type="button" data-comms-action="send-preview">"Send preview"</button>
                        <button type="button" data-comms-action="publish-note">"Publish local"</button>
                    </div>
                    {move || {
                        let items = hub_items(
                            &detail_messages,
                            &detail_communities,
                            &render_signal(local_drafts),
                            render_signal(active_surface),
                        );
                        match selected_hub_item(&items, render_signal(selected_hub_index)) {
                            Some(item) => {
                                let kind = hub_item_kind(&item, render_signal(selected_hub_index));
                                let chip_class = hub_item_chip_class(kind);
                                let surface_label = item.surface.label();
                                view! {
                                    <article class="comms-detail-card">
                                        <div class="comms-detail-head">
                                            <div>
                                                <p class="eyebrow">{surface_label}</p>
                                                <h4>{item.title.clone()}</h4>
                                            </div>
                                            <span class=chip_class>{kind}</span>
                                        </div>
                                        <p>{item.body.clone()}</p>
                                        <span class="hub-meta">{item.meta.clone()}</span>
                                        <dl class="comms-detail-grid">
                                            <div><dt>"Route"</dt><dd>{surface_label}</dd></div>
                                            <div><dt>"Workflow"</dt><dd>"Tenant change approval"</dd></div>
                                            <div><dt>"Receipt"</dt><dd>"REC-WF-7741"</dd></div>
                                            <div><dt>"Persistence"</dt><dd>"Local browser state only"</dd></div>
                                        </dl>
                                    </article>
                                }.into_any()
                            },
                            None => view! {
                                <article>
                                    <p class="eyebrow">"Empty channel"</p>
                                    <h4>"No visible items"</h4>
                                    <p>"This permitted envelope has no items for the selected channel."</p>
                                </article>
                            }.into_any(),
                        }
                    }}

                    {communication_composer(
                        active_surface,
                        set_selected_hub_index,
                        draft_body,
                        set_draft_body,
                        local_drafts,
                        set_local_drafts,
                    )}

                </div>

                <aside class="comms-context-rail" aria-label="People, provenance, and notification context">
                    <section>
                        <p class="screen-anchor">"PEOPLE"</p>
                        <div class="presence-stack">
                            <span><em>"OP"</em><strong>"Ops bot"</strong><small>"online"</small></span>
                            <span><em>"SR"</em><strong>"Security reviewer"</strong><small>"watching"</small></span>
                            <span><em>"FL"</em><strong>"Finance lead"</strong><small>"mail owner"</small></span>
                        </div>
                    </section>
                    <section>
                        <p class="screen-anchor">"PROVENANCE"</p>
                        <dl class="comms-kv">
                            <div><dt>"Envelope"</dt><dd>"tenant-admin"</dd></div>
                            <div><dt>"Workflow"</dt><dd>"Tenant change approval"</dd></div>
                            <div><dt>"Receipt"</dt><dd>"REC-WF-7741"</dd></div>
                        </dl>
                    </section>
                    <section class="comms-handoff-card" data-comms-handoff="true" aria-label="Local draft handoff state">
                        <p class="screen-anchor">"DRAFT HANDOFF BUS"</p>
                        <strong data-comms-handoff-title="true">"Messenger → Mail approval brief"</strong>
                        <span data-comms-handoff-status="true">"Select Promote to Mail or Publish local to carry context across surfaces."</span>
                        <dl class="comms-kv compact">
                            <div><dt>"Source"</dt><dd data-comms-handoff-source="true">"Messenger"</dd></div>
                            <div><dt>"Destination"</dt><dd data-comms-handoff-destination="true">"Mail"</dd></div>
                            <div><dt>"Audience"</dt><dd data-comms-handoff-audience="true">"CFO · SRE · Governance"</dd></div>
                            <div><dt>"Persistence"</dt><dd>"Browser local state only"</dd></div>
                        </dl>
                        <div class="comms-handoff-actions">
                            <button type="button" data-comms-action="thread-to-mail">"Promote to Mail"</button>
                            <button type="button" data-comms-action="publish-note">"Publish local"</button>
                        </div>
                    </section>
                    <section>
                        <p class="screen-anchor">"DELIVERY MATRIX"</p>
                        <div class="comms-delivery-matrix" aria-label="Local delivery readiness">
                            <span class="ready"><strong>"Messenger"</strong><em>"ops room draft"</em></span>
                            <span class="ready"><strong>"Mail"</strong><em>"approval brief"</em></span>
                            <span class="review"><strong>"Community"</strong><em>"council review"</em></span>
                            <span><strong>"Audit"</strong><em>"receipt attached"</em></span>
                        </div>
                    </section>
                    <section>
                        <p class="screen-anchor">"LOCAL NOTIFICATIONS"</p>
                        <ol class="notification-stack">
                            <li>"Draft queued locally"</li>
                            <li>"Evidence link available"</li>
                            <li>"No external send enabled"</li>
                            <li>"Workflow route preview ready"</li>
                        </ol>
                    </section>
                </aside>
            </div>
        </div>
    }
}
