---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-008-skill-assessments-and-profile-verification-bcs
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network + council-privacy
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-port-location]
---

# IP-008: skill-assessments + profile-verification BCs end-to-end

## Intent

Land both BCs together (privacy-adjacent):

- `skill-assessments`: per-skill quiz administration + scoring + passing-badge issuance; per-skill item-bank; anti-cheat (per-question time bound + per-IP rate cap).
- `profile-verification`: badge issuance (blue / organisation / government / employer-confirmed); ID-attest flow (third-party identity verification vendor); employer-confirm flow (employer's tenant-admin attests employment); revocation.

## Code Shape

```rust
// skill-assessments kernel/src/ports.rs
#[async_trait]
pub trait SkillAssessmentRepository: Send + Sync {
    async fn list_items(&self, skill_ref: &SkillRef, locale: &Locale) -> Result<Vec<QuizItem>, AssessmentError>;
    async fn submit_attempt(&self, attempt: Attempt) -> Result<Score, AssessmentError>;
    async fn issue_badge(&self, user: &UserRef, skill_ref: &SkillRef, score: &Score) -> Result<PassingBadge, AssessmentError>;
}

// profile-verification kernel/src/ports.rs
#[async_trait]
pub trait VerificationRequestRepository: Send + Sync {
    async fn open(&self, req: VerificationRequestNew) -> Result<VerificationRequest, VerifyError>;
    async fn id_attest(&self, req_id: &VerificationRequestId, attestation: IdAttestation) -> Result<VerificationBadge, VerifyError>;
    async fn employer_confirm(&self, req_id: &VerificationRequestId, employer_principal: &TenantAdminRef) -> Result<VerificationBadge, VerifyError>;
    async fn revoke(&self, badge_id: &BadgeId, reason: &str) -> Result<RevocationEvent, VerifyError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-network-skill-assessments-kernel
cargo nextest run -p oya-network-profile-verification-kernel
cargo run -p oya-dev-cli -- gate validate port-location --microservice network
```

## Test Plan

- Skill assessment scoring: pass/fail based on item-correctness threshold; passing-badge issuance idempotent.
- Anti-cheat: rapid-submission rate-limited at REST layer + per-attempt time-bound.
- ID-attest: third-party vendor receipt verified; audit-chain sealed.
- Employer-confirm: tenant-admin must be the *employer's* tenant-admin (Cedar entitlement check); badge revoked when employment ends per `network_profiles.experience.end != null`.

## Halt Conditions

- ID-attest vendor outage — degrade gracefully; queue attests for retry.

## Next IP

[`IP-009-pages-groups-events-bcs.md`](IP-009-pages-groups-events-bcs.md)

## References

- ADR-NET-0001 (storage); Bominal ADR-0028 (audit-chain).
- Industry references: LinkedIn skill-assessments + verification programs.
