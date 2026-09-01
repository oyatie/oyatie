//! Shared fixture for the facade's HTTP suites: temp-backed durable stores,
//! a booted process, and the three credentials the write tests distinguish
//! between — an in-tenant operator, a foreign-tenant operator, and a
//! recognized caller holding no role at all.

#![allow(dead_code)]

use std::path::PathBuf;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use foundry_ontology_app::{AppState, Config, OperatorCredential, compose, router};
use foundry_records_draft::RecordsLog;
use foundry_records_sqlite_draft::SqliteRecordsLog;
use http_body_util::BodyExt;
use tower::ServiceExt;

pub const TENANT: &str = "ten_acme";
const OPERATOR_TOKEN: &str = "operator-token-for-tests";
const FOREIGN_TOKEN: &str = "foreign-token-for-tests";
const ROLELESS_TOKEN: &str = "roleless-token-for-tests";

pub struct Fixture {
    action: PathBuf,
    denial: PathBuf,
}

impl Fixture {
    pub fn new(case: &str) -> Self {
        let stamp = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock is after the epoch")
                .as_nanos()
        );
        let temp = std::env::temp_dir();
        let fixture = Self {
            action: temp.join(format!("foundry-facade-{case}-action-{stamp}.sqlite")),
            denial: temp.join(format!("foundry-facade-{case}-denial-{stamp}.sqlite")),
        };
        let _ = std::fs::remove_file(&fixture.action);
        let _ = std::fs::remove_file(&fixture.denial);
        fixture
    }

    pub fn config(&self) -> Config {
        Config {
            listen_addr: "127.0.0.1:0".into(),
            action_log: self.action.clone(),
            denial_log: self.denial.clone(),
            tenants: vec![TENANT.into()],
            operators: vec![
                OperatorCredential {
                    token: OPERATOR_TOKEN.into(),
                    tenant_id: TENANT.into(),
                    principal_id: "prn_alice".into(),
                    roles: vec!["foundry-operator".into()],
                },
                OperatorCredential {
                    token: FOREIGN_TOKEN.into(),
                    tenant_id: "ten_other".into(),
                    principal_id: "prn_mallory".into(),
                    roles: vec!["foundry-operator".into()],
                },
                OperatorCredential {
                    token: ROLELESS_TOKEN.into(),
                    tenant_id: TENANT.into(),
                    principal_id: "prn_nobody".into(),
                    roles: Vec::new(),
                },
            ],
        }
    }

    pub fn state(&self) -> AppState {
        compose(&self.config()).expect("the fixture must boot")
    }

    pub fn operator_token(&self) -> &'static str {
        OPERATOR_TOKEN
    }

    pub fn foreign_token(&self) -> &'static str {
        FOREIGN_TOKEN
    }

    pub fn roleless_token(&self) -> &'static str {
        ROLELESS_TOKEN
    }

    /// The action log's head, read from the durable store itself rather
    /// than from anything the process reports about itself.
    pub fn log_head(&self) -> u64 {
        SqliteRecordsLog::open(&self.action)
            .expect("the action log opens")
            .head(TENANT)
            .expect("head is readable")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.action);
        let _ = std::fs::remove_file(&self.denial);
    }
}

/// POST a body to the write surface, optionally bearing a credential.
pub async fn post(fixture: &Fixture, token: Option<&str>, body: &str) -> (StatusCode, String) {
    let mut request = Request::builder()
        .method("POST")
        .uri("/v1/actions")
        .header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = router(fixture.state())
        .oneshot(
            request
                .body(Body::from(body.to_owned()))
                .expect("a well-formed request"),
        )
        .await
        .expect("the router answers");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("a readable body")
        .to_bytes();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

/// GET a path, optionally bearing a credential.
pub async fn get(fixture: &Fixture, token: Option<&str>, path: &str) -> (StatusCode, String) {
    let mut request = Request::builder().method("GET").uri(path);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = router(fixture.state())
        .oneshot(request.body(Body::empty()).expect("a well-formed request"))
        .await
        .expect("the router answers");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("a readable body")
        .to_bytes();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

/// Land one record through the REAL write path, so a read test reads what a
/// write actually produced rather than a hand-built projection.
pub async fn write_a_record(fixture: &Fixture, object_ref: &str, key: &str) {
    let body = format!(
        r#"{{"object_ref":"{object_ref}","action_type":"aty_record_write","idempotency_key":"{key}","occurred_at_epoch_seconds":1700000000,"properties":{{"name":"Ada"}}}}"#
    );
    let (status, reply) = post(fixture, Some(fixture.operator_token()), &body).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "the fixture write must land: {reply}"
    );
}
