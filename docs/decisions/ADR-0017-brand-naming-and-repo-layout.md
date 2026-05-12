# ADR-0017: Brand naming and repo layout — Oyatie / oYa logo / oyatie.com domain, oya-<context>-<role> Cargo prefix, repo path / GitHub slug oyatie retained, product code uses Oyatie everywhere, dev-side aliases tolerated for ~2 waves then sunset

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `council-architecture` + `gtm-marketing`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0011, ADR-0015, ADR-0018, ADR-0019

---

## Context

The 2026-05-08 user directive standardized the product as `Oyatie` (logo `oYa`, domain `oyatie.com`). The repo path + GitHub slug — `jason931225/oyatie` — remains stable because filesystem migration cost exceeds brand purity (LEDG-018 records the related Foundry-naming question). Without an authoritative Foundation-pack ADR pinning the brand scope + per-batch sequencing + alias-sunset path, the consolidation creates 6,560 touchpoints across docs, code, configs, npm, Cargo, urls, UI surfaces — and an unbounded number of accidental regressions where new code reintroduces deprecated aliases.

Cohesion (ADR-0001) makes the rename more important: the seven-axis claim depends on a single brand surface across SaaS, Workspace, Vertical, Foundry, Cloud, Search, and Ads. A mixed-brand product is observably two products, and the cohesion moat collapses to zero. The cross-axis contract registry (ADR-0011) requires consistent type identifiers; the flat-crates target (ADR-0015) cites `oya-*` prefix as the canonical Cargo name.

---

## Decision

We adopt **Oyatie** as the product brand, **`oYa`** as the logo abbreviation, **`oyatie.com`** as the domain, **`oya-<context>-<role>[-<capability>]`** as the Cargo prefix per ADR-0015, and explicitly retain the repo path / GitHub slug **`jason931225/oyatie`**. Brand consolidation proceeds via 17 rename sub-batches; dev-side aliases survive ~2 waves then sunset.

### Brand rules

| Element | Value | Notes |
|---|---|---|
| Product name | **Oyatie** | Title case; never `oyatie` in prose |
| Logo abbreviation | **oYa** | Capital-O lower-y capital-A; used in compact UI surfaces |
| Domain | **oyatie.com** | Primary; `*.oyatie.com` for tenant + service subdomains |
| Cargo prefix | **`oya-`** | Per ADR-0015 |
| npm scope | **`@oyatie/`** | For published JS/TS SDKs |
| Container registry | **`oyatie/<image>`** | Per ADR-0019 release governance |
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
- Docs site (`docs.oyatie.com`), trust portal, status page (`status.oyatie.com`).
- Email-from addresses, support handles, customer comms templates.
- Catalog records (`registry/catalog/<crate>.yaml: brand: Oyatie`).
- Capability namespaces (e.g. `oya.foundry.capability.invoke`).

### Stable dev surface

- Repo URL: `github.com/jason931225/oyatie`.
- Filesystem path: `/Users/jasonlee/oyatie/...` (per the user directive 2026-05-08).
- Internal git remote names.
- Some legacy CI artifact names where filesystem coupling is unavoidable for ~2 waves.

### 17 rename sub-batches

The full rename is sub-divided into batch-shaped work to avoid 6,560-touchpoint mega-PRs:

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
| 15 | `brand-rename-doc-trees` | Per-doc-tree replacements (recon outputs, audits) |
| 16 | `brand-rename-canonical-trio` | `CONSTITUTION.md`, `docs/DOC-CATALOG.md` (per [`DOC-CATALOG.md`](../DOC-CATALOG.md)), `CLAUDE.md` |
| 17 | `brand-rename-public-comms` | External GitHub repo description, README banner, releases banner |

Each sub-batch is a separate PR with bounded blast radius. Cross-batch order: brand-rename-cargo (sub-batch 2) is the most-PR-shape constrained because of the `Cargo.toml [workspace.members]` serialization invariant per ADR-0015.

### Dev-side alias tolerance + sunset

For ~2 waves, the following **aliases** are tolerated (warn at PR open; do not fail merge):

- Legacy Cargo crate aliases — old-name `Cargo.toml` shims accepted as deps; soft-warn.
- Deprecated brand mentions in dev docs (`AGENTS.md`, `docs/standards/*.md` excluding canonical sources) — soft-warn.
- Legacy env var aliases in non-production scripts — soft-warn.

After the alias sunset window (~2 waves; council-decided), `oya-foundry-fitness-brand-rename` lane promotes the warns to BLOCK errors. Legacy ADRs preserve decision rationale while using deprecated-term-safe wording.

### Boundary

- Applies to: every customer-, tenant-, regulator-, partner-visible product surface.
- Does not apply to: filesystem path, GitHub repo URL, git remote names, legacy ADR forensic content.

---

## Consequences

### Positive

- Brand becomes mechanically singular at the product surface.
- 17-batch sequencing makes the rename tractable; no mega-PR.
- Dev-side aliases decouple internal-rename pace from external-rename completeness; engineers don't pay for marketing.
- Cargo prefix `oya-` is a clean, KR-recognizable, SDK-friendly identifier.

### Negative

- 17 batches is real ops work over ~2 waves.
- Filesystem-vs-product-brand split is a teaching moment for new contributors.
- Subscription-mode adapter URLs (Anthropic / OpenAI / Gemini sessions) must continue to point at vendor domains; brand sweep is product-only.

### Operational

- On-call: not applicable (architectural).
- Runbooks: `runbooks/brand-rename-batch-execute.md`, `runbooks/brand-rename-rollback.md`, `runbooks/alias-sunset-promotion.md`.
- CI: `oya-foundry-fitness-brand-rename` (per-PR brand consistency), `oya-foundry-fitness-cargo-prefix` (every workspace member starts with `oya-`).
- Per-batch evidence: `EVT-RENAME-BATCH-COMPLETED` emitted to the audit chain.

---

## Alternatives considered

### Alternative A — Single mega-PR rename

- **Pros:** atomic.
- **Cons:** 6,560 touchpoints; merge conflict probability ~1; review impossible.
- **Rejected because:** scale.

### Alternative B — Force repo path migration during the brand sweep

- **Pros:** brand purity end-to-end.
- **Cons:** filesystem migration touches every dev's local clone, every CI cache key, every external doc link; cost exceeds value.
- **Rejected because:** user directive 2026-05-08.

### Alternative C — `oyatie-*` Cargo prefix instead of `oya-`

- **Pros:** longer, fully-spelled brand.
- **Cons:** 13-character prefix is visually noisy; `oya-` is concise + KR-readable.
- **Rejected because:** user directive 2026-05-08; ROADMAP §8 Q10 default.

---

## Open questions

1. **Q1.** Foundry naming evaluation (LEDG-018) — keep `Foundry` or rename per the ADR-0006 "no Palantir vocabulary" clause? Default: keep with differentiation rationale; council ratifies. → council.
2. **Q2.** Search consumer brand (LEDG-019) — `Oyatie Search` or a separate brand for KR-Naver-class? Default: `Oyatie Search` initially; revisit at W-Search-Stable. → ADR-0012 + GTM.
3. **Q3.** Per-region brand surface — does `Oyatie` translate or transliterate per locale? Default: transliterate (`오야띠`); per-pack content-safety adjudicates. → ADR-0010.
4. **Q4.** Alias sunset window — exactly 2 waves, or council-decided per-batch? Default: 2 waves baseline; council can extend/shorten per-batch. → ADR-0019.
5. **Q5.** Workspace personal-use surfaces (Connect Personal) — same brand or sub-brand? Default: same brand (Workspace is part of Oyatie). → ADR-0012.

---

## References

- `docs/PRD.md` §1 brand line, §9 (decision log: Oyatie standardization 2026-05-08; repo slug retained 2026-05-08)
- `docs/GLOSSARY.md` §8 ("Oyatie", "oYa"), §11 (deprecated terms)
- `docs/CONTRADICTION-LEDGER.md` LEDG-018 (Foundry naming), LEDG-019 (Search brand), brand-rename batch (LEDG-030+)
- `docs/ROADMAP.md` §5 (brand-rename 17 sub-batches), §8 Q9 (repo path retention), Q10 (Cargo prefix)
- ADR-0001 (cohesion — single brand), ADR-0011 (catalog `brand:` field), ADR-0015 (Cargo `oya-*` prefix), ADR-0018 (deprecated-term enforcement), ADR-0019 (per-batch evidence emission)
