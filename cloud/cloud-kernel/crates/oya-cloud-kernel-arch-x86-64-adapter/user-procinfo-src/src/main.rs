//! user-procinfo-x86_64: the ring-3 program that proves the **WAVE 1** libc-init
//! / process-info syscalls (uname, umask, getrusage, times, clock_getres, prctl)
//! return REAL results — never `-ENOSYS` (-38) — on x86_64, using only raw
//! Linux/x86_64 `syscall`s. The conformance sibling of user-clock-x86_64.
//!
//! Each of the six WAVE 1 syscalls is invoked and its result range-checked
//! against the Linux ABI contract the kernel implements:
//!   1. uname(buf)        -> 0, and buf.sysname=="Linux", buf.machine=="x86_64".
//!   2. umask(027)        -> previous mask (Linux init default 022), and a second
//!                           umask(022) returns the 027 we just set (state swap).
//!   3. getrusage(0,&ru)  -> 0, and the rusage struct was zeroed.
//!   4. times(&tms)       -> a non-negative monotonic tick (NOT -38), tms zeroed.
//!   5. clock_getres(MONO,&ts) -> 0, ts == {0, 1} (1 ns resolution).
//!   6. prctl(PR_GET_NAME,&buf) -> 0 (and buf zeroed by the kernel).
//! On success it prints exactly one proof line per syscall in the form
//!   `procinfo: <name> ok ...`
//! (regex-disjoint from the `[pid N] syscall NR -> RET` trace shape) and a final
//! `procinfo: WAVE1 ALL OK` then `exit(0)`. ANY ENOSYS / bad result -> exit(42)
//! with a diagnostic. Because the kernel logs every syscall as
//! `[pid N] syscall NR -> RET`, the captured serial trace independently shows
//! each WAVE 1 NR returning a non-(-38) value, which the probe asserts here too.
//!
//! `no_std` + `no_main`; raw `syscall` (outside the kernel TCB).

#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

// --- Linux x86_64 syscall numbers (WAVE 1 + write/exit) ---------------------
const SYS_WRITE: u64 = 1;
const SYS_UNAME: u64 = 63;
const SYS_UMASK: u64 = 95;
const SYS_GETRUSAGE: u64 = 98;
const SYS_TIMES: u64 = 100;
const SYS_PRCTL: u64 = 157;
const SYS_CLOCK_GETRES: u64 = 229;
const SYS_EXIT: u64 = 60;
const SYS_EXIT_GROUP: u64 = 231;

const STDOUT: u64 = 1;

const CLOCK_MONOTONIC: u64 = 1;
const PR_GET_NAME: u64 = 16;

const ENOSYS: i64 = -38;

const EXIT_OK: i32 = 0;
const EXIT_FAIL: i32 = 42;

// --- Raw syscall wrappers ---------------------------------------------------

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

// --- output helpers ---------------------------------------------------------

fn sys_write(fd: u64, buf: &[u8]) {
    // SAFETY: valid pointer/length handed to write.
    unsafe {
        syscall3(SYS_WRITE, fd, buf.as_ptr() as u64, buf.len() as u64);
    }
}

fn print(s: &[u8]) {
    sys_write(STDOUT, s);
}

fn print_i64(v: i64) {
    if v < 0 {
        print(b"-");
    }
    let mut u = if v < 0 { (v as i128).unsigned_abs() as u64 } else { v as u64 };
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    if u == 0 {
        print(b"0");
        return;
    }
    while u > 0 {
        i -= 1;
        buf[i] = b'0' + (u % 10) as u8;
        u /= 10;
    }
    print(&buf[i..]);
}

fn fail(msg: &[u8], rc: i64) -> ! {
    print(msg);
    print(b" rc=");
    print_i64(rc);
    print(b"\n");
    sys_exit(EXIT_FAIL)
}

fn sys_exit(code: i32) -> ! {
    // SAFETY: exit/exit_group terminate the process.
    unsafe {
        syscall1(SYS_EXIT, code as u64);
        syscall1(SYS_EXIT_GROUP, code as u64);
    }
    loop {
        core::hint::spin_loop();
    }
}

// --- ABI structs ------------------------------------------------------------

const UTS_FIELD_LEN: usize = 65;
const UTSNAME_SIZE: usize = UTS_FIELD_LEN * 6;

#[repr(C)]
#[derive(Clone, Copy)]
struct Timespec {
    tv_sec: i64,
    tv_nsec: i64,
}

/// Compare the first `needle.len()` bytes of `field` to `needle`.
fn field_eq(field: &[u8], needle: &[u8]) -> bool {
    if field.len() < needle.len() {
        return false;
    }
    let mut i = 0;
    while i < needle.len() {
        if field[i] != needle[i] {
            return false;
        }
        i += 1;
    }
    // The byte right after the match must be the field's NUL terminator.
    field[needle.len()] == 0
}

// --- Program logic ----------------------------------------------------------

#[unsafe(no_mangle)]
extern "C" fn user_main() -> ! {
    // 1. uname(buf): sysname must be "Linux", machine "x86_64".
    let mut uts = [0xAAu8; UTSNAME_SIZE];
    let rc = unsafe { syscall1(SYS_UNAME, uts.as_mut_ptr() as u64) };
    if rc == ENOSYS {
        fail(b"procinfo: uname ENOSYS", rc);
    }
    if rc != 0 {
        fail(b"procinfo: uname nonzero", rc);
    }
    if !field_eq(&uts[0..UTS_FIELD_LEN], b"Linux") {
        fail(b"procinfo: uname sysname!=Linux", 0);
    }
    if !field_eq(&uts[4 * UTS_FIELD_LEN..5 * UTS_FIELD_LEN], b"x86_64") {
        fail(b"procinfo: uname machine!=x86_64", 0);
    }
    print(b"procinfo: uname ok sysname=Linux machine=x86_64\n");

    // 2. umask(027) returns the prior mask (Linux init default 022); a second
    //    umask(022) must return the 027 we just installed (proves real swap).
    let prev = unsafe { syscall1(SYS_UMASK, 0o027) };
    if prev == ENOSYS {
        fail(b"procinfo: umask ENOSYS", prev);
    }
    if prev != 0o022 {
        fail(b"procinfo: umask prev!=022", prev);
    }
    let prev2 = unsafe { syscall1(SYS_UMASK, 0o022) };
    if prev2 != 0o027 {
        fail(b"procinfo: umask swap!=027", prev2);
    }
    print(b"procinfo: umask ok prev=022 swapped=027\n");

    // 3. getrusage(RUSAGE_SELF=0, &ru) -> 0, ru zeroed.
    let mut ru = [0xAAu8; 144];
    let rc = unsafe { syscall2(SYS_GETRUSAGE, 0, ru.as_mut_ptr() as u64) };
    if rc == ENOSYS {
        fail(b"procinfo: getrusage ENOSYS", rc);
    }
    if rc != 0 {
        fail(b"procinfo: getrusage nonzero", rc);
    }
    let mut any = 0u8;
    for b in ru.iter() {
        any |= *b;
    }
    if any != 0 {
        fail(b"procinfo: getrusage not zeroed", 0);
    }
    print(b"procinfo: getrusage ok zeroed=144\n");

    // 4. times(&tms) -> non-negative monotonic tick (NOT -38), tms zeroed.
    let mut tms = [0xAAu8; 32];
    let tick = unsafe { syscall1(SYS_TIMES, tms.as_mut_ptr() as u64) };
    if tick == ENOSYS {
        fail(b"procinfo: times ENOSYS", tick);
    }
    if tick < 0 {
        fail(b"procinfo: times negative", tick);
    }
    let mut any = 0u8;
    for b in tms.iter() {
        any |= *b;
    }
    if any != 0 {
        fail(b"procinfo: times tms not zeroed", 0);
    }
    print(b"procinfo: times ok tick=");
    print_i64(tick);
    print(b"\n");

    // 5. clock_getres(CLOCK_MONOTONIC, &ts) -> 0, ts == {0, 1}.
    let mut ts = Timespec { tv_sec: -1, tv_nsec: -1 };
    let rc = unsafe { syscall2(SYS_CLOCK_GETRES, CLOCK_MONOTONIC, &mut ts as *mut Timespec as u64) };
    if rc == ENOSYS {
        fail(b"procinfo: clock_getres ENOSYS", rc);
    }
    if rc != 0 {
        fail(b"procinfo: clock_getres nonzero", rc);
    }
    if ts.tv_sec != 0 || ts.tv_nsec != 1 {
        fail(b"procinfo: clock_getres res!={0,1}", ts.tv_nsec);
    }
    print(b"procinfo: clock_getres ok res=1ns\n");

    // 6. prctl(PR_GET_NAME, &buf) -> 0; kernel zeroes the 16-byte buffer.
    let mut name = [0xAAu8; 16];
    let rc = unsafe { syscall2(SYS_PRCTL, PR_GET_NAME, name.as_mut_ptr() as u64) };
    if rc == ENOSYS {
        fail(b"procinfo: prctl ENOSYS", rc);
    }
    if rc != 0 {
        fail(b"procinfo: prctl nonzero", rc);
    }
    let mut any = 0u8;
    for b in name.iter() {
        any |= *b;
    }
    if any != 0 {
        fail(b"procinfo: prctl name not zeroed", 0);
    }
    print(b"procinfo: prctl ok PR_GET_NAME\n");

    // The aggregate proof line. Deliberately NOT in the trace regex shape.
    print(b"procinfo: WAVE1 ALL OK\n");
    sys_exit(EXIT_OK);
}

/// The kernel enters here with a valid rsp (SysV initial stack), so we tail-call.
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
    sys_exit(EXIT_FAIL)
}
