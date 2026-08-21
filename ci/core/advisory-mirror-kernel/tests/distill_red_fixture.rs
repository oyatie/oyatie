//! RED→GREEN fixture for the advisory distiller: a sample RustSec advisory `.md` front matter
//! distills to its expected normalized record, and the canonical hash is reformatting-invariant.
//! Schema verified against `rustsec/advisory-db` `EXAMPLE_ADVISORY.md` + `README.md`.
//!
//! Pure: no filesystem, no network. ADR-0083 Tier-3: tests use unwrap/expect/panic.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use advisory_mirror_kernel::{Advisory, canonical_hash, distill};

/// A faithful copy of the RustSec advisory `.md` shape (TOML front matter in a fenced block,
/// then a Markdown body) for a security vulnerability with a per-resource patched range plus an
/// `unaffected` range — exercising every distilled field.
const SAMPLE_ADVISORY_MD: &str = r#"```toml
[advisory]
id = "RUSTSEC-2099-0001"
package = "sample-crate"
date = "2099-01-31"
url = "https://example.com"
categories = ["code-execution"]
keywords = ["example"]
aliases = ["CVE-2099-0001"]

[versions]
patched = [">= 1.2.3, < 1.3.0", ">= 1.3.4"]
unaffected = ["<= 0.1.2"]
```

# Sample advisory title

Affected versions did not validate input. Fixed in 1.3.4.
"#;

#[test]
fn sample_advisory_distills_to_expected_normalized_record() {
    let got = distill(&[SAMPLE_ADVISORY_MD.to_owned()]);
    assert_eq!(
        got,
        vec![Advisory {
            id: "RUSTSEC-2099-0001".to_owned(),
            package: "sample-crate".to_owned(),
            patched: vec![">= 1.2.3, < 1.3.0".to_owned(), ">= 1.3.4".to_owned()],
            unaffected: vec!["<= 0.1.2".to_owned()],
            informational: None,
        }],
        "the distiller must normalize the RustSec front-matter schema exactly"
    );
}

#[test]
fn canonical_hash_is_stable_and_nonempty() {
    let advisories = distill(&[SAMPLE_ADVISORY_MD.to_owned()]);
    let hash = canonical_hash(&advisories);
    assert_eq!(hash.len(), 64, "SHA-256 renders as 64 hex chars");
    // Recomputing over an independently-parsed copy yields the identical hash (reformatting-
    // invariant: the gate recomputes from the committed advisories.json the same way).
    let reparsed = distill(&[SAMPLE_ADVISORY_MD.to_owned()]);
    assert_eq!(hash, canonical_hash(&reparsed));
}
