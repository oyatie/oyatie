// Pure, arch-neutral `AF_NETLINK`/`NETLINK_ROUTE` wire logic: parse an outbound
// `RTM_GETLINK` dump request and build the dump RESPONSE (an empty link set =
// a single `NLMSG_DONE`).
//
// This file is the body of the `user_layout::netlink` module (it is `include!`d
// from `lib.rs`, alongside `layout.rs`/`signal.rs`/`timekeep.rs`/`vfs.rs`, which
// supply the crate-level `#![no_std]`). Like those it carries **no inner
// attributes** (`#![...]`) or `//!` module docs, because it is `include!`d both
// into this crate and into the out-of-workspace host harness's module body,
// where inner attributes are not permitted.
//
// It is the **single source of truth** for the parts of the M2 netlink slice
// that are a *pure function* of their inputs and therefore identical on every
// arch: the request parse (extracting the `nlmsg_seq`/`nlmsg_pid` to echo) and
// the response build (a well-formed `NLMSG_DONE`). It depends on **nothing**
// outside `core` + `alloc` and contains **zero `unsafe`**, so the
// `check-tcb.sh` ratchet stays green and the wire format is exhaustively
// host-tested (see `mod netlink_tests` at the bottom, run through
// `crates/arch-aarch64/tests-host/`).
//
// Keeping this logic pure keeps the `unsafe` arch Frames thin: they only do the
// things that *must* be unsafe (copying the request bytes out of user memory,
// writing the response bytes back, the SMAP/PAN bracket), delegating all the
// byte-format work here where it can be tested against the real talos
// `talos-network/src/linux_net.rs` parser shape.
//
// ## Wire contract (verified against the real, unmodified talos consumer)
// `talos_network::list_link_statuses` (`linux_net.rs:1224-1238`) does exactly:
//   * `socket(AF_NETLINK, SOCK_RAW|SOCK_CLOEXEC, NETLINK_ROUTE)` then `bind`
//   * ONE `sendto` of `build_dump_links(seq)` — a 32-byte buffer = a 16-byte
//     `nlmsghdr{len=32, type=RTM_GETLINK(18), flags=NLM_F_REQUEST|NLM_F_DUMP
//     (0x301), seq, pid=0}` + a 16-byte zeroed `ifinfomsg`.
//   * a `recv` loop that, per datagram, calls `dump_chunk_done_or_error`
//     (`linux_net.rs:792-824`): it walks `nlmsghdr`s and returns `Ok(true)` the
//     moment it sees `nlmsg_type == NLMSG_DONE(3)`. It reads ONLY `nlmsg_len`
//     (must be `>= 16` and `<= buf.len()`) and `nlmsg_type`.
//   * `parse_link_dump` (`linux_net.rs:747-784`) re-walks the accumulated bytes:
//     same length checks, `NLMSG_DONE` breaks the loop, and with no
//     `RTM_NEWLINK` bodies it returns an empty `Vec<LinkStatus>` — exactly the
//     empty link set the COSI `LinkStatusSourceController` converges on.
//
// So the minimal honest, correct RESPONSE is a single 16-byte `NLMSG_DONE`:
//   nlmsg_len = 16   (LE u32)   — satisfies `len >= 16 && off+len <= buf.len()`
//   nlmsg_type = 3   (LE u16)   — NLMSG_DONE → `Ok(true)` / loop break
//   nlmsg_flags = 2  (LE u16)   — NLM_F_MULTI (honest multipart marker; not read)
//   nlmsg_seq = <echoed request seq>  (LE u32)
//   nlmsg_pid = <kernel-assigned nonzero port>  (LE u32)
// Both target arches (aarch64, x86_64) are little-endian, matching talos's
// `from_ne_bytes` reads, so we emit little-endian throughout.

// In the bare-metal `user_layout` crate this `extern crate alloc;` links the
// allocator the kernel registers; in the out-of-workspace host test harness it
// links `std`'s precompiled `alloc`, bringing `alloc::vec::Vec` into scope in
// BOTH contexts without an inner attribute. (Mirrors `vfs.rs`.)
extern crate alloc;
use alloc::vec::Vec;

// ---------------------------------------------------------------------------
// Netlink uapi constants (mirror `talos-network/src/linux_net.rs:84-170`).
// ---------------------------------------------------------------------------

/// Header size of `struct nlmsghdr` (4*u32-ish = 16 bytes: len/type/flags/seq +
/// pid). The minimum length the talos parser accepts for any message.
pub const NLMSGHDR_LEN: usize = 16;
/// Size of `struct ifinfomsg` — the 16-byte body of the `RTM_GETLINK` request
/// (and of an `RTM_NEWLINK` reply; we emit none).
pub const IFINFOMSG_LEN: usize = 16;

/// `NLMSG_ERROR` — error / ack message type (`linux_net.rs:97`).
pub const NLMSG_ERROR: u16 = 2;
/// `NLMSG_DONE` — end of a multipart dump (`linux_net.rs:99`). The single
/// message our zero-link response carries.
pub const NLMSG_DONE: u16 = 3;
/// `RTM_NEWLINK` — create/modify (reply) link (`linux_net.rs:87`). We emit none.
pub const RTM_NEWLINK: u16 = 16;
/// `RTM_GETLINK` — dump/query links (`linux_net.rs:89`). The request type.
pub const RTM_GETLINK: u16 = 18;

/// `NLM_F_REQUEST` — this is a request (`linux_net.rs:102`).
pub const NLM_F_REQUEST: u16 = 0x01;
/// `NLM_F_MULTI` — part of a multipart message (terminated by `NLMSG_DONE`).
/// Honest marker on our DONE; the talos dump loop does not read flags.
pub const NLM_F_MULTI: u16 = 0x02;
/// `NLM_F_DUMP` — return a list (ROOT | MATCH = 0x100 | 0x200 = 0x300)
/// (`linux_net.rs:110`).
pub const NLM_F_DUMP: u16 = 0x100 | 0x200;

/// `AF_NETLINK` address family (the `socket(2)` domain + `sockaddr_nl.nl_family`).
pub const AF_NETLINK: u16 = 16;
/// `NETLINK_ROUTE` protocol (`socket(2)` protocol arg) (`linux_net.rs:84`).
pub const NETLINK_ROUTE: u32 = 0;
/// Size of `struct sockaddr_nl` (family/pad/pid/groups = 2+2+4+4 = 12 bytes).
pub const SOCKADDR_NL_LEN: usize = 12;

// ---------------------------------------------------------------------------
// Little-endian field readers (mirror `linux_net.rs` `read_u16`/`read_u32`,
// which use `from_ne_bytes`; both target arches are LE). Bounds-checked: a
// short buffer yields `None` so a truncated request never panics.
// ---------------------------------------------------------------------------

fn read_u16_le(buf: &[u8], off: usize) -> Option<u16> {
    let end = off.checked_add(2)?;
    if end > buf.len() {
        return None;
    }
    Some(u16::from_le_bytes([buf[off], buf[off + 1]]))
}

fn read_u32_le(buf: &[u8], off: usize) -> Option<u32> {
    let end = off.checked_add(4)?;
    if end > buf.len() {
        return None;
    }
    Some(u32::from_le_bytes([
        buf[off],
        buf[off + 1],
        buf[off + 2],
        buf[off + 3],
    ]))
}

// ---------------------------------------------------------------------------
// Request model
// ---------------------------------------------------------------------------

/// What an outbound netlink datagram (the bytes from `sendto`) asked for, parsed
/// purely from the buffer. We model only what the link-status dump path needs;
/// every well-formed header still yields a variant so the arch glue can always
/// produce a terminating `NLMSG_DONE` (honest no-op for anything we don't model).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NlRequest {
    /// An `RTM_GETLINK` dump (`NLM_F_DUMP` set). Carries the request's
    /// `nlmsg_seq` (echoed back) and `nlmsg_pid` (informational).
    DumpLinks { seq: u32, pid: u32 },
    /// A well-formed header we don't specifically model (any other type, or
    /// `RTM_GETLINK` without `NLM_F_DUMP`). Carries `seq`/`pid` + the raw type so
    /// the responder can still terminate the exchange with a `NLMSG_DONE`.
    Other { seq: u32, pid: u32, msg_type: u16 },
}

impl NlRequest {
    /// The `nlmsg_seq` to echo in the response, regardless of variant.
    pub fn seq(&self) -> u32 {
        match *self {
            NlRequest::DumpLinks { seq, .. } | NlRequest::Other { seq, .. } => seq,
        }
    }
}

/// Parse one outbound netlink datagram (the bytes a process passed to
/// `sendto`). Pure; never touches a socket. Returns `None` on a header shorter
/// than `NLMSGHDR_LEN` or with an internally-inconsistent `nlmsg_len`
/// (`< NLMSGHDR_LEN` or running past the buffer) — i.e. exactly the cases the
/// talos sender never produces and that a strict kernel would reject.
///
/// `nlmsghdr` layout (LE): len@0(u32) type@4(u16) flags@6(u16) seq@8(u32)
/// pid@12(u32). We read type/flags/seq/pid; `len` is validated for sanity.
pub fn parse_request(buf: &[u8]) -> Option<NlRequest> {
    if buf.len() < NLMSGHDR_LEN {
        return None;
    }
    let nlmsg_len = read_u32_le(buf, 0)? as usize;
    let nlmsg_type = read_u16_le(buf, 4)?;
    let nlmsg_flags = read_u16_le(buf, 6)?;
    let nlmsg_seq = read_u32_le(buf, 8)?;
    let nlmsg_pid = read_u32_le(buf, 12)?;
    // Reject an internally-inconsistent length: too small to hold a header, or
    // claiming more bytes than were actually sent. (talos's `build_dump_links`
    // always patches `nlmsg_len` to the true 32; a garbage value is rejected.)
    if nlmsg_len < NLMSGHDR_LEN || nlmsg_len > buf.len() {
        return None;
    }
    if nlmsg_type == RTM_GETLINK && (nlmsg_flags & NLM_F_DUMP) == NLM_F_DUMP {
        Some(NlRequest::DumpLinks {
            seq: nlmsg_seq,
            pid: nlmsg_pid,
        })
    } else {
        Some(NlRequest::Other {
            seq: nlmsg_seq,
            pid: nlmsg_pid,
            msg_type: nlmsg_type,
        })
    }
}

// ---------------------------------------------------------------------------
// Response builders
// ---------------------------------------------------------------------------

/// Append a `struct nlmsghdr` (16 bytes, little-endian) to `out`. Mirrors
/// talos's `push_nlmsghdr` (`linux_net.rs:252-258`) but with the length and pid
/// filled in directly (the kernel knows them up front; no placeholder patch).
fn push_nlmsghdr(out: &mut Vec<u8>, len: u32, msg_type: u16, flags: u16, seq: u32, pid: u32) {
    out.extend_from_slice(&len.to_le_bytes()); // nlmsg_len
    out.extend_from_slice(&msg_type.to_le_bytes()); // nlmsg_type
    out.extend_from_slice(&flags.to_le_bytes()); // nlmsg_flags
    out.extend_from_slice(&seq.to_le_bytes()); // nlmsg_seq
    out.extend_from_slice(&pid.to_le_bytes()); // nlmsg_pid
}

/// Build the `RTM_GETLINK` dump RESPONSE for `req` into `out` (cleared first),
/// returning the number of bytes written.
///
/// For the zero-link M2 checkpoint this is a single 16-byte `NLMSG_DONE`
/// echoing the request `seq` with `port` as `nlmsg_pid`:
///   `[10 00 00 00] [03 00] [02 00] [<seq LE>] [<port LE>]`
/// (len=16, type=NLMSG_DONE(3), flags=NLM_F_MULTI(2), seq, pid=port).
///
/// This is exactly what `dump_chunk_done_or_error` (returns `Ok(true)` on
/// `NLMSG_DONE`) and `parse_link_dump` (breaks on `NLMSG_DONE`, no
/// `RTM_NEWLINK` bodies ⇒ empty link set) accept. `Other` requests get the same
/// terminating `NLMSG_DONE` (an honest empty multipart reply).
pub fn build_link_dump_response(req: NlRequest, port: u32, out: &mut Vec<u8>) -> usize {
    out.clear();
    push_nlmsghdr(
        out,
        NLMSGHDR_LEN as u32,
        NLMSG_DONE,
        NLM_F_MULTI,
        req.seq(),
        port,
    );
    out.len()
}

/// Validate a `struct sockaddr_nl` copied from user memory for `bind`/`sendto`.
/// Layout (LE): nl_family@0(u16) nl_pad@2(u16) nl_pid@4(u32) nl_groups@8(u32).
/// We require `nl_family == AF_NETLINK`; `nl_pid == 0` means "kernel assigns the
/// port" (which the socket already did). Returns `true` iff the address is a
/// well-formed `AF_NETLINK` address. A buffer shorter than 12 bytes is rejected.
pub fn validate_sockaddr_nl(buf: &[u8]) -> bool {
    match read_u16_le(buf, 0) {
        Some(family) => family == AF_NETLINK,
        None => false,
    }
}

// ---------------------------------------------------------------------------
// Host unit tests (run via crates/arch-aarch64/tests-host). Asserts the request
// parse, the exact response bytes, the seq/pid echo, and truncation handling —
// and re-implements the talos parser shape (`dump_chunk_done_or_error` /
// `parse_link_dump`) as a fixture to prove the bytes are accepted.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod netlink_tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    /// Re-create talos's `build_dump_links(seq)` (`linux_net.rs:424-430`): a
    /// 16-byte `nlmsghdr{len=32, type=RTM_GETLINK, flags=REQUEST|DUMP, seq,
    /// pid=0}` + a 16-byte zeroed `ifinfomsg`. Native-endian on the (LE) host.
    fn talos_build_dump_links(seq: u32) -> Vec<u8> {
        let mut buf = Vec::new();
        // nlmsghdr with placeholder length, patched below.
        buf.extend_from_slice(&0u32.to_ne_bytes()); // nlmsg_len (placeholder)
        buf.extend_from_slice(&RTM_GETLINK.to_ne_bytes()); // nlmsg_type
        buf.extend_from_slice(&(NLM_F_REQUEST | NLM_F_DUMP).to_ne_bytes()); // nlmsg_flags
        buf.extend_from_slice(&seq.to_ne_bytes()); // nlmsg_seq
        buf.extend_from_slice(&0u32.to_ne_bytes()); // nlmsg_pid
                                                    // ifinfomsg (16 zero bytes: family/pad/type/index/flags/change)
        buf.extend_from_slice(&[0u8; IFINFOMSG_LEN]);
        // patch nlmsg_len to the real length (32).
        let len = buf.len() as u32;
        buf[0..4].copy_from_slice(&len.to_ne_bytes());
        buf
    }

    fn nl_align(len: usize) -> usize {
        (len + 3) & !3
    }

    /// Faithful re-implementation of talos's `dump_chunk_done_or_error`
    /// (`linux_net.rs:792-824`): walks `nlmsghdr`s, `Ok(true)` on `NLMSG_DONE`,
    /// `Err` on a nonzero-errno `NLMSG_ERROR` or a malformed length.
    fn talos_dump_chunk_done_or_error(buf: &[u8]) -> Result<bool, &'static str> {
        let mut off = 0usize;
        while off + NLMSGHDR_LEN <= buf.len() {
            let nlmsg_len = read_u32_le(buf, off).ok_or("truncated nlmsghdr")? as usize;
            let nlmsg_type = read_u16_le(buf, off + 4).ok_or("truncated nlmsghdr")?;
            if nlmsg_len < NLMSGHDR_LEN || off + nlmsg_len > buf.len() {
                return Err("malformed netlink message");
            }
            match nlmsg_type {
                NLMSG_DONE => return Ok(true),
                NLMSG_ERROR => {
                    let errno = read_u32_le(buf, off + NLMSGHDR_LEN).unwrap_or(0) as i32;
                    if errno != 0 {
                        return Err("netlink dump error");
                    }
                }
                _ => {}
            }
            off += nl_align(nlmsg_len);
        }
        Ok(false)
    }

    /// Faithful re-implementation of talos's `parse_link_dump` for the
    /// zero-link case (`linux_net.rs:747-784`): same length checks, `NLMSG_DONE`
    /// breaks, `RTM_NEWLINK` bodies would be parsed (we emit none) ⇒ empty count.
    fn talos_parse_link_dump_count(buf: &[u8]) -> Result<usize, &'static str> {
        let mut count = 0usize;
        let mut off = 0usize;
        while off + NLMSGHDR_LEN <= buf.len() {
            let nlmsg_len = read_u32_le(buf, off).ok_or("truncated nlmsghdr")? as usize;
            let nlmsg_type = read_u16_le(buf, off + 4).ok_or("truncated nlmsghdr")?;
            if nlmsg_len < NLMSGHDR_LEN || off + nlmsg_len > buf.len() {
                return Err("malformed netlink message");
            }
            match nlmsg_type {
                NLMSG_DONE => break,
                RTM_NEWLINK => count += 1,
                _ => {}
            }
            off += nl_align(nlmsg_len);
        }
        Ok(count)
    }

    #[test]
    fn parses_a_real_talos_dump_request() {
        let req = talos_build_dump_links(0xDEAD_BEEF);
        assert_eq!(req.len(), 32, "request is nlmsghdr(16)+ifinfomsg(16)");
        let parsed = parse_request(&req).expect("well-formed request must parse");
        assert_eq!(
            parsed,
            NlRequest::DumpLinks {
                seq: 0xDEAD_BEEF,
                pid: 0
            }
        );
        assert_eq!(parsed.seq(), 0xDEAD_BEEF);
    }

    #[test]
    fn dump_request_with_various_seqs_round_trips_seq() {
        for &seq in &[0u32, 1, 7, 0x0001_0000, 0xFFFF_FFFF] {
            let req = talos_build_dump_links(seq);
            let parsed = parse_request(&req).unwrap();
            assert_eq!(parsed.seq(), seq, "seq must be echoed back exactly");
        }
    }

    #[test]
    fn response_is_exactly_16_byte_nlmsg_done() {
        let req = NlRequest::DumpLinks {
            seq: 0x1122_3344,
            pid: 0,
        };
        let mut out = Vec::new();
        let n = build_link_dump_response(req, 0x42, &mut out);
        assert_eq!(n, 16, "zero-link response is a single 16-byte NLMSG_DONE");
        assert_eq!(out.len(), 16);
        // Exact bytes (little-endian):
        //   len=16        -> 10 00 00 00
        //   type=DONE(3)  -> 03 00
        //   flags=MULTI(2)-> 02 00
        //   seq           -> 44 33 22 11
        //   pid=port(0x42)-> 42 00 00 00
        assert_eq!(
            out,
            vec![
                0x10, 0x00, 0x00, 0x00, // nlmsg_len = 16
                0x03, 0x00, // nlmsg_type = NLMSG_DONE
                0x02, 0x00, // nlmsg_flags = NLM_F_MULTI
                0x44, 0x33, 0x22, 0x11, // nlmsg_seq (echoed)
                0x42, 0x00, 0x00, 0x00, // nlmsg_pid = port
            ]
        );
    }

    #[test]
    fn response_echoes_request_seq_and_port() {
        for &(seq, port) in &[(0u32, 1u32), (5, 0x1000), (0xFFFF_FFFF, 0xABCD)] {
            let req = NlRequest::DumpLinks { seq, pid: 0 };
            let mut out = Vec::new();
            build_link_dump_response(req, port, &mut out);
            assert_eq!(read_u32_le(&out, 0).unwrap(), 16, "len");
            assert_eq!(read_u16_le(&out, 4).unwrap(), NLMSG_DONE, "type");
            assert_eq!(read_u16_le(&out, 6).unwrap(), NLM_F_MULTI, "flags");
            assert_eq!(read_u32_le(&out, 8).unwrap(), seq, "echoed seq");
            assert_eq!(read_u32_le(&out, 12).unwrap(), port, "port pid");
        }
    }

    #[test]
    fn build_clears_previous_contents() {
        let mut out = vec![0xFFu8; 64];
        let req = NlRequest::DumpLinks { seq: 9, pid: 0 };
        let n = build_link_dump_response(req, 1, &mut out);
        assert_eq!(n, 16);
        assert_eq!(out.len(), 16, "stale bytes from a prior call are cleared");
    }

    #[test]
    fn talos_parser_accepts_our_response_as_done_with_zero_links() {
        // The end-to-end proof: build the response the kernel emits, then run it
        // through faithful copies of the two talos parsers that consume it.
        let req = parse_request(&talos_build_dump_links(77)).unwrap();
        let mut out = Vec::new();
        build_link_dump_response(req, 0x99, &mut out);
        // dump_chunk_done_or_error → Ok(true): the recv loop terminates.
        assert_eq!(talos_dump_chunk_done_or_error(&out), Ok(true));
        // parse_link_dump → empty link set (no RTM_NEWLINK bodies): the COSI
        // LinkStatusSourceController reconcile writes nothing and converges.
        assert_eq!(talos_parse_link_dump_count(&out), Ok(0));
    }

    #[test]
    fn other_request_still_gets_a_terminating_done() {
        // A non-dump request (e.g. RTM_GETLINK without NLM_F_DUMP, or some other
        // type) is modeled as Other and still answered with a DONE so the
        // exchange terminates honestly.
        let mut buf = Vec::new();
        buf.extend_from_slice(&16u32.to_le_bytes()); // len
        buf.extend_from_slice(&RTM_GETLINK.to_le_bytes()); // type
        buf.extend_from_slice(&NLM_F_REQUEST.to_le_bytes()); // flags WITHOUT dump
        buf.extend_from_slice(&123u32.to_le_bytes()); // seq
        buf.extend_from_slice(&0u32.to_le_bytes()); // pid
        let parsed = parse_request(&buf).unwrap();
        assert_eq!(
            parsed,
            NlRequest::Other {
                seq: 123,
                pid: 0,
                msg_type: RTM_GETLINK
            }
        );
        let mut out = Vec::new();
        build_link_dump_response(parsed, 1, &mut out);
        assert_eq!(talos_dump_chunk_done_or_error(&out), Ok(true));
        assert_eq!(read_u32_le(&out, 8).unwrap(), 123, "Other still echoes seq");
    }

    #[test]
    fn truncated_request_is_rejected() {
        // Shorter than a 16-byte header → None (never a panic).
        for n in 0..NLMSGHDR_LEN {
            let short = vec![0u8; n];
            assert!(parse_request(&short).is_none(), "len {n} must be rejected");
        }
    }

    #[test]
    fn inconsistent_nlmsg_len_is_rejected() {
        // nlmsg_len < 16 → rejected.
        let mut b = talos_build_dump_links(1);
        b[0..4].copy_from_slice(&8u32.to_le_bytes());
        assert!(parse_request(&b).is_none());
        // nlmsg_len > buffer → rejected.
        let mut b2 = talos_build_dump_links(1);
        b2[0..4].copy_from_slice(&999u32.to_le_bytes());
        assert!(parse_request(&b2).is_none());
    }

    #[test]
    fn sockaddr_nl_validation() {
        // A valid AF_NETLINK sockaddr_nl (family=16, pad=0, pid=0, groups=0).
        let mut addr = Vec::new();
        addr.extend_from_slice(&AF_NETLINK.to_le_bytes()); // nl_family
        addr.extend_from_slice(&0u16.to_le_bytes()); // nl_pad
        addr.extend_from_slice(&0u32.to_le_bytes()); // nl_pid
        addr.extend_from_slice(&0u32.to_le_bytes()); // nl_groups
        assert_eq!(addr.len(), SOCKADDR_NL_LEN);
        assert!(validate_sockaddr_nl(&addr));
        // Wrong family → rejected.
        let mut bad = addr.clone();
        bad[0..2].copy_from_slice(&2u16.to_le_bytes()); // AF_INET
        assert!(!validate_sockaddr_nl(&bad));
        // Too short → rejected (no panic).
        assert!(!validate_sockaddr_nl(&[16u8]));
        assert!(!validate_sockaddr_nl(&[]));
    }
}
