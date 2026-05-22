---
microservice: connect
doc_class: DPIA
date: 2026-05-20
owner_team: council-privacy + axis-integration
status: Accepted
related_adrs: [ADR-0244, ADR-0255, ADR-0263, ADR-0272, ADR-0276]
related_regulations: [GDPR Art. 35, KR PIPA Art. 33, US-state CPRA §1798.185, EU AI Act Art. 27]
companion_docs:
  - microservices/connect/threat-model.md
  - microservices/connect/compliance.md
review_cadence: annually + on every new vendor onboarded
doc_status: published
---

# DPIA — connect (Integration Substrate)

GDPR Art. 35 Data Protection Impact Assessment for the integration-substrate processing.

## Article 35(1) trigger

This µservice mediates third-party data flows on behalf of tenants. Categories of processing requiring DPIA:
- Large-scale processing of personal data (vendor webhooks deliver customer PII).
- Systematic monitoring of data subjects (catalog telemetry; abuse-defence fingerprinting).
- Sensitive special-categories data (health vendors via pack-us-healthcare; payment vendors).

## Article 35(7)(a) — systematic description

### Purpose
Brokering tenant-authorized integrations with ≥500 external SaaS APIs. Each integration: OAuth grant → token storage → outbound call → response canonicalization → audit emission.

### Necessity
Without a shared substrate, every product µservice re-implements credential storage + OAuth + webhook reception. Centralization improves auditability + isolation + crypto rigor.

### Proportionality
Connect holds the *minimum*: OAuth refresh tokens (encrypted at rest in OpenBao); per-wiring HMAC signing secrets; audit trails. Tenant data (Salesforce contacts, Shopify orders) flows *through* but is not retained beyond the workflow-engine's working-set window.

## Article 35(7)(b) — necessity + proportionality assessment

| Aspect | Assessment |
|---|---|
| Lawful basis | Art. 6(1)(b) contract performance — substrate is necessary to deliver the tenant's contracted integrations. Art. 6(1)(f) legitimate interest for audit + abuse-defence (balanced via UX-floor §3.2.3). |
| Data minimization | Only `tenant_id`, `principal_id`, `payload_digest` (not full payload) retained in audit chain. Full payload retained for max 7d in DLQ per data-residency.md. |
| Storage limitation | OAuth refresh tokens: until grant revoked. Webhook signing secrets: until wiring deleted. DLQ payloads: 7d default (max 30d). Audit chain: per pack overlay (KR: 5yr; EU: 6yr; US-healthcare: 6yr). |
| Accuracy | Schema-drift detection ensures mapped fields stay current. Wiring owners notified on detected drift. |

## Article 35(7)(c) — risk assessment

| # | Risk | Likelihood | Impact | Score |
|---|---|---|---|---|
| R-01 | OAuth refresh token theft | Low (OpenBao + sidecar isolation) | Critical (full tenant data access at vendor) | High |
| R-02 | Webhook payload exfiltration | Low (HMAC verify + per-tenant DNS) | High (PII exposure) | Medium-High |
| R-03 | Audit log spoofing | Very Low (audit chain Merkle-sealed) | High (compliance breach) | Medium |
| R-04 | Abuse-defence false-positive impacts data subject | Low (UX-floor preserves default path) | Medium (UX degradation) | Low-Medium |
| R-05 | Cross-tenant data leak via adapter bug | Low (Kata sandbox + Cedar gate) | Critical | Medium |
| R-06 | Vendor-side breach exposes tokens | Medium (vendor-dependent) | High | Medium-High |
| R-07 | PII in DLQ payloads visible to ops | Low (step-up auth + payload-digest-only default) | Medium-High | Medium |
| R-08 | Minor user data routed through connector (COPPA <13 per ADR-0292) | Low (consumer-facing surface filters minors) | Critical | Medium |

## Article 35(7)(d) — mitigations

1. **OpenBao + sidecar isolation (R-01):** Per ADR-0296, adapter processes see only ≤60s access tokens, never refresh tokens. Plus mTLS + SPIFFE on every internal call.
2. **HMAC + replay-window (R-02):** Per `signature-verification.cedar`, all inbound payloads HMAC-verified with replay-window ≤5min + idempotency-key dedup.
3. **Audit chain Merkle-seal (R-03):** Per ADR-0028 + ADR-0263, every audit event signed by per-µservice signing key; chain Merkle-sealed; replay requires substrate compromise + collision attack.
4. **UX-floor (R-04):** Per documentation-rigor §3.2.3, default-path latency ≤2ms; challenges only on confirmed bot-score; a11y alternatives; tenant-tier-adaptive.
5. **Kata sandbox + Cedar (R-05):** Per ADR-0254, adapters run in Cloud Hypervisor + Kata; cross-tenant data flow physically isolated. Cedar `tenant_id` filter on every action.
6. **Vendor breach response (R-06):** Per ADR-0273, per-tenant DKIM/SPF/DMARC enables targeted re-issuance; OAuth grants can be auto-revoked en masse via SCIM webhook.
7. **DLQ payload protection (R-07):** Encrypted-at-rest; UI shows digest only; full payload requires step-up auth + audit event `DLQPayloadAccessAttempt`.
8. **Minor user filtering (R-08):** Per ADR-0292, consumer-facing surfaces refuse <13; KOSA 14-17 tier; EU age-verification. Connector wiring requires confirmation of audience age-tier.

## Article 35(9) — data subject consultation

Where applicable, tenant admins consult their data subjects via in-product consent surfaces per ADR-0272 (cookie consent per-purpose). For pack-eu and pack-us-state-CPRA, DPAs are notified at onboarding.

## DPA notifications

- KR PIPC: notified at first KR tenant onboarding (PIPA Art. 33).
- EU lead DPA: per pack-eu activation (Ireland DPC, France CNIL, depending on tenant HQ).
- US state AGs: per CPRA/CCPA opt-out registries.

## Residual risk

After mitigations: **acceptable** for all R-NN rows. Residual = vendor-side breach (R-06) which oyatie cannot directly mitigate; transferred via vendor sub-processor agreements + data-portability evidence per ADR-0276.

## Review cadence

Annual + on every new vendor onboarded + on any pack overlay activation + on any new BC addition.

## References

- ADR-0272 cookie consent per-purpose
- ADR-0276 backup portability GDPR Art. 20
- docs/standards/dpia-template.md (canonical structure)
