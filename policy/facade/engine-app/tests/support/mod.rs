#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use policy_authoring_app::{DecisionExpectation, PolicyCase, PolicyProject, PolicySource};
use policy_pdp_bundle_file::FilePolicyBundleStore;
use policy_pdp_kernel::{EntityRecord, EntitySlice};
use shared_audit_digest_adapter_awslc::Ed25519ChainSigner;
use shared_audit_event_kernel::encode_hex;
use shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, Decision, EntityRef, Obligation,
};
use shared_ulid_id_kernel::SeededIdGenerator;

pub fn ids() -> Arc<SeededIdGenerator> {
    Arc::new(SeededIdGenerator::default())
}

pub fn entity(kind: &str, id: &str) -> EntityRef {
    EntityRef {
        entity_type: kind.into(),
        entity_id: id.into(),
    }
}

pub fn request(id: &str) -> AuthorizationRequest {
    AuthorizationRequest {
        request_id: format!("request-{id}"),
        tenant_id: "tenant".into(),
        principal: entity("User", id),
        action: "read".into(),
        resource: entity("Doc", "spec"),
        context: BTreeMap::new(),
        min_policy_version: None,
    }
}

pub fn context_entities() -> Vec<EntityRecord> {
    [entity("Group", "eng"), entity("Doc", "spec")]
        .into_iter()
        .map(|uid| EntityRecord {
            uid,
            attributes: BTreeMap::new(),
            parents: vec![],
        })
        .collect()
}

pub fn entities(id: &str) -> EntitySlice {
    let mut entities = context_entities();
    entities.push(EntityRecord {
        uid: entity("User", id),
        attributes: BTreeMap::new(),
        parents: if id == "alice" {
            vec![entity("Group", "eng")]
        } else {
            vec![]
        },
    });
    EntitySlice { entities }
}

pub fn project() -> PolicyProject {
    PolicyProject {
        source: PolicySource {
            schema_src: "entity Group; entity User in [Group]; entity Doc; action \"read\" appliesTo { principal: [User], resource: [Doc] };".into(),
            policies_src: "@id(\"readers\") @obligation(\"record-access\") permit(principal in Group::\"eng\", action == Action::\"read\", resource);".into(),
            tenant_policies: BTreeMap::new(), templates: vec![], template_links: vec![],
            action_map: BTreeMap::from([("read".into(), "Action::\"read\"".into())]),
        },
        cases: vec![
            PolicyCase {
                name: "member may read".into(), request: request("alice"), entities: entities("alice"),
                expected: DecisionExpectation {
                    decision: Decision::Allow, determining_policy_ids: vec!["readers".into()],
                    obligations: vec![Obligation { obligation_id: "record-access".into(), parameters: BTreeMap::new() }],
                },
            },
            PolicyCase {
                name: "nonmember denied".into(), request: request("bob"), entities: entities("bob"),
                expected: DecisionExpectation { decision: Decision::Deny, determining_policy_ids: vec![], obligations: vec![] },
            },
        ],
    }
}

static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);
pub struct Fixture {
    pub root: PathBuf,
    pub store: FilePolicyBundleStore,
    pub signer: Ed25519ChainSigner,
}

impl Fixture {
    pub fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "policy-engine-test-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let trust = root.join("trust");
        std::fs::create_dir(&trust).unwrap();
        let signer = Ed25519ChainSigner::generate("publication").unwrap();
        std::fs::write(
            trust.join("publication.pub"),
            encode_hex(&signer.public_key_bytes()),
        )
        .unwrap();
        let store = FilePolicyBundleStore::new(root.join("bundle.json"), trust);
        Self {
            root,
            store,
            signer,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
