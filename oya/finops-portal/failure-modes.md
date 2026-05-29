---
doc_id: finops-portal/failure-modes
authored: 2026-05-18
status: ready
authority: ADR-0152 RPO/RTO + ADR-0162 audit-log integrity
classification: internal
---

# Failure modes — finops-portal

This document enumerates the failure modes of `finops-portal`,
their detection mechanism, blast radius, and the mitigation owner.
It is the input to the chaos-engineering test plan + the SRE
on-call training curriculum.

## Failure-mode catalog

### F-01: Postgres primary unavailable

- **Detection**: pod readiness probe fails → out of LB → SLO
  burn-rate alert.
- **Blast radius**: writes block (finalize, credit-apply); reads
  continue from read-replicas.
- **Mitigation**: postgres operator promotes a replica to primary
  in < 30 s. If promotion fails → SEV-2 + manual intervention.
- **Owner**: ops-platform (postgres operator); ops-finops
  (application visibility).

### F-02: Mimir query degraded

- **Detection**: `drilldown-query-latency-p99` SLO breach.
- **Blast radius**: drill-down dashboards slow; invoice render
  continues (postgres rollup serves the invoice header).
- **Mitigation**: recording-rule pre-aggregation; reduce
  cardinality.
- **Owner**: observability µservice + ops-platform.

### F-03: Audit-chain endpoint unavailable

- **Detection**: `CreditApplicationSealMiss` alert.
- **Blast radius**: invoices finalize but seal is delayed;
  quarterly emit re-emits.
- **Mitigation**: reconciler (`runbooks/credit-application-
  reconciliation.md`).
- **Owner**: audit-chain µservice (root cause); ops-finops
  (reconcile).

### F-04: OpenCost data plane stale

- **Detection**: `opencost_data_freshness_seconds > 300` alert.
- **Blast radius**: drill-down shows stale numbers; invoices for
  the current period may be incomplete.
- **Mitigation**: pause finalize for the affected period; alert
  ops-finops to investigate OpenCost.
- **Owner**: observability + finops-portal.

### F-05: Cedar policy fails to load

- **Detection**: `/ready` probe returns 503; deployment fails
  rollout.
- **Blast radius**: contained — the pod never accepts traffic.
- **Mitigation**: rollback (`runbooks/finops-portal-deploy-
  rollback.md`).
- **Owner**: ops-finops + security.

### F-06: Signed-URL HMAC key expired

- **Detection**: `FocusExportFailureRate` with class
  `SignerKeyExpired`.
- **Blast radius**: new FOCUS exports fail; existing URLs (within
  5 min) still work.
- **Mitigation**: rotate key + redeploy.
- **Owner**: ops-finops + secrets µservice.

### F-07: Quarterly Ed25519 key compromise

- **Detection**: manual report OR secret-scanning alert.
- **Blast radius**: signatures issued under the compromised key
  are repudiable.
- **Mitigation**: `incident-playbook.md` §Key compromise.
- **Owner**: security + ops-finops.

### F-08: Cross-tenant data leak (logic bug)

- **Detection**: tenant report OR audit-chain anomaly query.
- **Blast radius**: HIGH (regulatory + reputational).
- **Mitigation**: `incident-playbook.md` §Cross-tenant leak.
- **Owner**: ops-finops + security + exec.

### F-09: Cost-allocation policy mis-promotion (logic bug)

- **Detection**: `CostAllocationPolicyChangedAlert`.
- **Blast radius**: fleet-wide cost shift; credits owed.
- **Mitigation**: `runbooks/cost-allocation-policy-rollback.md`.
- **Owner**: ops-finops.

### F-10: Quarterly emit slip past cure window

- **Detection**: `QuarterlyRegulatorEmitMiss`.
- **Blast radius**: regulatory finding risk; HIGH.
- **Mitigation**: `runbooks/quarterly-regulator-emit-miss.md`.
- **Owner**: ops-finops + compliance.

### F-11: SeaweedFS bucket lifecycle deletes early

- **Detection**: ops report; restoration request.
- **Blast radius**: FOCUS exports beyond lifecycle window
  unavailable; tenants can re-trigger.
- **Mitigation**: restore from backup if needed
  (`runbooks/focus-export-failure.md` §Path B).
- **Owner**: cloud-iac + ops-finops.

### F-12: HPA scale-up cascading failure

- **Detection**: HPA at max + p99 still rising.
- **Blast radius**: tenant-facing latency degraded across the
  cell.
- **Mitigation**: add a cell (`multi-region-strategy.md`); rate-
  limit incoming requests via the API gateway.
- **Owner**: ops-finops + ops-platform.

### F-13: Tenancy µservice unavailable

- **Detection**: api crate readiness fails because tenant-id
  resolver is down.
- **Blast radius**: new requests fail; in-flight requests
  complete.
- **Mitigation**: cached principal claims survive short outages
  (5 min); page on > 5 min.
- **Owner**: tenancy µservice + ops-platform.

### F-14: Anomaly-explanation determinism regression

- **Detection**: byte-equality unit test fails OR explanation
  changes between runs.
- **Blast radius**: audit-chain seal envelope hash diverges
  between repeated explanations.
- **Mitigation**: bisect the change; revert; gate hardens.
- **Owner**: ops-finops.

## Chaos engineering matrix

The chaos test plan exercises each failure mode at least
quarterly via the chaos-substrate µservice. Drill IDs map 1:1 to
failure-mode IDs (F-01..F-14).

| ID    | Drill cadence | Result expected                                              |
|-------|---------------|--------------------------------------------------------------|
| F-01  | quarterly     | failover < 30s; SLO burn but not breach                      |
| F-02  | quarterly     | drill-down degrades; invoice render unaffected               |
| F-03  | monthly       | reconciler clears within 1 h                                 |
| F-04  | quarterly     | finalize paused for affected period; alert fires             |
| F-05  | monthly       | rollback completes; readiness blocks traffic                 |
| F-06  | quarterly     | rotate completes; export resumes                             |
| F-07  | annually      | game-day playbook executed                                   |
| F-08  | annually      | game-day playbook executed                                   |
| F-09  | quarterly     | retire + rollback; credits applied                           |
| F-10  | annually      | cure-window expiry simulated                                 |
| F-11  | quarterly     | restore drill runs                                           |
| F-12  | quarterly     | add-cell drill runs                                          |
| F-13  | quarterly     | cached principal survives 5min outage                        |
| F-14  | monthly       | determinism unit test passes                                 |

## References

- ADR-0152 RPO/RTO classes.
- ADR-0162 audit-log integrity.
- `runbooks/*.md`.
- `incident-playbook.md`.
