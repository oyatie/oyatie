---
doc_class: DPIA
template_id: TPL-DPIA
microservice: slides
status: Accepted
date: 2026-05-17
owner_team: axis-workspace + ops-security + dpo-office
applicable_jurisdictions: [kr, eu, us, us-healthcare, jp, sg, au, in, br, ae, ksa]
doc_status: published
---

# Data Protection Impact Assessment — slides µservice

This DPIA is required because the µservice processes personal data at scale across 11 packs, supports T2 AI-content-generation that may operate on data subjects' content (GDPR Art. 35(3)(a) + EU AI Act Annex III triggers possible per pack/usage), enables real-time large-audience broadcast (potentially monitored attendee data per GDPR Art. 35(3)(c)), and (in pack `us-healthcare`) processes Protected Health Information under HIPAA when used for clinical presentations.

## 1. Description of processing

### 1.1 Nature of processing

slides processes:
- Deck content authored by tenant users (text, images, video, audio, charts linked to sheets, tables, equations, animations, transitions).
- Editor session metadata (open/close times, author identity, edit ops via Loro CRDT).
- Comments and suggestions (author identity + timestamp + content).
- Version history (Ed25519-sealed diffs).
- Presenter and audience identity during present-mode and broadcast-mode (attendee count is the minimal default; per-viewer presence opt-in).
- Real-time engagement signals (reactions, polls, Q&A) during broadcast-mode.
- AI-content-generation prompts + completions (foundry-runtime archived 90d for audit).
- Alt-text suggestions (T1 AI-assist) — operates on uploaded image binaries; per-pack PHI redaction in us-healthcare.
- Imported PPTX/ODP/Keynote files (may contain embedded PII, PHI, financial data, attorney-client material per tenant context).

### 1.2 Scope

- Personal data categories: identifiers (OIDC sub, display name, email), behavioral (edit ops, broadcast attendance, present-mode timing), media (uploaded images potentially containing biometric/facial data depending on tenant usage), special-category data (potential in us-healthcare pack; potential in eu pack via tenant-uploaded medical/legal content).
- Data subjects: tenant employees (authors, reviewers, presenters), tenant customers + collaborators (external editors), audience members (broadcast viewers).
- Geographic scope: 11 packs covering KR, EU, US, US-healthcare, JP, SG, AU, IN, BR, AE, KSA. Cross-pack collab forbidden by overlay enforcement.
- Volume baseline: 10k active editor sessions per region (XL tier: 200k); 500 baseline / 5000 max broadcast viewers per deck.

### 1.3 Context

- Tenant value: presentation authoring is workspace-foundational; tenants control content. Slides is processor under GDPR Art. 28 with tenant as controller in most cases (controller-to-controller for cross-tenant share, processor in tenant-to-tenant collab).
- Purposes are limited to: authoring, collaboration, presentation, broadcast, sharing, AI-assist (under foundry-runtime contract), audit.
- Retention: per-pack default (eu 90d audit log + 7y signed-output archive PDF/A; us-healthcare 6y per HIPAA; tenant-tunable upward only within pack max).

### 1.4 Purposes

- Authoring + collaboration (lawful basis GDPR Art. 6(1)(b) — contract).
- Audit + compliance (Art. 6(1)(c) — legal obligation; Art. 6(1)(f) — legitimate interest).
- Broadcast attendance metrics (Art. 6(1)(b) — performance of contract w/ presenter; broadcast viewers via tenant T&C).
- AI-assist (Art. 6(1)(b) and tenant opt-in/opt-out per pack; never used for training without explicit opt-in per foundry-runtime contract).

## 2. Necessity and proportionality

### 2.1 Lawful basis

Per Art. 6: (b) contract with tenant; (c) legal obligation for retention; (f) legitimate interest for fraud/abuse prevention. Per Art. 9 (special category) — explicit consent required where applicable (US-healthcare pack; EU pack if tenant uploads health data); per Art. 22 — no solely automated decisions affecting data subjects within slides (T2 AI-content-generation is decision-support, not legal-effect; ADR-SLIDES-0006).

### 2.2 Data minimization (Art. 5(1)(c))

- Broadcast attendee count default = aggregate-only; per-viewer identities not stored beyond 24h unless tenant opts in.
- Audit metadata pseudonymized at OIDC sub level; identity↔sub mapping in tenancy µservice only.
- AI prompts + completions hashed; full text retained 90d for audit; cryptographic delete on retention expiry.
- Speaker-notes never broadcast (T-I-07 mitigation).

### 2.3 Accuracy + storage limitation

- Version-history enables tenant correction (Art. 16).
- Per-pack retention enforced; un-restorable past retention.
- Export-on-request supported via SDK (Art. 20 — data portability via PPTX/ODP/PDF/MP4 export).

### 2.4 Security (Art. 32)

- Encryption in transit: TLS 1.3 + WSS.
- Encryption at rest: SSE-KMS per-pack key for S3; pg_tde for Postgres; Redis at-rest encryption.
- Per-tenant isolation: RLS + per-cell Redis cluster + per-tenant S3 prefix + per-tenant CDN cache key.
- Resilience: HPA + multi-AZ + cross-region DR (per-pack).
- Testing: AC drills + threat-model verification + quarterly red-team.

## 3. Risk assessment

### 3.1 Risks to data subjects

| Risk ID | Risk | Likelihood (low/med/high) | Severity (low/med/high) | Inherent risk | Residual risk after mitigations |
|---|---|---|---|---|---|
| R-1 | Unauthorized disclosure via per-slide ACL bypass | med | high | high | low (Cedar per-slide enforcement, audit) |
| R-2 | Cross-tenant leakage via CDN cache | low | high | med | low (per-tenant cache key) |
| R-3 | Broadcast attendee identification beyond minimum | med | med | med | low (aggregate-default, opt-in) |
| R-4 | Speaker-notes leak during broadcast | low | high | med | very low (broadcast frame excludes notes layer) |
| R-5 | PHI exposure in alt-text via T1 AI | med (us-healthcare pack only) | high | high | low (per-pack PHI redaction + consent gate) |
| R-6 | T2 AI-content-generation processes special-category without consent | med | high | high | low (consent gate + risk-class refusal of Annex III high-risk by default) |
| R-7 | Stale chart-live-link reveals revoked sheet data | med | med | med | low (revocation cascade ≤ 5s; ADR-SLIDES-0008) |
| R-8 | Imported PPTX with embedded malware | high | high | high | low (gVisor sandbox + ClamAV + OPSWAT) |
| R-9 | Version-history persistence past retention | low | med | low | very low (cryptographic delete on retention expiry) |
| R-10 | Audit log compromise revealing engagement patterns | low | med | low | very low (audit-chain Ed25519 + access controls + per-pack retention) |
| R-11 | LiveKit broadcast attendee re-identification via signaling logs | med | med | med | low (signaling logs scrubbed; pseudonymous tokens) |
| R-12 | Cross-pack residency violation via misconfigured tenant | low | high | med | very low (overlay enforced + admission-gate refusal) |
| R-13 | T2 generated deck contains hallucinated PII | med | med | med | low (provenance watermark in generated decks; review prompt) |
| R-14 | Comment thread revealing private discussion to wider audience after access change | low | high | med | low (per-comment Cedar evaluation; revocation invalidates cache) |
| R-15 | MP4 export with embedded audience-camera frames stored beyond intent | low | high | med | low (audience-camera not captured into export by default; tenant opt-in required) |

### 3.2 Pack-specific risks

| Pack | Additional risk | Mitigation |
|---|---|---|
| us-healthcare | PHI in deck content (clinical presentations) | HIPAA-Business-Associate Agreement; PHI redaction in AI flows; 6y retention + cryptographic delete; T2 refused unless explicit PHI-consent flag per Annex III high-risk |
| eu | Special category data (Art. 9) | explicit consent flag; pack overlay enforces |
| kr | PIPA + 전자문서법 + 전자거래기본법 (presentations as electronic records) | KR-region pinning; KR PII inventory; KR breach notification per PIPA Art. 34 |
| jp | APPI | JP-region pinning; APPI consent + cross-border transfer controls |
| sg | PDPA Singapore | SG-region pinning; PDPA consent + notification |
| au | PDPA Australia (Privacy Act 1988) | AU-region pinning; APP notification |
| in | DPDPA 2023 | IN-region pinning; DPDPA consent + data principal rights |
| br | LGPD | BR-region pinning; ANPD requirements |
| ae | UAE PDPL | AE-region pinning; PDPL controller-processor agreement |
| ksa | KSA PDPL | KSA-region pinning; PDPL data subject rights |

## 4. Mitigations summary

- **Per-slide Cedar v4.2 default-deny ACL** (ADR-SLIDES-0007) — per-slide named-block granularity.
- **Loro CRDT no-silent-loss invariant** (AC-06) — every edit either merged or surfaced as conflict; never dropped.
- **gVisor sandbox + ClamAV + OPSWAT for imports** — untrusted file parsing isolated.
- **Per-pack residency overlay** — cross-pack collab forbidden.
- **Per-tenant CDN cache key + RLS + per-cell Redis + per-tenant S3 prefix** — isolation across all storage tiers.
- **Chart-live-link revocation cascade** (ADR-SLIDES-0008) — ≤ 5s.
- **Speaker-notes excluded from broadcast stream** by design (broadcast layer composes audience-frame only).
- **EU AI Act risk-class stamp** (ADR-SLIDES-0006) — T2 refused on Annex III high-risk by default.
- **Audit-chain Ed25519 seal** end-to-end for every save, ACL change, broadcast session, AI invocation, export job.
- **Per-pack PHI redaction in AI flows** (us-healthcare).
- **Retention + cryptographic delete on expiry** — per-pack policy.
- **WCAG 2.2 AA + reduced-motion + color-blind-safe** (ADR-SLIDES-0004) — accessibility default.
- **Subresource Integrity (SHA-384)** on every WASM chunk.
- **Strict CSP** — no inline scripts; no eval.

## 5. Consultation

- **Data Protection Officer (DPO) office**: consulted on broadcast attendee data (R-3, R-11) — aggregate-default approved.
- **Council-architecture**: per-slide ACL granularity (ADR-SLIDES-0007) approved.
- **Council-design-system**: reduced-motion default-on (ADR-SLIDES-0004) approved.
- **Ops-security**: gVisor sandbox + dual-scanner approved.
- **us-healthcare pack BAA review**: HIPAA Privacy Officer reviewed PHI redaction + retention; approved.
- **Tenant transparency**: data flows + AI-content-generation transparency notices published to `docs/standards/slides-data-flows.md` (per tenant T&C reference).

## 6. Documentation + ongoing review

- This DPIA reviewed annually or upon material change (new BC, new AI capability tier, new pack).
- Per ADR-0123 hyperscaler-maturity gate, DPIA evidence is a HG-SLIDES required artifact.
- Per-pack legal review SLA: 30d before pack activation.

## 7. Sign-off

- DPO office sign-off: pending IP-015 completion.
- Council-architecture sign-off: pending IP-015 completion.
- Ops-security sign-off: pending IP-015 completion.

## References

- GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44.
- EU AI Act Art. 16 + Annex III.
- HIPAA §§164.308, 164.310, 164.312 (us-healthcare pack).
- KR PIPA Arts. 17, 18, 28, 34.
- APPI (jp pack).
- PDPA SG + AU.
- DPDPA 2023 (in pack).
- LGPD (br pack).
- UAE PDPL.
- KSA PDPL.
- ISO 27001:2022.
- OWASP ASVS v4.
- NIST SSDF.
- W3C WCAG 2.2.
- ADR-SLIDES-0001 through ADR-SLIDES-0008.
