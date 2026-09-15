// Upstream assertions are included unchanged in this test-only executable.
extern crate self as imap_proto;
pub use search_client::ResponseType;

#[path = "stalwart/search.rs"]
mod search;
#[path = "stalwart/search_client.rs"]
mod search_client;

mod utils {
    pub mod server {
        pub use crate::search_client::TestServer;
    }
}

const REVISION: &str = "474dd0229cb20cf513036619781ed97bd8073c3f";
const TOKEN: &str = "0123456789abcdef0123456789abcdef";
