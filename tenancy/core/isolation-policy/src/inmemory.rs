//! Deterministic in-memory stand-ins for the isolation-policy ports.
//!
//! These exist so the pure logic in [`crate::rls`] and [`crate::claims`] can be
//! exercised end-to-end with no database, no key store and no network. They are
//! test doubles and are named as such.
//!
//! **[`UnsignedTokenIssuer`] is not a security control.** Its checksum is
//! FNV-1a-64 — a non-cryptographic hash any caller can recompute — so it
//! detects accidental corruption and provides zero forgery resistance. The
//! crate-level Gaps paragraph says the same thing at the top of the file
//! because it is the single most important limitation of this crate.

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::claims::{decode_claim_pairs, encode_claim_pairs};
use crate::rls::{render_policy_ddl, validate_rendered_ddl};
use crate::{
    IsolationKernelError, JwtIssuer, JwtVerifier, RlsInstaller, RlsPolicy, SigningKeyStore,
};

/// FNV-1a-64 offset basis.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a-64 prime.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// The wire prefix every token this module mints carries. It names the format
/// AND the fact that the format is unsigned.
pub const UNSIGNED_TOKEN_PREFIX: &str = "oya-unsigned.v1.";

/// FNV-1a-64 over `bytes`.
///
/// Non-cryptographic by construction: it is here because the lockfile is frozen
/// and a real digest would need a dependency. Never use it to authenticate.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// `fnv1a64` rendered as the fixed-width lowercase hex this module embeds in
/// tokens and fingerprints.
pub fn fnv1a64_hex(bytes: &[u8]) -> String {
    format!("{:016x}", fnv1a64(bytes))
}

/// An unsigned, deterministic token issuer/verifier/key-store test double.
///
/// Round-trips claim pairs through the canonical encoding with an FNV checksum
/// appended. Tampering with the payload alone is detected; re-checksumming a
/// forged payload is trivially possible, which is exactly the gap a real
/// Ed25519 implementation closes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsignedTokenIssuer {
    key_label: String,
}

impl UnsignedTokenIssuer {
    /// Build an issuer whose fingerprint is derived from `key_label`.
    pub fn new(key_label: impl Into<String>) -> Self {
        Self {
            key_label: key_label.into(),
        }
    }

    /// The deterministic fingerprint advertised for this "key".
    pub fn fingerprint(&self) -> String {
        format!("fnv1a64:{}", fnv1a64_hex(self.key_label.as_bytes()))
    }

    fn checksum(&self, payload: &str) -> String {
        let mut material = self.fingerprint();
        material.push('.');
        material.push_str(payload);
        fnv1a64_hex(material.as_bytes())
    }
}

impl JwtIssuer for UnsignedTokenIssuer {
    fn issue(&self, claims: &[(String, String)]) -> Result<String, IsolationKernelError> {
        if claims.is_empty() {
            return Err(IsolationKernelError::JwtSignFailed);
        }
        let payload = encode_claim_pairs(claims);
        let checksum = self.checksum(&payload);
        Ok(format!("{UNSIGNED_TOKEN_PREFIX}{payload}.{checksum}"))
    }
}

impl JwtVerifier for UnsignedTokenIssuer {
    fn verify(&self, token: &str) -> Result<Vec<(String, String)>, IsolationKernelError> {
        let body = token
            .strip_prefix(UNSIGNED_TOKEN_PREFIX)
            .ok_or(IsolationKernelError::JwtVerifyFailed)?;
        // The checksum is fixed-width hex and holds no '.', so the LAST '.'
        // separates it from a payload that may contain any byte.
        let (payload, checksum) = body
            .rsplit_once('.')
            .ok_or(IsolationKernelError::JwtVerifyFailed)?;
        if checksum != self.checksum(payload) {
            return Err(IsolationKernelError::JwtVerifyFailed);
        }
        decode_claim_pairs(payload).map_err(|_| IsolationKernelError::JwtVerifyFailed)
    }
}

impl SigningKeyStore for UnsignedTokenIssuer {
    fn current_key_fingerprint(&self) -> Result<String, IsolationKernelError> {
        if self.key_label.is_empty() {
            return Err(IsolationKernelError::KeyStoreUnavailable);
        }
        Ok(self.fingerprint())
    }
}

/// An in-memory [`RlsInstaller`]: renders the DDL, refuses anything that does
/// not FORCE row-level security, and remembers what it "applied" so
/// [`RlsInstaller::verify`] can compare against a fresh render.
///
/// It stands in for the Postgres adapter; it cannot substitute for the real
/// post-install catalog probe (see the crate Gaps paragraph).
#[derive(Debug, Default)]
pub struct InMemoryRlsInstaller {
    applied: RefCell<BTreeMap<String, String>>,
}

impl InMemoryRlsInstaller {
    /// A fresh installer with nothing applied.
    pub fn new() -> Self {
        Self::default()
    }

    /// The DDL recorded for `policy`'s table, if any.
    pub fn applied_ddl(&self, qualified_name: &str) -> Option<String> {
        self.applied
            .try_borrow()
            .ok()
            .and_then(|applied| applied.get(qualified_name).cloned())
    }

    /// How many tables have had a policy applied.
    pub fn applied_count(&self) -> usize {
        self.applied.try_borrow().map_or(0, |applied| applied.len())
    }

    /// Apply DDL this installer did not render itself.
    ///
    /// This is the shape the real Postgres adapter has — it executes a block of
    /// text — and it is where the IP-006 halt condition can actually bite:
    /// [`validate_rendered_ddl`] is checking bytes that came from somewhere
    /// else, not a literal it just built. [`RlsInstaller::install`] renders and
    /// then routes through here, so both paths enforce the same rule in the
    /// same place.
    pub fn install_rendered(
        &self,
        qualified_name: &str,
        ddl: &str,
    ) -> Result<(), IsolationKernelError> {
        validate_rendered_ddl(qualified_name, ddl).map_err(|source| {
            IsolationKernelError::InstallFailed {
                qualified_name: qualified_name.to_owned(),
                source,
            }
        })?;
        let mut applied = self
            .applied
            .try_borrow_mut()
            .map_err(|_| IsolationKernelError::VerifyFailed)?;
        applied.insert(qualified_name.to_owned(), ddl.to_owned());
        Ok(())
    }
}

impl RlsInstaller for InMemoryRlsInstaller {
    fn install(&self, policy: &RlsPolicy) -> Result<(), IsolationKernelError> {
        let ddl = render_policy_ddl(policy)?;
        self.install_rendered(&policy.table.qualified_name(), &ddl)
    }

    fn verify(&self, policy: &RlsPolicy) -> Result<bool, IsolationKernelError> {
        let expected = render_policy_ddl(policy)?;
        let applied = self
            .applied
            .try_borrow()
            .map_err(|_| IsolationKernelError::VerifyFailed)?;
        Ok(applied.get(&policy.table.qualified_name()) == Some(&expected))
    }
}
