//! Gate 5, retention half — a stalled consumer holds the retention floor
//! over every change (nothing skipped, the account `retention-blocked`);
//! retiring its cursor releases the account in one compaction pass; a
//! consumer re-enabled with a cursor below the floor is poison and
//! acknowledges nothing.
use super::feed::{FOUNDRY, accounts, touch};
use super::{append, at};
use crate::contract::suite::{Fixture, Gate, GateFailure};
use mail_api::{ChangeFeed, FeedRead, MetadataStore, Precondition};
use mail_kernel::{Retention, RetentionPolicy};

/// Changes committed under a stalled consumer; the retention policy the
/// gate compacts with allows a tenth of them.
const CHANGES: usize = 2_000;

pub fn run<F: Fixture>(fixture: &F, gate: &mut Gate) -> Result<(), GateFailure> {
    let store = fixture.open(&[FOUNDRY], &accounts());
    let start = at(gate, "fixture", touch(&store, "a"))?;
    at(
        gate,
        "fixture",
        store.acknowledge(FOUNDRY, "a", start, 0, 0),
    )?;
    // The consumer stalls while CHANGES commits land.
    let mut n = 0;
    while n < CHANGES {
        let batch = (CHANGES - n).min(500);
        let mut commands = Vec::with_capacity(batch);
        for i in 0..batch {
            commands.push(at(
                gate,
                "fixture",
                append(&store, "inbox", n + i + 2, "", "R"),
            )?);
        }
        at(
            gate,
            "fixture",
            store.execute("a", Precondition::Observed(0), commands),
        )?;
        n += batch;
    }
    let policy = RetentionPolicy {
        max_age_secs: 0,
        max_rows: CHANGES as u64 / 10,
    };
    let cursors = at(gate, "retention blocked", store.cursors("a", &[FOUNDRY]))?;
    let retention = at(
        gate,
        "retention blocked",
        store.compact_history("a", i64::MAX / 2, policy, &cursors),
    )?;
    gate.check(
        "retention blocked",
        matches!(&retention, Retention::Blocked { consumer, .. } if consumer == FOUNDRY.name()),
        format!("{retention:?}"),
    )?;
    // Every change is still readable from the cursor, none skipped.
    let first = at(gate, "no row skipped", store.changes(FOUNDRY, "a", 1000))?;
    gate.check(
        "no row skipped",
        matches!(&first, FeedRead::Changes { cursor, .. } if cursor.0 == start),
        format!("{first:?}"),
    )?;
    let (mut rows, mut last) = (0, start);
    loop {
        let page = at(gate, "no row skipped", store.history("a", last, 1000))?;
        rows += page.rows.len();
        last = page.revision;
        if !page.has_more {
            break;
        }
    }
    gate.check(
        "no row skipped",
        rows == CHANGES,
        format!("{rows} of {CHANGES} rows readable"),
    )?;
    // Retiring the cursor releases the account in one compaction pass.
    at(
        gate,
        "cursor retire releases",
        store.retire_cursor(FOUNDRY, Some("t"), "ops@x.org", "consumer redeployed"),
    )?;
    let cursors = at(
        gate,
        "cursor retire releases",
        store.cursors("a", &[FOUNDRY]),
    )?;
    let after = at(
        gate,
        "cursor retire releases",
        store.compact_history("a", i64::MAX / 2, policy, &cursors),
    )?;
    gate.check(
        "cursor retire releases",
        matches!(after, Retention::Advanced { floor } if floor > start),
        format!("{after:?}"),
    )?;
    // A consumer re-enabled with a cursor below the floor is poison and
    // acknowledges nothing: acked at r1 while disabled, compaction passes
    // r1, then the consumer comes back.
    let disabled = fixture.open(&[], &accounts());
    let r1 = at(gate, "fixture", touch(&disabled, "a"))?;
    at(
        gate,
        "fixture",
        disabled.acknowledge(FOUNDRY, "a", r1, 0, 0),
    )?;
    for _ in 0..3 {
        at(gate, "fixture", touch(&disabled, "a"))?;
    }
    let tight = RetentionPolicy {
        max_age_secs: 0,
        max_rows: 1,
    };
    at(
        gate,
        "below floor",
        disabled.compact_history("a", i64::MAX / 2, tight, &[]),
    )?;
    let read = at(gate, "below floor", disabled.changes(FOUNDRY, "a", 10))?;
    gate.check(
        "below floor",
        matches!(read, FeedRead::BelowFloor { .. }),
        format!("{read:?}"),
    )?;
    let ack = disabled.acknowledge(FOUNDRY, "a", r1, 0, 0);
    gate.check(
        "below floor",
        ack.is_err(),
        format!("acknowledged below the floor: {ack:?}"),
    )?;
    let poisoned = at(gate, "below floor", disabled.poisoned(FOUNDRY))?;
    gate.check(
        "below floor",
        poisoned
            .iter()
            .any(|(d, reason)| d.account == "a" && reason == "cursor-below-floor"),
        format!("{poisoned:?}"),
    )
}
