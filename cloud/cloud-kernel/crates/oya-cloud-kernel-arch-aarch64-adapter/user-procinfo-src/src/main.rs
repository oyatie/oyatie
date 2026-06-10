//! user-procinfo: an aarch64 EL0 program that proves the **WAVE 1** libc-init /
//! process-info syscalls (uname, umask, getrusage, times, clock_getres, prctl)
//! return REAL results — never `-ENOSYS` (-38) — using only raw Linux/aarch64
//! `svc #0`. The aarch64 analogue of user-procinfo-x86_64.
//!
//! Each of the six WAVE 1 syscalls is invoked and its result range-checked
//! against the Linux ABI contract the kernel implements:
//!   1. uname(buf)        -> 0, buf.sysname=="Linux", buf.machine=="aarch64".
//!   2. umask(027)        -> previous mask (Linux init default 022), and a second
//!                           umask(022) returns the 027 we just set (state swap).
//!   3. getrusage(0,&ru)  -> 0, rusage struct zeroed.
//!   4. times(&tms)       -> non-negative monotonic tick (NOT -38), tms zeroed.
//!   5. clock_getres(MONO,&ts) -> 0, ts == {0, 1} (1 ns resolution).
//!   6. prctl(PR_GET_NAME,&buf) -> 0 (kernel zeroes the 16-byte buffer).
//! Prints one `procinfo: <name> ok ...` proof line per syscall (regex-disjoint
//! from the `[pid N] syscall NR -> RET` trace shape) and a final
//! `procinfo: WAVE1 ALL OK`, then `exit(0)`. ANY ENOSYS / bad result -> exit(42).
//!
//! `no_std` + `no_main`; raw `svc` (outside the kernel TCB).

#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

// --- Linux aarch64 syscall numbers (WAVE 1 + write/exit) --------------------
const SYS_WRITE: usize = 64;
const SYS_CLOCK_GETRES: usize = 114;
const SYS_TIMES: usize = 153;
const SYS_UNAME: usize = 160;
const SYS_GETRUSAGE: usize = 165;
const SYS_UMASK: usize = 166;
const SYS_PRCTL: usize = 167;
const SYS_EXIT: usize = 93;
const SYS_EXIT_GROUP: usize = 94;

const STDOUT: usize = 1;

const CLOCK_MONOTONIC: usize = 1;
const PR_GET_NAME: usize = 16;

const ENOSYS: isize = -38;

const EXIT_OK: usize = 0;
const EXIT_FAIL: usize = 42;

// --- Raw syscall wrappers ---------------------------------------------------

#[inline(always)]
unsafe fn syscall1(nr: usize, a0: usize) -> isize {
    let ret;
    asm!("svc #0", in("x8") nr, inout("x0") a0 => ret, options(nostack));
    ret
}

#[inline(always)]
unsafe fn syscall2(nr: usize, a0: usize, a1: usize) -> isize {
    let ret;
    asm!("svc #0", in("x8") nr, inout("x0") a0 => ret, in("x1") a1, options(nostack));
    ret
}

// --- output helpers ---------------------------------------------------------

fn sys_write(fd: usize, buf: &[u8]) {
    // SAFETY: valid pointer/length handed to write.
    unsafe {
        let _ = syscall_write(SYS_WRITE, fd, buf.as_ptr() as usize, buf.len());
    }
}

#[inline(always)]
unsafe fn syscall_write(nr: usize, a0: usize, a1: usize, a2: usize) -> isize {
    let ret;
    asm!(
        "svc #0",
        in("x8") nr,
        inout("x0") a0 => ret,
        in("x1") a1,
        in("x2") a2,
        options(nostack),
    );
    ret
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

fn fail(msg: &[u8], rc: isize) -> ! {
    print(msg);
    print(b" rc=");
    print_i64(rc as i64);
    print(b"\n");
    sys_exit(EXIT_FAIL)
}

fn sys_exit(code: usize) -> ! {
    // SAFETY: exit/exit_group terminate the process and never return.
    unsafe {
        syscall1(SYS_EXIT, code);
        syscall1(SYS_EXIT_GROUP, code);
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

/// Compare the first `needle.len()` bytes of `field` to `needle`, requiring a
/// NUL terminator immediately after.
fn field_eq(field: &[u8], needle: &[u8]) -> bool {
    if field.len() < needle.len() + 1 {
        return false;
    }
    let mut i = 0;
    while i < needle.len() {
        if field[i] != needle[i] {
            return false;
        }
        i += 1;
    }
    field[needle.len()] == 0
}

// --- Program logic ----------------------------------------------------------

#[unsafe(no_mangle)]
extern "C" fn user_main() -> ! {
    // 1. uname(buf): sysname must be "Linux", machine "aarch64".
    let mut uts = [0xAAu8; UTSNAME_SIZE];
    let rc = unsafe { syscall1(SYS_UNAME, uts.as_mut_ptr() as usize) };
    if rc == ENOSYS {
        fail(b"procinfo: uname ENOSYS", rc);
    }
    if rc != 0 {
        fail(b"procinfo: uname nonzero", rc);
    }
    if !field_eq(&uts[0..UTS_FIELD_LEN], b"Linux") {
        fail(b"procinfo: uname sysname!=Linux", 0);
    }
    if !field_eq(&uts[4 * UTS_FIELD_LEN..5 * UTS_FIELD_LEN], b"aarch64") {
        fail(b"procinfo: uname machine!=aarch64", 0);
    }
    print(b"procinfo: uname ok sysname=Linux machine=aarch64\n");

    // 2. umask(027) returns prior mask (Linux init default 022); umask(022) then
    //    returns the 027 we just installed (proves real swap).
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
    let rc = unsafe { syscall2(SYS_GETRUSAGE, 0, ru.as_mut_ptr() as usize) };
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
    let tick = unsafe { syscall1(SYS_TIMES, tms.as_mut_ptr() as usize) };
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
    print_i64(tick as i64);
    print(b"\n");

    // 5. clock_getres(CLOCK_MONOTONIC, &ts) -> 0, ts == {0, 1}.
    let mut ts = Timespec { tv_sec: -1, tv_nsec: -1 };
    let rc = unsafe { syscall2(SYS_CLOCK_GETRES, CLOCK_MONOTONIC, &mut ts as *mut Timespec as usize) };
    if rc == ENOSYS {
        fail(b"procinfo: clock_getres ENOSYS", rc);
    }
    if rc != 0 {
        fail(b"procinfo: clock_getres nonzero", rc);
    }
    if ts.tv_sec != 0 || ts.tv_nsec != 1 {
        fail(b"procinfo: clock_getres res!={0,1}", ts.tv_nsec as isize);
    }
    print(b"procinfo: clock_getres ok res=1ns\n");

    // 6. prctl(PR_GET_NAME, &buf) -> 0; kernel zeroes the 16-byte buffer.
    let mut name = [0xAAu8; 16];
    let rc = unsafe { syscall2(SYS_PRCTL, PR_GET_NAME, name.as_mut_ptr() as usize) };
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

/// `_start`: the kernel enters here with `SP_EL0` already pointing at a valid
/// SysV initial stack, so we just tail-call Rust.
#[unsafe(no_mangle)]
#[unsafe(naked)]
#[unsafe(link_section = ".text.start")]
unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        "bl   {user_main}",
        "1:",
        "wfe",
        "b    1b",
        user_main = sym user_main,
    )
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    sys_exit(EXIT_FAIL)
}
