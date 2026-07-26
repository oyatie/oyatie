#![recursion_limit = "512"]

/// Stable DOM host that the WASM island replaces children within, never its own root node.
pub const DASHBOARD_MOUNT_HOST_ID: &str = "oya-dashboard-island-root";

pub mod app;
pub mod client_session_state;
pub mod design_system;
pub mod render_envelope;
#[cfg(any(feature = "ssr", test))]
pub mod server;
pub mod shell_capability_registry;
#[cfg(any(feature = "ssr", test))]
pub mod token_broker;

pub use app::{App, DashboardIsland, shell_landmark_label, shell_scope_notice_text};
#[cfg(any(feature = "ssr", test))]
pub use app::{render_envelope_json, static_dashboard_html};

#[cfg(all(target_arch = "wasm32", any(feature = "csr", feature = "hydrate")))]
pub fn mount_app() {
    mount_dashboard_islands();
}

#[cfg(all(target_arch = "wasm32", any(feature = "csr", feature = "hydrate")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn mount_dashboard_islands() {
    console_error_panic_hook::set_once();
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

    element.set_inner_html("");

    if let Ok(parent) = element.dyn_into::<web_sys::HtmlElement>() {
        leptos::mount::mount_to(parent, DashboardIsland).forget();
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
    fn scope_notice_names_production_contract_source_honestly() {
        let notice = shell_scope_notice_text();

        assert_eq!(
            notice,
            "Operator console scope: panels render from the production shell-BFF contract source with deny-by-default module visibility; no PHI/PII · shell covers close, workflow, people, mail, messenger, and community."
        );
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

        assert!(html.contains("id=\"oya-dashboard-island-root\""));
        assert!(html.contains("data-island=\"render-envelope-dashboard\""));
        assert!(html.contains("Selective WASM islands"));
        assert!(
            html.to_ascii_lowercase()
                .contains("production shell-bff contract source")
        );
    }
}
