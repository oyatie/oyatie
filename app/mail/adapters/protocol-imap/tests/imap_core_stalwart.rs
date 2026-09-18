#![cfg(feature = "upstream-tests")]
// Unchanged pinned upstream suites compile only into this harness. Crate
// aliases resolve the upstream `imap_proto`, `imap` and `directory` paths.
extern crate self as directory;
extern crate self as imap;
extern crate self as imap_proto;
pub use client::{AssertResult, ImapConnection, ResponseType, Type};
pub use fixture::Credentials;
#[path = "stalwart/core_client.rs"]
mod client;
#[path = "stalwart/core_fixture.rs"]
mod fixture;
#[path = "stalwart/core_tests.rs"]
mod tests;
const REVISION: &str = "474dd0229cb20cf513036619781ed97bd8073c3f";
pub mod op {
    pub mod list {
        /// The upstream pattern-matching unit test runs against the server's
        /// LIST matcher.
        pub fn matches_pattern(patterns: &[String], mailbox: &str) -> bool {
            patterns
                .iter()
                .any(|pattern| mail_protocol_imap::list_matches(pattern, mailbox))
        }
    }
}
mod utils {
    pub mod server {
        pub use crate::fixture::TestServer;
    }
    pub mod smtp {
        pub use crate::fixture::SmtpConnection;
    }
}
mod suite {
    pub use crate::{AssertResult, ImapConnection, Type};
    pub fn resources_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("STALWART_IMAP_APPEND_SOURCE"))
            .parent()
            .unwrap()
            .join("../../resources/imap")
    }
    pub mod append {
        include!(env!("STALWART_IMAP_APPEND_SOURCE"));
    }
    pub mod basic {
        include!(env!("STALWART_IMAP_BASIC_SOURCE"));
    }
    pub mod store {
        include!(env!("STALWART_IMAP_STORE_SOURCE"));
    }
    pub mod fetch {
        include!(env!("STALWART_IMAP_FETCH_SOURCE"));
    }
    pub mod copy_move {
        include!(env!("STALWART_IMAP_COPY_MOVE_SOURCE"));
    }
    pub mod idle {
        include!(env!("STALWART_IMAP_IDLE_SOURCE"));
    }
    pub mod mailbox {
        include!(env!("STALWART_IMAP_MAILBOX_SOURCE"));
    }
}
