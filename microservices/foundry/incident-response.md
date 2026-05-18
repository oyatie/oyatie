---
doc_class: INCIDENT-RESPONSE
microservice: foundry
status: Accepted
date: 2026-05-18
owner_team: ops-sre-reliability + axis-foundry
related_adrs: [ADR-0136, ADR-0137]
---

# Incident Response — foundry (consolidated)

## Scope

Cross-BC incident classification + escalation for foundry. Per-BC incident-
response docs preserved at `bc-sources/<bc>/incident-response.md`.

## Severity classification

| Sev | Definition | Foundry examples | Page | Stakeholders |
|---|---|---|---|---|
| SEV-1 | Multiple tenants impacted; data loss or safety boundary breach | Kill-switch fails to engage; cross-tenant data leak; provider credential exfiltration | <5min | ops-sre-oncall + axis-foundry on-call + council-security + ops-leadership |
| SEV-2 | Single tenant impact ≥10% error rate; substrate degradation | Runtime pool sustained crash-loop; guardrail false-positive surge >5%; eval pool exhaustion blocking promotion | <15min | ops-sre-oncall + axis-foundry on-call |
| SEV-3 | Single-BC degradation under SLO budget | Single capability-runner crash; eval-run failures isolated to one capability | <1h | axis-foundry on-call |
| SEV-4 | Cosmetic / tracked | Dashboard widget broken; non-blocking eval flake | next business day | axis-foundry |

## Runbook index

The 36 cross-BC runbooks live at `microservices/foundry/runbooks/<bc>-<runbook>.md`:

### runtime
- `runtime-autonomy-violation-quarantine.md`
- `runtime-capability-registry-resync.md`
- `runtime-emergency-runtime-drain.md`
- `runtime-redis-failover.md`
- `runtime-runtime-pod-crash.md`
- `runtime-session-state-recovery.md`

### supervisor
- `supervisor-autonomy-violation.md`
- `supervisor-deployment-rollback.md`
- `supervisor-fleet-state-recovery.md`
- `supervisor-kill-switch-engage.md`
- `supervisor-kubernetes-operator-restart.md`
- `supervisor-supervision-bus-replay.md`

### eval
- `eval-clickhouse-rebalance.md`
- `eval-eval-set-rollback.md`
- `eval-golden-output-restore.md`
- `eval-gpu-pool-rebalance.md`
- `eval-parity-regression-triage.md`
- `eval-replay-divergence-investigation.md`

### evidence
- `evidence-audit-chain-backlog.md`
- `evidence-blob-storage-restore.md`
- `evidence-evidence-archive-migration.md`
- `evidence-evidence-pack-rebuild.md`
- `evidence-pack-assembly-fail.md`
- `evidence-regulator-export-reissue.md`

### guardrails
- `guardrails-cedar-engine-restart.md`
- `guardrails-classifier-model-rollback.md`
- `guardrails-false-positive-tenant-relief.md`
- `guardrails-jailbreak-escalation.md`
- `guardrails-policy-rule-rollback.md`
- `guardrails-rule-store-restore.md`

### providers
- `providers-adapter-version-pin.md`
- `providers-credential-rotation.md`
- `providers-in-house-model-rollback.md`
- `providers-provider-credentials-revoke.md`
- `providers-provider-outage-failover.md`
- `providers-rate-limit-cascade-recovery.md`

## Cross-BC incident scenarios

| Scenario | Affected BCs | Lead BC owner | Runbook |
|---|---|---|---|
| Kill-switch engagement | supervisor + runtime + evidence | supervisor | `supervisor-kill-switch-engage.md` |
| Provider outage cascade | providers + runtime + evidence | providers | `providers-provider-outage-failover.md` |
| Cross-tenant data leak suspected | runtime + evidence + guardrails | council-security | escalate SEV-1; halt; forensic capture |
| Autonomy violation | runtime + guardrails + supervisor + evidence | supervisor | `supervisor-autonomy-violation.md` + `runtime-autonomy-violation-quarantine.md` |
| Audit-chain backlog (evidence sealing falls behind) | evidence | evidence | `evidence-audit-chain-backlog.md` |
| Eval pool exhaustion blocking promotion | eval + supervisor | eval | `eval-gpu-pool-rebalance.md` |

## Communications

- **Internal**: oya-sre slack channel + on-call ladder per
  `docs/AGENTS.md`.
- **Tenant-facing**: status-page entry within 15 min of SEV-1/SEV-2 detect;
  RCA published within 5 business days.
- **Regulator-facing**: per-pack overlays declare regulator notification
  thresholds (e.g., GDPR Art.33 — 72h breach notification).

## Post-incident

- Blameless post-mortem within 5 business days; published to
  `docs/post-mortems/<date>-<incident>.md`.
- Action items tracked in registry/fixuptasks.jsonl.
- Repeat-mistake-prevention controls per `feedback_repeat_mistake_prevention.md`.

## Per-BC archives

- `bc-sources/<bc>/incident-response.md` — per-BC sev classification +
  per-BC stakeholders.

## References

- ADR-0136 / ADR-0137: foundry topology.
- `feedback_repeat_mistake_prevention.md` — second-occurrence controls.
- `docs/AGENTS.md` — on-call ladder authority.
