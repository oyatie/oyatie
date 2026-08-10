//! Encoding helpers for Kubernetes API objects rendered by this crate.
//!
//! Kubernetes kubeconfig `*-data` fields are `[]byte` values in the v1 config
//! schema. In YAML/JSON kubeconfigs those bytes are represented as standard
//! RFC 4648 base64 text. Keeping this conversion in one small module lets the
//! current model-byte PKI bridge swap in real PEM/DER bytes later without
//! changing kubeconfig rendering call sites.

const BASE64_STANDARD: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encode bytes as unwrapped standard RFC 4648 base64.
///
/// This intentionally has no PEM markers and no line wrapping: callers provide
/// the raw certificate/key bytes, and kubeconfig `*-data` fields carry the
/// base64 representation of those bytes.
pub fn base64_standard(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);

        out.push(BASE64_STANDARD[(b0 >> 2) as usize] as char);
        out.push(BASE64_STANDARD[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);

        if chunk.len() > 1 {
            out.push(BASE64_STANDARD[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }

        if chunk.len() > 2 {
            out.push(BASE64_STANDARD[(b2 & 0b0011_1111) as usize] as char);
        } else {
            out.push('=');
        }
    }

    out
}

/// Encode bytes for a kubeconfig `certificate-authority-data`,
/// `client-certificate-data`, or `client-key-data` field.
pub fn kubeconfig_data(data: &[u8]) -> String {
    base64_standard(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_standard_matches_rfc_4648_vectors() {
        assert_eq!(base64_standard(b""), "");
        assert_eq!(base64_standard(b"f"), "Zg==");
        assert_eq!(base64_standard(b"fo"), "Zm8=");
        assert_eq!(base64_standard(b"foo"), "Zm9v");
        assert_eq!(base64_standard(b"foob"), "Zm9vYg==");
        assert_eq!(base64_standard(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_standard(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn kubeconfig_data_base64_encodes_model_bytes_without_raw_markers() {
        let model = b"KUBEROS-MODEL-CERTIFICATE\nserial=1\n";
        let encoded = kubeconfig_data(model);

        assert_eq!(encoded, base64_standard(model));
        assert!(!encoded.contains("KUBEROS-MODEL-CERTIFICATE"));
        assert!(!encoded.contains('\n'));
        assert!(!encoded.contains("-----BEGIN"));
    }
}
