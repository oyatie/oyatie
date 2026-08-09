---
id: ADR-SDK-0006
title: "Developer KYC runs in-house without external KYC SaaS"
status: Proposed
date: 2026-05-18
microservice: developer-sdk
related_oyatie_adrs:
  - ADR-0131
  - ADR-0173
  - ADR-0213
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
decision_owner: axis-ecosystem + council-compliance + ops-security
---

# ADR-SDK-0006: Developer KYC runs in-house without external KYC SaaS

## Context

- The named pressure is `developer-payout-risk-without-identity-vendor-capture`.
- Developer-sdk must verify developers before marketplace payouts, API monetization, and high-trust plugin publication.
- Prior incident class `kyc_vendor_defaulted` assumed Onfido, Persona, or Stripe Identity without a build-vs-buy decision.
- Prior incident class `kyc_decision_without_evidence` stored a boolean verification flag without feature evidence or review trail.
- Prior incident class `kyc_region_leak` allowed identity images to move outside their compliance pack.
- ADR-0173 rejects avoidable vendor lock-in for strategic control planes.
- ADR-0213 makes developer onboarding part of the Ecosystem-as-a-Service substrate.
- ADR-0243 requires KYC state transitions to be Cedar-gated.
- ADR-0244 requires developer account KYC records to be tenant and principal scoped.
- ADR-0251 requires compliance pack overlays for regional identity obligations.
- ADR-0263 requires KYC decisions to produce audit events, traces, metrics, and dashboards.
- Developer KYC touches high-risk PII: government id images, face templates, liveness video, address, tax id, sanctions hits, and beneficial-owner records.
- GDPR Art. 9 biometric data rules apply where face template or liveness evidence is biometric.
- GDPR Art. 22 automated decision-making constraints apply when automated rejection has significant effect.
- KR PIPA Art. 23 and Art. 29 require elevated controls for sensitive information.
- CCPA/CPRA access and deletion obligations apply to California developer accounts.
- Sanctions screening must support OFAC, EU consolidated sanctions, UK HMT, UN lists, and KR FSC lists.
- False positives must route to human compliance review.
- False negatives can lead to payout fraud, sanctions violations, and marketplace trust loss.
- The system must support progressive verification: low-risk read-only developer accounts need less evidence than payout-enabled publishers.
- The system must keep raw identity documents out of ordinary application logs, traces, and analytics.
- The system must support vendor-free operation in self-hosted and sovereign deployments.

## Decision

- We choose `in-house KYC pipeline with modular verification adapters`.
- The named pattern is `risk-tiered identity verification with human-review escalation`, inspired by Stripe Identity and Persona but owned by oyatie.
- External KYC SaaS products are not canonical dependencies.
- Onfido is rejected as canonical.
- Persona is rejected as canonical.
- Stripe Identity is rejected as canonical.
- The pipeline uses in-house document intake, OCR, liveness, sanctions screening, and compliance review state machines.
- OCR uses self-hosted Tesseract 5.x plus a future ML document parser behind foundry-runtime.
- Liveness uses in-house challenge-response video checks and device attestation.
- Face-template extraction is optional and pack-gated.
- Sanctions screening uses licensed sanctions data files loaded into a pack-local screening store.
- Address verification uses document evidence and bank payout account verification, not consumer credit bureau by default.
- Developer risk tiers are `read_only`, `sandbox_publisher`, `payout_enabled`, `high_volume_payout`, and `regulated_connector`.
- `read_only` requires email, passkey, and abuse reputation only.
- `sandbox_publisher` requires email, passkey, business profile, and developer agreement.
- `payout_enabled` requires government id, tax profile, sanctions clear, and payout account verification.
- `high_volume_payout` requires enhanced due diligence and human review.
- `regulated_connector` requires business registration, beneficial-owner record, and pack-specific attestations.
- Automated approval is allowed only for low-risk matches with confidence >= 0.995 and no sanctions candidate.
- Automated denial is forbidden for payout-enabled or above; the system may mark `needs_review`.
- Human review SLA is p95 <= 2 business days.
- Developer-submitted document image retention is 30 days after approval unless pack requires longer.
- Derived verification evidence retention is 7 years.
- Liveness video retention is 30 days after approval unless dispute or fraud hold applies.
- Sanctions screening refresh cadence is daily.
- KYC re-check cadence is 12 months for payout-enabled and 3 months for high-volume payout.
- KYC state transition requires Cedar action `developer-sdk.kyc.transition`.
- KYC evidence read requires Cedar action `developer-sdk.kyc.evidence.read`.
- KYC review assignment requires Cedar action `developer-sdk.kyc.review.assign`.
- Public developer portal exposes status and remediation, never raw reviewer notes.
- The p95 intake API target is 300 ms excluding upload transfer.
- The p95 automated screening target is 5 minutes after complete evidence submission.

## Alternatives Considered

### Onfido hosted KYC

- Pro: mature document verification.
- Pro: liveness checks are available.
- Pro: international coverage is broad.
- Con: strategic identity evidence leaves oyatie custody.
- Con: pack residency is vendor-dependent.
- Con: pricing scales per check.
- Con: self-hosted cells cannot rely on it.
- Tradeoff: speed but vendor capture.
- Rejected as canonical.

### Persona hosted KYC

- Pro: strong workflow product.
- Pro: flexible verification templates.
- Pro: good developer experience.
- Con: identity workflow becomes vendor-defined.
- Con: Cedar and audit-chain integration are secondary.
- Con: regional evidence residency depends on vendor availability.
- Con: external UI can drift from oyatie trust model.
- Tradeoff: UX speed but platform-control loss.
- Rejected as canonical.

### Stripe Identity

- Pro: close pairing with payouts if Stripe were used.
- Pro: established fraud tooling.
- Pro: simple integration.
- Con: payout ADR rejects Stripe as canonical.
- Con: KYC evidence remains in Stripe's domain.
- Con: self-hosting and sovereign cells lose equivalence.
- Tradeoff: convenient if hosted payments are accepted, but this platform does not choose that path.
- Rejected.

### No KYC until payout threshold

- Pro: less onboarding friction.
- Pro: lower initial review cost.
- Pro: faster developer activation.
- Con: malicious publishers can build reputation before verification.
- Con: sanctions issues can appear late.
- Con: marketplace trust suffers.
- Tradeoff: growth speed but risk accumulation.
- Rejected; progressive verification is selected instead.

## Consequences

- Positive: identity evidence remains under oyatie custody and pack residency.
- Positive: KYC state transitions become Cedar-governed and auditable.
- Positive: progressive verification minimizes friction for low-risk developers.
- Positive: in-house pipeline works in self-hosted and sovereign cells.
- Positive: human review prevents hard automated denial for high-impact cases.
- Negative: engineering and compliance workload is higher than hosted KYC.
- Negative: sanctions data licensing remains an operational dependency.
- Negative: OCR and liveness quality require continuous evaluation.
- Negative: compliance reviewers need tooling and training.
- Neutral: vendors may still supply raw sanctions or registry data under data-feed contracts.
- Neutral: future ML document parser remains behind foundry-runtime and review gates.
- Follow-up work: implement `SDK-IP-009-kyc-evidence-kernel`.
- Follow-up work: implement `SDK-IP-010-sanctions-screening-adapter`.
- Follow-up work: implement `SDK-IP-011-human-review-console`.
- Follow-up work: add KYC DPIA overlay and reviewer runbook.

## Implementation Notes

- Data shape `DeveloperKycCaseV1` is the state machine root.
- Field `kyc_case_id` is ULID prefixed by `kyc_`.
- Field `developer_account_id` references developer-sdk account.
- Field `tenant_id` scopes marketplace or platform tenant.
- Field `risk_tier` is one of the five tiers named above.
- Field `state` is `not_started`, `evidence_requested`, `submitted`, `auto_screening`, `needs_review`, `approved`, `rejected`, `expired`, or `suspended`.
- Field `review_reason` is nullable and versioned.
- Field `sanctions_status` is `not_checked`, `clear`, `candidate_match`, `confirmed_match`, or `false_positive`.
- Field `document_evidence_refs` points to encrypted object storage.
- Field `liveness_evidence_ref` points to encrypted object storage.
- Field `tax_profile_ref` links to developer tax profile.
- Field `payout_account_ref` links to payout account verification.
- Field `beneficial_owner_refs` is required for regulated connector and high-volume payout business accounts.
- Data shape `KycEvidenceDigestV1` stores only hashes and classification outputs for long retention.
- Field `evidence_digest` is SHA-256 over encrypted evidence bytes.
- Field `model_or_rule_version` records OCR/liveness/sanctions rule version.
- Field `confidence_score` is decimal string with 4 places.
- API endpoint `GET /v1/developer/kyc/status` returns developer-visible status.
- API endpoint `POST /v1/developer/kyc/evidence` uploads encrypted evidence.
- API endpoint `POST /v1/developer/kyc/submit` submits the case.
- API endpoint `GET /v1/internal/kyc/cases` lists reviewer cases.
- API endpoint `POST /v1/internal/kyc/cases/{case_id}/transition` changes state.
- API endpoint `POST /v1/internal/kyc/cases/{case_id}/assign` assigns reviewer.
- Cedar principal for developer is `DeveloperSdk::DeveloperAccount`.
- Cedar principal for reviewer is `Oyatie::Principal::Human("compliance-reviewer")`.
- Cedar principal for worker is `Oyatie::Principal::Service("developer-sdk.kyc-worker")`.
- Cedar action `developer-sdk.kyc.evidence.upload` applies to `DeveloperSdk::KycCase`.
- Cedar action `developer-sdk.kyc.transition` applies to `DeveloperSdk::KycCase`.
- Cedar action `developer-sdk.kyc.evidence.read` applies to `DeveloperSdk::KycEvidence`.
- Cedar action `developer-sdk.kyc.review.assign` applies to `DeveloperSdk::KycCase`.
- Cedar context field `risk_tier` controls required evidence.
- Cedar context field `pack_id` controls regional constraints.
- Cedar context field `reviewer_training_current` must be true for human review.
- Cedar context field `automated_denial` must be false for payout-enabled and above.
- Example permit: principal `developer-sdk.kyc-worker`, action `developer-sdk.kyc.transition`, resource `DeveloperSdk::KycCase::"kyc_01HY"`, context `{from:"auto_screening", to:"approved", confidence_score:"0.9970", sanctions_status:"clear", risk_tier:"payout_enabled"}`.
- Example forbid: same principal and action with context `{to:"rejected", automated_denial:true, risk_tier:"payout_enabled"}`.
- Raw evidence object path is `kyc/{pack_id}/{tenant_id}/{developer_account_id}/{kyc_case_id}/raw/`.
- Evidence encryption uses per-pack KMS keys.
- Reviewer UI masks document number except last four characters.
- Logs must not include raw document OCR text.
- Traces must not include image filenames supplied by the user.
- Audit event `DeveloperKycEvidenceSubmitted` emits on upload.
- Audit event `DeveloperKycScreeningCompleted` emits on automated screening.
- Audit event `DeveloperKycReviewDecisionRecorded` emits on human decision.
- Audit event `DeveloperKycExpired` emits on re-check expiry.
- OpenTelemetry span `developer_sdk.kyc.screen` wraps screening.
- Metric `oya_developer_sdk_kyc_case_state_total` counts cases by state and risk tier.
- Metric `oya_developer_sdk_kyc_screening_duration_seconds` tracks automated screening duration.
- Metric `oya_developer_sdk_kyc_review_age_seconds` tracks human review age.
- Metric `oya_developer_sdk_kyc_sanctions_candidate_total` tracks candidate matches.
- Dashboard `developer-sdk-kyc-operations.json` shows state, review queue, screening duration, sanctions candidates, and pack split.
- SLO `developer-sdk-kyc-screening.openslo.yaml` sets p95 <= 5 minutes.
- SLO `developer-sdk-kyc-human-review.openslo.yaml` sets p95 <= 2 business days.
- Failure mode `sanctions_feed_stale` blocks approvals until feed freshness returns under 24 hours.
- Failure mode `ocr_low_confidence` routes to human review.
- Failure mode `liveness_replay_detected` routes to fraud review and suspends payout release.
- Failure mode `evidence_region_mismatch` refuses upload and emits Sev-2 security alert.
- Failure mode `reviewer_policy_denied` prevents state change.

## Verification

- Test `kyc_payout_enabled_requires_government_id` verifies evidence requirements.
- Test `kyc_sanctions_candidate_blocks_approval` verifies candidate match routes to review.
- Test `kyc_automated_denial_forbidden_for_payout_enabled` verifies Cedar refusal.
- Test `kyc_raw_evidence_pack_residency` verifies object paths stay within pack.
- Test `kyc_logs_do_not_contain_ocr_text` scans logs.
- Test `kyc_recheck_cadence_high_volume` verifies 3-month expiry.
- Test `kyc_recheck_cadence_payout_enabled` verifies 12-month expiry.
- Test `kyc_reviewer_training_required` verifies reviewer eligibility context.
- Test `kyc_sanctions_feed_freshness_required` verifies stale feed blocks approvals.
- Test `kyc_state_machine_no_skip_review` verifies illegal transitions fail.
- Metric `oya_developer_sdk_kyc_screening_duration_seconds` must meet p95 <= 5 minutes.
- Metric `oya_developer_sdk_kyc_review_age_seconds` must meet p95 <= 2 business days.
- Metric `oya_developer_sdk_kyc_sanctions_candidate_total` must page compliance on confirmed matches.
- Dashboard `developer-sdk-kyc-operations.json` must include pack residency and review queue panels.
- Dashboard `marketplace-risk-overview.json` must include KYC state by tenant_class.
- CI check `kyc-no-external-saas-client` rejects Onfido, Persona, and Stripe Identity SDK dependencies.
- CI check `kyc-cedar-transition-coverage` verifies state transitions map to Cedar.
- CI check `kyc-evidence-data-class` verifies sensitive data annotations.
- CI check `kyc-log-scrub` verifies logs and traces exclude raw PII.
- CI check `oya-governance-observability-emission --microservice developer-sdk` verifies ADR-0263 telemetry.
- Load test screens 10,000 low-risk cases and requires p95 <= 5 minutes.
- Chaos test marks sanctions feed stale and verifies approval blocks.
- Privacy test exports a DSR bundle and verifies raw liveness video respects retention.
- Security test attempts cross-pack evidence read and expects 403.
- Audit query verifies every approval has screening and decision evidence events.

## References

- ADR-0131: Per-microservice flat layout.
- ADR-0173: Vendor lock-in avoidance and stack ownership.
- ADR-0213: Ecosystem-as-a-Service architecture.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0251: Compliance pack cell certification levels.
- ADR-0263: Observability emission contract.
- FATF Recommendation 10 customer due diligence.
- FATF Recommendation 16 wire transfers.
- OFAC Sanctions Compliance Guidance.
- EU Consolidated Financial Sanctions List guidance.
- UK HMT financial sanctions guidance.
- GDPR Arts. 9, 22, 25, and 32.
- KR PIPA Arts. 23, 28, and 29.
- NIST SP 800-63A identity proofing guidance.
- ISO/IEC 27001:2022 A.5.34 privacy and PII protection.
- Tesseract OCR 5.x documentation.
- OpenBao and per-pack KMS documentation.
