---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping (audit-chain)
microservice: audit-chain
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-audit-chain, council-architecture, ops-compliance
related_adrs: [ADR-0028, ADR-0003, ADR-0117, ADR-0123, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/audit-chain/threat-model.md
  - microservices/audit-chain/dpia.md
  - microservices/audit-chain/policy/seal-integrity.md
  - microservices/audit-chain/policy/data-residency.md
  - microservices/audit-chain/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (audit-chain µservice)

## Purpose

Canonical control-to-framework mapping for audit-chain. Because this µservice is the evidence backbone, **almost every SOC 2 CC4.x / ISO 27001 A.8.15 / GDPR Art. 30 / HIPAA §164.312(b) / KR PIPA Art. 29 control across every other µservice ultimately points here** for its evidence emission. This document tells external auditors which audit-chain artifact satisfies which framework clause for every µservice.

## Enforced Frameworks

### SOC 2 Type 2 (2017 TSC + 2022 Points of Focus)

| TSC | Control objective | Implementation in audit-chain | Evidence |
|---|---|---|---|
| CC4.1 Internal monitoring | Audit-chain emits SLI; verification correctness lane | `microservices/audit-chain/slos/` + `oya-check-verification-correctness` lane | self-SLO + lane history |
| CC4.2 Deficiency communication | `VerificationFailed` events emit on tamper detection | `runbooks/signature-verification-failure.md` | event stream |
| CC6.1 Logical access | Cedar policy + SPIFFE binding | `policy/tenant-scope.cedar` + `policy/ci-scope.cedar` + `policy/auditor-scope.cedar` | policy artifacts |
| CC6.2 Authentication + authorization | OIDC + SPIFFE + per-tenant API keys | `policy/seal-integrity.md` §"Signing call authenticated by SPIFFE" | OpenBao audit log |
| CC6.6 Logical access control | Three layers: Cedar + Postgres role + HSM IAM | `policy/seal-integrity.md` §"HSM Signing Policy" | layered policy artifacts |
| CC6.7 Information transmission + disposal | mTLS + KMS at rest + DSR cascade | `policy/data-residency.md` §"DSR Cascade" | DSR runner output |
| CC6.8 Vulnerability management | `cargo deny` + Trivy + Grype | governance µservice's supply-chain spec | CI lane history |
| CC7.1 System operations | HA emission + DR-pair sealing + autoscaling | `capacity-model.md` + `multi-region.md` | deployed state |
| CC7.2 Monitoring system inputs | Self-SLO + OnCall paging | `failure-modes.md` | SLO history |
| CC7.4 Incident response | Severity-classified response per `incident-response.md` | runbooks | postmortem evidence |
| CC8.1 Change management | Signed commits + LEAN lanes + CODEOWNERS | branch-protection.yaml + governance µservice | branch-protection state |

### ISO 27001:2022 (Annex A control families)

| Annex A | Control | Implementation in audit-chain | Evidence |
|---|---|---|---|
| A.5.14 Information transfer | mTLS + cross-pack-replication-forbidden | `policy/data-residency.md` | enforcement at emission |
| A.5.15 Access control | Cedar + Postgres role separation + HSM partition isolation | `policy/*.cedar` + `policy/seal-integrity.md` | policy artifacts |
| A.5.17 Authentication information | OpenBao + SPIFFE + 24h-cert rotation | `policy/seal-integrity.md` §"SI-09" | OpenBao audit log |
| A.5.27 Learning from incidents | Postmortems published per `incident-response.md` | `evidence/postmortems/` | postmortem corpus |
| A.5.28 Collection of evidence | THIS MICROSERVICE IS THE IMPLEMENTATION | self | every SealRecord |
| A.5.33 Protection of records | S3 Object Lock Compliance mode + Merkle integrity | `policy/seal-integrity.md` §"SI-04" + §"FM-SI-03" | S3 bucket policy |
| A.8.2 Privileged access rights | JIT elevation + 2-person rule | `policy/ci-scope.cedar` §"PERMIT 5" + §"FORBID 2-person bypass" | OpenBao audit |
| A.8.3 Information access restriction | Cedar default-deny | `policy/tenant-scope.cedar` | policy artifact |
| A.8.5 Secure authentication | SPIFFE-bound mTLS for every call | `policy/seal-integrity.md` §"SI-09" | SPIFFE attestation log |
| A.8.7 Protection against malware | Trivy + Grype + signed images | governance µservice | container scan history |
| A.8.11 Data masking | Caller-redaction contract per Bominal ADR-0003 | source µservices redact; audit-chain treats opaque | source-µservice SDK redactor |
| A.8.12 Data leakage prevention | Cross-tenant query refusal (Cedar) + audit-chain payload-class enforcement | `policy/tenant-scope.cedar` | LEAN lane |
| A.8.15 Logging | THIS MICROSERVICE IS THE IMPLEMENTATION | self | every emission |
| A.8.16 Monitoring activities | Self-SLO + cross-channel root validator | `slos/` + `oya:audit_chain_root_cross_channel_match` recording rule | SLO + recording rules |
| A.8.24 Use of cryptography | TLS 1.3 + Ed25519 (HSM-backed) + AES-256-GCM at rest | `policy/seal-integrity.md` §"SI-07" + §"SI-08" | crypto inventory |
| A.8.25 Secure development life cycle | LEAN lanes + spec-driven-development | governance µservice | lane history |
| A.8.28 Secure coding | `cargo clippy` + `cargo deny` + crypto-crate version-pin | governance µservice | CI history |
| A.8.34 Protection during audit testing | Auditor JIT tokens + scoped reads | `policy/auditor-scope.cedar` | auditor audit-of-audits |

### GDPR

| Art. | Requirement | Implementation in audit-chain | Evidence |
|---|---|---|---|
| 5(1)(d) Accuracy | Chain integrity per `policy/seal-integrity.md` | self | SealRecord |
| 5(1)(e) Storage limitation | Per-pack retention matrix | `policy/data-residency.md` §"Retention" | retention-cascade output |
| 5(1)(f) Integrity + confidentiality | Ed25519 + Merkle + Cedar | `policy/seal-integrity.md` | SealRecord + Cedar audit |
| 5(2) Accountability | Every state change has an audit-chain record | self | every emission |
| 17 Right to erasure | DSR cascade with chain-preservation | `policy/data-residency.md` §"DSR Cascade" | DSR audit |
| 25 Privacy by design + default | Default-deny Cedar + caller-redaction + pack-pinning | `policy/seal-integrity.md` + `policy/data-residency.md` | architecture |
| 28 Processor terms | DPA template + per-tenant audit access | `legal/dpa-template.md` | tenant DPAs on file |
| 30 Records of processing | THIS MICROSERVICE IS THE PLATFORM-WIDE REGISTER | self | every emission |
| 32 Security of processing | Threat-model mitigations | `threat-model.md` | mitigation cross-mapping |
| 33 Breach notification | 72h chain per `incident-response.md` | runbooks | incident audit trail |
| 35 DPIA | `dpia.md` | self | DPIA |
| 44–46 Cross-border transfers | SCC-only export bundle; pack-pinning default | `policy/data-residency.md` §"Exception" | transfer register |

## Suggested Frameworks (per-pack activation)

### pack-kr (KR-ISMS-P + KR PIPA + 전자문서법)

| Section | Requirement | Implementation |
|---|---|---|
| KR-ISMS-P §2.5 인적보안 | Access control | Cedar + JIT + 2-person rule |
| KR-ISMS-P §2.6 암호화 | TLS 1.3 + Ed25519 + AES-256 | `policy/seal-integrity.md` |
| KR-ISMS-P §2.9 사고관리 | Sev-1/2 reporting to PIPC within 72h | `incident-response.md` |
| KR-ISMS-P §2.12 위반관리 | Tamper detection per `policy/seal-integrity.md` §"SI-13..SI-14" | self |
| KR PIPA Art. 28 | Retention limitation | retention-cascade |
| KR PIPA Art. 29 | Technical safeguards | mapped in `threat-model.md` |
| KR PIPA Art. 29-2 | Encryption | Ed25519 + AES-256 |
| KR PIPA Art. 33 | DPIA | `dpia.md` |
| KR PIPA Art. 34 | Breach notification (72h to PIPC + 72h to subjects) | `incident-response.md` |
| KR PIPA Art. 36 | Right to erasure | DSR cascade |
| **KR 전자문서법 Art. 5** | Electronic document integrity | **Ed25519 + Merkle + WORM** — load-bearing |
| **KR 전자문서법 Art. 6** | Electronic document storage | S3 WORM + 3y retention default |
| **KR 전자문서법 Art. 7** | Electronic document verification | verification SDK |

### pack-us-healthcare (HIPAA)

| 45 CFR | Requirement | Implementation |
|---|---|---|
| §164.308(a)(1)(ii)(A) Risk analysis | `threat-model.md` + `dpia.md` |
| §164.308(a)(1)(ii)(B) Risk management | Mitigations in `dpia.md` §6 |
| §164.308(a)(4)(ii)(B) Access authorization | Cedar `auditor-scope.cedar` |
| §164.308(a)(6) Incident procedures | `incident-response.md` |
| §164.312(a)(1) Access control | Cedar + SPIFFE |
| **§164.312(b) Audit controls** | **THIS MICROSERVICE IS THE IMPLEMENTATION** |
| **§164.312(c)(1) Integrity** | Ed25519 + Merkle + WORM |
| §164.312(d) Person/entity authentication | OIDC + MFA + SPIFFE |
| §164.312(e)(1) Transmission security | TLS 1.3 |
| §164.314(a)(1) Business associate contracts | BAA template |
| **§164.316(b)(2) 6y retention** | retention-cascade enforces |
| §164.502(a) TPO permitted uses | Operations purpose only |
| §164.404 Notification | `incident-response.md` |

### pack-eu (GDPR + EDPB + eIDAS + NIS2)

- EDPB Guidelines 4/2019 Art. 25: pseudonymisation + pack-pinning + default-deny Cedar.
- EDPB Guidelines 9/2022 breach: 72h chain in `incident-response.md`.
- EDPB Recommendations 01/2020 (post-Schrems II): pseudonymisation + EU-pack KMS keys for SSE; supplementary measures `legal/schrems-supplementary-measures.md`.
- **eIDAS 910/2014 Art. 26 (AdES)**: HSM-Ed25519 satisfies AdES.
- NIS2 (2022/2555): incident-reporting timelines integrated.

### pack-jp (APPI)

| Art. | Requirement | Implementation |
|---|---|---|
| APPI Art. 17 | Purpose | DPIA §2.4 |
| APPI Art. 20 | Security measures | threat-model |
| APPI Art. 24 | Cross-border restrictions | pack-pinning |
| APPI Art. 26-2 | Breach notification | `incident-response.md` |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/audit-chain-compliance-overlay.md`. Structure mirrors above:
- pack-sg: PDPA + MAS Notice 644 (finance ≥ 5y retention).
- pack-au: Privacy Act APP + APRA-CPS 234 (finance ≥ 7y retention).
- pack-in: DPDPA 2023 + RBI Master Direction (finance ≥ 7y retention).
- pack-br: LGPD + BACEN Res. 4.893/2021 (finance ≥ 5y).
- pack-ae: UAE PDPL Federal Decree-Law 45/2021.
- pack-ksa: KSA PDPL + SAMA Cybersecurity Framework 2017 (finance ≥ 10y retention).

## Continuous Compliance Evidence

### Lane: `oya-governance-compliance-evidence-recency` (cross-cutting)

Refuses merges on evidence > 90d old without refresh stamp. Forces quarterly re-validation.

### Evidence emission

- `evidence/compliance/<framework>/<control>/<date>.json` — control evidence (config snapshot, lane-run output, audit-chain seal).
- `microservices/audit-chain/evidence/multispectrum/<change_id>-<unix_ts>.json` — per-changeset evidence.

### Per-framework continuous runs

- Daily: SOC 2 CC4.x + CC7.x; ISO 27001 A.8.15 + A.8.16.
- Weekly: CC8.x; A.5.27.
- Monthly: CC3.x; A.5.7; key-rotation status.
- Quarterly: full matrix re-validation.
- Annually: full external auditor re-attestation.

### Audit evidence delivery

External auditors receive a frozen evidence pack scoped to (tenant, framework, engagement-window) signed by pack HSM key; auditor JIT token (per `policy/auditor-scope.cedar`) scopes; every read audit-emitted.

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=compliance-evidence-recency --microservice audit-chain` — exit 0.
- `buck2 build //:quality-lane-registry-authority-check # lane=authority-cohesion` — exit 0.
- Annual SOC 2 Type 2 + ISO 27001:2022 audit results in `evidence/audits/`.

## References

- `microservices/audit-chain/threat-model.md`.
- `microservices/audit-chain/dpia.md`.
- `microservices/audit-chain/policy/*`.
- `microservices/audit-chain/incident-response.md`.
- Bominal ADR-0028 + ADR-0003.
- ADR-0117 + ADR-0123 + ADR-0131 + ADR-0140.
- SOC 2 + ISO 27001:2022 + GDPR + KR PIPA + KR 전자문서법 + HIPAA + APPI + PDPA + Privacy Act 1988 + DPDPA 2023 + LGPD + UAE PDPL + KSA PDPL + SAMA.
- eIDAS 910/2014.

---



## §day-one-cert-readiness
This anchor is closed for `audit-chain` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `audit-chain` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +9 more.
- Example: `audit-emit` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `audit-chain`; owner `axis-audit-chain`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `audit-chain`.
- Capability records cited: `microservices/audit-chain/capabilities/audit-emit.yaml`, `microservices/audit-chain/capabilities/seal-mint.yaml`, `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- API surfaces cited: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar/policy artifacts cited: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`; +10 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar binding: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- State/event binding: `audit_chain.audit_chain`.
- Capability binding: `audit-emit`, `seal-mint`, `verify-merkle`.
- SLO binding: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`.
- Runbook binding: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `audit-chain`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `audit-chain`.
- `policy-engine` supplies the signed Cedar corpus while `audit-chain` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `audit-chain` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `audit-chain`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `audit-chain` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `audit-chain` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar` without changing domain code.
- Data classes under pack control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `audit-emit` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `audit-chain`; owner `axis-audit-chain`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `audit-chain`.
- Capability records cited: `microservices/audit-chain/capabilities/audit-emit.yaml`, `microservices/audit-chain/capabilities/seal-mint.yaml`, `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- API surfaces cited: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar/policy artifacts cited: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`; +10 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar binding: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- State/event binding: `audit_chain.audit_chain`.
- Capability binding: `audit-emit`, `seal-mint`, `verify-merkle`.
- SLO binding: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`.
- Runbook binding: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `audit-chain`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `audit-chain`.
- `policy-engine` supplies the signed Cedar corpus while `audit-chain` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `audit-chain` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `audit-chain`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `audit-chain` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `audit-chain` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `audit-chain` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`, `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`; +15 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `audit-emit` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.audit-chain.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `audit-chain`; owner `axis-audit-chain`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `audit-chain`.
- Capability records cited: `microservices/audit-chain/capabilities/audit-emit.yaml`, `microservices/audit-chain/capabilities/seal-mint.yaml`, `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- API surfaces cited: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar/policy artifacts cited: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`; +10 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar binding: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- State/event binding: `audit_chain.audit_chain`.
- Capability binding: `audit-emit`, `seal-mint`, `verify-merkle`.
- SLO binding: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`.
- Runbook binding: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `audit-chain`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `audit-chain`.
- `policy-engine` supplies the signed Cedar corpus while `audit-chain` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `audit-chain` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `audit-chain`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `audit-chain` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `audit-chain` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `audit-chain` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `audit-emit` touches those data classes.
- Signal sources: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`; +9 more.
- Example event class: `oya.audit.chain.audit.emit.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `audit-chain`; owner `axis-audit-chain`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `audit-chain`.
- Capability records cited: `microservices/audit-chain/capabilities/audit-emit.yaml`, `microservices/audit-chain/capabilities/seal-mint.yaml`, `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- API surfaces cited: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar/policy artifacts cited: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`; +10 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar binding: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- State/event binding: `audit_chain.audit_chain`.
- Capability binding: `audit-emit`, `seal-mint`, `verify-merkle`.
- SLO binding: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`.
- Runbook binding: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `audit-chain`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `audit-chain`.
- `policy-engine` supplies the signed Cedar corpus while `audit-chain` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `audit-chain` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `audit-chain`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `audit-chain` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `audit-chain` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `audit-chain` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.audit-chain.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `audit-emit` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `audit-emit` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `audit-chain`; owner `axis-audit-chain`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `audit-chain`.
- Capability records cited: `microservices/audit-chain/capabilities/audit-emit.yaml`, `microservices/audit-chain/capabilities/seal-mint.yaml`, `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- API surfaces cited: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar/policy artifacts cited: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`; +10 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar binding: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- State/event binding: `audit_chain.audit_chain`.
- Capability binding: `audit-emit`, `seal-mint`, `verify-merkle`.
- SLO binding: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`.
- Runbook binding: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `audit-chain`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `audit-chain`.
- `policy-engine` supplies the signed Cedar corpus while `audit-chain` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `audit-chain` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `audit-chain`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `audit-chain` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `audit-chain` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `audit-chain` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`, `audit_chain.audit_chain`.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `audit_chain.audit_chain` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `audit-chain`; owner `axis-audit-chain`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `audit-chain`.
- Capability records cited: `microservices/audit-chain/capabilities/audit-emit.yaml`, `microservices/audit-chain/capabilities/seal-mint.yaml`, `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- API surfaces cited: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar/policy artifacts cited: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`; +10 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar binding: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- State/event binding: `audit_chain.audit_chain`.
- Capability binding: `audit-emit`, `seal-mint`, `verify-merkle`.
- SLO binding: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`.
- Runbook binding: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `audit-chain`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `audit-chain`.
- `policy-engine` supplies the signed Cedar corpus while `audit-chain` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `audit-chain` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `audit-chain`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `audit-chain` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `audit-chain` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `audit-chain` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`; +10 more.
- Example: `audit-emit` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `audit-chain`; owner `axis-audit-chain`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `audit-chain`.
- Capability records cited: `microservices/audit-chain/capabilities/audit-emit.yaml`, `microservices/audit-chain/capabilities/seal-mint.yaml`, `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- API surfaces cited: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar/policy artifacts cited: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`; +10 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar binding: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- State/event binding: `audit_chain.audit_chain`.
- Capability binding: `audit-emit`, `seal-mint`, `verify-merkle`.
- SLO binding: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`.
- Runbook binding: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `audit-chain`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `audit-chain`.
- `policy-engine` supplies the signed Cedar corpus while `audit-chain` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `audit-chain` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `audit-chain`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `audit-chain` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `audit-chain` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.audit-chain` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/audit-chain/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.
- Example: `audit-emit` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `audit-chain`; owner `axis-audit-chain`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `audit-chain`.
- Capability records cited: `microservices/audit-chain/capabilities/audit-emit.yaml`, `microservices/audit-chain/capabilities/seal-mint.yaml`, `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- API surfaces cited: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar/policy artifacts cited: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`; +10 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar binding: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- State/event binding: `audit_chain.audit_chain`.
- Capability binding: `audit-emit`, `seal-mint`, `verify-merkle`.
- SLO binding: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`.
- Runbook binding: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `audit-chain`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `audit-chain`.
- `policy-engine` supplies the signed Cedar corpus while `audit-chain` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `audit-chain` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `audit-chain`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `audit-chain` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `audit-chain` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `audit-chain` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`, `microservices/audit-chain/iac/helm/audit-storage/Chart.yaml`, `microservices/audit-chain/iac/helm/audit-storage/templates/deployment.yaml`, `microservices/audit-chain/iac/helm/audit-storage/templates/networkpolicy.yaml`; +7 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `audit-emit` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `audit-chain`; owner `axis-audit-chain`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `audit-chain`.
- Capability records cited: `microservices/audit-chain/capabilities/audit-emit.yaml`, `microservices/audit-chain/capabilities/seal-mint.yaml`, `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- API surfaces cited: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar/policy artifacts cited: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`; +10 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar binding: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- State/event binding: `audit_chain.audit_chain`.
- Capability binding: `audit-emit`, `seal-mint`, `verify-merkle`.
- SLO binding: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`.
- Runbook binding: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `audit-chain`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `audit-chain`.
- `policy-engine` supplies the signed Cedar corpus while `audit-chain` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `audit-chain` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `audit-chain`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `audit-chain` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `audit-chain` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `audit-chain` is in annual full-scope pentest and every major `audit-emit` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`, `microservices/audit-chain/iac/helm/audit-storage/Chart.yaml`, `microservices/audit-chain/iac/helm/audit-storage/templates/deployment.yaml`, `microservices/audit-chain/iac/helm/audit-storage/templates/networkpolicy.yaml`; +13 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `audit-chain` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `audit-chain`; owner `axis-audit-chain`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `audit-chain`.
- Capability records cited: `microservices/audit-chain/capabilities/audit-emit.yaml`, `microservices/audit-chain/capabilities/seal-mint.yaml`, `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- API surfaces cited: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar/policy artifacts cited: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`; +10 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar binding: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- State/event binding: `audit_chain.audit_chain`.
- Capability binding: `audit-emit`, `seal-mint`, `verify-merkle`.
- SLO binding: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`.
- Runbook binding: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `audit-chain`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `audit-chain`.
- `policy-engine` supplies the signed Cedar corpus while `audit-chain` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `audit-chain` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `audit-chain`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `audit-chain` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `audit-chain` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `audit-chain` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `audit-emit` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `audit-chain`; owner `axis-audit-chain`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `audit-chain`.
- Capability records cited: `microservices/audit-chain/capabilities/audit-emit.yaml`, `microservices/audit-chain/capabilities/seal-mint.yaml`, `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- API surfaces cited: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar/policy artifacts cited: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`; +10 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar binding: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- State/event binding: `audit_chain.audit_chain`.
- Capability binding: `audit-emit`, `seal-mint`, `verify-merkle`.
- SLO binding: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`.
- Runbook binding: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `audit-chain`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `audit-chain`.
- `policy-engine` supplies the signed Cedar corpus while `audit-chain` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `audit-chain` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `audit-chain`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `audit-chain` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `audit-chain` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `audit-chain` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/audit-chain/catalog/oya-audit-chain-emission-adapter.yaml`, `microservices/audit-chain/catalog/oya-audit-chain-emission-api.yaml`, `microservices/audit-chain/catalog/oya-audit-chain-emission-app.yaml`, `microservices/audit-chain/catalog/oya-audit-chain-emission-domain.yaml`, `microservices/audit-chain/catalog/oya-audit-chain-emission-kernel.yaml`, `microservices/audit-chain/catalog/oya-audit-chain-emission-rest.yaml`; +19 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `audit-emit` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `audit-chain`; owner `axis-audit-chain`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `audit-chain`.
- Capability records cited: `microservices/audit-chain/capabilities/audit-emit.yaml`, `microservices/audit-chain/capabilities/seal-mint.yaml`, `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- API surfaces cited: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar/policy artifacts cited: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`; +10 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar binding: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- State/event binding: `audit_chain.audit_chain`.
- Capability binding: `audit-emit`, `seal-mint`, `verify-merkle`.
- SLO binding: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`.
- Runbook binding: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `audit-chain`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `audit-chain`.
- `policy-engine` supplies the signed Cedar corpus while `audit-chain` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `audit-chain` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `audit-chain`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `audit-chain` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `audit-chain` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `audit-chain` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `audit-emit` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `audit-emit` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `audit-chain`; owner `axis-audit-chain`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `audit-chain`.
- Capability records cited: `microservices/audit-chain/capabilities/audit-emit.yaml`, `microservices/audit-chain/capabilities/seal-mint.yaml`, `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- API surfaces cited: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar/policy artifacts cited: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`; +10 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar binding: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- State/event binding: `audit_chain.audit_chain`.
- Capability binding: `audit-emit`, `seal-mint`, `verify-merkle`.
- SLO binding: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`.
- Runbook binding: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `audit-chain`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `audit-chain`.
- `policy-engine` supplies the signed Cedar corpus while `audit-chain` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `audit-chain` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `audit-chain`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `audit-chain` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `audit-chain` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.
- State/event surfaces carrying classification: `audit_chain.audit_chain`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `audit-emit` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `audit-chain`; owner `axis-audit-chain`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `audit-chain`.
- Capability records cited: `microservices/audit-chain/capabilities/audit-emit.yaml`, `microservices/audit-chain/capabilities/seal-mint.yaml`, `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- API surfaces cited: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar/policy artifacts cited: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`; +10 more.
- Data classes declared for this control: `AUDIT`, `INTERNAL_ONLY`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`, `microservices/audit-chain/contracts/openapi/audit-chain.yaml`, `microservices/audit-chain/contracts/proto/audit-chain.proto`.
- Cedar binding: `microservices/audit-chain/policy/auditor-scope.cedar`, `microservices/audit-chain/policy/ci-scope.cedar`, `microservices/audit-chain/policy/data-residency.md`, `microservices/audit-chain/policy/public-read.cedar`, `microservices/audit-chain/policy/seal-integrity.md`, `microservices/audit-chain/policy/tenant-scope.cedar`.
- State/event binding: `audit_chain.audit_chain`.
- Capability binding: `audit-emit`, `seal-mint`, `verify-merkle`.
- SLO binding: `microservices/audit-chain/slos/chain-of-custody-integrity-correctness.openslo.yaml`, `microservices/audit-chain/slos/evidence-export-freshness.openslo.yaml`, `microservices/audit-chain/slos/merkle-chain-verification-latency.openslo.yaml`, `microservices/audit-chain/slos/seal-storage-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-availability.openslo.yaml`, `microservices/audit-chain/slos/seal-write-latency.openslo.yaml`.
- Runbook binding: `microservices/audit-chain/runbooks/audit-chain-restart.md`, `microservices/audit-chain/runbooks/audit-export.md`, `microservices/audit-chain/runbooks/hsm-key-rotation.md`, `microservices/audit-chain/runbooks/merkle-seal-recovery.md`, `microservices/audit-chain/runbooks/retention-cascade.md`, `microservices/audit-chain/runbooks/signature-verification-failure.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `audit-chain`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `audit-chain`.
- `policy-engine` supplies the signed Cedar corpus while `audit-chain` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `audit-chain` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `audit-chain`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `audit-chain` applies the most restrictive policy and emits a degraded-mode audit event.
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

