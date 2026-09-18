use super::{Session, response::Output};

pub(super) fn capabilities(
    session: &Session,
    encrypted: bool,
    starttls: bool,
    output: &mut Output,
) {
    // Every token here is a row of tests/capability_matrix.rs with a proof.
    output.extend_from_slice(b"* CAPABILITY IMAP4rev1 UIDPLUS MOVE ENABLE UTF8=ACCEPT BINARY PREVIEW ESEARCH SEARCHRES OBJECTID+ IDLE SORT ESORT LITERAL+ NAMESPACE UNSELECT MULTIAPPEND CONDSTORE QRESYNC THREAD=REFERENCES THREAD=ORDEREDSUBJECT ID LIST-EXTENDED LIST-STATUS CHILDREN SPECIAL-USE CREATE-SPECIAL-USE");
    if !session.credential.is_empty() {
        output.extend_from_slice(b" UIDONLY UNAUTHENTICATE");
    }
    output.extend_from_slice(if encrypted {
        b" AUTH=PLAIN SASL-IR\r\n"
    } else if starttls {
        b" STARTTLS LOGINDISABLED\r\n"
    } else {
        b" LOGINDISABLED\r\n"
    });
}

/// RFC 2971: the server names itself honestly; client parameters are ignored.
pub(super) fn id(output: &mut Output) {
    output.extend_from_slice(b"* ID (\"name\" \"Oyatie mail\")\r\n");
}

pub(super) fn unauthenticate(
    session: &mut Session,
    output: &mut Output,
) -> Result<(), &'static str> {
    if session.credential.is_empty() {
        return Err("NO");
    }
    *session = Session::default();
    output.utf8 = false;
    output.condstore = false;
    output.qresync = false;
    output.objectid = false;
    output.uidonly = false;
    Ok(())
}

pub(super) fn requires_uid(session: &Session, verb: &str) -> bool {
    session.uidonly
        && session.selected.is_some()
        && matches!(
            verb.to_ascii_uppercase().as_str(),
            "FETCH" | "STORE" | "SEARCH" | "SORT" | "THREAD" | "COPY" | "MOVE"
        )
}
