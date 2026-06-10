//! user-smpdemo (aarch64): an EL0 program that PROVES cross-CPU scheduling under
//! `-smp` (P4·SMP·S4a). It forks `N` (> cpu_count) worker processes; the kernel
//! places them round-robin across the online CPUs and logs `sched: pid P -> cpu
//! K` the first time each runs. Each worker spins briefly (so the periodic timer
//! preempts it and the scheduler interleaves the workers across CPUs), then exits
//! with a distinct code; the parent `wait4`s all `N`. Under `-smp 4` the kernel's
//! `sched:` lines show workers on multiple distinct cpu indices (0,1,2,3 — not
//! all on the BSP); on 1-vCPU they are all `-> cpu 0`.
//!
//! Raw Linux/aarch64 syscalls only (`svc #0`); `no_std`/`no_main`; allows
//! `unsafe` because an EL0 program is outside the kernel TCB.

#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

const SYS_WRITE: usize = 64;
const SYS_EXIT: usize = 93;
const SYS_EXIT_GROUP: usize = 94;
const SYS_CLONE: usize = 220;
const SYS_GETPID: usize = 172;
const SYS_WAIT4: usize = 260;

const STDOUT: usize = 1;
const SIGCHLD: usize = 17;

/// Number of workers to fan out (> the test's `-smp 4`).
const N_WORKERS: usize = 8;

#[inline(always)]
unsafe fn syscall0(nr: usize) -> usize {
    let ret;
    asm!("svc #0", in("x8") nr, out("x0") ret, options(nostack));
    ret
}
#[inline(always)]
unsafe fn syscall1(nr: usize, a0: usize) -> usize {
    let ret;
    asm!("svc #0", in("x8") nr, inout("x0") a0 => ret, options(nostack));
    ret
}
#[inline(always)]
unsafe fn syscall2(nr: usize, a0: usize, a1: usize) -> usize {
    let ret;
    asm!("svc #0", in("x8") nr, inout("x0") a0 => ret, in("x1") a1, options(nostack));
    ret
}
#[inline(always)]
unsafe fn syscall3(nr: usize, a0: usize, a1: usize, a2: usize) -> usize {
    let ret;
    asm!("svc #0", in("x8") nr, inout("x0") a0 => ret, in("x1") a1, in("x2") a2,
         options(nostack));
    ret
}
#[inline(always)]
unsafe fn syscall4(nr: usize, a0: usize, a1: usize, a2: usize, a3: usize) -> usize {
    let ret;
    asm!("svc #0", in("x8") nr, inout("x0") a0 => ret, in("x1") a1, in("x2") a2,
         in("x3") a3, options(nostack));
    ret
}

fn print(s: &[u8]) {
    // SAFETY: valid pointer/length pair handed to the kernel `write`.
    let _ = unsafe { syscall3(SYS_WRITE, STDOUT, s.as_ptr() as usize, s.len()) };
}
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
    // SAFETY: getpid takes no args.
    (unsafe { syscall0(SYS_GETPID) }) as u32
}
fn fork() -> isize {
    // SAFETY: bare fork ABI (SIGCHLD exit-signal, newsp=0).
    unsafe { syscall2(SYS_CLONE, SIGCHLD, 0) as isize }
}
fn wait4(pid: isize, status: &mut i32, options: usize) -> isize {
    // SAFETY: valid i32 pointer for the kernel to write the status.
    unsafe { syscall4(SYS_WAIT4, pid as usize, status as *mut i32 as usize, options, 0) as isize }
}
fn sys_exit(code: i32) -> ! {
    // SAFETY: terminates the process; never returns.
    unsafe {
        syscall1(SYS_EXIT, code as usize);
        syscall1(SYS_EXIT_GROUP, code as usize);
    }
    loop {
        core::hint::spin_loop();
    }
}

/// A worker's exit code = `WORKER_BASE + index` (< 256). Lets the parent verify.
const WORKER_BASE: u32 = 10;

/// Pure-CPU spin iterations per worker (NO syscalls in the loop, so the kernel
/// syscall trace is not flooded). Long enough that the periodic timer preempts
/// the worker so the scheduler interleaves the workers across CPUs; short enough
/// that the whole 8-worker fan-out completes within the TCG test timeout.
const WORKER_SPINS: u64 = 3_000_000;

#[unsafe(no_mangle)]
extern "C" fn user_main() -> ! {
    print(b"smpdemo: parent pid=");
    print_u32(getpid());
    print(b" spawning workers\n");

    let mut spawned: u32 = 0;
    for idx in 0..N_WORKERS {
        let rc = fork();
        if rc == 0 {
            // ---- worker `idx`: pure-CPU spin, then exit ----
            let mut acc: u64 = idx as u64;
            let mut k: u64 = 0;
            while k < WORKER_SPINS {
                acc = unsafe { core::ptr::read_volatile(&acc) }.wrapping_add(k);
                k += 1;
            }
            if acc == 0xDEAD_BEEF_DEAD_BEEF {
                print(b"");
            }
            sys_exit((WORKER_BASE + idx as u32) as i32);
        } else if rc < 0 {
            print(b"smpdemo: fork FAILED\n");
            sys_exit(1);
        }
        spawned += 1;
    }

    print(b"smpdemo: parent spawned ");
    print_u32(spawned);
    print(b" workers, reaping\n");

    let mut reaped: u32 = 0;
    while reaped < spawned {
        let mut status: i32 = 0;
        let pid = wait4(-1, &mut status, 0);
        if pid < 0 {
            print(b"smpdemo: wait4 FAILED\n");
            sys_exit(1);
        }
        reaped += 1;
    }

    print(b"smpdemo: parent reaped ");
    print_u32(reaped);
    print(b" workers OK\n");
    sys_exit(0);
}

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
    sys_exit(1)
}
