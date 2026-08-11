#![forbid(unsafe_code)]
//! Relying-party attestation scaffold (RATS RFC 9334 / KBS shape).
//!
//! Forever contract (Round-2): guest collector (configfs-tsm, nonce-bound
//! `report_data`) → **off-node owned verifier** (pinned AMD KDS / Intel PCS;
//! stale/outage collateral ⇒ verdict **UNKNOWN, never PASS**) → short-TTL signed
//! attestation **result** → existing SVID issuer (ADR-0561) + Cedar context keys.
//!
//! Extends the existing [`ConfidentialPlatform`](https://docs.oyatie.com) story in
//! `cloud/cloud-kernel/.../confidential.rs` (SNP / TDX / ARM CCA) without claiming
//! live hardware quotes or implementing `#VC`/`TDCALL` bodies here.
//!
//! data_class: PUBLIC

use serde::Serialize;

/// TEE types already modeled on `ConfidentialPlatform` (SNP → TDX → ARM CCA).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TeeType {
    SevSnp,
    Tdx,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AttestationVerdict {
    Pass,
    Fail,
    Unknown,
}

impl AttestationVerdict {
    /// Law mirror of ADR-0710 D-2 / Round-2: collateral problems never Pass.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuestEvidence {
    pub tee: TeeType,
    /// Nonce-bound report_data (64 bytes conceptually; scaffold holds hex/length check only).
    pub report_data_len: usize,
    pub evidence_bytes_len: usize,
    pub hardware_quote_claimed: bool,
}

impl GuestEvidence {
    /// Scaffold collector: never claims a live hardware quote.
    pub fn scaffold_collector(tee: TeeType, nonce_len: usize) -> Self {
        Self {
            tee,
            report_data_len: nonce_len.min(64),
            evidence_bytes_len: 0,
            hardware_quote_claimed: false,
        }
    }
}

/// Short-TTL signed attestation **result** (not raw evidence).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttestationResult {
    pub verdict: AttestationVerdict,
    pub tee_type: &'static str,
    pub ttl_seconds: u32,
    pub hardware_verified: bool,
    pub notes: &'static str,
}

/// Off-node relying-party verifier port (owned). Scaffold stub only.
pub trait RelyingPartyVerifier {
    fn verify(
        &self,
        evidence: &GuestEvidence,
        collateral: CollateralStatus,
    ) -> AttestationResult;
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
    ) -> AttestationResult {
        // Hard ban: scaffold must never emit Pass when hardware was not verified,
        // and never Pass on stale/unavailable collateral.
        let verdict = match collateral {
            CollateralStatus::Stale | CollateralStatus::Unavailable => AttestationVerdict::Unknown,
            CollateralStatus::Fresh if evidence.hardware_quote_claimed => {
                // Even if a future path claims a quote, this stub still refuses Pass —
                // no fake hardware success.
                AttestationVerdict::Unknown
            }
            CollateralStatus::Fresh => AttestationVerdict::Unknown,
        };
        debug_assert!(
            !(matches!(verdict, AttestationVerdict::Pass)
                && (evidence.hardware_quote_claimed == false
                    || matches!(
                        collateral,
                        CollateralStatus::Stale | CollateralStatus::Unavailable
                    ))),
            "Pass forbidden without verified hardware and fresh collateral"
        );
        AttestationResult {
            verdict,
            tee_type: evidence.tee.as_str(),
            ttl_seconds: self.result_ttl_seconds,
            hardware_verified: false,
            notes: "Scaffold stub — no KDS/PCS fetch, no live SNP/TDX/CCA quote; UNKNOWN until real verifier lands.",
        }
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
        let r = v.verify(&evidence, CollateralStatus::Stale);
        assert_eq!(r.verdict, AttestationVerdict::Unknown);
        assert!(!r.hardware_verified);
    }

    #[test]
    fn outage_collateral_is_unknown_never_pass() {
        let v = StubRelyingPartyVerifier::default();
        let evidence = GuestEvidence::scaffold_collector(TeeType::Tdx, 64);
        let r = v.verify(&evidence, CollateralStatus::Unavailable);
        assert_eq!(r.verdict, AttestationVerdict::Unknown);
    }

    #[test]
    fn fresh_scaffold_still_unknown_without_hardware() {
        let v = StubRelyingPartyVerifier::default();
        let evidence = GuestEvidence::scaffold_collector(TeeType::ArmCca, 64);
        assert!(!evidence.hardware_quote_claimed);
        let r = v.verify(&evidence, CollateralStatus::Fresh);
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
}
