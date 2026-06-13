//! Deterministic length-delimited (de)serialization of a trustd
//! [`Certificate`] — the adapter-owned stand-in for X.509 DER.
//!
//! The trustd domain models a certificate as an in-memory value with no
//! ASN.1/DER (its "DER" is a hex stand-in, its signatures keyed hashes). To
//! satisfy the byte-oriented [`SvidVerifier::verify_peer`] port (whose W5
//! destination is real rustls-delivered DER), this module gives the adapter a
//! self-contained, total, round-tripping codec it owns end-to-end. It is NOT a
//! real DER parser and is never fed untrusted real certificates — the real
//! transport path is the deferred slice-1b (ADR-0561).
//!
//! Framing: every field is a `u32` big-endian length prefix followed by that
//! many bytes (strings UTF-8, lists length-prefixed by count). Decode is total
//! and fail-closed: any truncation/overrun yields `Err`.
//!
//! [`SvidVerifier::verify_peer`]: oya_identity_workload_svid_kernel::SvidVerifier::verify_peer

use oya_cloud_os_trustd_domain::certificate::{CertUsage, Certificate};
use oya_cloud_os_trustd_domain::x509::{DistinguishedName, SubjectAltNames, Validity};

/// Magic prefix so a non-leaf byte blob is rejected up front.
const MAGIC: &[u8; 4] = b"TSV1";

fn put_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    // Length-prefixed (u32 BE). A field longer than u32::MAX is unrepresentable
    // by construction in this shape model; callers never produce one, so the
    // saturating cast is dead-but-safe rather than a panic site.
    let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(&bytes[..len as usize]);
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    put_bytes(out, s.as_bytes());
}

fn put_list_strs(out: &mut Vec<u8>, items: &[String]) {
    let count = u32::try_from(items.len()).unwrap_or(u32::MAX);
    out.extend_from_slice(&count.to_be_bytes());
    for item in items.iter().take(count as usize) {
        put_str(out, item);
    }
}

/// Encode a trustd [`Certificate`] into the adapter's leaf byte form.
#[must_use]
pub fn encode(cert: &Certificate) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&cert.serial.to_be_bytes());
    put_str(&mut out, &cert.subject.common_name);
    put_list_strs(&mut out, &cert.subject.organizations);
    put_str(&mut out, &cert.issuer.common_name);
    put_list_strs(&mut out, &cert.issuer.organizations);
    out.extend_from_slice(&cert.validity.not_before.to_be_bytes());
    out.extend_from_slice(&cert.validity.not_after.to_be_bytes());
    out.push(usage_tag(cert.usage));
    put_list_strs(&mut out, &cert.sans.dns_names);
    put_list_strs(&mut out, &cert.sans.ip_addresses);
    put_list_strs(&mut out, &cert.sans.uris);
    put_bytes(&mut out, &cert.public_key_der);
    put_bytes(&mut out, &cert.signature);
    out
}

fn usage_tag(usage: CertUsage) -> u8 {
    match usage {
        CertUsage::CertificateAuthority => 0,
        CertUsage::ServerAuth => 1,
        CertUsage::ClientAuth => 2,
    }
}

fn usage_from_tag(tag: u8) -> Result<CertUsage, String> {
    match tag {
        0 => Ok(CertUsage::CertificateAuthority),
        1 => Ok(CertUsage::ServerAuth),
        2 => Ok(CertUsage::ClientAuth),
        other => Err(format!("unknown usage tag {other}")),
    }
}

/// A cursor over the encoded bytes that fails closed on any overrun.
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.pos.checked_add(n).ok_or("length overflow")?;
        if end > self.bytes.len() {
            return Err(format!(
                "truncated: need {n} bytes at offset {}, have {}",
                self.pos,
                self.bytes.len() - self.pos
            ));
        }
        let slice = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32, String> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn u64(&mut self) -> Result<u64, String> {
        let b = self.take(8)?;
        let mut arr = [0u8; 8];
        arr.copy_from_slice(b);
        Ok(u64::from_be_bytes(arr))
    }

    fn bytes_field(&mut self) -> Result<Vec<u8>, String> {
        let len = self.u32()? as usize;
        Ok(self.take(len)?.to_vec())
    }

    fn str_field(&mut self) -> Result<String, String> {
        let raw = self.bytes_field()?;
        String::from_utf8(raw).map_err(|e| format!("invalid UTF-8: {e}"))
    }

    fn list_strs(&mut self) -> Result<Vec<String>, String> {
        let count = self.u32()? as usize;
        let mut items = Vec::with_capacity(count.min(1024));
        for _ in 0..count {
            items.push(self.str_field()?);
        }
        Ok(items)
    }

    fn finished(&self) -> bool {
        self.pos == self.bytes.len()
    }
}

/// Decode the adapter's leaf byte form back into a trustd [`Certificate`].
/// Total and fail-closed: returns `Err(detail)` on a bad magic, truncation,
/// trailing garbage, invalid UTF-8, or unknown usage tag.
///
/// # Errors
/// A human-legible reason string on any decode failure.
pub fn decode(bytes: &[u8]) -> Result<Certificate, String> {
    let mut r = Reader::new(bytes);
    if r.take(4)? != MAGIC {
        return Err("bad magic (not a TSV1 leaf)".to_string());
    }
    let serial = r.u64()?;
    let subject = DistinguishedName {
        common_name: r.str_field()?,
        organizations: r.list_strs()?,
    };
    let issuer = DistinguishedName {
        common_name: r.str_field()?,
        organizations: r.list_strs()?,
    };
    let not_before = r.u64()?;
    let not_after = r.u64()?;
    let usage = usage_from_tag(r.u8()?)?;
    let sans = SubjectAltNames {
        dns_names: r.list_strs()?,
        ip_addresses: r.list_strs()?,
        uris: r.list_strs()?,
    };
    let public_key_der = r.bytes_field()?;
    let signature = r.bytes_field()?;
    if !r.finished() {
        return Err("trailing bytes after leaf".to_string());
    }
    Ok(Certificate {
        serial,
        subject,
        issuer,
        validity: Validity {
            not_before,
            not_after,
        },
        usage,
        sans,
        public_key_der,
        signature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_cloud_os_trustd_domain::ca::{CertificateAuthority, CertificateSigningRequest};
    use oya_cloud_os_trustd_domain::signer::InMemorySigner;
    use oya_cloud_os_trustd_domain::x509::KeyPair;

    fn sample_leaf() -> Certificate {
        let mut ca = CertificateAuthority::bootstrap(
            "oyatie-cell-7-ca",
            KeyPair::from_seed(b"ca"),
            InMemorySigner::from_seed("ca"),
            1_000,
            10_000_000,
        )
        .unwrap();
        let key = KeyPair::from_seed(b"wl");
        let csr = CertificateSigningRequest::for_workload(
            "secrets-sync",
            "spiffe://oyatie.cell-7/tenant/ten_acme/secrets-sync",
            &key,
            3_600,
        );
        ca.sign_csr(&csr, 2_000).unwrap()
    }

    #[test]
    fn round_trips_exactly() {
        let cert = sample_leaf();
        let encoded = encode(&cert);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(cert, decoded);
        // The signed identity survives the round trip.
        assert!(decoded
            .sans
            .covers_uri("spiffe://oyatie.cell-7/tenant/ten_acme/secrets-sync"));
    }

    #[test]
    fn rejects_bad_magic_and_truncation_and_trailing() {
        assert!(decode(b"XXXX").is_err());
        let good = encode(&sample_leaf());
        // truncated
        assert!(decode(&good[..good.len() - 3]).is_err());
        // trailing garbage
        let mut trailing = good.clone();
        trailing.push(0xFF);
        assert!(decode(&trailing).is_err());
        // empty
        assert!(decode(&[]).is_err());
    }
}
