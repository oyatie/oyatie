//! ADR-0704 / W0-B: emit must refuse the corpus destination root.
//!
//! Scanned over the WHOLE crate. The fence used to read `include_str!("lib.rs")` alone, which was
//! complete only while the crate was one file — a `mod other;` compiles a file the scan never
//! reads. `port-engine-kernel` closed that hole by REFUSING `mod` at compile time; this adapter
//! cannot, because it is modular. So the scanned set is enumerated and then PROVEN to be the whole
//! of `src/`.

use std::collections::BTreeSet;
use std::path::Path;

/// Every production source of this crate, read at compile time.
const PRODUCTION_SOURCES: &[(&str, &str)] = &[
    ("canary.rs", include_str!("../src/canary.rs")),
    ("error.rs", include_str!("../src/error.rs")),
    ("lib.rs", include_str!("../src/lib.rs")),
    ("materialize.rs", include_str!("../src/materialize.rs")),
];

/// The enumeration must BE the directory, not a subset somebody once curated.
#[test]
fn scanned_sources_are_the_whole_crate() {
    let candidates = [
        option_env!("CARGO_MANIFEST_DIR").map(|dir| Path::new(dir).join("src")),
        Some(Path::new("src").to_path_buf()),
    ];
    let src = candidates
        .into_iter()
        .flatten()
        .find(|path| path.is_dir())
        .expect("this crate's src/ must be locatable — a fence that cannot look has not looked");

    let on_disk: BTreeSet<String> = std::fs::read_dir(&src)
        .unwrap_or_else(|err| panic!("src/ must be readable to be scanned: {err}"))
        .map(|entry| entry.expect("readable dir entry").file_name())
        .filter_map(|name| name.to_str().map(ToOwned::to_owned))
        .filter(|name| name.ends_with(".rs"))
        .collect();

    let scanned: BTreeSet<String> = PRODUCTION_SOURCES
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect();

    assert_eq!(
        scanned, on_disk,
        "a source file exists that no architecture fence reads — add it to PRODUCTION_SOURCES"
    );
}

/// The bulk-emit hard stop, asserted over the sources rather than only over a call.
#[test]
fn a_production_source_refuses_the_corpus_destination() {
    let destination = ["k", "8", "s"].concat();
    let refusing = PRODUCTION_SOURCES
        .iter()
        .any(|(_, source)| source.contains(&destination));
    assert!(
        refusing,
        "no production source mentions the `{destination}` destination — the bulk-emit refusal \
         must be present in the code, not only in the commit message"
    );
}
