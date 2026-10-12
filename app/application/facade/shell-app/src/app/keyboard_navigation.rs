#[cfg(target_arch = "wasm32")]
use super::*;

const DOM_TABLIST_OWNERS: [(&str, &str); 5] = [
    ("settings-tabs", "settings-center"),
    ("identity-tabs", "identity-workforce-service"),
    ("finance-tabs", "finance-commercial-service"),
    ("cockpit-tabs", "cloud-ops-cockpit"),
    ("resource-tabs", "resource-audit-console"),
];

#[cfg(any(target_arch = "wasm32", test))]
pub(super) fn dom_tablist_owned(classes: &str) -> bool {
    let classes = classes.split_ascii_whitespace().collect::<Vec<_>>();
    !classes.contains(&"hub-tabs")
        && DOM_TABLIST_OWNERS
            .iter()
            .any(|(class, _)| classes.contains(class))
}

/// The document fallback owns only unwired DOM tablists. Hydrated listeners
/// retain their existing marker, manual activation model and island lifetime.
#[cfg(any(feature = "ssr", test))]
pub(crate) fn ssr_tab_navigation_script() -> String {
    const SCRIPT: &str = r#"
(() => {
  const owners = __TABLIST_OWNERS__;
  function controls(tab) {
    if (!(tab instanceof Element) || !tab.isConnected || tab.disabled ||
        tab.getAttribute('role') !== 'tab') return null;
    const tablist = tab.closest('[role="tablist"]');
    if (!tablist || tab.parentElement !== tablist ||
        tablist.hasAttribute('data-tablist-wired') ||
        tablist.hasAttribute('data-native-hub-tabs') || tablist.classList.contains('hub-tabs')) return null;
    const admitted = owners.filter(([name]) => tablist.classList.contains(name));
    if (admitted.length !== 1) return null;
    const [name, id] = admitted[0];
    const owner = tablist.closest(`#${id}`);
    const shell = document.querySelector('.console-app');
    if (!owner?.isConnected || document.getElementById(id) !== owner ||
        owner.closest('.console-app') !== shell || !shell ||
        owner.querySelector(`.${name}[role="tablist"]`) !== tablist) return null;
    const tabs = [...tablist.children].filter(child => child.getAttribute('role') === 'tab');
    const panels = tabs.map(candidate => document.getElementById(candidate.getAttribute('aria-controls')));
    const stem = name.slice(0, -5);
    if (!tabs.length || tabs.filter(candidate => candidate.getAttribute('aria-selected') === 'true').length !== 1 ||
        tabs.some((candidate, index) => !candidate.isConnected ||
          document.getElementById(candidate.id) !== candidate || !candidate.id.startsWith(`${stem}-tab-`) ||
          !['true', 'false'].includes(candidate.getAttribute('aria-selected')) ||
          !panels[index]?.isConnected || panels[index].getAttribute('role') !== 'tabpanel' ||
          panels[index].id !== candidate.id.replace(`${stem}-tab-`, `${stem}-panel-`) ||
          panels[index].getAttribute('aria-labelledby') !== candidate.id ||
          panels[index].closest(`#${id}`) !== owner)) return null;
    return {tabs, panels};
  }
  function activate(tab, owned, moveFocus) {
    owned.tabs.forEach((candidate, index) => {
      const selected = candidate === tab;
      candidate.setAttribute('aria-selected', String(selected));
      candidate.tabIndex = selected ? 0 : -1;
      candidate.classList.toggle('active', selected);
      owned.panels[index].classList.toggle('active', selected);
    });
    if (moveFocus) tab.focus();
  }
  document.querySelectorAll('[role="tablist"]').forEach(tablist => {
    const tab = [...tablist.children].find(child => child.getAttribute('role') === 'tab');
    const owned = controls(tab);
    owned?.tabs.forEach(candidate => {
      candidate.tabIndex = candidate.getAttribute('aria-selected') === 'true' ? 0 : -1;
    });
  });
  document.addEventListener('click', event => {
    if (!(event.target instanceof Element)) return;
    const tab = event.target.closest('[role="tab"]');
    const owned = controls(tab);
    if (owned) activate(tab, owned, event.isTrusted);
  });
  document.addEventListener('keydown', event => {
    const tab = document.activeElement;
    if (event.target !== tab) return;
    const owned = controls(tab);
    if (!owned) return;
    const current = owned.tabs.indexOf(tab);
    if (current < 0) return;
    let index;
    switch (event.key) {
      case 'Enter': case ' ': case 'Spacebar':
        event.preventDefault(); activate(tab, owned, true); return;
      case 'ArrowRight': case 'ArrowDown': index = (current + 1) % owned.tabs.length; break;
      case 'ArrowLeft': case 'ArrowUp': index = (current + owned.tabs.length - 1) % owned.tabs.length; break;
      case 'Home': index = 0; break;
      case 'End': index = owned.tabs.length - 1; break;
      default: return;
    }
    if (owned.tabs[index].disabled) return;
    event.preventDefault();
    owned.tabs.forEach((candidate, position) => { candidate.tabIndex = position === index ? 0 : -1; });
    owned.tabs[index].focus();
  });
})();
"#;
    SCRIPT.replace(
        "__TABLIST_OWNERS__",
        &serde_json::to_string(&DOM_TABLIST_OWNERS).expect("fixed DOM tab owners serialize"),
    )
}

#[cfg(target_arch = "wasm32")]
pub(super) const TABLIST_SELECTORS: [&str; 5] = [
    ".settings-tabs[role=\"tablist\"]",
    ".identity-tabs[role=\"tablist\"]",
    ".finance-tabs[role=\"tablist\"]",
    ".cockpit-tabs[role=\"tablist\"]",
    ".resource-tabs[role=\"tablist\"]",
];

/// Reusable client island (A-2): attaches canonical WAI-ARIA tablist keyboard
/// behaviour to every dashboard tablist. Idempotent — a `data-tablist-wired`
/// marker stops duplicate binding. The island retains listener closures until
/// rebinding or unmounting, then detaches them before releasing the callbacks.
#[cfg(target_arch = "wasm32")]
pub(super) fn wire_tablist_keyboard_navigation() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };

    for selector in TABLIST_SELECTORS {
        let Ok(Some(tablist)) = document.query_selector(selector) else {
            continue;
        };
        // Skip tablists already wired in a previous effect run.
        if tablist.has_attribute("data-tablist-wired") {
            continue;
        }
        let _ = tablist.set_attribute("data-tablist-wired", "true");

        let tabs = collect_tablist_tabs(&tablist);
        if tabs.is_empty() {
            continue;
        }

        // Establish the roving tabindex baseline: the selected tab is the single
        // tab stop (tabindex 0); the rest are reachable only via arrow keys.
        apply_roving_tabindex(&tabs);

        for tab in &tabs {
            wire_tab_click(tab, &tabs);
        }
        wire_tablist_keydown(&tablist, tabs);
    }
}

/// Collect the direct `role="tab"` element children of a tablist, in DOM order.
#[cfg(target_arch = "wasm32")]
pub(super) fn collect_tablist_tabs(tablist: &web_sys::Element) -> Vec<web_sys::HtmlElement> {
    use wasm_bindgen::JsCast;

    let children = tablist.children();
    let mut tabs = Vec::new();
    for index in 0..children.length() {
        if let Some(child) = children.item(index) {
            if child.get_attribute("role").as_deref() == Some("tab") {
                if let Ok(html) = child.dyn_into::<web_sys::HtmlElement>() {
                    tabs.push(html);
                }
            }
        }
    }
    tabs
}

#[cfg(target_arch = "wasm32")]
pub(super) fn apply_roving_tabindex(tabs: &[web_sys::HtmlElement]) {
    for tab in tabs {
        let selected = tab.get_attribute("aria-selected").as_deref() == Some("true");
        let _ = tab.set_attribute("tabindex", if selected { "0" } else { "-1" });
    }
}

/// Activate `target` within its tablist: update `aria-selected`, the `active`
/// class on tab + matching panel and roving tabindex. User tab interactions move
/// focus; automatic restoration can update selection without stealing focus.
#[cfg(target_arch = "wasm32")]
pub(super) fn activate_tab(
    target: &web_sys::HtmlElement,
    tabs: &[web_sys::HtmlElement],
    move_focus: bool,
) {
    let Ok(Some(tablist)) = target.closest("[role=tablist]") else {
        return;
    };
    // Native Hub callbacks own selection; fragment reveal may call this shared boundary.
    if tablist.has_attribute("data-native-hub-tabs") || !dom_tablist_owned(&tablist.class_name()) {
        return;
    }
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };

    for tab in tabs {
        let is_target = tab.is_same_node(Some(target.as_ref()));
        let _ = tab.set_attribute("aria-selected", if is_target { "true" } else { "false" });
        let _ = tab.set_attribute("tabindex", if is_target { "0" } else { "-1" });
        let class_list = tab.class_list();
        if is_target {
            let _ = class_list.add_1("active");
        } else {
            let _ = class_list.remove_1("active");
        }

        // Reflect the selection onto the controlled panel's `active` class so the
        // existing `.panel.active { display }` CSS reveals exactly one panel.
        if let Some(panel_id) = tab.get_attribute("aria-controls") {
            if let Some(panel) = document.get_element_by_id(&panel_id) {
                let panel_classes = panel.class_list();
                if is_target {
                    let _ = panel_classes.add_1("active");
                } else {
                    let _ = panel_classes.remove_1("active");
                }
            }
        }
    }

    if move_focus {
        let _ = target.focus();
    }
}

/// Attach a click activation handler to a tab (the tablists previously had no
/// pointer wiring, so this also makes them mouse-operable). The closure is kept
/// alive by the island registry and detached on unmount.
#[cfg(target_arch = "wasm32")]
pub(super) fn wire_tab_click(tab: &web_sys::HtmlElement, tabs: &[web_sys::HtmlElement]) {
    use wasm_bindgen::closure::Closure;

    let owned_tabs = tabs.to_vec();
    let tab_for_handler = tab.clone();
    let closure =
        Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
            activate_tab(&tab_for_handler, &owned_tabs, event.is_trusted());
        });

    register_island_listener(tab.clone().into(), "click", closure.into_js_value());
}

/// Attach the roving keydown handler to the tablist container (event delegation
/// over its tabs). Implements the manual-activation WAI-ARIA pattern: arrows and
/// Home/End move focus + the tab stop; Enter/Space activate the focused tab.
#[cfg(target_arch = "wasm32")]
pub(super) fn wire_tablist_keydown(tablist: &web_sys::Element, tabs: Vec<web_sys::HtmlElement>) {
    use wasm_bindgen::closure::Closure;

    let closure =
        Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
            if tabs.is_empty() {
                return;
            }

            // Locate the currently focused tab within this tablist.
            let active_element = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.active_element());
            let current = active_element.and_then(|active| {
                tabs.iter()
                    .position(|tab| tab.is_same_node(Some(active.as_ref())))
            });
            let Some(current) = current else {
                return;
            };

            let last = tabs.len() - 1;
            let target = match event.key().as_str() {
                "ArrowRight" | "ArrowDown" => Some(if current == last { 0 } else { current + 1 }),
                "ArrowLeft" | "ArrowUp" => Some(if current == 0 { last } else { current - 1 }),
                "Home" => Some(0),
                "End" => Some(last),
                "Enter" | " " | "Spacebar" => {
                    // Manual activation of the focused tab.
                    event.prevent_default();
                    activate_tab(&tabs[current], &tabs, true);
                    None
                }
                _ => None,
            };

            if let Some(index) = target {
                event.prevent_default();
                // Move focus + the roving tab stop without activating (manual model).
                for (position, tab) in tabs.iter().enumerate() {
                    let _ =
                        tab.set_attribute("tabindex", if position == index { "0" } else { "-1" });
                }
                let _ = tabs[index].focus();
            }
        });

    register_island_listener(tablist.clone().into(), "keydown", closure.into_js_value());
}
