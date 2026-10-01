use super::*;

/// Reveal an anchor destination through its existing tab/panel relationship.
#[cfg(target_arch = "wasm32")]
pub(super) fn reveal_fragment_destination(target: web_sys::Element, move_focus: bool) {
    use wasm_bindgen::JsCast;

    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };

    if let Ok(Some(panel)) = target.closest("[role=\"tabpanel\"]") {
        if let Some(tab) = panel
            .get_attribute("aria-labelledby")
            .and_then(|id| document.get_element_by_id(&id))
            .filter(|tab| {
                tab.get_attribute("role").as_deref() == Some("tab")
                    && tab.get_attribute("aria-controls").as_deref() == Some(panel.id().as_str())
            })
        {
            if let (Ok(tab), Ok(Some(tablist))) = (
                tab.clone().dyn_into::<web_sys::HtmlElement>(),
                tab.closest("[role=\"tablist\"]"),
            ) {
                activate_tab(&tab, &collect_tablist_tabs(&tablist), false);
            }
        }
    }

    if move_focus {
        if let Some(element) = target.dyn_ref::<web_sys::HtmlElement>() {
            if !element.has_attribute("tabindex") {
                let _ = element.set_attribute("tabindex", "-1");
            }
            let _ = element.focus();
        }
        target.scroll_into_view();
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_name = decodeURIComponent)]
    fn decode_fragment_id(fragment: &str) -> Result<String, wasm_bindgen::JsValue>;
}

#[cfg(target_arch = "wasm32")]
pub(super) fn current_fragment_destination(
    document: &web_sys::Document,
) -> Option<web_sys::Element> {
    if let Ok(Some(target)) = document.query_selector(":target") {
        return Some(target);
    }
    // Replacing the SSR target during WASM mount can leave :target unresolved.
    let url = document.url().ok()?;
    let (_, fragment) = url.split_once('#')?;
    let id = decode_fragment_id(fragment).ok()?;
    document.get_element_by_id(&id)
}

#[cfg(target_arch = "wasm32")]
pub(super) fn restore_fragment_destination() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    if let Some(target) = current_fragment_destination(&document) {
        // The Ontology bridge owns the canonical console target and restores its
        // focus only when context and facts match. Other targets use this restorer.
        let move_focus = target.id() != "ontology-command-console"
            && document
                .active_element()
                .is_none_or(|element| element.tag_name() == "BODY");
        reveal_fragment_destination(target, move_focus);
    }
}

/// Keep native anchor navigation and history, revealing hidden destinations
/// before the browser scrolls. Clicking the current fragment also restores it.
#[cfg(target_arch = "wasm32")]
pub(super) fn wire_fragment_navigation() {
    use wasm_bindgen::{JsCast, closure::Closure};

    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let click_document = document.clone();
    let click =
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            if event.default_prevented()
                || event.button() != 0
                || event.alt_key()
                || event.ctrl_key()
                || event.meta_key()
                || event.shift_key()
            {
                return;
            }
            let Some(element) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            else {
                return;
            };
            let Ok(Some(anchor)) = element.closest("a[href^=\"#\"]") else {
                return;
            };
            if let Some(target) = anchor
                .get_attribute("href")
                .and_then(|href| decode_fragment_id(href.trim_start_matches('#')).ok())
                .and_then(|id| click_document.get_element_by_id(&id))
            {
                reveal_fragment_destination(target, true);
            }
        });
    register_island_listener(document.into(), "click", click.into_js_value());

    for event in ["hashchange", "popstate"] {
        let changed = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            let Some(document) = web_sys::window().and_then(|window| window.document()) else {
                return;
            };
            if let Some(target) = current_fragment_destination(&document) {
                reveal_fragment_destination(target, true);
            }
        });
        register_island_listener(window.clone().into(), event, changed.into_js_value());
    }
}
