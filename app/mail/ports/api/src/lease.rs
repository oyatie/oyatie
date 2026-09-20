//! The fences every writer shares: the owner epoch on a queue job, the node
//! lease every hosted process holds, and the per-account batch lease taken
//! only for a large link mutation's metadata commit.
use mail_kernel::Error;

/// A job's owner epoch: the claimant increments it in the claim transaction
/// and every later read or settlement of that job names it in the same
/// statement, so a worker whose lease expired and was re-claimed can neither
/// read the content nor settle the job. "Ever claimed" ⇔ epoch ≥ 1.
pub type Epoch = u64;

/// Seconds a queue lease lasts before another claimant may take the job.
pub const QUEUE_LEASE_SECS: i64 = 120;

/// Seconds a node lease lasts; a crashed node holds schema advance for at
/// most this long, the safe direction. Renewed at a third of the TTL.
pub const NODE_LEASE_SECS: i64 = 120;

/// Link mutations above this count take the per-account batch lease for
/// their metadata commit (MULTIAPPEND, COPY/MOVE, `Email/set`, `Email/import`).
pub const BATCH_LEASE_MESSAGES: usize = 100;
/// The batch lease never outlives min(protocol deadline, this): the TTL the
/// hosted store (S9) writes with the lease it takes inside `execute`.
pub const BATCH_LEASE_SECS: i64 = 5;

/// Every hosted process — protocol node or worker — takes and renews a
/// node-scoped lease whose value carries its binary schema version, so a
/// schema advance can see which versions are live. Any other writer that
/// reads a live batch lease for an account returns `Busy`; the SQLite tier
/// serializes on its connection, so both leases are no-ops there.
pub trait NodeLease: Send + Sync {
    fn heartbeat(&self, node: &str, schema_version: u64, ttl_secs: i64) -> Result<(), Error>;
}

/// Wall-clock seconds for callers outside a database transaction; inside
/// one, the database's own time (`unixepoch()`) is the clock. Skew between
/// the two must stay far below `NODE_LEASE_SECS`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Clock;
impl Clock {
    pub fn now_secs(self) -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }
}
