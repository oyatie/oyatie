use super::*;

/// A live event listener owned by the keyboard island: the target, the event
/// name, and the leaked-into-`JsValue` callback whose lifetime must match the
/// listener's. Storing the `JsValue` here keeps the underlying closure alive
/// (so it is never dropped while attached) and lets cleanup detach it.
#[cfg(target_arch = "wasm32")]
pub(super) struct IslandListener {
    target: web_sys::EventTarget,
    event: &'static str,
    callback: wasm_bindgen::JsValue,
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static ISLAND_LISTENERS: std::cell::RefCell<Vec<IslandListener>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Register an island listener: attach the callback to the DOM, retain it in the
/// island registry so it outlives this call. The dashboard owns its cleanup.
#[cfg(target_arch = "wasm32")]
pub(super) fn register_island_listener(
    target: web_sys::EventTarget,
    event: &'static str,
    callback: wasm_bindgen::JsValue,
) {
    use wasm_bindgen::JsCast;

    if target
        .add_event_listener_with_callback(event, callback.unchecked_ref())
        .is_err()
    {
        return;
    }

    ISLAND_LISTENERS.with(|listeners| {
        listeners.borrow_mut().push(IslandListener {
            target,
            event,
            callback,
        });
    });
}

/// Detach every island listener on unmount so no callback is freed while its
/// listener is still attached to the DOM, then clear the registry.
#[cfg(target_arch = "wasm32")]
pub(super) fn detach_island_listeners() {
    use wasm_bindgen::JsCast;

    ISLAND_LISTENERS.with(|listeners| {
        for listener in listeners.borrow_mut().drain(..) {
            let _ = listener.target.remove_event_listener_with_callback(
                listener.event,
                listener.callback.unchecked_ref(),
            );
            if let Some(element) = listener.target.dyn_ref::<web_sys::Element>() {
                if element.get_attribute("role").as_deref() == Some("tablist") {
                    let _ = element.remove_attribute("data-tablist-wired");
                }
            }
        }
    });
}
