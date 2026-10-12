use super::*;

pub(super) fn communication_composer(
    sample: RwSignal<HubSample>,
    available: Signal<bool>,
) -> impl IntoView {
    view! { <div class="hub-composer" aria-label="Local draft composer">
        <label for="hub-composer-input">"Draft a local response"</label>
        <textarea id="hub-composer-input" rows="3" prop:value=move || render_hub_sample(sample).composer
            placeholder="Queue draft keeps the exact text in this browser only." disabled=move || !hub_editing_available(available) aria-describedby="hub-availability"
            on:input=move |event| { if hub_control_available(sample, available, event.current_target()) { edit_hub_sample(sample, |state| state.composer = event_target_value(&event)); } }></textarea>
        <div class="composer-actions"><button type="button" disabled=move || !hub_editing_available(available) aria-describedby="hub-availability"
            on:click=move |event| { if hub_control_available(sample, available, event.current_target()) { edit_hub_sample(sample, |state| { state.queue(); }); } }>"Queue draft"</button>
            <button type="button" class="secondary" disabled=move || !hub_editing_available(available) aria-describedby="hub-availability"
                on:click=move |event| { if hub_control_available(sample, available, event.current_target()) { edit_hub_sample(sample, |state| state.composer.clear()); } }>"Clear"</button></div>
        <p>"Drafts stay in this browser; changed source or page reload clears them. No external send or publication."</p>
    </div> }
}
