//! `FDBTransaction`: the unit of atomicity. `Send` but not `Sync`; one task
//! drives one transaction at a time.
//!
//! SAFETY, for every call below: `raw` is a live transaction owned by `self`
//! until `Drop`, and libfdb_c copies every key/value/param buffer before the
//! call returns, so borrowed slices never need to outlive it.

use std::ffi::c_int;
use std::ptr::NonNull;

use super::error::{FdbError, check};
use super::future::FdbFuture;
use super::options::{
    ConflictRangeType, KeySelector, MutationType, RangeOptions, TransactionOption,
};
use super::sys::{self, parts};
use crate::key::VersionstampedKey;

pub struct Transaction {
    raw: NonNull<sys::FDBTransaction>,
}

// SAFETY: a transaction may move between threads; concurrent use is
// prevented by the absence of `Sync`.
unsafe impl Send for Transaction {}

impl Transaction {
    pub(super) const fn from_raw(raw: NonNull<sys::FDBTransaction>) -> Self {
        Self { raw }
    }

    fn ptr(&self) -> *mut sys::FDBTransaction {
        self.raw.as_ptr()
    }

    pub fn set_option(&self, option: TransactionOption, value: i64) -> Result<(), FdbError> {
        let bytes = value.to_le_bytes();
        let (p, n) = (bytes.as_ptr(), sys::INT_OPTION_LEN);
        unsafe {
            check(sys::fdb_transaction_set_option(
                self.ptr(),
                option as c_int,
                p,
                n,
            ))
        }
    }

    pub fn set_read_version(&self, version: i64) {
        unsafe { sys::fdb_transaction_set_read_version(self.ptr(), version) }
    }

    pub fn get_read_version(&self) -> FdbFuture {
        FdbFuture::from_raw(unsafe { sys::fdb_transaction_get_read_version(self.ptr()) })
    }

    pub fn get(&self, key: &[u8], snapshot: bool) -> FdbFuture {
        let (k, kn) = parts(key);
        FdbFuture::from_raw(unsafe {
            sys::fdb_transaction_get(self.ptr(), k, kn, c_int::from(snapshot))
        })
    }

    pub fn get_range(
        &self,
        begin: KeySelector<'_>,
        end: KeySelector<'_>,
        o: RangeOptions,
    ) -> FdbFuture {
        let ((b, bn), (e, en)) = (parts(begin.key), parts(end.key));
        let (beq, eeq) = (c_int::from(begin.or_equal), c_int::from(end.or_equal));
        let (snapshot, reverse) = (c_int::from(o.snapshot), c_int::from(o.reverse));
        FdbFuture::from_raw(unsafe {
            sys::fdb_transaction_get_range(
                self.ptr(),
                b,
                bn,
                beq,
                begin.offset,
                e,
                en,
                eeq,
                end.offset,
                o.limit,
                o.target_bytes,
                o.mode as c_int,
                o.iteration,
                snapshot,
                reverse,
            )
        })
    }

    pub fn set(&self, key: &[u8], value: &[u8]) {
        let ((k, kn), (v, vn)) = (parts(key), parts(value));
        unsafe { sys::fdb_transaction_set(self.ptr(), k, kn, v, vn) }
    }

    pub fn clear(&self, key: &[u8]) {
        let (k, kn) = parts(key);
        unsafe { sys::fdb_transaction_clear(self.ptr(), k, kn) }
    }

    pub fn clear_range(&self, begin: &[u8], end: &[u8]) {
        let ((b, bn), (e, en)) = (parts(begin), parts(end));
        unsafe { sys::fdb_transaction_clear_range(self.ptr(), b, bn, e, en) }
    }

    pub fn atomic_op(&self, key: &[u8], param: &[u8], operation: MutationType) {
        let ((k, kn), (p, pn)) = (parts(key), parts(param));
        unsafe { sys::fdb_transaction_atomic_op(self.ptr(), k, kn, p, pn, operation as c_int) }
    }

    /// Writes `value` under `key` with its placeholder replaced by this
    /// transaction's versionstamp at commit.
    pub fn set_versionstamped_key(&self, key: &VersionstampedKey, value: &[u8]) {
        self.atomic_op(
            key.atomic_param(),
            value,
            MutationType::SetVersionstampedKey,
        );
    }

    /// Resolves when `key`'s value changes from what this transaction saw.
    pub fn watch(&self, key: &[u8]) -> FdbFuture {
        let (k, kn) = parts(key);
        FdbFuture::from_raw(unsafe { sys::fdb_transaction_watch(self.ptr(), k, kn) })
    }

    pub fn commit(&self) -> FdbFuture {
        FdbFuture::from_raw(unsafe { sys::fdb_transaction_commit(self.ptr()) })
    }

    /// Valid only after `commit` resolved successfully.
    pub fn committed_version(&self) -> Result<i64, FdbError> {
        let mut version = 0;
        unsafe {
            check(sys::fdb_transaction_get_committed_version(
                self.ptr(),
                &mut version,
            ))?
        };
        Ok(version)
    }

    /// Backs off per the client's retry policy; resolves `Ok` when the
    /// transaction may be retried, `Err` when the error is terminal.
    pub fn on_error(&self, error: FdbError) -> FdbFuture {
        FdbFuture::from_raw(unsafe { sys::fdb_transaction_on_error(self.ptr(), error.code()) })
    }

    pub fn reset(&self) {
        unsafe { sys::fdb_transaction_reset(self.ptr()) }
    }

    pub fn add_conflict_range(
        &self,
        begin: &[u8],
        end: &[u8],
        kind: ConflictRangeType,
    ) -> Result<(), FdbError> {
        let ((b, bn), (e, en)) = (parts(begin), parts(end));
        unsafe {
            check(sys::fdb_transaction_add_conflict_range(
                self.ptr(),
                b,
                bn,
                e,
                en,
                kind as c_int,
            ))
        }
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        // SAFETY: destroyed exactly once, here; outstanding futures hold
        // their own references inside libfdb_c.
        unsafe { sys::fdb_transaction_destroy(self.ptr()) }
    }
}
