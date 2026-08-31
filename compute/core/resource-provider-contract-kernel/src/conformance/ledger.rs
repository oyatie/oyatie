use crate::{OperationState, ProviderError, ResourceProvider};

use super::ledger_match::assert_operation_ledger_matches;
use super::{ConformanceFixture, ConformanceViolation, MAX_OPERATION_POLLS, violation};

/// Operation-ledger semantics for AIP-151 mutations: the durable ledger row
/// exists before acknowledgement, carries the idempotency key/request hash,
/// replays return the same operation, key reuse for a different mutation is
/// rejected, transitions are monotonic, and terminal ledger snapshots are
/// immutable.
pub async fn check_operation_ledger_semantics<F: ConformanceFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "operation_ledger";
    let mut provider = fixture.fresh_provider();
    let first_name = fixture.resource_name(1)?;
    let second_name = fixture.resource_name(2)?;
    provider
        .create(
            &first_name,
            fixture.resource_payload(1),
            &fixture.idempotency_key(101)?,
        )
        .await
        .map_err(|e| violation(CHECK, format!("seeding first create failed: {e}")))?;
    provider
        .create(
            &second_name,
            fixture.resource_payload(2),
            &fixture.idempotency_key(102)?,
        )
        .await
        .map_err(|e| violation(CHECK, format!("seeding second create failed: {e}")))?;

    let delete_key = fixture.idempotency_key(201)?;
    let mut operation = provider
        .delete(&first_name, &delete_key)
        .await
        .map_err(|e| violation(CHECK, format!("delete failed: {e}")))?;
    let operation_name = operation.name.clone();
    let initial_ledger = provider
        .operation_ledger_entry(&operation_name)
        .await
        .map_err(|e| violation(CHECK, format!("ledger read after delete failed: {e}")))?;
    assert_operation_ledger_matches(
        CHECK,
        fixture,
        &operation,
        &initial_ledger,
        &first_name,
        &delete_key,
    )?;

    let replay = provider
        .delete(&first_name, &delete_key)
        .await
        .map_err(|e| violation(CHECK, format!("delete replay failed: {e}")))?;
    if replay.name != operation_name {
        return Err(violation(
            CHECK,
            format!(
                "delete replay under the same key must return operation {operation_name:?}, got {:?}",
                replay.name
            ),
        ));
    }
    let replay_ledger = provider
        .operation_ledger_entry(&operation_name)
        .await
        .map_err(|e| violation(CHECK, format!("ledger read after replay failed: {e}")))?;
    if replay_ledger != initial_ledger {
        return Err(violation(
            CHECK,
            "idempotent replay must not create or mutate the operation ledger row".to_owned(),
        ));
    }
    assert_operation_ledger_matches(
        CHECK,
        fixture,
        &replay,
        &replay_ledger,
        &first_name,
        &delete_key,
    )?;
    if replay != operation {
        return Err(violation(
            CHECK,
            "immediate idempotent replay must return the same operation snapshot before polling"
                .to_owned(),
        ));
    }

    match provider.delete(&second_name, &delete_key).await {
        Err(ProviderError::IdempotencyKeyReuse { .. }) => {}
        other => {
            return Err(violation(
                CHECK,
                format!(
                    "same idempotency key against a different mutation must fail with idempotency_key_reuse, got {other:?}"
                ),
            ));
        }
    }

    let mut last_ledger = initial_ledger.clone();
    for _ in 0..MAX_OPERATION_POLLS {
        if operation.done {
            break;
        }
        operation = provider
            .poll_operation(&operation_name)
            .await
            .map_err(|e| violation(CHECK, format!("poll failed: {e}")))?;
        let ledger = provider
            .operation_ledger_entry(&operation_name)
            .await
            .map_err(|e| violation(CHECK, format!("ledger read after poll failed: {e}")))?;
        assert_operation_ledger_matches(
            CHECK,
            fixture,
            &operation,
            &ledger,
            &first_name,
            &delete_key,
        )?;
        if ledger.transition_sequence < last_ledger.transition_sequence {
            return Err(violation(
                CHECK,
                format!(
                    "transition_sequence regressed from {} to {}",
                    last_ledger.transition_sequence, ledger.transition_sequence
                ),
            ));
        }
        if ledger.state != last_ledger.state && !last_ledger.state.can_transition_to(ledger.state) {
            return Err(violation(
                CHECK,
                format!(
                    "operation state transition {:?} -> {:?} is not allowed",
                    last_ledger.state, ledger.state
                ),
            ));
        }
        last_ledger = ledger;
    }
    if !operation.done {
        return Err(violation(
            CHECK,
            format!("operation did not reach done within {MAX_OPERATION_POLLS} polls"),
        ));
    }
    if operation.metadata.state != OperationState::Succeeded {
        return Err(violation(
            CHECK,
            format!(
                "successful delete ledger must terminate as succeeded, got {:?}",
                operation.metadata.state
            ),
        ));
    }
    if !initial_ledger.state.is_terminal()
        && last_ledger.transition_sequence <= initial_ledger.transition_sequence
    {
        return Err(violation(
            CHECK,
            "non-terminal operations must advance transition_sequence before terminal".to_owned(),
        ));
    }

    let terminal_repoll = provider
        .poll_operation(&operation_name)
        .await
        .map_err(|e| violation(CHECK, format!("terminal re-poll failed: {e}")))?;
    let terminal_ledger_repoll = provider
        .operation_ledger_entry(&operation_name)
        .await
        .map_err(|e| violation(CHECK, format!("terminal ledger re-read failed: {e}")))?;
    if terminal_repoll != operation || terminal_ledger_repoll != last_ledger {
        return Err(violation(
            CHECK,
            "terminal operation and ledger row must be immutable".to_owned(),
        ));
    }

    let terminal_replay = provider
        .delete(&first_name, &delete_key)
        .await
        .map_err(|e| violation(CHECK, format!("terminal delete replay failed: {e}")))?;
    let terminal_replay_ledger = provider
        .operation_ledger_entry(&operation_name)
        .await
        .map_err(|e| violation(CHECK, format!("terminal replay ledger read failed: {e}")))?;
    assert_operation_ledger_matches(
        CHECK,
        fixture,
        &terminal_replay,
        &terminal_replay_ledger,
        &first_name,
        &delete_key,
    )?;
    if terminal_replay != operation || terminal_replay_ledger != last_ledger {
        return Err(violation(
            CHECK,
            "terminal idempotent replay must return the terminal operation snapshot without mutating the ledger row"
                .to_owned(),
        ));
    }

    match provider
        .operation_ledger_entry("operations/never-issued")
        .await
    {
        Err(ProviderError::NotFound { .. }) => Ok(()),
        other => Err(violation(
            CHECK,
            format!(
                "reading an unknown operation ledger row must fail with not_found, got {other:?}"
            ),
        )),
    }
}
