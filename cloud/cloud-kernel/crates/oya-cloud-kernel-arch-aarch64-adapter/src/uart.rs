//! PL011 UART driver (the QEMU `virt` console at `0x0900_0000`).
//!
//! This is Frame code: it touches MMIO directly, so it lives behind the safe
//! [`hal::ConsoleWrite`] trait. Register access goes through `tock-registers`
//! typed MMIO wrappers rather than raw pointer arithmetic.
//!
//! QEMU's PL011 is already initialized enough to transmit on reset, but we
//! configure the baud divisor and line control anyway so the driver is correct
//! on real(er) hardware and after a warm reset.

use core::fmt;

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

/// Base MMIO address of the PL011 on the QEMU `virt` machine.
pub const PL011_BASE: usize = 0x0900_0000;

register_bitfields! [
    u32,
    /// Flag register.
    FR [
        /// Transmit FIFO full.
        TXFF OFFSET(5) NUMBITS(1) [],
        /// Receive FIFO empty.
        RXFE OFFSET(4) NUMBITS(1) [],
        /// UART busy transmitting.
        BUSY OFFSET(3) NUMBITS(1) [],
    ],
    /// Line control register.
    LCRH [
        /// Word length.
        WLEN OFFSET(5) NUMBITS(2) [
            Bits8 = 0b11,
        ],
        /// Enable FIFOs.
        FEN OFFSET(4) NUMBITS(1) [],
    ],
    /// Control register.
    CR [
        /// UART enable.
        UARTEN OFFSET(0) NUMBITS(1) [],
        /// Transmit enable.
        TXE OFFSET(8) NUMBITS(1) [],
        /// Receive enable.
        RXE OFFSET(9) NUMBITS(1) [],
    ],
];

register_structs! {
    /// PL011 register block (only the fields we use are named).
    pub Pl011Regs {
        (0x000 => dr: ReadWrite<u32>),
        (0x004 => _reserved0),
        (0x018 => fr: ReadOnly<u32, FR::Register>),
        (0x01c => _reserved1),
        (0x024 => ibrd: WriteOnly<u32>),
        (0x028 => fbrd: WriteOnly<u32>),
        (0x02c => lcrh: WriteOnly<u32, LCRH::Register>),
        (0x030 => cr: WriteOnly<u32, CR::Register>),
        (0x034 => _reserved2),
        (0x1000 => @END),
    }
}

/// A PL011 UART instance bound to a fixed MMIO base.
pub struct Pl011 {
    base: usize,
}

impl Pl011 {
    /// Bind a driver to a PL011 at `base`.
    ///
    /// # Safety
    /// `base` must be the physical (or identity-mapped) address of a real
    /// PL011 register block that nothing else is concurrently driving.
    pub const unsafe fn new(base: usize) -> Self {
        Self { base }
    }

    fn regs(&self) -> &Pl011Regs {
        // SAFETY: `base` was promised by the caller of `new` to point at a
        // valid PL011 register block; `Pl011Regs` is `repr(C)` with the PL011
        // layout, and we only ever produce shared references to volatile MMIO
        // cells (whose accessors do the volatile reads/writes).
        unsafe { &*(self.base as *const Pl011Regs) }
    }

    /// Initialize line control: 8N1, FIFOs on, TX/RX enabled.
    ///
    /// The baud divisor assumes QEMU's default 24 MHz UART clock and targets
    /// 115200 baud, but on QEMU the exact divisor is cosmetic.
    pub fn init(&self) {
        let r = self.regs();
        // Disable while reconfiguring.
        r.cr.set(0);
        // 24_000_000 / (16 * 115200) = 13.02 -> IBRD=13, FBRD=1.
        r.ibrd.set(13);
        r.fbrd.set(1);
        r.lcrh.write(LCRH::WLEN::Bits8 + LCRH::FEN::SET);
        r.cr.write(CR::UARTEN::SET + CR::TXE::SET + CR::RXE::SET);
    }

    /// Block until the TX FIFO has room, then push one byte.
    pub fn put_byte(&self, byte: u8) {
        let r = self.regs();
        while r.fr.is_set(FR::TXFF) {
            core::hint::spin_loop();
        }
        r.dr.set(byte as u32);
    }
}

impl hal::ConsoleWrite for Pl011 {
    fn write_byte(&mut self, byte: u8) {
        // Translate LF to CRLF so the serial terminal shows clean lines.
        if byte == b'\n' {
            self.put_byte(b'\r');
        }
        self.put_byte(byte);
    }
}

impl fmt::Write for Pl011 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.as_bytes() {
            <Self as hal::ConsoleWrite>::write_byte(self, *b);
        }
        Ok(())
    }
}
