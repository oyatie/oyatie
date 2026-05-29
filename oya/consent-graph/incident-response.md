# consent-graph incident response

- Owner: axis-consent-graph + sre-axis
- Date: 2026-05-18
- Authority: ADR-0214, observability HG-CONSENT.

## 1. Severity matrix

| Severity | Condition | Response time | Channel |
|----------|-----------|---------------|---------|
| P0 | sovereignty violation; audit-chain divergence; consent forgery suspected; >10% deny-rate spike | <5min | pagerduty:axis-consent-graph-p0 + incident-commander on-call |
| P1 | revocation SLO 1h fast burn; partner suspended; cache hit-rate <50%; cross-pointer table write fails | <15min | pagerduty:axis-consent-graph-p1 |
| P2 | individual agreement Cedar compile failure spike; partner-handshake latency p95 > 60s | <1h | slack:#axis-consent-graph |
| P3 | Late-arrival reconciliation; cost-budget burn 80% | <1d | email:axis-consent-graph@oya |

## 2. P0 / P1 first-response checklist

1. Acknowledge page within 5min.
2. Open incident channel `#inc-consent-graph-<id>`.
3. Run `oya inc start consent-graph <slo-name>` to mint incident record (audit-chained).
4. Identify blast radius: which regions, which tenants, which agreement IDs.
5. Triage to specific runbook (see §3).
6. Coordinate cross-team if cross-µservice (audit-chain, ontology, Pulsar SRE).
7. Auto-suspend affected agreements *if* P0 sovereignty/forgery indicators present (handled by
   reconciler).
8. Mitigation deploy: ship via emergency rollout window (3-pod canary + 10-pod expand) — NEVER skip
   the audit-chain seal CI gate.
9. Post-incident: 5-whys + ADR-SVC-CG-* if structural change required + retrospective public to
   axis-consent-graph slack.

## 3. Runbook decision tree

```
Alert fires
├── sovereignty-violation-zero burn → runbooks/regional-sovereignty-violation.md
├── audit-chain-coverage-completeness < 1.0 → runbooks/audit-chain-divergence-recovery.md
├── revocation-propagation-latency p99 > 1s → runbooks/revocation-incident.md
├── partner-handshake or peer audit-chain root proof failed → runbooks/partner-onboarding.md (re-handshake)
├── partner-directory peer suspended → runbooks/partner-offboarding.md
├── DSAR cascade in-flight stuck → runbooks/GDPR-DSAR-cross-tenant.md
├── consent-forgery-detected anomaly → runbooks/consent-forgery-detected.md
└── data-residency rule change detected → runbooks/data-residency-enforcement.md
```

## 4. Communication

### 4.1 Internal
- Status updates every 15min in `#inc-consent-graph-<id>` for P0/P1.
- Hand-off explicit at on-call shift change.

### 4.2 External (customer + partner)
- P0 sovereignty violation → partner notification within 1h (legal + privacy officer review).
- P0 audit divergence → no automatic external notification (investigation first); audit-officer
  decides on disclosure within 24h.
- P1 partner handshake failure → notify affected partner via partner-directory channel.

### 4.3 Regulatory
- GDPR Art. 33 breach notification (data-leak suspected) → 72h to supervisory authority; DPO leads.
- HIPAA breach (US-healthcare pack agreement) → 60d to HHS Office for Civil Rights.
- KR PIPA breach → 24h to KCC (Korea Communications Commission).
- All P0 sovereignty + consent-forgery incidents trigger DPO consult immediately.

## 5. Forensic procedures

### 5.1 Audit-chain forensic snapshot
On P0 audit-divergence:
1. Freeze affected chain segments (audit-chain `freeze` API).
2. Snapshot Postgres tables: `consent_graph_agreements`, `consent_graph_cross_pointers`,
   `consent_graph_revocations`.
3. Export Pulsar topic backlogs (revocation + audit-bridge topics, 7d retention).
4. Generate forensic-report JSON sealed in audit-chain.
5. Forensic-report retained 10 years.

### 5.2 Consent forgery investigation
- Suspect events: bilateral chain entries where `paired_hmac` recomputes mismatched, OR where one
  side's chain has the event and the other doesn't.
- Procedure: per `runbooks/consent-forgery-detected.md`.
- Recovery: agreement auto-suspended; if forgery confirmed, agreement-id added to "tainted" list +
  permanent revocation.

## 6. Rollback policy

### 6.1 Code rollback
- Helm `--atomic --timeout 10m` deploys; failures auto-rollback.
- Manual rollback via `oya deploy rollback consent-graph <region> <version>`.
- Cedar schema rollback: REVERSE-incompatible — schema versions can only forward-migrate without an
  ADR-SVC-CG-* + 6mo sunset.

### 6.2 Data rollback
- Postgres point-in-time recovery (PITR): RPO 5min, RTO 30min.
- Audit-chain entries are append-only — no rollback; tampering requires explicit forensic seal
  override.

## 7. Recovery time objectives

| Incident class | RTO | RPO |
|----------------|-----|-----|
| Single-pod failure | 30s | 0 (K8s reschedule) |
| Region AZ outage | 60s | 0 (sync replica) |
| Full region outage | 30min | ≤30s (async DR) |
| Postgres data corruption | 30min | ≤5min (PITR) |
| Audit-chain divergence | 2h (review + replay) | n/a (chain is append-only) |
| Sovereignty bypass | 5min (auto-suspend) | n/a |

## 8. Game-day schedule

- Monthly: per-component kill drill in staging (rotate components).
- Quarterly: cross-µservice drill (audit-chain + consent-graph + ontology coordinated).
- Annually: production canary region full drill (single region, off-business-hours, customer-notified).

## 9. Post-mortem template

```markdown
# Incident <id> — <title>

- Date: <yyyy-mm-dd>
- Duration: <Hh:Mm>
- Severity: <P0/P1/P2/P3>
- Affected: <regions, tenants, agreement count>

## Summary
<1-paragraph>

## Timeline (UTC)
<HH:MM — event>

## Root cause
<5-whys>

## Mitigation
<what was done>

## Lessons + actions
- [ ] action 1 (owner, eta)
- [ ] action 2

## ADR follow-up (if structural)
ADR-SVC-CG-* link
```

Post-mortems are themselves sealed in audit-chain.

## 10. Cross-references

- `runbooks/*.md` — concrete procedures.
- `multi-region.md` for failover topology.
- `failure-modes.md` for FMEA mapping.
