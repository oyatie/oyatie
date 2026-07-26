//! The SNTP query/response state machine and the offset / round-trip-delay
//! computation, mirroring `siderolabs/talos`'s `pkg/ntp` client.
//!
//! The actual UDP socket is abstracted behind the [`NtpTransport`] trait so the
//! network syscall is mockable; an in-memory implementation is provided for
//! tests. The clock-offset math follows RFC 5905 section 8:
//!
//! ```text
//! offset = ((T2 - T1) + (T3 - T4)) / 2
//! delay  =  (T4 - T1) - (T3 - T2)
//! ```
//!
//! where T1 = originate, T2 = server receive, T3 = server transmit and
//! T4 = client destination (the moment the reply landed).

use crate::ntp::NtpPacket;
use crate::{Result, TimeError};

/// Abstraction over the UDP socket used to exchange NTP packets.
///
/// In production this wraps a `std::net::UdpSocket`; in tests an in-memory
/// fake replies with a canned server packet. `query` takes the encoded request
/// and the destination host and returns the raw reply bytes.
pub trait NtpTransport {
    /// Send `request` to `server` and return the raw reply datagram.
    fn query(&mut self, server: &str, request: &[u8]) -> Result<Vec<u8>>;
}

/// A single completed NTP measurement (one request/reply exchange).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NtpMeasurement {
    /// Estimated clock offset in milliseconds (local - reference; positive
    /// means the local clock is ahead and must be moved back).
    pub offset_ms: i64,
    /// Round-trip delay in milliseconds.
    pub delay_ms: i64,
    /// Server stratum from the reply.
    pub stratum: u8,
}

impl NtpMeasurement {
    /// Whether this measurement is precise enough to trust. Talos discards
    /// samples whose round-trip delay is implausibly large.
    pub fn is_acceptable(&self, max_delay_ms: i64) -> bool {
        self.delay_ms >= 0 && self.delay_ms <= max_delay_ms
    }
}

/// SNTP client: stateless apart from the request count it uses for round-robin
/// server selection. Drives an [`NtpTransport`].
#[derive(Debug, Default)]
pub struct NtpClient {
    attempts: usize,
}

impl NtpClient {
    /// Create a fresh client.
    pub fn new() -> Self {
        NtpClient { attempts: 0 }
    }

    /// Number of queries this client has issued.
    pub fn attempts(&self) -> usize {
        self.attempts
    }

    /// Perform one query against `server`.
    ///
    /// `send_unix_millis` is the local clock reading when the request leaves
    /// (T1) and `recv_unix_millis` is the local reading when the reply lands
    /// (T4). Both are supplied by the caller so the function is pure and
    /// testable. The server's T2/T3 come from the decoded reply.
    pub fn query<T: NtpTransport>(
        &mut self,
        transport: &mut T,
        server: &str,
        send_unix_millis: i64,
        recv_unix_millis: i64,
    ) -> Result<NtpMeasurement> {
        self.attempts += 1;

        let request = NtpPacket::new_client_query(send_unix_millis);
        let reply_bytes = transport.query(server, &request.encode())?;
        let reply = NtpPacket::decode(&reply_bytes)?;
        reply.validate_reply()?;

        let t1 = send_unix_millis;
        let t2 = reply.receive.to_unix_millis();
        let t3 = reply.transmit.to_unix_millis();
        let t4 = recv_unix_millis;

        let offset_ms = (t2 - t1).midpoint(t3 - t4);
        let delay_ms = (t4 - t1) - (t3 - t2);

        Ok(NtpMeasurement {
            offset_ms,
            delay_ms,
            stratum: reply.stratum.0,
        })
    }
}

/// An in-memory [`NtpTransport`] used for tests and offline modeling.
///
/// It echoes a server reply whose receive/transmit timestamps are derived from
/// a configured `server_unix_millis` reading, letting tests inject a known
/// clock offset.
#[derive(Debug, Clone)]
pub struct FakeTransport {
    /// The server's notion of "now" in Unix millis used to fill T2 and T3.
    pub server_unix_millis: i64,
    /// Stratum to advertise.
    pub stratum: u8,
    /// If set, the next `query` returns this error instead of a reply.
    pub fail_with: Option<TimeError>,
    /// Count of queries received.
    pub seen: usize,
}

impl FakeTransport {
    /// A healthy server at the given clock reading and stratum.
    pub fn healthy(server_unix_millis: i64, stratum: u8) -> Self {
        FakeTransport {
            server_unix_millis,
            stratum,
            fail_with: None,
            seen: 0,
        }
    }
}

impl NtpTransport for FakeTransport {
    fn query(&mut self, _server: &str, request: &[u8]) -> Result<Vec<u8>> {
        self.seen += 1;
        if let Some(err) = &self.fail_with {
            return Err(err.clone());
        }
        // Decode the client request to mirror its originate timestamp.
        let req = NtpPacket::decode(request)?;
        let mut reply = req;
        reply.mode = crate::ntp::NtpMode::Server;
        reply.stratum = crate::ntp::Stratum(self.stratum);
        reply.leap = crate::ntp::LeapIndicator::NoWarning;
        reply.originate = req.transmit;
        reply.receive = crate::ntp::NtpTimestamp::from_unix_millis(self.server_unix_millis);
        reply.transmit = crate::ntp::NtpTimestamp::from_unix_millis(self.server_unix_millis);
        Ok(reply.encode().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_offset_when_local_clock_is_behind() {
        // Local clock reads 1000 ms at send and recv; server reads 5000 ms.
        // The local clock is 4000 ms behind => positive offset toward future.
        let mut client = NtpClient::new();
        let mut tx = FakeTransport::healthy(5_000, 2);
        let m = client.query(&mut tx, "pool", 1_000, 1_000).unwrap();
        // offset = ((5000-1000)+(5000-1000))/2 = 4000
        assert_eq!(m.offset_ms, 4_000);
        assert_eq!(m.stratum, 2);
        assert_eq!(client.attempts(), 1);
    }

    #[test]
    fn computes_round_trip_delay() {
        // Send at 1000, reply lands at 1040 (40ms RTT), server processed at 5000.
        let mut client = NtpClient::new();
        let mut tx = FakeTransport::healthy(5_000, 1);
        let m = client.query(&mut tx, "pool", 1_000, 1_040).unwrap();
        // delay = (1040-1000) - (5000-5000) = 40
        assert_eq!(m.delay_ms, 40);
        assert!(m.is_acceptable(100));
        assert!(!m.is_acceptable(10));
    }

    #[test]
    fn propagates_transport_failure() {
        let mut client = NtpClient::new();
        let mut tx = FakeTransport::healthy(5_000, 2);
        tx.fail_with = Some(TimeError::transport("socket timeout"));
        let err = client.query(&mut tx, "pool", 1_000, 1_000).unwrap_err();
        assert_eq!(err.kind(), "transport");
    }

    #[test]
    fn rejects_unsynchronized_server() {
        let mut client = NtpClient::new();
        // stratum 0 is kiss-of-death => validate_reply rejects it.
        let mut tx = FakeTransport::healthy(5_000, 0);
        let err = client.query(&mut tx, "pool", 1_000, 1_000).unwrap_err();
        assert_eq!(err, TimeError::ServerUnsynchronized);
    }
}
