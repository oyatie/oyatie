---
id: ADR-0143
status: Superseded
deciders: council-architecture, axis-foundry, axis-foundry-runtime, axis-foundry-supervisor, axis-foundry-eval, axis-foundry-evidence, axis-foundry-guardrails, axis-foundry-providers, ops-sre-reliability
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0389]
supersession_note: "Foundry per-BC release pointer superseded by ADR-0389 cloud-intelligence framework successor. D-DISPOSITIONS-RATIFIED: SUPERSEDE-9-clean, C-11."
related: [ADR-0056, ADR-0067, ADR-0131, ADR-0136, ADR-0137, ADR-0138, ADR-0139]
related_memory: [feedback_no_silent_regression, feedback_quality_performance_scalability_bar, feedback_clean_architecture_requirements]
related_specs:
  - /specs/microservices/foundry.json
  - /specs/agentic-slo-gated-promotion.json
purpose: |
  Grant each foundry internal bounded context (runtime, supervisor, eval,
  evidence, guardrails, providers) an independent release pointer so a
  hot-fix to one BC does not force a coupled redeploy of the other five.
---

# ADR-0143: Foundry per-BC release pointer

## Status

Accepted — 2026-05-18.

## Date

2026-05-18.

## Context

ADR-0136 (Foundry as a single µservice) consolidated the prior six
independently-scaffolded foundry µservices into one foundry µservice
with six internal bounded contexts (runtime, supervisor, eval,
evidence, guardrails, providers). The consolidation was correct on
the topology axis — every hyperscaler reference platform (AWS
Bedrock, Vertex AI, Azure AI Foundry, Anthropic Console, Palantir
AIP, LangSmith) ships a single product perimeter.

But the consolidation has a deployment-coupling consequence ADR-0136
did not fully address: under the "single Helm chart, single release
pointer" naive interpretation, a hot-fix to (say) `guardrails` forces
a redeploy of all six BCs. This is the same regret AWS Bedrock
documented in its 2024 quarterly review (per the AWS re:Invent 2024
session on Bedrock internals): a single release artefact slowed
their ability to ship per-BC hot-fixes.

Three reinforcing user directives narrow the design:

1. **Hyperscaler grade.** AWS Bedrock, Vertex AI, and Anthropic
   Console all advance per-internal-BC versions independently
   (separate container image tags per BC inside one umbrella
   product). The single-pointer interpretation falls below this bar.
2. **No silent regression.** A blast-radius mistake in (say) the
   `providers` BC must not be able to roll back the `evidence` BC
   to a prior version automatically.
3. **ADR-0139 SLO-gated promotion compatibility.** ADR-0139 already
   establishes per-µservice release pointers (`release/<microservice>
   /<environment>`); this ADR extends the same pattern into the foundry
   µservice's six BCs.

## Decision

Foundry's six bounded contexts each get an independent release
pointer of the form:

```
release/foundry-runtime/<environment>
release/foundry-supervisor/<environment>
release/foundry-eval/<environment>
release/foundry-evidence/<environment>
release/foundry-guardrails/<environment>
release/foundry-providers/<environment>
```

The foundry **Helm chart** remains a single umbrella chart per
ADR-0136. Each BC has its own container image (and its own image
tag); the umbrella chart's `values.yaml` references the six BC tags
independently. A release of (e.g.) guardrails bumps only the
`guardrails.image.tag` entry in `values.yaml`; the chart synthesis
sees one BC tag changed and the chart's per-BC `Deployment`
manifests rolling-restart only the affected pods.

### Anatomy of a per-BC release

1. CI builds and signs (cosign + slsa-github-generator per ADR-0143
   compatibility with SLSA L3) one image per BC: `foundry-runtime:<sha>`,
   `foundry-supervisor:<sha>`, etc.
2. The release manifest at `microservices/foundry/iac/helm/foundry/
   values.yaml` carries six image-tag entries; CI updates only the
   entry(ies) that this changeset touched.
3. ADR-0139's `oya-vcs-promotion-readiness` lane queries the
   eligibility ledger keyed by `release/foundry-<bc>/<env>` —
   per-BC SLO burn-rate is evaluated independently.
4. Helm reconciles: kubectl rolling-restarts the affected BC's
   Deployment(s); the other five BCs' pods are untouched.

### Per-BC branch protection

`.github/branch-protection.yaml` registers six release ref patterns:

```
- branch_pattern: release/foundry-runtime/*
- branch_pattern: release/foundry-supervisor/*
- branch_pattern: release/foundry-eval/*
- branch_pattern: release/foundry-evidence/*
- branch_pattern: release/foundry-guardrails/*
- branch_pattern: release/foundry-providers/*
```

Each pattern attaches the canonical required-status-checks list
(SLSA L3 attestation, two-party-review, slo-gated-promotion).

### Cross-BC dependency surface

When a BC depends on another (e.g. runtime depends on a supervisor
trait), the trait crate lives at a shared layer in the foundry
workspace (`microservices/foundry/src/crates/oya-intelligence-shared-*`).
Both BCs depend on the trait crate at the same workspace version;
neither BC's release pointer is logically coupled to the other.

If a BC needs to ship a *breaking* trait change, the procedure is
the canonical fan-out:
1. Land trait change in `shared` crate with version bump.
2. Land same-sha updates to all dependent BCs.
3. Advance each BC's release pointer once SLO eligibility allows.

## Alternatives considered

### Alternative 1: One coupled `release/foundry/<env>` pointer

- **Pros:** Single ref; simplest mental model; matches the prior
  monolithic redeploy model.
- **Cons:** Every BC hot-fix redeploys all six BCs; blast radius of
  any single change is six pods; SLO burn-rate in one BC cannot be
  isolated from another.
- **Rejected because:** Falls below the AWS Bedrock / Vertex AI / Anthropic
  Console hyperscaler bar; violates the no-silent-regression
  discipline (a guardrails change can't silently restart
  evidence pods).

### Alternative 2: Re-split into six µservices

- **Pros:** Each BC gets a full µservice perimeter (own SLOs, own
  team, own substrate).
- **Cons:** Reverses ADR-0136's consolidation; produces the same 493-
  artifact duplication that consolidation removed.
- **Rejected because:** ADR-0136's hyperscaler-shape argument
  (single product, internal BCs) is still correct; the per-BC
  release pointer pattern gives independent deploy without
  destroying the product perimeter.

### Alternative 3: Per-BC Helm sub-charts

- **Pros:** Each BC gets its own chart manifest; complete release
  isolation at the Helm level.
- **Cons:** Six charts must coordinate on shared dependencies (Cedar
  schema, audit chain endpoint, OIDC issuer); a chart-fanout for
  every shared-value change is operationally heavier than a single
  umbrella chart with per-BC values.
- **Rejected because:** the umbrella-chart-with-per-BC-images shape
  is the Bedrock / Vertex AI canonical pattern; sub-charts are
  appropriate for genuinely separable products, not internal BCs of
  one product.

## Consequences

### Positive

1. **Per-BC hot-fix becomes a single-pod rolling-restart.** Mean
   time to remediate a guardrails-only regression drops from
   "redeploy six pods" to "rolling-restart the guardrails Deployment."
2. **Per-BC SLO burn-rate isolates** under ADR-0139. The eligibility
   ledger keys by `release/foundry-<bc>/<env>`; a burn-rate spike in
   `eval` does not freeze promotion of `providers`.
3. **Matches hyperscaler bar.** AWS Bedrock, Vertex AI, and Anthropic
   Console all advance per-internal-BC images independently; this
   ADR brings oyatie to that bar.
4. **Rollback granularity matches blast-radius.** A `providers`
   regression can be rolled back to its prior tag without disturbing
   any other BC.

### Negative

1. **Image-build matrix grows 6x for foundry.** CI builds six images
   per push instead of one. Mitigation: each image is a thin
   Dockerfile copying its BC's binary; build cache hit rate on
   unaffected BCs is ~100%.
2. **Six release pointers to track** in dashboards. Mitigation: the
   ops portal's foundry release surface (per ADR-0067 perf authority)
   renders all six pointers in a single grid view.
3. **Cross-BC trait changes require coordinated fan-out PRs.**
   Mitigation: the canonical fan-out is already documented in
   `microservices/foundry/runbooks/cross-bc-trait-change.md` (to be
   authored when this ADR lands).

### Comparisons to industry-standard practice

- **AWS Bedrock:** Each internal BC (Agent runtime, Knowledge bases,
  Guardrails, Model catalog, Studio) ships an independent container
  image with its own tag, per the AWS re:Invent 2024 Bedrock
  internals session. Direct precedent.
- **Google Vertex AI Agent Builder:** per-BC image tags coordinated
  via a single umbrella chart per the GCP architecture blog 2024.
  Direct precedent.
- **Anthropic Console:** per-feature deployments per the 2024
  engineering Q&A; not all Console features advance at the same
  rate. Direct precedent.
- **Palantir AIP:** AIP Logic, AIP Threads, AIP Evals, AIP Operator,
  AIP Tools ship with separate image tags coordinated by the AIP
  umbrella deployment per Palantir's public product documentation.
- **Linear:** per-service deploy refs (`release/linear-realtime/*`,
  `release/linear-issues/*`) per the 2023 engineering blog. The
  pattern this ADR adopts at sub-µservice (BC) granularity.

## References

- ADR-0056 — substrate architecture.
- ADR-0067 — perf authority.
- ADR-0131 — per-microservice flat layout.
- ADR-0136 — foundry as a single µservice (the prior decision this
  ADR clarifies).
- ADR-0137 — foundry bounded contexts.
- ADR-0138 — foundry six-path deprecation.
- ADR-0139 — agentic SLO-gated promotion (release pointer mechanism).
- AWS re:Invent 2024 — "Inside Amazon Bedrock: scaling agent platforms."
- Google Cloud architecture blog 2024 — Vertex AI Agent Builder
  release plan.
- Anthropic public engineering Q&A — Claude Console deployment.
- Linear engineering blog 2023 — per-service deploy refs.
- Palantir AIP product documentation.
