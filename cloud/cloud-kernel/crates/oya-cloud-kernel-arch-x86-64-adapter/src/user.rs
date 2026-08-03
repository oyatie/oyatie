//! Ring-3 user mode + **process model** for the x86_64 Frame: an ELF64 loader,
//! per-process address spaces, a Linux/SysV initial process stack, a full
//! register-context save/restore on every kernel entry, a cooperative-plus-
//! timer-preemptive scheduler, and the `clone`/`execve`/`wait4`/`exit` family so
//! a parent can spawn and reap a child.
//!
//! This is the x86_64 mirror of aarch64's **K7** (`arch-aarch64/src/user.rs` +
//! `process.rs`). It brings x86_64 from the single-process K4 ring-3 demo up to
//! a real process model: a first process **forks** a child, the child
//! **execve**s into a different image, **exit**s with a status, and the parent
//! **wait4**s to **reap** it.
//!
//! ## From one process to many
//!
//! The earlier bring-up ran exactly one ring-3 image to `exit` using global
//! statics (one page table, `iretq` in / `longjmp` out, no register save). The
//! process model moves all per-process state into [`crate::process::Process`] /
//! [`crate::process::AddressSpace`] on the kernel heap, owned by the global
//! [`crate::process::Scheduler`], and replaces the `iretq`/`longjmp` round-trip
//! with **register-context** entry stubs: both the `syscall` trampoline and the
//! timer IRQ save the full ring-3 GPR set into a [`Context`], call a Rust
//! handler that may switch to another process, then resume from the (possibly
//! switched) context via `sysretq` / `iretq`. Context switch = swap which
//! `Context` we restore + `Cr3::write` the next process's PML4.
//!
//! ## This is Frame code
//!
//! It touches page tables, copies into user memory, switches `CR3`, drops to
//! ring 3, and reads/writes user pointers — all `unsafe`. The safe kernel only
//! calls the single safe [`run_user`] entry; every dangerous site carries a
//! `// SAFETY:` note.

use core::sync::atomic::{AtomicBool, Ordering};

use x86_64::registers::model_specific::{
    Efer, EferFlags, GsBase, KernelGsBase, LStar, Msr, SFMask,
};
use x86_64::structures::paging::PageTableFlags as Ptf;

use crate::gdt;
use crate::process::{self, AddressSpace, Context, FileDesc, FileKind, Perm, Process, State};

// The arch-neutral layout/stack-builder math lives in `user_layout`.
use user_layout::{
    build_stack_image, user_range_ok, StackInputs, MMAP_BASE, MMAP_TOP, PAGE_MASK, PAGE_SIZE,
    USER_BASE, USER_STACK_BOTTOM, USER_STACK_TOP, USER_TOP,
};
// Pure signal math: bit ops, the SigAction/SignalState PODs, the default-action
// classifier, and the shared signal-frame offset constants + alignment fns.
use user_layout::signal as sig;

// ---------------------------------------------------------------------------
// The user programs: real static x86_64 ELFs, embedded at build time
// ---------------------------------------------------------------------------

/// The first process image: the freestanding **`user-spawn`** program (a
/// `no_std`/`no_main` crate in `crates/arch-x86_64/user-spawn-src/`, built to a
/// static non-PIE ELF64 linked at [`USER_BASE`]). It `clone()`s a child, the
/// child `execve`s into `user-exec`, and the parent `wait4()`s to reap it.
/// Rebuild with `crates/arch-x86_64/user-spawn-src/build.sh`.
#[cfg(not(any(
    feature = "signal-demo",
    feature = "clock-demo",
    feature = "svc-demo",
    feature = "init-demo",
    feature = "fsbase-demo",
    feature = "talos-init",
    feature = "smp-sched-demo"
)))]
static USER_ELF: &[u8] = include_bytes!("../../../out/user-spawn-x86_64.elf");

/// With `--features smp-sched-demo` (P4·SMP·S4a) the embedded image is the
/// no_std **`user-smpdemo`** fan-out supervisor (`crates/arch-x86_64/
/// user-smpdemo-src/`, built to `out/user-smpdemo-x86_64.elf`). It forks N=8
/// worker processes (> the test's `-smp 4`); the kernel places them round-robin
/// across the online CPUs and logs `sched: pid P -> cpu K` the first time each
/// runs. Under `-smp 4` those lines show workers on multiple distinct cpu
/// indices, proving the APs actually run processes. The parent `wait4`s all 8.
/// NON-default, verification-only; the default build keeps the user-spawn image
/// so the golden trace is untouched. Built out-of-repo via
/// crates/arch-x86_64/user-smpdemo-src/build.sh.
#[cfg(feature = "smp-sched-demo")]
static USER_ELF: &[u8] = include_bytes!("../../../out/user-smpdemo-x86_64.elf");

/// With `--features signal-demo` the embedded image is swapped to the
/// **`user-signal-x86_64`** program (installs a SIGUSR1 handler via
/// `rt_sigaction` + `SA_RESTORER`, raises it with `tgkill`, asserts the handler
/// ran, then exercises the VFS fd paths). NON-default, verification-only; the
/// golden harness runs the default build so the golden stays untouched. See
/// P3_SIGNALS_SPEC.md §7. Built out-of-repo to `out/user-signal-x86_64.elf`.
#[cfg(feature = "signal-demo")]
static USER_ELF: &[u8] = include_bytes!("../../../out/user-signal-x86_64.elf");

/// With `--features clock-demo` the embedded image is swapped to the
/// **`user-clock-x86_64`** program (clock_gettime/nanosleep prove the real TSC
/// timekeeper advances; prints `clock: mono advanced ...`). NON-default,
/// verification-only; the default build keeps the user-spawn image. See
/// P3_TIMEKEEPING_SPEC.md §5. Built out-of-repo to `out/user-clock-x86_64.elf`
/// via crates/arch-x86_64/user-clock-src/build.sh.
#[cfg(feature = "clock-demo")]
static USER_ELF: &[u8] = include_bytes!("../../../out/user-clock-x86_64.elf");

/// With `--features svc-demo` the embedded image is swapped to the **real
/// `talos-svc`** binary (`talos-init/src/svc.rs`) cross-compiled for
/// `x86_64-unknown-linux-musl` as a static, non-PIE ET_EXEC linked at
/// [`USER_BASE`] — the SAME unmodified Rust-std/musl workload that already runs
/// on aarch64 (`out/svc.elf`). It prints heartbeats, `nanosleep`s between them,
/// then `exit(0)`. This exercises the full musl C-runtime startup
/// (`_start`/`__init_tp`/`__libc_start_main`) on x86_64. NON-default,
/// verification-only; the default build keeps the user-spawn image so the golden
/// trace is untouched. Built out-of-repo (rust:alpine, `--platform linux/amd64`,
/// `RUSTFLAGS=-C relocation-model=static -C link-arg=-no-pie`) to
/// `out/svc-x86_64.elf`. See P3_PARITY_FLOOR_PLAN.md Slice A/B.
#[cfg(feature = "svc-demo")]
static USER_ELF: &[u8] = include_bytes!("../../../out/svc-x86_64.elf");

/// With `--features talos-init` the embedded image is the **real, unmodified
/// talos-init** PID1 (`../operating-system/talos-init`, the talos-machined 7-phase boot
/// sequencer that links talos-machined/network/platform/runtime-cri/cosi),
/// cross-compiled for `x86_64-unknown-linux-musl` as a static, non-PIE ET_EXEC
/// linked at [`USER_BASE`] (`out/talos-init-x86_64.elf`). It is much larger than
/// `svc` (~1.16 MiB), which is why the user VA window was enlarged from 2 MiB to
/// 8 MiB (`USER_NTABLES`). On boot it runs musl C-runtime startup then enters
/// the sequencer, whose phase-1 `MountPseudoFs` calls `mount(2)` — the kernel
/// returns `-ENOSYS`, which `talos-machined`'s skip-classifier does NOT tolerate,
/// so the real init aborts there. That abort is the Milestone-2 baseline.
/// NON-default, verification-only; the default build keeps the user-spawn image
/// so the golden trace is untouched. Built out-of-repo (rust:alpine,
/// `--platform linux/amd64`, `RUSTFLAGS=-C relocation-model=static -C
/// link-arg=-no-pie -C target-feature=+crt-static`). See MILESTONE_2_PLAN.md
/// Slice 0.
#[cfg(feature = "talos-init")]
static USER_ELF: &[u8] = include_bytes!("../../../out/talos-init-x86_64.elf");

/// With `--features init-demo` the embedded image is the no_std **PID1 init
/// supervisor** (`crates/arch-x86_64/user-init-src/`, built to
/// `out/user-init-x86_64.elf`). PID1 installs SIGCHLD + SIGTERM handlers, clones
/// a worker child that `execve`s the real `svc` (see `EXEC_ELF` below, also
/// swapped under init-demo), polls a short `clock_nanosleep` watching the SIGCHLD
/// `.bss` flag, then `wait4(WNOHANG)`-reaps the worker and exits cleanly. This is
/// the Milestone-1 parity-floor capstone. NON-default, verification-only; the
/// default build keeps the user-spawn image so the golden trace is untouched.
/// See P3_PARITY_FLOOR_PLAN.md Slice C. Built out-of-repo via
/// crates/arch-x86_64/user-init-src/build.sh.
#[cfg(feature = "init-demo")]
static USER_ELF: &[u8] = include_bytes!("../../../out/user-init-x86_64.elf");

/// With `--features fsbase-demo` the embedded image is the no_std **PID1 fsbase
/// supervisor** (`crates/arch-x86_64/user-fsbase-src/`, built to
/// `out/user-fsbase-x86_64.elf`). PID1 clones TWO worker children; each `execve`s
/// the real std/musl TLS worker (`EXEC_ELF` below, also swapped under
/// fsbase-demo) which sets a `__thread` variable to its own pid and re-reads it
/// across several `clock_nanosleep` yields. Two concurrent musl processes set
/// DISTINCT `%fs` bases that the 10 ms timer preemption interleaves — the proof
/// that `arch_prctl` FS-base is persisted+restored per-process. NON-default,
/// verification-only; the default build keeps the user-spawn image so the golden
/// trace is untouched. Built out-of-repo via
/// crates/arch-x86_64/user-fsbase-src/build.sh.
#[cfg(feature = "fsbase-demo")]
static USER_ELF: &[u8] = include_bytes!("../../../out/user-fsbase-x86_64.elf");

/// The image `execve` loads. We have no filesystem, so `execve` ignores its path
/// and always loads this embedded program. In the DEFAULT build it is
/// **`user-exec`** (prints pid/ppid then `exit(7)`); routing to a *different*
/// image proves the loader replaces the process image. Rebuild with
/// `crates/arch-x86_64/user-exec-src/build.sh`.
#[cfg(not(any(feature = "init-demo", feature = "fsbase-demo", feature = "talos-init")))]
static EXEC_ELF: &[u8] = include_bytes!("../../../out/user-exec-x86_64.elf");

/// With `--features talos-init` the `execve` target is the REAL **talos-svc**
/// musl binary (`out/svc-x86_64.elf`, the same unmodified Rust-std/musl heartbeat
/// worker that runs on aarch64). So when the real talos-init PID1's forked child
/// `execve`s `/usr/bin/svc`, it genuinely becomes the real svc worker (clock
/// heartbeats + `exit(0)`), which talos then reaps (restart=never) and proceeds.
/// This makes the "spawns a real service" claim honest. Mirrors the init-demo
/// arm. See MILESTONE_2_PLAN.md.
#[cfg(feature = "talos-init")]
static EXEC_ELF: &[u8] = include_bytes!("../../../out/svc-x86_64.elf");

/// With `--features fsbase-demo` the `execve` target is the **std/musl TLS
/// worker** (`out/fsbase-worker-x86_64.elf`, built from
/// `crates/arch-x86_64/fsbase-worker-src/`). Each of PID1's TWO cloned children
/// `execve`s this SAME binary; since each child has its own pid it sets a
/// DISTINCT `__thread` value (its pid) via `arch_prctl(ARCH_SET_FS)` during musl
/// startup, then re-reads it across `clock_nanosleep` yields. The two distinct
/// `%fs` bases are what the per-process save/restore fix must keep separate. See
/// THE FIX-PROOF GATE in the task brief.
#[cfg(feature = "fsbase-demo")]
static EXEC_ELF: &[u8] = include_bytes!("../../../out/fsbase-worker-x86_64.elf");

/// With `--features init-demo` the `execve` target is the REAL **talos-svc**
/// musl binary (`out/svc-x86_64.elf`, the same unmodified Rust-std/musl workload
/// that runs on aarch64). So when PID1's cloned child `execve`s, it genuinely
/// becomes the real svc worker — PID1 supervises a real service. FS-base-safe:
/// PID1 is no_std (never sets %fs) and the single svc child is the only musl
/// process live at a time. See P3_PARITY_FLOOR_PLAN.md Slice C.
#[cfg(feature = "init-demo")]
static EXEC_ELF: &[u8] = include_bytes!("../../../out/svc-x86_64.elf");

// ---------------------------------------------------------------------------
// Linux/x86_64 syscall numbers + errno-style returns (mirror aarch64 K7)
// ---------------------------------------------------------------------------

const SYS_READ: u64 = 0;
const SYS_WRITE: u64 = 1;
/// `open(path, flags, mode)` — Linux x86_64 nr 2. Unlike aarch64 (which only has
/// `openat`), x86_64-linux-musl's `fs::File::open`/`fs::write` issue the legacy
/// 2-arg `open(2)` (verified from the boot trace: `syscall 2 -> …`). We resolve
/// it through the same in-RAM VFS path as `openat` (its path is arg0), so e.g.
/// `open("/proc/sys/kernel/kptr_restrict")` returns `-ENOENT` for a non-existent
/// node — which talos's best-effort `write_sysctl` tolerates — instead of the
/// un-tolerated `-ENOSYS` it got when this syscall was unhandled.
const SYS_OPEN: u64 = 2;
const SYS_CLOSE: u64 = 3;
/// `mount(source, target, fstype, flags, data)` — Linux x86_64 nr 165. Slice 1:
/// records the mount in the in-RAM VFS and returns 0 so the real talos-init's
/// MountPseudoFs phase advances.
const SYS_MOUNT: u64 = 165;
/// `mkdir(path, mode)` — Linux x86_64 nr 83. This is the call the real
/// talos-init (built for x86_64-linux-musl) actually issues for
/// `create_dir_all` (verified from the boot trace: `syscall 83 -> -38`).
/// Slice 2: create the directory (idempotently) in the in-RAM VFS via
/// `with_vfs(|v| v.mkdir_p(path))` and return 0 so SystemDirectories advances.
const SYS_MKDIR: u64 = 83;
/// `mkdirat(dirfd, path, mode)` — Linux x86_64 nr 258. Wired alongside `mkdir`
/// for completeness (musl may use either form); `dirfd`/`mode` ignored
/// (absolute paths, no perm model). Slice 2.
const SYS_MKDIRAT: u64 = 258;
const SYS_MMAP: u64 = 9;
const SYS_BRK: u64 = 12;
/// POSIX signal syscalls (x86_64 numbers; spec §2). The 4th arg (`sigsetsize`)
/// is `r10` = a3, matching the Linux register order the dispatcher reads.
const SYS_RT_SIGACTION: u64 = 13;
const SYS_RT_SIGPROCMASK: u64 = 14;
const SYS_RT_SIGRETURN: u64 = 15;
const SYS_WRITEV: u64 = 20;
const SYS_SCHED_YIELD: u64 = 24;
const SYS_DUP2: u64 = 33;
const SYS_GETPID: u64 = 39;
const SYS_CLONE: u64 = 56;
/// `fork()` — Linux x86_64 nr 57. There is no aarch64 analogue (aarch64 musl's
/// `fork()` lowers to `clone(SIGCHLD, 0, …)`), but x86_64 musl issues the legacy
/// `fork(2)` directly (verified from the talos boot trace: `syscall 57 -> -38`).
/// It is exactly `clone(SIGCHLD, 0, …)`: a COW address-space copy with a SIGCHLD
/// exit signal and no new stack. We dispatch it through the same `sys_clone`
/// routine with the fork ABI hard-coded, so it reuses the existing COW fork +
/// child-admission path and returns the child pid to the parent / 0 to the child.
const SYS_FORK: u64 = 57;
const SYS_EXECVE: u64 = 59;
const SYS_EXIT: u64 = 60;
const SYS_WAIT4: u64 = 61;
const SYS_KILL: u64 = 62;
/// `sync()` — Linux x86_64 nr 162. talos's `power_off()` flushes filesystems via
/// musl `sync()` before rebooting. We have no writeback caches, so this is a
/// no-op returning 0 (Linux `sync` always succeeds and returns 0).
const SYS_SYNC: u64 = 162;
/// `reboot(magic1, magic2, cmd, arg)` — Linux x86_64 nr 169. talos's
/// `power_off()` calls musl `reboot(RB_POWER_OFF)`, which lowers to
/// `reboot(LINUX_REBOOT_MAGIC1, LINUX_REBOOT_MAGIC2, LINUX_REBOOT_CMD_POWER_OFF,
/// NULL)`. On the POWER_OFF command we power the machine off (clean QEMU exit).
const SYS_REBOOT: u64 = 169;
const SYS_RT_SIGPENDING: u64 = 127;
const SYS_SIGALTSTACK: u64 = 131;
const SYS_GETPPID: u64 = 110;
const SYS_OPENAT: u64 = 257;
const SYS_EXIT_GROUP: u64 = 231;
const SYS_TGKILL: u64 = 234;
const SYS_TKILL: u64 = 200;
/// `dup3(oldfd, newfd, flags)` — `dup2` plus an `O_CLOEXEC` flag we ignore
/// (no exec-close in our demos).
const SYS_DUP3: u64 = 292;

// ---- P3 timekeeping syscalls (x86_64 numbers; spec §2) --------------------
const SYS_NANOSLEEP: u64 = 35;
const SYS_GETTIMEOFDAY: u64 = 96;
const SYS_CLOCK_GETTIME: u64 = 228;
const SYS_CLOCK_NANOSLEEP: u64 = 230;

// ---- musl C-runtime startup surface (x86_64 numbers; Slice A) -------------
// These mirror the arch-neutral aarch64 `sys_*` behaviour 1:1; only the syscall
// *number* differs per arch. Verified against the Linux x86_64 syscall table
// (arch/x86/entry/syscalls/syscall_64.tbl).
/// `arch_prctl(code, addr)` — x86_64-ONLY (no aarch64 analogue). musl's
/// `__init_tp` calls `arch_prctl(ARCH_SET_FS, tp)` to point `%fs` at its TCB so
/// `%fs:0`-relative TLS works. We program the `IA32_FS_BASE` MSR directly.
const SYS_ARCH_PRCTL: u64 = 158;
const SYS_IOCTL: u64 = 16;
const SYS_FSTAT: u64 = 5;
const SYS_NEWFSTATAT: u64 = 262;
const SYS_LSEEK: u64 = 8;
const SYS_DUP: u64 = 32;
const SYS_FCNTL: u64 = 72;
const SYS_GETCWD: u64 = 79;
const SYS_SYSINFO: u64 = 99;
const SYS_GETUID: u64 = 102;
const SYS_GETGID: u64 = 104;
const SYS_GETEUID: u64 = 107;
const SYS_GETEGID: u64 = 108;
const SYS_GETTID: u64 = 186;
const SYS_FUTEX: u64 = 202;
const SYS_SCHED_GETAFFINITY: u64 = 204;
const SYS_SET_TID_ADDRESS: u64 = 218;
const SYS_SET_ROBUST_LIST: u64 = 273;
const SYS_POLL: u64 = 7;
const SYS_PPOLL: u64 = 271;
const SYS_PRLIMIT64: u64 = 302;
const SYS_GETRANDOM: u64 = 318;
const SYS_RSEQ: u64 = 334;
const SYS_READLINKAT: u64 = 267;
const SYS_FACCESSAT: u64 = 269;
const SYS_MUNMAP: u64 = 11;
const SYS_MREMAP: u64 = 25;
const SYS_MPROTECT: u64 = 10;

// ---- M2 network slice: minimal AF_NETLINK socket syscalls (x86_64 nrs) -----
// The real, unmodified talos-init's `list_link_statuses` (the only hard-fatal
// netlink consumer) issues exactly socket+bind+sendto+recvfrom+close on an
// `AF_NETLINK`/`NETLINK_ROUTE` socket to dump links. `close` already exists
// (`SYS_CLOSE`). x86_64 socket nr=41 is CONFIRMED from the live boot trace
// (`[pid 1] syscall 41 -> -38`).
const SYS_SOCKET: u64 = 41;
const SYS_BIND: u64 = 49;
const SYS_SENDTO: u64 = 44;
const SYS_RECVFROM: u64 = 45;

// ---- WAVE 1: libc-init / process-info surface (x86_64 nrs) -----------------
// Every real glibc/musl binary issues these during init; they were previously
// -ENOSYS. All ADDITIVE (new syscall numbers) so they cannot alter the existing
// 'user-spawn' golden trace (getpid/clone/getpid/getppid). The pure byte-layout
// math (utsname/umask/clock_getres) lives in the shared `user_layout::procinfo`;
// the handlers below do the bounds-checked, SMAP-bracketed user copy. Numbers
// verified against arch/x86/entry/syscalls/syscall_64.tbl. Mirrors aarch64 1:1.
/// `uname(buf)` — fill `struct utsname` (six NUL-padded 65-byte fields).
const SYS_UNAME: u64 = 63;
/// `umask(mask)` — set the per-process file-creation mask, return the previous.
const SYS_UMASK: u64 = 95;
/// `getrusage(who, usage)` — write a zeroed `struct rusage`, return 0.
const SYS_GETRUSAGE: u64 = 98;
/// `times(buf)` — write a zeroed `struct tms`, return a monotonic tick count.
const SYS_TIMES: u64 = 100;
/// `clock_getres(clk, res)` — write timespec {0, 1} (1 ns resolution), return 0.
const SYS_CLOCK_GETRES: u64 = 229;
/// `setpgid(pid, pgid)` — single-session model, accept and return 0.
const SYS_SETPGID: u64 = 109;
/// `getpgid(pid)` — single-session model, return the current pid.
const SYS_GETPGID: u64 = 121;
/// `getsid(pid)` — single-session model, return the current pid.
const SYS_GETSID: u64 = 124;
/// `setsid()` — single-session model, return the current pid.
const SYS_SETSID: u64 = 112;
/// `setpriority(which, who, prio)` — no scheduler priority model, return 0.
const SYS_SETPRIORITY: u64 = 141;
/// `getpriority(which, who)` — no scheduler priority model, return 0 (nice 0).
const SYS_GETPRIORITY: u64 = 140;
/// `prctl(option, ...)` — minimal: PR_SET_NAME accepts, PR_GET_NAME zeroes the
/// 16-byte user buffer, all other options succeed with 0.
const SYS_PRCTL: u64 = 157;

/// `prctl(PR_SET_NAME, name, …)` — set the (16-byte) thread name. We accept it.
const PR_SET_NAME: u64 = 15;
/// `prctl(PR_GET_NAME, buf, …)` — copy the thread name into a 16-byte user buf.
const PR_GET_NAME: u64 = 16;
/// Length of the kernel thread-name buffer `prctl` reads/writes.
const TASK_COMM_LEN: u64 = 16;

/// `arch_prctl` subfunction: set the `%fs` base (musl TLS).
const ARCH_SET_FS: u64 = 0x1002;
/// `arch_prctl` subfunction: read the `%fs` base back.
const ARCH_GET_FS: u64 = 0x1003;
/// `IA32_FS_BASE` model-specific register: the `%fs` segment base in long mode.
const IA32_FS_BASE: u32 = 0xC000_0100;

// ---- POSIX clock ids (Linux `time.h`) -------------------------------------
const CLOCK_REALTIME: u64 = 0;
const CLOCK_MONOTONIC: u64 = 1;
const CLOCK_PROCESS_CPUTIME_ID: u64 = 2;
const CLOCK_THREAD_CPUTIME_ID: u64 = 3;
const CLOCK_MONOTONIC_RAW: u64 = 4;

const ENOSYS: u64 = (-38i64) as u64;
const EFAULT: u64 = (-14i64) as u64;
const EINVAL: u64 = (-22i64) as u64;
const EBADF: u64 = (-9i64) as u64;
const ENOENT: u64 = (-2i64) as u64;
const ECHILD: u64 = (-10i64) as u64;
const ENOTTY: u64 = (-25i64) as u64;
const MAP_FAILED: u64 = (-1i64) as u64;

/// `clone` flag set by `fork()`: deliver `SIGCHLD` (17) on exit. We support
/// exactly the fork ABI (no new stack, no CLONE_VM).
const SIGCHLD: u64 = 17;
/// `clone` flags that mean "thread" (shared VM); rejected with -EINVAL.
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

// ---------------------------------------------------------------------------
// Per-CPU block the SYSCALL trampoline reads through GS
// ---------------------------------------------------------------------------

/// The per-CPU block `IA32_KERNEL_GS_BASE` points at. SYSCALL does **not** load
/// a kernel stack, so the trampoline does `swapgs` then `mov rsp, gs:0` to land
/// on a known-good kernel stack. `gs:8` is a scratch slot for the user `%rsp`.
///
/// **Field order is load-bearing:** the SYSCALL asm hardcodes `gs:0`
/// (`kernel_rsp`) and `gs:8` (`user_rsp_scratch`); `cpu_index` is **appended**
/// at `gs:16` so those fixed offsets never move. `cpu_index` is this CPU's
/// logical id, read by [`this_cpu_token`] to mint the `hal::cpu::CpuToken` that
/// selects this CPU's per-CPU slot. On the 1-vCPU image it is `0` for the boot
/// core, so per-CPU `current` always indexes slot 0 → golden byte-identical.
#[repr(C, align(64))]
struct PerCpu {
    /// `gs:0` — the kernel stack pointer the trampoline switches to.
    kernel_rsp: u64,
    /// `gs:8` — scratch: the user `%rsp`, saved across the dispatcher call.
    user_rsp_scratch: u64,
    /// `gs:16` — this CPU's logical index (the `hal::cpu::PerCpu` array slot).
    cpu_index: u64,
}

/// One per-CPU block per logical CPU. The BSP installs `&PERCPU[0]` into
/// `IA32_KERNEL_GS_BASE`; APs (S3) install `&PERCPU[k]`. Sized by the HAL's
/// `MAX_CPUS`; on 1-vCPU only `PERCPU[0]` is live.
static mut PERCPU: [PerCpu; hal::cpu::MAX_CPUS] = {
    const ONE: PerCpu = PerCpu {
        kernel_rsp: 0,
        user_rsp_scratch: 0,
        cpu_index: 0,
    };
    [ONE; hal::cpu::MAX_CPUS]
};

/// Mint a [`hal::cpu::CpuToken`] for the CPU we are running on by reading its
/// logical index from the active per-CPU block at `gs:16` (one `mov`). This is
/// the **only new x86 `unsafe` for S1** — a single GS-relative read.
///
/// The active `GS` must be the **kernel** per-CPU base (`&PERCPU[k]`). Every
/// `with_sched` caller guarantees this: the SYSCALL trampoline and the timer-IRQ
/// stub `swapgs` to the kernel base on entry (and back on exit); the
/// `extern "x86-interrupt"` page-fault COW path brackets its `with_sched` call
/// in [`with_kernel_gs`] (swapgs in/out) because the CPU does not swap GS for
/// `int`-style handlers. So at every mint site the active `GS:16` is this CPU's
/// `cpu_index` (always 0 on 1-vCPU → indexes slot 0, golden byte-identical).
///
/// # Safety
/// The caller must hold the no-migration invariant the token encodes — IRQs /
/// preemption masked so we cannot migrate before the token is dropped (the
/// existing trap-path invariant). The active `GS` base must be this CPU's
/// kernel `PERCPU` block, as the callers above ensure.
pub unsafe fn this_cpu_token() -> hal::cpu::CpuToken {
    let idx: u64;
    // SAFETY: `mov reg, gs:[16]` reads the `cpu_index` field of this CPU's
    // per-CPU block (offset 16, after kernel_rsp@0 + user_rsp_scratch@8). A
    // plain read with no side effects; GS is the kernel base at every call site.
    unsafe {
        core::arch::asm!(
            "mov {idx}, gs:[16]",
            idx = out(reg) idx,
            options(nomem, nostack, preserves_flags),
        );
    }
    hal::cpu::CpuToken::new(idx as usize)
}

/// Run `f` with the active `GS` base swapped to this CPU's **kernel** per-CPU
/// block, restoring the prior `GS` afterwards. For trap handlers the CPU enters
/// **without** swapping `GS` (regular interrupt/trap gates do not `swapgs`), so
/// when such a handler needs the scheduler — which mints its `CpuToken` from
/// `gs:16` (see [`this_cpu_token`]) — it must bracket that work in this helper.
///
/// # Safety
/// Must be called from a handler entered **from ring 3** with IRQs masked (the
/// gate cleared IF), so the active `GS` is the user base and `IA32_KERNEL_GS_BASE`
/// holds this CPU's `&PERCPU[k]`. The two `swapgs` are balanced, so `GS` is
/// exactly restored before returning to the faulting context. No IRQ can fire
/// between them (IF=0), so the swap is non-reentrant.
pub(crate) unsafe fn with_kernel_gs<R>(f: impl FnOnce() -> R) -> R {
    // SAFETY: from-ring3, IF=0 handler context; `swapgs` makes the kernel
    // per-CPU base active for the scheduler call, the second restores the user
    // base before iretq. Balanced and non-reentrant (IF masked).
    unsafe {
        core::arch::asm!("swapgs", options(nomem, nostack, preserves_flags));
    }
    let out = f();
    // SAFETY: restore the user GS base swapped out above (balanced pair).
    unsafe {
        core::arch::asm!("swapgs", options(nomem, nostack, preserves_flags));
    }
    out
}

/// Stamp the boot CPU's logical index (`0`) into `PERCPU[0].cpu_index` and
/// install `&PERCPU[0]` into `IA32_KERNEL_GS_BASE`, establishing the per-CPU
/// anchor [`this_cpu_token`] reads. Must run before the first `with_sched`.
///
/// # Safety
/// Boot core, called once before any per-CPU access; writes a unique static and
/// one MSR. APs (S3) will get their own index via a separate AP path.
unsafe fn install_bsp_percpu() {
    // SAFETY: unique mutable access to `PERCPU[0]` on the boot core before the
    // process model is live; index 0 is the BSP slot.
    unsafe {
        (*core::ptr::addr_of_mut!(PERCPU))[0].cpu_index = 0;
    }
    let base = core::ptr::addr_of!(PERCPU) as u64; // == &PERCPU[0]
    // Install `&PERCPU[0]` into BOTH the active GS base and IA32_KERNEL_GS_BASE.
    // - KernelGsBase is what the SYSCALL/timer trampolines `swapgs` to.
    // - The ACTIVE GS base must also be `&PERCPU[0]` so that boot-time ring-0
    //   `with_sched` calls (admit_first, the fd-table proof) — which run before
    //   any `swapgs` — read `cpu_index` (0) from `gs:16` correctly. With both
    //   bases equal to `&PERCPU[0]`, every `swapgs` is a no-op on 1-vCPU and
    //   `gs:16` is always slot 0 → golden byte-identical.
    // SAFETY: `&PERCPU[0]` is a valid, aligned, 'static block; setting both GS
    // bases to it is exactly the per-CPU anchor the scheduler mint reads.
    KernelGsBase::write(x86_64::VirtAddr::new(base));
    GsBase::write(x86_64::VirtAddr::new(base));
}

/// Stamp AP `idx`'s logical index into `PERCPU[idx].cpu_index` and install
/// `&PERCPU[idx]` into BOTH `GsBase` and `KernelGsBase` — the AP analog of
/// [`install_bsp_percpu`] (P4·SMP·S3). After this, `this_cpu_token()` on this AP
/// reads `gs:16 == idx` and indexes this CPU's per-CPU slot.
///
/// # Safety
/// Call once per AP, early in its long-mode entry before any per-CPU access,
/// with `idx < MAX_CPUS` and unique to this physical CPU. Writes a disjoint
/// `PERCPU` slot (no aliasing with other CPUs' slots) and two per-CPU MSRs.
pub(crate) unsafe fn install_ap_percpu(idx: usize) {
    // SAFETY: each AP writes its OWN disjoint `PERCPU[idx]` slot (different idx
    // per CPU), so there is no cross-CPU aliasing of the same slot; the array
    // base pointer arithmetic is in-bounds for `idx < MAX_CPUS`.
    unsafe {
        let percpu = core::ptr::addr_of_mut!(PERCPU) as *mut PerCpu;
        let slot = percpu.add(idx);
        (*slot).cpu_index = idx as u64;
        let base = slot as u64; // == &PERCPU[idx]
        // Install into both GS bases so a (no-op on this AP) swapgs and direct
        // `gs:16` reads both see this AP's block.
        // SAFETY: `slot` is a valid, aligned, 'static per-CPU block.
        KernelGsBase::write(x86_64::VirtAddr::new(base));
        GsBase::write(x86_64::VirtAddr::new(base));
    }
}

/// True once the process model is live (the first process is about to run / is
/// running). Set just before the IRQ0 gate is swapped to the preemption stub;
/// documents the boot-heartbeat -> scheduler-preemption handoff point.
static PROCESS_MODE: AtomicBool = AtomicBool::new(false);

// ---------------------------------------------------------------------------
// SYSCALL entry trampoline (LSTAR target) — builds a Context, dispatches, resumes
// ---------------------------------------------------------------------------
//
// The CPU enters here in ring 0 on a `syscall` instruction with:
//   * `rcx` = user RIP to return to, `r11` = user RFLAGS,
//   * `rax` = syscall number, args in `rdi, rsi, rdx, r10, r8, r9`,
//   * CS/SS = the kernel selectors from STAR, IF cleared by FMASK,
//   * `rsp` STILL the user stack (SYSCALL does not switch stacks).
//
// Unlike the K4 single-process trampoline (which longjmp'd out on exit), the
// process-model trampoline saves the FULL ring-3 register set into a `Context`
// on the kernel stack — matching `process::Context`'s field order — then calls
// the Rust `dispatch(&mut Context)`. The dispatcher may rewrite the Context to a
// *different* process (clone/exit/wait4/yield reschedule). On return the stub
// reloads CS-independent state from the (possibly switched) Context and resumes
// that process via `sysretq` (rcx<-rip, r11<-rflags, rsp<-rsp).
//
// Context field order (matches process::Context, 18 u64 = 144 bytes):
//   rax rbx rcx rdx rsi rdi rbp r8 r9 r10 r11 r12 r13 r14 r15 rip rsp rflags
core::arch::global_asm!(
    r#"
    .section .text, "ax"
    .global __kuberos_syscall_entry
    .type __kuberos_syscall_entry, @function
__kuberos_syscall_entry:
    swapgs                          // GS -> kernel per-CPU base
    mov     gs:8, rsp               // stash user rsp in the per-CPU scratch slot
    mov     rsp, gs:0               // switch to the kernel stack

    // Build a process::Context on the kernel stack (high-to-low push order =
    // rflags, rsp, rip, r15..r8, rbp, rdi, rsi, rdx, rcx, rbx, rax; so the final
    // RSP points at the rax field at offset 0). rcx=user RIP, r11=user RFLAGS,
    // and the stashed user rsp at gs:8 are the ring-3 continuation.
    push    r11                     // [17] rflags (user RFLAGS from r11)
    push    qword ptr gs:8          // [16] rsp    (user rsp from scratch)
    push    rcx                     // [15] rip    (user RIP from rcx)
    push    r15                     // [14]
    push    r14                     // [13]
    push    r13                     // [12]
    push    r12                     // [11]
    push    r11                     // [10] r11 (also stashed as rflags above)
    push    r10                     // [9]
    push    r9                      // [8]
    push    r8                      // [7]
    push    rbp                     // [6]
    push    rdi                     // [5]
    push    rsi                     // [4]
    push    rdx                     // [3]
    push    rcx                     // [2]
    push    rbx                     // [1]
    push    rax                     // [0] rax (syscall number / return value)

    mov     rdi, rsp                // &mut Context
    call    {dispatch}

    // Reload the (possibly switched) context and sysretq back to ring 3.
    pop     rax                     // [0]
    pop     rbx                     // [1]
    pop     rcx                     // [2] (overwritten below by rip for sysret)
    pop     rdx                     // [3]
    pop     rsi                     // [4]
    pop     rdi                     // [5]
    pop     rbp                     // [6]
    pop     r8                      // [7]
    pop     r9                      // [8]
    pop     r10                     // [9]
    pop     r11                     // [10] (overwritten below by rflags)
    pop     r12                     // [11]
    pop     r13                     // [12]
    pop     r14                     // [13]
    pop     r15                     // [14]
    pop     rcx                     // [15] rip -> rcx for sysretq target RIP
    // [16] rsp and [17] rflags remain on the stack; load them, restore user rsp.
    mov     r11, [rsp + 8]          // [17] rflags -> r11 for sysretq
    mov     rsp, [rsp]              // [16] user rsp -> rsp (drops the kernel frame)
    swapgs                          // GS -> user base
    sysretq
"#,
    dispatch = sym dispatch,
);

extern "C" {
    /// The LSTAR target defined in the `global_asm!` above.
    fn __kuberos_syscall_entry();
}

// ---------------------------------------------------------------------------
// Timer IRQ trampoline (preemption) — builds a Context, schedules, resumes
// ---------------------------------------------------------------------------
//
// `interrupts.rs` installs this as the IRQ0 handler *while the process model is
// live* (see `set_preempt_handler`). The CPU enters in ring 0 having pushed the
// hardware interrupt frame (SS, RSP, RFLAGS, CS, RIP) on the kernel stack (the
// CPU loaded RSP0 from the TSS on the ring3->ring0 transition). We save the GPRs
// into a Context laid out identically to the syscall path, copy the hardware
// frame's RIP/RSP/RFLAGS into it, call the Rust preempt hook (which may switch
// processes), then write the (possibly switched) RIP/RSP/RFLAGS back into the
// hardware frame, restore the GPRs, and `iretq` into the chosen process.
core::arch::global_asm!(
    r#"
    .section .text, "ax"
    .global __kuberos_timer_entry
    .type __kuberos_timer_entry, @function
__kuberos_timer_entry:
    cld                             // F-0020: the interrupt gate clears IF/TF but
                                    // NOT DF; the interrupted ring-3 (musl) code may
                                    // have DF=1 (a backward `rep movs` in memmove).
                                    // The SysV AMD64 ABI requires DF=0 on entry to
                                    // the Rust `preempt`/`ack_timer` callees, whose
                                    // compiler-emitted `rep movs`/`stos` (e.g. the
                                    // 144-byte `*frame = ctx` copy) would otherwise
                                    // run BACKWARD and smear the kernel RSP0 stack —
                                    // corrupting a saved return address (the Ring0
                                    // insn-fetch fault at a user-stack RIP). Clear it
                                    // once here, before any Rust runs on either branch.
    // P4·SMP·S4a: this gate is shared by all CPUs. An AP idling in ring-0 `hlt`
    // (between scheduling decisions, before it has a ring-3 process) can take a
    // periodic timer tick HERE — entered from ring 0, NOT ring 3. In that case we
    // must NOT `swapgs` (GS is already the AP's kernel per-CPU base) and must NOT
    // run the ring-3 `preempt` (there is no user context to save); the tick's
    // only job is to wake the AP from `hlt`, which taking the IRQ already did. So
    // branch on the saved CS RPL: ring 0 -> EOI + bare iretq; ring 3 -> the full
    // save/preempt/restore path below. On 1-vCPU the BSP's timer ALWAYS fires
    // from ring 3 (IRQs are masked in every ring-0 window), so this branch is
    // never taken there -> the golden path is byte-identical.
    test    qword ptr [rsp + 8], 3  // (saved CS) & 3 == 0  =>  came from ring 0
    jnz     2f                      // nonzero RPL => ring 3 => normal path
    // --- ring-0 entry (AP idle tick): EOI then bare iretq, no swapgs/preempt ---
    push    rax
    push    rcx
    push    rdx
    call    {ack_timer}
    pop     rdx
    pop     rcx
    pop     rax
    iretq
2:
    swapgs                          // we came from ring 3: GS -> kernel base

    // The CPU already pushed [rip, cs, rflags, rsp, ss] (low->high) at the
    // current rsp. Build the Context below it. We push placeholder rip/rsp/rflags
    // first (filled from the hw frame), then the GPRs in reverse so the final rsp
    // points at the rax field.
    push    qword ptr [rsp + 16]    // [17] rflags (hw frame RFLAGS)
    push    qword ptr [rsp + 32]    // [16] rsp    (hw frame RSP; +32 accounts for
                                    //              the one push just done)
    push    qword ptr [rsp + 16]    // [15] rip    (hw frame RIP; +16 accounts for
                                    //              the two pushes just done)
    push    r15                     // [14]
    push    r14                     // [13]
    push    r13                     // [12]
    push    r12                     // [11]
    push    r11                     // [10]
    push    r10                     // [9]
    push    r9                      // [8]
    push    r8                      // [7]
    push    rbp                     // [6]
    push    rdi                     // [5]
    push    rsi                     // [4]
    push    rdx                     // [3]
    push    rcx                     // [2]
    push    rbx                     // [1]
    push    rax                     // [0]

    mov     rdi, rsp                // &mut Context
    call    {preempt}

    // Write the (possibly switched) rip/rsp/rflags back into the hardware iretq
    // frame, which sits just above the 18-word Context (18*8 = 144 bytes).
    mov     rax, [rsp + 15*8]       // rip
    mov     [rsp + 144 + 0], rax    // hw RIP
    mov     rax, [rsp + 16*8]       // rsp
    mov     [rsp + 144 + 24], rax   // hw RSP
    mov     rax, [rsp + 17*8]       // rflags
    mov     [rsp + 144 + 16], rax   // hw RFLAGS

    // Restore GPRs (skip the 3 trailing rip/rsp/rflags slots; iretq uses the hw
    // frame for those).
    pop     rax                     // [0]
    pop     rbx                     // [1]
    pop     rcx                     // [2]
    pop     rdx                     // [3]
    pop     rsi                     // [4]
    pop     rdi                     // [5]
    pop     rbp                     // [6]
    pop     r8                      // [7]
    pop     r9                      // [8]
    pop     r10                     // [9]
    pop     r11                     // [10]
    pop     r12                     // [11]
    pop     r13                     // [12]
    pop     r14                     // [13]
    pop     r15                     // [14]
    add     rsp, 24                 // drop the [15..17] rip/rsp/rflags scratch

    // EOI/re-arm is now performed by the caps-gated `apic::ack_timer` called at
    // the tail of `preempt` (above), which picks the active timer tier (x2APIC
    // EOI + TSC-deadline re-arm / x2APIC EOI / `out 0x20, al`) once at boot. The
    // trampoline is therefore EOI-agnostic; it only saves/restores state and
    // iretqs. (Pre-P4 this inlined `out 0x20, al` here for the 8259.)
    swapgs                          // GS -> user base (returning to ring 3)
    iretq
"#,
    preempt = sym preempt,
    ack_timer = sym ack_timer_shim,
);

/// `extern "C"` shim around the caps-gated [`crate::apic::ack_timer`] so the
/// ring-0 AP-idle branch of `__kuberos_timer_entry` can EOI the tick that woke it
/// from `hlt` without the full ring-3 preempt path. Issuing the EOI is required
/// (the periodic LVT timer needs it to keep ticking); the rest of `preempt` is
/// not (no user context to preempt while idling in ring 0).
extern "C" fn ack_timer_shim() {
    // SAFETY: timer IRQ context; EOI for the in-service timer vector is the
    // required protocol for every tier (the same call `preempt` makes at its tail).
    unsafe { crate::apic::ack_timer() };
}

extern "C" {
    /// The timer IRQ entry defined in the `global_asm!` above. Installed by
    /// [`crate::interrupts::set_preempt_handler`] when the process model starts.
    pub fn __kuberos_timer_entry();
}

// ---------------------------------------------------------------------------
// SMAP user-access bracket (STAC/CLAC), unchanged from the K4 path
// ---------------------------------------------------------------------------

/// RAII guard that opens an explicit supervisor-access window over user pages
/// (`stac` iff `CR4.SMAP` is set), needed when ring 0 reads the *current*
/// process's user VA (a US page). Reads of another process's memory go through
/// the heap identity alias (kernel pages), which SMAP does not gate.
struct UserAccess {
    armed: bool,
}

impl UserAccess {
    fn open() -> Self {
        let smap = x86_64::registers::control::Cr4::read()
            .contains(x86_64::registers::control::Cr4Flags::SUPERVISOR_MODE_ACCESS_PREVENTION);
        if smap {
            // SAFETY: `stac` sets AC, permitting ring-0 access to US pages. Valid
            // precisely because CR4.SMAP is set; paired with the `clac` in drop.
            unsafe {
                core::arch::asm!("stac", options(nomem, nostack, preserves_flags));
            }
        }
        UserAccess { armed: smap }
    }
}

impl Drop for UserAccess {
    fn drop(&mut self) {
        if self.armed {
            // SAFETY: closes the window opened by the matching `stac`.
            unsafe {
                core::arch::asm!("clac", options(nomem, nostack, preserves_flags));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ELF64 reading helpers
// ---------------------------------------------------------------------------

fn rd_u64(d: &[u8], off: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&d[off..off + 8]);
    u64::from_le_bytes(b)
}
fn rd_u32(d: &[u8], off: usize) -> u32 {
    let mut b = [0u8; 4];
    b.copy_from_slice(&d[off..off + 4]);
    u32::from_le_bytes(b)
}
fn rd_u16(d: &[u8], off: usize) -> u16 {
    let mut b = [0u8; 2];
    b.copy_from_slice(&d[off..off + 2]);
    u16::from_le_bytes(b)
}

/// The parsed pieces of a loaded ELF the stack builder needs.
struct LoadedElf {
    /// `e_entry` — the ring-3 RIP to enter at.
    entry: u64,
    /// User VA of the program-header copy (for `AT_PHDR`).
    phdr_va: u64,
    /// `e_phentsize` (for `AT_PHENT`).
    phentsize: u64,
    /// `e_phnum` (for `AT_PHNUM`).
    phnum: u64,
    /// One past the highest page-rounded VA the loaded image + header copy use.
    image_end: u64,
}

// ---------------------------------------------------------------------------
// Minimal ELF64 loader (operates on a per-process AddressSpace)
// ---------------------------------------------------------------------------

/// Parse `elf` (a static `EM_X86_64` ET_EXEC ELF64), map + copy its `PT_LOAD`
/// segments into `space` with per-segment W^X, zero each `.bss` tail, map the
/// ELF header + program-header table read-only for `AT_PHDR`, and return
/// [`LoadedElf`]. Panics on a malformed image (a fixed build input).
///
/// # Safety
/// `space` must be a fresh/clear address space; writes into its mapped pages.
unsafe fn load_elf(space: &mut AddressSpace, elf: &[u8]) -> LoadedElf {
    let d = elf;
    assert!(d.len() >= 64, "ELF too small");
    assert_eq!(&d[0..4], b"\x7fELF", "bad ELF magic");
    assert_eq!(d[4], 2, "not ELFCLASS64");
    assert_eq!(d[5], 1, "not little-endian (ELFDATA2LSB)");
    assert_eq!(rd_u16(d, 16), 2, "not ET_EXEC (need static non-PIE)");
    assert_eq!(rd_u16(d, 18), 62, "not EM_X86_64");

    let entry = rd_u64(d, 24);
    let phoff = rd_u64(d, 32) as usize;
    let phentsize = rd_u16(d, 54) as usize;
    let phnum = rd_u16(d, 56) as usize;
    let ehsize = rd_u16(d, 52) as usize;

    const PT_LOAD: u32 = 1;
    let mut image_end = USER_BASE as u64;

    for i in 0..phnum {
        let ph = phoff + i * phentsize;
        if rd_u32(d, ph) != PT_LOAD {
            continue;
        }
        let p_flags = rd_u32(d, ph + 4);
        let p_offset = rd_u64(d, ph + 8) as usize;
        let p_vaddr = rd_u64(d, ph + 16);
        let p_filesz = rd_u64(d, ph + 32) as usize;
        let p_memsz = rd_u64(d, ph + 40);

        assert!(p_vaddr >= USER_BASE as u64, "segment below user base");
        assert!(
            p_vaddr + p_memsz <= USER_TOP as u64,
            "segment exceeds user window"
        );

        let perm = Perm::from_pflags(p_flags);

        // Copy the file image (maps the pages it touches), then map+zero the
        // [p_filesz, p_memsz) .bss tail. Writes go through each frame's heap
        // identity alias (no SMAP bracket needed: heap frames are kernel pages).
        if p_filesz > 0 {
            // SAFETY: range checked to lie in the window; maps + copies.
            unsafe { space.copy_to_user(p_vaddr as usize, &d[p_offset..p_offset + p_filesz], perm) };
        }
        let seg_end = p_vaddr + p_memsz;
        let mut va = (p_vaddr + p_filesz as u64) & !(PAGE_SIZE as u64 - 1);
        while va < seg_end {
            // SAFETY: in-window; ensures a zeroed frame with `perm`.
            unsafe { space.map_page(va as usize, perm) };
            va += PAGE_SIZE as u64;
        }

        let end = (seg_end + PAGE_SIZE as u64 - 1) & !(PAGE_SIZE as u64 - 1);
        if end > image_end {
            image_end = end;
        }
    }

    // Map the ELF header + program-header table read-only for AT_PHDR, on a fresh
    // page above the image.
    let phdrs_end = phoff + phnum * phentsize;
    let hdr_span = core::cmp::max(ehsize, phdrs_end);
    let phdr_page_va = image_end as usize;
    // SAFETY: maps pages on demand within the window and copies the header span.
    unsafe { space.copy_to_user(phdr_page_va, &d[0..hdr_span], Perm::ReadOnly) };
    let phdr_va = (phdr_page_va + phoff) as u64;
    let header_end = ((phdr_page_va + hdr_span) as u64 + PAGE_SIZE as u64 - 1)
        & !(PAGE_SIZE as u64 - 1);

    LoadedElf {
        entry,
        phdr_va,
        phentsize: phentsize as u64,
        phnum: phnum as u64,
        image_end: header_end,
    }
}

// ---------------------------------------------------------------------------
// Linux/SysV x86_64 initial process stack (reuses the shared `user_layout`)
// ---------------------------------------------------------------------------

const ARGV0: &[u8] = b"/init\0";
const ENV0: &[u8] = b"PATH=/usr/bin\0";
const RANDOM16: [u8; 16] = [
    0x9e, 0x37, 0x79, 0xb9, 0x7f, 0x4a, 0x7c, 0x15, 0xf3, 0x9c, 0xc0, 0x60, 0x5c, 0xed, 0xc8, 0x34,
];

/// Build the Linux/SysV x86_64 initial process stack in `space` and return the
/// final 16-byte-aligned ring-3 `%rsp` (pointing at `argc`).
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
    assert!(img.sp as u64 >= elf.image_end, "stack underran the loaded image");

    // SAFETY: every VA the builder produced lies inside the stack region;
    // `copy_to_user` maps pages on demand and writes through the heap alias.
    unsafe {
        space.copy_to_user(img.argv_vas.as_slice()[0], ARGV0, Perm::ReadWrite);
        space.copy_to_user(img.envp_vas.as_slice()[0], ENV0, Perm::ReadWrite);
        space.copy_to_user(img.random_va, &RANDOM16, Perm::ReadWrite);
        let words = img.words.as_slice();
        let word_bytes = core::slice::from_raw_parts(words.as_ptr() as *const u8, words.len() * 8);
        space.copy_to_user(img.sp, word_bytes, Perm::ReadWrite);
    }

    // Pre-map the rest of the stack region down to STACK_BOTTOM (no demand-paging
    // for ring-3 stack growth yet) — mirrors aarch64's `build_initial_stack`. The
    // x86_64 page-fault handler only resolves CoW (present + write) faults, not
    // not-present stack-growth faults, so without this the real talos-init's
    // deeper stack use (e.g. the LoadConfig YAML parse) would fault on an
    // unmapped stack page below the initially-touched top. Pre-mapping the full
    // 256 KiB stack is invisible to EL0 behaviour (no new syscalls), so the
    // golden/default path is unaffected.
    let mut va = USER_STACK_BOTTOM;
    while va < img.sp as usize {
        // SAFETY: in-window; ensures a zeroed RW frame in this space.
        unsafe { space.map_page(va, Perm::ReadWrite) };
        va += PAGE_SIZE;
    }

    img.sp as u64
}

/// Load `elf_bytes` into a fresh address space and return a fully-initialised,
/// Runnable [`Process`] (pid 0 placeholder) whose saved context resumes at the
/// program entry with a SysV stack and a clean ring-3 RFLAGS.
///
/// # Safety
/// Builds page tables and copies into user memory; boot core, paging up.
unsafe fn build_process(ppid: u32, elf_bytes: &[u8]) -> Process {
    // SAFETY: boot core; fresh kernel-half-shared space.
    let mut space = unsafe { AddressSpace::new() };
    // SAFETY: fresh space; the loader maps + copies into it.
    let elf = unsafe { load_elf(&mut space, elf_bytes) };
    // SAFETY: builds the initial stack image in the space.
    let sp = unsafe { build_initial_stack(&mut space, &elf) };

    // SAFETY: boot core; takes ownership of the prepared space.
    let mut proc = unsafe { Process::new_loaded(ppid, space) };
    proc.brk_cur = elf.image_end as usize;
    proc.mmap_cur = MMAP_BASE;
    // Wire fds 0/1/2 -> /dev/console for the freshly-loaded image. (On execve
    // this fresh table is discarded in favour of preserving the live process's
    // fds — see `sys_execve` — so only the very first process actually adopts
    // these; but building them here keeps `build_process` self-contained.)
    proc.init_std_fds();
    // Initial ring-3 context: entry in RIP, SysV stack in RSP, x0..= 0, and a
    // clean RFLAGS (bit 1 reserved-1 + IF set so IRQs stay on in ring 3).
    proc.ctx = Context::zeroed();
    proc.ctx.rip = elf.entry;
    proc.ctx.rsp = sp;
    proc.ctx.rflags = (1 << 1) | (1 << 9);
    proc.state = State::Runnable;
    proc
}

// ---------------------------------------------------------------------------
// Syscall dispatch outcome (mirrors aarch64's SyscallOutcome)
// ---------------------------------------------------------------------------

/// What the syscall dispatch wants the trampoline to do, carried out-of-band
/// from the user-visible return value because process-model calls manipulate the
/// run queue rather than simply returning a number.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SyscallOutcome {
    /// Write `ret` into the current process's `rax` and resume it.
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
    /// `complete_waits`, which may have already written the wake's `rax`
    /// (reaped-child pid) into this process's `ctx` — clobbering it back to the
    /// pre-block frame and corrupting the `wait4` return (premature/garbage
    /// ECHILD). Only `wait4`'s blocking path uses this.
    BlockedReschedule,
    /// The live frame has *already* been rewritten to the context that should
    /// run next (e.g. `execve` reset the current process in place); resume it
    /// directly without saving or re-scheduling.
    Resume,
}

// ---------------------------------------------------------------------------
// Syscall dispatcher (mirrors aarch64 `handle_svc` semantics)
// ---------------------------------------------------------------------------

/// Dispatch one ring-3 syscall. `frame` is the live [`Context`] the trampoline
/// built; `frame.rax` is the Linux nr, args in `rdi,rsi,rdx,r10,r8,r9` (the
/// Linux register order; r10 is the 4th arg, not rcx). On return the trampoline
/// resumes from `frame` (possibly a different process after a reschedule).
///
/// Called from the SYSCALL trampoline with `&mut Context` in `rdi`; runs in ring
/// 0 on the kernel stack with the kernel GS active.
extern "C" fn dispatch(frame: &mut Context) {
    let num = frame.rax;
    let (a0, a1, a2, a3, a4, _a5) =
        (frame.rdi, frame.rsi, frame.rdx, frame.r10, frame.r8, frame.r9);

    let outcome: SyscallOutcome = match num {
        SYS_WRITE => SyscallOutcome::Return(sys_write(a0, a1, a2)),
        // SAFETY: validates the iovec array + each buffer before reading.
        SYS_WRITEV => SyscallOutcome::Return(unsafe { sys_writev(a0, a1, a2) }),
        SYS_READ => SyscallOutcome::Return(sys_read(a0, a1, a2)),
        SYS_READLINKAT => SyscallOutcome::Return(EINVAL),
        SYS_IOCTL => SyscallOutcome::Return(ENOTTY),
        // SAFETY: copies the path string from userspace via the bounds-checked,
        // SMAP-bracketed user-access path before resolving it. (path = arg1.)
        SYS_OPENAT => SyscallOutcome::Return(unsafe { sys_openat(a1) }),
        // open(path=a0, flags=a1, mode=a2): legacy 2-arg open the x86_64 musl
        // std uses; resolve via the same VFS path as openat. SAFETY: copies the
        // NUL-terminated path from userspace via the bounds-checked,
        // SMAP-bracketed path before resolving it.
        SYS_OPEN => SyscallOutcome::Return(unsafe { sys_openat(a0) }),
        SYS_FACCESSAT => SyscallOutcome::Return(ENOENT),
        // mount(source=a0, target=a1, fstype=a2, flags=a3, data=a4). Slice 1:
        // record the pseudo-fs mount in the in-RAM VFS and return 0 (data a4
        // ignored). SAFETY: copies the three NUL-terminated strings out of user
        // memory via the bounds-checked, SMAP-bracketed byte-copy path first.
        SYS_MOUNT => SyscallOutcome::Return(unsafe { sys_mount(a0, a1, a2, a3) }),
        // mkdir(path=a0, mode=a1): the form talos-init/musl issues on x86_64.
        // Slice 2: create the dir node idempotently and return 0. SAFETY: copies
        // the NUL-terminated path out of user memory via the bounds-checked,
        // SMAP-bracketed path.
        SYS_MKDIR => SyscallOutcome::Return(unsafe { sys_mkdir(a0) }),
        // mkdirat(dirfd=a0, path=a1, mode=a2): the same, dirfd ignored.
        SYS_MKDIRAT => SyscallOutcome::Return(unsafe { sys_mkdir(a1) }),
        SYS_CLOSE => SyscallOutcome::Return(sys_close(a0)),
        // ---- M2 network slice: minimal AF_NETLINK link-status dump ----
        // socket(domain=a0, type=a1, protocol=a2): an AF_NETLINK/NETLINK_ROUTE
        // socket allocates a fresh Netlink fd; any other domain → -EAFNOSUPPORT.
        SYS_SOCKET => SyscallOutcome::Return(sys_socket(a0, a1, a2)),
        // bind(fd=a0, addr=a1, addrlen=a2): validate the sockaddr_nl → 0.
        // SAFETY: copies the 12-byte sockaddr_nl out of user memory via the
        // bounds-checked, SMAP-bracketed copy path before reading it.
        SYS_BIND => SyscallOutcome::Return(unsafe { sys_bind(a0, a1, a2) }),
        // sendto(fd=a0, buf=a1, len=a2, flags=a3, addr=a4, addrlen=a5): copy the
        // request bytes, parse the RTM_GETLINK dump, arm the response. SAFETY:
        // validates + SMAP-copies the flat send buffer before parsing it.
        SYS_SENDTO => SyscallOutcome::Return(unsafe { sys_sendto(a0, a1, a2) }),
        // recvfrom(fd=a0, buf=a1, len=a2, flags=a3, addr=a4, addrlen=a5): drain
        // the armed NLMSG_DONE into the user buffer; write a sockaddr_nl into
        // `addr` if non-NULL. SAFETY: validates + SMAP-copies into the flat user
        // buffer (and the optional src-addr) before writing.
        SYS_RECVFROM => SyscallOutcome::Return(unsafe { sys_recvfrom(a0, a1, a2, a4, _a5) }),
        SYS_LSEEK => SyscallOutcome::Return((-29i64) as u64),
        SYS_DUP => SyscallOutcome::Return(a0),
        SYS_DUP2 => SyscallOutcome::Return(sys_dup3(a0, a1)),
        SYS_DUP3 => SyscallOutcome::Return(sys_dup3(a0, a1)),
        SYS_FCNTL => SyscallOutcome::Return(0),
        SYS_GETCWD => SyscallOutcome::Return(sys_getcwd(a0, a1)),
        SYS_NEWFSTATAT => SyscallOutcome::Return(sys_fstat(a2)),
        SYS_FSTAT => SyscallOutcome::Return(sys_fstat(a1)),
        SYS_GETPID => SyscallOutcome::Return(current_pid() as u64),
        SYS_GETPPID => SyscallOutcome::Return(current_ppid() as u64),
        SYS_GETTID => SyscallOutcome::Return(current_pid() as u64),
        SYS_GETUID | SYS_GETEUID | SYS_GETGID | SYS_GETEGID => SyscallOutcome::Return(0),
        SYS_SET_TID_ADDRESS => SyscallOutcome::Return(current_pid() as u64),
        SYS_SET_ROBUST_LIST => SyscallOutcome::Return(0),
        SYS_FUTEX => SyscallOutcome::Return(0),
        SYS_RSEQ => SyscallOutcome::Return(ENOSYS),
        SYS_GETRANDOM => SyscallOutcome::Return(sys_getrandom(a0, a1)),
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
        // poll(fds, nfds, timeout_ms): legacy poll; same fd-readiness marking as
        // ppoll, but its timeout is a plain int (ms) we treat as "no wait" (the
        // fds we mark are immediately ready). musl uses poll at stdio setup.
        SYS_POLL => SyscallOutcome::Return(sys_ppoll(a0, a1, 0)),
        SYS_SCHED_GETAFFINITY => SyscallOutcome::Return(sys_sched_getaffinity(a1, a2)),
        SYS_PRLIMIT64 => SyscallOutcome::Return(0),
        // `arch_prctl(ARCH_SET_FS, addr)` programs `%fs` base for musl TLS; x86_64
        // has no aarch64 analogue (aarch64 sets TPIDR_EL0 at process build).
        // SAFETY: writes the IA32_FS_BASE MSR / reads back through a validated buf.
        SYS_ARCH_PRCTL => SyscallOutcome::Return(unsafe { sys_arch_prctl(a0, a1) }),
        SYS_BRK => SyscallOutcome::Return(sys_brk(a0)),
        // mmap(addr, len, prot, flags, fd, off): fd is arg5 = r8 = a4 (NOT r10).
        SYS_MMAP => SyscallOutcome::Return(sys_mmap(a1, a4)),
        SYS_MUNMAP => SyscallOutcome::Return(0),
        SYS_MREMAP => SyscallOutcome::Return(MAP_FAILED),
        SYS_MPROTECT => SyscallOutcome::Return(0),
        SYS_SCHED_YIELD => SyscallOutcome::Reschedule,

        // ---- POSIX signals (real delivery; see `sig::*` helpers + the
        // return-to-user `deliver_pending_signals` hook below). The 4th arg
        // (`sigsetsize`) is `a3` = r10, per the Linux register order. ----
        // SAFETY: validate sigsetsize + read/write the user structs through the
        // bounds-checked, SMAP-bracketed user-access path before any access.
        SYS_RT_SIGACTION => SyscallOutcome::Return(unsafe { sys_rt_sigaction(a0, a1, a2, a3) }),
        SYS_RT_SIGPROCMASK => {
            SyscallOutcome::Return(unsafe { sys_rt_sigprocmask(a0, a1, a2, a3) })
        }
        SYS_RT_SIGPENDING => SyscallOutcome::Return(unsafe { sys_rt_sigpending(a0, a1) }),
        // rt_sigreturn restores the pre-signal context from the user frame; it
        // returns `Resume` (no trace line, no rax overwrite).
        // SAFETY: bounds-checks the frame before restoring it.
        SYS_RT_SIGRETURN => unsafe { sys_rt_sigreturn(frame) },
        SYS_SIGALTSTACK => SyscallOutcome::Return(unsafe { sys_sigaltstack(a0, a1) }),
        SYS_KILL => SyscallOutcome::Return(sys_kill(a0, a1)),
        SYS_TKILL => SyscallOutcome::Return(sys_kill(a0, a1)),
        SYS_TGKILL => SyscallOutcome::Return(sys_tgkill(a0, a1, a2)),

        // ---- P3 timekeeping (mirror aarch64; see `timekeeping.rs`) ----
        // clock_gettime(clockid=a0, *ts=a1) — honours the clockid, writes a
        // timespec through the validated, SMAP-bracketed user-access path.
        SYS_CLOCK_GETTIME => SyscallOutcome::Return(sys_clock_gettime(a0, a1)),
        // gettimeofday(*tv=a0, *tz=a1) — REALTIME timeval; tz ignored.
        SYS_GETTIMEOFDAY => SyscallOutcome::Return(sys_gettimeofday(a0)),
        // nanosleep(*req=a0, *rem=a1) — relative sleep on the real TSC.
        SYS_NANOSLEEP => SyscallOutcome::Return(sys_nanosleep(a0, a1)),
        // clock_nanosleep(clockid=a0, flags=a1, *req=a2, *rem=a3): the 4th arg is
        // r10 = a3 (Linux register order). v1 is relative-only (clockid/flags
        // dropped, like aarch64); req=a2, rem=a3.
        SYS_CLOCK_NANOSLEEP => SyscallOutcome::Return(sys_nanosleep(a2, a3)),

        // ---- process model ----
        SYS_CLONE => SyscallOutcome::Return(sys_clone(frame, a0, a1)),
        // fork() == clone(SIGCHLD, 0, …): reuse the same COW fork path with the
        // fork ABI hard-coded (ignore fork's empty arg list). Unblocks x86_64
        // talos StartServices, which spawns the real svc via fork+execve.
        SYS_FORK => SyscallOutcome::Return(sys_clone(frame, SIGCHLD, 0)),
        // SAFETY: validates the path; rebuilds the image in place.
        SYS_EXECVE => unsafe { sys_execve(frame, a0) },
        SYS_WAIT4 => sys_wait4(frame, a0, a1, a2),
        // SAFETY: `frame` is the live frame; exit switches it to the next process.
        SYS_EXIT | SYS_EXIT_GROUP => unsafe { sys_exit(frame, a0 as i32) },

        // ---- shutdown ----
        // sync(): we have no writeback caches, so flushing is a no-op -> 0.
        SYS_SYNC => SyscallOutcome::Return(0),
        // reboot(magic1, magic2, cmd, arg): on RB_POWER_OFF this never returns
        // (powers the machine off -> clean QEMU exit); other commands return.
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
            frame.rax = ret;
        }
        SyscallOutcome::Reschedule => {
            process::save_current(frame);
            // SAFETY: `frame` is the live frame; `schedule` rewrites it to the
            // next runnable process.
            unsafe {
                if !process::schedule(frame) {
                    // Nothing runnable for THIS CPU: power off (BSP, empty table)
                    // or (re-)enter the ring-0 idle loop (work alive elsewhere).
                    idle_or_finish();
                }
            }
        }
        SyscallOutcome::BlockedReschedule => {
            // The blocking syscall ALREADY saved this process's context into its
            // `ctx` under the scheduler lock (atomic with the `Waiting` mark), so
            // we must NOT `save_current` here — a sibling CPU's `complete_waits`
            // may already have published the wake's `rax` into that `ctx`, and a
            // re-save would clobber it (the `wait4`-FAILED race). Just pick next.
            // SAFETY: `frame` is the live frame; `schedule` rewrites it to the
            // next runnable process (or returns false → idle/finish).
            unsafe {
                if !process::schedule(frame) {
                    idle_or_finish();
                }
            }
        }
        SyscallOutcome::Resume => {}
    }

    // Last step before resuming ring 3: deliver one pending, unblocked signal to
    // the now-current process (rewriting `frame` to enter its handler). The
    // single per-arch delivery site for the syscall path.
    // SAFETY: `frame` is the live Context the trampoline `sysretq`s from;
    // `deliver_pending_signals` only rewrites it to a validated handler frame in
    // the current process's ring-3 window (or terminates the process).
    unsafe {
        deliver_pending_signals(frame);
    }
}

/// Trace one syscall as `[pid <n>] syscall <nr> -> <ret>`, matching the aarch64
/// K7 format so both arches read identically.
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
/// absent → `-EBADF`; `/dev/console` copies a validated ring-3 buffer (the
/// current process's user VA, read inside an `STAC`/`CLAC` SMAP window) to the
/// 16550 console — byte-for-byte identical to the pre-fd-table path so the
/// behaviour is unchanged; `/dev/null` discards and reports `len`. Out-of-window
/// → `EFAULT`.
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
            let _smap = UserAccess::open();
            // SAFETY: `[buf, buf+len)` was validated to lie inside a mapped user
            // page of the current (live-CR3) process, readable by ring 0 inside
            // the STAC/CLAC window.
            let bytes = unsafe { core::slice::from_raw_parts(buf as *const u8, len as usize) };
            for &b in bytes {
                crate::console::_print(format_args!("{}", b as char));
            }
            len
        }
        // /dev/null: the buffer was validated (EFAULT parity); discard it.
        FileKind::Null => len,
        // A netlink fd is written via sendto (not write); a bare write() just
        // reports the bytes "accepted" (the talos dump path never takes this).
        FileKind::Netlink => len,
    }
}

/// `read(fd, buf, len)` routed through the fd table. Absent fd → `-EBADF`. An fd
/// opened against a VFS `File` node (Slice 3: baked `/machine-config.yaml`,
/// `/proc/cmdline`, …) copies `node.data` from the per-fd offset into the user
/// buffer (inside the STAC/CLAC SMAP window) and advances the offset, so
/// successive reads stream the file and a final read returns 0 (EOF). Console
/// and Null return 0 (EOF) — no input source yet — exactly as before, so the
/// behaviour on the golden/default path is unchanged.
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
        let mut staging = [0u8; READ_CHUNK];
        let want = core::cmp::min(len as usize, READ_CHUNK);
        let n = process::with_vfs(|v| v.read_at(node, off, &mut staging[..want]));
        if n == 0 {
            return 0;
        }
        {
            let _smap = UserAccess::open();
            // SAFETY: `[buf, buf+n)` ⊆ `[buf, buf+len)` was validated above to
            // lie inside a mapped ring-3 page of the current (live-CR3) process,
            // writable by ring 0 inside the STAC/CLAC SMAP window.
            let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, n) };
            dst.copy_from_slice(&staging[..n]);
        }
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
/// synthetic files are tiny; std loops to EOF, so a per-call cap is sufficient.
const READ_CHUNK: usize = 4096;

// ---------------------------------------------------------------------------
// P3 timekeeping syscalls (mirror aarch64; the real clock is `timekeeping.rs`)
// ---------------------------------------------------------------------------

/// Resolve a clockid to nanoseconds against the real TSC timekeeper, or `None`
/// for an unknown clock (caller returns `-EINVAL`). Mirrors aarch64:
///   * MONOTONIC / MONOTONIC_RAW -> `mono_ns`
///   * REALTIME                  -> `mono_ns + WALLCLOCK_OFFSET_NS`
///   * PROCESS/THREAD_CPUTIME_ID -> `mono_ns` (single process, no CPU accounting)
fn clock_ns(clk: u64) -> Option<u64> {
    match clk {
        CLOCK_MONOTONIC | CLOCK_MONOTONIC_RAW | CLOCK_PROCESS_CPUTIME_ID
        | CLOCK_THREAD_CPUTIME_ID => Some(crate::timekeeping::mono_ns()),
        CLOCK_REALTIME => Some(crate::timekeeping::real_ns()),
        _ => None,
    }
}

/// Write two `i64`s (a `timespec`/`timeval`) into the validated 16-byte user
/// buffer `dst`, through the STAC/CLAC supervisor-access window. `dst` must
/// already be `user_range_ok(dst, 16)`-validated by the caller.
///
/// # Safety
/// `dst..dst+16` lies inside a mapped, writable user page of the current
/// (live-CR3) process; the write happens inside the `UserAccess` SMAP bracket.
unsafe fn write_two_i64(dst: u64, a: i64, b: i64) {
    let _smap = UserAccess::open();
    // SAFETY: validated 16-byte RW user buffer of the current process, writable
    // by ring 0 inside the STAC/CLAC window.
    unsafe {
        (dst as *mut i64).write(a);
        ((dst + 8) as *mut i64).write(b);
    }
}

/// `clock_gettime(clk, ts)` — honours `clk` (see [`clock_ns`]), reads the real
/// TSC timekeeper, and writes a `timespec` (16 bytes) into the validated user
/// buffer. Unknown clock -> `-EINVAL`. Returns the constant `0` on success (the
/// time goes into the user buffer, never the trace — so the golden is unaffected).
fn sys_clock_gettime(clk: u64, ts: u64) -> u64 {
    let ns = match clock_ns(clk) {
        Some(ns) => ns,
        None => return EINVAL,
    };
    if !user_range_ok(ts, 16) {
        return EFAULT;
    }
    let (secs, nsec) = user_layout::timekeep::ns_to_timespec(ns);
    // SAFETY: validated 16-byte RW user buffer; written inside the SMAP bracket.
    unsafe { write_two_i64(ts, secs, nsec) };
    0
}

/// `gettimeofday(tv, _tz)` — REALTIME wall clock split into a `timeval`
/// (`tv_sec`, `tv_usec`) written to the validated 16-byte user buffer. NULL `tv`
/// is a harmless success; the timezone pointer is ignored.
fn sys_gettimeofday(tv: u64) -> u64 {
    if tv == 0 {
        return 0;
    }
    if !user_range_ok(tv, 16) {
        return EFAULT;
    }
    let (secs, usec) = user_layout::timekeep::ns_to_timeval(crate::timekeeping::real_ns());
    // SAFETY: validated 16-byte RW user buffer; written inside the SMAP bracket.
    unsafe { write_two_i64(tv, secs, usec) };
    0
}

/// Read + validate a user `timespec` at `req`, returning the number of TSC
/// cycles to wait (`Ok`) or an errno (`Err`). Mirrors aarch64's
/// `read_timespec_cycles`: validates the 16-byte buffer, reads `(tv_sec, tv_nsec)`
/// through the SMAP bracket, then uses the host-tested `timespec_to_cycles` with
/// the calibrated TSC frequency.
fn read_timespec_cycles(req: u64) -> Result<u64, u64> {
    if !user_range_ok(req, 16) {
        return Err(EFAULT);
    }
    let (tv_sec, tv_nsec) = {
        let _smap = UserAccess::open();
        // SAFETY: validated 16-byte readable user buffer of the current process,
        // read inside the STAC/CLAC window.
        unsafe {
            (
                (req as *const i64).read(),
                ((req + 8) as *const i64).read(),
            )
        }
    };
    match user_layout::timespec_to_cycles(tv_sec, tv_nsec, crate::timekeeping::tsc_hz()) {
        user_layout::SleepCycles::Wait(c) => Ok(c),
        user_layout::SleepCycles::Invalid => Err(EINVAL),
    }
}

/// `nanosleep(req, rem)` / `clock_nanosleep(_, _, req, rem)` — a relative sleep on
/// the real TSC. Converts the user `timespec` to TSC cycles, busy-waits on the
/// counter until the deadline, then writes a zero remaining time if `rem` is set.
/// v1 is relative-only (TIMER_ABSTIME/clockid dropped, like aarch64).
fn sys_nanosleep(req: u64, rem: u64) -> u64 {
    let cycles = match read_timespec_cycles(req) {
        Ok(c) => c,
        Err(e) => return e,
    };
    if cycles > 0 {
        let deadline = user_layout::deadline_after(crate::timekeeping::now_counter(), cycles);
        crate::timekeeping::sleep_until(deadline);
    }
    if rem != 0 && user_range_ok(rem, 16) {
        // SAFETY: validated 16-byte RW user buffer; written inside the bracket.
        unsafe { write_two_i64(rem, 0, 0) };
    }
    0
}

/// `writev(fd, iov, iovcnt)` routed through the fd table. Same fd semantics as
/// `write`: Console writes each iovec to the console, Null discards; both report
/// the total byte count. Each `iov_base/iov_len` pair and buffer is validated
/// before the SMAP-bracketed read.
///
/// # Safety
/// Reads the iovec array and each buffer from the current process's user memory;
/// all ranges are validated first and the reads are STAC/CLAC bracketed.
unsafe fn sys_writev(fd: u64, iov: u64, iovcnt: u64) -> u64 {
    let kind = match fd_kind(fd) {
        Some(k) => k,
        None => return EBADF,
    };
    if iovcnt == 0 {
        return 0;
    }
    let bytes_each = 16u64; // struct iovec { void *base; size_t len; }
    if !user_range_ok(iov, iovcnt.saturating_mul(bytes_each)) {
        return EFAULT;
    }
    let to_console = matches!(kind, FileKind::Console);
    let mut total: u64 = 0;
    for i in 0..iovcnt {
        let ent = iov + i * bytes_each;
        let (base, l) = {
            let _smap = UserAccess::open();
            // SAFETY: `ent`/`ent+8` lie within the validated iovec array of the
            // current (live-CR3) process, readable inside the SMAP window.
            let base = unsafe { (ent as *const u64).read() };
            let l = unsafe { ((ent + 8) as *const u64).read() };
            (base, l)
        };
        if l == 0 {
            continue;
        }
        if !user_range_ok(base, l) {
            return EFAULT;
        }
        if to_console {
            let _smap = UserAccess::open();
            // SAFETY: validated ring-3 buffer of the current process, read inside
            // the STAC/CLAC window.
            let bytes = unsafe { core::slice::from_raw_parts(base as *const u8, l as usize) };
            for &b in bytes {
                crate::console::_print(format_args!("{}", b as char));
            }
        }
        // /dev/null discards but still counts the bytes as written.
        total += l;
    }
    total
}

/// Resolve a userspace path string to a minimal devtmpfs [`FileKind`]: a
/// fixed-size copy of the NUL-terminated path is taken from the current
/// process's ring-3 memory through the bounds-checked, SMAP-bracketed read path
/// (never a raw deref of the user pointer), then matched. `/dev/console` and
/// `/dev/null` resolve; anything else → `None` (→ `-ENOENT`).
/// Maximum NUL-terminated user path/string length we copy across the ring-3
/// boundary. The pseudo-fs names + demo paths are tiny.
const MAXP: usize = 64;

/// Copy a NUL-terminated string from user VA `addr` into `buf`, returning the
/// byte length copied (excluding the NUL), or `None` on a NULL pointer / an
/// unmapped byte. Each byte's address is validated before the read inside the
/// STAC/CLAC SMAP window; never a raw deref of an unvalidated user pointer.
/// Shared by [`resolve_path`] and [`sys_mount`].
fn copy_user_cstr(addr: u64, buf: &mut [u8; MAXP]) -> Option<usize> {
    if addr == 0 {
        return None;
    }
    let mut n = 0usize;
    while n < MAXP {
        let a = addr + n as u64;
        if !user_range_ok(a, 1) {
            return None;
        }
        let b = {
            let _smap = UserAccess::open();
            // SAFETY: `a` is a validated 1-byte ring-3 address in the current
            // (live-CR3) process, readable inside the STAC/CLAC SMAP window.
            unsafe { (a as *const u8).read() }
        };
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
/// bytes are served via `FileDesc.node` once `sys_read` routes File reads) and a
/// `Dir` is not openable as a stream here, so the caller treats it as a miss.
fn nodekind_to_filekind(k: vfs::NodeKind) -> Option<(FileKind, bool)> {
    use vfs::NodeKind;
    match k {
        NodeKind::Console => Some((FileKind::Console, false)),
        NodeKind::Null => Some((FileKind::Null, false)),
        NodeKind::File => Some((FileKind::Null, true)),
        NodeKind::Dir => None,
    }
}

/// Resolve a userspace path string against the in-RAM VFS tree to a
/// `(FileKind, node)` pair: a fixed-size copy of the NUL-terminated path is
/// taken from the current process's ring-3 memory through the bounds-checked,
/// SMAP-bracketed read path (never a raw deref), then walked. `/dev/console` and
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
/// SMAP-bracketed [`resolve_path`]; never raw-derefs an unvalidated user pointer.
unsafe fn sys_openat(path: u64) -> u64 {
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
/// via the bounds-checked, SMAP-bracketed [`copy_user_cstr`]; never raw-derefs
/// an unvalidated user pointer.
unsafe fn sys_mount(source: u64, target: u64, fstype: u64, flags: u64) -> u64 {
    let mut sbuf = [0u8; MAXP];
    let mut tbuf = [0u8; MAXP];
    let mut fbuf = [0u8; MAXP];
    let tn = match copy_user_cstr(target, &mut tbuf) {
        Some(n) => n,
        None => return EFAULT,
    };
    let sn = copy_user_cstr(source, &mut sbuf).unwrap_or(0);
    let fn_ = copy_user_cstr(fstype, &mut fbuf).unwrap_or(0);
    process::with_vfs(|v| {
        let _ = v.do_mount(&sbuf[..sn], &tbuf[..tn], &fbuf[..fn_], flags);
    });
    0
}

/// `mkdir(path, mode)` / `mkdirat(_, path, _)` — Slice 2 in-RAM tmpfs `mkdir`.
/// Copies the NUL-terminated `path` out of user memory and creates the directory
/// (and any missing parents) in the global VFS via `mkdir_p`, returning **0
/// idempotently** (an already-existing directory is success, NOT `-EEXIST`,
/// because talos's `make_directory` ⇒ `create_dir_all` expects `mkdir -p`
/// semantics and has no `EEXIST` tolerance). This is what lets the real
/// talos-init's SystemDirectories phase (`/system`, `/system/state`,
/// `/system/run`, `/var`, `/run`, `/tmp`) complete instead of aborting on
/// `-ENOSYS`. `mode` (and `dirfd` for the `mkdirat` form) are ignored. A
/// NULL/unmapped path → `-EFAULT`; a non-dir component collision → `-ENOTDIR`.
///
/// # Safety
/// Reads the NUL-terminated `path` from userspace via the bounds-checked,
/// SMAP-bracketed [`copy_user_cstr`]; never raw-derefs an unvalidated user pointer.
unsafe fn sys_mkdir(path: u64) -> u64 {
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

use netlink as nl;

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
/// Reads the 12-byte `sockaddr_nl` from userspace via the bounds-checked,
/// SMAP-bracketed copy path; never raw-derefs an unvalidated user pointer.
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
    {
        let _smap = UserAccess::open();
        // SAFETY: `[addr, addr+12)` validated above to lie inside a mapped ring-3
        // page of the current (live-CR3) process, readable inside the STAC/CLAC
        // window.
        let src = unsafe { core::slice::from_raw_parts(addr as *const u8, nl::SOCKADDR_NL_LEN) };
        sa.copy_from_slice(src);
    }
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
/// Reads the flat `[buf, buf+min(len,cap))` send buffer from userspace via the
/// bounds-checked, SMAP-bracketed copy path; never raw-derefs an unvalidated
/// user pointer.
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
    {
        let _smap = UserAccess::open();
        // SAFETY: `[buf, buf+copy_len)` ⊆ a validated mapped ring-3 page of the
        // current (live-CR3) process, readable inside the STAC/CLAC window.
        let src = unsafe { core::slice::from_raw_parts(buf as *const u8, copy_len) };
        staging[..copy_len].copy_from_slice(src);
    }
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
/// Writes the response bytes (and the optional `sockaddr_nl`) into the
/// bounds-checked, SMAP-bracketed user buffers; never raw-derefs an unvalidated
/// user pointer.
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
    // Copy the next chunk of the armed response out under the lock into a small
    // staging buffer (one datagram = our 16-byte DONE), then to user memory.
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
    {
        let _smap = UserAccess::open();
        // SAFETY: `[buf, buf+n)` ⊆ `[buf, buf+len)` validated above to lie inside a
        // mapped, writable ring-3 page of the current (live-CR3) process, writable
        // inside the STAC/CLAC window.
        let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, n) };
        dst.copy_from_slice(&staging[..n]);
    }
    handle.advance_read_off(n as u64);
    // Fill the source address (kernel = pid 0 in nl_pid? no: we report OUR port)
    // if the caller provided one. musl's recv passes NULL/NULL, so this is
    // usually skipped; honest when a caller does pass a sockaddr.
    if addr != 0 && addrlen != 0 {
        // SAFETY: validates `addr`/`addrlen` and SMAP-brackets the writes itself.
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
/// Validates `addr`/`addrlen` before writing inside the SMAP bracket; never
/// raw-derefs an unvalidated user pointer.
unsafe fn write_sockaddr_nl(addr: u64, addrlen: u64, port: u32) {
    if !user_range_ok(addrlen, 4) {
        return;
    }
    // The caller's addrlen (in/out) must claim room for a full sockaddr_nl.
    let cap = {
        let _smap = UserAccess::open();
        // SAFETY: validated 4-byte user int (the in/out addrlen).
        unsafe { (addrlen as *const u32).read() }
    };
    if (cap as usize) < nl::SOCKADDR_NL_LEN || !user_range_ok(addr, nl::SOCKADDR_NL_LEN as u64) {
        return;
    }
    let mut sa = [0u8; nl::SOCKADDR_NL_LEN];
    sa[0..2].copy_from_slice(&(nl::AF_NETLINK).to_le_bytes()); // nl_family
    // nl_pad (2..4) stays 0.
    sa[4..8].copy_from_slice(&port.to_le_bytes()); // nl_pid
                                                   // nl_groups (8..12) stays 0.
    let _smap = UserAccess::open();
    // SAFETY: `[addr, addr+12)` validated writable; `[addrlen,addrlen+4)` validated.
    unsafe {
        let dst = core::slice::from_raw_parts_mut(addr as *mut u8, nl::SOCKADDR_NL_LEN);
        dst.copy_from_slice(&sa);
        (addrlen as *mut u32).write(nl::SOCKADDR_NL_LEN as u32);
    }
}

/// `dup2(oldfd, newfd)` / `dup3(oldfd, newfd, flags)` — duplicate `oldfd`'s
/// description into `newfd` (closing `newfd` first if open). Absent `oldfd` →
/// `-EBADF`, else `newfd`.
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
        let cur = s.current();
        let mut va = floor & !PAGE_MASK;
        while va < new {
            // SAFETY: in-window; ensures a zeroed RW frame in this space.
            unsafe { cur.space.map_page(va, Perm::ReadWrite) };
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
            unsafe { cur.space.map_page(va, Perm::ReadWrite) };
            va += PAGE_SIZE;
        }
        start as u64
    })
}

// ---- musl C-runtime startup helpers (Slice A) -----------------------------
//
// Behaviour mirrors the arch-neutral aarch64 `sys_*` helpers 1:1; the only
// x86_64-specifics are (a) the `arch_prctl` FS-base MSR (no aarch64 analogue),
// and (b) every user-memory access goes through the `UserAccess` STAC/CLAC SMAP
// bracket (aarch64 uses raw access; the math is identical).

/// `arch_prctl(code, addr)`. The one genuinely x86_64-specific startup syscall:
/// musl's `__init_tp` issues `arch_prctl(ARCH_SET_FS, tp)` to point the `%fs`
/// segment base at its thread-control block so `%fs:0`-relative TLS resolves.
/// We program the `IA32_FS_BASE` MSR directly (the FS base is unaffected by the
/// `swapgs` on the syscall/IRQ paths, so it persists across kernel entries).
/// `ARCH_GET_FS` reads it back into the validated user `*addr`. The base is
/// also saved per-process in `Process::fs_base` and restored by `switch_to`
/// via `wrmsr` on every context switch, so concurrent musl processes each keep
/// their own `%fs` TLS base (aarch64 does the equivalent via `TPIDR_EL0`).
///
/// # Safety
/// Writes/reads the `IA32_FS_BASE` MSR; on `ARCH_GET_FS` writes 8 bytes through
/// the validated, SMAP-bracketed user pointer of the current process.
unsafe fn sys_arch_prctl(code: u64, addr: u64) -> u64 {
    match code {
        ARCH_SET_FS => {
            // SAFETY: writing IA32_FS_BASE sets the user `%fs` base. The value is
            // an arbitrary user TLS pointer; it only affects ring-3 `%fs:`
            // accesses, never kernel state. Mirrors what the CPU's WRFSBASE does.
            unsafe { Msr::new(IA32_FS_BASE).write(addr) };
            // PERSIST it per-process: store the base into the CURRENT process so
            // the context switch (`process::switch_to`) can RESTORE it when this
            // process is scheduled back in. Without this the MSR is a single
            // global that a second concurrent musl process would clobber, leaving
            // us reading the wrong TLS after a preemption. `ARCH_GET_FS` reads it
            // back from here, not the MSR (so it is correct even mid-interleave).
            process::with_sched(|s| s.current().fs_base = addr);
            0
        }
        ARCH_GET_FS => {
            if !user_range_ok(addr, 8) {
                return EFAULT;
            }
            // Read the per-process saved base (the source of truth across
            // context switches), not the live MSR — they agree for the current
            // process, but the saved copy is what survives interleaving.
            let base = process::with_sched(|s| s.current().fs_base);
            let _smap = UserAccess::open();
            // SAFETY: validated 8-byte RW user buffer, written in the SMAP bracket.
            unsafe { (addr as *mut u64).write(base) };
            0
        }
        _ => EINVAL,
    }
}

/// `sysinfo(info)` — zero the `struct sysinfo` (112 bytes) and report 1 process,
/// mirroring aarch64. Just enough for musl/std probes that read it.
fn sys_sysinfo(info: u64) -> u64 {
    const SYSINFO_SIZE: u64 = 112;
    if !user_range_ok(info, SYSINFO_SIZE) {
        return EFAULT;
    }
    let _smap = UserAccess::open();
    // SAFETY: validated RW user buffer of SYSINFO_SIZE bytes, in the SMAP bracket.
    let out = unsafe { core::slice::from_raw_parts_mut(info as *mut u8, SYSINFO_SIZE as usize) };
    for b in out.iter_mut() {
        *b = 0;
    }
    // `procs` field (u16 at offset 0x60) = 1.
    out[0x60] = 1;
    0
}

// ---- WAVE 1: libc-init / process-info handlers (mirror aarch64) ------------
// Pure byte-layout math lives in `user_layout::procinfo`; these handlers do only
// the bounds-checked user copy inside the SMAP (STAC/CLAC) bracket.

/// The per-process file-creation mask. Single-process model (one umask for the
/// init chain), initialised to Linux's default `022`. A static is sufficient
/// here (the task permits it over a Process field) and keeps the change minimal.
static UMASK: core::sync::atomic::AtomicU32 =
    core::sync::atomic::AtomicU32::new(user_layout::procinfo::DEFAULT_UMASK);

/// `uname(buf)` — write the byte-exact `struct utsname` (six NUL-padded 65-byte
/// fields; `machine = "x86_64"`) into the validated user buffer. `-EFAULT` on a
/// bad pointer, else `0`.
fn sys_uname(buf: u64) -> u64 {
    let uts = user_layout::procinfo::build_utsname("x86_64");
    if !user_range_ok(buf, uts.len() as u64) {
        return EFAULT;
    }
    let _smap = UserAccess::open();
    // SAFETY: validated RW user buffer of exactly `uts.len()` bytes of the
    // current process, written inside the STAC/CLAC window.
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
    let _smap = UserAccess::open();
    // SAFETY: validated RW user buffer of RUSAGE_SIZE bytes, in the SMAP bracket.
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
        let _smap = UserAccess::open();
        // SAFETY: validated RW user buffer of TMS_SIZE bytes, in the SMAP bracket.
        let out = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, TMS_SIZE as usize) };
        for b in out.iter_mut() {
            *b = 0;
        }
    }
    (crate::timekeeping::mono_ns() / 1_000_000) as u64
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
    // SAFETY: validated 16-byte RW user buffer; written inside the SMAP bracket.
    unsafe { write_two_i64(res, secs, nsec) };
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
            let _smap = UserAccess::open();
            // SAFETY: validated RW user buffer of TASK_COMM_LEN bytes, in the
            // SMAP bracket.
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

/// `getcwd(buf, size)` — report `/` (root). Mirrors aarch64.
fn sys_getcwd(buf: u64, size: u64) -> u64 {
    const CWD: &[u8] = b"/\0";
    if size < CWD.len() as u64 {
        return (-34i64) as u64; // -ERANGE
    }
    if !user_range_ok(buf, CWD.len() as u64) {
        return EFAULT;
    }
    let _smap = UserAccess::open();
    // SAFETY: validated RW user buffer large enough for CWD, in the SMAP bracket.
    unsafe {
        core::ptr::copy_nonoverlapping(CWD.as_ptr(), buf as *mut u8, CWD.len());
    }
    CWD.len() as u64
}

/// `fstat`/`newfstatat` — zero a `struct stat` (Linux x86_64 `stat` is 144
/// bytes; we zero a generous 144 and set `st_mode` to a character device, so std
/// treats fd 0/1/2 as a tty-like console). Mirrors aarch64 (which uses 128); the
/// x86_64 `st_mode` lives at offset 24 (`st_dev`,`st_ino`,`st_nlink`,then
/// `st_mode`). Both arches only need a non-zero `S_IFCHR` mode for musl/std.
fn sys_fstat(stbuf: u64) -> u64 {
    const STAT_SIZE: u64 = 144;
    if !user_range_ok(stbuf, STAT_SIZE) {
        return EFAULT;
    }
    let _smap = UserAccess::open();
    // SAFETY: validated RW user buffer of STAT_SIZE bytes, in the SMAP bracket.
    let out = unsafe { core::slice::from_raw_parts_mut(stbuf as *mut u8, STAT_SIZE as usize) };
    for b in out.iter_mut() {
        *b = 0;
    }
    // x86_64 `struct stat`: st_mode (u32) at offset 24.
    const S_IFCHR: u32 = 0o020000;
    let mode = (S_IFCHR | 0o666).to_le_bytes();
    out[24..28].copy_from_slice(&mode);
    0
}

/// `ppoll(fds, nfds, timeout, ...)` — no real fds are pollable yet; honour the
/// timeout (a relative sleep on the real TSC) when there are no fds, else mark
/// each non-negative fd's requested `events` as ready. Mirrors aarch64.
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
            let deadline = user_layout::deadline_after(crate::timekeeping::now_counter(), cycles);
            crate::timekeeping::sleep_until(deadline);
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
    let _smap = UserAccess::open();
    for i in 0..nfds {
        let ent = fds + i * POLLFD_SIZE;
        // SAFETY: `ent` lies within the validated pollfd array, SMAP bracket open.
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

/// `getrandom(buf, len)` — fill `buf` with a cheap xorshift seeded from the TSC.
/// Mirrors aarch64 (whose seed is the generic-timer count). Not cryptographic;
/// enough to satisfy musl/std startup probes (e.g. stack-canary / HashMap seed).
fn sys_getrandom(buf: u64, len: u64) -> u64 {
    if len == 0 {
        return 0;
    }
    if !user_range_ok(buf, len) {
        return EFAULT;
    }
    let mut seed = crate::timekeeping::now_counter() ^ 0x9e37_79b9_7f4a_7c15;
    let _smap = UserAccess::open();
    // SAFETY: validated RW user buffer, written in the SMAP bracket.
    let out = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len as usize) };
    for b in out.iter_mut() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        *b = seed as u8;
    }
    len
}

/// `sched_getaffinity(_pid, cpusetsize, mask)` — report a single online CPU
/// (bit 0). Mirrors aarch64. Args here are `(cpusetsize=a1, mask=a2)` — the pid
/// (a0) is unused, matching aarch64's `_pid`.
fn sys_sched_getaffinity(cpusetsize: u64, mask: u64) -> u64 {
    if cpusetsize < 8 {
        return EINVAL;
    }
    if !user_range_ok(mask, cpusetsize) {
        return EFAULT;
    }
    let _smap = UserAccess::open();
    // SAFETY: validated RW user buffer of `cpusetsize` bytes, in the SMAP bracket.
    let out = unsafe { core::slice::from_raw_parts_mut(mask as *mut u8, cpusetsize as usize) };
    for b in out.iter_mut() {
        *b = 0;
    }
    out[0] = 0b1;
    8
}

// ---- POSIX signals --------------------------------------------------------
//
// All pure bit/layout math lives in `user_layout::signal` (`sig::*`); the code
// here is the thin unsafe Frame layer: it reads/writes the userspace
// `struct sigaction`/`sigset_t`/`sigaltstack` and the on-stack signal frame
// through the bounds-checked, STAC/CLAC-bracketed ring-3 access path, mutates
// the current process's `SignalState`, and — at the return-to-user boundary —
// rewrites the live `Context` to enter the handler.

/// A clean ring-3 RFLAGS: reserved bit 1 + IF (bit 9). Never copy
/// attacker-controlled EFLAGS from a sigreturn frame; re-impose this instead.
const USER_RFLAGS: u64 = (1 << 1) | (1 << 9);

/// userspace `struct sigaction` field offsets (≥ 32 bytes; spec §2).
const SA_HANDLER_OFF: u64 = 0;
const SA_FLAGS_OFF: u64 = 8;
const SA_RESTORER_OFF: u64 = 16;
const SA_MASK_OFF: u64 = 24;
const SA_STRUCT_SIZE: u64 = 32;

/// `rt_sigaction(signo, const act*, oldact*, sigsetsize)`.
///
/// # Safety
/// Reads/writes the user `struct sigaction` through the validated, SMAP-
/// bracketed ring-3 access path of the current process.
unsafe fn sys_rt_sigaction(signo: u64, act: u64, oldact: u64, sigsetsize: u64) -> u64 {
    if sigsetsize != sig::SIGSET_BYTES as u64 {
        return EINVAL;
    }
    if signo < 1 || signo > sig::NSIG as u64 {
        return EINVAL;
    }
    let signo32 = signo as u32;
    if act != 0 && (signo32 == sig::SIGKILL || signo32 == sig::SIGSTOP) {
        return EINVAL;
    }

    if oldact != 0 {
        if !user_range_ok(oldact, SA_STRUCT_SIZE) {
            return EFAULT;
        }
        let old = process::with_sched(|s| s.current().signals.action(signo32));
        let _smap = UserAccess::open();
        // SAFETY: validated 32-byte RW ring-3 buffer in the current process.
        unsafe {
            ((oldact + SA_HANDLER_OFF) as *mut u64).write(old.handler);
            ((oldact + SA_FLAGS_OFF) as *mut u64).write(old.flags);
            ((oldact + SA_RESTORER_OFF) as *mut u64).write(old.restorer);
            ((oldact + SA_MASK_OFF) as *mut u64).write(old.mask);
        }
    }

    if act != 0 {
        if !user_range_ok(act, SA_STRUCT_SIZE) {
            return EFAULT;
        }
        let new = {
            let _smap = UserAccess::open();
            // SAFETY: validated 32-byte readable ring-3 buffer.
            unsafe {
                sig::SigAction {
                    handler: ((act + SA_HANDLER_OFF) as *const u64).read(),
                    flags: ((act + SA_FLAGS_OFF) as *const u64).read(),
                    restorer: ((act + SA_RESTORER_OFF) as *const u64).read(),
                    mask: ((act + SA_MASK_OFF) as *const u64).read(),
                }
            }
        };
        process::with_sched(|s| s.current().signals.actions[signo as usize] = new);
    }
    0
}

/// `rt_sigprocmask(how, const set*, oldset*, sigsetsize)`.
///
/// # Safety
/// Reads/writes the user `sigset_t` through the validated SMAP-bracketed path.
unsafe fn sys_rt_sigprocmask(how: u64, set: u64, oldset: u64, sigsetsize: u64) -> u64 {
    if sigsetsize != sig::SIGSET_BYTES as u64 {
        return EINVAL;
    }
    if oldset != 0 {
        if !user_range_ok(oldset, sig::SIGSET_BYTES as u64) {
            return EFAULT;
        }
        let cur = process::with_sched(|s| s.current().signals.blocked);
        let _smap = UserAccess::open();
        // SAFETY: validated 8-byte RW ring-3 buffer.
        unsafe { (oldset as *mut u64).write(cur) };
    }
    if set != 0 {
        if !user_range_ok(set, sig::SIGSET_BYTES as u64) {
            return EFAULT;
        }
        let arg = {
            let _smap = UserAccess::open();
            // SAFETY: validated 8-byte readable ring-3 buffer.
            unsafe { (set as *const u64).read() }
        };
        process::with_sched(|s| {
            let cur = s.current().signals.blocked;
            let next = match how {
                x if x == sig::SIG_BLOCK => cur | arg,
                x if x == sig::SIG_UNBLOCK => cur & !arg,
                x if x == sig::SIG_SETMASK => arg,
                _ => cur,
            };
            s.current().signals.blocked =
                sig::Sigset(next).block_unblockable_cleared().0;
        });
    }
    0
}

/// `rt_sigpending(set*, sigsetsize)`.
///
/// # Safety
/// Writes the user `sigset_t` through the validated SMAP-bracketed path.
unsafe fn sys_rt_sigpending(set: u64, sigsetsize: u64) -> u64 {
    if sigsetsize != sig::SIGSET_BYTES as u64 {
        return EINVAL;
    }
    if set == 0 || !user_range_ok(set, sig::SIGSET_BYTES as u64) {
        return EFAULT;
    }
    let pending = process::with_sched(|s| s.current().signals.pending);
    let _smap = UserAccess::open();
    // SAFETY: validated 8-byte RW ring-3 buffer.
    unsafe { (set as *mut u64).write(pending) };
    0
}

/// `sigaltstack(const ss*, old_ss*)`. struct sigaltstack { ss_sp@0, ss_flags@8
/// (int, padded), ss_size@16 } = 24 bytes.
///
/// # Safety
/// Reads/writes the user `struct sigaltstack` through the SMAP-bracketed path.
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
        let _smap = UserAccess::open();
        // SAFETY: validated 24-byte RW ring-3 buffer.
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
        let (sp, flags, size) = {
            let _smap = UserAccess::open();
            // SAFETY: validated 24-byte readable ring-3 buffer.
            unsafe {
                (
                    (ss as *const u64).read(),
                    ((ss + 8) as *const u32).read(),
                    ((ss + 16) as *const u64).read(),
                )
            }
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
    if target <= 0 {
        return EINVAL; // groups/broadcast out of v1 scope
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
        return 0;
    }
    if signo < 1 || signo > sig::NSIG as u64 {
        return EINVAL;
    }
    process::with_sched(|s| s.post_signal(target, signo as u32));
    0
}

// ---- Signal delivery on return-to-user ------------------------------------

/// Build a signal frame on the user stack and rewrite the live `Context` so the
/// trampoline's `sysretq`/`iretq` lands in the handler — for the lowest-numbered
/// pending, unblocked signal of the **now-current** process. Called as the last
/// step before returning to ring 3. At most one signal per return.
///
/// SIG_DFL/SIG_IGN dispositions take the §5 default action (Ignore: clear the
/// bit; Terminate: kill the process) instead of building a frame.
///
/// # Safety
/// `frame` is the live Context; the current process's space is the live CR3.
/// All user writes are bounds-checked then performed inside a STAC/CLAC window;
/// an overflowing frame terminates the process (never a raw OOB write).
unsafe fn deliver_pending_signals(frame: &mut Context) {
    let signo = match process::with_sched(|s| s.current().signals.next_deliverable()) {
        Some(s) => s,
        None => return,
    };
    let action = process::with_sched(|s| s.current().signals.action(signo));

    if !action.has_handler() {
        if action.is_ignore() {
            process::with_sched(|s| s.current().signals.clear_pending(signo));
            return;
        }
        match sig::default_action(signo) {
            sig::DefaultAction::Ignore
            | sig::DefaultAction::Stop
            | sig::DefaultAction::Continue => {
                process::with_sched(|s| s.current().signals.clear_pending(signo));
            }
            sig::DefaultAction::Terminate => {
                // SAFETY: `frame` is the live frame; terminate + switch.
                unsafe { terminate_current_by_signal(frame, signo) };
            }
        }
        return;
    }

    // Require an explicit SA_RESTORER (musl/glibc always supply one).
    if action.flags & sig::SA_RESTORER == 0 || action.restorer == 0 {
        // SAFETY: `frame` is the live frame.
        unsafe { terminate_current_by_signal(frame, signo) };
        return;
    }

    // Choose the stack: the alternate stack iff requested + configured + not
    // already on it, else the interrupted rsp.
    let (alt_sp, alt_size, on_alt) = process::with_sched(|s| {
        let p = s.current();
        (p.signals.altstack_sp, p.signals.altstack_size, p.signals.on_altstack)
    });
    let use_alt = action.flags & sig::SA_ONSTACK != 0 && alt_sp != 0 && !on_alt;
    let stack_top = if use_alt { alt_sp + alt_size } else { frame.rsp };

    let frame_base = sig::x64_frame_base(stack_top);
    // Bounds-check the WHOLE frame before any write; SIGSEGV-terminate on
    // overflow rather than a raw out-of-window write.
    if !user_range_ok(frame_base, sig::X64_FRAME_SIZE) {
        // SAFETY: `frame` is the live frame.
        unsafe { terminate_current_by_signal(frame, sig::SIGSEGV) };
        return;
    }

    let old_blocked = process::with_sched(|s| s.current().signals.blocked);
    let uc_base = frame_base + sig::X64_UC_OFF;
    let mc_base = uc_base + sig::X64_UC_MCONTEXT_OFF;

    {
        let _smap = UserAccess::open();
        // SAFETY: every target address lies inside the validated `[frame_base,
        // frame_base+X64_FRAME_SIZE)` window of the current process's live CR3,
        // writable from ring 0 inside the STAC/CLAC window.
        unsafe {
            // pretcode = restorer (the handler `ret`s through it -> rt_sigreturn).
            ((frame_base + sig::X64_PRETCODE_OFF) as *mut u64).write(action.restorer);

            // ucontext: zero flags/link/stack region, set uc_stack + uc_sigmask.
            core::ptr::write_bytes(uc_base as *mut u8, 0, sig::X64_UC_MCONTEXT_OFF as usize);
            ((uc_base + sig::X64_UC_STACK_OFF) as *mut u64).write(alt_sp);
            ((uc_base + sig::X64_UC_STACK_OFF + 16) as *mut u64).write(alt_size);
            ((uc_base + sig::X64_UC_SIGMASK_OFF) as *mut u64).write(old_blocked);

            // sigcontext: save the interrupted GPRs + rip/rsp/eflags (Linux order).
            core::ptr::write_bytes(mc_base as *mut u8, 0, sig::X64_SC_END_OFF as usize);
            ((mc_base + sig::X64_SC_R8_OFF) as *mut u64).write(frame.r8);
            ((mc_base + sig::X64_SC_R9_OFF) as *mut u64).write(frame.r9);
            ((mc_base + sig::X64_SC_R10_OFF) as *mut u64).write(frame.r10);
            ((mc_base + sig::X64_SC_R11_OFF) as *mut u64).write(frame.r11);
            ((mc_base + sig::X64_SC_R12_OFF) as *mut u64).write(frame.r12);
            ((mc_base + sig::X64_SC_R13_OFF) as *mut u64).write(frame.r13);
            ((mc_base + sig::X64_SC_R14_OFF) as *mut u64).write(frame.r14);
            ((mc_base + sig::X64_SC_R15_OFF) as *mut u64).write(frame.r15);
            ((mc_base + sig::X64_SC_RDI_OFF) as *mut u64).write(frame.rdi);
            ((mc_base + sig::X64_SC_RSI_OFF) as *mut u64).write(frame.rsi);
            ((mc_base + sig::X64_SC_RBP_OFF) as *mut u64).write(frame.rbp);
            ((mc_base + sig::X64_SC_RBX_OFF) as *mut u64).write(frame.rbx);
            ((mc_base + sig::X64_SC_RDX_OFF) as *mut u64).write(frame.rdx);
            ((mc_base + sig::X64_SC_RAX_OFF) as *mut u64).write(frame.rax);
            ((mc_base + sig::X64_SC_RCX_OFF) as *mut u64).write(frame.rcx);
            ((mc_base + sig::X64_SC_RSP_OFF) as *mut u64).write(frame.rsp);
            ((mc_base + sig::X64_SC_RIP_OFF) as *mut u64).write(frame.rip);
            ((mc_base + sig::X64_SC_EFLAGS_OFF) as *mut u64).write(frame.rflags);

            // siginfo: si_signo + si_code (minimal).
            let info = frame_base + sig::X64_SIGINFO_OFF;
            core::ptr::write_bytes(info as *mut u8, 0, sig::X64_SIGINFO_SIZE as usize);
            (info as *mut u32).write(signo);
        }
    }

    // Block the delivered signal + its sa_mask for the handler (restored from
    // uc_sigmask by rt_sigreturn), and clear its pending bit.
    process::with_sched(|s| {
        let p = s.current();
        let blocked = old_blocked | action.mask | sig::sig_bit(signo);
        p.signals.blocked = sig::Sigset(blocked).block_unblockable_cleared().0;
        p.signals.clear_pending(signo);
        if use_alt {
            p.signals.on_altstack = true;
        }
    });

    // Redirect the live Context into the handler: rdi=signo, rsi=siginfo*,
    // rdx=ucontext*, rsp=frame_base, rip=handler, clean RFLAGS.
    frame.rdi = signo as u64;
    frame.rsi = frame_base + sig::X64_SIGINFO_OFF;
    frame.rdx = uc_base;
    frame.rsp = frame_base;
    frame.rip = action.handler;
    frame.rflags = USER_RFLAGS;
}

/// `rt_sigreturn()` — restore the pre-signal context from the frame the delivery
/// path built (or a forged one, which we validate).
///
/// # Safety
/// `frame` is the live Context; the frame base is recovered from `frame.rsp`
/// (which, after the handler's `ret` popped `pretcode`, points just past it) and
/// fully bounds-checked before any read. RFLAGS is forced to a clean user value.
unsafe fn sys_rt_sigreturn(frame: &mut Context) -> SyscallOutcome {
    // After the handler `ret`s into the restorer, `pretcode` has been popped, so
    // rsp = frame_base + 8. Recover frame_base from the known restorer delta
    // (the same offset constant used to build it).
    let frame_base = frame.rsp.wrapping_sub(sig::X64_UC_OFF);
    if !user_range_ok(frame_base, sig::X64_FRAME_SIZE) {
        // Forged / corrupt rsp: do not trust it. SIGSEGV-terminate.
        // SAFETY: `frame` is the live frame.
        unsafe { terminate_current_by_signal(frame, sig::SIGSEGV) };
        return SyscallOutcome::Resume;
    }
    let uc_base = frame_base + sig::X64_UC_OFF;
    let mc_base = uc_base + sig::X64_UC_MCONTEXT_OFF;

    let saved = {
        let _smap = UserAccess::open();
        // SAFETY: `[frame_base, frame_base+X64_FRAME_SIZE)` validated in the live
        // CR3; read back the saved register state + uc_sigmask.
        unsafe {
            SavedSigContext {
                r8: ((mc_base + sig::X64_SC_R8_OFF) as *const u64).read(),
                r9: ((mc_base + sig::X64_SC_R9_OFF) as *const u64).read(),
                r10: ((mc_base + sig::X64_SC_R10_OFF) as *const u64).read(),
                r11: ((mc_base + sig::X64_SC_R11_OFF) as *const u64).read(),
                r12: ((mc_base + sig::X64_SC_R12_OFF) as *const u64).read(),
                r13: ((mc_base + sig::X64_SC_R13_OFF) as *const u64).read(),
                r14: ((mc_base + sig::X64_SC_R14_OFF) as *const u64).read(),
                r15: ((mc_base + sig::X64_SC_R15_OFF) as *const u64).read(),
                rdi: ((mc_base + sig::X64_SC_RDI_OFF) as *const u64).read(),
                rsi: ((mc_base + sig::X64_SC_RSI_OFF) as *const u64).read(),
                rbp: ((mc_base + sig::X64_SC_RBP_OFF) as *const u64).read(),
                rbx: ((mc_base + sig::X64_SC_RBX_OFF) as *const u64).read(),
                rdx: ((mc_base + sig::X64_SC_RDX_OFF) as *const u64).read(),
                rax: ((mc_base + sig::X64_SC_RAX_OFF) as *const u64).read(),
                rcx: ((mc_base + sig::X64_SC_RCX_OFF) as *const u64).read(),
                rsp: ((mc_base + sig::X64_SC_RSP_OFF) as *const u64).read(),
                rip: ((mc_base + sig::X64_SC_RIP_OFF) as *const u64).read(),
                mask: ((uc_base + sig::X64_UC_SIGMASK_OFF) as *const u64).read(),
            }
        }
    };

    // Restore GPRs / rsp / rip into the live frame; RE-IMPOSE a clean user
    // RFLAGS (never copy the frame's attacker-controllable EFLAGS).
    frame.r8 = saved.r8;
    frame.r9 = saved.r9;
    frame.r10 = saved.r10;
    frame.r11 = saved.r11;
    frame.r12 = saved.r12;
    frame.r13 = saved.r13;
    frame.r14 = saved.r14;
    frame.r15 = saved.r15;
    frame.rdi = saved.rdi;
    frame.rsi = saved.rsi;
    frame.rbp = saved.rbp;
    frame.rbx = saved.rbx;
    frame.rdx = saved.rdx;
    frame.rax = saved.rax;
    frame.rcx = saved.rcx;
    frame.rsp = saved.rsp;
    frame.rip = saved.rip;
    frame.rflags = USER_RFLAGS;

    process::with_sched(|s| {
        let p = s.current();
        p.signals.blocked = sig::Sigset(saved.mask).block_unblockable_cleared().0;
        p.signals.on_altstack = false;
    });

    // Resume the restored context (no trace line, no rax overwrite — the restored
    // rax is part of the pre-signal context). The §3 hook runs again on this
    // return, so a newly-unblocked pending signal delivers next.
    SyscallOutcome::Resume
}

/// Scratch for the registers `rt_sigreturn` reads out of the frame before
/// writing them into the live `Context` (keeps the unsafe read block tidy).
struct SavedSigContext {
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rdi: u64,
    rsi: u64,
    rbp: u64,
    rbx: u64,
    rdx: u64,
    rax: u64,
    rcx: u64,
    rsp: u64,
    rip: u64,
    mask: u64,
}

/// Terminate the current process because of fatal signal `sig` (default action
/// Terminate, or a frame-overflow SIGSEGV). Mirrors `sys_exit`'s epilogue.
///
/// # Safety
/// `frame` is the live Context; switches it to the next process or finishes the
/// workload when nothing is left to run.
unsafe fn terminate_current_by_signal(frame: &mut Context, signo: u32) {
    let pid = current_pid();
    crate::kprintln!("[pid {}] terminated by signal {}", pid, signo);

    let reaped = process::with_sched(|s| {
        s.terminate_current_by_signal(signo);
        s.complete_waits()
    });

    // P4·SMP·S4d: the current process is now `Zombie` + the `PROCS` lock is
    // released, so a remote reap may free its PML4/frames at any moment; park on
    // the kernel CR3 before anything else runs on the dying CR3 (see the identical
    // guard + rationale in `sys_exit`).
    // SAFETY: ring 0, this CPU; installs the kernel PML4 (kernel half stays mapped).
    unsafe { park_on_kernel_map() };

    // Switch to the next process runnable on THIS CPU; on success drop the reaped
    // children (we are off the dying CR3). SAFETY: `frame` is the live frame.
    let switched = unsafe { process::schedule(frame) };
    if switched {
        // P4·SMP·S4c: shoot down other CPUs before freeing the reaped children's
        // page tables/frames (same rationale as `sys_exit`). No-op on 1-vCPU.
        if !reaped.is_empty() {
            crate::shootdown::request_and_wait_others();
        }
        drop(reaped);
        return;
    }
    // Nothing runnable for THIS CPU — still on the dying CR3, so forget (leak)
    // the reaped procs and diverge: BSP+empty-table powers off, otherwise this
    // CPU (re-)enters its ring-0 idle loop. SAFETY: ring-0, schedule returned
    // false; never returns.
    core::mem::forget(reaped);
    unsafe { idle_or_finish() }
}

// ---- process-model syscalls -----------------------------------------------

/// `clone(flags, newsp, ...)` — we support exactly the `fork()` ABI:
/// `flags & 0xff == SIGCHLD` and `newsp == 0`. Anything requesting a shared VM
/// (threads) is rejected with -EINVAL. The parent's live context is duplicated
/// for the child (resumes at the same post-syscall RIP) with rax forced to 0;
/// the parent's return (the child pid) is written by the caller into its rax.
fn sys_clone(frame: &Context, flags: u64, newsp: u64) -> u64 {
    if flags & CLONE_VM != 0 || newsp != 0 {
        return EINVAL;
    }
    if flags & 0xff != SIGCHLD {
        return EINVAL;
    }
    // SAFETY: boot core; deep-copies the parent address space + context.
    let child_pid = process::with_sched(|s| unsafe { s.fork_current(frame) });
    // P4·SMP·S4c: `cow_clone` write-protected the PARENT's writable leaves in
    // place; the PROCS lock is now released, so shoot down the other online CPUs
    // (IF enabled) so any CPU running the parent drops its stale writable TLB
    // entries before it writes again. No-op on 1-vCPU.
    crate::shootdown::request_and_wait_others();
    crate::kprintln!("[pid {}] clone -> child pid {}", current_pid(), child_pid);
    child_pid as u64
}

/// `execve(path, argv, envp)` — replace the current process image. No
/// filesystem, so any non-NULL `path` re-loads the embedded `user-exec` image
/// (enough to demonstrate exec semantics: tear down + fresh ELF + reset RIP/RSP,
/// keeping the pid). The pid/ppid/child list are preserved.
///
/// # Safety
/// Validates `path`; rebuilds the current process's address space in place and
/// rewrites the live context to the new entry.
unsafe fn sys_execve(frame: &mut Context, path: u64) -> SyscallOutcome {
    if path == 0 {
        return SyscallOutcome::Return(EFAULT);
    }
    let pid = current_pid();
    crate::kprintln!("[pid {}] execve -> loading embedded exec image", pid);

    // Build a brand-new address space + reset context from the EXEC image.
    // SAFETY: boot core, paging up; builds page tables + copies the image.
    let fresh = unsafe { build_process(current_ppid(), EXEC_ELF) };

    // Move the fresh image into the current slot, preserving identity. The *old*
    // AddressSpace is swapped out and returned so we can drop it only **after**
    // switching CR3 to the new one — freeing it here (still the live map) would
    // corrupt the page-table walk.
    let (old_space, cr3, new_ctx) = process::with_sched(|s| {
        let cur = s.current();
        let old = core::mem::replace(&mut cur.space, fresh.space);
        cur.brk_cur = fresh.brk_cur;
        cur.mmap_cur = fresh.mmap_cur;
        cur.ctx = fresh.ctx;
        // execve replaces the whole image: the old TLS is gone. Reset the saved
        // `%fs` base to the "never set" sentinel; the fresh musl image re-issues
        // `arch_prctl(ARCH_SET_FS)` during its C-runtime startup to set its own.
        cur.fs_base = 0;
        // execve resets signal handlers to default, preserving ONLY the blocked
        // mask (POSIX). v1 simplification: also resets SIG_IGN -> SIG_DFL, which
        // POSIX keeps; no demo relies on it (spec §1.3 / §8.9).
        let m = cur.signals.blocked;
        cur.signals = sig::SignalState::new();
        cur.signals.blocked = m;
        (old, cur.space.cr3(), cur.ctx)
    });

    // Install the new (same-pid) address space and load the reset context into
    // the live frame so the trampoline's `sysretq` lands at the new entry.
    // SAFETY: `cr3` is the freshly-built PML4 (kernel-half keeps ring 0 mapped);
    // writing CR3 does the full TLB flush.
    unsafe {
        let phys = x86_64::structures::paging::frame::PhysFrame::containing_address(
            x86_64::PhysAddr::new(cr3),
        );
        x86_64::registers::control::Cr3::write(
            phys,
            x86_64::registers::control::Cr3Flags::empty(),
        );
    }
    *frame = new_ctx;

    // P4·SMP·S4c: the old AS is about to be freed; this CPU switched CR3 off it +
    // flushed locally, but any OTHER online CPU that ran this process may still
    // cache the old AS's translations. Shoot them down (IF enabled, no lock held)
    // BEFORE freeing, so no CPU faults into / writes through a freed page table
    // or frame. No-op on 1-vCPU.
    crate::shootdown::request_and_wait_others();

    // Now that CR3 points at the new space + no CPU holds a stale entry, free the
    // old image's tables/frames.
    drop(old_space);
    SyscallOutcome::Resume
}

/// `wait4(pid, status, options, rusage)` — block the parent until a matching
/// child becomes a zombie, reap it, write the encoded status to `*status`, and
/// return the child pid. `WNOHANG` returns 0 immediately if no child has exited.
fn sys_wait4(frame: &Context, pid: u64, status: u64, options: u64) -> SyscallOutcome {
    let target = pid as i64; // -1 = any child
    if status != 0 && !user_range_ok(status, 4) {
        return SyscallOutcome::Return(EFAULT);
    }

    // Decide ECHILD / reap / WNOHANG / block ATOMICALLY under ONE scheduler lock.
    // Folding the zombie re-check and the `Waiting` transition into a SINGLE lock
    // hold closes the cross-CPU wait/wakeup TOCTOU: previously `try_reap` and the
    // `Waiting` mark were two separate `with_sched` acquisitions, so a child that
    // exited on another CPU in the GAP between them — taking the lock to mark
    // itself `Zombie` + run `complete_waits`, which did NOT yet see us `Waiting` —
    // had its wakeup LOST, and the parent blocked forever (the ~1/10 multi-CPU
    // reap-demo hang). Now the exit is serialized either BEFORE this section (we
    // reap the zombie, no block) or AFTER (we are already `Waiting`, so
    // `complete_waits` wakes us).
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
        // (which writes the reaped pid into `ctx.rax`) is never clobbered by a
        // post-block `save_current`. The epilogue uses `BlockedReschedule` to skip
        // that redundant (racy) save.
        s.block_current_for_wait(frame, target, status);
        Wait::Blocked
    });
    match outcome {
        Wait::NoChild => SyscallOutcome::Return(ECHILD),
        Wait::Reaped(cpid, encoded, child) => {
            if status != 0 {
                // Write through the current process's address space identity alias.
                process::with_sched(|s| {
                    // SAFETY: validated 4-byte RW user buffer in the current space.
                    unsafe { s.current().space.write_u32(status as usize, encoded as u32) };
                });
            }
            crate::kprintln!(
                "[pid {}] wait4 reaped child {} (status {:#x})",
                current_pid(),
                cpid,
                encoded
            );
            // P4·SMP·S4d: `child` is the reaped Process, moved out of the table by
            // `try_reap` but NOT yet freed. The `PROCS` lock is released and we run
            // on the PARENT's own surviving CR3, so freeing the child's page tables
            // + COW frames now cannot pull the live map out from under us. Shoot
            // down the other online CPUs FIRST so none retains a stale TLB entry
            // into a frame we are about to return to the allocator (x86 has no TLB
            // broadcast; this IPI is the only invalidation). No-op on 1-vCPU, where
            // the child is dropped here exactly when the old inline `table[cpid] =
            // None` dropped it (golden byte-identical).
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
/// and never return to the dead process. If it was the last one, the workload is
/// finished.
///
/// The careful ordering avoids freeing the *current* address space while `CR3`
/// still points at it: (1) mark zombie + run wait4 completion (only *moves*
/// reaped procs out, no frees), (2) switch `CR3`/the live frame to the next
/// runnable process, then (3) drop the reaped processes.
///
/// # Safety
/// `frame` is the live frame; this rewrites it to the next process's context.
unsafe fn sys_exit(frame: &mut Context, status: i32) -> SyscallOutcome {
    let pid = current_pid();
    crate::kprintln!("[pid {}] exit({})", pid, status);

    // Notify the parent: post SIGCHLD to ppid. BIT-ONLY (no print, no state
    // change) so it must not flip a Waiting parent to Runnable and emits no
    // trace line — keeps the golden byte-identical. A SIG_DFL SIGCHLD is the
    // Ignore default, cleared (delivering nothing) at the parent's next return.
    let ppid = current_ppid();
    process::with_sched(|s| s.post_signal(ppid, sig::SIGCHLD));

    let reaped = process::with_sched(|s| {
        s.exit_current(status);
        s.complete_waits()
    });

    // P4·SMP·S4d: the current process is now `Zombie` and the `PROCS` lock is
    // released, so its parent (on another CPU) may reap it — freeing its PML4 +
    // frames — at any instant. We are STILL on its (dying) CR3, and on x86 the MMU
    // re-walks the PML4 CR3 names on every access, so a freed-and-reused PML4 makes
    // ring-0 accesses fault. Park on the kernel CR3 NOW, before anything else runs.
    // `schedule` below re-installs a survivor's CR3 if one is picked; if not, we
    // idle already parked on the safe kernel map. The reap-vs-this-section race is
    // otherwise closed by the lock (a reaper also needs `PROCS`), so the only
    // unprotected window is exactly here — which this park eliminates.
    // SAFETY: ring 0, this CPU; installs the kernel PML4 (kernel half keeps ring 0
    // mapped).
    unsafe { park_on_kernel_map() };

    // Try to switch to the next process runnable on THIS CPU (installs a
    // surviving CR3 + rewrites the live frame). On success it is then safe to
    // drop the reaped children's memory (we are off the dying CR3).
    // SAFETY: `frame` is the live frame; `schedule` rewrites it to the next.
    let switched = unsafe { process::schedule(frame) };
    if switched {
        // P4·SMP·S4c: `reaped` holds exited children about to be freed; shoot down
        // the other online CPUs (PROCS lock released) so none retains a
        // translation into a frame about to return to the allocator. No-op on
        // 1-vCPU. (x86 has no TLB-broadcast instruction, so this IPI is the ONLY
        // way to invalidate a sibling CPU's stale entry.)
        if !reaped.is_empty() {
            crate::shootdown::request_and_wait_others();
        }
        drop(reaped);
        return SyscallOutcome::Resume;
    }

    // Nothing runnable for THIS CPU. We are STILL on the dying process's CR3, so
    // we must NOT free the reaped processes here (no surviving CR3 installed) —
    // leak them (forget): at machine end this is the whole workload powering off;
    // under -smp the idle loop we jump into installs a surviving CR3 before any
    // further table work, and the small one-shot leak is acceptable for S4a.
    core::mem::forget(reaped);
    // BSP + empty table → power off; AP / work-alive-elsewhere → ring-0 idle loop.
    // SAFETY: ring-0 syscall context whose `schedule` returned false; diverges.
    unsafe { idle_or_finish() }
}

// ---- timer-preemption hook ------------------------------------------------

/// Called from the timer IRQ trampoline (`__kuberos_timer_entry`) when the
/// periodic PIT fires *while ring 3 was running*. Preempt the current process:
/// save its context and switch to the next runnable one. If only one process is
/// runnable this keeps it running (a no-op switch).
///
/// Runs in ring 0 with `&mut Context` in `rdi`; the trampoline does EOI + iretq.
extern "C" fn preempt(frame: &mut Context) {
    process::save_current(frame);
    // SAFETY: `frame` is the live frame the IRQ stub will iretq from; `schedule`
    // may rewrite it to the next runnable process (and switch CR3).
    unsafe {
        if !process::schedule(frame) {
            // Nothing runnable for THIS CPU on this tick: power off (BSP, empty
            // table) or (re-)enter the ring-0 idle loop (work alive elsewhere).
            // NOTE: `idle_or_finish` issues the timer EOI path implicitly only via
            // the BSP power-off / the idle loop's own re-arm; but if we are about
            // to jump to the idle loop we must EOI THIS tick first so the periodic
            // timer keeps firing. EOI here before diverging.
            crate::apic::ack_timer();
            idle_or_finish();
        }
        // Deliver one pending, unblocked signal to the (possibly just-switched)
        // current process before the IRQ stub iretqs back to ring 3.
        deliver_pending_signals(frame);

        // Caps-gated end-of-interrupt + (Tier-1) re-arm, replacing the inline
        // `out 0x20, al` the trampoline used to emit. Picks the active tier once
        // at boot: x2APIC EOI (+ TSC-deadline re-arm) / x2APIC EOI / 8259 EOI.
        // SAFETY: we are in the timer IRQ context (ring 0); issuing the EOI for
        // the in-service timer vector is the required protocol for every tier.
        crate::apic::ack_timer();
    }
}

// ---- workload teardown ----------------------------------------------------

/// The kernel (boot) CR3, captured once by [`run_user`] before the first process
/// runs. An idle/exiting CPU PARKS on it (see [`park_on_kernel_map`]) so it never
/// sleeps on a reaped process's freed PML4. `0` until `run_user` populates it
/// (only the BSP writes it, before any AP idles).
static mut SAVED_CR3: u64 = 0;

/// Install the saved kernel (boot) CR3 on THIS CPU. Called when a CPU is about to
/// go idle (`hlt`), or right after it marks its current process `Zombie`, so it is
/// NOT running on a user process's PML4 that a sibling CPU may reap + free (and
/// reuse) out from under it — the x86 analogue of the aarch64 `park_on_kernel_map`
/// (P4·SMP·S4d). On x86 a stale CR3 is even worse than a stale TLB entry: the MMU
/// re-walks the freed PML4 on the next access. `switch_to` reinstalls the picked
/// process's CR3 the moment this CPU schedules work again, so this is purely an
/// idle-time safety net. NEVER reached on 1-vCPU (the BSP powers off directly
/// without entering the idle loop), so the golden trace + fast path are untouched.
///
/// # Safety
/// Ring 0, this CPU; `SAVED_CR3` is the kernel PML4 captured before any process
/// ran, whose kernel half keeps ring-0 code/data addressable.
unsafe fn park_on_kernel_map() {
    // SAFETY: read the kernel CR3 saved once by the BSP before the first process;
    // writing CR3 to it (full TLB flush) cannot lose ring-0 addressability.
    unsafe {
        let saved = core::ptr::read_volatile(core::ptr::addr_of!(SAVED_CR3));
        if saved == 0 {
            return; // not yet captured (cannot happen post-run_user); be safe.
        }
        let phys = x86_64::structures::paging::frame::PhysFrame::containing_address(
            x86_64::PhysAddr::new(saved),
        );
        x86_64::registers::control::Cr3::write(
            phys,
            x86_64::registers::control::Cr3Flags::empty(),
        );
    }
}

/// Reached only when the **last** process has exited: print the marker and power
/// the machine off. There is no kernel continuation to return to (the first
/// `run_user` dropped to ring 3 and never came back), so we finish here.
fn finish_workload() -> ! {
    crate::kprintln!("all processes exited");
    crate::kprintln!("kernel: OK");
    crate::exit::exit_qemu_success()
}

/// `reboot(magic1, magic2, cmd, arg)` — the only command we honour is
/// `RB_POWER_OFF` (what talos's `power_off()` issues): we validate the two Linux
/// reboot magics and, on a power-off request, terminate the VM cleanly via the
/// same `isa-debug-exit` path `finish_workload` uses (never returns). A reboot
/// with the wrong magics is `-EINVAL` (as Linux does); a recognised-magic reboot
/// with any *other* command returns 0 (we have nothing to restart).
fn sys_reboot(magic1: u64, magic2: u64, cmd: u64) -> u64 {
    if magic1 != LINUX_REBOOT_MAGIC1 || magic2 != LINUX_REBOOT_MAGIC2 {
        return EINVAL;
    }
    if cmd == RB_POWER_OFF {
        crate::kprintln!("reboot: RB_POWER_OFF -> powering off");
        crate::exit::exit_qemu_success();
    }
    0
}

// ---------------------------------------------------------------------------
// SYSCALL MSR setup (mirror of the K4 path)
// ---------------------------------------------------------------------------

/// Program the SYSCALL/SYSRET MSRs for the Linux fast path.
///
/// # Safety
/// Call once during bring-up on the boot core, after the GDT is installed and
/// [`PERCPU`] has been initialised.
unsafe fn init_syscall_msrs() {
    // EFER.SCE — enable syscall/sysret. (NXE is enabled by refine in run_user.)
    // SAFETY: read-modify-write that only adds SCE.
    unsafe {
        Efer::update(|f| *f |= EferFlags::SYSTEM_CALL_EXTENSIONS);
    }

    let (syscall_base, sysret_base) = gdt::syscall_star_selectors();
    let star = ((sysret_base.0 as u64) << 48) | ((syscall_base.0 as u64) << 32);
    // SAFETY: writing IA32_STAR with GDT-derived selector bases.
    unsafe {
        Msr::new(0xC000_0081).write(star);
    }

    LStar::write(x86_64::VirtAddr::new(
        __kuberos_syscall_entry as *const () as usize as u64,
    ));

    // FMASK: clear IF/DF/TF/AC on syscall entry.
    let fmask: u64 = (1 << 9) | (1 << 10) | (1 << 8) | (1 << 18);
    SFMask::write(x86_64::registers::rflags::RFlags::from_bits_truncate(fmask));

    // `&PERCPU` (the array base) == `&PERCPU[0]` (the BSP's block); the SYSCALL
    // trampoline's `gs:0`/`gs:8` therefore land in slot 0. (Already installed by
    // `install_bsp_percpu` before the first `with_sched`; re-affirmed here so
    // this remains the single source of the boot CPU's GS base.)
    let percpu = core::ptr::addr_of!(PERCPU) as u64;
    KernelGsBase::write(x86_64::VirtAddr::new(percpu));
}

// ---------------------------------------------------------------------------
// Page-table prep: open the boot upper levels + enable NXE (mirror of K4)
// ---------------------------------------------------------------------------

extern "C" {
    static mut boot_pml4: [u64; 512];
}

/// Add the US bit to boot PML4[0] (so the very first process — which we admit on
/// a fresh per-process PML4 anyway — and any boot-time walk can reach US leaves)
/// and enable EFER.NXE so the NX bit (bit 63) on data/stack leaves is honoured
/// rather than faulting as a reserved bit.
///
/// # Safety
/// Boot core, boot paging live; read-modify-write of EFER + boot PML4[0].
unsafe fn prepare_paging() {
    // SAFETY: adds only NXE; preserves LME/LMA/SCE.
    unsafe {
        Efer::update(|f| *f |= EferFlags::NO_EXECUTE_ENABLE);
    }
    // SAFETY: boot core; set US on the low PML4 entry (harmless: kernel huge
    // leaves stay US-clear, so no kernel memory becomes user-accessible).
    unsafe {
        (*core::ptr::addr_of_mut!(boot_pml4))[0] |= Ptf::USER_ACCESSIBLE.bits();
    }
}

// ---------------------------------------------------------------------------
// Entry: build the first process and run the workload
// ---------------------------------------------------------------------------

/// Safe entry the kernel calls to load + run the embedded `user-spawn` program
/// as the first process, then drive the scheduler until every process exits.
///
/// Never returns: the first process drops to ring 3, and the workload finishes
/// by powering off from [`finish_workload`].
/// Enable SSE/SSE2 so ring-3 (and ring-0) code may execute the x86_64 baseline
/// SIMD instructions the Rust/musl codegen freely emits (e.g. `movaps`/`movups`
/// for 16-byte stack moves). This is the x86_64 analogue of aarch64 enabling
/// Advanced SIMD/FP via `CPACR_EL1::FPEN::TrapNothing` before dropping to EL0.
///
/// Concretely: clear `CR0.EM` (no FPU emulation), set `CR0.MP`, and set
/// `CR4.OSFXSR` (FXSAVE/FXRSTOR + SSE state) and `CR4.OSXMMEXCPT` (#XM for
/// unmasked SIMD FP exceptions). Without these, the CPU raises #UD on the first
/// SSE instruction — exactly the invalid-opcode fault musl's `_start` hit.
///
/// # Safety
/// First and only SSE-enable, on the boot core before the workload runs. Only
/// *adds* the SSE-control bits; preserves every other CR0/CR4 bit (PG, PAE,
/// SMEP/SMAP set earlier, etc.).
unsafe fn enable_sse() {
    use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};
    // SAFETY: read-modify-write that clears EM and sets MP; leaves PG/PE/WP etc.
    unsafe {
        Cr0::update(|f| {
            f.remove(Cr0Flags::EMULATE_COPROCESSOR);
            f.insert(Cr0Flags::MONITOR_COPROCESSOR);
        });
    }
    // SAFETY: read-modify-write that only adds OSFXSR + OSXMMEXCPT; SSE/SSE2 are
    // architecturally mandatory on x86_64, so these bits are always supported
    // (never a reserved-bit #GP). Preserves SMEP/SMAP/PAE/etc.
    unsafe {
        Cr4::update(|f| {
            f.insert(Cr4Flags::OSFXSR);
            f.insert(Cr4Flags::OSXMMEXCPT_ENABLE);
        });
    }
}

pub fn run_user() -> ! {
    // SAFETY: called once from the boot flow after GDT/IDT/heap/PIC/PIT are up.
    // All dangerous steps operate on Frame-owned state and are audited.
    let (cr3, entry, sp) = unsafe {
        // P4·SMP·S4d: capture the kernel (boot) CR3 NOW, before we switch to any
        // process's PML4, so an idle/exiting CPU can later PARK on it (see
        // `park_on_kernel_map`) instead of sleeping on a reaped process's freed
        // PML4. `Cr3::read()` here returns the boot page table the kernel booted on.
        let (boot_frame, _boot_flags) = x86_64::registers::control::Cr3::read();
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(SAVED_CR3),
            boot_frame.start_address().as_u64(),
        );

        // 1. Enable NXE + open the boot PML4[0] US bit.
        prepare_paging();

        // 1b. Enable SSE/SSE2 (musl/std emit `movaps`/`movups`). x86_64 analogue
        //     of aarch64's CPACR_EL1 FP/SIMD enable; without it the first SSE op
        //     #UD-faults in ring 3.
        enable_sse();

        // 1c. Per-CPU anchor: stamp the boot CPU's logical index (0) into its
        //     PERCPU block and install `&PERCPU[0]` into IA32_KERNEL_GS_BASE
        //     BEFORE the first `with_sched` — `with_sched` now mints a CpuToken
        //     by reading `cpu_index` through that MSR, so the anchor must exist
        //     first. On 1-vCPU this is always 0, so the per-CPU `current`
        //     indexes slot 0 exactly as the old single global field → golden
        //     byte-identical. APs (S3) stamp their own index into PERCPU[k].
        install_bsp_percpu();

        // 2. Build process 1 from the embedded image and admit it as current.
        let first = build_process(0, USER_ELF);
        let entry = first.ctx.rip;
        let sp = first.ctx.rsp;
        let cr3 = first.space.cr3();
        let _pid = process::with_sched(|s| s.admit_first(first));

        // 3. Point the per-CPU kernel-stack slot + arm the SYSCALL MSRs.
        let kernel_rsp = current_kernel_rsp();
        (*core::ptr::addr_of_mut!(PERCPU))[0].kernel_rsp = kernel_rsp;
        init_syscall_msrs();

        (cr3, entry, sp)
    };

    crate::kprintln!("process: pid 1 entering ring 3 at {:#x} (rsp={:#x})", entry, sp);

    // Proof of life for the per-process fd table: assert pid 1's fds 0/1/2 all
    // resolve to /dev/console, then announce it once. This line is deliberately
    // NOT in the `[pid N] syscall NR -> RET` shape, so it never collides with a
    // syscall-trace consumer and stays cosmetic.
    let std_ok = process::with_sched(|s| {
        let p = s.current();
        p.fd_kind(0) == Some(FileKind::Console)
            && p.fd_kind(1) == Some(FileKind::Console)
            && p.fd_kind(2) == Some(FileKind::Console)
    });
    assert!(std_ok, "pid 1 fd table not wired to /dev/console");
    crate::kprintln!("vfs: fd table live (0,1,2 -> /dev/console)");

    // Mask IRQs while we swap the timer handler + switch CR3: the raw timer stub
    // assumes it was entered from ring 3 (it `swapgs`es and uses TSS.RSP0), so it
    // must not fire while we are still in ring 0 here. The `iretq` below restores
    // a ring-3 RFLAGS with IF set, so the periodic PIT resumes preempting only
    // once we are actually running in ring 3.
    x86_64::instructions::interrupts::disable();

    // Calibrate the TSC against PIT channel 2 and publish the timekeeper base now
    // that IRQs are masked (so the busy-wait calibration is not perturbed) and
    // before channel 0 becomes the live preemption tick. Channel 2 is independent
    // of channel 0, so this neither needs nor disturbs IRQ0.
    // SAFETY: boot core, IRQs masked, single PIT programmer (we own the PIT here).
    unsafe {
        crate::timekeeping::init_timekeeper();
    }

    // P4: pick the timer tier ONCE from the immutable boot CpuCaps (reading
    // `DetectedCaps` directly in the Frame — the cr4.rs precedent), then arm the
    // tier-appropriate interrupt chip + timer. Done here (IRQs masked, after
    // `init_timekeeper` so `tsc_hz()` is valid, before the preempt handler is
    // installed). Tiers 1/2 (`x2apic`) mask the 8259 + enable x2APIC + arm the
    // LVT timer (the PIT stays idle); Tier 3 (`!x2apic`) leaves the legacy
    // PIC/PIT path untouched. All three drive the SAME `__kuberos_timer_entry`
    // -> `preempt` -> `apic::ack_timer` path; only the ack differs.
    // SAFETY: boot core, IRQs masked, after `init_timekeeper`, single APIC/PIC
    // programmer; reconfigures the interrupt chip + timer per the chosen tier.
    let timer_tier = unsafe { crate::apic::init_for_caps(crate::hal_caps::probe_cpu_caps()) };
    // Announce the active tier on a NON-`[pid N] syscall` line so no syscall-
    // trace consumer matches it (same cosmetic discipline as the lines above).
    crate::kprintln!("{}", crate::apic::tier_announce());
    let _ = timer_tier;

    // SAFETY: switch to the process model: install the timer IRQ as a preemption
    // source, mark PROCESS_MODE on (so the ISR preempts rather than heartbeats),
    // switch CR3 to process 1's PML4, then drop to ring 3 at its entry. The
    // kernel-half of the PML4 keeps ring 0 addressable for the syscall/timer
    // trampolines; the syscall trampoline switches to the per-CPU kernel stack.
    unsafe {
        // Install our register-saving timer IRQ entry (replaces the heartbeat
        // ISR) and mark the process model live. The timer IRQ runs on the TSS's
        // dedicated RSP0 stack (loaded by the CPU on the ring3->ring0 transition);
        // the syscall trampoline runs on the separate per-CPU kernel stack. They
        // never overlap because FMASK masks IRQs during syscall handling.
        crate::interrupts::set_preempt_handler(__kuberos_timer_entry as *const () as u64);
        PROCESS_MODE.store(true, Ordering::Relaxed);

        // Switch to process 1's address space.
        let phys = x86_64::structures::paging::frame::PhysFrame::containing_address(
            x86_64::PhysAddr::new(cr3),
        );
        x86_64::registers::control::Cr3::write(
            phys,
            x86_64::registers::control::Cr3Flags::empty(),
        );

        enter_ring3(entry, sp);
    }
}

/// Drop to ring 3 at `entry` with stack `rsp` via `iretq`. Unlike the K4 path
/// there is no kernel return: control only comes back through the syscall /
/// timer trampolines (which run on the per-CPU kernel stack), and the workload
/// finishes by powering off.
///
/// # Safety
/// The user pages must be mapped (US + W^X), the SYSCALL MSRs armed, CR3 = the
/// first process's PML4, and IRQs configured. Runs ring-3 code.
unsafe fn enter_ring3(entry: u64, rsp: u64) -> ! {
    let (user_cs, user_ss) = gdt::user_selectors();
    let rflags: u64 = (1 << 1) | (1 << 9); // reserved-1 + IF

    // SAFETY: build a valid long-mode iretq frame (SS, RSP, RFLAGS, CS, RIP) with
    // RPL-3 selectors and the mapped ring-3 entry/stack, then iretq into ring 3.
    unsafe {
        core::arch::asm!(
            "push {ss}",
            "push {ursp}",
            "push {flags}",
            "push {cs}",
            "push {entry}",
            "iretq",
            ss = in(reg) user_ss.0 as u64,
            ursp = in(reg) rsp,
            flags = in(reg) rflags,
            cs = in(reg) user_cs.0 as u64,
            entry = in(reg) entry,
            options(noreturn),
        );
    }
}

// ---------------------------------------------------------------------------
// P4·SMP·S4a — AP idle→schedule loop + machine-wide termination
// ---------------------------------------------------------------------------

/// The AP scheduler entry (P4·SMP·S4a). Called by `smp::ap_rust_entry` AFTER the
/// AP has installed its per-CPU anchor, GDT/TSS, IDT and local x2APIC, published
/// its online bit, enabled SSE, and armed its periodic LVT timer. This is the
/// AP's idle→schedule loop: it pops a process from THIS CPU's run queue (or the
/// global Runnable set), drops to ring 3 to run it, and — when it has no work —
/// `hlt`s until its next periodic tick (the S4a wake source; the reschedule IPI
/// that cuts wake latency is S4b). NEVER returns to `ap_rust_entry`.
///
/// # Safety
/// Called once per AP, in ring 0 on the AP's own stack, with the process model
/// live (`PROCESS_MODE`), this AP's per-CPU GS anchor installed, and its periodic
/// timer armed. The ring-3 drops use this AP's own TSS (RSP0) and per-CPU kernel
/// stack, installed in `ap_rust_entry`.
/// AP bootstrap + scheduler entry (P4·SMP·S4a). Called by `smp::ap_rust_entry`
/// after the S3 per-CPU bring-up (anchor + GDT/TSS + IDT + x2APIC + online bit).
/// It completes the per-CPU state the scheduler needs — waits for the process
/// model to go live, enables SSE on this AP, arms this AP's periodic timer, sets
/// this AP's SYSCALL MSRs + per-CPU kernel stack — then enters the idle→schedule
/// loop. NEVER returns.
///
/// # Safety
/// Called once per AP, ring 0, on the AP's own stack, with this AP's per-CPU GS
/// anchor (`&PERCPU[idx]`) already installed by `install_ap_percpu(idx)` and
/// `idx < MAX_CPUS` unique to this CPU. IRQs masked on entry.
pub(crate) unsafe fn ap_bootstrap_and_run(idx: usize) -> ! {
    // 1. Wait for the BSP to bring the process model live (it installs the
    //    preempt handler + sets PROCESS_MODE in `run_user`, and caches the
    //    periodic-timer count). Until then there is nothing to schedule and the
    //    timer count is not yet published.
    while !PROCESS_MODE.load(Ordering::Acquire) {
        core::hint::spin_loop();
    }

    // 2. Enable SSE/SSE2 on this AP (musl/std ring-3 code emits SSE; without it
    //    the first SSE op #UD-faults). Per-CPU CR0/CR4 bits.
    // SAFETY: AP, adds only the SSE-control bits, preserves PG/PAE/SMEP/etc.
    unsafe { enable_sse() };

    // 3. Per-CPU SYSCALL MSRs (STAR/LSTAR/FMASK/SCE) + this AP's kernel stack for
    //    the syscall trampoline. These MSRs are per-CPU, so each AP programs its
    //    own. CRITICALLY this must NOT touch KernelGsBase (the AP already
    //    installed `&PERCPU[idx]`); a verbatim `init_syscall_msrs` would reset it
    //    to slot 0, so we inline the AP-correct subset.
    // SAFETY: AP, GDT installed, this AP's PERCPU slot anchored.
    unsafe {
        Efer::update(|f| {
            *f |= EferFlags::SYSTEM_CALL_EXTENSIONS | EferFlags::NO_EXECUTE_ENABLE
        });
        let (syscall_base, sysret_base) = gdt::syscall_star_selectors();
        let star = ((sysret_base.0 as u64) << 48) | ((syscall_base.0 as u64) << 32);
        Msr::new(0xC000_0081).write(star);
        LStar::write(x86_64::VirtAddr::new(
            __kuberos_syscall_entry as *const () as usize as u64,
        ));
        let fmask: u64 = (1 << 9) | (1 << 10) | (1 << 8) | (1 << 18);
        SFMask::write(x86_64::registers::rflags::RFlags::from_bits_truncate(fmask));
        // This AP's kernel stack top for the SYSCALL trampoline (gs:0). A fresh
        // top below the current rsp, mirroring the BSP's `current_kernel_rsp`.
        let kernel_rsp = current_kernel_rsp();
        (*core::ptr::addr_of_mut!(PERCPU))[idx].kernel_rsp = kernel_rsp;
    }

    // 4. Arm THIS AP's periodic LVT timer (reusing the BSP-calibrated count) so it
    //    receives preemption ticks + hlt wakeups.
    // SAFETY: AP, x2APIC enabled in S3 bring-up; programs this CPU's APIC timer.
    unsafe { crate::apic::arm_periodic_ap() };

    // 5. Enter the idle→schedule loop. NEVER returns.
    // SAFETY: all per-CPU scheduler prerequisites are now up on this AP.
    unsafe { ap_run_scheduler() }
}

pub(crate) unsafe fn ap_run_scheduler() -> ! {
    // A throwaway bootstrap "from" frame: on the first `schedule`, `save_current`
    // writes into whatever `current`-slot pid this AP owns (initially none → a
    // no-op, exactly its documented "may have been reaped" guard).
    let mut frame = process::Context::zeroed();
    loop {
        // Pick under IRQs-disabled so a tick cannot interleave the decision; the
        // `with_sched` lock is `lock_irqsave` so it is masked across the table
        // ops regardless, but masking here also makes the hlt/schedule decision
        // atomic w.r.t. the periodic tick (the §3.4 "no lost wakeup" shape, even
        // though S4a's wake is the tick itself, not yet an IPI).
        x86_64::instructions::interrupts::disable();
        // SAFETY: ring 0 on this AP; `schedule` may switch CR3 + rewrite `frame`
        // to the picked process. Returns false if nothing is runnable for THIS
        // CPU right now.
        let got = unsafe { process::schedule(&mut frame) };
        if got {
            // A process was picked, CR3 switched, `frame` holds its full context.
            // Drop to ring 3 on this AP. From there the periodic timer preempts
            // it (ring-3 entry → `preempt`), and when this CPU next runs dry the
            // preempt/exit path routes back here via `idle_or_finish`.
            // SAFETY: CR3 is the picked process's PML4; `frame` is its saved
            // ring-3 context; this AP's SYSCALL MSRs + TSS are armed.
            unsafe { enter_ring3_full(&frame) };
        }
        // Nothing runnable for this CPU right now.
        if !process::any_alive() {
            // The whole workload is done. The BSP powers the machine off; APs
            // just halt (the machine is about to die). Routing this through the
            // BSP check is what lets the workload terminate even when the LAST
            // process exited on an AP and the BSP only notices here in its idle
            // loop on the next tick.
            if this_cpu_index() == 0 {
                finish_workload(); // `-> !`: power off.
            }
            // AP at end-of-workload: halt until power-off but keep IF ENABLED so
            // we still SERVICE + ack any in-flight P4·SMP·S4c shootdown IPI (a
            // sibling exiting/reaping may be waiting on our ack). `sti; hlt` is
            // atomic w.r.t. an IPI delivered just before the hlt. A halted CPU
            // runs nothing, so a serviced shootdown is a harmless local flush.
            // P4·SMP·S4d: park on the kernel CR3 first so this halted CPU is not
            // sleeping on a reaped process's freed PML4.
            // SAFETY: ring 0, this CPU; installs the kernel PML4.
            unsafe { park_on_kernel_map() };
            loop {
                // SAFETY: enable IF then hlt atomically; wake on the shootdown IPI.
                unsafe {
                    core::arch::asm!("sti; hlt", options(nomem, nostack, preserves_flags));
                }
            }
        }
        // Work may land on our queue later (a fork placed here, or — S4b — a
        // steal/IPI). P4·SMP·S4d: park on the kernel CR3 BEFORE idling so that if
        // the process this CPU last ran is reaped (its PML4 freed + reused) while
        // we `hlt`, our live CR3 is the kernel PML4, not a stale/freed user PML4 —
        // closing the idle-CPU use-after-free. Re-enable IRQs and `hlt` until the
        // next periodic tick wakes us to retry. `sti; hlt` is atomic w.r.t. a tick
        // delivered just before the hlt, so we never miss the wake.
        // SAFETY: ring 0, this CPU; installs the kernel PML4.
        unsafe { park_on_kernel_map() };
        unsafe {
            core::arch::asm!("sti; hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

/// A CPU's `schedule` found nothing runnable for it. Decide what to do — the
/// single replacement for the bare `finish_workload()` calls now that more than
/// one CPU can run dry:
///
/// * **Workload finished** (`!any_alive()` — no live process anywhere): only the
///   **BSP** (cpu 0) powers the machine off via [`finish_workload`]; an AP just
///   halts (the machine is about to die). This preserves the 1-vCPU behavior
///   exactly: on a single CPU "schedule found nothing" only ever happens when the
///   last process exited, and that CPU IS the BSP → `finish_workload` as before.
///
/// * **Work still alive elsewhere** (another CPU is running a process, or a fork
///   will place one here): the calling CPU has no IMMEDIATE work, so it abandons
///   the current trap frame and (re-)enters its ring-0 idle→schedule loop
///   [`ap_run_scheduler`], `hlt`-ing until its next periodic tick re-runs the
///   scheduler. We reset RSP to a fresh kernel-stack top and `jmp` (never iretq
///   back to a ring-3 frame we are discarding). NEVER returns.
///
/// # Safety
/// Ring 0, in a trap/IRQ context whose `schedule` just returned false. Discards
/// the live trap frame deliberately (no user context worth preserving).
unsafe fn idle_or_finish() -> ! {
    if !process::any_alive() {
        if this_cpu_index() == 0 {
            finish_workload(); // BSP: power off — `-> !`.
        }
        // AP at end-of-workload: halt until power-off, IF ENABLED so we still ack
        // any in-flight P4·SMP·S4c shootdown IPI (else a sibling sender hangs). A
        // halted CPU runs nothing, so the serviced flush is harmless.
        // P4·SMP·S4d: park on the kernel CR3 first (see `park_on_kernel_map`).
        // SAFETY: ring 0, this CPU; installs the kernel PML4.
        unsafe { park_on_kernel_map() };
        loop {
            // SAFETY: enable IF then hlt atomically; wake on the shootdown IPI.
            unsafe {
                core::arch::asm!("sti; hlt", options(nomem, nostack, preserves_flags));
            }
        }
    }
    // Work remains elsewhere; (re-)enter this CPU's ring-0 idle loop on a fresh
    // kernel stack. SAFETY: reading rsp is side-effect-free; we then abandon the
    // current frame and jump to the idle loop, which never returns.
    let rsp = unsafe { current_kernel_rsp() };
    // SAFETY: set rsp to a fresh ring-0 stack top and jump to the idle loop
    // (`extern "C" fn() -> !`). The idle loop owns the CPU from here.
    unsafe {
        core::arch::asm!(
            "mov rsp, {rsp}",
            "jmp {idle}",
            rsp = in(reg) rsp,
            idle = sym ap_run_scheduler,
            options(noreturn),
        );
    }
}

/// This CPU's logical index (0 = BSP), read from the per-CPU GS anchor. Cheap
/// `gs:16` read; valid wherever `this_cpu_token` is (kernel GS active).
fn this_cpu_index() -> usize {
    // SAFETY: ring-0 trap context, kernel GS active (the same invariant
    // `this_cpu_token` requires); a side-effect-free GS-relative read.
    let token = unsafe { this_cpu_token() };
    token.cpu_index()
}

/// Drop to ring 3 restoring the FULL saved [`Context`] (all GPRs + RIP/RSP/RFLAGS)
/// via `iretq`. Unlike [`enter_ring3`] (which only sets entry RIP/RSP for a fresh
/// process) this resumes an already-running process the scheduler picked — used
/// by an AP picking up a forked worker whose context has live register values.
///
/// # Safety
/// CR3 must be `ctx`'s process PML4, the SYSCALL MSRs armed, and `ctx` a valid
/// ring-3 context. Runs ring-3 code; never returns to the caller.
unsafe fn enter_ring3_full(ctx: &Context) -> ! {
    let (user_cs, user_ss) = gdt::user_selectors();
    let cs = user_cs.0 as u64;
    let ss = user_ss.0 as u64;
    // We address the saved GPRs from a single pointer register (`rcx`) by their
    // `#[repr(C)]` field offsets (rax@0, rbx@8, …, r15@112, rip@120, rsp@128,
    // rflags@136), avoiding the "more registers than available" allocation that a
    // 15-operand `in(reg)` load hits. `rcx` is loaded LAST (it carries the Context
    // pointer until then).
    // SAFETY: build a long-mode iretq frame (SS, RSP, RFLAGS, CS, RIP) with RPL-3
    // selectors from the saved context, restore the GPRs from memory, then iretq.
    unsafe {
        core::arch::asm!(
            // Build the iretq frame from the Context fields (ptr in rcx).
            "push {ss}",                 // SS
            "push qword ptr [rcx + 128]",// RSP  (ctx.rsp)
            "push qword ptr [rcx + 136]",// RFLAGS (ctx.rflags)
            "push {cs}",                 // CS
            "push qword ptr [rcx + 120]",// RIP  (ctx.rip)
            // Restore GPRs from the Context.
            "mov rax, [rcx + 0]",
            "mov rbx, [rcx + 8]",
            "mov rdx, [rcx + 24]",
            "mov rsi, [rcx + 32]",
            "mov rdi, [rcx + 40]",
            "mov rbp, [rcx + 48]",
            "mov r8,  [rcx + 56]",
            "mov r9,  [rcx + 64]",
            "mov r10, [rcx + 72]",
            "mov r11, [rcx + 80]",
            "mov r12, [rcx + 88]",
            "mov r13, [rcx + 96]",
            "mov r14, [rcx + 104]",
            "mov r15, [rcx + 112]",
            // rcx itself last (we no longer need the pointer).
            "mov rcx, [rcx + 16]",
            "iretq",
            in("rcx") ctx as *const Context,
            ss = in(reg) ss,
            cs = in(reg) cs,
            options(noreturn),
        );
    }
}

/// Read a kernel stack pointer the syscall/timer trampolines can switch to,
/// biased below the live `rsp` and 16-aligned so the trampolines' use of the
/// kernel stack never clobbers `run_user`'s own frame.
///
/// # Safety
/// Reads `rsp`; the returned value is only used as a fresh kernel stack top.
unsafe fn current_kernel_rsp() -> u64 {
    let rsp: u64;
    // SAFETY: reading the stack pointer has no side effects.
    unsafe {
        core::arch::asm!("mov {}, rsp", out(reg) rsp, options(nomem, nostack, preserves_flags));
    }
    (rsp - 0x2000) & !0xf
}
