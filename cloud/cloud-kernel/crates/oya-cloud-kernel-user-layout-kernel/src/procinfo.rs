// Pure process-info / libc-init layout math (no inner attributes; `include!`d
// by `lib.rs` so the same source unit-tests on the host).
//
// Every real glibc/musl binary queries these during init. The byte-exact
// serialization of the kernel's answer is a pure function of its inputs and
// therefore identical across architectures, so it lives here (the single safe,
// arch-neutral home). The arch Frame backend supplies only the machine string
// (`"aarch64"` / `"x86_64"`) and performs the bounds-checked user copy of the
// returned bytes via its existing PAN/SMAP-bracketed pattern. Zero `unsafe`.

/// Size in bytes of one `struct utsname` field on Linux (`__NEW_UTS_LEN + 1`).
pub const UTS_FIELD_LEN: usize = 65;
/// Number of fields in `struct utsname`: sysname, nodename, release, version,
/// machine, domainname.
pub const UTS_FIELD_COUNT: usize = 6;
/// Total size of `struct utsname` (`65 * 6 = 390` bytes).
pub const UTSNAME_SIZE: usize = UTS_FIELD_LEN * UTS_FIELD_COUNT;

/// The default process file-creation mask (`022`), matching Linux's init value.
pub const DEFAULT_UMASK: u32 = 0o022;
/// `umask` only carries the low nine permission bits.
pub const UMASK_MASK: u32 = 0o777;

/// Build a `struct utsname` (six NUL-padded 65-byte fields) byte-for-byte as
/// Linux's `uname(2)` returns it. `machine` is the only arch-varying field
/// (`"aarch64"` / `"x86_64"`); the rest are fixed for this kernel:
///   sysname="Linux", nodename="localhost", release="6.6.0",
///   version="#1 SMP PREEMPT", domainname="(none)".
/// Each field is the C string followed by a NUL and zero padding to 65 bytes,
/// so the result is exactly [`UTSNAME_SIZE`] bytes the caller copies to user.
pub fn build_utsname(machine: &str) -> [u8; UTSNAME_SIZE] {
    let mut out = [0u8; UTSNAME_SIZE];
    let fields: [&[u8]; UTS_FIELD_COUNT] = [
        b"Linux",
        b"localhost",
        b"6.6.0",
        b"#1 SMP PREEMPT",
        machine.as_bytes(),
        b"(none)",
    ];
    let mut i = 0;
    while i < UTS_FIELD_COUNT {
        let base = i * UTS_FIELD_LEN;
        let src = fields[i];
        // Copy up to FIELD_LEN-1 bytes, always leaving at least one trailing NUL
        // (the field is already zero-initialised, so short strings stay padded).
        let n = if src.len() < UTS_FIELD_LEN - 1 {
            src.len()
        } else {
            UTS_FIELD_LEN - 1
        };
        let mut j = 0;
        while j < n {
            out[base + j] = src[j];
            j += 1;
        }
        i += 1;
    }
    out
}

/// Apply a new `umask` request, returning `(previous, stored)`: the caller swaps
/// `stored` into its per-process slot and returns `previous`. Only the low nine
/// permission bits of the request are honoured (`mode & 0o777`), per Linux.
pub fn umask_swap(previous: u32, request: u64) -> (u32, u32) {
    (previous & UMASK_MASK, (request as u32) & UMASK_MASK)
}

/// `clock_getres` resolution for a clock id: this kernel reports 1-nanosecond
/// resolution for every clock it recognises (via the shared [`crate`] clock-id
/// validator the arch passes in), as `(tv_sec, tv_nsec)`. Returns `None` for an
/// unknown clock so the arch handler can map it to `-EINVAL`.
pub fn clock_getres(known_clock: bool) -> Option<(i64, i64)> {
    if known_clock {
        Some((0, 1))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utsname_is_six_nul_padded_65_byte_fields() {
        let u = build_utsname("aarch64");
        assert_eq!(u.len(), 390);
        // sysname
        assert_eq!(&u[0..5], b"Linux");
        assert_eq!(u[5], 0);
        // nodename @ 65
        assert_eq!(&u[65..74], b"localhost");
        assert_eq!(u[74], 0);
        // release @ 130
        assert_eq!(&u[130..135], b"6.6.0");
        // version @ 195
        assert_eq!(&u[195..209], b"#1 SMP PREEMPT");
        // machine @ 260
        assert_eq!(&u[260..267], b"aarch64");
        assert_eq!(u[267], 0);
        // domainname @ 325
        assert_eq!(&u[325..331], b"(none)");
    }

    #[test]
    fn utsname_machine_varies_by_arch() {
        let x = build_utsname("x86_64");
        assert_eq!(&x[260..266], b"x86_64");
        assert_eq!(x[266], 0);
    }

    #[test]
    fn utsname_every_field_is_nul_terminated() {
        let u = build_utsname("aarch64");
        for i in 0..UTS_FIELD_COUNT {
            // The last byte of each 65-byte field must be a guaranteed NUL.
            assert_eq!(u[i * UTS_FIELD_LEN + (UTS_FIELD_LEN - 1)], 0);
        }
    }

    #[test]
    fn umask_swaps_and_masks_to_low_nine_bits() {
        // Default previous (022), request 027 -> previous 022 returned, store 027.
        assert_eq!(umask_swap(0o022, 0o027), (0o022, 0o027));
        // High bits in the request are dropped.
        assert_eq!(umask_swap(0o022, 0o7777), (0o022, 0o777));
        // High bits in a corrupt previous are also masked off on read.
        assert_eq!(umask_swap(0o1022, 0).0, 0o022);
    }

    #[test]
    fn clock_getres_is_one_nanosecond_or_none() {
        assert_eq!(clock_getres(true), Some((0, 1)));
        assert_eq!(clock_getres(false), None);
    }
}
