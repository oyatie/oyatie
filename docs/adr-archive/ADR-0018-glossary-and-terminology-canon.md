---
id: ADR-0018
status: Superseded
superseded_by: [ADR-0709]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0018: Glossary and terminology canon — industry-aligned vocabulary, Oyatie-specific terms with industry analog, Korean-English parity table, oya-check-glossary CI lane

> **Status:** Accepted
> **Owner:** `council-architecture`
> **Date:** 2026-05-09 (rewritten 2026-05-13 — canonical glossary updated)
> **Related:** ADR-0001, ADR-0011, ADR-0056, ADR-0017, ADR-0019, ADR-0055, ADR-0058

---

## Context

Multiple microservices, multiple regulatory packs, KR + JP + EN + EU + IN + BR + KSA + UAE + ANZ + SG locales, and an agent-runtime that authors prose at scale all push terminology into chaos. Without an authoritative glossary, "tenant" means three things in three microservices; "wave" gets confused with "milestone". Cohesion (ADR-0001) requires a single vocabulary across the entire flat catalog.

Session decisions 2026-05-13 established a revised canonical glossary that supersedes prior usage of "platform", "vertical", "arm", "Product Group", "Ontology", "Shell", and "Workspace" (as a product name).

---

## Decision

We adopt the **glossary canon** with five rules, an industry-aligned vocabulary list, Oyatie-specific terms, a Korean-English parity table, and a CI lane that hard-fails forbidden tokens.

### The five vocabulary rules

1. **Industry-standard term wins** when one exists and is unambiguous.
2. **Oyatie-specific term** is reserved for genuinely new concepts or for renames the brand has explicitly chosen. Industry analog is required in every Oyatie-specific entry.
3. **Korean and English are co-equal** for KR-specific terms.
4. **Renamed terms** carry a `Replaces:` line listing prior Oyatie names.
5. **Stale terms are removed** from canonical usage, not marked as retired in active docs. Historical context is confined to ADRs that explicitly document the rename.

### Canonical glossary (current)

| Oyatie term | Definition | Industry analog | Replaces |
|---|---|---|---|
| **Oyatie** | The product brand | (brand) | — |
| **oYa** | The logo abbreviation | (brand) | — |
| **Application** | The B2B unified shell; tenants sign in and enable products à-la-carte | "App shell" / "Workspace shell" | Shell, Modular Product Shell |
| **Ontology** | Engine-enforced typed-entity domain-data layer (ADR-0006) | Palantir Ontology; Apache Atlas + DDD aggregate persistence | Ontology (OG) |
| **Workflow** | Cross-product action/orchestration adapter; state machines, DAGs, approvals, SLA, handoffs (ADR-0035) | "Workflow engine" / "BPM" | (was Corporate-owned in Bominal; now shared) |
| **Foundry** | Oyatie's AI agent runtime + control plane + engineering platform | "Agent platform" / "AI orchestration runtime" | — |
| **microservice** | Independent, modular product or feature in the flat catalog; integrates with others via Workflow + Ontology | "Microservice" (standard) | vertical, arm, product-group |
| **flat catalog** | The canonical list of all shared microservices; no grouping by industry or arm | "Service catalog" | Arms, Product Groups, verticals |
| **shared** | Cross-cutting substrate consumed by every microservice | "Platform substrate" | platform |
| **Connect** | The communications + community product (dual-context: Professional + Personal) | "Workspace" (collaboration suite) | Workspace (as oyatie product name) |
| **Capability** | Discrete unit of agent-invocable functionality with declared inputs/outputs/policy | "Tool" (LangChain) / "Function" (OpenAI) | — |
| **Autonomy ceiling** | Per-tenant maximum tier of agent autonomy (T1..T4) (ADR-0007) | "Permission tier" / "Agent governance level" | — |
| **Plane** | Control / Data / Analytics (ADR-0004) | Industry standard | — |
| **Wave** | A coordinated sequence of work landing together (ADR-0016) | "Release train" / "Increment" (SAFe) | — |
| **Ecosystem-as-a-Service (EaaS)** | The cohesion thesis (ADR-0001) | "Platform-as-a-Service" / "Vertical SaaS" | — |

### Forbidden terms (stale — do not use in new content)

| Forbidden | Use instead | Authority |
|---|---|---|
| `platform` (as architectural label for oyatie substrate) | `shared` | `[[feedback-glossary-shared-not-platform]]` |
| `Ontology`, `OG` | `Ontology` | `[[feedback-glossary-ontology-not-object-graph]]`; ADR-0055 |
| `Workspace` (as oyatie product name) | `Connect` | `[[feedback-flat-product-catalog]]` override #7 |
| `Shell`, `Modular Product Shell` | `Application` | `[[feedback-bominal-inheritance-precedence]]` override #8 |
| `vertical`, `arm`, `Product Group` (as architectural grouping) | `microservice`, `flat catalog` | `[[feedback-flat-product-catalog]]`; ADR-0058 |
| `<shared\|vertical>` (BNF slot2 enum) | `<microservice>` open kebab (BNF v4.1) | ADR-0056 v4.1 |
| `oya-platform-*` crate prefix | `oya-<microservice>-*` per BNF v4.1 | ADR-0056 |
| `oya-shared-*` crate prefix for products | `oya-<microservice>-*` per BNF v4.1 (shared only for substrate) | ADR-0056 |
| M0 / M1 / M2 / M3 / minimum-shippable-tier (milestone labels) | Wave names per ADR-0016 | ADR-0016 |

### Korean-English parity (illustrative — full list in GLOSSARY.md §9)

| Korean (canonical) | English (canonical) |
|---|---|
| 개인정보보호법 | KR Personal Information Protection Act (PIPA) |
| 한국인터넷진흥원 | Korea Internet & Security Agency (KISA) |
| 식품의약품안전처 | Ministry of Food and Drug Safety (MFDS) |
| 금융위원회 | Financial Services Commission (FSC) |
| 망분리 | Network separation |
| 전자세금계산서 | Electronic tax invoice |
| 청소년보호법 | Juvenile Protection Act |
| 의료법 | Medical Service Act |
| 신용정보법 | Credit Information Use and Protection Act |
| 공공정보법 | Public Data Provision Act |

### CI lane: `oya-check-glossary`

The lane runs on every PR touching markdown / Rust source / catalog records and:

1. Hard-fails on forbidden tokens: `Object Graph`, `Workspace` (as product), `Shell` (as UI), `platform` (as architectural label), `vertical` (as architectural grouping), `Arm` (as architectural grouping), `Product Group`, `<shared|vertical>` in BNF context.
2. Warns on inconsistent capitalization (`oyatie` vs `Oyatie`).
3. Warns on uncited acronyms.

---

## Consequences

### Positive

- Vocabulary becomes auditable; drift is CI-blocked rather than manual.
- Stale terms removed from forward work; historical context preserved in specific rename ADRs.

### Negative

- Initial sweep cost is real; every legacy doc gets a glossary review.

---

## Related

- ADR-0001 (cohesion — single vocabulary)
- ADR-0055 (Object Graph → Ontology rename)
- ADR-0056 (BNF v4.1 — `<shared|vertical>` slot2 retired)
- ADR-0058 (Flat microservice catalog — Product Groups retired)
- `[[feedback-glossary-shared-not-platform]]`
- `[[feedback-glossary-ontology-not-object-graph]]`
- `[[feedback-flat-product-catalog]]`
- `[[feedback-bominal-inheritance-precedence]]`
