// ADR-0083 Tier 3: integration tests legitimately use `.unwrap()` / `.expect()` / `panic!()`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(dead_code)]
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use policy_pdp_kernel::*;
use shared_platform_contracts_kernel::pdp::*;

mod doubles;

pub use doubles::*;

pub fn entity_ref(entity_type: &str, entity_id: &str) -> EntityRef {
    EntityRef {
        entity_type: entity_type.to_owned(),
        entity_id: entity_id.to_owned(),
    }
}

pub fn request() -> AuthorizationRequest {
    AuthorizationRequest {
        request_id: "req-1".to_owned(),
        tenant_id: "acme".to_owned(),
        principal: entity_ref("OyaPlatform::Principal", "alice"),
        action: "resource.read".to_owned(),
        resource: entity_ref("OyaPlatform::TenantResource", "doc-1"),
        context: BTreeMap::new(),
        min_policy_version: None,
    }
}

pub fn slice() -> EntitySlice {
    EntitySlice {
        entities: vec![
            EntityRecord {
                uid: entity_ref("OyaPlatform::Principal", "alice"),
                attributes: BTreeMap::from([("tenant_id".to_owned(), serde_json::json!("acme"))]),
                parents: vec![entity_ref("OyaPlatform::Group", "tenant-admins")],
            },
            EntityRecord {
                uid: entity_ref("OyaPlatform::Group", "tenant-admins"),
                attributes: BTreeMap::new(),
                parents: vec![],
            },
        ],
    }
}

pub fn allow_outcome(request: &AuthorizationRequest, version: PolicyVersion) -> PdpOutcome {
    let response = AuthorizationResponse {
        decision_id: "dec-runtime-allow".to_owned(),
        request_id: request.request_id.clone(),
        decision: Decision::Allow,
        policy_version: version.clone(),
        determining_policy_ids: vec!["permit-admin".to_owned()],
        obligations: vec![],
    };
    let audit = DecisionAuditRecord {
        decision_id: response.decision_id.clone(),
        request_id: response.request_id.clone(),
        tenant_id: request.tenant_id.clone(),
        principal: request.principal.clone(),
        action: request.action.clone(),
        resource: request.resource.clone(),
        decision: response.decision,
        policy_version: version,
        determining_policy_ids: response.determining_policy_ids.clone(),
        cache_hit: false,
    };
    PdpOutcome {
        response,
        audit,
        cache_hit: false,
    }
}

pub fn seed_bundle_json_without_overlays() -> String {
    // A pre-G004 flat bundle document: no `tenant_policies` field at all.
    serde_json::json!({
        "version": "psv-000001",
        "schema_src": "schema",
        "policies_src": "policies",
        "templates": [],
        "template_links": [],
        "action_map": {},
    })
    .to_string()
}

pub fn decision_authz_request<'a>(
    caller_tenant: &'a str,
    target_tenant: &'a str,
) -> DecisionAuthzRequest<'a> {
    DecisionAuthzRequest {
        caller_tenant,
        caller_id: "control-plane",
        target_tenant,
        target_subject_id: "wl-secrets-sync",
        action: "tenant-rbac.policy.admission",
        resource_type: "OyaPlatform::TenantResource",
        resource_id: "tenant-rbac/policy-admissions/pa-1",
    }
}
