//! SecureBoot signing state and key enrollment.
//!
//! Models the boundary Talos crosses when producing SecureBoot-capable images:
//! a Platform Key (PK), Key Exchange Keys (KEK), and the signature databases
//! (`db`/`dbx`). UKIs and the systemd-boot stub must be signed with a key whose
//! certificate chains to an entry in `db`. See `internal/pkg/secureboot`.

/// The UEFI key hierarchy enrollment phase. UEFI only accepts updates to a
/// given database when it is in the matching mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnrollmentPhase {
    /// Setup Mode: PK is empty, any key may be enrolled (factory state).
    Setup,
    /// User Mode: PK is set, SecureBoot is enforced.
    User,
}

/// A signing key identity (modeled by its certificate fingerprint and the
/// database it belongs to). The private key material itself never crosses this
/// boundary — signing is delegated to [`Signer`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningKey {
    /// Human-readable name (e.g. `"Talos Secure Boot Signer"`).
    pub name: String,
    /// Hex SHA-256 fingerprint of the certificate.
    pub fingerprint: String,
}

impl SigningKey {
    /// Create a signing key.
    pub fn new(name: &str, fingerprint: &str) -> SigningKey {
        SigningKey {
            name: name.to_string(),
            fingerprint: fingerprint.to_string(),
        }
    }
}

/// The UEFI key material Talos enrolls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecureBootKeys {
    /// Platform Key.
    pub pk: SigningKey,
    /// Key Exchange Keys.
    pub kek: Vec<SigningKey>,
    /// Authorized signature database (keys that may sign bootable images).
    pub db: Vec<SigningKey>,
    /// Forbidden signature database (revoked keys/hashes).
    pub dbx: Vec<SigningKey>,
}

impl SecureBootKeys {
    /// A minimal valid keyset with a single signer enrolled in both KEK and db.
    pub fn single_signer(pk: SigningKey, signer: SigningKey) -> SecureBootKeys {
        SecureBootKeys {
            pk,
            kek: vec![signer.clone()],
            db: vec![signer],
            dbx: Vec::new(),
        }
    }

    /// Whether a key with the given fingerprint is authorized to sign a
    /// bootable image: present in `db` and *not* revoked in `dbx`.
    pub fn authorizes(&self, fingerprint: &str) -> bool {
        let in_db = self.db.iter().any(|k| k.fingerprint == fingerprint);
        let revoked = self.dbx.iter().any(|k| k.fingerprint == fingerprint);
        in_db && !revoked
    }

    /// Revoke a key by moving it into `dbx`.
    pub fn revoke(&mut self, fingerprint: &str) {
        if let Some(pos) = self.db.iter().position(|k| k.fingerprint == fingerprint) {
            let key = self.db.remove(pos);
            self.dbx.push(key);
        }
    }
}

/// A detached signature over some image payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// Fingerprint of the signing key.
    pub signer_fingerprint: String,
    /// Opaque signature bytes (in real life a PKCS#7 blob).
    pub bytes: Vec<u8>,
}

/// The named PE sections a Talos UKI carries, in canonical order. Mirrors the
/// `.linux`/`.initrd`/`.cmdline`/... sections systemd's `ukify` emits. The
/// ordering is load-bearing: PCR 11 is measured by hashing each present
/// section's name+content in this exact sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UkiSection {
    /// OS release identification (`.osrel`).
    OsRel,
    /// Kernel command line (`.cmdline`).
    Cmdline,
    /// Distro initrd (`.initrd`).
    Initrd,
    /// Splash image (`.splash`).
    Splash,
    /// Device tree blob (`.dtb`).
    Dtb,
    /// Linux kernel (`.linux`).
    Linux,
    /// PCR public key (`.pcrpkey`).
    PcrPkey,
    /// PCR signature JSON (`.pcrsig`).
    PcrSig,
}

impl UkiSection {
    /// The PE section name (with leading dot) as written into the UKI.
    pub fn name(self) -> &'static str {
        match self {
            UkiSection::OsRel => ".osrel",
            UkiSection::Cmdline => ".cmdline",
            UkiSection::Initrd => ".initrd",
            UkiSection::Splash => ".splash",
            UkiSection::Dtb => ".dtb",
            UkiSection::Linux => ".linux",
            UkiSection::PcrPkey => ".pcrpkey",
            UkiSection::PcrSig => ".pcrsig",
        }
    }

    /// Canonical measurement order (the order `systemd-stub` extends PCR 11 in).
    pub fn measurement_order() -> [UkiSection; 8] {
        [
            UkiSection::OsRel,
            UkiSection::Cmdline,
            UkiSection::Initrd,
            UkiSection::Splash,
            UkiSection::Dtb,
            UkiSection::Linux,
            UkiSection::PcrPkey,
            UkiSection::PcrSig,
        ]
    }

    /// Whether this section participates in PCR 11 measurement. The `.pcrsig`
    /// section is the *output* of measurement and is therefore never measured
    /// itself.
    pub fn is_measured(self) -> bool {
        !matches!(self, UkiSection::PcrSig)
    }
}

/// A deterministic, dependency-free 64-bit FNV-1a hash used to stand in for the
/// SHA-256 that a real UKI/PCR pipeline would use. Good enough to model
/// collision-free identity and tamper detection in tests.
pub fn fnv1a(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Render a hash as lowercase hex.
pub fn hex64(h: u64) -> String {
    format!("{h:016x}")
}

/// A Unified Kernel Image under construction: an ordered set of PE sections,
/// optionally signed. Models `internal/pkg/secureboot/uki`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Uki {
    sections: Vec<(UkiSection, Vec<u8>)>,
    /// Detached PE (Authenticode-style) signature once signed.
    pub signature: Option<Signature>,
}

impl Uki {
    /// Start an empty UKI.
    pub fn new() -> Uki {
        Uki::default()
    }

    /// Add or replace a section. Sections are kept in canonical measurement
    /// order regardless of insertion order so the digest is stable.
    pub fn with_section(mut self, sec: UkiSection, data: impl Into<Vec<u8>>) -> Uki {
        let data = data.into();
        if let Some(slot) = self.sections.iter_mut().find(|(s, _)| *s == sec) {
            slot.1 = data;
        } else {
            self.sections.push((sec, data));
        }
        self.sections.sort_by_key(|(s, _)| {
            UkiSection::measurement_order()
                .iter()
                .position(|o| o == s)
                .unwrap_or(usize::MAX)
        });
        self
    }

    /// Look up a section's bytes.
    pub fn section(&self, sec: UkiSection) -> Option<&[u8]> {
        self.sections
            .iter()
            .find(|(s, _)| *s == sec)
            .map(|(_, d)| d.as_slice())
    }

    /// Whether the minimal sections required for a bootable UKI are present
    /// (kernel + cmdline + initrd + os-release).
    pub fn is_complete(&self) -> bool {
        [
            UkiSection::Linux,
            UkiSection::Cmdline,
            UkiSection::Initrd,
            UkiSection::OsRel,
        ]
        .iter()
        .all(|s| self.section(*s).is_some())
    }

    /// The Authenticode-style digest over all sections in canonical order.
    /// Tampering with any section (or reordering) changes this digest.
    pub fn digest(&self) -> u64 {
        let mut buf = Vec::new();
        for sec in UkiSection::measurement_order() {
            if let Some(data) = self.section(sec) {
                buf.extend_from_slice(sec.name().as_bytes());
                buf.push(0);
                buf.extend_from_slice(&(data.len() as u64).to_le_bytes());
                buf.extend_from_slice(data);
            }
        }
        fnv1a(&buf)
    }

    /// Sign the UKI's PE digest with the given signer, attaching the signature.
    pub fn sign(&mut self, signer: &dyn Signer) {
        let payload = hex64(self.digest());
        self.signature = Some(signer.sign(payload.as_bytes()));
    }

    /// Whether the UKI has been signed.
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }
}

/// A PCR 11 policy: the expected measurement value plus a signature over it,
/// produced at build time so the TPM can release secrets only when the booted
/// UKI measures to a value the build authority signed. Models
/// `internal/pkg/secureboot/measure`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcrPolicy {
    /// The expected PCR 11 value (hex) after the UKI's sections are extended.
    pub expected: String,
    /// Signature over `expected` by the PCR signing key.
    pub signature: Signature,
}

/// Compute the PCR 11 value a given UKI would extend the TPM to. Each measured
/// section is folded in as `pcr = H(pcr || H(name) || H(content))`, mirroring
/// the TPM "extend" operation over `systemd-stub`'s section measurements.
pub fn measure_pcr11(uki: &Uki) -> String {
    let mut pcr: u64 = 0;
    for sec in UkiSection::measurement_order() {
        if !sec.is_measured() {
            continue;
        }
        if let Some(data) = uki.section(sec) {
            let name_h = fnv1a(sec.name().as_bytes());
            let content_h = fnv1a(data);
            let mut buf = Vec::with_capacity(24);
            buf.extend_from_slice(&pcr.to_le_bytes());
            buf.extend_from_slice(&name_h.to_le_bytes());
            buf.extend_from_slice(&content_h.to_le_bytes());
            pcr = fnv1a(&buf);
        }
    }
    hex64(pcr)
}

impl PcrPolicy {
    /// Build and sign a PCR 11 policy for the given UKI.
    pub fn build(uki: &Uki, signer: &dyn Signer) -> PcrPolicy {
        let expected = measure_pcr11(uki);
        let signature = signer.sign(expected.as_bytes());
        PcrPolicy {
            expected,
            signature,
        }
    }

    /// Verify that a UKI booted to the expected PCR value and the policy
    /// signature was produced by an authorized key.
    pub fn verify(&self, uki: &Uki, keys: &SecureBootKeys) -> bool {
        measure_pcr11(uki) == self.expected && keys.authorizes(&self.signature.signer_fingerprint)
    }
}

/// The signing boundary. Real implementations shell out to `sbsign`/`pesign`;
/// the in-memory test impl produces a deterministic pseudo-signature.
pub trait Signer {
    /// Fingerprint of the key this signer holds.
    fn key_fingerprint(&self) -> &str;

    /// Sign `payload`, producing a detached signature.
    fn sign(&self, payload: &[u8]) -> Signature;
}

/// The overall SecureBoot context used while building/installing an image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecureBootState {
    /// Whether SecureBoot enforcement is enabled (User mode).
    pub phase: EnrollmentPhase,
    /// The enrolled key material.
    pub keys: SecureBootKeys,
}

impl SecureBootState {
    /// Construct in User (enforcing) mode with the given keys.
    pub fn enforcing(keys: SecureBootKeys) -> SecureBootState {
        SecureBootState {
            phase: EnrollmentPhase::User,
            keys,
        }
    }

    /// Whether enforcement is active.
    pub fn is_enforcing(&self) -> bool {
        self.phase == EnrollmentPhase::User
    }

    /// Verify that a signature was produced by an authorized key. When not
    /// enforcing, any signature is accepted (SecureBoot disabled in firmware).
    pub fn verify(&self, sig: &Signature) -> bool {
        if !self.is_enforcing() {
            return true;
        }
        self.keys.authorizes(&sig.signer_fingerprint)
    }

    /// Verify a fully-built UKI is bootable under this SecureBoot state: it must
    /// be complete, signed, and (when enforcing) signed by an authorized key.
    pub fn verify_uki(&self, uki: &Uki) -> Result<(), String> {
        if !uki.is_complete() {
            return Err("uki is missing required sections".to_string());
        }
        match &uki.signature {
            None => {
                if self.is_enforcing() {
                    Err("uki is not signed".to_string())
                } else {
                    Ok(())
                }
            }
            Some(sig) => {
                if self.verify(sig) {
                    Ok(())
                } else {
                    Err(format!(
                        "uki signed by untrusted key {}",
                        sig.signer_fingerprint
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct InMemorySigner {
        fp: String,
    }
    impl Signer for InMemorySigner {
        fn key_fingerprint(&self) -> &str {
            &self.fp
        }
        fn sign(&self, payload: &[u8]) -> Signature {
            // Deterministic pseudo-signature: length-tagged echo. Truncation to
            // a single byte is intentional for this in-memory test stub.
            #[allow(clippy::cast_possible_truncation)]
            let mut bytes = vec![payload.len() as u8];
            bytes.extend_from_slice(payload);
            Signature {
                signer_fingerprint: self.fp.clone(),
                bytes,
            }
        }
    }

    fn keys() -> SecureBootKeys {
        SecureBootKeys::single_signer(
            SigningKey::new("PK", "pkfp"),
            SigningKey::new("Talos Signer", "abc123"),
        )
    }

    #[test]
    fn authorized_signature_verifies_when_enforcing() {
        let state = SecureBootState::enforcing(keys());
        let signer = InMemorySigner {
            fp: "abc123".to_string(),
        };
        let sig = signer.sign(b"uki-payload");
        assert!(state.is_enforcing());
        assert!(state.verify(&sig));
    }

    #[test]
    fn unauthorized_signature_rejected_when_enforcing() {
        let state = SecureBootState::enforcing(keys());
        let rogue = InMemorySigner {
            fp: "deadbeef".to_string(),
        };
        assert!(!state.verify(&rogue.sign(b"x")));
    }

    #[test]
    fn revocation_moves_key_to_dbx_and_blocks() {
        let mut state = SecureBootState::enforcing(keys());
        assert!(state.keys.authorizes("abc123"));
        state.keys.revoke("abc123");
        assert!(!state.keys.authorizes("abc123"));
        assert_eq!(state.keys.dbx.len(), 1);
        assert!(state.keys.db.is_empty());
    }

    #[test]
    fn disabled_secureboot_accepts_anything() {
        let state = SecureBootState {
            phase: EnrollmentPhase::Setup,
            keys: keys(),
        };
        let rogue = InMemorySigner {
            fp: "nope".to_string(),
        };
        assert!(!state.is_enforcing());
        assert!(state.verify(&rogue.sign(b"x")));
    }

    fn complete_uki() -> Uki {
        Uki::new()
            .with_section(UkiSection::Linux, b"vmlinuz-bytes".to_vec())
            .with_section(UkiSection::Initrd, b"initramfs-bytes".to_vec())
            .with_section(UkiSection::Cmdline, b"talos.platform=metal".to_vec())
            .with_section(UkiSection::OsRel, b"ID=talos\nVERSION=v1.7.0".to_vec())
    }

    #[test]
    fn uki_sections_kept_in_canonical_order() {
        // Insert out of order; digest must not depend on insertion order.
        let a = Uki::new()
            .with_section(UkiSection::Linux, b"k".to_vec())
            .with_section(UkiSection::OsRel, b"o".to_vec());
        let b = Uki::new()
            .with_section(UkiSection::OsRel, b"o".to_vec())
            .with_section(UkiSection::Linux, b"k".to_vec());
        assert_eq!(a.digest(), b.digest());
    }

    #[test]
    fn uki_completeness() {
        assert!(complete_uki().is_complete());
        let partial = Uki::new().with_section(UkiSection::Linux, b"k".to_vec());
        assert!(!partial.is_complete());
    }

    #[test]
    fn tampering_changes_digest() {
        let clean = complete_uki();
        let tampered = clean
            .clone()
            .with_section(UkiSection::Cmdline, b"init=/bin/sh".to_vec());
        assert_ne!(clean.digest(), tampered.digest());
    }

    #[test]
    fn signing_attaches_signature_over_digest() {
        let mut uki = complete_uki();
        let signer = InMemorySigner {
            fp: "abc123".to_string(),
        };
        assert!(!uki.is_signed());
        uki.sign(&signer);
        assert!(uki.is_signed());
        let expected = signer.sign(hex64(uki.digest()).as_bytes());
        assert_eq!(uki.signature.as_ref().unwrap(), &expected);
    }

    #[test]
    fn verify_uki_enforces_signature_and_trust() {
        let state = SecureBootState::enforcing(keys());
        let mut uki = complete_uki();
        // Unsigned -> rejected when enforcing.
        assert!(state.verify_uki(&uki).is_err());
        // Signed by trusted key -> accepted.
        uki.sign(&InMemorySigner {
            fp: "abc123".to_string(),
        });
        assert!(state.verify_uki(&uki).is_ok());
        // Signed by untrusted key -> rejected.
        let mut rogue = complete_uki();
        rogue.sign(&InMemorySigner {
            fp: "deadbeef".to_string(),
        });
        assert!(state.verify_uki(&rogue).is_err());
        // Incomplete -> rejected regardless.
        let mut incomplete = Uki::new().with_section(UkiSection::Linux, b"k".to_vec());
        incomplete.sign(&InMemorySigner {
            fp: "abc123".to_string(),
        });
        assert!(state.verify_uki(&incomplete).is_err());
    }

    #[test]
    fn pcr_measurement_is_deterministic_and_sensitive() {
        let uki = complete_uki();
        let m1 = measure_pcr11(&uki);
        let m2 = measure_pcr11(&uki);
        assert_eq!(m1, m2);
        let other = uki
            .clone()
            .with_section(UkiSection::Cmdline, b"different".to_vec());
        assert_ne!(measure_pcr11(&other), m1);
    }

    #[test]
    fn pcrsig_section_not_measured() {
        let base = complete_uki();
        let with_sig = base
            .clone()
            .with_section(UkiSection::PcrSig, b"signature".to_vec());
        // Adding the pcrsig output does not change the measurement.
        assert_eq!(measure_pcr11(&base), measure_pcr11(&with_sig));
    }

    #[test]
    fn pcr_policy_build_and_verify() {
        let uki = complete_uki();
        let signer = InMemorySigner {
            fp: "abc123".to_string(),
        };
        let policy = PcrPolicy::build(&uki, &signer);
        let keyset = keys();
        assert!(policy.verify(&uki, &keyset));
        // A different UKI would not match the expected measurement.
        let other = uki
            .clone()
            .with_section(UkiSection::Linux, b"evil".to_vec());
        assert!(!policy.verify(&other, &keyset));
        // A policy signed by an untrusted key is rejected.
        let rogue_policy = PcrPolicy::build(&uki, &InMemorySigner { fp: "rogue".into() });
        assert!(!rogue_policy.verify(&uki, &keyset));
    }

    #[test]
    fn fnv_hash_is_stable() {
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
        assert_ne!(fnv1a(b"a"), fnv1a(b"b"));
        assert_eq!(hex64(0x1234).len(), 16);
    }
}
