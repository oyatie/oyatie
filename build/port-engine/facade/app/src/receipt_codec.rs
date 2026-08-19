//! Stable six-axis receipt encoding + golden compare (W0-B Slice 12).
//!
//! Hand-rolled key=value lines in fixed axis order — no serde (no lock absorb). Golden is a
//! hermetic package-local fixture under `src/golden-receipt-v0.txt` (neutral mini fixture only;
//! no `k8s/` corpus emission).

use port_engine_api::Receipt;
use port_engine_hash::digest_bytes;

/// Embedded golden receipt for the Slice 11/12 mini pipeline fixture.
pub const GOLDEN_RECEIPT_V0: &str = include_str!("golden-receipt-v0.txt");

/// Encode a receipt as stable ordered lines (trailing newline).
#[must_use]
pub fn format_receipt(receipt: &Receipt) -> String {
    format!(
        "pin={}\nsnapshot_digest={}\nengine_digest={}\nrulepack_digest={}\ntoolchain_digest={}\nformatter_digest={}\n",
        receipt.pin,
        receipt.snapshot_digest.0,
        receipt.engine_digest.0,
        receipt.rulepack_digest.0,
        receipt.toolchain_digest.0,
        receipt.formatter_digest.0,
    )
}

/// The value the golden records for an axis it deliberately does not pin.
pub const GOLDEN_VARIES: &str = "<varies>";

/// True when `receipt` matches the embedded golden, axis by axis.
///
/// The comparison is per axis rather than byte-for-byte because the axes do not make the same kind
/// of claim. Five of them SHOULD hold across an engine change, and pinning them catches a real
/// defect: a snapshot digest that moved while the corpus did not is a bug, and so is a formatter
/// digest that moved while the formatter did not.
///
/// `engine_digest` is different. It is a content hash of the engine's own sources, so it moves on
/// every commit that touches the engine — which is the normal case, not a defect. Pinning it would
/// mean refreshing the golden on every commit, and a golden refreshed reflexively is not a check.
/// So the golden records `<varies>` for it and this asserts its SHAPE instead, which still catches
/// the failure that matters: an axis gone empty or malformed.
#[must_use]
pub fn matches_golden(receipt: &Receipt) -> bool {
    let actual = normalize(&format_receipt(receipt));
    let expected = normalize(GOLDEN_RECEIPT_V0);

    let mut lines = actual.lines().zip(expected.lines());
    let paired = lines
        .by_ref()
        .all(|(actual, expected)| match expected.split_once('=') {
            Some((axis, GOLDEN_VARIES)) => actual.starts_with(axis) && well_formed_digest(actual),
            _ => actual == expected,
        });
    // Arity is part of the claim: an axis added or dropped must not read as a match.
    paired && actual.lines().count() == expected.lines().count()
}

/// Whether an axis line carries a digest of the expected shape.
fn well_formed_digest(line: &str) -> bool {
    let Some((_, value)) = line.split_once('=') else {
        return false;
    };
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Per-region digests, sorted by region id.
///
/// The roll-up answers "did anything change"; this answers "how much, and where". A whole-program
/// decision can touch a handful of declarations and every call site that uses them, and under one
/// digest that is indistinguishable from an accidental one-line change.
#[must_use]
pub fn region_digests(
    emitted: &std::collections::BTreeMap<port_engine_api::RegionId, Vec<u8>>,
) -> Vec<(String, String)> {
    emitted
        .iter()
        .map(|(region, bytes)| (region.0.clone(), digest_bytes(bytes).0))
        .collect()
}

/// Content digest of an emitted region tree (sorted region id + bytes).
#[must_use]
pub fn emit_tree_digest(
    emitted: &std::collections::BTreeMap<port_engine_api::RegionId, Vec<u8>>,
) -> port_engine_api::Digest {
    // LENGTH-PREFIXED, like the snapshot and engine preimages and for the same reason: a
    // separator-delimited encoding is only unambiguous while the separator cannot appear in the
    // content, and emitted source is arbitrary. Prefixing each field with its length makes the
    // encoding injective without relying on that.
    let mut preimage = Vec::new();
    for (region, bytes) in emitted {
        preimage.extend_from_slice(region.0.len().to_string().as_bytes());
        preimage.push(b':');
        preimage.extend_from_slice(region.0.as_bytes());
        preimage.extend_from_slice(bytes.len().to_string().as_bytes());
        preimage.push(b':');
        preimage.extend_from_slice(bytes);
    }
    digest_bytes(&preimage)
}

fn normalize(text: &str) -> String {
    // Accept either LF or CRLF golden checkouts; keep final newline.
    let mut body = text.replace("\r\n", "\n");
    if !body.ends_with('\n') {
        body.push('\n');
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use port_engine_api::Digest;
    use std::collections::BTreeMap;

    #[test]
    fn format_receipt_is_six_ordered_lines() {
        let r = Receipt {
            pin: "p".into(),
            snapshot_digest: Digest("s".into()),
            engine_digest: Digest("e".into()),
            rulepack_digest: Digest("r".into()),
            toolchain_digest: Digest("t".into()),
            formatter_digest: Digest("f".into()),
        };
        let text = format_receipt(&r);
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 6);
        assert!(lines[0].starts_with("pin="));
        assert!(lines[5].starts_with("formatter_digest="));
        assert!(text.ends_with('\n'));
    }

    #[test]
    fn emit_tree_digest_is_order_stable() {
        let mut a = BTreeMap::new();
        a.insert(port_engine_api::RegionId("b".into()), b"1".to_vec());
        a.insert(port_engine_api::RegionId("a".into()), b"0".to_vec());
        let mut b = BTreeMap::new();
        b.insert(port_engine_api::RegionId("a".into()), b"0".to_vec());
        b.insert(port_engine_api::RegionId("b".into()), b"1".to_vec());
        assert_eq!(emit_tree_digest(&a), emit_tree_digest(&b));
    }
}
