//! G004 conformance suite: the embedded Cedar PDP evaluated against the
//! LOCKED FD-001 seed bundle (schema + structural tenant-isolation forbid +
//! RBAC/ABAC policies + PBAC template) through the vendor-neutral
//! `PolicyDecisionPoint` port, on the REAL cedar-policy engine.
//!
//! Test ladder rungs covered here (AMENDMENT 7): contract conformance
//! (locked PDP request/response shapes) + integration against the real
//! engine substrate. The cedar/ fixtures are crate-local copies of the G001
//! contract-lock seeds, guarded against drift by
//! `crate_local_cedar_seeds_match_canonical` (same pattern as the
//! backbone-proto parity test).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use oya_audit_chain_domain::{
    AuditAppendInput, AuditChain, AuditChainError, Ed25519SigningKey, Ed25519VerificationKeySet,
    Plane, append as audit_append,
};
use oya_audit_chain_file_adapter::FileAuditLedger;
use oya_data_boundary_kernel::{DataClass, Purpose};
use oya_shared_pdp_adapter_cedar::{
    AuditChainCedarPdp, CedarPdp, PDP_DECISION_AUDIT_SURFACE, PdpAuditChainError,
    PdpDecisionAuditChainLogger,
};
use oya_shared_pdp_kernel::{
    EntityRecord, EntitySlice, PdpError, PolicyBundle, PolicyDecisionPoint, TemplateLink,
    TemplateSrc,
};
use oya_shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, Decision, EntityRef, PolicyVersion,
};
use oya_shared_ulid_id_kernel::SeededIdGenerator;

const SCHEMA_SRC: &str = include_str!("../cedar/platform.cedarschema");
const POLICIES_SRC: &str = include_str!("../cedar/platform-policies.cedar");
const TEMPLATE_SRC: &str = include_str!("../cedar/platform-templates.cedar");

const TEMPLATE_ID: &str = "pbac-resource-read-grant";

fn entity_ref(entity_type: &str, entity_id: &str) -> EntityRef {
    EntityRef {
        entity_type: entity_type.to_owned(),
        entity_id: entity_id.to_owned(),
    }
}

fn action_map() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "resource.read".to_owned(),
            r#"OyaPlatform::Action::"ReadResource""#.to_owned(),
        ),
        (
            "resource.write".to_owned(),
            r#"OyaPlatform::Action::"WriteResource""#.to_owned(),
        ),
        (
            "tenant.administer".to_owned(),
            r#"OyaPlatform::Action::"AdministerTenant""#.to_owned(),
        ),
    ])
}

fn locked_seed_bundle(version: &str, template_links: Vec<TemplateLink>) -> PolicyBundle {
    PolicyBundle {
        version: PolicyVersion::new(version).unwrap(),
        schema_src: SCHEMA_SRC.to_owned(),
        policies_src: POLICIES_SRC.to_owned(),
        templates: vec![TemplateSrc {
            template_id: TEMPLATE_ID.to_owned(),
            src: TEMPLATE_SRC.to_owned(),
        }],
        template_links,
        action_map: action_map(),
    }
}

/// Two tenants in two cells; one cross-tenant-polluted admin group — the
/// exact misconfiguration the structural forbid must neutralize (mirrors the
/// locked contract-kernel validation fixture).
fn entity_slice() -> EntitySlice {
    let string_attrs = |pairs: &[(&str, &str)]| -> BTreeMap<String, serde_json::Value> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), serde_json::json!(v)))
            .collect()
    };
    EntitySlice {
        entities: vec![
            EntityRecord {
                uid: entity_ref("OyaPlatform::Tenant", "acme"),
                attributes: string_attrs(&[
                    ("tenant_id", "acme"),
                    ("cell_id", "cell-001"),
                    ("lifecycle_state", "active"),
                ]),
                parents: vec![],
            },
            EntityRecord {
                uid: entity_ref("OyaPlatform::Tenant", "globex"),
                attributes: string_attrs(&[
                    ("tenant_id", "globex"),
                    ("cell_id", "cell-002"),
                    ("lifecycle_state", "active"),
                ]),
                parents: vec![],
            },
            EntityRecord {
                uid: entity_ref("OyaPlatform::Group", "tenant-admins"),
                attributes: string_attrs(&[("tenant_id", "acme")]),
                parents: vec![entity_ref("OyaPlatform::Tenant", "acme")],
            },
            EntityRecord {
                uid: entity_ref("OyaPlatform::Principal", "alice"),
                attributes: string_attrs(&[
                    ("tenant_id", "acme"),
                    ("kind", "human"),
                    ("step_up_class", "a"),
                ]),
                parents: vec![entity_ref("OyaPlatform::Group", "tenant-admins")],
            },
            // bob: acme principal WITHOUT step-up class and WITHOUT group
            // membership — only a PBAC template link can let him read.
            EntityRecord {
                uid: entity_ref("OyaPlatform::Principal", "bob"),
                attributes: string_attrs(&[("tenant_id", "acme"), ("kind", "human")]),
                parents: vec![],
            },
            // mallory belongs to ANOTHER tenant but is (mis)joined to the
            // same group — the structural forbid must still deny everything.
            EntityRecord {
                uid: entity_ref("OyaPlatform::Principal", "mallory"),
                attributes: string_attrs(&[
                    ("tenant_id", "globex"),
                    ("kind", "human"),
                    ("step_up_class", "a"),
                ]),
                parents: vec![entity_ref("OyaPlatform::Group", "tenant-admins")],
            },
            EntityRecord {
                uid: entity_ref("OyaPlatform::WorkloadIdentity", "payments"),
                attributes: string_attrs(&[
                    ("tenant_id", "acme"),
                    ("spiffe_id", "spiffe://oyatie/acme/payments"),
                ]),
                parents: vec![entity_ref("OyaPlatform::Tenant", "acme")],
            },
            EntityRecord {
                uid: entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
                attributes: string_attrs(&[
                    ("tenant_id", "acme"),
                    ("resource_kind", "document"),
                    ("data_class", "restricted"),
                    ("cell_id", "cell-001"),
                ]),
                parents: vec![entity_ref("OyaPlatform::Tenant", "acme")],
            },
        ],
    }
}

fn request(
    request_id: &str,
    tenant_id: &str,
    principal: EntityRef,
    action: &str,
    resource: EntityRef,
) -> AuthorizationRequest {
    AuthorizationRequest {
        request_id: request_id.to_owned(),
        tenant_id: tenant_id.to_owned(),
        principal,
        action: action.to_owned(),
        resource,
        context: BTreeMap::new(),
        min_policy_version: None,
    }
}

fn pdp(links: Vec<TemplateLink>) -> CedarPdp {
    CedarPdp::load(
        &locked_seed_bundle("psv-000001", links),
        Arc::new(SeededIdGenerator::default()),
        64,
    )
    .expect("locked seed bundle must load")
}

fn unique_ledger_path(test_name: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("test-side: system clock must be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "oya-pdp-audit-chain-{test_name}-{}-{now}.ledger",
        std::process::id()
    ))
}

fn audit_input(tenant_id: &str, decision: &str) -> AuditAppendInput {
    AuditAppendInput {
        tenant_id: tenant_id.to_owned(),
        surface: PDP_DECISION_AUDIT_SURFACE.to_owned(),
        plane: Plane::Control,
        purpose: Purpose::CoreService,
        data_classes: vec![DataClass::InternalOnly, DataClass::Audit],
        decision: decision.to_owned(),
    }
}

// ---------------------------------------------------------------- RBAC ----

#[test]
fn rbac_group_grant_allows_tenant_admin_within_tenant() {
    let pdp = pdp(vec![]);
    let outcome = pdp
        .authorize(
            &request(
                "req-rbac-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "tenant.administer",
                entity_ref("OyaPlatform::Tenant", "acme"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(outcome.response.decision, Decision::Allow);
    assert_eq!(
        outcome.response.determining_policy_ids,
        vec!["rbac-tenant-admin-group".to_owned()],
        "every allow is attributable to its permit policy"
    );
}

#[test]
fn rbac_non_member_is_denied_by_default() {
    let pdp = pdp(vec![]);
    let outcome = pdp
        .authorize(
            &request(
                "req-rbac-2",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "tenant.administer",
                entity_ref("OyaPlatform::Tenant", "acme"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(outcome.response.decision, Decision::Deny);
}

// ---------------------------------------------------------------- ABAC ----

#[test]
fn abac_step_up_class_gates_restricted_reads() {
    let pdp = pdp(vec![]);
    // alice asserted step-up class "a": allow, attributed to the ABAC policy.
    let allowed = pdp
        .authorize(
            &request(
                "req-abac-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(allowed.response.decision, Decision::Allow);
    assert_eq!(
        allowed.response.determining_policy_ids,
        vec!["abac-step-up-restricted-read".to_owned()]
    );
    // bob has no step_up_class attribute: deny-by-default.
    let denied = pdp
        .authorize(
            &request(
                "req-abac-2",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(denied.response.decision, Decision::Deny);
}

// ---------------------------------------------------------------- PBAC ----

#[test]
fn pbac_template_link_grants_scoped_read() {
    let link = TemplateLink {
        template_id: TEMPLATE_ID.to_owned(),
        link_id: "pbac-link-bob-doc1".to_owned(),
        principal: entity_ref("OyaPlatform::Principal", "bob"),
        resource: entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    };
    // Without the link: deny-by-default (proved by the ABAC test above).
    let pdp = pdp(vec![link]);
    let outcome = pdp
        .authorize(
            &request(
                "req-pbac-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(outcome.response.decision, Decision::Allow);
    assert_eq!(
        outcome.response.determining_policy_ids,
        vec!["pbac-link-bob-doc1".to_owned()],
        "the allow is attributable to the template instantiation"
    );
}

// ---------------------------------------- structural tenant isolation ----

#[test]
fn structural_forbid_overrides_cross_tenant_group_membership() {
    let pdp = pdp(vec![]);
    for (req_id, action, resource) in [
        (
            "req-iso-1",
            "tenant.administer",
            entity_ref("OyaPlatform::Tenant", "acme"),
        ),
        (
            "req-iso-2",
            "resource.read",
            entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
        ),
    ] {
        let outcome = pdp
            .authorize(
                &request(
                    req_id,
                    "globex",
                    entity_ref("OyaPlatform::Principal", "mallory"),
                    action,
                    resource,
                ),
                &entity_slice(),
            )
            .unwrap();
        assert_eq!(
            outcome.response.decision,
            Decision::Deny,
            "{req_id}: structural forbid must override every permit"
        );
    }
}

#[test]
fn structural_forbid_overrides_misissued_cross_tenant_template_link() {
    let link = TemplateLink {
        template_id: TEMPLATE_ID.to_owned(),
        link_id: "pbac-link-mallory-doc1".to_owned(),
        principal: entity_ref("OyaPlatform::Principal", "mallory"),
        resource: entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    };
    let pdp = pdp(vec![link]);
    let outcome = pdp
        .authorize(
            &request(
                "req-iso-3",
                "globex",
                entity_ref("OyaPlatform::Principal", "mallory"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(
        outcome.response.decision,
        Decision::Deny,
        "a mis-issued cross-tenant grant can never defeat the structural forbid"
    );
}

// ------------------------------------------------- zookie freshness ----

#[test]
fn stale_policy_version_pin_is_refused_not_answered() {
    let pdp = pdp(vec![]);
    let mut req = request(
        "req-zookie-1",
        "acme",
        entity_ref("OyaPlatform::Principal", "alice"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    );
    req.min_policy_version = Some(PolicyVersion::new("psv-000002").unwrap());
    let err = pdp.authorize(&req, &entity_slice()).unwrap_err();
    assert!(
        matches!(err, PdpError::StalePolicyVersion { .. }),
        "expected stale-version refusal, got {err:?}"
    );
    // Pinning the LOADED version succeeds (read-your-writes satisfied).
    req.min_policy_version = Some(pdp.loaded_policy_version());
    assert!(pdp.authorize(&req, &entity_slice()).is_ok());
}

// --------------------------------------------- decision cache + swap ----

#[test]
fn cache_replays_decision_content_with_fresh_decision_ids() {
    let pdp = pdp(vec![]);
    let req = request(
        "req-cache-1",
        "acme",
        entity_ref("OyaPlatform::Principal", "alice"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    );
    let first = pdp.authorize(&req, &entity_slice()).unwrap();
    assert!(!first.cache_hit);
    let mut replay = req.clone();
    replay.request_id = "req-cache-2".to_owned();
    let second = pdp.authorize(&replay, &entity_slice()).unwrap();
    assert!(second.cache_hit, "identical decision surface must hit");
    assert_eq!(second.response.decision, first.response.decision);
    assert_eq!(
        second.response.determining_policy_ids,
        first.response.determining_policy_ids
    );
    assert_ne!(
        second.response.decision_id, first.response.decision_id,
        "every decision — cached or not — gets its own audit-chain key"
    );
    assert_eq!(second.response.request_id, "req-cache-2");
}

#[test]
fn bundle_swap_revokes_immediately_and_disarms_prior_cache() {
    let link = TemplateLink {
        template_id: TEMPLATE_ID.to_owned(),
        link_id: "pbac-link-bob-doc1".to_owned(),
        principal: entity_ref("OyaPlatform::Principal", "bob"),
        resource: entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    };
    let pdp = pdp(vec![link]);
    let req = request(
        "req-revoke-1",
        "acme",
        entity_ref("OyaPlatform::Principal", "bob"),
        "resource.read",
        entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
    );
    let before = pdp.authorize(&req, &entity_slice()).unwrap();
    assert_eq!(before.response.decision, Decision::Allow);
    // Revocation: the policy store pushes a bundle WITHOUT the grant. The
    // swap is atomic; the version change makes every cached allow
    // unreachable (sub-60s revocation = bundle propagation latency).
    pdp.swap_bundle(&locked_seed_bundle("psv-000002", vec![]))
        .unwrap();
    let mut after_req = req.clone();
    after_req.request_id = "req-revoke-2".to_owned();
    let after = pdp.authorize(&after_req, &entity_slice()).unwrap();
    assert_eq!(after.response.decision, Decision::Deny);
    assert!(!after.cache_hit, "prior cache entries must be unreachable");
    assert_eq!(after.response.policy_version.as_str(), "psv-000002");
}

#[test]
fn rejected_bundle_never_replaces_a_serving_one() {
    let pdp = pdp(vec![]);
    let mut broken = locked_seed_bundle("psv-000099", vec![]);
    broken.policies_src =
        "permit (principal, action, resource) when { principal.nonexistent_attr == \"x\" };"
            .to_owned();
    let err = pdp.swap_bundle(&broken).unwrap_err();
    assert!(matches!(err, PdpError::BundleRejected { .. }));
    assert_eq!(
        pdp.loaded_policy_version().as_str(),
        "psv-000001",
        "the serving bundle must keep serving after a rejected swap"
    );
}

// --------------------------------------------------- audit + errors ----

#[test]
fn every_decision_yields_an_attributable_audit_record() {
    let pdp = pdp(vec![]);
    let allow = pdp
        .authorize(
            &request(
                "req-audit-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(allow.audit.decision_id, allow.response.decision_id);
    assert_eq!(allow.audit.decision, Decision::Allow);
    assert_eq!(allow.audit.tenant_id, "acme");
    assert_eq!(allow.audit.policy_version.as_str(), "psv-000001");
    assert!(!allow.audit.determining_policy_ids.is_empty());

    let deny = pdp
        .authorize(
            &request(
                "req-audit-2",
                "acme",
                entity_ref("OyaPlatform::Principal", "bob"),
                "resource.write",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(deny.audit.decision, Decision::Deny);
    assert_ne!(deny.audit.decision_id, allow.audit.decision_id);
}

#[test]
fn audited_pdp_persists_signed_audit_chain_event_with_decision_lineage() {
    let ledger_path = unique_ledger_path("decision-lineage");
    let signer = Ed25519SigningKey::from_seed_bytes("pdp-audit-test-key", [7_u8; 32]).unwrap();
    let trusted_keys = Ed25519VerificationKeySet::single(signer.verification_key()).unwrap();
    let logger = PdpDecisionAuditChainLogger::new(
        FileAuditLedger::new(ledger_path.clone()),
        signer,
        trusted_keys.clone(),
    )
    .unwrap();
    let pdp = AuditChainCedarPdp::load(
        &locked_seed_bundle("psv-000001", vec![]),
        Arc::new(SeededIdGenerator::default()),
        64,
        logger,
    )
    .unwrap();

    let outcome = pdp
        .authorize(
            &request(
                "req-audit-chain-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();

    let persisted = FileAuditLedger::new(ledger_path.clone())
        .load_with_trusted_keys(&trusted_keys)
        .unwrap();
    let events = persisted.events();
    assert_eq!(
        events.len(),
        1,
        "exactly one durable audit-chain event is appended"
    );
    let event = &events[0];
    assert_eq!(event.tenant_id, "acme");
    assert_eq!(event.surface, PDP_DECISION_AUDIT_SURFACE);
    assert_eq!(event.plane, Plane::Control);
    assert_eq!(event.purpose, Purpose::CoreService);
    assert_eq!(
        event.data_classes,
        vec![DataClass::InternalOnly, DataClass::Audit]
    );
    assert!(
        event.ed25519_signature.is_some(),
        "audit-chain event must be signed"
    );

    let lineage: serde_json::Value = serde_json::from_str(&event.decision).unwrap();
    assert_eq!(lineage["decision_id"], outcome.response.decision_id);
    assert_eq!(lineage["request_id"], "req-audit-chain-1");
    assert_eq!(lineage["tenant_id"], "acme");
    assert_eq!(lineage["resource"]["entity_id"], "acme-doc-1");
    assert_eq!(lineage["action"], "resource.read");
    assert_eq!(lineage["decision"], "allow");
    assert_eq!(lineage["policy_version"], "psv-000001");

    std::fs::remove_file(ledger_path).ok();
}

#[test]
fn audited_pdp_refuses_unsigned_prior_multi_tenant_ledger_replay() {
    let ledger_path = unique_ledger_path("unsigned-prior-ledger");
    let ledger = FileAuditLedger::new(ledger_path.clone());
    let mut chain = AuditChain::multi_tenant_shards();
    audit_append(&mut chain, audit_input("acme", "preexisting-acme"), None).unwrap();
    audit_append(
        &mut chain,
        audit_input("globex", "preexisting-globex"),
        None,
    )
    .unwrap();
    ledger.append_chain(&chain).unwrap();

    let signer = Ed25519SigningKey::from_seed_bytes("pdp-audit-test-key", [7_u8; 32]).unwrap();
    let trusted_keys = Ed25519VerificationKeySet::single(signer.verification_key()).unwrap();
    let err = PdpDecisionAuditChainLogger::new(ledger, signer, trusted_keys).unwrap_err();

    assert!(
        matches!(
            err,
            PdpAuditChainError::TrustedSignatureReplay(
                AuditChainError::MissingEd25519Signature { .. }
            )
        ),
        "unsigned prior ledger must fail closed at startup, got {err:?}"
    );

    std::fs::remove_file(ledger_path).ok();
}

#[test]
fn audited_pdp_refuses_attacker_signed_prior_multi_tenant_ledger_replay() {
    let ledger_path = unique_ledger_path("attacker-signed-prior-ledger");
    let ledger = FileAuditLedger::new(ledger_path.clone());
    let trusted_signer =
        Ed25519SigningKey::from_seed_bytes("pdp-audit-test-key", [7_u8; 32]).unwrap();
    let attacker_signer =
        Ed25519SigningKey::from_seed_bytes("attacker-pdp-audit-key", [9_u8; 32]).unwrap();
    let mut chain = AuditChain::multi_tenant_shards();
    audit_append(
        &mut chain,
        audit_input("acme", "attacker-signed-acme"),
        Some(&attacker_signer),
    )
    .unwrap();
    audit_append(
        &mut chain,
        audit_input("globex", "attacker-signed-globex"),
        Some(&attacker_signer),
    )
    .unwrap();
    ledger.append_chain(&chain).unwrap();

    let trusted_keys =
        Ed25519VerificationKeySet::single(trusted_signer.verification_key()).unwrap();
    let err = PdpDecisionAuditChainLogger::new(ledger, trusted_signer, trusted_keys).unwrap_err();

    assert!(
        matches!(
            err,
            PdpAuditChainError::TrustedSignatureReplay(
                AuditChainError::MissingTrustedEd25519Key { .. }
            )
        ),
        "attacker-signed prior ledger must fail closed at startup, got {err:?}"
    );

    std::fs::remove_file(ledger_path).ok();
}

#[test]
fn audited_pdp_refuses_serving_signer_when_trusted_key_material_differs() {
    let ledger_path = unique_ledger_path("untrusted-serving-signer");
    let signer = Ed25519SigningKey::from_seed_bytes("pdp-audit-test-key", [7_u8; 32]).unwrap();
    let key_id_collision =
        Ed25519SigningKey::from_seed_bytes("pdp-audit-test-key", [9_u8; 32]).unwrap();
    let trusted_keys =
        Ed25519VerificationKeySet::single(key_id_collision.verification_key()).unwrap();
    let err = PdpDecisionAuditChainLogger::new(
        FileAuditLedger::new(ledger_path.clone()),
        signer,
        trusted_keys,
    )
    .unwrap_err();

    assert!(
        matches!(
            err,
            PdpAuditChainError::UntrustedSigner(
                AuditChainError::Ed25519SignatureKeyMismatch { .. }
            )
        ),
        "serving signer must match trusted key material, got {err:?}"
    );

    std::fs::remove_file(ledger_path).ok();
}

#[test]
fn audited_pdp_maps_untrusted_persisted_ledger_to_audit_chain_emission() {
    let ledger_path = unique_ledger_path("audit-emission-fail-closed");
    let signer = Ed25519SigningKey::from_seed_bytes("pdp-audit-test-key", [7_u8; 32]).unwrap();
    let trusted_keys = Ed25519VerificationKeySet::single(signer.verification_key()).unwrap();
    let logger = PdpDecisionAuditChainLogger::new(
        FileAuditLedger::new(ledger_path.clone()),
        signer,
        trusted_keys,
    )
    .unwrap();
    let pdp = AuditChainCedarPdp::load(
        &locked_seed_bundle("psv-000001", vec![]),
        Arc::new(SeededIdGenerator::default()),
        64,
        logger,
    )
    .unwrap();

    let mut untrusted_chain = AuditChain::multi_tenant_shards();
    audit_append(
        &mut untrusted_chain,
        audit_input("acme", "post-startup-unsigned"),
        None,
    )
    .unwrap();
    FileAuditLedger::new(ledger_path.clone())
        .append_chain(&untrusted_chain)
        .unwrap();

    let err = pdp
        .authorize(
            &request(
                "req-audit-chain-fail-closed",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap_err();

    match err {
        PdpError::AuditChainEmission { detail } => {
            assert!(
                detail.contains("trusted signature replay failed"),
                "audit-chain emission detail must name trusted replay failure, got {detail:?}"
            );
            assert!(
                detail.contains("MissingEd25519Signature"),
                "audit-chain emission detail must preserve signature cause, got {detail:?}"
            );
        }
        other => {
            panic!("untrusted persisted ledger must fail closed as audit emission, got {other:?}")
        }
    }

    std::fs::remove_file(ledger_path).ok();
}

#[test]
fn unknown_action_fails_closed() {
    let pdp = pdp(vec![]);
    let err = pdp
        .authorize(
            &request(
                "req-err-1",
                "acme",
                entity_ref("OyaPlatform::Principal", "alice"),
                "resource.purge",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap_err();
    assert!(matches!(err, PdpError::UnknownAction { .. }));
}

#[test]
fn obligations_ride_out_with_annotated_permits() {
    let mut bundle = locked_seed_bundle("psv-000001", vec![]);
    bundle.policies_src.push_str(
        "\n@id(\"workload-read-grant\")\n@obligation(\"emit-step-up-audit\")\npermit (\n  principal is OyaPlatform::WorkloadIdentity,\n  action == OyaPlatform::Action::\"ReadResource\",\n  resource\n)\nwhen { principal.tenant_id == resource.tenant_id };\n",
    );
    let pdp = CedarPdp::load(&bundle, Arc::new(SeededIdGenerator::default()), 64).unwrap();
    let outcome = pdp
        .authorize(
            &request(
                "req-obl-1",
                "acme",
                entity_ref("OyaPlatform::WorkloadIdentity", "payments"),
                "resource.read",
                entity_ref("OyaPlatform::TenantResource", "acme-doc-1"),
            ),
            &entity_slice(),
        )
        .unwrap();
    assert_eq!(outcome.response.decision, Decision::Allow);
    assert_eq!(
        outcome
            .response
            .obligations
            .iter()
            .map(|o| o.obligation_id.as_str())
            .collect::<Vec<_>>(),
        vec!["emit-step-up-audit"]
    );
}

// ------------------------------------------------ seed parity guard ----

// option_env!, not env!: CARGO_MANIFEST_DIR is undefined at buck2 compile
// time (hermetic sandbox), and the buck2 lane must still COMPILE this target
// (FRIC-019). The cargo lane enforces parity; buck2 skips with a notice.
fn manifest_dir() -> Option<&'static Path> {
    option_env!("CARGO_MANIFEST_DIR").map(Path::new)
}

fn repo_root() -> Option<PathBuf> {
    let mut dir = manifest_dir()?.to_path_buf();
    loop {
        if dir.join("Cargo.toml").is_file() && dir.join("docs/decisions").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[test]
fn crate_local_cedar_seeds_match_canonical() {
    const PAIRS: &[(&str, &str)] = &[
        (
            "cedar/platform.cedarschema",
            "libs/oya-shared-platform-contracts-kernel/cedar/platform.cedarschema",
        ),
        (
            "cedar/platform-policies.cedar",
            "libs/oya-shared-platform-contracts-kernel/cedar/platform-policies.cedar",
        ),
        (
            "cedar/platform-templates.cedar",
            "libs/oya-shared-platform-contracts-kernel/cedar/platform-templates.cedar",
        ),
    ];
    let (Some(crate_dir), Some(root)) = (manifest_dir(), repo_root()) else {
        eprintln!(
            "cedar_seed_parity: repo root marker not reachable (hermetic sandbox); \
             skipped {} pairs — cargo CI lane enforces parity",
            PAIRS.len()
        );
        return;
    };
    let mut mismatches = Vec::new();
    for (local, canonical) in PAIRS {
        let local_bytes = std::fs::read(crate_dir.join(local))
            .unwrap_or_else(|e| panic!("crate-local cedar seed missing: {local}: {e}"));
        let canonical_path = root.join(canonical);
        let canonical_bytes = std::fs::read(&canonical_path).unwrap_or_else(|e| {
            panic!(
                "canonical cedar seed missing: {}: {e}",
                canonical_path.display()
            )
        });
        if local_bytes != canonical_bytes {
            mismatches.push(format!("{local} != {canonical}"));
        }
    }
    assert!(
        mismatches.is_empty(),
        "crate-local cedar seed copies drifted from the canonical contract-lock \
         sources (canonical wins; sync the crate copy in the same change):\n  {}",
        mismatches.join("\n  ")
    );
}
