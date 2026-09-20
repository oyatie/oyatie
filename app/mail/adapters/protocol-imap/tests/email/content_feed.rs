//! `BlobStore` and `ChangeFeed` forwarding for the ObservedStore double.
use super::ObservedStore;
use mail_api::{AuditRow, BlobStore, ChangeFeed, Consumer, Dirty, FeedRead, Resume};
use mail_kernel::{BlobRef, Error};

impl BlobStore for ObservedStore {
    fn persist(&self, a: &str, s: &str, r: &[u8], ttl: i64) -> Result<BlobRef, Error> {
        self.inner.persist(a, s, r, ttl)
    }
    fn renew(&self, a: &str, s: &str, ttl: i64) -> Result<(), Error> {
        self.inner.renew(a, s, ttl)
    }
    fn read(&self, a: &str, b: &BlobRef) -> Result<Vec<u8>, Error> {
        self.inner.read(a, b)
    }
    fn orphan_sweep(&self, now: i64, limit: usize) -> Result<usize, Error> {
        self.inner.orphan_sweep(now, limit)
    }
}

impl ChangeFeed for ObservedStore {
    fn dirty(
        &self,
        c: Consumer,
        r: &mut Resume,
        now: i64,
        per: usize,
        limit: usize,
    ) -> Result<Vec<Dirty>, Error> {
        self.inner.dirty(c, r, now, per, limit)
    }
    fn changes(&self, c: Consumer, a: &str, limit: usize) -> Result<FeedRead, Error> {
        self.inner.changes(c, a, limit)
    }
    fn acknowledge(
        &self,
        c: Consumer,
        a: &str,
        rev: u64,
        now: i64,
        delay: i64,
    ) -> Result<(), Error> {
        self.inner.acknowledge(c, a, rev, now, delay)
    }
    fn poison(&self, c: Consumer, a: &str, reason: &str) -> Result<(), Error> {
        self.inner.poison(c, a, reason)
    }
    fn poisoned(&self, c: Consumer) -> Result<Vec<(Dirty, String)>, Error> {
        self.inner.poisoned(c)
    }
    fn cursors(&self, a: &str, enabled: &[Consumer]) -> Result<Vec<(Consumer, u64)>, Error> {
        self.inner.cursors(a, enabled)
    }
    fn reconcile(&self, c: Consumer, op: &str, reason: &str) -> Result<u64, Error> {
        self.inner.reconcile(c, op, reason)
    }
    fn dead_letter(
        &self,
        c: Consumer,
        a: &str,
        rev: u64,
        op: &str,
        reason: &str,
    ) -> Result<(), Error> {
        self.inner.dead_letter(c, a, rev, op, reason)
    }
    fn retire_cursor(
        &self,
        c: Consumer,
        t: Option<&str>,
        op: &str,
        reason: &str,
    ) -> Result<u64, Error> {
        self.inner.retire_cursor(c, t, op, reason)
    }
    fn audit(&self, limit: usize) -> Result<Vec<AuditRow>, Error> {
        self.inner.audit(limit)
    }
}
