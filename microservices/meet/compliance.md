---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-meet, council-architecture, ops-compliance
related_adrs: [ADR-0008, ADR-0028, ADR-0117, ADR-0123, ADR-0135, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/meet/threat-model.md
  - microservices/meet/dpia.md
  - microservices/meet/policy/data-residency.md
  - microservices/meet/policy/recording-consent.md
  - microservices/meet/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (meet µservice)

## Purpose

Canonical control-to-framework mapping for the meet µservice. Tells an external auditor (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / KR PIPC / HIPAA OCR / SEC / FINRA / MiFID II reviewer / EU AI Act conformity assessor) exactly which control implementation satisfies which framework clause, with pointers to the evidence artifact.

## Enforced Frameworks (every pack)

### SOC 2 Type 2 (2017 TSC + 2022 PoF)

| TSC | Objective | Implementation | Evidence |
|---|---|---|---|
| CC1.1 | Integrity + ethical values | Code-of-conduct + signed commits | `docs/standards/code-review.md` + branch-protection.yaml |
| CC1.5 | Accountability for performance | Per-µservice SLO + on-call | `slos/*.openslo.yaml` + `incident-response.md` |
| CC3.1 | Risk identification | Threat model + DPIA + risk register | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to objectives | Multi-spectrum review per ADR + IP | `evidence/multispectrum/` |
| CC3.3 | Risk of fraud | Audit-chain Ed25519 seals; four-eyes recording disclosure | `policy/recording-consent.md` |
| CC4.1 | Internal monitoring | LEAN CI lanes + per-µservice SLO | `/specs/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain on every state transition | ADR-0028 + audit-chain µservice |
| CC6.1 | Logical + physical access | OIDC + MFA + Cedar + JIT | `policy/*.cedar` |
| CC6.2 | Authn + authz | Per-tenant API keys + SPIFFE + LiveKit JWT scope | `policy/tenant-scope.cedar` |
| CC6.3 | Access lifecycle | OpenBao adds/removes + audit | OpenBao audit log |
| CC6.6 | Logical access controls | Postgres RLS + Cedar + Lobby evaluation | `threat-model.md` T-I-01 mitigation |
| CC6.7 | Transmission + disposal | mTLS + SRTP in transit + KMS at rest + DSR cascade | `policy/data-residency.md` §DSR |
| CC6.8 | Vulnerability management | `cargo deny` + Trivy + Grype; weekly CVE; ffmpeg + LiveKit + Whisper CVE feed | `/specs/supply-chain.json` |
| CC7.1 | System operations | HA Postgres + per-tenant rate limits + HPA + LiveKit StatefulSet | `capacity-model.md` |
| CC7.2 | Monitoring inputs | Self-observability via observability µservice | `slos/` + `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate alerts + cardinality alerts | OpenSLO manifests |
| CC7.4 | Incident response | Severity-classified response + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates | observability promotion gate per ADR-0139 |
| CC9.1 | Risk mitigation | Multi-region + DR + automated rollback | `multi-region.md` |
| CC9.2 | Vendor risk | Sub-processor list + per-vendor DPA (LiveKit/Whisper/SRS/OBS) | `legal/sub-processors.md` |

**Privacy Criteria (P1–P8):**

| P# | Criterion | Implementation |
|---|---|---|
| P1 | Notice + privacy practices | DPA template + tenant onboarding notice + in-meeting recording banner |
| P2 | Choice + consent | Recording-consent modal + per-participant AI-summary opt-in + E2E opt-in tier |
| P3 | Collection | OTel SDK PII redactor + `data_class` annotation |
| P4 | Use, retention, disposal | Retention matrix in `policy/data-residency.md`; DSR cascade with face-blur/voice-mask |
| P5 | Access | Tenant operators read own data; participants read their own attendance |
| P6 | Disclosure to third parties | Sub-processor list + transfer register + RTMP-egress allow-list |
| P7 | Quality | Audit-chain integrity + four-eyes recording disclosure |
| P8 | Monitoring + enforcement | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Threat-model review cadence; threat-intel feeds | `threat-model.md` |
| A.5.10 | Acceptable use | Internal AUP + onboarding | `docs/standards/onboarding.md` |
| A.5.14 | Info transfer | mTLS + SRTP + KMS + signed audit-chain | `threat-model.md` Trust Boundary 3 |
| A.5.15 | Access control | Cedar fragments + OIDC + MFA + LiveKit token scope | `policy/*.cedar` |
| A.5.17 | Authentication info | OpenBao secret lifecycle + rotation | OpenBao audit log |
| A.5.18 | Access rights | Per-meeting ACL + four-eyes recording disclosure | `policy/tenant-scope.cedar` |
| A.5.23 | Cloud-service security | Multi-region + DR posture | `multi-region.md` |
| A.5.26 | Incident response | Severity-classified IR; postmortems | `incident-response.md` |
| A.5.30 | ICT readiness for BCDR | DR pair + RPO/RTO targets | `multi-region.md` |
| A.5.31 | Legal + statutory | Per-pack regulatory cross-mapping below | this doc |
| A.5.34 | Privacy + PII protection | Data-class taxonomy + DSR cascade + four-eyes | `policy/data-residency.md` §DSR |
| A.8.2 | Privileged access rights | JIT elevation; two-person rule for admin ops | OpenBao audit |
| A.8.3 | Info access restriction | Cedar + RLS + per-tenant key bindings | `threat-model.md` T-S-01 mitigation |
| A.8.5 | Secure authentication | OIDC + MFA; mTLS internal; LiveKit JWT verify Ed25519 | `policy/tenant-scope.cedar` |
| A.8.7 | Protection against malware | ffmpeg gVisor sandbox; image-scan on every release | `runbooks/sfu-degraded.md` |
| A.8.11 | Data masking | Span redactor; transcript redactor; search-result Cedar filter | `policy/redaction-phi.md` (pack-us-healthcare) |
| A.8.12 | Data leakage prevention | DLP via PII detectors + cardinality limits + egress NetworkPolicy + RTMP allow-list | `threat-model.md` T-I-08 mitigation |
| A.8.20 | Networks security | Service mesh + mTLS + NetworkPolicy | k8s NetworkPolicy review |
| A.8.21 | Network services | TLS termination + WAF + DDoS + WebRTC ICE | ingress configuration |
| A.8.23 | Web filtering | n/a (server-side service) | – |
| A.8.25 | Secure development lifecycle | LEAN gates + multispectrum review | `evidence/multispectrum/` |
| A.8.27 | Application security | OWASP API Top 10 + OWASP ASVS v4 level 2; cargo audit | `threat-model.md` |
| A.8.28 | Secure coding | `cargo clippy -- -D warnings` + `cargo deny` | CI lanes |
| A.8.32 | Change management | PR + LEAN + branch-protection | branch-protection.yaml |
| A.8.34 | Audit findings remediation | Audit-finding tracker per engagement | ops-compliance |

### GDPR

| Article | Implementation | Evidence |
|---|---|---|
| Art. 5 (principles) | Data-class taxonomy + minimisation + retention | `policy/data-residency.md` |
| Art. 6 (lawful basis) | Per-class lawful-basis declared in `dpia.md` §2.2 | `dpia.md` |
| Art. 9 (special-category) | Pack-us-healthcare BAA + KR PIPA Art. 23 consent + biometric Art. 9(2)(a) | `legal/baa-template.md` |
| Art. 13/14 (transparency) | Tenant onboarding notice; recording-consent modal; joint-controllership clause | `legal/dpa-template.md` + `policy/recording-consent.md` |
| Art. 17 (erasure) | DSR cascade with face-blur/voice-mask | `policy/data-residency.md` §DSR |
| Art. 22 (automated decisions) | AI summary classified non-binding; participant opt-out; meaningful information per Art. 22(3) | ADR-MEET-0006 |
| Art. 25 (privacy-by-design) | Recording-consent invariant; redactor; Cedar; E2E mode | `policy/recording-consent.md` |
| Art. 28 (processor) | Per-tenant DPA | `legal/dpa-template.md` |
| Art. 30 (records of processing) | Audit-chain ledger | audit-chain µservice |
| Art. 32 (security) | Every mitigation in `threat-model.md` | `threat-model.md` |
| Art. 33 (breach notification) | IR playbook; 72h GDPR clock | `incident-response.md` |
| Art. 35 (DPIA) | This DPIA satisfies | `dpia.md` |
| Art. 44–50 (transfers) | Pack-pinning; SCC required for cross-border | `policy/data-residency.md` |

### EU AI Act (Regulation 2024/1689)

| Article | Implementation | Evidence |
|---|---|---|
| Art. 13 (transparency) | Captions/summary labelled "AI-generated"; system-card-equivalent published | `capabilities/T1-assist.yaml` + ADR-MEET-0006 |
| Art. 50 (AI-generated content) | Per-output labelling; tenant-admin must surface in privacy notice | `capabilities/*.yaml` |
| Risk-class: minimal-risk (T0 suggest), low-risk (transcription, summary), medium-risk (live translation across language barriers per Art. 50(4)) | per-capability classification | ADR-MEET-0006 |
| Art. 26 (deployer obligations) | Tenant attests deployment-context at first-enable | `legal/dpa-template.md` |

### SEC Rule 17a-4(f) + FINRA Rule 4511 + SEC Rule 17a-3 (pack-us-financial)

| Clause | Implementation |
|---|---|
| 17a-4(f) WORM | S3 Object Lock on recording bucket; content_hash sealed; 3-7y retention |
| 17a-4(f) tamper-evident | Audit-chain Ed25519 over recording_id + content_hash |
| 17a-4(b)(4) easily-accessible | Recording manifest queryable by message_id + date + participant |
| FINRA 4511 supervisory review | Four-eyes disclosure path; supervisor entitlement Cedar-gated |
| SEC 17a-3 book-and-records | Per-meeting attendance log + recording manifest preserved |

### MiFID II (pack-eu — investment firm communications)

| Clause | Implementation |
|---|---|
| Art. 16(7) recording of telephone + electronic comms | Recording enabled for tenant-attested investment-firm; 5-7y retention floor |
| RTS 6 tamper-evident | Audit-chain Ed25519 + content_hash |
| Art. 35 audit access | Auditor scope Cedar policy time-boxed |

### HIPAA (pack-us-healthcare)

| HIPAA clause | Implementation |
|---|---|
| §164.308(a)(1)(ii)(A) (risk analysis) | DPIA + threat-model |
| §164.308(a)(4)(ii)(B) (access authorization) | Cedar + OIDC + MFA |
| §164.310 (physical safeguards) | OCI-managed datacenter (BAA-eligible) |
| §164.312(a)(1) (access control) | Cedar + Postgres RLS + four-eyes |
| §164.312(b) (audit controls) | Audit-chain ≥ 6y retention |
| §164.312(c)(1) (integrity) | content-hash + audit-chain |
| §164.312(e)(1) (transmission security) | mTLS + SRTP + KMS |
| §164.502(b) (minimum-necessary) | Transcript redactor + search redaction |
| §164.514 (de-identification) | Safe Harbour 18-identifier redactor |
| BAA template | `legal/baa-template.md` |

### KR PIPA (pack-kr)

| KR clause | Implementation |
|---|---|
| KR PIPA Art. 15 (recording consent) | Modal consent banner at join + audit-chain participant_consent_acknowledged |
| KR PIPA Art. 17 (cross-border consent) | Pack-pinning; cross-border requires explicit consent |
| KR PIPA Art. 22-2 (sensitive consent) | pack-kr sensitive recordings require additional consent flow |
| KR PIPA Art. 23 (sensitive data) | Encryption + Cedar entitlement + four-eyes for disclosure |
| KR PIPA Art. 28 (processor) | Tenant DPA |
| KR PIPA Art. 29 (technical safeguards) | All `threat-model.md` mitigations map to Art. 29 controls |
| KR PIPA Art. 29-2 (KR-specific) | Audit log retention ≥ 1 year |
| KR-ISMS-P §2.5 (personnel) | Two-person rule + JIT elevation |
| KR-ISMS-P §2.7 (access control) | Cedar |
| KR 정보통신망법 §49 (intercept) | Server-side admin recording-disclosure only via four-eyes |
| KR 전자문서법 Art. 5 (integrity) | Audit-chain Ed25519 seal |

### pack-eu additional (ePrivacy + eIDAS + AVMS Directive)

| Clause | Implementation |
|---|---|
| ePrivacy Directive 2002/58/EC Art. 5 | Confidentiality of communications via Cedar + RLS + E2E mode |
| ePrivacy Directive Art. 5(3) | Embedded analytics gated; default off |
| eIDAS 910/2014 | Ed25519 audit-chain seals = AdES on transcripts |
| AVMS Directive 2010/13/EU | When meet hosts public webinar broadcasts as AV-media-on-demand, content classification minima apply |
| NIS2 2022/2555 (when thresholds engaged) | IR playbook 24h/72h/1mo timelines |

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per pack-overlay `regional-packs/<pack>/meet-compliance-overlay.md`.

## Continuous Compliance Evidence

CI lane `oya-governance-compliance-evidence-recency --microservice meet` evaluates every 24h:

- All policy/*.cedar files lint clean.
- All Helm charts pass `helm lint`.
- All OpenSLO manifests pass schema validation.
- All runbooks have a `last_drill_date` within 90 days.
- All threat-model rows have a re-review date within 90 days for residual ≥ M.
- All DPIA rows have a re-review date within 365 days.
- Per-tenant DPA + BAA signed status reflected in compliance dashboard.
- EU AI Act per-capability risk-class document present and current per ADR-MEET-0006.

Output: `microservices/meet/evidence/compliance-evidence-<unix_ts>.json`.

## References

- `microservices/meet/threat-model.md`.
- `microservices/meet/dpia.md`.
- `microservices/meet/policy/data-residency.md`.
- `microservices/meet/policy/recording-consent.md`.
- `microservices/observability/compliance.md` (shape reference).
- ADR-0028 (Bominal) + ADR-0008 + ADR-0135 + ADR-0139 + ADR-0131.
- ADR-MEET-0001..0006.
- Standards cited inline above.

---



## §day-one-cert-readiness
This anchor is closed for `meet` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `meet` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +15 more.
- Example: `meeting-topic-and-participant-hint` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `meet` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `meeting-topic-and-participant-hint` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `meet` against ADR-0292 §D-1: minor-user refusal, teen tier and age-verification handling.

### Service-specific answer
- Minor exposure for `meet` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `meeting-topic-and-participant-hint` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen-tier activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `meet` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `meet` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`, `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`; +16 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `meeting-topic-and-participant-hint` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.meet.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `meet` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `meet` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `meeting-topic-and-participant-hint` touches those data classes.
- Signal sources: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +16 more.
- Example event class: `oya.meet.meeting.topic.and.participant.hint.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `meet` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `meet` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.meet.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `meeting-topic-and-participant-hint` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `meeting-topic-and-participant-hint` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `meet` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `meet` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`, `meet.meet`.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `meet.meet` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `meet` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `meet` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +14 more.
- Example: `meeting-topic-and-participant-hint` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `meet` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.meet` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/meet/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.
- Example: `meeting-topic-and-participant-hint` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `meet` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `meet` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`, `microservices/meet/iac/helm/meet/Chart.yaml`, `microservices/meet/iac/helm/meet/templates/deployment.yaml`, `microservices/meet/iac/helm/meet/templates/hpa.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `meeting-topic-and-participant-hint` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `meet` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `meet` is in annual full-scope pentest and every major `meeting-topic-and-participant-hint` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`, `microservices/meet/iac/helm/meet/Chart.yaml`, `microservices/meet/iac/helm/meet/templates/deployment.yaml`, `microservices/meet/iac/helm/meet/templates/hpa.yaml`; +17 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `meet` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `meet` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `meet` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `meeting-topic-and-participant-hint` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `meet` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `meet` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/meet/catalog/oya-meet-audio-kernel.yaml`, `microservices/meet/catalog/oya-meet-e2e-encryption-kernel.yaml`, `microservices/meet/catalog/oya-meet-live-stream-egress-adapter-srs-rtmp.yaml`, `microservices/meet/catalog/oya-meet-meeting-instance-adapter-coturn.yaml`, `microservices/meet/catalog/oya-meet-meeting-instance-adapter-livekit.yaml`, `microservices/meet/catalog/oya-meet-meeting-room-adapter-postgres.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `meeting-topic-and-participant-hint` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `meet` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `meet` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `meeting-topic-and-participant-hint` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `meeting-topic-and-participant-hint` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `meet` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `meet.meet`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `meeting-topic-and-participant-hint` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `meet`; owner `axis-meet`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `meet`.
- Capability records cited: `microservices/meet/capabilities/T0-suggest.yaml`, `microservices/meet/capabilities/T1-assist.yaml`, `microservices/meet/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar/policy artifacts cited: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- SLO and dashboard evidence: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/meet/contracts/asyncapi/meet-events.yaml`, `microservices/meet/contracts/openapi/meet.yaml`, `microservices/meet/contracts/proto/meet.proto`.
- Cedar binding: `microservices/meet/policy/auditor-scope.cedar`, `microservices/meet/policy/ci-scope.cedar`, `microservices/meet/policy/data-residency.md`, `microservices/meet/policy/meeting-scope.cedar`, `microservices/meet/policy/public-read.cedar`, `microservices/meet/policy/recording-consent.md`; +2 more.
- State/event binding: `meet.meet`.
- Capability binding: `meeting-topic-and-participant-hint`, `live-captions-transcription-summary`, `auto-mute-noise-detect-and-auto-translate`.
- SLO binding: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/e2e-mls-handshake-latency.openslo.yaml`, `microservices/meet/slos/live-caption-latency.openslo.yaml`, `microservices/meet/slos/media-glass-to-glass-latency.openslo.yaml`, `microservices/meet/slos/meeting-summary-post-end-latency.openslo.yaml`, `microservices/meet/slos/participant-join-latency.openslo.yaml`; +5 more.
- Runbook binding: `microservices/meet/runbooks/coturn-key-rotation.md`, `microservices/meet/runbooks/live-caption-stalled.md`, `microservices/meet/runbooks/lobby-bypass-incident.md`, `microservices/meet/runbooks/recording-storage-degraded.md`, `microservices/meet/runbooks/sfu-degraded.md`, `microservices/meet/runbooks/transcription-classifier-rollback.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `meet`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `meet`.
- `policy-engine` supplies the signed Cedar corpus while `meet` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `meet` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `meet`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `meet` applies the most restrictive policy and emits a degraded-mode audit event.
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

