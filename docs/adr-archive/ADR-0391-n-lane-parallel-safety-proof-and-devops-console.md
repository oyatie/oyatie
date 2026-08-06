---
id: ADR-0391
title: "N-lane parallel safety proof and unified DevOps console"
status: Superseded
date: 2026-05-28
authority: founder
owner: council-architecture
planning_impact: true
supersedes: []
superseded_by: [ADR-0709]
related: [ADR-0388]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0391 — N-lane parallel safety proof and unified DevOps console

## Status

Accepted — 2026-05-28.

## Context

Two gaps block production confidence for cloud-intelligence v1 and the broader N-lane parallel agent model:

1. **Safety proof gap**: N lanes of parallel agent work (cloud-intelligence lanes K/R/Z/A/C/N per ADR-0390) run concurrently on disjoint crate paths. There is no gate that verifies the disjoint-path invariant holds — that no two lanes touch the same file. Without this, merging a batch of parallel PRs risks silent cross-lane collisions.

2. **Visibility gap**: there is no unified operator console. Founders and operators currently monitor lane progress, subscription-pool health, token-window utilization, and proof harness results across disconnected tools (GitHub PRs, Jenkins build history, Grafana dashboards, CLI commands). This creates alert fatigue, slow incident response, and no single pane of glass for the cloud-intelligence service state.

The parallel-swarm model (per project memory) already requires disjoint file paths per lane and claims-aware one-service-per-lane isolation. ADR-0388 establishes the doc-axis convention. This ADR formalises: (a) the N-lane parallel safety proof layer as a first-class gate, and (b) the DevOps console v0 (operator-only) microservice.

## Goals

1. Define the `oya gate validate lane-overlap` gate and its algorithm.
2. Specify the formal invariants provable by Loom + proptest + chaos for the N-lane model.
3. Define the DevOps console v0 microservice architecture and v0 scope.
4. Establish the console as an aggregator (no new data storage) over existing system APIs.

## Non-Goals

- Tenant-facing dashboard (blocked on ≥3 tenants enrolled).
- Write operations from console v0 (read-only; writes in v1).
- Alerting rules in v0 (Grafana already has alert routing; console adds visibility, not a parallel alert system).
- WebSocket real-time push in v0 (polling every 10s is sufficient for operator use; push in v1).
- Lane-overlap gate for non-cloud-intelligence lanes (generalisation deferred until the gate is proven stable on cloud-intelligence lanes).

## Proposal

### Part A — N-lane parallel safety proof

#### Gate: `oya gate validate lane-overlap`

```
For each open PR in the current merge batch:
  1. git diff --name-only origin/dev...HEAD  →  file_set(lane_i)
  2. Assert: ∀ i ≠ j, file_set(lane_i) ∩ file_set(lane_j) = ∅
  3. On violation: emit the overlapping paths + PR pair; block merge.
```

This gate runs as part of `oya verify --ci-required` in the Jenkins pipeline. The "merge batch" boundary is derived from the merge-queue projected state (ADR-0111): PRs that are concurrently enqueued constitute a batch.

#### Formal invariants

| Invariant | Proof method | Scope |
|---|---|---|
| No two lanes touch the same file | Gate (`git diff` set-intersection) | All parallel PRs in a merge batch |
| No two active `SeatLease` hold the same `SeatId` | Loom (N concurrent tasks, exhaustive interleavings) | ADR-0390 P2 kernel |
| Pool invariant: lease_count + free_count = seat_count | proptest state machine | ADR-0390 P2 kernel |
| Receipt monotonicity: no receipt updated after emit | proptest | ADR-0390 P4 |
| Cedar forbid-wins: cross-tenant requests always Forbid | proptest (10 adversarial tests) | ADR-0390 P1 |
| Audit consumer resumes after kill: no receipts dropped | Chaos harness | ADR-0390 P7 + Valkey Stream |

The Loom proof for `SubscriptionPool::lease/complete` (ADR-0390 P2) is the N→∞-safe concurrency proof for the OAuth-pool kernel. It must cover N=100 concurrent simulated lanes with exhaustive interleaving exploration. The chaos harness (N=50 simulated tenants × K=1000 concurrent requests) is the end-to-end proof that N-lane load does not produce pool corruption.

### Part B — DevOps console v0

#### Microservice identity

- **Path**: `microservices/devops-console/` (ADR-0131 flat layout, ADR-0132 no-suite).
- **Stack**: Leptos/Rust-WASM shell served by an Axum static-file backend (ADR-0393). Single-concern: aggregation + display. No new data storage.
- **Auth**: mTLS sidecar (Istio) for v0 — console accessible from within the cluster or via `kubectl port-forward` only. JWT-based auth for external access lands in v1.
- **Cedar**: operator/founder realm only (same Cedar policy as cloud-intelligence gateway).

#### Architecture

```
                  ┌──────────────────────────────────────────┐
                  │         devops-console (microservice)      │
                  │                                           │
  browser ───────>│  Leptos shell (axum static-file serve)   │
                  │  ┌────────────────────────────────────┐  │
                  │  │  /api/subscriptions  (admin API)    │  │
                  │  │  /api/lanes          (GitHub API)  │  │
                  │  │  /api/proof          (Jenkins API)  │  │
                  │  │  /api/metrics        (Prometheus)   │  │
                  │  └────────────────────────────────────┘  │
                  │                                           │
                  │  Cedar: operator/founder realm only       │
                  └──────────────────────────────────────────┘
```

The backend is a thin aggregator: it calls the cloud-intelligence admin API (ADR-0390 Lane A), GitHub API, Jenkins API, and Prometheus query API. No new data storage — all state lives in existing systems.

#### v0 panel specifications

1. **Subscription admin panel**: list tenants; per-tenant seat pool state (Active/Reserved/Cooldown/Blacklisted count); 5h/weekly token window utilization bars; last-refresh timestamp per seat. Source: cloud-intelligence admin API (ADR-0390 Lane A).

2. **Lane progress board**: open PRs per lane (GitHub API); their CI status (Jenkins API); claimed deliverables (masterplan claim refs, `oya plan status`); merge-queue position (ADR-0111 projected state). Source: GitHub + Jenkins APIs + masterplan refs.

3. **Proof harness results**: last Loom run pass/fail + interleaving count; last proptest run shrink log; last chaos run fault-injection summary. Source: Valkey key `proof-results:<suite>` updated by CI (lean Valkey for v0; ClickHouse history in v1).

4. **Audit chain health**: `oya_cloud_intelligence_p7_audit_lag_seconds` gauge; chain depth; last Sigstore attestation timestamp. Source: Prometheus.

5. **Gateway health**: P0 in-flight requests; P3 upstream error rate by provider; P6 response status distribution (last 5 min). Source: Prometheus.

#### v0 deliverables

| Deliverable | Exit criteria |
|---|---|
| `microservices/devops-console/` scaffold | ADR-0131 flat layout; `cargo check` passes; axum serves `/healthz`. |
| Subscription admin panel (read-only) | Calls cloud-intelligence admin API; renders seat pool state + token windows. |
| Lane progress board | Calls GitHub + Jenkins APIs; renders PR list + CI status + claim refs. |
| Health overview | Renders P0/P3/P6/P7 Prometheus metrics as sparklines on a single page. |

## Alternatives Considered

| Alternative | Reason rejected |
|---|---|
| Extend Grafana instead of a new console | Grafana covers metrics; does not aggregate GitHub PRs, masterplan claim refs, or proof harness results. The console is a different information class. |
| Per-lane overlap check at commit time (git hook) | Commit-time hooks run in an agent's worktree context; they can't see other lanes' in-flight diffs. A PR-level gate has the full picture. |
| WebSocket real-time push in v0 | Polling at 10s intervals is sufficient for operator use; WebSocket adds server-side state management complexity for marginal latency gain. |
| Write operations in console v0 | Read-only is sufficient for visibility; writes (seat provisioning, policy reload) require additional Cedar policy + audit trail work that is out of v0 scope. |
| ClickHouse for proof harness results in v0 | Valkey key is simpler for v0; ClickHouse adds queryable history which is only useful once there are enough runs to trend. |

## Cross-Cutting Concerns

- **ADR-0131 flat layout**: `microservices/devops-console/` with `src/` as canonical code root.
- **ADR-0132 no-suite**: single-concern µservice (aggregation + display); no bundle grouping.
- **Dogfood tenancy**: the console itself runs as an oyatie-dogfood tenant workload; it traverses the same Cedar authorization path as any other operator-realm service.
- **Observability**: the console backend exports its own `/metrics` endpoint (`oya_devops_console_api_requests_total{endpoint, status}`).
- **ADR-0111 merge-queue**: the lane-overlap gate reads the merge-queue projected state to determine batch membership.

## Migration Plan

- The lane-overlap gate is additive: it adds a new `oya gate validate lane-overlap` check to `oya verify --ci-required`. No existing gates are removed or modified.
- The DevOps console is a new microservice; no existing microservices are modified. The cloud-intelligence admin API (ADR-0390 Lane A) must ship before the console subscription panel can display real data. Until then, the panel renders an empty state with a "pending Lane A" notice.

## Open Issues

- [ ] **Lane-overlap gate: merge batch boundary from ADR-0111 projected state.** Validate the interface: does `oya gate validate lane-overlap` read the projected-state file directly, or does it call `oya gen board-sync`? Design with ADR-0111 merge-queue owner.
- [ ] **Lane-overlap gate performance**: validate `git diff --name-only` × 10 PRs in < 5s in a repo of this size.
- [ ] **Leptos shell + Axum single flat-layout µservice**: scaffold the crate, confirm it compiles and serves a hello-world shell.
- [ ] **Prometheus RBAC on Talos**: confirm Prometheus query API is accessible from the `cloud-intelligence` namespace pod without additional RBAC.
- [ ] **GitHub batch commit-status endpoint**: review GitHub API docs for batch endpoint to avoid N+1 on lane count.
- [ ] **Console auth v1 path**: document the JWT-based auth design for external console access (separate follow-up ADR or ADR-0391 amendment).
