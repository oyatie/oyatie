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

/// True when `receipt` matches the embedded golden byte-for-byte (after normalize).
#[must_use]
pub fn matches_golden(receipt: &Receipt) -> bool {
    normalize(&format_receipt(receipt)) == normalize(GOLDEN_RECEIPT_V0)
}

/// Content digest of an emitted region tree (sorted region id + bytes).
#[must_use]
pub fn emit_tree_digest(emitted: &std::collections::BTreeMap<port_engine_api::RegionId, Vec<u8>>) -> port_engine_api::Digest {
    let mut preimage = Vec::new();
    for (region, bytes) in emitted {
        preimage.extend_from_slice(region.0.as_bytes());
        preimage.push(0);
        preimage.extend_from_slice(bytes);
        preimage.push(0);
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
