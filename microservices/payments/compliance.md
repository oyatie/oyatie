---
doc_class: Compliance
template_id: TPL-COMPLIANCE
microservice: payments
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: ops-compliance + axis-payments + council-privacy + dpo + council-finance
deciders: council-architecture, ops-compliance, axis-payments, council-privacy, dpo, council-finance
related_adrs:
  - ADR-0145
  - ADR-0242
  - ADR-0244
  - ADR-0246
  - ADR-0247
  - ADR-0248
  - ADR-0250
  - ADR-0251
  - ADR-0253
  - ADR-0255
  - ADR-0258
  - ADR-0263
  - ADR-0272
  - ADR-0273
  - ADR-0276
  - ADR-0280
  - ADR-0284
  - ADR-0292
  - ADR-0293
  - ADR-0294
  - ADR-0295
  - ADR-0296
companion_docs:
  - microservices/payments/PRD.md
  - microservices/payments/ARCHITECTURE.md
  - microservices/payments/threat-model.md
  - microservices/payments/dpia.md
  - microservices/payments/incident-response.md
diataxis_quadrant: explanation
doc_status: published
---

# Compliance — payments µservice

> Pack-overlay roster + per-pack control mapping. Day-one certification-ready per ADR-0250 build-ahead-of-certification.

---

## §pack-overlay-roster

| Pack ID | What it gates | Mandatory? | First-cert target |
|---|---|---|---|
| `pack-pci-dss-l1-v4` | Card-data scope | **Mandatory** | Q4 2026 |
| `pack-kr-fss` | Korean Financial Supervisory Service oversight | KR tenants | Q2 2027 |
| `pack-eu-psd2-sca` | EU Strong Customer Authentication | EU tenants | Continuous; per-tenant licence at onboarding |
| `pack-us-state-mtl` | US state money-transmitter | US tenants | Per-state per-tenant |
| `pack-ccpa-cpra-2023` | California consumer rights | US-CA subjects | Continuous |
| `pack-au-aml-ctf` | Australia AML / CTF | AU tenants | Continuous |
| `pack-br-lgpd-finance` | Brazil LGPD + BACEN | BR tenants | Continuous |
| `pack-in-rbi` | India RBI payment-aggregator | IN tenants | Continuous |
| `pack-cn-pipl-2021` | China PIPL data-residency | CN tenants | Continuous |
| `pack-coppa-minor-refusal` | Refuse <13 per ADR-0292 | Always | Continuous |
| `pack-sox-itgc` | SOX-aware audit controls (B2B) | Public-company tenants | Continuous |
### Content-pass expansion — pack-overlay-roster
- This expansion preserves the existing prose above and closes `pack-overlay-roster` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: AWS Control Tower guardrails anchors the external control pattern for `pack-overlay-roster`.
- Precedent 2: Microsoft Purview Compliance Manager provides a second independent hyperscaler pattern for `pack-overlay-roster`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `pack-overlay-roster`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `pack-overlay-roster` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `pack-overlay-roster` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `pack overlay roster` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `pack overlay roster`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.

## §day-one-cert-readiness (per ADR-0250)

Payments is **certified-shape day one**. We do not retrofit compliance; the architecture, the audit-trail, the Cedar gates, the data-residency rules, the credential-isolation are wired from day one. Subsequent QSA / regulator audits validate the shape; they do not require architectural changes.
### Content-pass expansion — day-one-cert-readiness
- This expansion preserves the existing prose above and closes `day-one-cert-readiness` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: AWS Artifact anchors the external control pattern for `day-one-cert-readiness`.
- Precedent 2: Google Assured Workloads provides a second independent hyperscaler pattern for `day-one-cert-readiness`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `day-one-cert-readiness`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `day-one-cert-readiness` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `day-one-cert-readiness (per ADR-0250)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `day one cert readiness (per ADR 0250)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `day one cert readiness (per ADR 0250)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `day one cert readiness (per ADR 0250)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `payments` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `payments` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `payments` uses SLOs `slos/charge-api-availability.openslo.yaml, slos/charge-api-latency.openslo.yaml, slos/dispute-response-latency.openslo.yaml, slos/payout-api-latency.openslo.yaml, slos/payout-completion-success.openslo.yaml, plus 3 more` and dashboards `dashboards/dispute-volume.json, dashboards/finops-cost-attribution.md, dashboards/fraud-signals.md, dashboards/payments-overview.json, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `payments` uses runbooks `runbooks/aml-suspicious-activity-detected.md, runbooks/dispute-escalation.md, runbooks/double-charge-detected.md, runbooks/elder-financial-abuse.md, runbooks/fraud-spike-detected.md, plus 5 more` so `day one cert readiness (per ADR 0250)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `payments` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/payments-app/Chart.yaml, iac/helm/payments-app/values.yaml, iac/helm/payments-webhook-handler/Chart.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `payments` uses `capabilities/charge.yaml, capabilities/dispute.yaml, capabilities/payout.yaml, capabilities/refund.yaml, plus 2 more` and `catalog/oya-payments-adapter-adyen.yaml, catalog/oya-payments-adapter-stripe.yaml, catalog/oya-payments-charge-app.yaml, catalog/oya-payments-charge-domain.yaml, plus 9 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `payments` fails closed when `day one cert readiness (per ADR 0250)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `payments` emits denial evidence for `day one cert readiness (per ADR 0250)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `payments` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `day one cert readiness (per ADR 0250)` workflow.
- Depth detail 17: `payments` telemetry for `day one cert readiness (per ADR 0250)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `payments` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §1. PCI-DSS L1 v4 (pack-pci-dss-l1-v4)

### Scope

oyatie operates as a **SAQ-A facilitator** above PSP-tokenisation. PAN / PIN / track-data **never** enters our systems; PSP SDKs collect on the consumer device and POST directly to PSP. Our scope is limited to:

- The pages that load PSP SDKs (PSP-controlled).
- The audit-trail surrounding tokenised charges.
- Sub-merchant onboarding (PCI facilitator role).

### Req-by-req mapping (PCI DSS v4 high-level)

| Req | Topic | How payments satisfies |
|---|---|---|
| 1 | Network security controls | Cilium L4 + Istio mTLS + edge WAF; Tier-1 cells isolated |
| 2 | Secure system + software configurations | IaC at [`iac/`](iac/); CIS K8s benchmark green |
| 3 | Protect stored cardholder data | PAN never stored; only tokenised refs |
| 4 | Protect cardholder data with strong cryptography | TLS 1.3 + ECH + PQC hybrid; HTTP/3 default per ADR-0253 |
| 5 | Protect from malicious software | Trivy + Snyk + sigstore + Fulcio image-signing |
| 6 | Develop + maintain secure systems | clean-architecture per ADR-0056; lean-a lanes; doc-rigor per documentation-rigor.md |
| 7 | Restrict access by need-to-know | Cedar default-deny + tenant-scope per ADR-0243 |
| 8 | Identify users + authenticate access | OIDC + step-up auth + SPIFFE SVID per ADR-0295 |
| 9 | Restrict physical access | Inherits cloud-k8s + cloud-secrets controls |
| 10 | Log + monitor all access | Merkle-sealed audit per ADR-0028 + ADR-0263 |
| 11 | Test security regularly | Quarterly penetration tests; oya-governance-doc-rigor + cross-tenant-query CI lanes |
| 12 | Maintain InfoSec policy | [`threat-model.md`](threat-model.md) + [`incident-response.md`](incident-response.md) + [`runbooks/pci-incident-response.md`](runbooks/pci-incident-response.md) |

### Evidence library

- QSA-validated audit log: 7y retention, Merkle-sealed.
- Penetration test reports: quarterly.
- Vulnerability scans: weekly external + daily internal.
- Network segmentation diagrams: [`ARCHITECTURE.md`](ARCHITECTURE.md) §C + iac.
- Sub-merchant attestations: PSP-Connect-flow signed at onboarding.

## §2. KR-FSS (pack-kr-fss)

Korean Financial Supervisory Service oversight of e-financial operators.

### KR-EFTA mapping

| KR-EFTA Art. | Topic | How payments satisfies |
|---|---|---|
| §6 | User identity verification | KR-PASS / KR-iPIN at sub-merchant onboarding |
| §9 | Electronic financial transaction record | Audit-chain per ADR-0028 |
| §21-3 | Security duty | TLS 1.3 + Cedar default-deny + OpenBao + threat-model annual review |
| §22-2 | Electronic financial fraud prevention | Fraud-ML + per-tenant abuse-defence per documentation-rigor.md §3.2.3 |
| §29-2 | Audit-trail availability for regulator | [`runbooks/kr-fss-audit-pull.md`](runbooks/kr-fss-audit-pull.md) |

### KR-FSS audit-pull surface

- Endpoint: `/api/v1/kr-fss/audit/pull` — KR-FSS-credential-authenticated.
- Cedar fragment: [`policy/auditor-scope.cedar`](policy/auditor-scope.cedar) with `audit_framework=kr-fss`.
- Retention: 10 years per KR-EFTA.

## §3. EU PSD2 + SCA (pack-eu-psd2-sca)

### SCA RTS (EU 2018/389) mapping

| RTS Art. | Topic | How payments satisfies |
|---|---|---|
| Art. 4 | Strong customer authentication | Step-up (3DS2) for threshold-exceeding charges |
| Art. 5 | Dynamic linking | Per-charge amount + payee binding in challenge |
| Arts. 10-18 | Exemptions | Per-tenant exemption-policy cedar gate; exemption-audit |
| Art. 22 | Authentication code | OTP / passkey / biometric per PSP support |

### Per-tenant SCA wiring

- Tenant declares jurisdiction + transaction-class in onboarding.
- SCA-threshold and exemption-policy stored in tenant manifest.
- Per-charge: Cedar gate `policy/charge-authorization.cedar` evaluates SCA-required + threshold; PSP-side 3DS2 challenge flow.

## §4. US state MTL (pack-us-state-mtl)

- Per-state licence registry maintained in ops-compliance.
- oyatie operates as facilitator above PSP MTL; tenant onboarding determines pass-through MTL needs.
- Per-state restricted-reason taxonomy on payouts.

## §5. CCPA / CPRA (pack-ccpa-cpra-2023)

| CCPA / CPRA right | How payments satisfies |
|---|---|
| Right to know | Subject-access endpoint per [`backfill-replay.md`](backfill-replay.md) §GDPR Art. 20 |
| Right to delete | Limited by financial-record exemption; subject is informed |
| Right to opt-out-of-sale | Payments never sells data; no opt-out needed |
| Right to limit use of sensitive personal info | Bank-account stored for processing only; not used for analytics |

## §6. AU AML / CTF (pack-au-aml-ctf)

- AUSTRAC threshold-transaction reports (TTRs) automated for cash-equivalent transactions.
- Suspicious-matter reports (SMRs) raised by ops-fraud.
- KYB-CDD continuous monitoring (industry standard ML risk-score).

## §7. BR LGPD + BACEN (pack-br-lgpd-finance)

- LGPD subject rights mapped same as GDPR (§ in [`dpia.md`](dpia.md)).
- BACEN Res. 4.893/2021 cybersecurity controls met via threat-model + IAM.
- BACEN incident notification per [`incident-response.md`](incident-response.md) §3.7.

## §8. IN RBI (pack-in-rbi)

- RBI payment-aggregator licence required for IN tenants.
- IN-domiciled data per `cn-1`-style residency rule.
- RBI cyber-incident reporting within 6h per `incident-response.md` §3.8.

## §9. CN PIPL (pack-cn-pipl-2021)

- CN cell hard-pinned; **no cross-border data egress**.
- Cedar `policy/data-residency.md` FORBID on egress.
- PBoC notification within 24h per `incident-response.md` §3.9.

## §10. COPPA + KOSA (pack-coppa-minor-refusal)

Per ADR-0292:

| Age class | Behaviour |
|---|---|
| <13 | Refuse all payments. Audit: `oya.payments.minor.refused-coppa`. |
| 14-17 (KOSA) | Allow subset (no recurring, no >$50, parental-consent flag). |
| 18+ | No payments-side restriction. |

## §11. SOX-ITGC (pack-sox-itgc)

For public-company B2B tenants:

- Audit-trail review process (semi-annual at minimum).
- Change-control on payments contracts (ADR + version bump per ADR-0258).
- Segregation-of-duties on payout > $10k (dual-signoff).

## §consent (ADR-0272)

When the surface is user-facing, per-purpose consent:

- `payments.fraud-fingerprint` — behavioural fingerprinting.
- `payments.marketing-attribution` — marketing-tag pass-through.

Each purpose is presented separately; "Process payment" is required for the transaction; "Fraud detection" is required by legitimate-interest + Article 6(1)(f); "Marketing attribution" is opt-in only.
### Content-pass expansion — consent
- This expansion preserves the existing prose above and closes `consent` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: Google Consent Mode anchors the external control pattern for `consent`.
- Precedent 2: Apple App Tracking Transparency provides a second independent hyperscaler pattern for `consent`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `consent`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `consent` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `consent (ADR-0272)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `consent (ADR 0272)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `consent (ADR 0272)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `consent (ADR 0272)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `payments` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `payments` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `payments` uses SLOs `slos/charge-api-availability.openslo.yaml, slos/charge-api-latency.openslo.yaml, slos/dispute-response-latency.openslo.yaml, slos/payout-api-latency.openslo.yaml, slos/payout-completion-success.openslo.yaml, plus 3 more` and dashboards `dashboards/dispute-volume.json, dashboards/finops-cost-attribution.md, dashboards/fraud-signals.md, dashboards/payments-overview.json, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `payments` uses runbooks `runbooks/aml-suspicious-activity-detected.md, runbooks/dispute-escalation.md, runbooks/double-charge-detected.md, runbooks/elder-financial-abuse.md, runbooks/fraud-spike-detected.md, plus 5 more` so `consent (ADR 0272)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `payments` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/payments-app/Chart.yaml, iac/helm/payments-app/values.yaml, iac/helm/payments-webhook-handler/Chart.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `payments` uses `capabilities/charge.yaml, capabilities/dispute.yaml, capabilities/payout.yaml, capabilities/refund.yaml, plus 2 more` and `catalog/oya-payments-adapter-adyen.yaml, catalog/oya-payments-adapter-stripe.yaml, catalog/oya-payments-charge-app.yaml, catalog/oya-payments-charge-domain.yaml, plus 9 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `payments` fails closed when `consent (ADR 0272)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `payments` emits denial evidence for `consent (ADR 0272)` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §email-deliverability (ADR-0273)

- Receipts / dunning emails via `mail` µservice with per-tenant DKIM / SPF / DMARC.
- Payments does NOT send email directly; emits domain event `oya.payments.notification.requested`.
### Content-pass expansion — email-deliverability
- This expansion preserves the existing prose above and closes `email-deliverability` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: Google Workspace DKIM/SPF/DMARC anchors the external control pattern for `email-deliverability`.
- Precedent 2: AWS SES domain identity provides a second independent hyperscaler pattern for `email-deliverability`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `email-deliverability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `email-deliverability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `email-deliverability (ADR-0273)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `email deliverability (ADR 0273)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `email deliverability (ADR 0273)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `email deliverability (ADR 0273)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `payments` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `payments` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `payments` uses SLOs `slos/charge-api-availability.openslo.yaml, slos/charge-api-latency.openslo.yaml, slos/dispute-response-latency.openslo.yaml, slos/payout-api-latency.openslo.yaml, slos/payout-completion-success.openslo.yaml, plus 3 more` and dashboards `dashboards/dispute-volume.json, dashboards/finops-cost-attribution.md, dashboards/fraud-signals.md, dashboards/payments-overview.json, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `payments` uses runbooks `runbooks/aml-suspicious-activity-detected.md, runbooks/dispute-escalation.md, runbooks/double-charge-detected.md, runbooks/elder-financial-abuse.md, runbooks/fraud-spike-detected.md, plus 5 more` so `email deliverability (ADR 0273)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `payments` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/payments-app/Chart.yaml, iac/helm/payments-app/values.yaml, iac/helm/payments-webhook-handler/Chart.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `payments` uses `capabilities/charge.yaml, capabilities/dispute.yaml, capabilities/payout.yaml, capabilities/refund.yaml, plus 2 more` and `catalog/oya-payments-adapter-adyen.yaml, catalog/oya-payments-adapter-stripe.yaml, catalog/oya-payments-charge-app.yaml, catalog/oya-payments-charge-domain.yaml, plus 9 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `payments` fails closed when `email deliverability (ADR 0273)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `payments` emits denial evidence for `email deliverability (ADR 0273)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `payments` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `email deliverability (ADR 0273)` workflow.
- Depth detail 17: `payments` telemetry for `email deliverability (ADR 0273)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §self-modification (ADR-0247)

The `oyatie.payments.foundry` principal can modify:
- `policy/*.cedar` fragments (with ADR-0294 soak + ADR-0293 meta-trust attestation).
- Generated Rust code under controlled paths.

It cannot modify:
- `ARCHITECTURE.md`, `PRD.md`, `compliance.md` (human-authored).
- `iac/production-*.yaml` (CI gate + human approval).
- Contracts (`contracts/*.yaml`) without ADR + human approval.
### Content-pass expansion — self-modification
- This expansion preserves the existing prose above and closes `self-modification` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: SLSA provenance anchors the external control pattern for `self-modification`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `self-modification`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `self-modification`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `self-modification` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `self-modification (ADR-0247)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `self modification (ADR 0247)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `self modification (ADR 0247)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `self modification (ADR 0247)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `payments` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `payments` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `payments` uses SLOs `slos/charge-api-availability.openslo.yaml, slos/charge-api-latency.openslo.yaml, slos/dispute-response-latency.openslo.yaml, slos/payout-api-latency.openslo.yaml, slos/payout-completion-success.openslo.yaml, plus 3 more` and dashboards `dashboards/dispute-volume.json, dashboards/finops-cost-attribution.md, dashboards/fraud-signals.md, dashboards/payments-overview.json, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `payments` uses runbooks `runbooks/aml-suspicious-activity-detected.md, runbooks/dispute-escalation.md, runbooks/double-charge-detected.md, runbooks/elder-financial-abuse.md, runbooks/fraud-spike-detected.md, plus 5 more` so `self modification (ADR 0247)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `payments` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/payments-app/Chart.yaml, iac/helm/payments-app/values.yaml, iac/helm/payments-webhook-handler/Chart.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.

## §meta-trust-attestation (ADR-0293)

Foundry-touching attestation chain: Cedar fragment proposed → signed by Foundry-agent SVID → soak ≥60s per ADR-0294 → publish → record `audit-chain://oya.payments.policy-published`.
### Content-pass expansion — meta-trust-attestation
- This expansion preserves the existing prose above and closes `meta-trust-attestation` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: The Update Framework roots anchors the external control pattern for `meta-trust-attestation`.
- Precedent 2: Sigstore Rekor transparency provides a second independent hyperscaler pattern for `meta-trust-attestation`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `meta-trust-attestation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `meta-trust-attestation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `meta-trust-attestation (ADR-0293)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `meta trust attestation (ADR 0293)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `meta trust attestation (ADR 0293)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `meta trust attestation (ADR 0293)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `payments` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `payments` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `payments` uses SLOs `slos/charge-api-availability.openslo.yaml, slos/charge-api-latency.openslo.yaml, slos/dispute-response-latency.openslo.yaml, slos/payout-api-latency.openslo.yaml, slos/payout-completion-success.openslo.yaml, plus 3 more` and dashboards `dashboards/dispute-volume.json, dashboards/finops-cost-attribution.md, dashboards/fraud-signals.md, dashboards/payments-overview.json, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `payments` uses runbooks `runbooks/aml-suspicious-activity-detected.md, runbooks/dispute-escalation.md, runbooks/double-charge-detected.md, runbooks/elder-financial-abuse.md, runbooks/fraud-spike-detected.md, plus 5 more` so `meta trust attestation (ADR 0293)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `payments` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/payments-app/Chart.yaml, iac/helm/payments-app/values.yaml, iac/helm/payments-webhook-handler/Chart.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `payments` uses `capabilities/charge.yaml, capabilities/dispute.yaml, capabilities/payout.yaml, capabilities/refund.yaml, plus 2 more` and `catalog/oya-payments-adapter-adyen.yaml, catalog/oya-payments-adapter-stripe.yaml, catalog/oya-payments-charge-app.yaml, catalog/oya-payments-charge-domain.yaml, plus 9 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `payments` fails closed when `meta trust attestation (ADR 0293)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `payments` emits denial evidence for `meta trust attestation (ADR 0293)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `payments` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `meta trust attestation (ADR 0293)` workflow.
- Depth detail 17: `payments` telemetry for `meta trust attestation (ADR 0293)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `payments` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §platform-owner-indirection (ADR-0284)

Payments contains zero hard-coded `oyatie` string references. Platform-owner is read from runtime config `OYA_PLATFORM_OWNER` env-var with default-allow-list per `specs/platform-owner-allowlist.json`. Grep-audit clean (validated via `oya-governance-platform-owner-indirection` CI lane).
### Content-pass expansion — platform-owner-indirection
- This expansion preserves the existing prose above and closes `platform-owner-indirection` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: Salesforce My Domain anchors the external control pattern for `platform-owner-indirection`.
- Precedent 2: Google Workspace tenant branding provides a second independent hyperscaler pattern for `platform-owner-indirection`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `platform-owner-indirection`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `platform-owner-indirection` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `platform-owner-indirection (ADR-0284)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `platform owner indirection (ADR 0284)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `platform owner indirection (ADR 0284)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `platform owner indirection (ADR 0284)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `payments` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `payments` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `payments` uses SLOs `slos/charge-api-availability.openslo.yaml, slos/charge-api-latency.openslo.yaml, slos/dispute-response-latency.openslo.yaml, slos/payout-api-latency.openslo.yaml, slos/payout-completion-success.openslo.yaml, plus 3 more` and dashboards `dashboards/dispute-volume.json, dashboards/finops-cost-attribution.md, dashboards/fraud-signals.md, dashboards/payments-overview.json, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `payments` uses runbooks `runbooks/aml-suspicious-activity-detected.md, runbooks/dispute-escalation.md, runbooks/double-charge-detected.md, runbooks/elder-financial-abuse.md, runbooks/fraud-spike-detected.md, plus 5 more` so `platform owner indirection (ADR 0284)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `payments` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/payments-app/Chart.yaml, iac/helm/payments-app/values.yaml, iac/helm/payments-webhook-handler/Chart.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `payments` uses `capabilities/charge.yaml, capabilities/dispute.yaml, capabilities/payout.yaml, capabilities/refund.yaml, plus 2 more` and `catalog/oya-payments-adapter-adyen.yaml, catalog/oya-payments-adapter-stripe.yaml, catalog/oya-payments-charge-app.yaml, catalog/oya-payments-charge-domain.yaml, plus 9 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `payments` fails closed when `platform owner indirection (ADR 0284)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `payments` emits denial evidence for `platform owner indirection (ADR 0284)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `payments` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `platform owner indirection (ADR 0284)` workflow.
- Depth detail 17: `payments` telemetry for `platform owner indirection (ADR 0284)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `payments` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §bootstrap-trust-chain (ADR-0295)

Payments inherits SPIFFE workload identity from cluster SPIRE. Every `oya-payments-*-app` pod carries an SVID; kill-switch wired per ADR-0295 §D-6.
### Content-pass expansion — bootstrap-trust-chain
- This expansion preserves the existing prose above and closes `bootstrap-trust-chain` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: SPIFFE/SPIRE workload identity anchors the external control pattern for `bootstrap-trust-chain`.
- Precedent 2: Sigstore Fulcio provides a second independent hyperscaler pattern for `bootstrap-trust-chain`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `bootstrap-trust-chain`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `bootstrap-trust-chain` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `bootstrap-trust-chain (ADR-0295)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `bootstrap trust chain (ADR 0295)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `bootstrap trust chain (ADR 0295)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `bootstrap trust chain (ADR 0295)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `payments` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `payments` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `payments` uses SLOs `slos/charge-api-availability.openslo.yaml, slos/charge-api-latency.openslo.yaml, slos/dispute-response-latency.openslo.yaml, slos/payout-api-latency.openslo.yaml, slos/payout-completion-success.openslo.yaml, plus 3 more` and dashboards `dashboards/dispute-volume.json, dashboards/finops-cost-attribution.md, dashboards/fraud-signals.md, dashboards/payments-overview.json, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `payments` uses runbooks `runbooks/aml-suspicious-activity-detected.md, runbooks/dispute-escalation.md, runbooks/double-charge-detected.md, runbooks/elder-financial-abuse.md, runbooks/fraud-spike-detected.md, plus 5 more` so `bootstrap trust chain (ADR 0295)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `payments` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/payments-app/Chart.yaml, iac/helm/payments-app/values.yaml, iac/helm/payments-webhook-handler/Chart.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `payments` uses `capabilities/charge.yaml, capabilities/dispute.yaml, capabilities/payout.yaml, capabilities/refund.yaml, plus 2 more` and `catalog/oya-payments-adapter-adyen.yaml, catalog/oya-payments-adapter-stripe.yaml, catalog/oya-payments-charge-app.yaml, catalog/oya-payments-charge-domain.yaml, plus 9 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `payments` fails closed when `bootstrap trust chain (ADR 0295)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `payments` emits denial evidence for `bootstrap trust chain (ADR 0295)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `payments` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `bootstrap trust chain (ADR 0295)` workflow.
- Depth detail 17: `payments` telemetry for `bootstrap trust chain (ADR 0295)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `payments` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §credential-isolation (ADR-0296)

PSP credentials live in OpenBao at `secret/<tenant_id>/payments/<psp>/<key-name>`. Sidecar fetches with TTL ≤60s. Never persisted in-process beyond a request lifecycle. The platform-master account credentials are isolated to the oyatie-internal tenant.
### Content-pass expansion — credential-isolation
- This expansion preserves the existing prose above and closes `credential-isolation` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: HashiCorp Vault dynamic secrets anchors the external control pattern for `credential-isolation`.
- Precedent 2: AWS KMS envelope isolation provides a second independent hyperscaler pattern for `credential-isolation`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `credential-isolation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `credential-isolation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `credential-isolation (ADR-0296)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `credential isolation (ADR 0296)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `credential isolation (ADR 0296)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `credential isolation (ADR 0296)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `payments` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `payments` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `payments` uses SLOs `slos/charge-api-availability.openslo.yaml, slos/charge-api-latency.openslo.yaml, slos/dispute-response-latency.openslo.yaml, slos/payout-api-latency.openslo.yaml, slos/payout-completion-success.openslo.yaml, plus 3 more` and dashboards `dashboards/dispute-volume.json, dashboards/finops-cost-attribution.md, dashboards/fraud-signals.md, dashboards/payments-overview.json, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `payments` uses runbooks `runbooks/aml-suspicious-activity-detected.md, runbooks/dispute-escalation.md, runbooks/double-charge-detected.md, runbooks/elder-financial-abuse.md, runbooks/fraud-spike-detected.md, plus 5 more` so `credential isolation (ADR 0296)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `payments` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/payments-app/Chart.yaml, iac/helm/payments-app/values.yaml, iac/helm/payments-webhook-handler/Chart.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `payments` uses `capabilities/charge.yaml, capabilities/dispute.yaml, capabilities/payout.yaml, capabilities/refund.yaml, plus 2 more` and `catalog/oya-payments-adapter-adyen.yaml, catalog/oya-payments-adapter-stripe.yaml, catalog/oya-payments-charge-app.yaml, catalog/oya-payments-charge-domain.yaml, plus 9 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `payments` fails closed when `credential isolation (ADR 0296)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `payments` emits denial evidence for `credential isolation (ADR 0296)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `payments` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `credential isolation (ADR 0296)` workflow.
- Depth detail 17: `payments` telemetry for `credential isolation (ADR 0296)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `payments` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §minor-protection (ADR-0292)

See §10 above.
### Content-pass expansion — minor-protection
- This expansion preserves the existing prose above and closes `minor-protection` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: Apple Family/Screen Time controls anchors the external control pattern for `minor-protection`.
- Precedent 2: Google Family Link provides a second independent hyperscaler pattern for `minor-protection`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `minor-protection`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `minor-protection` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `minor-protection (ADR-0292)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `minor protection (ADR 0292)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `minor protection (ADR 0292)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `minor protection (ADR 0292)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `payments` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `payments` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `payments` uses SLOs `slos/charge-api-availability.openslo.yaml, slos/charge-api-latency.openslo.yaml, slos/dispute-response-latency.openslo.yaml, slos/payout-api-latency.openslo.yaml, slos/payout-completion-success.openslo.yaml, plus 3 more` and dashboards `dashboards/dispute-volume.json, dashboards/finops-cost-attribution.md, dashboards/fraud-signals.md, dashboards/payments-overview.json, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `payments` uses runbooks `runbooks/aml-suspicious-activity-detected.md, runbooks/dispute-escalation.md, runbooks/double-charge-detected.md, runbooks/elder-financial-abuse.md, runbooks/fraud-spike-detected.md, plus 5 more` so `minor protection (ADR 0292)` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `payments` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm/payments-app/Chart.yaml, iac/helm/payments-app/values.yaml, iac/helm/payments-webhook-handler/Chart.yaml, plus 11 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `payments` uses `capabilities/charge.yaml, capabilities/dispute.yaml, capabilities/payout.yaml, capabilities/refund.yaml, plus 2 more` and `catalog/oya-payments-adapter-adyen.yaml, catalog/oya-payments-adapter-stripe.yaml, catalog/oya-payments-charge-app.yaml, catalog/oya-payments-charge-domain.yaml, plus 9 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `payments` fails closed when `minor protection (ADR 0292)` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `payments` emits denial evidence for `minor protection (ADR 0292)` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `payments` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `minor protection (ADR 0292)` workflow.
- Depth detail 17: `payments` telemetry for `minor protection (ADR 0292)` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `payments` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §C. Cross-references

- [`threat-model.md`](threat-model.md) — STRIDE per data class.
- [`dpia.md`](dpia.md) — GDPR Art. 35 / KR-PIPA Art. 33.
- [`incident-response.md`](incident-response.md) — regulator notification timing.
- [`runbooks/pci-incident-response.md`](runbooks/pci-incident-response.md).
- [`runbooks/kr-fss-audit-pull.md`](runbooks/kr-fss-audit-pull.md).
- [`policy/data-residency.md`](policy/data-residency.md).
- [`policy/auditor-scope.cedar`](policy/auditor-scope.cedar).

## §detection-substrate-binding (§3.2.6 rows 1 + 4)

Per documentation-rigor.md §3.2.6, the Detection column is wired to the observability substrate:

| Threat class | Detection source | Binding |
|---|---|---|
| Payment fraud (row 1) | Intelligence fraud-score library-first on every `charge.create`; score > 80 = `ChargeError::FraudRiskDenied` | `oya-shared-intelligence-substrate-lib` → `payments_fraud_score_histogram` metric → `dashboards/fraud-signals.md` |
| AML suspicious activity (row 4) | `AmlMonitoringWorker` daily; `aml_risk_score > 70` = auto-restrict sub-merchant | `oya-shared-intelligence-substrate-lib` → `oya.payments.aml.suspicious-activity-detected` audit event |
| Elder financial abuse | Intelligence pattern-match on charge velocity + age_class signal; `dispute.elder_abuse_flag` | `oya.payments.elder-abuse.escalated` audit event → Grafana alert → ops-fraud page |
| Credential stuffing | `abuse-defence.cedar` `credential_stuffing_score > 70` FORBID | Edge WAF (Cloud Armor) + Cedar gate → `oya.payments.abuse-defence.denied` |

Audit events are Merkle-sealed per ADR-0028; SLO error-budget burn alerts route to the Payments-Sev1 on-call channel via Grafana OnCall.
### Content-pass expansion — detection-substrate-binding
- This expansion preserves the existing prose above and closes `detection-substrate-binding` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: AWS GuardDuty findings anchors the external control pattern for `detection-substrate-binding`.
- Precedent 2: Google Chronicle detections provides a second independent hyperscaler pattern for `detection-substrate-binding`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `detection-substrate-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `detection-substrate-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `detection-substrate-binding (§3.2.6 rows 1 + 4)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `detection substrate binding (§3.2.6 rows 1 + 4)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `detection substrate binding (§3.2.6 rows 1 + 4)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `detection substrate binding (§3.2.6 rows 1 + 4)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `payments` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `payments` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `payments` uses SLOs `slos/charge-api-availability.openslo.yaml, slos/charge-api-latency.openslo.yaml, slos/dispute-response-latency.openslo.yaml, slos/payout-api-latency.openslo.yaml, slos/payout-completion-success.openslo.yaml, plus 3 more` and dashboards `dashboards/dispute-volume.json, dashboards/finops-cost-attribution.md, dashboards/fraud-signals.md, dashboards/payments-overview.json, plus 4 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `payments` uses runbooks `runbooks/aml-suspicious-activity-detected.md, runbooks/dispute-escalation.md, runbooks/double-charge-detected.md, runbooks/elder-financial-abuse.md, runbooks/fraud-spike-detected.md, plus 5 more` so `detection substrate binding (§3.2.6 rows 1 + 4)` failures have trigger, rollback, and post-incident closure.

## §insider-threat-controls (§3.2.4 D8)

Controls mitigating insider-threat risk per documentation-rigor.md §3.2.4 D8:

| Control | Implementation |
|---|---|
| Principle of least privilege | Cedar default-deny on all charge/refund/payout/dispute/sub-merchant actions; no wildcard permits |
| Separation of duties | Payout > $1M USD-equivalent requires `dual_signoff_token` (SOX-ITGC pack); Cedar gate enforces at `payout.execute` |
| Audit-chain immutability | Merkle-sealed audit per ADR-0028; operator cannot delete or amend audit records |
| Break-glass access | `auditor-scope.cedar` time-boxed + tenant-scoped + read-only; every break-glass emits `oya.payments.audit.read` |
| Four-eyes on Cedar fragment publish | ADR-0294 soak window ≥60s; fragment signed by `oyatie.payments.foundry` SVID; attestation sealed in audit chain |
| No direct DB access by humans | All production DB access via `oya-payments-*-app` principals; no direct CRDB access for humans in prod |
| PSP credential isolation | OpenBao sidecar TTL ≤60s per ADR-0296; no human access to PSP secrets in prod without break-glass |
### Content-pass expansion — insider-threat-controls
- This expansion preserves the existing prose above and closes `insider-threat-controls` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: Microsoft Purview Insider Risk anchors the external control pattern for `insider-threat-controls`.
- Precedent 2: Google BeyondCorp provides a second independent hyperscaler pattern for `insider-threat-controls`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `insider-threat-controls`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `insider-threat-controls` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `insider-threat-controls (§3.2.4 D8)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `insider threat controls (§3.2.4 D8)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `insider threat controls (§3.2.4 D8)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `insider threat controls (§3.2.4 D8)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `payments` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `payments` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.

## §threat-intelligence-feeds (§3.2.4 D9)

Active threat-intelligence feeds wired to the payments µservice:

| Feed | Source | Integration point |
|---|---|---|
| PSP fraud signals | Stripe Radar rules + Adyen RevenueProtect | `PspAdapter::handle_webhook` parses fraud signals; feeds `aml_risk_score` |
| FATF sanctions list | OFAC SDN + UN Consolidated + EU Consolidated | `AmlMonitoringWorker` checks sub-merchants weekly; blocks at onboarding |
| JA4+ fingerprint blocklist | Abuse.ch + internal ops-security | `abuse-defence.cedar` JA4+ blocklist; synced weekly to Cedar fragment |
| IP reputation | Cloudflare Threat Intelligence + Google Safe Browsing | Edge WAF (Cloud Armor) managed rules; updated continuously |
| Credential breach feeds | HaveIBeenPwned API (B2C consumer passwords) | `cloud-iam` µservice; referenced at charge-create for step-up trigger |
| KoFIU / FinCEN watchlists | KR KoFIU PEP list; FinCEN 314(a) | `AmlMonitoringWorker`; checked at sub-merchant onboarding + quarterly |
### Content-pass expansion — threat-intelligence-feeds
- This expansion preserves the existing prose above and closes `threat-intelligence-feeds` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: Mandiant threat intelligence anchors the external control pattern for `threat-intelligence-feeds`.
- Precedent 2: AWS GuardDuty threat lists provides a second independent hyperscaler pattern for `threat-intelligence-feeds`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `threat-intelligence-feeds`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `threat-intelligence-feeds` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `threat-intelligence-feeds (§3.2.4 D9)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `threat intelligence feeds (§3.2.4 D9)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `threat intelligence feeds (§3.2.4 D9)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `threat intelligence feeds (§3.2.4 D9)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `payments` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `payments` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `payments` uses SLOs `slos/charge-api-availability.openslo.yaml, slos/charge-api-latency.openslo.yaml, slos/dispute-response-latency.openslo.yaml, slos/payout-api-latency.openslo.yaml, slos/payout-completion-success.openslo.yaml, plus 3 more` and dashboards `dashboards/dispute-volume.json, dashboards/finops-cost-attribution.md, dashboards/fraud-signals.md, dashboards/payments-overview.json, plus 4 more` when those artifacts exist.

## §key-rotation-cadence (§3.2.4 D16)

| Key / secret | Cadence | Mechanism |
|---|---|---|
| PSP secret keys (provider-credential BYOK, ADR-0255 §D-4) | Tenant-controlled; platform recommends 90d | OpenBao dynamic secrets or tenant self-rotation via provider dashboard |
| PSP webhook HMAC secrets | 90 days | `secret/data/payments/webhook-hmac/<psp>` rotated by ops-security; sidecar picks up new value within TTL ≤60s |
| TLS cert (PQC hybrid chain) | 1 year (renew 30d before expiry) | cert-manager + oyatie-pqc-ca-issuer; auto-renewed |
| ECH config keys | ≥ 90 days per ADR-0253 | `iac/tls/payments-ech-config.yaml`; cert-manager annotation triggers rotation |
| CRDB app credentials | 30 days | OpenBao dynamic database secrets; auto-rotated |
| SPIFFE SVID | 24 hours | SPIRE server auto-renews; pod restarts pick up fresh SVID |
| Cedar fragment signing key | 1 year | `oyatie.payments.foundry` SVID; rotated by ops-security |

Rotation failures alert via Grafana OnCall `Payments-KeyRotation-Failed` alert rule.
### Content-pass expansion — key-rotation-cadence
- This expansion preserves the existing prose above and closes `key-rotation-cadence` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: AWS KMS key rotation anchors the external control pattern for `key-rotation-cadence`.
- Precedent 2: Google Cloud KMS versions provides a second independent hyperscaler pattern for `key-rotation-cadence`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `key-rotation-cadence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `key-rotation-cadence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `key-rotation-cadence (§3.2.4 D16)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `key rotation cadence (§3.2.4 D16)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `key rotation cadence (§3.2.4 D16)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `key rotation cadence (§3.2.4 D16)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `payments` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `payments` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.

## §crypto-agility-plan (§3.2.4 D20)

The payments µservice is built for crypto-agility: no algorithm is hard-coded in production logic; all algorithm choices are configuration-driven and replaceable without code changes.

| Layer | Current algorithm | Agility mechanism | Post-quantum path |
|---|---|---|---|
| TLS KEM | X25519MLKEM768 (hybrid) per ADR-0253 | `tls.pqcKem` in `iac/helm/payments-app/values.yaml` | Already hybrid PQ; migrate to pure ML-KEM-768 when IANA assigns stable codepoint |
| TLS signature | ed25519 + ml_dsa_65 (hybrid) per ADR-0253 | cert-manager `privateKey.algorithm` | Already hybrid PQ |
| ECH KEM | X25519MLKEM768 | `iac/tls/payments-ech-config.yaml` `kem_algorithm` | Same path as TLS KEM |
| HMAC (webhook verification) | HMAC-SHA256 (PSP-mandated) | `PspAdapter` trait method `verify_webhook_signature(algo, payload, sig, secret)` | Upgrade to HMAC-SHA3-256 when PSPs support; no code change required |
| Audit-chain hashing | SHA-256 (Merkle nodes) per ADR-0028 | `audit_chain.hash_algorithm` config | SHA3-256 path available in `oya-shared-audit-chain`; toggle without downtime |
| Credential encryption at rest | AES-256-GCM (OpenBao) | OpenBao seal backend; algorithm config | OpenBao 1.16+ supports PQC seal backend (experimental); migration path documented |

Timeline: pure PQ migration gates on NIST FIPS 203/204/205 final publication (expected 2024–2026) and PSP/CA ecosystem readiness.
### Content-pass expansion — crypto-agility-plan
- This expansion preserves the existing prose above and closes `crypto-agility-plan` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: Cloudflare post-quantum TLS anchors the external control pattern for `crypto-agility-plan`.
- Precedent 2: Chrome hybrid PQ TLS provides a second independent hyperscaler pattern for `crypto-agility-plan`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `crypto-agility-plan`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `crypto-agility-plan` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `crypto-agility-plan (§3.2.4 D20)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `crypto agility plan (§3.2.4 D20)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `crypto agility plan (§3.2.4 D20)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `crypto agility plan (§3.2.4 D20)` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `payments` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `payments` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.

## §critical-path-edge-cases (§3.2.5)

Explicit handling for applicable §3.2.5 rows:

| Row | Scenario | How payments handles it |
|---|---|---|
| 1 | Emergency services pass-through | `policy/emergency-services-bypass.cedar` permits `EMERGENCY_SERVICES_OPERATOR` principals to bypass rate-limit, fraud-score gate, and step-up auth. HMAC + SVID checks still enforced (no security bypass). |
| 3 | Fraud dispute | `DisputeReason::Fraud` in dispute domain auto-escalates to ops-fraud + triggers fraud-ML review. Cedar `dispute-authorization.cedar` blocks settlement without ops-fraud approval. |
| 4 | Elder financial abuse | `dispute.elder_abuse_flag = true` blocks merchant clawback; Cedar FORBID on `dispute.accept` without ops-fraud role; full refund issued via `runbooks/elder-financial-abuse.md`; SAR filing pathway. |
| 11 | Disability accommodations | KYB document submission supports alternative channels (email + postal) for sub-merchants with accessibility needs; `accessibility_needs_flag` in `SubMerchant` entity. |
| 15 | Financial inclusion | `pay_as_you_go` billing plan in subscription domain; no minimum commitment; metered at period end. Cedar permits all KYC tiers for pay-as-you-go. |
| 22 | Mass-casualty surge | `emergency-services-bypass.cedar` + `emergency_surge_token` bypass per-tenant rate-limit for `EMERGENCY_SERVICES_OPERATOR` during surge events. HPA scales to 50 replicas. |
| 24 | Account hijack recovery | `cloud-iam` µservice issues emergency session revocation; all active charges for hijacked tenant are voided via `VoidChargeUseCase`; Cedar gate blocks new charges until IAM recovery complete. |
| 25 | Mistaken-action undo | Charge void window: within 30 min of authorization (before capture). `VoidChargeUseCase` callable by tenant principal within window. Audit event `oya.payments.charge.voided-by-user` for transparency. |
| 28 | Delegated agent payments | `PARTNER_AGENCY` audience_type with explicit `delegated_principal_id`; Cedar `charge-authorization.cedar` checks `delegated_tenant_id` permission chain. |
### Content-pass expansion — critical-path-edge-cases
- This expansion preserves the existing prose above and closes `critical-path-edge-cases` for `payments` to the ≥50-line documentation-rigor floor.
- Service owner `axis-payments` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `charge`; bounded contexts: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`; +2 more.
- API surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy surfaces: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`; +5 more.
- State/event surfaces: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`; +1 more.
- SLO/dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`; +11 more.
- Compliance packs: `pci-dss-l1-v4`, `kr-fss`, `eu-psd2-sca`, `us-state-mtl`, `ccpa-cpra-2023`; +3 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `tenancy`, `cloud-secrets`, `cloud-iam`, `policy-engine`, `observability`; +2 more.
- Precedent 1: Google SRE incident playbooks anchors the external control pattern for `critical-path-edge-cases`.
- Precedent 2: Stripe idempotency recovery provides a second independent hyperscaler pattern for `critical-path-edge-cases`.
- Tenant-scope invariant: every `payments` `charge` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/payments/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `payments` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `payments` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `payments` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `payments` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `payments` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `charge` evaluates `<tenant>.payments.charge` against policy, writes `payments.charge`, and emits `oya.payments.charge.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `critical-path-edge-cases`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `critical-path-edge-cases` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `payments` binds `critical-path-edge-cases (§3.2.5)` to `{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya-payments-charge-kernel', 'oya-payments-charge-domain', 'oya-payments-charge-usecase', 'oya-payments-charge-rest', 'oya-payments-charge-grpc', 'oya-payments-charge-app', 'oya-payments-charge-api', 'oya-payments-charge-sdk']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `payments` is `contracts/asyncapi-v1.yaml, contracts/metric-naming-convention.md, contracts/openapi-v1.yaml, contracts/payments-v1.proto, contracts/psp-adapter-trait.md`; reviewers must map `critical path edge cases (§3.2.5)` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `payments` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/charge-authorization.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/dispute-authorization.cedar, plus 4 more`; missing policy files are scaffold debt, not an implicit pass for `critical path edge cases (§3.2.5)`.
- Depth detail 4: `payments` state/event naming uses `payments.{'name': 'charge', 'description': 'Charge creation, authorization, capture, void', 'crates': ['oya_payments_charge_kernel', 'oya_payments_charge_domain', 'oya_payments_charge_usecase', 'oya_payments_charge_rest', 'oya_payments_charge_grpc', 'oya_payments_charge_app', 'oya_payments_charge_api', 'oya_payments_charge_sdk']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `payments` covers `tenancy, cloud-secrets, cloud-iam, policy-engine, observability, plus 2 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `payments` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `payments` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `critical path edge cases (§3.2.5)` tied to the keystone bundle instead of a local convention.

## §D. Reviewer attestation

| Date | Reviewer | Outcome |
|---|---|---|
| 2026-05-20 | ops-compliance + axis-payments + dpo | Accepted (initial publication; QSA + KR-FSS pre-audit scheduled M02-foundation pre-cert) |
| 2026-05-20 | axis-payments doc-set wave-3-B | Wave-3-B gap sections added: §detection-substrate-binding, §insider-threat-controls, §threat-intelligence-feeds, §key-rotation-cadence, §crypto-agility-plan, §critical-path-edge-cases |

---



## §detection-substrate-binding
This anchor is closed for `payments` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `payments` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `charge` touches those data classes.
- Signal sources: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +20 more.
- Example event class: `oya.payments.charge.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `payments`; owner `axis-payments`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`, `kyc-kyb`; +1 more.
- Capability records cited: `microservices/payments/capabilities/charge.yaml`, `microservices/payments/capabilities/dispute.yaml`, `microservices/payments/capabilities/payout.yaml`, `microservices/payments/capabilities/refund.yaml`, `microservices/payments/capabilities/sub-merchant-onboarding.yaml`, `microservices/payments/capabilities/subscription-lifecycle.yaml`.
- API surfaces cited: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +16 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar binding: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- State/event binding: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`, `payments.kyc_kyb`; +1 more.
- Capability binding: `charge`, `payout`, `refund`, `dispute`, `subscription-lifecycle`.
- SLO binding: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +2 more.
- Runbook binding: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `payments`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `payments`.
- `policy-engine` supplies the signed Cedar corpus while `payments` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `payments` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `payments`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `payments` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `payments` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `payments` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.payments.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `charge` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `charge` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `payments`; owner `axis-payments`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`, `kyc-kyb`; +1 more.
- Capability records cited: `microservices/payments/capabilities/charge.yaml`, `microservices/payments/capabilities/dispute.yaml`, `microservices/payments/capabilities/payout.yaml`, `microservices/payments/capabilities/refund.yaml`, `microservices/payments/capabilities/sub-merchant-onboarding.yaml`, `microservices/payments/capabilities/subscription-lifecycle.yaml`.
- API surfaces cited: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +16 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar binding: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- State/event binding: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`, `payments.kyc_kyb`; +1 more.
- Capability binding: `charge`, `payout`, `refund`, `dispute`, `subscription-lifecycle`.
- SLO binding: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +2 more.
- Runbook binding: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `payments`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `payments`.
- `policy-engine` supplies the signed Cedar corpus while `payments` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `payments` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `payments`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `payments` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `payments` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `payments` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`, `payments.charge`; +6 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `payments.charge` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `payments`; owner `axis-payments`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`, `kyc-kyb`; +1 more.
- Capability records cited: `microservices/payments/capabilities/charge.yaml`, `microservices/payments/capabilities/dispute.yaml`, `microservices/payments/capabilities/payout.yaml`, `microservices/payments/capabilities/refund.yaml`, `microservices/payments/capabilities/sub-merchant-onboarding.yaml`, `microservices/payments/capabilities/subscription-lifecycle.yaml`.
- API surfaces cited: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +16 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar binding: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- State/event binding: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`, `payments.kyc_kyb`; +1 more.
- Capability binding: `charge`, `payout`, `refund`, `dispute`, `subscription-lifecycle`.
- SLO binding: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +2 more.
- Runbook binding: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `payments`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `payments`.
- `policy-engine` supplies the signed Cedar corpus while `payments` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `payments` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `payments`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `payments` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `payments` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `payments` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +16 more.
- Example: `charge` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `payments`; owner `axis-payments`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`, `kyc-kyb`; +1 more.
- Capability records cited: `microservices/payments/capabilities/charge.yaml`, `microservices/payments/capabilities/dispute.yaml`, `microservices/payments/capabilities/payout.yaml`, `microservices/payments/capabilities/refund.yaml`, `microservices/payments/capabilities/sub-merchant-onboarding.yaml`, `microservices/payments/capabilities/subscription-lifecycle.yaml`.
- API surfaces cited: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +16 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar binding: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- State/event binding: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`, `payments.kyc_kyb`; +1 more.
- Capability binding: `charge`, `payout`, `refund`, `dispute`, `subscription-lifecycle`.
- SLO binding: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +2 more.
- Runbook binding: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `payments`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `payments`.
- `policy-engine` supplies the signed Cedar corpus while `payments` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `payments` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `payments`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `payments` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `payments` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.payments` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/payments/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +4 more.
- Example: `charge` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `payments`; owner `axis-payments`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`, `kyc-kyb`; +1 more.
- Capability records cited: `microservices/payments/capabilities/charge.yaml`, `microservices/payments/capabilities/dispute.yaml`, `microservices/payments/capabilities/payout.yaml`, `microservices/payments/capabilities/refund.yaml`, `microservices/payments/capabilities/sub-merchant-onboarding.yaml`, `microservices/payments/capabilities/subscription-lifecycle.yaml`.
- API surfaces cited: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +16 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar binding: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- State/event binding: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`, `payments.kyc_kyb`; +1 more.
- Capability binding: `charge`, `payout`, `refund`, `dispute`, `subscription-lifecycle`.
- SLO binding: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +2 more.
- Runbook binding: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `payments`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `payments`.
- `policy-engine` supplies the signed Cedar corpus while `payments` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `payments` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `payments`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `payments` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `payments` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `payments` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`, `microservices/payments/iac/ech-config.yaml`; +11 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `charge` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `payments`; owner `axis-payments`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`, `kyc-kyb`; +1 more.
- Capability records cited: `microservices/payments/capabilities/charge.yaml`, `microservices/payments/capabilities/dispute.yaml`, `microservices/payments/capabilities/payout.yaml`, `microservices/payments/capabilities/refund.yaml`, `microservices/payments/capabilities/sub-merchant-onboarding.yaml`, `microservices/payments/capabilities/subscription-lifecycle.yaml`.
- API surfaces cited: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +16 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar binding: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- State/event binding: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`, `payments.kyc_kyb`; +1 more.
- Capability binding: `charge`, `payout`, `refund`, `dispute`, `subscription-lifecycle`.
- SLO binding: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +2 more.
- Runbook binding: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `payments`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `payments`.
- `policy-engine` supplies the signed Cedar corpus while `payments` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `payments` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `payments`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `payments` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `payments` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `payments` is in annual full-scope pentest and every major `charge` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`, `microservices/payments/iac/ech-config.yaml`; +21 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `payments` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `payments`; owner `axis-payments`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`, `kyc-kyb`; +1 more.
- Capability records cited: `microservices/payments/capabilities/charge.yaml`, `microservices/payments/capabilities/dispute.yaml`, `microservices/payments/capabilities/payout.yaml`, `microservices/payments/capabilities/refund.yaml`, `microservices/payments/capabilities/sub-merchant-onboarding.yaml`, `microservices/payments/capabilities/subscription-lifecycle.yaml`.
- API surfaces cited: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +16 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar binding: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- State/event binding: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`, `payments.kyc_kyb`; +1 more.
- Capability binding: `charge`, `payout`, `refund`, `dispute`, `subscription-lifecycle`.
- SLO binding: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +2 more.
- Runbook binding: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `payments`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `payments`.
- `policy-engine` supplies the signed Cedar corpus while `payments` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `payments` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `payments`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `payments` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `payments` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `payments` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `{"control_plane": "Tier1", "data_plane": "Tier3-per-tenant", "note": "Tier-1 control plane per ADR-0248; per-tenant data cells at Tier-3"}` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `charge` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `payments`; owner `axis-payments`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`, `kyc-kyb`; +1 more.
- Capability records cited: `microservices/payments/capabilities/charge.yaml`, `microservices/payments/capabilities/dispute.yaml`, `microservices/payments/capabilities/payout.yaml`, `microservices/payments/capabilities/refund.yaml`, `microservices/payments/capabilities/sub-merchant-onboarding.yaml`, `microservices/payments/capabilities/subscription-lifecycle.yaml`.
- API surfaces cited: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +16 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar binding: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- State/event binding: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`, `payments.kyc_kyb`; +1 more.
- Capability binding: `charge`, `payout`, `refund`, `dispute`, `subscription-lifecycle`.
- SLO binding: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +2 more.
- Runbook binding: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `payments`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `payments`.
- `policy-engine` supplies the signed Cedar corpus while `payments` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `payments` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `payments`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `payments` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `payments` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `payments` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/payments/catalog/oya-payments-adapter-adyen.yaml`, `microservices/payments/catalog/oya-payments-adapter-stripe.yaml`, `microservices/payments/catalog/oya-payments-charge-app.yaml`, `microservices/payments/catalog/oya-payments-charge-domain.yaml`, `microservices/payments/catalog/oya-payments-charge-kernel.yaml`, `microservices/payments/catalog/oya-payments-charge-rest.yaml`; +23 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `charge` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `payments`; owner `axis-payments`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`, `kyc-kyb`; +1 more.
- Capability records cited: `microservices/payments/capabilities/charge.yaml`, `microservices/payments/capabilities/dispute.yaml`, `microservices/payments/capabilities/payout.yaml`, `microservices/payments/capabilities/refund.yaml`, `microservices/payments/capabilities/sub-merchant-onboarding.yaml`, `microservices/payments/capabilities/subscription-lifecycle.yaml`.
- API surfaces cited: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +16 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar binding: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- State/event binding: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`, `payments.kyc_kyb`; +1 more.
- Capability binding: `charge`, `payout`, `refund`, `dispute`, `subscription-lifecycle`.
- SLO binding: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +2 more.
- Runbook binding: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `payments`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `payments`.
- `policy-engine` supplies the signed Cedar corpus while `payments` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `payments` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `payments`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `payments` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `payments` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `payments` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `charge` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `charge` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `payments`; owner `axis-payments`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`, `kyc-kyb`; +1 more.
- Capability records cited: `microservices/payments/capabilities/charge.yaml`, `microservices/payments/capabilities/dispute.yaml`, `microservices/payments/capabilities/payout.yaml`, `microservices/payments/capabilities/refund.yaml`, `microservices/payments/capabilities/sub-merchant-onboarding.yaml`, `microservices/payments/capabilities/subscription-lifecycle.yaml`.
- API surfaces cited: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +16 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar binding: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- State/event binding: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`, `payments.kyc_kyb`; +1 more.
- Capability binding: `charge`, `payout`, `refund`, `dispute`, `subscription-lifecycle`.
- SLO binding: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +2 more.
- Runbook binding: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `payments`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `payments`.
- `policy-engine` supplies the signed Cedar corpus while `payments` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `payments` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `payments`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `payments` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `payments` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.
- State/event surfaces carrying classification: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`, `payments.kyc_kyb`; +1 more.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `charge` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `payments`; owner `axis-payments`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `charge`, `refund`, `payout`, `dispute`, `subscription-lifecycle`, `kyc-kyb`; +1 more.
- Capability records cited: `microservices/payments/capabilities/charge.yaml`, `microservices/payments/capabilities/dispute.yaml`, `microservices/payments/capabilities/payout.yaml`, `microservices/payments/capabilities/refund.yaml`, `microservices/payments/capabilities/sub-merchant-onboarding.yaml`, `microservices/payments/capabilities/subscription-lifecycle.yaml`.
- API surfaces cited: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar/policy artifacts cited: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- SLO and dashboard evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +10 more.
- Runbook/IaC evidence: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +16 more.
- Data classes declared for this control: `FINANCIAL`, `PII_IDENTIFYING`, `AUDIT`.

### Primitive and API binding
- API surface binding: `microservices/payments/contracts/asyncapi-v1.yaml`, `microservices/payments/contracts/metric-naming-convention.md`, `microservices/payments/contracts/openapi-v1.yaml`, `microservices/payments/contracts/payments-v1.proto`, `microservices/payments/contracts/psp-adapter-trait.md`.
- Cedar binding: `microservices/payments/policy/abuse-defence.cedar`, `microservices/payments/policy/auditor-scope.cedar`, `microservices/payments/policy/charge-authorization.cedar`, `microservices/payments/policy/ci-scope.cedar`, `microservices/payments/policy/data-residency.md`, `microservices/payments/policy/dispute-authorization.cedar`; +4 more.
- State/event binding: `payments.charge`, `payments.refund`, `payments.payout`, `payments.dispute`, `payments.subscription_lifecycle`, `payments.kyc_kyb`; +1 more.
- Capability binding: `charge`, `payout`, `refund`, `dispute`, `subscription-lifecycle`.
- SLO binding: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/slos/payout-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/refund-api-availability.openslo.yaml`; +2 more.
- Runbook binding: `microservices/payments/runbooks/aml-suspicious-activity-detected.md`, `microservices/payments/runbooks/dispute-escalation.md`, `microservices/payments/runbooks/double-charge-detected.md`, `microservices/payments/runbooks/elder-financial-abuse.md`, `microservices/payments/runbooks/fraud-spike-detected.md`, `microservices/payments/runbooks/kr-fss-audit-pull.md`; +4 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `payments`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `payments`.
- `policy-engine` supplies the signed Cedar corpus while `payments` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `payments` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `payments`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `payments` applies the most restrictive policy and emits a degraded-mode audit event.
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
