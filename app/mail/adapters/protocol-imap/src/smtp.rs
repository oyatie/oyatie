mod auth;
mod data;
mod envelope;
mod limits;
mod tls;
use crate::wire::write;
pub use limits::{MAX_LINE_BYTES, SmtpParams};
use limits::{Meter, Read};
use mail_kernel::{Error, MAX_MESSAGE_BYTES};
use mail_service::MailService;
use std::{io, sync::Arc};
pub use tls::{smtp_starttls_session, smtp_starttls_session_with};
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

/// How one session runs: the port it serves, whether STARTTLS is offered,
/// whether TLS is already established, whether to greet.
#[derive(Clone, Copy)]
struct Mode {
    submission: bool,
    starttls: bool,
    tls: bool,
    greeting: bool,
}

pub async fn smtp_session<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    service: Arc<MailService>,
) -> io::Result<()> {
    smtp_session_with(stream, service, &SmtpParams::default()).await
}

pub async fn smtp_session_with<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    service: Arc<MailService>,
    params: &SmtpParams,
) -> io::Result<()> {
    let mode = Mode {
        submission: false,
        starttls: false,
        tls: false,
        greeting: true,
    };
    session(stream, service, mode, params).await.map(|_| ())
}

/// An inbound session on an already-encrypted connection: no STARTTLS is
/// offered and one sent is refused as already in TLS mode.
pub async fn smtp_tls_session_with<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    service: Arc<MailService>,
    params: &SmtpParams,
) -> io::Result<()> {
    let mode = Mode {
        submission: false,
        starttls: false,
        tls: true,
        greeting: true,
    };
    session(stream, service, mode, params).await.map(|_| ())
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
    let mode = Mode {
        submission: true,
        starttls: false,
        tls: true,
        greeting: true,
    };
    session(stream, service, mode, &SmtpParams::default())
        .await
        .map(|_| ())
}

async fn session<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    service: Arc<MailService>,
    mode: Mode,
    params: &SmtpParams,
) -> io::Result<Option<S>> {
    let Mode {
        submission,
        starttls,
        tls,
        greeting,
    } = mode;
    let host = params.hostname.as_str();
    let mut auth = (submission && !starttls).then(auth::Submission::default);
    let mut stream = BufReader::new(stream);
    let mut meter = Meter::new(params);
    if greeting {
        write(
            stream.get_mut(),
            format!("220 {host} ESMTP Oyatie\r\n").as_bytes(),
        )
        .await?;
    }
    let mut greeted = false;
    let mut esmtp = false;
    let mut sender = None;
    let mut recipients = vec![];
    loop {
        let bytes = match limits::command(&mut stream, &mut meter, host).await? {
            Ok(bytes) => bytes,
            // A refusal that ends the session, or the line-too-long notice.
            Err(limits::Refusal { reply, close }) => {
                write(stream.get_mut(), reply.as_bytes()).await?;
                if close {
                    return Ok(None);
                }
                continue;
            }
        };
        let Ok(command) = std::str::from_utf8(&bytes) else {
            write(stream.get_mut(), b"500 5.5.1 Invalid command.\r\n").await?;
            continue;
        };
        let (verb, arg) = command.split_once(' ').unwrap_or((command, ""));
        let reply = match verb.to_ascii_uppercase().as_str() {
            "STARTTLS" => {
                if tls {
                    "504 5.7.4 Already in TLS mode.\r\n"
                } else if !starttls {
                    "502 5.7.0 TLS not available.\r\n"
                } else if !esmtp || sender.is_some() {
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
                    write(stream.get_mut(), b"220 2.0.0 Ready to start TLS.\r\n").await?;
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
                let extensions = if starttls {
                    "250-STARTTLS\r\n"
                } else if auth
                    .as_ref()
                    .is_some_and(|state| state.credentials.is_none())
                {
                    "250-AUTH PLAIN\r\n"
                } else {
                    ""
                };
                let banner = if esmtp {
                    format!(
                        "250-{host}\r\n250-SIZE {MAX_MESSAGE_BYTES}\r\n{extensions}250 8BITMIME\r\n"
                    )
                } else {
                    format!("250 {host}\r\n")
                };
                write(stream.get_mut(), banner.as_bytes()).await?;
                continue;
            }
            "MAIL" => {
                sender = None;
                recipients.clear();
                if !greeted {
                    "503 5.5.1 Send EHLO first\r\n"
                } else if submission && starttls {
                    "530 5.7.0 TLS required for submission\r\n"
                } else {
                    let reply = envelope::sender(arg, auth.as_ref(), &service).await;
                    if let Ok(address) = &reply {
                        sender = Some(address.clone());
                    }
                    reply.map_or_else(|reply| reply, |_| "250 2.1.0 Sender accepted\r\n")
                }
            }
            "RCPT" => {
                if sender.is_none() {
                    "503 5.5.1 Send MAIL first\r\n"
                } else if recipients.len() >= 100 {
                    "452 4.5.3 Recipient limit\r\n"
                } else {
                    let reply = envelope::recipient(arg, auth.as_ref(), &service).await;
                    if let Ok(address) = &reply {
                        recipients.push(address.clone());
                    }
                    reply.map_or_else(|reply| reply, |_| "250 2.1.5 Recipient accepted\r\n")
                }
            }
            "DATA" if arg.is_empty() => {
                if recipients.is_empty() {
                    "503 5.5.1 Send RCPT first\r\n"
                } else {
                    write(stream.get_mut(), b"354 End with <CRLF>.<CRLF>\r\n").await?;
                    let raw = match data::read(&mut stream, &mut meter).await {
                        Ok(raw) => raw,
                        Err(error) => {
                            let quota =
                                format!("452 4.7.28 {host} Session exceeded transfer quota.\r\n");
                            let reply = match error.kind() {
                                io::ErrorKind::FileTooLarge => {
                                    b"552 5.3.4 Message too large\r\n".as_slice()
                                }
                                io::ErrorKind::QuotaExceeded => quota.as_bytes(),
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
            "HELP" => "250 2.0.0 Help: RFC 5321\r\n",
            "LHLO" => "502 5.5.1 Invalid command: EHLO expected.\r\n",
            "QUIT" if arg.is_empty() => {
                write(stream.get_mut(), b"221 2.0.0 Bye.\r\n").await?;
                return Ok(None);
            }
            _ => "500 5.5.1 Invalid command.\r\n",
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
}
