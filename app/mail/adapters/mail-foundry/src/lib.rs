#![forbid(unsafe_code)]

use foundry_records_draft::{ActionEnvelope, RecordsLog, RecordsLogError};
use mail_api::{Event, Events};
use mail_kernel::{Error, valid_identifier};
use sha2::{Digest, Sha256};

mod submission;
pub use submission::publish_authorized;

/// Export references to committed account revisions. Consumers fetch message
/// and contact data through authorized APIs; the shared log contains no bodies
/// or credentials and an export grants no permission to read the source.
/// `source` is the stable mail-service identity; `consumer` identifies the
/// destination subscription. Neither should change when a worker moves hosts.
/// The log must be a dedicated mail ingress stream. Foundry's ontology action
/// log accepts canonical ontology edits through its authorized submission path;
/// these ingress records are not ontology edits and cannot be appended there.
pub fn publish(
    events: &dyn Events,
    log: &mut dyn RecordsLog,
    source: &str,
    consumer: &str,
    limit: usize,
) -> Result<usize, Error> {
    if !valid_identifier(source) || !valid_identifier(consumer) {
        return Err(Error::Invalid);
    }
    let mut published = 0;
    for event in events.pending(consumer, limit.min(1000))? {
        log.append(envelope(source, &event)?)
            .map_err(|error| match error {
                RecordsLogError::IdempotencyConflict { .. } => Error::Conflict,
                RecordsLogError::Storage { .. } => Error::Unavailable,
            })?;
        events.acknowledge(consumer, event.sequence)?;
        published += 1;
    }
    Ok(published)
}

fn envelope(source: &str, event: &Event) -> Result<ActionEnvelope, Error> {
    if !valid_identifier(&event.tenant)
        || !valid_identifier(&event.account)
        || event.revision == 0
        || event.observed_at_ms == 0
    {
        // Legacy outbox records without an observation time cannot be silently
        // assigned today's timestamp: that would fabricate provenance on replay.
        return Err(Error::Unavailable);
    }
    let object = format!(
        "mail/{:x}/account/{:x}",
        Sha256::digest(source.as_bytes()),
        Sha256::digest(event.account.as_bytes())
    );
    let key = format!("{object}/revision/{}", event.revision);
    let payload = serde_json::to_vec(
        &serde_json::json!({"source":source,"accountId":event.account,"revision":event.revision}),
    )
    .map_err(|_| Error::Unavailable)?;
    ActionEnvelope::new(
        &event.tenant,
        object,
        "mail.account.changed",
        key,
        1,
        payload,
        event.observed_at_ms,
    )
    .map_err(|_| Error::Invalid)
}
