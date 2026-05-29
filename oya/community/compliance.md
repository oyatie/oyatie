---
doc_class: Compliance
template_id: TPL-COMPLIANCE
microservice: community
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-community
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/community/threat-model.md
  - microservices/community/dpia.md
  - microservices/community/policy/community-isolation.md
  - microservices/community/policy/data-residency.md
doc_status: published
---

# Compliance: community µservice

## Section 230 + Similar Safe-Harbor Posture

oyatie operates the community µservice as an **interactive computer service provider** under 47 USC §230(c)(1) (US) and equivalent intermediary safe-harbor regimes per pack:

| Pack | Safe-harbor regime | Posture |
|---|---|---|
| pack-us | 47 USC §230(c)(1)+(c)(2) (CDA) | Provider; tenant is publisher; good-faith moderation under (c)(2)(A) |
| pack-eu | DSA 2022/2065 Arts. 4-8 (mere conduit / caching / hosting) | Hosting provider; notice-and-action under Art. 16; transparency reports |
| pack-kr | Telecommunications Business Act Art. 22-5 + Information Communications Network Act Art. 44 | Information-communications-service provider; notice-and-takedown 24 h |
| pack-jp | Provider Liability Limitation Act (2001) Arts. 3 + 4 | Provider; sender disclosure on court order only |
| pack-au | Online Safety Act 2021 + Broadcasting Services Act Schedule 5 | Industry code obligations; eSafety Commissioner notice response 24 h |
| pack-in | IT Rules 2021 (Intermediary Guidelines) Rules 3 + 4 | Significant social media intermediary thresholds; due diligence + grievance officer |
| pack-br | Marco Civil da Internet 2014 Arts. 18 + 19 | Court order required for content removal; user notice |
| pack-sg | Online Safety (Miscellaneous Amendments) Act 2022 + POFMA 2019 | Content provider; takedown directions response 24 h |

## HIPAA (pack-us-healthcare)

When PHI may be processed (tenant in healthcare vertical, BAA signed):

| Safeguard | Implementation |
|---|---|
| 45 CFR §164.308 Administrative | Role-based access (Cedar); workforce training; per-tenant BAA |
| 45 CFR §164.310 Physical | Cloud provider physical security (SOC 2); per-region segregation |
| 45 CFR §164.312 Technical | Encryption at rest (KMS); encryption in transit (mTLS); audit-chain |
| 45 CFR §164.314 Organizational | BAA with each sub-processor (S3, hosting) |
| 45 CFR §164.316 Policies + Procedures | This document; threat-model.md; dpia.md |

PHI in community: tenant opt-in only. Default warning surfaced: "do not post PHI". When opted in, Cedar entitlement `phi_eligible == true` required for post create; classifier alerts on suspected PHI in non-eligible spaces.

## GDPR

| Article | Implementation |
|---|---|
| Art. 5 (principles) | Lawful, fair, transparent; data minimisation; retention matrix |
| Art. 6 (lawfulness) | (1)(b) contract + (1)(f) legitimate interest (abuse prevention) |
| Art. 9 (special category) | Explicit consent for PHI under pack-us-healthcare |
| Art. 13/14 (notice) | Tenant-onboarding privacy notice; member onboarding consent |
| Art. 15 (access) | Tenant export; member self-service |
| Art. 17 (erasure) | DSR cascade runbook |
| Art. 22 (ADM) | Moderation is reversible; appeal; human-in-loop two-eyes for bans |
| Art. 25 (by design + default) | Per-tenant isolation; deny-by-default Cedar |
| Art. 28 (processor) | DPA with each tenant |
| Art. 30 (records of processing) | Audit-chain seals every event |
| Art. 32 (security) | mTLS + RLS + Cedar + audit-chain + DSR cascade |
| Art. 33/34 (breach) | 72 h authority notification; data subject notification when high risk |
| Art. 35 (DPIA) | dpia.md |

## KR PIPA (pack-kr)

| Article | Implementation |
|---|---|
| Art. 3 (principles) | Lawful processing; purpose limitation; data minimisation |
| Art. 15 (collection + use) | Tenant-onboarding consent; purpose declared |
| Art. 17 (provision to third parties) | Sub-processor list in Annex B of dpia.md |
| Art. 18 (purpose-limited use) | Cedar action scope |
| Art. 22-2 (children under 14) | Tenant opt-in flow; parental consent gate |
| Art. 23 (sensitive data) | PHI / political opinion / sexual orientation flagged; explicit consent |
| Art. 28 (cross-border transfer) | Default no transfer; opt-in flow per data-residency.md |
| Art. 29 (security) | mTLS + RLS + Cedar + audit-chain |
| Art. 33 (DPIA) | dpia.md; PIPC notification when threshold engaged |
| Art. 34 (breach) | 24 h authority notification |
| Art. 36 (DSR) | DSR cascade runbook; 10 d response |

## APPI (pack-jp)

Arts. 17/18/20/21/23/24 covered: purpose limitation; consent for special-category; cross-border disclosure under Art. 24.

## PDPA (pack-sg)

§§11-26: protection obligation; retention limitation; transfer limitation per data-residency.md.

## Privacy Act 1988 (pack-au)

APPs 1-13: especially APP 6 (use + disclosure), APP 8 (cross-border), APP 11 (security).

## DPDPA 2023 (pack-in)

§§6-10 (consent + notice + processing limits); §16 cross-border restrictions per data-residency.md.

## LGPD (pack-br)

Arts. 6/7/11/14/18/33/46/48: lawful processing; consent; cross-border per Art. 33.

## UAE PDPL / KSA PDPL

Local-only default; cross-border via authority approval.

## SOC 2 Type 2 Mapping

| TSC | Control | Evidence |
|---|---|---|
| CC6.1 (logical access) | tenancy JWT + Cedar | policy/*.cedar; audit-chain logs |
| CC6.2 (provisioning) | tenant onboarding workflow | tenancy µservice handoff |
| CC6.3 (auth) | OIDC + 2FA for admin/mod | tenancy posture |
| CC6.6 (system protection) | mTLS + WAF | iac/ overlays |
| CC7.1 (monitoring) | observability SLOs | slos/*.openslo.yaml |
| CC7.2 (anomaly detection) | foundry-guardrails | bridge integration |
| CC7.4 (incident) | incident-response.md | runbooks/ |
| CC8.1 (change mgmt) | branch-protection + CI gates | governance µservice |

## ISO 27001:2022 Mapping

Annex A controls per threat-model.md `enforced_frameworks`.

## Transparency Report (pack-eu DSA + similar)

Quarterly: per-tenant moderation actions count; appeal outcomes; authority orders received; response times.

## Retention + Erasure Cadence

- Daily TTL job per retention matrix.
- DSR cascade as per `policy/data-residency.md`.
- Legal hold overrides retention; documented per-tenant.

## Audit Evidence Catalog

- `audit-chain` seals = primary evidence stream.
- Per-tenant audit log via `auditor-scope.cedar`.
- Cedar fragment coverage report (CI artifact).
- Penetration test report (annual).
- DPIA review minutes (annual).
- Sub-processor list + DPAs.

---



## §day-one-cert-readiness
This anchor is closed for `community` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `community` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +10 more.
- Example: `moderate-action` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more without changing domain code.
- Data classes under pack control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `moderate-action` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against ADR-0292 §D-1: minor-user refusal, teen tier and age-verification handling.

### Service-specific answer
- Minor exposure for `community` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `moderate-action` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen-tier activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `community` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`, `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`; +15 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `moderate-action` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.community.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `community` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `moderate-action` touches those data classes.
- Signal sources: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +14 more.
- Example event class: `oya.community.moderate.action.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against documentation-rigor.md §3.2.6.E: model inventory, retrain cadence and promotion gates.

### Service-specific answer
- Local ML posture: `True` for direct model use; inherited detection/intelligence models still require versioned consumption evidence.
- Model inventory key: `manifest.json:ml_models` or the Intelligence audience tag `community.moderate-action` if models are substrate-hosted.
- Promotion gates: offline eval, bias/fairness report, drift threshold, SLO budget, rollback model id, and human approval for high-risk/adverse-action paths.
- Retraining cadence is model-specific; high-risk models require documented data cut, feature schema, holdout set, and pack-specific legal review.
- Example: `moderate-action` model output is never the sole authority for a legal/financial/employment/minor-impacting decision; Cedar and human-review policies remain in control.
- Deprecated model versions sunset under ADR-0258 with traffic split, canary, rollback, and post-promotion audit.
- Model cards include intended use, non-use, data provenance, performance by segment, failure modes, and owner.
- Services without local models keep this as a negative declaration so future agents cannot silently add ML without the lifecycle gate.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: NIST AI RMF model-governance lifecycle is the reference pattern for the control shape described here.
- Precedent 2: Google Model Cards is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against documentation-rigor.md §3.2.6.E: fairness metrics, thresholds and disaggregated false-positive audit.

### Service-specific answer
- Fairness audit applies to `community` risk/detection decisions that affect access, ranking, safety, money, employment, health, or protected classes.
- Metrics: false-positive rate ratio, false-negative rate ratio, calibration by segment, equalized-odds gap, appeal overturn rate, and challenge-friction rate.
- Thresholds: no protected segment exceeds 1.25x baseline false-positive rate without documented mitigation and human review.
- Segments are derived from lawful, minimized attributes; `community` never stores protected attributes solely to make a product feature easier.
- Example: `moderate-action` abuse/risk score challenge rate is compared across locale, accessibility profile, age tier, and jurisdiction pack.
- Audit cadence: every model/rule promotion, quarterly for active high-risk detectors, and after any SEV involving false positives.
- Fairness reports are retained in audit evidence; raw protected-attribute joins remain in restricted analytics cells.
- If the service has no ML, deterministic rules still get false-positive and appeal-rate monitoring.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Microsoft Fairlearn audit pattern is the reference pattern for the control shape described here.
- Precedent 2: NIST AI RMF measurement function is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `community` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.community.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `moderate-action` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `moderate-action` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `community` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`, `community.community`.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `community.community` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `community` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +16 more.
- Example: `moderate-action` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.community` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/community/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.
- Example: `moderate-action` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `community` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`, `microservices/community/iac/helm/community/Chart.yaml`, `microservices/community/iac/helm/community/templates/deployment.yaml`, `microservices/community/iac/helm/community/templates/hpa.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `moderate-action` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `community` is in annual full-scope pentest and every major `moderate-action` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`, `microservices/community/iac/helm/community/Chart.yaml`, `microservices/community/iac/helm/community/templates/deployment.yaml`, `microservices/community/iac/helm/community/templates/hpa.yaml`; +19 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `community` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `community` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `moderate-action` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `community` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/community/catalog/oya-community-kb-article-store-adapter-postgres.yaml`, `microservices/community/catalog/oya-community-kb-article-store-adapter-s3.yaml`, `microservices/community/catalog/oya-community-kb-article-store-adapter.yaml`, `microservices/community/catalog/oya-community-kb-article-store-api.yaml`, `microservices/community/catalog/oya-community-kb-article-store-app.yaml`, `microservices/community/catalog/oya-community-kb-article-store-domain.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `moderate-action` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `community` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `moderate-action` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `moderate-action` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `community` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.
- State/event surfaces carrying classification: `community.community`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `moderate-action` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `community`; owner `axis-community`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `community`.
- Capability records cited: `microservices/community/capabilities/bug-bounty-submission.yaml`, `microservices/community/capabilities/handshake-mode.yaml`, `microservices/community/capabilities/linkedin-mode.yaml`, `microservices/community/capabilities/moderate-action.yaml`, `microservices/community/capabilities/post-create.yaml`, `microservices/community/capabilities/reddit-mode.yaml`; +4 more.
- API surfaces cited: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar/policy artifacts cited: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`; +12 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/community/contracts/asyncapi/community-events.yaml`, `microservices/community/contracts/openapi/community.yaml`, `microservices/community/contracts/proto/community.proto`.
- Cedar binding: `microservices/community/policy/anonymity-mode-fully-anonymous.cedar`, `microservices/community/policy/anonymity-mode-identity-anchored.cedar`, `microservices/community/policy/anonymity-mode-persona-anchored.cedar`, `microservices/community/policy/anonymity-mode-pseudonymous.cedar`, `microservices/community/policy/auditor-scope.cedar`, `microservices/community/policy/ci-scope.cedar`; +4 more.
- State/event binding: `community.community`.
- Capability binding: `moderate-action`, `post-create`, `vote-cast`.
- SLO binding: `microservices/community/slos/audit-chain-seal-latency.openslo.yaml`, `microservices/community/slos/feed-render-latency.openslo.yaml`, `microservices/community/slos/kb-article-publish-latency.openslo.yaml`, `microservices/community/slos/moderation-action-latency.openslo.yaml`, `microservices/community/slos/post-create-latency.openslo.yaml`, `microservices/community/slos/search-query-latency.openslo.yaml`; +1 more.
- Runbook binding: `microservices/community/runbooks/kb-attachment-restore.md`, `microservices/community/runbooks/moderation-queue-clear.md`, `microservices/community/runbooks/post-mass-deletion.md`, `microservices/community/runbooks/search-rebuild.md`, `microservices/community/runbooks/spam-flood-throttle.md`, `microservices/community/runbooks/vote-anomaly.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `community`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `community`.
- `policy-engine` supplies the signed Cedar corpus while `community` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `community` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `community`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `community` applies the most restrictive policy and emits a degraded-mode audit event.
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

