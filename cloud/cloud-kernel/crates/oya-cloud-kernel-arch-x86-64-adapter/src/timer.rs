//! Programmable Interval Timer (PIT, 8254) setup.
//!
//! Programs PIT channel 0 in rate-generator mode (mode 2) for a periodic IRQ0
//! at roughly `hz` ticks per second. The IRQ is handled in
//! [`crate::interrupts`]. We keep to the legacy PIT rather than the APIC timer
//! to match the minimal, well-trodden `blog_os` path.

use x86_64::instructions::port::Port;

/// PIT input clock: 1.193182 MHz.
pub const PIT_FREQUENCY: u32 = 1_193_182;

/// PIT channel-0 data port.
const CHANNEL0_DATA: u16 = 0x40;
/// PIT mode/command port.
const MODE_COMMAND: u16 = 0x43;

/// Program PIT channel 0 for a periodic interrupt at ~`hz` Hz.
///
/// # Safety
/// Performs raw port I/O to the PIT; call once during bring-up before
/// unmasking IRQ0.
pub unsafe fn init(hz: u32) {
    let divisor = (PIT_FREQUENCY / hz).clamp(1, 0xFFFF) as u16;

    let mut command: Port<u8> = Port::new(MODE_COMMAND);
    let mut data: Port<u8> = Port::new(CHANNEL0_DATA);

    // SAFETY: standard PIT init. Command byte 0x36 = channel 0, access mode
    // lobyte/hibyte, mode 2 (rate generator), binary. Then write the 16-bit
    // reload divisor low byte then high byte.
    unsafe {
        command.write(0x36u8);
        data.write((divisor & 0xFF) as u8);
        data.write((divisor >> 8) as u8);
    }
}

/// The configured tick frequency reported through the HAL `TimerApi`.
pub const fn frequency() -> u64 {
    PIT_FREQUENCY as u64
}
