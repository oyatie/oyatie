---
purpose: Oyatie — Documentation System
doc_status: published
---

# Oyatie — Documentation System

> **Status:** Draft v0.1 — 2026-05-09. Authored to answer "where does each kind of doc live, who owns it, when does it update, how is it generated?" The system is **Diátaxis-aligned** (tutorials / how-to guides / reference / explanation) with explicit per-audience surfaces.
> **Owner:** `council-architecture` for the system itself; per-doc owners in [`DOC-CATALOG.md`](DOC-CATALOG.md).
> **Companion:** [`DOC-CATALOG.md`](DOC-CATALOG.md) is the per-doc protocol; this doc is the system-level overview that names the kinds of docs.

---

## 1. Why this doc exists

Engineering documentation typically rots because no one decided what counts as a *kind* of doc. We adopt the **Diátaxis framework** (Daniele Procida's four-quadrant model) and assign every Oyatie document to one quadrant. Each quadrant has one storage location, one ownership model, one update cadence, one generation pipeline.

The four Diátaxis quadrants:

| Quadrant | Goal | Example for Oyatie |
|---|---|---|
| **Tutorials** | Learn-by-doing for a beginner | "Build your first Foundry capability in 30 minutes" |
| **How-to guides** | Solve a specific task | "Add a new tenant to the Cloud control plane" |
| **Reference** | Look up exact info | `oya verify --help`; `Capability` Rust struct rustdoc |
| **Explanation** | Understand the *why* | "Why Foundry consolidates Foundry — DESIGN §3" |

Plus a fifth, project-management quadrant for the consolidated docs themselves (PRD/ROADMAP/RISK-REGISTER/etc.). The consolidated docs are *project artifacts*, not user docs. They are exposed selectively to outside readers (trust portal mirrors).

---

## 2. Audiences

| Audience | What they read |
|---|---|
| **External developer / ISV** | Public API docs, SDK reference, marketplace docs, plugin authoring tutorials |
| **Tenant operator** | Tenant-admin how-to guides, security/compliance whitepapers, trust portal |
| **Tenant builder** | Workflow Studio tutorials, plugin authoring, no-code recipes |
| **Internal engineer + OSS contributor** | Per-crate rustdoc, in-tree CONTEXT/AGENTS/CLAUDE/CONTRIBUTING docs, ADRs, the canonical `docs/` set |
| **Foundry agent** | Machine-readable catalog (`machine-readable/`), capability schemas, evidence emission contracts |
| **Operator (SRE)** | Runbooks, SLO catalog, incident-management playbook, dashboards |
| **Auditor / Regulator** | Compliance matrix, evidence portal, control-evidence packs, trust portal |
| **Council / Founder** | PRD, ROADMAP, RISK-REGISTER, FINOPS, HIRING-CAPACITY, GTM-PLAN |

---

## 3. Storage map (where every kind of doc lives)

| Kind of doc | Location | Owning team | Generation |
|---|---|---|---|
| **Public API tutorials** (Diátaxis: Tutorials) | `docs.oyatie.com/tutorials/` (Leptos site sourced from `docs/site/tutorials/` mdbook) | `gtm-marketing` + per-axis | mdbook + Leptos overlay; Rust codegen interpolates current OpenAPI examples |
| **Public API how-to guides** (Diátaxis: How-to) | `docs.oyatie.com/guides/` | per-axis team | Same pipeline |
| **Public API reference** (Diátaxis: Reference) | `docs.oyatie.com/reference/` | `platform-api-sdk` | Auto-generated from OpenAPI / proto in `contracts/` |
| **SDK reference (Rust / TS / Python / Go)** | `docs.oyatie.com/sdk/<lang>/` | `platform-api-sdk` | Auto-generated from each SDK source |
| **Public conceptual docs** (Diátaxis: Explanation) | `docs.oyatie.com/concepts/` | `gtm-marketing` + `council-architecture` | Hand-authored, mdbook |
| **Tenant-admin user guide** | `docs.oyatie.com/admin/` | `axis-saas` + `gtm-customer-success` | mdbook |
| **Workflow Studio user guide** | `docs.oyatie.com/studio/` | `axis-saas` + `axis-foundry` | mdbook + interactive Leptos demos |
| **Plugin authoring guide** | `docs.oyatie.com/plugins/` | `axis-foundry` (plugin substrate is Foundry's foundry surface) | mdbook + Wasmtime sandbox demos |
| **Marketplace listing pages** | `marketplace.oyatie.com/` | `axis-saas` | Generated from marketplace records |
| **Per-crate Rust docs (rustdoc)** | `https://docs.rs/oya-<crate>` (also in-tree `target/doc/`) | per-crate owner | `cargo doc` |
| **Per-crate README** | `crates/oya-<crate>/README.md` | per-crate owner | Hand-authored; auto-included in rustdoc landing |
| **In-tree contributor docs** | `docs/standards/`, (retired; see `docs/teams/`), (retired; see `docs/`), `docs/runbooks/` | per-team | Hand-authored, in-repo |
| **CLAUDE.md / AGENTS.md / CONTEXT.md** (per directory) | rooted at each directory needing context | per-directory owner | Hand-authored; agent-consumed |
| **ADRs** | `decisions/ADR-####-<title>.md` | author per ADR; promotion by `crew-adr-promotion` | Hand-authored; indexed in [`ADR-INDEX.md`](ADR-INDEX.md) |
| **Runbooks** | `docs/runbooks/<runbook-id>.md` | `ops-sre-reliability` + per-axis | Hand-authored; indexed in [`RUNBOOKS-INDEX.md`](RUNBOOKS-INDEX.md) |
| **Threat models per service** | `docs/security/threat-models/<service>.md` | `ops-security` + per-team | Hand-authored; references in [`security-program/security-program.json`](security-program/security-program.json) |
| **Audit reports** | `evidence/audits/<date>-<topic>/` (sealed); not `docs/audits/` | per-auditor (council member or outside auditor) | Hand-authored / sealed evidence |
| **Mistakes-and-fixes ledger** | `docs/MISTAKES-LEDGER.md` | `council-architecture` (curator) | Hand-authored on every prevention-loop trigger |
| **Consolidated PM docs** | `docs/*.md` | per-doc owner (see DOC-CATALOG) | Hand-authored; agents may co-author per `agent_authoring_allowed` flag |
| **Per-product PRDs** | `docs/products/<product-id>/PRD.md` | per-product team | Hand-authored using the template at `products/_TEMPLATE.md` |
| **Per-team charters** | `docs/teams/<team-id>/CHARTER.md` | per-team | Hand-authored using a fixed shape |
| **Regional pack docs** | `docs/regional-packs/<pack-id>/PACK.md` | `regional-packs` team + per-pack maintainer | Hand-authored from a pack template |
| **Machine-readable manifests** | `docs/machine-readable/*.json` | `council-architecture` (schema); per-doc-owner (content) | Hand-authored or programmatically emitted |
| **Capability registry** | `registry/capability-templates/<capability-id>.yaml` | per-capability team | Hand-authored YAML; CI validation |
| **Catalog records** | `registry/catalog/<crate>.yaml` | per-crate owner | Hand-authored YAML; `oya catalog scaffold` agent-assisted |
| **Trust portal** | `trust.oyatie.com/` | `ops-compliance` + `council-privacy` | Generated from compliance evidence + audit-chain anchor proofs + DPIA / SOC2 / ISMS-P attestations |
| **Status page** | `status.oyatie.com/` | `ops-sre-reliability` | Generated from SLO catalog + incident management |
| **Engineering dev surface** | `dev.oyatie.com/` per ADR-0025 | `axis-foundry` (foundry surface) | Mix: catalog UI + Leptos portal |
| **Internal wiki** | `wiki.oyatie.com/` within the first-party Rust portal | per-team | Mix: agent-authored + human-edited; Backstage is at most a one-way import/reference source |

---

## 4. The five doc kinds × five generation pipelines

The toolchain that produces docs:

| Pipeline | Inputs | Outputs |
|---|---|---|
| **`oya-doc rustdoc`** | All `oya-*` Rust crates | Per-crate API reference at `docs.rs` mirror + in-tree `target/doc/` |
| **`oya-doc openapi`** | All `contracts/openapi/**/*.yaml` + proto schemas | Public API reference at `docs.oyatie.com/reference/` + per-language SDK docs |
| **`oya-doc mdbook`** | Hand-authored `docs/site/**/*.md` | `docs.oyatie.com/{tutorials,guides,concepts,admin,studio,plugins}/` |
| **`oya-doc adr-index`** | All `decisions/ADR-*.md` | `docs/ADR-INDEX.md` + `machine-readable/decisions.json` |
| **`oya-doc catalog`** | All `registry/catalog/*.yaml` + `registry/capability-templates/*.yaml` | `dev.oyatie.com/` portal + `machine-readable/{products,catalog,contracts,batches}.json` |

All five live under `oya doc <subcommand>` (sub-CLI of the persona-split per [`TOOLCHAIN.md §3`](TOOLCHAIN.md)). Implementation language: Rust.

Public site (`docs.oyatie.com`) is hosted via Cloud axis with the Leptos portal + mdbook content, served behind the public CDN per regional pack.

---

## 5. Documentation update protocol (cross-references DOC-CATALOG)

For consolidated PM docs: see [`DOC-CATALOG.md §3`](DOC-CATALOG.md) — the formal protocol with checklists.

For other docs:

| Kind | Trigger | Cadence | Validator |
|---|---|---|---|
| Per-crate rustdoc | every commit to crate | per-commit (CI) | `cargo doc --no-deps` succeeds |
| Per-crate README | crate's public surface changes | per-change | `oya doc lint readme` |
| OpenAPI / proto reference | `contracts/` change | per-change | `oya doc openapi diff` (semver gate) |
| SDK docs | SDK source change | per-change | per-SDK CI |
| Public conceptual docs | concept evolves | quarterly | `oya doc lint concepts` |
| Tutorials | new feature | per feature | manual verification of every step |
| ADR | new decision | per decision | `oya doc adr-index` regenerates |
| Runbook | new procedure / drift on existing | per-change + quarterly | `oya doc lint runbooks` |
| Threat model | service changes auth/data flow | per-change + annually | `oya doc lint threat-model` |
| Capability YAML | capability published | per-publish | `oya doc lint capability` |
| Catalog YAML | crate added/role-changed | per-change | `oya catalog validate` |

---

## 6. Diátaxis quadrant assignment per existing doc cluster

| Doc cluster | Quadrant | Why |
|---|---|---|
| `docs/standards/ci-lanes.md` | Reference | Lookup table for lanes |
| `docs/standards/code-review.md` (PR shape) | How-to | Procedure for opening a PR |
| `docs/standards/prevention-doctrine.md` | Explanation | Philosophy / why |
| `docs/MISTAKES-LEDGER.md` | Reference | Lookup of past mistakes |
| `docs/runbooks/*` | How-to | Solve a specific operational task |
| `docs/agents/*` | Reference | Lookup for agent-side surfaces |
| `docs/wiki/quickref/*` | Reference | Lookup |
| `decisions/ADR-*.md` | Explanation | Why this decision |
| `docs/PRD.md` | Project-management (the 5th quadrant we add) | What we're building |
| `docs/DESIGN.md` | Project-management + Explanation | How it interlocks + why |
| `docs/SPEC.md` | Reference | Surface lookup |
| `docs/ROADMAP.md` | Project-management | When |
| `docs/TOOLCHAIN.md` | Explanation | Why these tools |
| `docs/PRIVACY-PROGRAM.md` | Reference + Explanation | Lookup of policy + why |
| `docs/COMPLIANCE-MATRIX.md` | Reference | Lookup |
| `docs/RUNBOOKS-INDEX.md` | Reference | Lookup of runbooks |

---

## 7. Documentation-as-a-product (DaaP)

Documentation is treated as a product, not a debt:
- Every public docs page has a feedback widget; feedback flows to the owning team's queue.
- Every public API change auto-generates a "Migration guide" stub; per-axis team fills in.
- Every plugin marketplace listing requires a tutorial + how-to + reference + concept page (Diátaxis quadrants).
- Every new team-facing tool requires a tutorial + how-to + reference (3 of 4 quadrants minimum) within the same wave it ships in.
- Documentation defects (broken link, wrong example, outdated screenshot) are first-class issues with a `kind:doc-defect` label and an SLA per severity.

---

## 8. Translation / multilingual

Per [DESIGN §12 Regional Pack Architecture](DESIGN.md), each regional pack supplies translations for the public-facing docs that match its locale.

| Surface | Source language | Translated to |
|---|---|---|
| `docs.oyatie.com/tutorials/` | English (canonical) | Per-pack translations (KR / JP / ES / PT-BR / ZH / FR / DE / AR / HI ...) |
| `docs.oyatie.com/guides/` | English | Per-pack translations |
| `docs.oyatie.com/concepts/` | English | Per-pack translations |
| `docs.oyatie.com/reference/` | Auto-generated (locale-neutral) | (no translation needed) |
| `docs.oyatie.com/admin/` | English | Per-pack translations |
| `docs.oyatie.com/studio/` | English | Per-pack translations |
| `decisions/ADR-*.md` | English (canonical engineering) | (not translated; engineering docs stay English) |
| `docs/*.md` | English (canonical) | (not translated; PM docs stay English) |

Translation pipeline: `oya doc translate` invokes Foundry capabilities to draft translations + human-review queue per pack.

---

## 9. Doc generation pipeline (CI lane)

The CI lane `oya-governance-docs` is active as a documentation-system
contract guard. `oya doc rustdoc`, `oya doc openapi`, `oya doc mdbook`, and
`oya doc adr-index` are now active generator checks.
Rustdoc uses an isolated `target/oya-rustdoc-check` directory and a rustup-pinned
`rustdoc` to avoid mixed Homebrew/rustup metadata. The OpenAPI lane validates
versioned `contracts/openapi/**/*.yaml` sources, `x-oyatie-data-class`
annotations on parameters and schema properties, operation-to-runtime bindings in
[`registry/openapi/runtime-bindings.tsv`](../registry/openapi/runtime-bindings.tsv),
including the `oya-intelligence-api` inbound boundary for the
`foundry.capability.invoke` REST surface and typed runtime response-status parity,
with runtime-bound operations requiring explicit numeric response keys rather than
OpenAPI `default` or `1XX`-through-`5XX` ranges and status enums requiring
fieldless variants plus explicit `Self::Variant => <status>` code arms,
runtime response bodies requiring concrete `application/json` schema refs plus
exact status-to-schema mappings in the runtime binding registry,
schema-to-Rust-struct shape/type bindings in
[`registry/openapi/schema-bindings.tsv`](../registry/openapi/schema-bindings.tsv),
per-property `x-oyatie-rust-type` plus OpenAPI `type`/`format` parity, and
ADR-0037 semver metadata without requiring an external OpenAPI generator in the
bootstrap environment.
The mdBook lane validates the committed `docs/site` source tree, chapter graph,
and local links without requiring an external `mdbook` binary in the bootstrap
environment. The remaining public-doc generators are validated through
[`registry/docs/pipeline.tsv`](../registry/docs/pipeline.tsv):
each documented `oya doc` generator is either wired to an active check, guarded
for first adoption, or explicitly tracked with a `blocked:` rationale.

The product target remains the six-generator pipeline below:

1. `oya doc rustdoc` — `cargo doc --workspace --no-deps` with rustup-pinned `rustdoc`; diagnostics 0
2. `oya doc openapi` — OpenAPI 3.2 source shape, per-field `x-oyatie-data-class` annotations, operation-to-runtime bindings, typed explicit runtime response-status parity, exact response schema refs, schema-to-Rust-struct shape/type parity, and ADR-0037 semver metadata pass; semver-violating change requires explicit ADR
3. `oya doc mdbook` — committed site source validates; summary chapter graph and local links pass
4. `oya doc adr-index` — regenerates `ADR-INDEX.md` + `machine-readable/decisions.json`; checks committed copy matches
5. `oya doc catalog` — regenerates `machine-readable/{products,catalog,contracts,batches}.json`; checks committed copy matches
6. `oya doc lint` — every doc has Sources-scanned footer with a date ≤ 90 days old (warn) or ≤ 365 days old (block)

---

## 10. Tools that write docs (agent-authored)

Foundry agents can author or update specific doc kinds — see `agent_authoring_allowed` per row in [`DOC-CATALOG.md`](DOC-CATALOG.md). Examples:

- **`oya-intelligence-capability-doc-writer`** — drafts capability YAML + reference docs from a capability spec.
- **`oya-intelligence-rustdoc-fixer`** — proposes rustdoc fixes when CI flags missing or stale doc comments.
- **`oya-intelligence-runbook-extractor`** — distills runbooks from postmortems and on-call notes.
- **`oya-intelligence-translation-drafter`** — drafts per-pack translations of docs.
- **`oya-governance-adr-promoter`** — drafts ADR promotion PRs for Proposed → Accepted moves on `crew-adr-promotion` queue.
- **`oya-intelligence-glossary-extractor`** — finds new domain terms in PRs and proposes [`GLOSSARY.md`](GLOSSARY.md) rows.

Every agent-authored doc PR carries a `kind:agent-authored` label and goes through the human review per CLAUDE.md `## Code Review` rules.

---

## 11. Open questions

1. **Public docs hosting** — own static-hosting-on-Cloud-axis or use Vercel / Cloudflare Pages during bootstrap? Default: own; bootstrap via Cloudflare Pages until Cloud axis preview is ready.
2. **Per-region docs CDN** — separate per-pack edge or single global edge with locale negotiation? Default: single global edge; per-pack mirrors for residency-strict regions.
3. **Comment + feedback storage** — own (`oya-platform-feedback`) or third-party (Disqus / Hyvor — both have license risk)? Default: own.
4. **Search inside docs** — use the search axis or a separate Algolia-class index? Default: own; use the search axis after preview lands.
5. **Doc translation provider** — Foundry capability (any of Claude/OpenAI/Gemini) or human-only? Default: Foundry-drafted, human-reviewed.

---

## 12. Sources scanned

- Diátaxis framework reference (Daniele Procida)
- Existing in-tree docs in `docs/`, `decisions/`, `registry/`, `~/.claude/plans/look-at-all-outstanding-buzzing-teacup.md`
- ADR-0017 (Bench), ADR-0015 (repo structure), ADR-0025 (dev.oyatie.com), ADR-0050 (governance umbrella), ADR-0001 (deprecation)
- Industry references: rustdoc, mdbook, OpenAPI Generator, Backstage TechDocs, Stripe API docs (a public-API benchmark), Anthropic / OpenAI / Google docs sites (style benchmark), Cloudflare Workers docs (developer-tutorial benchmark), MDN (web reference benchmark)
- [`TOOLCHAIN.md`](TOOLCHAIN.md) for the tools that write/serve the docs
- [`DOC-CATALOG.md`](DOC-CATALOG.md) for the per-doc protocol

*Footer regenerated whenever this doc is edited.*
