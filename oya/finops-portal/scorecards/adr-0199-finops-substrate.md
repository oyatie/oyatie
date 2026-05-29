---
scorecard_id: finops-portal/adr-0199-finops-substrate
authored: 2026-05-18
authority: ADR-0199 cost-attribution canonical + FinOps substrate
status: ready
---

# Scorecard — ADR-0199 FinOps substrate

ADR-0199 is the canonical authority for `finops-portal`. It names this
µservice as the in-house Phase 2 UX layer over OpenCost + Mimir +
FOCUS 1.3.

## Compliance evidence

| Criterion                                                       | Status | Evidence                                              |
|-----------------------------------------------------------------|--------|-------------------------------------------------------|
| Manifest cites ADR-0199 as canonical authority                  | ✓      | `manifest.json#doctrine.canonical_authority`         |
| OpenCost is the cost-aggregation engine (NOT this µservice)     | ✓      | README §"What this µservice is NOT" + PRD §Out-of-scope|
| Prometheus rules own anomaly detection (NOT this µservice)      | ✓      | `runbooks/tenant-cost-anomaly-spike.md` clarifies     |
| FOCUS 1.3 native export                                         | ✓      | IP-014 + `capabilities/focus-export.capability.yaml`  |
| Per-tenant cost attribution labels propagated                   | ✓      | Helm `costAttribution.*` + ServiceMonitor relabel     |
| Quarterly regulator-evidence emit (per ADR-0174 + ADR-0162)     | ✓      | IP-015                                                |
| Audit-chain seal classes declared                               | ✓      | `manifest.json#audit_chain.seal_events` (5 classes)   |
| Credit-ledger append-only invariant                             | ✓      | IP-013 kernel invariants                              |
| Cost-allocation policy editable + auditable                     | ✓      | IP-009 + IP-010                                       |
| Anomaly explanation deterministic + human-in-loop               | ✓      | IP-011 + EU AI Act risk class "limited"               |
| LTS pins declared                                               | ✓      | `manifest.json#lts_pins.opencost: 1.110.0`            |

## Cited dependencies

- `observability` µservice via OpenCost prom federation.
- `cloud-iac` µservice via SeaweedFS s3 (FOCUS export storage).
- `tenancy` µservice via tenant-id resolver.
- `audit-chain` µservice via regulator-evidence emit.

## Gaps + remediation

- **Gap**: OpenCost custom-pricing configmap not yet wired into the
  IP-005 API layer's policy-propagation latency SLI. **Remediation**:
  tracked in IP-005.

## Verdict

**PASS**.

## References

- ADR-0199 cost-attribution canonical + FinOps substrate.
- ADR-0174 chargeback formula.
- ADR-0162 per-tenant audit-log slicing.
- IPs 001..015.
