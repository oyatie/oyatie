# Agentic SLO-Gated Promotion (dev → staging → production)

## Problem Statement
How might we make every fast-forward of `staging` and `production` strictly
conditional on hyperscaler-grade SLO evidence, so no human and no LLM are
in the loop, yet bad code cannot reach production within the existing
auto-promotion cadence?

## Master Plan Placement

**Milestone 1 (foundation).** This work is substrate consumed by every
oyatie product; it cannot land in a later milestone without leaving M01
products promoting unguarded. Per the M-CC-folds-into-M01 rule, the
existing `M-CC-P11` (retired VCS ratchet substrate) renumbers into M01, and this
substrate is either an extension of that phase or two sibling sub-phases
of M01 covering observability (Layer A) and promotion gating (Layer B)
respectively. Final sub-phase decomposition is captured in the ADR (#18)
and the master-plan amendment that accompanies it.

## Recommended Direction

Adopt the industry-leading OSS observability stack as the runtime substrate;
own the differentiator on top. Promotion becomes event-driven and
per-component, gated by a Foundry-native SLO engine implementing the
canonical Google SRE multi-window multi-burn-rate model on top of
Prometheus / Mimir time-series storage.

### Layer A — adopted OSS (commodity, deployed not written)

All Apache-2.0; deployed alongside oyatie services; configured-as-code, not
re-implemented. This IS the hyperscaler-grade practice (AWS Managed
Prometheus, GCP Managed Service for Prometheus, Grafana Cloud all run
exactly this stack internally).

1. **Grafana Alloy** — unified OpenTelemetry collector (metrics + logs +
   traces + profiles). Replaces what would have been a custom collector.
2. **Prometheus + Mimir** — TSDB + horizontally-scalable long-term metrics
   storage. PromQL is the canonical query language.
3. **Loki** — log aggregation + LogQL.
4. **Tempo** — distributed trace storage + TraceQL.
5. **Pyroscope** — continuous profiling.
6. **Grafana** — visualization, dashboards, ad-hoc query UI.
7. **Alertmanager** — alert routing + deduplication + silencing.
8. **Grafana OnCall** (Apache-2.0) — incident-paging integration.
   Replaces any PagerDuty/Opsgenie dependency.

### Layer B — oyatie owned differentiator (where engineering goes)

The IP that makes the gate agentic + Foundry-native. Nothing here is
deferred; everything ships as one unit.

9.  **OpenSLO manifests per component** — `slo/<component>.openslo.yaml`
    (industry-standard OpenSLO spec; same shape used by Sloth, Nobl9).
    SLI, 30-day SLO target, error-budget windows, burn-rate alert
    thresholds (2 % / 1 h fast burn, 5 % / 6 h slow burn — Google SRE
    workbook Ch. 5).
10. **`observability-slo-engine` crate** — reads OpenSLO manifests,
    queries Prometheus/Mimir via PromQL for burn-rate computation, emits
    per-component eligibility verdicts. The differentiator: it knows about
    components, ledger, release pointers, and the agentic gate. The TSDB
    is upstream; the SLO model is ours.
11. **Per-component release pointers** — ref naming
    `release/<component>/<environment>`. Single tree-wide `staging` /
    `production` refs deprecated. Each component fast-forwards
    independently.
12. **`registry/promotion-eligibility.jsonl`** — append-only ledger of
    `(component, source_sha, target_env, verdict, burn_rate_snapshot,
    evaluated_at, evaluator_version)`. Union-merge driver already in
    `.gitattributes` for JSONL ledgers.
13. **`retired VCS ratchet` CI lane** — asserts the latest ledger
    record for every component touched by the SHA is `eligible` for the
    target environment. Added to `branch-protection.yaml`
    required-status-checks on `dev` and `staging`.
14. **Continuous burn-rate evaluator** — runs inside
    `observability-slo-engine`; 1-minute cadence; writes eligibility
    records for every (component, environment, current-sha) tuple.
    Idempotent. Event source.
15. **Event-driven promote workflows** — `promote-dev-to-staging.yml` and
    `promote-staging-to-production.yml` rewritten to fire on
    `repository_dispatch` event `eligibility-changed`, emitted by the
    evaluator. 30-min / 1-h crons retained only as reconciliation
    heartbeat, not as primary trigger.
16. **Automated rollback primitive** — production-tier burn-rate breach
    triggers `release/<component>/production` to fast-forward back to the
    prior ledger pointer. Signed, linear, recorded as `rollback` verdict.
17. **Canary cohort weighting** — staging traffic ramped progressively
    (1 % → 10 % → 50 % → 100 %) per component via Layer-A service-mesh
    traffic-split, so burn-rate windows accumulate real signal before
    production promotion. Without this the gate is theatre.
18. **ADR-####** — captures: chosen design; explicitly-rejected paths
    (custom TSDB / proprietary observability stack / LLM-reasoning gate /
    rollback-first / single-error-budget / monorepo-wide refs /
    Datadog-Honeycomb-vendored); the adopt-vs-build boundary line; the
    hyperscaler-citation matrix (Google SRE workbook, OpenSLO, OTel,
    Grafana stack adoption by AWS / GCP / Cloudflare / Shopify).
19. **`branch-protection.yaml` update** — adds
    `retired VCS ratchet` to required checks on `dev` and
    `staging`.
20. **Decommission of FUTURE-marked stubs** — references in
    `promote-dev-to-staging.yml` and `promote-staging-to-production.yml`
    to `governance-canary-cohort-observability` /
    `-full-rollout-observability` are replaced by Layer-B components; no
    placeholder lanes remain.

## Key Assumptions to Validate

- [ ] PromQL is sufficient to express every burn-rate predicate oyatie
      needs — validated by: write the worst-case multi-window
      multi-burn-rate query against a representative component SLO;
      confirm no expressiveness gap drives us to a custom DSL.
- [ ] Self-hosted Mimir at oyatie's scale is operationally reasonable for
      the team — validated by: capacity estimate for 30-day retention
      across all components at projected traffic; confirm single-tenant
      Mimir cluster fits on commodity hardware and a small ops budget.
- [ ] OpenSLO covers every SLI shape oyatie needs (availability, latency,
      correctness, freshness) — validated by: enumerate SLI types in
      master plan, confirm each maps to an OpenSLO indicator type.
- [ ] Per-component release pointers don't collide with `cargo` workspace
      assumption that one repo HEAD describes all crates — validated by:
      confirm release pipelines and supply-chain attestation don't
      require monolithic SHA.
- [ ] Canary cohort weighting is implementable for every deployable
      component before this ships — validated by: catalog deployment
      surfaces; confirm each has traffic-split capability.

## Minimum-shippable scope

There is no smallest-actionable subset. The design ships as one unit. The minimum implementable
slice is the full 20-piece substrate above, delivered as one master-plan
phase. The work decomposes into IPs but does not ship partially: a partial
gate is a permissive gate.

## Not Doing (and Why)

- **Build a parallel proprietary observability stack** — long-term-wrong;
  industry-leading practice IS to adopt the OSS leaders. The
  differentiator is the SLO engine + agentic gate, not the TSDB.
- **Vendor-managed observability (Datadog / GCP Ops / Honeycomb)** —
  ruled out by adopt-OSS-leaders choice; would couple the gate to an
  external read endpoint.
- **LLM-in-loop reasoning gate** — deterministic thresholds with an open
  evidence trail is the industry-standard answer.
- **Single-error-budget gate** — strict subset of multi-window
  burn-rate; would be deferral by another name.
- **Monorepo-wide single release ref** — coarse for a flat product
  catalog; per-component is the industry-leading shape (Linear, Stripe,
  Google per-binary).
- **Rollback-first as the primary gate** — rollback is a co-delivered
  safety net, not the gate. Permitting bad ships and reverting is below
  the bar.

## Open Questions

- ADR number for the design capture (Layer-A/Layer-B boundary, rejected
  paths, hyperscaler-citation matrix).
- M01 sub-phase decomposition: extension of the renumbered `M-CC-P11`
  vs. two sibling sub-phases (observability + promotion-gate). Decided in
  the master-plan amendment that ships with the ADR.
- **Bootstrap ordering** within M01: (a) gate lights up incrementally —
  each component lands with its OpenSLO manifest, so coverage grows with
  M01; or (b) gate lands last and retroactively gates earlier components.
  (a) is the hyperscaler answer; (b) is the simpler integration.
- Whether the burn-rate evaluator runs as a long-lived service or as a
  scheduled job inside `observability-slo-engine` against Prometheus
  snapshots. (Long-lived service is the hyperscaler answer.)
- How `release/<component>/<environment>` ref proliferation interacts
  with GitHub branch-protection (per-rule cap on matched refs); may need
  pattern-based protection rules.
- Where the Layer-A cluster runs (single Kubernetes cluster alongside
  staging/prod workloads, or dedicated observability cluster).
  Hyperscaler practice favours dedicated; cost-honest answer for
  early-stage favours shared.
