//! The discard path: empty one tenant's projection so a rebuild can
//! start from nothing.
//!
//! Separate from the write path because it is the one operation here
//! that destroys rather than accumulates, and that difference should be
//! visible in the file list rather than buried in a trait impl.

use foundry_projection_draft::ProjectionStoreError;
use rusqlite::{Connection, OptionalExtension};

use crate::{require_trimmed, storage};

pub(crate) fn reset_tenant(
    connection: &mut Connection,
    tenant_id: &str,
) -> Result<u64, ProjectionStoreError> {
    require_trimmed(tenant_id, "blank tenant")?;
    // One immediate transaction, like an apply: a discard that got
    // half-way would leave precisely the mixture it was called to
    // escape, and the head is cleared LAST so a crash mid-discard
    // leaves the head still claiming rows rather than claiming zero
    // over rows that are still there.
    let transaction = connection
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(storage)?;
    let discarded: u64 = transaction
        .query_row(
            "SELECT applied_ordinal FROM projection_heads WHERE tenant_id = ?1",
            [tenant_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(storage)?
        .unwrap_or(0)
        .try_into()
        .unwrap_or(0);
    for table in [
        "projection_entries",
        "projection_objects",
        "projection_property_index",
        "projection_links",
        "projection_poisons",
        "projection_heads",
    ] {
        transaction
            .execute(
                &format!("DELETE FROM {table} WHERE tenant_id = ?1"),
                [tenant_id],
            )
            .map_err(storage)?;
    }
    transaction.commit().map_err(storage)?;
    Ok(discarded)
}
