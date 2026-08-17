//! # oya-advisory-mirror-kernel
//!
//! Pure, I/O-free distiller for RustSec advisory `.md` files plus a deterministic
//! content hash over the distilled set. This is the owned, hermetic core of the
//! supply-chain advisory pipeline (replaces the reverted #974 shell `cargo-audit`/`cargo-deny`):
//! the network-and-clock-bearing reconciler/producer feeds raw advisory text in, the
//! hermetic `cloud-ci-supply-chain-audit` gate consumes the vendored snapshot out.
//!
//! ## Why owned, not the `rustsec`/`cargo-lock` crates
//! Those crates pull `git2` → `libgit2-sys` (a C dependency, a rust-purity strike) and a
//! network-fetching index, which would defeat both hermeticity and the zero-non-Rust posture.
//! The RustSec advisory format is a small, stable TOML-front-matter `.md` (see
//! `EXAMPLE_ADVISORY.md` / `README.md` in `rustsec/advisory-db`), so a `toml`-only parser
//! over passed-in text is sufficient and adds ZERO new crates to `Cargo.lock`.
//!
//! ## Contract
//! - [`distill`] `(&[String]) -> Vec<Advisory>` is PURE: it parses each advisory's TOML
//!   front matter into a normalized [`Advisory`] record. It operates only on the text handed
//!   to it — no filesystem, no network, no clock. Non-advisory / unparseable / WITHDRAWN
//!   inputs are dropped (a withdrawn advisory was retracted and must never block).
//! - [`canonical_hash`] `(&[Advisory]) -> String` is a deterministic, order-independent
//!   content hash of the distilled set. The producer stamps it into the mirror manifest; the
//!   gate recomputes it from the committed `advisories.json` and fails closed on a mismatch
//!   (a desynced / corrupted mirror).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// The mirror schema string the producer stamps into `mirror-manifest.json` and the gate
/// validates. Bump on any breaking change to the `Advisory` shape or the hash pre-image.
pub const MIRROR_SCHEMA: &str = "oya-advisory-mirror/v1";

/// A normalized RustSec advisory record — the subset the supply-chain gate matches on.
///
/// `patched` / `unaffected` are the raw semver `VersionReq` strings VERBATIM from the
/// advisory's `[versions]` table (the gate parses them with `semver`, fail-closed on a
/// malformed req). `informational` is `Some("unmaintained")` / `Some("unsound")` /
/// `Some("notice")` for an informational advisory, `None` for a security vulnerability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Advisory {
    /// The `RUSTSEC-YYYY-NNNN` identifier.
    pub id: String,
    /// The affected crate name (matched against locked crate names).
    pub package: String,
    /// `[versions] patched` — semver `VersionReq` strings; a locked version satisfying ANY
    /// of these is NOT affected. Empty for an unmaintained advisory (no fix exists).
    #[serde(default)]
    pub patched: Vec<String>,
    /// `[versions] unaffected` — semver `VersionReq` strings; a locked version satisfying ANY
    /// of these was never affected.
    #[serde(default)]
    pub unaffected: Vec<String>,
    /// `[advisory] informational` (`unmaintained` / `unsound` / `notice`), or `None` for a
    /// security vulnerability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub informational: Option<String>,
}

/// Distill raw RustSec advisory `.md` texts into normalized [`Advisory`] records.
///
/// PURE: operates only on the passed-in text. Each text is the full `.md` (TOML front matter
/// in a fenced code block, followed by the Markdown description). Inputs that are not a
/// parseable advisory (no `[advisory]` table, missing `id`/`package`) or that are WITHDRAWN
/// (retracted) are dropped — they must never produce a blocking finding.
pub fn distill(advisory_md_texts: &[String]) -> Vec<Advisory> {
    advisory_md_texts
        .iter()
        .filter_map(|text| parse_advisory(text))
        .collect()
}

/// Parse one advisory `.md` text into an [`Advisory`], or `None` if it is not a live advisory.
fn parse_advisory(text: &str) -> Option<Advisory> {
    let front_matter = extract_front_matter(text)?;
    let doc: toml::Value = toml::from_str(&front_matter).ok()?;
    let advisory = doc.get("advisory")?;

    // A withdrawn advisory was retracted by RustSec — drop it (never block on a withdrawn id).
    if advisory.get("withdrawn").is_some() {
        return None;
    }

    let id = advisory.get("id").and_then(toml::Value::as_str)?.to_owned();
    let package = advisory
        .get("package")
        .and_then(toml::Value::as_str)?
        .to_owned();
    let informational = advisory
        .get("informational")
        .and_then(toml::Value::as_str)
        .map(str::to_owned);

    let versions = doc.get("versions");
    let patched = string_array(versions, "patched");
    let unaffected = string_array(versions, "unaffected");

    Some(Advisory {
        id,
        package,
        patched,
        unaffected,
        informational,
    })
}

/// Extract the TOML front matter from a RustSec advisory `.md`: the contents of the FIRST
/// fenced code block (```` ```toml ```` … ```` ``` ```` or a bare ```` ``` ````) at the top of
/// the file. Returns `None` if there is no fenced block.
fn extract_front_matter(text: &str) -> Option<String> {
    let mut started = false;
    let mut buf = String::new();
    for line in text.lines() {
        let is_fence = line.trim_start().starts_with("```");
        if !started {
            if is_fence {
                started = true;
            }
            continue;
        }
        if is_fence {
            return Some(buf);
        }
        buf.push_str(line);
        buf.push('\n');
    }
    None
}

/// Collect a string array from `table[key]` (e.g. `[versions] patched`), preserving source
/// order. A missing / non-array value yields an empty `Vec`.
fn string_array(table: Option<&toml::Value>, key: &str) -> Vec<String> {
    table
        .and_then(|t| t.get(key))
        .and_then(toml::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// A deterministic, order-independent content hash of the distilled advisory set.
///
/// The pre-image is built from the advisories sorted by `id` (deduped by `id`), with each
/// record's `id`, `package`, `informational`, `patched`, and `unaffected` joined by unit/record
/// separators. The producer stamps this into `mirror-manifest.json#content_hash`; the gate
/// recomputes it from the committed `advisories.json` and fails closed (`SCA-MIRROR-MALFORMED`)
/// on a mismatch.
///
/// SHA-256 collision-resistant integrity anchor. Detects accidental corruption, a desynced
/// regeneration of the committed mirror, AND provides a tamper-evident content seal.
/// The gate fails closed (`SCA-MIRROR-MALFORMED`) on any mismatch.
pub fn canonical_hash(advisories: &[Advisory]) -> String {
    let mut sorted: Vec<&Advisory> = advisories.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    sorted.dedup_by(|a, b| a.id == b.id);

    let mut pre = String::new();
    for advisory in sorted {
        pre.push_str(&advisory.id);
        pre.push('\u{1f}');
        pre.push_str(&advisory.package);
        pre.push('\u{1f}');
        pre.push_str(advisory.informational.as_deref().unwrap_or(""));
        pre.push('\u{1f}');
        pre.push_str(&advisory.patched.join(","));
        pre.push('\u{1f}');
        pre.push_str(&advisory.unaffected.join(","));
        pre.push('\u{1e}');
    }
    sha256_hex(pre.as_bytes())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    // The real quinn-proto advisory front matter (RUSTSEC-2026-0185) — the gate's self-test
    // target. A security vulnerability (no `informational`) with a single patched range.
    const QUINN_ADVISORY: &str = r#"```toml
[advisory]
id = "RUSTSEC-2026-0185"
package = "quinn-proto"
date = "2026-06-22"
url = "https://github.com/quinn-rs/quinn/pull/2694"
categories = ["denial-of-service"]
cvss = "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:N/A:H"
keywords = ["oom"]
aliases = ["GHSA-4w2j-m93h-cj5j"]

[versions]
patched = [">= 0.11.15"]
```

# quinn-proto OOM

Long description here.
"#;

    const UNMAINTAINED_ADVISORY: &str = r#"```toml
[advisory]
id = "RUSTSEC-2024-0436"
package = "paste"
date = "2024-10-07"
url = "https://example.com"
informational = "unmaintained"

[versions]
patched = []
```

paste is unmaintained.
"#;

    const WITHDRAWN_ADVISORY: &str = r#"```toml
[advisory]
id = "RUSTSEC-2020-9999"
package = "retracted-crate"
withdrawn = "2020-02-01"

[versions]
patched = [">= 1.0.0"]
```

This advisory was withdrawn.
"#;

    #[test]
    fn distills_security_vulnerability_to_expected_record() {
        let got = distill(&[QUINN_ADVISORY.to_owned()]);
        assert_eq!(
            got,
            vec![Advisory {
                id: "RUSTSEC-2026-0185".to_owned(),
                package: "quinn-proto".to_owned(),
                patched: vec![">= 0.11.15".to_owned()],
                unaffected: vec![],
                informational: None,
            }]
        );
    }

    #[test]
    fn distills_unmaintained_with_empty_patched() {
        let got = distill(&[UNMAINTAINED_ADVISORY.to_owned()]);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].informational.as_deref(), Some("unmaintained"));
        assert!(got[0].patched.is_empty());
    }

    #[test]
    fn drops_withdrawn_advisories() {
        assert!(distill(&[WITHDRAWN_ADVISORY.to_owned()]).is_empty());
    }

    #[test]
    fn drops_non_advisory_text() {
        // The EXAMPLE_ADVISORY template uses a placeholder id but is still a parseable advisory;
        // a plain Markdown file with no fenced TOML is not.
        assert!(distill(&["# Just a readme\n\nNo front matter here.".to_owned()]).is_empty());
    }

    #[test]
    fn canonical_hash_is_order_independent() {
        let a = distill(&[QUINN_ADVISORY.to_owned(), UNMAINTAINED_ADVISORY.to_owned()]);
        let b = distill(&[UNMAINTAINED_ADVISORY.to_owned(), QUINN_ADVISORY.to_owned()]);
        assert_eq!(canonical_hash(&a), canonical_hash(&b));
    }

    #[test]
    fn canonical_hash_changes_on_tamper() {
        let original = distill(&[QUINN_ADVISORY.to_owned()]);
        let mut tampered = original.clone();
        // Simulate a silently-weakened patched range (the corruption SCA-MIRROR-MALFORMED catches).
        tampered[0].patched = vec![">= 0.0.0".to_owned()];
        assert_ne!(canonical_hash(&original), canonical_hash(&tampered));
    }
}
