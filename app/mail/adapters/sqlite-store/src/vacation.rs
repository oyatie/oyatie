use super::{storage, submission};
use mail_kernel::{Account, Error};
use rusqlite::{Connection, params};

pub(super) fn maybe_reply(
    tx: &Connection,
    account: &Account,
    received_at: i64,
    raw: &[u8],
) -> Result<(), Error> {
    let Some(reply) = account.vacation.reply(received_at, raw, &account.address) else {
        return Ok(());
    };
    let inserted = tx
        .execute(
            "INSERT OR IGNORE INTO vacation_sent(account,sender) VALUES(?1,?2)",
            params![account.id, reply.to],
        )
        .map_err(storage)?;
    if inserted != 1 {
        return Ok(());
    }
    let message = format!(
        "From: {}\r\nTo: {}\r\nSubject: {}\r\nAuto-Submitted: auto-replied\r\n\r\n{}\r\n",
        account.address, reply.to, reply.subject, reply.text
    );
    match submission::enqueue_tx(
        tx,
        &account.id,
        &account.address,
        &[reply.to],
        message.as_bytes(),
        true,
    ) {
        Ok(_) | Err(Error::OverQuota | Error::Forbidden) => Ok(()),
        Err(error) => Err(error),
    }
}
