---
id: ADR-0239
status: Superseded
superseded_by: [ADR-709]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0239: Foundry Scope Clarification (Internal-Only Amendment)

- **Status:** Accepted (amendment)
- **Date:** 2026-05-18
- **Owner:** council-architecture
- **Amends:** ADR-0136 (Foundry consolidation 6→1)
- **Related:** ADR-0220 (Consumer Intelligence Substrate — `microservices/intelligence/`)
- **PR:** #143 close-out

## Context

ADR-0136 consolidated six prior foundry candidates into a single `microservices/foundry/` µservice. Subsequent session work (PR #142–#143) drifted on Foundry's audience:

- Some agent dispatches assumed Foundry hosts CONSUMER AI surfaces ("Foundry-powered 1:1 prep", "Foundry HR Q&A agent", "Foundry management-cockpit insights").
- Reality: Foundry is the INTERNAL retired external agent harness agentic development pipeline. Consumer-facing AI is a SEPARATE substrate.

User directive 2026-05-18: "Foundry is internal tool. oyatie intelligence + ontology + workflow is consumer facing." This amendment codifies the split so future agent dispatches don't conflate.

## Decision (amendment)

**`microservices/foundry/` is INTERNAL only.** Its surfaces serve:

1. retired external agent harness agentic development toolchain (agent dispatch, eval, gates).
2. CI/CD orchestration (build, deploy, evidence collection).
3. Internal eval substrate (multispectrum review, A-family adherence facets).
4. Internal evidence collection (audit chain integration for build provenance).

Consumer-facing AI is a SEPARATE µservice **`microservices/intelligence/`** (canonical pathing per ADR-0220). The brand label shown to users is "oyatie intelligence" — similar to Apple Intelligence / Microsoft Copilot / Google AI.

### Concrete impact on past references

| Past framing (drifted) | Corrected framing |
| --- | --- |
| "Foundry agent for 1:1 prep" | "oyatie intelligence 1:1-prep agent" |
| "Foundry HR Q&A" | "oyatie intelligence HR Q&A" |
| "Foundry-powered customer feature X" | "oyatie intelligence feature X (consumer surface)" |
| "Foundry eval substrate" | KEEP — internal eval IS Foundry |
| "Foundry CI gate" | KEEP — CI orchestration IS Foundry |
| "Foundry agentic dev pipeline" | KEEP — retired external agent harness IS Foundry |

### Substrate-sharing

Foundry and Intelligence share substrate but partition by µservice + cell + tenant boundary:

| Substrate | Foundry use (INTERNAL) | Intelligence use (CONSUMER) |
| --- | --- | --- |
| Milvus (ADR-0192) | Internal eval corpora | Per-tenant RAG; per-cell logical isolation |
| Wasmtime (ADR-0200) | Internal tool sandboxing | Tenant context sandboxing |
| Cedar (ADR-0150) | Internal agent permissions | Tenant-scoped consumer AI access |

The shared runtime is intentional (Karpenter pool fungibility per ADR-0198); the cell + µservice + tenant boundaries are NOT.

## In-house roadmap

Both Foundry AND Intelligence are Class C in-house mandatory per ADR-0211 — they are the differentiation. No vendor replacement path. Day-one in-house.

## Alternatives considered

### Alternative 1 — "Single µservice serves both internal + consumer"

**Rejected because** the audiences are fundamentally different. Internal agents have access to source code + CI secrets + evidence chain; consumer agents have access to per-tenant data + consent-graph slices. Co-locating creates audit-chain leakage risk + Cedar policy complexity that does not justify the cost saving.

### Alternative 2 — "Rebrand Foundry to include consumer scope"

**Rejected because** the naming "Foundry" (Palantir-class internal product platform) is established for the internal retired external agent harness / dev / eval surface. Renaming would create churn across 100+ ADR / IP citations.

### Alternative 3 — "Keep current drift; tolerate ambiguous framing"

**Rejected because** drift compounds. Each agent dispatch under the wrong framing reinforces the mistake; the auditable taxonomy (which audience does this surface serve?) becomes muddled.

## Consequences

### Positive

- **Clean audience model.** Every AI surface declares INTERNAL or CONSUMER. Cedar policy maps cleanly.
- **No accidental data leakage.** Internal Foundry agents never receive tenant data; consumer Intelligence agents never receive source/CI secrets.
- **Predictable for contributors.** Adding a new AI surface, the contributor asks "is this for our engineers or for tenants?" and routes accordingly.

### Negative

- **Past-PR drift cleanup.** Some prior commits cite "Foundry agent for $consumer-feature"; sweep-and-rename pass queued.
  - **Mitigation:** `oya-check-canonical-glossary-compliance` (queued per ADR-0221 §M-03) detects the drift pattern; sweep PR is PR-144 Wave 3A.

### Operational

- **Naming sweep.** `microservices/oyatie-intelligence/` (incorrect; legacy brand prefix violates naming) → `microservices/intelligence/` (correct; brand label "oyatie intelligence" shown to users only).
- **Manifest `audience` field.** Every µservice manifest declares `audience: INTERNAL | B2B-tenant | B2C-consumer | DEVELOPER` per ADR-0221 §M-04 (queued).

## References

- ADR-0136 — Foundry consolidation 6→1 (this amendment clarifies scope).
- ADR-0150 — Cedar policy engine.
- ADR-0192 — Milvus vector DB (shared substrate).
- ADR-0200 — Wasmtime substrate (shared sandbox runtime).
- ADR-0211 — In-house tech stack policy (Foundry + Intelligence both Class C).
- ADR-0220 — Consumer intelligence substrate (`microservices/intelligence/`).
- ADR-0221 — Agentic dev pipeline hardening (§M-03 canonical glossary; §M-04 audience-of-µservice field).

## Named industry sources

- Apple Intelligence — consumer-AI brand pattern; lives across consumer products as a brand surface.
- Microsoft Copilot — consumer-AI brand; M365 + GitHub + Windows surfaces.
- Google AI — consumer-AI brand under one umbrella.
- Palantir Foundry — internal platform tooling brand; serves dev + eval + ops audiences.
- AWS internal CI vs. AWS consumer products — same audience split.
