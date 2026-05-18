---
doc_class: IncidentResponse
microservice: forms
status: Accepted
date: 2026-05-17
owner_team: ops-sre-reliability + ops-security + axis-forms
doc_status: published
---

# Forms — Incident Response Playbook

## On-call rotation

Forms on-call follows the oyatie standard (24/7; 3-tier escalation; ops-security included for any Sev-1 with privacy axis).

## Sev classification

| Sev | Definition | Page |
|---|---|---|
| Sev-1 (P0) | Tenant data loss, cross-tenant leak, PHI/PII breach, cluster-wide outage | Immediate page; ops-sre + ops-security + council-privacy + axis-forms |
| Sev-2 | Multi-tenant degradation, captcha degradation, single-pack outage | Page within 15 min |
| Sev-3 | Single-tenant degradation, queue backlog, T2 quality drop | Slack ping + ticket |
| Sev-4 | Internal-tool degradation, dashboard render slow | Ticket only |

## Sev-1 protocol (selected)

### PII leak (FM-06; P0)

1. **Detect**: `oya_forms_export_pii_unredacted_total > 0` OR external report.
2. **Stop the bleed**: revoke affected export tokens; lock affected forms (Cedar policy mode `LOCKED`).
3. **Forensics**: identify scope (which tenants, which subjects); preserve evidence.
4. **Notify**:
   - Pack-eu: data-controller tenant within 72h (GDPR Art. 33); affected subjects without undue delay (Art. 34).
   - Pack-us-hc: tenant (Covered Entity) within 60d (HIPAA §164.404); HHS Secretary; subjects.
   - Pack-kr: PIPC within 72h (PIPA Art. 34); subjects.
   - Pack-br: ANPD (LGPD Art. 48).
5. **Remediate**: per `runbooks/pii-leak-incident-p0.md`.
6. **Postmortem**: blameless; within 5 business days; publish in evidence ledger.
7. **Regulatory follow-up**: per pack DPO.

### Cross-pack write attempt (FM-15)

1. **Detect**: `oya_forms_cross_pack_write_attempt_total > 0`.
2. **Quarantine**: identify the writer (could be misconfigured tenant tooling OR insider).
3. **Verify**: was data actually written cross-pack? If yes ⇒ Sev-1 data-residency breach; treat as PII leak.
4. **Block source**: revoke source credentials; engage ops-security.
5. **Notify**: pack supervisory authority per residency contract.

### Cluster-wide outage

1. **Detect**: `oya_forms_visual_canvas_rest_up{pack=*}` == 0 across pack.
2. **Status page**: update status.oyatie.dev within 5 min.
3. **DR failover** (active-passive packs): trigger per `multi-region.md`.
4. **RTO target**: ≤ 15min.
5. **Tenant comms**: per pack.

## Postmortem template

- Timeline (UTC).
- Root cause (per "5 whys").
- Contributing factors.
- Detection mechanism (and missed signals).
- Impact (scope, duration, affected tenants).
- Remediation actions (with owners + due dates).
- Prevention actions.
- Lessons learned.

## Drill cadence

- Sev-1 tabletop quarterly.
- Sev-2 chaos drill quarterly (induce FM-02 + FM-09).
- Full Sev-1 drill (PII leak simulation) annually.

## Evidence

Every incident produces:
- Audit-chain seals (forensic evidence).
- Postmortem markdown at `evidence/incidents/<incident-id>.md`.
- Regulatory notification copies under `legal/incidents/<incident-id>/`.

## References

- `failure-modes.md`.
- All runbooks under `runbooks/`.
- GDPR Arts. 33-34.
- HIPAA §164.404.
- KR PIPA Art. 34.
- LGPD Art. 48.
- Google SRE Workbook ch. 15 (Postmortems).
