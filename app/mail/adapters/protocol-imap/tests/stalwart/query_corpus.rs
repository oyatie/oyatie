// Seed corpus mirroring tests/src/jmap/compliance/mod.rs `seed_emails` at the
// pinned revision: same keys, senders, mailboxes, keywords and now-relative
// receive times, so the unchanged Email/query predicates see the same corpus.
use super::builders::{ALPHA, JPEG, Msg, RE, TU, head, m, message, multipart, text};
use base64::{Engine, engine::general_purpose::STANDARD};

pub struct Seed {
    pub key: &'static str,
    pub raw: String,
    pub mailbox: &'static str,
    pub keywords: Vec<String>,
    pub hours_ago: i64,
}
impl Seed {
    pub fn new(
        key: &'static str,
        raw: String,
        mailbox: &'static str,
        keywords: &[&str],
        hours_ago: i64,
    ) -> Self {
        Self {
            key,
            raw,
            mailbox,
            keywords: keywords.iter().map(|k| (*k).to_owned()).collect(),
            hours_ago,
        }
    }
}

/// Seeds 1-11 of the upstream corpus (plain-simple .. very-old).
pub fn first(days: impl Fn(i64) -> String) -> Vec<Seed> {
    vec![
        Seed::new(
            "plain-simple",
            m(
                "Alice Sender <alice@example.com>",
                TU,
                "Meeting tomorrow morning",
                days(10),
                "<plain-simple-001@test>",
                "Let's meet tomorrow at 9am in the conference room.",
            ),
            "inbox",
            &["$seen"],
            10 * 24,
        ),
        Seed::new(
            "html-attachment",
            multipart(
                "mixed",
                "----=_Part_001_boundary",
                head(
                    "Bob Jones <bob@example.org>",
                    TU,
                    Some("charlie@example.net"),
                    "Q3 Financial Report",
                    &days(9),
                    "<html-attach-001@test>",
                ),
                vec![
                    text(
                        "html",
                        "<html><body><h1>Q3 Report</h1><p>Please find the report attached.</p></body></html>",
                    ),
                    vec![
                        "Content-Type: application/pdf; name=\"report.pdf\"".into(),
                        "Content-Disposition: attachment; filename=\"report.pdf\"".into(),
                        "Content-Transfer-Encoding: base64".into(),
                        String::new(),
                        STANDARD.encode(
                            "%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n",
                        ),
                    ],
                ],
            ),
            "inbox",
            &["$seen", "$flagged"],
            9 * 24,
        ),
        Seed::new(
            "thread-starter",
            m(
                TU,
                "alice@example.com",
                ALPHA,
                days(8),
                "<thread-ALPHA-001@test>",
                "I'd like to discuss the Project Alpha timeline.",
            ),
            "folderA",
            &["$seen"],
            8 * 24,
        ),
        Seed::new(
            "thread-reply-1",
            message(Msg {
                from: "Alice Sender <alice@example.com>",
                to: TU,
                subject: RE,
                date: days(7),
                message_id: "<thread-ALPHA-002@test>",
                in_reply_to: "<thread-ALPHA-001@test>",
                references: "<thread-ALPHA-001@test>",
                body: "Sure, let's discuss. How about Thursday?".into(),
                ..Msg::default()
            }),
            "inbox",
            &[],
            7 * 24,
        ),
        Seed::new(
            "thread-reply-2",
            message(Msg {
                from: "Bob Jones <bob@example.org>",
                to: "testuser@example.com, alice@example.com",
                subject: RE,
                date: days(6),
                message_id: "<thread-ALPHA-003@test>",
                in_reply_to: "<thread-ALPHA-002@test>",
                references: "<thread-ALPHA-001@test> <thread-ALPHA-002@test>",
                body: "Thursday works for me. I'll bring the xylophone presentation materials."
                    .into(),
                ..Msg::default()
            }),
            "inbox",
            &["$answered"],
            6 * 24,
        ),
        Seed::new(
            "multi-mailbox",
            m(
                "David Cross <david@example.com>",
                TU,
                "Cross-filed document",
                days(5),
                "<multi-mb-001@test>",
                "This document should appear in multiple folders.",
            ),
            "inbox+folderA",
            &["$seen"],
            5 * 24,
        ),
        Seed::new(
            "large-email",
            m(
                "Eve Large <eve@example.com>",
                TU,
                "Detailed analysis with data",
                days(4),
                "<large-001@test>",
                &format!(
                    "Start of analysis. {}End of analysis.",
                    "This is a detailed paragraph of analysis text that covers various topics. "
                        .repeat(700)
                ),
            ),
            "folderB",
            &[],
            4 * 24,
        ),
        Seed::new(
            "html-only",
            multipart(
                "alternative",
                "----=_Alt_001_boundary",
                head(
                    "Frank Newsletter <frank@example.com>",
                    TU,
                    None,
                    "Newsletter: Weekly Digest",
                    &days(3),
                    "<html-only-001@test>",
                ),
                vec![
                    text("plain", "Weekly Digest - plain text version"),
                    text(
                        "html",
                        "<html><body><h1>Weekly Digest</h1><p>Here is your <b>weekly digest</b> of news.</p><img src=\"cid:image1\"/></body></html>",
                    ),
                ],
            ),
            "inbox",
            &["$seen"],
            3 * 24,
        ),
        Seed::new(
            "no-subject",
            m(
                "Grace Minimal <grace@example.com>",
                TU,
                "",
                days(2),
                "<no-subj-001@test>",
                "This message has no subject.",
            ),
            "inbox",
            &["$seen"],
            2 * 24,
        ),
        Seed::new(
            "custom-keywords",
            m(
                "Henry Tags <henry@example.com>",
                TU,
                "Tagged message",
                days(1),
                "<custom-kw-001@test>",
                "This message has custom keywords applied.",
            ),
            "inbox",
            &["$seen", "$forwarded", "custom_label"],
            24,
        ),
        Seed::new(
            "very-old",
            m(
                "Iris Archive <iris@example.com>",
                TU,
                "Archived correspondence",
                days(30),
                "<old-001@test>",
                "This is an old archived email from a month ago.",
            ),
            "folderA",
            &["$seen"],
            30 * 24,
        ),
    ]
}
