//! A real, dependency-free wire protocol for the Talos machine API subset.
//!
//! This module is deliberately self-contained: it depends only on `std` and is
//! usable by both a server and a client over any [`std::io::Read`] +
//! [`std::io::Write`] (a `TcpStream`, a Unix `UnixStream`, an in-memory
//! `Vec<u8>`, ...). It provides three layers:
//!
//! 1. **Framing** ([`write_frame`] / [`read_frame`]): a length-prefixed frame
//!    of a 4-byte big-endian unsigned length followed by exactly that many
//!    payload bytes. This is symmetric and survives partial reads/writes.
//! 2. **A message model** ([`Request`] / [`Response`]) for the Talos machine
//!    API subset: `Version`, `Hostname`, `ServiceList`, `ServiceStart{name}`,
//!    `Dmesg`, and `Reboot`, with matching responses plus an [`Response::Error`]
//!    variant.
//! 3. **Manual, dependency-free (de)serialization** ([`Encode`] / [`Decode`])
//!    using a small hand-rolled tagged binary format. No serde, no protobuf.
//!
//! The end-to-end entry points a server and client use are
//! [`write_message`] and [`read_message`], which compose the codec with the
//! framing layer: each [`Request`]/[`Response`] is encoded to bytes and written
//! as exactly one frame, and read back as exactly one frame and decoded.

use std::io::{self, Read, Write};

/// The maximum frame payload length this codec will accept, as a guard against
/// a malicious or corrupt length prefix forcing an unbounded allocation. 16 MiB
/// is far larger than any real machine-API message (dmesg dumps are streamed in
/// chunks below this).
pub const MAX_FRAME_LEN: usize = 16 * 1024 * 1024;

// ===========================================================================
// Errors
// ===========================================================================

/// An error from the wire protocol: either an I/O failure from the underlying
/// stream, or a protocol/decoding violation (truncated, garbage, or oversized
/// data).
#[derive(Debug)]
pub enum WireError {
    /// The underlying stream returned an I/O error.
    Io(io::Error),
    /// A frame's declared length exceeded [`MAX_FRAME_LEN`].
    FrameTooLarge {
        /// The declared length that was rejected.
        len: usize,
    },
    /// The byte buffer ended before a value could be fully decoded.
    UnexpectedEof,
    /// An unknown tag byte was encountered while decoding a tagged value.
    UnknownTag {
        /// A short label for the kind of value being decoded.
        context: &'static str,
        /// The unexpected tag byte.
        tag: u8,
    },
    /// Decoded bytes were not valid UTF-8 where a string was expected.
    InvalidUtf8,
    /// Trailing bytes remained after a value was fully decoded from a frame.
    TrailingBytes {
        /// How many bytes were left over.
        remaining: usize,
    },
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireError::Io(e) => write!(f, "wire i/o error: {e}"),
            WireError::FrameTooLarge { len } => {
                write!(f, "frame length {len} exceeds maximum {MAX_FRAME_LEN}")
            }
            WireError::UnexpectedEof => write!(f, "unexpected end of input while decoding"),
            WireError::UnknownTag { context, tag } => {
                write!(f, "unknown {context} tag byte {tag:#04x}")
            }
            WireError::InvalidUtf8 => write!(f, "invalid utf-8 in decoded string"),
            WireError::TrailingBytes { remaining } => {
                write!(f, "{remaining} trailing byte(s) after decoded value")
            }
        }
    }
}

impl std::error::Error for WireError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WireError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for WireError {
    fn from(e: io::Error) -> Self {
        WireError::Io(e)
    }
}

/// A convenience result alias for the wire protocol.
pub type WireResult<T> = Result<T, WireError>;

// ===========================================================================
// Framing: 4-byte big-endian length prefix + payload.
// ===========================================================================

/// Write a single length-prefixed frame: a 4-byte big-endian length followed by
/// `payload`. The write is flushed to the underlying writer.
///
/// Errors if the payload exceeds [`MAX_FRAME_LEN`] (so peers agree on the same
/// bound in both directions) or on any I/O failure.
pub fn write_frame<W: Write>(w: &mut W, payload: &[u8]) -> WireResult<()> {
    if payload.len() > MAX_FRAME_LEN {
        return Err(WireError::FrameTooLarge { len: payload.len() });
    }
    let len = payload.len() as u32;
    w.write_all(&len.to_be_bytes())?;
    w.write_all(payload)?;
    w.flush()?;
    Ok(())
}

/// Read a single length-prefixed frame, returning the payload bytes.
///
/// This handles partial reads: [`Read::read_exact`] is used to loop until the
/// full 4-byte header and the full payload have been consumed. A header
/// declaring more than [`MAX_FRAME_LEN`] bytes is rejected before allocating.
///
/// A clean EOF *before any byte of the header* is reported as
/// [`WireError::UnexpectedEof`]; callers that want to distinguish "stream
/// closed cleanly between frames" can use [`read_frame_opt`].
pub fn read_frame<R: Read>(r: &mut R) -> WireResult<Vec<u8>> {
    match read_frame_opt(r)? {
        Some(payload) => Ok(payload),
        None => Err(WireError::UnexpectedEof),
    }
}

/// Like [`read_frame`], but returns `Ok(None)` if the stream is at a clean EOF
/// on a frame boundary (no bytes of the next header available). Any partial
/// header is still an error.
pub fn read_frame_opt<R: Read>(r: &mut R) -> WireResult<Option<Vec<u8>>> {
    let mut header = [0u8; 4];
    // Read the header byte-by-byte tolerance: detect clean EOF on the first byte.
    let mut filled = 0;
    while filled < 4 {
        match r.read(&mut header[filled..]) {
            Ok(0) => {
                if filled == 0 {
                    // Clean EOF exactly on a frame boundary.
                    return Ok(None);
                }
                // EOF in the middle of the header: truncated frame.
                return Err(WireError::UnexpectedEof);
            }
            Ok(n) => filled += n,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(WireError::Io(e)),
        }
    }

    let len = u32::from_be_bytes(header) as usize;
    if len > MAX_FRAME_LEN {
        return Err(WireError::FrameTooLarge { len });
    }

    let mut payload = vec![0u8; len];
    // read_exact loops internally and surfaces a truncated payload as
    // UnexpectedEof.
    match r.read_exact(&mut payload) {
        Ok(()) => Ok(Some(payload)),
        Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => Err(WireError::UnexpectedEof),
        Err(e) => Err(WireError::Io(e)),
    }
}

// ===========================================================================
// Low-level primitive (de)serialization helpers.
//
// Layout conventions used throughout the message codec:
//   * u8/u32/u64: fixed-width big-endian.
//   * bool:       a single byte, 0 or 1 (any non-zero decodes as true).
//   * string:     u32 big-endian byte length + raw UTF-8 bytes.
//   * bytes:      u32 big-endian byte length + raw bytes.
//   * vec<T>:     u32 big-endian element count + each element encoded in order.
// ===========================================================================

fn put_u8(buf: &mut Vec<u8>, v: u8) {
    buf.push(v);
}

fn put_u32(buf: &mut Vec<u8>, v: u32) {
    buf.extend_from_slice(&v.to_be_bytes());
}

fn put_u64(buf: &mut Vec<u8>, v: u64) {
    buf.extend_from_slice(&v.to_be_bytes());
}

fn put_bool(buf: &mut Vec<u8>, v: bool) {
    buf.push(v as u8);
}

fn put_str(buf: &mut Vec<u8>, s: &str) {
    put_u32(buf, s.len() as u32);
    buf.extend_from_slice(s.as_bytes());
}

fn put_bytes(buf: &mut Vec<u8>, b: &[u8]) {
    put_u32(buf, b.len() as u32);
    buf.extend_from_slice(b);
}

/// A cursor over a byte slice used while decoding. Every read is bounds-checked
/// and returns [`WireError::UnexpectedEof`] rather than panicking.
///
/// This is part of the public [`Decode`] surface so that external
/// implementations of [`Decode::decode_from`] can be written, but most callers
/// only need the [`Decode::decode`] / [`Encode::encode`] entry points.
pub struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    /// Create a cursor positioned at the start of `data`.
    pub fn new(data: &'a [u8]) -> Self {
        Cursor { data, pos: 0 }
    }

    /// Bytes not yet consumed.
    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    fn take(&mut self, n: usize) -> WireResult<&'a [u8]> {
        if self.remaining() < n {
            return Err(WireError::UnexpectedEof);
        }
        let s = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    fn get_u8(&mut self) -> WireResult<u8> {
        Ok(self.take(1)?[0])
    }

    fn get_u32(&mut self) -> WireResult<u32> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn get_u64(&mut self) -> WireResult<u64> {
        let b = self.take(8)?;
        Ok(u64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    fn get_bool(&mut self) -> WireResult<bool> {
        Ok(self.get_u8()? != 0)
    }

    fn get_str(&mut self) -> WireResult<String> {
        let len = self.get_u32()? as usize;
        // Guard against a corrupt length asking for more than the buffer holds.
        let bytes = self.take(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| WireError::InvalidUtf8)
    }

    fn get_bytes(&mut self) -> WireResult<Vec<u8>> {
        let len = self.get_u32()? as usize;
        Ok(self.take(len)?.to_vec())
    }
}

// ===========================================================================
// The Encode/Decode traits and the entry points.
// ===========================================================================

/// Manual, dependency-free serialization to bytes. Append-style so composite
/// types can encode their fields into a shared buffer without intermediate
/// allocations.
pub trait Encode {
    /// Append this value's wire bytes to `buf`.
    fn encode_into(&self, buf: &mut Vec<u8>);

    /// Encode this value into a fresh `Vec<u8>`.
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encode_into(&mut buf);
        buf
    }
}

/// Manual, dependency-free deserialization from bytes.
pub trait Decode: Sized {
    /// Decode a value, consuming exactly the bytes it needs from `cur`.
    fn decode_from(cur: &mut Cursor<'_>) -> WireResult<Self>;

    /// Decode a value from a complete byte slice, requiring that the entire
    /// slice is consumed (no trailing garbage).
    fn decode(bytes: &[u8]) -> WireResult<Self> {
        let mut cur = Cursor::new(bytes);
        let value = Self::decode_from(&mut cur)?;
        if cur.remaining() != 0 {
            return Err(WireError::TrailingBytes {
                remaining: cur.remaining(),
            });
        }
        Ok(value)
    }
}

/// Encode a message and write it as exactly one length-prefixed frame. This is
/// the primary send path for both server and client.
pub fn write_message<W: Write, M: Encode>(w: &mut W, msg: &M) -> WireResult<()> {
    let payload = msg.encode();
    write_frame(w, &payload)
}

/// Read exactly one length-prefixed frame and decode it into a message. This is
/// the primary receive path for both server and client.
pub fn read_message<R: Read, M: Decode>(r: &mut R) -> WireResult<M> {
    let payload = read_frame(r)?;
    M::decode(&payload)
}

/// Like [`read_message`], but returns `Ok(None)` on a clean EOF at a frame
/// boundary (e.g. the peer closed the connection between requests).
pub fn read_message_opt<R: Read, M: Decode>(r: &mut R) -> WireResult<Option<M>> {
    match read_frame_opt(r)? {
        Some(payload) => Ok(Some(M::decode(&payload)?)),
        None => Ok(None),
    }
}

// ===========================================================================
// Reboot mode (mirrors machine::RebootMode but kept wire-local so the codec is
// self-contained and round-trips independently).
// ===========================================================================

/// The reboot mode carried by a [`Request::Reboot`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireRebootMode {
    /// Normal full reboot via firmware.
    Default,
    /// In-place kexec into the new kernel without a firmware cycle.
    Powercycle,
}

impl WireRebootMode {
    fn tag(self) -> u8 {
        match self {
            WireRebootMode::Default => 0,
            WireRebootMode::Powercycle => 1,
        }
    }

    fn from_tag(tag: u8) -> WireResult<Self> {
        match tag {
            0 => Ok(WireRebootMode::Default),
            1 => Ok(WireRebootMode::Powercycle),
            other => Err(WireError::UnknownTag {
                context: "reboot mode",
                tag: other,
            }),
        }
    }
}

// ===========================================================================
// The Request model.
// ===========================================================================

/// A request from a client to the machine API server.
///
/// The first byte of the encoded form is a discriminant tag; the remaining
/// bytes are the variant's payload (empty for unit-like variants).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Ask for the node's version info.
    Version,
    /// Ask for the node's hostname.
    Hostname,
    /// List the running services.
    ServiceList,
    /// Start a named service.
    ServiceStart {
        /// The service id to start (e.g. `etcd`).
        name: String,
    },
    /// Stream/dump the kernel ring buffer.
    Dmesg,
    /// Reboot the node in the given mode.
    Reboot {
        /// The reboot mode.
        mode: WireRebootMode,
    },
}

// Discriminant tags for Request. Kept explicit & stable for wire compatibility.
const REQ_VERSION: u8 = 1;
const REQ_HOSTNAME: u8 = 2;
const REQ_SERVICE_LIST: u8 = 3;
const REQ_SERVICE_START: u8 = 4;
const REQ_DMESG: u8 = 5;
const REQ_REBOOT: u8 = 6;

impl Encode for Request {
    fn encode_into(&self, buf: &mut Vec<u8>) {
        match self {
            Request::Version => put_u8(buf, REQ_VERSION),
            Request::Hostname => put_u8(buf, REQ_HOSTNAME),
            Request::ServiceList => put_u8(buf, REQ_SERVICE_LIST),
            Request::ServiceStart { name } => {
                put_u8(buf, REQ_SERVICE_START);
                put_str(buf, name);
            }
            Request::Dmesg => put_u8(buf, REQ_DMESG),
            Request::Reboot { mode } => {
                put_u8(buf, REQ_REBOOT);
                put_u8(buf, mode.tag());
            }
        }
    }
}

impl Decode for Request {
    fn decode_from(cur: &mut Cursor<'_>) -> WireResult<Self> {
        let tag = cur.get_u8()?;
        match tag {
            REQ_VERSION => Ok(Request::Version),
            REQ_HOSTNAME => Ok(Request::Hostname),
            REQ_SERVICE_LIST => Ok(Request::ServiceList),
            REQ_SERVICE_START => Ok(Request::ServiceStart {
                name: cur.get_str()?,
            }),
            REQ_DMESG => Ok(Request::Dmesg),
            REQ_REBOOT => Ok(Request::Reboot {
                mode: WireRebootMode::from_tag(cur.get_u8()?)?,
            }),
            other => Err(WireError::UnknownTag {
                context: "request",
                tag: other,
            }),
        }
    }
}

// ===========================================================================
// Response payload sub-types.
// ===========================================================================

/// The version info returned for a [`Request::Version`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VersionReply {
    /// The Talos version tag (e.g. `v1.8.0`).
    pub tag: String,
    /// The build commit SHA.
    pub sha: String,
    /// The OS/architecture (e.g. `amd64`).
    pub arch: String,
}

impl Encode for VersionReply {
    fn encode_into(&self, buf: &mut Vec<u8>) {
        put_str(buf, &self.tag);
        put_str(buf, &self.sha);
        put_str(buf, &self.arch);
    }
}

impl Decode for VersionReply {
    fn decode_from(cur: &mut Cursor<'_>) -> WireResult<Self> {
        Ok(VersionReply {
            tag: cur.get_str()?,
            sha: cur.get_str()?,
            arch: cur.get_str()?,
        })
    }
}

/// A single service's runtime state, returned in a [`Response::ServiceList`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceEntry {
    /// The service id (e.g. `apid`, `etcd`).
    pub id: String,
    /// The lifecycle state string (e.g. `Running`).
    pub state: String,
    /// Whether the health check passes (`None` if unknown).
    pub healthy: Option<bool>,
}

impl Encode for ServiceEntry {
    fn encode_into(&self, buf: &mut Vec<u8>) {
        put_str(buf, &self.id);
        put_str(buf, &self.state);
        // Optionality is encoded as a presence byte followed by the value.
        match self.healthy {
            None => put_u8(buf, 0),
            Some(v) => {
                put_u8(buf, 1);
                put_bool(buf, v);
            }
        }
    }
}

impl Decode for ServiceEntry {
    fn decode_from(cur: &mut Cursor<'_>) -> WireResult<Self> {
        let id = cur.get_str()?;
        let state = cur.get_str()?;
        let healthy = match cur.get_u8()? {
            0 => None,
            1 => Some(cur.get_bool()?),
            other => {
                return Err(WireError::UnknownTag {
                    context: "optional bool",
                    tag: other,
                });
            }
        };
        Ok(ServiceEntry { id, state, healthy })
    }
}

/// A wire-local error response, carrying a numeric gRPC-style code and message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireErrorReply {
    /// The numeric status code (mirrors `common::Code::as_i32`).
    pub code: i32,
    /// A human-readable message.
    pub message: String,
}

impl Encode for WireErrorReply {
    fn encode_into(&self, buf: &mut Vec<u8>) {
        // i32 transported as u32 two's-complement bits, big-endian.
        put_u32(buf, self.code as u32);
        put_str(buf, &self.message);
    }
}

impl Decode for WireErrorReply {
    fn decode_from(cur: &mut Cursor<'_>) -> WireResult<Self> {
        Ok(WireErrorReply {
            code: cur.get_u32()? as i32,
            message: cur.get_str()?,
        })
    }
}

// ===========================================================================
// The Response model.
// ===========================================================================

/// A response from the machine API server to a client.
///
/// Each successful variant matches a [`Request`] variant; [`Response::Error`]
/// can be returned for any request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// Reply to [`Request::Version`].
    Version(VersionReply),
    /// Reply to [`Request::Hostname`].
    Hostname {
        /// The node's hostname.
        hostname: String,
    },
    /// Reply to [`Request::ServiceList`].
    ServiceList {
        /// The services and their states.
        services: Vec<ServiceEntry>,
    },
    /// Reply to [`Request::ServiceStart`].
    ServiceStart {
        /// A human-readable result message (e.g. `Service "etcd" started`).
        resp: String,
    },
    /// Reply to [`Request::Dmesg`]: the kernel ring-buffer bytes.
    Dmesg {
        /// The raw dmesg bytes.
        data: Vec<u8>,
    },
    /// Reply to [`Request::Reboot`]: acknowledged before the node goes down.
    Reboot {
        /// Monotonic-ish ack id (e.g. an actor/sequence id), 0 if unused.
        ack: u64,
    },
    /// An error response, valid for any request.
    Error(WireErrorReply),
}

// Discriminant tags for Response.
const RESP_VERSION: u8 = 1;
const RESP_HOSTNAME: u8 = 2;
const RESP_SERVICE_LIST: u8 = 3;
const RESP_SERVICE_START: u8 = 4;
const RESP_DMESG: u8 = 5;
const RESP_REBOOT: u8 = 6;
const RESP_ERROR: u8 = 7;

impl Encode for Response {
    fn encode_into(&self, buf: &mut Vec<u8>) {
        match self {
            Response::Version(v) => {
                put_u8(buf, RESP_VERSION);
                v.encode_into(buf);
            }
            Response::Hostname { hostname } => {
                put_u8(buf, RESP_HOSTNAME);
                put_str(buf, hostname);
            }
            Response::ServiceList { services } => {
                put_u8(buf, RESP_SERVICE_LIST);
                put_u32(buf, services.len() as u32);
                for svc in services {
                    svc.encode_into(buf);
                }
            }
            Response::ServiceStart { resp } => {
                put_u8(buf, RESP_SERVICE_START);
                put_str(buf, resp);
            }
            Response::Dmesg { data } => {
                put_u8(buf, RESP_DMESG);
                put_bytes(buf, data);
            }
            Response::Reboot { ack } => {
                put_u8(buf, RESP_REBOOT);
                put_u64(buf, *ack);
            }
            Response::Error(e) => {
                put_u8(buf, RESP_ERROR);
                e.encode_into(buf);
            }
        }
    }
}

impl Decode for Response {
    fn decode_from(cur: &mut Cursor<'_>) -> WireResult<Self> {
        let tag = cur.get_u8()?;
        match tag {
            RESP_VERSION => Ok(Response::Version(VersionReply::decode_from(cur)?)),
            RESP_HOSTNAME => Ok(Response::Hostname {
                hostname: cur.get_str()?,
            }),
            RESP_SERVICE_LIST => {
                let count = cur.get_u32()? as usize;
                let mut services = Vec::with_capacity(count.min(1024));
                for _ in 0..count {
                    services.push(ServiceEntry::decode_from(cur)?);
                }
                Ok(Response::ServiceList { services })
            }
            RESP_SERVICE_START => Ok(Response::ServiceStart {
                resp: cur.get_str()?,
            }),
            RESP_DMESG => Ok(Response::Dmesg {
                data: cur.get_bytes()?,
            }),
            RESP_REBOOT => Ok(Response::Reboot {
                ack: cur.get_u64()?,
            }),
            RESP_ERROR => Ok(Response::Error(WireErrorReply::decode_from(cur)?)),
            other => Err(WireError::UnknownTag {
                context: "response",
                tag: other,
            }),
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor as IoCursor;

    // ---- Framing -----------------------------------------------------------

    #[test]
    fn frame_round_trip_empty_and_nonempty() {
        for payload in [vec![], b"hello".to_vec(), vec![0u8; 1000]] {
            let mut buf = Vec::new();
            write_frame(&mut buf, &payload).unwrap();
            // 4-byte header + payload.
            assert_eq!(buf.len(), 4 + payload.len());
            let mut r = IoCursor::new(buf);
            let got = read_frame(&mut r).unwrap();
            assert_eq!(got, payload);
        }
    }

    #[test]
    fn frame_header_is_big_endian() {
        let mut buf = Vec::new();
        write_frame(&mut buf, &[0xaa, 0xbb]).unwrap();
        assert_eq!(&buf[0..4], &[0x00, 0x00, 0x00, 0x02]);
        assert_eq!(&buf[4..], &[0xaa, 0xbb]);
    }

    #[test]
    fn multiple_frames_back_to_back() {
        let mut buf = Vec::new();
        write_frame(&mut buf, b"one").unwrap();
        write_frame(&mut buf, b"two").unwrap();
        write_frame(&mut buf, b"three").unwrap();
        let mut r = IoCursor::new(buf);
        assert_eq!(read_frame(&mut r).unwrap(), b"one");
        assert_eq!(read_frame(&mut r).unwrap(), b"two");
        assert_eq!(read_frame(&mut r).unwrap(), b"three");
        // Clean EOF afterwards.
        assert!(read_frame_opt(&mut r).unwrap().is_none());
    }

    #[test]
    fn clean_eof_on_boundary_is_none() {
        let mut r = IoCursor::new(Vec::<u8>::new());
        assert!(read_frame_opt(&mut r).unwrap().is_none());
        // read_frame (non-opt) reports it as an error.
        let mut r2 = IoCursor::new(Vec::<u8>::new());
        assert!(matches!(read_frame(&mut r2), Err(WireError::UnexpectedEof)));
    }

    #[test]
    fn truncated_header_is_error() {
        // Only 2 bytes of the 4-byte header.
        let mut r = IoCursor::new(vec![0x00, 0x00]);
        assert!(matches!(read_frame(&mut r), Err(WireError::UnexpectedEof)));
    }

    #[test]
    fn truncated_payload_is_error() {
        // Header says 10 bytes, only 3 provided.
        let mut data = 10u32.to_be_bytes().to_vec();
        data.extend_from_slice(b"abc");
        let mut r = IoCursor::new(data);
        assert!(matches!(read_frame(&mut r), Err(WireError::UnexpectedEof)));
    }

    #[test]
    fn oversized_frame_is_rejected_on_read() {
        let big = (MAX_FRAME_LEN as u32 + 1).to_be_bytes().to_vec();
        let mut r = IoCursor::new(big);
        match read_frame(&mut r) {
            Err(WireError::FrameTooLarge { len }) => assert_eq!(len, MAX_FRAME_LEN + 1),
            other => panic!("expected FrameTooLarge, got {other:?}"),
        }
    }

    #[test]
    fn oversized_frame_is_rejected_on_write() {
        // Build a fake oversized payload without actually allocating 16MiB twice:
        // a 16MiB+1 vec is acceptable for a test but we keep it minimal by
        // checking the boundary with exactly MAX+1 via a thin wrapper.
        let payload = vec![0u8; MAX_FRAME_LEN + 1];
        let mut buf = Vec::new();
        match write_frame(&mut buf, &payload) {
            Err(WireError::FrameTooLarge { len }) => assert_eq!(len, MAX_FRAME_LEN + 1),
            other => panic!("expected FrameTooLarge, got {other:?}"),
        }
        assert!(buf.is_empty());
    }

    /// A reader that yields its data one byte at a time to exercise the
    /// partial-read loops in `read_frame`.
    struct DripReader {
        data: Vec<u8>,
        pos: usize,
    }

    impl Read for DripReader {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.pos >= self.data.len() || buf.is_empty() {
                return Ok(0);
            }
            buf[0] = self.data[self.pos];
            self.pos += 1;
            Ok(1)
        }
    }

    #[test]
    fn partial_reads_are_handled() {
        let mut buf = Vec::new();
        write_frame(&mut buf, b"a partial-read payload of some length").unwrap();
        let mut r = DripReader { data: buf, pos: 0 };
        let got = read_frame(&mut r).unwrap();
        assert_eq!(got, b"a partial-read payload of some length");
    }

    #[test]
    fn partial_reads_multiple_messages() {
        let mut buf = Vec::new();
        write_message(&mut buf, &Request::Version).unwrap();
        write_message(
            &mut buf,
            &Request::ServiceStart {
                name: "etcd".into(),
            },
        )
        .unwrap();
        let mut r = DripReader { data: buf, pos: 0 };
        assert_eq!(
            read_message::<_, Request>(&mut r).unwrap(),
            Request::Version
        );
        assert_eq!(
            read_message::<_, Request>(&mut r).unwrap(),
            Request::ServiceStart {
                name: "etcd".into()
            }
        );
    }

    // ---- Request round-trips ----------------------------------------------

    fn req_round_trip(req: Request) {
        let bytes = req.encode();
        let decoded = Request::decode(&bytes).unwrap();
        assert_eq!(decoded, req, "request did not round-trip");
        // Also through the full frame path.
        let mut buf = Vec::new();
        write_message(&mut buf, &req).unwrap();
        let mut r = IoCursor::new(buf);
        let via_frame: Request = read_message(&mut r).unwrap();
        assert_eq!(via_frame, req);
    }

    #[test]
    fn every_request_round_trips() {
        req_round_trip(Request::Version);
        req_round_trip(Request::Hostname);
        req_round_trip(Request::ServiceList);
        req_round_trip(Request::ServiceStart {
            name: String::new(),
        });
        req_round_trip(Request::ServiceStart {
            name: "etcd".into(),
        });
        req_round_trip(Request::ServiceStart {
            name: "a name with spaces/and-symbols".into(),
        });
        req_round_trip(Request::Dmesg);
        req_round_trip(Request::Reboot {
            mode: WireRebootMode::Default,
        });
        req_round_trip(Request::Reboot {
            mode: WireRebootMode::Powercycle,
        });
    }

    // ---- Response round-trips ----------------------------------------------

    fn resp_round_trip(resp: Response) {
        let bytes = resp.encode();
        let decoded = Response::decode(&bytes).unwrap();
        assert_eq!(decoded, resp, "response did not round-trip");
        let mut buf = Vec::new();
        write_message(&mut buf, &resp).unwrap();
        let mut r = IoCursor::new(buf);
        let via_frame: Response = read_message(&mut r).unwrap();
        assert_eq!(via_frame, resp);
    }

    #[test]
    fn every_response_round_trips() {
        resp_round_trip(Response::Version(VersionReply {
            tag: "v1.8.0".into(),
            sha: "deadbeef".into(),
            arch: "amd64".into(),
        }));
        resp_round_trip(Response::Hostname {
            hostname: "node-1".into(),
        });
        resp_round_trip(Response::ServiceList { services: vec![] });
        resp_round_trip(Response::ServiceList {
            services: vec![
                ServiceEntry {
                    id: "apid".into(),
                    state: "Running".into(),
                    healthy: Some(true),
                },
                ServiceEntry {
                    id: "etcd".into(),
                    state: "Waiting".into(),
                    healthy: Some(false),
                },
                ServiceEntry {
                    id: "kubelet".into(),
                    state: "Initialized".into(),
                    healthy: None,
                },
            ],
        });
        resp_round_trip(Response::ServiceStart {
            resp: "Service \"etcd\" started".into(),
        });
        resp_round_trip(Response::Dmesg { data: vec![] });
        resp_round_trip(Response::Dmesg {
            data: (0u8..=255).collect(),
        });
        resp_round_trip(Response::Reboot { ack: 0 });
        resp_round_trip(Response::Reboot { ack: u64::MAX });
        resp_round_trip(Response::Error(WireErrorReply {
            code: 5,
            message: "no logs for system/ghost".into(),
        }));
        resp_round_trip(Response::Error(WireErrorReply {
            code: -1,
            message: String::new(),
        }));
    }

    #[test]
    fn unicode_strings_round_trip() {
        resp_round_trip(Response::Hostname {
            hostname: "nöde-ünïcode-🚀".into(),
        });
        req_round_trip(Request::ServiceStart {
            name: "服务-名".into(),
        });
    }

    // ---- Decode error paths ------------------------------------------------

    #[test]
    fn unknown_request_tag_is_error() {
        match Request::decode(&[0xff]) {
            Err(WireError::UnknownTag {
                context: "request",
                tag: 0xff,
            }) => {}
            other => panic!("expected UnknownTag, got {other:?}"),
        }
    }

    #[test]
    fn unknown_response_tag_is_error() {
        assert!(matches!(
            Response::decode(&[0x00]),
            Err(WireError::UnknownTag {
                context: "response",
                ..
            })
        ));
    }

    #[test]
    fn empty_payload_is_unexpected_eof() {
        assert!(matches!(
            Request::decode(&[]),
            Err(WireError::UnexpectedEof)
        ));
        assert!(matches!(
            Response::decode(&[]),
            Err(WireError::UnexpectedEof)
        ));
    }

    #[test]
    fn truncated_string_length_is_eof() {
        // ServiceStart tag, then a length prefix of 100 but no string bytes.
        let mut bytes = vec![REQ_SERVICE_START];
        bytes.extend_from_slice(&100u32.to_be_bytes());
        assert!(matches!(
            Request::decode(&bytes),
            Err(WireError::UnexpectedEof)
        ));
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let mut bytes = Request::Version.encode();
        bytes.push(0x99); // garbage after a complete Version request.
        assert!(matches!(
            Request::decode(&bytes),
            Err(WireError::TrailingBytes { remaining: 1 })
        ));
    }

    #[test]
    fn invalid_utf8_in_string_is_error() {
        // Hostname response with a 1-byte invalid-UTF8 string.
        let mut bytes = vec![RESP_HOSTNAME];
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.push(0xff); // not valid UTF-8.
        assert!(matches!(
            Response::decode(&bytes),
            Err(WireError::InvalidUtf8)
        ));
    }

    #[test]
    fn bad_reboot_mode_tag_is_error() {
        let bytes = vec![REQ_REBOOT, 9];
        assert!(matches!(
            Request::decode(&bytes),
            Err(WireError::UnknownTag {
                context: "reboot mode",
                tag: 9
            })
        ));
    }

    #[test]
    fn bad_optional_bool_tag_is_error() {
        // ServiceList with one entry whose healthy presence byte is bogus.
        let mut bytes = vec![RESP_SERVICE_LIST];
        bytes.extend_from_slice(&1u32.to_be_bytes()); // 1 entry
        put_str(&mut bytes, "id");
        put_str(&mut bytes, "Running");
        bytes.push(7); // invalid presence tag
        assert!(matches!(
            Response::decode(&bytes),
            Err(WireError::UnknownTag {
                context: "optional bool",
                ..
            })
        ));
    }

    #[test]
    fn garbage_frame_payload_decodes_to_error() {
        // A valid frame whose payload is garbage for the message type.
        let mut buf = Vec::new();
        write_frame(&mut buf, &[0xde, 0xad, 0xbe, 0xef]).unwrap();
        let mut r = IoCursor::new(buf);
        let res: WireResult<Request> = read_message(&mut r);
        assert!(res.is_err());
    }

    // ---- End-to-end client/server simulation -------------------------------

    #[test]
    fn simulated_request_response_exchange() {
        // Client encodes a request into a pipe; "server" reads it, builds a
        // response, writes it back; client reads the response.
        let mut to_server = Vec::new();
        write_message(&mut to_server, &Request::ServiceList).unwrap();

        let mut server_in = IoCursor::new(to_server);
        let req: Request = read_message(&mut server_in).unwrap();
        assert_eq!(req, Request::ServiceList);

        let resp = match req {
            Request::ServiceList => Response::ServiceList {
                services: vec![ServiceEntry {
                    id: "apid".into(),
                    state: "Running".into(),
                    healthy: Some(true),
                }],
            },
            _ => Response::Error(WireErrorReply {
                code: 12,
                message: "unimplemented".into(),
            }),
        };

        let mut to_client = Vec::new();
        write_message(&mut to_client, &resp).unwrap();
        let mut client_in = IoCursor::new(to_client);
        let got: Response = read_message(&mut client_in).unwrap();
        assert_eq!(got, resp);
    }

    #[test]
    fn read_message_opt_clean_eof() {
        let mut r = IoCursor::new(Vec::<u8>::new());
        let got: Option<Request> = read_message_opt(&mut r).unwrap();
        assert!(got.is_none());
    }
}
