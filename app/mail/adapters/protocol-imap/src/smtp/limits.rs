//! Per-session parameters and the metered reader: a command line that
//! stalls past the buffer is discarded to its CRLF and refused, every byte
//! read (command lines here, the body through `data::read`) is charged
//! against the session's transfer quota, and a session open past its
//! lifetime or idle past its timeout is closed with the reference's codes.
//!
//! "Stalled" is decided by whether more bytes are already waiting when the
//! buffer fills without a CRLF, which is the reference receiver's own
//! per-read semantics. On a socket that makes the outcome depend on
//! segmentation, as it does there; the `limits` oracle pins the behaviour by
//! sending the same 4097-byte line both ways.
use std::{
    io,
    net::{IpAddr, Ipv4Addr},
    time::{Duration, Instant},
};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

/// Longest command line the server buffers while waiting for its CRLF; a
/// line whose CRLF has already arrived parses whatever its length (an
/// unknown verb is then an invalid command), bounded by `HARD_LINE_BYTES`.
pub const MAX_LINE_BYTES: usize = 4096;
const HARD_LINE_BYTES: usize = 64 * 1024;

/// What the facade knows about the connection and the policy it applies;
/// a harness sets these per session.
#[derive(Clone, Debug)]
pub struct SmtpParams {
    pub peer: IpAddr,
    pub hostname: String,
    /// Close with `221 2.0.0` after this long without a command.
    pub idle_timeout: Duration,
    /// Close with `421 4.3.2` once the session has been open this long.
    pub max_duration: Duration,
    /// Close with `452 4.7.28` once the client has sent this many bytes.
    pub transfer_bytes: usize,
    /// The largest message this session accepts: advertised as `SIZE`, and
    /// enforced in both places that can refuse an oversized message — the
    /// `SIZE=` parameter on MAIL FROM, and the DATA reader.
    ///
    /// The store keeps its own ceiling at `MAX_MESSAGE_BYTES`. Nothing
    /// diverges while this defaults to that constant and no setting raises
    /// it; the moment one does, a session could advertise and accept a
    /// message the store then refuses, so the store's ceiling has to move
    /// with it.
    pub max_message_size: usize,
    /// What this session checks about its peer, and how far a failure goes.
    /// Default is every check disabled, so a deployment turns them on
    /// deliberately rather than discovering them by losing mail.
    pub authentication: super::verify::Authentication,
}
impl Default for SmtpParams {
    fn default() -> Self {
        Self {
            peer: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            hostname: "localhost".into(),
            idle_timeout: Duration::from_secs(30 * 60),
            max_duration: Duration::from_secs(60 * 60),
            transfer_bytes: 256 * 1024 * 1024,
            max_message_size: mail_kernel::MAX_MESSAGE_BYTES,
            authentication: super::verify::Authentication::default(),
        }
    }
}

pub(super) struct Meter {
    bytes_left: usize,
    valid_until: Instant,
    idle: Duration,
}
impl Meter {
    pub fn new(params: &SmtpParams) -> Self {
        Self {
            bytes_left: params.transfer_bytes,
            valid_until: Instant::now() + params.max_duration,
            idle: params.idle_timeout,
        }
    }
    /// Charge `bytes` read; the quota is checked before the lifetime, as
    /// the reference implementation orders them.
    pub fn charge(&mut self, bytes: usize) -> Option<Read> {
        if bytes > self.bytes_left {
            return Some(Read::Quota);
        }
        self.bytes_left -= bytes;
        (Instant::now() >= self.valid_until).then_some(Read::Loiter)
    }
}

#[derive(Debug)]
pub(super) enum Read {
    Line(Vec<u8>),
    /// A line exceeded `MAX_LINE_BYTES`; it was discarded to its CRLF.
    TooLong,
    Eof,
    Quota,
    Loiter,
    Idle,
}

/// One command line without its CRLF, or the reason none was read.
pub(super) async fn line<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    meter: &mut Meter,
) -> io::Result<Read> {
    let mut bytes = Vec::new();
    let mut discarding = false;
    loop {
        let read = tokio::time::timeout(
            meter.idle,
            reader
                .take((MAX_LINE_BYTES + 1) as u64)
                .read_until(b'\n', &mut bytes),
        )
        .await;
        let n = match read {
            Ok(n) => n?,
            Err(_) => return Ok(Read::Idle),
        };
        if n == 0 {
            return Ok(Read::Eof);
        }
        if let Some(stop) = meter.charge(n) {
            return Ok(stop);
        }
        // The newline ends this line either way: refusing it here keeps the
        // next command in lockstep, as the reference's receiver does.
        if bytes.ends_with(b"\n") {
            if discarding || bytes.len() > HARD_LINE_BYTES {
                return Ok(Read::TooLong);
            }
            let end = if bytes.ends_with(b"\r\n") { 2 } else { 1 };
            bytes.truncate(bytes.len() - end);
            return Ok(Read::Line(bytes));
        }
        if bytes.len() > HARD_LINE_BYTES {
            discarding = true;
        }
        // At the buffer without a CRLF: too long only if the peer has
        // stalled; data already waiting is the rest of this line.
        if bytes.len() >= MAX_LINE_BYTES && !discarding {
            let pending = tokio::time::timeout(Duration::ZERO, reader.fill_buf()).await;
            if !matches!(pending, Ok(Ok(more)) if !more.is_empty()) {
                discarding = true;
            }
        }
        if discarding {
            bytes.clear();
        }
    }
}

/// What to send instead of a command, and whether the session ends.
pub(super) struct Refusal {
    pub reply: String,
    pub close: bool,
}

/// One command line, or the refusal its absence calls for.
pub(super) async fn command<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    meter: &mut Meter,
    host: &str,
) -> io::Result<Result<Vec<u8>, Refusal>> {
    let refusal = |reply: String, close: bool| Ok(Err(Refusal { reply, close }));
    match line(reader, meter).await? {
        Read::Line(bytes) => Ok(Ok(bytes)),
        Read::TooLong => refusal("554 5.3.4 Line is too long.\r\n".into(), false),
        Read::Eof => refusal(String::new(), true),
        Read::Quota => refusal(
            format!("452 4.7.28 {host} Session exceeded transfer quota.\r\n"),
            true,
        ),
        Read::Loiter => refusal(
            format!("421 4.3.2 {host} Session open for too long.\r\n"),
            true,
        ),
        Read::Idle => refusal(
            format!("221 2.0.0 {host} Disconnecting inactive client.\r\n"),
            true,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn a_stalled_overlong_line_is_discarded_to_its_crlf_but_a_complete_one_parses() {
        let params = SmtpParams::default();
        let mut meter = Meter::new(&params);
        let (mut client, server) = tokio::io::duplex(65536);
        let mut input = tokio::io::BufReader::new(server);
        // The peer stalls past the buffer: too long, then the tail is skipped.
        client
            .write_all(&vec![b'A'; MAX_LINE_BYTES + 1])
            .await
            .unwrap();
        let read = tokio::spawn(async move {
            let first = line(&mut input, &mut meter).await.unwrap();
            let second = line(&mut input, &mut meter).await.unwrap();
            let third = line(&mut input, &mut meter).await.unwrap();
            (
                matches!(first, Read::TooLong),
                matches!(second, Read::Line(l) if l == b"NOOP"),
                third,
                input,
                meter,
            )
        });
        tokio::time::sleep(Duration::from_millis(100)).await;
        // A complete overlong line, already waiting, is a line.
        client
            .write_all(
                &[
                    b"tail\r\nNOOP\r\n".to_vec(),
                    vec![b'B'; MAX_LINE_BYTES + 1],
                    b"\r\n".to_vec(),
                ]
                .concat(),
            )
            .await
            .unwrap();
        let (too_long, noop, third, _input, _meter) = read.await.unwrap();
        assert!(too_long && noop);
        assert!(matches!(third, Read::Line(l) if l.len() == MAX_LINE_BYTES + 1));
        let mut meter = Meter::new(&SmtpParams {
            transfer_bytes: 4,
            ..SmtpParams::default()
        });
        let mut input = tokio::io::BufReader::new(std::io::Cursor::new(b"NOOP\r\n".to_vec()));
        assert!(matches!(
            line(&mut input, &mut meter).await.unwrap(),
            Read::Quota
        ));
    }
}
