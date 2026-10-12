use super::*;

pub(super) fn communication_hub(
    sample: RwSignal<HubSample>,
    set_active: WriteSignal<ProductSurface>,
    available: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="communications-hub interactive-hub" data-hub-source=move || render_hub_sample(sample).source
            data-hub-context=move || render_hub_sample(sample).context.id() data-hub-mode=if cfg!(target_arch = "wasm32") { "island" } else { "ssr" }>
            <div class="hub-tabs" role="tablist" aria-label="Work hub channels" data-native-hub-tabs="true">
                {HubChannel::ALL.into_iter().map(|channel| view! {
                    <button type="button" id=hub_tab_id(channel) role="tab" aria-controls=hub_panel_id(channel)
                        aria-selected=move || (render_hub_sample(sample).channel == channel).to_string()
                        tabindex=move || if render_hub_sample(sample).tab_stop == channel { "0" } else { "-1" }
                        class=move || if render_hub_sample(sample).channel == channel { "hub-tab active" } else { "hub-tab" }
                        disabled=move || !hub_editing_available(available) aria-describedby="hub-availability"
                        on:click=move |event| { if hub_control_available(sample, available, event.current_target()) { select_product_surface(channel.surface(), set_active, sample); } }
                        on:keydown=move |event| {
                            if !hub_control_available(sample, available, event.current_target()) { return; }
                            match event.key().as_str() {
                                "Enter" | " " | "Spacebar" => { event.prevent_default(); select_product_surface(channel.surface(), set_active, sample); }
                                key => if let Some(next) = hub_focus_channel(channel, key) { event.prevent_default(); edit_hub_sample(sample, |state| state.tab_stop = next); focus_hub_element(&hub_tab_id(next)); },
                            }
                        }>{channel.label()}</button>
                }).collect_view()}
            </div>
            <p id="hub-availability" role="status">{move || if hub_editing_available(available) {
                "Sample communications and browser-local drafts only; no live send, publication or receipt creation."
            } else if cfg!(target_arch = "wasm32") {
                "Work Hub unavailable while the requested sample context is loading, failed or mismatched. Local controls are disabled."
            } else { "Server sample preview; local controls require the matching WebAssembly island. No message sent or posted." }}</p>
            <div class="hub-channel-panel" role="tabpanel" id=move || hub_panel_id(render_hub_sample(sample).channel)
                aria-labelledby=move || hub_tab_id(render_hub_sample(sample).channel)>
                <div class="comms-kpi-strip" aria-label="Sample communications summary">
                    <span><strong>{move || render_hub_sample(sample).visible().len()}</strong><small>"visible items"</small></span>
                    <span><strong>{move || render_hub_sample(sample).drafts.len()}</strong><small>"browser-local drafts"</small></span>
                    <span><strong>"Unavailable"</strong><small>"live delivery"</small></span>
                </div>
                <div class="hub-route-board" aria-label="Sample channel routes">
                    <div><p class="screen-anchor">"SAMPLE ROUTES"</p><strong>"Open a complete channel sample"</strong>
                        <span data-comms-route-status="true">{move || format!("{} sample preview · no message sent or posted", render_hub_sample(sample).channel.label())}</span></div>
                    {HubChannel::ALL.into_iter().map(|channel| view! {
                        <button type="button" data-hub-route=channel.label() disabled=move || !hub_editing_available(available) aria-describedby="hub-availability"
                            on:click=move |event| { if hub_control_available(sample, available, event.current_target()) { select_product_surface(channel.surface(), set_active, sample); } }>{channel.label()}</button>
                    }).collect_view()}
                </div>
                {move || comms_product_board(render_hub_sample(sample).channel.surface())}
                {comms_receipt_bridge(sample, set_active, available)}
                <div class="comms-service-toolbar" aria-label="Communications workspace controls">
                    <label><span aria-hidden="true">"⌕"</span><input data-comms-search="true" aria-label="Search communications" placeholder="Search sample and local items"
                        prop:value=move || render_hub_sample(sample).query disabled=move || !hub_editing_available(available) aria-describedby="hub-availability"
                        on:input=move |event| { if hub_control_available(sample, available, event.current_target()) { edit_hub_sample(sample, |state| { state.query = event_target_value(&event); state.reconcile_selection(); }); } } /></label>
                    <div class="comms-filter-pills" role="toolbar" aria-label="Communication filters">
                        {HubFilter::ALL.into_iter().map(|filter| view! {
                            <button type="button" data-comms-filter=match filter { HubFilter::All => "all", HubFilter::Drafts => "draft", HubFilter::Evidence => "evidence" }
                                class=move || if render_hub_sample(sample).filter == filter { "active" } else { "" }
                                aria-pressed=move || (render_hub_sample(sample).filter == filter).to_string()
                                disabled=move || !hub_editing_available(available) aria-describedby="hub-availability"
                                on:click=move |event| { if hub_control_available(sample, available, event.current_target()) { edit_hub_sample(sample, |state| { state.filter = filter; state.reconcile_selection(); }); } }>{filter.label()}</button>
                        }).collect_view()}
                        <button type="button" disabled aria-describedby="hub-unread-reason">"Unread"</button>
                    </div>
                    <button type="button" data-comms-action="new-thread" disabled=move || !hub_editing_available(available) aria-describedby="hub-availability"
                        on:click=move |event| { if hub_control_available(sample, available, event.current_target()) { focus_hub_element("hub-composer-input"); } }>"New local draft"</button>
                    <span id="hub-unread-reason">"Unread and notifications unavailable: no live read state is connected."</span>
                    <span data-comms-status="true" role="status">{move || render_hub_sample(sample).outcome}</span>
                </div>
                <div class="hub-workspace comms-service-shell">
                    <aside class="comms-sidebar" aria-label="Communications channels"><p class="screen-anchor">"SAMPLE CHANNELS"</p>
                        {HubChannel::ALL.into_iter().map(|channel| view! {
                            <button type="button" data-hub-route=channel.label() class=move || if render_hub_sample(sample).channel == channel { "active" } else { "" }
                                aria-pressed=move || (render_hub_sample(sample).channel == channel).to_string()
                                disabled=move || !hub_editing_available(available) aria-describedby="hub-availability"
                                on:click=move |event| { if hub_control_available(sample, available, event.current_target()) { select_product_surface(channel.surface(), set_active, sample); } }>
                                <strong>{channel.label()}</strong><span>{move || format!("{} sample/local items", render_hub_sample(sample).rows(channel).len())}</span>
                            </button>
                        }).collect_view()}
                    </aside>
                    <div class="hub-list" role="list" aria-label="Channel items">{move || {
                        let current = render_hub_sample(sample); let rows = current.visible();
                        if rows.is_empty() { return view! { <p role="status">"No matching sample or local items. Clear search or choose All to recover."</p> }.into_any(); }
                        rows.into_iter().map(|row| { let key = row.key; let kind = hub_row_kind(&row); view! {
                            <div role="listitem"><button type="button" class=if Some(key) == current.selected { "hub-item active" } else { "hub-item" }
                                data-comms-item="true" data-comms-kind=kind aria-pressed=(Some(key) == current.selected).to_string()
                                disabled=move || !hub_editing_available(available) aria-describedby="hub-availability"
                                on:click=move |event| { if hub_control_available(sample, available, event.current_target()) { edit_hub_sample(sample, |state| { state.selected = Some(key); state.preview = false; }); } }>
                                <span class=hub_item_chip_class(kind)>{row.item.source}</span><strong>{row.item.title}</strong><p>{row.item.body}</p>
                                <small><em>{if kind == "draft" { "local draft" } else if kind == "evidence" { "sample evidence" } else { "sample" }}</em><b>{row.item.meta}</b></small>
                            </button></div>
                        } }).collect_view().into_any()
                    }}</div>
                    <div class="hub-detail" aria-live="polite">
                        <div class="comms-message-toolbar" aria-label="Selected sample actions">
                            <button type="button" data-comms-action="mark-reviewed" disabled=move || !hub_editing_available(available) || render_hub_sample(sample).selected_row().is_none()
                                aria-describedby="hub-availability" on:click=move |event| { if hub_control_available(sample, available, event.current_target()) { edit_hub_sample(sample, HubSample::review_selected); } }>"Review locally"</button>
                            <button type="button" data-comms-action="send-preview" disabled=move || !hub_editing_available(available) || render_hub_sample(sample).selected_row().is_none()
                                aria-describedby="hub-availability" on:click=move |event| { if hub_control_available(sample, available, event.current_target()) { edit_hub_sample(sample, |state| { state.preview = true; state.outcome = "Selected text previewed locally; no external send.".into(); }); } }>"Preview selected text"</button>
                            {hub_copy_button(HubChannel::Mail, sample, set_active, available)}{hub_copy_button(HubChannel::Community, sample, set_active, available)}
                        </div>
                        {move || { let current = render_hub_sample(sample); match current.selected_row() {
                            Some(row) => view! { <article class="comms-detail-card"><div class="comms-detail-head"><div><p class="eyebrow">{current.channel.label()}</p><h4>{row.item.title}</h4></div>
                                <span class="status-chip">{if current.reviewed.contains(&row.key) { "reviewed locally" } else { "sample/local item" }}</span></div>
                                <p>{row.item.body.clone()}</p><span class="hub-meta">{row.item.meta}</span><dl class="comms-detail-grid">
                                    <div><dt>"Route"</dt><dd>{current.channel.label()}</dd></div><div><dt>"Source"</dt><dd>{row.item.source}</dd></div><div><dt>"Persistence"</dt><dd>"Browser-local drafts only; no backend write"</dd></div>
                                </dl>{current.preview.then(|| view! { <details open><summary>"Local text preview; no send"</summary><p>{row.item.body}</p></details> })}</article> }.into_any(),
                            None => view! { <article><p class="eyebrow">"Empty selection"</p><h4>"No visible item selected"</h4><p>"Clear search or choose All, then select a visible item."</p></article> }.into_any(),
                        }}}
                        {communication_composer(sample, available)}
                    </div>
                    <aside class="comms-context-rail" aria-label="Sample provenance and unavailable services">
                        <section><p class="screen-anchor">"SAMPLE PROVENANCE"</p><p>{move || render_hub_sample(sample).provenance}</p></section>
                        <section class="comms-handoff-card" data-comms-handoff="true"><p class="screen-anchor">"LOCAL COPY"</p><strong>"Copy selected text into a local destination draft"</strong>
                            <span>"The destination draft is selected in this browser. No delivery or approval follows."</span><div class="comms-handoff-actions">
                                {hub_copy_button(HubChannel::Mail, sample, set_active, available)}{hub_copy_button(HubChannel::Community, sample, set_active, available)}</div>
                            <p id="hub-copy-reason">"Copy requires a selected visible item and the matching browser island."</p>
                        </section>
                        <section><p class="screen-anchor">"UNAVAILABLE SERVICES"</p><p id="hub-service-reason">"Live directory, presence, task creation, evidence attachment, send, publication and notification services are not connected."</p>
                            <button type="button" data-comms-action="directory" disabled aria-describedby="hub-service-reason">"Directory"</button>
                            <button type="button" data-comms-action="create-task" disabled aria-describedby="hub-service-reason">"Create task"</button>
                            <button type="button" data-comms-action="attach-evidence" disabled aria-describedby="hub-service-reason">"Attach evidence"</button>
                        </section>
                    </aside>
                </div>
            </div>
            {HubChannel::ALL.into_iter().map(|channel| move || {
                (render_hub_sample(sample).channel != channel).then(|| view! {
                    <div role="tabpanel" id=hub_panel_id(channel) aria-labelledby=hub_tab_id(channel) hidden></div>
                })
            }).collect_view()}
        </div>
    }
}

fn hub_copy_button(
    channel: HubChannel,
    sample: RwSignal<HubSample>,
    set_active: WriteSignal<ProductSurface>,
    available: Signal<bool>,
) -> impl IntoView {
    view! { <button type="button" data-comms-action=if channel == HubChannel::Mail { "thread-to-mail" } else { "publish-note" }
    disabled=move || !hub_editing_available(available) || render_hub_sample(sample).selected_row().is_none() aria-describedby="hub-copy-reason hub-availability"
    on:click=move |event| { if hub_control_available(sample, available, event.current_target()) { edit_hub_sample(sample, |state| { if state.copy_to(channel) { set_active.set(channel.surface()); } }); } }>{format!("Copy sample to {}", channel.label())}</button> }
}

pub(super) fn hub_tab_id(channel: HubChannel) -> String {
    format!("hub-tab-{}", channel.label().to_ascii_lowercase())
}
pub(super) fn hub_panel_id(channel: HubChannel) -> String {
    format!("hub-panel-{}", channel.label().to_ascii_lowercase())
}
pub(super) fn hub_focus_channel(channel: HubChannel, key: &str) -> Option<HubChannel> {
    let index = HubChannel::ALL
        .iter()
        .position(|current| *current == channel)?;
    let next = match key {
        "ArrowRight" | "ArrowDown" => (index + 1) % 3,
        "ArrowLeft" | "ArrowUp" => (index + 2) % 3,
        "Home" => 0,
        "End" => 2,
        _ => return None,
    };
    Some(HubChannel::ALL[next])
}
pub(super) fn hub_editing_available(available: Signal<bool>) -> bool {
    cfg!(target_arch = "wasm32") && available.try_get().unwrap_or(false)
}
fn focus_hub_element(id: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        if let Some(element) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id(id))
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let _ = element.focus();
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = id;
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn communication_hub_html(
    envelope: &TenantRenderEnvelope,
    channel: HubChannel,
) -> String {
    let owner = Owner::new();
    let html = owner.with(|| {
        let mut initial = HubSample::from_envelope(envelope);
        initial.select_channel(channel);
        let sample = RwSignal::new(initial);
        let (_, set_active) = signal(channel.surface());
        communication_hub(sample, set_active, Signal::derive(|| false))
            .into_view()
            .to_html()
    });
    owner.cleanup();
    html
}

#[cfg(target_arch = "wasm32")]
pub(super) fn hub_control_available(
    sample: RwSignal<HubSample>,
    available: Signal<bool>,
    target: Option<web_sys::EventTarget>,
) -> bool {
    use wasm_bindgen::JsCast;
    if !hub_editing_available(available) {
        return false;
    }
    let Some(target) = target.and_then(|target| target.dyn_into::<web_sys::Element>().ok()) else {
        return false;
    };
    if !target.is_connected() {
        return false;
    }
    let Ok(Some(owner)) = target.closest(".communications-hub[data-hub-source]") else {
        return false;
    };
    sample.try_get_untracked().is_some_and(|current| {
        owner.get_attribute("data-hub-source").as_deref() == Some(current.source.as_str())
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn hub_control_available<T>(
    _sample: RwSignal<HubSample>,
    _available: Signal<bool>,
    _target: Option<T>,
) -> bool {
    false
}
