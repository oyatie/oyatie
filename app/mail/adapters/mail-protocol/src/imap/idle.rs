use super::{Session, response::Output, selected, syntax};
use crate::wire::write;
use mail_kernel::Error;
use mail_service::MailService;
use std::{collections::BTreeMap, io, sync::Arc, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, BufReader};

#[cfg(test)]
mod tests;

struct State {
    session: Session,
    revision: Option<u64>,
    flags: BTreeMap<(String, u32), String>,
}

impl State {
    fn refresh(&mut self, service: &MailService, output: &mut Output) -> Result<(), Error> {
        let session = &mut self.session;
        output.uidonly = session.uidonly;
        // The empty selection reads the durable revision without message bodies.
        // Authorization is rechecked even when the mailbox has not changed.
        let revision = service
            .messages(&session.credential, &session.account_id, &[])?
            .revision;
        if self.revision == Some(revision) {
            return Ok(());
        }
        if session.selected.is_none() {
            self.revision = Some(revision);
            self.flags.clear();
            return Ok(());
        }
        let account = service.read(&session.credential, &session.account_id)?;
        selected::synchronize(
            &account,
            &[String::new(), "IDLE".into()],
            &mut session.selected,
            output,
        );
        let mut flags = BTreeMap::new();
        if let Some(selection) = &session.selected {
            let messages: BTreeMap<_, _> = account
                .messages
                .iter()
                .map(|message| (message.id.as_str(), message))
                .collect();
            for (index, (id, uid)) in selection.ids.iter().enumerate() {
                if output.is_closed() {
                    break;
                }
                let Some(message) = messages.get(id.as_str()) else {
                    continue;
                };
                let key = (id.clone(), *uid);
                let mut value = format!("FLAGS ({})", syntax::flags(message));
                if selection.condstore {
                    value.push_str(&format!(" MODSEQ ({})", message.modseq));
                }
                if self.flags.get(&key) != Some(&value) {
                    output.fetch_start(index + 1, *uid);
                    output.extend_from_slice(format!(" {value})\r\n").as_bytes());
                }
                flags.insert(key, value);
            }
        }
        self.revision = Some(account.revision);
        self.flags = flags;
        Ok(())
    }
}

// read_until is cancellation safe only if its destination survives cancellation.
// Keeping the partial line here preserves a fragmented DONE across refreshes.
async fn continuation<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    bytes: &mut Vec<u8>,
) -> io::Result<Option<bool>> {
    let remaining = 8193_usize.saturating_sub(bytes.len());
    let count = stream
        .take(remaining as u64)
        .read_until(b'\n', bytes)
        .await?;
    if count == 0 {
        return Ok(None);
    }
    if bytes.len() > 8192 || !bytes.ends_with(b"\r\n") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid IDLE framing",
        ));
    }
    Ok(Some(bytes.eq_ignore_ascii_case(b"DONE\r\n")))
}

pub(super) async fn run<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    service: Arc<MailService>,
    session: &mut Session,
    parts: &[String],
) -> io::Result<bool> {
    let tag = &parts[0];
    if parts.len() != 2 || session.credential.is_empty() {
        write(
            stream.get_mut(),
            format!("{tag} BAD IDLE unavailable\r\n").as_bytes(),
        )
        .await?;
        return Ok(false);
    }
    bounded(stream, service, session, tag, Duration::from_secs(30 * 60)).await
}

async fn bounded<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    service: Arc<MailService>,
    session: &mut Session,
    tag: &str,
    lifetime: Duration,
) -> io::Result<bool> {
    let mut aligned = true;
    match tokio::time::timeout(
        lifetime,
        active(stream, service, session, tag, &mut aligned),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            // Cancellation can interrupt an output chunk halfway through a line.
            // Close then; never append BYE inside a partially written response.
            if aligned {
                write(stream.get_mut(), b"* BYE IDLE lifetime exceeded\r\n").await?;
            }
            Ok(true)
        }
    }
}

async fn emit<S: AsyncWrite + Unpin>(
    stream: &mut S,
    bytes: &[u8],
    aligned: &mut bool,
) -> io::Result<()> {
    *aligned = false;
    write(stream, bytes).await?;
    *aligned = bytes.ends_with(b"\r\n");
    Ok(())
}

async fn active<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut BufReader<S>,
    service: Arc<MailService>,
    session: &mut Session,
    tag: &str,
    aligned: &mut bool,
) -> io::Result<bool> {
    let mut state = State {
        session: std::mem::take(session),
        revision: None,
        flags: BTreeMap::new(),
    };
    let mut input = Vec::new();
    let mut done = None;
    let mut started = false;
    loop {
        let (send, mut receive) = tokio::sync::mpsc::channel(2);
        let service = service.clone();
        let worker = tokio::task::spawn_blocking(move || -> io::Result<_> {
            let mut output = Output::new(send);
            let result = state.refresh(&service, &mut output);
            output.finish()?;
            Ok((state, result))
        });
        // One refresh at a time, with the same bounded output used by commands.
        // No snapshots accumulate while the client applies backpressure.
        loop {
            tokio::select! {
                line = continuation(stream, &mut input), if started && done.is_none() => {
                    let Some(valid) = line? else { return Ok(true) };
                    done = Some(valid);
                }
                bytes = receive.recv() => {
                    let Some(bytes) = bytes else { break };
                    emit(stream.get_mut(), &bytes, aligned).await?;
                }
            }
        }
        let (next, result) = worker
            .await
            .map_err(|_| io::Error::other("IDLE worker failed"))??;
        state = next;
        if result.is_err() {
            if started {
                emit(
                    stream.get_mut(),
                    b"* BYE IDLE authorization or storage unavailable\r\n",
                    aligned,
                )
                .await?;
            } else {
                emit(
                    stream.get_mut(),
                    format!("{tag} NO IDLE unavailable\r\n").as_bytes(),
                    aligned,
                )
                .await?;
                *session = state.session;
            }
            return Ok(started);
        }
        if !started {
            emit(stream.get_mut(), b"+ Idling\r\n", aligned).await?;
            started = true;
        }
        if done.is_none() {
            tokio::select! {
                line = continuation(stream, &mut input) => {
                    let Some(valid) = line? else { return Ok(true) };
                    done = Some(valid);
                }
                _ = tokio::time::sleep(Duration::from_millis(250)) => {}
            }
        }
        if let Some(valid) = done {
            let status = if valid { "OK" } else { "BAD" };
            emit(
                stream.get_mut(),
                format!("{tag} {status} IDLE completed\r\n").as_bytes(),
                aligned,
            )
            .await?;
            *session = state.session;
            return Ok(false);
        }
    }
}
