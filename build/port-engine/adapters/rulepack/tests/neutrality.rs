//! Neutrality fence: no production source of this adapter may carry corpus vocabulary.
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
    ("error.rs", include_str!("../src/error.rs")),
    ("lib.rs", include_str!("../src/lib.rs")),
    ("pack.rs", include_str!("../src/pack.rs")),
    ("policy.rs", include_str!("../src/policy.rs")),
    ("rule.rs", include_str!("../src/rule.rs")),
    ("seams.rs", include_str!("../src/seams.rs")),
    ("wire.rs", include_str!("../src/wire.rs")),
    ("sources.rs", include_str!("../src/sources.rs")),
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
        "a source file exists that no neutrality fence reads — add it to PRODUCTION_SOURCES"
    );
}

/// A CANARY SET, not a decision procedure: no finite list can decide "corpus-specific". The
/// structural properties carry the weight (seam types name no corpus type; rule semantics live in
/// data). This is the cheap backstop on top.
#[test]
fn no_production_source_carries_corpus_vocabulary() {
    let needles = [
        ["k", "8", "s", ".", "i", "o"].concat(),
        ["k", "ube", "rnete", "s"].concat(),
        ["k", "ube", "let"].concat(),
        ["api", "machin", "ery"].concat(),
    ];

    for (name, source) in PRODUCTION_SOURCES {
        for needle in &needles {
            assert!(
                !source.contains(needle),
                "{name} must not embed corpus needle `{needle}`"
            );
        }
    }
}
