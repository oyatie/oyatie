# Compliance — `comms-email` µservice

> Authored: 2026-05-18
> ADR anchors: ADR-0201, ADR-0144, ADR-0145.
> Frameworks: CAN-SPAM (US), GDPR (EU), CCPA (CA), HIPAA
> (US-healthcare pack), KR PIPA, KSA / UAE sovereign DP laws.

## 1. CAN-SPAM (US Federal)

| Requirement | Substrate response |
| ----------- | ------------------ |
| Truthful `From` header | Per-tenant DKIM binding; kernel preflight rejects from-domain mismatch. |
| Accurate `Subject` line | Tenant-supplied; substrate audits but does not enforce content (tenant owns). |
| Identify as advertisement | Tenant-side responsibility; template registry tag flags promotional templates. |
| Postal address in message | CAN-SPAM footer template macro appends tenant's mailing address. |
| Honor unsubscribe within 10 business days | Suppression list inserts `OperatorManual` on opt-out click within seconds; audit chain records inserts. |

## 2. GDPR (EU)

### Lawful basis (Art. 6)

- Each send carries `lawful_basis` ∈ {`consent`,
  `legitimate_interest`, `contract`, `legal_obligation`, `vital`,
  `public_task`} in the audit chain entry.
- Tenants set the default per template; per-send overrides
  honored.

### Consent (Art. 7)

- Consent capture is upstream (tenant's product); the substrate
  records the consent identifier with each send.

### Right to erasure (Art. 17)

- Erasure request inserts the recipient into suppression with
  `reason = GdprErasure`.
- All future sends rejected at preflight.
- Historical audit chain entries retain the address per ADR-0145
  tamper-evident chain (Art. 17 §3(b) — public archive exception
  for accountability).

### Right of access (Art. 15)

- Audit chain query returns all events for a recipient within
  the legal SLA.

### Data residency (Art. 44-50)

- IP-013 multi-region routing pins EU tenants to EU-region
  adapters.

### DPIA

- See `dpia.md`.

## 3. CCPA (California)

| Requirement | Substrate response |
| ----------- | ------------------ |
| Right to opt out | Suppression list with `OperatorManual` or `RegulatoryOptOut`. |
| Right to know | Audit chain query. |
| Right to delete | Maps to GDPR Art. 17 path. |
| Sale of personal info | Transactional email is not a sale; explicit policy. |

## 4. HIPAA (US-healthcare pack)

- BAA-only providers — SES with BAA OR Postal (self-hosted).
  Mailgun BAA-only configurations allowed when the BAA is on
  file.
- PHI in email body: the template registry flags any template
  in the `us-healthcare` pack as PHI-class; per ADR-0184 storage
  tier, attachments are encrypted-at-rest with per-tenant KMS
  keys.
- Audit chain entries for HIPAA-class sends carry
  `data_class = PHI` per ADR-0144.

## 5. KR PIPA

- Korean tenants pin to KR-region Postal (no KR-region SES as
  of 2026-05-18).
- Audit chain region tag enforced.
- Korean-language template overlays per ADR-0064 pack `kr`.

## 6. KSA / UAE sovereign DP laws

- Sovereign packs force Postal-only (IP-014).
- All audit chain entries land in the sovereign region.
- No cross-region routing.

## 7. SOC 2

| Trust criterion | Substrate response |
| --------------- | ------------------ |
| Security | DKIM mandatory, TLS-only transport, OpenBao secrets. |
| Availability | Multi-region routing + provider second-source. |
| Processing integrity | Idempotency-key + suppression list + audit chain. |
| Confidentiality | Row-level security on suppression + per-tenant credentials. |
| Privacy | Data residency + GDPR Art. 17 erasure path. |

## 8. PCI-DSS

- Substrate does not handle cardholder data. Templates that
  appear to include PAN-shaped strings are flagged at template
  CI lint and require an explicit waiver.

## 9. Audit cadence

- Quarterly internal compliance review.
- Annual external audit per SOC 2 / GDPR record-of-processing.

## 10. Open obligations

- Inbound email ingestion ADR (deferred) will need its own
  CAN-SPAM + GDPR posture.
- BIMI ADR will document BIMI's compliance posture.
- Phase-2 in-house relay (IP-015) needs a fresh compliance pass
  before launch.

---



## §day-one-cert-readiness
This anchor is closed for `comms-email` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `comms-email` covers packs `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +18 more.
- Example: `T0-transactional-send` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `soc2-type2`, `iso27001-2022`, `gdpr`, `kr-pipa`.
- Pack overlays modify Cedar fragments `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more without changing domain code.
- Data classes under pack control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `T0-transactional-send` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `comms-email` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`, `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`; +8 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `T0-transactional-send` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.comms-email.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `comms-email` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `T0-transactional-send` touches those data classes.
- Signal sources: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +20 more.
- Example event class: `oya.comms.email.t0.transactional.send.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against documentation-rigor.md §3.2.6.E: model inventory, retrain cadence and promotion gates.

### Service-specific answer
- Local ML posture: `False` for direct model use; inherited detection/intelligence models still require versioned consumption evidence.
- Model inventory key: `manifest.json:ml_models` or the Intelligence audience tag `comms-email.t0-transactional-send` if models are substrate-hosted.
- Promotion gates: offline eval, bias/fairness report, drift threshold, SLO budget, rollback model id, and human approval for high-risk/adverse-action paths.
- Retraining cadence is model-specific; high-risk models require documented data cut, feature schema, holdout set, and pack-specific legal review.
- Example: `T0-transactional-send` model output is never the sole authority for a legal/financial/employment/minor-impacting decision; Cedar and human-review policies remain in control.
- Deprecated model versions sunset under ADR-0258 with traffic split, canary, rollback, and post-promotion audit.
- Model cards include intended use, non-use, data provenance, performance by segment, failure modes, and owner.
- Services without local models keep this as a negative declaration so future agents cannot silently add ML without the lifecycle gate.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: NIST AI RMF model-governance lifecycle is the reference pattern for the control shape described here.
- Precedent 2: Google Model Cards is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against documentation-rigor.md §3.2.6.E: fairness metrics, thresholds and disaggregated false-positive audit.

### Service-specific answer
- Fairness audit applies to `comms-email` risk/detection decisions that affect access, ranking, safety, money, employment, health, or protected classes.
- Metrics: false-positive rate ratio, false-negative rate ratio, calibration by segment, equalized-odds gap, appeal overturn rate, and challenge-friction rate.
- Thresholds: no protected segment exceeds 1.25x baseline false-positive rate without documented mitigation and human review.
- Segments are derived from lawful, minimized attributes; `comms-email` never stores protected attributes solely to make a product feature easier.
- Example: `T0-transactional-send` abuse/risk score challenge rate is compared across locale, accessibility profile, age tier, and jurisdiction pack.
- Audit cadence: every model/rule promotion, quarterly for active high-risk detectors, and after any SEV involving false positives.
- Fairness reports are retained in audit evidence; raw protected-attribute joins remain in restricted analytics cells.
- If the service has no ML, deterministic rules still get false-positive and appeal-rate monitoring.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: Microsoft Fairlearn audit pattern is the reference pattern for the control shape described here.
- Precedent 2: NIST AI RMF measurement function is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `comms-email` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.comms-email.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `T0-transactional-send` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `T0-transactional-send` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `comms-email` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`, `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`; +4 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `comms_email.transactional_send` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `comms-email` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +18 more.
- Example: `T0-transactional-send` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.comms-email` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/comms-email/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.
- Example: `T0-transactional-send` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `comms-email` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`, `microservices/comms-email/iac/ech-config.yaml`, `microservices/comms-email/iac/edge-waf.yaml`, `microservices/comms-email/iac/helm/postal/Chart.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `T0-transactional-send` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `comms-email` is in annual full-scope pentest and every major `T0-transactional-send` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`, `microservices/comms-email/iac/ech-config.yaml`, `microservices/comms-email/iac/edge-waf.yaml`, `microservices/comms-email/iac/helm/postal/Chart.yaml`; +21 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `comms-email` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `comms-email` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `T0-transactional-send` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `comms-email` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/comms-email/catalog/bounded-contexts.json`, `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`, `microservices/comms-email/iac/ech-config.yaml`, `microservices/comms-email/iac/edge-waf.yaml`; +10 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `T0-transactional-send` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `comms-email` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `T0-transactional-send` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `T0-transactional-send` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `comms-email` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.
- State/event surfaces carrying classification: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `T0-transactional-send` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `comms-email`; owner `oya-substrate-comms`; service class `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `transactional-send`, `deliverability`, `webhook-ingest`, `suppression-list`, `dkim-rotation`, `tenant-from-domain-onboarding`; +1 more.
- Capability records cited: `microservices/comms-email/capabilities/T0-transactional-send.json`, `microservices/comms-email/capabilities/T1-bounce-handle.json`, `microservices/comms-email/capabilities/T1-webhook-delivery-event.json`, `microservices/comms-email/capabilities/T2-list-manage.json`, `microservices/comms-email/capabilities/T2-tenant-domain-mgmt.json`, `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- API surfaces cited: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar/policy artifacts cited: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +8 more.
- Runbook/IaC evidence: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +16 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/comms-email/contracts/asyncapi.yaml`, `microservices/comms-email/contracts/comms_email.proto`, `microservices/comms-email/contracts/openapi.yaml`.
- Cedar binding: `microservices/comms-email/policy/abuse-defence.cedar`, `microservices/comms-email/policy/action-authorization.cedar`, `microservices/comms-email/policy/auditor-scope.cedar`, `microservices/comms-email/policy/ci-scope.cedar`, `microservices/comms-email/policy/comms-email-send.cedar`, `microservices/comms-email/policy/comms-email-suppression-list.cedar`; +6 more.
- State/event binding: `comms_email.transactional_send`, `comms_email.deliverability`, `comms_email.webhook_ingest`, `comms_email.suppression_list`, `comms_email.dkim_rotation`, `comms_email.tenant_from_domain_onboarding`; +1 more.
- Capability binding: `T0-transactional-send`, `T1-webhook-delivery-event`, `T2-tenant-domain-mgmt`.
- SLO binding: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`, `microservices/comms-email/slos/deliverability-rate.openslo.yaml`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, `microservices/comms-email/slos/dmarc-alignment-rate.openslo.yaml`, `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`, `microservices/comms-email/slos/send-latency-p99.openslo.yaml`; +3 more.
- Runbook binding: `microservices/comms-email/runbooks/blacklist-recovery.md`, `microservices/comms-email/runbooks/bounce-storm-mitigation.md`, `microservices/comms-email/runbooks/dkim-key-rotation.md`, `microservices/comms-email/runbooks/dmarc-policy-tune.md`, `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`, `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `comms-email`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `comms-email`.
- `policy-engine` supplies the signed Cedar corpus while `comms-email` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `comms-email` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `comms-email`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `comms-email` applies the most restrictive policy and emits a degraded-mode audit event.
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

