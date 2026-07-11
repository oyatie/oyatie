//! M02-P04-IP-001 — REST transport kernel.
//!
//! Defines the canonical `UseCaseRequest`/`UseCaseResponse` traits + the
//! canonical `AuditEvent` shape that every transport (REST/SSE/WebSocket)
//! MUST project byte-identically (modulo transport metadata).
//!
//! Boundary: no I/O, std-only.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

// ── Canonical use-case ports (shared across transports) ────────────────────

/// Canonical request envelope projected by every transport.
pub trait UseCaseRequest {
    fn use_case_id(&self) -> &str;
    fn tenant_id(&self) -> &str;
    fn payload(&self) -> &BTreeMap<String, String>;
}

/// Canonical response envelope projected by every transport.
pub trait UseCaseResponse {
    fn use_case_id(&self) -> &str;
    fn status(&self) -> ResponseStatus;
    fn body(&self) -> &BTreeMap<String, String>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseStatus {
    Ok,
    Denied,
    NotFound,
    Error,
}

/// Canonical audit event — byte-identical across transports.
///
/// `transport` is the ONLY field that differs per transport. The audit-
/// parity invariant (M02-P04 acceptance) asserts equality of all OTHER
/// fields after constructing the event from each transport adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEvent {
    pub use_case_id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub status: ResponseStatus,                   // data_class: INTERNAL_ONLY
    pub canonical_payload: Vec<(String, String)>, // data_class: INTERNAL_ONLY
}

impl AuditEvent {
    /// Construct the canonical audit event from any `UseCaseRequest` +
    /// `UseCaseResponse` pair. The canonical_payload is sorted (BTree
    /// ordering) to guarantee byte-identical encoding across transports.
    pub fn canonical<Req: UseCaseRequest, Res: UseCaseResponse>(req: &Req, res: &Res) -> Self {
        let mut payload: Vec<(String, String)> = req
            .payload()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        payload.sort();
        Self {
            use_case_id: req.use_case_id().to_string(),
            tenant_id: req.tenant_id().to_string(),
            status: res.status(),
            canonical_payload: payload,
        }
    }

    /// Canonical byte encoding — stable across transports.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut s = String::new();
        s.push_str("use_case_id=");
        s.push_str(&self.use_case_id);
        s.push_str(";tenant_id=");
        s.push_str(&self.tenant_id);
        s.push_str(";status=");
        s.push_str(match self.status {
            ResponseStatus::Ok => "ok",
            ResponseStatus::Denied => "denied",
            ResponseStatus::NotFound => "not_found",
            ResponseStatus::Error => "error",
        });
        s.push_str(";payload=");
        for (k, v) in &self.canonical_payload {
            s.push_str(k);
            s.push('=');
            s.push_str(v);
            s.push(',');
        }
        s.into_bytes()
    }
}

// ── REST-specific request/response shape ────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestRequest {
    pub method: String,                    // data_class: INTERNAL_ONLY
    pub path: String,                      // data_class: INTERNAL_ONLY
    pub use_case_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub payload: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

impl UseCaseRequest for RestRequest {
    fn use_case_id(&self) -> &str {
        &self.use_case_id
    }
    fn tenant_id(&self) -> &str {
        &self.tenant_id
    }
    fn payload(&self) -> &BTreeMap<String, String> {
        &self.payload
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestResponse {
    pub status_code: u16,               // data_class: INTERNAL_ONLY
    pub use_case_id: String,            // data_class: INTERNAL_ONLY
    pub status: ResponseStatus,         // data_class: INTERNAL_ONLY
    pub body: BTreeMap<String, String>, // data_class: INTERNAL_ONLY
}

impl UseCaseResponse for RestResponse {
    fn use_case_id(&self) -> &str {
        &self.use_case_id
    }
    fn status(&self) -> ResponseStatus {
        self.status
    }
    fn body(&self) -> &BTreeMap<String, String> {
        &self.body
    }
}

/// Canonical fixture builder — every transport adapter constructs a request
/// using this fixture so the audit-parity test is deterministic.
pub fn fixture_request_payload() -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    m.insert("account_id".into(), "acct-001".into());
    m.insert("session_id".into(), "sess-042".into());
    m
}

pub fn fixture_use_case_id() -> &'static str {
    "foundry.account.view"
}

pub fn fixture_tenant_id() -> &'static str {
    "tenant-alpha"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rest_request_implements_use_case_request() {
        let req = RestRequest {
            method: "GET".into(),
            path: "/v1/accounts/acct-001".into(),
            use_case_id: fixture_use_case_id().into(),
            tenant_id: fixture_tenant_id().into(),
            payload: fixture_request_payload(),
        };
        assert_eq!(req.use_case_id(), "foundry.account.view");
        assert_eq!(req.tenant_id(), "tenant-alpha");
        assert_eq!(req.payload().len(), 2);
    }

    #[test]
    fn audit_event_is_deterministic() {
        let req = RestRequest {
            method: "GET".into(),
            path: "/v1/accounts/acct-001".into(),
            use_case_id: fixture_use_case_id().into(),
            tenant_id: fixture_tenant_id().into(),
            payload: fixture_request_payload(),
        };
        let res = RestResponse {
            status_code: 200,
            use_case_id: fixture_use_case_id().into(),
            status: ResponseStatus::Ok,
            body: BTreeMap::new(),
        };
        let ev1 = AuditEvent::canonical(&req, &res);
        let ev2 = AuditEvent::canonical(&req, &res);
        assert_eq!(ev1, ev2);
        assert_eq!(ev1.canonical_bytes(), ev2.canonical_bytes());
    }

    #[test]
    fn canonical_bytes_sort_payload() {
        let req = RestRequest {
            method: "GET".into(),
            path: "/v1/accounts/acct-001".into(),
            use_case_id: fixture_use_case_id().into(),
            tenant_id: fixture_tenant_id().into(),
            payload: fixture_request_payload(),
        };
        let res = RestResponse {
            status_code: 200,
            use_case_id: fixture_use_case_id().into(),
            status: ResponseStatus::Ok,
            body: BTreeMap::new(),
        };
        let ev = AuditEvent::canonical(&req, &res);
        let bytes = ev.canonical_bytes();
        let s = String::from_utf8(bytes).unwrap();
        // account_id sorts before session_id
        let i = s.find("account_id").unwrap();
        let j = s.find("session_id").unwrap();
        assert!(i < j);
    }
}
