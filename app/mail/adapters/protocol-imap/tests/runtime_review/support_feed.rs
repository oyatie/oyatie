//! `BlobStore` and `ChangeFeed` forwarding for the Adapter double.
use super::Adapter;
use mail_api::{AuditRow, BlobStore, ChangeFeed, Consumer, Dirty, FeedRead, Resume};
use mail_kernel::{BlobRef, Error};

impl BlobStore for Adapter {
    fn persist(&self, a: &str, s: &str, r: &[u8], ttl: i64) -> Result<BlobRef, Error> {
        self.db.persist(a, s, r, ttl)
    }
    fn renew(&self, a: &str, s: &str, ttl: i64) -> Result<(), Error> {
        self.db.renew(a, s, ttl)
    }
    fn read(&self, a: &str, b: &BlobRef) -> Result<Vec<u8>, Error> {
        self.db.read(a, b)
    }
    fn orphan_sweep(&self, now: i64, limit: usize) -> Result<usize, Error> {
        self.db.orphan_sweep(now, limit)
    }
}

impl ChangeFeed for Adapter {
    fn dirty(
        &self,
        c: Consumer,
        r: &mut Resume,
        now: i64,
        per: usize,
        limit: usize,
    ) -> Result<Vec<Dirty>, Error> {
        self.db.dirty(c, r, now, per, limit)
    }
    fn changes(&self, c: Consumer, a: &str, limit: usize) -> Result<FeedRead, Error> {
        self.db.changes(c, a, limit)
    }
    fn acknowledge(
        &self,
        c: Consumer,
        a: &str,
        rev: u64,
        now: i64,
        delay: i64,
    ) -> Result<(), Error> {
        self.db.acknowledge(c, a, rev, now, delay)
    }
    fn poison(&self, c: Consumer, a: &str, reason: &str) -> Result<(), Error> {
        self.db.poison(c, a, reason)
    }
    fn poisoned(&self, c: Consumer) -> Result<Vec<(Dirty, String)>, Error> {
        self.db.poisoned(c)
    }
    fn cursors(&self, a: &str, enabled: &[Consumer]) -> Result<Vec<(Consumer, u64)>, Error> {
        self.db.cursors(a, enabled)
    }
    fn reconcile(&self, c: Consumer, op: &str, reason: &str) -> Result<u64, Error> {
        self.db.reconcile(c, op, reason)
    }
    fn dead_letter(
        &self,
        c: Consumer,
        a: &str,
        rev: u64,
        op: &str,
        reason: &str,
    ) -> Result<(), Error> {
        self.db.dead_letter(c, a, rev, op, reason)
    }
    fn retire_cursor(
        &self,
        c: Consumer,
        t: Option<&str>,
        op: &str,
        reason: &str,
    ) -> Result<u64, Error> {
        self.db.retire_cursor(c, t, op, reason)
    }
    fn audit(&self, limit: usize) -> Result<Vec<AuditRow>, Error> {
        self.db.audit(limit)
    }
}
