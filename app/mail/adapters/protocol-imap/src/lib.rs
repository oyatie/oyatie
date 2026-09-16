#![forbid(unsafe_code)]
mod imap;
mod jmap;
mod pop;
mod sasl;
mod smtp;
mod wire;
pub use imap::{imap_session, imap_starttls_session};
pub use jmap::jmap_router;
pub use pop::{pop_session, pop_starttls_session};
pub use smtp::{smtp_session, smtp_starttls_session, submission_session};
