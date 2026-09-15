#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod txn;

use messenger_domain::validate_admit;

pub const MAX_ADMIT_ATTEMPTS: u32 = 32;

pub use messenger_conversation_api::RoomAuthority;
pub use messenger_domain::{
    Admission, AdmitCommand, AuthorityEvent, AuthorityRecord, AuthoritySync, Error, send_endpoint,
};
pub use txn::{admit_into, txn_key};

/// Admit one event: reuse a committed transaction identity, bind the outbox
/// result to that identity, and retry serializable conflicts.
pub async fn admit<A: RoomAuthority>(
    authority: &A,
    command: &AdmitCommand,
) -> Result<Admission, Error> {
    validate_admit(command)?;
    let mut attempts = 0;
    loop {
        attempts += 1;
        match attempt(authority, command).await {
            Ok(admission) => return Ok(admission),
            Err(Error::Unavailable(_)) if attempts < MAX_ADMIT_ATTEMPTS => {}
            Err(error) => return Err(error),
        }
    }
}

async fn attempt<A: RoomAuthority>(
    authority: &A,
    command: &AdmitCommand,
) -> Result<Admission, Error> {
    let snapshot = authority.snapshot().await?;
    let expected = snapshot.generation;
    let (next, admission) = admit_into(&snapshot, command)?;
    authority.commit(expected, next).await?;
    Ok(admission)
}
