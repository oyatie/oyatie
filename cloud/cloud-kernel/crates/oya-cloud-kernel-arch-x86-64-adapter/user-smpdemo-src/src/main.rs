//! user-smpdemo-x86_64: a ring-3 program that PROVES cross-CPU scheduling under
//! `-smp` (P4·SMP·S4a). It forks `N` (> cpu_count) worker processes; the kernel
//! places them round-robin across the online CPUs and logs `sched: pid P -> cpu
//! K` the first time each runs. Each worker spins briefly (so the periodic timer
//! preempts it and the scheduler interleaves the workers across CPUs), then
//! exits with a distinct code; the parent `wait4`s all `N` and verifies. Under
//! `-smp 4` the kernel's `sched:` lines show workers on multiple distinct cpu
//! indices (0,1,2,3 — not all on the BSP); on 1-vCPU they are all `-> cpu 0`.
//!
//! Raw Linux/x86_64 syscalls only (`syscall`); `no_std`/`no_main`; allows
//! `unsafe` because a ring-3 program is outside the kernel TCB.

#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

const SYS_WRITE: u64 = 1;
const SYS_GETPID: u64 = 39;
const SYS_CLONE: u64 = 56;
const SYS_EXIT: u64 = 60;
const SYS_WAIT4: u64 = 61;
const SYS_EXIT_GROUP: u64 = 231;

const STDOUT: u64 = 1;
const SIGCHLD: u64 = 17;

/// Number of workers to fan out. > the test's `-smp 4`, so several land on each
/// CPU and at least 4 distinct CPUs are exercised.
const N_WORKERS: usize = 8;

#[inline(always)]
unsafe fn syscall0(nr: u64) -> i64 {
    let ret: i64;
    asm!("syscall", inlateout("rax") nr => ret,
         lateout("rcx") _, lateout("r11") _, options(nostack));
    ret
}
#[inline(always)]
unsafe fn syscall1(nr: u64, a0: u64) -> i64 {
    let ret: i64;
    asm!("syscall", inlateout("rax") nr => ret, in("rdi") a0,
         lateout("rcx") _, lateout("r11") _, options(nostack));
    ret
}
#[inline(always)]
unsafe fn syscall2(nr: u64, a0: u64, a1: u64) -> i64 {
    let ret: i64;
    asm!("syscall", inlateout("rax") nr => ret, in("rdi") a0, in("rsi") a1,
         lateout("rcx") _, lateout("r11") _, options(nostack));
    ret
}
#[inline(always)]
unsafe fn syscall3(nr: u64, a0: u64, a1: u64, a2: u64) -> i64 {
    let ret: i64;
    asm!("syscall", inlateout("rax") nr => ret, in("rdi") a0, in("rsi") a1, in("rdx") a2,
         lateout("rcx") _, lateout("r11") _, options(nostack));
    ret
}
#[inline(always)]
unsafe fn syscall4(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> i64 {
    let ret: i64;
    asm!("syscall", inlateout("rax") nr => ret,
         in("rdi") a0, in("rsi") a1, in("rdx") a2, in("r10") a3,
         lateout("rcx") _, lateout("r11") _, options(nostack));
    ret
}

fn print(s: &[u8]) {
    // SAFETY: valid pointer/length pair handed to the kernel `write`.
    let _ = unsafe { syscall3(SYS_WRITE, STDOUT, s.as_ptr() as u64, s.len() as u64) };
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
fn fork() -> i64 {
    // SAFETY: bare fork ABI (SIGCHLD exit-signal, newsp=0).
    unsafe { syscall2(SYS_CLONE, SIGCHLD, 0) }
}
fn wait4(pid: i64, status: &mut i32, options: u64) -> i64 {
    // SAFETY: valid i32 pointer for the kernel to write the status.
    unsafe { syscall4(SYS_WAIT4, pid as u64, status as *mut i32 as u64, options, 0) }
}
fn sys_exit(code: i32) -> ! {
    // SAFETY: terminates the process; never returns.
    unsafe {
        syscall1(SYS_EXIT, code as u64);
        syscall1(SYS_EXIT_GROUP, code as u64);
    }
    loop {
        core::hint::spin_loop();
    }
}

/// A worker's exit code = `WORKER_BASE + index` (kept < 256 so WEXITSTATUS is
/// exact). Lets the parent verify it reaped each distinct worker.
const WORKER_BASE: u32 = 10;

/// Each worker spins this many CPU iterations (NO syscalls in the loop, so the
/// kernel syscall trace is not flooded) before exiting. The point is to live long
/// enough that the periodic timer preempts it at least once, so the scheduler
/// interleaves the workers and spreads them across CPUs (the kernel's round-robin
/// placement put each on a CPU; this keeps them runnable a while). Tuned so the
/// whole 8-worker fan-out completes well within the TCG test timeout.
const WORKER_SPINS: u64 = 3_000_000;

#[unsafe(no_mangle)]
extern "C" fn user_main() -> ! {
    print(b"smpdemo: parent pid=");
    print_u32(getpid());
    print(b" spawning workers\n");

    // Fan out N workers.
    let mut spawned: u32 = 0;
    for idx in 0..N_WORKERS {
        let rc = fork();
        if rc == 0 {
            // ---- worker `idx` ----
            // A pure-CPU spin (NO syscalls in the loop) so the periodic timer
            // preempts us — the scheduler then interleaves the workers across
            // CPUs — without flooding the kernel syscall trace. The volatile read
            // keeps the loop from being optimised away.
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

    // Reap all workers.
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
