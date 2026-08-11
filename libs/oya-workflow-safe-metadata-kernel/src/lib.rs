//! Workflow safe-metadata kernel.
//!
//! Pure, zero-dependency Group-A denylist for detecting raw secret material in
//! metadata refs. Group-B token-shape patterns (API keys, bearer headers, etc.)
//! are intentionally out of scope here and land in FANOUT-04 migration.

/// Returns `true` when `value` carries Group-A raw secret material.
///
/// Group-A covers assignment-shaped secret payloads, PEM block markers, and
/// common PEM-adjacent phrases. Matching is ASCII case-insensitive.
pub fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    contains_secret_assignment(&lower)
        || lower.contains("-----begin")
        || lower.contains("-----end")
        || lower.contains("private key")
        || contains_pem_variant(&lower)
}

/// Match `secret=` / `secret =` / `secret:` / `secret :` assignment shapes.
///
/// URI / credential refs are not assignments:
/// - `scheme:secret:name` (e.g. OpenBao) — colon immediately before `secret`
/// - `secret://...` — `secret` is the URI scheme, not an assignment key
fn contains_secret_assignment(lower: &str) -> bool {
    let bytes = lower.as_bytes();
    let needle = b"secret";
    let mut i = 0;
    while i + needle.len() <= bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            let mut j = i + needle.len();
            while j < bytes.len() && bytes[j] == b' ' {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'=' || bytes[j] == b':') {
                if bytes[j] == b':' {
                    // `*:secret:*` credential-reference form — keep as safe.
                    if i > 0 && bytes[i - 1] == b':' {
                        i += 1;
                        continue;
                    }
                    // `secret://...` URI scheme form — keep as safe.
                    if j + 1 < bytes.len() && bytes[j + 1] == b'/' {
                        i += 1;
                        continue;
                    }
                }
                return true;
            }
        }
        i += 1;
    }
    false
}

fn contains_pem_variant(lower: &str) -> bool {
    const PEM_VARIANTS: &[&str] = &[
        "begin rsa private key",
        "begin private key",
        "begin encrypted private key",
        "begin openssh private key",
        "begin ec private key",
        "begin dsa private key",
        "begin certificate",
        "begin cert request",
        "begin x509 crl",
        "begin pkcs7",
        "begin pkcs8",
    ];

    PEM_VARIANTS.iter().any(|needle| lower.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_secret_assignment_needles() {
        assert!(contains_raw_secret_material("secret=super-secret-token"));
        assert!(contains_raw_secret_material("openbao://vault/secret=abc"));
        assert!(contains_raw_secret_material("SECRET=UPPERCASE"));
        assert!(contains_raw_secret_material("secret = spaced-token"));
        assert!(contains_raw_secret_material("secret: colon-token"));
        assert!(contains_raw_secret_material("secret : spaced-colon"));
    }

    #[test]
    fn detects_pem_block_markers_and_variants() {
        assert!(contains_raw_secret_material(
            "-----BEGIN RSA PRIVATE KEY-----\nMIIB..."
        ));
        assert!(contains_raw_secret_material(
            "-----begin openssh private key-----\n..."
        ));
        assert!(contains_raw_secret_material(
            "-----BEGIN CERTIFICATE-----\nMIID..."
        ));
        assert!(contains_raw_secret_material("-----END RSA PRIVATE KEY-----"));
        assert!(contains_raw_secret_material("begin pkcs8 encrypted private key"));
    }

    #[test]
    fn detects_private_key_phrase_without_pem_header() {
        assert!(contains_raw_secret_material("my private key material"));
    }

    #[test]
    fn rejects_safe_metadata_negatives() {
        assert!(!contains_raw_secret_material("openbao://vault/kv/data/tenant/workflow/ref-001"));
        assert!(!contains_raw_secret_material("workflow-trigger-app:cloud-substrate-ref-required"));
        assert!(!contains_raw_secret_material("ten_acme/workflow/run-001"));
        assert!(!contains_raw_secret_material("sk-ref-001"));
        assert!(!contains_raw_secret_material("bearer-token-ref"));
        assert!(!contains_raw_secret_material("authorization-policy-bundle-v3"));
        assert!(!contains_raw_secret_material("api_key_ref=not-a-needle"));
        // Valid OpenBao credential reference used by execution-engine-app.
        assert!(!contains_raw_secret_material("openbao:secret:workflow-execution"));
        // Canonical secret-scheme URI used by secrets lease lifecycle.
        assert!(!contains_raw_secret_material("secret://ten_alpha/db-creds"));
    }
}
