#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::sync::Arc;

use billing_accounting_api::{
    JournalLineRequest, JournalPostRequest, JurisdictionDto, PayrollPostingRequest, PeriodStateDto,
    VatDeadlineRequest,
};
use billing_accounting_http_adapter::{
    ACCOUNTING_HEALTH_PATH, ACCOUNTING_JOURNALS_PATH, ACCOUNTING_PAYROLL_POSTINGS_PATH,
    ACCOUNTING_VAT_WORKFLOW_PLANS_PATH, AccountingAuthzProvider,
    AccountingMutationAuthorizationError, AccountingMutationAuthorizer, AccountingMutationResource,
    ConfiguredBearerPrincipalVerifier, VerifiedPrincipal, accounting_runtime_routes,
    accounting_server_config, dispatch_accounting_request,
};
use oya_http_middleware_kernel::HttpRequest;
use oya_http_router_kernel::HttpMethod;

/// Break-glass bearer secret bound to the test principal/tenant (ten_acme).
const BEARER: &str = "accounting-test-secret";

/// PDP authorizer that allows any verified principal — isolates the AUTHN gate
/// in happy-path tests. The cross-tenant RED test relies on the handler's
/// body-tenant cross-check (true blast radius), not on this authorizer.
struct AllowAllAuthorizer;
impl AccountingMutationAuthorizer for AllowAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &AccountingMutationResource,
    ) -> Result<(), AccountingMutationAuthorizationError> {
        Ok(())
    }
}

/// PDP authorizer that denies every decision — proves PDP-deny -> 403.
struct DenyAllAuthorizer;
impl AccountingMutationAuthorizer for DenyAllAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &AccountingMutationResource,
    ) -> Result<(), AccountingMutationAuthorizationError> {
        Err(AccountingMutationAuthorizationError::Denied)
    }
}

/// PDP authorizer that PANICS — proves a PDP fault is mapped to 403 (deny),
/// never propagated as allow, never a 500.
struct PanicAuthorizer;
impl AccountingMutationAuthorizer for PanicAuthorizer {
    fn ensure_authorized(
        &self,
        _principal: &VerifiedPrincipal,
        _resource: &AccountingMutationResource,
    ) -> Result<(), AccountingMutationAuthorizationError> {
        panic!("simulated PDP outage");
    }
}

fn provider_with(authorizer: Arc<dyn AccountingMutationAuthorizer>) -> AccountingAuthzProvider {
    let verifier = ConfiguredBearerPrincipalVerifier::new(BEARER, "sp_accounting", "ten_acme")
        .expect("verifier construction");
    AccountingAuthzProvider::new(Arc::new(verifier), authorizer)
}

fn allow_provider() -> AccountingAuthzProvider {
    provider_with(Arc::new(AllowAllAuthorizer))
}

#[test]
fn accounting_runtime_dispatches_journal_payroll_and_vat() {
    let journal = dispatch_accounting_request(
        authed_json_request(
            HttpMethod::Post,
            ACCOUNTING_JOURNALS_PATH,
            &journal_request(),
        ),
        allow_provider(),
    );
    let journal_body: serde_json::Value =
        serde_json::from_slice(&journal.body).expect("journal json");
    assert_eq!(journal.status, 202);
    assert_eq!(journal_body["accepted"], true);
    assert_eq!(
        journal_body["auditTopic"],
        "audit.accounting.journal.posted"
    );
    assert_eq!(journal_body["service"], "accounting");

    let payroll = dispatch_accounting_request(
        authed_json_request(
            HttpMethod::Post,
            ACCOUNTING_PAYROLL_POSTINGS_PATH,
            &payroll_request(),
        ),
        allow_provider(),
    );
    let payroll_body: serde_json::Value =
        serde_json::from_slice(&payroll.body).expect("payroll json");
    assert_eq!(payroll.status, 202);
    assert_eq!(
        payroll_body["auditTopic"],
        "audit.accounting.payroll.posted"
    );
    // SECURITY (ADR-0592): tenant-scoped LOGICAL idempotency key surfaced in the
    // 202 response. The body fingerprint is a separate field, NOT embedded in the
    // key, so the surfaced key carries no `#<fingerprint>` suffix.
    let payroll_key = payroll_body["idempotencyKey"]
        .as_str()
        .expect("idempotency key string");
    assert_eq!(
        payroll_key, "idem-v2:ten_acme:payroll-posted:jrn_payroll_2026_01",
        "payroll idempotency key must be the tenant-scoped logical key, got: {payroll_key}"
    );

    let vat = dispatch_accounting_request(
        authed_json_request(
            HttpMethod::Post,
            ACCOUNTING_VAT_WORKFLOW_PLANS_PATH,
            &vat_request(),
        ),
        allow_provider(),
    );
    let vat_body: serde_json::Value = serde_json::from_slice(&vat.body).expect("vat json");
    assert_eq!(vat.status, 200);
    assert_eq!(vat_body["opened"], true);
    assert_eq!(vat_body["workflowRef"], "workflow/accounting/vat/kr");
    assert!(!vat_body["requiredSteps"].as_array().unwrap().is_empty());
}

#[test]
fn accounting_runtime_rejects_invalid_json_and_domain_errors() {
    // Authenticated request with a malformed body still reaches the handler and
    // gets a 400 (authn passes; body deserialization fails AFTER the authz gate).
    let invalid_json = HttpRequest {
        method: HttpMethod::Post,
        path: ACCOUNTING_JOURNALS_PATH.to_owned(),
        headers: BTreeMap::from([
            ("content-type".to_owned(), "application/json".to_owned()),
            ("authorization".to_owned(), format!("Bearer {BEARER}")),
        ]),
        body: b"{not-json".to_vec(),
        path_captures: BTreeMap::new(),
        matched_template: None,
    };
    let invalid_response = dispatch_accounting_request(invalid_json, allow_provider());
    let invalid_body: serde_json::Value =
        serde_json::from_slice(&invalid_response.body).expect("invalid json response");
    assert_eq!(invalid_response.status, 400);
    assert_eq!(invalid_body["error"]["code"], "VALIDATION_ERROR");

    let unbalanced = dispatch_accounting_request(
        authed_json_request(
            HttpMethod::Post,
            ACCOUNTING_JOURNALS_PATH,
            &JournalPostRequest {
                lines: vec![JournalLineRequest {
                    account_code: "EXP-WAGES".to_owned(),
                    debit_minor: 1_000_000,
                    credit_minor: 0,
                }],
                ..journal_request()
            },
        ),
        allow_provider(),
    );
    let unbalanced_body: serde_json::Value =
        serde_json::from_slice(&unbalanced.body).expect("domain error json");
    assert_eq!(unbalanced.status, 400);
    assert!(
        unbalanced_body["error"]["details"]
            .as_str()
            .unwrap()
            .contains("UnbalancedJournal")
    );
}

#[test]
fn accounting_runtime_manifest_and_health_preserve_honest_non_claims() {
    let routes = accounting_runtime_routes();
    assert_eq!(routes.len(), 4);
    assert!(
        routes
            .iter()
            .any(|route| route.path == ACCOUNTING_VAT_WORKFLOW_PLANS_PATH)
    );

    let config = accounting_server_config();
    assert_eq!(config.max_body_bytes, 64 * 1024);

    // Health is a non-mutation route: the authz middleware passes it through
    // WITHOUT a credential (no money mutation, no PDP decision).
    let health = dispatch_accounting_request(
        HttpRequest {
            method: HttpMethod::Get,
            path: ACCOUNTING_HEALTH_PATH.to_owned(),
            headers: BTreeMap::new(),
            body: Vec::new(),
            path_captures: BTreeMap::new(),
            matched_template: None,
        },
        allow_provider(),
    );
    let body: serde_json::Value = serde_json::from_slice(&health.body).expect("health json");
    assert_eq!(health.status, 200);
    assert_eq!(body["runtimeAdapter"], "router-ready");
    assert_eq!(body["deployedListener"], false);
    assert_eq!(body["storageAttached"], false);
    assert_eq!(body["workflowExecution"], false);
    assert_eq!(body["statutoryFilingRails"], false);
    assert_eq!(body["paymentExecution"], false);
    assert_eq!(body["payrollNetworkCall"], false);
}

fn mock_json_request<T: serde::Serialize>(
    method: HttpMethod,
    path: &str,
    payload: &T,
) -> HttpRequest {
    HttpRequest {
        method,
        path: path.to_owned(),
        headers: BTreeMap::from([("content-type".to_owned(), "application/json".to_owned())]),
        body: serde_json::to_vec(payload).expect("serialize request"),
        path_captures: BTreeMap::new(),
        matched_template: None,
    }
}

/// A request carrying a valid break-glass bearer (verifies to ten_acme).
fn authed_json_request<T: serde::Serialize>(
    method: HttpMethod,
    path: &str,
    payload: &T,
) -> HttpRequest {
    let mut request = mock_json_request(method, path, payload);
    request
        .headers
        .insert("authorization".to_owned(), format!("Bearer {BEARER}"));
    request
}

// ---------------------------------------------------------------------------
// AUTH-005 RED tests (ADR-0593): the empty MiddlewareChain::new() left every
// money-mutation route unauthenticated. These prove the fail-closed gate.
// ---------------------------------------------------------------------------

#[test]
fn unauthenticated_money_mutation_is_rejected_401() {
    // No Authorization header at all -> 401 BEFORE the body is deserialized.
    let request = mock_json_request(
        HttpMethod::Post,
        ACCOUNTING_JOURNALS_PATH,
        &journal_request(),
    );
    let response = dispatch_accounting_request(request, allow_provider());
    assert_eq!(response.status, 401);
    let body: serde_json::Value = serde_json::from_slice(&response.body).expect("401 json");
    assert_eq!(body["error"]["code"], "UNAUTHENTICATED");
}

#[test]
fn wrong_bearer_money_mutation_is_rejected_401() {
    let mut request = mock_json_request(
        HttpMethod::Post,
        ACCOUNTING_PAYROLL_POSTINGS_PATH,
        &payroll_request(),
    );
    request.headers.insert(
        "authorization".to_owned(),
        "Bearer not-the-secret".to_owned(),
    );
    let response = dispatch_accounting_request(request, allow_provider());
    assert_eq!(response.status, 401);
}

#[test]
fn cross_tenant_money_mutation_is_rejected_403() {
    // Verified principal is ten_acme (from the bearer), but the body claims a
    // DIFFERENT tenant — the cross-tenant body-substitution attack. Must be
    // denied 403 by the handler's body-tenant cross-check even though the PDP
    // authorizer would allow.
    let cross_tenant_body = JournalPostRequest {
        tenant_id: "ten_victim".to_owned(),
        ..journal_request()
    };
    let request = authed_json_request(
        HttpMethod::Post,
        ACCOUNTING_JOURNALS_PATH,
        &cross_tenant_body,
    );
    let response = dispatch_accounting_request(request, allow_provider());
    assert_eq!(response.status, 403);
}

#[test]
fn pdp_deny_money_mutation_is_rejected_403() {
    let request = authed_json_request(
        HttpMethod::Post,
        ACCOUNTING_JOURNALS_PATH,
        &journal_request(),
    );
    let response = dispatch_accounting_request(request, provider_with(Arc::new(DenyAllAuthorizer)));
    assert_eq!(response.status, 403);
}

#[test]
fn pdp_fault_money_mutation_denies_403_not_500() {
    // A panicking (faulting) PDP must DENY (403), never allow and never 500.
    let request = authed_json_request(
        HttpMethod::Post,
        ACCOUNTING_JOURNALS_PATH,
        &journal_request(),
    );
    let response = dispatch_accounting_request(request, provider_with(Arc::new(PanicAuthorizer)));
    assert_eq!(response.status, 403);
}

#[test]
fn authenticated_authorized_same_tenant_money_mutation_succeeds() {
    // Sanity GREEN: verified ten_acme + matching body tenant + PDP allow -> 202.
    let request = authed_json_request(
        HttpMethod::Post,
        ACCOUNTING_JOURNALS_PATH,
        &journal_request(),
    );
    let response = dispatch_accounting_request(request, allow_provider());
    assert_eq!(response.status, 202);
}

fn digest() -> String {
    format!("sha256:{}", "b".repeat(64))
}

fn lines() -> Vec<JournalLineRequest> {
    vec![
        JournalLineRequest {
            account_code: "EXP-WAGES".to_owned(),
            debit_minor: 1_000_000,
            credit_minor: 0,
        },
        JournalLineRequest {
            account_code: "LIAB-NETPAY".to_owned(),
            debit_minor: 0,
            credit_minor: 1_000_000,
        },
    ]
}

fn journal_request() -> JournalPostRequest {
    JournalPostRequest {
        journal_id: "jrn_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-01".to_owned(),
        period_state: PeriodStateDto::Open,
        source_documents: vec!["src/payroll/run/prun_kr_2026_01".to_owned()],
        approval_evidence_ref: "audit/accounting/journal/approval".to_owned(),
        lines: lines(),
    }
}

fn payroll_request() -> PayrollPostingRequest {
    PayrollPostingRequest {
        journal_id: "jrn_payroll_2026_01".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        period: "2026-01".to_owned(),
        source_payroll_digest: digest(),
        wage_ledger_refs: vec!["audit/payroll/wage-ledger/001".to_owned()],
        approval_evidence_ref: "audit/accounting/payroll/approval".to_owned(),
        reversal_path_ref: "audit/accounting/payroll/reversal".to_owned(),
        lines: lines(),
    }
}

fn vat_request() -> VatDeadlineRequest {
    VatDeadlineRequest {
        return_id: "vat_2026_q1".to_owned(),
        tenant_id: "ten_acme".to_owned(),
        legal_entity_id: "le_kr_001".to_owned(),
        jurisdiction: JurisdictionDto::Korea,
        period: "2026-01".to_owned(),
        deadline_epoch_seconds: 1_779_519_600,
        now_epoch_seconds: 1_779_519_601,
        workflow_ref: "workflow/accounting/vat/kr".to_owned(),
        hometax_export_hash: digest(),
        evidence_ref: "audit/accounting/vat/evidence".to_owned(),
    }
}
