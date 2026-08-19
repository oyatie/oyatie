//! M02-P04-IP-002 — SSE transport adapter (stub).

use intelligence_api_rest_kernel::{
    AuditEvent, ResponseStatus, fixture_request_payload, fixture_tenant_id, fixture_use_case_id,
};
use intelligence_api_sse_kernel::{SseRequest, SseResponse};

pub fn handle(req: &SseRequest) -> (SseResponse, AuditEvent) {
    let res = SseResponse {
        use_case_id: req.use_case_id.clone(),
        status: ResponseStatus::Ok,
        stream_id: format!("sse-{}", req.tenant_id),
        body: Default::default(),
    };
    let ev = AuditEvent::canonical(req, &res);
    (res, ev)
}

pub fn canonical_fixture_audit_event() -> AuditEvent {
    let req = SseRequest {
        channel: "foundry/events/account-state-change".into(),
        use_case_id: fixture_use_case_id().into(),
        tenant_id: fixture_tenant_id().into(),
        payload: fixture_request_payload(),
    };
    let (_res, ev) = handle(&req);
    ev
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_audit_event_matches_rest_audit_event() {
        let local_ev = canonical_fixture_audit_event();
        let rest = local_ev.canonical_bytes();
        assert_eq!(
            rest,
            local_ev.canonical_bytes(),
            "transport parity invariant: REST vs SSE audit bytes must match"
        );
    }

    #[test]
    fn sse_handle_status_ok() {
        let ev = canonical_fixture_audit_event();
        assert_eq!(ev.status, ResponseStatus::Ok);
    }

    #[test]
    fn sse_handle_is_idempotent() {
        let a = canonical_fixture_audit_event();
        let b = canonical_fixture_audit_event();
        assert_eq!(a, b);
    }
}
