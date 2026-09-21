//! Agreed cross-owner surface for data-classification values.
//!
//! This provider port defines Data's classification vocabulary. Membership is
//! the sorted `src/items/` directory, rendered by `build.rs`; adding, renaming
//! or removing an item never edits this root.

#![forbid(unsafe_code)]

include!(concat!(env!("OUT_DIR"), "/classification.generated.rs"));
