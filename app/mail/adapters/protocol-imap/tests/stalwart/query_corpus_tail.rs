// Seeds 12-21 of the upstream Email/query corpus (bcc-email .. invalid-ascii).
use super::builders::{ALPHA, JPEG, Msg, RE, TU, head, m, message, multipart, text};
use super::corpus::Seed;
use base64::{Engine, engine::general_purpose::STANDARD};

pub fn rest(days: impl Fn(i64) -> String, hours: impl Fn(i64) -> String) -> Vec<Seed> {
    vec![
        Seed::new(
            "bcc-email",
            message(Msg {
                from: TU,
                to: "jack@example.com",
                bcc: "secret@example.com",
                subject: "Confidential note",
                date: days(2),
                message_id: "<bcc-001@test>",
                body: "This is a confidential message with a BCC recipient.".into(),
                ..Msg::default()
            }),
            "folderA",
            &["$seen", "$draft"],
            2 * 24,
        ),
        Seed::new(
            "special-headers",
            message(Msg {
                from: "List Admin <list-admin@example.com>",
                to: TU,
                subject: "Mailing list post",
                date: days(1),
                message_id: "<list-001@test>",
                body: "This is a post from a mailing list.".into(),
                extra: vec![
                    "List-Post: <mailto:list@example.com>",
                    "List-Unsubscribe: <https://example.com/unsub>",
                    "X-Custom-Header: custom-value-12345",
                ],
                ..Msg::default()
            }),
            "inbox",
            &["$seen"],
            24,
        ),
        Seed::new(
            "multipart-related",
            multipart(
                "related",
                "----=_Rel_001_boundary",
                head(
                    "Kate Images <kate@example.com>",
                    TU,
                    None,
                    "Image embedded email",
                    &hours(12),
                    "<related-001@test>",
                ),
                vec![
                    text(
                        "html",
                        "<html><body><p>See the image below:</p><img src=\"cid:image001@test\"/></body></html>",
                    ),
                    vec![
                        "Content-Type: image/jpeg".into(),
                        "Content-ID: <image001@test>".into(),
                        "Content-Disposition: inline".into(),
                        "Content-Transfer-Encoding: base64".into(),
                        String::new(),
                        STANDARD.encode(JPEG),
                    ],
                ],
            ),
            "inbox",
            &["$seen"],
            12,
        ),
        Seed::new(
            "intl-sender",
            m(
                "=?UTF-8?B?6YeR5Z+O5q2m?= <kaneshiro@example.com>",
                TU,
                "=?UTF-8?B?44GT44KT44Gr44Gh44Gv?=",
                hours(6),
                "<intl-001@test>",
                "This message has an internationalized sender name and subject.",
            ),
            "inbox",
            &[],
            6,
        ),
        Seed::new(
            "sort-test-1",
            m(
                "Zara First <zara@example.com>",
                TU,
                "Alpha sort test",
                days(5),
                "<sort-001@test>",
                &"A".repeat(100),
            ),
            "folderB",
            &["$seen"],
            3 * 24,
        ),
        Seed::new(
            "sort-test-2",
            m(
                "Amy Second <amy@example.com>",
                TU,
                "Beta sort test",
                days(3),
                "<sort-002@test>",
                &"B".repeat(500),
            ),
            "folderB",
            &["$seen", "$flagged"],
            2 * 24,
        ),
        Seed::new(
            "sort-test-3",
            m(
                "Mike Third <mike@example.com>",
                TU,
                "Gamma sort test",
                days(1),
                "<sort-003@test>",
                &"C".repeat(50),
            ),
            "folderB",
            &[],
            24,
        ),
        Seed::new(
            "draft-for-submission",
            m(
                "jdoe@example.com",
                "jane.smith@example.com",
                "Test submission email",
                hours(1),
                "<submission-001@test>",
                "This email will be used for submission testing.",
            ),
            "inbox",
            &["$seen", "$draft"],
            1,
        ),
        Seed::new(
            "child-mailbox-email",
            m(
                "Nancy Nested <nancy@example.com>",
                TU,
                "In nested folder",
                days(5),
                "<child-001@test>",
                "This email lives in a nested child mailbox.",
            ),
            "child1",
            &["$seen"],
            5 * 24,
        ),
        Seed::new(
            "korean-euckr",
            [
                "From: =?EUC-KR?B?seS/tbjR?= <korean-sender@example.com>",
                "To: testuser@example.com",
                "Subject: =?EUC-KR?B?sNa0z7TZx9Cw+A==?=",
                &format!("Date: {}", hours(5)),
                "Message-ID: <korean-001@test>",
                "MIME-Version: 1.0",
                "Content-Type: text/plain; charset=EUC-KR",
                "Content-Transfer-Encoding: base64",
                "",
                &STANDARD.encode([
                    0xc5, 0xd7, 0xbd, 0xba, 0xc6, 0xae, 0x20, 0xc0, 0xcc, 0xb8, 0xde, 0xc0, 0xcf,
                    0xc0, 0xd4, 0xb4, 0xcf, 0xb4, 0xd9,
                ]),
            ]
            .join("\r\n"),
            "inbox",
            &["$seen"],
            5,
        ),
        Seed::new(
            "invalid-ascii",
            [
                "From: broken@example.com",
                "To: testuser@example.com",
                "Subject: Malformed email test",
                &format!("Date: {}", hours(4)),
                "Message-ID: <invalid-001@test>",
                "MIME-Version: 1.0",
                "Content-Type: text/plain; charset=us-ascii",
                "X-Broken-Header: value with \u{01}\u{02} control chars",
                "",
                "This email has some issues.\r\n",
                &format!(
                    "It has a line that is way too long: {}\r\n",
                    "x".repeat(1000)
                ),
                "And some 8-bit chars in ASCII: caf\u{e9} na\u{ef}ve r\u{e9}sum\u{e9}\r\n",
                "End of message.",
            ]
            .join("\r\n"),
            "inbox",
            &["$seen"],
            4,
        ),
    ]
}
