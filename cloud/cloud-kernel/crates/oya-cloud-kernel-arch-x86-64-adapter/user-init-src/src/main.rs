//! user-init-x86_64: a `no_std`/`no_main` ring-3 **PID1 init supervisor** that
//! proves the kuberos *init contract* on x86_64, using only raw Linux/x86_64
//! syscalls (`syscall`). The x86_64 analogue of ../../../kernel-usermode-tests/init
//! (aarch64).
//!
//! This is the Milestone-1 "parity floor" capstone: it demonstrates that the
//! kernel can host an init-style PID1 that supervises a real worker via
//! **signal-driven** child reaping — the honest, kernel-faithful pattern given
//! this kernel deliberately has NO EINTR (a blocking syscall is never woken by a
//! signal). So PID1 does NOT block in wait4; it polls.
//!
//! Flow:
//!   1. (VFS contract) re-`openat("/dev/console")` and `dup2` it onto fds 0/1/2,
//!      exercising the fd table on the inherited stdio (idempotent: the kernel
//!      already wired 0/1/2 to the console at process creation);
//!   2. install a **SIGCHLD** (17) handler via `rt_sigaction` + `SA_RESTORER`
//!      (a `mov rax,15; syscall` = `rt_sigreturn` trampoline) — the handler
//!      bumps a `.bss` counter;
//!   3. install a **SIGTERM** (15) handler the same way — it sets a `.bss`
//!      shutdown flag (proves a second disposition is installable; the demo
//!      drives the SIGCHLD path to completion);
//!   4. `clone(SIGCHLD)` (the `fork()` ABI) a worker child;
//!      - in the **child**: `execve` the worker image (the kernel loads the real
//!        `svc` musl binary under `--features init-demo`); the worker prints its
//!        heartbeats and `exit(0)`s;
//!      - print `init: spawned svc pid=N` in PID1 right after the clone;
//!   5. in PID1, loop on a SHORT `clock_nanosleep` (bounded iterations) checking
//!      the SIGCHLD `.bss` counter. The kernel posts SIGCHLD to PID1 when the
//!      worker exits; on PID1's next return-to-user (from a nanosleep or a timer
//!      preempt) the kernel delivers it -> the handler runs -> bumps the counter
//!      -> PID1's loop observes it;
//!   6. on observing SIGCHLD: print `init: SIGCHLD received`, then `wait4(-1,
//!      &status, WNOHANG, NULL)` to **reap**, verify the reaped pid is the worker
//!      and `WEXITSTATUS == 0`, print `init: reaped svc pid=N status=M`;
//!   7. print `init: shutdown` and `exit(0)`.
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
const SYS_CLOSE: u64 = 3;
const SYS_RT_SIGACTION: u64 = 13;
const SYS_DUP2: u64 = 33;
const SYS_GETPID: u64 = 39;
const SYS_CLONE: u64 = 56;
const SYS_EXECVE: u64 = 59;
const SYS_EXIT: u64 = 60;
const SYS_WAIT4: u64 = 61;
const SYS_CLOCK_NANOSLEEP: u64 = 230;
const SYS_OPENAT: u64 = 257;

const STDOUT: u64 = 1;
const AT_FDCWD: u64 = (-100i64) as u64;

const CLOCK_MONOTONIC: u64 = 1;

/// `clone` flag used by `fork()`: deliver SIGCHLD (17) to the parent on exit.
const SIGCHLD: u64 = 17;
/// SIGTERM (15): the orderly-shutdown signal init handles.
const SIGTERM: u64 = 15;

const SA_RESTORER: u64 = 0x0400_0000;
const SIGSET_SIZE: u64 = 8;

/// `wait4` option: return immediately (0) if no child has exited yet — the
/// non-blocking reap PID1 uses once its SIGCHLD handler has fired.
const WNOHANG: u64 = 1;

/// Expected worker exit status. The real `svc` heartbeat worker `exit(0)`s; a
/// trivial fallback worker also exits 0. PID1 asserts `WEXITSTATUS == 0`.
const EXPECT_STATUS: u32 = 0;

/// Bounded poll budget: at ~5 ms per nanosleep this is generous head-room for
/// the worker to start, heartbeat, and exit under slow TCG, while guaranteeing
/// PID1 can never spin forever if something goes wrong.
const MAX_POLL_ITERS: u32 = 4000;
/// Short per-iteration sleep (~5 ms): keeps PID1 returning to user often so a
/// posted SIGCHLD is delivered promptly, without busy-spamming the tracer.
const POLL_SLEEP_NS: i64 = 5_000_000;

/// The SIGCHLD handler bumps this `.bss` counter. Observed back in `user_main`'s
/// poll loop. A counter (not a bare bool) lets PID1 see *how many* children
/// reported, and a non-zero value cannot be faked by a stray zero-write.
static SIGCHLD_COUNT: AtomicU32 = AtomicU32::new(0);
/// The SIGTERM handler sets this `.bss` shutdown flag (proves a second handler
/// is installed + independently deliverable). The SIGCHLD path drives this demo.
static SHUTDOWN_FLAG: AtomicU32 = AtomicU32::new(0);
const SHUTDOWN_MAGIC: u32 = 0x5747; // "SHUTD"-ish magic.

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

// --- Signal handlers + restorer ---------------------------------------------

/// SIGCHLD handler: bump the `.bss` counter. The kernel enters with rdi=signo;
/// it returns through `__restore` -> rt_sigreturn (SA_RESTORER).
extern "C" fn sigchld_handler(_signo: u64) {
    SIGCHLD_COUNT.fetch_add(1, Ordering::SeqCst);
}

/// SIGTERM handler: set the `.bss` shutdown flag.
extern "C" fn sigterm_handler(_signo: u64) {
    SHUTDOWN_FLAG.store(SHUTDOWN_MAGIC, Ordering::SeqCst);
}

/// `__restore`: the SA_RESTORER trampoline. After a handler `ret`s here, this
/// issues `rt_sigreturn` (`mov rax, 15; syscall`), exactly like musl's
/// `__restore_rt`.
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn __restore() -> ! {
    core::arch::naked_asm!("mov rax, 15", "syscall");
}

/// Install `handler` for `signo` via `rt_sigaction` + `SA_RESTORER`. Returns
/// true on success.
fn install_handler(signo: u64, handler: extern "C" fn(u64)) -> bool {
    // struct sigaction { sa_handler@0; sa_flags@8; sa_restorer@16; sa_mask@24 }.
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

// --- VFS contract: re-open /dev/console and dup2 onto 0/1/2 -----------------

/// Exercise the fd table on the inherited stdio: `openat("/dev/console")`, then
/// `dup2` it onto fds 0,1,2. Idempotent (the kernel already wired 0/1/2 to the
/// console at process creation) but proves the VFS openat/dup2 paths for PID1.
/// Returns true on success.
fn reopen_console_stdio() -> bool {
    const CONSOLE_PATH: &[u8] = b"/dev/console\0";
    // openat(AT_FDCWD, "/dev/console", O_RDWR=2, 0)
    // SAFETY: CONSOLE_PATH is a valid NUL-terminated user pointer.
    let fd = unsafe {
        syscall4(
            SYS_OPENAT,
            AT_FDCWD,
            CONSOLE_PATH.as_ptr() as u64,
            2, /* O_RDWR */
            0,
        )
    };
    if fd < 0 {
        return false;
    }
    // dup2 onto 0,1,2.
    for target in 0u64..=2 {
        // SAFETY: dup2 duplicates an open fd to a fixed slot.
        let d = unsafe { syscall3(SYS_DUP2, fd as u64, target, 0) };
        if d != target as i64 {
            return false;
        }
    }
    // Close the original fd if it was not one of 0/1/2.
    if fd > 2 {
        // SAFETY: closing an open fd.
        let _ = unsafe { syscall1(SYS_CLOSE, fd as u64) };
    }
    true
}

// --- Program logic ----------------------------------------------------------

/// The path passed to execve. The kernel has no filesystem, so it ignores the
/// string content and loads its embedded worker image (the real `svc` under
/// `--features init-demo`) — but the pointer must be a valid, non-NULL user
/// address (execve validates it).
const SVC_PATH: &[u8] = b"/svc\0";

#[unsafe(no_mangle)]
extern "C" fn user_main() -> ! {
    let my_pid = getpid();
    print(b"init: PID1 starting pid=");
    print_u32(my_pid);
    print(b"\n");

    // (1) VFS contract: re-open /dev/console -> dup2 0/1/2.
    if reopen_console_stdio() {
        print(b"init: console stdio ready\n");
    } else {
        // Non-fatal: the inherited fds 0/1/2 already point at the console.
        print(b"init: console reopen skipped (using inherited fds)\n");
    }

    // (2) install SIGCHLD handler.
    if !install_handler(SIGCHLD, sigchld_handler) {
        print(b"init: rt_sigaction(SIGCHLD) FAILED\n");
        sys_exit(1);
    }
    // (3) install SIGTERM handler.
    if !install_handler(SIGTERM, sigterm_handler) {
        print(b"init: rt_sigaction(SIGTERM) FAILED\n");
        sys_exit(1);
    }
    print(b"init: handlers installed (SIGCHLD, SIGTERM)\n");

    // (4) clone the worker child.
    // SAFETY: bare fork ABI: rdi = SIGCHLD exit-signal, rsi = 0 (no new stack).
    let rc = unsafe { syscall2(SYS_CLONE, SIGCHLD, 0) };
    if rc == 0 {
        // ---- child ----
        // execve the worker. The kernel loads the embedded worker image (the
        // real svc under init-demo). On success this never returns.
        // SAFETY: SVC_PATH is a valid NUL-terminated user pointer; argv/envp NULL.
        unsafe {
            syscall3(SYS_EXECVE, SVC_PATH.as_ptr() as u64, 0, 0);
        }
        // execve only returns on failure.
        print(b"init: child execve FAILED\n");
        sys_exit(1);
    } else if rc < 0 {
        print(b"init: clone FAILED\n");
        sys_exit(1);
    }

    // ---- PID1 (parent) ----
    let svc_pid = rc as u32;
    print(b"init: spawned svc pid=");
    print_u32(svc_pid);
    print(b"\n");

    // (5) poll loop: short clock_nanosleep, watch the SIGCHLD counter. This is
    // the signal-driven reap: the kernel posts SIGCHLD to PID1 when svc exits;
    // on PID1's next return-to-user the handler runs and bumps the counter.
    let req = Timespec { tv_sec: 0, tv_nsec: POLL_SLEEP_NS };
    let mut saw_sigchld = false;
    let mut iters: u32 = 0;
    while iters < MAX_POLL_ITERS {
        if SIGCHLD_COUNT.load(Ordering::SeqCst) > 0 {
            saw_sigchld = true;
            break;
        }
        // A short sleep; ignore EINTR-style returns (this kernel has none) and
        // any benign nonzero — we only care about the flag and the loop bound.
        let _ = clock_nanosleep(&req);
        iters += 1;
    }

    if !saw_sigchld {
        print(b"init: TIMEOUT waiting for SIGCHLD\n");
        sys_exit(1);
    }

    // (6) the handler fired -> reap with WNOHANG (proves the SIGNAL drove this,
    // not a synchronous blocking wait4).
    print(b"init: SIGCHLD received\n");

    let mut status: i32 = 0;
    // SAFETY: passes a valid i32 pointer for the kernel to write the status.
    let reaped = unsafe {
        syscall4(
            SYS_WAIT4,
            (-1i64) as u64, // any child
            &mut status as *mut i32 as u64,
            WNOHANG,
            0,
        )
    };
    if reaped <= 0 {
        print(b"init: wait4(WNOHANG) found no zombie (reaped=");
        if reaped < 0 {
            print(b"err)\n");
        } else {
            print(b"0)\n");
        }
        sys_exit(1);
    }

    // Decode WEXITSTATUS: Linux encodes a normal exit as (code & 0xff) << 8.
    let wexit = ((status >> 8) & 0xff) as u32;
    print(b"init: reaped svc pid=");
    print_u32(reaped as u32);
    print(b" status=");
    print_u32(wexit);
    print(b"\n");

    if reaped as u32 != svc_pid || wexit != EXPECT_STATUS {
        print(b"init: MISMATCH (pid/status)\n");
        sys_exit(1);
    }

    // (7) clean shutdown.
    print(b"init: shutdown\n");
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
