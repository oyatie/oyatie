use super::*;

pub(super) fn command_center_workbench(
    envelope: TenantRenderEnvelope,
    available: Signal<bool>,
) -> impl IntoView {
    let valid = inbox_sample_valid(&envelope);
    view! {
        <section id="command-center-workbench" class="command-center-workbench" aria-labelledby="command-workbench-title"
            data-inbox-source=inbox_sample_source(&envelope) data-inbox-context=envelope.context.id()
            data-inbox-label=daily_sample_label(&envelope)
            data-inbox-hub-source=HubSample::envelope_source(&envelope)
            data-inbox-mode=if cfg!(target_arch = "wasm32") { "island" } else { "ssr" }
            data-inbox-available=move || (valid && inbox_sample_available(available)).to_string()
            inner_html=inbox_body_html(&envelope)></section>
    }
}

fn inbox_sample_available(available: Signal<bool>) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        available.try_get().unwrap_or(false)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        available.try_get_untracked().unwrap_or(false)
    }
}
