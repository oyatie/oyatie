//! Global console plumbing for the aarch64 Frame.
//!
//! Provides a process-wide [`Pl011`] behind a tiny spinlock plus `print!` /
//! `println!` macros the rest of the Frame uses. The safe kernel never touches
//! this directly; it drives the console through the [`hal::ConsoleWrite`] API.
//!
//! The kernel is single-core during bring-up (secondary cores are parked in the
//! boot stub), so the lock exists mainly to make the global mutable access
//! sound rather than to arbitrate real contention.

use core::cell::UnsafeCell;
use core::fmt::{self, Write};
use core::sync::atomic::{AtomicBool, Ordering};

use crate::uart::{Pl011, PL011_BASE};

/// A minimal spinlock — enough for the single-core bring-up console.
pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

// SAFETY: access to the inner value is gated by the atomic `locked` flag, which
// provides mutual exclusion; only one holder ever sees `&mut T`.
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    /// Create a new unlocked spinlock.
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    /// Run `f` with exclusive access to the protected value.
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        // SAFETY: we hold the lock (the CAS above succeeded), so we are the
        // unique accessor of the inner value for the duration of `f`.
        let r = f(unsafe { &mut *self.value.get() });
        self.locked.store(false, Ordering::Release);
        r
    }
}

/// The global console UART. Initialized in [`init`].
static CONSOLE: SpinLock<Option<Pl011>> = SpinLock::new(None);

/// Bring up the global console UART. Idempotent.
pub fn init() {
    CONSOLE.with(|slot| {
        if slot.is_none() {
            // SAFETY: `PL011_BASE` is the QEMU `virt` PL011 MMIO base; nothing
            // else drives it. After the MMU is enabled this address is
            // identity-mapped as Device memory, so it stays valid.
            let uart = unsafe { Pl011::new(PL011_BASE) };
            uart.init();
            *slot = Some(uart);
        }
    });
}

/// Write pre-formatted arguments to the global console.
pub fn _print(args: fmt::Arguments) {
    CONSOLE.with(|slot| {
        if let Some(uart) = slot.as_mut() {
            let _ = uart.write_fmt(args);
        }
    });
}

/// Print to the Frame console without a trailing newline.
#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::console::_print(::core::format_args!($($arg)*))
    };
}

/// Print to the Frame console with a trailing newline.
#[macro_export]
macro_rules! kprintln {
    () => { $crate::kprint!("\n") };
    ($($arg:tt)*) => {
        $crate::console::_print(::core::format_args!("{}\n", ::core::format_args!($($arg)*)))
    };
}
