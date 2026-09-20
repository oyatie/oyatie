use mail_service::MailService;
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpStream, sync::Semaphore, task::JoinSet};
use tokio_rustls::TlsAcceptor;

pub(super) enum Protocol {
    Smtp,
    Imap,
    Submission,
    ImapStartTls,
    SubmissionStartTls,
    Pop,
    PopStartTls,
}

pub(super) fn spawn(
    sessions: &mut JoinSet<()>,
    stream: TcpStream,
    service: &Arc<MailService>,
    tls: &TlsAcceptor,
    capacity: &Arc<Semaphore>,
    protocol: Protocol,
) {
    let Ok(permit) = capacity.clone().try_acquire_owned() else {
        return;
    };
    let service = service.clone();
    let tls = tls.clone();
    sessions.spawn(async move {
        let _permit = permit;
        match protocol {
            Protocol::Smtp | Protocol::SubmissionStartTls => {
                // The peer address is the session's identity for policy.
                let params = mail_protocol_imap::SmtpParams {
                    peer: stream
                        .peer_addr()
                        .map_or(std::net::Ipv4Addr::UNSPECIFIED.into(), |a| a.ip()),
                    ..mail_protocol_imap::SmtpParams::default()
                };
                let _ = mail_protocol_imap::smtp_starttls_session_with(
                    stream,
                    service,
                    matches!(protocol, Protocol::SubmissionStartTls),
                    |stream| tls.accept(stream),
                    &params,
                )
                .await;
            }
            Protocol::ImapStartTls => {
                let _ = mail_protocol_imap::imap_starttls_session(stream, service, |stream| {
                    tls.accept(stream)
                })
                .await;
            }
            Protocol::PopStartTls => {
                let _ = mail_protocol_imap::pop_starttls_session(stream, service, |stream| {
                    tls.accept(stream)
                })
                .await;
            }
            Protocol::Imap | Protocol::Submission | Protocol::Pop => {
                if let Ok(Ok(stream)) =
                    tokio::time::timeout(Duration::from_secs(10), tls.accept(stream)).await
                {
                    let _ = match protocol {
                        Protocol::Imap => {
                            mail_protocol_imap::imap_session(stream, service, true).await
                        }
                        Protocol::Submission => {
                            mail_protocol_imap::submission_session(stream, service, true).await
                        }
                        Protocol::Pop => {
                            mail_protocol_imap::pop_session(stream, service, true).await
                        }
                        _ => unreachable!(),
                    };
                }
            }
        }
    });
}
