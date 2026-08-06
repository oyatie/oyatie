---
id: ADR-0165
status: Superseded
deciders: council-architecture, axis-cloud-k8s, axis-observability, ops-sre-reliability, axis-governance
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-701]
related: [ADR-0114, ADR-0121, ADR-0128, ADR-0139, ADR-0145, ADR-0148, ADR-0157, ADR-0158, ADR-0160]
related_specs:
  - /specs/chaos-engineering-substrate-canonical.json
  - /specs/agentic-slo-gated-promotion.json
---

# ADR-0165 — Chaos Engineering Substrate (Chaos Mesh 2.x; SLO-driven nightly drills against staging)

## Status

Accepted (2026-05-18). Adopts Chaos Mesh 2.x as the canonical chaos engineering substrate. Drills run nightly against the staging environment of every µservice that declares production SLOs; failure to meet SLO during a drill is a release blocker.

## Context

ADR-0114 named the canary observability + rollback contract. ADR-0139 named the agentic SLO-gated promotion contract. ADR-0128 named the hyperscaler architecture invariants (INV-CELL-ISOLATION, INV-SHUFFLE-SHARDING, INV-AT-LEAST-3-REPLICAS, etc.). ADR-0145 named the inter-µservice communication reform with retry + timeout + circuit-breaker invariants.

Each of these declares *what the system should do under stress*. None pins *how the stress is created*. The hyperscaler precedent — Netflix Simian Army (2011), AWS Fault Injection Simulator (2021), Google's DiRT (Disaster Recovery Testing) program (2010s) — converges on one principle: **deliberately and continuously inject failure**, observe the SLO response, fix the system if the SLO breaks. Chaos engineering is not optional at hyperscaler scale; it is the only credible way to maintain the invariants over time.

Without an explicit chaos substrate:

- INV-AT-LEAST-3-REPLICAS is declared but never tested under pod-kill.
- INV-CELL-ISOLATION is declared but never tested under cross-cell partition.
- Retry + circuit-breaker invariants (ADR-0145) are declared but never tested under transient failure injection.
- The SLO-gated promotion (ADR-0139) gates *fresh code*; it doesn't test *resilience over time*.

ADR-0165 makes chaos engineering a first-class lane parallel to (not replacement for) the SLO-gated promotion lane.

## Decision

Oyatie adopts **Chaos Mesh 2.x** (CNCF incubating) as the canonical chaos engineering substrate. Every µservice that declares production SLOs ships a per-µservice chaos catalog at `chaos/scenarios/*.yaml`; a nightly job runs each scenario against the µservice's staging environment; SLO breach during a scenario is a release blocker.

### Operational shape

1. **Chaos Mesh control plane per cell.** Cluster-scoped operator in the cloud-k8s pack.
2. **Per-µservice chaos catalog.** Each µservice's `microservices/<ms>/chaos/scenarios/*.yaml` declares one or more `Workflow` CRDs from Chaos Mesh. Each scenario targets the µservice's staging deployment.
3. **Nightly drill cadence.** GitHub Actions workflow `.github/workflows/chaos-nightly.yml` runs each µservice's catalog at 02:00 cell-local-time against staging. Per-µservice schedule overridable.
4. **SLO-driven evaluation window.** During the drill, Prometheus queries the µservice's production SLOs:
   - `request-success-rate >= 95%` during drill (relaxed from 99.5% during canary; absorbing failure injection is the point).
   - `request-duration-p99 <= 2× normal budget` during drill.
   - `audit-chain-emission-success-rate >= 99.9%` (audit invariant; not relaxable).
   - Per-µservice custom queries in `slos/chaos-gates.openslo.yaml`.
5. **Failure modes per scenario.** Chaos Mesh primitives:
   - **PodChaos** — `pod-kill`, `pod-failure`, `container-kill`. Tests INV-AT-LEAST-3-REPLICAS.
   - **NetworkChaos** — `delay`, `loss`, `duplicate`, `corrupt`, `partition`. Tests retry + circuit-breaker invariants.
   - **IOChaos** — `latency`, `fault`, `attrOverride`, `mistake`. Tests disk slowness / fs-fail tolerance.
   - **TimeChaos** — clock skew. Tests time-dependent code (audit-chain timestamp tolerance, JWT expiry tolerance).
   - **StressChaos** — CPU + memory pressure. Tests scaling + HPA.
   - **DNSChaos** — DNS resolution failure. Tests cross-µservice resolution resilience.
   - **HTTPChaos** — inject HTTP 5xx responses. Tests upstream-dependency-failure handling.
6. **Cross-µservice drill.** Some scenarios are fleet-wide (`partition cell-A from cell-B`); these live at `microservices/cloud-k8s/chaos/scenarios/` (cross-cell partition is a cloud-k8s pack concern).
7. **Manual GameDay drills.** Quarterly, the SRE team runs scheduled GameDay drills (Google DiRT pattern): a planned wide-impact scenario (e.g. simulate full cell loss) with a runbook validation.
8. **Production chaos: explicitly opted in.** Some µservices (audit-chain, foundry) MAY opt in to production chaos drills with severe SLO gates + per-tenant whitelist. Default = staging-only.

### Catalog requirements per µservice

Every µservice with production SLOs MUST declare at minimum:

- **`pod-kill`** scenario (test pod resilience).
- **`network-delay-100ms`** scenario (test latency tolerance).
- **`dependency-failure`** scenario per declared downstream dependency (test circuit-breaker).
- **`disk-slow-1000ms`** scenario (test IO failure tolerance).
- **`time-skew-30s`** scenario (test clock tolerance) — required for time-sensitive µservices (audit-chain, JWT-issuer in tenancy).

### CI gate

`cloud-ci/Rust gate packet chaos-engineering-catalog`:

- Refuses merge if a µservice with production SLOs lacks the minimum scenario set.
- Refuses merge if a scenario references a target that doesn't exist (no orphan scenarios).
- Refuses merge if a scenario's SLO gate references an SLO query that doesn't exist in the µservice's `slos/`.

### Release blocker semantics

A scenario failing its SLO gate against staging:

- Blocks promotion of any in-flight ChangeSet for that µservice until either (a) the µservice is fixed (SLO recovers), (b) the scenario is amended with explicit ADR justification (cannot just delete the scenario).
- Emits an audit-chain seal `ChaosScenarioFailed`.
- Pages the µservice's on-call.

## Alternatives considered

### Alternative A — AWS Fault Injection Simulator (FIS) only

- **Pros:** AWS-managed; integrated with AWS console; mature.
- **Cons:** AWS-specific (violates ADR-0121 portability invariant); doesn't run against GCP / Azure / on-prem cells; per-pack support is partial.
- **Rejected because:** portability invariant.

### Alternative B — Gremlin (commercial chaos SaaS)

- **Pros:** mature commercial product; broad failure-mode library; UI is excellent.
- **Cons:** licensing cost scales unfavorably at fleet scale; agent runs in-cluster with credentials — sovereign packs (ADR-0164) cannot accept external SaaS agents; ADR-0049 residency concerns.
- **Rejected because:** sovereign + cost.

### Alternative C — Litmus Chaos (CNCF graduated)

- **Pros:** CNCF graduated (higher than Chaos Mesh's incubating); good Argo Workflows integration.
- **Cons:** Litmus's chaos primitive coverage is similar to Chaos Mesh; the Chaos Mesh CRD shape (`Workflow`) composes better with Flagger (ADR-0160); ecosystem at parity but Chaos Mesh has slightly broader primitive coverage in our evaluation.
- **Rejected because:** Chaos Mesh's Workflow CRD + broader IOChaos + TimeChaos coverage wins on technical fit. (Litmus remains a credible swap-in if Chaos Mesh project health regresses.)

### Alternative D — Custom Rust chaos harness (NIH)

- **Pros:** maximum control.
- **Cons:** rebuilding Chaos Mesh's primitive surface is years of work; not a defensible engineering investment.
- **Rejected because:** NIH.

### Alternative E — Chaos Mesh 2.x (this ADR)

- **Pros:** CNCF incubating; rich primitive set; Workflow CRD composes; Helm chart available; open-source; per-cell deployable; works in sovereign packs (no external SaaS dependency).
- **Cons:** incubating not graduated; operator footprint per cell.
- **Accepted.**

## Consequences

### Positive

1. **Invariants tested continuously.** Every invariant declared in ADR-0128 + ADR-0145 has a nightly drill that proves it holds.
2. **Resilience regression caught before production.** A µservice that loses its retry logic, fails to scale, or breaks its circuit-breaker is caught by the nightly drill BEFORE the next production deploy.
3. **GameDay quarterly cadence.** Wide-impact scenarios rehearsed; runbooks validated.
4. **CNCF + open-source.** No vendor lock-in; sovereign-pack compatible.
5. **Audit-chain trail.** Every drill emits an audit-chain seal; SOC 2 CC7.x (incident response + simulation) evidence rolls up.
6. **Aligns with SRE workbook practice.** Google SRE Workbook Chapter 17 (Testing for Reliability) mandates chaos testing; this ADR operationalizes.

### Negative

1. **Chaos Mesh operator per cell.** Each cell installs Chaos Mesh; ops adds to on-call.
2. **Nightly drill resource cost.** Each drill consumes staging-environment capacity; per-µservice staging must be sized accordingly.
3. **Drill flakiness risk.** Poorly-authored scenarios can produce false-positive failures; per-µservice scenario tuning required during onboarding.
4. **Time-skew chaos affects shared infrastructure.** TimeChaos on the audit-chain µservice in shared-cell mode affects other tenants in the same cell. Production chaos requires explicit opt-in.

### Operational

1. Chaos Mesh 2.x Helm chart shipped at `microservices/cloud-iac/iac/helm/chaos-mesh/`.
2. Per-µservice chaos catalog at `microservices/<ms>/chaos/scenarios/`.
3. New CI lane `cloud-ci/Rust gate packet chaos-engineering-catalog` enforces minimum scenarios + SLO-gate wiring.
4. New GitHub workflow `.github/workflows/chaos-nightly.yml` runs the fleet drill nightly.
5. New IP at `microservices/governance/IP-NEW-chaos-engineering-substrate.md` (Companion) wires `oya-check-chaos-engineering-catalog` into the gate.
6. Per-cell quarterly GameDay runbook in `microservices/cloud-k8s/runbooks/gameday-{quarter}.md`.
7. Production chaos opt-in per µservice in `microservices/<ms>/manifest.json#chaos_production_optin`.

## References

- Netflix Simian Army (2011) — origin of chaos engineering — Netflix tech blog.
- Principles of Chaos Engineering — https://principlesofchaos.org/
- Google SRE Workbook Chapter 17 (Testing for Reliability).
- Google DiRT (Disaster Recovery Testing) program — Google SRE Book.
- AWS Fault Injection Simulator — https://aws.amazon.com/fis/
- Chaos Mesh (CNCF incubating) — https://chaos-mesh.org/
- Litmus Chaos (CNCF graduated) — https://litmuschaos.io/
- Gremlin — https://www.gremlin.com/
- ADR-0114 — canary observability + rollback (SLO infra reused).
- ADR-0121 — onprem K8s stack (portability invariant; Chaos Mesh is K8s-native).
- ADR-0128 — hyperscaler architecture invariants (invariants tested).
- ADR-0139 — agentic SLO-gated promotion (parallel lane).
- ADR-0145 — inter-µservice communication reform (retry + circuit-breaker tested).
- ADR-0148 — Istio service mesh (NetworkChaos targets mesh).
- ADR-0157 — api-gateway tier.
- ADR-0158 — multi-region disposition (cross-cell partition scenarios).
- ADR-0160 — progressive delivery via Flagger (Chaos Mesh Workflow CRD composes).
