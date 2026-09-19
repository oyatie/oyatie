// The capability matrix: every advertised token or session entry names the
// test(s) proving it. A proof is a test function in this crate's tests/ tree
// (pinned upstream oracles run under Buck; authored tests run under cargo);
// the matrix test checks each named function exists in the named file.
pub struct Proof {
    pub file: &'static str,
    pub source: &'static str,
    pub needle: &'static str,
}
pub struct Row {
    pub token: &'static str,
    pub proof: &'static [Proof],
}
#[macro_export]
macro_rules! proof {
    ($file:literal, $needle:literal) => {
        Proof {
            file: $file,
            source: include_str!(concat!("../", $file)),
            needle: concat!("fn ", $needle),
        }
    };
}
#[macro_export]
macro_rules! row {
    ($token:literal, [$(($file:literal, $needle:literal)),+ $(,)?]) => {
        Row { token: $token, proof: &[$(proof!($file, $needle)),+] }
    };
}

/// Tokens advertised in every state and on every transport.
pub const IMAP_ALWAYS: &[Row] = &[
    row!(
        "IMAP4rev1",
        [
            ("stalwart/core_tests.rs", "upstream_imap_basic"),
            ("stalwart/core_tests.rs", "upstream_imap_fetch"),
            ("stalwart/core_tests.rs", "upstream_imap_store")
        ]
    ),
    row!("ID", [("stalwart/core_tests.rs", "upstream_imap_basic")]),
    row!(
        "UIDPLUS",
        [
            ("stalwart/thread.rs", "upstream_imap_append"),
            ("stalwart/core_tests.rs", "upstream_imap_copy_move"),
            ("stalwart/core_tests.rs", "upstream_imap_idle")
        ]
    ),
    row!(
        "MOVE",
        [
            ("stalwart/core_tests.rs", "upstream_imap_copy_move"),
            (
                "wire.rs",
                "imap_hierarchy_uses_the_same_mailboxes_and_preserves_uids_on_move"
            )
        ]
    ),
    row!(
        "ENABLE",
        [
            ("stalwart/condstore.rs", "upstream_imap_condstore"),
            ("imap_uidonly.rs", "upstream_imap_uidonly")
        ]
    ),
    row!(
        "UTF8=ACCEPT",
        [
            (
                "wire/mailbox_encoding.rs",
                "enabled_utf8_mailboxes_roundtrip_and_search_rejects_a_charset_parameter"
            ),
            (
                "wire/fetch_extensions.rs",
                "enable_utf8_is_authenticated_additive_and_scoped_to_one_session"
            )
        ]
    ),
    row!(
        "BINARY",
        [
            ("stalwart/core_tests.rs", "upstream_imap_fetch"),
            ("imap_structure_stalwart.rs", "upstream_imap_body_structure")
        ]
    ),
    row!(
        "PREVIEW",
        [
            ("stalwart/core_tests.rs", "upstream_imap_fetch"),
            (
                "wire/fetch_extensions.rs",
                "preview_is_utf8_plain_text_bounded_and_supports_lazy_without_marking_seen"
            )
        ]
    ),
    row!(
        "ESEARCH",
        [
            ("stalwart/search.rs", "upstream_imap_search"),
            (
                "wire/sort.rs",
                "esort_reports_oracle_numeric_extrema_and_save_reuses_matching_identities"
            )
        ]
    ),
    row!(
        "SEARCHRES",
        [
            ("stalwart/core_tests.rs", "upstream_imap_fetch"),
            ("stalwart/core_tests.rs", "upstream_imap_store"),
            (
                "wire/sort.rs",
                "esort_reports_oracle_numeric_extrema_and_save_reuses_matching_identities"
            )
        ]
    ),
    row!(
        "OBJECTID+",
        [
            ("imap_objectid.rs", "upstream_imap_objectid"),
            (
                "wire/fetch_extensions.rs",
                "object_ids_survive_copy_move_keyword_changes_and_late_thread_bridges"
            )
        ]
    ),
    row!(
        "IDLE",
        [
            ("stalwart/core_tests.rs", "upstream_imap_idle"),
            (
                "wire/idle.rs",
                "idle_announces_new_mail_flags_and_renumbered_expunge_without_client_polling"
            )
        ]
    ),
    row!(
        "SORT",
        [
            ("stalwart/search.rs", "upstream_imap_search"),
            (
                "wire/sort.rs",
                "sort_all_standard_keys_use_oracle_index_order_and_preserve_flags"
            )
        ]
    ),
    row!(
        "ESORT",
        [
            ("stalwart/search.rs", "upstream_imap_search"),
            (
                "wire/sort.rs",
                "esort_reports_oracle_numeric_extrema_and_save_reuses_matching_identities"
            )
        ]
    ),
    row!(
        "LITERAL+",
        [
            ("stalwart/thread.rs", "upstream_imap_append"),
            (
                "wire/literals.rs",
                "append_literal_mailbox_and_nonsync_body_commit_before_ack"
            )
        ]
    ),
    row!(
        "NAMESPACE",
        [
            (
                "wire/session.rs",
                "namespace_and_unselect_leave_deleted_messages_intact_and_clear_saved_selection"
            ),
            (
                "wire/session.rs",
                "unselect_readonly_preserves_deleted_and_namespace_rechecks_revocation"
            )
        ]
    ),
    row!(
        "UNSELECT",
        [
            ("stalwart/core_tests.rs", "upstream_imap_mailbox"),
            ("stalwart/core_tests.rs", "upstream_imap_copy_move"),
            (
                "wire/session.rs",
                "namespace_and_unselect_leave_deleted_messages_intact_and_clear_saved_selection"
            )
        ]
    ),
    row!(
        "MULTIAPPEND",
        [(
            "wire/multiappend.rs",
            "multiappend_mixed_literals_commit_once_with_distinct_metadata_and_exact_octets"
        )]
    ),
    row!(
        "CONDSTORE",
        [("stalwart/condstore.rs", "upstream_imap_condstore")]
    ),
    row!(
        "QRESYNC",
        [
            ("stalwart/condstore.rs", "upstream_imap_condstore"),
            ("imap_uidonly.rs", "upstream_imap_uidonly")
        ]
    ),
    row!(
        "THREAD=REFERENCES",
        [("stalwart/thread.rs", "upstream_imap_thread")]
    ),
    row!(
        "THREAD=ORDEREDSUBJECT",
        [(
            "wire/thread.rs",
            "thread_groups_shared_conversations_filters_queries_and_keeps_uid_identity"
        )]
    ),
    row!(
        "LIST-EXTENDED",
        [("stalwart/core_tests.rs", "upstream_imap_mailbox")]
    ),
    row!(
        "LIST-STATUS",
        [("stalwart/core_tests.rs", "upstream_imap_copy_move")]
    ),
    row!(
        "CHILDREN",
        [("stalwart/core_tests.rs", "upstream_imap_mailbox")]
    ),
    row!(
        "SPECIAL-USE",
        [("stalwart/core_tests.rs", "upstream_imap_mailbox")]
    ),
    row!(
        "CREATE-SPECIAL-USE",
        [("stalwart/core_tests.rs", "upstream_imap_mailbox")]
    ),
];

#[path = "imap_modes.rs"]
mod imap_modes;
#[path = "jmap.rs"]
mod jmap;
pub use imap_modes::{IMAP_AUTHENTICATED, IMAP_ENCRYPTED, IMAP_PLAINTEXT, IMAP_STARTTLS};
pub use jmap::{JMAP_ACCOUNT_CAPABILITIES, JMAP_CAPABILITIES, JMAP_QUERY_SORTS, JMAP_URLS};
