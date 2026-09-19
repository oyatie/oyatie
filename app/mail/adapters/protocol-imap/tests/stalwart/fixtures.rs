use mail_api::BlobStore;
use mail_api::MetadataStore;
use mail_kernel::Command;
use mail_sqlite_store::SqliteStore;
use std::collections::BTreeMap;

// Transport fixtures corresponding to the pinned compliance corpus. Assertions
// remain included unchanged from upstream; these fixtures use fixed dates.
fn message(from: &str, subject: &str, id: &str, headers: &str, body: &str) -> String {
    format!(
        "From: {from}\r\nTo: testuser@example.com\r\nSubject: {subject}\r\nDate: Mon, 14 Sep 2026 12:00:00 +0000\r\nMessage-ID: <{id}>\r\nMIME-Version: 1.0\r\n{headers}\r\n{body}"
    )
}
const TEXT: &str = "Content-Type: text/plain; charset=UTF-8\r\n";
pub fn plain() -> String {
    message(
        "Alice Sender <alice@example.com>",
        "Meeting tomorrow morning",
        "plain-simple-001@test",
        TEXT,
        "Let's meet tomorrow at 9am in the conference room.",
    )
}

pub fn seed(db: &SqliteStore) -> BTreeMap<String, String> {
    let mut emails = BTreeMap::from([
        ("plain-simple".into(), "e1".into()),
        ("custom-keywords".into(), "e2".into()),
    ]);
    let html = message(
        "Bob Jones <bob@example.org>",
        "Q3 Financial Report",
        "html-attach-001@test",
        "Cc: charlie@example.net\r\nContent-Type: multipart/mixed; boundary=mixed\r\n",
        "--mixed\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<html><body><h1>Q3 Report</h1><p>Please find the report attached.</p></body></html>\r\n--mixed\r\nContent-Type: application/pdf; name=report.pdf\r\nContent-Disposition: attachment; filename=report.pdf\r\nContent-Transfer-Encoding: base64\r\n\r\nJVBERi0xLjQK\r\n--mixed--\r\n",
    );
    let alternative = message(
        "Frank Newsletter <frank@example.com>",
        "Newsletter: Weekly Digest",
        "html-only-001@test",
        "Content-Type: multipart/alternative; boundary=alt\r\n",
        "--alt\r\nContent-Type: text/plain; charset=UTF-8\r\n\r\nWeekly Digest - plain text version\r\n--alt\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<html><body><h1>Weekly Digest</h1><p>Here is your <b>weekly digest</b> of news.</p><img src=\"cid:image1\"/></body></html>\r\n--alt--\r\n",
    );
    let related = message(
        "Kate Images <kate@example.com>",
        "Image embedded email",
        "related-001@test",
        "Content-Type: multipart/related; boundary=rel\r\n",
        "--rel\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<html><body><p>See the image below:</p><img src=\"cid:image001@test\"/></body></html>\r\n--rel\r\nContent-Type: image/jpeg\r\nContent-ID: <image001@test>\r\nContent-Disposition: inline\r\nContent-Transfer-Encoding: base64\r\n\r\n/9j/2Q==\r\n--rel--\r\n",
    );
    let fixtures = [
        ("html-attachment", html),
        (
            "thread-starter",
            message(
                "testuser@example.com",
                "Project Alpha Discussion",
                "thread-alpha-001@test",
                TEXT,
                "I'd like to discuss the Project Alpha timeline.",
            ),
        ),
        (
            "thread-reply-1",
            message(
                "Alice Sender <alice@example.com>",
                "Re: Project Alpha Discussion",
                "thread-alpha-002@test",
                &format!(
                    "{TEXT}In-Reply-To: <thread-alpha-001@test>\r\nReferences: <thread-alpha-001@test>\r\n"
                ),
                "Sure, let's discuss. How about Thursday?",
            ),
        ),
        (
            "thread-reply-2",
            message(
                "Bob Jones <bob@example.org>",
                "Re: Project Alpha Discussion",
                "thread-alpha-003@test",
                &format!(
                    "{TEXT}In-Reply-To: <thread-alpha-002@test>\r\nReferences: <thread-alpha-001@test> <thread-alpha-002@test>\r\n"
                ),
                "Thursday works for me. I'll bring the xylophone presentation materials.",
            ),
        ),
        (
            "multi-mailbox",
            message(
                "David Cross <david@example.com>",
                "Cross-filed document",
                "multi-mb-001@test",
                TEXT,
                "This document should appear in multiple folders.",
            ),
        ),
        (
            "large-email",
            message(
                "Eve Large <eve@example.com>",
                "Detailed analysis with data",
                "large-001@test",
                TEXT,
                &format!(
                    "Start of analysis. {}End of analysis.",
                    "This is a detailed paragraph of analysis text that covers various topics. "
                        .repeat(700)
                ),
            ),
        ),
        ("html-only", alternative),
        (
            "no-subject",
            message(
                "Grace Minimal <grace@example.com>",
                "",
                "no-subj-001@test",
                TEXT,
                "This message has no subject.",
            ),
        ),
        (
            "bcc-email",
            message(
                "testuser@example.com",
                "Confidential note",
                "bcc-001@test",
                &format!("{TEXT}Bcc: secret@example.com\r\n"),
                "This is a confidential message with a BCC recipient.",
            ),
        ),
        (
            "special-headers",
            message(
                "List Admin <list-admin@example.com>",
                "Mailing list post",
                "list-001@test",
                &format!(
                    "{TEXT}List-Post: <mailto:list@example.com>\r\nList-Unsubscribe: <https://example.com/unsub>\r\nX-Custom-Header: custom-value-12345\r\n"
                ),
                "This is a post from a mailing list.",
            ),
        ),
        ("multipart-related", related),
        (
            "intl-sender",
            message(
                "=?UTF-8?B?6YeR5Z+O5q2m?= <kaneshiro@example.com>",
                "=?UTF-8?B?44GT44KT44Gr44Gh44Gv?=",
                "intl-001@test",
                TEXT,
                "This message has an internationalized sender name and subject.",
            ),
        ),
        (
            "korean-euckr",
            message(
                "=?EUC-KR?B?seS/tbjR?= <korean-sender@example.com>",
                "=?EUC-KR?B?sNa0z7TZx9Cw+A==?=",
                "korean-001@test",
                "Content-Type: text/plain; charset=EUC-KR\r\nContent-Transfer-Encoding: base64\r\n",
                "xde9usauIMDMuN7Az8DUtM+02Q==",
            ),
        ),
        (
            "invalid-ascii",
            message(
                "broken@example.com",
                "Malformed email test",
                "invalid-001@test",
                "Content-Type: text/plain; charset=us-ascii\r\nX-Broken-Header: value with \u{01}\u{02} control chars\r\n",
                &format!(
                    "This email has some issues.\r\nIt has a line that is way too long: {}\r\nAnd some 8-bit chars in ASCII: café naïve résumé\r\nEnd of message.",
                    "x".repeat(1000)
                ),
            ),
        ),
    ];
    for (key, raw) in fixtures {
        let mut mailboxes = vec!["inbox".into()];
        if key == "multi-mailbox" {
            mailboxes.push("m3".into());
        }
        let state = db.account("a").unwrap();
        let state = db
            .execute(
                "a",
                mail_api::Precondition::Observed(state.revision),
                vec![
                    db.append(
                        "a",
                        mailboxes,
                        raw.as_bytes(),
                        vec!["$seen".into()],
                        1789387200,
                    )
                    .unwrap(),
                ],
            )
            .unwrap();
        emails.insert(key.into(), state.ids.last().unwrap().clone());
    }
    emails
}
