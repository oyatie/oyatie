use super::{
    condstore,
    response::Output,
    state::Selection,
    syntax::{Token, sequence_set, tokens},
};
use mail_kernel::Account;
use mail_service::MailService;

pub(super) fn execute(
    service: &MailService,
    token: &str,
    account: &Account,
    parts: &[String],
    selected: &mut Option<Selection>,
    output: &mut Output,
) -> Result<Option<String>, &'static str> {
    if parts.len() != 4 {
        return Err("BAD");
    }
    let (options, identifiers) = super::objectid::options(tokens(&parts[3]).ok_or("BAD")?)?;
    let folder = identifiers
        .as_ref()
        .and_then(|ids| ids.find(account))
        .or_else(|| super::folders::find(account, &parts[2]));
    let largest = folder
        .and_then(|folder| {
            super::selected::messages_in(account, &folder.id)
                .last()
                .and_then(|m| m.uid_in(&folder.id))
        })
        .unwrap_or_default();
    let (enabled, resync) = options_parse(&options, output, largest)?;
    if identifiers.is_some() {
        super::objectid::activate(output);
    }
    let folder = folder.ok_or("NO")?;
    if output.uidonly && resync.as_ref().is_some_and(|r| r.sequence_match) {
        return Err("BAD [UIDREQUIRED]");
    }
    let readonly = if parts[1].eq_ignore_ascii_case("EXAMINE") {
        true
    } else {
        match service.authorize(token, &account.id, mail_api::Action::Write) {
            Ok(_) => false,
            Err(mail_kernel::Error::Forbidden) => true,
            Err(_) => return Err("NO"),
        }
    };
    let messages = super::selected::messages_in(account, &folder.id);
    let mut earlier = Vec::new();
    if let Some(Resync {
        validity, since, ..
    }) = &resync
        && *validity == folder.uid_validity
    {
        earlier = condstore::vanished(service, token, account, &folder.id, *since)?;
    }
    *selected = Some(Selection {
        mailbox: folder.id.clone(),
        uid_validity: folder.uid_validity,
        readonly,
        condstore: enabled,
        qresync: output.qresync,
        modseq: account.mail_modseq,
        saved: Default::default(),
        ids: messages
            .iter()
            .map(|m| (m.id.clone(), m.uid_in(&folder.id).unwrap()))
            .collect(),
    });
    output.condstore = enabled;
    if output.objectid {
        output.extend_from_slice(
            format!(
                "* OK [OBJECTID {}] Mailbox identity\r\n",
                super::objectid::compound(account, folder)
            )
            .as_bytes(),
        );
    }
    output.extend_from_slice(format!("* {} EXISTS\r\n* 0 RECENT\r\n* FLAGS (\\Seen \\Answered \\Flagged \\Deleted \\Draft)\r\n* OK [UIDVALIDITY {}] UIDs valid\r\n* OK [UIDNEXT {}] Next UID\r\n", messages.len(), folder.uid_validity, folder.uid_next).as_bytes());
    if enabled {
        output.extend_from_slice(
            format!(
                "* OK [HIGHESTMODSEQ {}] Highest modification sequence\r\n",
                account.mail_modseq
            )
            .as_bytes(),
        );
    }
    if let Some(Resync {
        validity,
        since,
        known,
        ..
    }) = resync
        && validity == folder.uid_validity
    {
        let contains = |uid| {
            known
                .as_ref()
                .is_none_or(|ranges| ranges.iter().any(|(a, b)| uid >= *a && uid <= *b))
        };
        if !earlier.is_empty() {
            output.extend_from_slice(
                format!("* VANISHED (EARLIER) {}\r\n", condstore::ranges(earlier)).as_bytes(),
            );
        }
        for (i, m) in messages.iter().enumerate() {
            let uid = m.uid_in(&folder.id).unwrap();
            if m.modseq > since && contains(uid) {
                output.fetch_start(i + 1, uid);
                output.extend_from_slice(
                    format!(
                        " FLAGS ({}) MODSEQ ({}))\r\n",
                        super::syntax::flags(m),
                        m.modseq
                    )
                    .as_bytes(),
                );
            }
        }
    }
    Ok(None)
}

fn options_parse(
    tokens: &[Token],
    output: &Output,
    largest: u32,
) -> Result<(bool, Option<Resync>), &'static str> {
    if tokens.is_empty() {
        return Ok((output.condstore, None));
    }
    let [Token::Open, rest @ .., Token::Close] = tokens else {
        return Err("BAD");
    };
    let mut rest = rest;
    let mut enabled = output.condstore;
    let mut resync = None;
    while !rest.is_empty() {
        let [Token::Word(word), tail @ ..] = rest else {
            return Err("BAD");
        };
        rest = tail;
        if word.eq_ignore_ascii_case("CONDSTORE") {
            enabled = true;
            continue;
        }
        if !word.eq_ignore_ascii_case("QRESYNC") || !output.qresync || resync.is_some() {
            return Err("BAD");
        }
        let [
            Token::Open,
            Token::Word(validity),
            Token::Word(since),
            tail @ ..,
        ] = rest
        else {
            return Err("BAD");
        };
        let validity = u32::try_from(condstore::number(validity)?).map_err(|_| "BAD")?;
        let since = condstore::number(since)?;
        rest = tail;
        let mut known = None;
        let mut sequence_match = false;
        if let [Token::Word(set), tail @ ..] = rest {
            known = Some(sequence_set(set, largest).ok_or("BAD")?);
            rest = tail;
        }
        if let [
            Token::Open,
            Token::Word(sequences),
            Token::Word(uids),
            Token::Close,
            tail @ ..,
        ] = rest
        {
            sequence_match = true;
            sequence_set(sequences, u32::MAX).ok_or("BAD")?;
            let uids = sequence_set(uids, largest).ok_or("BAD")?;
            if known.is_none() {
                known = Some(uids);
            }
            rest = tail;
        }
        let [Token::Close, tail @ ..] = rest else {
            return Err("BAD");
        };
        rest = tail;
        resync = Some(Resync {
            validity,
            since,
            known,
            sequence_match,
        });
        enabled = true;
    }
    Ok((enabled, resync))
}

struct Resync {
    validity: u32,
    since: u64,
    known: Option<Vec<(u32, u32)>>,
    sequence_match: bool,
}
