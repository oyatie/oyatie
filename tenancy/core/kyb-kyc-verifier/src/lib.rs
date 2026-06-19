//! KYB / KYC verifier domain — pure logic for verification case, document
//! requirements, screening result aggregation, decision state, expiry, and
//! escalation. Provider calls are out of scope here.
//!
//! Wave 15-IMPL-truth-up scaffold; full implementation lands in IP-018 execution.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(dead_code)]

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VerificationCaseId(pub String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationKind {
    Kyb,
    Kyc,
    Ubo,
    Sanctions,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VerificationDecision {
    Pending,
    Approved,
    Rejected,
    EscalatedToHuman,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentRequirement {
    pub name: String,
    pub mandatory: bool,
    pub jurisdiction: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreeningResult {
    pub provider: String,
    pub hit: bool,
    pub details: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationCase {
    pub id: VerificationCaseId,
    pub kind: VerificationKind,
    pub decision: VerificationDecision,
    pub requirements: Vec<DocumentRequirement>,
    pub screenings: Vec<ScreeningResult>,
}

/// Decide whether activation may proceed. Stub returns Pending until IP-018 lands.
pub fn decide(case: &VerificationCase) -> VerificationDecision {
    case.decision
}
