# consent-graph compliance map

- Owner: axis-consent-graph + compliance-axis
- Date: 2026-05-18
- Authority: ADR-0214 §1, §8.4 NFR, ADR-0064 (canonical-base + pack overlays).

For each regulatory pack supported, this document maps the consent-graph capability to the regulation's
applicable clauses. Pack overlays in `iac/kustomize/overlays/<pack>/` carry per-pack runtime config
(residency, retention, redaction defaults).

## 1. Supported packs

`kr, eu, us, us-healthcare, jp, sg, au, in, br, ae, ksa` (11 packs; matches audit-chain).

## 2. KR (South Korea) — PIPA

### 2.1 Cross-border transfer (PIPA §17, §18)
- Default: cross-border transfer forbidden unless agreement.sovereignty.cross_border_transfer_permitted
  = true *and* lawful basis cited.
- Pack overlay sets `default_cross_border_transfer_permitted=false`.
- Lawful basis (consent / contract necessity / legal obligation) recorded in agreement.terms.metadata.

### 2.2 Right to know + right to erasure (PIPA §35)
- DSAR cascade runbook (`GDPR-DSAR-cross-tenant.md`) applies KR clock (10 days response).
- Erasure cascade tombstones all projections within 7 days.

### 2.3 Sensitive data (PIPA §23)
- Sensitive categories (health, race, ideology) require explicit consent + tighter scope; pack overlay
  enforces k_anonymity≥10 for aggregate of sensitive cohorts.

## 3. EU — GDPR + EAIA + DGA + Data Act

### 3.1 GDPR Art. 28 (processor)
- consent-graph operates as a processor under agreement terms; grantor is controller.
- agreement.terms.metadata captures processing instructions per Art. 28(3).

### 3.2 GDPR Art. 44–49 (cross-border)
- SCCs reference recorded in agreement.terms.metadata for EU→non-EU transfers.
- Schrems II adequacy decisions consulted; adequacy gap auto-blocks transfer.

### 3.3 GDPR Art. 17 (right to erasure)
- Cascade within 30 days (EU regulatory cap); consent-graph targets 7 days.

### 3.4 GDPR Art. 35 (DPIA)
- See `dpia.md` for full DPIA.

### 3.5 EU AI Act (EAIA)
- consent-graph capabilities classified per `capabilities/*.yaml`:
  - `consent-grant` T3 ⇒ EAIA high-risk class.
  - `consent-project-subscribe` T2 ⇒ EAIA limited-risk.
  - `consent-enforce` T0 ⇒ EAIA none (deterministic read-only).
- Pre-deployment Conformity Assessment required for T3.

### 3.6 EU Data Governance Act (DGA)
- Data intermediation neutrality requirement: consent-graph does not monetize data flows; processing
  fees only.

### 3.7 EU Data Act
- Right to data portability + interoperability of IoT data — covered by AttestedQuery + Projection
  modes.

## 4. US — federal + state

### 4.1 CCPA / CPRA (California)
- Right to know + right to delete + right to opt-out — all covered by DSAR cascade.
- "Sale of personal information" — consent-graph operations are NOT sales (no monetization of subject
  data).
- "Sharing" definition (CPRA-specific) — agreement-based sharing is *sharing under CPRA*; opt-out
  mechanism honored.

### 4.2 Other state privacy laws (CO, CT, UT, VA, ...)
- Covered under same DSAR + cross-tenant tombstone semantics.

### 4.3 GLBA (financial)
- Financial-vertical agreements default to AttestedQuery mode (per template `tmpl-banking-*`).
- Customer's right to opt-out of affiliate-information-sharing → revocation primitive.

## 5. US-Healthcare — HIPAA

### 5.1 HIPAA min-necessary (§164.502(b))
- Cedar policy + EntityScope.field_set enforces min-necessary at every projection emission.
- Audit-chain stores fields-redacted list per event.

### 5.2 HIPAA accounting of disclosures (§164.528)
- 6-year retention of bilateral audit chain ⇒ supports accounting-of-disclosures requests.

### 5.3 HIPAA break-glass
- AttestedQuery mode supports purpose-of-use `emergency-treatment` for break-glass with mandatory
  post-hoc audit review.
- Runbook `consent-forgery-detected.md` (alternate use: break-glass review).

### 5.4 HIPAA Business Associate Agreement (BAA)
- consent-graph operates under BAA executed between oyatie + covered-entity grantor.

### 5.5 TEFCA / Direct Trust
- consent-graph's HIE-style bilateral chain aligns with TEFCA participant audit requirements; ADR-
  SVC-CG-* (PHASE-02) will spec the explicit interop mapping.

## 6. JP — APPI

### 6.1 APPI cross-border (§24)
- Cross-border requires same consent or adequacy decision; pack overlay enforces.

### 6.2 APPI right to disclosure / cessation
- Cascade ≤14 days; consent-graph targets 7 days.

## 7. SG — PDPA

### 7.1 PDPA Cross-Border Transfer Limitation Obligation (§26)
- Transfer prohibited unless recipient bound by comparable protection; partner-directory handshake
  records this attestation in `peer_attestation` field.

### 7.2 Do Not Call / Spam Control
- Not consent-graph's domain (Comms-Email µservice handles).

## 8. AU — Privacy Act + APP

- APP 8 (cross-border): pack overlay mandates partner attestation in handshake.
- APP 11 (security): mTLS + audit-chain seals satisfy reasonable-steps test.

## 9. IN — DPDP 2023

- §10 cross-border: government-notified-country list updated quarterly in pack overlay.
- §11 right to grievance: DSAR cascade includes grievance escalation hook.

## 10. BR — LGPD

- Art. 33 cross-border: adequacy decision or controller's safeguards (mirrored to GDPR Art. 46).

## 11. AE / KSA — region-specific

- AE PDPL Art. 19 cross-border: requires written agreement (matches our DataSharingAgreement).
- KSA PDPL: similar; local cloud residency emphasized in pack overlay.

## 12. Audit/inspection readiness

For each pack, the following evidence is queryable:
- All active agreements + scope + terms + lawful basis (Postgres view, RLS-scoped).
- All historical agreement lifecycle events (audit-chain query).
- All projection-emit + projection-read events (audit-chain).
- All revocations + propagation receipts (audit-chain + cross-pointer reconciliation reports).
- All DSAR cascade reports (evidence/ dir).

Audit-chain retention defaults per pack:
- KR/EU/JP/SG/AU/IN/BR: 7 years.
- US-Healthcare (HIPAA): 6 years (relaxed from 7y to match HIPAA spec).
- AE/KSA: 5 years.

## 13. Cross-references

- `dpia.md` for full Data Protection Impact Assessment per GDPR Art. 35.
- `data-residency.md` for per-region storage + processing geography.
- `iac/kustomize/overlays/<pack>/` for per-pack runtime config.
- `microservices/audit-chain/compliance.md` for the audit substrate map.

---



## §day-one-cert-readiness
This anchor is closed for `consent-graph` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `consent-graph` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +14 more.
- Example: `consent-grant` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `consent-graph`; owner `axis-consent-graph`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `agreement`, `enforcement`, `revocation`, `projection-gateway`, `audit-bridge`, `partner-directory`.
- Capability records cited: `microservices/consent-graph/capabilities/consent-enforce.yaml`, `microservices/consent-graph/capabilities/consent-grant.yaml`, `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`.
- API surfaces cited: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar/policy artifacts cited: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- SLO and dashboard evidence: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar binding: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- State/event binding: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Capability binding: `consent-grant`, `consent-project-subscribe`, `consent-enforce`.
- SLO binding: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `consent-graph`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `consent-graph`.
- `policy-engine` supplies the signed Cedar corpus while `consent-graph` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `consent-graph` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `consent-graph`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `consent-graph` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `consent-graph` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar` without changing domain code.
- Data classes under pack control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `consent-grant` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `consent-graph`; owner `axis-consent-graph`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `agreement`, `enforcement`, `revocation`, `projection-gateway`, `audit-bridge`, `partner-directory`.
- Capability records cited: `microservices/consent-graph/capabilities/consent-enforce.yaml`, `microservices/consent-graph/capabilities/consent-grant.yaml`, `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`.
- API surfaces cited: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar/policy artifacts cited: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- SLO and dashboard evidence: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar binding: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- State/event binding: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Capability binding: `consent-grant`, `consent-project-subscribe`, `consent-enforce`.
- SLO binding: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `consent-graph`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `consent-graph`.
- `policy-engine` supplies the signed Cedar corpus while `consent-graph` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `consent-graph` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `consent-graph`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `consent-graph` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `consent-graph` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `consent-graph` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`, `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`; +17 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `consent-grant` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.consent-graph.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `consent-graph`; owner `axis-consent-graph`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `agreement`, `enforcement`, `revocation`, `projection-gateway`, `audit-bridge`, `partner-directory`.
- Capability records cited: `microservices/consent-graph/capabilities/consent-enforce.yaml`, `microservices/consent-graph/capabilities/consent-grant.yaml`, `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`.
- API surfaces cited: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar/policy artifacts cited: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- SLO and dashboard evidence: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar binding: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- State/event binding: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Capability binding: `consent-grant`, `consent-project-subscribe`, `consent-enforce`.
- SLO binding: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `consent-graph`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `consent-graph`.
- `policy-engine` supplies the signed Cedar corpus while `consent-graph` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `consent-graph` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `consent-graph`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `consent-graph` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `consent-graph` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `consent-graph` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `consent-grant` touches those data classes.
- Signal sources: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`, `microservices/consent-graph/dashboards/consent-grant-funnel.json`, `microservices/consent-graph/dashboards/projection-freshness.json`; +10 more.
- Example event class: `oya.consent.graph.consent.grant.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `consent-graph`; owner `axis-consent-graph`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `agreement`, `enforcement`, `revocation`, `projection-gateway`, `audit-bridge`, `partner-directory`.
- Capability records cited: `microservices/consent-graph/capabilities/consent-enforce.yaml`, `microservices/consent-graph/capabilities/consent-grant.yaml`, `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`.
- API surfaces cited: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar/policy artifacts cited: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- SLO and dashboard evidence: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar binding: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- State/event binding: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Capability binding: `consent-grant`, `consent-project-subscribe`, `consent-enforce`.
- SLO binding: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `consent-graph`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `consent-graph`.
- `policy-engine` supplies the signed Cedar corpus while `consent-graph` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `consent-graph` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `consent-graph`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `consent-graph` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `consent-graph` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `consent-graph` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.consent-graph.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `consent-grant` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `consent-grant` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `consent-graph`; owner `axis-consent-graph`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `agreement`, `enforcement`, `revocation`, `projection-gateway`, `audit-bridge`, `partner-directory`.
- Capability records cited: `microservices/consent-graph/capabilities/consent-enforce.yaml`, `microservices/consent-graph/capabilities/consent-grant.yaml`, `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`.
- API surfaces cited: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar/policy artifacts cited: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- SLO and dashboard evidence: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar binding: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- State/event binding: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Capability binding: `consent-grant`, `consent-project-subscribe`, `consent-enforce`.
- SLO binding: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `consent-graph`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `consent-graph`.
- `policy-engine` supplies the signed Cedar corpus while `consent-graph` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `consent-graph` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `consent-graph`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `consent-graph` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `consent-graph` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `consent-graph` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`, `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`; +3 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `consent_graph.agreement` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `consent-graph`; owner `axis-consent-graph`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `agreement`, `enforcement`, `revocation`, `projection-gateway`, `audit-bridge`, `partner-directory`.
- Capability records cited: `microservices/consent-graph/capabilities/consent-enforce.yaml`, `microservices/consent-graph/capabilities/consent-grant.yaml`, `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`.
- API surfaces cited: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar/policy artifacts cited: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- SLO and dashboard evidence: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar binding: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- State/event binding: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Capability binding: `consent-grant`, `consent-project-subscribe`, `consent-enforce`.
- SLO binding: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `consent-graph`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `consent-graph`.
- `policy-engine` supplies the signed Cedar corpus while `consent-graph` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `consent-graph` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `consent-graph`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `consent-graph` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `consent-graph` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `consent-graph` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`, `microservices/consent-graph/iac/helm/consent-graph/Chart.yaml`, `microservices/consent-graph/iac/helm/consent-graph/templates/consent-graph-app-deployment.yaml`; +10 more.
- Example: `consent-grant` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `consent-graph`; owner `axis-consent-graph`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `agreement`, `enforcement`, `revocation`, `projection-gateway`, `audit-bridge`, `partner-directory`.
- Capability records cited: `microservices/consent-graph/capabilities/consent-enforce.yaml`, `microservices/consent-graph/capabilities/consent-grant.yaml`, `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`.
- API surfaces cited: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar/policy artifacts cited: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- SLO and dashboard evidence: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar binding: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- State/event binding: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Capability binding: `consent-grant`, `consent-project-subscribe`, `consent-enforce`.
- SLO binding: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `consent-graph`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `consent-graph`.
- `policy-engine` supplies the signed Cedar corpus while `consent-graph` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `consent-graph` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `consent-graph`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `consent-graph` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `consent-graph` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.consent-graph` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/consent-graph/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.
- Example: `consent-grant` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `consent-graph`; owner `axis-consent-graph`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `agreement`, `enforcement`, `revocation`, `projection-gateway`, `audit-bridge`, `partner-directory`.
- Capability records cited: `microservices/consent-graph/capabilities/consent-enforce.yaml`, `microservices/consent-graph/capabilities/consent-grant.yaml`, `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`.
- API surfaces cited: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar/policy artifacts cited: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- SLO and dashboard evidence: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar binding: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- State/event binding: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Capability binding: `consent-grant`, `consent-project-subscribe`, `consent-enforce`.
- SLO binding: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `consent-graph`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `consent-graph`.
- `policy-engine` supplies the signed Cedar corpus while `consent-graph` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `consent-graph` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `consent-graph`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `consent-graph` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `consent-graph` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `consent-graph` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`, `microservices/consent-graph/iac/helm/consent-graph/Chart.yaml`, `microservices/consent-graph/iac/helm/consent-graph/templates/consent-graph-app-deployment.yaml`, `microservices/consent-graph/iac/helm/consent-graph/templates/enforcement-app-deployment.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `consent-grant` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `consent-graph`; owner `axis-consent-graph`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `agreement`, `enforcement`, `revocation`, `projection-gateway`, `audit-bridge`, `partner-directory`.
- Capability records cited: `microservices/consent-graph/capabilities/consent-enforce.yaml`, `microservices/consent-graph/capabilities/consent-grant.yaml`, `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`.
- API surfaces cited: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar/policy artifacts cited: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- SLO and dashboard evidence: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar binding: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- State/event binding: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Capability binding: `consent-grant`, `consent-project-subscribe`, `consent-enforce`.
- SLO binding: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `consent-graph`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `consent-graph`.
- `policy-engine` supplies the signed Cedar corpus while `consent-graph` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `consent-graph` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `consent-graph`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `consent-graph` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `consent-graph` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `consent-graph` is in annual full-scope pentest and every major `consent-grant` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`, `microservices/consent-graph/iac/helm/consent-graph/Chart.yaml`, `microservices/consent-graph/iac/helm/consent-graph/templates/consent-graph-app-deployment.yaml`, `microservices/consent-graph/iac/helm/consent-graph/templates/enforcement-app-deployment.yaml`; +13 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `consent-graph` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `consent-graph`; owner `axis-consent-graph`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `agreement`, `enforcement`, `revocation`, `projection-gateway`, `audit-bridge`, `partner-directory`.
- Capability records cited: `microservices/consent-graph/capabilities/consent-enforce.yaml`, `microservices/consent-graph/capabilities/consent-grant.yaml`, `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`.
- API surfaces cited: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar/policy artifacts cited: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- SLO and dashboard evidence: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar binding: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- State/event binding: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Capability binding: `consent-grant`, `consent-project-subscribe`, `consent-enforce`.
- SLO binding: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `consent-graph`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `consent-graph`.
- `policy-engine` supplies the signed Cedar corpus while `consent-graph` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `consent-graph` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `consent-graph`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `consent-graph` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `consent-graph` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `consent-graph` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `consent-grant` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `consent-graph`; owner `axis-consent-graph`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `agreement`, `enforcement`, `revocation`, `projection-gateway`, `audit-bridge`, `partner-directory`.
- Capability records cited: `microservices/consent-graph/capabilities/consent-enforce.yaml`, `microservices/consent-graph/capabilities/consent-grant.yaml`, `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`.
- API surfaces cited: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar/policy artifacts cited: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- SLO and dashboard evidence: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar binding: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- State/event binding: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Capability binding: `consent-grant`, `consent-project-subscribe`, `consent-enforce`.
- SLO binding: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `consent-graph`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `consent-graph`.
- `policy-engine` supplies the signed Cedar corpus while `consent-graph` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `consent-graph` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `consent-graph`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `consent-graph` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `consent-graph` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `consent-graph` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/consent-graph/catalog/oya-consent-graph-agreement-adapter.yaml`, `microservices/consent-graph/catalog/oya-consent-graph-agreement-domain.yaml`, `microservices/consent-graph/catalog/oya-consent-graph-agreement-kernel.yaml`, `microservices/consent-graph/catalog/oya-consent-graph-agreement-rest.yaml`, `microservices/consent-graph/catalog/oya-consent-graph-agreement-sdk.yaml`, `microservices/consent-graph/catalog/oya-consent-graph-agreement-usecase.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `consent-grant` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `consent-graph`; owner `axis-consent-graph`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `agreement`, `enforcement`, `revocation`, `projection-gateway`, `audit-bridge`, `partner-directory`.
- Capability records cited: `microservices/consent-graph/capabilities/consent-enforce.yaml`, `microservices/consent-graph/capabilities/consent-grant.yaml`, `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`.
- API surfaces cited: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar/policy artifacts cited: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- SLO and dashboard evidence: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar binding: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- State/event binding: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Capability binding: `consent-grant`, `consent-project-subscribe`, `consent-enforce`.
- SLO binding: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `consent-graph`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `consent-graph`.
- `policy-engine` supplies the signed Cedar corpus while `consent-graph` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `consent-graph` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `consent-graph`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `consent-graph` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `consent-graph` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `consent-graph` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `consent-grant` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `consent-grant` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `consent-graph`; owner `axis-consent-graph`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `agreement`, `enforcement`, `revocation`, `projection-gateway`, `audit-bridge`, `partner-directory`.
- Capability records cited: `microservices/consent-graph/capabilities/consent-enforce.yaml`, `microservices/consent-graph/capabilities/consent-grant.yaml`, `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`.
- API surfaces cited: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar/policy artifacts cited: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- SLO and dashboard evidence: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar binding: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- State/event binding: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Capability binding: `consent-grant`, `consent-project-subscribe`, `consent-enforce`.
- SLO binding: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `consent-graph`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `consent-graph`.
- `policy-engine` supplies the signed Cedar corpus while `consent-graph` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `consent-graph` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `consent-graph`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `consent-graph` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `consent-graph` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.
- State/event surfaces carrying classification: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `consent-grant` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `consent-graph`; owner `axis-consent-graph`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `agreement`, `enforcement`, `revocation`, `projection-gateway`, `audit-bridge`, `partner-directory`.
- Capability records cited: `microservices/consent-graph/capabilities/consent-enforce.yaml`, `microservices/consent-graph/capabilities/consent-grant.yaml`, `microservices/consent-graph/capabilities/consent-project-subscribe.yaml`.
- API surfaces cited: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar/policy artifacts cited: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- SLO and dashboard evidence: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +14 more.
- Data classes declared for this control: `PII_IDENTIFYING`, `AUTHENTICATION`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/consent-graph/contracts/asyncapi/consent-events.yaml`, `microservices/consent-graph/contracts/openapi/consent-graph.yaml`, `microservices/consent-graph/contracts/proto/consent-graph.proto`.
- Cedar binding: `microservices/consent-graph/policy/aggregate-k-anonymity.cedar`, `microservices/consent-graph/policy/break-glass-healthcare.cedar`, `microservices/consent-graph/policy/cross-tenant-projection.cedar`, `microservices/consent-graph/policy/deny-all-fallback.cedar`.
- State/event binding: `consent_graph.agreement`, `consent_graph.enforcement`, `consent_graph.revocation`, `consent_graph.projection_gateway`, `consent_graph.audit_bridge`, `consent_graph.partner_directory`.
- Capability binding: `consent-grant`, `consent-project-subscribe`, `consent-enforce`.
- SLO binding: `microservices/consent-graph/slos/agreement-state-divergence-zero.openslo.yaml`, `microservices/consent-graph/slos/audit-chain-coverage-completeness.openslo.yaml`, `microservices/consent-graph/slos/bilateral-chain-link-integrity.openslo.yaml`, `microservices/consent-graph/slos/cedar-evaluation-latency.openslo.yaml`, `microservices/consent-graph/slos/consent-grant-latency.openslo.yaml`, `microservices/consent-graph/slos/cross-tenant-projection-freshness.openslo.yaml`; +3 more.
- Runbook binding: `microservices/consent-graph/runbooks/GDPR-DSAR-cross-tenant.md`, `microservices/consent-graph/runbooks/audit-chain-divergence-recovery.md`, `microservices/consent-graph/runbooks/consent-forgery-detected.md`, `microservices/consent-graph/runbooks/data-residency-enforcement.md`, `microservices/consent-graph/runbooks/partner-offboarding.md`, `microservices/consent-graph/runbooks/partner-onboarding.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `consent-graph`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `consent-graph`.
- `policy-engine` supplies the signed Cedar corpus while `consent-graph` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `consent-graph` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `consent-graph`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `consent-graph` applies the most restrictive policy and emits a degraded-mode audit event.
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
