# ADR-0203 — Documentation engine: three-tier separation

- Status: Accepted
- Date: 2026-05-18
- Deciders: Substrate architecture authority (oya-architecture-authority)
- Tags: substrate, documentation, developer-experience, marketing
- Supersedes: none
- Superseded by: none
- Related: ADR-0170 (developer portal — Backstage + TechDocs already
  selected),
  ADR-0173 (vendor lock-in avoidance — no commercial-only docs
  platform allowed),
  ADR-0185 (Workflow Studio client stack — SvelteKit Phase 1).

## Context

Three audiences with three different access patterns consume
oyatie documentation:

1. **Engineers reading the raw repository.** They live in
   `git`, `rg`, `bat`, IDE preview. They want a fast, navigable,
   in-repo rendering that survives offline + air-gapped review.
2. **Engineers browsing the service catalog.** They live in the
   internal developer portal (ADR-0170 Backstage). They want
   per-service docs federated alongside ownership, SLOs, ADR
   index, dashboards.
3. **External customers / prospects.** They live on the public
   web. They want polished marketing, onboarding, API reference,
   and search.

A single docs engine that tries to serve all three either degrades
to a lowest-common-denominator markdown previewer or accretes a
custom SPA layer for marketing that drifts from the in-repo
source.

## Decision

Three engines, three audiences, single Markdown source-of-truth.

### Tier 1 — In-repo technical docs: mdbook

- **Engine**: `mdbook` (Rust-native, MPL-2.0).
- **Sources**:
  - `docs/standards/` — engineering standards (e.g.
    `wasm-runtime-canonical.md`, `krm-iac-cluster-tier-boundaries.md`)
  - `docs/operators/` — operator runbooks
  - `docs/decisions/` — ADR index rendered as a navigable book
- **Audience**: internal engineers, raw repo readers.
- **Output**: static HTML, committed-render not required; build
  artifact published to internal docs bucket on `dev` branch
  promotion.

### Tier 2 — Service catalog: Backstage TechDocs

Backstage is Spotify-origin, donated to the CNCF (incubating).
TechDocs is its "docs-like-code" plugin: Markdown rendered via
MkDocs inside the Backstage UI.

- **Engine**: Backstage TechDocs (Markdown rendered via MkDocs
  inside the Backstage UI, per ADR-0170).
- **Sources**: per-µservice docs under
  `microservices/<ms>/` — PRD, IPs, runbooks, SLOs, dashboards,
  ownership manifests. TechDocs reads the same Markdown files
  mdbook reads when scoped to a single µservice.
- **Audience**: internal engineers browsing the service catalog.
- **Output**: live inside Backstage portal (ADR-0170).

### Tier 3 — Public docs / marketing: SvelteKit

- **Engine**: SvelteKit (ADR-0185 Phase 1 client stack; matches
  the canonical web rendering substrate).
- **Sources**:
  - Processed JSON exported from the same Markdown
    source-of-truth (mdbook generator emits a JSON sidecar; a
    follow-up generator script produces the SvelteKit content
    bundle).
  - OpenAPI 3.2.0 + AsyncAPI 3.1.0 contracts rendered via
    Redoc (preferred) or Stoplight Elements.
- **Audience**: external customers, prospects, search engines.
- **Output**: live at the public marketing domain.

### Cross-render contract

- Markdown source-of-truth lives in the repository under
  `docs/` and `microservices/<ms>/`.
- mdbook reads the in-repo Markdown directly.
- Backstage TechDocs reads the same Markdown via MkDocs at portal
  build time.
- SvelteKit reads a JSON content bundle produced by a generator
  step (the generator is owned by the `microservices/docs/`
  µservice).
- Diagrams: Mermaid + C4-PlantUML committed as source; rendered
  at build time by each tier.

### API reference

- OpenAPI 3.2.0 contracts under `microservices/<ms>/contracts/`
  → Redoc (preferred) → embedded in the SvelteKit public docs.
- AsyncAPI 3.1.0 contracts (event-driven µservices) → AsyncAPI's
  rendering CLI → embedded in the SvelteKit public docs.

## Alternatives considered

- **Docusaurus** — React-based, mature, but breaks the
  Rust-primary stack alignment (ADR-0185 picked SvelteKit Phase 1
  + Leptos Phase 2). Adding React just for docs creates a second
  client stack the discipline gate must whitelist. Rejected.
- **MkDocs alone** — Pure Python; no Rust path; lacks the
  marketing-page composition we want for Tier 3. Acceptable as
  the engine inside Backstage TechDocs (Tier 2) because that
  choice is already made by ADR-0170. Rejected as canonical for
  Tiers 1 + 3.
- **GitBook** — Commercial, vendor-locked. ADR-0173 forbids.
  Rejected.
- **Notion / Confluence / similar SaaS** — Vendor lock-in plus
  loss of source-of-truth in `git`. Rejected.
- **Single engine across all three tiers** — Always degrades to
  lowest-common-denominator or accretes a custom SPA for
  marketing. Rejected on principle.

## Consequences

- New µservice `microservices/docs/` (already scaffolded) owns
  the generator step that produces the SvelteKit content bundle
  + integrates the Redoc / AsyncAPI rendering pipeline.
- mdbook + TechDocs read the same Markdown; SvelteKit reads the
  generated bundle. No drift because all three trace back to the
  same `docs/` and `microservices/<ms>/` source.
- New contributors point at
  `docs/standards/wasm-runtime-canonical.md` and
  `docs/standards/krm-iac-cluster-tier-boundaries.md` as
  example engineering-standards reads rendered by mdbook.
- The discipline gate `oya-check-client-stack-discipline`
  (ADR-0185) already enforces SvelteKit / Leptos. No additional
  gate is required for Tier 3.

## Standards anchor

- `microservices/docs/` — µservice substrate.
- `docs/standards/` — mdbook source root.
- ADR-0170 (developer portal) — Backstage TechDocs binding.
- ADR-0185 (client stack) — SvelteKit binding.

## Migration

- T+0 (this ADR): canonical engines named; mdbook+TechDocs
  already operable; SvelteKit generator scaffolded.
- T+30d: SvelteKit generator produces the JSON bundle for at
  least one µservice end-to-end.
- T+60d: All µservices expose docs through all three tiers
  (each µservice has at minimum a PRD, IPs, and runbook visible
  in mdbook + TechDocs; public-facing µservices add a SvelteKit
  surface).

## In-house roadmap

Two of the three engines are community standards we adopt as-is.
The third tier (Backstage) is community standard with a conditional
Phase-2 in-house option named.

### Keep as community standards (no replacement planned)

- **mdbook** (Rust community standard, MPL-2.0, used by the Rust
  Project itself for The Rust Book / Cargo Book / Rust Reference)
  is the Tier-1 engine. Adopting mdbook *is* the in-house posture
  for a Rust-primary substrate. No Phase-2 replacement planned.
- **SvelteKit** (Tier 3) is covered by ADR-0185 as the canonical
  web client stack. Adopting SvelteKit *is* the in-house posture
  for Phase-1 web; Leptos is the Phase-2 in-house Rust-native
  successor named in ADR-0185.

### Phase 0 vendor adapter (gated)

- **Backstage TechDocs** (Tier 2) — Spotify-origin, donated to
  CNCF (incubating). Apache-2.0. Used as the developer-portal
  substrate per ADR-0170. Kept behind an adapter so we never
  bake Backstage assumptions into business logic.

### Phase 2 in-house build (conditional)

`oya-developer-portal` — in-house Rust-native developer-portal
substrate. Triggered ONLY if Backstage scope grows beyond
"service catalog + TechDocs" in a way that conflicts with our
substrate (e.g. plugin model that requires Node/TS runtime
inside our Rust-primary control plane, or licensing posture
changes, or scaling characteristics that Backstage cannot meet
for our µservice count).

**Trigger conditions** (any fires Phase 2). Numeric, not
aspirational:

1. Plugin model forces a Node/TS runtime into the cluster
   control plane in a way that violates ADR-0185 client-stack
   discipline.
2. Backstage upstream changes licensing in a way that violates
   ADR-0173.
3. Service-catalog scale exceeds Backstage's per-instance
   indexing budget. Concrete threshold: µservice count > 500
   OR per-µservice doc-page count > 10,000 with sustained
   p99 catalog query > 2 s.
4. Backstage scope creeps beyond "service catalog +
   TechDocs" into "IDP universal control plane" (e.g.
   per-Backstage-plugin replacing oyatie µservices); the
   substrate must remain the source-of-truth, not Backstage.

Until any trigger fires, Backstage is the canonical Tier-2
engine.

### In-house contribution path

Fixes / features in mdbook + Backstage TechDocs that land in our
integration are contributed upstream. Per ADR-0173.

## Open questions

- Search across all three tiers — single search index vs
  per-tier index — deferred to a follow-up ADR.
- Versioned docs (per-API-version reference) layout in SvelteKit
  deferred to a follow-up ADR.
