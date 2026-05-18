---
doc_class: IncidentResponse
title: Incident response playbook
microservice: translate
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-translate + ops-security + council-privacy
related_adrs: [ADR-0117, ADR-0130, ADR-0131, ADR-TRANSLATE-0004]
related_artifacts:
  - microservices/translate/failure-modes.md
  - microservices/translate/threat-model.md
  - microservices/translate/compliance.md
  - microservices/translate/policy/data-residency.md
  - microservices/translate/runbooks/
review_cadence: annually + after every Sev-1 + after every regulator-notifiable incident
doc_status: published
---

# Incident Response — translate µservice

## Severity Classification

| Severity | Criteria | Response time | Escalation |
|---|---|---|---|
| **Sev-1 (P0)** | Cross-region leak (R-02) / cross-tenant TM (R-08) / cross-pack policy bypass (FM-73) / credential leak (T-01) / data class boundary breached / regulator-notifiable | ≤ 15 min ack; ≤ 5 min halt | IC + DPO + ops-security + axis-translate + council-privacy; CEO if regulator-notifiable |
| **Sev-2** | Multi-tenant degradation; vendor outage (single vendor); QE regression; document parse cascade | ≤ 30 min ack | IC + on-call + axis-translate |
| **Sev-3** | Single-tenant degradation; per-tenant rate-limit cascade | ≤ 60 min ack | on-call |
| **Sev-4** | Documentation drift; non-blocking warning | next business day | engineer |

## Common Roles

- **IC (Incident Commander)** — declares severity, runs the incident, drives runbook execution.
- **PrivacyLead** — council-privacy member; consulted if data-class boundary may have been crossed.
- **CommsLead** — ops-comms; drafts tenant + regulator notifications.
- **EngLead** — axis-translate engineer on-call; executes runbook.
- **SecLead** — ops-security; consulted on credential / sandbox / cross-region / cross-tenant events.

## Sev-1 Master Playbook

| Step | Action | Time |
|---|---|---|
| 1 | Detect: monitoring alert or human report | t = 0 |
| 2 | IC declares Sev-1; opens `#inc-translate-<id>` channel | ≤ 5 min |
| 3 | IC + EngLead identify FM-### from `failure-modes.md` | ≤ 5 min |
| 4 | Execute runbook §"Halt" (e.g., halt egress, demote all engines, pause bulk-jobs) | ≤ 5 min |
| 5 | PrivacyLead engaged; preliminary data-class boundary assessment | ≤ 15 min |
| 6 | SecLead engaged; preliminary credential / sandbox / Cedar policy posture review | ≤ 15 min |
| 7 | CommsLead drafts tenant + regulator notification per `compliance.md` §"Breach Notification" | ≤ 60 min |
| 8 | Runbook execution per FM-###; recovery; verification per runbook §"Verification" | per runbook |
| 9 | Post-incident: postmortem within 5 business days; controls update; LEAN-lane refinement | ≤ 5 d |
| 10 | Council-privacy + council-architecture review at next monthly cadence | ≤ 30 d |

## Cross-Region Leak (FM-70 / R-02) — special P0 protocol

This is the highest-criticality incident for translate µservice. Every sovereign tenant's contract hangs on "no cross-border inference" being invariant.

| Step | Action | Time |
|---|---|---|
| 1 | Detect: `oya_translate_residency_violation_total > 0` canary triggers Page-1 | t = 0 |
| 2 | IC declares Sev-1 (P0); pages CEO + DPO + council-privacy + council-architecture | ≤ 15 min |
| 3 | Halt all engine egress: `cargo run -p oya-dev-cli -- translate halt-egress --pack <pack> --reason "<id>"` | ≤ 5 min |
| 4 | Quarantine offending segment hash; isolate from TM | ≤ 5 min |
| 5 | Identify exposed tenant(s) + data-class boundary crossed | ≤ 30 min |
| 6 | Regulator notification clock starts (per `compliance.md` §"Breach Notification"): PIPC (KR) 72h, EU DPA 72h, IN DPB 72h, HHS (HIPAA) 60d | ≤ 1 h preparation |
| 7 | Execute `runbooks/sovereign-tenant-cross-region-leak-incident-p0.md` end-to-end | per runbook |
| 8 | Post-incident: root-cause class identified; LEAN-lane `oya-translate-data-residency-correctness` refined; ADR amendment if needed | ≤ 5 d |

## Cross-Tenant Breach (FM-13 / R-08) — special P0 protocol

| Step | Action | Time |
|---|---|---|
| 1 | Detect: `oya_translate_tm_cross_tenant_match_total > 0` canary | t = 0 |
| 2 | IC declares Sev-1; pages DPO + ops-security | ≤ 15 min |
| 3 | Halt TM leverage path: `cargo run -p oya-dev-cli -- translate halt-tm --pack <pack>` | ≤ 5 min |
| 4 | Identify affected tenants (A, B); preserve evidence | ≤ 30 min |
| 5 | Notify both tenants per DPA SLAs | per DPA |
| 6 | Postmortem; Cedar policy audit; LEAN-lane refinement | ≤ 5 d |

## Credential Compromise (FM-81 / T-01)

Inherited from foundry-providers' `runbooks/credential-rotation.md` (emergency path).

| Step | Action | Time |
|---|---|---|
| 1 | Detect: credential canary triggers (logs/error/git regex sweep) | t = 0 |
| 2 | IC declares Sev-1 | ≤ 15 min |
| 3 | Emergency rotation (≤ 5 min) per `cloud-secrets` / `foundry-providers` runbook | ≤ 5 min |
| 4 | Audit OpenBao access log; identify resolution pattern | ≤ 30 min |
| 5 | Tenant notification if vendor-side log exposure suspected | ≤ 24 h |
| 6 | Postmortem | ≤ 5 d |

## Document-Sandbox Escape (FM-43)

If gVisor seccomp violation observed:

| Step | Action | Time |
|---|---|---|
| 1 | IC declares Sev-1 (escape attempt; potentially exploit-grade) | ≤ 15 min |
| 2 | Quarantine offending document hash; halt doc-translate worker pod | ≤ 5 min |
| 3 | SecLead + EngLead identify CVE class (Pandoc / LibreOffice / format-parser) | ≤ 60 min |
| 4 | Pin previous sandbox image version if patch not yet available | ≤ 30 min |
| 5 | Coordinate with vendor (Pandoc / LibreOffice maintainers) if novel CVE | per vendor |
| 6 | Postmortem + threat-model T-06 refresh | ≤ 5 d |

## Regulator Notification Templates

Templates live in `compliance.md` §"Breach Notification" + `policy/data-residency.md` §"Per-Pack Overlay Sections". CommsLead uses these per-pack templates; do not free-draft per incident.

### Per-pack notification SLAs

| Pack | SLA | Authority |
|---|---|---|
| pack-kr | 72 h | PIPC |
| pack-eu | 72 h | Lead EU DPA |
| pack-us | per-state PII laws | various |
| pack-us-healthcare | 60 d | HHS + affected |
| pack-jp | per APPI Art. 24 | PPC |
| pack-sg | per PDPC Notification | PDPC |
| pack-au | per OAIC NDB scheme | OAIC |
| pack-in | 72 h | Data Protection Board |
| pack-br | "in a reasonable time" per LGPD Art. 48 | ANPD |
| pack-ae | per UAE PDPL | UAE DOA |
| pack-ksa | per KSA PDPL | SDAIA |
| pack-cn-stub | not in production M01 | — |

## Postmortem Template

- Incident ID + Severity.
- Detection chain (alert → page → IC).
- Timeline (each step + actor + duration).
- Root cause (5-whys).
- Mitigations applied (immediate + short-term + long-term).
- LEAN-lane gaps identified (which lane would have prevented? add / refine).
- ADR amendments needed.
- Tenant + regulator notifications recorded.
- Owner sign-off (axis-translate + ops-security + council-privacy + IC).

## On-Call Rotation

- 24×7 across follow-the-sun (KR + EU + US).
- Primary + secondary on-call per pack-region.
- Page via PagerDuty integration (Grafana Mimir → Alertmanager → PagerDuty).
- Drill cadence: quarterly tabletop + biannual chaos.

## Verification

- Quarterly tabletop exercises every Sev-1 protocol.
- Annual penetration test by external firm.
- Annual SOC 2 audit verifies incident-response procedures.

## References

- `microservices/translate/failure-modes.md`.
- `microservices/translate/threat-model.md`.
- `microservices/translate/compliance.md`.
- `microservices/translate/policy/data-residency.md`.
- `microservices/translate/runbooks/` (all 7).
- ADR-0130 — SLO-gated promotion + rollback.
- ADR-TRANSLATE-0004 — residency-bound inference.
- NIST SP 800-61 Rev. 2 (Computer Security Incident Handling Guide).
- ISO/IEC 27035:2023 (Information security incident management).
