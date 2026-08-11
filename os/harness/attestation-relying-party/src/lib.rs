#![forbid(unsafe_code)]
//! Relying-party attestation scaffold (RATS RFC 9334 / KBS shape).
//!
//! Forever contract (Round-2): guest collector (configfs-tsm, nonce-bound
//! `report_data`) → **off-node owned verifier** (pinned AMD KDS / Intel PCS;
//! stale/outage collateral ⇒ verdict **UNKNOWN, never PASS**) → short-TTL signed
//! attestation **result** → existing SVID issuer (ADR-0561) + Cedar context keys.
//!
//! Locked shape: hardware-agnostic quote/evidence schema first; SEV-SNP/TDX/CCA
//! adapters; day-1 private-kernel-attested = attested-identity (host in TCB,
//! labeled); guest-pull/operator-excluded confidentiality is F1 Isolation
//! evidence-gated target. UNKNOWN≠PASS. Short-TTL → SVID+Cedar.
//!
//! Extends the existing ConfidentialPlatform story in
//! `cloud/cloud-kernel/.../confidential.rs` (SNP / TDX / ARM CCA) without claiming
//! live hardware quotes or implementing `#VC`/`TDCALL` bodies here.
//!
//! Scaffolds ≠ production; no Accept claims. Naming ban: no asterkube/kuberos.
//!
//! data_class: PUBLIC

use serde::{Deserialize, Serialize};

/// TEE types already modeled on `ConfidentialPlatform` (SNP → TDX → ARM CCA).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeeType {
    #[serde(rename = "sev-snp")]
    SevSnp,
    #[serde(rename = "tdx")]
    Tdx,
    #[serde(rename = "arm-cca")]
    ArmCca,
}

impl TeeType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SevSnp => "sev-snp",
            Self::Tdx => "tdx",
            Self::ArmCca => "arm-cca",
        }
    }
}

/// Closed Cedar context key names for attestation injection (scaffold inventory).
pub const CEDAR_CONTEXT_KEYS: [&str; 7] = [
    "context.attestation.verified",
    "context.attestation.tee_type",
    "context.attestation.tcb_status",
    "context.attestation.measurement_id",
    "context.attestation.policy_hash",
    "context.attestation.debug_disabled",
    "context.attestation.age_seconds",
];

/// Attestation verdict. Stale/outage MUST map to [`Self::Unknown`], never Pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttestationVerdict {
    Pass,
    Fail,
    Unknown,
}

impl AttestationVerdict {
    /// Round-2 lock: collateral problems never Pass (UNKNOWN≠PASS).
    /// Fresh collateral alone is insufficient for Pass (evidence must still verify).
    pub const fn refuse_pass_unless_fresh(status: CollateralStatus) -> Option<Self> {
        match status {
            CollateralStatus::Stale | CollateralStatus::Unavailable => Some(Self::Unknown),
            CollateralStatus::Fresh => None,
        }
    }
}

/// Endorsement / KDS / PCS collateral freshness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollateralStatus {
    Fresh,
    Stale,
    Unavailable,
}

/// Guest-side evidence envelope (collector output). Scaffold — no configfs I/O.
///
/// Carries opaque evidence bytes + challenge/nonce so a real
/// [`RelyingPartyVerifier`] can verify quote signature, measurement, and nonce
/// binding without trusting caller-supplied claim flags alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuestEvidence {
    pub tee: TeeType,
    /// Nonce / challenge material bound into report_data (scaffold may be empty).
    pub challenge: Vec<u8>,
    /// Opaque quote / evidence bytes (scaffold may be empty).
    pub evidence_bytes: Vec<u8>,
    pub hardware_quote_claimed: bool,
}

impl GuestEvidence {
    /// Scaffold collector: never claims a live hardware quote; empty evidence.
    pub fn scaffold_collector(tee: TeeType, nonce_len: usize) -> Self {
        Self {
            tee,
            challenge: vec![0u8; nonce_len.min(64)],
            evidence_bytes: Vec::new(),
            hardware_quote_claimed: false,
        }
    }

    pub fn report_data_len(&self) -> usize {
        self.challenge.len().min(64)
    }

    pub fn evidence_bytes_len(&self) -> usize {
        self.evidence_bytes.len()
    }
}

/// Verifier-authenticated claims destined for Cedar / SVID adapters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedAttestationClaims {
    pub tcb_status: String,
    pub measurement_id: String,
    pub policy_hash: String,
    pub debug_disabled: bool,
    /// Unix seconds at issuance (scaffold uses 0 = unset).
    pub issued_at_unix: u64,
}

impl VerifiedAttestationClaims {
    pub fn scaffold_unknown() -> Self {
        Self {
            tcb_status: "unknown".into(),
            measurement_id: String::new(),
            policy_hash: String::new(),
            debug_disabled: false,
            issued_at_unix: 0,
        }
    }

    pub fn age_seconds(&self, now_unix: u64) -> Option<u64> {
        if self.issued_at_unix == 0 || now_unix < self.issued_at_unix {
            None
        } else {
            Some(now_unix - self.issued_at_unix)
        }
    }
}

/// Short-TTL attestation **result** (not raw evidence).
///
/// Day-1 path: short-TTL result → SVID + Cedar. A signed transport envelope for
/// off-node results is future work (scaffolds ≠ production); do not treat this
/// struct alone as authenticatable across a trust boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationResult {
    pub verdict: AttestationVerdict,
    pub tee_type: String,
    pub ttl_seconds: u32,
    pub hardware_verified: bool,
    pub claims: VerifiedAttestationClaims,
    pub notes: String,
}

impl AttestationResult {
    /// Non-bypassable fail-closed constructor: stale/unavailable collateral
    /// cannot yield Pass regardless of caller intent.
    pub fn fail_closed(
        mut verdict: AttestationVerdict,
        tee: TeeType,
        collateral: CollateralStatus,
        ttl_seconds: u32,
        hardware_verified: bool,
        claims: VerifiedAttestationClaims,
        notes: impl Into<String>,
    ) -> Self {
        if let Some(forced) = AttestationVerdict::refuse_pass_unless_fresh(collateral) {
            verdict = forced;
        }
        if matches!(verdict, AttestationVerdict::Pass) && !hardware_verified {
            verdict = AttestationVerdict::Unknown;
        }
        Self {
            verdict,
            tee_type: tee.as_str().to_owned(),
            ttl_seconds,
            hardware_verified,
            claims,
            notes: notes.into(),
        }
    }
}

/// Typed verifier failures distinct from attestation verdicts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifierError {
    /// Quote / evidence bytes could not be parsed.
    EvidenceParse(String),
    /// Collateral transport failed (retriable).
    CollateralTransport(String),
    /// Cryptographic processing failed (invalid evidence).
    Crypto(String),
}

/// Off-node relying-party verifier port (owned). Scaffold stub only.
pub trait RelyingPartyVerifier {
    fn verify(
        &self,
        evidence: &GuestEvidence,
        collateral: CollateralStatus,
    ) -> Result<AttestationResult, VerifierError>;
}

/// Stub verifier: honest about no hardware; stale/outage ⇒ Unknown never Pass.
#[derive(Debug, Clone, Copy)]
pub struct StubRelyingPartyVerifier {
    pub result_ttl_seconds: u32,
}

impl Default for StubRelyingPartyVerifier {
    fn default() -> Self {
        Self {
            result_ttl_seconds: 300,
        }
    }
}

impl RelyingPartyVerifier for StubRelyingPartyVerifier {
    fn verify(
        &self,
        evidence: &GuestEvidence,
        collateral: CollateralStatus,
    ) -> Result<AttestationResult, VerifierError> {
        // Hard ban: scaffold must never emit Pass when hardware was not verified,
        // and never Pass on stale/unavailable collateral.
        let intended = match collateral {
            CollateralStatus::Stale | CollateralStatus::Unavailable => AttestationVerdict::Unknown,
            CollateralStatus::Fresh => AttestationVerdict::Unknown,
        };
        Ok(AttestationResult::fail_closed(
            intended,
            evidence.tee,
            collateral,
            self.result_ttl_seconds,
            false,
            VerifiedAttestationClaims::scaffold_unknown(),
            "Scaffold stub — no KDS/PCS fetch, no live SNP/TDX/CCA quote; UNKNOWN until real verifier lands.",
        ))
    }
}

/// Extension note for `ConfidentialPlatform::attestation_report` consumers.
pub fn confidential_platform_extension_note() -> &'static str {
    "Extend ConfidentialPlatform (SNP/TDX/ARM CCA) with relying-party verify off-node; \
     this harness owns the RP types only. Guest report production stays on the sealed \
     platform trait; do not fake hardware PASS here."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_collateral_is_unknown_never_pass() {
        let v = StubRelyingPartyVerifier::default();
        let evidence = GuestEvidence::scaffold_collector(TeeType::SevSnp, 64);
        let r = v.verify(&evidence, CollateralStatus::Stale).unwrap();
        assert_eq!(r.verdict, AttestationVerdict::Unknown);
        assert!(!r.hardware_verified);
    }

    #[test]
    fn outage_collateral_is_unknown_never_pass() {
        let v = StubRelyingPartyVerifier::default();
        let evidence = GuestEvidence::scaffold_collector(TeeType::Tdx, 64);
        let r = v
            .verify(&evidence, CollateralStatus::Unavailable)
            .unwrap();
        assert_eq!(r.verdict, AttestationVerdict::Unknown);
    }

    #[test]
    fn fresh_scaffold_still_unknown_without_hardware() {
        let v = StubRelyingPartyVerifier::default();
        let evidence = GuestEvidence::scaffold_collector(TeeType::ArmCca, 64);
        assert!(!evidence.hardware_quote_claimed);
        assert_eq!(evidence.evidence_bytes_len(), 0);
        let r = v.verify(&evidence, CollateralStatus::Fresh).unwrap();
        assert_eq!(r.verdict, AttestationVerdict::Unknown);
    }

    #[test]
    fn cedar_keys_are_closed_and_stable() {
        assert_eq!(CEDAR_CONTEXT_KEYS.len(), 7);
        assert!(CEDAR_CONTEXT_KEYS
            .iter()
            .all(|k| k.starts_with("context.attestation.")));
    }

    #[test]
    fn refuse_pass_helper_never_passes_on_stale() {
        assert_eq!(
            AttestationVerdict::refuse_pass_unless_fresh(CollateralStatus::Stale),
            Some(AttestationVerdict::Unknown)
        );
        assert_eq!(
            AttestationVerdict::refuse_pass_unless_fresh(CollateralStatus::Fresh),
            None
        );
    }

    #[test]
    fn fail_closed_constructor_blocks_pass_on_stale() {
        let r = AttestationResult::fail_closed(
            AttestationVerdict::Pass,
            TeeType::SevSnp,
            CollateralStatus::Stale,
            300,
            true,
            VerifiedAttestationClaims::scaffold_unknown(),
            "attempted bypass",
        );
        assert_eq!(r.verdict, AttestationVerdict::Unknown);
    }

    #[test]
    fn tee_type_serde_matches_as_str() {
        for tee in [TeeType::SevSnp, TeeType::Tdx, TeeType::ArmCca] {
            let s = serde_json::to_string(&tee).unwrap();
            assert_eq!(s, format!("\"{}\"", tee.as_str()));
            let back: TeeType = serde_json::from_str(&s).unwrap();
            assert_eq!(back, tee);
        }
    }
}
