#![forbid(unsafe_code)]
mod imap;
mod jmap;
mod pop;
mod sasl;
mod smtp;
mod wire;
pub use imap::{imap_session, imap_starttls_session, list_matches, render as fetch_render};
pub use jmap::jmap_router;
pub use pop::{pop_session, pop_starttls_session};
pub use sasl::oauth_bearer;
pub use smtp::{
    Authentication, AuthenticationLog, MailDns, SmtpParams, Stage, Verifier, Verify, smtp_session,
    smtp_session_with, smtp_starttls_session, smtp_starttls_session_with, smtp_tls_session_with,
    submission_session, submission_session_with,
};
