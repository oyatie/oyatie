//! Durable sealed persistence for the trustd certificate authority.
//!
//! `trustd` cannot treat its CA as process-local memory: the root certificate,
//! issuance policy, and serial counter must survive restarts, while private root
//! and signing key material must never be written in plaintext. This module
//! models that boundary with a small [`KeySealer`] trait and an atomic file
//! format that stores public CA state plus sealed private material, a sealed
//! authentication envelope over the complete canonical state payload, and a
//! sealer-owned monotonic checkpoint over the serialized `next_serial`. A wrong
//! sealer, tampered state file, file rollback/replay, key/certificate mismatch,
//! stale serial counter, or signer mismatch fails closed before the CA is
//! restored.

use crate::ca::{CertificateAuthority, IssuancePolicy};
use crate::certificate::{CertUsage, Certificate};
use crate::error::{Result, TrustError};
use crate::signer::EcdsaP256Signer;
use crate::x509::{DistinguishedName, KeyPair, SubjectAltNames, Validity, hex_decode, hex_encode};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const FORMAT_VERSION: &str = "trustd-ca-state-v3";
const STATE_AUTH_CONTEXT: &[u8] = b"trustd-ca-state-auth-envelope-v3";
const STATE_ROLLBACK_CONTEXT: &[u8] = b"trustd-ca-state-monotonic-checkpoint-v3";
const KEYPAIR_CONTEXT: &[u8] = b"trustd-ca-keypair-private-der-v2";
const SIGNER_CONTEXT: &[u8] = b"trustd-ca-ecdsa-p256-pkcs8-der-v2";

/// Boundary used to seal and open CA private material before durable storage.
///
/// Production implementations should wrap KMS/HSM/TPM/OpenBao-backed sealing
/// and expose a rollback-resistant monotonic checkpoint. The trait deliberately
/// exposes only opaque byte sealing, a non-secret key id, and checkpoint methods
/// so this crate never stores root material, HSM admin secrets, recovery shares,
/// unseal keys, passwords, PINs, DEKs, or KEKs in repo-visible state.
pub trait KeySealer {
    /// Seal `plaintext` for `context`, returning bytes safe to persist.
    fn seal(&self, context: &[u8], plaintext: &[u8]) -> Result<Vec<u8>>;

    /// Open a previously sealed blob for `context`.
    fn open(&self, context: &[u8], sealed: &[u8]) -> Result<Vec<u8>>;

    /// Non-secret stable identifier for the sealing key, safe to persist.
    fn key_id(&self) -> String;

    /// Return the rollback-resistant checkpoint for `context`.
    ///
    /// This checkpoint is the latest persisted CA `next_serial` accepted for the
    /// sealing key. It must live outside the state file (for example in
    /// KMS/HSM/TPM/OpenBao metadata or another rollback-resistant store), be
    /// durable across process restarts, and be impossible to move backwards by
    /// replaying an older state file.
    fn monotonic_checkpoint(&self, context: &[u8]) -> Result<u64>;

    /// Advance the rollback-resistant checkpoint for `context` to `value`.
    ///
    /// Implementations must initialize an absent checkpoint, accept idempotent
    /// writes of the current value, and fail closed instead of decreasing it.
    fn advance_monotonic_checkpoint(&self, context: &[u8], value: u64) -> Result<()>;
}

impl CertificateAuthority<EcdsaP256Signer> {
    /// Persist this CA to `path`, sealing private key material with `sealer`.
    ///
    /// The write is staged to a temporary sibling file, synced, atomically
    /// renamed into place, and then the parent directory is best-effort synced.
    pub fn save_sealed_state(&self, path: impl AsRef<Path>, sealer: &impl KeySealer) -> Result<()> {
        let state = SealedCaState::from_ca(self, sealer)?;
        let rendered = state.render();
        // Advance the out-of-file checkpoint before publishing the file. If the
        // subsequent write fails, older on-disk state fails closed on restart
        // instead of reusing a serial that was already checkpointed.
        sealer.advance_monotonic_checkpoint(STATE_ROLLBACK_CONTEXT, state.next_serial)?;
        write_state_file(path.as_ref(), &rendered)
    }

    /// Load a CA previously written by [`CertificateAuthority::save_sealed_state`].
    ///
    /// Private root/signing material is opened through `sealer`; wrong keys,
    /// tampering, file rollback/replay, key/cert mismatches, broken serial
    /// state, and signer mismatch fail closed before the restored CA is
    /// returned.
    pub fn load_sealed_state(path: impl AsRef<Path>, sealer: &impl KeySealer) -> Result<Self> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|err| io_error("read", path, err))?;
        SealedCaState::parse(&text)?.into_ca(sealer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SealedCaState {
    sealer_key_id: String,
    cert: Certificate,
    policy: IssuancePolicy,
    next_serial: u64,
    sealed_keypair_private_der: Vec<u8>,
    sealed_signer_pkcs8_der: Vec<u8>,
    sealed_state_auth: Vec<u8>,
}

impl SealedCaState {
    fn from_ca(
        ca: &CertificateAuthority<EcdsaP256Signer>,
        sealer: &impl KeySealer,
    ) -> Result<Self> {
        let signer_pkcs8_der = ca.signing_backend().private_key_der();
        let mut state = SealedCaState {
            sealer_key_id: sealer.key_id(),
            cert: ca.certificate().clone(),
            policy: *ca.policy(),
            next_serial: ca.peek_serial(),
            sealed_keypair_private_der: sealer.seal(KEYPAIR_CONTEXT, ca.keypair().private_der())?,
            sealed_signer_pkcs8_der: sealer.seal(SIGNER_CONTEXT, &signer_pkcs8_der)?,
            sealed_state_auth: Vec::new(),
        };
        state.sealed_state_auth =
            sealer.seal(STATE_AUTH_CONTEXT, state.authenticated_payload().as_bytes())?;
        Ok(state)
    }

    fn into_ca(self, sealer: &impl KeySealer) -> Result<CertificateAuthority<EcdsaP256Signer>> {
        if self.sealer_key_id != sealer.key_id() {
            return Err(TrustError::verification_failed(
                "sealed CA state was sealed by a different key id",
            ));
        }
        let authenticated_payload = sealer.open(STATE_AUTH_CONTEXT, &self.sealed_state_auth)?;
        if authenticated_payload != self.authenticated_payload().as_bytes() {
            return Err(TrustError::verification_failed(
                "sealed CA state authentication envelope does not match public state",
            ));
        }
        self.verify_monotonic_checkpoint(sealer)?;
        let keypair_private_der = sealer.open(KEYPAIR_CONTEXT, &self.sealed_keypair_private_der)?;
        let signer_pkcs8_der = sealer.open(SIGNER_CONTEXT, &self.sealed_signer_pkcs8_der)?;
        let keypair = KeyPair::new(keypair_private_der, self.cert.public_key_der.clone());
        let signer = EcdsaP256Signer::from_pkcs8_der(&signer_pkcs8_der)?;
        CertificateAuthority::from_persisted_parts(
            self.cert,
            keypair,
            signer,
            self.policy,
            self.next_serial,
        )
    }

    fn verify_monotonic_checkpoint(&self, sealer: &impl KeySealer) -> Result<()> {
        let checkpoint = sealer.monotonic_checkpoint(STATE_ROLLBACK_CONTEXT)?;
        if self.next_serial != checkpoint {
            return Err(TrustError::verification_failed(format!(
                "sealed CA state rollback/replay detected: state next_serial {} does not match monotonic checkpoint {checkpoint}",
                self.next_serial
            )));
        }
        Ok(())
    }

    fn render(&self) -> String {
        let mut out = self.authenticated_payload();
        put_line(
            &mut out,
            "sealed.state_auth",
            &hex_encode(&self.sealed_state_auth),
        );
        out
    }

    fn authenticated_payload(&self) -> String {
        let mut out = String::new();
        put_line(&mut out, "format", FORMAT_VERSION);
        put_line(&mut out, "sealer.key_id", &self.sealer_key_id);
        put_line(&mut out, "cert.serial", &self.cert.serial.to_string());
        put_line(
            &mut out,
            "cert.subject",
            &hex_encode(self.cert.subject.to_rfc().as_bytes()),
        );
        put_line(
            &mut out,
            "cert.issuer",
            &hex_encode(self.cert.issuer.to_rfc().as_bytes()),
        );
        put_line(
            &mut out,
            "cert.validity.not_before",
            &self.cert.validity.not_before.to_string(),
        );
        put_line(
            &mut out,
            "cert.validity.not_after",
            &self.cert.validity.not_after.to_string(),
        );
        put_line(&mut out, "cert.usage", usage_tag(self.cert.usage));
        put_line(
            &mut out,
            "cert.sans.dns",
            &encode_string_list(&self.cert.sans.dns_names),
        );
        put_line(
            &mut out,
            "cert.sans.ip",
            &encode_string_list(&self.cert.sans.ip_addresses),
        );
        put_line(
            &mut out,
            "cert.sans.uri",
            &encode_string_list(&self.cert.sans.uris),
        );
        put_line(
            &mut out,
            "cert.public_key_der",
            &hex_encode(&self.cert.public_key_der),
        );
        put_line(
            &mut out,
            "cert.signature",
            &hex_encode(&self.cert.signature),
        );
        put_line(
            &mut out,
            "policy.max_ttl_secs",
            &self.policy.max_ttl_secs.to_string(),
        );
        put_line(
            &mut out,
            "policy.allow_ca_requests",
            bool_tag(self.policy.allow_ca_requests),
        );
        put_line(
            &mut out,
            "policy.require_common_name",
            bool_tag(self.policy.require_common_name),
        );
        put_line(
            &mut out,
            "policy.max_sans",
            &self.policy.max_sans.to_string(),
        );
        put_line(
            &mut out,
            "policy.validate_sans",
            bool_tag(self.policy.validate_sans),
        );
        put_line(&mut out, "next_serial", &self.next_serial.to_string());
        put_line(
            &mut out,
            "sealed.keypair_private_der",
            &hex_encode(&self.sealed_keypair_private_der),
        );
        put_line(
            &mut out,
            "sealed.signer_pkcs8_der",
            &hex_encode(&self.sealed_signer_pkcs8_der),
        );
        out
    }

    fn parse(text: &str) -> Result<Self> {
        let fields = parse_fields(text)?;
        let format = required(&fields, "format")?;
        if format != FORMAT_VERSION {
            return Err(TrustError::invalid(format!(
                "unsupported CA state format '{format}'"
            )));
        }
        reject_unknown_fields(&fields)?;

        let cert = Certificate {
            serial: parse_u64(required(&fields, "cert.serial")?, "cert.serial")?,
            subject: DistinguishedName::parse_rfc(&decode_string_field(
                required(&fields, "cert.subject")?,
                "cert.subject",
            )?)?,
            issuer: DistinguishedName::parse_rfc(&decode_string_field(
                required(&fields, "cert.issuer")?,
                "cert.issuer",
            )?)?,
            validity: Validity {
                not_before: parse_u64(
                    required(&fields, "cert.validity.not_before")?,
                    "cert.validity.not_before",
                )?,
                not_after: parse_u64(
                    required(&fields, "cert.validity.not_after")?,
                    "cert.validity.not_after",
                )?,
            },
            usage: parse_usage(required(&fields, "cert.usage")?)?,
            sans: SubjectAltNames {
                dns_names: decode_string_list(required(&fields, "cert.sans.dns")?)?,
                ip_addresses: decode_string_list(required(&fields, "cert.sans.ip")?)?,
                uris: decode_string_list(required(&fields, "cert.sans.uri")?)?,
            },
            public_key_der: hex_decode(required(&fields, "cert.public_key_der")?)?,
            signature: hex_decode(required(&fields, "cert.signature")?)?,
        };
        cert.validate()?;
        cert.sans.validate()?;

        Ok(SealedCaState {
            sealer_key_id: required(&fields, "sealer.key_id")?.to_string(),
            cert,
            policy: IssuancePolicy {
                max_ttl_secs: parse_u64(
                    required(&fields, "policy.max_ttl_secs")?,
                    "policy.max_ttl_secs",
                )?,
                allow_ca_requests: parse_bool(required(&fields, "policy.allow_ca_requests")?)?,
                require_common_name: parse_bool(required(&fields, "policy.require_common_name")?)?,
                max_sans: parse_usize(required(&fields, "policy.max_sans")?, "policy.max_sans")?,
                validate_sans: parse_bool(required(&fields, "policy.validate_sans")?)?,
            },
            next_serial: parse_u64(required(&fields, "next_serial")?, "next_serial")?,
            sealed_keypair_private_der: hex_decode(required(
                &fields,
                "sealed.keypair_private_der",
            )?)?,
            sealed_signer_pkcs8_der: hex_decode(required(&fields, "sealed.signer_pkcs8_der")?)?,
            sealed_state_auth: hex_decode(required(&fields, "sealed.state_auth")?)?,
        })
    }
}

fn put_line(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push('=');
    out.push_str(value);
    out.push('\n');
}

fn parse_fields(text: &str) -> Result<BTreeMap<String, String>> {
    let mut fields = BTreeMap::new();
    for (idx, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(|| {
            TrustError::invalid(format!("CA state line {} is missing '='", idx + 1))
        })?;
        if fields.insert(key.to_string(), value.to_string()).is_some() {
            return Err(TrustError::invalid(format!(
                "duplicate CA state field '{key}'"
            )));
        }
    }
    Ok(fields)
}

fn reject_unknown_fields(fields: &BTreeMap<String, String>) -> Result<()> {
    for key in fields.keys() {
        if !STATE_FIELDS.iter().any(|known| key == known) {
            return Err(TrustError::invalid(format!(
                "unknown CA state field '{key}'"
            )));
        }
    }
    Ok(())
}

const STATE_FIELDS: &[&str] = &[
    "format",
    "sealer.key_id",
    "cert.serial",
    "cert.subject",
    "cert.issuer",
    "cert.validity.not_before",
    "cert.validity.not_after",
    "cert.usage",
    "cert.sans.dns",
    "cert.sans.ip",
    "cert.sans.uri",
    "cert.public_key_der",
    "cert.signature",
    "policy.max_ttl_secs",
    "policy.allow_ca_requests",
    "policy.require_common_name",
    "policy.max_sans",
    "policy.validate_sans",
    "next_serial",
    "sealed.keypair_private_der",
    "sealed.signer_pkcs8_der",
    "sealed.state_auth",
];

fn required<'a>(fields: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str> {
    fields
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| TrustError::invalid(format!("missing CA state field '{key}'")))
}

fn parse_u64(value: &str, field: &str) -> Result<u64> {
    value
        .parse()
        .map_err(|_| TrustError::invalid(format!("CA state field '{field}' is not a u64")))
}

fn parse_usize(value: &str, field: &str) -> Result<usize> {
    value
        .parse()
        .map_err(|_| TrustError::invalid(format!("CA state field '{field}' is not a usize")))
}

fn bool_tag(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn parse_bool(value: &str) -> Result<bool> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(TrustError::invalid(format!(
            "CA state boolean field has invalid value '{value}'"
        ))),
    }
}

fn usage_tag(usage: CertUsage) -> &'static str {
    match usage {
        CertUsage::CertificateAuthority => "certificate-authority",
        CertUsage::ServerAuth => "server-auth",
        CertUsage::ClientAuth => "client-auth",
    }
}

fn parse_usage(value: &str) -> Result<CertUsage> {
    match value {
        "certificate-authority" => Ok(CertUsage::CertificateAuthority),
        "server-auth" => Ok(CertUsage::ServerAuth),
        "client-auth" => Ok(CertUsage::ClientAuth),
        other => Err(TrustError::invalid(format!(
            "unknown certificate usage '{other}'"
        ))),
    }
}

fn encode_string_list(values: &[String]) -> String {
    values
        .iter()
        .map(|value| hex_encode(value.as_bytes()))
        .collect::<Vec<_>>()
        .join(",")
}

fn decode_string_list(value: &str) -> Result<Vec<String>> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value
        .split(',')
        .map(|part| decode_string_field(part, "string-list"))
        .collect()
}

fn decode_string_field(value: &str, field: &str) -> Result<String> {
    let bytes = hex_decode(value)?;
    String::from_utf8(bytes)
        .map_err(|_| TrustError::invalid(format!("CA state field '{field}' is not UTF-8")))
}

fn write_state_file(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| io_error("create parent", parent, err))?;
    }
    let tmp = temporary_path(path);
    let write_result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp)
            .map_err(|err| io_error("open temp", &tmp, err))?;
        file.write_all(contents.as_bytes())
            .map_err(|err| io_error("write temp", &tmp, err))?;
        file.sync_all()
            .map_err(|err| io_error("sync temp", &tmp, err))?;
        drop(file);
        fs::rename(&tmp, path).map_err(|err| io_error("rename", path, err))?;
        if let Some(parent) = path.parent()
            && let Ok(dir) = OpenOptions::new().read(true).open(parent)
        {
            let _ = dir.sync_all();
        }
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    write_result
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut tmp = path.as_os_str().to_os_string();
    tmp.push(format!(".{}.tmp", std::process::id()));
    PathBuf::from(tmp)
}

fn io_error(action: &str, path: &Path, err: io::Error) -> TrustError {
    if err.kind() == io::ErrorKind::NotFound {
        TrustError::not_found(format!("failed to {action} '{}': {err}", path.display()))
    } else {
        TrustError::Other(format!("failed to {action} '{}': {err}", path.display()))
    }
}

#[cfg(test)]
const TAG_LEN: usize = 8;

#[cfg(test)]
#[derive(Debug, Clone)]
struct StaticKeySealer {
    key: Vec<u8>,
    checkpoint: std::cell::Cell<Option<u64>>,
}

#[cfg(test)]
impl StaticKeySealer {
    fn new(key: Vec<u8>) -> Result<Self> {
        if key.is_empty() {
            return Err(TrustError::invalid("sealing key is empty"));
        }
        Ok(StaticKeySealer {
            key,
            checkpoint: std::cell::Cell::new(None),
        })
    }

    fn stream_byte(&self, context: &[u8], index: usize) -> u8 {
        let block = fnv64_many(&[
            b"stream".as_slice(),
            &self.key,
            context,
            &index.to_be_bytes(),
        ]);
        block.to_be_bytes()[index % 8]
    }

    fn tag(&self, context: &[u8], ciphertext: &[u8]) -> u64 {
        fnv64_many(&[b"tag".as_slice(), &self.key, context, ciphertext])
    }
}

#[cfg(test)]
impl KeySealer for StaticKeySealer {
    fn seal(&self, context: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
        let ciphertext: Vec<u8> = plaintext
            .iter()
            .enumerate()
            .map(|(idx, byte)| byte ^ self.stream_byte(context, idx))
            .collect();
        let mut sealed = Vec::with_capacity(TAG_LEN + ciphertext.len());
        sealed.extend_from_slice(&self.tag(context, &ciphertext).to_be_bytes());
        sealed.extend_from_slice(&ciphertext);
        Ok(sealed)
    }

    fn open(&self, context: &[u8], sealed: &[u8]) -> Result<Vec<u8>> {
        if sealed.len() < TAG_LEN {
            return Err(TrustError::verification_failed(
                "sealed CA state is too short to authenticate",
            ));
        }
        let (tag_bytes, ciphertext) = sealed.split_at(TAG_LEN);
        let stored_tag = u64::from_be_bytes(
            tag_bytes
                .try_into()
                .map_err(|_| TrustError::invalid("sealed CA tag has invalid length"))?,
        );
        let expected_tag = self.tag(context, ciphertext);
        if stored_tag != expected_tag {
            return Err(TrustError::verification_failed(
                "sealed CA state authentication failed",
            ));
        }
        Ok(ciphertext
            .iter()
            .enumerate()
            .map(|(idx, byte)| byte ^ self.stream_byte(context, idx))
            .collect())
    }

    fn key_id(&self) -> String {
        hex_encode(&fnv64_many(&[b"key-id".as_slice(), &self.key]).to_be_bytes())
    }

    fn monotonic_checkpoint(&self, context: &[u8]) -> Result<u64> {
        if context != STATE_ROLLBACK_CONTEXT {
            return Err(TrustError::invalid("unknown monotonic checkpoint context"));
        }
        self.checkpoint.get().ok_or_else(|| {
            TrustError::verification_failed("sealed CA state monotonic checkpoint is unavailable")
        })
    }

    fn advance_monotonic_checkpoint(&self, context: &[u8], value: u64) -> Result<()> {
        if context != STATE_ROLLBACK_CONTEXT {
            return Err(TrustError::invalid("unknown monotonic checkpoint context"));
        }
        if let Some(current) = self.checkpoint.get()
            && value < current
        {
            return Err(TrustError::verification_failed(
                "sealed CA state monotonic checkpoint cannot move backwards",
            ));
        }
        self.checkpoint.set(Some(value));
        Ok(())
    }
}

#[cfg(test)]
fn fnv64_many(chunks: &[&[u8]]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for chunk in chunks {
        for &byte in *chunk {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ca::CertificateSigningRequest;

    fn temp_ca_state_path(test_name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "trustd-{test_name}-{}-{nonce}.state",
            std::process::id()
        ))
    }

    fn ca() -> CertificateAuthority<EcdsaP256Signer> {
        CertificateAuthority::bootstrap(
            "talos-ca",
            KeyPair::from_seed(b"ca-root-key"),
            EcdsaP256Signer::generate().unwrap(),
            1000,
            1_000_000,
        )
        .unwrap()
    }

    fn replace_state_line(path: &Path, key: &str, replacement: &str) {
        let original = fs::read_to_string(path).unwrap();
        let old_line = original
            .lines()
            .find(|line| line.starts_with(&format!("{key}=")))
            .unwrap_or_else(|| panic!("state line {key} exists"));
        let tampered = original.replacen(old_line, replacement, 1);
        fs::write(path, tampered).unwrap();
    }

    #[test]
    fn sealed_state_round_trips_real_signer_without_plaintext_root_key() {
        let path = temp_ca_state_path("roundtrip-real-signer");
        let sealer = StaticKeySealer::new(b"unit-test-kms-root".to_vec()).unwrap();
        let mut ca = ca();
        let node_key = KeyPair::from_seed(b"node-persisted");
        let csr = CertificateSigningRequest::for_node(
            "node-persisted",
            &node_key,
            CertUsage::ClientAuth,
            3600,
        );
        let before_restart = ca.sign_csr(&csr, 2000).unwrap();
        let next_serial = ca.peek_serial();

        ca.save_sealed_state(&path, &sealer).unwrap();
        let persisted = fs::read_to_string(&path).unwrap();
        assert!(persisted.contains("sealed.keypair_private_der="));
        assert!(persisted.contains("sealed.signer_pkcs8_der="));
        assert!(persisted.contains("sealed.state_auth="));
        assert!(!persisted.contains("ca-root-key"));
        assert!(!persisted.contains(&hex_encode(ca.keypair().private_der())));
        assert!(!persisted.contains(&hex_encode(&ca.signing_backend().private_key_der())));

        let mut restored =
            CertificateAuthority::<EcdsaP256Signer>::load_sealed_state(&path, &sealer).unwrap();
        assert_eq!(restored.peek_serial(), next_serial);
        assert!(restored.verify(&before_restart, 2500).is_ok());
        let after_restart = restored.sign_csr(&csr, 2500).unwrap();
        assert_eq!(after_restart.serial, next_serial);
        assert!(restored.verify(&after_restart, 2600).is_ok());

        let _ = fs::remove_file(path);
    }

    #[test]
    fn sealed_state_rejects_wrong_sealer_and_tampering() {
        let path = temp_ca_state_path("tamper-real-signer");
        let sealer = StaticKeySealer::new(b"unit-test-kms-root".to_vec()).unwrap();
        ca().save_sealed_state(&path, &sealer).unwrap();

        let wrong = StaticKeySealer::new(b"wrong-kms-root".to_vec()).unwrap();
        assert_eq!(
            CertificateAuthority::<EcdsaP256Signer>::load_sealed_state(&path, &wrong)
                .err()
                .unwrap()
                .kind(),
            "verification_failed"
        );

        let tampered = fs::read_to_string(&path).unwrap().replacen(
            "sealed.signer_pkcs8_der=",
            "sealed.signer_pkcs8_der=00",
            1,
        );
        fs::write(&path, tampered).unwrap();
        assert_eq!(
            CertificateAuthority::<EcdsaP256Signer>::load_sealed_state(&path, &sealer)
                .err()
                .unwrap()
                .kind(),
            "verification_failed"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn sealed_state_rejects_public_state_metadata_tampering() {
        let sealer = StaticKeySealer::new(b"unit-test-kms-root".to_vec()).unwrap();

        for (name, key, replacement) in [
            (
                "policy-allows-ca-requests",
                "policy.allow_ca_requests",
                "policy.allow_ca_requests=true",
            ),
            ("serial-counter", "next_serial", "next_serial=99"),
            ("certificate-usage", "cert.usage", "cert.usage=server-auth"),
        ] {
            let path = temp_ca_state_path(name);
            ca().save_sealed_state(&path, &sealer).unwrap();
            replace_state_line(&path, key, replacement);

            assert_eq!(
                CertificateAuthority::<EcdsaP256Signer>::load_sealed_state(&path, &sealer)
                    .err()
                    .unwrap_or_else(|| panic!("{name} tamper must fail closed"))
                    .kind(),
                "verification_failed"
            );

            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn sealed_state_rejects_whole_file_rollback_before_duplicate_serial() {
        let path = temp_ca_state_path("rollback-replay");
        let sealer = StaticKeySealer::new(b"unit-test-kms-root".to_vec()).unwrap();
        let mut ca = ca();

        ca.save_sealed_state(&path, &sealer).unwrap();
        let stale_state = fs::read_to_string(&path).unwrap();

        let node_key = KeyPair::from_seed(b"node-rollback");
        let csr = CertificateSigningRequest::for_node(
            "node-rollback",
            &node_key,
            CertUsage::ClientAuth,
            3600,
        );
        let issued = ca.sign_csr(&csr, 2000).unwrap();
        assert_eq!(issued.serial, 2);
        assert_eq!(ca.peek_serial(), 3);
        ca.save_sealed_state(&path, &sealer).unwrap();

        fs::write(&path, stale_state).unwrap();
        let replayed = CertificateAuthority::<EcdsaP256Signer>::load_sealed_state(&path, &sealer);
        let err = match replayed {
            Ok(mut replayed) => {
                let duplicate = replayed.sign_csr(&csr, 2500).unwrap();
                panic!(
                    "whole-file rollback accepted stale next_serial={} and reissued serial {}",
                    replayed.peek_serial(),
                    duplicate.serial
                );
            }
            Err(err) => err,
        };
        assert_eq!(err.kind(), "verification_failed");

        let _ = fs::remove_file(path);
    }
}
