use mail_api::DeliveryOutcome;
use smtp_proto::{Response, response::parser::ResponseReceiver};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};

pub(super) async fn reply<S: AsyncRead + Unpin>(
    stream: &mut BufReader<S>,
) -> Result<Response<String>, DeliveryOutcome> {
    tokio::time::timeout(Duration::from_secs(300), parse_reply(stream))
        .await
        .map_err(|_| temporary())?
}

async fn parse_reply<S: AsyncRead + Unpin>(
    stream: &mut BufReader<S>,
) -> Result<Response<String>, DeliveryOutcome> {
    let mut parser = ResponseReceiver::default();
    let mut first = None;
    let mut total = 0;
    loop {
        let mut line = Vec::new();
        (&mut *stream)
            .take(513)
            .read_until(b'\n', &mut line)
            .await
            .map_err(|_| temporary())?;
        total += line.len();
        if line.len() > 512 || line.len() < 5 || !line.ends_with(b"\r\n") || total > 32768 {
            return Err(temporary());
        }
        let code = &line[..3];
        if let Some(first) = &first {
            if code != first {
                return Err(temporary());
            }
        } else {
            first = Some(code.to_vec());
        }
        let mut bytes = line.iter();
        match parser.parse(&mut bytes) {
            Ok(reply) if bytes.next().is_none() => return Ok(reply),
            Err(smtp_proto::Error::NeedsMoreData { .. }) => {}
            _ => return Err(temporary()),
        }
    }
}

pub(super) async fn command<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    command: &[u8],
) -> Result<Response<String>, DeliveryOutcome> {
    command_for(stream, command, Duration::from_secs(300)).await
}

pub(super) async fn command_for<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    command: &[u8],
    duration: Duration,
) -> Result<Response<String>, DeliveryOutcome> {
    tokio::time::timeout(duration, async {
        stream
            .get_mut()
            .write_all(command)
            .await
            .map_err(|_| temporary())?;
        stream.get_mut().flush().await.map_err(|_| temporary())?;
        parse_reply(stream).await
    })
    .await
    .map_err(|_| temporary())?
}

pub(super) fn expect(
    reply: Response<String>,
    expected: u16,
) -> Result<Response<String>, DeliveryOutcome> {
    if reply.code == expected {
        Ok(reply)
    } else if (400..500).contains(&reply.code) {
        Err(DeliveryOutcome::Temporary(reply.code))
    } else if (500..600).contains(&reply.code) {
        Err(DeliveryOutcome::Permanent(reply.code))
    } else {
        Err(temporary())
    }
}

pub(super) fn temporary() -> DeliveryOutcome {
    DeliveryOutcome::Temporary(451)
}

pub(super) fn capability(reply: &Response<String>, name: &str) -> bool {
    reply.message.lines().skip(1).any(|line| {
        line.split_ascii_whitespace()
            .next()
            .is_some_and(|word| word.eq_ignore_ascii_case(name))
    })
}

pub(super) fn message_valid(raw: &[u8]) -> bool {
    !raw.is_empty()
        && raw.len() <= mail_kernel::MAX_MESSAGE_BYTES
        && raw.ends_with(b"\r\n")
        && raw.split_inclusive(|b| *b == b'\n').all(|line| {
            line.len() <= 1000
                && line.ends_with(b"\r\n")
                && !line[..line.len() - 2].contains(&b'\r')
                && !line.contains(&0)
        })
}
