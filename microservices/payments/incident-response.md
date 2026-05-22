---
doc_class: IncidentResponse
template_id: TPL-INCIDENT-RESPONSE
microservice: payments
status: Accepted
date: 2026-05-20
owner_team: axis-payments + ops-security + council-finance + dpo
related_adrs: [ADR-0028, ADR-0244, ADR-0251, ADR-0263]
companion_docs:
  - microservices/payments/threat-model.md
  - microservices/payments/compliance.md
  - microservices/payments/failure-modes.md
  - microservices/payments/runbooks/pci-incident-response.md
  - microservices/payments/runbooks/dispute-escalation.md
diataxis_quadrant: how-to
doc_status: published
---

# Incident Response — payments µservice

> Per-pack regulator-notification timing, escalation chains, evidence retention. Covers PCI-DSS, KR-FSS, EU PSD2, US state MTL, AU AML, BR LGPD, IN RBI, CN PIPL.

---

## §1. Incident classification

| Class | Definition | Examples |
|---|---|---|
| `IR-FIN-CRITICAL` | Financial loss / liability >$100k OR multi-tenant ledger corruption | Cross-tenant payout misroute (FM-13), audit-chain tampering (FM-11), Cedar fragment drift exposing cross-tenant data (FM-12) |
| `IR-SEC-CRITICAL` | Security incident: data breach, credential compromise, unauthorised access | PAN exposure (T-I-01), OpenBao credential compromise (T-E-02), live Cedar tampering (T-T-03) |
| `IR-AVAIL-MAJOR` | Sustained outage of charge / refund / payout (>30 min in any pack region) | PSP outage cascade (FM-01), idempotency-store outage (FM-15), OpenBao outage (FM-16) |
| `IR-COMP-VIOLATION` | Pack-overlay violation: SCA bypass, KR-FSS audit-trail gap, CN-PIPL data-egress | Cross-border egress from cn-1 cell; SCA-skipped EU charge >threshold |
| `IR-FRAUD-PATTERN` | Suspected fraud / sub-merchant abuse pattern | Mass-chargeback (FM-05), KYC-revoked sub-merchant continuing to receive funds |
| `IR-DATA-RIGHTS` | GDPR / KR-PIPA / LGPD / CPRA subject-rights gap | Subject-access-request gone wrong; over-disclosure |

## §2. Severity → trigger time → owner

| Severity | Trigger time | Page chain | Owner |
|---|---|---|---|
| Sev-0 (catastrophic) | Immediate (<5 min) | On-call → ops-security director → CTO → CEO | ops-security + axis-payments + council-finance |
| Sev-1 | Immediate (<5 min) | On-call → ops-security director → director-of-payments | ops-security + axis-payments |
| Sev-2 | Within 15 min | On-call → director-of-payments | axis-payments + ops-sre-reliability |
| Sev-3 | Within 1 h | On-call | axis-payments |
| Sev-4 | Next business day | — | axis-payments |

## §3. Per-pack regulator notification

The clock starts at the moment the incident is **confirmed** (not detected).

### 3.1 PCI-DSS L1 v4

- **Card-data exposure / loss / theft**: Notify card-networks (Visa, Mastercard, Amex, Discover, JCB), acquiring bank, and QSA **within 24h** of confirmation.
- Forensic investigation per PCI Forensic Investigator (PFI) framework.
- Required artifacts: incident log (audit-chain replay), affected-cards count, attack vector, remediation timeline.
- Reference: PCI DSS v4 Req 12.10.

### 3.2 KR-FSS (Electronic Financial Transaction Act §21-3)

- **Any e-finance security incident** affecting ≥1 user: notify **KR-FSS within 24h**.
- KR-FSS may demand 사고조사 (incident investigation) within 7 days.
- Required artifacts: KR-localised incident log; user-impact list; root-cause analysis.

### 3.3 EU PSD2 Art. 96

- **Major operational or security incident**: notify the home-member-state competent authority **within 4h of detection** for high-severity; 24h for moderate.
- Initial report → intermediate report (3 business days) → final report (within 2 weeks).
- ECB-EBA template.

### 3.4 GDPR Art. 33

- **Personal-data breach likely to result in risk** to rights / freedoms: notify supervisory authority **within 72h**.
- If high-risk to subjects: notify subjects directly per Art. 34.
- Reference: EDPB Guidelines 9/2022.

### 3.5 US State MTL + state breach laws

- Per-state notification (NY DFS, California, Massachusetts, etc.) — typically **72h** for breach to attorney general; subject notification per state breach law.

### 3.6 AU AUSTRAC + Privacy Act

- AUSTRAC Suspicious Matter Report **within 3 business days** of suspicion.
- OAIC Notifiable Data Breach scheme **within 30 days**.

### 3.7 BR BACEN + LGPD Art. 48

- BACEN cyber-incident: **within 24h**.
- ANPD breach notification: **within 2 business days**.

### 3.8 IN RBI

- RBI cyber-incident report: **within 6h** of detection.
- CERT-In: **within 6h**.

### 3.9 CN PIPL Art. 57

- PBoC notification for payment-incidents: **within 24h**.
- CAC for cross-border data incidents: **within 24h**.

### 3.10 EU AI Act Art. 73 (post-2026-08)

- Where fraud-ML is in scope: serious-incident notification **within 15 days**; **within 2 days** for widespread infringement.

## §4. Internal escalation chain

```text
Detection (alarm / on-call)
  │
  ▼
On-call engineer (paged via Grafana OnCall)
  │ (15 min triage)
  ▼
Tech-lead-on-rotation (axis-payments)
  │ (assess severity)
  ▼
Director-of-payments + ops-security director (Sev-1+)
  │ (30 min)
  ▼
CTO + DPO (Sev-0)
  │ (incident-command established)
  ▼
CEO + Legal (Sev-0 with regulatory implications)
```

## §5. Communication channels

| Channel | Use |
|---|---|
| `#payments-oncall` Slack | Day-to-day ops |
| `#inc-payments-<id>` Slack | Per-incident channel |
| Status page `status.oyatie.dev/payments` | Public-facing |
| Per-tenant webhook `status-events` | Per-tenant push notification |
| Email to ops-security distribution | Regulatory / legal coordination |
| PagerDuty escalation policy "Payments-Sev1" | Pager flow |

## §6. Evidence retention

| Artifact | Retention | Where |
|---|---|---|
| Audit-chain seals during incident window | 10 years | governance µservice (Merkle root) |
| OpenBao audit log | 7 years | OpenBao audit sink → audit-chain |
| Kubernetes audit log | 7 years | observability cluster |
| Application logs | 90 days hot / 7 years cold | Loki + S3 IA |
| PSP-side webhook receipts | 7 years | charges table `metadata` JSONB |
| Post-mortem | indefinite | `docs/post-mortems/` |

## §7. Post-mortem cadence

- Post-mortem complete within **5 business days** of incident close.
- Blameless format per Google SRE Workbook ch. 13.
- Posted to `docs/post-mortems/<YYYYMMDD>-payments-<slug>.md`.
- Action items tracked in `oya-foundry-pipeline://post-mortem-action-items`.

## §8. Communication templates

### 8.1 Internal status update

```text
[Sev-N] payments-<bc>: <one-line summary>
Detected: 14:32 UTC
Confirmed: 14:35 UTC
Customer-impact: <none / partial-region / global>
ETA: <best-estimate>
Lead: <on-call-name>
Status-page updated: yes/no
```

### 8.2 Customer-facing (status page)

```text
Title: Degraded charge processing in EU
Severity: Major
Time-detected: 14:32 UTC, 2026-05-20
Affected: EU-pinned tenants on Adyen-routed charges
Current status: Investigating
Workaround: Tenants with multi-PSP policy fall over to Stripe EU automatically; tenants with single-PSP policy may see degraded availability.
ETA: Update in 30 min.
```

### 8.3 Regulator notification (PCI example)

Per QSA template; includes: incident-id, detection-time, confirmation-time, affected-cards-count, attack-vector-hypothesis, remediation-steps-taken, root-cause-analysis-eta.

## §9. Drill cadence

- Quarterly Sev-1 drill (table-top): one of the FM-XX failure modes randomly selected.
- Annual Sev-0 drill (live-fire): includes pager, regulator-mock-notification, status-page-update, customer-comm.

## §10. References

- [`threat-model.md`](threat-model.md).
- [`compliance.md`](compliance.md).
- [`failure-modes.md`](failure-modes.md).
- [`runbooks/pci-incident-response.md`](runbooks/pci-incident-response.md).
- [PCI DSS v4 Req 12.10 — Incident Response](https://pcisecuritystandards.org).
- [GDPR Art. 33-34](https://gdpr-info.eu).
- [EU PSD2 Art. 96 + EBA Guidelines](https://eba.europa.eu).
- [Google SRE Workbook ch. 13 — Postmortem Culture](https://sre.google/workbook/postmortem-culture/).
