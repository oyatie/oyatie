//! EL0 user-mode + **process model**: an ELF64 loader, per-process address
//! spaces, a Linux/SysV **initial process stack** (argc/argv/envp/auxv), the
//! drop-to-EL0 path, a cooperative-plus-preemptive scheduler, and a Linux-ABI
//! syscall layer rich enough to run a **real static musl binary** *and* the
//! `clone`/`execve`/`wait4`/`exit` family so a parent can spawn and reap a child.
//!
//! This is **Frame** code: it touches page tables, copies into user memory,
//! `eret`s to EL0, switches `TTBR0_EL1`, and reads/writes user pointers — all of
//! which require `unsafe`. The safe kernel only calls the single safe
//! [`run_user`] entry; everything dangerous is encapsulated here (and in
//! [`crate::process`]) with a safety note at each site.
//!
//! ## From one process to many
//!
//! The earlier bring-up ran exactly one EL0 image to `exit` using global
//! statics (one page table, one frame pool, one brk cursor). The process model
//! moves all of that per-process state into [`crate::process::Process`] /
//! [`crate::process::AddressSpace`], which live on the kernel heap and are owned
//! by the global [`crate::process::Scheduler`]. The loader below builds a
//! `Process`; the syscall layer operates on *the current* process; and
//! `clone`/`execve`/`wait4`/`exit` manipulate the table + run queue.
//!
//! ## Per-process address space
//!
//! Each process owns a full L1/L2/L3 hierarchy plus its backing frames (see
//! [`crate::process::AddressSpace`]). `TTBR0_EL1` selects which is live. The
//! user window itself is unchanged from the single-process design: a 2 MiB
//! region at `USER_BASE` (`0x40_0000`) holding the ELF image, header copy, TLS,
//! brk heap, mmap region, and a 256 KiB initial stack growing down from the top.

use aarch64_cpu::asm::barrier;
use aarch64_cpu::registers::{CPACR_EL1, SCTLR_EL1, SP_EL0, TPIDR_EL0, TTBR0_EL1};
use tock_registers::interfaces::{Readable, Writeable};

use crate::exceptions::TrapFrame;
use crate::process::{self, AddressSpace, FileDesc, FileKind, Process, State};

// The pure layout/allocator/stack-builder math lives in `user_layout` so it can
// be unit-tested on the host. We pull its symbols in unqualified to keep the
// Frame code below readable.
use crate::user_layout::{
    build_stack_image, compute_tls_layout, deadline_after, timespec_to_cycles, user_range_ok,
    PagePerm, SleepCycles, StackInputs, TlsPhdr, MMAP_BASE, MMAP_TOP, PAGE_MASK, PAGE_SIZE,
    USER_BASE, USER_STACK_BOTTOM, USER_STACK_TOP, USER_TOP,
};
// Pure signal math: bit ops, the SigAction/SignalState PODs, the default-action
// classifier, and the shared signal-frame offset constants + alignment fns.
use crate::user_layout::signal as sig;

/// The user program image, embedded into the kernel at build time. This is the
/// freestanding **`user-spawn`** EL0 program (links at `USER_BASE`, ET_EXEC)
/// that exercises the process model: it `clone()`s a child, the child `exit`s
/// with a status, and the parent `wait4()`s to reap it. The same image is also
/// the `execve` target, so an `execve` re-runs the spawn demo, keeping the pid.
///
/// (The earlier single-process bring-up embedded a real static musl binary,
/// `out/user-musl.elf`; the kernel's ELF loader + syscall layer below still
/// handle that image — it simply does not exercise fork/wait, so the dedicated
/// `user-spawn` program is used to drive the new code paths.)
#[cfg(not(any(
    feature = "signal-demo",
    feature = "clock-demo",
    feature = "init-demo",
    feature = "talos-init",
    feature = "smp-sched-demo"
)))]
static USER_ELF: &[u8] = include_bytes!("../../../out/user-spawn.elf");

/// With `--features smp-sched-demo` (P4·SMP·S4a) the embedded image is the
/// no_std **`user-smpdemo`** fan-out supervisor (`crates/arch-aarch64/
/// user-smpdemo-src/`, built to `out/user-smpdemo.elf`). It forks N=8 worker
/// processes (> the test's `-smp 4`); the kernel places them round-robin across
/// the online CPUs and logs `sched: pid P -> cpu K` the first time each runs.
/// Under `-smp 4` those lines show workers on multiple distinct cpu indices,
/// proving the APs actually run processes. The parent `wait4`s all 8.
/// NON-default, verification-only; the default build keeps the user-spawn image
/// so the golden trace is untouched. Built via
/// crates/arch-aarch64/user-smpdemo-src/build.sh.
#[cfg(feature = "smp-sched-demo")]
static USER_ELF: &[u8] = include_bytes!("../../../out/user-smpdemo.elf");

/// With `--features signal-demo` the embedded image is swapped to the
/// **`user-signal`** program (installs a SIGUSR1 handler via `rt_sigaction` +
/// `SA_RESTORER`, raises it with `tgkill`, asserts the handler ran, then
/// exercises the VFS fd paths). This is a NON-default, verification-only image;
/// the golden harness runs the default build, so the golden stays untouched.
/// See P3_SIGNALS_SPEC.md §7. Built out-of-repo to `out/user-signal.elf`.
#[cfg(feature = "signal-demo")]
static USER_ELF: &[u8] = include_bytes!("../../../out/user-signal.elf");

/// With `--features clock-demo` the embedded image is swapped to the
/// **`user-clock`** program (clock_gettime/clock_nanosleep prove the real
/// timekeeper advances; prints `clock: mono advanced ...`). NON-default,
/// verification-only; the golden harness runs the default build so the golden
/// stays untouched (the clock syscalls are not on the traced path and
/// clock_gettime's return is the constant 0). See P3_TIMEKEEPING_SPEC.md §5.
/// Built out-of-repo to `out/user-clock.elf` via ../kernel-usermode-tests/clock/build.sh.
#[cfg(feature = "clock-demo")]
static USER_ELF: &[u8] = include_bytes!("../../../out/user-clock.elf");

/// With `--features talos-init` the embedded image is the **real, unmodified
/// talos-init** PID1 (`../operating-system/talos-init`, the talos-machined 7-phase boot
/// sequencer that links talos-machined/network/platform/runtime-cri/cosi),
/// cross-compiled for `aarch64-unknown-linux-musl` as a static, non-PIE ET_EXEC
/// linked at [`USER_BASE`] (`out/talos-init.elf`). It is much larger than `svc`
/// (~1.03 MiB), which is why the user VA window was enlarged from 2 MiB to 8 MiB
/// (`USER_NTABLES`). On boot it runs musl C-runtime startup then enters the
/// sequencer, whose phase-1 `MountPseudoFs` calls `mount(2)` — the kernel returns
/// `-ENOSYS`, which `talos-machined`'s skip-classifier does NOT tolerate, so the
/// real init aborts there. That abort is the Milestone-2 baseline. NON-default,
/// verification-only; the default build keeps the user-spawn image so the golden
/// trace is untouched. Built out-of-repo (rust:alpine, `--platform linux/arm64`,
/// `RUSTFLAGS=-C relocation-model=static -C link-arg=-no-pie -C
/// target-feature=+crt-static`). See MILESTONE_2_PLAN.md Slice 0.
#[cfg(feature = "talos-init")]
static USER_ELF: &[u8] = include_bytes!("../../../out/talos-init.elf");

/// With `--features init-demo` the embedded image is the no_std **PID1 init
/// supervisor** (`../kernel-usermode-tests/init/`, built to `out/user-init.elf`). PID1
/// installs SIGCHLD + SIGTERM handlers, clones a worker child that `execve`s the
/// real `svc` (see `EXEC_ELF` below, also swapped under init-demo), polls a short
/// `clock_nanosleep` watching the SIGCHLD `.bss` flag, then `wait4(WNOHANG)`-reaps
/// the worker and exits cleanly. This is the Milestone-1 parity-floor capstone.
/// NON-default, verification-only; the default build keeps the user-spawn image
/// so the golden stays untouched. See P3_PARITY_FLOOR_PLAN.md Slice C. Built
/// out-of-repo via ../kernel-usermode-tests/init/build.sh.
#[cfg(feature = "init-demo")]
static USER_ELF: &[u8] = include_bytes!("../../../out/user-init.elf");

/// The image `execve` loads. We have no filesystem, so `execve` ignores its path
/// argument and always loads this embedded program. In the DEFAULT build it is
/// `user-exec` (a distinct ELF that prints + `exit(7)`s); routing `execve` to a
/// *different* image than the caller proves the loader truly replaces the process
/// image (tear down old mappings, fresh ELF, reset PC/SP) rather than merely
/// restarting it.
#[cfg(not(any(feature = "init-demo", feature = "talos-init")))]
static EXEC_ELF: &[u8] = include_bytes!("../../../out/user-exec.elf");

/// With `--features talos-init` the `execve` target is the REAL **talos-svc**
/// musl binary (`out/svc.elf`, the same unmodified Rust-std/musl heartbeat worker
/// that runs on x86_64 as `out/svc-x86_64.elf`). So when the real talos-init PID1
/// clones a child that `execve`s `/usr/bin/svc`, it genuinely becomes the real
/// svc worker (clock heartbeats + `exit(0)`), which talos then reaps
/// (restart=never) and proceeds. This makes the "spawns a real service" claim
/// honest. Mirrors the init-demo arm. See MILESTONE_2_PLAN.md.
#[cfg(feature = "talos-init")]
static EXEC_ELF: &[u8] = include_bytes!("../../../out/svc.elf");

/// With `--features init-demo` the `execve` target is the REAL **talos-svc** musl
/// binary (`out/svc.elf`, the same unmodified Rust-std/musl workload that runs on
/// x86_64 as `out/svc-x86_64.elf`). So when PID1's cloned child `execve`s, it
/// genuinely becomes the real svc worker — PID1 supervises a real service.
/// FS-base-safe: PID1 is no_std and the single svc child is the only musl process
/// live at a time. See P3_PARITY_FLOOR_PLAN.md Slice C.
#[cfg(feature = "init-demo")]
static EXEC_ELF: &[u8] = include_bytes!("../../../out/svc.elf");

/// Saved kernel `TTBR0_EL1` so we can restore the pure identity map once every
/// process has exited and the workload is complete.
static mut SAVED_TTBR0: u64 = 0;

// ---- ELF64 reading helpers ------------------------------------------------

fn rd_u64(data: &[u8], off: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&data[off..off + 8]);
    u64::from_le_bytes(b)
}
fn rd_u32(data: &[u8], off: usize) -> u32 {
    let mut b = [0u8; 4];
    b.copy_from_slice(&data[off..off + 4]);
    u32::from_le_bytes(b)
}
fn rd_u16(data: &[u8], off: usize) -> u16 {
    let mut b = [0u8; 2];
    b.copy_from_slice(&data[off..off + 2]);
    u16::from_le_bytes(b)
}

/// The parsed pieces of a loaded ELF the stack builder needs.
struct LoadedElf {
    entry: u64,
    phdr_va: u64,
    phentsize: u64,
    phnum: u64,
    image_end: usize,
    tls: Option<TlsImage>,
}

/// The raw `PT_TLS` description needed to materialise the TLS image.
struct TlsImage {
    file_off: usize,
    phdr: TlsPhdr,
}

// ---- Minimal ELF64 loader (operates on an AddressSpace) -------------------

/// Parse `elf_bytes`, copy its `PT_LOAD` segments into `space` with per-segment
/// permissions, zero `.bss`, map the ELF header + program headers read-only for
/// `AT_PHDR`, and return [`LoadedElf`]. Panics on a malformed image.
///
/// # Safety
/// `space` must be a fresh/clear address space; writes into its mapped pages.
unsafe fn load_elf(space: &mut AddressSpace, elf_bytes: &[u8]) -> LoadedElf {
    let d = elf_bytes;
    assert!(d.len() >= 64, "ELF too small");
    assert_eq!(&d[0..4], b"\x7fELF", "bad ELF magic");
    assert_eq!(d[4], 2, "not ELF64");
    assert_eq!(d[5], 1, "not little-endian");
    assert_eq!(rd_u16(d, 16), 2, "not ET_EXEC (need static non-PIE)");
    assert_eq!(rd_u16(d, 18), 0xB7, "not EM_AARCH64");

    let entry = rd_u64(d, 24);
    let phoff = rd_u64(d, 32) as usize;
    let phentsize = rd_u16(d, 54) as usize;
    let phnum = rd_u16(d, 56) as usize;
    let ehsize = rd_u16(d, 52) as usize;

    let mut image_end = USER_BASE;
    let mut tls: Option<TlsImage> = None;

    for i in 0..phnum {
        let ph = phoff + i * phentsize;
        let p_type = rd_u32(d, ph);
        const PT_TLS: u32 = 7;
        if p_type == PT_TLS {
            let p_offset = rd_u64(d, ph + 8) as usize;
            let p_filesz = rd_u64(d, ph + 32) as usize;
            let p_memsz = rd_u64(d, ph + 40) as usize;
            let p_align = rd_u64(d, ph + 48) as usize;
            tls = Some(TlsImage {
                file_off: p_offset,
                phdr: TlsPhdr {
                    filesz: p_filesz,
                    memsz: p_memsz,
                    align: p_align,
                },
            });
            continue;
        }
        if p_type != 1 {
            continue; // not PT_LOAD
        }
        let p_flags = rd_u32(d, ph + 4);
        let p_offset = rd_u64(d, ph + 8) as usize;
        let p_vaddr = rd_u64(d, ph + 16) as usize;
        let p_filesz = rd_u64(d, ph + 32) as usize;
        let p_memsz = rd_u64(d, ph + 40) as usize;

        assert!(p_vaddr >= USER_BASE, "segment below user base");
        assert!(p_vaddr + p_memsz <= USER_TOP, "segment exceeds user window");

        let pf_x = (p_flags & 0x1) != 0;
        let pf_w = (p_flags & 0x2) != 0;
        let perm = if pf_w {
            PagePerm::ReadWrite
        } else if pf_x {
            PagePerm::ReadExec
        } else {
            PagePerm::ReadOnly
        };

        if p_filesz > 0 {
            // SAFETY: range checked to lie in the window; maps + copies.
            unsafe { space.copy_to_user(p_vaddr, &d[p_offset..p_offset + p_filesz], perm) };
        }
        let seg_end = p_vaddr + p_memsz;
        let mut va = (p_vaddr + p_filesz) & !PAGE_MASK;
        while va < seg_end {
            // SAFETY: in-window; ensures a zeroed frame with `perm`.
            unsafe { space.map_page(va, perm) };
            va += PAGE_SIZE;
        }

        let end = (seg_end + PAGE_MASK) & !PAGE_MASK;
        if end > image_end {
            image_end = end;
        }
    }

    // Map the ELF header + program header table for AT_PHDR (read-only).
    let phdrs_end = phoff + phnum * phentsize;
    let hdr_span = core::cmp::max(ehsize, phdrs_end);
    let phdr_page_va = image_end;
    // SAFETY: maps pages on demand within the window and copies the header span.
    unsafe { space.copy_to_user(phdr_page_va, &d[0..hdr_span], PagePerm::ReadOnly) };
    let phdr_va = (phdr_page_va + phoff) as u64;
    let header_end = (phdr_page_va + hdr_span + PAGE_MASK) & !PAGE_MASK;

    LoadedElf {
        entry,
        phdr_va,
        phentsize: phentsize as u64,
        phnum: phnum as u64,
        image_end: header_end,
        tls,
    }
}

/// Materialise the program's TLS block + TCB in `space` and return the
/// thread-pointer value for `TPIDR_EL0`, or `None` if there is no `PT_TLS`.
///
/// # Safety
/// `space` must hold the loaded image; maps pages on demand and writes through
/// their identity alias.
unsafe fn setup_tls(space: &mut AddressSpace, elf: &LoadedElf, elf_bytes: &[u8]) -> Option<u64> {
    let tls = elf.tls.as_ref()?;
    let lay = compute_tls_layout(elf.image_end, &tls.phdr);
    assert!(
        user_range_ok(lay.tp as u64, (lay.region_end - lay.tp) as u64),
        "TLS region escapes the user window"
    );
    let mut va = lay.tp & !PAGE_MASK;
    while va < lay.region_end {
        // SAFETY: range checked; ensures a zeroed RW frame.
        unsafe { space.map_page(va, PagePerm::ReadWrite) };
        va += PAGE_SIZE;
    }
    if lay.filesz > 0 {
        let src = &elf_bytes[tls.file_off..tls.file_off + lay.filesz];
        // SAFETY: image_va..image_va+filesz is inside the just-mapped region.
        unsafe { space.copy_to_user(lay.image_va, src, PagePerm::ReadWrite) };
    }
    Some(lay.tp as u64)
}

// ---- Auxiliary-vector / initial-stack builder -----------------------------

const ARGV0: &[u8] = b"/init\0";
const ENV0: &[u8] = b"PATH=/usr/bin\0";
const RANDOM16: [u8; 16] = [
    0x9e, 0x37, 0x79, 0xb9, 0x7f, 0x4a, 0x7c, 0x15, 0xf3, 0x9c, 0xc0, 0x60, 0x5c, 0xed, 0xc8, 0x34,
];

/// Build the Linux/SysV aarch64 initial process stack in `space` and return the
/// final 16-byte-aligned `SP_EL0`.
///
/// # Safety
/// `space` must hold the loaded image; maps the stack pages on demand.
unsafe fn build_initial_stack(space: &mut AddressSpace, elf: &LoadedElf) -> u64 {
    let argv: [&[u8]; 1] = [ARGV0];
    let envp: [&[u8]; 1] = [ENV0];
    let inp = StackInputs {
        entry: elf.entry,
        phdr_va: elf.phdr_va,
        phentsize: elf.phentsize,
        phnum: elf.phnum,
        argv: &argv,
        envp: &envp,
        random: &RANDOM16,
        stack_top: USER_STACK_TOP,
    };
    let img = build_stack_image(&inp);

    // SAFETY: every VA the builder produced lies inside the stack region;
    // `copy_to_user` maps pages on demand and writes through the identity alias.
    unsafe {
        space.copy_to_user(img.argv_vas.as_slice()[0], ARGV0, PagePerm::ReadWrite);
        space.copy_to_user(img.envp_vas.as_slice()[0], ENV0, PagePerm::ReadWrite);
        space.copy_to_user(img.random_va, &RANDOM16, PagePerm::ReadWrite);
        let words = img.words.as_slice();
        let word_bytes = core::slice::from_raw_parts(words.as_ptr() as *const u8, words.len() * 8);
        space.copy_to_user(img.sp, word_bytes, PagePerm::ReadWrite);
    }

    // Pre-map the rest of the stack region down to STACK_BOTTOM (no demand-paging
    // for EL0 stack growth yet).
    let mut va = USER_STACK_BOTTOM;
    while va < img.sp {
        // SAFETY: in-window; ensures a zeroed RW frame.
        unsafe { space.map_page(va, PagePerm::ReadWrite) };
        va += PAGE_SIZE;
    }

    img.sp as u64
}

/// Load `elf_bytes` into a fresh address space and return a fully-initialised,
/// Runnable [`Process`] (pid 0 placeholder; the scheduler assigns the real pid)
/// whose saved context will `eret` to the program entry with a SysV stack.
///
/// # Safety
/// Builds page tables and copies into user memory; must run on the boot core
/// with the MMU up.
unsafe fn build_process(ppid: u32, elf_bytes: &[u8]) -> Process {
    let mut space = AddressSpace::new();
    // SAFETY: fresh space; the loader maps + copies into it.
    let elf = unsafe { load_elf(&mut space, elf_bytes) };
    // SAFETY: image is loaded; lays out TLS in the same space.
    let tp = unsafe { setup_tls(&mut space, &elf, elf_bytes) };
    let brk_start = match elf.tls.as_ref() {
        Some(tls) => compute_tls_layout(elf.image_end, &tls.phdr).region_end,
        None => elf.image_end,
    };
    // SAFETY: builds the initial stack image in the space.
    let sp = unsafe { build_initial_stack(&mut space, &elf) };

    let mut proc = Process::new_loaded(ppid, space);
    proc.brk_cur = brk_start;
    proc.mmap_cur = MMAP_BASE;
    proc.tpidr = tp.unwrap_or(0);
    // Wire fds 0/1/2 -> /dev/console for the freshly-loaded image. (On execve
    // this fresh table is discarded in favour of preserving the live process's
    // fds — see `sys_execve` — so only the very first process actually adopts
    // these; but building them here keeps `build_process` self-contained.)
    proc.init_std_fds();
    // Initial EL0 context: entry in ELR, EL0t SPSR, SP_EL0 = sp, x0..x30 = 0.
    proc.ctx.elr = elf.entry;
    proc.ctx.spsr = SPSR_EL0T;
    proc.ctx.sp = sp;
    proc.state = State::Runnable;
    proc
}

// ---- SPSR constants -------------------------------------------------------

/// SPSR_EL1 value to enter EL0 with SP_EL0 (M[3:0]=0b0000 = EL0t), DAIF clear.
const SPSR_EL0T: u64 = 0;
/// SPSR to resume at EL1h (M=0b0101) with DAIF masked, used when the whole
/// workload is finished and we return control to the kernel boot flow.
const SPSR_KERNEL_RETURN: u64 = 0b0101 | (0b1111 << 6);

// ---- Linux aarch64 syscall numbers we recognise --------------------------

const SYS_GETCWD: u64 = 17;
/// `mkdirat(dirfd, path, mode)` — Linux aarch64 nr 34. Slice 2: creates the
/// directory (idempotently) in the in-RAM VFS via `with_vfs(|v| v.mkdir_p(path))`
/// and returns 0, so the real talos-init's SystemDirectories phase advances
/// instead of aborting on `-ENOSYS`. `dirfd` is ignored (paths are absolute);
/// `mode` is ignored (no permission model in M2).
const SYS_MKDIRAT: u64 = 34;
/// `mount(source, target, fstype, flags, data)` — Linux aarch64 nr 40. Slice 1:
/// records the mount in the in-RAM VFS and returns 0 so the real talos-init's
/// MountPseudoFs phase advances.
const SYS_MOUNT: u64 = 40;
const SYS_DUP: u64 = 23;
/// `dup3(oldfd, newfd, flags)` — arm64 has no `dup2` (that is `dup3` with
/// `flags == 0`). We ignore `O_CLOEXEC` (no exec-close in our demos).
const SYS_DUP3: u64 = 24;
const SYS_FCNTL: u64 = 25;
const SYS_IOCTL: u64 = 29;
const SYS_FACCESSAT: u64 = 48;
const SYS_OPENAT: u64 = 56;
const SYS_CLOSE: u64 = 57;
const SYS_LSEEK: u64 = 62;
const SYS_READ: u64 = 63;
const SYS_WRITE: u64 = 64;
const SYS_WRITEV: u64 = 66;
const SYS_PPOLL: u64 = 73;
const SYS_READLINKAT: u64 = 78;
const SYS_NEWFSTATAT: u64 = 79;
const SYS_FSTAT: u64 = 80;
const SYS_EXIT: u64 = 93;
const SYS_EXIT_GROUP: u64 = 94;
const SYS_SET_TID_ADDRESS: u64 = 96;
const SYS_FUTEX: u64 = 98;
const SYS_SET_ROBUST_LIST: u64 = 99;
const SYS_NANOSLEEP: u64 = 101;
const SYS_SCHED_YIELD: u64 = 124;
const SYS_CLOCK_GETTIME: u64 = 113;
const SYS_CLOCK_NANOSLEEP: u64 = 115;
const SYS_SCHED_GETAFFINITY: u64 = 123;
const SYS_KILL: u64 = 129;
const SYS_TKILL: u64 = 130;
const SYS_TGKILL: u64 = 131;
const SYS_SIGALTSTACK: u64 = 132;
const SYS_RT_SIGACTION: u64 = 134;
const SYS_RT_SIGPROCMASK: u64 = 135;
const SYS_RT_SIGPENDING: u64 = 136;
const SYS_RT_SIGRETURN: u64 = 139;
const SYS_GETTIMEOFDAY: u64 = 169;
const SYS_GETPID: u64 = 172;
const SYS_GETPPID: u64 = 173;
const SYS_GETUID: u64 = 174;
const SYS_GETEUID: u64 = 175;
const SYS_GETGID: u64 = 176;
const SYS_GETEGID: u64 = 177;
const SYS_GETTID: u64 = 178;
const SYS_SYSINFO: u64 = 179;
const SYS_BRK: u64 = 214;
const SYS_MUNMAP: u64 = 215;
const SYS_MREMAP: u64 = 216;
/// `sync()` — Linux aarch64 nr 81. talos's `power_off()` flushes filesystems via
/// musl `sync()` before rebooting. We have no writeback caches, so this is a
/// no-op returning 0 (Linux `sync` always succeeds and returns 0).
const SYS_SYNC: u64 = 81;
/// `reboot(magic1, magic2, cmd, arg)` — Linux aarch64 nr 142. talos's
/// `power_off()` calls musl `reboot(RB_POWER_OFF)`, which lowers to
/// `reboot(LINUX_REBOOT_MAGIC1, LINUX_REBOOT_MAGIC2, LINUX_REBOOT_CMD_POWER_OFF,
/// NULL)`. On the POWER_OFF command we power the machine off via PSCI SYSTEM_OFF
/// (clean QEMU exit).
const SYS_REBOOT: u64 = 142;
const SYS_CLONE: u64 = 220;
const SYS_EXECVE: u64 = 221;
const SYS_MMAP: u64 = 222;
const SYS_MPROTECT: u64 = 226;
const SYS_WAIT4: u64 = 260;
const SYS_PRLIMIT64: u64 = 261;
const SYS_GETRANDOM: u64 = 278;

// ---- M2 network slice: minimal AF_NETLINK socket syscalls (aarch64 nrs) -----
// The real, unmodified talos-init's `list_link_statuses` (the only hard-fatal
// netlink consumer) issues exactly socket+bind+sendto+recvfrom+close on an
// `AF_NETLINK`/`NETLINK_ROUTE` socket to dump links. `close` already exists
// (`SYS_CLOSE` = 57). aarch64 socket nr=198 is CONFIRMED from the live boot
// trace (`[pid 1] syscall 198 -> -38`). musl's `recv` lowers to `recvfrom`.
const SYS_SOCKET: u64 = 198;
const SYS_BIND: u64 = 200;
const SYS_SENDTO: u64 = 206;
const SYS_RECVFROM: u64 = 207;

// ---- WAVE 1: libc-init / process-info surface (aarch64 nrs) ----------------
// Every real glibc/musl binary issues these during init; they were previously
// -ENOSYS. All ADDITIVE (new syscall numbers) so they cannot alter the existing
// 'user-spawn' golden trace (getpid/clone/getpid/getppid). The pure byte-layout
// math (utsname/umask/clock_getres) lives in the shared `user_layout::procinfo`;
// the handlers below do the bounds-checked user copy via the existing pattern.
/// `uname(buf)` — fill `struct utsname` (six NUL-padded 65-byte fields).
const SYS_UNAME: u64 = 160;
/// `umask(mask)` — set the per-process file-creation mask, return the previous.
const SYS_UMASK: u64 = 166;
/// `getrusage(who, usage)` — write a zeroed `struct rusage`, return 0.
const SYS_GETRUSAGE: u64 = 165;
/// `times(buf)` — write a zeroed `struct tms`, return a monotonic tick count.
const SYS_TIMES: u64 = 153;
/// `clock_getres(clk, res)` — write timespec {0, 1} (1 ns resolution), return 0.
const SYS_CLOCK_GETRES: u64 = 114;
/// `setpgid(pid, pgid)` — single-session model, accept and return 0.
const SYS_SETPGID: u64 = 154;
/// `getpgid(pid)` — single-session model, return the current pid.
const SYS_GETPGID: u64 = 155;
/// `getsid(pid)` — single-session model, return the current pid.
const SYS_GETSID: u64 = 156;
/// `setsid()` — single-session model, return the current pid.
const SYS_SETSID: u64 = 157;
/// `setpriority(which, who, prio)` — no scheduler priority model, return 0.
const SYS_SETPRIORITY: u64 = 140;
/// `getpriority(which, who)` — no scheduler priority model, return 0 (nice 0).
const SYS_GETPRIORITY: u64 = 141;
/// `prctl(option, ...)` — minimal: PR_SET_NAME accepts, PR_GET_NAME zeroes the
/// 16-byte user buffer, all other options succeed with 0.
const SYS_PRCTL: u64 = 167;

/// `prctl(PR_SET_NAME, name, …)` — set the (16-byte) thread name. We accept it.
const PR_SET_NAME: u64 = 15;
/// `prctl(PR_GET_NAME, buf, …)` — copy the thread name into a 16-byte user buf.
const PR_GET_NAME: u64 = 16;
/// Length of the kernel thread-name buffer `prctl` reads/writes.
const TASK_COMM_LEN: u64 = 16;

// ---- errno-style returns --------------------------------------------------

const ENOSYS: u64 = (-38i64) as u64;
const EFAULT: u64 = (-14i64) as u64;
const EINVAL: u64 = (-22i64) as u64;
const ENOTTY: u64 = (-25i64) as u64;
const EBADF: u64 = (-9i64) as u64;
const ECHILD: u64 = (-10i64) as u64;
const MAP_FAILED: u64 = (-1i64) as u64;

/// `clone` flag set by glibc/musl `fork()`: deliver `SIGCHLD` (17) on exit, no
/// other sharing. We support exactly the fork ABI (no new stack, no CLONE_VM).
const SIGCHLD: u64 = 17;
/// `clone` flags that, if present, mean "thread" not "fork" (shared VM). We do
/// not support threads; such a clone returns -EINVAL.
const CLONE_VM: u64 = 0x0000_0100;

/// `wait4` option: do not block if no child has exited yet.
const WNOHANG: u64 = 1;

/// `reboot(2)` magic numbers (Linux `include/uapi/linux/reboot.h`). musl's
/// `reboot(cmd)` calls `reboot(LINUX_REBOOT_MAGIC1, LINUX_REBOOT_MAGIC2, cmd, 0)`;
/// we recognise the magics + the POWER_OFF command and actually power down.
const LINUX_REBOOT_MAGIC1: u64 = 0xfee1_dead;
const LINUX_REBOOT_MAGIC2: u64 = 0x2812_1969;
/// `LINUX_REBOOT_CMD_POWER_OFF` — power the machine off. talos's `power_off()`
/// uses exactly this command.
const RB_POWER_OFF: u64 = 0x4321_fedc;

/// What the SVC dispatch wants the trap epilogue to do. Carried out-of-band
/// from the user-visible return value because process-model calls manipulate the
/// run queue rather than simply returning a number.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SyscallOutcome {
    /// Write `ret` into the current process's x0 and resume it.
    Return(u64),
    /// The current process voluntarily yielded: save its context, pick the next
    /// runnable process, and switch the live frame to it. Used by `sched_yield`,
    /// where the process stays `Runnable` (NOT wakeable by `complete_waits`), so
    /// saving its context AFTER the syscall returns is race-free.
    Reschedule,
    /// The current process BLOCKED and has ALREADY saved its own context into its
    /// `ctx` under the scheduler lock, atomically with the `Waiting` transition
    /// (P4·SMP·S4d). The epilogue must therefore pick the next process WITHOUT
    /// re-saving: a post-block `save_current` would race a sibling CPU's
    /// `complete_waits`, which may have already written the wake's `x0`
    /// (reaped-child pid) into this process's `ctx` — clobbering it back to the
    /// pre-block frame and corrupting the `wait4` return (premature/garbage
    /// ECHILD). Only `wait4`'s blocking path uses this.
    BlockedReschedule,
    /// The live frame has *already* been rewritten to the context that should
    /// run next (e.g. `execve` reset the current process in place); `eret`
    /// directly without saving or re-scheduling.
    Resume,
    /// Every process has exited; tear down and return to the kernel.
    Finished,
}

/// Handle one SVC from EL0. Reads x8 (number) + x0..x5 (args), dispatches the
/// Linux-ABI call against the **current process**, and either writes the result
/// into the frame's x0 (and resumes), reschedules to another process, or — when
/// the last process exits — diverts the `eret` back into the kernel.
///
/// # Safety
/// `frame` must point at the live trap frame the vector stub pushed; user
/// pointers are validated before any access.
pub unsafe fn handle_svc(frame: *mut TrapFrame) {
    // SAFETY: the synchronous vector just pushed a valid frame at `frame`.
    let f = unsafe { &mut *frame };
    let num = f.regs[8];
    let (a0, a1, a2, a3, a4, _a5) = (
        f.regs[0], f.regs[1], f.regs[2], f.regs[3], f.regs[4], f.regs[5],
    );

    let outcome: SyscallOutcome = match num {
        SYS_WRITE => SyscallOutcome::Return(sys_write(a0, a1, a2)),
        // SAFETY: validates the iovec array + each buffer before reading.
        SYS_WRITEV => SyscallOutcome::Return(unsafe { sys_writev(a0, a1, a2) }),
        SYS_READ => SyscallOutcome::Return(sys_read(a0, a1, a2)),
        SYS_READLINKAT => SyscallOutcome::Return(EINVAL),
        SYS_IOCTL => SyscallOutcome::Return(ENOTTY),
        // SAFETY: copies the path string from userspace via the bounds-checked,
        // PAN-bracketed user-access path before resolving it.
        SYS_OPENAT => SyscallOutcome::Return(unsafe { sys_openat(a1) }),
        SYS_FACCESSAT => SyscallOutcome::Return((-2i64) as u64),
        // mount(source=a0, target=a1, fstype=a2, flags=a3, data=a4). Slice 1:
        // record the pseudo-fs mount in the in-RAM VFS and return 0 (data a4
        // ignored). SAFETY: copies the three NUL-terminated strings out of user
        // memory via the bounds-checked, PAN-bracketed byte-copy path before use.
        SYS_MOUNT => SyscallOutcome::Return(unsafe { sys_mount(a0, a1, a2, a3) }),
        // mkdirat(dirfd=a0, path=a1, mode=a2). Slice 2: create the dir node
        // idempotently in the in-RAM VFS and return 0. dirfd/mode ignored
        // (absolute paths, no perm model). SAFETY: copies the NUL-terminated
        // path out of user memory via the bounds-checked, PAN-bracketed path.
        SYS_MKDIRAT => SyscallOutcome::Return(unsafe { sys_mkdirat(a1) }),
        SYS_CLOSE => SyscallOutcome::Return(sys_close(a0)),
        // ---- M2 network slice: minimal AF_NETLINK link-status dump ----
        // socket(domain=a0, type=a1, protocol=a2): an AF_NETLINK/NETLINK_ROUTE
        // socket allocates a fresh Netlink fd; any other domain → -EAFNOSUPPORT.
        SYS_SOCKET => SyscallOutcome::Return(sys_socket(a0, a1, a2)),
        // bind(fd=a0, addr=a1, addrlen=a2): validate the sockaddr_nl → 0.
        // SAFETY: copies the 12-byte sockaddr_nl out of user memory via the
        // bounds-checked, PAN-bracketed copy path before reading it.
        SYS_BIND => SyscallOutcome::Return(unsafe { sys_bind(a0, a1, a2) }),
        // sendto(fd=a0, buf=a1, len=a2, flags=a3, addr=a4, addrlen=a5): copy the
        // request bytes, parse the RTM_GETLINK dump, arm the response. SAFETY:
        // validates + copies the flat send buffer before parsing it.
        SYS_SENDTO => SyscallOutcome::Return(unsafe { sys_sendto(a0, a1, a2) }),
        // recvfrom(fd=a0, buf=a1, len=a2, flags=a3, addr=a4, addrlen=a5): drain
        // the armed NLMSG_DONE into the user buffer; write a sockaddr_nl into
        // `addr` if non-NULL. SAFETY: validates + copies into the flat user
        // buffer (and the optional src-addr) before writing.
        SYS_RECVFROM => SyscallOutcome::Return(unsafe { sys_recvfrom(a0, a1, a2, a4, _a5) }),
        SYS_LSEEK => SyscallOutcome::Return((-29i64) as u64),
        SYS_DUP => SyscallOutcome::Return(a0),
        SYS_DUP3 => SyscallOutcome::Return(sys_dup3(a0, a1)),
        SYS_FCNTL => SyscallOutcome::Return(0),
        SYS_GETCWD => SyscallOutcome::Return(sys_getcwd(a0, a1)),
        SYS_NEWFSTATAT => SyscallOutcome::Return(sys_fstat(a2)),
        SYS_FSTAT => SyscallOutcome::Return(sys_fstat(a1)),
        SYS_SET_TID_ADDRESS => SyscallOutcome::Return(current_pid() as u64),
        SYS_FUTEX => SyscallOutcome::Return(0),
        SYS_SET_ROBUST_LIST => SyscallOutcome::Return(0),
        SYS_NANOSLEEP => SyscallOutcome::Return(sys_nanosleep(a0, a1)),
        SYS_CLOCK_NANOSLEEP => SyscallOutcome::Return(sys_nanosleep(a2, a3)),
        SYS_SCHED_YIELD => {
            // Cooperative yield: re-queue the current process and switch.
            SyscallOutcome::Reschedule
        }
        // ---- POSIX signals (real delivery; see `sig::*` helpers + the
        // return-to-user `deliver_pending_signals` hook below) ----
        // SAFETY: validate sigsetsize + read/write the user structs through the
        // bounds-checked EL0-alias path before any access.
        SYS_RT_SIGACTION => SyscallOutcome::Return(unsafe { sys_rt_sigaction(a0, a1, a2, a3) }),
        SYS_RT_SIGPROCMASK => {
            SyscallOutcome::Return(unsafe { sys_rt_sigprocmask(a0, a1, a2, a3) })
        }
        SYS_RT_SIGPENDING => SyscallOutcome::Return(unsafe { sys_rt_sigpending(a0, a1) }),
        // rt_sigreturn restores the pre-signal context from the user frame; it
        // returns `Resume` (no trace line, no x0 overwrite) — the live frame is
        // rewritten in place.
        // SAFETY: bounds-checks the frame before restoring it.
        SYS_RT_SIGRETURN => unsafe { sys_rt_sigreturn(f) },
        SYS_SIGALTSTACK => SyscallOutcome::Return(unsafe { sys_sigaltstack(a0, a1) }),
        SYS_KILL => SyscallOutcome::Return(sys_kill(a0, a1)),
        SYS_TKILL => SyscallOutcome::Return(sys_kill(a0, a1)),
        SYS_TGKILL => SyscallOutcome::Return(sys_tgkill(a0, a1, a2)),
        SYS_GETPID => SyscallOutcome::Return(current_pid() as u64),
        SYS_GETPPID => SyscallOutcome::Return(current_ppid() as u64),
        SYS_GETTID => SyscallOutcome::Return(current_pid() as u64),
        SYS_GETUID | SYS_GETEUID | SYS_GETGID | SYS_GETEGID => SyscallOutcome::Return(0),
        SYS_CLOCK_GETTIME => SyscallOutcome::Return(sys_clock_gettime(a0, a1)),
        SYS_GETTIMEOFDAY => SyscallOutcome::Return(sys_gettimeofday(a0)),
        SYS_SYSINFO => SyscallOutcome::Return(sys_sysinfo(a0)),

        // ---- WAVE 1: libc-init / process-info surface (additive) ----
        // uname(buf): write `struct utsname` (six NUL-padded 65-byte fields).
        SYS_UNAME => SyscallOutcome::Return(sys_uname(a0)),
        // umask(mask): swap the per-process mask, return the previous.
        SYS_UMASK => SyscallOutcome::Return(sys_umask(a0)),
        // getrusage(who, usage): zeroed `struct rusage` -> user; return 0.
        SYS_GETRUSAGE => SyscallOutcome::Return(sys_getrusage(a1)),
        // times(buf): zeroed `struct tms` -> user; return a monotonic tick count.
        SYS_TIMES => SyscallOutcome::Return(sys_times(a0)),
        // clock_getres(clk, res): timespec {0, 1} (1 ns) for a known clock.
        SYS_CLOCK_GETRES => SyscallOutcome::Return(sys_clock_getres(a0, a1)),
        // Single-session process-group/session identity model.
        SYS_GETPGID | SYS_SETSID => SyscallOutcome::Return(current_pid() as u64),
        SYS_GETSID => SyscallOutcome::Return(current_pid() as u64),
        SYS_SETPGID => SyscallOutcome::Return(0),
        SYS_GETPRIORITY | SYS_SETPRIORITY => SyscallOutcome::Return(0),
        // prctl(option, …): PR_SET_NAME accepts, PR_GET_NAME zeroes a 16-byte
        // user buffer, every other option succeeds with 0.
        SYS_PRCTL => SyscallOutcome::Return(sys_prctl(a0, a1)),
        SYS_PPOLL => SyscallOutcome::Return(sys_ppoll(a0, a1, a2)),
        SYS_GETRANDOM => SyscallOutcome::Return(sys_getrandom(a0, a1, a2)),
        SYS_SCHED_GETAFFINITY => SyscallOutcome::Return(sys_sched_getaffinity(a0, a1, a2)),
        SYS_BRK => SyscallOutcome::Return(sys_brk(a0)),
        SYS_MMAP => SyscallOutcome::Return(sys_mmap(a1, a4)),
        SYS_MUNMAP => SyscallOutcome::Return(0),
        SYS_MREMAP => SyscallOutcome::Return(MAP_FAILED),
        SYS_MPROTECT => SyscallOutcome::Return(0),
        SYS_PRLIMIT64 => SyscallOutcome::Return(0),

        // ---- process model ----
        SYS_CLONE => SyscallOutcome::Return(sys_clone(f, a0, a1)),
        // SAFETY: validates the path/argv/envp pointers; rebuilds the image.
        SYS_EXECVE => unsafe { sys_execve(f, a0) },
        SYS_WAIT4 => sys_wait4(f, a0, a1, a2),
        // SAFETY: `f` is the live frame; exit switches it to the next process.
        SYS_EXIT | SYS_EXIT_GROUP => unsafe { sys_exit(f, a0 as i32) },

        // ---- shutdown ----
        // sync(): we have no writeback caches, so flushing is a no-op -> 0.
        SYS_SYNC => SyscallOutcome::Return(0),
        // reboot(magic1, magic2, cmd, arg): on RB_POWER_OFF this never returns
        // (PSCI SYSTEM_OFF -> clean QEMU exit); other commands return.
        SYS_REBOOT => SyscallOutcome::Return(sys_reboot(a0, a1, a2)),

        _ => {
            log_syscall(num, ENOSYS);
            SyscallOutcome::Return(ENOSYS)
        }
    };

    match outcome {
        SyscallOutcome::Return(ret) => {
            if num != SYS_WRITE && num != SYS_WRITEV {
                log_syscall(num, ret);
            }
            f.regs[0] = ret;
        }
        SyscallOutcome::Reschedule => {
            // SAFETY: `f` is the live frame; `schedule` saves the current ctx
            // and rewrites `f` to the next runnable process.
            unsafe {
                process::save_current(f);
                if !process::schedule(f) {
                    // Nothing runnable for THIS CPU: BSP+empty-table diverts to
                    // power-off; otherwise (re-)enter this CPU's EL1 idle loop.
                    idle_or_finish(f);
                }
            }
        }
        SyscallOutcome::BlockedReschedule => {
            // The blocking syscall ALREADY saved this process's context into its
            // `ctx` under the scheduler lock (atomic with the `Waiting` mark), so
            // we must NOT `save_current` here — a sibling CPU's `complete_waits`
            // may already have published the wake's `x0` into that `ctx`, and a
            // re-save would clobber it (the `wait4`-FAILED race). Just pick next.
            // SAFETY: `f` is the live frame; `schedule` rewrites it to the next
            // runnable process (or returns false → idle/finish).
            unsafe {
                if !process::schedule(f) {
                    idle_or_finish(f);
                }
            }
        }
        // The frame already holds the context to resume (execve reset in place).
        SyscallOutcome::Resume => {}
        SyscallOutcome::Finished => {
            finish_workload(f);
            // Workload finished: returning to the kernel, not to EL0 — do NOT
            // attempt signal delivery against a (possibly reaped) process.
            return;
        }
    }

    // Last step before returning to EL0: deliver one pending, unblocked signal
    // to the now-current process (rewriting `f` to enter its handler). This is
    // the single per-arch delivery site for the syscall path. Skipped above for
    // `Finished` (we're diverting back into the kernel, not to a user process).
    // SAFETY: `f` is the live trap frame the trampoline will `eret` from;
    // `deliver_pending_signals` only ever rewrites it to a validated handler
    // frame inside the current process's EL0 window (or terminates the process).
    unsafe {
        deliver_pending_signals(f);
    }
}

/// Trace one syscall as `[syscall] <nr> -> <ret>` with the current pid for
/// process-model visibility.
fn log_syscall(num: u64, ret: u64) {
    crate::kprintln!("[pid {}] syscall {} -> {}", current_pid(), num, ret as i64);
}

// ---- helpers that read the current process --------------------------------

fn current_pid() -> u32 {
    process::with_sched(|s| s.current_pid())
}
fn current_ppid() -> u32 {
    process::with_sched(|s| s.current().ppid)
}

// ---- I/O + memory syscalls (operate on the current process) ---------------

/// Resolve `fd` against the current process's fd table to its backing
/// [`FileKind`], or `None` (→ `-EBADF`) if the fd is not open.
fn fd_kind(fd: u64) -> Option<FileKind> {
    process::with_sched(|s| s.current().fd_kind(fd as i64))
}

/// `write(fd, buf, len)` routed through the fd table. `fd` is looked up:
/// absent → `-EBADF`; `/dev/console` → the existing console write (byte-for-byte
/// identical to the pre-fd-table path so the golden trace is unaffected);
/// `/dev/null` → discard and report `len`.
fn sys_write(fd: u64, buf: u64, len: u64) -> u64 {
    let kind = match fd_kind(fd) {
        Some(k) => k,
        None => return EBADF,
    };
    if len == 0 {
        return 0;
    }
    if !user_range_ok(buf, len) {
        return EFAULT;
    }
    match kind {
        FileKind::Console => {
            // SAFETY: range validated to lie inside the current process's mapped
            // EL0 window (live in TTBR0), which EL1 may also read.
            let bytes = unsafe { core::slice::from_raw_parts(buf as *const u8, len as usize) };
            for &b in bytes {
                crate::console::_print(format_args!("{}", b as char));
            }
            len
        }
        // /dev/null: validate the buffer (for EFAULT parity) then discard.
        FileKind::Null => len,
        // A netlink fd is written via sendto (not write); a bare write() just
        // reports the bytes "accepted" (the talos dump path never takes this).
        FileKind::Netlink => len,
    }
}

/// `read(fd, buf, len)` routed through the fd table. Absent fd → `-EBADF`. An fd
/// opened against a VFS `File` node (Slice 3: baked `/machine-config.yaml`,
/// `/proc/cmdline`, …) copies `node.data` from the per-fd offset into the user
/// buffer and advances the offset (so successive reads stream the file and a
/// final read returns 0 = EOF). Console and Null return 0 (EOF) — no input
/// source yet — exactly as before, so the golden trace is unaffected.
fn sys_read(fd: u64, buf: u64, len: u64) -> u64 {
    let handle = match process::with_sched(|s| s.current().fd_handle(fd as i64)) {
        Some(h) => h,
        None => return EBADF,
    };
    if len != 0 && !user_range_ok(buf, len) {
        return EFAULT;
    }
    // A File-backed fd serves bytes from the VFS node at the per-fd offset.
    if let Some(node) = handle.node {
        if len == 0 {
            return 0;
        }
        let off = handle.read_off() as usize;
        // Copy out of the VFS into a bounded staging buffer, then into the
        // validated EL0 window (the VFS lives in EL1 kernel memory).
        let mut staging = [0u8; READ_CHUNK];
        let want = core::cmp::min(len as usize, READ_CHUNK);
        let n = process::with_vfs(|v| v.read_at(node, off, &mut staging[..want]));
        if n == 0 {
            return 0;
        }
        // SAFETY: `buf..buf+len` was validated above to lie inside the current
        // process's mapped EL0 window (live in TTBR0); EL1 may write EL0 pages.
        let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, n) };
        dst.copy_from_slice(&staging[..n]);
        handle.advance_read_off(n as u64);
        return n as u64;
    }
    match handle.kind {
        FileKind::Console | FileKind::Null => 0,
        // A bare read() on a netlink fd (no preceding sendto, or a fully-drained
        // dump) returns 0 (EOF). The talos dump path always uses recvfrom; this
        // arm only exists for match exhaustiveness.
        FileKind::Netlink => 0,
    }
}

/// Bound for a single `read(2)` copy out of a VFS `File` node. The baked
/// synthetic files are tiny; musl/std read them in larger chunks but a kernel
/// need only serve up to this many bytes per call (std loops to EOF).
const READ_CHUNK: usize = 4096;

/// # Safety
/// Reads the iovec array and each buffer; all ranges are validated first.
unsafe fn sys_writev(fd: u64, iov: u64, iovcnt: u64) -> u64 {
    let kind = match fd_kind(fd) {
        Some(k) => k,
        None => return EBADF,
    };
    if iovcnt == 0 {
        return 0;
    }
    let bytes_each = 16u64;
    if !user_range_ok(iov, iovcnt.saturating_mul(bytes_each)) {
        return EFAULT;
    }
    let to_console = matches!(kind, FileKind::Console);
    let mut total: u64 = 0;
    for i in 0..iovcnt {
        let ent = iov + i * bytes_each;
        // SAFETY: `ent` is within the validated iovec array.
        let base = unsafe { (ent as *const u64).read() };
        // SAFETY: the iov_len field follows iov_base.
        let l = unsafe { ((ent + 8) as *const u64).read() };
        if l == 0 {
            continue;
        }
        if !user_range_ok(base, l) {
            return EFAULT;
        }
        if to_console {
            // SAFETY: validated EL0 buffer, readable at EL1.
            let buf = unsafe { core::slice::from_raw_parts(base as *const u8, l as usize) };
            for &b in buf {
                crate::console::_print(format_args!("{}", b as char));
            }
        }
        // /dev/null discards but still counts the bytes as written.
        total += l;
    }
    total
}

/// Maximum NUL-terminated user path/string length we copy across the EL0
/// boundary. The pseudo-fs names + demo paths are tiny; a kernel need not honour
/// arbitrarily long names here.
const MAXP: usize = 64;

/// Copy a NUL-terminated string from user VA `addr` into `buf`, returning the
/// byte length copied (excluding the NUL), or `None` on a NULL pointer / an
/// unmapped byte. Each byte's address is validated before the read (the PAN
/// bracket below), never a raw deref of an unvalidated user pointer. Shared by
/// [`resolve_path`] and [`sys_mount`].
fn copy_user_cstr(addr: u64, buf: &mut [u8; MAXP]) -> Option<usize> {
    if addr == 0 {
        return None;
    }
    let mut n = 0usize;
    while n < MAXP {
        let a = addr + n as u64;
        // Validate each byte's address before reading it (PAN bracket below).
        if !user_range_ok(a, 1) {
            return None;
        }
        // SAFETY: `a` is a validated 1-byte EL0 address in the current
        // (live-TTBR0) process; aarch64 EL1 may read EL0 pages (no PAN trap on
        // this configuration — mirrors the other validated user reads here).
        let b = unsafe { (a as *const u8).read() };
        if b == 0 {
            break;
        }
        buf[n] = b;
        n += 1;
    }
    Some(n)
}

/// Map a `vfs::NodeKind` to the process-layer [`FileKind`] the
/// write/read fast paths match on. Console/Null project 1:1 (so the golden is
/// byte-identical); a regular `File` (Slice 3) projects to `Null` for now (its
/// bytes are served via `FileDesc.node` once `sys_read` routes File reads) and
/// a `Dir` is not openable as a stream here, so the caller treats it as a miss.
fn nodekind_to_filekind(k: crate::vfs::NodeKind) -> Option<(FileKind, bool)> {
    use crate::vfs::NodeKind;
    match k {
        NodeKind::Console => Some((FileKind::Console, false)),
        NodeKind::Null => Some((FileKind::Null, false)),
        // A regular file: carry its node id (the `true` flag) so `openat` records
        // it for Slice-3 reads; project to `Null` for the existing fast paths.
        NodeKind::File => Some((FileKind::Null, true)),
        NodeKind::Dir => None,
    }
}

/// Resolve a userspace path string against the in-RAM VFS tree to a
/// `(FileKind, node)` pair: a fixed-size copy of the NUL-terminated path is
/// taken from the current process's EL0 memory through the bounds-checked,
/// PAN-bracketed read path (never a raw deref), then walked. `/dev/console` and
/// `/dev/null` resolve to their `FileKind` (byte-identical to the old
/// two-string match); a regular `File` node resolves carrying `Some(node)` for
/// Slice-3 reads. A missing path, a directory, or a walk error → `None`
/// (→ `-ENOENT`).
fn resolve_path(path: u64) -> Option<(FileKind, Option<u32>)> {
    let mut buf = [0u8; MAXP];
    let n = copy_user_cstr(path, &mut buf)?;
    let id = process::with_vfs(|v| v.walk(&buf[..n]).ok())?;
    let kind = process::with_vfs(|v| v.kind(id));
    let (fk, is_file) = nodekind_to_filekind(kind)?;
    Some((fk, if is_file { Some(id) } else { None }))
}

/// `openat(dirfd, path, flags, mode)` — resolve `path` against the in-RAM VFS
/// and, on a hit, allocate the lowest free fd for a fresh open file
/// description. Unknown paths / directories → `-ENOENT`.
///
/// # Safety
/// Reads the NUL-terminated `path` from userspace via the bounds-checked,
/// PAN-bracketed [`resolve_path`]; never raw-derefs an unvalidated user pointer.
unsafe fn sys_openat(path: u64) -> u64 {
    const ENOENT: u64 = (-2i64) as u64;
    let (kind, node) = match resolve_path(path) {
        Some(r) => r,
        None => return ENOENT,
    };
    let handle = match node {
        Some(id) => FileDesc::file(kind, id),
        None => FileDesc::new(kind),
    };
    process::with_sched(|s| s.current().alloc_fd(handle)) as u64
}

/// `mount(source, target, fstype, flags, data)` — Slice 1 in-RAM tmpfs shim.
/// Copies the three NUL-terminated strings out of user memory, records the mount
/// in the global VFS (auto-creating the target dir so the tree stays walkable),
/// and returns **0** — the truthful "mount accepted" path a real privileged
/// kernel takes. This is what lets the real talos-init's MountPseudoFs phase
/// (`proc`/`sysfs`/`devtmpfs`/`tmpfs`/`devpts` → `/proc`,`/sys`,`/dev`,`/run`,
/// `/dev/pts`,`/dev/shm`) complete instead of aborting on `-ENOSYS`. `flags` is
/// recorded but not enforced; `data` (a4) is ignored.
///
/// # Safety
/// Reads the NUL-terminated `source`/`target`/`fstype` strings from userspace
/// via the bounds-checked, PAN-bracketed [`copy_user_cstr`]; never raw-derefs an
/// unvalidated user pointer.
unsafe fn sys_mount(source: u64, target: u64, fstype: u64, flags: u64) -> u64 {
    let mut sbuf = [0u8; MAXP];
    let mut tbuf = [0u8; MAXP];
    let mut fbuf = [0u8; MAXP];
    // A NULL/unmapped target is the only hard error (-EFAULT); source/fstype may
    // legitimately be NULL for some fstypes (treated as empty).
    let tn = match copy_user_cstr(target, &mut tbuf) {
        Some(n) => n,
        None => return EFAULT,
    };
    let sn = copy_user_cstr(source, &mut sbuf).unwrap_or(0);
    let fn_ = copy_user_cstr(fstype, &mut fbuf).unwrap_or(0);
    // Record-only: the pseudo set never errors against the pre-populated tree;
    // map any WalkErr to 0 too (record-only success convention, spec §3.3).
    process::with_vfs(|v| {
        let _ = v.do_mount(&sbuf[..sn], &tbuf[..tn], &fbuf[..fn_], flags);
    });
    0
}

/// `mkdirat(dirfd, path, mode)` — Slice 2 in-RAM tmpfs `mkdir`. Copies the
/// NUL-terminated `path` out of user memory and creates the directory (and any
/// missing parents) in the global VFS via `mkdir_p`, returning **0 idempotently**
/// (an already-existing directory is success, NOT `-EEXIST`, because talos's
/// `make_directory` ⇒ `create_dir_all` expects `mkdir -p` semantics and has no
/// `EEXIST` tolerance). This is what lets the real talos-init's SystemDirectories
/// phase (`/system`, `/system/state`, `/system/run`, `/var`, `/run`, `/tmp`)
/// complete instead of aborting on `-ENOSYS`. `dirfd`/`mode` are ignored.
/// A NULL/unmapped path → `-EFAULT`; a component that collides with a non-dir
/// node → `-ENOTDIR`.
///
/// # Safety
/// Reads the NUL-terminated `path` from userspace via the bounds-checked,
/// PAN-bracketed [`copy_user_cstr`]; never raw-derefs an unvalidated user pointer.
unsafe fn sys_mkdirat(path: u64) -> u64 {
    const ENOTDIR: u64 = (-20i64) as u64;
    let mut buf = [0u8; MAXP];
    let n = match copy_user_cstr(path, &mut buf) {
        Some(n) => n,
        None => return EFAULT,
    };
    process::with_vfs(|v| match v.mkdir_p(&buf[..n]) {
        Ok(_) => 0,
        Err(_) => ENOTDIR,
    })
}

/// `close(fd)` — drop the fd-table slot. Absent fd → `-EBADF`, else 0.
fn sys_close(fd: u64) -> u64 {
    let ok = process::with_sched(|s| s.current().close_fd(fd as i64));
    if ok {
        0
    } else {
        EBADF
    }
}

// ---------------------------------------------------------------------------
// M2 network slice: AF_NETLINK / RTM_GETLINK link-status dump
// ---------------------------------------------------------------------------

use crate::netlink as nl;

/// Linux `AF_NETLINK` socket domain.
const AF_NETLINK: u64 = 16;
/// Max bytes we copy from a `sendto` send buffer. talos's `build_dump_links`
/// request is 32 bytes; this cap bounds the in-kernel staging copy.
const NL_SEND_CAP: usize = 256;

/// Monotonic netlink port-id allocator. Real Linux assigns each `AF_NETLINK`
/// socket a unique nonzero port id (echoed as `nlmsg_pid`); the talos parser
/// does not check it but we hand out a nonzero value for forward-compat. Starts
/// at 1; only the single-core trap path increments it.
static NL_NEXT_PORT: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(1);

fn next_nl_port() -> u32 {
    let p = NL_NEXT_PORT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    if p == 0 {
        // Wrapped past u32::MAX back to 0; skip the reserved kernel port 0.
        NL_NEXT_PORT.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
    } else {
        p
    }
}

/// `socket(domain, type, protocol)` — M2 network slice. Only an `AF_NETLINK`
/// socket is modeled: it allocates a fresh `FileKind::Netlink` fd (with a unique
/// nonzero port id) and returns the fd. Every other domain → `-ENOSYS`, which is
/// EXACTLY the pre-slice behavior (the `_ => ENOSYS` fallthrough): talos's
/// `link_up("lo")`/`if_nametoindex` open an `AF_INET`/`SOCK_DGRAM` socket and
/// rely on `privileged_op_skip_reason(ENOSYS=38)` (`boot.rs:614`) to SKIP and
/// continue — returning `-EAFNOSUPPORT(97)` here would break that tolerance
/// (97 is not a skip reason), hard-failing `link_up` before the netlink dump.
/// So we keep `-ENOSYS` for non-netlink domains and only add AF_NETLINK support.
/// `type`/`protocol` are accepted as talos passes them (`SOCK_RAW|SOCK_CLOEXEC`,
/// `NETLINK_ROUTE`); we don't gate on them since only the route dump uses this.
fn sys_socket(domain: u64, _ty: u64, _protocol: u64) -> u64 {
    if domain != AF_NETLINK {
        return ENOSYS;
    }
    let handle = process::FileDesc::netlink(next_nl_port());
    process::with_sched(|s| s.current().alloc_fd(handle)) as u64
}

/// `bind(fd, addr, addrlen)` — M2 network slice. Validates `fd` is a netlink
/// socket and that `addr` is a well-formed 12-byte `sockaddr_nl`
/// (`nl_family == AF_NETLINK`); the port was already assigned at `socket` time
/// (`nl_pid == 0` means "kernel assigns", which we did). Returns 0 on success.
///
/// # Safety
/// Reads the 12-byte `sockaddr_nl` from userspace after bounds-validating the
/// range; never raw-derefs an unvalidated user pointer.
unsafe fn sys_bind(fd: u64, addr: u64, addrlen: u64) -> u64 {
    let handle = match process::with_sched(|s| s.current().fd_handle(fd as i64)) {
        Some(h) => h,
        None => return EBADF,
    };
    if handle.netlink.is_none() {
        return EINVAL;
    }
    if (addrlen as usize) < nl::SOCKADDR_NL_LEN || !user_range_ok(addr, nl::SOCKADDR_NL_LEN as u64) {
        return EINVAL;
    }
    let mut sa = [0u8; nl::SOCKADDR_NL_LEN];
    // SAFETY: `[addr, addr+12)` validated above to lie inside the current
    // process's mapped EL0 window (live in TTBR0), which EL1 may read.
    let src = unsafe { core::slice::from_raw_parts(addr as *const u8, nl::SOCKADDR_NL_LEN) };
    sa.copy_from_slice(src);
    if !nl::validate_sockaddr_nl(&sa) {
        return EINVAL;
    }
    0
}

/// `sendto(fd, buf, len, flags, addr, addrlen)` — M2 network slice. Copies the
/// flat send buffer out of user memory, parses it as a netlink request
/// (`RTM_GETLINK` dump), and ARMS the per-fd response (a single `NLMSG_DONE`
/// echoing the request seq, with this socket's port as `nlmsg_pid`). Resets the
/// drain cursor so the following `recvfrom` reads the response from the start.
/// Returns `len` (bytes "sent"). The dest `addr`/`addrlen` are the kernel
/// address (pid 0) and are ignored.
///
/// # Safety
/// Reads the flat `[buf, buf+min(len,cap))` send buffer from userspace after
/// bounds-validating the range; never raw-derefs an unvalidated user pointer.
unsafe fn sys_sendto(fd: u64, buf: u64, len: u64) -> u64 {
    let handle = match process::with_sched(|s| s.current().fd_handle(fd as i64)) {
        Some(h) => h,
        None => return EBADF,
    };
    let nlfd = match handle.netlink.as_ref() {
        Some(n) => n,
        None => return EINVAL,
    };
    if len == 0 {
        return 0;
    }
    let copy_len = core::cmp::min(len as usize, NL_SEND_CAP);
    if !user_range_ok(buf, copy_len as u64) {
        return EFAULT;
    }
    let mut staging = [0u8; NL_SEND_CAP];
    // SAFETY: `[buf, buf+copy_len)` validated to lie inside the current process's
    // mapped EL0 window (live in TTBR0), readable at EL1.
    let src = unsafe { core::slice::from_raw_parts(buf as *const u8, copy_len) };
    staging[..copy_len].copy_from_slice(src);
    // Parse + build entirely in the pure, host-tested netlink.
    let req = match nl::parse_request(&staging[..copy_len]) {
        Some(r) => r,
        // A request we can't even frame: report it "sent" but arm nothing (the
        // recvfrom will then return 0). talos always sends a well-formed dump.
        None => return len,
    };
    {
        let mut resp = nlfd.response.lock();
        nl::build_link_dump_response(req, nlfd.port, &mut resp);
    }
    // Reset the drain cursor: the next recvfrom starts at the response head.
    handle.reset_read_off();
    len
}

/// `recvfrom(fd, buf, len, flags, addr, addrlen)` — M2 network slice. Drains the
/// armed dump response (built by the preceding `sendto`) into the user buffer
/// starting at the per-fd cursor, advances the cursor, and — if `addr` is
/// non-NULL — writes a `sockaddr_nl` (family=AF_NETLINK, pid=this socket's port)
/// into it (updating `*addrlen` to 12). Returns the byte count copied (0 once the
/// dump is fully drained = EOF). The flat datagram form musl's `recv` lowers to.
///
/// # Safety
/// Writes the response bytes (and the optional `sockaddr_nl`) into user buffers
/// after bounds-validating each range; never raw-derefs an unvalidated user
/// pointer.
unsafe fn sys_recvfrom(fd: u64, buf: u64, len: u64, addr: u64, addrlen: u64) -> u64 {
    let handle = match process::with_sched(|s| s.current().fd_handle(fd as i64)) {
        Some(h) => h,
        None => return EBADF,
    };
    let nlfd = match handle.netlink.as_ref() {
        Some(n) => n,
        None => return EINVAL,
    };
    if len != 0 && !user_range_ok(buf, len) {
        return EFAULT;
    }
    let off = handle.read_off() as usize;
    let mut staging = [0u8; NL_SEND_CAP];
    let n = {
        let resp = nlfd.response.lock();
        if off >= resp.len() {
            0
        } else {
            let avail = resp.len() - off;
            let n = core::cmp::min(avail, core::cmp::min(len as usize, NL_SEND_CAP));
            staging[..n].copy_from_slice(&resp[off..off + n]);
            n
        }
    };
    if n == 0 {
        return 0;
    }
    // SAFETY: `[buf, buf+n)` ⊆ `[buf, buf+len)` validated above to lie inside the
    // current process's mapped, writable EL0 window (live in TTBR0); EL1 may
    // write EL0 pages.
    let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, n) };
    dst.copy_from_slice(&staging[..n]);
    handle.advance_read_off(n as u64);
    if addr != 0 && addrlen != 0 {
        // SAFETY: validates `addr`/`addrlen` ranges itself before writing.
        unsafe { write_sockaddr_nl(addr, addrlen, nlfd.port) };
    }
    n as u64
}

/// Write a `sockaddr_nl{family=AF_NETLINK, pad=0, pid=port, groups=0}` (12 bytes)
/// into the user `addr`, and `*addrlen = 12`, iff both pointers validate. Best
/// effort: a too-small/invalid buffer is silently skipped (the datagram itself
/// already succeeded — matching Linux, which truncates rather than failing recv).
///
/// # Safety
/// Validates `addr`/`addrlen` before writing; never raw-derefs an unvalidated
/// user pointer.
unsafe fn write_sockaddr_nl(addr: u64, addrlen: u64, port: u32) {
    if !user_range_ok(addrlen, 4) {
        return;
    }
    // SAFETY: validated 4-byte EL0 int (the in/out addrlen).
    let cap = unsafe { (addrlen as *const u32).read() };
    if (cap as usize) < nl::SOCKADDR_NL_LEN || !user_range_ok(addr, nl::SOCKADDR_NL_LEN as u64) {
        return;
    }
    let mut sa = [0u8; nl::SOCKADDR_NL_LEN];
    sa[0..2].copy_from_slice(&(nl::AF_NETLINK).to_le_bytes()); // nl_family
    // nl_pad (2..4) stays 0.
    sa[4..8].copy_from_slice(&port.to_le_bytes()); // nl_pid
                                                   // nl_groups (8..12) stays 0.
    // SAFETY: `[addr, addr+12)` validated writable EL0; `[addrlen,addrlen+4)`
    // validated. EL1 may write EL0 pages.
    unsafe {
        let dst = core::slice::from_raw_parts_mut(addr as *mut u8, nl::SOCKADDR_NL_LEN);
        dst.copy_from_slice(&sa);
        (addrlen as *mut u32).write(nl::SOCKADDR_NL_LEN as u32);
    }
}

/// `dup3(oldfd, newfd, flags)` — duplicate `oldfd`'s description into `newfd`
/// (closing `newfd` first if open). Absent `oldfd` → `-EBADF`, else `newfd`.
fn sys_dup3(oldfd: u64, newfd: u64) -> u64 {
    match process::with_sched(|s| s.current().dup_to(oldfd as i64, newfd as i64)) {
        Some(fd) => fd as u64,
        None => EBADF,
    }
}

/// `brk(addr)` against the current process's heap cursor + address space.
fn sys_brk(addr: u64) -> u64 {
    process::with_sched(|s| {
        let floor = s.current().brk_cur;
        if addr == 0 {
            return floor as u64;
        }
        let new = addr as usize;
        if new < floor || new > MMAP_BASE {
            return floor as u64;
        }
        let old = floor;
        let cur = s.current();
        let mut va = old & !PAGE_MASK;
        while va < new {
            // SAFETY: in-window; ensures a zeroed RW frame in this space.
            unsafe { cur.space.map_page(va, PagePerm::ReadWrite) };
            va += PAGE_SIZE;
        }
        cur.brk_cur = new;
        new as u64
    })
}

/// Anonymous `mmap(_, len, _, _, fd, _)` against the current process. Only
/// anonymous mappings (fd == -1) are supported; bump-allocated from the window.
fn sys_mmap(len: u64, fd: u64) -> u64 {
    if len == 0 {
        return EINVAL;
    }
    if (fd as i64) != -1 {
        return MAP_FAILED;
    }
    let len = (len as usize + PAGE_MASK) & !PAGE_MASK;
    process::with_sched(|s| {
        let cur = s.current();
        let start = cur.mmap_cur;
        if start + len > MMAP_TOP {
            return MAP_FAILED;
        }
        cur.mmap_cur = start + len;
        let mut va = start;
        while va < start + len {
            // SAFETY: in-window; ensures a zeroed RW frame in this space.
            unsafe { cur.space.map_page(va, PagePerm::ReadWrite) };
            va += PAGE_SIZE;
        }
        start as u64
    })
}

// ---- POSIX clock ids (Linux `time.h`) -------------------------------------
const CLOCK_REALTIME: u64 = 0;
const CLOCK_MONOTONIC: u64 = 1;
const CLOCK_PROCESS_CPUTIME_ID: u64 = 2;
const CLOCK_THREAD_CPUTIME_ID: u64 = 3;
const CLOCK_MONOTONIC_RAW: u64 = 4;

/// Resolve a clockid to nanoseconds against the real timekeeper, or `None` for an
/// unknown clock (caller returns `-EINVAL`):
///   * MONOTONIC / MONOTONIC_RAW -> `mono_ns`
///   * REALTIME                  -> `mono_ns + WALLCLOCK_OFFSET_NS`
///   * PROCESS/THREAD_CPUTIME_ID -> `mono_ns` (single process, no CPU accounting)
fn clock_ns(clk: u64) -> Option<u64> {
    match clk {
        CLOCK_MONOTONIC | CLOCK_MONOTONIC_RAW | CLOCK_PROCESS_CPUTIME_ID
        | CLOCK_THREAD_CPUTIME_ID => Some(crate::timer::mono_ns()),
        CLOCK_REALTIME => Some(crate::timer::real_ns()),
        _ => None,
    }
}

/// `clock_gettime(clk, ts)` — honours `clk` (see [`clock_ns`]), reads the real
/// timekeeper, and writes a `timespec` (16 bytes) into the validated user buffer.
/// Unknown clock -> `-EINVAL`. The return value is the constant `0` on success
/// (the time goes into the user buffer, never the trace).
fn sys_clock_gettime(clk: u64, ts: u64) -> u64 {
    let ns = match clock_ns(clk) {
        Some(ns) => ns,
        None => return EINVAL,
    };
    if !user_range_ok(ts, 16) {
        return EFAULT;
    }
    let (secs, nsec) = user_layout::timekeep::ns_to_timespec(ns);
    // SAFETY: validated 16-byte RW user buffer.
    unsafe {
        (ts as *mut i64).write(secs);
        ((ts + 8) as *mut i64).write(nsec);
    }
    0
}

/// `gettimeofday(tv, _tz)` — REALTIME wall clock split into a `timeval`
/// (`tv_sec`, `tv_usec`) written to the validated 16-byte user buffer. NULL `tv`
/// is a harmless success.
fn sys_gettimeofday(tv: u64) -> u64 {
    if tv == 0 {
        return 0;
    }
    if !user_range_ok(tv, 16) {
        return EFAULT;
    }
    let (secs, usec) = user_layout::timekeep::ns_to_timeval(crate::timer::real_ns());
    // SAFETY: validated 16-byte RW user buffer.
    unsafe {
        (tv as *mut i64).write(secs);
        ((tv + 8) as *mut i64).write(usec);
    }
    0
}

fn sys_sysinfo(info: u64) -> u64 {
    const SYSINFO_SIZE: u64 = 112;
    if !user_range_ok(info, SYSINFO_SIZE) {
        return EFAULT;
    }
    // SAFETY: validated RW user buffer of SYSINFO_SIZE bytes.
    let out = unsafe { core::slice::from_raw_parts_mut(info as *mut u8, SYSINFO_SIZE as usize) };
    for b in out.iter_mut() {
        *b = 0;
    }
    out[0x60] = 1;
    0
}

// ---- WAVE 1: libc-init / process-info handlers ----------------------------
// Pure byte-layout math lives in `user_layout::procinfo`; these handlers do only
// the bounds-checked user copy via the existing PAN-bracketed write pattern.

/// The per-process file-creation mask. Single-process model (one umask for the
/// init chain), initialised to Linux's default `022`. A static is sufficient
/// here (the task permits it over a Process field) and keeps the change minimal.
static UMASK: core::sync::atomic::AtomicU32 =
    core::sync::atomic::AtomicU32::new(user_layout::procinfo::DEFAULT_UMASK);

/// `uname(buf)` — write the byte-exact `struct utsname` (six NUL-padded 65-byte
/// fields; `machine = "aarch64"`) into the validated user buffer. `-EFAULT` on a
/// bad pointer, else `0`.
fn sys_uname(buf: u64) -> u64 {
    let uts = user_layout::procinfo::build_utsname("aarch64");
    if !user_range_ok(buf, uts.len() as u64) {
        return EFAULT;
    }
    // SAFETY: validated RW user buffer of exactly `uts.len()` bytes in the
    // current (live-TTBR0) process; aarch64 EL1 may write EL0 pages.
    unsafe {
        core::ptr::copy_nonoverlapping(uts.as_ptr(), buf as *mut u8, uts.len());
    }
    0
}

/// `umask(mask)` — store `mask & 0777` as the new per-process mask and return the
/// previous value. Never faults.
fn sys_umask(mask: u64) -> u64 {
    let prev = UMASK.load(core::sync::atomic::Ordering::Relaxed);
    let (previous, stored) = user_layout::procinfo::umask_swap(prev, mask);
    UMASK.store(stored, core::sync::atomic::Ordering::Relaxed);
    previous as u64
}

/// `getrusage(_who, usage)` — write a zeroed `struct rusage` (so `ru_utime` /
/// `ru_stime` and every counter read as 0) into the validated user buffer and
/// return 0. We keep no per-process CPU accounting. `struct rusage` is 144 bytes
/// on Linux (two 16-byte timevals + 14 `long` counters).
fn sys_getrusage(usage: u64) -> u64 {
    const RUSAGE_SIZE: u64 = 144;
    if !user_range_ok(usage, RUSAGE_SIZE) {
        return EFAULT;
    }
    // SAFETY: validated RW user buffer of RUSAGE_SIZE bytes.
    let out = unsafe { core::slice::from_raw_parts_mut(usage as *mut u8, RUSAGE_SIZE as usize) };
    for b in out.iter_mut() {
        *b = 0;
    }
    0
}

/// `times(buf)` — write a zeroed `struct tms` (`tms_utime`/`tms_stime`/
/// `tms_cutime`/`tms_cstime`, four `clock_t` = 32 bytes) into the validated user
/// buffer (NULL is allowed) and return a monotonic tick count, per the Linux
/// ABI. We report monotonic milliseconds as the tick (USER_HZ-agnostic, strictly
/// non-decreasing, which is all `times(2)`'s return contract requires).
fn sys_times(buf: u64) -> u64 {
    if buf != 0 {
        const TMS_SIZE: u64 = 32;
        if !user_range_ok(buf, TMS_SIZE) {
            return EFAULT;
        }
        // SAFETY: validated RW user buffer of TMS_SIZE bytes.
        let out = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, TMS_SIZE as usize) };
        for b in out.iter_mut() {
            *b = 0;
        }
    }
    (crate::timer::mono_ns() / 1_000_000) as u64
}

/// `clock_getres(clk, res)` — for any clock this kernel recognises, write a
/// `timespec {tv_sec: 0, tv_nsec: 1}` (1 ns resolution) into the validated
/// 16-byte user buffer (NULL `res` is allowed) and return 0; unknown clock ->
/// `-EINVAL`.
fn sys_clock_getres(clk: u64, res: u64) -> u64 {
    let (secs, nsec) = match user_layout::procinfo::clock_getres(clock_ns(clk).is_some()) {
        Some(v) => v,
        None => return EINVAL,
    };
    if res == 0 {
        return 0;
    }
    if !user_range_ok(res, 16) {
        return EFAULT;
    }
    // SAFETY: validated 16-byte RW user buffer.
    unsafe {
        (res as *mut i64).write(secs);
        ((res + 8) as *mut i64).write(nsec);
    }
    0
}

/// `prctl(option, arg2, …)` — minimal: `PR_SET_NAME` accepts the name and returns
/// 0; `PR_GET_NAME` zeroes the 16-byte user buffer at `arg2` and returns 0; every
/// other option succeeds with 0 (no Process state is added for prctl).
fn sys_prctl(option: u64, arg2: u64) -> u64 {
    match option {
        PR_SET_NAME => 0,
        PR_GET_NAME => {
            if !user_range_ok(arg2, TASK_COMM_LEN) {
                return EFAULT;
            }
            // SAFETY: validated RW user buffer of TASK_COMM_LEN bytes.
            let out =
                unsafe { core::slice::from_raw_parts_mut(arg2 as *mut u8, TASK_COMM_LEN as usize) };
            for b in out.iter_mut() {
                *b = 0;
            }
            0
        }
        _ => 0,
    }
}

fn sys_getcwd(buf: u64, size: u64) -> u64 {
    const CWD: &[u8] = b"/\0";
    if size < CWD.len() as u64 {
        return (-34i64) as u64;
    }
    if !user_range_ok(buf, CWD.len() as u64) {
        return EFAULT;
    }
    // SAFETY: validated RW user buffer large enough for CWD.
    unsafe {
        core::ptr::copy_nonoverlapping(CWD.as_ptr(), buf as *mut u8, CWD.len());
    }
    CWD.len() as u64
}

fn sys_fstat(stbuf: u64) -> u64 {
    const STAT_SIZE: u64 = 128;
    if !user_range_ok(stbuf, STAT_SIZE) {
        return EFAULT;
    }
    // SAFETY: validated RW user buffer of STAT_SIZE bytes.
    let out = unsafe { core::slice::from_raw_parts_mut(stbuf as *mut u8, STAT_SIZE as usize) };
    for b in out.iter_mut() {
        *b = 0;
    }
    const S_IFCHR: u32 = 0o020000;
    let mode = (S_IFCHR | 0o666).to_le_bytes();
    out[16..20].copy_from_slice(&mode);
    0
}

fn read_timespec_cycles(ts: u64) -> Result<u64, u64> {
    if !user_range_ok(ts, 16) {
        return Err(EFAULT);
    }
    // SAFETY: validated 16-byte readable user buffer.
    let (tv_sec, tv_nsec) = unsafe {
        let s = (ts as *const i64).read();
        let n = ((ts + 8) as *const i64).read();
        (s, n)
    };
    match timespec_to_cycles(tv_sec, tv_nsec, crate::timer::frequency()) {
        SleepCycles::Wait(c) => Ok(c),
        SleepCycles::Invalid => Err(EINVAL),
    }
}

fn sys_nanosleep(req: u64, rem: u64) -> u64 {
    let cycles = match read_timespec_cycles(req) {
        Ok(c) => c,
        Err(e) => return e,
    };
    if cycles > 0 {
        let deadline = deadline_after(crate::timer::now(), cycles);
        crate::timer::sleep_until(deadline);
    }
    if rem != 0 && user_range_ok(rem, 16) {
        // SAFETY: validated 16-byte RW user buffer.
        unsafe {
            (rem as *mut i64).write(0);
            ((rem + 8) as *mut i64).write(0);
        }
    }
    0
}

fn sys_ppoll(fds: u64, nfds: u64, timeout_ts: u64) -> u64 {
    if fds == 0 || nfds == 0 {
        if timeout_ts == 0 {
            return 0;
        }
        let cycles = match read_timespec_cycles(timeout_ts) {
            Ok(c) => c,
            Err(e) => return e,
        };
        if cycles > 0 {
            let deadline = deadline_after(crate::timer::now(), cycles);
            crate::timer::sleep_until(deadline);
        }
        return 0;
    }
    const POLLFD_SIZE: u64 = 8;
    let total = match nfds.checked_mul(POLLFD_SIZE) {
        Some(t) => t,
        None => return EINVAL,
    };
    if !user_range_ok(fds, total) {
        return EFAULT;
    }
    let mut ready: u64 = 0;
    for i in 0..nfds {
        let ent = fds + i * POLLFD_SIZE;
        // SAFETY: `ent` lies within the validated pollfd array.
        let fd = unsafe { (ent as *const i32).read() };
        // SAFETY: the `events` field follows `fd` at offset 4.
        let events = unsafe { ((ent + 4) as *const i16).read() };
        let revents: i16 = if fd < 0 { 0 } else { events };
        // SAFETY: `revents` is the last field of the validated pollfd entry.
        unsafe {
            ((ent + 6) as *mut i16).write(revents);
        }
        if revents != 0 {
            ready += 1;
        }
    }
    ready
}

fn sys_getrandom(buf: u64, len: u64, _flags: u64) -> u64 {
    if len == 0 {
        return 0;
    }
    if !user_range_ok(buf, len) {
        return EFAULT;
    }
    let mut seed = crate::timer::now() ^ 0x9e37_79b9_7f4a_7c15;
    // SAFETY: validated RW user buffer.
    let out = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len as usize) };
    for b in out.iter_mut() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        *b = seed as u8;
    }
    len
}

fn sys_sched_getaffinity(_pid: u64, cpusetsize: u64, mask: u64) -> u64 {
    if cpusetsize < 8 {
        return EINVAL;
    }
    if !user_range_ok(mask, cpusetsize) {
        return EFAULT;
    }
    // SAFETY: validated RW user buffer of `cpusetsize` bytes.
    let out = unsafe { core::slice::from_raw_parts_mut(mask as *mut u8, cpusetsize as usize) };
    for b in out.iter_mut() {
        *b = 0;
    }
    out[0] = 0b1;
    8
}

// ---- POSIX signals --------------------------------------------------------
//
// All the pure bit/layout math lives in `user_layout::signal` (`sig::*`); the
// code here is the thin unsafe Frame layer: it reads/writes the userspace
// `struct sigaction`/`sigset_t`/`sigaltstack` through the bounds-checked EL0
// alias (aarch64 EL1 may read/write the live-TTBR0 process's EL0 pages after a
// `user_range_ok`), mutates the current process's `SignalState`, and — at the
// return-to-user boundary — builds a signal frame on the user stack and rewrites
// the live trap frame to enter the handler.

/// userspace `struct sigaction` field offsets (≥ 32 bytes; spec §2).
const SA_HANDLER_OFF: u64 = 0;
const SA_FLAGS_OFF: u64 = 8;
const SA_RESTORER_OFF: u64 = 16;
const SA_MASK_OFF: u64 = 24;
const SA_STRUCT_SIZE: u64 = 32;

/// `rt_sigaction(signo, const act*, oldact*, sigsetsize)`.
///
/// # Safety
/// Reads/writes the user `struct sigaction` through the validated EL0 alias.
unsafe fn sys_rt_sigaction(signo: u64, act: u64, oldact: u64, sigsetsize: u64) -> u64 {
    if sigsetsize != sig::SIGSET_BYTES as u64 {
        return EINVAL;
    }
    if signo < 1 || signo > sig::NSIG as u64 {
        return EINVAL;
    }
    let signo32 = signo as u32;
    // SIGKILL/SIGSTOP dispositions are immutable.
    if act != 0 && (signo32 == sig::SIGKILL || signo32 == sig::SIGSTOP) {
        return EINVAL;
    }

    // Write the existing disposition to `oldact` first (if requested).
    if oldact != 0 {
        if !user_range_ok(oldact, SA_STRUCT_SIZE) {
            return EFAULT;
        }
        let old = process::with_sched(|s| s.current().signals.action(signo32));
        // SAFETY: validated 32-byte RW user buffer in the current EL0 space.
        unsafe {
            ((oldact + SA_HANDLER_OFF) as *mut u64).write(old.handler);
            ((oldact + SA_FLAGS_OFF) as *mut u64).write(old.flags);
            ((oldact + SA_RESTORER_OFF) as *mut u64).write(old.restorer);
            ((oldact + SA_MASK_OFF) as *mut u64).write(old.mask);
        }
    }

    // Install the new disposition (if provided).
    if act != 0 {
        if !user_range_ok(act, SA_STRUCT_SIZE) {
            return EFAULT;
        }
        // SAFETY: validated 32-byte readable user buffer in the current EL0 space.
        let new = unsafe {
            sig::SigAction {
                handler: ((act + SA_HANDLER_OFF) as *const u64).read(),
                flags: ((act + SA_FLAGS_OFF) as *const u64).read(),
                restorer: ((act + SA_RESTORER_OFF) as *const u64).read(),
                mask: ((act + SA_MASK_OFF) as *const u64).read(),
            }
        };
        process::with_sched(|s| s.current().signals.actions[signo as usize] = new);
    }
    0
}

/// `rt_sigprocmask(how, const set*, oldset*, sigsetsize)`.
///
/// # Safety
/// Reads/writes the user `sigset_t` through the validated EL0 alias.
unsafe fn sys_rt_sigprocmask(how: u64, set: u64, oldset: u64, sigsetsize: u64) -> u64 {
    if sigsetsize != sig::SIGSET_BYTES as u64 {
        return EINVAL;
    }
    // Write the current mask to `oldset` first.
    if oldset != 0 {
        if !user_range_ok(oldset, sig::SIGSET_BYTES as u64) {
            return EFAULT;
        }
        let cur = process::with_sched(|s| s.current().signals.blocked);
        // SAFETY: validated 8-byte RW user buffer.
        unsafe { (oldset as *mut u64).write(cur) };
    }
    if set != 0 {
        if !user_range_ok(set, sig::SIGSET_BYTES as u64) {
            return EFAULT;
        }
        // SAFETY: validated 8-byte readable user buffer.
        let arg = unsafe { (set as *const u64).read() };
        process::with_sched(|s| {
            let cur = s.current().signals.blocked;
            let next = match how {
                x if x == sig::SIG_BLOCK => cur | arg,
                x if x == sig::SIG_UNBLOCK => cur & !arg,
                x if x == sig::SIG_SETMASK => arg,
                _ => cur, // unknown `how`: leave unchanged
            };
            // SIGKILL/SIGSTOP can never be blocked.
            s.current().signals.blocked =
                sig::Sigset(next).block_unblockable_cleared().0;
        });
    }
    0
}

/// `rt_sigpending(set*, sigsetsize)`.
///
/// # Safety
/// Writes the user `sigset_t` through the validated EL0 alias.
unsafe fn sys_rt_sigpending(set: u64, sigsetsize: u64) -> u64 {
    if sigsetsize != sig::SIGSET_BYTES as u64 {
        return EINVAL;
    }
    if set == 0 || !user_range_ok(set, sig::SIGSET_BYTES as u64) {
        return EFAULT;
    }
    let pending = process::with_sched(|s| s.current().signals.pending);
    // SAFETY: validated 8-byte RW user buffer.
    unsafe { (set as *mut u64).write(pending) };
    0
}

/// `sigaltstack(const ss*, old_ss*)`. struct sigaltstack { ss_sp@0, ss_flags@8
/// (int, padded), ss_size@16 } = 24 bytes.
///
/// # Safety
/// Reads/writes the user `struct sigaltstack` through the validated EL0 alias.
unsafe fn sys_sigaltstack(ss: u64, old_ss: u64) -> u64 {
    const SS_SIZE: u64 = 24;
    if old_ss != 0 {
        if !user_range_ok(old_ss, SS_SIZE) {
            return EFAULT;
        }
        let (sp, size, on) = process::with_sched(|s| {
            let p = s.current();
            (p.signals.altstack_sp, p.signals.altstack_size, p.signals.on_altstack)
        });
        let flags: u32 = if sp == 0 {
            sig::SS_DISABLE
        } else if on {
            1 /* SS_ONSTACK */
        } else {
            0
        };
        // SAFETY: validated 24-byte RW user buffer.
        unsafe {
            (old_ss as *mut u64).write(sp);
            ((old_ss + 8) as *mut u32).write(flags);
            ((old_ss + 16) as *mut u64).write(size);
        }
    }
    if ss != 0 {
        if !user_range_ok(ss, SS_SIZE) {
            return EFAULT;
        }
        // SAFETY: validated 24-byte readable user buffer.
        let (sp, flags, size) = unsafe {
            (
                (ss as *const u64).read(),
                ((ss + 8) as *const u32).read(),
                ((ss + 16) as *const u64).read(),
            )
        };
        if flags & sig::SS_DISABLE != 0 {
            process::with_sched(|s| {
                let p = s.current();
                p.signals.altstack_sp = 0;
                p.signals.altstack_size = 0;
            });
        } else {
            if size < sig::MINSIGSTKSZ {
                return EINVAL;
            }
            process::with_sched(|s| {
                let p = s.current();
                p.signals.altstack_sp = sp;
                p.signals.altstack_size = size;
            });
        }
    }
    0
}

/// `kill(pid, sig)` / `tkill(tid, sig)` — single-thread model (tid == pid).
fn sys_kill(pid: u64, signo: u64) -> u64 {
    let target = pid as i64;
    // `pid <= 0` selects process groups / broadcast — out of v1 scope.
    if target <= 0 {
        return EINVAL;
    }
    sys_signal_one(target as u32, signo)
}

/// `tgkill(tgid, tid, sig)` — single-thread model (tgid == tid == pid).
fn sys_tgkill(tgid: u64, tid: u64, signo: u64) -> u64 {
    let _ = tgid;
    let target = tid as i64;
    if target <= 0 {
        return EINVAL;
    }
    sys_signal_one(target as u32, signo)
}

/// Shared core for kill/tkill/tgkill: `sig == 0` is an existence probe
/// (`ESRCH` if the target is gone); a valid `sig` is posted bit-only.
fn sys_signal_one(target: u32, signo: u64) -> u64 {
    const ESRCH: u64 = (-3i64) as u64;
    let exists = process::with_sched(|s| s.pid_exists(target));
    if !exists {
        return ESRCH;
    }
    if signo == 0 {
        return 0; // existence probe only
    }
    if signo < 1 || signo > sig::NSIG as u64 {
        return EINVAL;
    }
    process::with_sched(|s| s.post_signal(target, signo as u32));
    0
}

// ---- Signal delivery on return-to-user ------------------------------------

/// Build a signal frame on the user stack and rewrite the live trap frame so the
/// trampoline's `eret` lands in the handler — for the lowest-numbered pending,
/// unblocked signal of the **now-current** process. Called as the last step
/// before returning to EL0. At most one signal is delivered per return (the next
/// delivers on the next return, e.g. after the handler's `rt_sigreturn`).
///
/// SIG_DFL/SIG_IGN dispositions take the §5 default action (Ignore: clear the
/// bit; Terminate: kill the process) instead of building a frame.
///
/// # Safety
/// `f` is the live trap frame; the current process's space is the live TTBR0.
/// All user writes are bounds-checked (`user_range_ok`) before the EL0-alias
/// write; an overflowing frame terminates the process (never a raw OOB write).
unsafe fn deliver_pending_signals(f: &mut TrapFrame) {
    let signo = match process::with_sched(|s| s.current().signals.next_deliverable()) {
        Some(s) => s,
        None => return,
    };
    let action = process::with_sched(|s| s.current().signals.action(signo));

    // No real handler: take the default action and return (so a later pending
    // signal can deliver on the next return).
    if !action.has_handler() {
        if action.is_ignore() {
            process::with_sched(|s| s.current().signals.clear_pending(signo));
            return;
        }
        match sig::default_action(signo) {
            sig::DefaultAction::Ignore
            | sig::DefaultAction::Stop
            | sig::DefaultAction::Continue => {
                // Discarded in v1 (job control out of scope).
                process::with_sched(|s| s.current().signals.clear_pending(signo));
            }
            sig::DefaultAction::Terminate => {
                // SAFETY: `f` is the live frame; terminate + switch mirror exit.
                unsafe { terminate_current_by_signal(f, signo) };
            }
        }
        return;
    }

    // We require an explicit SA_RESTORER (musl/glibc always supply one). Without
    // it we cannot return from the handler safely; treat as a fatal signal.
    if action.flags & sig::SA_RESTORER == 0 || action.restorer == 0 {
        // SAFETY: `f` is the live frame.
        unsafe { terminate_current_by_signal(f, signo) };
        return;
    }

    // Choose the stack: the alternate stack iff requested + configured + not
    // already running on it, else the interrupted SP.
    let (alt_sp, alt_size, on_alt) = process::with_sched(|s| {
        let p = s.current();
        (p.signals.altstack_sp, p.signals.altstack_size, p.signals.on_altstack)
    });
    let use_alt =
        action.flags & sig::SA_ONSTACK != 0 && alt_sp != 0 && !on_alt;
    // The trap stub does NOT stash SP_EL0 into `f.sp` (it stores xzr there), so
    // read the live EL0 stack pointer directly. This is also the value we must
    // save into the sigcontext for rt_sigreturn to restore.
    // SAFETY: reading SP_EL0 of the interrupted EL0 thread mid-trap.
    let user_sp = unsafe { read_sp_el0() };
    let stack_top = if use_alt { alt_sp + alt_size } else { user_sp };

    let frame_base = sig::aa_frame_base(stack_top);
    // Bounds-check the WHOLE frame before any write; SIGSEGV-terminate on
    // overflow rather than a raw out-of-window write.
    if !user_range_ok(frame_base, sig::AA_FRAME_SIZE) {
        // SAFETY: `f` is the live frame.
        unsafe { terminate_current_by_signal(f, sig::SIGSEGV) };
        return;
    }

    let old_blocked = process::with_sched(|s| s.current().signals.blocked);
    let uc_base = frame_base + sig::AA_UC_OFF;
    let mc_base = uc_base + sig::AA_UC_MCONTEXT_OFF;

    // SAFETY: every target address lies inside the validated `[frame_base,
    // frame_base+AA_FRAME_SIZE)` window in the current process's live EL0 space.
    unsafe {
        // ---- siginfo: si_signo, si_code (minimal; SI_KERNEL=0x80 not used) ----
        let info = frame_base + sig::AA_SIGINFO_OFF;
        // Zero the siginfo region first.
        core::ptr::write_bytes(info as *mut u8, 0, sig::AA_SIGINFO_SIZE as usize);
        (info as *mut u32).write(signo); // si_signo
        // si_code at +8 left 0 (SI_USER).

        // ---- ucontext: flags/link/stack zeroed; uc_sigmask = old blocked ----
        core::ptr::write_bytes(uc_base as *mut u8, 0, sig::AA_UC_MCONTEXT_OFF as usize);
        ((uc_base + sig::AA_UC_STACK_OFF) as *mut u64).write(alt_sp);
        ((uc_base + sig::AA_UC_STACK_OFF + 16) as *mut u64).write(alt_size);
        ((uc_base + sig::AA_UC_SIGMASK_OFF) as *mut u64).write(old_blocked);

        // ---- sigcontext (uc_mcontext): save the interrupted register state ----
        core::ptr::write_bytes(
            mc_base as *mut u8,
            0,
            (sig::AA_SC_PSTATE_OFF + 8 + sig::AA_RESERVED_TAIL) as usize,
        );
        ((mc_base + sig::AA_SC_FAULT_OFF) as *mut u64).write(0);
        let regs_ptr = (mc_base + sig::AA_SC_REGS_OFF) as *mut u64;
        for i in 0..31 {
            regs_ptr.add(i).write(f.regs[i]);
        }
        ((mc_base + sig::AA_SC_SP_OFF) as *mut u64).write(user_sp);
        ((mc_base + sig::AA_SC_PC_OFF) as *mut u64).write(f.elr);
        ((mc_base + sig::AA_SC_PSTATE_OFF) as *mut u64).write(f.spsr);
    }

    // Block the delivered signal + its sa_mask for the handler's duration
    // (restored from uc_sigmask by rt_sigreturn), and clear its pending bit.
    process::with_sched(|s| {
        let p = s.current();
        let blocked = old_blocked | action.mask | sig::sig_bit(signo);
        p.signals.blocked = sig::Sigset(blocked).block_unblockable_cleared().0;
        p.signals.clear_pending(signo);
        if use_alt {
            p.signals.on_altstack = true;
        }
    });

    // Redirect into the handler: x0=signo, x1=siginfo*, x2=ucontext*,
    // x30(LR)=restorer, elr=handler, EL0t SPSR. The handler runs on the signal
    // frame, so set SP_EL0 = frame_base directly (the trap stub never restores
    // `f.sp`, it leaves SP_EL0 untouched across the eret).
    f.regs[0] = signo as u64;
    f.regs[1] = frame_base + sig::AA_SIGINFO_OFF;
    f.regs[2] = uc_base;
    f.regs[30] = action.restorer;
    f.elr = action.handler;
    f.spsr = SPSR_EL0T;
    // SAFETY: `frame_base` is a validated 16-aligned EL0 address; setting SP_EL0
    // makes the eret land the handler on the freshly-built signal frame.
    unsafe { write_sp_el0(frame_base) };
}

/// Read the interrupted EL0 thread's stack pointer (`SP_EL0`).
///
/// # Safety
/// Valid only mid-trap from EL0 (the trap stub does not stash SP_EL0, so the
/// register still holds the EL0 value).
#[inline(always)]
unsafe fn read_sp_el0() -> u64 {
    let sp: u64;
    // SAFETY: reads a system register; no memory access.
    unsafe {
        core::arch::asm!("mrs {x}, sp_el0", x = out(reg) sp, options(nostack, nomem));
    }
    sp
}

/// Set the EL0 thread's stack pointer (`SP_EL0`) so the next `eret` resumes EL0
/// on that stack.
///
/// # Safety
/// `sp` must be a valid EL0 stack pointer; takes effect on the next eret to EL0.
#[inline(always)]
unsafe fn write_sp_el0(sp: u64) {
    // SAFETY: writes a system register; no memory access.
    unsafe {
        core::arch::asm!("msr sp_el0, {x}", x = in(reg) sp, options(nostack, nomem));
    }
}

/// `rt_sigreturn()` — restore the pre-signal context from the frame the
/// delivery path built (or a forged one, which we validate).
///
/// # Safety
/// `f` is the live trap frame; the frame base is read from the live `SP_EL0`
/// (the restorer entered rt_sigreturn with SP_EL0 == frame_base, the value we
/// set in the delivery path) and fully bounds-checked before any read. The
/// restored PSTATE is forced to `SPSR_EL0T` (never trust the frame's status
/// bits).
unsafe fn sys_rt_sigreturn(f: &mut TrapFrame) -> SyscallOutcome {
    // SAFETY: reading SP_EL0 of the interrupted EL0 (handler/restorer) thread.
    let frame_base = unsafe { read_sp_el0() };
    if !user_range_ok(frame_base, sig::AA_FRAME_SIZE) {
        // Forged / corrupt SP: do not trust it. SIGSEGV-terminate.
        // SAFETY: `f` is the live frame.
        unsafe { terminate_current_by_signal(f, sig::SIGSEGV) };
        return SyscallOutcome::Resume;
    }
    let uc_base = frame_base + sig::AA_UC_OFF;
    let mc_base = uc_base + sig::AA_UC_MCONTEXT_OFF;

    // SAFETY: `[frame_base, frame_base+AA_FRAME_SIZE)` validated in the live EL0
    // space; read back the saved register state + uc_sigmask.
    let (regs, sp, pc, saved_mask) = unsafe {
        let regs_ptr = (mc_base + sig::AA_SC_REGS_OFF) as *const u64;
        let mut regs = [0u64; 31];
        for i in 0..31 {
            regs[i] = regs_ptr.add(i).read();
        }
        let sp = ((mc_base + sig::AA_SC_SP_OFF) as *const u64).read();
        let pc = ((mc_base + sig::AA_SC_PC_OFF) as *const u64).read();
        let mask = ((uc_base + sig::AA_UC_SIGMASK_OFF) as *const u64).read();
        (regs, sp, pc, mask)
    };

    // Restore GPRs / PC into the live frame; RE-IMPOSE a clean EL0t PSTATE
    // (never copy attacker-controlled SPSR from the frame). The saved SP is
    // restored into SP_EL0 directly (the eret does not reload `f.sp`).
    f.regs = regs;
    f.elr = pc;
    f.spsr = SPSR_EL0T;
    // SAFETY: `sp` is the pre-signal EL0 stack pointer saved by the delivery
    // path; restoring it into SP_EL0 resumes the interrupted code on its stack.
    unsafe { write_sp_el0(sp) };

    // Restore the blocked mask (un-blocks the handled signal), force-clear
    // KILL/STOP, and leave the altstack.
    process::with_sched(|s| {
        let p = s.current();
        p.signals.blocked = sig::Sigset(saved_mask).block_unblockable_cleared().0;
        p.signals.on_altstack = false;
    });

    // Resume the restored context. No trace line, no x0 overwrite — the restored
    // x0 is part of the pre-signal context. The §3 hook runs again on this
    // return, so a newly-unblocked pending signal delivers next.
    SyscallOutcome::Resume
}

/// Terminate the current process because of fatal signal `sig` (default action
/// Terminate, or a frame-overflow SIGSEGV). Mirrors `sys_exit`'s epilogue.
///
/// # Safety
/// `f` is the live trap frame; switches it to the next process or diverts back
/// to the kernel when the workload is done.
unsafe fn terminate_current_by_signal(f: &mut TrapFrame, signo: u32) {
    let pid = current_pid();
    crate::kprintln!("[pid {}] terminated by signal {}", pid, signo);

    let reaped = process::with_sched(|s| {
        s.terminate_current_by_signal(signo);
        s.complete_waits()
    });

    // P4·SMP·S4d: the current process is now `Zombie` + the `PROCS` lock is
    // released, so a remote reap may free its L1/frames at any moment; park on the
    // kernel identity map before anything else runs on the dying TTBR0 (see the
    // identical guard + rationale in `sys_exit`).
    // SAFETY: EL1, this CPU; installs the kernel L1 (upper GiBs keep EL1 mapped).
    unsafe { park_on_kernel_map() };

    // Switch to the next process runnable on THIS CPU; on success drop the reaped
    // children (off the dying TTBR0). SAFETY: `f` is the live frame.
    let switched = unsafe { process::schedule(f) };
    if switched {
        // P4·SMP·S4c: shoot down other CPUs before freeing the reaped children's
        // page tables/frames (same rationale as `sys_exit`). No-op on 1-vCPU.
        if !reaped.is_empty() {
            crate::shootdown::request_and_wait_others();
        }
        drop(reaped);
        return;
    }
    // Nothing runnable for THIS CPU — still on the dying TTBR0, so forget (leak)
    // the reaped procs and decide: BSP+empty-table diverts the eret to power-off;
    // otherwise (re-)enter this CPU's EL1 idle loop (idle_or_finish diverges).
    core::mem::forget(reaped);
    // SAFETY: EL1, schedule returned false.
    unsafe { idle_or_finish(f) };
}

// ---- process-model syscalls -----------------------------------------------

/// `clone(flags, newsp, ...)` — we support exactly the `fork()` ABI:
/// `flags == SIGCHLD` (optionally with no extra sharing) and `newsp == 0`.
/// Anything requesting a shared VM (threads) is rejected with -EINVAL.
///
/// The parent's live trap frame `f` is duplicated for the child (so the child
/// resumes at the same post-SVC PC) with x0 forced to 0; the parent's return
/// (the child pid) is written by the caller into its own x0.
fn sys_clone(f: &TrapFrame, flags: u64, newsp: u64) -> u64 {
    // Threads (CLONE_VM) and stack-supplying clones are not supported yet.
    if flags & CLONE_VM != 0 || newsp != 0 {
        return EINVAL;
    }
    // Require the SIGCHLD exit-signal that fork() uses; ignore the other
    // bookkeeping flags musl may OR in (CLONE_CHILD_*TID etc. need no action
    // here because we do not implement the tid write-back).
    if flags & 0xff != SIGCHLD {
        return EINVAL;
    }
    // The trap stub does not stash SP_EL0, so read the parent's live EL0 stack
    // pointer here and hand it to the child (its copied stack is at the same VA).
    let sp_el0: u64;
    // SAFETY: reading SP_EL0 of the calling EL0 thread mid-SVC.
    unsafe {
        core::arch::asm!("mrs {x}, sp_el0", x = out(reg) sp_el0, options(nostack, nomem));
    }
    let child_pid = process::with_sched(|s| s.fork_current(f, sp_el0));
    // P4·SMP·S4c: `cow_clone` write-protected the PARENT's writable leaves in
    // place. The PROCS lock is now RELEASED (we are past `with_sched`, DAIF.I
    // restored), so shoot down the other online CPUs — any CPU running this
    // parent must drop its stale writable TLB entries before it writes again
    // (else it would write the now-shared page without taking the COW fault).
    // No-op on 1-vCPU.
    crate::shootdown::request_and_wait_others();
    crate::kprintln!(
        "[pid {}] clone -> child pid {}",
        process::with_sched(|s| s.current_pid()),
        child_pid
    );
    child_pid as u64
}

/// `execve(path, argv, envp)` — replace the current process image. We have no
/// filesystem, so any non-NULL `path` re-loads the **embedded** program image
/// (this is enough to demonstrate exec semantics: tear down + fresh ELF + reset
/// PC/SP, keeping the pid). The pid, ppid, and child list are preserved.
///
/// # Safety
/// Validates `path`; rebuilds the current process's address space in place and
/// rewrites the live trap frame to the new entry.
unsafe fn sys_execve(f: &mut TrapFrame, path: u64) -> SyscallOutcome {
    // A NULL path is invalid; otherwise we ignore the actual string (no FS) and
    // re-load the embedded image, which is enough to demonstrate exec semantics.
    if path == 0 {
        return SyscallOutcome::Return(EFAULT);
    }
    let pid = current_pid();
    crate::kprintln!("[pid {}] execve -> loading embedded exec image", pid);

    // Build a brand-new address space + reset context from the EXEC image.
    // SAFETY: boot core, MMU up; builds page tables + copies the image.
    let fresh = unsafe { build_process(current_ppid(), EXEC_ELF) };

    // Move the fresh image into the current process slot, preserving identity
    // (pid/ppid/children). The *old* AddressSpace is swapped out and returned so
    // we can drop it only **after** switching TTBR0 to the new one — freeing it
    // here (while it is still the live map) would corrupt the page-table walk.
    let (old_space, ttbr0, tpidr, new_ctx) = process::with_sched(|s| {
        let cur = s.current();
        let old = core::mem::replace(&mut cur.space, fresh.space);
        cur.brk_cur = fresh.brk_cur;
        cur.mmap_cur = fresh.mmap_cur;
        cur.tpidr = fresh.tpidr;
        cur.ctx = fresh.ctx;
        // execve resets signal handlers to default, preserving ONLY the blocked
        // mask (POSIX). v1 simplification: this also resets SIG_IGN -> SIG_DFL,
        // which POSIX keeps; no demo relies on it (spec §1.3 / §8.9).
        let m = cur.signals.blocked;
        cur.signals = sig::SignalState::new();
        cur.signals.blocked = m;
        (old, cur.space.ttbr0(), cur.tpidr, clone_ctx(&cur.ctx))
    });

    // Install the new (same-pid) address space + TLS/SP and load the reset
    // context into the live frame so the `eret` lands at the new entry point.
    // SAFETY: `ttbr0` is the freshly-built L1 (upper GiBs keep the kernel map);
    // we rewrite the live frame the trampoline will restore from.
    unsafe {
        TTBR0_EL1.set(ttbr0);
        barrier::dsb(barrier::SY);
        core::arch::asm!("tlbi vmalle1", "dsb sy", "isb", options(nostack));
        TPIDR_EL0.set(tpidr);
        SP_EL0.set(new_ctx.sp);
    }
    f.regs = new_ctx.regs;
    f.sp = new_ctx.sp;
    f.elr = new_ctx.elr;
    f.spsr = new_ctx.spsr;

    // P4·SMP·S4c: the old address space is about to be freed (its page tables +
    // frames). The PROCS lock was released at the `with_sched` closure end and
    // THIS CPU just switched TTBR0 off it + flushed locally; any OTHER online CPU
    // that ran this process may still cache the old AS's translations. Shoot them
    // down (IRQs enabled — we hold no lock) BEFORE freeing, so no CPU can fault
    // into / write through a freed page table or frame. No-op on 1-vCPU.
    crate::shootdown::request_and_wait_others();

    // Now that TTBR0 points at the new space and no CPU holds a stale entry, it
    // is safe to free the old image's page tables + frames.
    drop(old_space);
    SyscallOutcome::Resume
}

/// Clone a [`TrapFrame`] by value (POD `#[repr(C)]`).
fn clone_ctx(f: &TrapFrame) -> TrapFrame {
    TrapFrame {
        regs: f.regs,
        sp: f.sp,
        elr: f.elr,
        spsr: f.spsr,
    }
}

/// `wait4(pid, status, options, rusage)` — block the parent until a matching
/// child becomes a zombie, reap it, write the encoded status to `*status`, and
/// return the child pid. `WNOHANG` returns 0 immediately if no child has exited.
fn sys_wait4(f: &TrapFrame, pid: u64, status: u64, options: u64) -> SyscallOutcome {
    let target = pid as i64; // -1 = any child
    if status != 0 && !user_range_ok(status, 4) {
        return SyscallOutcome::Return(EFAULT);
    }

    // P4·SMP·S4d: read the live EL0 stack pointer + TLS base NOW (the trap stub
    // does not stash SP_EL0), so the blocking branch below can save THIS process's
    // full context into its `ctx` ATOMICALLY with the `Waiting` transition, under
    // the scheduler lock — before any sibling CPU's `complete_waits` can wake it.
    // SAFETY: side-effect-free reads of this CPU's SP_EL0 / TPIDR_EL0.
    let blk_sp_el0: u64;
    unsafe {
        core::arch::asm!("mrs {x}, sp_el0", x = out(reg) blk_sp_el0, options(nostack, nomem));
    }
    let blk_tpidr = TPIDR_EL0.get();

    // Decide ECHILD / reap / WNOHANG / block ATOMICALLY under ONE scheduler lock.
    // Folding the zombie re-check and the `Waiting` transition into a SINGLE lock
    // hold closes the cross-CPU wait/wakeup TOCTOU: previously `try_reap` and the
    // `Waiting` mark were two separate `with_sched` acquisitions, so a child that
    // exited on another CPU in the GAP between them — taking the lock to mark
    // itself `Zombie` + run `complete_waits`, which did NOT yet see us `Waiting` —
    // had its wakeup LOST, and the parent blocked forever (the ~1/10 multi-CPU
    // reap-demo hang). Now the exit is serialized either BEFORE this section (we
    // reap the zombie, no block) or AFTER (we are already `Waiting`, so
    // `complete_waits` wakes us). The user-memory status write stays OUTSIDE the
    // lock (it touches only this process's own page).
    enum Wait {
        NoChild,
        Reaped(u32, i32, process::Process),
        WouldBlock,
        Blocked,
    }
    let outcome = process::with_sched(|s| {
        if !s.has_child(target) {
            return Wait::NoChild;
        }
        if let Some((cpid, encoded, child)) = s.try_reap(target) {
            return Wait::Reaped(cpid, encoded, child);
        }
        if options & WNOHANG != 0 {
            return Wait::WouldBlock;
        }
        // P4·SMP·S4d: save THIS process's full context AND flip it to `Waiting`
        // atomically under the held lock, so a sibling CPU's `complete_waits` wake
        // (which writes the reaped pid into `ctx.regs[0]`) is never clobbered by a
        // post-block `save_current`. The epilogue uses `BlockedReschedule` to skip
        // that redundant (racy) save.
        s.block_current_for_wait(f, blk_sp_el0, blk_tpidr, target, status);
        Wait::Blocked
    });
    match outcome {
        Wait::NoChild => SyscallOutcome::Return(ECHILD),
        Wait::Reaped(cpid, encoded, child) => {
            if status != 0 {
                // SAFETY: validated 4-byte RW user buffer in the current space.
                unsafe { (status as *mut i32).write(encoded) };
            }
            crate::kprintln!(
                "[pid {}] wait4 reaped child {} (status {:#x})",
                current_pid(),
                cpid,
                encoded
            );
            // P4·SMP·S4d: `child` is the reaped Process, moved out of the table by
            // `try_reap` but NOT yet freed. The `PROCS` lock is released (we are
            // past `with_sched`) and we are running on the PARENT's own surviving
            // TTBR0, so freeing the child's page tables + COW frames now cannot pull
            // the live map out from under us. Shoot down the other online CPUs FIRST
            // so none retains a stale TLB translation into a frame we are about to
            // return to the allocator (same rationale as `sys_exit`'s deferred
            // `drop(reaped)`); no-op on 1-vCPU, where the child is simply dropped
            // here exactly when the old inline `table[cpid] = None` dropped it
            // (golden byte-identical).
            crate::shootdown::request_and_wait_others();
            drop(child);
            SyscallOutcome::Return(cpid as u64)
        }
        Wait::WouldBlock => SyscallOutcome::Return(0),
        // The context was already saved atomically with the `Waiting` mark inside
        // the lock (see `block_current_for_wait`); the epilogue must NOT re-save.
        Wait::Blocked => SyscallOutcome::BlockedReschedule,
    }
}

/// `exit`/`exit_group(status)` — mark the current process a zombie, deliver its
/// status to a parent blocked in `wait4`, switch to the next runnable process,
/// and never return to the dead process. If it was the last one, the workload
/// is finished.
///
/// The careful ordering here avoids freeing the *current* address space while
/// `TTBR0` still points at it: we (1) mark zombie + run the wait4 completion,
/// which only *moves* reaped processes out of the table (no frees yet), (2)
/// switch `TTBR0`/the live frame to the next runnable process, and only then
/// (3) drop the reaped processes — by which point `TTBR0` is a survivor's space.
///
/// # Safety
/// `f` is the live trap frame; this rewrites it to the next process's context.
unsafe fn sys_exit(f: &mut TrapFrame, status: i32) -> SyscallOutcome {
    let pid = current_pid();
    crate::kprintln!("[pid {}] exit({})", pid, status);

    // Notify the parent of the child's death: post SIGCHLD to ppid. This is
    // BIT-ONLY (no print, no state change) — it must not flip a Waiting parent
    // to Runnable (that's `complete_waits`'s job below) nor emit any trace line,
    // so the golden trace stays byte-identical. A SIG_DFL SIGCHLD is the Ignore
    // default, cleared (delivering nothing) at the parent's next return.
    let ppid = current_ppid();
    process::with_sched(|s| s.post_signal(ppid, sig::SIGCHLD));

    // Mark zombie + store status, then complete any blocked wait4 (deferring the
    // actual memory free of reaped children to `reaped` below).
    let reaped = process::with_sched(|s| {
        s.exit_current(status);
        s.complete_waits()
    });

    // P4·SMP·S4d: the current process is now `Zombie` and the `PROCS` lock is
    // RELEASED, so its parent (on another CPU) may reap it — freeing its L1 +
    // frames — at any instant from here on. We are STILL on its (dying) TTBR0, so
    // park on the kernel identity map BEFORE anything else can run: a remote reap
    // freeing the dying L1 must not pull the live map out from under this CPU.
    // `switch_to` below re-installs a survivor's TTBR0 if one is picked; if not,
    // we idle already parked on the safe kernel map. The reap-vs-this-section race
    // is otherwise closed by the lock (a reaper also needs `PROCS`), so the only
    // unprotected window is exactly here — which this park eliminates.
    // SAFETY: EL1, this CPU; installs the kernel L1 (upper GiBs keep EL1 mapped).
    unsafe { park_on_kernel_map() };

    // Try to switch to the next process runnable on THIS CPU (installs a
    // surviving TTBR0 + rewrites the live frame); on success drop the reaped
    // children (we are off the dying TTBR0).
    // SAFETY: `f` is the live frame; `schedule` rewrites it to the next process.
    let switched = unsafe { process::schedule(f) };
    if switched {
        // P4·SMP·S4c: `reaped` holds exited children about to be freed (their
        // page tables + COW frames). The PROCS lock is released (past every
        // `with_sched`/`schedule`), so shoot down the other online CPUs BEFORE
        // the free, so none retains a translation into a frame we are about to
        // return to the allocator. No-op on 1-vCPU. (Only needed when something
        // was actually reaped; an empty vec makes the local invalidation moot,
        // but the shootdown is cheap + unconditional for simplicity.)
        if !reaped.is_empty() {
            crate::shootdown::request_and_wait_others();
        }
        drop(reaped);
        return SyscallOutcome::Resume;
    }
    // Nothing runnable for THIS CPU. We already parked on the kernel identity map
    // above (right after marking Zombie), so we are NOT on the dying TTBR0 here.
    // Keep the conservative leak of the reaped children (a shutdown-time memory-
    // only concern, not a fault): `forget` preserves the prior golden teardown
    // exactly. BSP+empty-table diverts the eret to the power-off continuation
    // (`Finished`); otherwise this CPU (re-)enters its EL1 idle loop
    // (`idle_or_finish` never returns in that case).
    core::mem::forget(reaped);
    // SAFETY: EL1 syscall context whose `schedule` returned false.
    if unsafe { idle_or_finish(f) } {
        SyscallOutcome::Finished
    } else {
        // Unreachable in practice (the idle branch diverges), but keep the
        // type-checker happy: if it ever returned, resume the (rewritten) frame.
        SyscallOutcome::Resume
    }
}

// ---- workload teardown ----------------------------------------------------

/// EL1 continuation reached only when the **last** process has exited.
extern "C" fn user_return() -> ! {
    finish_user();
}

/// Install the saved kernel-only identity map (`SAVED_TTBR0`) on THIS CPU and
/// flush its TLB. Called when a CPU is about to go idle (`wfe`) with no process
/// to run (P4·SMP·S4d): an idle CPU must NOT keep a user process's L1 as its live
/// `TTBR0`, because that process can be reaped + its page-table frames freed (and
/// reused) by a sibling CPU while this CPU sleeps on them — the upper-GiB kernel
/// identity entries in the freed L1 then go stale and the next kernel access (the
/// `wfe` wake's `schedule`, or `finish_user`) faults at EL1. Parking on the
/// always-valid `SAVED_TTBR0` removes that use-after-free window. `switch_to`
/// reinstalls the picked process's `TTBR0` the moment this CPU schedules work
/// again, so it is purely an idle-time safety net. NEVER reached on 1-vCPU (the
/// BSP powers off directly without entering the idle loop), so the golden trace
/// and the single-core fast path are untouched.
///
/// # Safety
/// EL1, this CPU's own context; `SAVED_TTBR0` is the kernel L1 captured before any
/// process ran, whose upper GiBs keep EL1 code/data/MMIO addressable.
unsafe fn park_on_kernel_map() {
    // SAFETY: read the kernel L1 saved once by the BSP before the first process;
    // switching to it + flushing the local TLB cannot lose EL1 addressability.
    unsafe {
        let saved = core::ptr::read_volatile(core::ptr::addr_of!(SAVED_TTBR0));
        TTBR0_EL1.set(saved);
        barrier::dsb(barrier::SY);
        core::arch::asm!("tlbi vmalle1", "dsb sy", "isb", options(nostack));
    }
}

/// Restore the kernel-only identity map and return to the kernel boot flow.
fn finish_user() -> ! {
    // SAFETY: `SAVED_TTBR0` is the kernel's original TTBR0 captured before the
    // first process ran; switching back + flushing the TLB restores it.
    unsafe {
        let saved = core::ptr::read_volatile(core::ptr::addr_of!(SAVED_TTBR0));
        TTBR0_EL1.set(saved);
        barrier::dsb(barrier::SY);
        core::arch::asm!("tlbi vmalle1", "dsb sy", "isb", options(nostack));
    }
    crate::kprintln!("all processes exited");
    crate::arch::user_finished()
}

/// Divert the live `eret` back into the kernel: rewrite ELR/SPSR so the
/// trampoline returns to `user_return` at EL1h with IRQs masked.
fn finish_workload(f: &mut TrapFrame) {
    f.elr = user_return as *const () as usize as u64;
    f.spsr = SPSR_KERNEL_RETURN;
}

/// `reboot(magic1, magic2, cmd, arg)` — the only command we honour is
/// `RB_POWER_OFF` (what talos's `power_off()` issues): we validate the two Linux
/// reboot magics and, on a power-off request, power the machine off via PSCI
/// SYSTEM_OFF (the same path the timer-demo teardown uses), cleanly terminating
/// QEMU (never returns). A reboot with the wrong magics is `-EINVAL` (as Linux
/// does); a recognised-magic reboot with any *other* command returns 0 (we have
/// nothing to restart).
fn sys_reboot(magic1: u64, magic2: u64, cmd: u64) -> u64 {
    if magic1 != LINUX_REBOOT_MAGIC1 || magic2 != LINUX_REBOOT_MAGIC2 {
        return EINVAL;
    }
    if cmd == RB_POWER_OFF {
        crate::kprintln!("reboot: RB_POWER_OFF -> powering off");
        crate::arch::power_off();
    }
    0
}

// ---- timer-preemption hook ------------------------------------------------

/// Called from the IRQ path when the periodic timer fires *while EL0 was
/// running* (a lower-EL IRQ). Preempt the current process: save its context and
/// switch to the next runnable one. If only one process is runnable this is a
/// no-op (it keeps running).
///
/// # Safety
/// `frame` is the live trap frame the IRQ stub pushed; may be rewritten.
pub unsafe fn on_timer_preempt(frame: *mut TrapFrame) {
    // SAFETY: the IRQ vector just pushed a valid frame at `frame`.
    let f = unsafe { &mut *frame };
    // SAFETY: saves the current EL0 context and switches to the next runnable.
    unsafe {
        process::save_current(f);
        if !process::schedule(f) {
            // Nothing runnable for THIS CPU on this tick: BSP+empty-table diverts
            // the eret to power-off; otherwise (re-)enter this CPU's EL1 idle
            // loop (idle_or_finish never returns in that case). If it returns
            // true (BSP finish) we let the eret to the power-off continuation run.
            idle_or_finish(f);
            return;
        }
        // Deliver one pending, unblocked signal to the (possibly just-switched)
        // current process before the IRQ stub `eret`s back to EL0.
        deliver_pending_signals(f);
    }
}

// ---- entry: build the first process and run the workload ------------------

/// Safe entry the kernel calls to load and run the embedded program as the
/// first process, then drive the scheduler until every process has exited.
///
/// Never returns to its caller in the normal flow (it diverts to
/// [`crate::arch::user_finished`], which powers off).
pub fn run_user() -> ! {
    // SAFETY: called once from the boot flow after MMU/heap/GIC are up; all the
    // dangerous steps are audited and operate on Frame-owned state.
    unsafe {
        // Capture the kernel TTBR0 so we can restore it after the workload.
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SAVED_TTBR0), TTBR0_EL1.get());

        // Enable Advanced SIMD / floating-point at EL0/EL1 (musl uses V regs).
        CPACR_EL1.write(CPACR_EL1::FPEN::TrapNothing);
        barrier::isb(barrier::SY);

        // Build process 1 from the embedded image and admit it as current.
        let first = build_process(0, USER_ELF);
        let entry = first.ctx.elr;
        let sp = first.ctx.sp;
        let tpidr = first.tpidr;
        let ttbr0 = first.space.ttbr0();
        let pid = process::with_sched(|s| s.admit_first(first));

        crate::kprintln!(
            "process: pid {} entering EL0 at {:#x} (sp={:#x})",
            pid,
            entry,
            sp
        );

        // Proof of life for the per-process fd table: assert pid 1's fds 0/1/2
        // all resolve to /dev/console, then announce it once. This line is
        // deliberately NOT in the `[pid N] syscall NR -> RET` shape, so the
        // diff-oracle's trace regex never matches it and the golden is unchanged.
        let std_ok = process::with_sched(|s| {
            let p = s.current();
            p.fd_kind(0) == Some(FileKind::Console)
                && p.fd_kind(1) == Some(FileKind::Console)
                && p.fd_kind(2) == Some(FileKind::Console)
        });
        assert!(std_ok, "pid 1 fd table not wired to /dev/console");
        crate::kprintln!("vfs: fd table live (0,1,2 -> /dev/console)");

        // Publish the timekeeper base (counter sample + mult/shift) so the clock
        // syscalls read a real monotonic/realtime clock. Single boot-core writer.
        crate::timer::init_timekeeper();

        // Arm the periodic timer as a *preemption* source and unmask IRQs so the
        // scheduler can time-slice between processes while they run at EL0.
        crate::timer::init_preempt(SCHED_TICK_MS);
        aarch64_cpu::registers::DAIF.write(aarch64_cpu::registers::DAIF::I::Unmasked);

        // Install the first process's address space + TLS and drop to EL0.
        // SAFETY: `ttbr0` is its L1 (upper GiBs keep the kernel identity map).
        TTBR0_EL1.set(ttbr0);
        barrier::dsb(barrier::SY);
        core::arch::asm!("tlbi vmalle1", "dsb sy", "isb", options(nostack));
        TPIDR_EL0.set(tpidr);
        SP_EL0.set(sp);
        enter_el0(entry, sp);
    }
}

/// Scheduler time-slice (timer preemption period), in milliseconds.
const SCHED_TICK_MS: u64 = 10;

/// Drop to EL0: set SP_EL0, ELR_EL1=entry, SPSR_EL1=EL0t, then `eret`.
///
/// # Safety
/// The user mappings must be installed and `entry`/`sp` inside the EL0 window.
/// Never returns to the caller; control resumes in EL0.
unsafe fn enter_el0(entry: u64, sp: u64) -> ! {
    // SAFETY: standard EL1->EL0 transition with a clean EL0t SPSR.
    unsafe {
        core::arch::asm!(
            "msr sp_el0, {sp}",
            "msr elr_el1, {entry}",
            "msr spsr_el1, {spsr}",
            "isb",
            "eret",
            sp = in(reg) sp,
            entry = in(reg) entry,
            spsr = in(reg) SPSR_EL0T,
            options(noreturn, nostack),
        );
    }
}

// ---------------------------------------------------------------------------
// P4·SMP·S4a — AP idle→schedule loop + machine-wide termination
// ---------------------------------------------------------------------------

/// AP bootstrap + scheduler entry (P4·SMP·S4a). Called by `smp::ap_rust_entry`
/// after the S3 per-CPU bring-up (MMU on, anchor, local GIC + timer PPI, online
/// bit). Completes this AP's scheduler prerequisites — waits for the process
/// model to go live, enables FP/SIMD at EL0/EL1 on this AP, arms this AP's
/// preemption timer, unmasks IRQs — then enters the idle→schedule loop. NEVER
/// returns.
///
/// # Safety
/// Called once per AP, EL1, on the AP's own stack, with this AP's per-CPU anchor
/// (`TPIDR_EL1 = idx`) installed and `idx < MAX_CPUS` unique. IRQs masked on entry.
pub(crate) unsafe fn ap_bootstrap_and_run(_idx: usize) -> ! {
    // 0. Install THIS AP's exception vector base (`VBAR_EL1` is per-CPU). Without
    //    it, the first EL0 exception on this AP (e.g. a COW data abort or an SVC
    //    from a process it runs) would vector to a garbage base → silent hang.
    //    The BSP set its own in `exceptions::init`; each AP must set its own.
    // SAFETY: AP, EL1; installs the statically-linked vector table base.
    unsafe { crate::exceptions::init() };

    // 1. Wait for the BSP to switch the timer to preemption mode (it publishes
    //    INTERVAL + the process model in `run_user`). Until then there is nothing
    //    to schedule and the timer interval is not yet published.
    while !crate::timer::preempt_active() {
        core::hint::spin_loop();
    }

    // 2. Enable Advanced SIMD / FP at EL0/EL1 on this AP (musl/std EL0 code uses
    //    V registers). Per-CPU CPACR_EL1 (a safe system-register write, like the
    //    BSP's in `run_user`).
    CPACR_EL1.write(CPACR_EL1::FPEN::TrapNothing);
    barrier::isb(barrier::SY);

    // 3. Arm THIS AP's preemption timer (reusing the BSP's interval) so it gets
    //    preemption ticks + `wfe` wakeups, then unmask IRQs.
    // SAFETY: AP, GIC + timer PPI up; programs this CPU's CNTP timer.
    unsafe {
        crate::timer::arm_preempt_ap();
        aarch64_cpu::registers::DAIF.write(aarch64_cpu::registers::DAIF::I::Unmasked);
    }

    // 4. Enter the idle→schedule loop. NEVER returns.
    // SAFETY: all per-CPU scheduler prerequisites are now up on this AP.
    unsafe { ap_run_scheduler() }
}

/// The AP idle→schedule loop (P4·SMP·S4a). Pops a process from THIS CPU's run
/// queue (or the global Runnable set), drops to EL0 to run it, and — when it has
/// no work — `wfe`s until its next periodic tick re-runs the scheduler. Also the
/// re-entry target when a CPU running an EL0 process finds nothing else runnable
/// (`idle_or_finish`). NEVER returns.
///
/// # Safety
/// EL1 on this CPU's stack, the process model live, this AP's per-CPU anchor +
/// preemption timer up.
pub(crate) unsafe fn ap_run_scheduler() -> ! {
    let mut frame = process::zeroed_trapframe();
    loop {
        // Mask IRQs across the schedule decision (the `with_sched` lock is
        // `lock_irqsave` regardless; masking here also makes the wfe/schedule
        // choice atomic w.r.t. the periodic tick — no lost wakeup).
        aarch64_cpu::registers::DAIF.write(aarch64_cpu::registers::DAIF::I::Masked);
        // SAFETY: EL1 on this AP; `schedule` may switch TTBR0 + rewrite `frame`
        // to the picked process. Returns false if nothing is runnable here.
        let got = unsafe { process::schedule(&mut frame) };
        if got {
            // A process was picked; `frame` + TTBR0/TPIDR/SP_EL0 are installed.
            // Drop to EL0; the periodic timer preempts it (lower-EL IRQ →
            // `on_timer_preempt`), and when this CPU next runs dry the preempt
            // path routes back here via `idle_or_finish`.
            // SAFETY: TTBR0 is the picked process's space; `frame` its context.
            unsafe { enter_el0_full(&frame) };
        }
        if !process::any_alive() {
            // The whole workload is done. The BSP restores the kernel map +
            // powers the machine off (`finish_user`, which never returns); APs
            // just halt. Routing through the BSP check lets the workload
            // terminate even when the LAST process exited on an AP and the BSP
            // only notices here in its idle loop on the next tick.
            if this_cpu_index() == 0 {
                finish_user(); // `-> !`: restore kernel TTBR0 + power off.
            }
            // AP at end-of-workload: halt until the BSP powers off, but keep IRQs
            // ENABLED so we still SERVICE + ack any in-flight P4·SMP·S4c shootdown
            // SGI (a sibling CPU exiting/reaping may be waiting on our ack). A
            // halted CPU runs no process, so a serviced shootdown is a harmless
            // local flush; ignoring it would HANG the sender. (No timer is armed
            // to fire here, so this does not perturb the 1-vCPU path — 1-vCPU
            // never reaches this loop, the BSP powers off directly.)
            // P4·SMP·S4d: park on the kernel identity map first so this halted CPU
            // is not sleeping on a reaped process's freed L1.
            // SAFETY: EL1, this CPU; installs the always-valid kernel L1.
            unsafe { park_on_kernel_map() };
            loop {
                aarch64_cpu::registers::DAIF.write(aarch64_cpu::registers::DAIF::I::Unmasked);
                aarch64_cpu::asm::wfe();
            }
        }
        // Work may land on our queue later. P4·SMP·S4d: park on the kernel identity
        // map BEFORE idling so that if the process this CPU last ran is reaped (its
        // page-table frames freed + reused) while we `wfe`, our live TTBR0 is the
        // always-valid kernel L1, not a stale/freed user L1 — closing the idle-CPU
        // use-after-free that faulted the multi-process demo at teardown.
        // SAFETY: EL1, this CPU; installs the always-valid kernel L1.
        unsafe { park_on_kernel_map() };
        // Unmask IRQs + `wfe` until the next periodic tick (or, S4b, a reschedule
        // SGI) wakes us to retry.
        aarch64_cpu::registers::DAIF.write(aarch64_cpu::registers::DAIF::I::Unmasked);
        aarch64_cpu::asm::wfe();
    }
}

/// A CPU's `schedule` found nothing runnable for it (P4·SMP·S4a) — the SMP-aware
/// replacement for the bare `finish_workload(f)` at the schedule-false sites:
///
/// * **Workload finished** (`!any_alive()`): the **BSP** (cpu 0) diverts the live
///   `eret` back into the kernel via [`finish_workload`] (which powers off) —
///   returning `true` so the caller lets the `eret` happen and skips signal
///   delivery; an AP halts in `wfe` (the machine powers off via the BSP).
///   On 1-vCPU "schedule found nothing" only happens when the last process
///   exited, and that CPU IS the BSP → identical to the old `finish_workload`.
///
/// * **Work alive elsewhere**: the calling CPU abandons the current trap frame
///   and (re-)enters its EL1 idle→schedule loop [`ap_run_scheduler`] on a fresh
///   stack, `wfe`-ing for its next tick. NEVER returns in that case.
///
/// Returns `true` iff it diverted the frame to finish (BSP power-off path).
///
/// # Safety
/// EL1, in a trap/IRQ context whose `schedule` just returned false. May discard
/// the live trap frame (jumping to the idle loop) — no user context worth saving.
unsafe fn idle_or_finish(f: &mut TrapFrame) -> bool {
    if !process::any_alive() {
        if this_cpu_index() == 0 {
            finish_workload(f); // BSP: divert eret to the power-off continuation.
            return true;
        }
        // AP at end-of-workload: halt until power-off, IRQs ENABLED so we still
        // ack any in-flight P4·SMP·S4c shootdown SGI (else a sibling sender would
        // hang). A halted CPU runs nothing, so the serviced flush is harmless.
        // P4·SMP·S4d: park on the kernel identity map first (see `park_on_kernel_map`).
        // SAFETY: EL1, this CPU; installs the always-valid kernel L1.
        unsafe { park_on_kernel_map() };
        loop {
            aarch64_cpu::registers::DAIF.write(aarch64_cpu::registers::DAIF::I::Unmasked);
            aarch64_cpu::asm::wfe();
        }
    }
    // Work remains elsewhere; (re-)enter this CPU's EL1 idle loop on a fresh
    // stack. SAFETY: abandon the current frame and branch to the idle loop, which
    // never returns. We read the current SP for a fresh idle stack top below it.
    let sp_top: u64;
    unsafe {
        core::arch::asm!("mov {x}, sp", x = out(reg) sp_top, options(nostack, nomem));
    }
    let idle_sp = (sp_top - 0x2000) & !0xf;
    unsafe {
        core::arch::asm!(
            "mov sp, {sp}",
            "b {idle}",
            sp = in(reg) idle_sp,
            idle = sym ap_run_scheduler,
            options(noreturn),
        );
    }
}

/// This CPU's logical index (0 = BSP), read from the per-CPU `TPIDR_EL1` anchor.
fn this_cpu_index() -> usize {
    // SAFETY: EL1 trap context; the per-CPU anchor is installed (the same
    // invariant `this_cpu_token` requires); a side-effect-free sysreg read.
    let token = unsafe { crate::percpu::this_cpu_token() };
    token.cpu_index()
}

/// Drop to EL0 restoring the FULL saved [`TrapFrame`] (x0..x30 + ELR/SPSR) via
/// `eret`. Unlike [`enter_el0`] (which only sets entry ELR/SP for a fresh
/// process) this resumes an already-running process the scheduler picked — used
/// by an AP picking up a forked worker whose context has live register values.
/// `switch_to` already installed TTBR0/TPIDR/SP_EL0 for `ctx`.
///
/// # Safety
/// TTBR0 must be `ctx`'s process space (set by `switch_to`), and `ctx` a valid
/// EL0 context. Runs EL0 code; never returns to the caller.
unsafe fn enter_el0_full(ctx: &TrapFrame) -> ! {
    // Address the saved GPRs from a pointer pinned in x9 (regs x0..x30 at offsets
    // 0..240, ELR@256, SPSR@264). SP_EL0 was already set by `switch_to`. x9 holds
    // the pointer across every `ldr` and is itself loaded LAST, so no `ldr`
    // overwrites the base before it is finished with.
    // SAFETY: load ELR/SPSR + x0..x30 from the frame and `eret` into EL0.
    unsafe {
        core::arch::asm!(
            "ldr x10, [x9, #256]",     // ctx.elr
            "msr elr_el1, x10",
            "ldr x10, [x9, #264]",     // ctx.spsr
            "msr spsr_el1, x10",
            // Load x0..x30 (x9 itself loaded last from its saved value).
            "ldr x0,  [x9, #0]",
            "ldr x1,  [x9, #8]",
            "ldr x2,  [x9, #16]",
            "ldr x3,  [x9, #24]",
            "ldr x4,  [x9, #32]",
            "ldr x5,  [x9, #40]",
            "ldr x6,  [x9, #48]",
            "ldr x7,  [x9, #56]",
            "ldr x8,  [x9, #64]",
            "ldr x11, [x9, #88]",
            "ldr x12, [x9, #96]",
            "ldr x13, [x9, #104]",
            "ldr x14, [x9, #112]",
            "ldr x15, [x9, #120]",
            "ldr x16, [x9, #128]",
            "ldr x17, [x9, #136]",
            "ldr x18, [x9, #144]",
            "ldr x19, [x9, #152]",
            "ldr x20, [x9, #160]",
            "ldr x21, [x9, #168]",
            "ldr x22, [x9, #176]",
            "ldr x23, [x9, #184]",
            "ldr x24, [x9, #192]",
            "ldr x25, [x9, #200]",
            "ldr x26, [x9, #208]",
            "ldr x27, [x9, #216]",
            "ldr x28, [x9, #224]",
            "ldr x29, [x9, #232]",
            "ldr x30, [x9, #240]",
            "ldr x10, [x9, #80]",      // x10 = regs[10]
            "ldr x9,  [x9, #72]",      // x9 = regs[9], LAST (frees the base)
            "eret",
            in("x9") ctx as *const TrapFrame,
            options(noreturn, nostack),
        );
    }
}

// Keep SCTLR import meaningful; referenced so the use does not warn.
#[allow(dead_code)]
const _SCTLR_TOUCH: fn() -> u64 = || SCTLR_EL1.get();
