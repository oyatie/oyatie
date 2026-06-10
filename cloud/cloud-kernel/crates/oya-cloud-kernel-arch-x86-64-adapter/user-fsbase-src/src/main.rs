//! user-fsbase-x86_64: a `no_std`/`no_main` ring-3 **PID1 supervisor** that
//! proves the x86_64 `arch_prctl` FS-base-per-process fix, using only raw
//! Linux/x86_64 syscalls (`syscall`).
//!
//! ## What it proves
//!
//! On x86_64, musl points `%fs` at its thread-control block via
//! `arch_prctl(ARCH_SET_FS, tp)`, which the kernel programs into the single,
//! global `IA32_FS_BASE` MSR. With TWO concurrent musl processes each setting a
//! DISTINCT base, the MSR must be SAVED per-process and RESTORED on every context
//! switch — otherwise, after the 10 ms timer preempts process A and resumes
//! process B, B runs with A's `%fs` base and reads A's TLS -> corruption.
//!
//! This PID1 is itself `no_std`: it never calls `arch_prctl`, so its own
//! `fs_base` stays 0 (the kernel leaves the MSR untouched for it). It:
//!   1. installs a SIGCHLD handler (so child exits are observable);
//!   2. `clone()`s worker child **A**, which `execve`s the musl TLS worker;
//!   3. `clone()`s worker child **B**, which `execve`s the SAME musl TLS worker;
//!      each child has a distinct pid, so each sets a DISTINCT `__thread` value
//!      (its pid) and a DISTINCT `%fs` base. The 10 ms preemption interleaves
//!      the two musl workers across their `clock_nanosleep` yields;
//!   4. polls a short `clock_nanosleep`, reaping zombies via `wait4(-1, WNOHANG)`
//!      until BOTH workers are reaped;
//!   5. asserts BOTH workers exited with status 0 (each kept reading ITS OWN
//!      TLS — a worker that read the wrong TLS exits non-zero or faults), then
//!      prints `fsbase: BOTH workers correct` and `exit(0)`.
//!
//! WITHOUT the fix (`--features fsbase-demo-nofix` on the kernel) the workers
//! clobber each other's `%fs` base: at least one detects a wrong TLS read and
//! `exit(42)` (or faults), so PID1 sees a non-zero status and `exit(1)` — the
//! whole run never prints `kernel: OK`. That visible failure is the
//! falsifiability proof that this demo exercises the fix.
//!
//! All proof lines are deliberately NOT in the `[pid N] syscall NR -> RET` shape
//! the golden extractor matches, so they never perturb the trace harness.
//!
//! `no_std` + `no_main`; it allows `unsafe` because, as a ring-3 program making
//! raw `syscall`s, it is outside the kernel TCB.

#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicU32, Ordering};

// --- Linux x86_64 syscall numbers ------------------------------------------
const SYS_WRITE: u64 = 1;
const SYS_RT_SIGACTION: u64 = 13;
const SYS_GETPID: u64 = 39;
const SYS_CLONE: u64 = 56;
const SYS_EXECVE: u64 = 59;
const SYS_EXIT: u64 = 60;
const SYS_WAIT4: u64 = 61;
const SYS_CLOCK_NANOSLEEP: u64 = 230;

const STDOUT: u64 = 1;
const CLOCK_MONOTONIC: u64 = 1;

/// `clone` flag used by `fork()`: deliver SIGCHLD (17) to the parent on exit.
const SIGCHLD: u64 = 17;

const SA_RESTORER: u64 = 0x0400_0000;
const SIGSET_SIZE: u64 = 8;

/// `wait4` option: return immediately (0) if no child has exited yet.
const WNOHANG: u64 = 1;

/// Expected worker exit status: each musl TLS worker `exit(0)`s iff it kept
/// reading ITS OWN TLS across every preemption. A worker that read the WRONG TLS
/// (the bug) `exit(42)`s instead — so a non-zero reaped status is the corruption
/// signal PID1 surfaces.
const EXPECT_STATUS: u32 = 0;

/// How many workers we clone. TWO concurrent musl processes is the minimum that
/// exercises the FS-base clobber (one process alone never triggers the bug).
const WORKERS: u32 = 2;

/// Bounded poll budget. At ~5 ms per nanosleep this is generous head-room for
/// both workers to start, loop, and exit under slow TCG, while guaranteeing PID1
/// can never spin forever.
const MAX_POLL_ITERS: u32 = 8000;
/// Short per-iteration sleep (~5 ms): keeps PID1 returning to user often so a
/// posted SIGCHLD is delivered promptly, and so the workers get interleaved.
const POLL_SLEEP_NS: i64 = 5_000_000;

/// The SIGCHLD handler bumps this `.bss` counter (proves child exits are
/// signal-observable; the reap loop is driven by wait4(WNOHANG) regardless).
static SIGCHLD_COUNT: AtomicU32 = AtomicU32::new(0);

// --- Raw syscall wrappers (x86_64: nr in rax; args rdi,rsi,rdx,r10,r8,r9) ---

#[inline(always)]
unsafe fn syscall0(nr: u64) -> i64 {
    let ret: i64;
    asm!(
        "syscall",
        inlateout("rax") nr => ret,
        lateout("rcx") _, lateout("r11") _,
        options(nostack),
    );
    ret
}

#[inline(always)]
unsafe fn syscall1(nr: u64, a0: u64) -> i64 {
    let ret: i64;
    asm!(
        "syscall",
        inlateout("rax") nr => ret,
        in("rdi") a0,
        lateout("rcx") _, lateout("r11") _,
        options(nostack),
    );
    ret
}

#[inline(always)]
unsafe fn syscall2(nr: u64, a0: u64, a1: u64) -> i64 {
    let ret: i64;
    asm!(
        "syscall",
        inlateout("rax") nr => ret,
        in("rdi") a0, in("rsi") a1,
        lateout("rcx") _, lateout("r11") _,
        options(nostack),
    );
    ret
}

#[inline(always)]
unsafe fn syscall3(nr: u64, a0: u64, a1: u64, a2: u64) -> i64 {
    let ret: i64;
    asm!(
        "syscall",
        inlateout("rax") nr => ret,
        in("rdi") a0, in("rsi") a1, in("rdx") a2,
        lateout("rcx") _, lateout("r11") _,
        options(nostack),
    );
    ret
}

#[inline(always)]
unsafe fn syscall4(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> i64 {
    let ret: i64;
    // The 4th arg goes in r10 (NOT rcx) for the Linux x86_64 syscall ABI.
    asm!(
        "syscall",
        inlateout("rax") nr => ret,
        in("rdi") a0, in("rsi") a1, in("rdx") a2, in("r10") a3,
        lateout("rcx") _, lateout("r11") _,
        options(nostack),
    );
    ret
}

// --- Thin helpers -----------------------------------------------------------

fn print(s: &[u8]) {
    // SAFETY: valid pointer/length pair handed to the kernel `write`.
    unsafe {
        syscall3(SYS_WRITE, STDOUT, s.as_ptr() as u64, s.len() as u64);
    }
}

/// Print a small non-negative integer in decimal (enough for pids/status).
fn print_u32(mut v: u32) {
    let mut buf = [0u8; 10];
    let mut i = buf.len();
    if v == 0 {
        print(b"0");
        return;
    }
    while v > 0 {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    print(&buf[i..]);
}

fn getpid() -> u32 {
    // SAFETY: getpid takes no arguments and only reads kernel state.
    (unsafe { syscall0(SYS_GETPID) }) as u32
}

fn sys_exit(code: i32) -> ! {
    // SAFETY: exit terminates the process and never returns.
    unsafe {
        syscall1(SYS_EXIT, code as u64);
    }
    loop {
        core::hint::spin_loop();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Timespec {
    tv_sec: i64,
    tv_nsec: i64,
}

/// `clock_nanosleep(clockid, flags=0, req, rem=NULL)` — relative sleep.
fn clock_nanosleep(req: &Timespec) -> i64 {
    // SAFETY: `req` is a valid 16-byte readable timespec; rem is NULL.
    unsafe {
        syscall4(
            SYS_CLOCK_NANOSLEEP,
            CLOCK_MONOTONIC,
            0, // flags: relative
            req as *const Timespec as u64,
            0, // rem = NULL
        )
    }
}

// --- Signal handler + restorer ----------------------------------------------

/// SIGCHLD handler: bump the `.bss` counter.
extern "C" fn sigchld_handler(_signo: u64) {
    SIGCHLD_COUNT.fetch_add(1, Ordering::SeqCst);
}

/// `__restore`: the SA_RESTORER trampoline (`mov rax, 15; syscall` =
/// rt_sigreturn), exactly like musl's `__restore_rt`.
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn __restore() -> ! {
    core::arch::naked_asm!("mov rax, 15", "syscall");
}

/// Install `handler` for `signo` via `rt_sigaction` + `SA_RESTORER`.
fn install_handler(signo: u64, handler: extern "C" fn(u64)) -> bool {
    #[repr(C)]
    struct KSigaction {
        handler: u64,
        flags: u64,
        restorer: u64,
        mask: u64,
    }
    let act = KSigaction {
        handler: handler as *const () as u64,
        flags: SA_RESTORER,
        restorer: __restore as *const () as u64,
        mask: 0,
    };
    // SAFETY: `&act` is a valid 32-byte readable user pointer; oldact = NULL.
    let rc = unsafe {
        syscall4(
            SYS_RT_SIGACTION,
            signo,
            &act as *const KSigaction as u64,
            0,
            SIGSET_SIZE,
        )
    };
    rc == 0
}

// --- Program logic ----------------------------------------------------------

/// The path passed to execve. The kernel has no filesystem, so it ignores the
/// string content and loads its embedded worker image (the musl TLS worker under
/// `--features fsbase-demo`) — but the pointer must be a valid, non-NULL user
/// address (execve validates it).
const WORKER_PATH: &[u8] = b"/fsbase-worker\0";

/// Clone one worker child; in the child this `execve`s the musl TLS worker and
/// never returns. Returns the child pid in the parent, or `i64` < 0 on a clone
/// failure.
fn spawn_worker() -> i64 {
    // SAFETY: bare fork ABI: rdi = SIGCHLD exit-signal, rsi = 0 (no new stack).
    let rc = unsafe { syscall2(SYS_CLONE, SIGCHLD, 0) };
    if rc == 0 {
        // ---- child ----
        // SAFETY: WORKER_PATH is a valid NUL-terminated user pointer; argv/envp
        // NULL. The kernel ignores the path and loads the embedded musl worker.
        unsafe {
            syscall3(SYS_EXECVE, WORKER_PATH.as_ptr() as u64, 0, 0);
        }
        // execve only returns on failure.
        print(b"fsbase: child execve FAILED\n");
        sys_exit(1);
    }
    rc
}

#[unsafe(no_mangle)]
extern "C" fn user_main() -> ! {
    let my_pid = getpid();
    print(b"fsbase: PID1 starting pid=");
    print_u32(my_pid);
    print(b"\n");

    // Install the SIGCHLD handler so child exits are signal-observable.
    if !install_handler(SIGCHLD, sigchld_handler) {
        print(b"fsbase: rt_sigaction(SIGCHLD) FAILED\n");
        sys_exit(1);
    }

    // Clone the TWO worker children. Each execve's the SAME musl TLS worker; each
    // has a distinct pid and so sets a distinct `__thread`/`%fs` base.
    let mut worker_pids = [0u32; WORKERS as usize];
    let mut i = 0;
    while i < WORKERS as usize {
        let rc = spawn_worker();
        if rc < 0 {
            print(b"fsbase: clone FAILED\n");
            sys_exit(1);
        }
        worker_pids[i] = rc as u32;
        print(b"fsbase: spawned worker pid=");
        print_u32(worker_pids[i]);
        print(b"\n");
        i += 1;
    }

    // Poll loop: short clock_nanosleep, reap zombies via wait4(-1, WNOHANG) until
    // BOTH workers are reaped. The two musl workers run concurrently and are
    // interleaved by the 10 ms timer preemption between PID1's nanosleeps.
    let req = Timespec { tv_sec: 0, tv_nsec: POLL_SLEEP_NS };
    let mut reaped = 0u32;
    let mut all_ok = true;
    let mut iters: u32 = 0;
    while reaped < WORKERS && iters < MAX_POLL_ITERS {
        let mut status: i32 = 0;
        // SAFETY: passes a valid i32 pointer for the kernel to write the status.
        let r = unsafe {
            syscall4(
                SYS_WAIT4,
                (-1i64) as u64, // any child
                &mut status as *mut i32 as u64,
                WNOHANG,
                0,
            )
        };
        if r > 0 {
            // Decode WEXITSTATUS: Linux encodes a normal exit as (code&0xff)<<8.
            let wexit = ((status >> 8) & 0xff) as u32;
            print(b"fsbase: reaped worker pid=");
            print_u32(r as u32);
            print(b" status=");
            print_u32(wexit);
            print(b"\n");
            if wexit != EXPECT_STATUS {
                // A worker that read the WRONG TLS exits non-zero -> corruption.
                all_ok = false;
            }
            reaped += 1;
        } else {
            // No zombie yet: yield and keep the workers interleaving.
            let _ = clock_nanosleep(&req);
            iters += 1;
        }
    }

    if reaped < WORKERS {
        print(b"fsbase: TIMEOUT (reaped ");
        print_u32(reaped);
        print(b" of ");
        print_u32(WORKERS);
        print(b")\n");
        sys_exit(1);
    }

    if !all_ok {
        // At least one worker detected a wrong-TLS read (the FS-base clobber).
        // This is the WITHOUT-fix outcome: print the unambiguous FAIL verdict the
        // gate greps for (the kernel still prints its generic "all processes
        // exited / kernel: OK" lifecycle marker afterward, so the DEMO
        // verdict — pass vs fail — must be read from THIS `fsbase:` line).
        print(b"fsbase: TLS CORRUPTION detected (a worker read the wrong TLS)\n");
        print(b"fsbase: DEMO FAIL\n");
        sys_exit(1);
    }

    // WITH-fix outcome: both workers kept reading their own distinct `%fs` base
    // across every preemption. This is the discriminating PASS verdict.
    print(b"fsbase: BOTH workers correct\n");
    let _ = SIGCHLD_COUNT.load(Ordering::SeqCst); // touch the .bss flag
    print(b"fsbase: DEMO PASS\n");
    print(b"fsbase: shutdown\n");
    sys_exit(0);
}

/// `_start`: the ELF entry point. The kernel's process-model loader enters here
/// with `rsp` already pointing at a valid SysV initial stack. We tail-call Rust.
#[unsafe(no_mangle)]
#[unsafe(naked)]
#[unsafe(link_section = ".text._start")]
unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "call {user_main}",
        "ud2",
        user_main = sym user_main,
    )
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    sys_exit(1)
}
