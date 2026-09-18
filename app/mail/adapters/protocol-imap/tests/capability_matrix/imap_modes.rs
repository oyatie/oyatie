// Matrix rows: see rows.rs for the `Row`/`Proof` shape and the proof rule.
use super::{Proof, Row};
use crate::{proof, row};

/// Tokens added once a session is authenticated.
pub const IMAP_AUTHENTICATED: &[Row] = &[
    row!("UIDONLY", [("imap_uidonly.rs", "upstream_imap_uidonly")]),
    row!(
        "UNAUTHENTICATE",
        [(
            "stalwart/uidonly_runtime.rs",
            "unauthenticate_clears_identity_and_saved_results_before_another_tenant_login"
        )]
    ),
];

/// Tokens on an encrypted transport.
pub const IMAP_ENCRYPTED: &[Row] = &[
    row!(
        "AUTH=PLAIN",
        [
            ("stalwart/core_tests.rs", "upstream_imap_basic"),
            (
                "wire/authenticate.rs",
                "sasl_plain_supports_initial_response_and_continuation_with_same_read_policy"
            )
        ]
    ),
    row!(
        "SASL-IR",
        [(
            "wire/authenticate.rs",
            "sasl_plain_supports_initial_response_and_continuation_with_same_read_policy"
        )]
    ),
];

/// Tokens on a plaintext transport that can upgrade.
pub const IMAP_STARTTLS: &[Row] = &[
    row!(
        "STARTTLS",
        [
            (
                "wire/starttls.rs",
                "imap_starttls_rejects_buffered_plaintext_without_invoking_upgrade"
            ),
            (
                "wire/fetch_extensions.rs",
                "rejected_plaintext_enable_cannot_cross_the_starttls_session_boundary"
            )
        ]
    ),
    row!(
        "LOGINDISABLED",
        [("wire.rs", "imap_refuses_credentials_on_plaintext_transport")]
    ),
];

/// Tokens on a plaintext transport without STARTTLS.
pub const IMAP_PLAINTEXT: &[Row] = &[row!(
    "LOGINDISABLED",
    [("wire.rs", "imap_refuses_credentials_on_plaintext_transport")]
)];
