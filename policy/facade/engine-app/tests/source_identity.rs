#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod support;

use policy_pdp_kernel::{TemplateLink, TemplateSrc};
use support::*;

#[test]
fn every_serving_input_participates_in_the_content_identity() {
    let original = project().source;
    let version = original.content_version().unwrap();
    let mut variants = vec![];
    let mut changed = original.clone();
    changed.schema_src.push('\n');
    variants.push(changed);
    let mut changed = original.clone();
    changed.policies_src.push('\n');
    variants.push(changed);
    let mut changed = original.clone();
    changed.tenant_policies.insert(
        "tenant".into(),
        "permit(principal, action, resource);".into(),
    );
    variants.push(changed);
    let mut changed = original.clone();
    changed.templates.push(TemplateSrc {
        template_id: "grant".into(),
        src: "permit(principal == ?principal, action, resource == ?resource);".into(),
    });
    variants.push(changed);
    let mut changed = original.clone();
    changed.template_links.push(TemplateLink {
        template_id: "grant".into(),
        link_id: "grant-alice".into(),
        principal: entity("User", "alice"),
        resource: entity("Doc", "spec"),
    });
    variants.push(changed);
    let mut changed = original.clone();
    changed
        .action_map
        .insert("alias".into(), "Action::\"read\"".into());
    variants.push(changed);
    for variant in variants {
        assert_ne!(variant.content_version().unwrap(), version);
    }
}

#[test]
fn json_order_does_not_change_identity_but_case_changes_change_qualification_receipt() {
    let candidate = project();
    let reordered = serde_json::to_value(&candidate).unwrap();
    let decoded: policy_engine_app::PolicyProject = serde_json::from_value(reordered).unwrap();
    assert_eq!(
        decoded.source.content_version().unwrap(),
        candidate.source.content_version().unwrap()
    );
    let first = candidate.prepare(ids()).unwrap();
    let mut renamed = project();
    renamed.cases[0].name = "renamed case".into();
    let second = renamed.prepare(ids()).unwrap();
    assert_eq!(first.bundle().version, second.bundle().version);
    assert_ne!(
        first.report().qualification_digest,
        second.report().qualification_digest
    );
}
