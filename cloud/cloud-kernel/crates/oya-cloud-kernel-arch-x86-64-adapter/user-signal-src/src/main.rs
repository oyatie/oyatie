//! user-signal-x86_64: the ring-3 program that exercises the kuberos **real
//! POSIX signal-delivery** slice end to end, then the **VFS fd** paths, using
//! only raw Linux/x86_64 syscalls (`syscall`). The x86_64 analogue of
//! ../../../kernel-usermode-tests/signal (aarch64).
//!
//! Flow:
//!   1. install a `SIGUSR1` (10) handler via `rt_sigaction`, with `SA_RESTORER`
//!      pointing at an in-program `__restore` trampoline (`mov rax,15; syscall`
//!      = `rt_sigreturn`);
//!   2. raise it at ourselves via `tgkill(getpid(), getpid(), SIGUSR1)`;
//!   3. the kernel delivers it on the return-to-user from `tgkill`: the handler
//!      runs (setting a `.bss` flag) and `ret`s through the restorer ->
//!      `rt_sigreturn`, which resumes us right after the `tgkill` syscall;
//!   4. assert the flag; print a **regex-disjoint** proof line
//!      `signal: SIGUSR1 handler ran, flag=0x...`;
//!   5. exercise the VFS fd syscalls: `openat("/dev/null")`, `dup2`/`dup3` it,
//!      `write` to it, `close`; print `vfs: openat/dup/close ok`;
//!   6. `exit(0)` on full success, `exit(42)` on any failure.
//!
//! `no_std` + `no_main`; raw `syscall`.

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
const SYS_EXIT: u64 = 60;
const SYS_TGKILL: u64 = 234;
const SYS_OPENAT: u64 = 257;

const STDOUT: u64 = 1;
const AT_FDCWD: u64 = (-100i64) as u64;

const SIGUSR1: u64 = 10;
const SA_RESTORER: u64 = 0x0400_0000;
const SIGSET_SIZE: u64 = 8;

/// The `.bss` flag the SIGUSR1 handler sets, observed back in `user_main` after
/// `tgkill` returns. A non-trivial magic so a stray zero-write cannot fake it.
static HANDLER_FLAG: AtomicU32 = AtomicU32::new(0);
const FLAG_MAGIC: u32 = 0xABCD;

// --- Raw syscall wrappers ---------------------------------------------------

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
    // SAFETY: valid pointer/length handed to write.
    unsafe {
        syscall3(SYS_WRITE, STDOUT, s.as_ptr() as u64, s.len() as u64);
    }
}

fn print_hex(v: u32) {
    let mut buf = [0u8; 8];
    let hexd = b"0123456789abcdef";
    let mut started = false;
    print(b"0x");
    let mut i = 0;
    for shift in (0..8).rev() {
        let nib = ((v >> (shift * 4)) & 0xf) as usize;
        if nib != 0 || started || shift == 0 {
            buf[i] = hexd[nib];
            i += 1;
            started = true;
        }
    }
    print(&buf[..i]);
}

fn getpid() -> u64 {
    // SAFETY: getpid takes no arguments and only reads kernel state.
    (unsafe { syscall0(SYS_GETPID) }) as u64
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

// --- Signal handler + restorer ----------------------------------------------

/// The SIGUSR1 handler. The kernel enters it with rdi=signo, rsi=siginfo*,
/// rdx=ucontext*, and the restorer address pushed as the return address. It sets
/// the `.bss` flag and returns; the `ret` lands in `__restore` -> rt_sigreturn.
extern "C" fn sigusr1_handler(_signo: u64) {
    HANDLER_FLAG.store(FLAG_MAGIC, Ordering::SeqCst);
}

/// `__restore`: the SA_RESTORER trampoline. After the handler `ret`s here, this
/// issues `rt_sigreturn` (`mov rax, 15; syscall`), exactly like musl's
/// `__restore_rt`.
#[unsafe(no_mangle)]
#[unsafe(naked)]
unsafe extern "C" fn __restore() -> ! {
    core::arch::naked_asm!("mov rax, 15", "syscall");
}

// --- Program logic ----------------------------------------------------------

#[unsafe(no_mangle)]
extern "C" fn user_main() -> ! {
    print(b"user-signal: installing SIGUSR1 handler\n");

    // struct sigaction { sa_handler@0; sa_flags@8; sa_restorer@16; sa_mask@24 }.
    #[repr(C)]
    struct KSigaction {
        handler: u64,
        flags: u64,
        restorer: u64,
        mask: u64,
    }
    let act = KSigaction {
        handler: sigusr1_handler as *const () as u64,
        flags: SA_RESTORER,
        restorer: __restore as *const () as u64,
        mask: 0,
    };
    // SAFETY: `&act` is a valid 32-byte readable user pointer; oldact = NULL.
    let rc = unsafe {
        syscall4(
            SYS_RT_SIGACTION,
            SIGUSR1,
            &act as *const KSigaction as u64,
            0,
            SIGSET_SIZE,
        )
    };
    if rc != 0 {
        print(b"user-signal: rt_sigaction FAILED\n");
        sys_exit(42);
    }

    // Raise SIGUSR1 at ourselves; delivered on the return-to-user from tgkill.
    let pid = getpid();
    print(b"user-signal: raising SIGUSR1 via tgkill\n");
    // SAFETY: tgkill(tgid, tid, sig); tgid == tid == our pid (single thread).
    let rc = unsafe { syscall3(SYS_TGKILL, pid, pid, SIGUSR1) };
    if rc != 0 {
        print(b"user-signal: tgkill FAILED\n");
        sys_exit(42);
    }

    let flag = HANDLER_FLAG.load(Ordering::SeqCst);
    if flag != FLAG_MAGIC {
        print(b"user-signal: handler did NOT run\n");
        sys_exit(42);
    }
    // Proof line — NOT in the `[pid N] syscall NR -> RET` shape.
    print(b"signal: SIGUSR1 handler ran, flag=");
    print_hex(flag);
    print(b"\n");

    // ---- exercise the VFS fd paths (openat/dup2/dup3/write/close) ----
    if !vfs_exercise() {
        print(b"user-signal: vfs exercise FAILED\n");
        sys_exit(42);
    }
    print(b"vfs: openat/dup/close ok\n");

    print(b"user-signal: OK\n");
    sys_exit(0);
}

/// Exercise the minimal devtmpfs fd layer: open `/dev/null`, dup2 it to a fresh
/// fd, write to the dup, then close both. Returns true on success.
fn vfs_exercise() -> bool {
    const NULL_PATH: &[u8] = b"/dev/null\0";
    // openat(AT_FDCWD, "/dev/null", O_WRONLY=1, 0)
    // SAFETY: NULL_PATH is a valid NUL-terminated user pointer.
    let fd = unsafe {
        syscall4(
            SYS_OPENAT,
            AT_FDCWD,
            NULL_PATH.as_ptr() as u64,
            1, /* O_WRONLY */
            0,
        )
    };
    if fd < 0 {
        return false;
    }
    // dup2(fd, 7) -> 7.
    let newfd: u64 = 7;
    // SAFETY: dup2 duplicates an open fd to a new slot.
    let dup = unsafe { syscall3(SYS_DUP2, fd as u64, newfd, 0) };
    if dup != newfd as i64 {
        return false;
    }
    // write(newfd, msg, len) -> discarded by /dev/null, returns len.
    const MSG: &[u8] = b"to /dev/null\n";
    // SAFETY: valid buffer; newfd is the dup of the /dev/null description.
    let w = unsafe { syscall3(SYS_WRITE, newfd, MSG.as_ptr() as u64, MSG.len() as u64) };
    if w != MSG.len() as i64 {
        return false;
    }
    // close both fds.
    // SAFETY: closing two open fds.
    let c1 = unsafe { syscall1(SYS_CLOSE, fd as u64) };
    let c2 = unsafe { syscall1(SYS_CLOSE, newfd) };
    c1 == 0 && c2 == 0
}

/// The kernel enters here with a valid rsp (SysV initial stack) already set up,
/// so we just tail-call Rust.
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
    sys_exit(42)
}
