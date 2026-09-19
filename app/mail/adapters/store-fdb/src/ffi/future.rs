//! `FDBFuture` as a Rust `Future`: readiness arrives through
//! `fdb_future_set_callback`, which wakes the task's `Waker`.

use std::ffi::c_void;
use std::future::Future;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use super::error::{FdbError, check};
use super::sys;

/// What an `FDBFuture` resolves to; the C API leaves reading the wrong type
/// undefined, so every getter checks the tag first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Void,
    Int64,
    Value,
    Key,
    KeyValues,
}

/// A pending `libfdb_c` operation. Resolves to [`Ready`] or the operation's
/// error; dropping it before completion cancels the operation.
pub struct FdbFuture {
    raw: NonNull<sys::FDBFuture>,
    kind: Kind,
    slot: Option<Arc<Slot>>,
    taken: bool,
}

struct Slot(Mutex<Option<Waker>>);

/// A completed, successful future; values are copied out on demand.
pub struct Ready {
    raw: NonNull<sys::FDBFuture>,
    kind: Kind,
}

/// One page of a range read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyValues {
    pub pairs: Vec<(Vec<u8>, Vec<u8>)>,
    pub more: bool,
}

// SAFETY: the C API documents FDBFuture as safe to use from any thread; a
// `Ready` only ever reads through getters that copy out under `&self`.
unsafe impl Send for FdbFuture {}
unsafe impl Send for Ready {}
unsafe impl Sync for Ready {}

impl FdbFuture {
    pub(super) fn from_raw(raw: *mut sys::FDBFuture, kind: Kind) -> Self {
        let raw = NonNull::new(raw).expect("libfdb_c returned a null FDBFuture");
        let (slot, taken) = (None, false);
        Self {
            raw,
            kind,
            slot,
            taken,
        }
    }

    fn ready(&mut self) -> Ready {
        self.taken = true;
        Ready {
            raw: self.raw,
            kind: self.kind,
        }
    }

    /// Blocks the calling thread until the operation completes.
    pub fn block(mut self) -> Result<Ready, FdbError> {
        // SAFETY: raw is a live future owned by self.
        unsafe {
            check(sys::fdb_future_block_until_ready(self.raw.as_ptr()))?;
            check(sys::fdb_future_get_error(self.raw.as_ptr()))?;
        }
        Ok(self.ready())
    }
}

impl Future for FdbFuture {
    type Output = Result<Ready, FdbError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Once taken the handle belongs to a `Ready`, which may have destroyed it.
        assert!(!self.taken, "FdbFuture polled after completion");
        let raw = self.raw.as_ptr();
        // SAFETY: raw is a live future owned by self for every call below.
        if unsafe { sys::fdb_future_is_ready(raw) } != 0 {
            // An errored future stays owned by self so Drop destroys it.
            return Poll::Ready(
                unsafe { check(sys::fdb_future_get_error(raw)) }.map(|()| self.ready()),
            );
        }
        if let Some(slot) = &self.slot {
            *slot.0.lock().unwrap_or_else(|poison| poison.into_inner()) = Some(cx.waker().clone());
            return Poll::Pending;
        }
        let slot = Arc::new(Slot(Mutex::new(Some(cx.waker().clone()))));
        let parameter = Arc::into_raw(Arc::clone(&slot)).cast_mut().cast::<c_void>();
        self.slot = Some(slot);
        // SAFETY: the callback fires at most once and reclaims the Arc leaked
        // here; a future destroyed before firing leaks one `Slot`, never UB.
        match unsafe { check(sys::fdb_future_set_callback(raw, wake, parameter)) } {
            Ok(()) => Poll::Pending,
            Err(error) => {
                // SAFETY: the callback was not registered, so the Arc is ours.
                drop(unsafe { Arc::from_raw(parameter.cast_const().cast::<Slot>()) });
                Poll::Ready(Err(error))
            }
        }
    }
}

unsafe extern "C" fn wake(_future: *mut sys::FDBFuture, parameter: *mut c_void) {
    // SAFETY: parameter is the Arc leaked by poll; this is its only reclaim.
    let slot = unsafe { Arc::from_raw(parameter.cast_const().cast::<Slot>()) };
    let waker = slot
        .0
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .take();
    if let Some(waker) = waker {
        waker.wake();
    }
}

impl Drop for FdbFuture {
    fn drop(&mut self) {
        if !self.taken {
            // SAFETY: destroyed exactly once; cancels if still outstanding.
            unsafe { sys::fdb_future_destroy(self.raw.as_ptr()) }
        }
    }
}

/// SAFETY, for every getter: `raw` is a live, ready future owned by `self`
/// whose tag matches the getter; out-pointers are valid for the call;
/// returned memory lives until the future is destroyed and is copied
/// before the getter returns.
impl Ready {
    /// Error 2000 (`client_invalid_operation`) when the getter does not
    /// match what the future resolves to.
    fn expect(&self, kind: Kind) -> Result<(), FdbError> {
        (self.kind == kind)
            .then_some(())
            .ok_or(FdbError::from_code(2000))
    }

    /// `fdb_future_get_value`: `None` when the key is absent.
    pub fn value(&self) -> Result<Option<Vec<u8>>, FdbError> {
        self.expect(Kind::Value)?;
        let (f, mut present, mut ptr, mut len) = (self.raw.as_ptr(), 0, std::ptr::null(), 0);
        unsafe {
            check(sys::fdb_future_get_value(
                f,
                &mut present,
                &mut ptr,
                &mut len,
            ))?;
            Ok((present != 0).then(|| copy(ptr, len)))
        }
    }

    /// `fdb_future_get_key`.
    pub fn key(&self) -> Result<Vec<u8>, FdbError> {
        self.expect(Kind::Key)?;
        let (f, mut ptr, mut len) = (self.raw.as_ptr(), std::ptr::null(), 0);
        unsafe {
            check(sys::fdb_future_get_key(f, &mut ptr, &mut len))?;
            Ok(copy(ptr, len))
        }
    }

    /// `fdb_future_get_int64` (read versions).
    pub fn int64(&self) -> Result<i64, FdbError> {
        self.expect(Kind::Int64)?;
        let mut out = 0;
        unsafe { check(sys::fdb_future_get_int64(self.raw.as_ptr(), &mut out))? };
        Ok(out)
    }

    /// `fdb_future_get_keyvalue_array`, copied into owned pairs.
    pub fn key_values(&self) -> Result<KeyValues, FdbError> {
        self.expect(Kind::KeyValues)?;
        let (f, mut array, mut count, mut more) = (self.raw.as_ptr(), std::ptr::null(), 0, 0);
        unsafe {
            check(sys::fdb_future_get_keyvalue_array(
                f, &mut array, &mut count, &mut more,
            ))?;
            // An empty page may come back as a null array pointer.
            let count = if array.is_null() {
                0
            } else {
                usize::try_from(count).unwrap_or(0)
            };
            let pairs = (0..count)
                .map(|i| *array.add(i))
                .map(|kv| (copy(kv.key, kv.key_length), copy(kv.value, kv.value_length)))
                .collect();
            let more = more != 0;
            Ok(KeyValues { pairs, more })
        }
    }
}

impl Drop for Ready {
    fn drop(&mut self) {
        // SAFETY: destroyed exactly once, here.
        unsafe { sys::fdb_future_destroy(self.raw.as_ptr()) }
    }
}

/// # Safety
/// `ptr` must be readable for `len` bytes, or `len` must be zero.
unsafe fn copy(ptr: *const u8, len: std::ffi::c_int) -> Vec<u8> {
    let len = usize::try_from(len).unwrap_or(0);
    if len == 0 {
        return Vec::new();
    }
    // SAFETY: caller contract.
    unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
}

/// Drives one future to completion on the current thread. For tests and
/// tools; a serving runtime supplies its own executor.
pub fn block_on<F: Future>(future: F) -> F::Output {
    struct Unpark(std::thread::Thread);
    impl std::task::Wake for Unpark {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = Waker::from(Arc::new(Unpark(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::park(),
        }
    }
}
