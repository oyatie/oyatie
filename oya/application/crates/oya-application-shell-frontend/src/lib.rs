#![recursion_limit = "512"]

pub mod app;
pub mod client_session_state;
pub mod design_system;
pub mod render_envelope;
pub mod shell_capability_registry;
#[cfg(any(feature = "ssr", test))]
pub mod token_broker;

pub use app::{App, DashboardIsland, shell_landmark_label, shell_scope_notice_text};
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
    use super::{shell_landmark_label, shell_scope_notice_text};

    #[test]
    fn scope_notice_names_transitional_integration_constraints_honestly() {
        let notice = shell_scope_notice_text();

        assert!(notice.contains("transitional in-process data"));
        assert!(notice.contains("live service integration"));
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

    #[test]
    fn static_dashboard_shows_ops_cluster_health_from_typed_api() {
        let html = crate::app::static_dashboard_html();

        assert!(html.contains("data-ops-cluster-health-source=\"typed-api\""));
        assert!(html.contains("GET /ops/v1/clusters/{cluster_id}/health"));
        assert!(html.contains("cell-us-east-2"));
        assert!(html.contains("role=\"status\" aria-live=\"polite\""));
        assert!(html.contains("class=\"ds-remediation-route\""));
        assert!(html.contains("yellow · observed 2026-07-01T05:00:00Z · typed ops API"));
        assert!(html.contains("traceparent fixture only"));
        assert!(!html.to_ascii_lowercase().contains("ssh"));
    }

    #[test]
    fn static_dashboard_exposes_developer_portal_approved_template_story() {
        let html = crate::app::static_dashboard_html();

        assert!(html.contains("Approved service template provisioning"));
        assert!(html.contains("Rust API + Leptos Shell Service"));
        assert!(html.contains("Quota and cost preview"));
        assert!(html.contains("Submit provisioning request"));
        assert!(html.contains("op-devportal-001a"));
        assert!(html.contains("developer_portal.provisioning.requested"));
        assert!(html.contains("policy denied fixture"));
        assert!(html.contains("Platform engineer"));
        assert!(html.contains("Security reviewer"));
        assert!(html.contains("Tenant admin"));
    }
}
