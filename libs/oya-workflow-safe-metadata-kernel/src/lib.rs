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
    lower.contains("secret=")
        || lower.contains("-----begin")
        || lower.contains("-----end")
        || lower.contains("private key")
        || contains_pem_variant(&lower)
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
    }
}
