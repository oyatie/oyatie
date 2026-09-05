mod bundle;
mod cache;
mod decision_authz;
mod entity;
mod error;
mod fingerprint;
mod runtime;
mod runtime_fixtures;

use std::collections::BTreeMap;

use shared_platform_contracts_kernel::pdp::{
    AuthorizationRequest, AuthorizationResponse, Decision, EntityRef, PolicyVersion,
};

use crate::{DecisionAuditRecord, EntityRecord, EntitySlice, PdpOutcome};

fn entity_ref(entity_type: &str, entity_id: &str) -> EntityRef {
    EntityRef {
        entity_type: entity_type.to_owned(),
        entity_id: entity_id.to_owned(),
    }
}

fn request() -> AuthorizationRequest {
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

fn slice() -> EntitySlice {
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

fn allow_outcome(request: &AuthorizationRequest, version: PolicyVersion) -> PdpOutcome {
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
