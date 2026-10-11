#![recursion_limit = "512"]

/// Stable DOM host that the WASM island replaces children within, never its own root node.
pub const DASHBOARD_MOUNT_HOST_ID: &str = "dashboard-island-root";

pub mod app;
#[cfg(all(target_arch = "wasm32", any(feature = "csr", feature = "hydrate")))]
mod browser_panic_hook;
pub mod client_session_state;
pub mod design_system;
pub mod render_envelope;
#[cfg(any(feature = "ssr", test))]
pub mod server;
pub mod shell_capability_registry;
mod shell_context_grants;
mod shell_modules;
#[cfg(any(feature = "ssr", test))]
pub mod token_broker;

pub use app::{App, DashboardIsland, shell_landmark_label, shell_scope_notice_text};
#[cfg(any(feature = "ssr", test))]
pub use app::{render_envelope_json, static_dashboard_html};

#[cfg(all(target_arch = "wasm32", any(feature = "csr", feature = "hydrate")))]
thread_local! {
    static DASHBOARD_UNMOUNT: std::cell::RefCell<Option<Box<dyn FnOnce()>>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(all(target_arch = "wasm32", any(feature = "csr", feature = "hydrate")))]
pub fn mount_app() {
    mount_dashboard_islands();
}

#[cfg(all(target_arch = "wasm32", any(feature = "csr", feature = "hydrate")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn mount_dashboard_islands() {
    browser_panic_hook::set_once();
    mount_dashboard_island_by_id(DASHBOARD_MOUNT_HOST_ID);
}

#[cfg(all(target_arch = "wasm32", any(feature = "csr", feature = "hydrate")))]
fn mount_dashboard_island_by_id(element_id: &str) {
    use wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(element) = document.get_element_by_id(element_id) else {
        return;
    };

    if let Ok(parent) = element.dyn_into::<web_sys::HtmlElement>() {
        if let Some(unmount) = DASHBOARD_UNMOUNT.with(|previous| previous.borrow_mut().take()) {
            unmount();
        }
        parent.set_inner_html("");
        let owner = leptos::prelude::Owner::new();
        let mount = owner.with(|| leptos::mount::mount_to(parent, DashboardIsland));
        DASHBOARD_UNMOUNT.with(|previous| {
            *previous.borrow_mut() = Some(Box::new(move || {
                drop(mount);
                owner.cleanup();
            }));
        });
    }
}

#[cfg(all(target_arch = "wasm32", feature = "hydrate"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    mount_dashboard_islands();
}

#[cfg(test)]
mod tests {
    use super::{shell_landmark_label, shell_scope_notice_text};

    #[test]
    fn scope_notice_names_the_anonymous_preview_honestly() {
        let notice = shell_scope_notice_text();

        assert!(notice.contains("Anonymous preview only"));
        assert!(notice.contains("No tenant or role was verified"));
        assert!(notice.contains("live service status was not queried"));
        assert!(notice.contains("requires sign-in"));
        assert!(notice.contains("no PHI/PII"));
    }

    #[test]
    fn shell_landmark_label_is_specific_to_control_center() {
        let label = shell_landmark_label();

        assert!(label.contains("Oyatie"));
        assert!(label.contains("Cloud/Tenant Control Center"));
    }

    #[test]
    fn static_dashboard_names_selective_island_boundary() {
        let html = crate::app::static_dashboard_html();

        assert!(html.contains("id=\"dashboard-island-root\""));
        assert!(html.contains("data-island=\"render-envelope-dashboard\""));
        assert!(html.contains("Selective WASM islands"));
        assert!(html.to_ascii_lowercase().contains("anonymous preview only"));
    }
}
