use std::time::Instant;

use data_ontology_kernel::{ActionInvocationRequest, ActionTypeId};
use foundry_edits::{EditSet, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue};
use foundry_records_draft::RecordsLogError;
use foundry_spine::{ActionSubmission, ApplyOutcome, WriteError, submit_through};
use foundry_submission_draft::{
    ActionSubmitter, Submission, SubmitError, SubmitRequest, SubmitResponse,
};

use crate::{AppState, Surface};

impl ActionSubmitter for AppState {
    fn submit_in_tenant<'a>(
        &'a self,
        credential: &'a str,
        expected_tenant: &'a str,
        request: SubmitRequest,
    ) -> Submission<'a> {
        self.submit_measured(credential, Some(expected_tenant), request, Instant::now())
    }

    fn submit<'a>(&'a self, credential: &'a str, request: SubmitRequest) -> Submission<'a> {
        self.submit_measured(credential, None, request, Instant::now())
    }
}

impl AppState {
    fn submit_measured<'a>(
        &'a self,
        credential: &'a str,
        expected_tenant: Option<&'a str>,
        request: SubmitRequest,
        started: Instant,
    ) -> Submission<'a> {
        Box::pin(async move {
            let result = submit(self, credential, expected_tenant, request).await;
            if result.is_ok() {
                self.metrics.submit_served();
                self.metrics.invocation_answered(started.elapsed());
            } else {
                self.metrics.submit_refused();
            }
            result
        })
    }
}

/// Preserve the HTTP handler's observation window while using the same port
/// and authorization path as typed callers. Timing conveys no authority.
pub(super) struct TimedSubmission<'a> {
    pub state: &'a AppState,
    pub started: Instant,
}

impl ActionSubmitter for TimedSubmission<'_> {
    fn submit<'a>(&'a self, credential: &'a str, request: SubmitRequest) -> Submission<'a> {
        self.state
            .submit_measured(credential, None, request, self.started)
    }

    fn submit_in_tenant<'a>(
        &'a self,
        credential: &'a str,
        expected_tenant: &'a str,
        request: SubmitRequest,
    ) -> Submission<'a> {
        self.state
            .submit_measured(credential, Some(expected_tenant), request, self.started)
    }
}

async fn submit(
    state: &AppState,
    credential: &str,
    expected_tenant: Option<&str>,
    request: SubmitRequest,
) -> Result<SubmitResponse, SubmitError> {
    let caller = state
        .verifier
        .verify(Some(credential))
        .ok_or(SubmitError::Credential)?;
    if expected_tenant.is_some_and(|tenant| tenant != caller.tenant_id) {
        return Err(SubmitError::TenantMismatch);
    }
    crate::submission_limits::check(&request)?;
    let action_id =
        ActionTypeId::new(request.action_type.clone()).map_err(|_| SubmitError::Surface {
            cause: "the action type is not an action id",
        })?;
    // Only the verified credential selects a tenant. A known credential for
    // an unserved tenant is refused before either policy or persistence.
    let (served_tenant, tenant) = state
        .tenants
        .get_key_value(&caller.tenant_id)
        .ok_or(SubmitError::UnservedTenant)?;
    let decision = state
        .pep
        .decide(&caller, Surface::Invoke, &request.object_ref, served_tenant)
        .map_err(|_| SubmitError::Authorization)?;
    let mut tenant = tenant.lock().await;
    let edits = edits_for(&request).map_err(|_| SubmitError::Surface {
        cause: "the submission carries no representable edit",
    })?;
    let submission = ActionSubmission {
        request: ActionInvocationRequest {
            tenant_id: caller.tenant_id,
            principal_id: caller.principal_id,
            action_id,
            entity_id: request.object_ref,
            idempotency_key: request.idempotency_key,
            requested_at_epoch_seconds: request.occurred_at_epoch_seconds,
        },
        decision,
        parameters: Vec::new(),
        edits,
    };
    let (log, denial_log, projection, store) = tenant.write_handles();
    let outcome = submit_through(submission, log, denial_log, projection, store).map_err(
        |error| match error {
            WriteError::Refused(refused) => {
                state.metrics.denial_issued(refused.recorded_on_trail);
                SubmitError::Refused {
                    gate: refused.gate.label().into(),
                    cause: refused.cause,
                }
            }
            WriteError::Log(RecordsLogError::IdempotencyConflict { .. }) => SubmitError::Conflict,
            WriteError::Log(RecordsLogError::Storage { .. }) => SubmitError::Unavailable,
        },
    )?;
    // Acceptance follows append + fold. Mirror refusal remains observable
    // lag; retrying an accepted action must not fabricate a second write.
    if let Err(error) = outcome.mirror {
        tracing::warn!(?error, "durable projection mirror refused");
    }
    Ok(match outcome.outcome {
        ApplyOutcome::Applied { receipt } => SubmitResponse {
            outcome: "applied",
            ordinal: receipt.ordinal,
            deduplicated: receipt.deduplicated,
            poison_reason: None,
        },
        ApplyOutcome::Poisoned { receipt, reason } => SubmitResponse {
            outcome: "poisoned",
            ordinal: receipt.ordinal,
            deduplicated: receipt.deduplicated,
            poison_reason: Some(format!("{reason:?}")),
        },
    })
}

/// The edit set is a PURE FUNCTION OF THE REQUEST, and that is the retry
/// contract, not a simplification. An earlier version chose the edit kind
/// by asking whether the projection already held the object — which made
/// the same request produce different bytes before and after it landed, so
/// a retry arrived as divergent content under a spent key and conflicted
/// instead of deduplicating. Nothing about how the payload is built may
/// depend on state the request cannot see.
fn edits_for(request: &SubmitRequest) -> Result<EditSet, ()> {
    let properties: Vec<WireProperty> = request
        .properties
        .iter()
        .filter_map(|(name, value)| {
            WireProperty::new(
                name,
                WireTier::Scalar,
                WireDataClass::InternalOnly,
                WireValue::String(value.clone()),
            )
            .ok()
        })
        .collect();
    if properties.len() != request.properties.len() {
        return Err(());
    }
    let edit = OntologyEdit::create_object(SEEDED_ENTITY_TYPE, properties).map_err(|_| ())?;
    EditSet::new(vec![edit]).map_err(|_| ())
}

const SEEDED_ENTITY_TYPE: &str = "ety_record";
