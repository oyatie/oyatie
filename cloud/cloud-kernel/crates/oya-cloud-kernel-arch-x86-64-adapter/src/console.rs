//! Global serial console for the x86_64 Frame (16550 UART at port 0x3F8).
//!
//! Wraps the `uart_16550` `SerialPort` behind a spinlock and exposes
//! `kprint!`/`kprintln!` plus a [`ConsoleWrite`] handle the safe kernel drives
//! through the HAL. Single-core during bring-up, so the lock mainly makes the
//! global mutable access sound.

use core::fmt::{self, Write};

use spin::Mutex;
use uart_16550::SerialPort;

use hal::ConsoleWrite;

/// COM1 base I/O port on the PC platform.
pub const COM1_BASE: u16 = 0x3F8;

/// The global serial console, initialized in [`init`].
static CONSOLE: Mutex<Option<SerialPort>> = Mutex::new(None);

/// Bring up COM1. Idempotent.
pub fn init() {
    let mut guard = CONSOLE.lock();
    if guard.is_none() {
        // SAFETY: `COM1_BASE` is the standard COM1 I/O port; nothing else in
        // the Frame drives it. `SerialPort::new` only records the base; `init`
        // programs the divisor/FIFO. Port I/O to 0x3F8 is side-effect-free
        // w.r.t. memory.
        let mut port = unsafe { SerialPort::new(COM1_BASE) };
        port.init();
        *guard = Some(port);
    }
}

/// Write pre-formatted arguments to the global console.
pub fn _print(args: fmt::Arguments) {
    let mut guard = CONSOLE.lock();
    if let Some(port) = guard.as_mut() {
        let _ = port.write_fmt(args);
    }
}

/// A thin [`ConsoleWrite`] handle over COM1 for the safe kernel's banner.
///
/// Writing goes through a freshly-constructed `SerialPort` view of the same
/// port; on a single core the kernel banner and the IRQ path do not run
/// concurrently while it is held.
pub struct Com1;

impl ConsoleWrite for Com1 {
    fn write_byte(&mut self, byte: u8) {
        _print(format_args!("{}", byte as char));
    }

    fn write_str(&mut self, s: &str) {
        _print(format_args!("{}", s));
    }
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
