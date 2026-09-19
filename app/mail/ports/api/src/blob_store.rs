use mail_kernel::{BlobRef, Command, Error};
use sha2::{Digest, Sha256};

/// Post-literal window a command's bodies stay reserved: the 300 s literal
/// deadline plus skew and the retry budget, measured from the last literal.
pub const COMMAND_RESERVATION_SECS: i64 = 300 + 60;
/// JMAP uploads stay reserved for 24 h (RFC 8620 §6.1).
pub const UPLOAD_RESERVATION_SECS: i64 = 86_400;

/// Bodies live outside the metadata transaction. `persist` is unfenced and
/// idempotent; a reservation scoped to the persisting command keeps the body
/// alive until the metadata commit links it; the sweep is the sole expiry
/// decider. A reference binds `(hash, version_id)`, so bytes re-persisted
/// after a sweep are a new version and an old reference stays dangling.
pub trait BlobStore: Send + Sync {
    /// Persist `raw` for `account` under the reservation `scope`, creating or
    /// renewing that reservation to `now + ttl_secs` on its own key.
    fn persist(
        &self,
        account: &str,
        scope: &str,
        raw: &[u8],
        ttl_secs: i64,
    ) -> Result<BlobRef, Error>;
    /// Renew `scope`'s reservation without touching any body.
    fn renew(&self, account: &str, scope: &str, ttl_secs: i64) -> Result<(), Error>;
    /// Bytes of a body this account links or holds a live reservation for;
    /// a hash the account does not own is `NotFound`, never a hit.
    fn read(&self, account: &str, blob: &BlobRef) -> Result<Vec<u8>, Error>;
    /// Delete up to `limit` bodies with no link and no live reservation,
    /// writing a tombstone for each in the same transaction; returns the count.
    fn orphan_sweep(&self, now: i64, limit: usize) -> Result<usize, Error>;

    /// One body, one command: persist under a scope of its own and return
    /// the `Append` that references it.
    fn append(
        &self,
        account: &str,
        mailboxes: Vec<String>,
        raw: &[u8],
        keywords: Vec<String>,
        received_at: i64,
    ) -> Result<Command, Error> {
        let scope = format!("append:{:x}", Sha256::digest(raw));
        let blob = self.persist(account, &scope, raw, COMMAND_RESERVATION_SECS)?;
        Ok(Command::Append {
            mailboxes,
            received_at,
            blob,
            keywords,
        })
    }
}
