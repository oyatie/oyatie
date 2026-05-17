---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry
deciders: ops-sre-reliability, axis-foundry, ops-security, council-architecture
related_adrs: [ADR-0024, ADR-0026, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/foundry-eval/threat-model.md
  - microservices/foundry-eval/dpia.md
  - microservices/foundry-eval/incident-response.md
  - microservices/foundry-eval/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident affecting foundry-eval
doc_status: published
---

# Failure-Mode Catalog (foundry-eval µservice)

## Purpose

Enumerate the failure scenarios on-call must handle for foundry-eval: detection signal, immediate mitigation, RCA path, RTO, and owning runbook. Cross-referenced from `incident-response.md` for severity classification.

## Index format

Each failure carries: FM-ID; Trigger; Detection (SLI/alert/metric); Tenant impact; Severity; Immediate mitigation; RTO; Runbook; Postmortem owner.

## FM-01: Eval-set authoring regression (newly promoted version regresses pass-rate)

| Field | Value |
|---|---|
| Trigger | Capability owner promotes a new eval-set version that includes over-fit / contaminated / miscalibrated cases |
| Detection | `oya_foundry_eval_pass_rate{capability="<cap>"}` drops ≥ 10pp within first nightly cadence |
| Tenant impact | Future capability re-publishes hit a falsely-failing gate; capability owner unable to ship; tenant upgrades blocked |
| Severity | Sev-2 (eval signal compromised) |
| Immediate mitigation | Auto-page on > 10pp drop; trigger `runbooks/eval-set-rollback.md` |
| RTO | ≤ 30 min for rollback to prior signed version |
| Recovery runbook | `runbooks/eval-set-rollback.md` |
| Postmortem owner | axis-foundry + capability owner team |

## FM-02: GPU runner pool exhaustion (publish-gate stalls)

| Field | Value |
|---|---|
| Trigger | Adversarial flood of eval-set submissions; provider rate-limit causing slow case completion; GPU quota hit; gVisor overhead spike |
| Detection | `oya_foundry_eval_gpu_pool_queue_depth_seconds > 300` for ≥ 5min OR publish-gate p99 latency > 5s |
| Tenant impact | Capability publish stalls for everyone; tenant upgrades delayed |
| Severity | Sev-2 (operational; not eval-correctness compromised) |
| Immediate mitigation | Manual scale-out; cluster-autoscaler nudge; priority-class triage; pause nightly cadence |
| RTO | ≤ 30 min for autoscale; ≤ 1h for full triage |
| Recovery runbook | `runbooks/gpu-pool-rebalance.md` |
| Postmortem owner | ops-sre-reliability + axis-foundry |

## FM-03: Parity regression (in-house variant loses cohort vs incumbent)

| Field | Value |
|---|---|
| Trigger | Judge rotation introduces bias OR provider drift OR eval-set contamination OR tokeniser drift OR genuine quality regression in in-house variant |
| Detection | `oya_foundry_eval_parity_margin{delta_kind="in_house_vs_incumbent"} < 0` on previously-positive cohort |
| Tenant impact | Cutover-eligibility verdict regressed; future cutover decisions delayed |
| Severity | Sev-2 (cutover decisioning compromised) |
| Immediate mitigation | Pause cutover-eligibility recomputation; engage `runbooks/parity-regression-triage.md`; if reverse-cutover needed, automated emission |
| RTO | ≤ 1h for categorisation; ≤ 24h for resolution |
| Recovery runbook | `runbooks/parity-regression-triage.md` |
| Postmortem owner | axis-foundry |

## FM-04: Replay-determinism divergence > 100ms p99

| Field | Value |
|---|---|
| Trigger | Provider drift; sandbox CUDA bleed; tokeniser drift; capability prompt template changed; golden mismatch |
| Detection | `oya_foundry_eval_replay_divergence_ms{quantile="0.99"} > 100` sustained ≥ 5min |
| Tenant impact | Model-upgrade gate held; replay-based drift detection compromised |
| Severity | Sev-1 if critical capability; Sev-2 otherwise |
| Immediate mitigation | Categorise divergence source; hold model-upgrade until resolved |
| RTO | ≤ 1h for categorisation; ≤ 4h for resolution; Sev-1 → ExecSponsor engagement |
| Recovery runbook | `runbooks/replay-divergence-investigation.md` |
| Postmortem owner | axis-foundry + ops-security if sandbox escape suspected |

## FM-05: Golden-output integrity breach (Cosign verify fails or block-validator mismatch)

| Field | Value |
|---|---|
| Trigger | Object tampering (T-T-02), bit-rot, transient S3 read-error, mass golden loss |
| Detection | `oya_foundry_eval_golden_cosign_verify_failed_total > 0` OR monthly block-validator quarantine |
| Tenant impact | Affected capability eval halts; potential signal of tampering |
| Severity | Sev-2 (signature breach) or Sev-1 (mass loss) |
| Immediate mitigation | Quarantine; 2-person rule investigation; restore from DR pair |
| RTO | ≤ 1h per-object restore from DR; ≤ 4h mass restore |
| Recovery runbook | `runbooks/golden-output-restore.md` |
| Postmortem owner | axis-foundry + ops-security |

## FM-06: ClickHouse cluster imbalance / query-latency spike / cross-tenant leak

| Field | Value |
|---|---|
| Trigger | Data skew, disk pressure, ZooKeeper coordination latency, T-I-01 misconfiguration |
| Detection | `clickhouse_query_latency_seconds{quantile="0.99"} > 0.5` OR shard size variance > 20% OR `oya_foundry_eval_cross_tenant_query_leak_total > 0` (Sev-1) |
| Tenant impact | Parity analytics slow / unavailable; cross-tenant leak = breach |
| Severity | Sev-3 latency / Sev-2 unavailable / Sev-1 leak |
| Immediate mitigation | Rebalance shards; compact; for leak → freeze endpoint + ops-security |
| RTO | ≤ 1h rebalance; ≤ 5min leak freeze |
| Recovery runbook | `runbooks/clickhouse-rebalance.md` |
| Postmortem owner | ops-sre-reliability + axis-foundry; ops-security for leaks |

## FM-07: Eval-runner-worker outage (publish-gate stalls)

| Field | Value |
|---|---|
| Trigger | Worker crashloop (Postgres unreachable, OpenBao token-renewal failure, eval-set parse bug) |
| Detection | `oya_foundry_eval_eval_runner_worker_alive == 0` for ≥ 2min |
| Tenant impact | Publish-gate verdict path unavailable; capability re-publishes stall |
| Severity | Sev-2 (operationally blocking; eval-correctness intact) |
| Immediate mitigation | HA leader-election re-runs; standby takes over |
| RTO | ≤ 5min HA failover; ≤ 30min full root-cause |
| Recovery runbook | `runbooks/eval-runner-down.md` (inherits pattern from observability runbooks/evaluator-down.md) |
| Postmortem owner | axis-foundry |

## FM-08: Replay-engine-worker outage (model upgrades stalled)

| Field | Value |
|---|---|
| Trigger | Worker crashloop; per-subject DEK unwrap failure; S3 unreachable |
| Detection | `oya_foundry_eval_replay_engine_alive == 0` for ≥ 2min |
| Tenant impact | Model-upgrade gate held (fail-closed per ADR-0024) |
| Severity | Sev-2 |
| Immediate mitigation | HA leader-election; restart |
| RTO | ≤ 5min HA failover |
| Recovery runbook | `runbooks/replay-divergence-investigation.md` §"Engine outage" |
| Postmortem owner | axis-foundry |

## FM-09: Postgres outage (eval-set metadata unreachable)

| Field | Value |
|---|---|
| Trigger | Patroni primary failure; PV corruption; planned maintenance over-runs |
| Detection | Postgres health probe fails for ≥ 60s |
| Tenant impact | Eval-set authoring + registry reads stall; cached path holds 5min |
| Severity | Sev-2 |
| Immediate mitigation | Patroni promotes replica; verify HA |
| RTO | ≤ 5min HA failover; ≤ 30min full root-cause |
| Recovery runbook | `runbooks/postgres-failover.md` (standard pattern) |
| Postmortem owner | ops-sre-reliability + axis-foundry |

## FM-10: KMS outage (per-subject DEK unwrap fails)

| Field | Value |
|---|---|
| Trigger | KMS regional outage; KMS rate-limit hit |
| Detection | `oya_foundry_eval_dek_unwrap_failed_total > 0` rate-sustained |
| Tenant impact | Replay halts; golden read halts (encrypted goldens) |
| Severity | Sev-2 |
| Immediate mitigation | Failover to DR pair KMS; pause replay sampling |
| RTO | ≤ 1h provider-dependent |
| Recovery runbook | `runbooks/kms-failover.md` (standard pattern) |
| Postmortem owner | ops-security + axis-foundry |

## FM-11: Cosign / Rekor public-log unreachable

| Field | Value |
|---|---|
| Trigger | Rekor public-log outage; Sigstore upstream issue |
| Detection | Eval-set load fails Rekor inclusion-proof verification |
| Tenant impact | Eval-set registry fails to load new versions; cached versions still usable |
| Severity | Sev-2 |
| Immediate mitigation | Switch to mirrored Rekor (we maintain internal Rekor mirror); audit-emit fall-back use |
| RTO | ≤ 15min switch to mirror; provider recovery dependent |
| Recovery runbook | `runbooks/sigstore-failover.md` (inherits ADR-0024 §"Eval kernel" pattern) |
| Postmortem owner | ops-security + axis-foundry |

## FM-12: Provider model API rate-limit / outage

| Field | Value |
|---|---|
| Trigger | Provider hits our QPS limit; provider service outage |
| Detection | `oya_foundry_eval_provider_invoke_failed_total{provider="<p>",reason="rate_limit|outage"} > 0` sustained |
| Tenant impact | Cases for that provider stall; multi-provider routes degrade |
| Severity | Sev-3 |
| Immediate mitigation | Rate-limit increase request (ops-finance); fall over to alternate provider per foundry-providers router-preference |
| RTO | ≤ 1h alternate; provider-dependent for original |
| Recovery runbook | `runbooks/provider-rate-limit.md` (cross-cuts to foundry-providers) |
| Postmortem owner | ops-finance + axis-foundry-providers + axis-foundry |

## FM-13: Adversarial-cohort drift (newly-disclosed pattern not in cohort)

| Field | Value |
|---|---|
| Trigger | Anthropic / Apollo / UK AISI discloses a new prompt-injection pattern not in our cohort |
| Detection | Quarterly cohort-freshness review |
| Tenant impact | Adversarial gate may not detect new pattern; downstream capability may be vulnerable |
| Severity | Sev-3 |
| Immediate mitigation | Patch adversarial cohort with new pattern; re-run publish-gate on affected capabilities |
| RTO | ≤ 1 week (cohort authoring + review) |
| Recovery runbook | `runbooks/adversarial-cohort-refresh.md` |
| Postmortem owner | ops-security + axis-foundry |

## FM-14: Judge consistency breach (Cohen's κ < 0.7 after quarterly rotation)

| Field | Value |
|---|---|
| Trigger | New judge model disagrees materially with prior judge on gold pool |
| Detection | `oya_foundry_eval_judge_cohen_kappa{capability="<cap>"} < 0.7` |
| Tenant impact | HumanJudged cohort verdicts unreliable for that capability |
| Severity | Sev-3 |
| Immediate mitigation | Pause cutover-eligibility for affected capabilities; revert to prior judge; investigate rubric drift |
| RTO | ≤ 1 day |
| Recovery runbook | `runbooks/judge-rotation-rollback.md` |
| Postmortem owner | axis-foundry |

## FM-15: DSR cascade SLA breach (per-subject DEK shred > 30d)

| Field | Value |
|---|---|
| Trigger | Shred queue backlog; KMS rate-limit; signing-key issues |
| Detection | `oya_foundry_eval_dsr_cascade_sla_pct < 99` sustained |
| Tenant impact | GDPR Art. 17 / KR PIPA Art. 36 SLA missed; regulatory breach risk |
| Severity | Sev-2 (regulatory breach risk) |
| Immediate mitigation | Boost shred-queue throughput; engage council-privacy if SLA truly missed |
| RTO | varies; per-subject SLA still 30d, but operational catch-up may take days |
| Recovery runbook | `runbooks/dsr-cascade-recovery.md` |
| Postmortem owner | council-privacy + axis-foundry |

## RTO / RPO Summary

| Failure | RTO | RPO |
|---|---|---|
| Eval-set authoring regression | 30 min | 0 (revert by version pointer) |
| GPU pool exhaustion | 30 min | 0 (queued cases re-dispatched) |
| Parity regression | 1 h categorisation | 0 |
| Replay divergence > 100ms | 1 h categorisation | 0 |
| Golden-output integrity breach | 1 h per-object restore | 0 (per-object signature) |
| ClickHouse imbalance | 1 h | 0 (replication-factor) |
| ClickHouse cross-tenant leak | 5 min freeze | N/A (breach) |
| Eval-runner-worker outage | 5 min HA | 0 (stateless) |
| Replay-engine-worker outage | 5 min HA | 0 |
| Postgres outage | 5 min HA | 5 min (PITR) |
| KMS outage | 1 h (DR) | 0 |
| Cosign/Rekor outage | 15 min switch to mirror | 0 |
| Provider rate-limit | 1 h alternate | 0 |
| Adversarial cohort drift | 1 week | N/A (cohort refresh) |
| Judge κ < 0.7 | 1 day | N/A (rollback) |
| DSR SLA breach | varies | varies |

## SLO on Failure-Detection Pipeline

Meta-SLO: foundry-eval substrate itself has SLO that no failure remains undetected beyond detection-window target.

| SLI | Target | Burn-rate alert |
|---|---|---|
| Alert-to-page latency p99 | ≤ 60 s | 14.4× burn over 1h |
| Detection coverage (synthetic faults caught) | ≥ 99.5% | 6× burn over 6h |
| False-positive page rate | ≤ 1 / week / on-call | informational |
| Replay-determinism breach detection lag | ≤ 5 min | 14.4× burn over 1h |

## References

- `microservices/foundry-eval/threat-model.md`.
- `microservices/foundry-eval/dpia.md`.
- `microservices/foundry-eval/incident-response.md`.
- `microservices/foundry-eval/runbooks/`.
- `microservices/foundry-eval/capacity-model.md`.
- Google SRE Workbook ch. 12 (Postmortem culture).
- ADR-0024, ADR-0026.
