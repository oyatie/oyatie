use super::*;

pub(super) fn communication_composer(
    active_surface: ReadSignal<ProductSurface>,
    set_selected_hub_index: WriteSignal<usize>,
    draft_body: ReadSignal<String>,
    set_draft_body: WriteSignal<String>,
    local_drafts: ReadSignal<Vec<LocalDraft>>,
    set_local_drafts: WriteSignal<Vec<LocalDraft>>,
) -> impl IntoView {
    view! {
        <div class="hub-composer" aria-label="Local draft composer">
            <label for="hub-composer-input">"Draft a local response"</label>
            <textarea
                id="hub-composer-input"
                rows="3"
                prop:value=move || render_signal(draft_body)
                placeholder="Type here; Queue draft keeps it local to this browser island."
                on:input=move |event| set_draft_body.set(event_target_value(&event))
            ></textarea>
            <div class="composer-actions">
                <button
                    type="button"
                    on:click=move |_| {
                        let body = render_signal(draft_body).trim().to_string();
                        if body.is_empty() {
                            return;
                        }
                        let mut drafts = render_signal(local_drafts);
                        drafts.insert(0, LocalDraft {
                            surface: render_signal(active_surface),
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
    }
}
