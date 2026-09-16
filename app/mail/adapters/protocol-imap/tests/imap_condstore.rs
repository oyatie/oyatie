#![cfg(feature = "upstream-tests")]
// Test-only upstream source retains its original assertions and fixture builders.
extern crate self as imap_proto;
pub use client::{AssertResult, ImapConnection, ResponseType, Type};
#[path = "stalwart/condstore_client.rs"]
mod client;
#[path = "stalwart/condstore.rs"]
mod condstore;
mod imap {
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
}
mod utils {
    pub mod server {
        pub struct TestServer;
        impl TestServer {
            pub async fn wait_for_tasks(&self) {}
        }
    }
}
const TOKEN: &str = "0123456789abcdef0123456789abcdef";
