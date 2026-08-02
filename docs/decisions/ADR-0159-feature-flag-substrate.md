---
id: ADR-0159
status: Accepted
deciders: council-architecture, axis-tenancy, axis-governance, ops-sre-reliability, ops-product
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: [ADR-0632]
related: [ADR-0007, ADR-0009, ADR-0049, ADR-0110, ADR-0114, ADR-0128, ADR-0139, ADR-0145, ADR-0157, ADR-0158]
related_specs:
  - /specs/feature-flag-substrate-canonical.json
  - /specs/hyperscaler-architecture-invariants.json
---
# ADR-0159 — Dedicated Feature-Flag µservice (runtime gradual rollout) separate from ChangeSet acceptance (code-deploy gating)

## Status

Accepted (2026-05-18). Promotes a dedicated `feature-flags` µservice as the canonical *runtime* feature-flag substrate. Distinct from the ChangeSet `acceptance_status` (ADR-0110), which gates code-deploy. Distinct from progressive delivery (ADR-0160 / ADR-0114), which gates traffic shape during a deploy.

**Amended — 2026-07-09 (RELEASE-001 runtime-safety declaration):** the retained non-crate `feature-flags` release surface may carry a data-only RELEASE-001 runtime-safety declaration at `oya/feature-flags/release/runtime-safety-policy.json`, owned narrowly by `oya/feature-flags/release/OWNERS`. The declaration documents alignment between this ADR's feature-flag lifecycle cleanup discipline and ADR-0139 SLO-gated promotion and ADR-0176 brownout degradation signaling, without claiming live rollout execution, tenant traffic, measured SLOs, runtime observability-engine readiness, or Oya CD parity/cutover.

## ADR-0632 product-protocol reconciliation

The tenant, Workflow Studio, portal, and SDK-facing feature-flag contract is public HTTPS REST documented by OpenAPI 3.2.0, with signed/versioned webhooks, AsyncAPI/CloudEvents events, SSE, or WebSocket used where their delivery semantics apply. Public GraphQL, gRPC, gRPC-Web, and Connect are forbidden. Any gRPC flag-evaluation adapter is internal-only gRPC/proto3 over HTTP/2 behind the public contract.

## Context

Oyatie has three conflated concepts today:

1. **ChangeSet `acceptance_status`** (ADR-0110) — does this code change merge to `dev`? Does it merge to `staging`? Does it promote to `production`? Gates the *code-deploy* lifecycle.
2. **Canary / progressive-delivery weighting** (ADR-0114) — once deployed, how much traffic does the new version handle? Gates the *traffic shape* during a deploy.
3. **Runtime feature flag** — for code that IS deployed to production, is a given feature ENABLED for tenant T / persona-tier P / cohort C right now? Gates the *runtime behavior* per tenant.

ADR-0110 + ADR-0114 together cover (1) + (2). Concept (3) — runtime feature flagging — has no canonical substrate today, leading to ad-hoc per-µservice environment-variable flags, Cedar-policy edge cases, and dead-code paths that cannot be cleanly removed.

The hyperscaler precedent for (3) is unambiguous:

- **LaunchDarkly** is the canonical SaaS feature-flag tool used by Stripe, Atlassian, IBM, etc.
- **AWS AppConfig** is the AWS-managed runtime config + feature-flag tier.
- **Flagsmith** + **Unleash** + **OpenFeature** are the open-source equivalents; OpenFeature is the CNCF specification.
- **Statsig** / **Optimizely** add experimentation (A/B testing) on top of feature flags.

ADR-0159 makes runtime feature flagging a first-class µservice (`feature-flags`) implementing the OpenFeature spec server-side, with per-tenant + per-persona-tier + per-cohort targeting and an audit-chain seal on every flag evaluation that changes user-visible behavior.

## Decision

Oyatie adopts a dedicated `feature-flags` µservice as the canonical runtime feature-flag substrate. Properties:

### Three orthogonal gating tiers

1. **Code-deploy gate** = ChangeSet `acceptance_status` (ADR-0110). Code lives in `dev` / `staging` / `production`.
2. **Traffic-shape gate** = Progressive delivery via Flagger (ADR-0160) with SLO-gated promotion (ADR-0139). 1% → 5% → 25% → 100% of traffic to the new version.
3. **Runtime gate** = Feature flag (this ADR). Per-tenant + per-persona-tier + per-cohort: is the feature ON?

All three required for any user-visible change. ChangeSet promotes the code. Flagger ramps the traffic. Feature flag enables the behavior. Rollback at any tier independent of the others.

### OpenFeature compliance

The `feature-flags` µservice implements the OpenFeature spec server-side:

- Public HTTPS REST surface for flag evaluation (`evaluateBoolean`, `evaluateString`, `evaluateNumber`, `evaluateObject`), with an internal-only gRPC/proto3 over HTTP/2 adapter for sibling services.
- Per-µservice Rust SDK (`oya-feature-flag-client`) wrapping the OpenFeature Rust SDK.
- Per-µservice TypeScript / Python SDKs for the workflow-studio / portal tiers.
- Per-tenant flag evaluation context: `tenant_id`, `persona_tier`, `pack_id`, `cohort_ids[]`, `user_id` (hashed).

### Targeting rules

Each flag has zero-or-more targeting rules. A targeting rule is `(predicate, variant, percentage)`:

- **Predicate** — Cedar fragment (per ADR-0007) that the flag µservice evaluates against the evaluation context.
- **Variant** — boolean / string / number / JSON object.
- **Percentage** — deterministic hash of `(tenant_id, flag_key)` mod 100; tenants below the threshold get the variant.

Rules evaluate in declaration order; first match wins. Default variant if no rule matches.

### Per-cell deployment

`feature-flags` µservice is `active_active` per cell (ADR-0158 disposition). Flag definitions replicate globally (eventually consistent ~5 sec). Flag evaluation is cell-local (< 1 ms p99). The µservice's `multi-region.md` declares this.

### Audit-chain on flag changes

Every flag definition change (create / edit / delete / archive) emits an audit-chain seal (per ADR-0003). Every flag *evaluation* does NOT emit (volume too high); evaluations are tracked via the µservice's own observability tier (counter + cardinality metrics).

A flag tagged `audit_required: true` emits an audit-chain seal per *evaluation* (used for kill-switches and compliance-gated features). The PRD documents when this tag is appropriate.

### Lifecycle: feature flag → cleanup

Every feature flag declared MUST also declare:

- `created_at` — timestamp.
- `owner` — axis owner.
- `intent` — `release_toggle` (kill-switch for a feature being rolled out) / `experiment` (A/B test) / `permission_toggle` (per-tenant feature gate that is durable) / `kill_switch` (emergency disable).
- `sunset_at` — timestamp; for `release_toggle` and `experiment`, this is mandatory; for `permission_toggle` it is null (durable).

A CI lane `cloud-ci/Rust gate packet feature-flag-lifecycle` refuses merge if (a) a flag has been `release_toggle` for > 90 days past its `sunset_at`, (b) a flag has been `experiment` for > 180 days, (c) any code references a flag whose definition no longer exists in `feature-flags` µservice's catalog.

### Cedar integration

Flag predicates are Cedar fragments. Same Cedar evaluator that the governance µservice uses (per ADR-0007). One Cedar evaluator path; multiple consumers.

## Alternatives considered

### Alternative A — No dedicated feature-flag µservice; use ChangeSet acceptance_status for runtime gating

- **Pros:** zero new µservice.
- **Cons:** ChangeSet gates code-deploy, not runtime behavior. Conflating the two means every per-tenant feature toggle requires a code-deploy cycle. Cannot do gradual rollout at runtime. Cannot do kill-switches without redeploy.
- **Rejected because:** the three concepts (code-deploy / traffic-shape / runtime) are orthogonal and conflating them is a known anti-pattern (cite Martin Fowler "FeatureToggle" 2017 — explicit separation of release toggles, ops toggles, experimentation, permissioning).

### Alternative B — Per-µservice ENV-var flags

- **Pros:** zero new µservice; trivial implementation.
- **Cons:** ENV-var changes require pod restart; cannot do per-tenant flagging without massive overlay multiplication; cannot do percentage rollouts; cannot do experiment cohorts; cannot audit flag changes.
- **Rejected because:** ENV-var is the historical anti-pattern this ADR replaces.

### Alternative C — Commercial SaaS (LaunchDarkly / Statsig)

- **Pros:** zero infra-build cost; mature product.
- **Cons:** sovereign-tenant data leaves the cell (ADR-0049 residency invariant); per-pack on-prem / air-gap variants (ADR-0164) cannot use external SaaS; vendor pricing scales unfavorably at fleet scale; not open-source.
- **Rejected because:** ADR-0049 + ADR-0164 require sovereign-cell containment.

### Alternative D — Dedicated `feature-flags` µservice implementing OpenFeature (this ADR)

- **Pros:** open spec (OpenFeature is CNCF-incubating); same Cedar evaluator as governance; per-cell deployment satisfies sovereign-pin; lifecycle gates clean up dead flags; audit-chain integration uniform.
- **Cons:** new µservice to own + operate; another tier of latency in the request path (~0.5-1 ms added — flag eval typically cached client-side).
- **Accepted.**

### Alternative E — Use Cedar policy directly as feature-flag substrate

- **Pros:** zero new µservice; Cedar already exists.
- **Cons:** Cedar is authorization; mixing authorization (deny / allow) with feature-flagging (enabled / disabled) in one policy surface confuses the audit story; Cedar policy changes require ChangeSet promotion (slow), whereas feature flags need runtime updates (seconds).
- **Rejected because:** authorization and feature-flagging have different SLAs and audit requirements; mixing them is the same anti-pattern as Alternative A.

## Consequences

### Positive

1. **Three gating tiers explicit and orthogonal.** Code-deploy / traffic-shape / runtime: any one can rollback without the others.
2. **OpenFeature compliance.** CNCF spec; any future OSS or commercial tooling integrates.
3. **Per-tenant + per-persona-tier + per-cohort targeting.** First-class Cedar predicate; same evaluator as authorization.
4. **Sovereign-cell containment.** Per-cell deployment satisfies ADR-0049 + ADR-0164; no external SaaS dependency.
5. **Flag lifecycle gates close the dead-code problem.** CI refuses to merge code that references retired flags; CI refuses to leave `release_toggle` flags past their `sunset_at`.
6. **Kill-switch surface for compliance-sensitive features.** A flag tagged `audit_required: true` lets the legal team disable a feature globally within seconds + audit every evaluation.

### Negative

1. **One more µservice to own.** `microservices/feature-flags/` added to the inventory (35 µservices total after ADR-0157 + this one).
2. **Added latency.** ~0.5-1 ms per flag eval (mitigated by client-side cache with 30-sec TTL).
3. **Per-µservice integration cost.** Every µservice that uses feature flags must integrate the `oya-feature-flag-client` SDK.
4. **Flag-explosion risk.** Without lifecycle gates, feature flags accumulate as dead code. Lifecycle gates exist precisely to bound this.

### Operational

1. New µservice scaffolded at `microservices/feature-flags/` per ADR-0131 flat layout. PRD skeleton ships with this ADR (see Companion).
2. New CI lane `cloud-ci/Rust gate packet feature-flag-lifecycle` enforces flag-cleanup discipline.
3. Per-pack Helm overlay `iac/kustomize/components/feature-flags-overlay-{kr,eu,us,jp,ksa}/`.
4. Rust SDK `crates/oya-feature-flag-client/` wraps OpenFeature Rust SDK.
5. Companion spec `specs/feature-flag-substrate-canonical.json` declares the OpenFeature-spec mapping + Cedar-predicate shape + lifecycle policy.

## References

- OpenFeature CNCF specification — https://openfeature.dev/
- LaunchDarkly architecture and feature-flag lifecycle — https://launchdarkly.com/blog/
- AWS AppConfig feature-flag service — https://docs.aws.amazon.com/appconfig/
- Statsig — feature flagging + experimentation — https://docs.statsig.com/
- Flagsmith / Unleash OSS feature-flag servers — https://www.flagsmith.com/ / https://www.getunleash.io/
- Martin Fowler — "FeatureToggle" (2017) — release toggle vs. experiment toggle vs. ops toggle vs. permission toggle taxonomy.
- Cedar v4.2 LTS — predicate evaluator.
- ADR-0003 — audit-chain emission contract.
- ADR-0007 — Cedar authorization policy.
- ADR-0009 — cell architecture.
- ADR-0049 — cross-region residency.
- ADR-0110 — ChangeSet state machine (code-deploy gate).
- ADR-0114 — canary observability + rollback (traffic-shape gate).
- ADR-0128 — hyperscaler architecture invariants.
- ADR-0139 — agentic SLO-gated promotion.
- ADR-0145 — inter-µservice communication reform (SDK transport).
- ADR-0157 — api-gateway tier.
- ADR-0158 — multi-region disposition.
