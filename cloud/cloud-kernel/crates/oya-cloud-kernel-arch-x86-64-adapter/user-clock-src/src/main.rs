//! user-clock-x86_64: the ring-3 program that proves the kuberos **real
//! timekeeping** + clock syscalls (P3 slice 3, Tier 1) advance on x86_64, using
//! only raw Linux/x86_64 `syscall`s. The x86_64 analogue of
//! ../../../kernel-usermode-tests/clock (aarch64).
//!
//! Flow:
//!   1. `clock_gettime(CLOCK_MONOTONIC, &t0)`,
//!   2. `nanosleep(&req(~50ms), NULL)`,
//!   3. `clock_gettime(CLOCK_MONOTONIC, &t1)`,
//!   4. assert the monotonic delta is in a loose band (~20..500 ms — wide for
//!      TCG jitter) AND `clock_gettime(CLOCK_REALTIME)`'s seconds >= the fixed
//!      2024-01-01 epoch offset,
//!   5. print `clock: mono advanced delta_ms=NN realtime_sec=NNN` (regex-disjoint
//!      from `[pid N] syscall NR -> RET`) and `exit(0)`,
//!   6. on any failure print a diagnostic and `exit(42)`.
//!
//! `no_std` + `no_main`; raw `syscall`.

#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

// --- Linux x86_64 syscall numbers -------------------------------------------
const SYS_WRITE: u64 = 1;
const SYS_NANOSLEEP: u64 = 35;
const SYS_EXIT: u64 = 60;
const SYS_CLOCK_GETTIME: u64 = 228;
const SYS_EXIT_GROUP: u64 = 231;

const STDOUT: u64 = 1;

const CLOCK_REALTIME: u64 = 0;
const CLOCK_MONOTONIC: u64 = 1;

/// The fixed wall-clock epoch the kernel adds to monotonic for CLOCK_REALTIME:
/// 2024-01-01T00:00:00Z. Realtime seconds must be at least this.
const WALLCLOCK_OFFSET_SECS: i64 = 1_704_067_200;

/// Loose acceptance band for the measured monotonic delta (milliseconds).
const DELTA_MS_MIN: i64 = 20;
const DELTA_MS_MAX: i64 = 500;

/// Sleep request: ~50 ms.
const SLEEP_NS: i64 = 50_000_000;

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

// --- timespec + helpers -----------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Timespec {
    tv_sec: i64,
    tv_nsec: i64,
}

fn clock_gettime(clk: u64, ts: &mut Timespec) -> i64 {
    // SAFETY: `ts` is a valid 16-byte writable buffer for the kernel to fill.
    unsafe { syscall2(SYS_CLOCK_GETTIME, clk, ts as *mut Timespec as u64) }
}

fn nanosleep(req: &Timespec) -> i64 {
    // nanosleep(*req, *rem=NULL).
    // SAFETY: `req` is a valid 16-byte readable timespec; rem is NULL.
    unsafe { syscall2(SYS_NANOSLEEP, req as *const Timespec as u64, 0) }
}

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

fn delta_ms(a: &Timespec, b: &Timespec) -> i64 {
    let a_ns = a.tv_sec.wrapping_mul(1_000_000_000).wrapping_add(a.tv_nsec);
    let b_ns = b.tv_sec.wrapping_mul(1_000_000_000).wrapping_add(b.tv_nsec);
    (b_ns - a_ns) / 1_000_000
}

// --- Program logic ----------------------------------------------------------

#[unsafe(no_mangle)]
extern "C" fn user_main() -> ! {
    let mut t0 = Timespec::default();
    let mut t1 = Timespec::default();
    let mut rt = Timespec::default();

    if clock_gettime(CLOCK_MONOTONIC, &mut t0) != 0 {
        print(b"clock: clock_gettime(MONOTONIC) t0 FAILED\n");
        sys_exit(EXIT_FAIL);
    }

    let req = Timespec { tv_sec: 0, tv_nsec: SLEEP_NS };
    let rc = nanosleep(&req);
    if rc != 0 {
        print(b"clock: nanosleep FAILED rc=");
        print_i64(rc);
        print(b"\n");
        sys_exit(EXIT_FAIL);
    }

    if clock_gettime(CLOCK_MONOTONIC, &mut t1) != 0 {
        print(b"clock: clock_gettime(MONOTONIC) t1 FAILED\n");
        sys_exit(EXIT_FAIL);
    }

    if clock_gettime(CLOCK_REALTIME, &mut rt) != 0 {
        print(b"clock: clock_gettime(REALTIME) FAILED\n");
        sys_exit(EXIT_FAIL);
    }

    let d_ms = delta_ms(&t0, &t1);

    if d_ms < DELTA_MS_MIN || d_ms > DELTA_MS_MAX {
        print(b"clock: delta out of band delta_ms=");
        print_i64(d_ms);
        print(b"\n");
        sys_exit(EXIT_FAIL);
    }
    if rt.tv_sec < WALLCLOCK_OFFSET_SECS {
        print(b"clock: realtime below epoch realtime_sec=");
        print_i64(rt.tv_sec);
        print(b"\n");
        sys_exit(EXIT_FAIL);
    }

    // The proof line. Deliberately NOT in the `[pid N] syscall NR -> RET` shape.
    print(b"clock: mono advanced delta_ms=");
    print_i64(d_ms);
    print(b" realtime_sec=");
    print_i64(rt.tv_sec);
    print(b"\n");
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
