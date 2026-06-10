//! user-spawn-x86_64: a ring-3 user program that exercises the kuberos
//! **process model** end to end, using only raw Linux/x86_64 syscalls
//! (`syscall`). The x86_64 analogue of ../../../kernel-usermode-tests/spawn (aarch64).
//!
//! Flow (exercises clone + execve + wait4 + exit together):
//!   1. parent prints a banner and its pid (`getpid`),
//!   2. parent `clone(SIGCHLD)` (the `fork()` ABI) to create a child,
//!   3. the **child** prints its pid + ppid, then `execve()`s into a fresh image
//!      (`user-exec`) — replacing its process image while keeping its pid; the
//!      exec'd program prints and `exit(7)`s,
//!   4. the **parent** `wait4(-1, &status, 0, NULL)` blocks until the child
//!      becomes a zombie, reaps it, and prints the reaped pid + `WEXITSTATUS`
//!      (7, from the exec'd image — proving exec replaced the image),
//!   5. parent `exit(0)`.
//!
//! `no_std` + `no_main`; it allows `unsafe` because, as a ring-3 program making
//! raw `syscall`s, it is outside the kernel TCB.

#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

// --- Linux x86_64 syscall numbers we use -----------------------------------
const SYS_WRITE: u64 = 1;
const SYS_GETPID: u64 = 39;
const SYS_CLONE: u64 = 56;
const SYS_EXECVE: u64 = 59;
const SYS_EXIT: u64 = 60;
const SYS_WAIT4: u64 = 61;
const SYS_GETPPID: u64 = 110;
const SYS_EXIT_GROUP: u64 = 231;

const STDOUT: u64 = 1;

/// `clone` flag used by `fork()`: deliver SIGCHLD (17) to the parent on exit.
const SIGCHLD: u64 = 17;

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

fn sys_write(fd: u64, buf: &[u8]) {
    // SAFETY: valid pointer/length pair handed to the kernel `write`.
    let _ = unsafe { syscall3(SYS_WRITE, fd, buf.as_ptr() as u64, buf.len() as u64) };
}

fn print(s: &[u8]) {
    sys_write(STDOUT, s);
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

fn getppid() -> u32 {
    // SAFETY: getppid takes no arguments and only reads kernel state.
    (unsafe { syscall0(SYS_GETPPID) }) as u32
}

/// `fork()` via `clone(SIGCHLD, 0, ...)`. Returns child pid in the parent, 0 in
/// the child (the kernel sets the child's rax to 0).
fn fork() -> i64 {
    // SAFETY: clone with the bare fork ABI: rdi = SIGCHLD exit-signal, rsi = 0
    // (no new stack -> child shares the same SP, correct after a full address-
    // space copy). We pass newsp = 0 explicitly.
    unsafe { syscall2(SYS_CLONE, SIGCHLD, 0) }
}

/// `wait4(pid, &status, options, rusage)` -> reaped pid (status written).
fn wait4(pid: i64, status: &mut i32, options: u64) -> i64 {
    // SAFETY: passes a valid i32 pointer for the kernel to write the status.
    unsafe {
        syscall4(
            SYS_WAIT4,
            pid as u64,
            status as *mut i32 as u64,
            options,
            0,
        )
    }
}

fn sys_exit(code: i32) -> ! {
    // SAFETY: exit/exit_group terminate the process and never return.
    unsafe {
        syscall1(SYS_EXIT, code as u64);
        syscall1(SYS_EXIT_GROUP, code as u64);
    }
    loop {
        // SAFETY: HLT is invalid in ring 3, so just spin (never reached).
        core::hint::spin_loop();
    }
}

// --- Program logic ----------------------------------------------------------

/// Exit code the **exec'd** image (`user-exec`) terminates with; the parent
/// confirms it reaps this, proving execve replaced the child's image.
const EXEC_EXIT_CODE: u32 = 7;

/// The path passed to execve. The kernel has no filesystem, so it ignores the
/// string content and loads its embedded `user-exec` image — but the pointer
/// must be a valid, non-NULL user address (execve validates it).
const EXEC_PATH: &[u8] = b"/user-exec\0";

/// Rust entry, tail-called by the `_start` asm stub.
#[unsafe(no_mangle)]
extern "C" fn user_main() -> ! {
    print(b"user-spawn: parent pid=");
    print_u32(getpid());
    print(b" forking a child\n");

    let rc = fork();
    if rc == 0 {
        // ---- child ----
        print(b"user-spawn: child pid=");
        print_u32(getpid());
        print(b" ppid=");
        print_u32(getppid());
        print(b" execve-ing into user-exec\n");
        // Replace our image. On success this never returns; the exec'd program
        // runs and exits with EXEC_EXIT_CODE.
        // SAFETY: EXEC_PATH is a valid NUL-terminated user pointer; argv/envp
        // are NULL (the kernel ignores them for the embedded image).
        unsafe {
            syscall3(
                SYS_EXECVE,
                EXEC_PATH.as_ptr() as u64,
                0, // argv = NULL
                0, // envp = NULL
            );
        }
        print(b"user-spawn: execve FAILED\n");
        sys_exit(1);
    } else if rc < 0 {
        print(b"user-spawn: fork FAILED\n");
        sys_exit(1);
    }

    // ---- parent ----
    let child = rc as u32;
    print(b"user-spawn: parent forked child pid=");
    print_u32(child);
    print(b", waiting\n");

    let mut status: i32 = 0;
    let reaped = wait4(-1, &mut status, 0);
    if reaped < 0 {
        print(b"user-spawn: wait4 FAILED\n");
        sys_exit(1);
    }

    // Decode WEXITSTATUS: Linux encodes a normal exit as (code & 0xff) << 8.
    let wexit = ((status >> 8) & 0xff) as u32;
    print(b"user-spawn: reaped child pid=");
    print_u32(reaped as u32);
    print(b" exit status=");
    print_u32(wexit);
    print(b"\n");

    if reaped as u32 == child && wexit == EXEC_EXIT_CODE {
        print(b"user-spawn: OK parent reaped exec'd child correctly\n");
        sys_exit(0);
    }
    print(b"user-spawn: MISMATCH\n");
    sys_exit(1);
}

/// `_start`: the ELF entry point. The kernel's process-model loader enters here
/// with `rsp` already pointing at a valid SysV initial stack (high in the 2 MiB
/// user window). We just tail-call Rust, which never returns.
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
