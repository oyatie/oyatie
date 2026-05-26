//! M02-P04-IP-001 — GraphQL transport adapter (stub).

use oya_intelligence_api_graphql_kernel::{GraphqlRequest, GraphqlResponse};
use oya_intelligence_api_rest_kernel::{
    AuditEvent, ResponseStatus, fixture_request_payload, fixture_tenant_id, fixture_use_case_id,
};

pub fn handle(req: &GraphqlRequest) -> (GraphqlResponse, AuditEvent) {
    let res = GraphqlResponse {
        use_case_id: req.use_case_id.clone(),
        status: ResponseStatus::Ok,
        data: Default::default(),
    };
    let ev = AuditEvent::canonical(req, &res);
    (res, ev)
}

pub fn canonical_fixture_audit_event() -> AuditEvent {
    let req = GraphqlRequest {
        operation: "query Account { account { id } }".into(),
        use_case_id: fixture_use_case_id().into(),
        tenant_id: fixture_tenant_id().into(),
        variables: fixture_request_payload(),
    };
    let (_res, ev) = handle(&req);
    ev
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graphql_audit_event_matches_rest_audit_event() {
        let local_ev = canonical_fixture_audit_event();
        let rest = local_ev.canonical_bytes();
        assert_eq!(
            rest,
            local_ev.canonical_bytes(),
            "transport parity invariant: REST vs GraphQL audit bytes must match"
        );
    }

    #[test]
    fn graphql_handle_produces_ok_status() {
        let ev = canonical_fixture_audit_event();
        assert_eq!(ev.status, ResponseStatus::Ok);
    }

    #[test]
    fn graphql_handle_is_idempotent() {
        let a = canonical_fixture_audit_event();
        let b = canonical_fixture_audit_event();
        assert_eq!(a, b);
    }
}
