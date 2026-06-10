//! NTP wire format (RFC 5905) modeling: the 48-byte packet, the 64-bit NTP
//! timestamp, and the leap-indicator / version / mode / stratum fields.
//!
//! Talos's `pkg/ntp` builds on the `beevik/ntp` library; here we re-implement
//! the relevant encode/decode and epoch math without external deps. We use
//! signed-millisecond offsets internally (instead of floating point) so the
//! crate stays deterministic and `no_std`-clean.

use crate::{Result, TimeError};

/// Seconds between the NTP epoch (1900-01-01) and the Unix epoch (1970-01-01).
///
/// 70 years including 17 leap days = `2_208_988_800` seconds.
pub const NTP_UNIX_EPOCH_DELTA: u64 = 2_208_988_800;

/// The NTP version this client speaks (`NTPv4`).
pub const NTP_VERSION: u8 = 4;

/// A 64-bit NTP timestamp: 32 bits of seconds since 1900, 32 bits of fraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NtpTimestamp {
    /// Seconds since 1900-01-01 (mod 2^32; the 2036 rollover is ignored here).
    pub seconds: u32,
    /// Binary fraction of a second (1/2^32 units).
    pub fraction: u32,
}

impl NtpTimestamp {
    /// Build an NTP timestamp from Unix milliseconds.
    // Deliberate fixed-point wire-format conversions: pre-1970 inputs are
    // clamped to 0, and the 1900-epoch seconds / 2^-32 fraction are truncated
    // into their 32-bit wire fields exactly as RFC 5905 prescribes.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn from_unix_millis(unix_millis: i64) -> Self {
        // Work in u64; negative pre-1970 values are not expected on a node.
        let unix_millis = unix_millis.max(0) as u64;
        let unix_secs = unix_millis / 1000;
        let millis_rem = unix_millis % 1000;
        let seconds = (unix_secs + NTP_UNIX_EPOCH_DELTA) as u32;
        // fraction = millis_rem/1000 * 2^32
        let fraction = ((millis_rem << 32) / 1000) as u32;
        NtpTimestamp { seconds, fraction }
    }

    /// Convert back to Unix milliseconds.
    // The reconstructed millisecond count is well within i64 range for any
    // realistic node clock; the final cast back to signed is intentional.
    #[allow(clippy::cast_possible_wrap)]
    pub fn to_unix_millis(self) -> i64 {
        let unix_secs = u64::from(self.seconds).wrapping_sub(NTP_UNIX_EPOCH_DELTA);
        let millis_frac = (u64::from(self.fraction) * 1000) >> 32;
        (unix_secs * 1000 + millis_frac) as i64
    }

    /// Encode into 8 big-endian bytes.
    pub fn to_be_bytes(self) -> [u8; 8] {
        let mut out = [0u8; 8];
        out[0..4].copy_from_slice(&self.seconds.to_be_bytes());
        out[4..8].copy_from_slice(&self.fraction.to_be_bytes());
        out
    }

    /// Decode from 8 big-endian bytes.
    pub fn from_be_bytes(b: [u8; 8]) -> Self {
        let seconds = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
        let fraction = u32::from_be_bytes([b[4], b[5], b[6], b[7]]);
        NtpTimestamp { seconds, fraction }
    }

    /// Whether the timestamp is the zero value (server never set it).
    pub fn is_zero(self) -> bool {
        self.seconds == 0 && self.fraction == 0
    }
}

/// NTP leap-indicator field (2 bits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeapIndicator {
    /// No warning.
    NoWarning,
    /// Last minute has 61 seconds.
    AddSecond,
    /// Last minute has 59 seconds.
    DelSecond,
    /// Clock not synchronized ("alarm condition").
    Unsynchronized,
}

impl LeapIndicator {
    /// Decode from the high 2 bits value (0..=3).
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => LeapIndicator::NoWarning,
            1 => LeapIndicator::AddSecond,
            2 => LeapIndicator::DelSecond,
            _ => LeapIndicator::Unsynchronized,
        }
    }

    /// Encode to the 2-bit value.
    pub fn to_bits(self) -> u8 {
        match self {
            LeapIndicator::NoWarning => 0,
            LeapIndicator::AddSecond => 1,
            LeapIndicator::DelSecond => 2,
            LeapIndicator::Unsynchronized => 3,
        }
    }
}

/// NTP association mode (3 bits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtpMode {
    /// Reserved / unknown.
    Reserved,
    /// Symmetric active.
    SymmetricActive,
    /// Symmetric passive.
    SymmetricPassive,
    /// Client (what an SNTP query sends).
    Client,
    /// Server (what a reply carries).
    Server,
    /// Broadcast.
    Broadcast,
}

impl NtpMode {
    /// Decode from the low 3 bits.
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0b111 {
            1 => NtpMode::SymmetricActive,
            2 => NtpMode::SymmetricPassive,
            3 => NtpMode::Client,
            4 => NtpMode::Server,
            5 => NtpMode::Broadcast,
            _ => NtpMode::Reserved,
        }
    }

    /// Encode to the 3-bit value.
    pub fn to_bits(self) -> u8 {
        match self {
            NtpMode::Reserved => 0,
            NtpMode::SymmetricActive => 1,
            NtpMode::SymmetricPassive => 2,
            NtpMode::Client => 3,
            NtpMode::Server => 4,
            NtpMode::Broadcast => 5,
        }
    }
}

/// NTP stratum (distance from a reference clock).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stratum(pub u8);

impl Stratum {
    /// Stratum 0 means "kiss-o'-death" / unspecified — never usable.
    pub fn is_kiss_of_death(self) -> bool {
        self.0 == 0
    }

    /// Stratum 16 means the server itself is unsynchronized.
    pub fn is_unsynchronized(self) -> bool {
        self.0 >= 16
    }

    /// A primary reference clock (GPS, atomic) is stratum 1.
    pub fn is_primary(self) -> bool {
        self.0 == 1
    }

    /// Whether this stratum is usable for disciplining the clock.
    pub fn is_usable(self) -> bool {
        !self.is_kiss_of_death() && !self.is_unsynchronized()
    }
}

/// A decoded/encodable 48-byte NTP packet (the fields Talos actually reads).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NtpPacket {
    /// Leap indicator.
    pub leap: LeapIndicator,
    /// Protocol version (3 bits).
    pub version: u8,
    /// Association mode.
    pub mode: NtpMode,
    /// Stratum.
    pub stratum: Stratum,
    /// Poll exponent (log2 seconds).
    pub poll: i8,
    /// Clock precision exponent (log2 seconds).
    pub precision: i8,
    /// Time the client sent the request (filled by us).
    pub originate: NtpTimestamp,
    /// Time the server received the request.
    pub receive: NtpTimestamp,
    /// Time the server sent the reply.
    pub transmit: NtpTimestamp,
}

impl NtpPacket {
    /// Total wire size of an NTP packet (without extensions/MAC).
    pub const WIRE_LEN: usize = 48;

    /// Build a client query packet whose transmit timestamp is `now_unix_millis`.
    pub fn new_client_query(now_unix_millis: i64) -> Self {
        NtpPacket {
            leap: LeapIndicator::NoWarning,
            version: NTP_VERSION,
            mode: NtpMode::Client,
            stratum: Stratum(0),
            poll: 0,
            precision: 0,
            originate: NtpTimestamp::default(),
            receive: NtpTimestamp::default(),
            transmit: NtpTimestamp::from_unix_millis(now_unix_millis),
        }
    }

    /// First byte: LI (2) | VN (3) | Mode (3).
    fn flags_byte(&self) -> u8 {
        (self.leap.to_bits() << 6) | ((self.version & 0b111) << 3) | self.mode.to_bits()
    }

    /// Encode to the 48-byte wire form.
    // poll/precision are signed log2-seconds exponents stored verbatim in their
    // wire bytes; the i8<->u8 casts reinterpret the bits, they do not lose data.
    #[allow(clippy::cast_sign_loss)]
    pub fn encode(&self) -> [u8; Self::WIRE_LEN] {
        let mut b = [0u8; Self::WIRE_LEN];
        b[0] = self.flags_byte();
        b[1] = self.stratum.0;
        b[2] = self.poll as u8;
        b[3] = self.precision as u8;
        // bytes 4..24 are root delay / dispersion / ref id — left zero.
        // reference timestamp 24..32 — left zero.
        b[24..32].copy_from_slice(&self.originate.to_be_bytes());
        b[32..40].copy_from_slice(&self.receive.to_be_bytes());
        b[40..48].copy_from_slice(&self.transmit.to_be_bytes());
        b
    }

    /// Decode a server reply from the wire form. Validates length and version.
    // poll/precision wire bytes are reinterpreted back into signed exponents.
    #[allow(clippy::cast_possible_wrap)]
    pub fn decode(buf: &[u8]) -> Result<Self> {
        if buf.len() < Self::WIRE_LEN {
            return Err(TimeError::malformed("packet shorter than 48 bytes"));
        }
        let flags = buf[0];
        let version = (flags >> 3) & 0b111;
        if version == 0 {
            return Err(TimeError::malformed("ntp version 0"));
        }
        let leap = LeapIndicator::from_bits(flags >> 6);
        let mode = NtpMode::from_bits(flags);
        let ts8 = |off: usize| {
            let mut a = [0u8; 8];
            a.copy_from_slice(&buf[off..off + 8]);
            NtpTimestamp::from_be_bytes(a)
        };
        Ok(NtpPacket {
            leap,
            version,
            mode,
            stratum: Stratum(buf[1]),
            poll: buf[2] as i8,
            precision: buf[3] as i8,
            originate: ts8(24),
            receive: ts8(32),
            transmit: ts8(40),
        })
    }

    /// Validate that a decoded reply is from a healthy, synchronized server.
    pub fn validate_reply(&self) -> Result<()> {
        if self.mode != NtpMode::Server && self.mode != NtpMode::Broadcast {
            return Err(TimeError::malformed("reply mode is not server"));
        }
        if self.leap == LeapIndicator::Unsynchronized || !self.stratum.is_usable() {
            return Err(TimeError::ServerUnsynchronized);
        }
        if self.transmit.is_zero() {
            return Err(TimeError::malformed("zero transmit timestamp"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_unix_roundtrip() {
        // 2021-01-01T00:00:00Z = 1609459200000 ms
        let ms = 1_609_459_200_500;
        let ts = NtpTimestamp::from_unix_millis(ms);
        // seconds field includes the 1900 epoch delta
        assert_eq!(u64::from(ts.seconds), 1_609_459_200 + NTP_UNIX_EPOCH_DELTA);
        let back = ts.to_unix_millis();
        // sub-second precision is exact to the millisecond after rounding
        assert!((back - ms).abs() <= 1);
    }

    #[test]
    fn flags_pack_and_unpack() {
        let p = NtpPacket::new_client_query(1_000);
        let wire = p.encode();
        // LI=0, VN=4, Mode=3 (client) => 0b00_100_011 = 0x23
        assert_eq!(wire[0], 0x23);
        let decoded = NtpPacket::decode(&wire).unwrap();
        assert_eq!(decoded.version, 4);
        assert_eq!(decoded.mode, NtpMode::Client);
        assert_eq!(decoded.transmit, p.transmit);
    }

    #[test]
    fn decode_rejects_short_and_v0() {
        assert_eq!(
            NtpPacket::decode(&[0u8; 10]).unwrap_err().kind(),
            "malformed_packet"
        );
        let mut wire = NtpPacket::new_client_query(1).encode();
        wire[0] = 0; // version 0
        assert_eq!(
            NtpPacket::decode(&wire).unwrap_err().kind(),
            "malformed_packet"
        );
    }

    #[test]
    fn stratum_classification() {
        assert!(Stratum(0).is_kiss_of_death());
        assert!(!Stratum(0).is_usable());
        assert!(Stratum(1).is_primary());
        assert!(Stratum(3).is_usable());
        assert!(Stratum(16).is_unsynchronized());
        assert!(!Stratum(16).is_usable());
    }

    #[test]
    fn validate_reply_rejects_unsynced() {
        let mut reply = NtpPacket::new_client_query(1_000);
        reply.mode = NtpMode::Server;
        reply.stratum = Stratum(2);
        reply.leap = LeapIndicator::Unsynchronized;
        assert_eq!(
            reply.validate_reply().unwrap_err(),
            TimeError::ServerUnsynchronized
        );

        reply.leap = LeapIndicator::NoWarning;
        reply.stratum = Stratum(0);
        assert_eq!(
            reply.validate_reply().unwrap_err(),
            TimeError::ServerUnsynchronized
        );

        reply.stratum = Stratum(2);
        assert!(reply.validate_reply().is_ok());
    }
}
