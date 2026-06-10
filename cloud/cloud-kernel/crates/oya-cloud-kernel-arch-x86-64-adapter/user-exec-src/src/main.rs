//! user-exec-x86_64: the tiny ring-3 program that user-spawn's child
//! `execve()`s into, demonstrating that the kernel can **replace a process
//! image** (tear down old mappings, load a fresh ELF, reset RIP/RSP) while
//! keeping the pid. The x86_64 analogue of ../../../kernel-usermode-tests/exec (aarch64).
//!
//! It prints its pid/ppid and a banner, then `exit(7)` — a distinct exit code so
//! the parent's `wait4` can confirm it reaped the *exec'd* image's status, not
//! the original program's. `no_std` + `no_main`; raw `syscall`.

#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

const SYS_WRITE: u64 = 1;
const SYS_GETPID: u64 = 39;
const SYS_EXIT: u64 = 60;
const SYS_GETPPID: u64 = 110;
const SYS_EXIT_GROUP: u64 = 231;
const STDOUT: u64 = 1;

const EXEC_EXIT_CODE: i32 = 7;

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

fn print(s: &[u8]) {
    // SAFETY: valid pointer/length handed to write.
    unsafe {
        syscall3(SYS_WRITE, STDOUT, s.as_ptr() as u64, s.len() as u64);
    }
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

#[unsafe(no_mangle)]
extern "C" fn user_main() -> ! {
    // SAFETY: getpid/getppid only read kernel state.
    let pid = (unsafe { syscall0(SYS_GETPID) }) as u32;
    let ppid = (unsafe { syscall0(SYS_GETPPID) }) as u32;
    print(b"user-exec: hello from exec'd image pid=");
    print_u32(pid);
    print(b" ppid=");
    print_u32(ppid);
    print(b" exiting with 7\n");
    sys_exit(EXEC_EXIT_CODE);
}

/// The kernel enters here with a valid rsp (SysV initial stack) already set up
/// by execve, so we just tail-call Rust.
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
