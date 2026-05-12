# ADR-0018: Glossary and terminology canon — industry-aligned vocabulary, Oyatie-specific terms with industry analog, Korean-English parity table, retired terms appendix, oya-foundry-fitness-glossary CI lane

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `council-architecture`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0011, ADR-0015, ADR-0016, ADR-0017, ADR-0019

---

## Context

Seven axes, multiple regulatory packs, KR + JP + EN + EU + IN + BR + KSA + UAE + ANZ + SG locales, and an agent-runtime that authors prose at scale all push terminology into chaos. Without an authoritative glossary, "tenant" means three things in three axes (per-axis customer; per-axis schema namespace; per-axis billing entity); "wave" gets confused with "milestone"; "Foundry" risks colliding with industry analogs the user does not use. The contradiction ledger LEDG-018 + LEDG-019 + LEDG-020 record naming-related drifts.

Cohesion (ADR-0001) requires a single vocabulary across the seven axes. Industry alignment (DDD, clean-arch, Diátaxis, SRE, AWS canon for cell-architecture and shuffle-sharding, Google SRE workbook for SLO/SLI/SLA/error-budget, MCP for tool-use schemas) wins where the industry term is unambiguous. Oyatie-specific terms are reserved for genuinely new concepts (Object Graph, Bench, Capability namespace, Autonomy ceiling, Persona tier, Plane gate, Wave, Claim ceiling, Foundation bypass, Catalog record, Capability record, Repoctl). Korean-English parity is required for KR statutes and KR-locale customer-facing text.

---

## Decision

We adopt the **glossary canon** with five rules, an industry-aligned vocabulary list, an Oyatie-specific terms list, a Korean-English parity table, a retired-terms appendix, and a CI lane that hard-fails forbidden tokens + warns on inconsistent usage.

### The five vocabulary rules

1. **Industry-standard term wins** when one exists and is unambiguous. Use `hexagonal architecture`, `clean architecture`, `bounded context`, `aggregate`, `value object`, `entity`, `repository`, `saga`, `outbox pattern`, `CQRS`, `event sourcing`, `idempotency key`, `backpressure`, `cell architecture`, `shuffle sharding`, `SLO / SLI / SLA / error budget`, `OIDC / SAML / STS / Cedar / OPA`, `OLTP / OLAP / HTAP`, `k-anonymity`, `Differential Privacy`, `BM25 / TF-IDF`, `HNSW / IVF / PQ`, `RAG`, `MCP`, `CPM / CPC / CPA / ROAS`, `HIPAA / PCI-DSS / SOC2 / ISO 27001 / NIST CSF / GDPR / PIPA / KISA / CSAP / K-ISMS-P / KCMVP`, `HL7 v2 / FHIR R4 / DICOM / NCPDP / X12 EDI / ICD-10-CM / SNOMED CT / LOINC / RxNorm / ISA-95 / OPC UA / MES / SCADA`, `NACHA / SWIFT / RTP`, `KYC / KYB / AML`.

2. **Oyatie-specific term** is reserved for genuinely new concepts or for renames the brand has explicitly chosen. Industry analog is required in every Oyatie-specific entry.

3. **Korean and English are co-equal** for KR-specific terms; the canonical pair lists both with the legal/industry-canonical form first.

4. **Renamed terms** carry a `Replaces:` line listing prior Oyatie names (e.g. Bench replaces "shell" per legacy ADR-0017 ancestry).

5. **Deprecated terms** are kept in the retired appendix, never removed.

### Oyatie-specific terms (with industry analog — illustrative; full list in GLOSSARY.md §8)

| Oyatie term | Definition | Industry analog |
|---|---|---|
| **Oyatie** | The product brand | (brand) |
| **oYa** | The logo abbreviation | (brand) |
| **Bench** | The user-facing app shell that hosts vertical workspaces | "Workspace shell" / "App shell" |
| **Object Graph (OG)** | Engine-enforced typed-entity domain-data layer (ADR-0006) | Closest: Apache Atlas + DDD aggregate persistence |
| **Foundry** | Oyatie's AI agent runtime + control plane + engineering platform (axis 4) | "Agent platform" / "AI orchestration runtime" / "AI gateway"; closest commercial: LangSmith + AWS Bedrock Agents |
| **Capability** | Discrete unit of agent-invocable functionality with declared inputs/outputs/policy | "Tool" (LangChain) / "Function" (OpenAI) / "Skill" (Microsoft Copilot Studio) |
| **Capability namespace** | Scoped collection of capabilities a tenant binds to | "API surface area" / "service catalog scope" |
| **Autonomy ceiling** | Per-tenant maximum tier of agent autonomy (T1..T4) (ADR-0007) | "Permission tier" / "Agent governance level" |
| **Persona tier (T1..T4)** | Agent-action authority levels | (Oyatie-specific) |
| **Pillar (data ownership)** | Org-owned / Person-owned / Public / Opt-in-Consumer (ADR-0008) | "Data domain" / "Data product" (Data Mesh) |
| **Plane** | Control / Data / Analytics (ADR-0004) | Industry standard |
| **Plane gate** | CI gate that triggers when a surface changes plane class | "Cross-plane review" |
| **Wave** | A coordinated sequence of work landing together (ADR-0016) | "Release train" / "Increment" (SAFe) |
| **Claim ceiling** | Mechanical block preventing a preview slice from claiming a foundation guarantee that the foundation hasn't shipped | "Capability gating" / "Feature flag with provenance" |
| **Foundation bypass** | Tracked, expirable carve-out from a foundation gate | "Tech-debt waiver" / "Exception ticket" |
| **Catalog record** | YAML manifest describing a flat-crate (ADR-0015) | "Service catalog entry" (Backstage) |
| **Capability record** | YAML manifest declaring an agent capability | "Tool manifest" / "Function spec" |
| **Repoctl** | Internal CLI for everyday engineering tasks | "Developer CLI" |
| **Ecosystem-as-a-Service (EaaS)** | The cohesion thesis (ADR-0001) | (industry uses "Platform-as-a-Service" / "Vertical SaaS"; EaaS is Oyatie-shaped) |
| **Team** | Coordinated multi-worker work bundle (replaces "CUG" / "Closed-User-Group" retired 2026-05-09) | "Cross-functional team" / "Pod" / "Squad" |

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

### Retired terms appendix (forensic only; kept never removed)

| Old | New | Reason |
|---|---|---|
| Pre-directive brand aliases | Oyatie | Brand standardization (ADR-0017) |
| `oyatie-*` Cargo prefix | `oya-*` | ADR-0015 + ADR-0017 |
| shell (UI) | Bench | per legacy ADR-0017 |
| CUG / Closed-User-Group | Team | 2026-05-09 |
| M0 / M1 / M2 / M3 / MVP | Wave names (W-Foundation, etc.) | ADR-0016 |
| Postmortem (long-form) | mistakes-and-fixes-ledger entry | per `docs/MISTAKES-LEDGER.md` |
| `repoctl pre-push` (slash command) | `repoctl check` | per CLAUDE.md sweep |

### CI lane: `oya-foundry-fitness-glossary`

The lane runs on every PR touching markdown / Rust source / catalog records / capability records and:

1. Detects forbidden tokens (M0..M3, MVP, milestone-zero, milestone-one, CUG, Closed-User-Group, `oyatie-*` prefixes outside the allowed alias window per ADR-0017).
2. Warns on inconsistent capitalization (`oyatie` vs `Oyatie`, `oYa` vs `OYA`).
3. Warns on industry-term inconsistency (e.g. `service-level objective` vs `Service Level Objective` — pick one per the GLOSSARY).
4. Warns on uncited acronyms (every acronym in the doc must appear in GLOSSARY §10).
5. Hard-fails on forbidden vocab; soft-warns on inconsistencies until cleanup PR.

### Boundary

- Applies to: every consolidated doc, ADR, PRD, RUNBOOK, README, per-team CHARTER, capability record, catalog record, marketing copy, customer comms template.
- Does not apply to: legacy ADRs (forensic), comments inside code (where local technical idioms apply), filesystem paths.

---

## Consequences

### Positive

- Vocabulary becomes auditable; "what does X mean?" is a glossary lookup.
- Industry alignment lowers onboarding cost for new contributors and external auditors.
- KR-EN parity table makes regulator-facing comms direct.
- Forbidden-vocab enforcement closes recurring drift (M0..M3 vocabulary, CUG, brand inconsistency).

### Negative

- Initial sweep cost is real (every legacy doc gets a glossary review).
- Per-PR vocabulary review may slow some PRs; mitigation: warn-mode for inconsistencies during the sweep window.

### Operational

- On-call: `EVT-GLOSSARY-LANE-DENY-RATE > N` weekly summary to council.
- Runbooks: `runbooks/glossary-amendment-pr.md`, `runbooks/term-deprecation-protocol.md`.
- CI: `oya-foundry-fitness-glossary` is a P1 lane (warns by default; promotes to BLOCK per ADR-0017 alias sunset).
- Per-quarter audit: glossary regenerated from machine-readable mirror at `machine-readable/glossary.json`.

---

## Alternatives considered

### Alternative A — No central glossary

- **Pros:** zero centralization cost.
- **Cons:** vocabulary chaos demonstrated; cross-axis review burdened.
- **Rejected because:** cohesion.

### Alternative B — Per-axis glossaries

- **Pros:** axis autonomy.
- **Cons:** "tenant" means three things in three axes — exactly the failure mode this ADR exists to prevent.
- **Rejected because:** ADR-0001.

### Alternative C — Industry-only (no Oyatie-specific terms)

- **Pros:** zero new vocabulary.
- **Cons:** Object Graph, Autonomy ceiling, Plane gate, Foundation bypass don't have industry analogs at the right granularity.
- **Rejected because:** Oyatie does ship genuinely new concepts.

---

## Open questions

1. **Q1.** Should `Foundry` be renamed to avoid LEDG-018 collision? Default: keep with differentiation rationale; council ratifies. → council.
2. **Q2.** Per-locale glossary fragments (KR-only, JP-only, EU-only) — separate sub-glossaries or unified? Default: unified canonical; per-locale customer-facing text generated from canonical via translation pack. → ADR-0010.
3. **Q3.** Customer-facing vocabulary (e.g. tenant-admin UI) — does the lane apply? Default: yes for catalog-tagged customer surfaces. → ADR-0011.
4. **Q4.** Machine-readable mirror (`machine-readable/glossary.json`) — auto-generated from this ADR's tables, or hand-curated? Default: auto-generated. → ADR-0019.

---

## References

- `docs/GLOSSARY.md` (full glossary — illustrative tables in this ADR cite §1, §8, §9, §10, §11)
- `docs/CONTRADICTION-LEDGER.md` LEDG-018 (Foundry naming), LEDG-019 (Search brand), LEDG-020 (Quant repo)
- `docs/PRD.md` §3.1 vocabulary update
- ADR-0001 (cohesion), ADR-0011 (catalog as a generated source), ADR-0015 (Cargo prefix), ADR-0016 (forbidden M0..M3/MVP vocab), ADR-0017 (brand canon), ADR-0019 (machine-readable mirror cadence)
- DDD canon (Eric Evans, Vaughn Vernon, Alistair Cockburn), Google SRE workbook, AWS Well-Architected (cell architecture / shuffle sharding), Diátaxis (https://diataxis.fr/), Anthropic MCP spec
