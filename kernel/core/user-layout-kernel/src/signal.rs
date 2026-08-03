// Pure, arch-neutral POSIX-signal math for the EL0/ring-3 user Frame.
//
// This file is the body of the `user_layout::signal` module (it is `include!`d
// from `lib.rs`, alongside `layout.rs`, which supplies the crate-level
// `#![no_std]`). Like `layout.rs` it carries **no inner attributes** (`#![...]`)
// or `//!` module docs, because it is `include!`d both into this crate and into
// the out-of-workspace host harness's module body, where inner attributes are
// not permitted.
//
// It is the **single source of truth** for everything about signal delivery
// that is a *pure function* of its inputs and therefore identical on every
// arch: the signal numbers, the sigset bit math, the `SigAction`/`SignalState`
// PODs, the default-action classifier, and — most importantly — the
// signal-frame **offset constants** and the **stack-alignment arithmetic** the
// two arch backends share between the deliver path and the `rt_sigreturn` path
// (so the two never drift). It depends on **nothing** outside `core` and
// contains **zero `unsafe`**, so the `check-tcb.sh` ratchet stays green and the
// math is exhaustively host-tested (see `mod signal_tests` at the bottom, run
// through `crates/arch-aarch64/tests-host/`).
//
// Keeping this logic pure also keeps the `unsafe` arch Frames thin: they only
// do the things that *must* be unsafe (writing the frame into user memory,
// rewriting the live trap frame, the sigreturn register restore), delegating
// all the fiddly arithmetic and layout here where it can be tested.

// ---- Signal numbers (identical numbering on aarch64 & x86_64) -------------

/// Number of signals (1..=64). State arrays use a 65-element array so a signal
/// number indexes directly; slot 0 is unused.
pub const NSIG: usize = 64;
/// Userspace `sigset_t` is 8 bytes / 64 bits in the Linux rt_* ABI.
pub const SIGSET_BYTES: usize = 8;

pub const SIGHUP: u32 = 1;
pub const SIGINT: u32 = 2;
pub const SIGQUIT: u32 = 3;
pub const SIGILL: u32 = 4;
pub const SIGTRAP: u32 = 5;
pub const SIGABRT: u32 = 6;
pub const SIGBUS: u32 = 7;
pub const SIGFPE: u32 = 8;
pub const SIGKILL: u32 = 9;
pub const SIGUSR1: u32 = 10;
pub const SIGSEGV: u32 = 11;
pub const SIGUSR2: u32 = 12;
pub const SIGPIPE: u32 = 13;
pub const SIGALRM: u32 = 14;
pub const SIGTERM: u32 = 15;
pub const SIGCHLD: u32 = 17;
pub const SIGCONT: u32 = 18;
pub const SIGSTOP: u32 = 19;
pub const SIGURG: u32 = 23;
pub const SIGWINCH: u32 = 28;

// ---- sigaction flags / sentinel handler values ----------------------------

pub const SA_SIGINFO: u64 = 0x0000_0004;
pub const SA_RESTORER: u64 = 0x0400_0000;
/// Accepted for ABI compatibility; v1 never interrupts a blocking call so there
/// is nothing to restart (documented in the spec §8.3).
pub const SA_RESTART: u64 = 0x1000_0000;
pub const SA_ONSTACK: u64 = 0x0800_0000;

pub const SIG_DFL: u64 = 0;
pub const SIG_IGN: u64 = 1;

/// `sigaltstack` `ss_flags`: disable the alternate stack.
pub const SS_DISABLE: u32 = 2;
/// Minimum acceptable alternate-stack size.
pub const MINSIGSTKSZ: u64 = 2048;

/// `rt_sigprocmask` `how` values.
pub const SIG_BLOCK: u64 = 0;
pub const SIG_UNBLOCK: u64 = 1;
pub const SIG_SETMASK: u64 = 2;

/// Bit mask of signals that can never be blocked or have their disposition
/// changed: SIGKILL(9) and SIGSTOP(19). Stored as a [`Sigset`]-shaped `u64`.
pub const fn unblockable_bits() -> u64 {
    sig_bit(SIGKILL) | sig_bit(SIGSTOP)
}

/// The `Sigset` bit for signal `sig` (1..=64). Returns 0 for out-of-range
/// signals so callers never shift out of bounds.
pub const fn sig_bit(sig: u32) -> u64 {
    if sig >= 1 && sig as usize <= NSIG {
        1u64 << (sig - 1)
    } else {
        0
    }
}

// ---- Sigset newtype --------------------------------------------------------

/// A 64-bit signal set (`sigset_t`), bit `n-1` = signal `n`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sigset(pub u64);

impl Sigset {
    pub const fn empty() -> Sigset {
        Sigset(0)
    }

    /// Add signal `sig` (no-op for out-of-range).
    pub fn add(&mut self, sig: u32) {
        self.0 |= sig_bit(sig);
    }

    /// Remove signal `sig`.
    pub fn remove(&mut self, sig: u32) {
        self.0 &= !sig_bit(sig);
    }

    /// True iff `sig` is a member.
    pub fn contains(&self, sig: u32) -> bool {
        self.0 & sig_bit(sig) != 0
    }

    /// Force-clear SIGKILL/SIGSTOP — those can never be blocked. Returns the
    /// sanitized set (used everywhere a blocked mask is computed).
    pub fn block_unblockable_cleared(self) -> Sigset {
        Sigset(self.0 & !unblockable_bits())
    }

    /// Decode 8 little-endian bytes (the userspace `sigset_t`).
    pub fn from_bytes(b: [u8; SIGSET_BYTES]) -> Sigset {
        Sigset(u64::from_le_bytes(b))
    }

    /// Encode to 8 little-endian bytes.
    pub fn to_bytes(self) -> [u8; SIGSET_BYTES] {
        self.0.to_le_bytes()
    }
}

// ---- SigAction POD ---------------------------------------------------------

/// A signal disposition, mirroring the userspace `struct sigaction` fields the
/// kernel cares about. `Clone+Copy` POD so the per-process action table is a
/// trivially-copyable array.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SigAction {
    /// `sa_handler` / `sa_sigaction`: a user PC, or `SIG_DFL`(0) / `SIG_IGN`(1).
    pub handler: u64,
    /// `sa_mask`: extra signals to block for the duration of the handler.
    pub mask: u64,
    /// `sa_flags`: SA_SIGINFO / SA_RESTORER / SA_ONSTACK / SA_RESTART.
    pub flags: u64,
    /// `sa_restorer`: the trampoline the handler returns through (must be set
    /// when delivering, since we require SA_RESTORER).
    pub restorer: u64,
}

impl SigAction {
    /// The default (`SIG_DFL`, no mask, no flags) disposition.
    pub const fn default_action() -> SigAction {
        SigAction {
            handler: SIG_DFL,
            mask: 0,
            flags: 0,
            restorer: 0,
        }
    }

    /// True iff this disposition has a real user handler (not SIG_DFL/SIG_IGN).
    pub fn has_handler(&self) -> bool {
        self.handler > SIG_IGN
    }

    /// True iff this disposition is SIG_IGN.
    pub fn is_ignore(&self) -> bool {
        self.handler == SIG_IGN
    }
}

// ---- Default-action classifier --------------------------------------------

/// What the kernel does for a signal that has no user handler installed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DefaultAction {
    /// Terminate the process (WIFSIGNALED). The common case.
    Terminate,
    /// Discard silently (SIGCHLD / SIGURG / SIGWINCH).
    Ignore,
    /// Stop the process (job control). v1 treats as Ignore at the delivery site.
    Stop,
    /// Continue a stopped process. v1 treats as Ignore at the delivery site.
    Continue,
}

/// The POSIX default action for `sig`.
pub fn default_action(sig: u32) -> DefaultAction {
    match sig {
        x if x == SIGCHLD || x == SIGURG || x == SIGWINCH => DefaultAction::Ignore,
        x if x == SIGSTOP => DefaultAction::Stop,
        x if x == SIGCONT => DefaultAction::Continue,
        _ => DefaultAction::Terminate,
    }
}

// ---- Per-process signal state ---------------------------------------------

/// All signal state a process carries. `Clone+Copy` so `fork` inheritance is a
/// plain field copy and `execve` reset is `SignalState::new()`.
#[derive(Clone, Copy)]
pub struct SignalState {
    /// Disposition per signal; index by signal number directly (slot 0 unused).
    pub actions: [SigAction; NSIG + 1],
    /// Currently-blocked signals (`Sigset` bits).
    pub blocked: u64,
    /// Raised-but-undelivered signals (`Sigset` bits).
    pub pending: u64,
    /// `sigaltstack` base (0 = none).
    pub altstack_sp: u64,
    /// `sigaltstack` size.
    pub altstack_size: u64,
    /// Re-entrancy guard: true while running on the alternate stack.
    pub on_altstack: bool,
}

impl SignalState {
    /// All-`SIG_DFL`, nothing blocked/pending, no alt stack.
    pub const fn new() -> SignalState {
        SignalState {
            actions: [SigAction::default_action(); NSIG + 1],
            blocked: 0,
            pending: 0,
            altstack_sp: 0,
            altstack_size: 0,
            on_altstack: false,
        }
    }

    /// Raise `sig` on this process (set its pending bit). Bit-only; no other
    /// state change. SIGKILL/SIGSTOP and ordinary signals all just set a bit.
    pub fn raise(&mut self, sig: u32) {
        self.pending |= sig_bit(sig);
    }

    /// Clear `sig` from pending (after it is delivered / discarded).
    pub fn clear_pending(&mut self, sig: u32) {
        self.pending &= !sig_bit(sig);
    }

    /// The lowest-numbered deliverable signal: pending and not blocked. SIGKILL
    /// is never blockable so it always shows through. Returns the signal number
    /// (1..=64) or `None`.
    pub fn next_deliverable(&self) -> Option<u32> {
        let deliverable = self.pending & !self.blocked.block_unblockable_cleared_u64();
        if deliverable == 0 {
            return None;
        }
        // Lowest set bit -> lowest signal number.
        let idx = deliverable.trailing_zeros();
        Some(idx + 1)
    }

    /// The disposition for `sig` (clamped to the valid range; out-of-range maps
    /// to the default disposition).
    pub fn action(&self, sig: u32) -> SigAction {
        if sig >= 1 && sig as usize <= NSIG {
            self.actions[sig as usize]
        } else {
            SigAction::default_action()
        }
    }
}

// A tiny helper so `next_deliverable` can sanitize the blocked mask without a
// `Sigset` allocation; mirrors `Sigset::block_unblockable_cleared`.
trait BlockSanitize {
    fn block_unblockable_cleared_u64(self) -> u64;
}
impl BlockSanitize for u64 {
    fn block_unblockable_cleared_u64(self) -> u64 {
        self & !unblockable_bits()
    }
}

// ---------------------------------------------------------------------------
// Signal-frame layout — shared offset constants + alignment arithmetic
// ---------------------------------------------------------------------------
//
// The two arch backends build a signal frame on the user stack when delivering
// a handler, and read it back in rt_sigreturn. The layout MUST be identical
// between the two paths (spec §8.2 risk: "frame layout drift"), so all of it
// lives here as pure constants + functions, host-tested below.
//
// We lay each arch's frame at the Linux `struct rt_sigframe` / `ucontext`
// offsets so a real musl/glibc handler that walks `ucontext` (e.g. to read the
// saved PC) sees sane values, and so deliver+sigreturn agree on where each
// saved register lives.

// ---- aarch64 frame ---------------------------------------------------------
//
// Linux aarch64 `struct rt_sigframe` { siginfo_t info; struct ucontext uc; }.
// `struct ucontext` { unsigned long uc_flags; ucontext *uc_link; stack_t
// uc_stack; sigset_t uc_sigmask; ... ; struct sigcontext uc_mcontext; }. On
// aarch64 the `uc_mcontext` (`struct sigcontext`) sits at offset 176 inside
// `ucontext`. `struct sigcontext` = { __u64 fault_address; __u64 regs[31];
// __u64 sp; __u64 pc; __u64 pstate; ... __reserved[] }.
//
// We place the frame as: [ siginfo @ AA_SIGINFO_OFF ][ ucontext @ AA_UC_OFF ].
// The handler gets a1 = frame_base + AA_SIGINFO_OFF, a2 = frame_base + AA_UC_OFF.

/// siginfo_t sits at the very base of the rt_sigframe.
pub const AA_SIGINFO_OFF: u64 = 0;
/// We reserve 128 bytes for siginfo_t (Linux siginfo is 128 bytes).
pub const AA_SIGINFO_SIZE: u64 = 128;
/// ucontext follows siginfo.
pub const AA_UC_OFF: u64 = AA_SIGINFO_OFF + AA_SIGINFO_SIZE;

// Offsets *within* `struct ucontext`.
/// `uc_flags` (u64).
pub const AA_UC_FLAGS_OFF: u64 = 0;
/// `uc_link` (pointer).
pub const AA_UC_LINK_OFF: u64 = 8;
/// `uc_stack.ss_sp` / `ss_flags` / `ss_size` (stack_t, 24 bytes).
pub const AA_UC_STACK_OFF: u64 = 16;
/// `uc_sigmask` (sigset_t) — the blocked mask to restore on sigreturn.
pub const AA_UC_SIGMASK_OFF: u64 = 40;
/// `uc_mcontext` (`struct sigcontext`) at the Linux offset 176 within ucontext.
pub const AA_UC_MCONTEXT_OFF: u64 = 176;

// Offsets *within* `struct sigcontext` (relative to AA_UC_MCONTEXT_OFF).
pub const AA_SC_FAULT_OFF: u64 = 0;
/// regs[0..31] (x0..x30) start here.
pub const AA_SC_REGS_OFF: u64 = 8;
/// `sp` (SP_EL0).
pub const AA_SC_SP_OFF: u64 = AA_SC_REGS_OFF + 31 * 8; // 256
/// `pc` (ELR).
pub const AA_SC_PC_OFF: u64 = AA_SC_SP_OFF + 8; // 264
/// `pstate` (SPSR).
pub const AA_SC_PSTATE_OFF: u64 = AA_SC_PC_OFF + 8; // 272

/// Total bytes the aarch64 frame occupies (siginfo + ucontext through the end
/// of sigcontext, rounded up to a 16-byte boundary). We reserve a generous
/// `__reserved` tail (musl reads an `fpsimd_context` magic here; we leave it
/// zeroed, which musl tolerates by stopping at the zero terminator record).
pub const AA_RESERVED_TAIL: u64 = 512;
pub const AA_FRAME_SIZE: u64 =
    aa_align_up(AA_UC_OFF + AA_UC_MCONTEXT_OFF + AA_SC_PSTATE_OFF + 8 + AA_RESERVED_TAIL, 16);

/// Round `x` up to a multiple of `align` (a power of two). `const` so it can
/// size the frame above.
pub const fn aa_align_up(x: u64, align: u64) -> u64 {
    (x + (align - 1)) & !(align - 1)
}

/// Compute the aarch64 signal-frame base given the chosen stack top `sp`.
/// AArch64 requires the SP to be 16-byte aligned at handler entry, so we
/// reserve `AA_FRAME_SIZE` below `sp` and clear the low 4 bits.
pub fn aa_frame_base(sp: u64) -> u64 {
    (sp - AA_FRAME_SIZE) & !0xf
}

// ---- x86_64 frame ----------------------------------------------------------
//
// Linux x86_64 `struct rt_sigframe` { char *pretcode; struct ucontext uc;
// siginfo_t info; ... }. `pretcode` (the restorer return address the handler
// `ret`s to) is at the lowest address, so when the handler is entered with
// `rsp = frame_base`, executing `ret` pops `pretcode`. The System V ABI
// requires that at a function's entry (after the implicit return-address push)
// `(rsp+8) % 16 == 0`; since the CPU lands in the handler with rsp = frame_base
// and `pretcode` occupying [frame_base, frame_base+8), we need
// `(frame_base + 8) % 16 == 0`, i.e. `frame_base % 16 == 8`.
//
// `struct ucontext` { unsigned long uc_flags; ucontext *uc_link; stack_t
// uc_stack; struct sigcontext uc_mcontext; sigset_t uc_sigmask; }. On x86_64
// `uc_mcontext` is at offset 40 within ucontext (after uc_flags(8) + uc_link(8)
// + uc_stack(24)). The `sigcontext` register order is: r8,r9,r10,r11,r12,r13,
// r14,r15,rdi,rsi,rbp,rbx,rdx,rax,rcx,rsp,rip,eflags,cs/gs/fs/ss(packed),err,
// trapno,oldmask,cr2,fpstate,reserved...

/// `pretcode` (restorer return address) at the base of the frame.
pub const X64_PRETCODE_OFF: u64 = 0;
/// ucontext follows pretcode.
pub const X64_UC_OFF: u64 = 8;

// Offsets within `struct ucontext`.
pub const X64_UC_FLAGS_OFF: u64 = 0;
pub const X64_UC_LINK_OFF: u64 = 8;
pub const X64_UC_STACK_OFF: u64 = 16;
/// `uc_mcontext` (`struct sigcontext`) at offset 40 within ucontext.
pub const X64_UC_MCONTEXT_OFF: u64 = 40;

// Offsets within `struct sigcontext` (relative to X64_UC_MCONTEXT_OFF), in the
// Linux x86_64 register order.
pub const X64_SC_R8_OFF: u64 = 0;
pub const X64_SC_R9_OFF: u64 = 8;
pub const X64_SC_R10_OFF: u64 = 16;
pub const X64_SC_R11_OFF: u64 = 24;
pub const X64_SC_R12_OFF: u64 = 32;
pub const X64_SC_R13_OFF: u64 = 40;
pub const X64_SC_R14_OFF: u64 = 48;
pub const X64_SC_R15_OFF: u64 = 56;
pub const X64_SC_RDI_OFF: u64 = 64;
pub const X64_SC_RSI_OFF: u64 = 72;
pub const X64_SC_RBP_OFF: u64 = 80;
pub const X64_SC_RBX_OFF: u64 = 88;
pub const X64_SC_RDX_OFF: u64 = 96;
pub const X64_SC_RAX_OFF: u64 = 104;
pub const X64_SC_RCX_OFF: u64 = 112;
pub const X64_SC_RSP_OFF: u64 = 120;
pub const X64_SC_RIP_OFF: u64 = 128;
pub const X64_SC_EFLAGS_OFF: u64 = 136;
/// cs/gs/fs/ss packed (we leave zero), err, trapno, oldmask, cr2.
pub const X64_SC_CSGSFS_OFF: u64 = 144;
pub const X64_SC_ERR_OFF: u64 = 152;
pub const X64_SC_TRAPNO_OFF: u64 = 160;
pub const X64_SC_OLDMASK_OFF: u64 = 168;
pub const X64_SC_CR2_OFF: u64 = 176;
pub const X64_SC_FPSTATE_OFF: u64 = 184;
/// End of the register portion of sigcontext.
pub const X64_SC_END_OFF: u64 = 192;

/// `uc_sigmask` (sigset_t) follows `uc_mcontext` — the blocked mask to restore.
pub const X64_UC_SIGMASK_OFF: u64 = X64_UC_MCONTEXT_OFF + X64_SC_END_OFF;

/// siginfo_t follows the ucontext.
pub const X64_SIGINFO_OFF: u64 = X64_UC_OFF + X64_UC_SIGMASK_OFF + 8 + 8; // +sigmask+pad
/// Linux siginfo is 128 bytes.
pub const X64_SIGINFO_SIZE: u64 = 128;

/// Total frame size, rounded up to 16 then biased so the alignment constraint
/// can be met by `x64_frame_base`.
pub const X64_FRAME_SIZE: u64 = x64_align_up(X64_SIGINFO_OFF + X64_SIGINFO_SIZE, 16);

pub const fn x64_align_up(x: u64, align: u64) -> u64 {
    (x + (align - 1)) & !(align - 1)
}

/// Compute the x86_64 signal-frame base given the chosen stack top `sp`. We
/// reserve `X64_FRAME_SIZE` below `sp`, 16-align, then subtract 8 so that the
/// handler entry sees `(rsp+8) % 16 == 0` after the implicit (simulated) push of
/// `pretcode` — i.e. `frame_base % 16 == 8`.
pub fn x64_frame_base(sp: u64) -> u64 {
    let aligned = (sp - X64_FRAME_SIZE) & !0xf; // 16-aligned
    aligned - 8 // bias so frame_base % 16 == 8
}

// ---------------------------------------------------------------------------
// Host unit tests (run via crates/arch-aarch64/tests-host/)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod signal_tests {
    use super::*;

    #[test]
    fn sigset_add_remove_contains() {
        let mut s = Sigset::empty();
        assert!(!s.contains(SIGUSR1));
        s.add(SIGUSR1);
        assert!(s.contains(SIGUSR1));
        assert_eq!(s.0, 1u64 << (SIGUSR1 - 1));
        s.add(SIGCHLD);
        assert!(s.contains(SIGCHLD) && s.contains(SIGUSR1));
        s.remove(SIGUSR1);
        assert!(!s.contains(SIGUSR1));
        assert!(s.contains(SIGCHLD));
        // Out-of-range signals are no-ops, never panic / shift OOB.
        s.add(0);
        s.add(65);
        s.add(u32::MAX);
        assert!(!s.contains(0) && !s.contains(65));
    }

    #[test]
    fn sigset_roundtrips_bytes() {
        let s = Sigset(0x0123_4567_89ab_cdef);
        assert_eq!(Sigset::from_bytes(s.to_bytes()), s);
        // Little-endian: low byte first.
        assert_eq!(s.to_bytes()[0], 0xef);
        assert_eq!(s.to_bytes()[7], 0x01);
    }

    #[test]
    fn unblockable_signals_are_always_cleared() {
        // A mask trying to block everything cannot keep KILL/STOP.
        let all = Sigset(u64::MAX).block_unblockable_cleared();
        assert!(!all.contains(SIGKILL));
        assert!(!all.contains(SIGSTOP));
        // But it keeps the others.
        assert!(all.contains(SIGUSR1));
        assert!(all.contains(SIGTERM));
        // The dedicated bit constant matches.
        assert_eq!(unblockable_bits(), sig_bit(SIGKILL) | sig_bit(SIGSTOP));
    }

    #[test]
    fn next_deliverable_picks_lowest_unblocked() {
        let mut st = SignalState::new();
        // Nothing pending -> nothing to deliver.
        assert_eq!(st.next_deliverable(), None);
        // Raise USR1 (10) and TERM (15); both unblocked -> lowest = 10.
        st.raise(SIGUSR1);
        st.raise(SIGTERM);
        assert_eq!(st.next_deliverable(), Some(SIGUSR1));
        // Block USR1 -> TERM is next.
        st.blocked |= sig_bit(SIGUSR1);
        assert_eq!(st.next_deliverable(), Some(SIGTERM));
        // Block both -> none.
        st.blocked |= sig_bit(SIGTERM);
        assert_eq!(st.next_deliverable(), None);
        // SIGKILL pending is deliverable even if "blocked" (cannot be blocked).
        st.raise(SIGKILL);
        st.blocked |= sig_bit(SIGKILL);
        assert_eq!(st.next_deliverable(), Some(SIGKILL));
    }

    #[test]
    fn default_action_classification() {
        assert_eq!(default_action(SIGCHLD), DefaultAction::Ignore);
        assert_eq!(default_action(SIGURG), DefaultAction::Ignore);
        assert_eq!(default_action(SIGWINCH), DefaultAction::Ignore);
        assert_eq!(default_action(SIGSTOP), DefaultAction::Stop);
        assert_eq!(default_action(SIGCONT), DefaultAction::Continue);
        assert_eq!(default_action(SIGKILL), DefaultAction::Terminate);
        assert_eq!(default_action(SIGTERM), DefaultAction::Terminate);
        assert_eq!(default_action(SIGSEGV), DefaultAction::Terminate);
        assert_eq!(default_action(SIGUSR1), DefaultAction::Terminate);
    }

    #[test]
    fn sigaction_default_and_handler_classification() {
        let dfl = SigAction::default_action();
        assert_eq!(dfl.handler, SIG_DFL);
        assert!(!dfl.has_handler());
        assert!(!dfl.is_ignore());
        let ign = SigAction {
            handler: SIG_IGN,
            ..SigAction::default_action()
        };
        assert!(ign.is_ignore());
        assert!(!ign.has_handler());
        let real = SigAction {
            handler: 0x40_1000,
            ..SigAction::default_action()
        };
        assert!(real.has_handler());
        assert!(!real.is_ignore());
    }

    #[test]
    fn signalstate_new_is_all_default() {
        let st = SignalState::new();
        assert_eq!(st.blocked, 0);
        assert_eq!(st.pending, 0);
        assert_eq!(st.altstack_sp, 0);
        assert!(!st.on_altstack);
        for sig in 1..=NSIG as u32 {
            assert_eq!(st.action(sig).handler, SIG_DFL);
        }
    }

    // ---- aarch64 frame layout / alignment ---------------------------------

    #[test]
    fn aa_align_up_rounds_to_power_of_two() {
        assert_eq!(aa_align_up(0, 16), 0);
        assert_eq!(aa_align_up(1, 16), 16);
        assert_eq!(aa_align_up(16, 16), 16);
        assert_eq!(aa_align_up(17, 16), 32);
        assert_eq!(aa_align_up(255, 16), 256);
    }

    #[test]
    fn aa_sigcontext_offsets_are_consistent() {
        // sp/pc/pstate are laid contiguously after the 31 GPRs.
        assert_eq!(AA_SC_REGS_OFF, 8);
        assert_eq!(AA_SC_SP_OFF, 8 + 31 * 8);
        assert_eq!(AA_SC_PC_OFF, AA_SC_SP_OFF + 8);
        assert_eq!(AA_SC_PSTATE_OFF, AA_SC_PC_OFF + 8);
        // ucontext starts after a full 128-byte siginfo.
        assert_eq!(AA_UC_OFF, 128);
        // sigcontext at the Linux ucontext offset 176.
        assert_eq!(AA_UC_MCONTEXT_OFF, 176);
    }

    #[test]
    fn aa_frame_base_is_16_aligned_and_below_sp() {
        for sp in [0x60_0000u64, 0x5f_fff0, 0x5f_ff08, 0x5f_abcd] {
            let base = aa_frame_base(sp);
            assert_eq!(base & 0xf, 0, "aarch64 handler SP must be 16-aligned");
            assert!(base < sp);
            assert!(sp - base >= AA_FRAME_SIZE);
        }
    }

    // ---- x86_64 frame layout / alignment ----------------------------------

    #[test]
    fn x64_sigcontext_offsets_match_linux_order() {
        assert_eq!(X64_SC_R8_OFF, 0);
        assert_eq!(X64_SC_RIP_OFF, 128);
        assert_eq!(X64_SC_RSP_OFF, 120);
        assert_eq!(X64_SC_EFLAGS_OFF, 136);
        assert_eq!(X64_UC_MCONTEXT_OFF, 40);
        // uc_sigmask sits right after the register block.
        assert_eq!(X64_UC_SIGMASK_OFF, 40 + 192);
    }

    #[test]
    fn x64_frame_base_meets_sysv_alignment() {
        // After the handler is entered with rsp = frame_base and `pretcode`
        // occupying [base, base+8), the SysV rule (rsp+8)%16==0 requires
        // base % 16 == 8.
        for sp in [0x60_0000u64, 0x5f_fff0, 0x5f_ff08, 0x5f_dead0] {
            let base = x64_frame_base(sp);
            assert_eq!(base % 16, 8, "(rsp+8) must be 16-aligned at handler entry");
            assert!(base < sp);
        }
    }
}
