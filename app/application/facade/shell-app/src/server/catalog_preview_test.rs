use super::*;
use leptos::prelude::*;

#[path = "catalog_preview_observer_test.rs"]
mod observer;
#[path = "catalog_preview_report_test.rs"]
mod report;

fn rendered_server() -> String {
    let owner = Owner::new();
    let html = owner.with(|| super::super::document::ServerShell().into_view().to_html());
    owner.cleanup();
    html
}

fn attribute(html: &str, name: &str) -> String {
    html.split_once(&format!("{name}=\""))
        .expect("native attribute")
        .1
        .split('"')
        .next()
        .unwrap()
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

#[test]
fn production_catalog_script_is_served_once_without_a_second_renderer() {
    let html = rendered_server();
    assert_eq!(html.matches(script().as_str()).count(), 1);
    assert_eq!(html.matches("data-catalog-root=\"true\"").count(), 1);
    assert!(!html.contains("Service catalog ready"));
    assert!(!html.contains("Cross-service routes</span><strong>7"));
    assert!(html.contains("signing in does not grant live Catalog authority"));
}

#[test]
fn served_catalog_admission_contract_matches_native_context_cards() {
    let generated = routes::script();
    let json = generated
        .split_once("const admittedIds = ")
        .unwrap()
        .1
        .split_once(";\n")
        .unwrap()
        .0;
    let admitted: std::collections::BTreeMap<String, Vec<String>> =
        serde_json::from_str(json).unwrap();
    for context in crate::render_envelope::OperatorContext::ALL {
        let expected: Vec<_> = crate::shell_capability_registry::permitted_module_cards(context)
            .iter()
            .map(|card| {
                let identities: Vec<_> = crate::shell_modules::PRODUCTION_MODULES
                    .iter()
                    .filter(|entry| entry.display_name == card.name && entry.group == card.group)
                    .collect();
                assert_eq!(identities.len(), 1);
                identities[0].capability_id.to_owned()
            })
            .collect();
        assert_eq!(admitted.get(context.id()), Some(&expected));
    }
    let html = rendered_server();
    let marker = html.find("data-catalog-link=\"gates\"").unwrap();
    let start = html[..marker].rfind("<a").unwrap();
    let anchor = html[start..].split_once('>').unwrap().0;
    assert!(anchor.contains("href=\"#resource-panel-gates\""));
    assert!(html.contains("id=\"resource-panel-gates\""));
}

#[test]
fn served_connected_catalog_links_have_unique_empty_reason_descriptions() {
    let html = rendered_server();
    let links: Vec<_> = html.match_indices("data-catalog-link=\"").collect();
    assert_eq!(links.len(), 13);
    let reasons = html
        .split("<small ")
        .skip(1)
        .map(|fragment| fragment.split_once('>').unwrap().0)
        .filter(|attributes| attributes.contains("data-catalog-link-reason=\""))
        .count();
    assert_eq!(reasons, links.len());
    for (marker, _) in links {
        let start = html[..marker].rfind("<a").unwrap();
        let anchor = html[start..].split_once('>').unwrap().0;
        let key = attribute(anchor, "data-catalog-link");
        let reason_id = format!("catalog-link-unavailable-{key}");
        assert_eq!(
            attribute(anchor, "aria-describedby"),
            format!("catalog-availability {reason_id}")
        );
        let reason_marker = format!("data-catalog-link-reason=\"{key}\"");
        assert_eq!(html.matches(&reason_marker).count(), 1);
        assert_eq!(html.matches(&format!("id=\"{reason_id}\"")).count(), 1);
        let reason_position = html.find(&reason_marker).unwrap();
        let reason_start = html[..reason_position].rfind("<small").unwrap();
        let (reason, content) = html[reason_start..].split_once('>').unwrap();
        assert_eq!(attribute(reason, "id"), reason_id);
        assert!(reason.contains("hidden"));
        assert!(content.split_once("</small>").unwrap().0.is_empty());
    }
}

#[test]
fn canonical_catalog_source_contains_only_the_native_context_cards() {
    let html = rendered_server();
    assert_eq!(attribute(&html, "data-catalog-admitted"), "true");
    assert_eq!(attribute(&html, "data-catalog-available"), "true");
    let source: serde_json::Value =
        serde_json::from_str(&attribute(&html, "data-catalog-source")).unwrap();
    let context = crate::render_envelope::OperatorContext::TenantAdmin;
    let envelope = crate::render_envelope::server_derived_envelope(context);
    assert_eq!(source[0], context.id());
    let records = source[3].as_array().unwrap();
    assert_eq!(records.len(), envelope.modules.len());
    assert_eq!(records.len(), 7);
    assert_eq!(html.matches("data-catalog-module=\"").count(), 7);
    let ontology = records
        .iter()
        .find(|record| record[0] == "foundry-ontology")
        .unwrap();
    assert_eq!(
        ontology[1][3][1],
        "Object types, instances, actions, and their audit trail · Preview only · live ontology status requires a verified session"
    );
    for (record, module) in records.iter().zip(envelope.modules.iter()) {
        let registered = crate::shell_modules::PRODUCTION_MODULES
            .iter()
            .find(|entry| entry.display_name == module.name && entry.group == module.group)
            .unwrap();
        assert_eq!(record[0], registered.capability_id);
        assert_eq!(record[1][1][1], module.name);
        assert_eq!(record[1][2][1], module.group);
        assert_eq!(record[1][3][1], module.description);
        assert_eq!(record[1][4][1], module.action_label);
    }
    let compatibility = crate::app::static_dashboard_html();
    assert_eq!(
        attribute(&compatibility, "data-catalog-source"),
        source.to_string()
    );
}

#[test]
fn privileged_catalog_controls_are_described_and_disabled_in_served_html() {
    let html = rendered_server();
    for effect in ["request", "stage"] {
        let marker = format!("data-catalog-unavailable=\"{effect}\"");
        for occurrence in html.match_indices(&marker) {
            let button = &html[..occurrence.0];
            let attributes = button.rsplit_once("<button").unwrap().1;
            assert!(attributes.contains("disabled"));
            let end = &html[occurrence.0..].split_once('>').unwrap().0;
            assert!(end.contains("aria-describedby=\"catalog-unavailable-effects\""));
        }
        assert!(html.contains(&marker));
    }
    assert!(html.contains("No request, receipt, file or durable pin is created"));
}

#[test]
fn ssr_catalog_channels_are_complete_matching_native_templates() {
    let html = rendered_server();
    let hub_source = attribute(&html, "data-catalog-hub-source");
    let _: serde_json::Value = serde_json::from_str(&hub_source).unwrap();
    for channel in ["Messenger", "Mail", "Community"] {
        let marker = format!("data-catalog-hub-template=\"{channel}\"");
        let template = html
            .split_once(&marker)
            .unwrap()
            .1
            .split_once("</template>")
            .unwrap()
            .0;
        assert_eq!(attribute(template, "data-hub-source"), hub_source);
        assert!(template.contains("data-hub-mode=\"ssr\""));
        assert!(template.contains("class=\"hub-list\""));
        assert!(template.contains("class=\"hub-detail\""));
        assert!(template.contains("class=\"hub-composer\""));
        assert!(template.contains(&format!("data-comms-board-surface=\"{channel}\"")));
        assert!(template.contains(&format!("id=\"hub-panel-{}\"", channel.to_lowercase())));
    }
}
