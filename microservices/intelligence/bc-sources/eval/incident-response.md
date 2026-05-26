---
doc_class: IncidentResponse
title: Incident Response Plan
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry + ops-security
deciders: ops-sre-reliability, axis-foundry, ops-security, council-privacy, council-architecture
related_adrs: [ADR-0024, ADR-0028, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/intelligence-eval/threat-model.md
  - microservices/intelligence-eval/dpia.md
  - microservices/intelligence-eval/failure-modes.md
  - microservices/intelligence-eval/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Incident Response Plan (foundry-eval µservice)

## Purpose

Define how on-call detects, declares, classifies, mitigates, communicates, and recovers from incidents. Aligns with Google SRE Workbook ch. 14 (Incident Response) + NIST SP 800-61r3 (Computer Security Incident Handling Guide) + per-pack breach-notification timelines (GDPR Art. 33 / KR PIPA Art. 34 / HIPAA / NIS2).

## Severity Definitions

| Severity | Definition | Initial response time | Escalation |
|---|---|---|---|
| Sev-1 | Production breach: cross-tenant data leak, mass live-PHI exposure, mass baseline corruption, mass cutover incorrectness, EU AI Act §17 logging complete loss, sustained cross-pack misroute | ≤ 5 min | ExecSponsor + council-architecture + ops-security + council-privacy (if PII/PHI) |
| Sev-2 | Service degradation: publish-gate stalled > 30 min, replay-engine outage, ClickHouse unavailable, baseline integrity breach (single-object scope), parity-regression (critical capability), DSR SLA breach | ≤ 15 min | ops-sre-reliability + axis-foundry + ops-security (security-adjacent) |
| Sev-3 | Operational degradation: GPU pool elevated queue, ClickHouse latency spike, eval-set authoring regression, judge κ < 0.7 | ≤ 1 h | ops-sre-reliability + axis-foundry |
| Sev-4 | Informational: alert tuning, capacity forecast review, FinOps review | next business day | per-team triage |

## Roles

| Role | Responsibility |
|---|---|
| IC (Incident Commander) | drives investigation + mitigation; coordinates roles; declares severity |
| Comms Lead | tenant status-page updates; internal slack; regulatory notification trigger |
| Tech Lead | hands-on investigation; engages SMEs; executes runbook steps |
| Scribe | timeline + decisions recorded; post-incident artefacts |
| Exec Sponsor | Sev-1 only; resource arbitration; external comms approval |
| Privacy Lead | Sev-1 with PII/PHI; GDPR Art. 33 / KR PIPA / HIPAA breach notification |
| Security Lead | Sev-1 with security indicators; forensic trace; tampering investigation |

## Detection Channels

- Mimir + Alertmanager + Grafana OnCall (primary).
- ClickHouse query-leak detector → Sev-1 fast-path.
- LEAN-check failures (BLOCKER) → Sev-3 default; promoted on scope.
- Tenant report (via Application Shell support → triaged to foundry-eval if matched).
- External: auditor finding; provider security advisory; CVE disclosure.

## Severity-1 Response

### Within 5 min

1. **Page IC + Comms Lead + Privacy Lead (if PII/PHI) + Security Lead (if security indicators)**.
2. **Open #inc-<id> Slack channel**; cross-link Grafana incident.
3. **Declare Sev-1 + initial scope** in Slack.

### Within 15 min

1. **Containment**: freeze affected endpoint(s); cordon affected nodes; rotate compromised credentials. Per `runbooks/clickhouse-rebalance.md` Sev-1 path for cross-tenant leak; per `runbooks/baseline-output-restore.md` for mass-loss.
2. **Audit-chain emission**: incident-detection event emitted to foundry-evidence.
3. **Status page**: update tenant-facing status with non-confidential summary.

### Within 1 hour

1. **Categorise** per `failure-modes.md`.
2. **Engage SMEs** per category.
3. **Decide containment expansion** (e.g., per-tenant freeze, per-capability freeze, full-pack freeze).

### Within 24 hours

1. **Stabilisation**: affected components return to known-good state.
2. **Tenant communication**: detailed status; per-tenant DPA disclosure if PII/PHI.
3. **Regulatory notification trigger** (per below).

### Within 72 hours

- **GDPR Art. 33** breach notification to lead supervisory authority (for PII-scope incidents).
- **KR PIPA Art. 34** notification (PIPC + affected subjects).
- **HIPAA breach notification** (within 60d per §164.404; expedite if Sev-1).
- **NIS2 Art. 23**: initial notification ≤ 24h; detailed ≤ 72h; final ≤ 1 month (when in NIS2 scope).
- **EU AI Act**: serious-incident notification to EU AI Office per Art. 73 (within 15d default; reduced if substantial breach).

### Within 10 business days

- **Postmortem** published to `evidence/postmortems/<year>/<incident-id>.md`.
- **Action items** tracked + owned.
- **This document updated** if response gap identified.

## Severity-2 Response

### Within 15 min

1. **Page IC**.
2. **Open #inc-<id>**; assign IC.
3. **Categorise** per failure-modes.md.
4. **Execute runbook** for category.

### Within 30 min

1. **Mitigation underway**: workaround in place.
2. **Status page**: brief notice if tenant-facing impact.

### Within 5 business days

- **Postmortem** if root cause non-obvious or if recurrence likely.
- Action items tracked.

## Severity-3 / Severity-4 Response

- Standard team triage; no formal incident process.
- Track via Issues; postmortem only on recurrence or scope-creep.

## Communication Templates

### Internal: Slack incident channel template

```
**Incident #<id>**
Severity: Sev-X
IC: @<user>
Comms Lead: @<user>
Tech Lead: @<user>
Privacy Lead: @<user> (if PII/PHI)
Security Lead: @<user> (if security)
Scope: <one-line>
Detection signal: <metric / alert / report>
Containment: <action / status>
Last update: <ts>
Next update: <ts>
```

### External: Tenant status page (per-pack)

- Reaches tenant operators within 15 min of Sev-1 detection.
- Plain English; no internal jargon; no per-tenant identifiers.
- Updates every 30 min until stable; daily until resolved.

### Regulatory notification template

Per-pack legal templates at `legal/regulatory-notification-<framework>.md`:
- `legal/regulatory-notification-gdpr.md`
- `legal/regulatory-notification-kr-pipa.md`
- `legal/regulatory-notification-hipaa.md`
- `legal/regulatory-notification-eu-ai-act.md`
- `legal/regulatory-notification-nis2.md`
- (per-pack equivalents for APPI, PDPA, APRA-CPS, DPDPA, LGPD, etc.)

## Postmortem Template

Per `templates/postmortem.md`:
- Summary (1 paragraph)
- Timeline (UTC; minute granularity for Sev-1)
- Detection (what fired? what didn't?)
- Containment (what worked? what didn't?)
- Root cause (5-whys)
- Resolution (what brought back to known-good?)
- Action items (owned + tracked)
- What went well (always; reinforce)
- What we learned (always; broader applicability)
- Was this preventable in design? (always; design-level prevention)

## On-Call Rotation

Per `runbooks/oncall-rotation.md`:
- 24/7 primary + secondary per pack.
- Rotation cadence: weekly handoff.
- Cross-pack escalation chain to axis-foundry global on-call (single primary global; rotates monthly).
- Compensation per company policy.

## Drills

| Drill | Cadence | Owner |
|---|---|---|
| Sev-1 ClickHouse cross-tenant leak drill | Quarterly | ops-security |
| Sev-1 mass baseline corruption drill | Quarterly | axis-foundry |
| Sev-1 replay-determinism mass divergence drill | Semi-annually | axis-foundry |
| Sev-1 KMS outage drill | Semi-annually | ops-security |
| Postmortem review meeting | Monthly | axis-foundry |

Drill reports under `evidence/drills/<year>/<drill-id>.md`.

## Per-Pack Overlays

### pack-eu (EU AI Act high-risk)

- Sev-1 with significant AI-system impact: EU AI Office notification per Art. 73.
- GDPR Art. 33 + EU AI Act Art. 73 align on 72h initial notification.

### pack-us-healthcare

- HIPAA breach notification: §164.404 (individuals) + §164.408 (Secretary, > 500 affected).
- 60-day deadline for tenant notification (can be accelerated per Sev-1).

### pack-kr

- KR PIPA Art. 34 (PIPC + subjects): ≤ 72h for serious breach + immediate for material breach.
- KR PIPC Notice 2021-9 (notification specifics).

## References

- ADR-0024 (eval harness).
- ADR-0028 (audit-chain).
- `microservices/intelligence-eval/threat-model.md`.
- `microservices/intelligence-eval/dpia.md`.
- `microservices/intelligence-eval/failure-modes.md`.
- `microservices/intelligence-eval/runbooks/*`.
- Google SRE Workbook ch. 14.
- NIST SP 800-61r3 (Computer Security Incident Handling Guide).
- GDPR Art. 33; EU AI Act Art. 73; HIPAA §164.404; KR PIPA Art. 34; NIS2 Art. 23.
