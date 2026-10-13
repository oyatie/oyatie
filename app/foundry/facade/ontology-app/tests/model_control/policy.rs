use crate::model_support::*;
use foundry_ontology_app::authz::{ModelOperation, ModelPepError};
use foundry_ontology_app::{Caller, PolicyEnforcementPoint, Surface};

#[test]
fn strict_model_policy_has_separate_read_author_publish_grants() {
    let pep = PolicyEnforcementPoint::load("psv-model-fixture").unwrap();
    for (roles, expected) in [
        (vec![], [false, false, false]),
        (vec!["foundry-operator"], [false, false, false]),
        (vec!["foundry-model-reader"], [true, false, false]),
        (vec!["foundry-model-author"], [true, true, false]),
        (vec!["foundry-model-publisher"], [true, false, true]),
        (
            vec!["foundry-model-author", "foundry-model-publisher"],
            [true, true, true],
        ),
    ] {
        let caller = Caller {
            tenant_id: TENANT.into(),
            principal_id: "usr_alice".into(),
            roles: roles.iter().map(|role| (*role).into()).collect(),
        };
        for (operation, allowed) in [
            ModelOperation::Read,
            ModelOperation::Author,
            ModelOperation::Publish,
        ]
        .into_iter()
        .zip(expected)
        {
            let result = pep.decide_model(&caller, operation, TENANT, now() + 3600);
            assert_eq!(result.is_ok(), allowed, "{roles:?} {operation:?}");
            if let Ok(decision) = result {
                assert_eq!(decision.operation, operation);
                assert_eq!(decision.tenant_id, TENANT);
                assert_eq!(decision.principal_id, "usr_alice");
                assert!(!decision.decision_id.is_empty());
                assert_eq!(decision.policy_version.as_str(), "psv-model-fixture");
            }
            assert_eq!(
                pep.decide_model(&caller, operation, "ten_foreign", now() + 3600),
                Err(ModelPepError::Denied)
            );
        }
        assert_eq!(
            pep.decide(&caller, Surface::Use, "ety_record", TENANT)
                .is_ok(),
            roles == ["foundry-operator"]
        );
        assert_eq!(
            pep.decide(&caller, Surface::Invoke, "ent_record", TENANT)
                .is_ok(),
            roles == ["foundry-operator"]
        );
    }
}

#[tokio::test]
async fn model_only_membership_reads_model_while_object_and_status_remain_denied() {
    let fixture = crate::facade_support::Fixture::new("model-only-entry");
    let state = instrument(fixture.state(), &["foundry-model-reader"], Some(origin()));
    let bytes = body(
        send(&state.router, request("GET", WORKSPACE, Body::empty())).await,
        StatusCode::OK,
    )
    .await;
    let model = ModelWorkspace::decode(&bytes).unwrap();
    assert_eq!(model.document, baseline().document);
    assert_eq!(model.generation, number(1));
    assert_eq!(
        model.capabilities,
        ModelCapabilities {
            author: false,
            publish: false
        }
    );
    for path in [
        "/v1/tenants/ten_acme/status",
        "/v1/tenants/ten_acme/types",
        "/v1/tenants/ten_acme/objects",
    ] {
        body(
            send(&state.router, request("GET", path, Body::empty())).await,
            StatusCode::FORBIDDEN,
        )
        .await;
    }
    refusal(
        send(
            &state.router,
            request(
                "POST",
                DRAFTS,
                CreateModelDraft {
                    expected_generation: number(1),
                    document: changed(),
                }
                .encode()
                .unwrap(),
            ),
        )
        .await,
        StatusCode::FORBIDDEN,
        ModelRefusalCode::Forbidden,
    )
    .await;
}
