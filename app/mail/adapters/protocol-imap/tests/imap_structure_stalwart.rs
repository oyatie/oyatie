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
    use std::cell::RefCell;
    use std::path::PathBuf;
    thread_local! {
        pub static CURRENT: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    }
    /// The upstream suite walks one directory and panics on its first
    /// mismatch, so the harness hands it one golden pair at a time in a
    /// private directory: every file is judged and nothing is written into
    /// the shared resources tree.
    pub fn resources_dir() -> PathBuf {
        CURRENT.with(|c| c.borrow().clone().expect("golden pair selected"))
    }
    pub mod body_structure {
        include!(env!("STALWART_IMAP_BODY_STRUCTURE_SOURCE"));
    }
}

/// Golden files whose mismatch encodes Stalwart-internal synthesis, refused
/// by decision (see the PR record): 000 renders audio/image leaves as text
/// and comment-only From as a synthetic address; 004/011 give implicit
/// message/rfc822 digest parts subtype NIL; 008 BINARY[2] returns outer
/// header bytes; 011 BINARY[] drops the epilogue. The failing set must
/// equal this set exactly.
const WAIVED: &[&str] = &["000", "004", "007", "008", "011"];

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
    let shared = std::path::Path::new(env!("STALWART_IMAP_BODY_STRUCTURE_SOURCE"))
        .parent()
        .unwrap()
        .join("../../resources/imap");
    let mut stems: Vec<String> = std::fs::read_dir(&shared)
        .unwrap()
        .filter_map(|e| {
            let path = e.unwrap().path();
            (path.extension().is_some_and(|x| x == "txt"))
                .then(|| path.file_stem().unwrap().to_string_lossy().into_owned())
        })
        .collect();
    stems.sort();
    let scratch = std::env::temp_dir().join(format!("mail-body-structure-{}", std::process::id()));
    let mut failed = Vec::new();
    for stem in &stems {
        let dir = scratch.join(stem);
        std::fs::create_dir_all(&dir).unwrap();
        for ext in ["txt", "imap"] {
            std::fs::copy(
                shared.join(format!("{stem}.{ext}")),
                dir.join(format!("{stem}.{ext}")),
            )
            .unwrap();
        }
        suite::CURRENT.with(|c| *c.borrow_mut() = Some(dir));
        let outcome = std::panic::catch_unwind(suite::body_structure::test);
        let passed = outcome.is_ok();
        println!(
            "{}",
            serde_json::json!({"suite":"tests/src/imap/body_structure.rs","golden":stem,"passed":passed,"waived":WAIVED.contains(&stem.as_str())})
        );
        if !passed {
            failed.push(stem.clone());
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
    println!(
        "{}",
        serde_json::json!({"suite":"tests/src/imap/body_structure.rs","upstream_revision":"474dd0229cb20cf513036619781ed97bd8073c3f","golden_messages":stems.len(),"passed":stems.len()-failed.len(),"failures":failed})
    );
    assert_eq!(
        failed, WAIVED,
        "golden mismatches must equal the waived set exactly"
    );
}
