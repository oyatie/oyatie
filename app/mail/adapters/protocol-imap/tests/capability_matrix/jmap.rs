// Matrix rows: see rows.rs for the `Row`/`Proof` shape and the proof rule.
use super::{Proof, Row};
use crate::{proof, row};

/// JMAP Session `capabilities` and per-account `accountCapabilities`.
pub const JMAP_CAPABILITIES: &[Row] = &[
    row!(
        "urn:ietf:params:jmap:core",
        [("stalwart.rs", "upstream_jmap_compliance")]
    ),
    row!(
        "urn:ietf:params:jmap:mail",
        [
            ("stalwart.rs", "upstream_jmap_compliance"),
            ("jmap_query_stalwart.rs", "upstream_jmap_email_query")
        ]
    ),
    row!(
        "urn:ietf:params:jmap:submission",
        [(
            "submission_stalwart.rs",
            "unchanged_upstream_submission_suite"
        )]
    ),
    row!(
        "urn:ietf:params:jmap:vacationresponse",
        [(
            "jmap_vacation_stalwart.rs",
            "unchanged_upstream_vacation_suite"
        )]
    ),
];
/// Core is a server-wide capability; the three others are per account.
pub const JMAP_ACCOUNT_CAPABILITIES: &[&str] = &[
    "urn:ietf:params:jmap:mail",
    "urn:ietf:params:jmap:submission",
    "urn:ietf:params:jmap:vacationresponse",
];
/// Session URLs. `eventSourceUrl` stays: RFC 8620 §2 requires the member and
/// the pinned core suite (`core/session-event-source-url`) asserts it, while
/// no test proves an EventSource route (none exists yet).
pub const JMAP_URLS: &[Row] = &[
    row!("apiUrl", [("stalwart.rs", "upstream_jmap_compliance")]),
    row!("downloadUrl", [("stalwart.rs", "upstream_jmap_compliance")]),
    row!("uploadUrl", [("stalwart.rs", "upstream_jmap_compliance")]),
    row!(
        "eventSourceUrl",
        [("stalwart.rs", "upstream_jmap_compliance")]
    ),
];
/// `emailQuerySortOptions` proven by the pinned Email/query suite.
pub const JMAP_QUERY_SORTS: &[Row] = &[
    row!(
        "receivedAt",
        [("jmap_query_stalwart.rs", "upstream_jmap_email_query")]
    ),
    row!(
        "sentAt",
        [("jmap_query_stalwart.rs", "upstream_jmap_email_query")]
    ),
    row!(
        "size",
        [("jmap_query_stalwart.rs", "upstream_jmap_email_query")]
    ),
    row!(
        "from",
        [("jmap_query_stalwart.rs", "upstream_jmap_email_query")]
    ),
    row!(
        "to",
        [("jmap_query_stalwart.rs", "upstream_jmap_email_query")]
    ),
    row!(
        "subject",
        [("jmap_query_stalwart.rs", "upstream_jmap_email_query")]
    ),
    row!(
        "hasKeyword",
        [("jmap_query_stalwart.rs", "upstream_jmap_email_query")]
    ),
];
