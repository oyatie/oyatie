#![recursion_limit = "512"]

pub mod app;
pub mod client_session_state;
pub mod render_envelope;
#[cfg(feature = "ssr")]
pub mod server_mock_catalog;

pub use app::{App, DashboardIsland, prototype_notice_text, shell_landmark_label};
#[cfg(feature = "ssr")]
pub use app::{render_envelope_json, static_dashboard_html};

#[cfg(all(target_arch = "wasm32", any(feature = "csr", feature = "hydrate")))]
pub fn mount_app() {
    mount_dashboard_islands();
}

#[cfg(all(target_arch = "wasm32", any(feature = "csr", feature = "hydrate")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn mount_dashboard_islands() {
    console_error_panic_hook::set_once();
    mount_dashboard_island_by_id("oya-dashboard-island-root");
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
    use super::{prototype_notice_text, shell_landmark_label};

    #[test]
    fn prototype_notice_names_mock_and_demo_constraints() {
        let notice = prototype_notice_text();

        assert!(notice.contains("Prototype/demo only"));
        assert!(notice.contains("no backend"));
        assert!(notice.contains("no real auth"));
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
    }
}
