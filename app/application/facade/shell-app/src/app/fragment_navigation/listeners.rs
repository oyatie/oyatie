use super::*;

/// Keep native anchor navigation and history, revealing hidden destinations
/// before the browser scrolls. Clicking the current fragment also restores it.
#[cfg(target_arch = "wasm32")]
pub(crate) fn wire_fragment_navigation() {
    use wasm_bindgen::{JsCast, closure::Closure};

    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    if fragment_owner_ready() || FRAGMENT_BLOCKED.with(std::cell::Cell::get) {
        return;
    }
    // Mount removes the SSR subtree even when listener installation must retry.
    FRAGMENT_FOCUS.with(|focus| {
        FragmentFocus::remember_mount_focus(
            &mut focus.borrow_mut(),
            document
                .active_element()
                .filter(|element| element.tag_name() != "BODY")
                .map(|node| FragmentFocus {
                    node,
                    url: document.url().unwrap_or_default(),
                    time_stamp: None,
                }),
        );
    });
    let click_document = document.clone();
    let click = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
        if !fragment_owner_ready() {
            return;
        }
        let Some(mouse) = event.dyn_ref::<web_sys::MouseEvent>() else {
            return;
        };
        if event.default_prevented()
            || mouse.button() != 0
            || mouse.alt_key()
            || mouse.ctrl_key()
            || mouse.meta_key()
            || mouse.shift_key()
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
        let Some(id) = anchor
            .get_attribute("href")
            .and_then(|href| decode_fragment_id(href.trim_start_matches('#')).ok())
        else {
            return;
        };
        if let Some(target) = click_document.get_element_by_id(&id) {
            cancel_fragment_activation();
            reveal_fragment_destination(target, true);
        } else if event.is_trusted() && anchor.is_connected() && id != "ontology-command-console" {
            let Some((url, current_id)) = anchor_destination(&anchor) else {
                return;
            };
            if current_id != id {
                return;
            }
            let old_url = click_document.url().unwrap_or_default();
            PENDING_FRAGMENT.with(|pending| {
                *pending.borrow_mut() = Some(FragmentActivation::new(anchor, old_url, url, id))
            });
        }
    });
    let mut handlers: Vec<(web_sys::EventTarget, &str, bool, wasm_bindgen::JsValue)> = vec![(
        document.clone().into(),
        "click",
        false,
        click.into_js_value(),
    )];
    for name in ["hashchange", "popstate"] {
        let changed = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
            if !fragment_owner_ready() {
                return;
            }
            let Some(document) = web_sys::window().and_then(|window| window.document()) else {
                return;
            };
            let url = document.url().unwrap_or_default();
            if name == "popstate" {
                FRAGMENT_FOCUS.with(|focus| *focus.borrow_mut() = None);
            }
            PENDING_FRAGMENT.with(|pending| {
                let mut pending = pending.borrow_mut();
                let keep = pending.as_mut().is_none_or(|activation| {
                    if name == "popstate" {
                        activation.popstate(event.is_trusted(), &url)
                    } else {
                        activation.hashchange(
                            event.is_trusted(),
                            &string_property(event.as_ref(), "oldURL").unwrap_or_default(),
                            &string_property(event.as_ref(), "newURL").unwrap_or_default(),
                            &url,
                        )
                    }
                });
                if !keep {
                    *pending = None;
                }
            });
            if let Some(target) = current_fragment_destination(&document) {
                cancel_fragment_activation();
                let newer_focus = name == "hashchange"
                    && event.is_trusted()
                    && FRAGMENT_FOCUS.with(|focus| {
                        focus.borrow().as_ref().is_some_and(|focus| {
                            focus.follows_hash(
                                event.time_stamp(),
                                &string_property(event.as_ref(), "newURL").unwrap_or_default(),
                                &url,
                                focus.node.is_connected()
                                    && document.active_element().as_ref() == Some(&focus.node),
                            )
                        })
                    });
                if !newer_focus {
                    reveal_fragment_destination(target, true);
                }
            }
        });
        handlers.push((window.clone().into(), name, false, changed.into_js_value()));
    }
    for name in ["pointerdown", "keydown"] {
        let cancel = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
            if fragment_owner_ready() && event.is_trusted() {
                cancel_fragment_activation();
            }
        });
        handlers.push((document.clone().into(), name, true, cancel.into_js_value()));
    }
    let focus_document = document.clone();
    let focus = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
        if !fragment_owner_ready() {
            return;
        }
        let Some(active) = focus_document.active_element() else {
            return;
        };
        FRAGMENT_FOCUS.with(|focus| {
            *focus.borrow_mut() = (active.tag_name() != "BODY").then(|| FragmentFocus {
                node: active.clone(),
                url: focus_document.url().unwrap_or_default(),
                time_stamp: Some(event.time_stamp()),
            });
        });
        PENDING_FRAGMENT.with(|pending| {
            let mut pending = pending.borrow_mut();
            if pending.as_ref().is_some_and(|activation| {
                active.tag_name() != "BODY" && active != activation.anchor
            }) {
                *pending = None;
            }
        });
    });
    handlers.push((document.into(), "focusin", true, focus.into_js_value()));
    let mut installed: Vec<(web_sys::EventTarget, &str, bool, wasm_bindgen::JsValue)> = Vec::new();
    for (target, name, capture, callback) in handlers {
        if target
            .add_event_listener_with_callback_and_bool(name, callback.unchecked_ref(), capture)
            .is_err()
        {
            cancel_fragment_activation();
            let mut retained = Vec::new();
            for (target, name, capture, callback) in installed {
                if target
                    .remove_event_listener_with_callback_and_bool(
                        name,
                        callback.unchecked_ref(),
                        capture,
                    )
                    .is_err()
                {
                    retained.push(callback);
                }
            }
            FRAGMENT_BLOCKED.with(|blocked| blocked.set(!retained.is_empty()));
            FRAGMENT_LISTENERS.with(|listeners| *listeners.borrow_mut() = retained);
            return;
        }
        installed.push((target, name, capture, callback));
    }
    FRAGMENT_LISTENERS.with(|listeners| {
        *listeners.borrow_mut() = installed
            .into_iter()
            .map(|(_, _, _, callback)| callback)
            .collect()
    });
    FRAGMENT_READY.with(|ready| ready.set(true));
}
