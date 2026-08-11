//! Classification of kernel-network failures for PID 1.
//!
//! These live in the library rather than in `src/main.rs` because that is the
//! only place they are actually *tested*: the `os-init-app-unittest` target
//! roots at `src/lib.rs`, so nothing under `main.rs`'s `#[cfg(test)] mod tests`
//! is ever compiled or run by the build graph. `net_errno` and its two tests
//! sat there unexercised; moving them here puts them back under the gate.
//!
//! All three functions are pure and host-testable: they inspect a value and
//! perform no syscall.

use os_kernel::error::Error;

/// Classify a kernel-network failure into the benign-skip reason, if any.
///
/// This is the port-aware replacement for calling [`net_errno`] on the error's
/// Display text. The `os_kernel_abi::KernelNet` contract classifies failures by
/// [`Error`] variant, so a substrate that reports "permission denied" without
/// spelling a Linux errno is handled correctly here — under the old text scan
/// it fell through to `-1` and was reported as an unexpected failure, which
/// silently made the sandbox-skip behavior Linux-only.
///
/// The variant is the *fallback*, not the first cut: four variants cannot carry
/// the five causes `privileged_op_skip_reason` deliberately separates, so a
/// variant-first match records `EPERM` for an `EACCES` and `EOPNOTSUPP` for an
/// `ENOTTY`, and `skip_log` prints an errno the kernel never returned.
pub fn net_skip_reason(e: &Error) -> Option<&'static str> {
    // Prefer the exact errno when the adapter retained it. `errno_error` keeps
    // `errno N` in the message precisely so this stays available, and boot.rs
    // relies on the distinction to tell a Sentry ioctl gap from a denial.
    if let Some(reason) =
        os_machined_domain::boot::privileged_op_skip_reason(net_errno(&e.to_string()))
    {
        return Some(reason);
    }
    // No usable errno — a substrate that reports in the port's vocabulary only.
    // Precision degrades to the variant; the skip decision does not.
    match e {
        Error::PermissionDenied(_) => Some("EPERM"),
        Error::Unsupported(_) => Some("EOPNOTSUPP"),
        _ => None,
    }
}

/// Whether a kernel-network failure means "already installed".
///
/// Re-running address/route installation is PID 1's normal path, so this is
/// benign. The `os_kernel_abi::KernelNet` contract makes it
/// [`Error::InvalidState`] on every substrate; the `EEXIST` text scan remains
/// as the fallback for unmigrated adapters.
pub fn net_already_exists(e: &Error) -> bool {
    matches!(e, Error::InvalidState(_)) || net_errno(&e.to_string()) == 17 // EEXIST
}

/// Extract the errno from an `os_network_domain` error's Display string.
///
/// The `linux_net` module is dep-light and surfaces failures as
/// `"<ctx>: errno <N>"` (socket-level) or `"netlink request failed: errno <N>"`
/// (rtnetlink ACK, where the kernel's negative errno has already been negated
/// back to positive). Both end in `errno <N>`, so we pull the trailing integer
/// (taking its absolute value, to be robust to either sign convention).
/// Returns `-1` when no errno is present, which forces the "unexpected"
/// classification (i.e. fail loudly rather than silently tolerate an
/// unparseable error).
pub fn net_errno(msg: &str) -> i32 {
    match msg.rsplit_once("errno ") {
        Some((_, tail)) => {
            let digits: String = tail
                .trim()
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '-')
                .collect();
            // i32::MIN has no positive abs in two's complement — treat as
            // unparseable (-1) rather than overflow/wrap.
            digits
                .parse::<i32>()
                .ok()
                .and_then(i32::checked_abs)
                .unwrap_or(-1)
        }
        None => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn net_errno_parses_linux_net_error_strings() {
        // rtnetlink ACK failure (kernel errno negated back to positive).
        assert_eq!(net_errno("netlink request failed: errno 1"), 1);
        // socket-level failure context prefix.
        assert_eq!(net_errno("socket(AF_NETLINK): errno 13"), 13);
        assert_eq!(net_errno("open(/sys/class/net/eth0): errno 2"), 2);
        // Absolute value taken, regardless of sign convention.
        assert_eq!(net_errno("netlink request failed: errno -1"), 1);
        // Trailing punctuation after the number is tolerated.
        assert_eq!(net_errno("foo: errno 95 (oops)"), 95);
    }

    #[test]
    fn net_errno_unparseable_yields_minus_one() {
        // No "errno" marker => -1 (forces the unexpected/fail classification).
        assert_eq!(net_errno("address exists"), -1);
        assert_eq!(net_errno(""), -1);
        assert_eq!(net_errno("errno notanumber"), -1);
    }

    #[test]
    fn skip_classification_survives_a_substrate_that_writes_no_errno() {
        // The regression this closes: a non-Linux substrate reports denial in
        // the port's vocabulary with no "errno N" in the text. The old text
        // scan yielded -1 and PID 1 reported it as an unexpected failure,
        // silently making sandbox tolerance Linux-only.
        assert_eq!(net_errno("permission denied: cannot configure link"), -1);
        assert_eq!(
            net_skip_reason(&Error::permission_denied("cannot configure link")),
            Some("EPERM")
        );
        assert_eq!(
            net_skip_reason(&Error::unsupported("substrate has no rtnetlink")),
            Some("EOPNOTSUPP")
        );

        // A genuinely unexpected failure must still fail loudly.
        assert_eq!(
            net_skip_reason(&Error::Other("bus fault".to_string())),
            None
        );

        // And the Linux text path is unchanged for unmigrated adapters.
        assert_eq!(
            net_skip_reason(&Error::Other("socket(AF_NETLINK): errno 13".to_string())),
            Some("EACCES")
        );
    }

    #[test]
    fn a_retained_errno_beats_the_coarser_variant() {
        // `errno_error` folds {EPERM, EACCES} into one variant and
        // {ENOSYS, EOPNOTSUPP, ENOTTY} into another, so a variant-first match
        // logged an errno the kernel never returned. The errno is still in the
        // message; read it first.
        assert_eq!(
            net_skip_reason(&Error::permission_denied("socket(AF_NETLINK): errno 13")),
            Some("EACCES")
        );
        assert_eq!(
            net_skip_reason(&Error::unsupported("ioctl(SIOCSIFFLAGS): errno 25")),
            Some("ENOTTY")
        );
        // A non-skippable errno must not become a skip via the probe: EEXIST is
        // handled by `net_already_exists`, not tolerated as a sandbox gap.
        assert_eq!(
            net_skip_reason(&Error::invalid_state("netlink request failed: errno 17")),
            None
        );
    }

    #[test]
    fn already_exists_is_recognised_from_the_variant_and_from_errno_text() {
        assert!(net_already_exists(&Error::invalid_state(
            "address on 'eth0' already exists: 10.0.0.5/24"
        )));
        // Legacy Linux spelling still recognised.
        assert!(net_already_exists(&Error::Other(
            "netlink request failed: errno 17".to_string()
        )));
        assert!(!net_already_exists(&Error::Other(
            "netlink request failed: errno 13".to_string()
        )));
    }

    #[test]
    fn i32_min_errno_does_not_overflow_the_classifier() {
        // Hostile/attacker-shaped errno 0x80000000 as text "errno -2147483648".
        // i32::abs would overflow; fail closed to -1 (unexpected) instead.
        assert_eq!(net_errno("netlink request failed: errno -2147483648"), -1);
    }

}
