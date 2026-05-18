---
id: ADR-SHORTS-0006
status: Accepted
date: 2026-05-17
microservice: shorts
deciders: council-privacy, council-architecture, axis-shorts, ops-legal, ops-security
owner: axis-shorts + council-privacy
supersedes: []
superseded_by: []
related:
  - ADR-0008
  - ADR-0135
  - ADR-0131
  - ADR-SHORTS-0003
  - ADR-SHORTS-0005
related_artifacts:
  - microservices/shorts/PRD.md (FR-18, FR-19; Tenant Outcome 3)
  - microservices/shorts/threat-model.md (T-I-09, T-L-11)
  - microservices/shorts/dpia.md (R-07)
  - microservices/shorts/runbooks/age-gate-bypass-incident.md
  - microservices/shorts/policy/tenant-scope.cedar
  - microservices/shorts/capabilities/T1-assist.yaml
  - microservices/shorts/capabilities/T2-auto.yaml
purpose: Establish pack-aware minor-protection floor + age-gate + parental-controls posture for shorts.
---

# ADR-SHORTS-0006: Minor protection + age-gate — pack-aware thresholds; default-deny posture; parental-controls as first-class BC

## Status

Accepted — 2026-05-17.

## Context

Short-form video platforms have been subject to repeated and substantial regulatory actions for failing to protect minors:
- TikTok (2019): $5.7M FTC settlement for COPPA violations; subsequent multi-jurisdiction actions.
- Instagram Reels / Meta (multiple): ongoing CA / UT / EU enforcement.
- YouTube Kids: $170M FTC settlement for COPPA violations.
- Snapchat: AU eSafety Commissioner BOSE non-compliance findings.

Per PRD Tenant Outcome 3, FR-18, FR-19: minor-protection at the regulatory floor is a hero-product differentiator.

The regulatory landscape (per-pack):
- **GDPR Art. 8 (EU)**: digital consent minimum 16y; member states may lower to 13y (FR / IE / UK pre-Brexit).
- **COPPA 15 USC §6501 (US)**: under-13 special protection; verifiable parental consent.
- **CA AB-2273 (California Age-Appropriate Design Code Act)**: minor-protection by design.
- **UT Social Media Regulation Act (Utah)**: parental consent + access-time restrictions for minors.
- **KR PIPA Art. 8**: under-14 parental consent required; KR 청소년 보호법 (Juvenile Protection Act).
- **KR Telecommunications Business Act**: video-sharing-platform obligations.
- **UK Online Safety Act 2023**: Ofcom illegal-content + safety-by-design.
- **AU Online Safety Act 2021 + BOSE 2022**: AU eSafety Commissioner.
- **LGPD Art. 14 (Brazil)**: under-12 special protection.
- **DPDPA 2023 §9 (India)**: minor protection.
- **EU DSA Art. 28**: minor-protection — chronological-only default; algorithmic-recommendation-opt-out; restricted DMs.
- **EU AVMSD 2018/1808 Art. 28b(2)**: video-sharing-platform minor-protection — restricted advertising to minors + age-gating before mature/adult content surfaces.

Per `threat-model.md` T-I-09 (minor-account discovery) and T-L-11 (compliance with GDPR Art. 8 + COPPA + CA AB-2273 + UT SMRA), age-attestation + parental-link tables are `SENSITIVE_CHILD_PROTECTION` data class.

Per ADR-SHORTS-0005 minor-account ranking default + ADR-SHORTS-0003 minor-protection classifier mandatory + Cedar `policy/tenant-scope.cedar` FORBID minor-adult-content.

## Decision

oyatie shorts adopts a **default-deny minor-protection posture**:

### 1. Pack-aware age thresholds

| Pack | Minor threshold | Regulatory source |
|---|---|---|
| pack-kr | < 14 years | KR PIPA Art. 8 + KR 청소년 보호법 |
| pack-eu | < 16 years (member-state-adjustable to 13y) | GDPR Art. 8 |
| pack-us | < 13 years | US COPPA 15 USC §6501 |
| pack-us (CA) | < 18 years (Age-Appropriate Design Code) | CA AB-2273 |
| pack-us (UT) | < 18 years (parental consent for minors) | UT Social Media Regulation Act |
| pack-uk | < 18 years (priority detection of harmful content for minors) | UK Online Safety Act 2023 |
| pack-au | < 16 years | AU Online Safety Act 2021 + BOSE 2022 |
| pack-br | < 12 years | LGPD Art. 14 |
| pack-in | < 18 years (parental consent for minors) | DPDPA 2023 §9 |
| pack-jp | n/a (no statutory minor-data threshold; APPI general protection) | – |
| pack-sg | n/a (PDPA general protection) | – |
| pack-ae | n/a (UAE PDPL general protection) | – |
| pack-ksa | n/a (PDPL general protection) | – |

### 2. Age-gate at signup (mandatory)

- Every user attests age at signup via `age-gate` BC (REST POST /age-attestations).
- Age-attestation stored in separated `shorts_age_attestations` table (`SENSITIVE_CHILD_PROTECTION` data class).
- Pack-threshold compared at attestation → `is_minor` flag derived.
- If minor: parental-consent required before account activation; routed to `parental-controls` BC.

### 3. Parental-controls (first-class BC; PRD FR-19)

- `parental-controls` BC: linked-account parental supervision (per-pack consent verification method).
- Consent methods (pack-aware):
  - pack-us (COPPA): credit-card verification, signed-form upload, driver's license, government ID, phone verification (FTC-approved methods).
  - pack-eu: parent's verifiable consent (member-state-defined).
  - pack-kr (PIPA Art. 8): parental written consent + KRW10 small-fee credit-card or PASS authentication.
  - pack-au: AU OSA-equivalent verifiable consent.
- Linked-account: parent's account ↔ minor's account; parent visibility into minor's activity (per OSA's "BOSE 2022" expectations).

### 4. Minor-account defaults (applied via Cedar + `feed-timeline` + `messenger-bridge` + `like-share-comment`)

- **Chronological-only feed** by default. Algorithmic-recommendation requires parental-consent attestation. Per EU DSA Art. 28 + KR 청소년 보호법 + CA AB-2273 + AVMSD Art. 28b.
- **DM-restricted** (share-to-DM blocked). Per OSA + KR 청소년 보호법 + COPPA.
- **Adult / mature content** restricted to `general_audience` classification only. Per AVMSD Art. 28b(2).
- **No behavioural profile**. Watch-time stored but not used for behavioural ranking. Per EU DSA Art. 28 + CA AB-2273.
- **No commercial advertising** (when monetization-stub activated; future M05-onward). Per AVMSD Art. 28b(4) + COPPA.
- **No federation** (Personal-tier always; per DCI-08). Per data-residency invariant.
- **Screen-time supervision** (parental-control configurable; daily cap optional).

### 5. Compile-time + runtime + Cedar enforcement

- `policy/tenant-scope.cedar` PERMIT 8 + FORBID minor-content + FORBID minor-algorithmic-without-consent + FORBID minor-DM.
- Compile-time: `MinorEndUser` is a distinct principal type; cross-type writes refused.
- Runtime: per-request Cedar evaluation; minor-protection-bypass-attempt metric emitted at > 0 (Sev-1 trigger per `runbooks/age-gate-bypass-incident.md`).

### 6. Age-attestation table isolation

- Separated Postgres tables `shorts_age_attestations` + `shorts_parental_links`.
- Cedar entitlements: `age_verification_reader` + `parental_link_reader` (rare; only minor-protection compliance flows).
- LEAN lane `oya-check-age-attestation-isolation` validates.
- Per `threat-model.md` T-I-09 mitigation: separate tables prevent enumeration pivot.

### 7. Backfill on pack-threshold change

- If a pack updates its statutory minor threshold mid-flight, existing attestations re-evaluated; affected accounts notified + reverted to minor-account defaults until parental consent if newly-minor.

## Alternatives Considered

### A. Single global minor threshold (13y per COPPA US baseline; ignore per-pack variation)

- Pros: simpler operations; one age-gate logic.
- Cons: fails KR 청소년 보호법 (14y), EU GDPR Art. 8 (16y member-state-adjustable), AU OSA, CA AB-2273 (18y), UT SMRA (18y); regulator-actionable for under-protection in EU/UK/AU/KR.
- Rejected: regulatorily incorrect.

### B. Age-gate optional; tenant-by-tenant opt-in

- Pros: simpler product surface.
- Cons: COPPA + GDPR Art. 8 + KR PIPA Art. 8 all establish age-gate as a default-on legal floor; tenant cannot waive end-user statutory protection.
- Rejected: regulatorily incorrect.

### C. Use credit-card-as-age-proxy only (US-style)

- Pros: simple verification; established COPPA-approved method.
- Cons: not applicable globally; KR / EU / AU require additional / alternative methods; financial-exclusion concerns.
- Rejected: insufficient global coverage.

### D. Allow tenants to override minor-protection defaults (e.g., let minor accounts opt-in to algorithmic)

- Pros: flexibility.
- Cons: EU DSA Art. 28 + KR 청소년 보호법 + CA AB-2273 + AVMSD Art. 28b establish minor-protection as a regulatory floor; tenant cannot waive.
- Rejected: regulatorily incorrect.

### E. Centralize parental-controls in cross-µservice "minor-protection" µservice

- Pros: single source of truth across products (shorts + social + messenger).
- Cons: large architectural lift; out-of-scope for M03; tight per-product policy requirements differ.
- Rejected (for M03); kept open for cross-product harmonisation in M05-onward.

### F. Skip parental-controls BC; rely on tenant-of-tenant flow

- Pros: simpler product surface.
- Cons: COPPA requires "verifiable parental consent" — must be acquired by oyatie + tenant jointly; tenant alone insufficient; CA AB-2273 + UT SMRA same.
- Rejected: regulatorily incorrect.

## Consequences

### Positive

- Hyperscaler-grade minor-protection from M03 launch.
- Tenant-of-tenant trust elevated by transparent minor-protection posture.
- Industry-leading: competitive parity differentiator vs TikTok / Reels / Shorts (all of which have been fined for under-protection).
- Audit-chain Ed25519 seal per age-attestation + parental-link event.
- Default-deny posture: tighter regulator alignment.
- Per-pack overlay: jurisdiction-specific compliance.

### Negative

- 11-pack-overlay maintenance overhead; per-pack regulatory updates required.
- Parental-consent verification methods vary widely per pack (CC + PASS + government-ID + signed-form); UX complexity.
- Parental-controls BC adds operational complexity; per-minor settings + screen-time cap monitoring.
- KYC tightening per pack: friction during onboarding for legitimate users.

### Operational

- `runbooks/age-gate-bypass-incident.md` covers Sev-1 bypass attempts.
- Per-pack threshold tracked in `iac/kustomize/overlays/pack-*/kustomization.yaml`.
- CI lane `oya-governance-pack-aware-age-gate` validates threshold per pack.
- Quarterly pack-threshold regulatory-update review (council-privacy + ops-legal).
- Quarterly bypass-tabletop drill.

### Regulatory

- **GDPR Art. 8**: child consent compliant per-member-state.
- **KR PIPA Art. 8 + KR 청소년 보호법**: < 14y parental consent + minor-protection routing.
- **COPPA 15 USC §6501**: < 13y verifiable parental consent; FTC-approved methods.
- **CA AB-2273**: minor-protection by design.
- **UT Social Media Regulation Act**: parental consent for minors.
- **UK Online Safety Act 2023**: safety-by-design for minors.
- **AU Online Safety Act 2021 + BOSE 2022**: minor-protection per Commissioner expectations.
- **LGPD Art. 14**: < 12y special protection.
- **DPDPA 2023 §9**: minor protection.
- **EU DSA Art. 28**: minor protection — chronological-only + algorithmic-opt-out + DM-restricted.
- **EU AVMSD Art. 28b(2)**: minor-protection regulatory floor.

## References

- ADR-0008 Data Use Boundary (paired; sensitive data class enforcement).
- Parallel ADR-0135 dual-context (paired; minor protection cross-cuts context).
- ADR-0131 per-µservice flat layout.
- ADR-SHORTS-0003 (content-moderation classifier; minor-protection verdicts mandatory + irreversible without human review per AVMSD Art. 28b(2)).
- ADR-SHORTS-0005 (feed ranking; minor-account chronological-only default).
- GDPR Art. 8 + 25.
- KR PIPA Arts. 8, 22-2; KR 청소년 보호법; KR Telecommunications Business Act.
- COPPA 15 USC §6501.
- CA AB-2273 (CA Age-Appropriate Design Code Act).
- UT Social Media Regulation Act.
- UK Online Safety Act 2023.
- AU Online Safety Act 2021 + BOSE 2022.
- LGPD Art. 14.
- DPDPA 2023 §9.
- EU DSA Regulation 2065/2022 Art. 28.
- EU AVMSD 2018/1808 Art. 28b.
- `microservices/shorts/threat-model.md` T-I-09, T-L-11.
- `microservices/shorts/dpia.md` R-07.
- `microservices/shorts/policy/tenant-scope.cedar` PERMIT 8 + FORBID minor-*.
- `microservices/shorts/policy/dual-context-isolation.md` DCI-11.
- `microservices/shorts/runbooks/age-gate-bypass-incident.md`.
- `microservices/shorts/capabilities/T1-assist.yaml`.
- `microservices/shorts/capabilities/T2-auto.yaml`.
- `microservices/shorts/iac/kustomize/overlays/pack-kr/kustomization.yaml` (KR threshold 14).
- `microservices/shorts/iac/kustomize/overlays/pack-eu/kustomization.yaml` (EU threshold 16).
