use super::{Session, syntax};
use crate::wire::{line, write};
use mail_service::MailService;
use std::{io, sync::Arc, time::Duration};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, BufReader};

const LIMIT: usize = 8192;

pub(in crate::imap) fn specifier(input: &[u8]) -> Option<(usize, bool)> {
    let number = input.strip_prefix(b"{")?.strip_suffix(b"}")?;
    let nonsync = number.ends_with(b"+");
    let number = if nonsync {
        &number[..number.len() - 1]
    } else {
        number
    };
    if number.is_empty() || !number.iter().all(u8::is_ascii_digit) {
        return None;
    }
    Some((std::str::from_utf8(number).ok()?.parse().ok()?, nonsync))
}

// Keep malformed unquoted markers too: trailing junk cannot turn an already
// transmitted non-sync payload into a command. Closed quoted strings are data.
fn marker(input: &[u8]) -> Option<(&[u8], bool)> {
    let (mut outside, mut inside) = (None, None);
    let (mut quoted, mut escaped) = (false, false);
    for (i, &b) in input.iter().enumerate() {
        if escaped {
            escaped = false;
        } else if quoted && b == b'\\' {
            escaped = true;
        } else if b == b'"' {
            quoted = !quoted;
            if !quoted {
                inside = None;
            }
        } else if b == b'{' {
            if quoted {
                inside = Some(i);
            } else {
                outside = Some(i);
            }
        }
    }
    let start = outside.or(inside)?;
    let end = start + input[start..].iter().position(|b| *b == b'}')? + 1;
    let separated = outside.is_some() && (start == 0 || b" (".contains(&input[start - 1]));
    Some((&input[start..end], separated && end == input.len()))
}

pub(super) async fn read<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    first: &[u8],
    service: Arc<MailService>,
    session: &Session,
    encrypted: bool,
) -> io::Result<Option<Vec<String>>> {
    // Frame the command before parsing strings: rejected non-sync payloads must
    // never be interpreted as commands, even with an invalid tag or UTF-8.
    let mut head = first.splitn(3, |b| *b == b' ');
    let tag = head.next().unwrap_or_default();
    let verb = head.next().unwrap_or_default();
    let args = head.next().unwrap_or_default();
    if tag.is_empty()
        || !tag
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(b))
        || verb.is_empty()
        || !verb.iter().all(u8::is_ascii_alphabetic)
    {
        reject(stream, "*").await?;
        return refuse_payload(first);
    }
    let tag = std::str::from_utf8(tag).unwrap();
    let verb = std::str::from_utf8(verb).unwrap();
    if std::str::from_utf8(args).is_err() {
        reject(stream, tag).await?;
        return refuse_payload(args);
    }
    let operation = if verb.eq_ignore_ascii_case("UID") {
        std::str::from_utf8(args)
            .unwrap()
            .split(' ')
            .next()
            .unwrap_or("")
    } else {
        verb
    }
    .to_ascii_uppercase();
    let bytes = if marker(args).is_some() {
        tokio::time::timeout(Duration::from_secs(60), async {
            let allowed = if verb.eq_ignore_ascii_case("LOGIN") {
                encrypted && session.credential.is_empty()
            } else if session.credential.is_empty() {
                false
            } else {
                let token = session.credential.clone();
                let account = session.account_id.clone();
                let action = match operation.as_str() {
                    "CREATE" | "DELETE" | "RENAME" | "SUBSCRIBE" | "UNSUBSCRIBE" | "APPEND"
                    | "COPY" | "MOVE" => mail_api::Action::Write,
                    _ => mail_api::Action::Read,
                };
                let authorized = tokio::task::spawn_blocking(move || {
                    service.authorize(&token, &account, action).is_ok()
                })
                .await
                .map_err(|_| io::Error::other("IMAP authorization worker failed"))?;
                authorized
                    && (!matches!(
                        operation.as_str(),
                        "SEARCH" | "SORT" | "THREAD" | "FETCH" | "STORE" | "COPY" | "MOVE"
                    ) || session.selected.is_some())
            };
            let string_command = matches!(
                operation.as_str(),
                "LOGIN"
                    | "CREATE"
                    | "DELETE"
                    | "RENAME"
                    | "SELECT"
                    | "EXAMINE"
                    | "LIST"
                    | "LSUB"
                    | "STATUS"
                    | "SUBSCRIBE"
                    | "UNSUBSCRIBE"
                    | "APPEND"
                    | "COPY"
                    | "MOVE"
                    | "SEARCH"
                    | "SORT"
                    | "THREAD"
                    | "FETCH"
            );
            let uid_command = !verb.eq_ignore_ascii_case("UID")
                || matches!(
                    operation.as_str(),
                    "SEARCH" | "SORT" | "THREAD" | "FETCH" | "COPY" | "MOVE"
                );
            collect(
                stream,
                tag,
                verb,
                args,
                allowed && string_command && uid_command,
            )
            .await
        })
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "IMAP literal stalled"))??
    } else {
        Some(args.to_vec())
    };
    let Some(bytes) = bytes else { return Ok(None) };
    let parts = std::str::from_utf8(&bytes)
        .ok()
        .and_then(|args| syntax::command_parts(&format!("{tag} {verb} {args}")));
    if parts.is_none() {
        reject(stream, tag).await?;
        return Ok(None);
    }
    Ok(parts)
}

pub(super) fn refuse_payload<T>(bytes: &[u8]) -> io::Result<Option<T>> {
    if marker(bytes).is_some_and(|(token, _)| token.ends_with(b"+}")) {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unconsumed IMAP literal",
        ))
    } else {
        Ok(None)
    }
}

async fn reject<S: AsyncWrite + AsyncRead + Unpin>(
    stream: &mut BufReader<S>,
    tag: &str,
) -> io::Result<()> {
    write(
        stream.get_mut(),
        format!("{tag} BAD Invalid command literal\r\n").as_bytes(),
    )
    .await
}

async fn collect<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    tag: &str,
    verb: &str,
    first: &[u8],
    allowed: bool,
) -> io::Result<Option<Vec<u8>>> {
    let mut bytes = first.to_vec();
    let mut suffix = 0;
    while let Some((token, separated)) = marker(&bytes[suffix..]) {
        // APPEND's final message literal has its own quota and binary framing.
        // Only a syntactically complete mailbox/options prefix can hand it off.
        if verb.eq_ignore_ascii_case("APPEND")
            && std::str::from_utf8(&bytes)
                .ok()
                .and_then(|args| syntax::command_parts(&format!("{tag} APPEND {args}")))
                .is_some_and(|parts| parts.len() >= 4)
        {
            return Ok(Some(bytes));
        }
        let nonsync = token.ends_with(b"+}");
        let size = specifier(token).map(|(size, _)| size).filter(|size| {
            if !allowed || !separated {
                return false;
            }
            bytes
                .len()
                .checked_add(*size)
                .and_then(|n| n.checked_add(4))
                .is_some_and(|n| n <= LIMIT)
        });
        let Some(size) = size else {
            reject(stream, tag).await?;
            return if nonsync {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unbounded IMAP literal",
                ))
            } else {
                Ok(None)
            };
        };
        if !nonsync {
            write(stream.get_mut(), b"+ Ready for literal\r\n").await?;
        }
        bytes.extend_from_slice(b"\r\n");
        let start = bytes.len();
        bytes.resize(start + size, 0);
        stream.read_exact(&mut bytes[start..]).await?;
        suffix = bytes.len();
        let tail = line(stream, LIMIT - bytes.len()).await?.ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "missing literal suffix")
        })?;
        bytes.extend_from_slice(&tail);
    }
    Ok(Some(bytes))
}
