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

use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

/// Hard ceiling for attestation-result TTL (short-TTL lock). Default stub uses 300s.
pub const MAX_RESULT_TTL_SECONDS: u32 = 600;

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

    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "sev-snp" => Some(Self::SevSnp),
            "tdx" => Some(Self::Tdx),
            "arm-cca" => Some(Self::ArmCca),
            _ => None,
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

#[derive(Debug, Deserialize)]
struct AttestationResultWire {
    verdict: AttestationVerdict,
    tee_type: String,
    ttl_seconds: u32,
    hardware_verified: bool,
    claims: VerifiedAttestationClaims,
    notes: String,
}

/// Short-TTL attestation **result** (not raw evidence).
///
/// Fields are private; construct only via [`Self::fail_closed`] (or validated
/// Deserialize). Day-1 path: short-TTL result → SVID + Cedar. A signed transport
/// envelope for off-node results is future work (scaffolds ≠ production).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttestationResult {
    verdict: AttestationVerdict,
    tee_type: String,
    ttl_seconds: u32,
    hardware_verified: bool,
    claims: VerifiedAttestationClaims,
    notes: String,
}

impl AttestationResult {
    pub fn verdict(&self) -> AttestationVerdict {
        self.verdict
    }
    pub fn tee_type(&self) -> &str {
        &self.tee_type
    }
    pub fn ttl_seconds(&self) -> u32 {
        self.ttl_seconds
    }
    pub fn hardware_verified(&self) -> bool {
        self.hardware_verified
    }
    pub fn claims(&self) -> &VerifiedAttestationClaims {
        &self.claims
    }
    pub fn notes(&self) -> &str {
        &self.notes
    }

    fn clamp_ttl(ttl_seconds: u32) -> u32 {
        if ttl_seconds == 0 {
            1
        } else {
            ttl_seconds.min(MAX_RESULT_TTL_SECONDS)
        }
    }

    /// Non-bypassable fail-closed constructor: stale/unavailable collateral
    /// cannot yield Pass; TTL is clamped to [`MAX_RESULT_TTL_SECONDS`].
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
            ttl_seconds: Self::clamp_ttl(ttl_seconds),
            hardware_verified,
            claims,
            notes: notes.into(),
        }
    }

    fn from_wire(wire: AttestationResultWire) -> Result<Self, String> {
        let tee = TeeType::try_from_str(&wire.tee_type)
            .ok_or_else(|| format!("unknown tee_type {}", wire.tee_type))?;
        // Deserialization has no collateral context — treat as Fresh for the
        // freshness rule, but still refuse Pass without hardware_verified and
        // clamp TTL. Callers transporting results across trust boundaries must
        // use the future signed envelope (deferred).
        let mut verdict = wire.verdict;
        if matches!(verdict, AttestationVerdict::Pass) && !wire.hardware_verified {
            verdict = AttestationVerdict::Unknown;
        }
        Ok(Self {
            verdict,
            tee_type: tee.as_str().to_owned(),
            ttl_seconds: Self::clamp_ttl(wire.ttl_seconds),
            hardware_verified: wire.hardware_verified,
            claims: wire.claims,
            notes: wire.notes,
        })
    }
}

impl<'de> Deserialize<'de> for AttestationResult {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = AttestationResultWire::deserialize(deserializer)?;
        Self::from_wire(wire).map_err(serde::de::Error::custom)
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

impl fmt::Display for VerifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EvidenceParse(m) | Self::CollateralTransport(m) | Self::Crypto(m) => {
                write!(f, "{m}")
            }
        }
    }
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
        assert_eq!(r.verdict(), AttestationVerdict::Unknown);
        assert!(!r.hardware_verified());
    }

    #[test]
    fn outage_collateral_is_unknown_never_pass() {
        let v = StubRelyingPartyVerifier::default();
        let evidence = GuestEvidence::scaffold_collector(TeeType::Tdx, 64);
        let r = v
            .verify(&evidence, CollateralStatus::Unavailable)
            .unwrap();
        assert_eq!(r.verdict(), AttestationVerdict::Unknown);
    }

    #[test]
    fn fresh_scaffold_still_unknown_without_hardware() {
        let v = StubRelyingPartyVerifier::default();
        let evidence = GuestEvidence::scaffold_collector(TeeType::ArmCca, 64);
        assert!(!evidence.hardware_quote_claimed);
        assert_eq!(evidence.evidence_bytes_len(), 0);
        let r = v.verify(&evidence, CollateralStatus::Fresh).unwrap();
        assert_eq!(r.verdict(), AttestationVerdict::Unknown);
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
        assert_eq!(r.verdict(), AttestationVerdict::Unknown);
    }

    #[test]
    fn ttl_is_clamped_to_max() {
        let r = AttestationResult::fail_closed(
            AttestationVerdict::Unknown,
            TeeType::Tdx,
            CollateralStatus::Fresh,
            u32::MAX,
            false,
            VerifiedAttestationClaims::scaffold_unknown(),
            "ttl clamp",
        );
        assert_eq!(r.ttl_seconds(), MAX_RESULT_TTL_SECONDS);
    }

    #[test]
    fn deserialize_refuses_pass_without_hardware() {
        let json = r#"{
            "verdict":"Pass",
            "tee_type":"sev-snp",
            "ttl_seconds":999999,
            "hardware_verified":false,
            "claims":{"tcb_status":"x","measurement_id":"","policy_hash":"","debug_disabled":false,"issued_at_unix":0},
            "notes":"bypass"
        }"#;
        let r: AttestationResult = serde_json::from_str(json).unwrap();
        assert_eq!(r.verdict(), AttestationVerdict::Unknown);
        assert_eq!(r.ttl_seconds(), MAX_RESULT_TTL_SECONDS);
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
