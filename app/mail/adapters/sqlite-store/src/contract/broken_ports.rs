//! `Broken`'s remaining violations: the resume position is ignored (every
//! round starts at the first tenant), and orphan bodies are never swept.
use super::Broken;
use mail_api::{
    AuditRow, BlobStore, ChangeFeed, Consumer, Dirty, FeedRead, MetadataStore, Resume,
    SubmissionStore,
};
use mail_kernel::{BlobRef, Error};

impl<T: MetadataStore + SubmissionStore + BlobStore + ChangeFeed> ChangeFeed for Broken<T> {
    fn dirty(
        &self,
        c: Consumer,
        _resume: &mut Resume,
        now: i64,
        per: usize,
        limit: usize,
    ) -> Result<Vec<Dirty>, Error> {
        self.inner.dirty(c, &mut Resume::default(), now, per, limit)
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
    fn cursors(&self, a: &str, e: &[Consumer]) -> Result<Vec<(Consumer, u64)>, Error> {
        self.inner.cursors(a, e)
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

impl<T: MetadataStore + SubmissionStore + BlobStore + ChangeFeed> BlobStore for Broken<T> {
    fn persist(&self, account: &str, scope: &str, raw: &[u8], ttl: i64) -> Result<BlobRef, Error> {
        self.inner.persist(account, scope, raw, ttl)
    }
    fn renew(&self, account: &str, scope: &str, ttl: i64) -> Result<(), Error> {
        self.inner.renew(account, scope, ttl)
    }
    fn read(&self, account: &str, blob: &BlobRef) -> Result<Vec<u8>, Error> {
        self.inner.read(account, blob)
    }
    /// Never sweeps: a cancelled upload's body stays forever.
    fn orphan_sweep(&self, _now: i64, _limit: usize) -> Result<usize, Error> {
        Ok(0)
    }
}
