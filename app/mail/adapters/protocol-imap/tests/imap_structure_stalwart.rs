#![cfg(feature = "upstream-tests")]
// The unchanged pinned BODYSTRUCTURE golden suite compiles only here. Crate
// aliases resolve its upstream-internal `email`, `imap`, `imap_proto`,
// `store` and `utils` paths to the shim; rendering comes from the server.
extern crate self as email;
extern crate self as imap;
extern crate self as imap_proto;
extern crate self as store;
extern crate self as utils;
#[path = "stalwart/structure_shim.rs"]
mod shim;
pub use shim::{Deserialize, ResponseCode, Serialize, StatusResponse};
pub mod protocol {
    pub mod fetch {
        pub use crate::shim::{BodyContents, DataItem, Section};
    }
}
pub mod op {
    pub mod fetch {
        pub use crate::shim::AsImapDataItem;
    }
}
pub mod message {
    pub mod metadata {
        pub use crate::shim::{MessageMetadata, build_metadata_contents};
    }
}
pub mod write {
    pub use crate::shim::{Archive, Archiver};
}
pub mod chained_bytes {
    pub use crate::shim::ChainedBytes;
}
mod suite {
    pub fn resources_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("STALWART_IMAP_BODY_STRUCTURE_SOURCE"))
            .parent()
            .unwrap()
            .join("../../resources/imap")
    }
    pub mod body_structure {
        include!(env!("STALWART_IMAP_BODY_STRUCTURE_SOURCE"));
    }
}

#[test]
fn upstream_imap_body_structure() {
    use sha2::{Digest, Sha256};
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(env!("STALWART_IMAP_BODY_STRUCTURE_SOURCE")))
        ),
        "115066eaf121be3f03489c6a1b2006f5e8e34c7764405d471158b338ce894bf7",
        "upstream body_structure suite changed; review before updating the baseline"
    );
    let fixtures = std::fs::read_dir(suite::resources_dir())
        .unwrap()
        .filter(|entry| {
            entry
                .as_ref()
                .unwrap()
                .path()
                .extension()
                .is_some_and(|e| e == "txt")
        })
        .count();
    println!(
        "{}",
        serde_json::json!({"suite":"tests/src/imap/body_structure.rs","upstream_revision":"474dd0229cb20cf513036619781ed97bd8073c3f","golden_messages":fixtures})
    );
    suite::body_structure::test();
}
