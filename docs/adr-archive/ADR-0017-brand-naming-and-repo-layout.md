---
id: ADR-0017
status: Superseded
superseded_by: [ADR-0701]
doc_status: published
amended_by: [ADR-0532]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0017: Brand naming and repo layout — Oyatie / oYa logo / oyatie.com domain, oya-<microservice>-<layer> Cargo prefix, repo path / GitHub slug oyatie retained

> **Status:** Accepted
> **Owner:** `council-architecture` + `gtm-marketing`
> **Date:** 2026-05-09 (rewritten 2026-05-13 — Cargo prefix updated to BNF v4.1; "axis" references removed)
> **Related:** ADR-0001, ADR-0011, ADR-0018, ADR-0056

---

## Context

The 2026-05-08 user directive standardized the product as `Oyatie` (logo `oYa`, domain `oyatie.com`). The repo path + GitHub slug — `jason931225/oyatie` — remains stable because filesystem migration cost exceeds brand purity. Cohesion (ADR-0001) makes the rename important: a mixed-brand product is observably two products, and the cohesion moat collapses to zero.

BNF v4.1 (ADR-0056) updated the Cargo prefix pattern from the old `oya-<context>-<role>` to `oya-<microservice>-(<bc>-)?<layer>`.

---

## Decision

We adopt **Oyatie** as the product brand, **`oYa`** as the logo abbreviation, **`oyatie.com`** as the domain, **`oya-<microservice>-(<bc>-)?<layer>`** as the Cargo prefix per BNF v4.1 (ADR-0056), and explicitly retain the repo path / GitHub slug **`jason931225/oyatie`**.

### Brand rules

| Element | Value | Notes |
|---|---|---|
| Product name | **Oyatie** | Title case; never `oyatie` in prose |
| Logo abbreviation | **oYa** | Capital-O lower-y capital-A |
| Domain | **oyatie.com** | Primary; `*.oyatie.com` for tenant + service subdomains |
| Cargo prefix | **`oya-`** | Per BNF v4.1 (ADR-0056) |
| npm scope | **`@oyatie/`** | For published JS/TS SDKs |
| Container registry | **`oyatie/<image>`** | Per release governance |
| Trust portal | **`trust.oyatie.com`** | Per ADR-0003 |
| MCP gateway | **`mcp.<tenant>.oyatie.com`** | Per Foundry MCP server |
| Repo path | **`jason931225/oyatie`** | Retained — filesystem migration cost exceeds brand purity |
| GitHub slug | **`oyatie`** | Same |
| Issue tracker | `jason931225/oyatie` GitHub Issues | Per `docs/agents/issue-tracker.md` |

### What sweeps to Oyatie (product surface)

Every customer-, tenant-, regulator-, partner-visible surface uses **Oyatie**:
- Product names + UI strings + marketing copy + landing pages.
- API responses (`X-Brand: Oyatie`), webhook signatures (`X-Oyatie-Signature`).
- SDK names, package names, image names, k8s namespace names.
- Release tags (`v<n>` with no brand prefix; release notes header `Oyatie v<n>`).
- Docs site (`docs.oyatie.com`), trust portal, status page.
- Email-from addresses, support handles, customer comms templates.
- Catalog records (`registry/catalog/<crate>.yaml: brand: Oyatie`).
- Capability namespaces (e.g. `oya.foundry.capability.invoke`).

### Stable dev surface

- Repo URL: `github.com/jason931225/oyatie`.
- Filesystem path: `/Users/jasonlee/oyatie/...`.
- Internal git remote names.

### 17 rename sub-batches

The full rename is sub-divided into 17 batch-shaped PRs to avoid 6,560-touchpoint mega-PRs:

| # | Sub-batch | Scope |
|---|---|---|
| 1 | `brand-rename-docs` | Markdown / docs site / per-product PRDs / runbooks |
| 2 | `brand-rename-cargo` | Cargo crate names + `Cargo.toml` package names + dep references |
| 3 | `brand-rename-npm` | npm package names + `package.json` + scope |
| 4 | `brand-rename-urls` | URL strings, env templates, public-API docs |
| 5 | `brand-rename-ui-svelte` | Svelte / SvelteKit web UI |
| 6 | `brand-rename-ui-mobile` | iOS Swift + Android Kotlin string tables |
| 7 | `brand-rename-ui-html` | Static HTML / hand-rendered UI |
| 8 | `brand-rename-config-yaml` | Helm + IaC + manifests |
| 9 | `brand-rename-config-quadlet` | Podman / quadlet configs |
| 10 | `brand-rename-config-json` | JSON configs (settings, manifests) |
| 11 | `brand-rename-rust-srv` | Rust server source-tree strings |
| 12 | `brand-rename-scripts` | shell + Node + Python scripts |
| 13 | `brand-rename-adrs-cosmetic` | Per-ADR brand mentions in legacy ADRs (forensic) |
| 14 | `brand-rename-design-system` | Design tokens, brand colors, type scale |
| 15 | `brand-rename-doc-trees` | Per-doc-tree replacements |
| 16 | `brand-rename-canonical-trio` | `CONSTITUTION.md`, `docs/DOC-CATALOG.md`, `CLAUDE.md` |
| 17 | `brand-rename-public-comms` | External GitHub repo description, README banner, releases banner |

Each sub-batch is a separate PR with bounded blast radius.

### Boundary

- Applies to: every customer-, tenant-, regulator-, partner-visible product surface.
- Does not apply to: filesystem path, GitHub repo URL, git remote names.

---

## Consequences

### Positive

- Brand becomes mechanically singular at the product surface.
- 17-batch sequencing makes the rename tractable; no mega-PR.
- Cargo prefix `oya-` is a clean, KR-recognizable, SDK-friendly identifier.

### Negative

- 17 batches is real ops work over ~2 waves.
- Filesystem-vs-product-brand split is a teaching moment for new contributors.

### Operational

- CI: `oya-check-glossary` (per ADR-0018) is the brand-consistency enforcement lane.
- Per-batch evidence: `EVT-RENAME-BATCH-COMPLETED` emitted to the audit chain.

---

## Amendment (2026-06-08, WAVE-1 Agentic Delivery Fabric convergence)

Amended in place (no tombstone; git history preserves the pre-amendment body). **ADR-0532** (platform
product-line taxonomy + canonical product names) supersedes the oyatie-internal app-naming and
repo-layout assumptions FOR THE LIFECYCLE-TOOLING PRODUCTS: the `oya-` prefix moves from a baked-in
assumption to a per-profile config value (`profile = 'neutral' | 'oyatie'`, ADR-0533), and the
gate-pack namespace becomes product-rooted (`<pack>.<gate>`). The product-surface brand rules
(Oyatie / oYa / oyatie.com) and the repo-path retention below are otherwise unchanged for first-party
oyatie surfaces; ADR-0532 only generalizes the prefix/layout for third-party adopters of the product
line.

## Related

- ADR-0001 (cohesion — single brand)
- ADR-0018 (glossary — forbidden term enforcement)
- ADR-0056 (BNF v4.1 — Cargo prefix `oya-<microservice>-(<bc>-)?<layer>`)
- ADR-0532 (platform product-line taxonomy — amends the prefix/layout assumptions for lifecycle tooling)
