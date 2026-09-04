use crate::{IdempotencyKey, OPERATION_NAME_PREFIX, Operation, OperationLedgerEntry, ResourceName};

use super::{ConformanceFixture, ConformanceViolation, violation};

pub(super) fn assert_operation_ledger_matches<F: ConformanceFixture>(
    check: &'static str,
    fixture: &F,
    operation: &Operation,
    ledger: &OperationLedgerEntry,
    name: &ResourceName,
    idempotency_key: &IdempotencyKey,
) -> Result<(), ConformanceViolation> {
    operation
        .validate()
        .map_err(|e| violation(check, format!("operation shape invalid: {e}")))?;
    ledger
        .validate()
        .map_err(|e| violation(check, format!("ledger shape invalid: {e}")))?;
    if operation.metadata != *ledger {
        return Err(violation(
            check,
            "operation metadata must be a snapshot of the durable ledger row".to_owned(),
        ));
    }
    let operation_id = operation
        .name
        .strip_prefix(OPERATION_NAME_PREFIX)
        .ok_or_else(|| violation(check, "operation name lacks AIP-151 prefix".to_owned()))?;
    if ledger.operation_id != operation_id {
        return Err(violation(
            check,
            format!(
                "ledger operation_id {:?} must match operation name {:?}",
                ledger.operation_id, operation.name
            ),
        ));
    }
    if ledger.idempotency_key != idempotency_key.as_str() {
        return Err(violation(
            check,
            format!(
                "ledger idempotency_key {:?} does not match request key {:?}",
                ledger.idempotency_key,
                idempotency_key.as_str()
            ),
        ));
    }
    let expected_orn = fixture.resource_orn(name);
    if ledger.resource_orn != expected_orn {
        return Err(violation(
            check,
            format!(
                "ledger resource_orn {:?} does not match expected {:?}",
                ledger.resource_orn, expected_orn
            ),
        ));
    }
    for (field, actual, expected) in [
        (
            "tenant_account_project",
            ledger.tenant_account_project.as_str(),
            fixture.tenant_account_project(),
        ),
        (
            "region_cell",
            ledger.region_cell.as_str(),
            fixture.region_cell(),
        ),
        ("principal", ledger.principal.as_str(), fixture.principal()),
    ] {
        if actual != expected {
            return Err(violation(
                check,
                format!("ledger {field} {actual:?} does not match expected {expected:?}"),
            ));
        }
    }
    Ok(())
}
