# ADR-0203 — Documentation engine: three-tier separation

- Status: Accepted
- Date: 2026-05-18
- Last reconciled: 2026-08-01
- Deciders: Substrate architecture authority (oya-architecture-authority)
- Tags: substrate, documentation, developer-experience, marketing
- Supersedes: none
- Superseded by: none
- Related: ADR-0157 (API gateway), ADR-0167 (tenant CLI), ADR-0173 (vendor lock-in avoidance),
  ADR-0176 (brownout signal), ADR-0182 (gateway/mesh separation), ADR-0258 (API versioning model),
  ADR-0393 (Leptos canonical app shell), ADR-0394 (first-party Rust developer portal)

## Context

Three audiences consume Oyatie documentation through different access patterns:

1. engineers reading the repository need fast, offline, air-gap-safe navigation;
2. engineers and operators browsing the service catalog need documentation composed with ownership,
   APIs, SLOs, runbooks, releases, and dashboards;
3. external customers and developers need polished, versioned public documentation and API
   reference.

A single renderer optimized for all three either loses repository fidelity or becomes a second
application platform. The source must remain in Git while each audience receives an appropriate
projection.

## Decision

Use three presentation tiers over one repository-owned Markdown and contract source of truth.

### Tier 1 — In-repository technical documentation

- **Engine:** `mdbook`.
- **Sources:** `docs/`, capability-local ADRs, runbooks, SLOs, and ownership documents.
- **Audience:** engineers, reviewers, and air-gapped operators.
- **Output:** a reproducible static artifact; rendered output is not committed authority.

### Tier 2 — First-party service-catalog documentation module

- **Engine:** the first-party Rust docs-portal module mounted in
  `app/ops-console/developer-portal/` per ADR-0394 and rendered through the Leptos shell per
  ADR-0393.
- **Sources:** repository Markdown plus canonical catalog, ownership, SLO, runbook, release, and API
  projections exposed through governed read APIs.
- **Audience:** internal engineers and operators browsing the capability catalog.
- **Output:** a live module in the first-party portal. The portal is a projection, never the source
  of truth.

Backstage and TechDocs may be consulted as feature references or one-way migration inputs. They are
not runtime dependencies, plugin hosts, catalog authorities, or deployment substrates.

### Tier 3 — Public documentation

- **Engine:** the canonical Leptos/Rust-WASM web shell from ADR-0393, with SSR and hydration.
- **Sources:** a deterministic content bundle projected from the repository source plus versioned
  OpenAPI 3.2 and AsyncAPI 3.1 contracts.
- **Audience:** external customers, developers, prospects, and search engines.
- **Output:** the public documentation and developer experience surface.

### Cross-render contract

- Markdown and contract sources remain repository-owned.
- Tier 1 reads Markdown directly.
- Tier 2 reads canonical content and metadata through owned projection APIs.
- Tier 3 reads a deterministic, provenance-bearing content bundle.
- No tier may write back to Markdown, catalog, ownership, or contract authority directly.
- Mermaid and C4-PlantUML remain source formats and are rendered during the applicable build.
- Every generated bundle is reproducible, freshness-checked, and traceable to an exact source SHA.

### API reference

- OpenAPI 3.2 contracts render the synchronous REST reference.
- AsyncAPI 3.1 contracts render event, webhook, and streaming reference.
- Protobuf descriptors may be shown for internal service owners but do not create a public gRPC
  contract.

## Alternatives considered

- **Backstage TechDocs or MkDocs as Tier 2 runtime:** rejected. This would introduce a parallel
  Node/Python portal substrate and a second catalog authority. Their information architecture may
  be used as a feature reference only.
- **Docusaurus:** rejected because it creates a second React/Node client stack.
- **GitBook, Notion, Confluence, or hosted documentation SaaS as authority:** rejected for vendor
  lock-in and loss of Git as source of truth.
- **One renderer for every tier:** rejected because repository, internal operational, and public
  presentation requirements have different reliability and interaction boundaries.

## Consequences

- Oyatie owns the Tier-2 and Tier-3 runtimes and their accessibility, security, and reliability.
- Documentation and API metadata remain authoritative outside the portal; portal indexes are
  disposable projections.
- The portal composes capability APIs and contains no duplicated domain logic.
- A Tier-2 outage degrades portal documentation without blocking raw repository access or public
  static reference artifacts.

## Migration

1. Retire any live Backstage/TechDocs runtime, plugin, or Helm authority rather than moving it into
   the capability tree.
2. Keep existing Markdown and catalog-shaped documents as migration inputs only when their
   provenance is explicit and no Backstage runtime is required to consume them.
3. Implement the first-party docs module under the multi-capability composition root.
4. Prove exact-SHA content freshness, degraded-mode behavior, accessibility, and cross-tier link
   integrity before promotion.

## Verification

- Repository scans find no promoted Backstage runtime, plugin package, or deployment chart.
- Tier-2 and Tier-3 builds consume the same canonical Markdown and contract inventory.
- A stale or provenance-free generated bundle fails closed.
- Browser evidence covers keyboard navigation, WCAG 2.2 AA, responsive rendering, search, and a
  degraded catalog dependency.
- OpenTelemetry evidence covers render latency, errors, saturation, source freshness, and failed
  downstream projections.

## Open questions

- Cross-tier search-index ownership and versioned-document retention remain follow-up decisions.
