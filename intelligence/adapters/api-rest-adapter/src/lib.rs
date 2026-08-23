//! M02-P04-IP-001 — REST transport adapter (stub).
//!
//! Maps incoming REST envelopes to the canonical use-case ports. Real wire
//! framing lives in `http-runtime-hyper-adapter`; this crate only owns
//! the use-case projection.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use intelligence_api_rest_kernel::{
    AuditEvent, ResponseStatus, RestRequest, RestResponse, fixture_request_payload,
    fixture_tenant_id, fixture_use_case_id,
};

/// Stub handler: projects an inbound REST request to the canonical audit
/// event after running the (currently no-op) use-case.
pub fn handle(req: &RestRequest) -> (RestResponse, AuditEvent) {
    let res = RestResponse {
        status_code: 200,
        use_case_id: req.use_case_id.clone(),
        status: ResponseStatus::Ok,
        body: Default::default(),
    };
    let ev = AuditEvent::canonical(req, &res);
    (res, ev)
}

/// Canonical fixture audit event — produced by REST transport for parity
/// comparison against the other 3 transports.
pub fn canonical_fixture_audit_event() -> AuditEvent {
    let req = RestRequest {
        method: "GET".into(),
        path: "/v1/accounts/acct-001".into(),
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
    fn rest_handle_produces_canonical_audit_event() {
        let ev = canonical_fixture_audit_event();
        assert_eq!(ev.use_case_id, "foundry.account.view");
        assert_eq!(ev.tenant_id, "tenant-alpha");
        assert_eq!(ev.status, ResponseStatus::Ok);
    }

    #[test]
    fn rest_canonical_bytes_match_known_shape() {
        let ev = canonical_fixture_audit_event();
        let bytes = ev.canonical_bytes();
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.starts_with("use_case_id=foundry.account.view"));
        assert!(s.contains(";tenant_id=tenant-alpha"));
        assert!(s.contains(";status=ok"));
    }

    #[test]
    fn rest_handle_is_idempotent() {
        let a = canonical_fixture_audit_event();
        let b = canonical_fixture_audit_event();
        assert_eq!(a, b);
        assert_eq!(a.canonical_bytes(), b.canonical_bytes());
    }
}
