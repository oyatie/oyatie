//! Raw `libfdb_c` declarations, transcribed one per line from `fdb_c.h` of
//! release 7.3.79 at header version 730 (hence the rustfmt skip). Option and
//! enum codes live on the typed enums in `options.rs`. Nothing here is called
//! outside this module tree.

use std::ffi::{c_char, c_int, c_void};

pub type FdbError = c_int;
pub type FdbBool = c_int;

#[repr(C)]
pub struct FDBFuture {
    _opaque: [u8; 0],
}
#[repr(C)]
pub struct FDBDatabase {
    _opaque: [u8; 0],
}
#[repr(C)]
pub struct FDBTransaction {
    _opaque: [u8; 0],
}

// `#pragma pack(push, 4)` in the header: 24 bytes, not 32.
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
pub struct FDBKeyValue {
    pub key: *const u8,
    pub key_length: c_int,
    pub value: *const u8,
    pub value_length: c_int,
}

pub type FDBCallback = unsafe extern "C" fn(future: *mut FDBFuture, parameter: *mut c_void);

// `Int` option values are passed as 8-byte little-endian buffers.
pub const INT_OPTION_LEN: c_int = 8;

#[rustfmt::skip]
#[link(name = "fdb_c")]
unsafe extern "C" {
    pub fn fdb_select_api_version_impl(runtime_version: c_int, header_version: c_int) -> FdbError;
    pub fn fdb_get_error(code: FdbError) -> *const c_char;
    pub fn fdb_error_predicate(predicate_test: c_int, code: FdbError) -> FdbBool;
    pub fn fdb_setup_network() -> FdbError;
    pub fn fdb_run_network() -> FdbError;
    pub fn fdb_stop_network() -> FdbError;

    pub fn fdb_create_database(cluster_file_path: *const c_char, out: *mut *mut FDBDatabase) -> FdbError;
    pub fn fdb_database_destroy(d: *mut FDBDatabase);
    pub fn fdb_database_create_transaction(d: *mut FDBDatabase, out: *mut *mut FDBTransaction) -> FdbError;

    pub fn fdb_transaction_destroy(tr: *mut FDBTransaction);
    pub fn fdb_transaction_set_option(tr: *mut FDBTransaction, option: c_int, value: *const u8, value_length: c_int) -> FdbError;
    pub fn fdb_transaction_set_read_version(tr: *mut FDBTransaction, version: i64);
    pub fn fdb_transaction_get_read_version(tr: *mut FDBTransaction) -> *mut FDBFuture;
    pub fn fdb_transaction_get(tr: *mut FDBTransaction, key: *const u8, key_length: c_int, snapshot: FdbBool) -> *mut FDBFuture;
    pub fn fdb_transaction_get_range(tr: *mut FDBTransaction, begin_key: *const u8, begin_key_length: c_int, begin_or_equal: FdbBool, begin_offset: c_int, end_key: *const u8, end_key_length: c_int, end_or_equal: FdbBool, end_offset: c_int, limit: c_int, target_bytes: c_int, mode: c_int, iteration: c_int, snapshot: FdbBool, reverse: FdbBool) -> *mut FDBFuture;
    pub fn fdb_transaction_set(tr: *mut FDBTransaction, key: *const u8, key_length: c_int, value: *const u8, value_length: c_int);
    pub fn fdb_transaction_atomic_op(tr: *mut FDBTransaction, key: *const u8, key_length: c_int, param: *const u8, param_length: c_int, operation_type: c_int);
    pub fn fdb_transaction_clear(tr: *mut FDBTransaction, key: *const u8, key_length: c_int);
    pub fn fdb_transaction_clear_range(tr: *mut FDBTransaction, begin_key: *const u8, begin_key_length: c_int, end_key: *const u8, end_key_length: c_int);
    pub fn fdb_transaction_watch(tr: *mut FDBTransaction, key: *const u8, key_length: c_int) -> *mut FDBFuture;
    pub fn fdb_transaction_commit(tr: *mut FDBTransaction) -> *mut FDBFuture;
    pub fn fdb_transaction_get_committed_version(tr: *mut FDBTransaction, out_version: *mut i64) -> FdbError;
    pub fn fdb_transaction_on_error(tr: *mut FDBTransaction, error: FdbError) -> *mut FDBFuture;
    pub fn fdb_transaction_reset(tr: *mut FDBTransaction);
    pub fn fdb_transaction_add_conflict_range(tr: *mut FDBTransaction, begin_key: *const u8, begin_key_length: c_int, end_key: *const u8, end_key_length: c_int, kind: c_int) -> FdbError;

    pub fn fdb_future_destroy(f: *mut FDBFuture);
    pub fn fdb_future_release_memory(f: *mut FDBFuture);
    pub fn fdb_future_block_until_ready(f: *mut FDBFuture) -> FdbError;
    pub fn fdb_future_is_ready(f: *mut FDBFuture) -> FdbBool;
    pub fn fdb_future_set_callback(f: *mut FDBFuture, callback: FDBCallback, parameter: *mut c_void) -> FdbError;
    pub fn fdb_future_get_error(f: *mut FDBFuture) -> FdbError;
    pub fn fdb_future_get_int64(f: *mut FDBFuture, out: *mut i64) -> FdbError;
    pub fn fdb_future_get_value(f: *mut FDBFuture, out_present: *mut FdbBool, out_value: *mut *const u8, out_value_length: *mut c_int) -> FdbError;
    pub fn fdb_future_get_key(f: *mut FDBFuture, out_key: *mut *const u8, out_key_length: *mut c_int) -> FdbError;
    pub fn fdb_future_get_keyvalue_array(f: *mut FDBFuture, out_kv: *mut *const FDBKeyValue, out_count: *mut c_int, out_more: *mut FdbBool) -> FdbError;
}

// C runtime, not libfdb_c: the network thread must be stopped before the
// process tears down its statics.
unsafe extern "C" {
    pub fn atexit(callback: unsafe extern "C" fn()) -> c_int;
}

/// Pointer and C length of a byte slice, as every key/value parameter wants.
/// Oversize inputs saturate and libfdb_c refuses them (keys > 10 KB, values
/// > 100 KB).
pub fn parts(bytes: &[u8]) -> (*const u8, c_int) {
    (
        bytes.as_ptr(),
        c_int::try_from(bytes.len()).unwrap_or(c_int::MAX),
    )
}
