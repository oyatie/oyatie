use std::{collections::BTreeMap, sync::Arc};

use foundry_submission_draft::{ActionSubmitter, SubmitError, SubmitRequest};
use mail_api::{Event, Events};
use mail_kernel::{Error, valid_identifier};
use sha2::{Digest, Sha256};

/// Publish account revision references through Foundry's authorized Record
/// action. This creates generic records, not work/contact ontology objects.
/// Each revision has its own object because the registered action creates
/// records. A reference never grants authority to read its source account.
///
/// `credentials` resolves an in-memory credential for the source tenant; it
/// must not perform blocking I/O. The destination independently verifies that
/// credential and constrains its tenant before writing. SQLite outbox reads
/// and acknowledgements run on blocking workers, outside the async executor.
/// Source, subscription, observation time, and payload remain stable on retry.
pub async fn publish_authorized(
    events: Arc<dyn Events>,
    destination: &dyn ActionSubmitter,
    credentials: &(dyn Fn(&str) -> Result<String, Error> + Send + Sync),
    source: &str,
    consumer: &str,
    limit: usize,
) -> Result<usize, Error> {
    if !valid_identifier(source) || !valid_identifier(consumer) {
        return Err(Error::Invalid);
    }
    let limit = limit.min(1000);
    let pending = {
        let events = Arc::clone(&events);
        let consumer = consumer.to_owned();
        tokio::task::spawn_blocking(move || events.pending(&consumer, limit))
            .await
            .map_err(|_| Error::Unavailable)??
    };
    if pending.len() > limit {
        return Err(Error::Unavailable);
    }
    let mut published = 0;
    for event in pending {
        let request = request(source, &event)?;
        let credential = credentials(&event.tenant)?;
        let result = destination
            .submit_in_tenant(&credential, &event.tenant, request)
            .await
            .map_err(submission_error)?;
        if result.outcome != "applied" || result.ordinal == 0 || result.poison_reason.is_some() {
            // Poison is durable at the destination but has not applied the
            // source reference. Never silently advance past it.
            return Err(Error::Unavailable);
        }
        let events = Arc::clone(&events);
        let consumer = consumer.to_owned();
        tokio::task::spawn_blocking(move || events.acknowledge(&consumer, event.sequence))
            .await
            .map_err(|_| Error::Unavailable)??;
        published += 1;
    }
    Ok(published)
}

fn request(source: &str, event: &Event) -> Result<SubmitRequest, Error> {
    if event.sequence == 0 {
        return Err(Error::Unavailable);
    }
    let envelope = super::envelope(source, event)?;
    // Bind the same validated source/account/revision identity used by the
    // ingress stream, while satisfying the ontology's entity-id vocabulary.
    let object_ref = format!(
        "ent_mail_{:x}",
        Sha256::digest(envelope.idempotency_key.as_bytes())
    );
    let mut note: serde_json::Value =
        serde_json::from_slice(&envelope.payload).map_err(|_| Error::Unavailable)?;
    note["observedAtMs"] = envelope.observed_at_epoch_ms.into();
    Ok(SubmitRequest {
        object_ref,
        action_type: "aty_record_write".into(),
        idempotency_key: envelope.idempotency_key,
        occurred_at_epoch_seconds: envelope.observed_at_epoch_ms / 1000,
        properties: BTreeMap::from([
            ("name".into(), "Mail account revision".into()),
            ("note".into(), note.to_string()),
        ]),
    })
}

fn submission_error(error: SubmitError) -> Error {
    match error {
        SubmitError::Credential
        | SubmitError::UnservedTenant
        | SubmitError::TenantMismatch
        | SubmitError::Authorization
        | SubmitError::Refused { .. } => Error::Forbidden,
        SubmitError::Surface { .. } => Error::Invalid,
        SubmitError::Conflict => Error::Conflict,
        SubmitError::Unavailable => Error::Unavailable,
    }
}
