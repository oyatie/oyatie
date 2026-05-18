---
doc_class: IncidentResponse
template_id: TPL-INCIDENT-RESPONSE
microservice: sites
status: Accepted
date: 2026-05-17
owner_team: axis-sites + ops-sre-reliability + ops-security
related_adrs: [ADR-0117, ADR-0131, ADR-SITES-0003, ADR-SITES-0004]
doc_status: published
---

# Incident Response — sites µservice

## Purpose

Define triage, escalation, and notification flows when a sites
incident materialises. Aligned with `runbooks/*.md` (per-symptom
playbooks).

## Severity classes

| Class | Definition | Examples |
|---|---|---|
| Sev-1 | Customer impact, full or partial outage of public-facing pages | CDN edge cache poisoning; cross-tenant search index leak; cert expiration causing TLS errors |
| Sev-2 | Editor-path outage; degraded but recoverable; single tenant | publish-pipeline stuck for one tenant; ACME renewal failure with 7+ days to expiry |
| Sev-3 | Operational toil; no end-user impact | one image-optimize worker OOM; transient Loro relay flap |

## First-responder rotation

- axis-sites on-call (primary).
- ops-sre-reliability on-call (substrate).
- ops-security on-call (Sev-1 + cross-tenant + privacy).
- council-privacy DPO (privacy-breach trigger).

## Notification timelines

### Internal

| Sev | Page | Slack channel | Status page |
|---|---|---|---|
| Sev-1 | Yes (PagerDuty `oya-sites-sev1`) | `#oya-sites-incident` | Update within 10 min |
| Sev-2 | Yes (PagerDuty `oya-sites-sev2`) | `#oya-sites-incident` | Update within 30 min |
| Sev-3 | No (ticket only) | `#oya-sites-ops` | n/a |

### External (regulatory)

| Trigger | Timeline | Recipient | Citation |
|---|---|---|---|
| Personal-data breach affecting EU subjects | 72h | EDPB-equivalent + tenant (controller) | GDPR Art. 33 |
| Personal-data breach affecting KR subjects | 72h | PIPC + tenant | KR PIPA Art. 34 |
| PHI breach (pack-us-healthcare) | 60d (individuals) + concurrently to HHS | HHS OCR + affected individuals | HIPAA 45 CFR §164.404, §164.410 |
| APPI leak (pack-jp) | 3 business days | PPC + tenant | APPI Art. 22 |
| LGPD breach (pack-br) | reasonable timeframe (often 48h) | ANPD + data subjects | LGPD Art. 48 |
| DPDPA breach (pack-in) | as prescribed | Data Protection Board | DPDPA §8(6) |
| EU DSA significant-risk event | per VLOSE/VLOPS threshold (currently sites is below threshold) | Digital Services Coordinator | EU DSA Art. 18 |
| NIS2 significant incident (pack-eu) | early warning 24h + report 72h + final 1mo | National CSIRT | NIS2 Art. 23 |

## Incident lifecycle

### 1. Detect

- PrometheusRule (per `iac/helm/templates/prometheusrule.yaml`).
- Per-runbook symptom signature.
- Tenant report via support.
- Pen-test / red-team finding.

### 2. Triage

- Confirm symptom; identify affected scope (tenants × packs × BCs).
- Classify Sev-1 / Sev-2 / Sev-3.
- Establish IC (incident commander) — typically axis-sites on-call.

### 3. Mitigate

- Refer to the relevant runbook:
  - `runbooks/publish-pipeline-rollback.md`
  - `runbooks/acme-cert-renewal-failure.md`
  - `runbooks/cdn-cache-purge-cascade.md`
  - `runbooks/custom-domain-dns-drift.md`
  - `runbooks/asset-optimization-degraded.md`
  - `runbooks/page-export-corruption.md`
  - `runbooks/ai-page-build-rollback.md`

### 4. Communicate

- Status-page update per cadence above.
- Slack channel running log.
- Affected-tenant per-incident email.
- Regulatory notification per trigger above.

### 5. Resolve

- Apply fix-up ChangeSet against `dev`.
- Verify SLO recovery (burn rate normalising).
- Close incident in PagerDuty.

### 6. Post-incident

- Within 5 business days: blameless post-mortem.
- Output: action items (tickets) + runbook updates + ADR follow-ups
  if needed.
- Long-tail tracking via `oya-dev-cli postmortem`.

## Specific paths

### TLS cert expiration (Sev-1)

- Detection: `oya_sites_cert_expiry_seconds < 86400 * 7` (7-day pre-expiry alarm) → ticket; `< 86400` → page.
- Mitigation: trigger immediate ACME renewal; refer
  `runbooks/acme-cert-renewal-failure.md`.

### Cross-tenant search-index leak (Sev-1)

- Detection: LEAN check `oya-check-search-index-tenant-scope` catches at CI; runtime alarm
  `oya_sites_search_cross_tenant_result_total > 0`.
- Mitigation: page ops-security; isolate tenant index; rebuild from Postgres.

### Custom-domain DNS drift (Sev-2)

- Detection: scheduled DNS verify job emits drift event.
- Mitigation: refer `runbooks/custom-domain-dns-drift.md`.

### AI-page-build hostile prompt → unsafe output (Sev-1 or Sev-2 depending on tenant)

- Detection: tenant report; runtime safety classifier flag.
- Mitigation: refer `runbooks/ai-page-build-rollback.md`; refuse subsequent T2 calls for tenant pending review.

## Privacy-breach decision tree

```
Breach detected
├── Personal data involved? — No → operational incident only
├── Yes → DPO notified within 4h
│   ├── EU subjects? → 72h notice to EDPB + tenant; consider data-subject notice
│   ├── KR subjects? → 72h notice to PIPC + tenant
│   ├── PHI? → 60d notice (HIPAA)
│   ├── JP subjects? → 3 business days PPC
│   ├── Other? → per pack-specific timeline
│   └── In all cases: forensic snapshot → audit-chain attestation
└── Public-facing site content tampered? → Page + ops-security + immediate cdn purge + cert revoke evaluation
```

## References

- ADR-0117, ADR-0131, ADR-SITES-0003, ADR-SITES-0004.
- `runbooks/*.md`.
- GDPR Art. 33-34.
- KR PIPA Art. 34.
- HIPAA 45 CFR §164.404, §164.410.
- APPI Art. 22.
- LGPD Art. 48.
- DPDPA §8(6).
- EU DSA Art. 18.
- NIS2 Art. 23.
- Google SRE Workbook ch. 9 (incident management).
