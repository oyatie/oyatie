//! IDT, CPU exception handlers, 8259 PIC, and the PIT timer IRQ.
//!
//! Installs an IDT (via the `x86_64` crate) with handlers for the common CPU
//! exceptions plus the timer IRQ. The double-fault handler runs on its own IST
//! stack (see [`crate::gdt`]). The legacy 8259 PIC pair is remapped to
//! vectors 0x20.. and the PIT is programmed for ~100 Hz; each tick prints
//! `timer tick <n>`, and after a few ticks the Frame prints the OK marker and
//! exits QEMU via the isa-debug-exit device.

use core::sync::atomic::{AtomicU32, Ordering};

use pic8259::ChainedPics;
use spin::{Lazy, Mutex};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

/// First vector the master PIC is remapped to (0x20). IRQ0 (PIT) -> 0x20.
pub const PIC_1_OFFSET: u8 = 0x20;
/// First vector the slave PIC is remapped to (0x28).
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Vector of the PIT timer interrupt (IRQ0).
const TIMER_VECTOR: u8 = PIC_1_OFFSET;

/// The chained 8259 PICs, remapped above the CPU exception range.
pub static PICS: Mutex<ChainedPics> =
    // SAFETY (const): the offsets are in the user-vector range and do not
    // collide with CPU exceptions (0..32). Actual port I/O happens in `init`.
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// Number of timer ticks observed so far.
pub static TICKS: AtomicU32 = AtomicU32::new(0);

/// How many ticks to observe before declaring success and exiting QEMU.
const TICKS_BEFORE_EXIT: u32 = 3;

/// When `true`, the timer ISR prints/exits as the *terminal* milestone (the
/// original timer-only demo). When `false`, the ticks are just a heartbeat
/// running *before* the ring-3 user step, so the ISR keeps ticking and does NOT
/// print `kernel: OK` / exit QEMU — `user::run_user` owns the terminal
/// OK marker in that case. Defaults to `true` so any path that only arms the
/// PIT (the preserved fallback) still reaches OK and exits.
pub static TIMER_OWNS_EXIT: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(true);

/// The IDT, built once via `Lazy` (no `static mut`).
static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.divide_error.set_handler_fn(divide_error_handler);
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    // SAFETY: `DOUBLE_FAULT_IST_INDEX` is a valid IST slot populated by
    // `gdt::init` (called before `init_idt`).
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt[TIMER_VECTOR].set_handler_fn(timer_interrupt_handler);
    // P4: the x2APIC spurious-interrupt vector (0xFF). The SVR (apic.rs) points
    // the APIC's spurious vector here; it must resolve to a real IDT gate that
    // simply returns (the APIC requires no EOI for the spurious vector) or a
    // spurious delivery would triple-fault. Harmless in the PIC/PIT (tier3) leg
    // since the APIC is never software-enabled there, so 0xFF never fires.
    idt[crate::apic::SPURIOUS_VECTOR].set_handler_fn(spurious_interrupt_handler);
    // P4·SMP·S4c: the cross-CPU TLB-shootdown IPI vector. A Fixed-delivery
    // x2APIC IPI lands here; the ISR flushes this CPU's stale TLB (CR3 reload),
    // acks the H1 sender, and EOIs the APIC.
    idt[crate::shootdown::SHOOTDOWN_VECTOR].set_handler_fn(shootdown_interrupt_handler);
    // P4·SMP·S4b: the reschedule IPI vector (0xF0). A Fixed-delivery x2APIC IPI
    // wakes an idle (`hlt`) AP so it re-runs its idle→schedule loop; the ISR does
    // nothing but EOI — taking the IRQ is the wake (no rendezvous, no ack).
    idt[crate::reschedule::RESCHED_VECTOR].set_handler_fn(reschedule_interrupt_handler);
    idt
});

/// Install the IDT. Must be called after [`crate::gdt::init`] so the IST index
/// the double-fault handler uses is valid.
///
/// # Safety
/// Call once on the boot core during bring-up, before enabling interrupts.
pub unsafe fn init_idt() {
    // `IDT` is a `'static` Lazy, so `load()` gets the required `'static`
    // reference. No `unsafe` is needed for the load itself, but the function is
    // `unsafe` to document its once/ordering contract relative to `gdt::init`.
    IDT.load();
}

/// Load the SHARED IDT on an AP (P4·SMP·S3). The IDT is a read-only `Lazy` table
/// the BSP already built; sharing it across CPUs is correct (the handlers are
/// reentrant-per-CPU as long as per-CPU state stays per-CPU). The AP must `lidt`
/// it itself so it has valid exception/IRQ vectors.
///
/// # Safety
/// Call once on an AP after its own GDT/TSS is loaded, with IRQs masked.
pub unsafe fn load_idt_ap() {
    IDT.load();
}

/// Swap the IRQ0 (PIT timer) IDT gate to point at a **raw** handler `entry`
/// (the process model's register-saving `__kuberos_timer_entry` stub) instead of
/// the boot `extern "x86-interrupt"` heartbeat handler.
///
/// The `x86_64` crate's `Entry` type does not expose a raw-address setter on a
/// shared `&IDT`, so we patch the 16-byte gate descriptor in the live IDT
/// directly: bytes [0..2] + [6..8] + [8..12] hold the 64-bit offset (low/mid/
/// high), and the type/attr/selector/IST bytes already set by the boot build are
/// preserved (present, DPL 0, 64-bit interrupt gate, kernel CS, IST 0). Because
/// only the offset changes, the new handler runs with IRQs masked on entry on
/// the kernel stack the CPU loads from TSS.RSP0 for the ring3->ring0 transition.
///
/// # Safety
/// Call once on the boot core after [`init_idt`], before entering the process
/// model and while IRQ0 is masked or not yet firing into ring 3. `entry` must be
/// a valid handler that performs its own register save / EOI / iretq.
pub unsafe fn set_preempt_handler(entry: u64) {
    // The IDT is a contiguous array of 16-byte gate descriptors at `&IDT[0]`.
    // SAFETY: `IDT` is loaded + `'static`; we patch one gate's offset fields in
    // place. Single core, IRQ0 not firing into this path yet.
    let idt_base = (&*IDT) as *const InterruptDescriptorTable as *const u8;
    let gate = unsafe { idt_base.add(TIMER_VECTOR as usize * 16) as *mut u8 };
    let off = entry;
    unsafe {
        // offset[15:0] at bytes [0..2]
        (gate as *mut u16).write_unaligned(off as u16);
        // offset[31:16] at bytes [6..8]
        (gate.add(6) as *mut u16).write_unaligned((off >> 16) as u16);
        // offset[63:32] at bytes [8..12]
        (gate.add(8) as *mut u32).write_unaligned((off >> 32) as u32);
    }
}

/// Initialize and remap the 8259 PIC pair, masking everything except IRQ0.
///
/// # Safety
/// Call once during bring-up before enabling interrupts.
pub unsafe fn init_pic() {
    let mut pics = PICS.lock();
    // SAFETY: standard PIC remap sequence; we are the only PIC driver.
    unsafe {
        pics.initialize();
        // Unmask only IRQ0 (timer) on the master; mask all else to keep the
        // demo deterministic. Master mask: bit0=IRQ0. 0xFE leaves IRQ0 enabled.
        pics.write_masks(0xFE, 0xFF);
    }
}

// ---------------------------------------------------------------------------
// Exception handlers
// ---------------------------------------------------------------------------

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    crate::kprintln!("EXCEPTION: breakpoint\n{:#?}", frame);
}

extern "x86-interrupt" fn divide_error_handler(frame: InterruptStackFrame) {
    crate::kprintln!("EXCEPTION: divide error\n{:#?}", frame);
    halt_loop();
}

extern "x86-interrupt" fn invalid_opcode_handler(frame: InterruptStackFrame) {
    crate::kprintln!("EXCEPTION: invalid opcode\n{:#?}", frame);
    halt_loop();
}

extern "x86-interrupt" fn general_protection_fault_handler(
    frame: InterruptStackFrame,
    error_code: u64,
) {
    crate::kprintln!(
        "EXCEPTION: general protection fault (code {:#x})\n{:#?}",
        error_code,
        frame
    );
    halt_loop();
}

extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    // F-0020: a CPU exception gate clears IF/TF but NOT DF. This handler is entered
    // from ring 3 on the faulting user instruction, which (for a musl backward
    // `memmove`) may have DF=1. The SysV AMD64 ABI requires DF=0 in the Rust code
    // below (and any compiler-emitted `rep movs`/`stos` in the COW page copy), so
    // clear it once on entry. Pairs with the same `cld` on the timer trampoline;
    // without it a backward string op would smear the kernel stack.
    // SAFETY: a single DF-clear; `nomem`/`nostack` hold but NOT `preserves_flags`
    // (it intentionally modifies DF), in fault context.
    unsafe { core::arch::asm!("cld", options(nomem, nostack)) };

    let addr = x86_64::registers::control::Cr2::read();

    // A ring-3 write to a *present* page is the copy-on-write signature: `fork`
    // write-protected + COW-tagged the parent's and child's writable pages, so the
    // first write to one faults here (present + write + user). If the faulting
    // page is COW-tagged, copy it and return (the `iretq` retries the store);
    // otherwise it is a genuine protection fault and stays fatal.
    let p = PageFaultErrorCode::PROTECTION_VIOLATION;
    let w = PageFaultErrorCode::CAUSED_BY_WRITE;
    let u = PageFaultErrorCode::USER_MODE;
    let i = PageFaultErrorCode::INSTRUCTION_FETCH;
    if error_code.contains(p | w | u) {
        if let Ok(cr2) = addr {
            let va = cr2.as_u64() as usize;
            // This handler is entered from ring 3 WITHOUT `swapgs` (a trap gate
            // does not swap GS), but `with_sched` mints its per-CPU `CpuToken`
            // from `gs:16`, which requires the KERNEL per-CPU base to be the
            // active GS. Bracket the scheduler call in `with_kernel_gs` so the
            // mint reads this CPU's `cpu_index` (0 on 1-vCPU), then the user GS
            // is restored before the `iretq` retries the faulting store.
            // SAFETY: from-ring3, IF-masked fault context; `try_cow_fault` edits
            // the live-CR3 thread's page table + flushes that VA's TLB.
            let resolved = unsafe {
                crate::user::with_kernel_gs(|| {
                    crate::process::with_sched(|s| s.try_cow_fault(va))
                })
            };
            if resolved {
                return;
            }
        }
    }

    // P4·SMP·S4c/S4d defense-in-depth (parity with aarch64's instruction-abort
    // retry): a ring-3 INSTRUCTION-FETCH protection fault may be a STALE-TLB
    // artifact — a sibling CPU relaxed/re-mapped this page (a COW copy or an
    // `execve` image swap) and the cross-CPU shootdown IPI to this core arrived
    // late or was missed, so this core kept the old (non-present/non-exec)
    // translation cached. If the PT leaf NOW maps the page present+US+executable,
    // it is stale: `invlpg` the VA and retry the fetch. Without this, the rare race
    // surfaced as the `0xbff…`/`0x3` instruction-fetch faults in the -smp talos
    // stress; the genuine-fault case still falls through to the fatal report.
    if error_code.contains(p | i | u) {
        if let Ok(cr2) = addr {
            let va = cr2.as_u64() as usize;
            // SAFETY: from-ring3, IF-masked fault context; only INSPECTS the PT and,
            // on a stale hit, invalidates one VA (see `with_kernel_gs` note above).
            let retried = unsafe {
                crate::user::with_kernel_gs(|| {
                    crate::process::with_sched(|s| s.try_retry_stale_insn_fetch(va))
                })
            };
            if retried {
                return;
            }
        }
    }

    crate::kprintln!(
        "EXCEPTION: page fault at {:?} (code {:?})\n{:#?}",
        addr,
        error_code,
        frame
    );
    halt_loop();
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, error_code: u64) -> ! {
    crate::kprintln!(
        "EXCEPTION: DOUBLE FAULT (code {:#x})\n{:#?}",
        error_code,
        frame
    );
    halt_loop();
}

// ---------------------------------------------------------------------------
// Timer IRQ
// ---------------------------------------------------------------------------

extern "x86-interrupt" fn timer_interrupt_handler(_frame: InterruptStackFrame) {
    let n = TICKS.fetch_add(1, Ordering::Relaxed) + 1;
    crate::kprintln!("timer tick {}", n);

    // Signal end-of-interrupt to the PIC so further timer IRQs are delivered.
    // SAFETY: we are in the IRQ0 handler; notifying EOI for that exact vector
    // is the required PIC protocol.
    unsafe {
        PICS.lock().notify_end_of_interrupt(TIMER_VECTOR);
    }

    if n >= TICKS_BEFORE_EXIT && TIMER_OWNS_EXIT.load(Ordering::Relaxed) {
        crate::kprintln!("kernel: OK");
        crate::exit::exit_qemu_success();
    }
}

/// The x2APIC spurious-interrupt handler (vector 0xFF). A spurious interrupt is
/// raised by the local APIC when an interrupt is de-asserted before it is
/// dispatched; the handler must simply return WITHOUT an EOI (the APIC does not
/// set the in-service bit for a spurious vector). No-op by design.
extern "x86-interrupt" fn spurious_interrupt_handler(_frame: InterruptStackFrame) {}

/// The P4·SMP·S4c cross-CPU TLB-shootdown IPI handler. A Fixed-delivery x2APIC
/// IPI from a sender lands here; the receiver flushes this CPU's stale TLB and
/// acks the H1 sender, then EOIs the APIC (a Fixed vector sets the in-service
/// bit, so an EOI is REQUIRED — unlike the spurious vector).
///
/// Entered via an interrupt gate that did NOT `swapgs` (the CPU only swaps GS for
/// SYSCALL, not `int`/IPI gates). The receiver needs this CPU's logical index
/// (read from `gs:16`), so it brackets the work in `with_kernel_gs` when it came
/// from ring 3; from ring 0 (an idle AP between schedules) GS is already the
/// kernel base. We branch on the saved CS RPL exactly like `__kuberos_timer_entry`.
extern "x86-interrupt" fn shootdown_interrupt_handler(frame: InterruptStackFrame) {
    // Was the interrupt taken from ring 3 (user) or ring 0 (kernel/idle AP)?
    let from_user = (frame.code_segment.rpl() as u16 & 3) != 0;
    if from_user {
        // SAFETY: from ring 3, IF=0 ISR context; bracket the per-CPU read in a
        // balanced swapgs pair so `gs:16` is this CPU's kernel base.
        unsafe {
            crate::user::with_kernel_gs(crate::shootdown::service_on_ipi);
        }
    } else {
        // Ring 0: GS is already the kernel per-CPU base (idle AP, or a sender
        // spinning at ring 0 servicing an inbound shootdown).
        crate::shootdown::service_on_ipi();
    }
    // EOI the local APIC: a Fixed-delivery vector set the in-service bit.
    // SAFETY: ISR context for a real (non-spurious) x2APIC vector.
    unsafe { crate::apic::eoi() };
}

/// The P4·SMP·S4b reschedule IPI handler (vector 0xF0). A Fixed-delivery x2APIC
/// IPI from a CPU that placed work on THIS CPU's run queue lands here. The handler
/// does NO per-CPU work: the mere act of taking the interrupt pulls an idle AP out
/// of `hlt` so it falls back into `ap_run_scheduler` and re-pops/steals. It only
/// EOIs the APIC (a Fixed vector sets the in-service bit, so an EOI is REQUIRED).
/// Unlike the shootdown ISR it needs no `gs:16` read, so no `swapgs` bracketing.
extern "x86-interrupt" fn reschedule_interrupt_handler(_frame: InterruptStackFrame) {
    // EOI the local APIC: a Fixed-delivery vector set the in-service bit. Waking
    // from `hlt` + the schedule re-check is the entire effect.
    // SAFETY: ISR context for a real (non-spurious) x2APIC vector.
    unsafe { crate::apic::eoi() };
}

/// Halt the core forever (used by fatal exception handlers).
fn halt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
