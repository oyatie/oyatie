//! `fsbase-worker` — a real Rust **std/musl** worker that proves the x86_64
//! `arch_prctl` FS-base-per-process fix by giving each of two concurrent
//! processes a **distinct** `%fs` base and reading `%fs:`-relative data across
//! scheduler preemptions.
//!
//! ## Why a hand-rolled distinct base (and not just musl's `__thread`)
//!
//! musl's own static TLS lands at the SAME virtual address in both worker
//! copies (they are the identical binary at the identical layout), so musl sets
//! the SAME numeric `%fs` base in both processes. With equal base VALUES the bug
//! is invisible: after a switch, `%fs:off` reads from that one VA in whichever
//! address space (CR3) is live, so each process still reads its own page even if
//! the MSR is never restored. The clobber only bites when the two base VALUES
//! DIFFER.
//!
//! So each worker establishes a genuinely DISTINCT `%fs` base:
//!   1. `mmap` a scratch region (same VA in both processes, but each backed by
//!      its OWN physical frames);
//!   2. pick `base = region + pid * SLOT` — a per-pid DISTINCT numeric address
//!      inside that region;
//!   3. `arch_prctl(ARCH_SET_FS, base)` and store its pid at `%fs:0`;
//!   4. loop in **ring 3** re-reading `%fs:0`. The 10 ms timer preempts the loop
//!      and switches to the other worker. On return the next `%fs:0` read uses
//!      whatever base the switch left in the MSR:
//!        * WITH the fix  -> switch_to restored OUR base  -> `%fs:0` == our pid;
//!        * WITHOUT the fix -> the MSR holds the OTHER worker's base value
//!          (region + other_pid*SLOT), so in OUR address space `%fs:0` reads OUR
//!          region at the OTHER worker's slot -> 0 (never written by us) ->
//!          mismatch -> we detect the clobber and `exit(42)`.
//!
//! WITH the per-process FS-base save/restore fix every read is our pid ->
//! `exit(0)`; WITHOUT it at least one worker reads the wrong value -> `exit(42)`,
//! which PID1 surfaces as `fsbase: TLS CORRUPTION detected`, so the run never
//! reaches `kernel: OK`. That is the falsifiability proof.
//!
//! It is a real Rust-std/musl process: the musl C-runtime startup
//! (`_start`/`__init_tp`/`arch_prctl`/`__libc_start_main`) runs to reach `main`,
//! exercising the musl-hosting path. Once in `main` we take over `%fs` for the
//! experiment and use only **raw syscalls** for I/O and exit, so we never call a
//! musl function that would itself touch `%fs`-based TLS after we repointed it.

use std::arch::asm;
use std::hint::black_box;

/// Number of progress rounds (each prints a heartbeat then spins a long ring-3
/// re-read loop). Enough that the run lasts well over a second of wall time, so
/// the 10 ms timer preemption interleaves the two workers hundreds of times
/// *while they are in ring 3 reading `%fs:0`*.
const ROUNDS: u32 = 8;
/// Inner ring-3 spin iterations per round. No syscall inside, so the worker
/// stays in **ring 3** (a kernel busy-wait nanosleep would pin the CPU in ring 0
/// where the timer does not preempt). ~30M iterations is several hundred ms of
/// ring-3 time per round under TCG — the 10 ms PIT fires (and switches) many
/// times inside it.
const INNER_SPINS: u64 = 30_000_000;
/// Per-pid `%fs` base slot stride. `base = region + pid * SLOT` gives each worker
/// a DISTINCT numeric base; SLOT is page-sized head-room so the two slots never
/// overlap (pids here are tiny — 2 and 3).
const SLOT: u64 = 0x1000;
/// Exit code when a `%fs:0` read returns the WRONG identity (the FS-base clobber).
const CORRUPT_EXIT: i32 = 42;

// --- Linux x86_64 syscall numbers (raw, libc-independent) -------------------
const SYS_WRITE: u64 = 1;
const SYS_MMAP: u64 = 9;
const SYS_GETPID: u64 = 39;
const SYS_EXIT: u64 = 60;
const SYS_ARCH_PRCTL: u64 = 158;
const ARCH_SET_FS: u64 = 0x1002;

const STDOUT: u64 = 1;

// mmap args.
const PROT_READ_WRITE: u64 = 0x3; // PROT_READ | PROT_WRITE
const MAP_PRIVATE_ANON: u64 = 0x22; // MAP_PRIVATE | MAP_ANONYMOUS
const MMAP_LEN: u64 = 0x10000; // 64 KiB scratch (plenty for a few pid slots)

#[inline(always)]
unsafe fn syscall6(nr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64) -> i64 {
    let ret: i64;
    asm!(
        "syscall",
        inlateout("rax") nr => ret,
        in("rdi") a0, in("rsi") a1, in("rdx") a2,
        in("r10") a3, in("r8") a4, in("r9") a5,
        lateout("rcx") _, lateout("r11") _,
        options(nostack),
    );
    ret
}

#[inline(always)]
unsafe fn syscall3(nr: u64, a0: u64, a1: u64, a2: u64) -> i64 {
    syscall6(nr, a0, a1, a2, 0, 0, 0)
}

// --- Raw I/O (no libc TLS) --------------------------------------------------

fn write_all(bytes: &[u8]) {
    // SAFETY: valid pointer/length pair handed to the kernel `write`.
    unsafe {
        syscall3(SYS_WRITE, STDOUT, bytes.as_ptr() as u64, bytes.len() as u64);
    }
}

/// Print a decimal u32 (pids/rounds) via a raw write — no libc, no TLS.
fn write_u32(mut v: u32) {
    let mut buf = [0u8; 10];
    let mut i = buf.len();
    if v == 0 {
        write_all(b"0");
        return;
    }
    while v > 0 {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
    write_all(&buf[i..]);
}

fn raw_getpid() -> u32 {
    // SAFETY: getpid takes no args and only reads kernel state.
    (unsafe { syscall6(SYS_GETPID, 0, 0, 0, 0, 0, 0) }) as u32
}

fn raw_exit(code: i32) -> ! {
    // SAFETY: exit terminates the process and never returns. Raw so it does not
    // run musl atexit handlers that might touch our repointed `%fs`.
    unsafe {
        syscall6(SYS_EXIT, code as u64, 0, 0, 0, 0, 0);
    }
    loop {
        core::hint::spin_loop();
    }
}

/// `arch_prctl(ARCH_SET_FS, base)` — point `%fs` at `base`.
#[inline(always)]
unsafe fn set_fs_base(base: u64) -> i64 {
    syscall6(SYS_ARCH_PRCTL, ARCH_SET_FS, base, 0, 0, 0, 0)
}

/// Store `v` at `%fs:0` (a `%fs:`-relative write that depends on the current
/// `%fs` base — i.e. on the per-process MSR value).
#[inline(always)]
unsafe fn fs_store(v: u64) {
    asm!("mov fs:[0], {v}", v = in(reg) v, options(nostack));
}

/// Read `%fs:0` (a `%fs:`-relative load that depends on the current `%fs` base).
#[inline(always)]
unsafe fn fs_load() -> u64 {
    let v: u64;
    asm!("mov {v}, fs:[0]", v = out(reg) v, options(nostack, readonly));
    v
}

fn main() {
    // `main` is reached via the full musl C-runtime startup (_start -> __init_tp
    // -> arch_prctl -> __libc_start_main), so this is a genuine musl process. We
    // now take over `%fs` and use only raw syscalls below.
    let my_pid = raw_getpid();

    // (1) mmap a per-process scratch region. Same VA in both processes, but each
    // is backed by its OWN physical frames.
    // SAFETY: a standard anonymous private mmap; addr=NULL lets the kernel pick.
    let region = unsafe {
        syscall6(
            SYS_MMAP,
            0,
            MMAP_LEN,
            PROT_READ_WRITE,
            MAP_PRIVATE_ANON,
            (-1i64) as u64, // fd
            0,
        )
    };
    if region < 0 {
        write_all(b"worker ");
        write_u32(my_pid);
        write_all(b": mmap FAILED\n");
        raw_exit(2);
    }
    let region = region as u64;

    // (2) DISTINCT per-pid `%fs` base = region + pid * SLOT. The two workers
    // (pid 2, pid 3) get DIFFERENT base values -> the clobber is now observable.
    let my_base = region + my_pid as u64 * SLOT;

    // (3) Point `%fs` at our distinct base and stamp our pid at `%fs:0`.
    // SAFETY: `my_base` is inside the freshly-mapped RW region; arch_prctl only
    // affects ring-3 `%fs:` accesses for this process.
    unsafe {
        if set_fs_base(my_base) != 0 {
            write_all(b"worker ");
            write_u32(my_pid);
            write_all(b": arch_prctl(ARCH_SET_FS) FAILED\n");
            raw_exit(2);
        }
        fs_store(my_pid as u64);
    }

    write_all(b"worker ");
    write_u32(my_pid);
    write_all(b": fs base set, looping ");
    write_u32(ROUNDS);
    write_all(b" rounds\n");

    // (4) Long ring-3 loop re-reading `%fs:0`. The 10 ms PIT preempts us mid-loop
    // and switches to the other worker; on return the next `%fs:0` read uses
    // whatever base the switch left in the MSR.
    for round in 1..=ROUNDS {
        write_all(b"worker ");
        write_u32(my_pid);
        write_all(b": round ");
        write_u32(round);
        write_all(b"/");
        write_u32(ROUNDS);
        write_all(b"\n");

        let mut acc: u64 = 0;
        let mut i: u64 = 0;
        while i < INNER_SPINS {
            // SAFETY: `%fs:0` is inside our mapped region; pure read.
            let seen = unsafe { fs_load() };
            if seen != my_pid as u64 {
                // The FS base was clobbered: `%fs:0` resolved through the OTHER
                // worker's base value into a slot we never wrote (-> 0/garbage),
                // or into the other worker's data. Either way it is NOT our pid.
                write_all(b"worker ");
                write_u32(my_pid);
                write_all(b": TLS CLOBBERED at round ");
                write_u32(round);
                write_all(b" (read ");
                write_u32(seen as u32);
                write_all(b", expected ");
                write_u32(my_pid);
                write_all(b")\n");
                raw_exit(CORRUPT_EXIT);
            }
            // Black-box so the read cannot be hoisted/cached across preemptions.
            acc = acc.wrapping_add(black_box(seen));
            i = black_box(i + 1);
        }
        black_box(acc);
    }

    // (5) Every read across every preemption matched our own identity.
    write_all(b"worker ");
    write_u32(my_pid);
    write_all(b": fs base intact across ");
    write_u32(ROUNDS);
    write_all(b" rounds, exiting 0\n");
    raw_exit(0);
}
