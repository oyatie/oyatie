use crate::wire::line;
use mail_kernel::MAX_MESSAGE_BYTES;
use std::{io, time::Duration};
use tokio::{io::AsyncBufRead, time::Instant};

pub(super) async fn read<R: AsyncBufRead + Unpin>(reader: &mut R) -> io::Result<Vec<u8>> {
    bounded(reader, Instant::now() + Duration::from_secs(300)).await
}

async fn bounded<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    deadline: Instant,
) -> io::Result<Vec<u8>> {
    tokio::time::timeout_at(deadline, async {
        let mut raw = Vec::new();
        loop {
            // RFC 5321's 1000-octet line limit excludes the transparency dot.
            let data = line(reader, 1001).await?.ok_or_else(|| {
                io::Error::new(io::ErrorKind::UnexpectedEof, "incomplete SMTP DATA")
            })?;
            if data == b"." {
                return Ok(raw);
            }
            let data = data.strip_prefix(b".").unwrap_or(&data);
            if data.len() > 998 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "SMTP data line too long",
                ));
            }
            if raw.len().saturating_add(data.len()).saturating_add(2) > MAX_MESSAGE_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::FileTooLarge,
                    "SMTP message too large",
                ));
            }
            raw.extend_from_slice(data);
            raw.extend_from_slice(b"\r\n");
        }
    })
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "SMTP DATA deadline"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncWriteExt, BufReader};

    #[tokio::test]
    async fn data_deadline_is_not_extended_by_active_input() {
        let (mut writer, reader) = tokio::io::duplex(4096);
        let task = tokio::spawn(async move {
            loop {
                if writer.write_all(b"continuing\r\n").await.is_err() {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });
        let result = bounded(
            &mut BufReader::new(reader),
            Instant::now() + Duration::from_millis(20),
        )
        .await;
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::TimedOut);
        task.abort();
        let _ = task.await;
    }
}
