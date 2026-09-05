#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use policy_bundle_content::{ContentIdentityError, PolicySource, content_digest};
use policy_pdp_kernel::{TemplateLink, TemplateSrc};
use shared_platform_contracts_kernel::pdp::EntityRef;

fn source() -> PolicySource {
    PolicySource {
        schema_src: "entity User; entity Doc; action \"read\" appliesTo { principal: [User], resource: [Doc] };".into(),
        policies_src: "@id(\"read-access\") @obligation(\"record-access\") permit(principal == User::\"alice\", action == Action::\"read\", resource);".into(),
        tenant_policies: BTreeMap::new(),
        templates: vec![],
        template_links: vec![],
        action_map: BTreeMap::from([("read".into(), "Action::\"read\"".into())]),
    }
}

#[test]
fn every_serving_input_participates_in_the_content_identity() {
    let original = source();
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
        principal: EntityRef {
            entity_type: "User".into(),
            entity_id: "alice".into(),
        },
        resource: EntityRef {
            entity_type: "Doc".into(),
            entity_id: "spec".into(),
        },
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
fn stable_fixture_identity_and_candidate_content_are_preserved() {
    let source = source();
    assert_eq!(
        source.content_version().unwrap().as_str(),
        "sha256:6ac21b95df1d3204e17bde8ed593d57f38eda8bd33fa9e7e5b3df5e294ec79d2"
    );
    assert_eq!(
        content_digest(b"oyatie-policy-source/v1\0", &source).unwrap(),
        "sha256:6ac21b95df1d3204e17bde8ed593d57f38eda8bd33fa9e7e5b3df5e294ec79d2"
    );
    let candidate = source.candidate().unwrap();
    assert_eq!(candidate.version, source.content_version().unwrap());
    assert_eq!(candidate.schema_src, source.schema_src);
    assert_eq!(candidate.policies_src, source.policies_src);
    assert_eq!(candidate.tenant_policies, source.tenant_policies);
    assert_eq!(candidate.templates, source.templates);
    assert_eq!(candidate.template_links, source.template_links);
    assert_eq!(candidate.action_map, source.action_map);
}

#[test]
fn generic_content_encoding_refusal_preserves_detail() {
    struct RefusesEncoding;
    impl serde::Serialize for RefusesEncoding {
        fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("fixture encoding refused"))
        }
    }

    assert!(matches!(
        content_digest(b"test-domain\0", &RefusesEncoding),
        Err(ContentIdentityError::Encoding { detail }) if detail == "fixture encoding refused"
    ));
}
