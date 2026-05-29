---
id: ADR-0170
status: Superseded
deciders: council-architecture, axis-developer-experience, ops-sre-reliability, council-api-sdk
date: 2026-05-18
owner: axis-developer-experience
supersedes: []
superseded_by: [ADR-0394]
related: [ADR-0011, ADR-0019, ADR-0025, ADR-0042, ADR-0131, ADR-0139, ADR-0167, ADR-0394]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/per-microservice-flat-layout.json
---

# ADR-0170 — Backstage-style internal developer portal

## Status

**SUPERSEDED by ADR-0394 (2026-05-29).** The internal developer portal is now a bespoke-Rust IDP central hub (Leptos portal over a bespoke-Rust ops-BFF); Backstage is retained as a feature reference only, and the Backstage Helm charts (observability + developer-sdk) plus ADR-SDK-0007 are quarantined/superseded. The discoverability *problem* and the catalog/SLO/ADR/runbook *surfaces* identified below stand; only the Backstage substrate is reversed. ADR-0203/0209/0213 references are retargeted onto ADR-0394. The body below is retained as the historical record.

~~Accepted (2026-05-18). Authorizes Backstage (Spotify) as the internal developer-portal substrate aggregating per-µservice catalog records, TechDocs, SLO dashboards, runbooks, team membership, and the ADR index. Tier C "nice-to-have" hyperscaler pattern per `/specs/hyperscaler-architecture-invariants.json` audit Row C4.~~

## Context

Oyatie now ships ~60 µservices under `microservices/<ms>/` (per ADR-0131 flat layout) and ~200+ Rust crates under `crates/`. Each µservice owns its own PRD, IPs, decisions/, runbooks/, dashboards/, scorecards/, SLOs, and tests. Discoverability across this surface is critical for:

- **Onboarding engineers** — "what does the `meet` µservice do, who owns it, what's its SLO, how do I run its tests, what are its open ADRs?"
- **Cross-µservice composition authors** — "which µservices already implement OAuth-2.1 tenant authentication so I can call the right one?"
- **Incident-response engineers** — "the `tasks` µservice paged; what's its runbook, who's on-call, what's its latest deploy?"
- **Architecture review** — "which µservices reference ADR-0139? Which haven't migrated to the new layout?"
- **Foundry agents per ADR-0025** — "list all capabilities exposed by µservices in the workflow product family."

The hyperscaler-reference is well-established: every major engineering organization runs an internal developer portal. Canonical references:

- **Spotify Backstage** — open-source; software catalog + TechDocs + scorecards + plugins; the industry-standard answer.
- **Netflix BAS** — internal-only ancestor; influenced Backstage.
- **Uber Service Hub** — internal-only; per-service ownership + SLO rollup.
- **Airbnb Service Hub** — internal-only; service-catalog + dependency graph.
- **AWS Developer Portal** — tenant-facing analog of an internal portal.

Today contributors discover µservices by:

1. `ls microservices/` (no metadata, no ownership, no SLO).
2. Reading `docs/AGENTS.md` and `docs/MASTERPLAN.md` (high-level, not per-µservice).
3. Grep'ing the ADR pile (high signal but slow).
4. Asking on Slack / PR comments.

This is the worst-case discoverability pattern. We have all the metadata — manifests, ADRs, runbooks, SLOs — but no aggregator. The cost compounds: every new µservice adds ~5 minutes to the onboarding cost for every future engineer; at fleet scale this is hundreds of engineer-hours/year of pure search-toil.

## Decision

Oyatie deploys Backstage as the canonical internal developer-portal substrate at `portal.oya.internal`. Backstage is configured to aggregate:

1. **Software catalog** — every `oya-*` crate, every `microservices/<ms>/` µservice, every tenant-facing product. Source of truth: per-µservice `manifest.json` + workspace `Cargo.toml`. Discovery via a Backstage catalog processor reading these files at PR-merge time.
2. **TechDocs** — Backstage's MkDocs-based per-component documentation surface. Source of truth: per-µservice `docs/` directories under `microservices/<ms>/`. Rendered on portal as a unified documentation site.
3. **SLO dashboards** — Backstage's Grafana plugin embeds the SLO panels per µservice. Source of truth: ADR-0139 Layer-A Grafana stack; per-µservice dashboards under `microservices/<ms>/dashboards/`.
4. **Runbooks** — Backstage's TechDocs surface aggregates per-µservice `runbooks/` directories.
5. **Team membership** — Backstage's Org catalog. Source of truth: `.github/CODEOWNERS` + `registry/teams.json` (synced by a Backstage processor).
6. **ADR index** — Backstage plugin surface (community ADR plugin) reading `docs/decisions/*.md` and rendering as a searchable index.
7. **Scorecards** — Backstage's Scaffolder + Tech Insights surfaces; backed by per-µservice `scorecards/` directories (already part of the flat layout per ADR-0131).
8. **API catalog** — per ADR-0011, every µservice publishes its OpenAPI + gRPC schemas; Backstage renders these in an explorable surface.

### What Backstage gives us out of the box

- Per-µservice page with name, owner, description, lifecycle, dependencies, dependents.
- Searchable component graph (text + tag + owner facets).
- Per-µservice "View Dashboard", "View Logs", "View Runbook", "View ADRs" deeplinks.
- Identity-aware: each engineer sees their on-call surface; can subscribe to per-µservice incident updates.
- Plugin ecosystem (40+ community plugins): GitHub Actions, ArgoCD, Sentry, PagerDuty, etc.
- Software templates: scaffold a new µservice via "Create" wizard backed by the flat layout per ADR-0131.

### Deployment shape

`microservices/observability/iac/helm/backstage/` ships the Helm chart per this ADR. Backstage runs in the observability cluster alongside Grafana + the SLO engine (ADR-0139). Authentication via OAuth-2.1 against the tenancy µservice (ADR-0002).

### Catalog metadata injection

Each µservice's `manifest.json` includes a `backstage_catalog_entity` block:

```json
{
  "backstage_catalog_entity": {
    "kind": "Component",
    "metadata": {
      "name": "meet",
      "owner": "axis-meet",
      "lifecycle": "production",
      "tags": ["video", "wave-b", "tier-a-api"]
    },
    "spec": {
      "type": "microservice",
      "providesApis": ["meet-rest-v1", "meet-grpc-v1"],
      "consumesApis": ["tenancy-grpc-v1", "audit-chain-grpc-v1"],
      "dependsOn": ["tenancy", "audit-chain", "observability"]
    }
  }
}
```

A Backstage catalog processor (registered via the per-µservice CI lane) reads these blocks and writes the canonical Backstage catalog entries.

### Scaffolder templates

Backstage Scaffolder templates back the "create new µservice" workflow. Templates implement the ADR-0131 flat layout authority, ensuring every new µservice ships with the canonical 16 directories + manifest.json + PRD.md + IP-001.

## Alternatives considered

### A. Custom-built developer portal
- Pros: tailored to Oyatie's exact metadata shape; no third-party dependency.
- Cons: NIH; Backstage covers 95% of what we'd build; the missing 5% is plugin-extension territory; engineering cost ~6 engineer-months for a worse version of Backstage. Spotify, Expedia, Netflix, Roku, American Airlines, Vodafone, and many others adopted Backstage rather than build.
- **Rejected**: NIH; Backstage is the industry-default substrate; we extend via plugins.

### B. GitHub repo READMEs + grep
- Pros: zero infrastructure.
- Cons: no aggregated view; no SLO embed; no on-call integration; no dependency graph; no scaffolder. The pattern that produced the current discoverability problem.
- **Rejected**: status quo; insufficient discoverability per the Context section.

### C. GitLab Service Desk + per-project wiki
- Pros: integrated with source control.
- Cons: Oyatie uses GitHub (per ADR-0017 brand-naming-and-repo-layout); per-repo wikis don't aggregate; no SLO embed; no scaffolder.
- **Rejected**: wrong source-control system + same fragmentation as option B.

### D. Confluence as the catalog
- Pros: tenant-friendly editing; widely understood.
- Cons: Confluence is a wiki, not a catalog — no metadata-driven entity graph; no scaffolder; no SLO embed; manual updates required; brittle to source-of-truth drift.
- **Rejected**: wiki vs catalog mismatch.

### E. Per-product portal, no fleet-wide aggregation
- Pros: per-product team owns its surface.
- Cons: cross-product discovery (the dominant use case for the workflow + ontology integration team) fails; fragments the engineer surface; not the hyperscaler shape.
- **Rejected**: fragments the engineer surface; cross-product use cases fail.

## Consequences

### Positive

1. **Hyperscaler-parity** — Oyatie engineers get a developer portal matching the Spotify/Netflix/Expedia/Vodafone industry default. Audit Row C4 closed.
2. **Aggregator over existing metadata** — we already have manifests, ADRs, runbooks, SLOs. Backstage projects them; no new authoring surface.
3. **Onboarding cost amortized** — new engineers find the right µservice + owner + runbook in ≤2 minutes vs the current ≥30-minute search.
4. **Scaffolder enforces ADR-0131** — the "Create µservice" template ships the canonical flat layout; no drift between authored and template-emitted µservices.
5. **Plugin ecosystem extensibility** — 40+ community plugins (GitHub Actions, ArgoCD, Cost Insights, Tech Insights) integrate at config-time cost only.

### Negative

1. **Backstage operator burden** — axis-developer-experience owns the Backstage upgrade cadence (~quarterly minor); plugin compatibility matrix to track.
2. **Node.js runtime in the portal cluster** — Backstage is TypeScript/Node.js (~200MB image); not Rust-first per ADR-0120, but ADR-0120 governs ON-PREM TOOLING, not the developer-portal substrate. Documented exception.
3. **Catalog drift risk** — if `manifest.json` and Backstage diverge, the portal misrepresents reality. Mitigation: CI lane validates manifest-to-catalog conformance.
4. **Identity coupling to tenancy µservice** — Backstage authenticates via tenancy µservice's OAuth-2.1; tenancy outage takes the portal offline. Acceptable because tenancy outage is already a fleet-wide P0.

### Operational

1. `microservices/observability/iac/helm/backstage/` is the canonical Helm chart (this ADR's skeleton).
2. Backstage version pinning: Backstage LTS releases (every ~6 months).
3. Catalog entity sync: per-µservice CI lane validates `manifest.json#backstage_catalog_entity` shape; merge updates the live catalog.
4. TechDocs build: per-µservice `docs/` rendered via MkDocs in CI; published to the Backstage TechDocs surface.
5. ADR plugin: community ADR plugin reads `docs/decisions/*.md`; renders searchable index.
6. SLO embed: Grafana plugin embeds per-µservice dashboards; deep-link to the full Grafana UI per ADR-0139 Layer-A.
7. RBAC: every Oyatie engineer authenticated via tenancy µservice; per-µservice surfaces restricted to owners + collaborators per `.github/CODEOWNERS`.

### Catalog kinds in use

Backstage's entity taxonomy maps onto Oyatie's structure as follows:

| Backstage kind | Oyatie entity | Source of truth |
|---|---|---|
| `Component` | `microservices/<ms>/` | per-µservice `manifest.json` |
| `Component` | `crates/oya-*` | workspace `Cargo.toml` + crate `Cargo.toml` |
| `API` | per-µservice OpenAPI + gRPC schema | `microservices/<ms>/contracts/` |
| `Resource` | Postgres DBs, Redis caches, S3 buckets per µservice | `microservices/<ms>/iac/` |
| `System` | tenant-facing product (workflow, social, messenger, etc.) | `specs/products/<product>.json` |
| `Domain` | bounded context per ADR-0137 | `microservices/<ms>/specs/bounded-contexts.json` |
| `Group` | engineering teams | `registry/teams.json` |
| `User` | engineer | `.github/CODEOWNERS` derivation |
| `Location` | repository / org | static config |

### Plugin set v1

| Plugin | Purpose |
|---|---|
| `@backstage/plugin-catalog` | core software-catalog surface |
| `@backstage/plugin-techdocs` | per-component documentation |
| `@backstage/plugin-scaffolder` | new-µservice scaffolding per ADR-0131 |
| `@backstage/plugin-github-actions` | per-µservice CI status |
| `@backstage/plugin-argo-cd` | per-µservice deploy status (ADR-0171) |
| `@backstage/plugin-grafana` | per-µservice SLO dashboards (ADR-0139) |
| `@backstage/plugin-pagerduty` | on-call routing per µservice |
| `@backstage/plugin-tech-insights` | scorecard checks against per-µservice `scorecards/` |
| `@backstage/plugin-cost-insights` | per-µservice infra spend (per ADR-0117 cost budgets) |
| community `adr-plugin` | ADR index reader for `docs/decisions/*.md` |

### Catalog-drift CI lane

A CI lane `oya-developer-portal-catalog-drift` runs on every PR touching `microservices/<ms>/manifest.json`. It:

1. Validates the `backstage_catalog_entity` block conforms to the Backstage JSON schema.
2. Ensures `metadata.owner` resolves to a Group in `registry/teams.json`.
3. Ensures `providesApis[]` and `consumesApis[]` reference declared contracts in `microservices/<ms>/contracts/`.
4. Ensures `dependsOn[]` references existing µservices.

Failures block merge.

### Performance budgets

- Catalog page p99 ≤500ms for the engineer's home view.
- Search p99 ≤300ms across the full catalog.
- TechDocs page p99 ≤800ms (MkDocs-rendered HTML cached at CDN edge).
- SLO embed (Grafana plugin) ≤2s p99 (Grafana-bound).
- Scaffolder template execution ≤60s p99 from "Create" click to PR opened.

### Migration / rollout plan

1. M01 slice: Helm chart skeleton + Backstage deploy in the observability cluster (this ADR's companion).
2. M01.5: catalog entity sync from `manifest.json`; basic Component + Group + User entities live.
3. M02: TechDocs + ADR plugin + Grafana plugin embedded.
4. M02.5: Scaffolder templates for new µservice creation (ADR-0131 compliance).
5. M03: ArgoCD + PagerDuty + Tech Insights + Cost Insights plugins.
6. M03 + 30 days: deprecation of the manual onboarding doc.

## References

- Spotify Backstage — https://backstage.io — canonical reference; OSS; software catalog + TechDocs + plugins.
- Backstage adopters — https://backstage.io/demos/ — Spotify, Expedia, Netflix, Roku, American Airlines, Vodafone, LinkedIn, Wayfair.
- Backstage Catalog model — https://backstage.io/docs/features/software-catalog/ — entity-kind taxonomy we adopt.
- Backstage Scaffolder — https://backstage.io/docs/features/software-templates/ — scaffolding templates for new µservices.
- Backstage TechDocs — https://backstage.io/docs/features/techdocs/ — MkDocs-based per-component documentation.
- Backstage Plugins Marketplace — https://backstage.io/plugins — Grafana, ArgoCD, GitHub Actions, PagerDuty, Cost Insights, Tech Insights.
- Netflix BAS — pre-Backstage ancestor; influenced the catalog model.
- Uber Service Hub — internal portal precedent; per-service SLO rollup.
- ADR-0011 — cross-microservice contract registry (API catalog source).
- ADR-0019 — doc catalog + update protocol (Backstage TechDocs source).
- ADR-0025 — Foundry as engineering platform (Foundry capabilities surface in the portal).
- ADR-0042 — observability stack OTel + in-house UI (SLO embed source).
- ADR-0120 — Rust-first on-prem tooling (Backstage is the documented exception).
- ADR-0131 — per-microservice flat layout (Scaffolder enforces this).
- ADR-0139 — agentic SLO-gated promotion (SLO embed surface).
- ADR-0167 — tenant-facing CLI (`oya` discoverability complements the engineer portal).
- `/specs/hyperscaler-architecture-invariants.json` — audit Row C4 closes here.
