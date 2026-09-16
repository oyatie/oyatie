#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

mod txn;

use messenger_conversation_api::RoomAuthority;
use messenger_domain::{Admission, AdmitCommand, Error, validate_admit};

const MAX_ADMIT_ATTEMPTS: u32 = 32;

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
    let (next, admission) = txn::admit_into(&snapshot, command)?;
    authority.commit(expected, next).await?;
    Ok(admission)
}
