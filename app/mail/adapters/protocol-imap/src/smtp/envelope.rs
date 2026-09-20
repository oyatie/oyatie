//! MAIL FROM and RCPT TO: the path grammar, the parameters this server
//! accepts, and the authorization each address needs.
use super::auth::Submission;
use mail_kernel::{Error, MAX_MESSAGE_BYTES, valid_address};
use mail_service::MailService;
use std::sync::Arc;

/// `Ok(address)` to record, `Err(reply)` to send instead of the acceptance.
pub(super) async fn sender(
    arg: &str,
    auth: Option<&Submission>,
    service: &Arc<MailService>,
) -> Result<String, &'static str> {
    let Some((address, params)) = path(arg, "FROM:") else {
        return Err("501 5.5.2 Invalid reverse path\r\n");
    };
    if !address.is_empty() && !valid_address(address) {
        return Err("501 5.1.7 Invalid sender\r\n");
    }
    if params.split_ascii_whitespace().any(|p| {
        !p.eq_ignore_ascii_case("BODY=8BITMIME")
            && !p.eq_ignore_ascii_case("BODY=7BIT")
            && !p.to_ascii_uppercase().starts_with("SIZE=")
    }) {
        return Err("555 5.5.4 Unsupported parameter\r\n");
    }
    if params
        .split_ascii_whitespace()
        .filter_map(|p| {
            p.get(..5)
                .filter(|p| p.eq_ignore_ascii_case("SIZE="))
                .map(|_| &p[5..])
        })
        .any(|n| n.parse::<usize>().map_or(true, |n| n > MAX_MESSAGE_BYTES))
    {
        return Err("552 5.3.4 Message too large\r\n");
    }
    if let Some(state) = auth {
        let reply = state.sender(service.clone(), address).await;
        if !reply.starts_with("250") {
            return Err(reply);
        }
    }
    Ok(address.to_owned())
}

pub(super) async fn recipient(
    arg: &str,
    auth: Option<&Submission>,
    service: &Arc<MailService>,
) -> Result<String, &'static str> {
    let Some((address, params)) = path(arg, "TO:") else {
        return Err("501 5.5.2 Invalid forward path\r\n");
    };
    if !params.is_empty() || !valid_address(address) {
        return Err("501 5.1.3 Invalid recipient\r\n");
    }
    let lookup = service.clone();
    let recipient = address.to_owned();
    match tokio::task::spawn_blocking(move || lookup.store.resolve(&recipient))
        .await
        .unwrap_or(Err(Error::Unavailable))
    {
        Ok(_) => Ok(address.to_owned()),
        Err(Error::NotFound)
            if service.outbound.is_some() && auth.is_some_and(|s| s.credentials.is_some()) =>
        {
            Ok(address.to_owned())
        }
        Err(Error::NotFound) => Err("550 5.7.1 Unknown recipient or relay denied\r\n"),
        Err(_) => Err("451 4.3.0 Directory unavailable\r\n"),
    }
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
