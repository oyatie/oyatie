//! The cluster secrets bundle and its PKI building blocks.
//!
//! This module mirrors `pkg/machinery/config/types/v1alpha1` secrets handling
//! and `pkg/machinery/config/generate/secrets`: the persisted *root of trust*
//! for a Talos cluster. A [`SecretsBundle`] holds four certificate authorities
//! (the Talos OS/API CA, the Kubernetes CA, the etcd CA and the front-proxy /
//! aggregator CA), the Kubernetes service-account signing key, the cluster ID
//! and secret, and the bootstrap/join tokens.
//!
//! No external crypto dependency is linked yet. Instead the *shape* and the
//! invariants are modeled faithfully: deterministic key derivation from seeds,
//! self-signed CA construction, leaf signing with monotonic serials, validity
//! windows, and signature verification against the issuing CA. Cryptographic
//! signing is funnelled through the [`Signer`] trait, and file material encoding
//! is funnelled through the [`SecretMaterialEncoder`] trait, so real signing and
//! PEM/DER material backends can replace the deterministic model backends
//! without changing controller call sites.

use crate::certsans::San;
use os_kernel::Role;
use os_kernel::error::{Error, Result};

// ---------------------------------------------------------------------------
// Signer boundary
// ---------------------------------------------------------------------------

/// The crypto signing boundary. Real Talos uses ECDSA-P256/Ed25519; here it is
/// abstracted so the controller logic is testable without a crypto crate.
pub trait Signer {
    /// Produce a signature over `tbs` (to-be-signed bytes).
    fn sign(&self, tbs: &[u8]) -> Vec<u8>;

    /// Verify that `sig` is a valid signature over `tbs` for this signer's key.
    fn verify(&self, tbs: &[u8], sig: &[u8]) -> bool;
}

/// A deterministic in-memory signer keyed by a seed. The "signature" is a
/// keyed FNV hash of the message — collision-resistant enough for the test
/// model and fully reproducible.
#[derive(Debug, Clone)]
pub struct InMemorySigner {
    key: u64,
}

impl InMemorySigner {
    /// Derive a signer from a string seed.
    pub fn from_seed(seed: &str) -> Self {
        InMemorySigner {
            key: fnv1a(seed.as_bytes()),
        }
    }

    /// Derive a signer from a key pair's private material.
    pub fn from_keypair(kp: &KeyPair) -> Self {
        InMemorySigner {
            key: fnv1a(kp.private_der()),
        }
    }

    fn mac(&self, tbs: &[u8]) -> Vec<u8> {
        let mut h = self.key ^ 0x9e3779b97f4a7c15;
        for &b in tbs {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h.to_be_bytes().to_vec()
    }
}

impl Signer for InMemorySigner {
    fn sign(&self, tbs: &[u8]) -> Vec<u8> {
        self.mac(tbs)
    }

    fn verify(&self, tbs: &[u8], sig: &[u8]) -> bool {
        self.mac(tbs) == sig
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Lowercase hex encoding helper.
pub fn hex(bytes: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(H[(b >> 4) as usize] as char);
        s.push(H[(b & 0x0f) as usize] as char);
    }
    s
}

// ---------------------------------------------------------------------------
// Key material
// ---------------------------------------------------------------------------

/// An asymmetric key pair. Private material is opaque bytes; the public key is
/// a deterministic transform so identity comparisons are stable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPair {
    private_der: Vec<u8>,
    public_der: Vec<u8>,
}

impl KeyPair {
    /// Build from explicit material.
    pub fn new(private_der: Vec<u8>, public_der: Vec<u8>) -> Self {
        KeyPair {
            private_der,
            public_der,
        }
    }

    /// Deterministically derive a key pair from a seed string.
    pub fn from_seed(seed: &str) -> Self {
        let private_der = seed.as_bytes().to_vec();
        let public_der: Vec<u8> = seed.bytes().rev().map(|b| b ^ 0xA5).collect();
        KeyPair {
            private_der,
            public_der,
        }
    }

    /// The private key bytes.
    pub fn private_der(&self) -> &[u8] {
        &self.private_der
    }

    /// The public key bytes.
    pub fn public_der(&self) -> &[u8] {
        &self.public_der
    }

    /// Whether `public_der` is this pair's public key.
    pub fn matches_public(&self, public_der: &[u8]) -> bool {
        self.public_der == public_der
    }

    /// A short stable fingerprint over the public key.
    pub fn fingerprint(&self) -> String {
        hex(&fnv1a(&self.public_der).to_be_bytes())
    }

    /// Deterministic model bytes for private-key file projection.
    ///
    /// This is intentionally not real PEM/DER. It is a stable, inspectable
    /// encoding used by the Rust port until a crypto backend is wired in.
    pub fn model_private_key_bytes(&self) -> Vec<u8> {
        format!(
            "KUBEROS-MODEL-PRIVATE-KEY\nprivate={}\npublic={}\n",
            hex(&self.private_der),
            hex(&self.public_der)
        )
        .into_bytes()
    }

    /// Deterministic model bytes for public-key file projection.
    ///
    /// This is intentionally not real PEM/DER. It keeps service-account public
    /// key material distinct from private key material in rendered outputs.
    pub fn model_public_key_bytes(&self) -> Vec<u8> {
        format!(
            "KUBEROS-MODEL-PUBLIC-KEY\npublic={}\n",
            hex(&self.public_der)
        )
        .into_bytes()
    }
}

// ---------------------------------------------------------------------------
// Validity window
// ---------------------------------------------------------------------------

/// A certificate validity window in whole Unix seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Validity {
    /// Not-before, Unix seconds.
    pub not_before: u64,
    /// Not-after, Unix seconds.
    pub not_after: u64,
}

impl Validity {
    /// Construct from a start and a non-zero TTL.
    pub fn from_duration(not_before: u64, ttl_secs: u64) -> Result<Self> {
        if ttl_secs == 0 {
            return Err(Error::invalid("zero TTL"));
        }
        let not_after = not_before
            .checked_add(ttl_secs)
            .ok_or_else(|| Error::invalid("validity overflow"))?;
        Ok(Validity {
            not_before,
            not_after,
        })
    }

    /// Whether `now` is within the window.
    pub fn contains(&self, now: u64) -> bool {
        now >= self.not_before && now < self.not_after
    }

    /// Whether the window has expired at `now`.
    pub fn is_expired(&self, now: u64) -> bool {
        now >= self.not_after
    }

    /// Seconds of life remaining at `now`.
    pub fn remaining(&self, now: u64) -> u64 {
        self.not_after.saturating_sub(now)
    }

    /// Total lifetime in seconds.
    pub fn total(&self) -> u64 {
        self.not_after.saturating_sub(self.not_before)
    }
}

// ---------------------------------------------------------------------------
// Certificates
// ---------------------------------------------------------------------------

/// Key-usage class of a certificate, mirroring the cases Talos cares about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertUsage {
    /// A certificate authority (may sign other certs).
    CertificateAuthority,
    /// A TLS server certificate.
    ServerAuth,
    /// A TLS client certificate.
    ClientAuth,
    /// A certificate valid for both client and server auth (etcd peers).
    ServerAndClientAuth,
}

impl CertUsage {
    /// Whether a cert with this usage may sign other certificates.
    pub fn can_sign(self) -> bool {
        matches!(self, CertUsage::CertificateAuthority)
    }

    /// Whether this usage permits TLS server authentication.
    pub fn server_auth(self) -> bool {
        matches!(self, CertUsage::ServerAuth | CertUsage::ServerAndClientAuth)
    }

    /// Whether this usage permits TLS client authentication.
    pub fn client_auth(self) -> bool {
        matches!(self, CertUsage::ClientAuth | CertUsage::ServerAndClientAuth)
    }

    fn tag(self) -> u8 {
        match self {
            CertUsage::CertificateAuthority => 0,
            CertUsage::ServerAuth => 1,
            CertUsage::ClientAuth => 2,
            CertUsage::ServerAndClientAuth => 3,
        }
    }
}

/// A distinguished name: a common name plus organizational units (used by
/// Talos to carry RBAC roles as `os:<role>` and Kubernetes group membership as
/// `system:masters`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Subject {
    /// Common name.
    pub common_name: String,
    /// Organizations / groups.
    pub organizations: Vec<String>,
}

impl Subject {
    /// A subject with only a common name.
    pub fn common(cn: impl Into<String>) -> Self {
        Subject {
            common_name: cn.into(),
            organizations: Vec::new(),
        }
    }

    /// Add an organization (group), chaining.
    pub fn with_org(mut self, org: impl Into<String>) -> Self {
        self.organizations.push(org.into());
        self
    }

    /// Add a Talos RBAC role as an `os:<role>` organization.
    pub fn with_role(mut self, role: Role) -> Self {
        self.organizations.push(role.as_ou().to_string());
        self
    }

    /// RFC-4514-ish string form.
    pub fn to_rfc(&self) -> String {
        let mut parts = vec![format!("CN={}", self.common_name)];
        for o in &self.organizations {
            parts.push(format!("O={o}"));
        }
        parts.join(",")
    }
}

/// A modeled X.509 certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Certificate {
    /// Serial number assigned by the issuing CA.
    pub serial: u64,
    /// Subject distinguished name.
    pub subject: Subject,
    /// Issuer distinguished name (equals subject for a self-signed CA).
    pub issuer: Subject,
    /// Validity window.
    pub validity: Validity,
    /// Key usage class.
    pub usage: CertUsage,
    /// Subject alternative names.
    pub sans: Vec<San>,
    /// The subject's public key.
    pub public_key_der: Vec<u8>,
    /// The CA signature over [`Certificate::tbs_bytes`].
    pub signature: Vec<u8>,
}

impl Certificate {
    /// The deterministic to-be-signed byte encoding.
    pub fn tbs_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.serial.to_be_bytes());
        out.extend_from_slice(self.subject.to_rfc().as_bytes());
        out.push(b'|');
        out.extend_from_slice(self.issuer.to_rfc().as_bytes());
        out.push(b'|');
        out.extend_from_slice(&self.validity.not_before.to_be_bytes());
        out.extend_from_slice(&self.validity.not_after.to_be_bytes());
        out.push(self.usage.tag());
        for san in &self.sans {
            out.push(b'|');
            out.extend_from_slice(san.to_string_repr().as_bytes());
        }
        out.push(b'|');
        out.extend_from_slice(&self.public_key_der);
        out
    }

    /// Whether this certificate is a CA.
    pub fn is_ca(&self) -> bool {
        self.usage.can_sign()
    }

    /// Whether the cert is valid at `now` (window only).
    pub fn is_valid_at(&self, now: u64) -> bool {
        self.validity.contains(now)
    }

    /// Whether a DNS SAN is present.
    pub fn covers_dns(&self, name: &str) -> bool {
        self.sans
            .iter()
            .any(|s| matches!(s, San::Dns(d) if d == name))
    }

    /// Structural validation: non-empty CN, non-empty key, ordered validity.
    pub fn validate(&self) -> Result<()> {
        if self.subject.common_name.trim().is_empty() {
            return Err(Error::invalid("certificate has empty common name"));
        }
        if self.public_key_der.is_empty() {
            return Err(Error::invalid("certificate has empty public key"));
        }
        if self.validity.not_after <= self.validity.not_before {
            return Err(Error::invalid("certificate validity is not ordered"));
        }
        Ok(())
    }

    /// Deterministic model bytes for certificate file projection.
    ///
    /// This is intentionally not real PEM/DER. It captures the certificate
    /// shape that the Rust migration currently models: subject, issuer,
    /// validity, usage, SANs, public key, and signature.
    pub fn model_certificate_bytes(&self) -> Vec<u8> {
        let sans = self
            .sans
            .iter()
            .map(San::to_string_repr)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "KUBEROS-MODEL-CERTIFICATE\nserial={}\nsubject={}\nissuer={}\nnot_before={}\nnot_after={}\nusage={:?}\nsans={}\npublic_key={}\nsignature={}\n",
            self.serial,
            self.subject.to_rfc(),
            self.issuer.to_rfc(),
            self.validity.not_before,
            self.validity.not_after,
            self.usage,
            sans,
            hex(&self.public_key_der),
            hex(&self.signature)
        )
        .into_bytes()
    }
}

// ---------------------------------------------------------------------------
// File material encoding boundary
// ---------------------------------------------------------------------------

/// Encodes certificate and key objects into the opaque bytes written to secret
/// files.
///
/// The default [`ModelSecretMaterialEncoder`] preserves the current
/// deterministic model bytes. A future real backend can implement this trait to
/// emit PEM/DER while the controller and projection code continue to pass
/// structured [`Certificate`] and [`KeyPair`] values.
pub trait SecretMaterialEncoder {
    /// Encode a certificate for a `*.crt` secret file.
    fn certificate_bytes(&self, cert: &Certificate) -> Result<Vec<u8>>;

    /// Encode a private key for a `*.key` secret file.
    fn private_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>>;

    /// Encode a public key for a `*.pub` secret file.
    fn public_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>>;
}

/// Deterministic material encoder used by the current Rust model.
#[derive(Debug, Clone, Copy, Default)]
pub struct ModelSecretMaterialEncoder;

/// PEM-armored deterministic material encoder used to exercise the file-format
/// seam before a real X.509/key backend is wired in.
///
/// The body of each PEM block is still the deterministic model material. This
/// encoder proves the projection can carry PEM-style opaque bytes without
/// changing controller inputs or Kubernetes consumers.
#[derive(Debug, Clone, Copy, Default)]
pub struct ModelPemSecretMaterialEncoder;

fn non_empty_material(kind: &str, bytes: Vec<u8>) -> Result<Vec<u8>> {
    if bytes.is_empty() {
        return Err(Error::invalid(format!("{kind} material is empty")));
    }
    Ok(bytes)
}

const BASE64_STANDARD: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_standard(data: &[u8]) -> String {
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

fn validate_pem_label(label: &str) -> Result<()> {
    if label.is_empty() {
        return Err(Error::invalid("PEM label is empty"));
    }
    if label.starts_with(' ') || label.ends_with(' ') || label.contains("  ") {
        return Err(Error::invalid("PEM label has invalid spacing"));
    }
    if !label
        .bytes()
        .all(|b| matches!(b, b'A'..=b'Z' | b'0'..=b'9' | b' ' | b'-'))
    {
        return Err(Error::invalid("PEM label has invalid characters"));
    }
    Ok(())
}

fn pem_block(label: &str, data: &[u8]) -> Result<Vec<u8>> {
    validate_pem_label(label)?;

    let encoded = base64_standard(data);
    let mut out = String::with_capacity(encoded.len() + label.len() * 2 + 40);
    out.push_str("-----BEGIN ");
    out.push_str(label);
    out.push_str("-----\n");
    for chunk in encoded.as_bytes().chunks(64) {
        let line =
            core::str::from_utf8(chunk).map_err(|_| Error::invalid("base64 output is invalid"))?;
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("-----END ");
    out.push_str(label);
    out.push_str("-----\n");
    Ok(out.into_bytes())
}

impl SecretMaterialEncoder for ModelSecretMaterialEncoder {
    fn certificate_bytes(&self, cert: &Certificate) -> Result<Vec<u8>> {
        non_empty_material("certificate", cert.model_certificate_bytes())
    }

    fn private_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>> {
        non_empty_material("private key", keypair.model_private_key_bytes())
    }

    fn public_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>> {
        non_empty_material("public key", keypair.model_public_key_bytes())
    }
}

impl SecretMaterialEncoder for ModelPemSecretMaterialEncoder {
    fn certificate_bytes(&self, cert: &Certificate) -> Result<Vec<u8>> {
        non_empty_material(
            "certificate",
            pem_block("CERTIFICATE", &cert.model_certificate_bytes())?,
        )
    }

    fn private_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>> {
        non_empty_material(
            "private key",
            pem_block("PRIVATE KEY", &keypair.model_private_key_bytes())?,
        )
    }

    fn public_key_bytes(&self, keypair: &KeyPair) -> Result<Vec<u8>> {
        non_empty_material(
            "public key",
            pem_block("PUBLIC KEY", &keypair.model_public_key_bytes())?,
        )
    }
}

/// A certificate authority: its self-signed certificate, its key pair, a
/// signer, and a monotonic serial counter.
#[derive(Debug, Clone)]
pub struct CertificateAuthority {
    cert: Certificate,
    keypair: KeyPair,
    signer: InMemorySigner,
    next_serial: u64,
}

impl CertificateAuthority {
    /// Bootstrap a self-signed CA named `name`, valid from `now` for `ttl_secs`.
    pub fn bootstrap(name: &str, keypair: KeyPair, now: u64, ttl_secs: u64) -> Result<Self> {
        let signer = InMemorySigner::from_keypair(&keypair);
        let subject = Subject::common(name);
        let validity = Validity::from_duration(now, ttl_secs)?;
        let mut cert = Certificate {
            serial: 1,
            subject: subject.clone(),
            issuer: subject,
            validity,
            usage: CertUsage::CertificateAuthority,
            sans: Vec::new(),
            public_key_der: keypair.public_der().to_vec(),
            signature: Vec::new(),
        };
        cert.signature = signer.sign(&cert.tbs_bytes());
        cert.validate()?;
        Ok(CertificateAuthority {
            cert,
            keypair,
            signer,
            next_serial: 2,
        })
    }

    /// Deterministically bootstrap a CA from a seed (test/generation helper).
    pub fn from_seed(name: &str, seed: &str, now: u64, ttl_secs: u64) -> Result<Self> {
        Self::bootstrap(name, KeyPair::from_seed(seed), now, ttl_secs)
    }

    /// The CA certificate.
    pub fn certificate(&self) -> &Certificate {
        &self.cert
    }

    /// The CA key pair.
    pub fn keypair(&self) -> &KeyPair {
        &self.keypair
    }

    /// The serial the next leaf will receive.
    pub fn peek_serial(&self) -> u64 {
        self.next_serial
    }

    /// A stable identity fingerprint over the CA's public key. Controllers mix
    /// this into their leaf input fingerprints so that *rotating* the CA (which
    /// changes the key but resets the serial to 1) still forces every leaf to be
    /// re-issued.
    pub fn identity_fingerprint(&self) -> u64 {
        fnv1a(&self.cert.public_key_der)
    }

    /// Whether the CA itself is expired at `now`.
    pub fn is_expired(&self, now: u64) -> bool {
        self.cert.validity.is_expired(now)
    }

    /// Sign a leaf certificate for `subject` with the given public key, usage,
    /// SANs and TTL, assigning a fresh serial.
    #[allow(clippy::too_many_arguments)]
    pub fn issue(
        &mut self,
        subject: Subject,
        public_key_der: Vec<u8>,
        usage: CertUsage,
        sans: Vec<San>,
        now: u64,
        ttl_secs: u64,
    ) -> Result<Certificate> {
        if self.is_expired(now) {
            return Err(Error::invalid_state("issuing CA is expired"));
        }
        if usage.can_sign() {
            return Err(Error::invalid("leaf may not request CA usage"));
        }
        let validity = Validity::from_duration(now, ttl_secs)?;
        let serial = self.next_serial;
        let mut leaf = Certificate {
            serial,
            subject,
            issuer: self.cert.subject.clone(),
            validity,
            usage,
            sans,
            public_key_der,
            signature: Vec::new(),
        };
        leaf.signature = self.signer.sign(&leaf.tbs_bytes());
        leaf.validate()?;
        self.next_serial = self
            .next_serial
            .checked_add(1)
            .ok_or_else(|| Error::Other("serial overflow".into()))?;
        Ok(leaf)
    }

    /// Verify `cert` was issued by this CA and is valid at `now`.
    pub fn verify(&self, cert: &Certificate, now: u64) -> Result<()> {
        cert.validate()?;
        if cert.issuer != self.cert.subject {
            return Err(Error::invalid("issuer is not this CA"));
        }
        if !cert.is_valid_at(now) {
            return Err(Error::invalid_state("certificate not valid at this time"));
        }
        if !self.signer.verify(&cert.tbs_bytes(), &cert.signature) {
            return Err(Error::invalid("signature does not verify against CA"));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Cluster identity & tokens
// ---------------------------------------------------------------------------

/// The cluster identity: a stable cluster ID plus a shared secret. Talos uses
/// these for the cluster discovery service and config encryption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterIdentity {
    /// The cluster ID (a base-ish stable token).
    pub id: String,
    /// The cluster shared secret.
    pub secret: String,
}

impl ClusterIdentity {
    /// Derive a deterministic identity from a seed.
    pub fn from_seed(seed: &str) -> Self {
        ClusterIdentity {
            id: hex(&fnv1a(format!("id:{seed}").as_bytes()).to_be_bytes()),
            secret: hex(&fnv1a(format!("secret:{seed}").as_bytes()).to_be_bytes()),
        }
    }

    /// Validate that neither field is empty.
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() || self.secret.is_empty() {
            return Err(Error::invalid("cluster identity has empty id or secret"));
        }
        Ok(())
    }
}

/// A bootstrap/join token: `<id>.<secret>` per the Kubernetes bootstrap-token
/// format Talos reuses for both the trustd join token and the kubeadm-style
/// bootstrap token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Token id (6 chars in Kubernetes; we keep it flexible).
    pub id: String,
    /// Token secret.
    pub secret: String,
}

impl Token {
    /// Parse from `id.secret`.
    pub fn parse(s: &str) -> Result<Self> {
        let (id, secret) = s
            .split_once('.')
            .ok_or_else(|| Error::parse("token must be 'id.secret'"))?;
        if id.is_empty() || secret.is_empty() {
            return Err(Error::parse("token has empty id or secret"));
        }
        Ok(Token {
            id: id.to_string(),
            secret: secret.to_string(),
        })
    }

    /// Derive a deterministic token from a seed.
    pub fn from_seed(seed: &str) -> Self {
        let id = &hex(&fnv1a(format!("tid:{seed}").as_bytes()).to_be_bytes())[..6];
        let secret = &hex(&fnv1a(format!("tsec:{seed}").as_bytes()).to_be_bytes())[..16];
        Token {
            id: id.to_string(),
            secret: secret.to_string(),
        }
    }

    /// The wire form `id.secret`.
    pub fn to_token_string(&self) -> String {
        format!("{}.{}", self.id, self.secret)
    }
}

// ---------------------------------------------------------------------------
// The secrets bundle
// ---------------------------------------------------------------------------

/// Which root certificate authority a leaf is signed by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CaKind {
    /// The Talos OS / machine API CA (`apid`, `trustd`, machined).
    Os,
    /// The Kubernetes CA (apiserver, kubelet, controller-manager, scheduler).
    Kubernetes,
    /// The etcd CA (etcd peer & client certs).
    Etcd,
    /// The front-proxy / aggregator CA.
    Aggregator,
}

impl CaKind {
    /// The CA common-name Talos assigns.
    pub fn common_name(self) -> &'static str {
        match self {
            CaKind::Os => "talos",
            CaKind::Kubernetes => "kubernetes",
            CaKind::Etcd => "etcd",
            CaKind::Aggregator => "front-proxy",
        }
    }

    /// All CA kinds in a stable order.
    pub fn all() -> [CaKind; 4] {
        [
            CaKind::Os,
            CaKind::Kubernetes,
            CaKind::Etcd,
            CaKind::Aggregator,
        ]
    }
}

/// The default 10-year CA lifetime Talos uses.
pub const CA_TTL_SECS: u64 = 10 * 365 * 24 * 60 * 60;

/// The complete cluster secrets bundle: the persisted root of trust.
#[derive(Debug, Clone)]
pub struct SecretsBundle {
    os_ca: CertificateAuthority,
    k8s_ca: CertificateAuthority,
    etcd_ca: CertificateAuthority,
    aggregator_ca: CertificateAuthority,
    service_account_key: KeyPair,
    cluster: ClusterIdentity,
    bootstrap_token: Token,
    join_token: Token,
}

impl SecretsBundle {
    /// Deterministically generate a full bundle from a single cluster seed and
    /// a creation time. This mirrors `secrets.NewBundle` / `genv1alpha1`.
    pub fn generate(seed: &str, now: u64) -> Result<Self> {
        Ok(SecretsBundle {
            os_ca: CertificateAuthority::from_seed(
                CaKind::Os.common_name(),
                &format!("{seed}:os-ca"),
                now,
                CA_TTL_SECS,
            )?,
            k8s_ca: CertificateAuthority::from_seed(
                CaKind::Kubernetes.common_name(),
                &format!("{seed}:k8s-ca"),
                now,
                CA_TTL_SECS,
            )?,
            etcd_ca: CertificateAuthority::from_seed(
                CaKind::Etcd.common_name(),
                &format!("{seed}:etcd-ca"),
                now,
                CA_TTL_SECS,
            )?,
            aggregator_ca: CertificateAuthority::from_seed(
                CaKind::Aggregator.common_name(),
                &format!("{seed}:agg-ca"),
                now,
                CA_TTL_SECS,
            )?,
            service_account_key: KeyPair::from_seed(&format!("{seed}:sa")),
            cluster: ClusterIdentity::from_seed(seed),
            bootstrap_token: Token::from_seed(&format!("{seed}:bootstrap")),
            join_token: Token::from_seed(&format!("{seed}:join")),
        })
    }

    /// Borrow a CA by kind.
    pub fn ca(&self, kind: CaKind) -> &CertificateAuthority {
        match kind {
            CaKind::Os => &self.os_ca,
            CaKind::Kubernetes => &self.k8s_ca,
            CaKind::Etcd => &self.etcd_ca,
            CaKind::Aggregator => &self.aggregator_ca,
        }
    }

    /// Mutably borrow a CA by kind (for issuing leaves).
    pub fn ca_mut(&mut self, kind: CaKind) -> &mut CertificateAuthority {
        match kind {
            CaKind::Os => &mut self.os_ca,
            CaKind::Kubernetes => &mut self.k8s_ca,
            CaKind::Etcd => &mut self.etcd_ca,
            CaKind::Aggregator => &mut self.aggregator_ca,
        }
    }

    /// The Kubernetes service-account signing key.
    pub fn service_account_key(&self) -> &KeyPair {
        &self.service_account_key
    }

    /// The cluster identity.
    pub fn cluster(&self) -> &ClusterIdentity {
        &self.cluster
    }

    /// The bootstrap token.
    pub fn bootstrap_token(&self) -> &Token {
        &self.bootstrap_token
    }

    /// The trustd join token.
    pub fn join_token(&self) -> &Token {
        &self.join_token
    }

    /// Replace (rotate) the CA of a given kind with a freshly bootstrapped one.
    /// Returns the new CA's certificate. Used by [`crate::rotation`].
    pub fn rotate_ca(&mut self, kind: CaKind, keypair: KeyPair, now: u64) -> Result<Certificate> {
        let ca = CertificateAuthority::bootstrap(kind.common_name(), keypair, now, CA_TTL_SECS)?;
        let cert = ca.certificate().clone();
        match kind {
            CaKind::Os => self.os_ca = ca,
            CaKind::Kubernetes => self.k8s_ca = ca,
            CaKind::Etcd => self.etcd_ca = ca,
            CaKind::Aggregator => self.aggregator_ca = ca,
        }
        Ok(cert)
    }

    /// Validate the whole bundle: every CA is a self-signed CA, the cluster
    /// identity is populated, and the tokens are well-formed.
    pub fn validate(&self) -> Result<()> {
        for kind in CaKind::all() {
            let ca = self.ca(kind);
            if !ca.certificate().is_ca() {
                return Err(Error::invalid(format!(
                    "{} CA is not a CA",
                    kind.common_name()
                )));
            }
            ca.certificate().validate()?;
        }
        self.cluster.validate()?;
        Token::parse(&self.bootstrap_token.to_token_string())?;
        Token::parse(&self.join_token.to_token_string())?;
        if self.service_account_key.public_der().is_empty() {
            return Err(Error::invalid("service-account key is empty"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ca_bootstrap_is_self_signed_and_verifies() {
        let ca = CertificateAuthority::from_seed("talos", "seed", 1000, CA_TTL_SECS).unwrap();
        assert!(ca.certificate().is_ca());
        assert_eq!(ca.certificate().issuer, ca.certificate().subject);
        assert!(ca.verify(ca.certificate(), 5000).is_ok());
    }

    #[test]
    fn ca_issues_leaf_with_monotonic_serial() {
        let mut ca = CertificateAuthority::from_seed("talos", "seed", 1000, CA_TTL_SECS).unwrap();
        let kp = KeyPair::from_seed("node-1");
        let a = ca
            .issue(
                Subject::common("a"),
                kp.public_der().to_vec(),
                CertUsage::ClientAuth,
                vec![],
                1000,
                3600,
            )
            .unwrap();
        let b = ca
            .issue(
                Subject::common("b"),
                kp.public_der().to_vec(),
                CertUsage::ClientAuth,
                vec![],
                1000,
                3600,
            )
            .unwrap();
        assert_eq!(a.serial + 1, b.serial);
        assert!(ca.verify(&a, 2000).is_ok());
    }

    #[test]
    fn ca_rejects_ca_usage_leaf_and_expired_issuer() {
        let mut ca = CertificateAuthority::from_seed("talos", "seed", 1000, 100).unwrap();
        let kp = KeyPair::from_seed("x");
        assert!(
            ca.issue(
                Subject::common("x"),
                kp.public_der().to_vec(),
                CertUsage::CertificateAuthority,
                vec![],
                1000,
                50
            )
            .is_err()
        );
        // After CA expiry, issuance fails.
        assert!(
            ca.issue(
                Subject::common("x"),
                kp.public_der().to_vec(),
                CertUsage::ClientAuth,
                vec![],
                2000,
                50
            )
            .is_err()
        );
    }

    #[test]
    fn foreign_ca_does_not_verify() {
        let mut ca = CertificateAuthority::from_seed("talos", "a", 1000, CA_TTL_SECS).unwrap();
        let other = CertificateAuthority::from_seed("talos", "b", 1000, CA_TTL_SECS).unwrap();
        let kp = KeyPair::from_seed("n");
        let leaf = ca
            .issue(
                Subject::common("n"),
                kp.public_der().to_vec(),
                CertUsage::ClientAuth,
                vec![],
                1000,
                3600,
            )
            .unwrap();
        assert!(other.verify(&leaf, 2000).is_err());
    }

    #[test]
    fn token_round_trips_and_validates() {
        let t = Token::from_seed("cluster");
        let s = t.to_token_string();
        assert_eq!(Token::parse(&s).unwrap(), t);
        assert!(Token::parse("noseparator").is_err());
        assert!(Token::parse(".empty").is_err());
    }

    #[test]
    fn bundle_generate_is_deterministic_and_valid() {
        let a = SecretsBundle::generate("cluster-x", 1000).unwrap();
        let b = SecretsBundle::generate("cluster-x", 1000).unwrap();
        a.validate().unwrap();
        // Same seed -> same root CA public keys & cluster id.
        assert_eq!(
            a.ca(CaKind::Kubernetes).keypair().public_der(),
            b.ca(CaKind::Kubernetes).keypair().public_der()
        );
        assert_eq!(a.cluster().id, b.cluster().id);
        // Different CAs have distinct keys.
        assert_ne!(
            a.ca(CaKind::Os).keypair().public_der(),
            a.ca(CaKind::Etcd).keypair().public_der()
        );
    }

    #[test]
    fn rotate_ca_replaces_root() {
        let mut bundle = SecretsBundle::generate("c", 1000).unwrap();
        let old = bundle
            .ca(CaKind::Kubernetes)
            .certificate()
            .public_key_der
            .clone();
        bundle
            .rotate_ca(CaKind::Kubernetes, KeyPair::from_seed("new-k8s"), 2000)
            .unwrap();
        let new = bundle
            .ca(CaKind::Kubernetes)
            .certificate()
            .public_key_der
            .clone();
        assert_ne!(old, new);
        bundle.validate().unwrap();
    }

    #[test]
    fn model_bytes_are_stable_and_distinguish_public_private_material() {
        let bundle = SecretsBundle::generate("encoding", 1000).unwrap();
        let ca = bundle.ca(CaKind::Kubernetes);
        let cert = ca.certificate().model_certificate_bytes();
        let cert_text = String::from_utf8(cert).unwrap();
        assert!(cert_text.contains("KUBEROS-MODEL-CERTIFICATE"));
        assert!(cert_text.contains("subject=CN=kubernetes"));
        assert!(cert_text.contains("usage=CertificateAuthority"));

        let private = ca.keypair().model_private_key_bytes();
        let public = ca.keypair().model_public_key_bytes();
        assert_ne!(private, public);
        assert!(String::from_utf8(private).unwrap().contains("private="));
        assert!(!String::from_utf8(public).unwrap().contains("private="));
    }

    #[test]
    fn model_secret_material_encoder_preserves_current_file_bytes() {
        let bundle = SecretsBundle::generate("material-encoder", 1000).unwrap();
        let ca = bundle.ca(CaKind::Kubernetes);
        let encoder = ModelSecretMaterialEncoder;

        assert_eq!(
            encoder.certificate_bytes(ca.certificate()).unwrap(),
            ca.certificate().model_certificate_bytes()
        );
        assert_eq!(
            encoder.private_key_bytes(ca.keypair()).unwrap(),
            ca.keypair().model_private_key_bytes()
        );
        assert_eq!(
            encoder
                .public_key_bytes(bundle.service_account_key())
                .unwrap(),
            bundle.service_account_key().model_public_key_bytes()
        );
    }

    #[test]
    fn model_pem_secret_material_encoder_armors_current_model_bytes() {
        let bundle = SecretsBundle::generate("pem-material-encoder", 1000).unwrap();
        let ca = bundle.ca(CaKind::Kubernetes);
        let encoder = ModelPemSecretMaterialEncoder;

        let cert = encoder.certificate_bytes(ca.certificate()).unwrap();
        let private = encoder.private_key_bytes(ca.keypair()).unwrap();
        let public = encoder
            .public_key_bytes(bundle.service_account_key())
            .unwrap();

        assert_eq!(
            cert,
            pem_block("CERTIFICATE", &ca.certificate().model_certificate_bytes()).unwrap()
        );
        assert_eq!(
            private,
            pem_block("PRIVATE KEY", &ca.keypair().model_private_key_bytes()).unwrap()
        );
        assert_eq!(
            public,
            pem_block(
                "PUBLIC KEY",
                &bundle.service_account_key().model_public_key_bytes()
            )
            .unwrap()
        );

        for material in [&cert, &private, &public] {
            let text = String::from_utf8(material.clone()).unwrap();
            assert!(text.starts_with("-----BEGIN "));
            assert!(text.contains("-----END "));
            assert!(!text.contains("KUBEROS-MODEL-"));
        }
    }

    #[test]
    fn model_pem_secret_material_encoder_wraps_body_to_sixty_four_columns() {
        let cert = Certificate {
            serial: 42,
            subject: Subject::common("long"),
            issuer: Subject::common("issuer"),
            validity: Validity::from_duration(1000, 3600).unwrap(),
            usage: CertUsage::ClientAuth,
            sans: vec![],
            public_key_der: vec![0xAB; 96],
            signature: vec![0xCD; 96],
        };

        let pem = String::from_utf8(
            ModelPemSecretMaterialEncoder
                .certificate_bytes(&cert)
                .unwrap(),
        )
        .unwrap();
        let lines = pem.lines().collect::<Vec<_>>();

        assert_eq!(lines.first().copied(), Some("-----BEGIN CERTIFICATE-----"));
        assert_eq!(lines.last().copied(), Some("-----END CERTIFICATE-----"));
        assert!(
            lines[1..lines.len() - 1]
                .iter()
                .all(|line| !line.is_empty())
        );
        assert!(
            lines[1..lines.len() - 1]
                .iter()
                .all(|line| line.len() <= 64)
        );
        assert!(
            lines[1..lines.len() - 2]
                .iter()
                .all(|line| line.len() == 64)
        );
    }

    #[test]
    fn model_pem_secret_material_encoder_rejects_invalid_pem_labels() {
        assert_eq!(pem_block("", b"x").unwrap_err().kind(), "invalid");
        assert_eq!(
            pem_block(" PRIVATE KEY", b"x").unwrap_err().kind(),
            "invalid"
        );
        assert_eq!(
            pem_block("PRIVATE_key", b"x").unwrap_err().kind(),
            "invalid"
        );
    }
}
