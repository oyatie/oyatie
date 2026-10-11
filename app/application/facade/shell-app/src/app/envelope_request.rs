use super::*;

pub(super) fn initial_envelope() -> Option<TenantRenderEnvelope> {
    #[cfg(any(feature = "ssr", test))]
    {
        Some(server_derived_envelope(OperatorContext::TenantAdmin))
    }

    #[cfg(not(any(feature = "ssr", test)))]
    {
        None
    }
}

#[cfg(target_arch = "wasm32")]
pub(super) fn request_render_envelope(
    context: OperatorContext,
    request_generation: RwSignal<u64>,
    set_envelope: WriteSignal<Option<TenantRenderEnvelope>>,
    set_selected_node_id: WriteSignal<String>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::{JsFuture, spawn_local};

    request_generation.update(|generation| *generation += 1);
    let generation = request_generation.get_untracked();
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

        if request_generation.try_get_untracked() != Some(generation) {
            return;
        }
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
pub(super) fn request_render_envelope(
    context: OperatorContext,
    _request_generation: RwSignal<u64>,
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
pub(super) fn request_render_envelope(
    _context: OperatorContext,
    _request_generation: RwSignal<u64>,
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
