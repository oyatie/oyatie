#![cfg(feature = "upstream-tests")]
// Test-only upstream source retains its original assertions and fixture builders.
extern crate self as imap_proto;
pub use client::{AssertResult, ImapConnection, ResponseType, Type};
#[path = "stalwart/condstore_client.rs"]
mod client;
#[path = "stalwart/thread.rs"]
mod thread;
mod imap {
    pub use crate::{AssertResult, ImapConnection, Type};
    pub fn resources_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("STALWART_IMAP_APPEND_SOURCE"))
            .parent()
            .unwrap()
            .join("../../resources/imap")
    }
    pub fn expand_uid_list(list: &str) -> std::collections::BTreeSet<u32> {
        let mut result = std::collections::BTreeSet::new();
        for item in list.split(',') {
            let (a, b) = item.split_once(':').unwrap_or((item, item));
            result.extend(a.parse::<u32>().unwrap()..=b.parse::<u32>().unwrap());
        }
        result
    }
    pub mod append {
        include!(env!("STALWART_IMAP_APPEND_SOURCE"));
    }
}
mod utils {
    pub mod server {
        pub struct TestServer {
            pub server: Server,
        }
        pub struct Server;
        impl Server {
            pub fn search_store(&self) -> &Self {
                self
            }
            pub fn is_mysql(&self) -> bool {
                false
            }
        }
        impl TestServer {
            pub async fn wait_for_tasks(&self) {}
        }
    }
}
const TOKEN: &str = "0123456789abcdef0123456789abcdef";
