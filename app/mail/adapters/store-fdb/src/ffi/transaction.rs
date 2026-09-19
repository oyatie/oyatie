//! `FDBTransaction`: the unit of atomicity. `Send` but not `Sync`; one task
//! drives one transaction at a time.
//!
//! SAFETY, for every call below: `raw` is a live transaction owned by `self`
//! until `Drop`, and libfdb_c copies every key/value/param buffer before the
//! call returns, so borrowed slices never need to outlive it.

use std::ffi::c_int;
use std::ptr::NonNull;

use super::error::{FdbError, check};
use super::future::{FdbFuture, Kind};
use super::options::{KeySelector, RangeOptions};
use super::sys::{self, parts};

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

    pub fn get_read_version(&self) -> FdbFuture {
        FdbFuture::from_raw(
            unsafe { sys::fdb_transaction_get_read_version(self.ptr()) },
            Kind::Int64,
        )
    }

    pub fn get(&self, key: &[u8], snapshot: bool) -> FdbFuture {
        let (k, kn) = parts(key);
        FdbFuture::from_raw(
            unsafe { sys::fdb_transaction_get(self.ptr(), k, kn, c_int::from(snapshot)) },
            Kind::Value,
        )
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
        FdbFuture::from_raw(
            unsafe {
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
            },
            Kind::KeyValues,
        )
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

    pub fn commit(&self) -> FdbFuture {
        FdbFuture::from_raw(
            unsafe { sys::fdb_transaction_commit(self.ptr()) },
            Kind::Void,
        )
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
}

impl Drop for Transaction {
    fn drop(&mut self) {
        // SAFETY: destroyed exactly once, here; outstanding futures hold
        // their own references inside libfdb_c.
        unsafe { sys::fdb_transaction_destroy(self.ptr()) }
    }
}
