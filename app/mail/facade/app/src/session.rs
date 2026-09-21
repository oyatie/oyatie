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
    authentication: &mail_protocol_imap::Authentication,
) {
    let Ok(permit) = capacity.clone().try_acquire_owned() else {
        return;
    };
    let service = service.clone();
    let tls = tls.clone();
    let authentication = authentication.clone();
    sessions.spawn(async move {
        let _permit = permit;
        match protocol {
            Protocol::Smtp | Protocol::SubmissionStartTls => {
                let params = smtp_params(&stream, &authentication, &protocol);
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
                let params = smtp_params(&stream, &authentication, &protocol);
                if let Ok(Ok(stream)) =
                    tokio::time::timeout(Duration::from_secs(10), tls.accept(stream)).await
                {
                    let _ = match protocol {
                        Protocol::Imap => {
                            mail_protocol_imap::imap_session(stream, service, true).await
                        }
                        Protocol::Submission => {
                            mail_protocol_imap::submission_session_with(
                                stream, service, true, &params,
                            )
                            .await
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

/// The peer address is the session's identity for policy; an unknown one
/// matches nothing.
fn smtp_params(
    stream: &TcpStream,
    authentication: &mail_protocol_imap::Authentication,
    protocol: &Protocol,
) -> mail_protocol_imap::SmtpParams {
    mail_protocol_imap::SmtpParams {
        peer: stream
            .peer_addr()
            .map_or(std::net::Ipv4Addr::UNSPECIFIED.into(), |a| a.ip()),
        authentication: authentication.clone(),
        max_message_size: max_message_size(protocol),
        ..mail_protocol_imap::SmtpParams::default()
    }
}

/// Advertised `SIZE`, the `MAIL FROM SIZE=` check and the DATA reader read
/// this one number. Submission is normalized on the way in and signed on the
/// way out, so it advertises the deliverable ceiling less both; inbound shares
/// this builder, does neither, and keeps the full one.
fn max_message_size(protocol: &Protocol) -> usize {
    match protocol {
        Protocol::Submission | Protocol::SubmissionStartTls => mail_kernel::MAX_DATA_BYTES,
        _ => mail_kernel::MAX_MESSAGE_BYTES,
    }
}

#[cfg(test)]
mod tests {
    use super::{Protocol, max_message_size};

    /// Reserving room for the signature on the inbound path would refuse mail
    /// this server accepted before, on a path that is never signed.
    #[test]
    fn only_submission_gives_up_room_for_the_signature() {
        for inbound in [Protocol::Smtp, Protocol::Imap, Protocol::Pop] {
            assert_eq!(
                max_message_size(&inbound),
                mail_kernel::MAX_MESSAGE_BYTES,
                "inbound must still advertise the full size"
            );
        }
        for submission in [Protocol::Submission, Protocol::SubmissionStartTls] {
            assert_eq!(max_message_size(&submission), mail_kernel::MAX_DATA_BYTES);
        }
        const { assert!(mail_kernel::MAX_DATA_BYTES < mail_kernel::MAX_MESSAGE_BYTES) };
    }
}
