//! Log / event tunnelling over the SideroLink link, and the WireGuard key type.
//!
//! Once the link is up, Talos ships two telemetry streams to the management
//! plane (Sidero / Omni) across it, as wired by the SideroLink controllers:
//!
//! - kernel-log (`kmsg`) lines, via the `LogTunnel`, addressed to a URL the
//!   management plane advertises;
//! - machine events, via the `EventSink`.
//!
//! Both destinations are modelled as the [`Sink`] trait. [`InMemorySink`]
//! records everything for assertions. [`Tunnel`] buffers entries while the link
//! is down and flushes them in order once it comes up, mirroring Talos's
//! reconnect/replay behaviour for the kmsg and event sinks.
//!
//! This module also defines [`WireguardPublicKey`], the canonical base64 key
//! used across provisioning and the manager.

use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use os_kernel::error::{Error, Result};

/// Length in characters of a base64-encoded 32-byte WireGuard key.
pub const KEY_B64_LEN: usize = 44;

/// A WireGuard public key in canonical 44-character base64 form.
///
/// We validate length / charset / padding rather than implement Curve25519,
/// keeping the crate dependency-free.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WireguardPublicKey {
    encoded: String,
}

impl WireguardPublicKey {
    /// Validate and wrap a base64 key string.
    pub fn parse(s: impl Into<String>) -> Result<Self> {
        let encoded: String = s.into();
        if encoded.len() != KEY_B64_LEN {
            return Err(Error::invalid("wireguard key must be 44 base64 characters"));
        }
        if !encoded.ends_with('=') {
            return Err(Error::invalid("wireguard key must end with base64 padding"));
        }
        for (i, c) in encoded.char_indices() {
            let is_b64 = c.is_ascii_alphanumeric() || c == '+' || c == '/';
            let is_pad = c == '=' && i == KEY_B64_LEN - 1;
            if !(is_b64 || is_pad) {
                return Err(Error::invalid("wireguard key has invalid base64 character"));
            }
        }
        Ok(WireguardPublicKey { encoded })
    }

    /// Derive a deterministic, well-formed key from a seed (test/model use only;
    /// not a real Curve25519 key).
    pub fn derive_from_seed(seed: &str) -> Self {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut state = os_kernel::id::Fingerprint::of_str(seed).value() | 1;
        let mut out = String::with_capacity(KEY_B64_LEN);
        for _ in 0..KEY_B64_LEN - 1 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            out.push(ALPHABET[(state % 64) as usize] as char);
        }
        out.push('=');
        WireguardPublicKey { encoded: out }
    }

    /// The encoded key string.
    pub fn as_str(&self) -> &str {
        &self.encoded
    }
}

impl fmt::Display for WireguardPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encoded)
    }
}

impl fmt::Debug for WireguardPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix: String = self.encoded.chars().take(8).collect();
        write!(f, "WireguardPublicKey({prefix}…)")
    }
}

/// The kind of telemetry an entry carries over the tunnel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkKind {
    /// A kernel-log (`kmsg`) line.
    KernelLog,
    /// A machine event (state transition, sequence, address change, …).
    Event,
}

/// A single telemetry record queued for the management plane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TunnelEntry {
    /// Which stream this belongs to.
    pub kind: SinkKind,
    /// Monotonic sequence number assigned by the tunnel.
    pub seq: u64,
    /// The opaque payload (a formatted kmsg line or serialized event).
    pub payload: String,
}

/// A telemetry destination — the management-plane endpoint the link feeds.
pub trait Sink {
    /// Deliver one entry. Returning an error signals the link is unavailable so
    /// the tunnel will re-buffer and retry.
    fn deliver(&mut self, entry: &TunnelEntry) -> Result<()>;
}

/// An in-memory sink that records every delivered entry, and can be toggled to
/// fail delivery to model a link outage.
#[derive(Debug, Default)]
pub struct InMemorySink {
    delivered: Vec<TunnelEntry>,
    /// When true, [`Sink::deliver`] errors instead of recording.
    pub available: bool,
}

impl InMemorySink {
    /// A new, available sink.
    pub fn new() -> Self {
        InMemorySink {
            delivered: Vec::new(),
            available: true,
        }
    }

    /// All entries delivered so far, in order.
    pub fn delivered(&self) -> &[TunnelEntry] {
        &self.delivered
    }

    /// The payloads of delivered entries of a given kind.
    pub fn payloads_of(&self, kind: SinkKind) -> Vec<&str> {
        self.delivered
            .iter()
            .filter(|e| e.kind == kind)
            .map(|e| e.payload.as_str())
            .collect()
    }
}

impl Sink for InMemorySink {
    fn deliver(&mut self, entry: &TunnelEntry) -> Result<()> {
        if !self.available {
            return Err(Error::Timeout);
        }
        self.delivered.push(entry.clone());
        Ok(())
    }
}

/// Buffers telemetry while the link is down and flushes it in order once up.
///
/// Models the kmsg and event sinks Talos runs over SideroLink: entries are
/// enqueued unconditionally; [`Tunnel::flush`] attempts delivery to the sink and
/// stops at the first failure, leaving the remainder buffered for the next
/// attempt so ordering is preserved across reconnects. A bounded capacity drops
/// the oldest entries (like a ring buffer) to avoid unbounded growth.
#[derive(Debug)]
pub struct Tunnel {
    queue: VecDeque<TunnelEntry>,
    next_seq: u64,
    capacity: usize,
    dropped: u64,
    up: bool,
}

impl Tunnel {
    /// Create a tunnel with a bounded backlog `capacity`. The link starts down.
    pub fn new(capacity: usize) -> Self {
        Tunnel {
            queue: VecDeque::new(),
            next_seq: 0,
            capacity: capacity.max(1),
            dropped: 0,
            up: false,
        }
    }

    /// Mark the link up (after a successful provision + WireGuard handshake).
    pub fn mark_up(&mut self) {
        self.up = true;
    }

    /// Mark the link down (handshake lost / endpoint change in progress).
    pub fn mark_down(&mut self) {
        self.up = false;
    }

    /// Whether the link is currently up.
    pub fn is_up(&self) -> bool {
        self.up
    }

    /// Number of entries currently buffered.
    pub fn backlog(&self) -> usize {
        self.queue.len()
    }

    /// Total entries dropped due to capacity overflow.
    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    /// Enqueue a kernel-log line.
    pub fn push_kmsg(&mut self, line: impl Into<String>) -> u64 {
        self.enqueue(SinkKind::KernelLog, line.into())
    }

    /// Enqueue a machine event.
    pub fn push_event(&mut self, event: impl Into<String>) -> u64 {
        self.enqueue(SinkKind::Event, event.into())
    }

    fn enqueue(&mut self, kind: SinkKind, payload: String) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        if self.queue.len() >= self.capacity {
            self.queue.pop_front();
            self.dropped += 1;
        }
        self.queue.push_back(TunnelEntry { kind, seq, payload });
        seq
    }

    /// Attempt to flush the backlog to `sink`, stopping at the first failure.
    ///
    /// Returns the number of entries successfully delivered. Does nothing while
    /// the link is down. Entries that fail to deliver stay buffered in order.
    pub fn flush<S: Sink>(&mut self, sink: &mut S) -> usize {
        if !self.up {
            return 0;
        }
        let mut delivered = 0;
        while let Some(entry) = self.queue.front() {
            match sink.deliver(entry) {
                Ok(()) => {
                    self.queue.pop_front();
                    delivered += 1;
                }
                Err(_) => break,
            }
        }
        delivered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wireguard_key_validation() {
        let k = WireguardPublicKey::derive_from_seed("node");
        assert_eq!(k.as_str().len(), KEY_B64_LEN);
        assert!(WireguardPublicKey::parse(k.as_str()).is_ok());
        assert!(WireguardPublicKey::parse("short").is_err());
        assert!(WireguardPublicKey::parse("!".repeat(44)).is_err());
        // No trailing padding.
        assert!(WireguardPublicKey::parse("A".repeat(44)).is_err());
    }

    #[test]
    fn key_debug_does_not_leak_full_key() {
        let k = WireguardPublicKey::derive_from_seed("secret-node");
        let dbg = alloc::format!("{k:?}");
        assert!(dbg.starts_with("WireguardPublicKey("));
        assert!(!dbg.contains(k.as_str()));
    }

    #[test]
    fn tunnel_buffers_while_down_and_flushes_when_up() {
        let mut tunnel = Tunnel::new(16);
        let mut sink = InMemorySink::new();

        tunnel.push_kmsg("kernel: booting");
        tunnel.push_event("event: BootDone");
        assert_eq!(tunnel.backlog(), 2);
        // Nothing flushes while down.
        assert_eq!(tunnel.flush(&mut sink), 0);

        tunnel.mark_up();
        assert_eq!(tunnel.flush(&mut sink), 2);
        assert_eq!(tunnel.backlog(), 0);
        assert_eq!(sink.delivered().len(), 2);
    }

    #[test]
    fn flush_preserves_order_and_resumes_after_outage() {
        let mut tunnel = Tunnel::new(16);
        let mut sink = InMemorySink::new();
        tunnel.mark_up();

        tunnel.push_kmsg("line-1");
        tunnel.push_kmsg("line-2");

        // Simulate the sink going unavailable mid-stream.
        sink.available = false;
        assert_eq!(tunnel.flush(&mut sink), 0);
        assert_eq!(tunnel.backlog(), 2);

        // Recover; backlog drains in original order.
        sink.available = true;
        tunnel.push_kmsg("line-3");
        assert_eq!(tunnel.flush(&mut sink), 3);
        assert_eq!(
            sink.payloads_of(SinkKind::KernelLog),
            ["line-1", "line-2", "line-3"]
        );
    }

    #[test]
    fn separates_kmsg_and_event_streams() {
        let mut tunnel = Tunnel::new(16);
        let mut sink = InMemorySink::new();
        tunnel.mark_up();
        tunnel.push_kmsg("k1");
        tunnel.push_event("e1");
        tunnel.push_kmsg("k2");
        tunnel.flush(&mut sink);
        assert_eq!(sink.payloads_of(SinkKind::KernelLog), ["k1", "k2"]);
        assert_eq!(sink.payloads_of(SinkKind::Event), ["e1"]);
    }

    #[test]
    fn bounded_capacity_drops_oldest() {
        let mut tunnel = Tunnel::new(2);
        tunnel.push_kmsg("a");
        tunnel.push_kmsg("b");
        tunnel.push_kmsg("c"); // evicts "a"
        assert_eq!(tunnel.backlog(), 2);
        assert_eq!(tunnel.dropped(), 1);

        let mut sink = InMemorySink::new();
        tunnel.mark_up();
        tunnel.flush(&mut sink);
        assert_eq!(sink.payloads_of(SinkKind::KernelLog), ["b", "c"]);
    }

    #[test]
    fn sequence_numbers_are_monotonic() {
        let mut tunnel = Tunnel::new(8);
        let s0 = tunnel.push_kmsg("x");
        let s1 = tunnel.push_event("y");
        assert_eq!(s0, 0);
        assert_eq!(s1, 1);
    }
}
