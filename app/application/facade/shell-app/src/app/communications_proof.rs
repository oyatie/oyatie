use super::*;

pub(super) fn comms_receipt_bridge(
    sample: RwSignal<HubSample>,
    set_active: WriteSignal<ProductSurface>,
    available: Signal<bool>,
) -> impl IntoView {
    view! { <section class="comms-receipt-bridge" aria-label="Sample communication destinations">
        <div class="comms-bridge-head"><div><p class="screen-anchor">"SAMPLE DESTINATIONS"</p><h4>"Inspect channels and existing sample evidence"</h4>
            <span data-comms-bridge-status="true">"These routes open previews; no handoff is sealed and no receipt is created."</span></div></div>
        <div class="comms-bridge-routes">{HubChannel::ALL.into_iter().map(|channel| view! { <button type="button"
            data-comms-bridge-route=channel.label().to_ascii_lowercase() class=move || if render_hub_sample(sample).channel == channel { "selected" } else { "" }
            aria-pressed=move || (render_hub_sample(sample).channel == channel).to_string() disabled=move || !hub_editing_available(available) aria-describedby="hub-availability"
            on:click=move |event| { if hub_control_available(sample, available, event.current_target()) { select_product_surface(channel.surface(), set_active, sample); } }><strong>{format!("Open {} sample", channel.label())}</strong></button> }).collect_view()}</div>
        <div class="comms-bridge-actions"><a href="#workflow-studio">"Open Workflow sample"</a><a href="#cloud-ops-cockpit">"Open Cloud sample"</a>
            <a href="#evidence-spine">"Open sample evidence"</a><a href="#audit-ledger-title">"Open Audit sample"</a>
            <button type="button" data-comms-bridge-action="seal" disabled aria-describedby="hub-seal-reason">"Seal handoff"</button>
            <p id="hub-seal-reason">"Sealing and receipt creation are unavailable; no evidence service is connected."</p></div>
    </section> }
}

pub(super) fn comms_product_board(surface: ProductSurface) -> impl IntoView {
    let (class, heading, description) = match surface {
        ProductSurface::Messenger => (
            "comms-product-board messenger-board",
            "Messenger sample workspace",
            "Inspect sample threads, review text locally, or copy a selected item to a local Mail draft.",
        ),
        ProductSurface::Mail => (
            "comms-product-board mail-board",
            "Mail sample workspace",
            "Inspect sample briefs and browser-local drafts. Recipients, approvals and external delivery are unavailable.",
        ),
        ProductSurface::Community => (
            "comms-product-board community-board",
            "Community sample workspace",
            "Inspect sample community notes and browser-local copies. Publication, audience grants, voting and moderation are unavailable.",
        ),
        ProductSurface::Workflow => (
            "comms-product-board",
            "Workflow sample",
            "Choose a valid communication channel to inspect its complete workspace.",
        ),
    };
    view! { <section class=class data-comms-product-board="true" data-comms-board-surface=surface.label() aria-label=heading>
        <div class="comms-board-head"><div><p class="screen-anchor">"COMMUNICATIONS SAMPLE"</p><h4>{heading}</h4><span>{description}</span></div></div>
        <div class="comms-board-grid"><article class="thread-transcript-card"><p class="screen-anchor">"SAMPLE TEXT"</p><p>"The list and detail below come from the current sample envelope. They are not live messages or delivery receipts."</p></article>
            <article class="mail-envelope-card"><p class="screen-anchor">"LOCAL DRAFTS"</p><p>"Queue and copy select a browser-local draft. No message is sent, posted, approved or persisted to a service."</p></article>
            <article class="community-post-card"><p class="screen-anchor">"SERVICE STATUS"</p><p>"Live presence, grants, polls, publication and moderation are unavailable. Sample evidence words do not verify a receipt."</p></article></div>
    </section> }
}
