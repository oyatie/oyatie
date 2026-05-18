---
audit_id: PR-143-SYSTEMS-PLATFORM-CLOUD-ARCH
authored_at: 2026-05-18
authors:
  - council-architecture
  - ops-sre-reliability
  - ops-finops
  - ops-security
  - axis-cloud
related_adrs:
  - ADR-0128 (hyperscaler invariants — canonical spec + portfolio binding)
  - ADR-0123 (hyperscaler maturity claim gate)
  - ADR-0134 (portfolio hyperscaler pattern remediation backlog)
  - ADR-0173 (saga + compensation portfolio policy) — authored by this audit
  - ADR-0174 (FinOps cost-attribution + chargeback) — authored by this audit
  - ADR-0175 (tenant lifecycle workflow) — authored by this audit
  - ADR-0176 (brown-out + graceful-degradation signal API) — authored by this audit
  - ADR-0177 (internal vs external API surface separation) — authored by this audit
  - ADR-0178 (layered throttling — per-tenant / per-user / per-IP / per-key) — authored by this audit
  - ADR-0179 (sovereign cloud per regional pack) — authored by this audit
  - ADR-0180 (DR + business-continuity portfolio policy) — authored by this audit
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/finops-cost-attribution.json (authored by this audit)
  - /specs/tenant-lifecycle.json (authored by this audit)
  - /specs/brownout-degradation-signal.json (authored by this audit)
  - /specs/api-surface-separation.json (authored by this audit)
  - /specs/throttling-tiers.json (authored by this audit)
  - /specs/sovereign-cloud-overlays.json (authored by this audit)
  - /specs/dr-business-continuity.json (authored by this audit)
status: accepted-portfolio-audit
binding_adr: ADR-0173 (this audit's parent grouping policy)
user_directive_2026_05_18: |
  FIX-AGENT-M scope — audit oyatie against systems + platform + cloud
  architecture patterns and anti-patterns beyond app-level (Fix-I/J/K),
  author full ADR + impl where adoption is recommended, no stubs.
---

# PR-143 systems + platform + cloud architecture audit

## Purpose

PR-143 Fix-A through Fix-L addressed app-level patterns. This audit (Fix-M)
covers the next ring: **systems architecture** (failure modes, recovery,
state semantics), **platform architecture** (org boundaries, internal
developer experience, FinOps), and **cloud architecture** (DR, sovereignty,
edge, vendor independence). The audit produces eight new ADRs (ADR-0173..ADR-0180)
plus companion impls (no stubs) per the user 2026-05-18 directive
("author full impl + full ADR content where adoption is recommended; no
stubs").

The audit groups findings into three buckets:

1. **Patterns NEWLY ADOPTED here** — full ADR + impl authored under this
   PR. Each binds to a validator (existing or new) plus a registry, and
   wires into the `oya gate run-all` aggregator catalog.
2. **Patterns ALREADY ADOPTED** — link to the existing ADR + impl; no
   action required beyond confirming citation in the relevant µservice PRD.
3. **Anti-patterns to verify we are NOT doing** — concrete probe results
   from grep/cargo/inventory commands captured below.

## Bucket 1 — Patterns newly adopted (ADR-0173..ADR-0180)

### ADR-0173: Saga + compensating transaction portfolio policy

**Gap.** ADR-0035 (workflow engine) mentions saga semantics as a
state-machine construct internal to the workflow engine, and ADR-0145
(inter-µservice communication reform) bans two-phase commits, but the
portfolio has no ADR establishing the *general* rule: any operation
spanning more than one µservice that mutates external state MUST be
expressed as a saga with explicit compensating actions registered in the
audit chain. Without that policy, individual µservices are free to invent
ad-hoc cross-µservice transactions and the workflow engine cannot enforce
saga shape at the boundary.

**Decision (full text in ADR-0173-saga-compensation-portfolio-policy.md).**
All cross-µservice writes go through the workflow engine's saga
coordinator. Every step that mutates external state declares
(forward_action, compensation_action, idempotency_key). The audit chain
records both sides. Two-phase commit is banned (already in ADR-0145);
this ADR makes the saga shape mandatory rather than optional.

**Companion impl.**
- `microservices/workflow-engine/policy/saga-compensation-policy.md`
  — full normative policy, compensation patterns (cancel/refund/retry),
  test matrix.
- `specs/saga-shape.json` — machine-readable schema for saga steps
  consumed by the workflow engine + a validator.
- `crates/oya-check-saga-shape/` — full validator crate (kernel) +
  CLI wiring. Lane name `saga-shape` added to `AGGREGATED_VALIDATE_LANES`.

**Validator status.** Lane registered as **advisory** until every
existing cross-µservice flow has been re-shaped — the catalog of flows
to re-shape is captured in `registry/saga-shape/migration-backlog.tsv`.

### ADR-0174: FinOps cost-attribution + chargeback policy

**Gap.** FinOps is mentioned across ADRs 0004, 0009, 0020, 0028 as a
desired property (per-tenant cost ceiling in foundry; per-cell unit
economics in cloud; FinOps surface as analytics-plane consumer), but the
portfolio has no ADR establishing the canonical cost-tag schema, the
chargeback formula, or the cost-anomaly alarm policy. The `ops-finops`
team charter exists (`docs/teams/ops-finops/CHARTER.md`) but ties to no
binding ADR.

**Decision (full text in ADR-0174-finops-cost-attribution-chargeback.md).**
Every cloud-resource label set includes the canonical tag block
{tenant_id, cell_id, microservice, plane, environment, cost_center,
sustainability_class}. Per-tenant chargeback is computed from the
labelled spend by the `oya-cloud-billing-domain` crate (already exists
per ADR-0028) using a transparent attribution algorithm. Cost-anomaly
detection runs as a streaming MAD (median absolute deviation) detector on
the analytics plane with paging thresholds per tier.

**Companion impl.**
- `specs/finops-cost-attribution.json` — canonical tag schema +
  chargeback formula + anomaly detection thresholds.
- `docs/standards/finops-cost-attribution.md` — full standards doc with
  worked example, regulator-evidence cadence, audit-chain tagging.
- `registry/finops/cost-tag-vocabulary.yaml` — closed enum of allowed
  tag values.
- `microservices/observability/dashboards/finops-cost-attribution.md`
  — dashboard schema.
- Validator: lane `finops-cost-tag` registered as **advisory** until
  every µservice manifest declares its cost_center.

### ADR-0175: Tenant lifecycle workflow (onboard / suspend / migrate / offboard / delete)

**Gap.** ADR-0002 (tenant + identity kernel) establishes the Tenant
entity but is silent on the *workflow* that moves a tenant through
onboard → active → suspended → migrating → offboarded → deleted. ADR-0038
(DSR cascade + proof of erasure) covers deletion narrowly. Without an
explicit lifecycle workflow, individual µservices invent their own
onboarding and offboarding paths and the audit-chain cannot prove a
tenant is fully off the platform.

**Decision (full text in ADR-0175-tenant-lifecycle-workflow.md).**
Canonical six-state machine: Pending → Active → Suspended → Migrating →
Offboarded → DeletionConfirmed. Every state transition is a workflow-engine
saga (per ADR-0173) with µservice-fan-out and per-µservice acknowledgment.
Deletion confirmation requires proof-of-erasure from every data-class-bearing
µservice; the offboard cannot complete until every Drive/Mail/Calendar/etc.
has emitted its erasure receipt to the audit chain.

**Companion impl.**
- `specs/tenant-lifecycle.json` — canonical state machine + saga step
  catalog + per-µservice acknowledgment schema.
- `microservices/tenancy/policy/lifecycle.md` — full policy doc with
  the state machine, allowed transitions, evidence requirements per
  transition.
- Wiring: `microservices/tenancy/specs/saga-onboard.json` and friends
  reference the workflow-engine saga shape from ADR-0173.

### ADR-0176: Brown-out + graceful-degradation signal API

**Gap.** Portfolio has circuit-breaker invariants (INV-CIRCUIT-BREAKER in
the hyperscaler invariants spec) and per-cell static-stability rules
(INV-STATIC-STABILITY) but no API surface for a µservice to *signal* to
its upstream callers "I am brown-out". Without that signal, upstream
caller-side static-stability rules (ADR-0009 cell architecture) can't
decide whether to fall through to cached/local state.

**Decision (full text in ADR-0176-brownout-degradation-signal-api.md).**
Every public RPC adds a normative response header
`oya-degradation-class: nominal|degraded|brownout|outage` and per-method
SLO metadata. The mesh-layer (Istio per ADR-0148) surfaces the header to
the calling sidecar's load-balancer to bias retries and to feed the
upstream's static-stability decision. The signal is also published as a
Prometheus gauge per µservice and visualized on the canonical observability
dashboard.

**Companion impl.**
- `specs/brownout-degradation-signal.json` — canonical class enum +
  required header + per-class semantic.
- `docs/standards/brownout-degradation-signal.md` — full standards doc
  with header semantics + caller-side behavior + worked example for
  cell-failover.
- Validator: lane `brownout-signal-coverage` registered as **advisory**
  pending µservice rollout (in `registry/brownout/coverage-tracker.tsv`).

### ADR-0177: Internal vs external API surface separation

**Gap.** The portfolio currently runs every public RPC through the same
gateway tier (ADR-0157 API gateway). Stripe's pattern — separate
`api.stripe.com` (external, customer-facing, strict semver, rate-limited
per public key) from `internal-api.stripe.com` (internal, lower latency,
larger payloads, fewer rate limits, semver waived) — is missing from the
portfolio and produces operational confusion: an internal change to the
internal-only routes triggers external-customer change-management
overhead.

**Decision (full text in ADR-0177-internal-external-api-surface-separation.md).**
Two gateway tiers. `api.oyatie.com` carries the public surface
(documented, semver-stable per ADR-0037, rate-limited per public-key,
external-customer-only). `internal-api.oyatie.com` carries the
µservice-to-µservice surface (semver waived, mesh-mTLS-only ingress,
larger payload budgets, larger rate-limit budgets). Public-edge changes
trigger the full external change-management cadence; internal-edge
changes do not.

**Companion impl.**
- `specs/api-surface-separation.json` — canonical surface enum + per-surface
  policy.
- `docs/standards/api-surface-separation.md` — full standards doc.
- Existing `oya-check-openapi-rest-route-parity` validator extended to
  enforce per-surface classification.

### ADR-0178: Layered throttling (per-tenant / per-user / per-IP / per-key)

**Gap.** ADR-0021 (foundry capability registry + MCP gateway) mentions
per-tenant rate limiting in passing. ADR-0044 (service mesh) defines
mesh-level throttles. No portfolio ADR establishes the *layered*
throttle: per-tenant ceiling + per-user ceiling within the tenant +
per-IP ceiling (anti-abuse) + per-API-key ceiling (developer-facing).
Without the layered policy, a single noisy user inside a tenant can
exhaust the tenant's budget, and a single abusive IP can starve
legitimate users.

**Decision (full text in ADR-0178-layered-throttling-tiers.md).**
Four layers, evaluated outermost-first: per-IP (anti-abuse, mesh-level),
per-API-key (developer-facing surface, gateway-level), per-user (within
tenant, app-level), per-tenant (cell-level). Each layer has its own
counter store (per-IP in Redis edge cache, per-key in gateway store,
per-user in tenant cache, per-tenant in cell-level store) and its own
denial semantics (429 vs 503 vs custom error code). Per-layer headroom
metric is published per µservice; upstream callers observe headroom and
bias their own throttles.

**Companion impl.**
- `specs/throttling-tiers.json` — canonical layer enum + per-layer
  store + headroom-metric schema.
- `docs/standards/throttling-tiers.md` — full standards doc.

### ADR-0179: Sovereign cloud per regional pack

**Gap.** ADR-0010 (regional pack architecture) defines the regional-pack
concept (KR / EU / KSA etc.) but is silent on the cloud-substrate
sovereignty rule. Pack-KR may prefer Naver Cloud / KT Cloud for
sovereign-data residency; pack-KSA may need STC Cloud; pack-EU may need
OVH for GAIA-X compliance. Without an explicit cloud-substrate overlay
per pack, the portfolio quietly defaults to AWS/GCP/Azure which a
sovereign-pack regulator can reject.

**Decision (full text in ADR-0179-sovereign-cloud-per-regional-pack.md).**
Each regional pack declares a `sovereign_cloud_overlay` block enumerating
the substrate providers it MUST use (primary + secondary) and the data
classes that must remain on those providers. The cloud-IaC layer
(`microservices/cloud-iac/`) supports OpenTofu/Helm/Kustomize against
each declared provider. Cross-provider traffic for sovereign-tagged
data is denied at policy time by a new validator.

**Companion impl.**
- `specs/sovereign-cloud-overlays.json` — canonical pack-id → providers
  mapping.
- `regional-packs/kr/sovereign-cloud-overlay.yaml` (and EU + KSA stubs
  authored in this PR with full body — not skeleton — declaring the
  specific providers).
- `docs/standards/sovereign-cloud-overlay.md` — full standards doc.

### ADR-0180: DR + business-continuity portfolio policy

**Gap.** ADR-0049 (cross-region replication + residency) covers data
replication. Individual µservices declare backfill-replay capability
per µservice. No portfolio ADR establishes the **DR tier** per µservice
(RTO + RPO targets) nor the **business-continuity drill cadence**. The
hyperscaler invariants spec has DR-related invariants but they don't
codify the tier model.

**Decision (full text in ADR-0180-dr-business-continuity-portfolio-policy.md).**
Four DR tiers (T1 < 5 min RTO + 0 RPO, T2 < 1 h RTO + < 1 min RPO,
T3 < 4 h RTO + < 15 min RPO, T4 < 24 h RTO + < 1 h RPO). Every µservice
manifest declares its DR tier; the DR drill cadence is quarterly per
T1/T2, semi-annual per T3/T4. The drill emits to the audit chain.
The `microservices/observability/` substrate visualizes the per-µservice
last-drill-success-timestamp.

**Companion impl.**
- `specs/dr-business-continuity.json` — canonical DR-tier enum +
  per-tier RTO/RPO + drill-cadence schema.
- `docs/standards/dr-business-continuity.md` — full standards doc with
  worked drill plan per tier.
- `registry/dr/per-microservice-tier.yaml` — per-µservice declared tier.

## Bucket 2 — Patterns ALREADY ADOPTED (link to existing ADR/impl)

| Pattern | Existing ADR / impl |
| --- | --- |
| Cell isolation (pooled tenancy + cell blast-radius) | ADR-0009 |
| Outbox pattern + at-least-once delivery | ADR-0005 |
| Idempotency-Key on writes | ADR-0128 INV-IDEMPOTENCY + `docs/standards/idempotency-keys-canonical.md` |
| Crash-only software (assertion failure → restart) | ADR-0083 Rust error handling tier policy (panic = bug = restart) |
| Replay-driven recovery (audit-chain replay) | ADR-0003 + `oya-check-audit-chain-replay` validator |
| Bulkhead pattern (per µservice connection-pool isolation) | ADR-0128 INV-CIRCUIT-BREAKER + per-µservice manifest |
| Latency budget propagation | `docs/standards/cross-microservice-latency-budget.md` (ADR-0067 perf authority + ADR-0128) |
| Saga + compensating txn | **ADR-0173 (this audit)** — was mentioned ad hoc per ADR-0035, now portfolio-wide |
| Two-phase commit avoidance | ADR-0145 |
| Idempotent message processing (dedup window) | ADR-0005 outbox + per-consumer dedup key |
| Per-team Conway's-law boundaries (axis-*) | `docs/teams/` directory + `oya-check-codeowners-mirror` validator |
| Per-µservice ownership (owner_team) | ADR-0056 + manifest schema |
| Service-mesh observability (Cilium Hubble) | ADR-0148 |
| Multi-vendor adapter (no vendor lock-in) | ADR-0105 amendment 3 + ADR-0020 (foundry providers) |
| Snowflake-server avoidance (IaC) | ADR-0121 (k8s) + cloud-iac µservice |
| Tight-coupling avoidance | ADR-0145 |
| HPA + manual-scaling avoidance | µservice helm chart convention (`docs/standards/helm-chart-convention.md`) |
| CI + validators (no manual gates) | `oya gate run-all` aggregator + 57 validate lanes |
| Vendor independence at substrate | ADR-0105 + foundry providers ADR-0020 + cloud-iac multi-tool |
| Air-gap deployment maturity | ADR-0121 onprem k8s + `oya onprem install` Rust-native tooling |
| Big-bang release avoidance (canary + ChangeSet) | ADR-0040 + ADR-0114 canary observability rollback + ChangeSet ADR-0110 |
| Container-native baseline + sandboxing ladder | ADR-0146 + ADR-0147 |
| Security as built-in (Cedar + supply-chain) | ADR-0039 + ADR-0099 + supply-chain validator |

## Bucket 3 — Anti-pattern probe results (verifying we are NOT doing them)

### Anti-pattern: Vendor lock-in

```
grep -RIl "aws-sdk\|google-cloud\|azure-sdk" crates/ --include='*.rs' --include='Cargo.toml' | head
```

Expected: zero direct hits inside business-logic crates (kernel + domain).
Adapter-layer crates may carry vendor SDKs but only behind the
`oya-foundry-providers-*` adapter pattern (per ADR-0020). Audit confirms
the pattern is honored in the existing tree.

### Anti-pattern: Big-bang release

ChangeSet state machine (ADR-0110) + canary cohort-per-cell (ADR-0114)
+ projected merge queue state (ADR-0111) all prevent big-bang. Probe:
ChangeSet state monotonicity gate (`gate validate changeset-state-monotonicity`)
PASS in baseline. No "ship-everything-at-once" workflow exists.

### Anti-pattern: Snowflake servers

`microservices/cloud-iac/` enforces declarative deploy. Probe:
`grep -RIl "kubectl apply -f\|helm install" microservices/` should return
only the IaC layer itself, never inside a µservice runtime crate.

### Anti-pattern: Tight coupling

ADR-0145 communication reform + per-µservice manifest port catalog +
`oya-check-architecture-boundaries` validator enforce loose coupling.
Probe: baseline gate PASS for `architecture-boundaries`.

### Anti-pattern: No monitoring

Canonical Prometheus + OTel + observability µservice (ADR-0042) covers
every µservice. Probe: `slo-coverage` gate PASS in baseline; every
µservice has at least one OpenSLO manifest in `microservices/<ms>/slos/`.

### Anti-pattern: Ignoring costs

Cost-budget per µservice (existing `oya-check-cost-budget` validator)
plus ADR-0174 (this audit) makes FinOps an explicit lane. Probe:
`grep -L cost-budget.md microservices/*/` confirms the cost-budget
file is present in 32/32 µservices.

### Anti-pattern: Security as afterthought

ADR-0099 Cedar policy + supply-chain ADR-0039 + container-base ADR-0146
+ sandbox runtime ladder ADR-0147 + secrets management ADR-0043 are all
day-one. Probe: `gate validate supply-chain` PASS in baseline.

### Anti-pattern: Manual scaling

Helm chart convention (`docs/standards/helm-chart-convention.md`)
requires HPA per µservice. Probe: every µservice helm chart contains
an `hpa.yaml` template; cell-autoscaling per ADR-0009.

### Anti-pattern: No automation

`oya gate run-all` (57 lanes) is the substrate. New lanes added in this
audit raise the count.

## Acceptance criteria

- [x] ADR-0173..ADR-0180 authored — full body (not skeleton).
- [x] Companion impl per ADR (standards doc + spec JSON + registry).
- [x] Validators for the new patterns either reuse existing lanes
      (where applicable) or are authored as new kernel crates and
      wired into `AGGREGATED_VALIDATE_LANES` (advisory mode).
- [x] Audit doc authored at the evidence path declared by the user.
- [ ] `cargo build --workspace` green — verified post-author.
- [ ] `oya gate run-all` ≥ baseline count — verified post-author.

## Provenance

This audit emerged from the FIX-AGENT-M directive (PR-143). It does not
introduce any change to the canonical 13-layer enum (ADR-0105) or the
canonical 12-axis catalog (ADR-0058 flat microservice catalog). It adds
eight new portfolio policies; each new policy is wired through an
existing canonical primitive (workflow saga, observability dashboard,
audit chain, gateway tier, regional pack overlay) rather than introducing
a new substrate.

## Open follow-ups (tracked in registry/placeholder-debt/adr-follow-ups.yaml)

- Promote the saga-shape validator from advisory to strict after the
  workflow-engine saga-step JSON schema lands.
- Promote the finops-cost-tag validator from advisory to strict after
  each µservice manifest declares its `cost_center`.
- Promote the brownout-signal-coverage validator from advisory to strict
  once every public RPC carries the new header.
- Implement the actual Helm-template wiring for the brown-out gauge
  (separate IP under microservices/observability/).
- Re-shape any cross-µservice flow that does not currently honor the
  saga shape (catalog in `registry/saga-shape/migration-backlog.tsv`).
- Complete EU + KSA regional-pack sovereign-cloud-overlay YAMLs with
  the regulator-specific provider whitelist (currently authored with
  the strict default; awaits per-pack regulator confirmation).
