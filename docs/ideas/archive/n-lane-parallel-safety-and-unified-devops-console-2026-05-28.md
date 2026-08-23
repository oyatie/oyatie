---
title: "N-lane parallel safety proof and unified DevOps console"
status: superseded
superseded_by: ADR-0391
date: 2026-05-28
companion_docs:
  - cloud-intelligence-v1-pipeline-2026-05-28.md
  - cloud-intelligence-bedrock-on-talos-2026-05-28.md
---

# N-lane parallel safety proof and unified DevOps console

**Status**: ideation artifact (2026-05-28).
**Companion docs**:
- `cloud-intelligence-v1-pipeline-2026-05-28.md` — v1 request pipeline (8 stages)
- `cloud-intelligence-bedrock-on-talos-2026-05-28.md` — positioning + phased delivery

## Problem Statement

Two gaps block production confidence for cloud-intelligence v1 and the broader N-lane parallel agent model:

1. **Safety proof gap**: N lanes of parallel agent work (cloud-intelligence lanes K/R/Z/A/C/N) run concurrently on disjoint crate paths. There is no gate that verifies the disjoint-path invariant holds — that no two lanes touch the same file. Without this, merging a batch of parallel PRs risks silent cross-lane collisions.

2. **Visibility gap**: there is no unified operator console. Founders and operators currently monitor lane progress, subscription-pool health, token-window utilization, and proof harness results across disconnected tools (GitHub PRs, Jenkins build history, Grafana dashboards, CLI commands). This creates alert fatigue, slow incident response, and no single pane of glass for the cloud-intelligence service state.

This idea-pager proposes: (a) a formal N-lane parallel safety proof layer, and (b) a DevOps console v0 (operator-only, not tenant-facing) that aggregates the key signal surfaces.

## Recommended Direction

### Part A — N-lane parallel safety proof

The lane model (ADR-0388 + parallel-swarm memory) already requires disjoint file paths per lane. The missing piece is automated verification at PR time.

**Proposed gate: `presubmit` (retired CLI `gate validate lane-overlap`)**

```
For each open PR in the current merge batch:
  1. git diff --name-only origin/dev...HEAD  →  file_set(lane_i)
  2. Assert: ∀ i ≠ j, file_set(lane_i) ∩ file_set(lane_j) = ∅
  3. On violation: emit the overlapping paths + PR pair; block merge.
```

This runs as part of `oya verify --ci-required` in the Jenkins pipeline.

**Proof properties (beyond the gate)**:

- **Loom**: the SubscriptionPool's `lease/complete` interleaving proof (P2 in the pipeline doc) IS the N→∞-safe concurrency proof for the OAuth-pool kernel. It must cover N=100 concurrent simulated lanes.
- **proptest**: `SeatId` uniqueness under N concurrent leases — no two active `SeatLease` values ever hold the same `SeatId`.
- **Integration**: the chaos harness (N=50 simulated tenants × K=1000 concurrent requests) is the end-to-end proof that N-lane load does not produce pool corruption.

**Formal invariants to prove**:

| Invariant | Proof method | Scope |
|---|---|---|
| No two lanes touch the same file | Gate (git diff set-intersection) | All parallel PRs in a merge batch |
| No two active `SeatLease` hold the same `SeatId` | Loom (N concurrent tasks, exhaustive interleavings) | P2 kernel |
| Pool invariant: lease_count + free_count = seat_count | proptest state machine | P2 kernel |
| Receipt monotonicity: no receipt is updated after emit | proptest | P4 |
| Cedar forbid-wins: cross-tenant requests always Forbid | proptest (10 adversarial tests) | P1 |
| Audit consumer resumes after kill: no receipts dropped | Chaos harness | P7 + Valkey Stream |

### Part B — DevOps console v0

**Scope (operator-only, v0)**:

The console is a microservice (`microservices/devops-console/`) — a SolidJS frontend served by an Axum backend — that aggregates:

1. **Subscription admin panel**: list tenants, per-tenant seat pool state (Active/Reserved/Cooldown/Blacklisted count), 5h/weekly token window utilization bars, last-refresh timestamp per seat.
2. **Lane progress board**: open PRs per lane (GitHub API), their CI status (Jenkins API), claimed deliverables (masterplan claim refs), merge-queue position.
3. **Proof harness results**: last Loom run pass/fail + interleaving count, last proptest run shrink log, last chaos run fault-injection summary.
4. **Audit chain health**: `cloud_intelligence_p7_audit_lag_seconds` gauge, chain depth, last Sigstore attestation timestamp.
5. **Gateway health**: P0 in-flight requests, P3 upstream error rate by provider, P6 response status distribution (last 5 min).

**Architecture**:

```
                  ┌──────────────────────────────────────────┐
                  │         devops-console (microservice)      │
                  │                                           │
  browser ───────>│  SolidJS SPA (axum static-file serve)    │
                  │  ┌────────────────────────────────────┐  │
                  │  │  /api/subscriptions  (admin API)    │  │
                  │  │  /api/lanes          (GitHub API)  │  │
                  │  │  /api/proof          (CI API)       │  │
                  │  │  /api/metrics        (Prometheus)   │  │
                  │  └────────────────────────────────────┘  │
                  │                                           │
                  │  Cedar: operator/founder realm only       │
                  │  (same Cedar policy as cloud-intelligence) │
                  └──────────────────────────────────────────┘
```

The console backend is a thin aggregator: it calls the cloud-intelligence admin API, GitHub API, Jenkins API, and Prometheus query API. No new data storage — all state lives in existing systems.

**v0 deliverables**:
- `microservices/devops-console/` crate scaffold (ADR-0131 flat layout).
- Subscription admin panel (read-only in v0; seat pool state + token windows).
- Lane progress board (PRs + CI status + claim refs).
- Single-page health overview: P0/P3/P6/P7 metrics as sparklines.

**v1+ (out of v0 scope)**:
- Tenant-facing dashboard (blocked on ≥3 tenants enrolled).
- Write operations (seat provisioning, policy reload) from the console.
- Proof harness trigger from the console UI.
- Alerting rules (Grafana-backed) wired to console notification panel.

## Key Assumptions

- [ ] **Lane-overlap gate can compute git diff set-intersection in < 5s for 10 open PRs.** Validate: benchmark `git diff --name-only` × 10 in a repo of this size.
- [ ] **SolidJS SPA + Axum static-file serve fits in a single flat-layout µservice.** Validate: scaffold the crate, confirm it compiles and serves a hello-world SPA.
- [ ] **Prometheus query API on Talos is accessible from the devops-console backend without additional RBAC.** Validate: curl the Prometheus HTTP API from a pod in the `cloud-intelligence` namespace.
- [ ] **GitHub API returns PR CI status in a single call (no N+1 on lane count).** Validate: review GitHub API docs for batch commit-status endpoint.

## Not Doing (and Why)

- **Tenant-facing dashboard in v0** — operator/founder console only; tenant-facing UI lands once we have ≥3 tenants.
- **Write operations from console v0** — read-only is sufficient for visibility; writes require additional Cedar policy + audit trail work.
- **Alerting rules in v0** — Grafana already has alert routing; the console adds visibility, not a parallel alert system.
- **WebSocket real-time push in v0** — polling every 10s is sufficient for operator use; push lands in v1.

## Open Questions

- **Lane-overlap gate: where does the "merge batch" boundary come from?** The merge queue (ADR-0111) projects the merge order; the gate needs to know which PRs are in the current batch. Option: gate reads the merge-queue projected state file. Validate design with ADR-0111.
- **Console auth: OpenBao JWT or cluster-internal mTLS?** For v0, mTLS sidecar (Istio) is sufficient — the console is only accessible from within the cluster or via `kubectl port-forward`. JWT-based auth lands in v1 when we expose the console externally.
- **Proof harness results storage**: where does the last Loom/proptest/chaos result live? Options: (a) Jenkins build artifacts, (b) a `proof-results` Valkey key updated by CI, (c) a dedicated table in ClickHouse. Lean Valkey key for v0 (simplest); ClickHouse for v1 (queryable history).
