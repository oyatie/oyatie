---
doc_class: FailureModes
title: Failure Modes + Recovery Posture
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + ops-sre-reliability
deciders: axis-foundry, ops-sre-reliability, council-architecture, ops-security
related_adrs: [ADR-0110, ADR-0111, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/governance/threat-model.md
  - microservices/governance/runbooks/lane-failure-triage.md
  - microservices/governance/runbooks/aggregation-rebuild.md
review_cadence: quarterly + post-incident
doc_status: published
---

# Failure Modes: governance µservice

## Purpose

Enumerate every plausible failure of governance components, classify by severity, document mitigation + detection + recovery + RTO/RPO. Each row maps to a runbook for hands-on response and to an alert in `dashboards/*.json`.

## Failure Mode Catalog

### F-01 — Lane runner OOM (single PR)

| Attribute | Value |
|---|---|
| Trigger | Pathological PR (e.g., 100k-file diff; circular dependency in workspace) exhausts runner memory > 8 GB |
| Blast radius | Single lane on single PR; one BLOCKER false-positive |
| Detection | cgroup OOM-kill; runner emits `lane-runner-oom` Finding (severity = OPERATIONAL); Grafana alert `lane_runner_oom_total > 5/min` |
| Mitigation | Per-runner cgroup ceiling 8 GB; per-lane input-size cap (10k files → auto-refuse with `pr-too-large` Finding); SIGKILL on overrun |
| Recovery | Auto: re-queue same lane on fresh runner (1 retry); manual: PR author splits diff per `lane-failure-triage.md` |
| RTO | ≤5 min (auto-recovery); ≤1h (manual diff-split) |
| RPO | 0 (lane runs are stateless; re-runnable) |
| Owner | axis-foundry |
| Severity | MEDIUM |

### F-02 — Lane false-positive blocks merge

| Attribute | Value |
|---|---|
| Trigger | Rule-pack bug emits BLOCKER on valid code |
| Blast radius | Every PR matching the rule-pack pattern; all `dev` merges blocked until fix |
| Detection | PR author opens issue + PR-comment on the governance repo; `oya-pr-review` agent flags `repeated-blocker-same-rule`; Grafana alert `blocker_false_positive_rate > 5%` |
| Mitigation | Rule-pack PR review by two CODEOWNERS (axis-foundry + ops-security); self-application test catches obvious false-positives at scaffold time |
| Recovery | Two-PR fix: (a) hot-patch rule-pack to downgrade BLOCKER → WARN with ADR rationale; (b) follow-up PR investigating root cause + restoring BLOCKER if appropriate |
| RTO | ≤2h (hot-patch); ≤2 weeks (full RCA + permanent fix) |
| RPO | 0 |
| Owner | axis-foundry |
| Severity | HIGH (every false-positive blocks production-tier delivery) |

### F-03 — Evidence emission gap (Finding emitted but not sealed)

| Attribute | Value |
|---|---|
| Trigger | Audit-chain µservice unreachable from evidence-emitter (network partition; upstream outage; key-rotation race) |
| Blast radius | Findings between gap-start and gap-end land in Postgres but lack audit-chain seal → SOC 2 non-repudiation chain broken for that window |
| Detection | Background reconciliation worker (`evidence-seal-reconciler`) detects unsealed Findings older than 5 min; Grafana alert `unsealed_findings_age_seconds_p99 > 300`; OnCall page |
| Mitigation | Local outbox pattern: evidence-emitter writes (Finding, sealed=false) to Postgres atomically; reconciler re-attempts seal until success |
| Recovery | Reconciler back-fills seals once audit-chain reachable; per-Finding seal latency p99 returns < 1s |
| RTO | ≤15 min (audit-chain recovery + back-fill) |
| RPO | 0 (no Finding loss; outbox is durable) |
| Owner | ops-security |
| Severity | HIGH (audit-chain integrity is load-bearing) |

### F-04 — Aggregation-index corruption

| Attribute | Value |
|---|---|
| Trigger | Concurrent commits to per-µservice sources during aggregation-indexer regen → race produces inconsistent central index |
| Blast radius | Central indices (`docs/prds/INDEX.md`, `registry/catalog/`, `/specs/products/`) refer to stale sources |
| Detection | `oya-check-aggregation-index-generation` lane on next PR detects divergence; emits `aggregation-divergence` BLOCKER; Grafana alert `aggregation_divergence_total > 0` |
| Mitigation | Aggregation-indexer holds Postgres advisory lock during regen; coalescing 15-min window; idempotent regen logic |
| Recovery | Re-run regen with lock held; commit corrected indices via scoped PAT (see T-E-03 mitigation); follow-up PR if root cause is structural |
| RTO | ≤30 min |
| RPO | 0 (sources are git-tracked) |
| Owner | axis-foundry |
| Severity | MEDIUM (downstream doc-publish noticeable; CI lane catches the divergence) |

### F-05 — Postgres failover / split-brain

| Attribute | Value |
|---|---|
| Trigger | Primary Postgres node failure; replica election; brief unavailability of writes |
| Blast radius | Lane writes paused; lane runs queue up; admission-gate verdict queries fall back to read-replica (potentially stale) |
| Detection | Patroni/PgBouncer monitoring; Grafana alert `postgres_primary_unhealthy`; OnCall page |
| Mitigation | HA Postgres via Patroni (per `iac/helm/postgres/values.yaml`); 2 sync replicas; automatic failover with bounded write-loss = 0 (sync replication) |
| Recovery | Auto-failover ≤30s; queued lane writes flush within 60s of recovery; admission-gate falls back to read-replica with explicit `stale-replica` tag in verdict |
| RTO | ≤2 min (auto-failover) |
| RPO | 0 (sync replication; no committed-write loss) |
| Owner | ops-sre-reliability |
| Severity | HIGH |

### F-06 — S3 / object-storage unavailable (regional outage)

| Attribute | Value |
|---|---|
| Trigger | OCI Object Storage regional outage (rare; per OCI SLA 99.9% monthly) |
| Blast radius | Evidence blob writes fail; new Findings persist to Postgres with `evidence_pending` state |
| Detection | Evidence-emitter circuit-breaker opens; Grafana alert `evidence_write_failure_rate > 1%`; OnCall page |
| Mitigation | Local outbox: evidence-emitter writes blob to local PV first, then to S3 async; reconciler re-attempts S3 write until success |
| Recovery | S3 recovers → outbox drains; per-blob latency p99 returns < 1s |
| RTO | bounded by OCI recovery; typical ≤2h |
| RPO | 0 (local-PV outbox is durable; sealed via audit-chain) |
| Owner | ops-sre-reliability |
| Severity | HIGH |

### F-07 — GitHub Actions outage

| Attribute | Value |
|---|---|
| Trigger | GitHub Actions vendor outage (sporadic; per GitHub status history ≤8h/year) |
| Blast radius | No lane runs dispatched; no PR admission decisions; merge queue stalls |
| Detection | Workflow-dispatch retries fail; Grafana alert `gha_workflow_dispatch_failure_rate > 50%`; OnCall page; GitHub status API polled every 60s |
| Mitigation | ARC self-hosted runners → independent of GitHub.com Actions runner pool; only the orchestration plane (workflows) depends on GitHub |
| Recovery | Workflows resume on GitHub recovery; queued PRs admit in order |
| RTO | bounded by GitHub recovery; typical ≤4h |
| RPO | 0 (PRs are durable on GitHub; lane runs re-issue on recovery) |
| Owner | ops-sre-reliability |
| Severity | HIGH (every PR blocked; merge-queue stalled) |

### F-08 — Industry-baseline refresh fetch fails

| Attribute | Value |
|---|---|
| Trigger | External baseline source (slsa.dev, csrc.nist.gov, owasp.org) unreachable at quarterly refresh |
| Blast radius | Quarterly refresh PR fails to open; pinned baseline becomes stale; conformance drift visible but uncorrected |
| Detection | `oya-governance-baseline-refresh` cron emits `baseline-refresh-fetch-failure` Finding; Grafana alert |
| Mitigation | Retry with exponential backoff (1h, 6h, 24h); fall back to cached baseline from previous quarter; refresh PR opens with `unable-to-refresh` annotation |
| Recovery | Manual re-fetch when source recovers; PR auto-re-opens |
| RTO | ≤72h (waiting for vendor recovery is acceptable for quarterly cadence) |
| RPO | 0 (pinned baseline still authoritative; refresh is additive) |
| Owner | council-architecture |
| Severity | LOW (quarterly cadence; non-urgent) |

### F-09 — Lane registry corruption (Postgres `lanes` table inconsistency)

| Attribute | Value |
|---|---|
| Trigger | Concurrent lane (re)registration; broken migration; manual DML on `lanes` |
| Blast radius | Lane-runtime worker cannot dispatch jobs; matrix-fanout fails |
| Detection | Lane-runtime worker startup health-check fails; Grafana alert `lane_registry_inconsistent`; OnCall page |
| Mitigation | Idempotent re-registration per `lane-execution.md` Invariant 8; uniqueness constraint on `(lane_id, version)`; ADR-required for any manual DML |
| Recovery | Truncate + re-register from workspace; Postgres backup PITR if no cleaner option |
| RTO | ≤1h |
| RPO | ≤1 PITR cycle (typically ≤15 min) |
| Owner | axis-foundry |
| Severity | HIGH |

### F-10 — Cedar policy fragment mis-deployment

| Attribute | Value |
|---|---|
| Trigger | Policy fragment edit deploys but is unparseable; or a forbid rule is too broad (denies legitimate reads) |
| Blast radius | API gateway refuses reads; tenants cannot access own Findings |
| Detection | Pre-deploy Cedar `validate` (in CI); post-deploy synthetic probe (replay-query as test tenant); Grafana alert `cedar_authz_deny_rate > 50%`; OnCall page |
| Mitigation | Cedar `validate` on every PR touching `policy/*.cedar`; staged rollout via `iac/kustomize/overlays/staging/`; canary deploy with 10% traffic |
| Recovery | Rollback via Git revert + `kubectl apply` of previous overlay |
| RTO | ≤15 min |
| RPO | 0 |
| Owner | ops-security |
| Severity | HIGH |

### F-11 — Lane bypass via admin-merge (T-E-01 occurrence)

| Attribute | Value |
|---|---|
| Trigger | ops-security operator uses GitHub admin-merge to bypass `required_status_checks` |
| Blast radius | Unverified PR lands on `dev`; downstream gates rely on contract being honoured |
| Detection | GitHub audit log webhook → `oya-check-protection-context-match` Finding (severity = AUDIT-CRITICAL); OnCall page within 5 min |
| Mitigation | `enforce_admins = true`; break-glass procedure requires two ops-security signatures + recorded justification per `runbooks/lane-bypass-emergency.md` |
| Recovery | Post-incident: revert PR if justification was insufficient; record retrospective in `evidence/audits/break-glass/<incident-id>.md` |
| RTO | bounded by incident response; ≤1h for revert decision |
| RPO | 0 (audit log is durable; bypass is replayable) |
| Owner | ops-security + council-architecture |
| Severity | CRITICAL (intentional gate bypass) |

### F-12 — Quarterly refresh promotes a softer baseline

| Attribute | Value |
|---|---|
| Trigger | Industry source publishes a less-stringent baseline (rare but possible; e.g., a spec relaxation) |
| Blast radius | Auto-PR proposes softer baseline pin; if merged without review, conformance posture weakens |
| Detection | PR auto-opened with `softer-baseline-proposed` label; council-architecture reviewer notified; `oya-check-claim-ceiling` lane refuses if claimed posture decreases |
| Mitigation | Quarterly refresh PR is opened by `axis-foundry-bot` but cannot self-merge; requires council-architecture explicit approval + ADR follow-up rationale per ADR-0133 §"Operational" |
| Recovery | Reject PR if softening unjustified; close with rationale; restore prior pin |
| RTO | bounded by review SLA; ≤1 week |
| RPO | 0 |
| Owner | council-architecture |
| Severity | HIGH (claim-ceiling honesty at stake) |

### F-13 — Self-application bootstrap paradox

| Attribute | Value |
|---|---|
| Trigger | Governance's own conformance lane fails on the governance µservice itself (e.g., a new lane added to the suite that retroactively fails governance's own structure) |
| Blast radius | No PR to governance can pass the suite; governance becomes self-locked |
| Detection | All PRs to `microservices/governance/` fail BLOCKER; Grafana alert `governance_self_lock`; OnCall page |
| Mitigation | Synthetic-probe fallback per PRD Open Q3: governance lanes run in self-application mode with a 24h amnesty window for first-application failures; council-architecture override available via break-glass |
| Recovery | Either (a) fix governance to satisfy the new lane in same PR; (b) defer the new lane via ADR until governance can comply; (c) break-glass merge with retroactive remediation IP |
| RTO | bounded by remediation; ≤24h |
| RPO | 0 |
| Owner | axis-foundry + council-architecture |
| Severity | HIGH |

### F-14 — Runner-pool autoscaler stuck

| Attribute | Value |
|---|---|
| Trigger | ARC controller fails to scale up despite load (e.g., quota exhaustion; node-group at max) |
| Blast radius | Lane runs queue up; per-PR p99 latency exceeds SLO |
| Detection | Grafana alert `arc_runner_pool_queue_depth_seconds > 60`; OnCall page |
| Mitigation | Pre-warmed pool of 8 standbys; quota alerts pre-emptively at 70% utilization; per-node-group max-replicas conservative initial |
| Recovery | Request quota increase from cloud provider; spill to spot-instance pool; manual scale of node-group |
| RTO | ≤4h (quota increase) |
| RPO | 0 |
| Owner | ops-sre-reliability |
| Severity | MEDIUM |

### F-15 — Aggregation-indexer scoped-PAT overrun (T-E-03 occurrence)

| Attribute | Value |
|---|---|
| Trigger | Aggregation-indexer attempts to write outside its scoped paths (rare; bug in indexer logic) |
| Blast radius | Either (a) GitHub PAT scope rejects write — clean failure; or (b) PAT scope is too permissive — central files outside allow-list mutated |
| Detection | GitHub returns 403 for case (a); `oya-check-aggregation-index-generation` lane fails on case (b); Grafana alert `aggregation_scope_overrun_total > 0` |
| Mitigation | Pre-push hook in aggregation-indexer asserts every path is in scope-allow-list; PAT scope reviewed quarterly per `runbooks/aggregation-rebuild.md` |
| Recovery | (a) Fix indexer logic + retry; (b) Git revert + investigate; ADR if recurring |
| RTO | ≤2h |
| RPO | 0 |
| Owner | axis-foundry + ops-security |
| Severity | HIGH (T-E-03 elevation-of-privilege class) |

## Cross-cutting Failure Modes (inherited)

| Source | Inheritance | Reference |
|---|---|---|
| `audit-chain` µservice | seal latency; key compromise | `microservices/audit-chain/failure-modes.md` |
| `observability` µservice | metric ingest gaps; SLO evaluator stale | `microservices/observability/failure-modes.md` |
| `cloud-secrets` µservice | OpenBao token rotation failure | `microservices/cloud-secrets/failure-modes.md` |
| `cloud-k8s` µservice | node failure; network partition | `microservices/cloud-k8s/failure-modes.md` |
| `tenancy` µservice | event-bus delivery loss | `microservices/tenancy/failure-modes.md` |

## RTO / RPO summary

| Severity class | RTO target | RPO target |
|---|---|---|
| CRITICAL (F-11) | ≤15 min for detection + ≤1h for revert | 0 |
| HIGH (F-02, F-03, F-05, F-06, F-07, F-09, F-10, F-12, F-13, F-15) | ≤2h | 0 |
| MEDIUM (F-01, F-04, F-14) | ≤4h | 0 |
| LOW (F-08) | ≤72h | 0 |

## Verification

- Per-failure-mode drill executed annually (`runbooks/<runbook>.md` documents drill steps).
- Game-day exercise quarterly: ops-sre-reliability fires synthetic F-01..F-07; measures actual RTO; records gaps.
- Per-incident postmortem: every realized failure produces an entry in `evidence/audits/postmortems/<incident-id>.md`.

## References

- `microservices/governance/runbooks/*.md` (6 runbooks).
- `microservices/governance/threat-model.md`.
- `microservices/governance/dashboards/*.json`.
- `microservices/observability/failure-modes.md` (shape reference).
- Google SRE Workbook ch. 7 (managing incidents).
- AWS Well-Architected Framework — Reliability Pillar.
