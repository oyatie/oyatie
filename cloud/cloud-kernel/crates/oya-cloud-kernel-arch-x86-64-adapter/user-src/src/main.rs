//! `user-hello-x86_64` — a tiny freestanding ring-3 user program for
//! kernel, the x86_64 analogue of the aarch64 `user-hello`.
//!
//! It runs in **ring 3** and talks to the kernel only through **raw
//! Linux/x86_64 syscalls** (`syscall`; number in `rax`, args in
//! `rdi,rsi,rdx,r10,r8,r9`, return in `rax`):
//!
//! ```text
//!   write(1, "hello from a loaded x86_64 ELF (ring 3) via syscall\n");  // rax=1
//!   exit(0);                                                             // rax=60
//! ```
//!
//! It is `no_std` / `no_main`, static, non-PIE, and links at the user base
//! `0x40_0000` (matching `user_layout::USER_BASE`) via `user.ld`. The kernel's
//! ELF64 loader maps its `PT_LOAD` segments at their recorded vaddrs verbatim
//! and `iretq`s to `_start`.
//!
//! Build it standalone (NOT a workspace member):
//!   cargo build --release   # target pinned in .cargo/config.toml
#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// The message printed through the kernel's `write` syscall.
static MSG: &[u8] = b"hello from a loaded x86_64 ELF (ring 3) via syscall\n";

/// Linux/x86_64 `write(fd, buf, len)` via the raw `syscall` instruction.
///
/// # Safety
/// `buf` must point at `len` valid readable bytes. The kernel validates the
/// range before reading it.
#[inline(always)]
unsafe fn sys_write(fd: u64, buf: *const u8, len: u64) -> i64 {
    let ret: i64;
    // SAFETY: `syscall` clobbers rcx/r11 (return RIP/RFLAGS) and rax (return);
    // we declare all three. The Linux x86_64 ABI passes nr in rax, args in
    // rdi/rsi/rdx, and returns in rax.
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") 1u64 => ret,
            in("rdi") fd,
            in("rsi") buf,
            in("rdx") len,
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }
    ret
}

/// Linux/x86_64 `exit(status)` via the raw `syscall` instruction. Never returns.
#[inline(always)]
fn sys_exit(status: i32) -> ! {
    // SAFETY: `exit` (nr 60) does not return; the kernel never resumes us.
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 60u64,
            in("rdi") status as u64,
            options(nostack, noreturn),
        );
    }
}

/// Ring-3 entry. The kernel `iretq`s here with `rsp` pointing at the SysV
/// initial stack (argc/argv/envp/auxv); we do not read it for this leaf demo.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // SAFETY: `MSG` is a valid static slice of `MSG.len()` readable bytes.
    let _ = unsafe { sys_write(1, MSG.as_ptr(), MSG.len() as u64) };
    sys_exit(0)
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    sys_exit(127)
}
