#[path = "facade_support/mod.rs"]
mod support;

use std::{collections::BTreeMap, sync::Arc};

use foundry_ontology_app::{PolicyEnforcementPoint, compose};
use foundry_submission_draft::{ActionSubmitter, SubmitError, SubmitRequest, conformance};
use policy_pdp_kernel::{EntitySlice, PdpError, PdpOutcome, PolicyDecisionPoint};
use shared_platform_contracts_kernel::pdp::{AuthorizationRequest, PolicyVersion};
use support::{Fixture, TENANT};

fn request() -> SubmitRequest {
    SubmitRequest {
        object_ref: "ent_alpha".into(),
        action_type: "aty_record_write".into(),
        idempotency_key: "idem_1".into(),
        occurred_at_epoch_seconds: 1_700_000_000,
        properties: BTreeMap::from([("name".into(), "Ada".into())]),
    }
}

#[tokio::test]
async fn runtime_qualifies_the_same_submission_contract_as_the_reference() {
    let fixture = Fixture::new("port-contract");
    conformance::check_submission(
        &fixture.state(),
        fixture.operator_token(),
        fixture.roleless_token(),
        "unknown",
        request(),
    )
    .await
    .unwrap();
    assert_eq!(fixture.log_head(), 1);
}

#[tokio::test]
async fn credential_selects_the_tenant_and_retry_identity_is_tenant_scoped() {
    let fixture = Fixture::new("port-tenant");
    let state = fixture.state();
    assert_eq!(
        state.submit(fixture.foreign_token(), request()).await,
        Err(SubmitError::UnservedTenant)
    );
    assert_eq!(fixture.log_head(), 0);
    let mut config = fixture.config();
    config.tenants.push("ten_other".into());
    let state = compose(&config).unwrap();
    for token in [fixture.foreign_token(), fixture.operator_token()] {
        let receipt = state.submit(token, request()).await.unwrap();
        assert_eq!(receipt.ordinal, 1);
        assert!(!receipt.deduplicated);
    }
    for tenant in [TENANT, "ten_other"] {
        let state = state.tenants[tenant].lock().await;
        let entries = state.action_log.replay(tenant, 1).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].envelope.tenant_id, tenant);
    }
}

#[tokio::test]
async fn a_misconfigured_destination_credential_cannot_copy_source_data_across_tenants() {
    let fixture = Fixture::new("port-source-tenant");
    let mut config = fixture.config();
    config.tenants.push("ten_other".into());
    let state = compose(&config).unwrap();
    conformance::check_tenant_binding(
        &state,
        fixture.operator_token(),
        fixture.foreign_token(),
        TENANT,
        request(),
    )
    .await
    .unwrap();
    assert_eq!(fixture.log_head(), 1);
    let foreign = state.tenants["ten_other"].lock().await;
    assert_eq!(foreign.action_log.head("ten_other").unwrap(), 0);
    assert_eq!(foreign.denial_log.head("ten_other").unwrap(), 0);
}

struct Outage;
impl PolicyDecisionPoint for Outage {
    fn authorize(&self, _: &AuthorizationRequest, _: &EntitySlice) -> Result<PdpOutcome, PdpError> {
        Err(PdpError::Evaluation {
            detail: "injected outage".into(),
        })
    }
    fn loaded_policy_version(&self) -> PolicyVersion {
        PolicyVersion::new("psv-outage").unwrap()
    }
}

#[tokio::test]
async fn a_spent_key_does_not_bypass_current_policy_or_authorizer_outage() {
    let fixture = Fixture::new("port-outage");
    let mut state = fixture.state();
    state
        .submit(fixture.operator_token(), request())
        .await
        .unwrap();
    state.pep = PolicyEnforcementPoint::with_pdp(Arc::new(Outage));
    assert_eq!(
        state.submit(fixture.operator_token(), request()).await,
        Err(SubmitError::Authorization)
    );
    assert_eq!(fixture.log_head(), 1);
}

#[tokio::test]
async fn schema_admission_refusal_still_reaches_the_denial_trail() {
    let fixture = Fixture::new("port-schema");
    let mut request = request();
    request
        .properties
        .insert("undeclared".into(), "value".into());
    assert!(matches!(
        fixture
            .state()
            .submit(fixture.operator_token(), request)
            .await,
        Err(SubmitError::Refused { .. })
    ));
    assert_eq!(fixture.log_head(), 0);
    assert_eq!(fixture.denial_head(), 1);
}
