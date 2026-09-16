use super::*;

pub(super) async fn refuse<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    tag: &str,
    status: &str,
    mut literal: Literal,
    utf8_enabled: bool,
) -> io::Result<Option<Append>> {
    write(
        stream.get_mut(),
        format!("{tag} {status} APPEND refused\r\n").as_bytes(),
    )
    .await?;
    let mut remaining = BUFFER_LIMIT;
    // Only already-validated, bounded non-sync frames can be discarded. Never
    // request another body after refusal or reinterpret its bytes as commands.
    while literal.nonsync {
        remaining = remaining
            .checked_sub(literal.size + 1)
            .ok_or_else(invalid)?;
        let copied = tokio::io::copy(
            &mut (&mut *stream).take(literal.size as u64),
            &mut tokio::io::sink(),
        )
        .await?;
        if copied != literal.size as u64 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "truncated refused APPEND",
            ));
        }
        let suffix = line(stream, 8192).await?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "missing refused APPEND suffix",
            )
        })?;
        remaining = remaining
            .checked_sub(suffix.len() + 2)
            .ok_or_else(invalid)?;
        let suffix = if literal.utf8 {
            suffix.strip_prefix(b")").ok_or_else(invalid)?
        } else {
            &suffix
        };
        if suffix.is_empty() {
            return Ok(None);
        }
        literal = Literal::next(suffix, utf8_enabled).map_err(|_| invalid())?;
    }
    Ok(None)
}

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid refused APPEND framing")
}
