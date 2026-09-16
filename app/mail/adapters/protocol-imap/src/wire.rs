use std::{io, time::Duration};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn write<W: AsyncWrite + Unpin>(writer: &mut W, bytes: &[u8]) -> io::Result<()> {
    tokio::time::timeout(Duration::from_secs(60), writer.write_all(bytes))
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "mail response stalled"))?
}

pub async fn line<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    limit: usize,
) -> io::Result<Option<Vec<u8>>> {
    let mut bytes = Vec::new();
    let n = tokio::time::timeout(
        Duration::from_secs(60),
        reader
            .take((limit + 1) as u64)
            .read_until(b'\n', &mut bytes),
    )
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "mail session idle"))??;
    if n == 0 {
        return Ok(None);
    }
    if bytes.len() > limit || !bytes.ends_with(b"\r\n") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid mail framing",
        ));
    }
    bytes.truncate(bytes.len() - 2);
    Ok(Some(bytes))
}

pub fn atoms(value: &str) -> Option<Vec<String>> {
    let mut result = vec![];
    let mut chars = value.chars().peekable();
    while chars.peek().is_some() {
        while chars.peek() == Some(&' ') {
            chars.next();
        }
        if chars.peek().is_none() {
            break;
        }
        let mut token = String::new();
        if chars.peek() == Some(&'"') {
            chars.next();
            loop {
                match chars.next()? {
                    '"' => break,
                    '\\' => {
                        let c = chars.next()?;
                        if c != '"' && c != '\\' {
                            return None;
                        }
                        token.push(c);
                    }
                    c if !c.is_control() => token.push(c),
                    _ => return None,
                }
            }
            if chars.peek().is_some_and(|c| *c != ' ') {
                return None;
            }
        } else {
            while let Some(c) = chars.peek().copied() {
                if c == ' ' {
                    break;
                }
                if c.is_control() || c == '"' {
                    return None;
                }
                token.push(c);
                chars.next();
            }
        }
        result.push(token);
    }
    Some(result)
}
