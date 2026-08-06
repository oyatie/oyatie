---
id: ADR-0060
status: Superseded
superseded_by: [ADR-709]
doc_status: published
---

# ADR-0060: Bominal-inheritance precedence — default inherit Bominal ADRs; session decisions override

> **Status:** Accepted
> **Owner:** `council-architecture`
> **Date:** 2026-05-13
> **Related:** ADR-0001, ADR-0055, ADR-0056, ADR-0058, ADR-0059, ADR-0061, ADR-0062

---

## Context

Oyatie is the same product as Bominal, built in parallel (user instruction 2026-05-13: "oyatie is just working in parallel. same product as bominal."). Bominal has an extensive set of architectural ADRs (ADR-0011 through ADR-0232+). Without an explicit precedence rule, every oyatie agent or author must re-derive decisions that Bominal has already made.

User instruction 2026-05-13: "we inherit bominal default but for anything we've discussed in this session it overrides bominal."

**Naming justification:** "Bominal-inheritance" describes the default adoption of Bominal's architectural decisions with translation to oyatie's glossary.

---

## Decision

Two-tier precedence for all architectural decisions in oyatie:

1. **Default (lower precedence):** Adopt Bominal ADR architecture decisions 1:1, translating Bominal terminology to oyatie canonical glossary (per ADR-0018).
2. **Override (higher precedence):** Anything decided in the 2026-05-13 /deep-interview session overrides Bominal when they conflict.

### Glossary translation table (always applied when reading Bominal)

| Bominal term | Oyatie term |
|---|---|
| "Object Graph" | "Ontology" |
| "Platform/Ops Arm" | "shared substrate" |
| "Arm" (any) | "microservice" or flat catalog entry |
| "Modular Product Shell" | "Application" |
| "Workspace" | "Connect" |
| `platform/*` directories | `shared/*` or specific µservice |
| `oya-platform-*` crates | `oya-<microservice>-*` per BNF v4.1 |

### Locked overrides (oyatie diverges from Bominal)

| # | Override | Bominal source | Oyatie decision | Memory |
|---|---|---|---|---|
| 1 | Workflow placement | Corporate-owned (Bominal ADR-0232 / family-workflow.md) | shared µservice; `oya-workflow-*` | `[[feedback-workflow-is-shared]]` |
| 2 | Object Graph naming | "Replaces Palantir Ontology terminology" (Bominal ADR-0106) | Renamed to **Ontology** (matches Palantir directly) | `[[feedback-glossary-ontology-not-object-graph]]`; ADR-0055 |
| 3 | Platform glossary | `platform/*` dirs + "Platform/Ops Arm" | `shared/*` glossary; `platform` retired | `[[feedback-glossary-shared-not-platform]]` |
| 4 | Arm grouping | "Arms" structure (Healthcare/Corporate/FinTech/Connect/Platform) | Flat microservice catalog; no grouping; Arms retired | `[[feedback-flat-product-catalog]]`; ADR-0058 |
| 5 | Product Groups | (would-be Arm equivalent in oyatie) | Retired; flat catalog only | ADR-0058 |
| 6 | BNF `shared\|vertical` slot2 | (no equivalent in Bominal) | Retired; slot2 = µservice name (open kebab); BNF v4.1 | ADR-0056 |
| 7 | Workspace product | `modules/workspace` (Bominal) | Workspace → dual-context per Bominal ADR-0208 model | `[[feedback-flat-product-catalog]]` |
| 8 | "Shell" terminology | "Modular Product Shell" (Bominal ADR-0121) | "Application" (capital A) | ADR-0061 |
| 9 | Sales segmentation | Architectural Arms | Sales labels (Healthcare/Enterprise/FinTech/Social) for GTM only, NOT architecture | `[[feedback-flat-product-catalog]]` |
| 10 | Workflow+Ontology centrality | (not explicit in Bominal) | Workflow + Ontology = THE ecosystem adapter layer; all inter-product flow through them | `[[feedback-workflow-objectgraph-adapter-layer]]`; ADR-0059 |

### Inherited from Bominal (1:1 with glossary translation)

| Bominal ADR | Decision inherited |
|---|---|
| ADR-0011 | Isolation-compatible operating model (customer × domain unit) |
| ADR-0017 | Unified governance catalog authority |
| ADR-0018 | Tenancy + RLS posture (JWT tenant_id claim + Postgres RLS) |
| ADR-0019 | Runtime target metadata model (catalog schema) |
| ADR-0020 | Multi-runtime platform standard (cluster/serverless/edge/vm) |
| ADR-0021 | OCI A1 Always Free launch profile |
| ADR-0028 | Audit-chain Merkle/Ed25519 |
| ADR-0100..ADR-0105 | Hexagonal + clean architecture |
| ADR-0106 | Ontology architecture (typed Object/Link/Action/Function; RLS; audit chain; jurisdiction overlays; plugin SDK; multi-renderer) — glossary: "Object Graph" → "Ontology" |
| ADR-0107 | Ontology agent gateway (LLM tool-call ingress) |
| ADR-0108..ADR-0112 | Property types (vector/geo/timeseries/ciphertext/struct) |
| ADR-0116 | Event streaming (outbox → Kafka KRaft at scale) |
| ADR-0117 | Cloud-native infrastructure architecture (OCI A1 → OKE stages) |
| ADR-0118 | Tenant activation + data import |
| ADR-0119 | Data tier assignment matrix |
| ADR-0120 | Platform-finance library → `oya-finance-library-*` |
| ADR-0121 | Modular product shell → "Application" (glossary override #8) |
| ADR-0123 | Cross-product auth cookie + redirect contract (two-cookie + PKCE + nonce) |
| ADR-0125 | Domain naming canon (Tenant / Organization / User / Person / Employee / Employment distinctions) |
| ADR-0126 | Employment classification (8 classes) |
| ADR-0127 | Sector × tier compliance packs |
| ADR-0128 + ADR-0190 | Versioned regulatory corpus (bominal-law / corpus.lock) |
| ADR-0132 | Data ownership pillars (org-pillar / person-pillar; Cedar enforcement) |
| ADR-0140 (retired per ADR-0145) | Multi-jurisdiction policy (per-jurisdiction Cedar overlays) |
| ADR-0208 | dual-context (Personal + Professional) |
| ADR-0209 | Client architecture (Leptos web + 5 native + SvelteKit prototype lane) |
| ADR-0210 | M03 KR group payroll + mail launch (= oyatie M03 first-paying-tenant target) |
| ADR-0215 | retention / legal hold / dual-context boundary |
| ADR-0223 | Proof Ladder L0..L7 |
| ADR-0224..ADR-0231 | 9 architecture planes |
| ADR-0232 | Wave integration framework |

### Operational rule

When authoring a new oyatie ADR, plan, doc, or code:

1. Check if the decision area has a Bominal ADR. If yes, default to Bominal.
2. Check the override list above. If the area is in the list, follow the oyatie override.
3. If divergence is needed in a new area, propose a new override + add it to this ADR.
4. Cite the source: "per Bominal ADR-#### (inherited)" or "per oyatie ADR-#### / `[[memory-slug]]` (override)".

---

## Consequences

### Quality / Performance / Scalability (per ADR-0062)

- Inherited Bominal ADRs include performance targets (ADR-0107 p99 targets, ADR-0117 cloud-native scaling stages, ADR-0028 audit-chain <1s segment-seal latency). These are binding in oyatie.
- `oya-check-benchmark-cli` verifies that any µservice graduating from Proof-Ladder L4→L5 has a PRD competitive-benchmark section citing at least one Bominal ADR equivalent or industry leader.
- Per `[[feedback-quality-performance-scalability-bar]]`: inherited Bominal perf targets are the floor; oyatie may set higher targets per-µservice.

### Positive

- Oyatie does not re-derive decisions Bominal has already made; large body of work inherited instantly.
- Override list is explicit; no implicit divergence.
- Glossary translation table eliminates ambiguity when reading Bominal docs.

### Negative

- Authors must read Bominal ADRs when exploring new decision areas; adds lookup step.
- Override list must be maintained; new session decisions must be explicitly added here.

---

## Related

- ADR-0001 (cohesion thesis)
- ADR-0055 (Object Graph → Ontology rename; override #2)
- ADR-0056 (BNF v4.1; override #6)
- ADR-0058 (Flat microservice catalog; overrides #4, #5, #9)
- ADR-0059 (Workflow + Ontology adapter layer; overrides #1, #10)
- ADR-0061 (Application; override #8)
- ADR-0062 (Quality/Performance/Scalability bar — Bominal perf ADRs inherited)
- `[[feedback-bominal-inheritance-precedence]]` — session decision 2026-05-13
