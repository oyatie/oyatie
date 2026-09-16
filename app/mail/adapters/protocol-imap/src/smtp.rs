mod auth;
mod data;
mod tls;
use crate::wire::{line, write};
use mail_kernel::{Error, MAX_MESSAGE_BYTES, valid_address};
use mail_service::MailService;
use std::{io, sync::Arc};
pub use tls::smtp_starttls_session;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

pub async fn smtp_session<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    service: Arc<MailService>,
) -> io::Result<()> {
    session(stream, service, false, false, true)
        .await
        .map(|_| ())
}

/// The caller must establish TLS before setting `protected`. The facade supplies
/// only a completed TLS stream; plaintext invocations fail before a greeting.
pub async fn submission_session<S: AsyncRead + AsyncWrite + Unpin>(
    mut stream: S,
    service: Arc<MailService>,
    protected: bool,
) -> io::Result<()> {
    if !protected {
        return write(&mut stream, b"554 5.7.0 TLS required\r\n").await;
    }
    session(stream, service, true, false, true)
        .await
        .map(|_| ())
}

async fn session<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    service: Arc<MailService>,
    submission: bool,
    starttls: bool,
    greeting: bool,
) -> io::Result<Option<S>> {
    let mut auth = (submission && !starttls).then(auth::Submission::default);
    let mut stream = BufReader::new(stream);
    if greeting {
        write(stream.get_mut(), b"220 localhost ESMTP Oyatie\r\n").await?;
    }
    let mut greeted = false;
    let mut esmtp = false;
    let mut sender = None;
    let mut recipients = vec![];
    while let Some(bytes) = line(&mut stream, if submission { 12288 } else { 512 }).await? {
        let Ok(command) = std::str::from_utf8(&bytes) else {
            write(stream.get_mut(), b"500 5.5.2 Invalid command\r\n").await?;
            continue;
        };
        let (verb, arg) = command.split_once(' ').unwrap_or((command, ""));
        if bytes.len() > 510 && !verb.eq_ignore_ascii_case("AUTH") {
            write(stream.get_mut(), b"500 5.5.2 Command too long\r\n").await?;
            continue;
        }
        let reply = match verb.to_ascii_uppercase().as_str() {
            "STARTTLS" => {
                if !starttls || !esmtp || sender.is_some() {
                    "503 5.5.1 STARTTLS out of sequence\r\n"
                } else if !arg.is_empty() {
                    "501 5.5.2 STARTTLS takes no parameters\r\n"
                } else {
                    if !stream.buffer().is_empty() {
                        write(
                            stream.get_mut(),
                            b"554 5.5.1 Pipelined STARTTLS refused\r\n",
                        )
                        .await?;
                        return Ok(None);
                    }
                    write(stream.get_mut(), b"220 2.0.0 Begin TLS negotiation\r\n").await?;
                    return Ok(Some(stream.into_inner()));
                }
            }
            "AUTH" => match auth.as_mut() {
                None => "538 5.7.11 Authentication requires TLS submission\r\n",
                Some(_) if !esmtp => "503 5.5.1 Send EHLO first\r\n",
                Some(state) if sender.is_some() || state.credentials.is_some() => {
                    "503 5.5.1 Authentication out of sequence\r\n"
                }
                Some(state) => {
                    state
                        .authenticate(&mut stream, service.clone(), arg)
                        .await?
                }
            },
            "EHLO" | "HELO" if !arg.is_empty() && arg.bytes().all(|b| b.is_ascii_graphic()) => {
                greeted = true;
                esmtp = verb.eq_ignore_ascii_case("EHLO");
                sender = None;
                recipients.clear();
                if verb.eq_ignore_ascii_case("EHLO") {
                    if starttls {
                        "250-localhost\r\n250-SIZE 26214400\r\n250-STARTTLS\r\n250 8BITMIME\r\n"
                    } else if auth
                        .as_ref()
                        .is_some_and(|state| state.credentials.is_none())
                    {
                        "250-localhost\r\n250-SIZE 26214400\r\n250-AUTH PLAIN\r\n250 8BITMIME\r\n"
                    } else {
                        "250-localhost\r\n250-SIZE 26214400\r\n250 8BITMIME\r\n"
                    }
                } else {
                    "250 localhost\r\n"
                }
            }
            "MAIL" => {
                sender = None;
                recipients.clear();
                if !greeted {
                    "503 5.5.1 Send EHLO first\r\n"
                } else if submission && starttls {
                    "530 5.7.0 TLS required for submission\r\n"
                } else if let Some((address, params)) = path(arg, "FROM:") {
                    if !address.is_empty() && !valid_address(address) {
                        "501 5.1.7 Invalid sender\r\n"
                    } else if params.split_ascii_whitespace().any(|p| {
                        !p.eq_ignore_ascii_case("BODY=8BITMIME")
                            && !p.eq_ignore_ascii_case("BODY=7BIT")
                            && !p.to_ascii_uppercase().starts_with("SIZE=")
                    }) {
                        "555 5.5.4 Unsupported parameter\r\n"
                    } else if params
                        .split_ascii_whitespace()
                        .filter_map(|p| {
                            p.get(..5)
                                .filter(|p| p.eq_ignore_ascii_case("SIZE="))
                                .map(|_| &p[5..])
                        })
                        .any(|n| n.parse::<usize>().map_or(true, |n| n > MAX_MESSAGE_BYTES))
                    {
                        "552 5.3.4 Message too large\r\n"
                    } else {
                        let reply = if let Some(state) = &auth {
                            state.sender(service.clone(), address).await
                        } else {
                            "250 2.1.0 Sender accepted\r\n"
                        };
                        if reply.starts_with("250") {
                            sender = Some(address.to_owned());
                        }
                        reply
                    }
                } else {
                    "501 5.5.2 Invalid reverse path\r\n"
                }
            }
            "RCPT" => {
                if sender.is_none() {
                    "503 5.5.1 Send MAIL first\r\n"
                } else if recipients.len() >= 100 {
                    "452 4.5.3 Recipient limit\r\n"
                } else if let Some((address, params)) = path(arg, "TO:") {
                    if !params.is_empty() || !valid_address(address) {
                        "501 5.1.3 Invalid recipient\r\n"
                    } else {
                        let lookup = service.clone();
                        let recipient = address.to_owned();
                        match tokio::task::spawn_blocking(move || lookup.store.resolve(&recipient))
                            .await
                            .unwrap_or(Err(Error::Unavailable))
                        {
                            Ok(_) => {
                                recipients.push(address.to_owned());
                                "250 2.1.5 Recipient accepted\r\n"
                            }
                            Err(Error::NotFound)
                                if service.outbound.is_some()
                                    && auth.as_ref().is_some_and(|s| s.credentials.is_some()) =>
                            {
                                recipients.push(address.to_owned());
                                "250 2.1.5 Recipient accepted\r\n"
                            }
                            Err(Error::NotFound) => {
                                "550 5.7.1 Unknown recipient or relay denied\r\n"
                            }
                            Err(_) => "451 4.3.0 Directory unavailable\r\n",
                        }
                    }
                } else {
                    "501 5.5.2 Invalid forward path\r\n"
                }
            }
            "DATA" if arg.is_empty() => {
                if recipients.is_empty() {
                    "503 5.5.1 Send RCPT first\r\n"
                } else {
                    write(stream.get_mut(), b"354 End with <CRLF>.<CRLF>\r\n").await?;
                    let raw = match data::read(&mut stream).await {
                        Ok(raw) => raw,
                        Err(error) => {
                            let reply = match error.kind() {
                                io::ErrorKind::FileTooLarge => {
                                    b"552 5.3.4 Message too large\r\n".as_slice()
                                }
                                io::ErrorKind::InvalidData => {
                                    b"554 5.6.0 Invalid message framing\r\n"
                                }
                                io::ErrorKind::TimedOut => b"421 4.4.2 DATA timed out\r\n",
                                io::ErrorKind::UnexpectedEof => return Ok(None),
                                _ => return Err(error),
                            };
                            // DATA is not complete: never reinterpret its remaining
                            // bytes as commands or hold capacity draining an attacker.
                            write(stream.get_mut(), reply).await?;
                            return Ok(None);
                        }
                    };
                    let from = sender.take().unwrap_or_default();
                    let targets = std::mem::take(&mut recipients);
                    let service = service.clone();
                    let credentials = auth.as_ref().and_then(|s| s.credentials.clone());
                    let accepted = tokio::task::spawn_blocking(move || {
                        if let Some((username, token)) = credentials {
                            service.submit(&token, &username, &from, &targets, &raw)
                        } else {
                            service.receive(&from, &targets, &raw)
                        }
                    })
                    .await;
                    match accepted.unwrap_or(Err(Error::Unavailable)) {
                        Ok(_) => "250 2.0.0 Queued\r\n",
                        Err(Error::Forbidden) if submission => {
                            "535 5.7.8 Submission no longer authorized\r\n"
                        }
                        Err(Error::Invalid) => "554 5.6.0 Invalid message\r\n",
                        Err(Error::OverQuota) => "452 4.2.2 Queue quota exceeded\r\n",
                        Err(_) => "451 4.3.0 Delivery failed\r\n",
                    }
                }
            }
            "RSET" if arg.is_empty() => {
                sender = None;
                recipients.clear();
                "250 2.0.0 Reset\r\n"
            }
            "NOOP" => "250 2.0.0 OK\r\n",
            "QUIT" if arg.is_empty() => {
                write(stream.get_mut(), b"221 2.0.0 Bye\r\n").await?;
                return Ok(None);
            }
            _ => "500 5.5.2 Command not supported\r\n",
        };
        write(stream.get_mut(), reply.as_bytes()).await?;
        if auth.as_ref().is_some_and(|state| state.failures >= 3) {
            write(
                stream.get_mut(),
                b"421 4.7.0 Too many authentication failures\r\n",
            )
            .await?;
            return Ok(None);
        }
    }
    Ok(None)
}

fn path<'a>(input: &'a str, prefix: &str) -> Option<(&'a str, &'a str)> {
    if !input.get(..prefix.len())?.eq_ignore_ascii_case(prefix) {
        return None;
    }
    let rest = input.get(prefix.len()..)?.trim_start().strip_prefix('<')?;
    let (address, params) = rest.split_once('>')?;
    if !params.is_empty() && !params.starts_with(' ') {
        return None;
    }
    Some((address, params.trim()))
}
