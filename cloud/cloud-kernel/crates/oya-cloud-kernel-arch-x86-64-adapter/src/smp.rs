//! x86_64 SMP / AP bring-up (P4·SMP·S3). Frame code (TCB).
//!
//! Brings secondary CPUs (APs) online under `qemu-system-x86_64 -smp N` and
//! leaves them idling at `hlt`. The BSP entered via PVH in 32-bit protected
//! mode, so its `_start` cannot be reused for an AP — a SIPI delivers an AP in
//! 16-bit real mode at `CS:IP = (vector<<8):0`. So this module provides a
//! NET-NEW `.code16 → .code32 → .code64` trampoline copied to a low-mem page
//! (`AP_TRAMPOLINE_PA = 0x8000`, SIPI vector `0x08`), driven by INIT-SIPI-SIPI
//! through the x2APIC ICR MSR (`apic::send_init_sipi`).
//!
//! End to end:
//!   1. [`enumerate`] counts CPUs from the ACPI MADT (Local-APIC entries),
//!      falling back to the documented contiguous-APIC-ID floor.
//!   2. [`X86_64::start_secondaries`] copies the trampoline blob to `0x8000`,
//!      then per AP fills a per-AP launch block (`{cpu_index, stack_top}`),
//!      issues INIT-SIPI-SIPI, and bounded-spins on [`ONLINE_MASK`] (printing
//!      `cpu k online` as it observes each bit — a single console writer).
//!   3. The trampoline switches real→protected→long, loads the SHARED
//!      `boot_pml4`/IDT, reads its launch block, sets `rsp`, and calls
//!      [`ap_rust_entry`], which sets GS-base + its own TSS + per-AP x2APIC,
//!      `Release`-publishes its online bit, and `hlt`-idles.
//!
//! The online-mask handshake is the H1 publish/observe discipline (AP
//! `fetch_or(Release)` after full init, BSP `load(Acquire)`).

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use hal::cpu::MAX_CPUS;

// ---------------------------------------------------------------------------
// Online mask (startup sync; bit 0 = BSP, pre-set)
// ---------------------------------------------------------------------------

/// CPUs that have published their online bit. Bit 0 (BSP) pre-set; each AP
/// `fetch_or(Release)`-s its bit after full init, the BSP `load(Acquire)`-s.
static ONLINE_MASK: AtomicU64 = AtomicU64::new(1);

/// `Acquire` read of the online mask.
#[inline]
pub fn online_mask() -> u64 {
    ONLINE_MASK.load(Ordering::Acquire)
}

/// Per-logical-CPU x2APIC id (P4·SMP·S4c): `APIC_IDS[k]` is the APIC id of
/// logical CPU `k`, recorded once during `start_secondaries` so the shootdown
/// sender can target a CPU by its logical index over the ICR. Slot 0 (BSP) is
/// the BSP's own id. `AtomicU32` per slot; written once at boot (single writer
/// per slot, the BSP), read `Relaxed` thereafter.
static APIC_IDS: [AtomicU32; MAX_CPUS] = [const { AtomicU32::new(0) }; MAX_CPUS];

/// Record logical CPU `cpu`'s x2APIC id (BSP, once at boot).
fn set_apic_id(cpu: usize, apic_id: u32) {
    APIC_IDS[cpu].store(apic_id, Ordering::Relaxed);
}

/// The x2APIC id of logical CPU `cpu` (for the shootdown ICR target).
#[inline]
pub fn apic_id_of(cpu: usize) -> u32 {
    APIC_IDS[cpu].load(Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// Per-AP launch block + stacks
// ---------------------------------------------------------------------------

/// Fixed low-mem physical address the trampoline blob is copied to. Below 1 MiB
/// (the SIPI ceiling), above the real-mode IVT/BDA (`< 0x500`) and the legacy
/// `0x7c00` boot area, clear of the 2 MiB+ image. SIPI start vector = `0x08`.
/// The low 1 GiB is identity-mapped (`boot.rs`), so this VA == PA is writable.
const AP_TRAMPOLINE_PA: usize = 0x8000;

/// SIPI start vector = trampoline page number (`0x8000 >> 12`).
const AP_SIPI_VECTOR: u8 = (AP_TRAMPOLINE_PA >> 12) as u8;

/// Fixed low-mem address of the per-AP launch block array the 64-bit trampoline
/// tail reads to learn its `cpu_index` + `stack_top`. Placed just past the
/// trampoline page, still in identity-mapped low memory.
const AP_LAUNCH_PA: usize = 0x9000;

/// One launch record per logical CPU, at `AP_LAUNCH_PA`. The 64-bit trampoline
/// indexes it by the value the BSP currently advertises in `AP_LAUNCH_INDEX`
/// (APs start serially, one at a time).
#[repr(C)]
#[derive(Clone, Copy)]
struct ApLaunch {
    cpu_index: u64,
    stack_top: u64,
}

/// The index of the AP currently being started, at `AP_LAUNCH_PA - 8`. The
/// trampoline reads this single word to find its own [`ApLaunch`] record. Safe
/// because APs are brought up serially (the BSP waits for each online bit before
/// starting the next), so exactly one AP reads it per SIPI round.
const AP_LAUNCH_INDEX_PA: usize = AP_LAUNCH_PA - 8;

/// Per-AP kernel stack size (64 KiB, matching the BSP boot stack).
const AP_STACK_SIZE: usize = 64 * 1024;

/// One 16-byte-aligned stack per logical CPU (slot 0 BSP unused). Lives in the
/// dedicated `.ap_stacks` NOLOAD section (placed AFTER the heap by `linker.ld`),
/// NOT in `.bss` — so its 8×64 KiB does not grow `.bss` and shift the boot stack
/// / page tables, keeping the default 1-vCPU layout byte-identical.
#[repr(C, align(16))]
struct ApStacks([[u8; AP_STACK_SIZE]; MAX_CPUS]);

#[link_section = ".ap_stacks"]
static mut AP_STACKS: ApStacks = ApStacks([[0; AP_STACK_SIZE]; MAX_CPUS]);

/// Top (one past the end) of AP `idx`'s stack.
fn ap_stack_top(idx: usize) -> u64 {
    let base = core::ptr::addr_of!(AP_STACKS) as usize;
    (base + (idx + 1) * AP_STACK_SIZE) as u64
}

// ---------------------------------------------------------------------------
// The .code16 → .code32 → .code64 AP trampoline
// ---------------------------------------------------------------------------
//
// Assembled at the image's high `.text.ap_trampoline` VA but COPIED to
// `AP_TRAMPOLINE_PA = 0x8000` before use. Every in-page reference therefore uses
// an address computed as `0x8000 + (label - ap_trampoline_start)`, so the blob
// is position-correct at its copy destination without runtime patching. CR3
// (`boot_pml4`), the IDT, and `_ap_long_mode` (high `.text`, reachable once
// paging is on) are all shared and referenced by their real linked addresses.

core::arch::global_asm!(
    r#"
    .section .text.ap_trampoline, "ax"
    .code16
    .balign 16
    .global ap_trampoline_start
ap_trampoline_start:
    cli
    cld
    // Real mode entered via SIPI: CS = vector<<8 = 0x0800, IP = 0, so CS:0 ==
    // 0x8000 (our copy destination). Mirror CS into DS so in-page data refs (the
    // GDT pointer) resolve relative to the 0x8000 base. All labels below are
    // expressed as in-page offsets (label - ap_trampoline_start), which the
    // assembler emits as plain 16-bit displacements (no absolute symbol in a
    // memory operand — that the assembler rejects).
    mov     ax, cs
    mov     ds, ax
    // Load the page-local GDT via a DS-relative offset to its pointer. The
    // offset is pre-folded to an absolute constant via `.set` so the memory
    // operand carries a single (absolute) symbol — the assembler rejects a raw
    // two-symbol difference in a memory operand.
    .set    ap_gdt_ptr_off, ap_gdt_ptr - ap_trampoline_start
    lgdt    [ap_gdt_ptr_off]
    // Set CR0.PE (protected mode).
    mov     eax, cr0
    or      eax, 1
    mov     cr0, eax
    // Far-jump to the 32-bit stub at its absolute copy address (0x8000+offset is
    // a constant immediate operand, not a memory operand — allowed). Selector
    // 0x08 = 32-bit code in the page-local GDT.
    .byte   0x66, 0xea            // ljmpl ptr16:32
    .long   (0x8000 + (ap_pm32 - ap_trampoline_start))
    .word   0x08

    .code32
    .balign 4
ap_pm32:
    // Reload data segments with the 32-bit data selector (0x10).
    mov     ax, 0x10
    mov     ds, ax
    mov     es, ax
    mov     ss, ax
    mov     fs, ax
    mov     gs, ax
    // CR4.PAE.
    mov     eax, cr4
    or      eax, (1 << 5)
    mov     cr4, eax
    // CR3 = shared boot_pml4.
    mov     eax, offset boot_pml4
    mov     cr3, eax
    // EFER.LME (MSR 0xC0000080, bit 8).
    mov     ecx, 0xC0000080
    rdmsr
    or      eax, (1 << 8)
    wrmsr
    // CR0.PG | CR0.PE.
    mov     eax, cr0
    or      eax, (1 << 31) | (1 << 0)
    mov     cr0, eax
    // Far-jump to 64-bit code. The 64-bit code segment is 0x18 in the
    // page-local GDT; `_ap_long_mode` lives at its real high .text address.
    .byte   0xea                 // ljmp ptr16:32 (still .code32)
    .long   _ap_long_mode
    .word   0x18

    // ---- page-local GDT (real-mode-reachable copy of the boot gdt32) ----
    .balign 16
ap_gdt:
    .quad   0x0000000000000000   // 0x00 null
    .quad   0x00CF9A000000FFFF   // 0x08 32-bit code
    .quad   0x00CF92000000FFFF   // 0x10 data
    .quad   0x00AF9A000000FFFF   // 0x18 64-bit code
ap_gdt_end:
    .balign 4
ap_gdt_ptr:
    .word   ap_gdt_end - ap_gdt - 1
    .long   (0x8000 + (ap_gdt - ap_trampoline_start))

    .global ap_trampoline_end
ap_trampoline_end:

    // ---- 64-bit AP entry (in high .text; mapped once paging is on) ----
    .code64
    .global _ap_long_mode
    .type _ap_long_mode, @function
_ap_long_mode:
    // Reload data segments with the long-mode data selector (0x10, page GDT).
    mov     ax, 0x10
    mov     ds, ax
    mov     es, ax
    mov     ss, ax
    mov     fs, ax
    mov     gs, ax
    // Read this AP's launch index, then its launch record (cpu_index+stack_top).
    mov     rax, [0x9000 - 8]            // AP_LAUNCH_INDEX
    mov     rcx, 16                      // sizeof(ApLaunch)
    mul     rcx                          // rax = idx * 16
    lea     rbx, [0x9000]               // AP_LAUNCH_PA
    add     rbx, rax
    mov     rdi, [rbx]                   // cpu_index (1st arg)
    mov     rsp, [rbx + 8]               // stack_top
    // Align the stack and call the Rust AP entry (diverges).
    and     rsp, -16
    call    {ap_rust_entry}
1:  hlt
    jmp     1b
    .size _ap_long_mode, . - _ap_long_mode
"#,
    ap_rust_entry = sym ap_rust_entry,
);

extern "C" {
    static ap_trampoline_start: u8;
    static ap_trampoline_end: u8;
}

// ---------------------------------------------------------------------------
// AP Rust entry (called by _ap_long_mode with rdi = cpu_index)
// ---------------------------------------------------------------------------

/// First Rust code on an AP. The trampoline established long mode on the shared
/// CR3, set this AP's stack, and passed its `cpu_index`. Here we install this
/// AP's per-CPU anchor + its OWN GDT/TSS + the shared IDT + its local x2APIC,
/// publish the online bit, then `hlt`-idle forever. NEVER returns.
///
/// # Safety
/// Entered once per AP from `_ap_long_mode`, long mode, IRQs masked, on this
/// AP's own stack, `idx` unique and `< MAX_CPUS`.
extern "C" fn ap_rust_entry(idx: u64) -> ! {
    let idx = idx as usize;

    // 1. GS-base anchor: stamp PERCPU[idx] + install into both GS bases.
    // SAFETY: AP, before any per-CPU access; writes this AP's disjoint slot.
    unsafe {
        crate::user::install_ap_percpu(idx);
    }

    // 2. This AP's OWN GDT + TSS (private RSP0/IST) + ring-0 CS/SS.
    // SAFETY: AP in long mode; loads its own GDT/TSS.
    unsafe {
        crate::gdt::init_ap(idx);
    }

    // 3. Shared IDT (read-only Lazy table; reentrant-per-CPU handlers).
    // SAFETY: the IDT was built by the BSP; loading it on this AP is correct.
    unsafe {
        crate::interrupts::load_idt_ap();
    }

    // 4. This AP's local APIC: software-enable x2APIC (LVT timer left masked —
    //    no scheduling in S3). The BSP gated AP bring-up on x2APIC presence.
    // SAFETY: AP, IRQs masked; touches only this CPU's APIC MSRs.
    unsafe {
        crate::apic::enable_x2apic_ap();
    }

    // 5. Publish the online bit (Release).
    ONLINE_MASK.fetch_or(1u64 << idx, Ordering::Release);

    // 6. P4·SMP·S4a: enter the AP scheduler. `ap_bootstrap_and_run` waits for the
    //    process model to go live, completes this AP's per-CPU scheduler state
    //    (SSE + SYSCALL MSRs + kernel stack + its periodic timer), then runs the
    //    idle→schedule loop — picking processes off this CPU's run queue (or the
    //    global Runnable set), dropping to ring 3 to run them, and `hlt`-idling
    //    on its periodic tick when it has no work. NEVER returns.
    // SAFETY: this AP's anchor/GDT/IDT/x2APIC are up and its online bit is
    //    published; `idx` is this CPU's unique logical index.
    unsafe { crate::user::ap_bootstrap_and_run(idx) }
}

// ---------------------------------------------------------------------------
// ACPI MADT enumeration (count enabled Local-APIC entries)
// ---------------------------------------------------------------------------

/// Enumerated CPU count + their APIC IDs.
#[derive(Clone, Copy)]
struct CpuTopology {
    count: usize,
    apic_id: [u32; MAX_CPUS],
}

/// Scan the BIOS/EBDA area for the RSDP, follow it to the MADT, and count
/// enabled processor entries (type 0 Local APIC + type 9 x2APIC), collecting
/// their APIC IDs. Falls back to the contiguous-ID floor (count derived from the
/// table; IDs assumed `0..N`). Returns `None` if no MADT is found.
///
/// # Safety
/// Read-only walks of firmware ACPI tables in the identity-mapped low 1 GiB.
unsafe fn walk_madt() -> Option<CpuTopology> {
    // SAFETY (delegated): bounded read-only scans of identity-mapped firmware
    // memory; every dereference is inside the low-1-GiB identity map.
    unsafe {
        let rsdp = find_rsdp()?;
        // RSDP: bytes[16..20] = RsdtAddress (ACPI 1.0); for our purposes the
        // 32-bit RSDT is sufficient under QEMU.
        let rev = *rsdp.add(15);
        let madt = if rev >= 2 {
            // XSDT (64-bit pointers) at offset 24.
            let xsdt = read_u64(rsdp.add(24)) as usize as *const u8;
            find_table(xsdt, true, *b"APIC")?
        } else {
            let rsdt = read_u32(rsdp.add(16)) as usize as *const u8;
            find_table(rsdt, false, *b"APIC")?
        };
        Some(parse_madt(madt))
    }
}

/// Find the RSDP by scanning the BIOS area `0xE0000..0x100000` for "RSD PTR ".
///
/// # Safety
/// Read-only scan of identity-mapped firmware memory.
unsafe fn find_rsdp() -> Option<*const u8> {
    const SIG: [u8; 8] = *b"RSD PTR ";
    let mut addr = 0xE_0000usize;
    // SAFETY (delegated): each candidate is in the identity-mapped low 1 GiB.
    unsafe {
        while addr < 0x10_0000 {
            let p = addr as *const u8;
            let mut ok = true;
            for (i, &c) in SIG.iter().enumerate() {
                if *p.add(i) != c {
                    ok = false;
                    break;
                }
            }
            if ok {
                return Some(p);
            }
            addr += 16; // RSDP is 16-byte aligned.
        }
    }
    None
}

/// Find an ACPI table by signature in the RSDT (32-bit entries) or XSDT (64-bit).
///
/// # Safety
/// `sdt` is a valid RSDT/XSDT pointer in identity-mapped memory.
unsafe fn find_table(sdt: *const u8, xsdt: bool, sig: [u8; 4]) -> Option<*const u8> {
    // SAFETY (delegated): the SDT header + entry array are in identity-mapped
    // firmware memory.
    unsafe {
        let len = read_u32(sdt.add(4)) as usize;
        let entry_size = if xsdt { 8 } else { 4 };
        let entries = (len.saturating_sub(36)) / entry_size;
        let mut i = 0;
        while i < entries {
            let ep = sdt.add(36 + i * entry_size);
            let table = if xsdt {
                read_u64(ep) as usize
            } else {
                read_u32(ep) as usize
            } as *const u8;
            if read_sig(table) == sig {
                return Some(table);
            }
            i += 1;
        }
    }
    None
}

/// Parse the MADT: walk its variable-length entries, counting enabled Local-APIC
/// (type 0) and x2APIC (type 9) processors and collecting their APIC IDs.
///
/// # Safety
/// `madt` is a valid MADT pointer in identity-mapped memory.
unsafe fn parse_madt(madt: *const u8) -> CpuTopology {
    let mut topo = CpuTopology {
        count: 0,
        apic_id: [0; MAX_CPUS],
    };
    // SAFETY (delegated): the MADT header + entries are in identity-mapped
    // firmware memory; the walk is bounded by the table length.
    unsafe {
        let len = read_u32(madt.add(4)) as usize;
        let mut off = 44; // MADT header is 44 bytes (then variable entries).
        while off + 2 <= len {
            let etype = *madt.add(off);
            let elen = *madt.add(off + 1) as usize;
            if elen == 0 {
                break;
            }
            match etype {
                0 => {
                    // Local APIC: [2]=ACPI id, [3]=APIC id, [4..8]=flags.
                    let apic_id = *madt.add(off + 3) as u32;
                    let flags = read_u32(madt.add(off + 4));
                    if (flags & 1) != 0 && topo.count < MAX_CPUS {
                        topo.apic_id[topo.count] = apic_id;
                        topo.count += 1;
                    }
                }
                9 => {
                    // Local x2APIC: [4..8]=x2APIC id, [8..12]=flags.
                    let apic_id = read_u32(madt.add(off + 4));
                    let flags = read_u32(madt.add(off + 8));
                    if (flags & 1) != 0 && topo.count < MAX_CPUS {
                        topo.apic_id[topo.count] = apic_id;
                        topo.count += 1;
                    }
                }
                _ => {}
            }
            off += elen;
        }
    }
    if topo.count == 0 {
        topo.count = 1; // at least the BSP.
    }
    topo
}

// --- little-endian unaligned reads of identity-mapped firmware memory --------

/// # Safety
/// `p` points at 4 readable bytes.
unsafe fn read_u32(p: *const u8) -> u32 {
    // SAFETY (delegated): 4 readable bytes; assemble LE without alignment.
    unsafe {
        (*p as u32)
            | ((*p.add(1) as u32) << 8)
            | ((*p.add(2) as u32) << 16)
            | ((*p.add(3) as u32) << 24)
    }
}

/// # Safety
/// `p` points at 8 readable bytes.
unsafe fn read_u64(p: *const u8) -> u64 {
    // SAFETY (delegated): 8 readable bytes.
    unsafe { read_u32(p) as u64 | ((read_u32(p.add(4)) as u64) << 32) }
}

/// # Safety
/// `p` points at 4 readable bytes (an ACPI table signature).
unsafe fn read_sig(p: *const u8) -> [u8; 4] {
    // SAFETY (delegated): 4 readable signature bytes.
    unsafe { [*p, *p.add(1), *p.add(2), *p.add(3)] }
}

/// Enumerate CPUs from the MADT, clamped to `MAX_CPUS`. Falls back to a single
/// CPU (BSP only) if no MADT is found.
///
/// **x2APIC gate (load-bearing):** AP bring-up rides the x2APIC ICR MSR, which
/// only exists when the part supports x2APIC. The default golden/talos x86 runs
/// use `-cpu qemu64` (NO x2apic) and must be byte-identical, so when x2APIC is
/// absent we return a single CPU IMMEDIATELY without walking ACPI — the MADT
/// walk never runs on the default path, so it cannot perturb the 1-vCPU boot.
/// The `-smp` test leg passes `+x2apic`, which enables both the walk and the
/// ICR-driven bring-up.
fn enumerate() -> CpuTopology {
    if !crate::hal_caps::probe_cpu_caps().x2apic {
        return CpuTopology {
            count: 1,
            apic_id: [0; MAX_CPUS],
        };
    }
    // SAFETY: read-only ACPI table walks in the identity-mapped low 1 GiB.
    if let Some(mut topo) = unsafe { walk_madt() } {
        topo.count = topo.count.clamp(1, MAX_CPUS);
        return topo;
    }
    CpuTopology {
        count: 1,
        apic_id: [0; MAX_CPUS],
    }
}

/// The enumerated CPU count (≥ 1).
pub fn cpu_count() -> u32 {
    enumerate().count as u32
}

// ---------------------------------------------------------------------------
// The hal::smp::Smp seam (implemented on X86_64; all unsafe is here in Frame)
// ---------------------------------------------------------------------------

/// Bounded spin cap waiting for an AP's online bit. Generous for TCG.
const AP_ONLINE_GUARD: u64 = 200_000_000;

/// Copy the trampoline blob to `AP_TRAMPOLINE_PA`.
///
/// # Safety
/// The destination low-mem page is identity-mapped + writable (`boot.rs`); the
/// source is the linked trampoline section.
unsafe fn install_trampoline() {
    // SAFETY: copy `len` bytes from the linked trampoline to the identity-mapped
    // low-mem destination page. Both are valid byte ranges.
    unsafe {
        let src = core::ptr::addr_of!(ap_trampoline_start);
        let end = core::ptr::addr_of!(ap_trampoline_end);
        let len = end as usize - src as usize;
        core::ptr::copy_nonoverlapping(src, AP_TRAMPOLINE_PA as *mut u8, len);
    }
}

/// Fill AP `idx`'s launch block + advertise its index for the trampoline tail.
///
/// # Safety
/// Writes the identity-mapped low-mem launch area. Called on the BSP before each
/// AP's SIPI, while no AP is mid-read (APs start serially).
unsafe fn set_launch(idx: usize) {
    // SAFETY: writes to identity-mapped low memory just past the trampoline page.
    unsafe {
        let rec = ApLaunch {
            cpu_index: idx as u64,
            stack_top: ap_stack_top(idx),
        };
        let arr = AP_LAUNCH_PA as *mut ApLaunch;
        arr.add(idx).write(rec);
        (AP_LAUNCH_INDEX_PA as *mut u64).write(idx as u64);
    }
}

impl hal::sealed::Sealed for crate::X86_64 {}

impl hal::smp::Smp for crate::X86_64 {
    fn cpu_count(&self) -> u32 {
        cpu_count()
    }

    fn start_secondaries(&mut self, _entry: hal::smp::ApEntry) -> Result<u32, hal::ArchError> {
        let topo = enumerate();
        if topo.count <= 1 {
            return Ok(1); // 1-vCPU: no AP code, no output.
        }

        // INIT-SIPI-SIPI is issued through the x2APIC ICR MSR (0x830), which GP-
        // faults unless the part actually supports x2APIC. Guard on it: without
        // x2APIC we cannot bring APs online via this path, so report only the
        // BSP rather than faulting (the `-smp` test legs pass `+x2apic`).
        if !crate::hal_caps::probe_cpu_caps().x2apic {
            crate::kprintln!("smp: x2APIC unavailable; APs not started");
            return Ok(1);
        }

        // x2APIC must be enabled on the BSP to issue ICR IPIs. The BSP enables it
        // in the tier-2/1 timer path; ensure it here so SMP works regardless.
        // SAFETY: copy the trampoline + (idempotently) enable the BSP x2APIC.
        unsafe {
            install_trampoline();
            crate::apic::enable_x2apic_ap();
        }

        // P4·SMP·S4c: record every logical CPU's APIC id so the shootdown sender
        // can target a CPU by index over the ICR. The BSP is logical 0.
        for k in 0..topo.count {
            set_apic_id(k, topo.apic_id[k]);
        }

        for k in 1..topo.count {
            // SAFETY: fill this AP's launch block, then issue INIT-SIPI-SIPI to
            // its APIC id, starting it at the trampoline page.
            unsafe {
                set_launch(k);
                crate::apic::send_init_sipi(topo.apic_id[k], AP_SIPI_VECTOR);
            }
            // Bounded-spin until the AP publishes its online bit.
            let mut guard = AP_ONLINE_GUARD;
            while (online_mask() & (1u64 << k)) == 0 && guard > 0 {
                guard -= 1;
                core::hint::spin_loop();
            }
            if (online_mask() & (1u64 << k)) != 0 {
                crate::kprintln!("cpu {} online", k);
            } else {
                crate::kprintln!("smp: AP {} failed to come online", k);
            }
        }

        Ok(online_mask().count_ones())
    }

    fn online_mask(&self) -> u64 {
        online_mask()
    }
}
