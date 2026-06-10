//! GDT + TSS with an IST stack for the double-fault handler, plus the ring-3
//! user segments + RSP0 kernel stack the (not-yet-active) user-mode slices need.
//!
//! The boot trampoline installs a minimal flat GDT to reach long mode. Once in
//! Rust we install a proper GDT that also carries a TSS, so the double-fault
//! handler can run on a known-good stack (IST entry 0) even if the kernel stack
//! is corrupted or overflowed. Without this, a fault during a fault triple-
//! faults and resets the machine.
//!
//! The GDT/TSS are built once via `spin::Lazy` (no `static mut`), keeping the
//! Frame free of the `static_mut_refs` footgun.
//!
//! # GDT layout (SYSCALL/SYSRET-ready ordering)
//!
//! This slice adds the ring-3 user code/data descriptors. Nothing executes in
//! ring 3 yet — these descriptors and the [`user_selectors`] / RSP0 plumbing
//! are the scaffolding the later `syscall`/`sysret` + `iretq`-to-ring-3 slices
//! build on. The ordering is chosen to satisfy *both* the SYSCALL entry and the
//! SYSRET exit selector conventions, so those slices only have to load `STAR`:
//!
//! | index | offset | descriptor                | role                         |
//! |-------|--------|---------------------------|------------------------------|
//! | 0     | `0x00` | null                      | required null descriptor     |
//! | 1     | `0x08` | kernel code (64-bit, DPL0)| `CS` after `SYSCALL`         |
//! | 2     | `0x10` | kernel data (DPL0)        | `SS` after `SYSCALL`         |
//! | 3     | `0x18` | user data (DPL3)          | `SS` after `SYSRET` (RPL 3) |
//! | 4     | `0x20` | user code (64-bit, DPL3)  | `CS` after `SYSRET` (RPL 3) |
//! | 5..6  | `0x28` | TSS (system, 2 entries)   | RSP0 + IST double-fault stack|
//!
//! **SYSCALL** loads `CS = STAR[47:32]` and `SS = STAR[47:32] + 8`, so the
//! kernel pair must be contiguous with code first: kernel code `0x08`, kernel
//! data `0x10` — unchanged from before, with `STAR[47:32] = 0x08`.
//!
//! **SYSRET** (to 64-bit mode) loads `CS = STAR[63:48] + 16` and
//! `SS = STAR[63:48] + 8` (both with RPL forced to 3), so the user pair must be
//! contiguous with **data first, code second**: user data `0x18`, user code
//! `0x20`, giving the SYSRET base `STAR[63:48] = 0x10` (so `+8 → 0x18` user
//! data, `+16 → 0x20` user code). See [`syscall_star_selectors`].
//!
//! Adding the two user descriptors before the TSS only shifts the TSS selector
//! value (now `0x28`); every consumer reads it from the recorded [`Selectors`]
//! (or [`DOUBLE_FAULT_IST_INDEX`]), never a hardcoded literal, so the kernel
//! code/data/TSS *roles* are unchanged. (The boot trampoline's temporary
//! `gdt32` is a separate table used only to reach long mode and is untouched.)

use spin::Lazy;
use x86_64::instructions::segmentation::{Segment, CS};
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::SS;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

/// IST index used for the double-fault handler stack.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// Size of the dedicated double-fault stack (20 KiB).
const IST_STACK_SIZE: usize = 4096 * 5;

/// Size of the dedicated RSP0 kernel stack the CPU switches to on a ring3→ring0
/// interrupt/exception (20 KiB, matching the IST stack).
const RSP0_STACK_SIZE: usize = 4096 * 5;

/// Selectors for the GDT entries we install.
///
/// `code`/`data` are the ring-0 kernel selectors loaded into `CS`/`SS`; `tss`
/// loads the task register. `user_code`/`user_data` are the ring-3 (RPL 3)
/// selectors the later user-mode slices load for `iretq`/`sysret`; nothing
/// loads them in this slice.
struct Selectors {
    code: SegmentSelector,
    data: SegmentSelector,
    // Read only by the (intentionally not-yet-called) user-mode seams below;
    // nothing loads the user selectors in this scaffolding slice.
    #[allow(dead_code)]
    user_data: SegmentSelector,
    #[allow(dead_code)]
    user_code: SegmentSelector,
    tss: SegmentSelector,
}

/// The TSS, carrying the RSP0 kernel stack (ring3→ring0 switches) and the IST
/// stack for double faults. Built once.
static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    // Dedicated double-fault stack. A `static mut` array would re-trip the
    // `static_mut_refs` lint, so we lazily box-allocate-free it as a leaked
    // static via a const-sized array on the heap is unavailable pre-heap; use
    // a plain function-local static array address instead.
    static IST_STACK: Lazy<[u8; IST_STACK_SIZE]> = Lazy::new(|| [0; IST_STACK_SIZE]);
    // Dedicated RSP0 stack: the kernel stack the CPU loads from
    // `tss.privilege_stack_table[0]` on a privilege change *into* ring 0 (a
    // ring3→ring0 interrupt/exception). Same lazy-static pattern as the IST
    // stack so we never touch a `static mut`.
    static RSP0_STACK: Lazy<[u8; RSP0_STACK_SIZE]> = Lazy::new(|| [0; RSP0_STACK_SIZE]);

    let mut tss = TaskStateSegment::new();

    // IST[0]: the known-good double-fault stack. Stacks grow down, so the CPU
    // loads the *top* (one past the end) of the array.
    let ist_start = VirtAddr::from_ptr(&*IST_STACK as *const u8);
    let ist_end = ist_start + IST_STACK_SIZE as u64;
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = ist_end;

    // RSP0: the ring-0 stack the CPU switches to on a ring3→ring0 transition.
    // Also grows down, so use the top of the array.
    let rsp0_start = VirtAddr::from_ptr(&*RSP0_STACK as *const u8);
    let rsp0_end = rsp0_start + RSP0_STACK_SIZE as u64;
    tss.privilege_stack_table[0] = rsp0_end;

    tss
});

/// The GDT plus the selectors needed to load CS/SS/TR. Built once.
///
/// Append order *is* the SYSCALL/SYSRET-required layout documented at the top
/// of this module: kernel code, kernel data, **user data, user code** (data
/// before code, for SYSRET), then the TSS.
static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    let code = gdt.append(Descriptor::kernel_code_segment());
    let data = gdt.append(Descriptor::kernel_data_segment());
    // User data BEFORE user code so SYSRET's `SS = base+8` / `CS = base+16`
    // lands on data then code (see the module-level SYSRET note). The crate's
    // `user_*_segment()` constructors already set DPL=3 and force RPL=3 in the
    // returned selectors.
    let user_data = gdt.append(Descriptor::user_data_segment());
    let user_code = gdt.append(Descriptor::user_code_segment());
    let tss = gdt.append(Descriptor::tss_segment(&TSS));
    (
        gdt,
        Selectors {
            code,
            data,
            user_data,
            user_code,
            tss,
        },
    )
});

/// The ring-3 user (code, data) segment selectors, each with RPL = 3.
///
/// For the later user-mode slices: an `iretq` to ring 3 loads these into
/// `CS`/`SS`, and `sysret` reconstructs the same selectors from `STAR`. Reading
/// them from the live GDT keeps callers off hardcoded selector literals if the
/// layout ever shifts again. Nothing loads them in this slice.
///
/// Returns `(user_code, user_data)`; both carry RPL 3 from the crate's
/// `user_*_segment()` descriptors.
// Scaffolding seam for the ring-3/sysret slices; not loaded in this slice.
#[allow(dead_code)]
pub fn user_selectors() -> (SegmentSelector, SegmentSelector) {
    (GDT.1.user_code, GDT.1.user_data)
}

/// The `STAR` MSR selector bases for the eventual SYSCALL/SYSRET fast path.
///
/// Returns `(syscall_base, sysret_base)`:
///   * `syscall_base` → `STAR[47:32]`: the kernel `CS` SYSCALL loads (`SS` is
///     `+8`). This is the kernel code selector (`0x08`); kernel data (`0x10`)
///     follows it.
///   * `sysret_base` → `STAR[63:48]`: the base SYSRET adds `+8`/`+16` to for
///     the user `SS`/`CS`. With user data at `0x18` and user code at `0x20`,
///     this base is `0x10` (`+8 → 0x18`, `+16 → 0x20`).
///
/// Exposed for the SYSCALL/SYSRET slice; unused in this scaffolding slice.
#[allow(dead_code)]
pub fn syscall_star_selectors() -> (SegmentSelector, SegmentSelector) {
    // syscall_base is just the kernel code selector. sysret_base is user_data
    // minus 8 (one descriptor), expressed as a selector with the appropriate
    // index; SYSRET ignores the RPL bits of STAR[63:48] and forces RPL 3.
    let syscall_base = GDT.1.code;
    let sysret_base = SegmentSelector(GDT.1.user_data.0 - 8);
    (syscall_base, sysret_base)
}

/// Install the GDT + TSS and load CS/SS and the task register.
///
/// # Safety
/// Call exactly once, early in bring-up on the boot core, before enabling
/// interrupts. Reloads segment registers and the task register.
pub unsafe fn init() {
    GDT.0.load();
    // SAFETY: the GDT was just loaded and contains these selectors at the
    // indices recorded in `GDT.1`; reloading CS/SS and the TR with them is the
    // required sequence to activate the new descriptors + TSS. We load only the
    // ring-0 kernel selectors here — the user selectors are installed by a
    // future ring-3 entry, not at boot.
    unsafe {
        CS::set_reg(GDT.1.code);
        SS::set_reg(GDT.1.data);
        load_tss(GDT.1.tss);
    }
}

// ---------------------------------------------------------------------------
// P4·SMP·S3 — per-AP GDT + TSS.
// ---------------------------------------------------------------------------
//
// The boot GDT carries a SINGLE TSS (one RSP0 + one IST stack). Two CPUs loading
// the same TSS would share that RSP0/IST — a corruption hazard the instant two
// CPUs take a ring-switch/double-fault concurrently. So each AP gets its OWN TSS
// (its own RSP0 + IST stacks) inside its OWN GDT copy, even though an idle S3 AP
// (no ring3↔ring0 switch, LVT timer masked) never actually uses RSP0/IST. This
// is the ARC-aligned, S4-ready choice.

/// Per-AP TSS, each with its own RSP0 + IST stack. Slot 0 (BSP) is unused — the
/// BSP loads the shared boot [`TSS`]. Built lazily per AP on first access.
static AP_TSS: [Lazy<TaskStateSegment>; hal::cpu::MAX_CPUS] =
    [const { Lazy::new(build_ap_tss) }; hal::cpu::MAX_CPUS];

/// Per-AP GDT carrying that AP's TSS descriptor. Same kernel/user code/data
/// layout as the boot [`GDT`]; only the TSS descriptor differs (it points at
/// `AP_TSS[idx]`). Each `Lazy` runs [`build_ap_gdt_current`], which reads
/// [`CURRENT_AP_BUILD`] (set by [`init_ap`] before forcing this entry) to pick
/// the right AP's TSS. The recorded selector set carries the AP's TSS selector.
static AP_GDT: [Lazy<(GlobalDescriptorTable, Selectors)>; hal::cpu::MAX_CPUS] =
    [const { Lazy::new(build_ap_gdt_current) }; hal::cpu::MAX_CPUS];

/// `Lazy` initializer for one `AP_GDT` slot: build the GDT for whichever AP is
/// currently being initialized (its index in [`CURRENT_AP_BUILD`]).
fn build_ap_gdt_current() -> (GlobalDescriptorTable, Selectors) {
    build_ap_gdt(ap_tss_for_current())
}

/// Build one AP's TSS with a fresh RSP0 + IST double-fault stack on the heap.
fn build_ap_tss() -> TaskStateSegment {
    // Heap-allocate the two stacks (the heap is up before APs start) and leak
    // them as 'static so the TSS can hold their top addresses. Leaking is fine:
    // an AP's stacks live for the whole machine lifetime.
    use alloc::vec;
    let ist: &'static mut [u8] = vec![0u8; IST_STACK_SIZE].leak();
    let rsp0: &'static mut [u8] = vec![0u8; RSP0_STACK_SIZE].leak();

    let mut tss = TaskStateSegment::new();
    let ist_top = VirtAddr::from_ptr(ist.as_ptr()) + IST_STACK_SIZE as u64;
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = ist_top;
    let rsp0_top = VirtAddr::from_ptr(rsp0.as_ptr()) + RSP0_STACK_SIZE as u64;
    tss.privilege_stack_table[0] = rsp0_top;
    tss
}

/// Build one AP's GDT with the same code/data descriptors as the boot GDT plus
/// `tss`'s descriptor. Returns the GDT and the selectors (the TSS selector is
/// the one the AP loads into TR).
fn build_ap_gdt(tss: &'static TaskStateSegment) -> (GlobalDescriptorTable, Selectors) {
    let mut gdt = GlobalDescriptorTable::new();
    let code = gdt.append(Descriptor::kernel_code_segment());
    let data = gdt.append(Descriptor::kernel_data_segment());
    let user_data = gdt.append(Descriptor::user_data_segment());
    let user_code = gdt.append(Descriptor::user_code_segment());
    let tss_sel = gdt.append(Descriptor::tss_segment(tss));
    (
        gdt,
        Selectors {
            code,
            data,
            user_data,
            user_code,
            tss: tss_sel,
        },
    )
}

/// Thread-local trick: the per-AP `AP_GDT` closure needs to know WHICH AP it is
/// building for. We pass it through a small per-call slot set by [`init_ap`]
/// right before it forces the matching `Lazy`. Single-writer per AP (the AP
/// itself), read once inside the closure.
static CURRENT_AP_BUILD: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(0);

/// Return the TSS for the AP currently being built (see [`CURRENT_AP_BUILD`]).
fn ap_tss_for_current() -> &'static TaskStateSegment {
    let idx = CURRENT_AP_BUILD.load(core::sync::atomic::Ordering::Relaxed);
    &AP_TSS[idx]
}

/// Load AP `idx`'s OWN GDT and TSS, plus the ring-0 CS/SS. The AP analog of
/// [`init`]: it gives this AP a private RSP0/IST so concurrent ring-switches /
/// double-faults across CPUs do not corrupt each other.
///
/// # Safety
/// Call once on AP `idx` (`idx < MAX_CPUS`, `idx != 0`) in long mode, after the
/// shared IDT/CR3 are active, with IRQs masked. Reloads the GDT, CS/SS, and TR.
pub unsafe fn init_ap(idx: usize) {
    // Tell the lazy builder which AP this is, then force its GDT (which forces
    // its TSS). Each AP touches only its own `idx`, so the shared atomic is set
    // and consumed before the next AP runs (APs are brought up serially by the
    // BSP, each waited-on before the next).
    CURRENT_AP_BUILD.store(idx, core::sync::atomic::Ordering::Relaxed);
    let gdt = &AP_GDT[idx];
    gdt.0.load();
    // SAFETY: this AP's GDT was just loaded; reload CS/SS with its kernel
    // selectors and TR with its own TSS selector (its private RSP0/IST).
    unsafe {
        CS::set_reg(gdt.1.code);
        SS::set_reg(gdt.1.data);
        load_tss(gdt.1.tss);
    }
}
