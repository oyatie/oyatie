use std::{collections::BTreeMap, sync::Mutex};

use foundry_submission_draft::{
    ActionSubmitter, Submission, SubmitError, SubmitRequest, SubmitResponse, conformance,
};

/// Deliberately local test adapter: credentials and permissions are configured
/// by the fixture, never supplied through the submission request.
#[derive(Default)]
struct Reference {
    entries: Mutex<BTreeMap<(String, String), (SubmitRequest, u64)>>,
}

fn tenant_for(credential: &str) -> Result<&'static str, SubmitError> {
    match credential {
        "operator" | "roleless" => Ok("ten_acme"),
        "foreign" => Ok("ten_other"),
        _ => Err(SubmitError::Credential),
    }
}

impl ActionSubmitter for Reference {
    fn submit_in_tenant<'a>(
        &'a self,
        credential: &'a str,
        expected_tenant: &'a str,
        request: SubmitRequest,
    ) -> Submission<'a> {
        Box::pin(async move {
            let tenant = tenant_for(credential)?;
            if expected_tenant != tenant {
                return Err(SubmitError::TenantMismatch);
            }
            self.submit(credential, request).await
        })
    }

    fn submit<'a>(&'a self, credential: &'a str, request: SubmitRequest) -> Submission<'a> {
        Box::pin(async move {
            let tenant = tenant_for(credential)?;
            if credential == "roleless" {
                return Err(SubmitError::Authorization);
            }
            let mut entries = self.entries.lock().map_err(|_| SubmitError::Unavailable)?;
            let key = (tenant.into(), request.idempotency_key.clone());
            let (ordinal, deduplicated) = match entries.get(&key) {
                Some((original, ordinal)) if original == &request => (*ordinal, true),
                Some(_) => return Err(SubmitError::Conflict),
                None => {
                    let ordinal =
                        entries.keys().filter(|(owner, _)| owner == tenant).count() as u64 + 1;
                    entries.insert(key, (request, ordinal));
                    (ordinal, false)
                }
            };
            Ok(SubmitResponse {
                outcome: "applied",
                ordinal,
                deduplicated,
                poison_reason: None,
            })
        })
    }
}

#[tokio::test]
async fn reference_qualifies_the_submission_port() {
    conformance::check_submission(
        &Reference::default(),
        "operator",
        "roleless",
        "unknown",
        request(),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn reference_qualifies_the_tenant_binding() {
    conformance::check_tenant_binding(
        &Reference::default(),
        "operator",
        "foreign",
        "ten_acme",
        request(),
    )
    .await
    .unwrap();
}

#[test]
fn submission_never_accepts_identity_or_policy_authority_fields() {
    for field in ["tenant_id", "principal_id", "decision", "permit"] {
        let mut value = serde_json::to_value(request()).unwrap();
        value[field] = serde_json::json!("forged");
        assert!(serde_json::from_value::<SubmitRequest>(value).is_err());
    }
}

fn request() -> SubmitRequest {
    SubmitRequest {
        object_ref: "ent_alpha".into(),
        action_type: "aty_record_write".into(),
        idempotency_key: "idem_1".into(),
        occurred_at_epoch_seconds: 100,
        properties: BTreeMap::from([("name".into(), "Ada".into())]),
    }
}
