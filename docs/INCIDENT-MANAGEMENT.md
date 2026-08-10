---
purpose: Oyatie — Incident Management
doc_status: published
---

# Oyatie — Incident Management

> **Status:** Draft v0.1 — 2026-05-09.
> **Owner:** `ops-sre-reliability`.
> **Companion:** [security-program/security-program.json](security-program/security-program.json), [RUNBOOKS-INDEX.md](RUNBOOKS-INDEX.md), [RISK-REGISTER.md](RISK-REGISTER.md), [`templates/incident-postmortem-template.md`](templates/incident-postmortem-template.md), [`../templates/checklists/incident-response.md`](../templates/checklists/incident-response.md).

## 1. Severity taxonomy

| Sev | Definition | Response time | Comms cadence |
|---|---|---|---|
| **Sev 1** | Tenant-data breach OR cross-tenant access OR audit-chain integrity failure OR multi-region outage OR regulatory-impact event | Page within 5min; resolve target ≤ 4h | Customer notification ≤ 24h; regulator ≤ 72h (PIPA Art 34 / GDPR Art 33) |
| **Sev 2** | Single-region outage OR data-plane SLO budget exhausted in < 6h burn | Page within 15min; resolve ≤ 24h | Customer per-tenant notification ≤ 48h |
| **Sev 3** | Surface degraded but functional OR control-plane SLO budget burning at 3-6× | Ticket within 1h; resolve ≤ 7d | Trust portal status update |
| **Sev 4** | Cosmetic / observability gap | Ticket within 24h; resolve ≤ 30d | Internal only |

## 2. Roles

| Role | Responsibility |
|---|---|
| **Incident Manager (IM)** | Coordinates response; declares severity; chairs the bridge |
| **Comms Manager (CM)** | Drafts customer + regulator + status-page comms |
| **Subject Matter Expert (SME)** | Per affected axis / surface; executes runbook |
| **Privacy Lead** | If Sev 1 with data-class touch |
| **Security Lead** | If security-class incident |
| **Founder** | Notified for Sev 1; approves regulator notification language |

Rotation per `docs/standards/on-call.md` (TBD).

## 3. Lifecycle

```
Detection → Triage (Sev assigned) → Response (IM + SME + bridge) →
Mitigation → Resolution → Postmortem → Prevention loop
```

### 3.1 Detection sources
- SLO burn-rate alerts (per [SLO-CATALOG.md §3](SLO-CATALOG.md))
- Audit-chain integrity check failure
- Customer-reported via `oya admin incident report`
- Synthetic monitoring failure (#138)
- Security alert (Trivy / Cosign / RUSTSEC / red-team)
- Foundry capability anomaly (per `oya-intelligence-anomaly-*`)

### 3.2 Triage
IM declares Sev within 5 minutes of detection. Re-classifies during response if new info emerges.

### 3.3 Response
- Bridge opens: voice + chat
- Runbook invoked from [RUNBOOKS-INDEX.md](RUNBOOKS-INDEX.md)
- Per-affected-tenant impact estimated
- Per-cell containment if cross-tenant risk
- Regulatory clock starts (PIPA 24h FSS / 72h PIPC; GDPR 72h Supervisory Authority; HIPAA 60d HHS; PCI per acquirer)

### 3.4 Mitigation vs Resolution
- **Mitigation:** customer impact stopped or contained
- **Resolution:** root cause fixed
- Both required before incident closed

### 3.5 Postmortem (per `templates/incident-postmortem-template.md`)
- Blameless
- Timeline reconstructed from audit chain + on-call notes
- Root cause(s) identified (5-Whys / Causal-Tree)
- Action items: prevention (mechanical) + mitigation (process)
- Each action item gets an owner + ETA + tracking issue
- Published within 30d on internal wiki; trust-portal mirror within 60d for customer-facing

### 3.6 Prevention loop (per `docs/standards/prevention-doctrine.md`)
- Every Sev 1/2 produces ≥ 1 prevention item
- Prevention is mechanical (CI gate / hook / validator / test / config-as-code), not process
- Prevention shipped within 30d (Sev 1) or 60d (Sev 2)
- `docs/MISTAKES-LEDGER.md` row added

## 4. Communications templates

Per Sev:

### Sev 1 customer notification
```
Subject: Oyatie Sev 1 Incident — <surface> — <YYYY-MM-DD HH:MM UTC>

What happened: <one paragraph>
What's affected: <surfaces / data classes / regions>
Status: investigating | mitigated | resolved
Next update: <time>
Trust portal: trust.oyatie.com/incidents/<id>
```

### Sev 1 regulator notification (PIPA Art 34 example)
Per [COMPLIANCE-MATRIX.md §3.1](COMPLIANCE-MATRIX.md) Art 34.

### Status-page entry
Per `status.oyatie.com` per affected region/surface.

## 5. Trust-portal publishing

- Sev 1 incident: live status updates during; postmortem within 30d
- Sev 2: status updates within 1h; postmortem within 30d
- Sev 3/4: aggregated weekly status

## 6. Drills

- Sev 1 game-day quarterly per axis
- Sev 1 game-day annually cross-axis (regulator-notification simulation)
- Region failover drill quarterly per region
- DR + tenant-restore drill quarterly per axis

## 7. Sources
ADR-0003 audit chain, ADR-0040 launch readiness 9-item, Google SRE workbook chapters on IM, KR-PIPA Art 34, GDPR Art 33, HIPAA Breach Notification Rule, PCI-DSS v4.0 Req 12, `docs/standards/prevention-doctrine.md`, `docs/MISTAKES-LEDGER.md`.
