//! Clean QEMU self-termination via the `isa-debug-exit` device.
//!
//! With `-device isa-debug-exit,iobase=0xf4,iosize=0x04`, writing a value `v`
//! to port 0xf4 makes QEMU exit with status `(v << 1) | 1`. We use this to end
//! the run after the demo ticks, mirroring the aarch64 PSCI SYSTEM_OFF path.

use x86_64::instructions::port::Port;

/// I/O port the isa-debug-exit device listens on.
const ISA_DEBUG_EXIT_PORT: u16 = 0xF4;

/// Value written for a successful exit. QEMU exits with `(0x10 << 1) | 1` = 33;
/// the run script treats that as success.
const EXIT_SUCCESS_CODE: u32 = 0x10;

/// Exit QEMU cleanly, signalling success. Never returns.
pub fn exit_qemu_success() -> ! {
    // SAFETY: writing to the isa-debug-exit I/O port is the device's sole
    // contract; it terminates the VM. No memory effects.
    unsafe {
        let mut port = Port::new(ISA_DEBUG_EXIT_PORT);
        port.write(EXIT_SUCCESS_CODE);
    }
    // If the device is absent (port write was a no-op), halt forever rather
    // than fall through.
    loop {
        x86_64::instructions::hlt();
    }
}
