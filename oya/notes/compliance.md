---
doc_class: ComplianceMatrix
title: notes µservice — Regulatory + Standards Compliance Matrix
microservice: notes
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-legal + council-privacy + axis-notes
review_cadence: annually + on every regulatory change
doc_status: published
---

# Compliance Matrix — notes µservice

## Scope

This matrix maps notes-µservice controls to: GDPR; KR PIPA + 통신비밀보호법 + 정보통신망법; HIPAA; APPI; PDPA (SG, AU); DPDPA 2023; LGPD; UAE PDPL; KSA PDPL + SAMA; EU AI Act; ePrivacy; WCAG 2.2 AA; SLSA L3; NIST SSDF; SOC 2 Type 2; ISO 27001:2022; OWASP ASVS v4; CIS Kubernetes Benchmark v1.9; FIPS 140-3.

## Pack Activation Status

| Pack | Activated at M02 | Conditional gates |
|---|---|---|
| pack-kr | YES | KR PIPA Art. 28 controls + 통신비밀보호법 Art. 13 + 정보통신망법 Art. 28 |
| pack-eu | conditional | first GDPR-scope tenant + SCC on file |
| pack-us | conditional | first US-scope tenant |
| pack-us-healthcare | conditional | signed BAA + HIPAA-eligible source-target |
| pack-jp / sg / au / in / br / ae / ksa | conditional | first tenant + local-DPA review |

## GDPR (EU)

| Article | Control | Artifact |
|---|---|---|
| Art. 5(1)(a) lawfulness | Consent (Personal) + legitimate interest (Professional) | `dpia.md` §2.1 |
| Art. 5(1)(b) purpose limitation | µservice scope statement | `PRD.md` §Purpose |
| Art. 5(1)(c) data minimisation | Personal-tier events opaque; Ontology writes minimal | `PRD.md` §Workflow events |
| Art. 5(1)(d) accuracy | Inline edit; admin edit forbidden Personal | `policy/dual-context-isolation.md` |
| Art. 5(1)(e) storage limitation | per-pack retention bounds | `policy/data-residency.md` |
| Art. 5(1)(f) integrity + confidentiality | E2E (Personal), tenant-DEK (Professional), TLS + mTLS, MAC-tagged ciphertext | `policy/e2e-personal-tier-default.md`; `threat-model.md` |
| Art. 6 lawful basis | per-tier basis | `dpia.md` §2.1 |
| Art. 9 special category | E2E-protected; server-side processing impossible for Personal | ADR-NOTES-0001 |
| Art. 17 erasure | DSR cascade runner | `policy/data-residency.md` §DSR cascade |
| Art. 22 automated decision | T2 auto-organize disabled at minimum-shippable-tier; opt-in per user | `capabilities/T2-auto.yaml` |
| Art. 25 PbD + PbDef | Per-microservice flat layout + LEAN lanes + Cedar default-deny | ADR-0131; ADR-0064 |
| Art. 28 processor | DPA + sub-processor list (foundry-runtime + drive + tasks) | `legal/sub-processors.md` (linked) |
| Art. 30 records | Workflow event ledger + audit-chain seals | `threat-model.md` §Audit |
| Art. 32 security | controls per `policy/e2e-personal-tier-default.md`; FIPS 140-3 where required | (this matrix) |
| Art. 33 + 34 breach notification | Sev-1 incident-response runbook | `incident-response.md` |
| Art. 35 DPIA | `dpia.md` (this directory) | `dpia.md` |
| Art. 44–50 transfers | pack pinning + SCC | `policy/data-residency.md` |
| Recital 26 anonymisation | content not anonymised; pseudonymisation via tenant_id + user_id | `dpia.md` §3 |

## EU AI Act

| Article | Control |
|---|---|
| Art. 50 transparency | AI-assist results labelled as AI-generated; pack-eu overlay enforces `evidence_topic: oya.notes.capability.t1_assist.evidence` |
| Limited-risk classification | T1 summarize / tag-suggest / link-suggest classified `limited_risk`; T2 auto-organize classified `limited_risk` with conformity-assessment commitments |
| Art. 27 conformity assessment (high-risk if applicable) | not currently in scope (notes is not safety-critical or HR-decision domain) |

## KR PIPA

| Article | Control |
|---|---|
| Art. 15 collection | consent at signup for Personal; tenant-of-tenant consent for Professional |
| Art. 17 third-party provision | tenant-controlled; sub-processor list under DPA |
| Art. 22-2 personal-information-protection-officer | per tenant + Council of Privacy |
| Art. 23 sensitive info | E2E protection on Personal-tier covers most cases; explicit consent required where not |
| Art. 28 security measures | Art. 29-aligned controls + audit-chain |
| Art. 29 cryptographic + identity controls | MLS RFC 9420 + Cedar + audit-chain; pack-kr overlay enforces |

## KR 통신비밀보호법 (Telecommunications Secrecy Act)

| Article | Control |
|---|---|
| Art. 13 | confidentiality of communications preserved by E2E (Personal) + tenant-DEK (Professional) |

## KR 정보통신망법 (Information & Communications Network Act)

| Article | Control |
|---|---|
| Art. 28 | technical + administrative controls per ADR-0028 audit-chain + ADR-NOTES-0001 E2E |

## HIPAA (pack-us-healthcare only)

| Section | Control |
|---|---|
| 45 CFR §164.308 administrative | risk analysis (DPIA) + workforce training |
| 45 CFR §164.310 physical | infra controls inherited from OCI HIPAA-eligible regions |
| 45 CFR §164.312(a)(2)(iv) encryption | tenant-DEK envelope + MLS (where Personal-tier in HIPAA scope, which is rare) |
| 45 CFR §164.312(b) audit | audit-chain Ed25519 seal per state transition |
| 45 CFR §164.502(b) minimum necessary | Cedar member-check + per-channel scope |
| 45 CFR §164.530(j) retention | 6-year floor for PHI-class notes |

## APPI (JP)

| Article | Control |
|---|---|
| Art. 17 + 18 collection/use limit | per `dpia.md` §2 |
| Art. 21 retention | 2-year floor (labor) per `policy/data-residency.md` |
| Art. 27 cross-border | pack-jp pinning |

## PDPA (SG) / PDPA (AU) / DPDPA 2023 (IN) / LGPD (BR) / UAE PDPL / KSA PDPL + SAMA

Pack overlays in `policy/data-residency.md` enforce per-pack retention + transfer rules. Each overlay carries:

- jurisdiction-specific consent text;
- retention floor;
- DPO contact;
- cross-border transfer permission gate.

## ePrivacy Directive 2002/58/EC

| Article | Control |
|---|---|
| Art. 5(3) cookies/storage | Web-clipper extension manifest declares minimum-permission storage; Workflow Studio shell uses essential storage only |
| Art. 5(1) confidentiality | E2E (Personal) + tenant-DEK (Professional) |

## WCAG 2.2 AA

| SC | Control |
|---|---|
| 1.3.1 info-and-relationships | semantic HTML in editor + clipper |
| 1.4.3 contrast | per WCAG AA design tokens |
| 2.1.1 keyboard | full keyboard navigation; no mouse-required affordance |
| 2.4.6 headings | Markdown heading hierarchy preserved |
| 3.1.1 language | per-pack language tag on rendered HTML |
| 4.1.3 status messages | ARIA live region for sync state + share-link emission |

## SLSA L3

- Reproducible Cargo builds (`cargo --frozen --locked`).
- HSM-signed artifacts at release.
- Provenance recorded in `evidence/release-pointer-*.json`.
- Dependency pinning verified by `oya gate validate version-pinning-conformance`.

## NIST SSDF (SP 800-218)

| Practice | Control |
|---|---|
| PO.1 prepare | Security requirements in PRD §NFR Security |
| PS.1 protect | Cedar v4 default-deny + LEAN lanes |
| PW.4 produce | Signed builds + reproducible Cargo |
| RV.1 respond | incident-response runbook |

## SOC 2 Type 2

| Trust Service Criterion | Control |
|---|---|
| CC6.1 logical access | OIDC + Cedar + per-tenant scope |
| CC6.7 transmission | mTLS + TLS 1.3 |
| CC7.2 monitoring | observability µservice + Prometheus rules |
| A1.2 availability | OpenSLO + burn-rate gates |

## ISO 27001:2022

| Annex A clause | Control |
|---|---|
| A.5.15 access control | Cedar + tenant-scope |
| A.8.3 cryptographic | MLS + tenant-DEK + FIPS 140-3 modules |
| A.5.23 cloud service customer | per-pack residency contract |
| A.8.24 use of cryptography | ADR-NOTES-0001 |

## OWASP ASVS v4

| Level | Section | Control |
|---|---|---|
| L2 | §2.4 password storage | PBKDF2-SHA256 ≥ 600k iter for share-link passphrase |
| L2 | §4 authorization | Cedar v4.2 default-deny |
| L2 | §6 cryptography | MLS + tenant-DEK + FIPS 140-3 |
| L2 | §8 data protection | E2E-default Personal-tier |
| L2 | §11 business logic | dual-context-isolation invariants |

## CIS Kubernetes Benchmark v1.9

Pod security per Helm templates:
- `runAsNonRoot: true`
- `readOnlyRootFilesystem: true`
- `allowPrivilegeEscalation: false`
- `capabilities.drop: [ALL]`
- NetworkPolicy default-deny + explicit egress

## FIPS 140-3

| Module | Use |
|---|---|
| openmls 0.6 (when built with `fips` feature) | MLS encryption for Personal-tier |
| Cargo-RustCrypto (FIPS-mode) | random for share-link tokens |
| OCI FIPS-validated KMS | tenant-DEK wrapping |

## Verification Cadence

- `oya gate validate per-microservice-layout` per PR.
- `oya gate validate version-pinning-conformance` per PR.
- `oya gate validate e2e-ai-refusal` per PR.
- Quarterly compliance review per active pack.
- Annual external pen-test + SOC 2 Type 2 audit cycle + ISO 27001 surveillance audit.

## References

- See header `references:` in `dpia.md`.
- ADR-NOTES-0001..0006.
- All policy + runbook artifacts in this directory.

---



## §day-one-cert-readiness
This anchor is closed for `notes` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `notes` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +20 more.
- Example: `next-word-and-title-suggest` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §pack-overlay-roster
This anchor is closed for `notes` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `next-word-and-title-suggest` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §minor-protection
This anchor is closed for `notes` against ADR-0292 §D-1: minor-user refusal, teen tier and age-verification handling.

### Service-specific answer
- Minor exposure for `notes` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `next-word-and-title-suggest` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen-tier activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §platform-owner-indirection
This anchor is closed for `notes` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `notes` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`, `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`; +20 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `next-word-and-title-suggest` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.notes.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §detection-substrate-binding
This anchor is closed for `notes` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `notes` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `next-word-and-title-suggest` touches those data classes.
- Signal sources: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +21 more.
- Example event class: `oya.notes.next.word.and.title.suggest.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §investigation-binding
This anchor is closed for `notes` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `notes` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.notes.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `next-word-and-title-suggest` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `next-word-and-title-suggest` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §insider-threat-controls
This anchor is closed for `notes` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `notes` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`, `notes.notes`.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `notes.notes` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §threat-intelligence-feeds
This anchor is closed for `notes` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `notes` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +18 more.
- Example: `next-word-and-title-suggest` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §key-rotation-cadence
This anchor is closed for `notes` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.notes` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/notes/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.
- Example: `next-word-and-title-suggest` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §crypto-agility-plan
This anchor is closed for `notes` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `notes` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`, `microservices/notes/iac/ech-config.yaml`, `microservices/notes/iac/edge-waf.yaml`, `microservices/notes/iac/helm/notes/Chart.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `next-word-and-title-suggest` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §pentest-and-bounty-cadence
This anchor is closed for `notes` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `notes` is in annual full-scope pentest and every major `next-word-and-title-suggest` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`, `microservices/notes/iac/ech-config.yaml`, `microservices/notes/iac/edge-waf.yaml`, `microservices/notes/iac/helm/notes/Chart.yaml`; +21 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `notes` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §facility-controls
This anchor is closed for `notes` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `notes` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `next-word-and-title-suggest` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §supply-chain-risk
This anchor is closed for `notes` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `notes` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/notes/catalog/oya-notes-backlink-graph-kernel.yaml`, `microservices/notes/catalog/oya-notes-checklist-kernel.yaml`, `microservices/notes/catalog/oya-notes-collab-edit-adapter-loro.yaml`, `microservices/notes/catalog/oya-notes-collab-edit-kernel.yaml`, `microservices/notes/catalog/oya-notes-daily-note-kernel.yaml`, `microservices/notes/catalog/oya-notes-e2e-key-management-adapter-mls.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `next-word-and-title-suggest` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `notes` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `notes` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `next-word-and-title-suggest` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `next-word-and-title-suggest` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §data-classification
This anchor is closed for `notes` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `notes.notes`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `next-word-and-title-suggest` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `notes`; owner `axis-notes`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `notes`.
- Capability records cited: `microservices/notes/capabilities/T0-suggest.yaml`, `microservices/notes/capabilities/T1-assist.yaml`, `microservices/notes/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar/policy artifacts cited: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- SLO and dashboard evidence: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +17 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/notes/contracts/asyncapi/notes-events.yaml`, `microservices/notes/contracts/openapi/notes.yaml`, `microservices/notes/contracts/proto/notes.proto`.
- Cedar binding: `microservices/notes/policy/abuse-defence.cedar`, `microservices/notes/policy/auditor-scope.cedar`, `microservices/notes/policy/ci-scope.cedar`, `microservices/notes/policy/data-residency.md`, `microservices/notes/policy/dual-context-isolation.md`, `microservices/notes/policy/e2e-personal-tier-default.md`; +6 more.
- State/event binding: `notes.notes`.
- Capability binding: `next-word-and-title-suggest`, `summarize-and-tag-suggest-and-link-suggest`, `auto-organize-vault`.
- SLO binding: `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml`, `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml`, `microservices/notes/slos/full-text-search-latency.openslo.yaml`, `microservices/notes/slos/graph-render-latency.openslo.yaml`, `microservices/notes/slos/note-create-latency.openslo.yaml`, `microservices/notes/slos/note-open-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md`, `microservices/notes/runbooks/attachment-loss-recovery.md`, `microservices/notes/runbooks/clinical-note-leak-recovery.md`, `microservices/notes/runbooks/crdt-divergence-recovery.md`, `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md`, `microservices/notes/runbooks/import-pipeline-failure.md`; +5 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `notes`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `notes`.
- `policy-engine` supplies the signed Cedar corpus while `notes` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `notes` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `notes`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `notes` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.
