---
doc_class: ComplianceMatrix
template_id: TPL-COMPLIANCE
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-compliance + axis-recordings + council-privacy
related_adrs: [ADR-0117, ADR-0135, ADR-0131, ADR-0133, ADR-RECORDINGS-0001, ADR-RECORDINGS-0002, ADR-RECORDINGS-0003, ADR-RECORDINGS-0006]
doc_status: published
---

# Compliance Matrix: recordings µservice

## Frameworks honoured

| Framework | Scope | Activation |
|---|---|---|
| SOC 2 Type 2 (CC6, CC7, CC8) | every pack | always-on |
| ISO/IEC 27001:2022 (Annex A controls per threat-model) | every pack | always-on |
| ISO/IEC 27037:2012 | digital-evidence preservation | always-on for ediscovery + legal-hold |
| NIST SP 800-86 | forensic-integrity | always-on |
| NIST SP 800-218 (SSDF) | secure SDLC | always-on |
| SLSA L3 | build provenance | always-on |
| OWASP ASVS v4.0.3 | application security | always-on |
| CIS Kubernetes Benchmark v1.9 | K8s hardening | always-on |
| eIDAS | qualified signatures on export bundles | conditional per tenant |
| EBU R128 | audio loudness | always-on |
| ITU-T G.107 | audio quality (E-model) | always-on |
| SMPTE-TT | timed-text | always-on |
| GDPR (Arts. 5/6/9/13/14/17/22/25/30/32/33/35/44-50) | pack-eu | always-on for EU tenants |
| ePrivacy Directive 2002/58 Art. 5(3) | pack-eu | always-on |
| EU AI Act (Arts. 13/27/50/Annex III) | pack-eu | always-on for AI capabilities |
| NIS2 Directive 2022/2555 | pack-eu | conditional (threshold) |
| HIPAA 45 CFR §§164.308/312/316/502/514/530 | pack-us-healthcare | BAA-conditional |
| HITECH Act | pack-us-healthcare | BAA-conditional |
| SEC Rule 17a-4(f) | pack-us-financial | always-on for SEC-regulated tenants |
| FINRA Rule 4511 | pack-us-financial | always-on for FINRA-regulated tenants |
| MiFID II Art. 16(7) | pack-eu (financial-services) | conditional |
| CFTC Rule 1.31 | pack-us-financial | conditional |
| FRCP Rule 26(f)/34 | every pack | always-on for ediscovery |
| Sedona Conference | every pack | always-on for ediscovery |
| KR PIPA Arts. 15/17/22-2/23/28/29 | pack-kr | always-on for KR tenants |
| KR-ISMS-P | pack-kr | tenant-conditional |
| KR 전자문서법 (Electronic Document Act) | pack-kr | always-on |
| KR 통신비밀보호법 (Wiretap Act) | pack-kr | always-on (recording-consent gate) |
| APPI (Japan) | pack-jp | always-on |
| PDPA 2012 (Singapore) | pack-sg | always-on |
| Privacy Act 1988 + TIA Act + Surveillance Devices Act | pack-au | always-on |
| DPDPA 2023 (India) | pack-in | always-on |
| LGPD (Brazil) | pack-br | always-on |
| UAE PDPL | pack-ae | always-on |
| KSA PDPL + SAMA | pack-ksa | always-on |

## Per-Article Control Mapping

### GDPR

| Article | Control surface | Evidence |
|---|---|---|
| Art. 5(1)(a) lawfulness | Cedar policies + recording-consent banner | `policy/cedar/*.cedar` + ingest-consent flag |
| Art. 5(1)(b) purpose-limitation | per-pack overlay | `policy/data-residency.md` |
| Art. 5(1)(c) data-minimisation | redaction overlay | ADR-RECORDINGS-0003 |
| Art. 5(1)(d) accuracy | content_hash + audit-chain seal | `tests/e2e/audit-chain-content-hash.rs` |
| Art. 5(1)(e) storage-limitation | retention purge + KMS-shred | ADR-RECORDINGS-0002 |
| Art. 5(1)(f) integrity + confidentiality | tenant-DEK envelope encryption | Bominal ADR-0111 |
| Art. 5(2) accountability | audit-chain Ed25519 seals | Bominal ADR-0028 |
| Art. 6 lawfulness | per-purpose legal basis | `dpia.md` |
| Art. 9 special-category | diarization opt-in | DPIA R-01 |
| Art. 13/14 transparency | producer-side recording-consent banner | meet / messenger |
| Art. 17 right-to-erasure | DSR cascade | `runbooks/dsr-cascade.md` (referenced from common pack) |
| Art. 22 automated-decision | auto-summary Annex III gate | ADR-RECORDINGS-0006 |
| Art. 25 by-design | redaction overlay; default-deny Cedar | always-on |
| Art. 30 record-of-processing | ROP entry per pack | `legal/transfer-register.md` |
| Art. 32 security | encryption + access control | `threat-model.md` |
| Art. 33 breach-notification | breach-detector emits to ops-compliance | always-on |
| Art. 35 DPIA | this set of docs | `dpia.md` |
| Arts. 44-50 transfer | residency pinning | ADR-0117 |

### HIPAA (pack-us-healthcare)

| 45 CFR Section | Control surface | Evidence |
|---|---|---|
| §164.308(a)(1) — security management | risk-analysis | `dpia.md` + `threat-model.md` |
| §164.308(a)(3) — workforce security | role-based access | tenancy µservice |
| §164.308(a)(4) — information-access management | Cedar policies | `policy/cedar/` |
| §164.308(a)(5) — security awareness + training | training curriculum | ops-security |
| §164.312(a)(1) — access control | mTLS + WebAuthn step-up | always-on |
| §164.312(b) — audit controls | audit-chain | always-on |
| §164.312(c)(1) — integrity | content_hash + Ed25519 seals | always-on |
| §164.312(d) — person + entity authentication | SPIFFE identity | always-on |
| §164.312(e)(1) — transmission security | TLS 1.3 + mTLS | always-on |
| §164.316 — policies + procedures | this doc + runbooks | always-on |
| §164.502(b) — minimum necessary | export-scope strict matching | ADR-RECORDINGS-0002 |
| §164.514 — de-identification (Safe Harbor) | redaction overlay matches 18 identifiers | ADR-RECORDINGS-0003 |
| §164.530(j) — 6-yr retention | pack-us-healthcare retention floor | ADR-RECORDINGS-0002 |

### SEC 17a-4(f) + FINRA 4511 + MiFID II 16(7) (pack-us-financial + pack-eu-financial)

| Rule | Control | Evidence |
|---|---|---|
| SEC 17a-4(f)(2) WORM | S3 object-lock + legal-hold-default-on | pack-us-financial overlay |
| SEC 17a-4(f)(3) accessibility | indexed retrieval + audit-chain | always-on |
| SEC 17a-4(b)(4) retention 3y / 6y | pack-default 3y; tenant-configurable to 7y | ADR-RECORDINGS-0002 |
| FINRA 4511 retention 6y | pack-default 6y | ADR-RECORDINGS-0002 |
| MiFID II Art. 16(7) — recording 5y + on-request to 7y | pack-default 5y; on-request extension to 7y | ADR-RECORDINGS-0002 |
| CFTC Rule 1.31 — recorded comms | aligned with SEC 17a-4 | ADR-RECORDINGS-0002 |

### EU AI Act

| Article | Control | Evidence |
|---|---|---|
| Art. 13 — technical documentation (high-risk) | per-capability `evidence_topic` | `capabilities/T2-auto.yaml` |
| Art. 27 — FRIA | DPIA section R-05 | `dpia.md` |
| Art. 50 — transparency | every transcription / summary / translate output labelled `ai-generated` | ADR-RECORDINGS-0006 |
| Annex III §4(a) — employment context | tenant high-risk attestation gate | ADR-RECORDINGS-0006 |
| Annex III §6 — law-enforcement context | tenant high-risk attestation gate | ADR-RECORDINGS-0006 |
| Annex III §8 — administration of justice | tenant high-risk attestation gate | ADR-RECORDINGS-0006 |

### KR PIPA + 전자문서법 + 통신비밀보호법

| Article | Control | Evidence |
|---|---|---|
| PIPA Art. 15 — collection consent | recording-consent banner | producer side |
| PIPA Art. 17 — third-party transfer | residency pinning | ADR-0117 |
| PIPA Art. 22-2 — DPIA | this set of docs | `dpia.md` |
| PIPA Art. 23 — sensitive-info | diarization opt-in | DPIA R-01 |
| PIPA Art. 28 — technical security | encryption-at-rest + KMS | always-on |
| PIPA Art. 29 — admin security | runbook + procedures | always-on |
| 전자문서법 Art. 5 — electronic-doc retention with integrity | audit-chain Merkle seal | always-on |
| 전자문서법 Art. 6 — long-term preservation | tiered storage hot + cold | ADR-RECORDINGS-0005 |
| 통신비밀보호법 — recording-consent | ingest refuses without `consent_banner_confirmed: true` | ingest contract |

### KR-ISMS-P

| §2 control | Recordings surface |
|---|---|
| §2.1 policy + governance | this set of docs |
| §2.5 access control | Cedar policies |
| §2.7 system + service security | sandbox (gVisor) + LTS pinning |
| §2.10 incident handling | runbooks |
| §2.12 privacy | DPIA |

## Annual Audit Calendar

| Audit | Owner | Cadence |
|---|---|---|
| SOC 2 Type 2 | ops-compliance + external | annual |
| ISO 27001:2022 surveillance | ops-compliance + external | annual |
| HIPAA risk-analysis review | council-privacy + external | annual |
| GDPR DPIA review | council-privacy | annual |
| EU AI Act FRIA review | council-privacy + ops-compliance | annual + per high-risk activation |
| KR PIPA Art. 22-2 PIA | council-privacy + KR PIPC | per major change |
| SEC 17a-4 attestation | ops-compliance | per pack-us-financial tenant onboarding |

## References

- All frameworks listed above.
- `dpia.md`, `threat-model.md`, `policy/data-residency.md`.
- `decisions/ADR-RECORDINGS-0001..0007.md`.

---



## §day-one-cert-readiness
This anchor is closed for `recordings` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `recordings` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +15 more.
- Example: `chapter-marker-suggest` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `recordings` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md` without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `chapter-marker-suggest` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `recordings` against ADR-0292 §D-1: minor-user refusal, teen tier and age-verification handling.

### Service-specific answer
- Minor exposure for `recordings` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `chapter-marker-suggest` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen-tier activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `recordings` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `recordings` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`, `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`; +17 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `chapter-marker-suggest` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.recordings.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `recordings` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `recordings` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `chapter-marker-suggest` touches those data classes.
- Signal sources: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`; +13 more.
- Example event class: `oya.recordings.chapter.marker.suggest.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §ml-model-lifecycle
This anchor is closed for `recordings` against documentation-rigor.md §3.2.6.E: model inventory, retrain cadence and promotion gates.

### Service-specific answer
- Local ML posture: `False` for direct model use; inherited detection/intelligence models still require versioned consumption evidence.
- Model inventory key: `manifest.json:ml_models` or the Intelligence audience tag `recordings.chapter-marker-suggest` if models are substrate-hosted.
- Promotion gates: offline eval, bias/fairness report, drift threshold, SLO budget, rollback model id, and human approval for high-risk/adverse-action paths.
- Retraining cadence is model-specific; high-risk models require documented data cut, feature schema, holdout set, and pack-specific legal review.
- Example: `chapter-marker-suggest` model output is never the sole authority for a legal/financial/employment/minor-impacting decision; Cedar and human-review policies remain in control.
- Deprecated model versions sunset under ADR-0258 with traffic split, canary, rollback, and post-promotion audit.
- Model cards include intended use, non-use, data provenance, performance by segment, failure modes, and owner.
- Services without local models keep this as a negative declaration so future agents cannot silently add ML without the lifecycle gate.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: NIST AI RMF model-governance lifecycle is the reference pattern for the control shape described here.
- Precedent 2: Google Model Cards is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §detection-fairness-audit
This anchor is closed for `recordings` against documentation-rigor.md §3.2.6.E: fairness metrics, thresholds and disaggregated false-positive audit.

### Service-specific answer
- Fairness audit applies to `recordings` risk/detection decisions that affect access, ranking, safety, money, employment, health, or protected classes.
- Metrics: false-positive rate ratio, false-negative rate ratio, calibration by segment, equalized-odds gap, appeal overturn rate, and challenge-friction rate.
- Thresholds: no protected segment exceeds 1.25x baseline false-positive rate without documented mitigation and human review.
- Segments are derived from lawful, minimized attributes; `recordings` never stores protected attributes solely to make a product feature easier.
- Example: `chapter-marker-suggest` abuse/risk score challenge rate is compared across locale, accessibility profile, age tier, and jurisdiction pack.
- Audit cadence: every model/rule promotion, quarterly for active high-risk detectors, and after any SEV involving false positives.
- Fairness reports are retained in audit evidence; raw protected-attribute joins remain in restricted analytics cells.
- If the service has no ML, deterministic rules still get false-positive and appeal-rate monitoring.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: Microsoft Fairlearn audit pattern is the reference pattern for the control shape described here.
- Precedent 2: NIST AI RMF measurement function is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `recordings` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `recordings` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.recordings.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `chapter-marker-suggest` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `chapter-marker-suggest` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `recordings` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `recordings` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`, `recordings.recordings`.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `recordings.recordings` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `recordings` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `recordings` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`; +12 more.
- Example: `chapter-marker-suggest` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `recordings` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.recordings` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/recordings/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.
- Example: `chapter-marker-suggest` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `recordings` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `recordings` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`, `microservices/recordings/iac/helm/recordings/Chart.yaml`, `microservices/recordings/iac/helm/recordings/templates/deployment.yaml`, `microservices/recordings/iac/helm/recordings/templates/hpa.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `chapter-marker-suggest` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `recordings` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `recordings` is in annual full-scope pentest and every major `chapter-marker-suggest` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`, `microservices/recordings/iac/helm/recordings/Chart.yaml`, `microservices/recordings/iac/helm/recordings/templates/deployment.yaml`, `microservices/recordings/iac/helm/recordings/templates/hpa.yaml`; +15 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `recordings` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `recordings` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `recordings` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `chapter-marker-suggest` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `recordings` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `recordings` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/recordings/catalog/oya-recordings-app.yaml`, `microservices/recordings/catalog/oya-recordings-export-adapter-pandoc.yaml`, `microservices/recordings/catalog/oya-recordings-legal-hold-kernel.yaml`, `microservices/recordings/catalog/oya-recordings-media-segment-adapter-cdn-cloudfront-stub-or-self.yaml`, `microservices/recordings/catalog/oya-recordings-media-segment-adapter-ffmpeg.yaml`, `microservices/recordings/catalog/oya-recordings-recording-adapter-postgres.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `chapter-marker-suggest` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `recordings` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `recordings` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `chapter-marker-suggest` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `chapter-marker-suggest` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `recordings` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `recordings.recordings`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `chapter-marker-suggest` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `recordings`; owner `axis-recordings`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `recordings`.
- Capability records cited: `microservices/recordings/capabilities/T0-suggest.yaml`, `microservices/recordings/capabilities/T1-assist.yaml`, `microservices/recordings/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar/policy artifacts cited: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- SLO and dashboard evidence: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/recordings/contracts/asyncapi/recordings-events.yaml`, `microservices/recordings/contracts/openapi/recordings.yaml`, `microservices/recordings/contracts/proto/recordings.proto`.
- Cedar binding: `microservices/recordings/policy/cedar/auditor-scope.cedar`, `microservices/recordings/policy/cedar/ci-scope.cedar`, `microservices/recordings/policy/cedar/legal-hold.cedar`, `microservices/recordings/policy/cedar/public-read.cedar`, `microservices/recordings/policy/cedar/tenant-scope.cedar`, `microservices/recordings/policy/data-residency.md`.
- State/event binding: `recordings.recordings`.
- Capability binding: `chapter-marker-suggest`, `transcription-diarization-summary-pii-redact`, `auto-translate-and-auto-publish`.
- SLO binding: `microservices/recordings/slos/export-mp4-latency.openslo.yaml`, `microservices/recordings/slos/export-transcript-pdf-latency.openslo.yaml`, `microservices/recordings/slos/legal-hold-chain-of-custody-correctness.openslo.yaml`, `microservices/recordings/slos/legal-hold-engagement-latency.openslo.yaml`, `microservices/recordings/slos/playback-start-latency.openslo.yaml`, `microservices/recordings/slos/recording-list-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/recordings/runbooks/ediscovery-export.md`, `microservices/recordings/runbooks/legal-hold-court-order-receipt.md`, `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`, `microservices/recordings/runbooks/redaction-overlay-corruption.md`, `microservices/recordings/runbooks/retention-policy-rollback.md`, `microservices/recordings/runbooks/transcode-pipeline-failure.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `recordings`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `recordings`.
- `policy-engine` supplies the signed Cedar corpus while `recordings` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `recordings` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `recordings`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `recordings` applies the most restrictive policy and emits a degraded-mode audit event.
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

