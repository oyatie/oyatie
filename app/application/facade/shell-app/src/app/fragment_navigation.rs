#[cfg(target_arch = "wasm32")]
use super::*;

struct FragmentActivation {
    #[cfg(target_arch = "wasm32")]
    anchor: web_sys::Element,
    old_url: String,
    intended_url: String,
    target_id: String,
    awaiting_hash: bool,
    own_pop_available: bool,
}

impl FragmentActivation {
    fn new(
        #[cfg(target_arch = "wasm32")] anchor: web_sys::Element,
        old_url: String,
        intended_url: String,
        target_id: String,
    ) -> Self {
        let changed_url = old_url != intended_url;
        Self {
            #[cfg(target_arch = "wasm32")]
            anchor,
            old_url,
            intended_url,
            target_id,
            awaiting_hash: changed_url,
            own_pop_available: changed_url,
        }
    }

    fn popstate(&mut self, trusted: bool, current_url: &str) -> bool {
        if current_url != self.intended_url {
            return false;
        }
        if !trusted {
            return true;
        }
        if self.awaiting_hash && self.own_pop_available {
            self.own_pop_available = false;
            return true;
        }
        false
    }

    fn hashchange(
        &mut self,
        trusted: bool,
        old_url: &str,
        new_url: &str,
        current_url: &str,
    ) -> bool {
        if current_url != self.intended_url {
            return false;
        }
        if !trusted {
            return true;
        }
        if self.awaiting_hash && old_url == self.old_url && new_url == self.intended_url {
            self.awaiting_hash = false;
            self.own_pop_available = false;
            return true;
        }
        false
    }

    fn permits_restore(
        &self,
        current_url: &str,
        current_anchor_url: &str,
        current_anchor_id: &str,
        target_id: &str,
        anchor_connected: bool,
        focus_owned: bool,
    ) -> bool {
        anchor_connected
            && focus_owned
            && current_url == self.intended_url
            && current_anchor_url == self.intended_url
            && current_anchor_id == self.target_id
            && target_id == self.target_id
            && target_id != "ontology-command-console"
    }
}

struct FragmentFocus {
    #[cfg(target_arch = "wasm32")]
    node: web_sys::Element,
    url: String,
    time_stamp: Option<f64>,
}

impl FragmentFocus {
    fn remember_mount_focus(remembered: &mut Option<Self>, observed: Option<Self>) {
        if observed.is_some() {
            *remembered = observed;
        }
    }

    fn follows_hash(&self, time_stamp: f64, new_url: &str, current_url: &str, owned: bool) -> bool {
        let Some(focus_time_stamp) = self.time_stamp else {
            return false;
        };
        owned
            && time_stamp.is_finite()
            && focus_time_stamp.is_finite()
            && time_stamp < focus_time_stamp
            && new_url == self.url
            && current_url == self.url
    }

    fn permits_mount_restore(
        owner_ready: bool,
        focus: Option<&Self>,
        current_url: &str,
        focused_id: Option<&str>,
        target_id: &str,
    ) -> bool {
        owner_ready
            && focus.is_none_or(|focus| focus.url == current_url && focused_id == Some(target_id))
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static PENDING_FRAGMENT: std::cell::RefCell<Option<FragmentActivation>> = const { std::cell::RefCell::new(None) };
    static FRAGMENT_FOCUS: std::cell::RefCell<Option<FragmentFocus>> = const { std::cell::RefCell::new(None) };
    // The document and window outlive disposable dashboard panels/listeners.
    static FRAGMENT_LISTENERS: std::cell::RefCell<Vec<wasm_bindgen::JsValue>> = const { std::cell::RefCell::new(Vec::new()) };
    static FRAGMENT_READY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static FRAGMENT_BLOCKED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Reveal an anchor destination through its existing tab/panel relationship.
#[cfg(target_arch = "wasm32")]
pub(super) fn reveal_fragment_destination(target: web_sys::Element, move_focus: bool) {
    use wasm_bindgen::JsCast;

    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };

    if let Ok(event) = web_sys::Event::new("oyatie-shell-destination") {
        let _ = target.dispatch_event(&event);
    }

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
    #[wasm_bindgen(catch, js_namespace = Reflect, js_name = get)]
    fn fragment_property(
        target: &wasm_bindgen::JsValue,
        name: &str,
    ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
}

#[cfg(target_arch = "wasm32")]
fn string_property(target: &wasm_bindgen::JsValue, name: &str) -> Option<String> {
    fragment_property(target, name).ok()?.as_string()
}

#[cfg(target_arch = "wasm32")]
fn anchor_destination(anchor: &web_sys::Element) -> Option<(String, String)> {
    let href = anchor.get_attribute("href")?;
    let id = decode_fragment_id(href.trim_start_matches('#')).ok()?;
    let url = string_property(anchor.as_ref(), "href")?;
    Some((url, id))
}

#[cfg(target_arch = "wasm32")]
fn cancel_fragment_activation() {
    PENDING_FRAGMENT.with(|pending| *pending.borrow_mut() = None);
}

#[cfg(target_arch = "wasm32")]
fn fragment_owner_ready() -> bool {
    FRAGMENT_READY.with(std::cell::Cell::get)
}

#[cfg(target_arch = "wasm32")]
pub(super) fn current_fragment_destination(
    document: &web_sys::Document,
) -> Option<web_sys::Element> {
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
        let active = document.active_element();
        let body_focus = active
            .as_ref()
            .is_none_or(|element| element.tag_name() == "BODY");
        let body_restore = body_focus
            && FRAGMENT_FOCUS.with(|focus| {
                let focus = focus.borrow();
                FragmentFocus::permits_mount_restore(
                    fragment_owner_ready(),
                    focus.as_ref(),
                    &document.url().unwrap_or_default(),
                    focus.as_ref().map(|focus| focus.node.id()).as_deref(),
                    &target.id(),
                )
            });
        let activation = PENDING_FRAGMENT.with(|pending| pending.borrow_mut().take());
        let activated_focus = fragment_owner_ready()
            && activation.is_some_and(|activation| {
                let Some((anchor_url, anchor_id)) = anchor_destination(&activation.anchor) else {
                    return false;
                };
                activation.permits_restore(
                    &document.url().unwrap_or_default(),
                    &anchor_url,
                    &anchor_id,
                    &target.id(),
                    activation.anchor.is_connected(),
                    body_focus || active.as_ref() == Some(&activation.anchor),
                )
            });
        let move_focus =
            target.id() != "ontology-command-console" && (body_restore || activated_focus);
        reveal_fragment_destination(target, move_focus);
    }
}

#[cfg(target_arch = "wasm32")]
mod listeners;
#[cfg(target_arch = "wasm32")]
pub(crate) use listeners::wire_fragment_navigation;

#[cfg(test)]
#[path = "fragment_navigation_test.rs"]
mod tests;
