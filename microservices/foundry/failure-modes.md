---
doc_class: FAILURE-MODES
microservice: foundry
status: Accepted
date: 2026-05-18
owner_team: ops-sre-reliability + axis-foundry
related_adrs: [ADR-0136, ADR-0137]
---

# Failure Modes — foundry (consolidated)

## Scope

Cross-BC failure-mode analysis. Per-BC FMEAs preserved at
`bc-sources/<bc>/failure-modes.md`.

## Cross-BC failure modes (FMEA-style)

| ID | Failure | Detection | Mitigation | Recovery | Sev cap |
|---|---|---|---|---|---|
| F-X1 | Capability descriptor desync (supervisor → runtime cache) | runtime emits cache-miss telemetry; comparison job every 5min | retry hot-reload subscriber; circuit-break to direct-pull fallback | runbook `runtime-capability-registry-resync.md` | SEV-2 |
| F-X2 | Kill-switch fails to propagate (supervisor → runtime fleets) | absence of expected fleet-halted ack within 10s | global kill-switch escalates to per-pod manual halt | runbook `supervisor-kill-switch-engage.md` step 4 | SEV-1 |
| F-X3 | Provider credential rotation drops in-flight invocations | provider returns auth-error spike; runtime correlates to rotation event | drain old-generation pods before rotation finalises (graceful credential rotation) | runbook `providers-credential-rotation.md` | SEV-2 |
| F-X4 | Guardrail ruleset version skew across runtime pods | inline-check version-mismatch telemetry | rolling-restart runtime pods; refuse out-of-version dispatch | runbook `guardrails-policy-rule-rollback.md` | SEV-2 |
| F-X5 | Evidence pack-builder backlog | `pack-assembly-rate` dashboard falls behind invocation rate | rate-limit new invocation while drain catches up; scheduled-for-distinct-tracked-work-queue for non-urgent packs | runbook `evidence-pack-assembly-fail.md` | SEV-2 |
| F-X6 | Eval pool starves production runtime pool (shared GPU) | mis-tagged pod count rising in production runtime | eval pool deploys to dedicated node pool; affinity rule refuses mixing | runbook `eval-gpu-pool-rebalance.md` | SEV-3 |
| F-X7 | Autonomy-tier-ceiling cache stale (tenancy → runtime) | autonomy-violation false-negative on tier downgrade | `TenantTierCeilingChanged` debounce ≤5s; cache TTL ≤60s; cold-recheck on cache-miss | runbook `runtime-autonomy-violation-quarantine.md` | SEV-1 |
| F-X8 | Audit-chain bridge backlog | seal latency > 1s SLO breach | bridge HPA on queue depth; secondary bridge endpoint per pack | runbook `evidence-audit-chain-backlog.md` | SEV-2 |
| F-X9 | Cross-tenant request bypass via Cedar policy regression | red-team test failure; tenant report | refuse via authz; recall offending policy; emit audit-chain incident | post-mortem template | SEV-1 |
| F-X10 | Session-state hot-tier Redis cluster split-brain | quorum loss detected by sentinel | promote secondary; refuse writes during election; AOF flush before resume | runbook `runtime-redis-failover.md` | SEV-1 |
| F-X11 | Provider router circuit-breaker false-positive trips primary provider | error spike on primary that recovers within 30s | hysteresis on breaker open; secondary-provider fallback chain | runbook `providers-provider-outage-failover.md` | SEV-2 |
| F-X12 | Eval-run divergence on replay (golden mismatch) | parity-analyzer flags drift | quarantine capability version; halt promotion; investigate provider drift | runbook `eval-replay-divergence-investigation.md` | SEV-2 |

## Per-BC FMEA archives

- `bc-sources/runtime/failure-modes.md`
- `bc-sources/supervisor/failure-modes.md`
- `bc-sources/eval/failure-modes.md`
- `bc-sources/evidence/failure-modes.md`
- `bc-sources/guardrails/failure-modes.md`
- `bc-sources/providers/failure-modes.md`

## References

- ADR-0136 / ADR-0137: foundry topology.
- `microservices/foundry/incident-response.md` — sev definitions + runbook
  index.
