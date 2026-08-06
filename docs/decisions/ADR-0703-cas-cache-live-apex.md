---
id: ADR-0703
title: "Live CAS/cache policy (activation-gated RE remains fail-closed)"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-06
door: two-way
owner: council-architecture
supersedes: [ADR-0038, ADR-0106, ADR-0172, ADR-0310, ADR-0377, ADR-0556]
superseded_by: []
amends: []
amended_by: []
depends_on: []
related: []
milestone: W0
deliverables:
  - id: ADR-0703-D1
    description: "Live apex source-of-truth for topic cas_cache: Live CAS/cache policy (activation-gated RE remains fail-closed)."
    exit_criteria: "docs/decisions/ADR-0703-cas-cache-live-apex.md is Accepted with planning_impact true; member ADRs listed in supersedes are archived under docs/adr-archive/."
    verified_by: "oya-ci-required"
---
# ADR-0703: Live CAS/cache policy (activation-gated RE remains fail-closed)

## Status

**Accepted** — live consolidated source-of-truth entry for topic `cas_cache` (E5 2026-08-06).

## Context

Oyatie ADR corpus cleanup: agents must not treat every historical Accepted file as equal live law.
This apex consolidates **6** Accepted ADRs in the `cas_cache` topic. Member files are
**Superseded** by this apex and then archived; full text remains in git history.

Live resolution: prefer this apex; follow `supersedes` for provenance.

## Decision

1. **This ADR is the live reading entry** for topic `cas_cache` under the end-state ADR policy.
2. **Member ADRs listed in `supersedes`** are historical; normative gist is preserved below.
3. **Contradictions** among members are resolved by later higher-number members and by
   ADR-0515 / ADR-0363 / ADR-0562 / ADR-0615 / ADR-0635 / ADR-0637–0639 when applicable.
4. **Activation-sensitive** items (warm CAS, RE workers) remain fail-closed until explicit go-gate.

## Preserved member gists

- **ADR-38** (ADR-0038-trust-framework-and-dsr-cascade-and-proof-of-erasure): We adopt the **trust framework** as the cross-microservice lineage + DSR cascade + proof-of-erasure spine. Per-tenant trust portal is the customer-visible surface; cross-microservice lineage tracks per-data-class flow across all all microservices; DSR cascade walks the lineage on every request; per-store proof-of-erasure is Cosign-signed and audit-
- **ADR-106** (Rename `application` layer to `usecase` (amends ADR-0105)): Rename the `application` layer to **`usecase`**. The `app` layer is unchanged. The canonical enum size stays at 13 (per ADR-0105); only the spelling changes. ### Why `usecase` - **Clean Architecture canonical name.** Uncle Bob's "Clean Architecture" book names the port-only orchestration ring "Use Cases." The new name borrows from a 30-year-old, we
- **ADR-172** (ADR-0172-cqrs-read-replicas): Oyatie adopts a per-bounded-context CQRS split for THREE specific high-read BCs at M02 graduation; all other BCs remain single-primary Postgres. The CQRS split per BC: ### Pattern: command-side primary + query-side read replicas ``` ┌─────────────────────────┐ writes ────▶│ Postgres primary │── async repl ───┐ (HTTP POST) │ (command-side) │ │ └────
- **ADR-310** (ADR-0310-investigation-case-management): ### §B. Investigation case-management substrate Establish `microservices/detection/case-management/` (subdirectory of detection µservice — case-management is a closely-coupled companion primitive to detection substrate; same µservice flat layout per ADR-0131) exposing seven substrate primitives: 1. **Case lifecycle** — six-phase canonical workflow 
- **ADR-377** (ADR-0377-github-board-git-ref-cas-fallback): Use **GitHub Issues + exclusive scoped labels** as the human/audit board projection, and use **plain git refs as the concurrency lock**. ### 1. Board projection `oya gen board-sync` reads `/specs/masterplan.json` deliverables and emits an idempotent diff against GitHub Issues: - one issue per deliverable; - stable issue identity from the deliverabl
- **ADR-556** (Build cache-warmth classification: deliberate cold/warm policy-as-data + the col): ### D1 — The cache-warmth classification is POLICY-AS-DATA (R0 pack-shape) The classification ships as `/specs/cache-warmth-policy.json` — born pack-shaped per the ADR-0548 paved-road rule and the ADR-0544/ADR-0546 precedent: every build class maps to `{warmth: cold|warm, cache_read: bool, cache_write: bool, reason}`, all repo-specifics live in the

## Consequences

- Agent default read path: `docs/decisions/ADR-0xxx` apex files + this topic.
- Citations to member numbers remain valid via `docs/decisions/_disposition/adr-redirect.v1.json`.
- Further body merge refinements may land as amendments to this apex only.

### ADR-38 residual

**ADR-0038-trust-framework-and-dsr-cascade-and-proof-of-erasure** — We adopt the **trust framework** as the cross-microservice lineage + DSR cascade + proof-of-erasure spine. Per-tenant trust portal is the customer-visible surface; cross-microservice lineage tracks per-data-class flow across all all microservices; DSR cascade walks the lineage on every request; per-store proof-of-erasure is Cosign-signed and audit-chained. ### Cross-axis trust framework ```rust //

### ADR-377 residual

**ADR-0377-github-board-git-ref-cas-fallback** — Use **GitHub Issues + exclusive scoped labels** as the human/audit board projection, and use **plain git refs as the concurrency lock**. ### 1. Board projection `oya gen board-sync` reads `/specs/masterplan.json` deliverables and emits an idempotent diff against GitHub Issues: - one issue per deliverable; - stable issue identity from the deliverable id, not from issue number; - labels for status,

### ADR-106 residual

**Rename `application` layer to `usecase` (amends ADR-0105)** — Rename the `application` layer to **`usecase`**. The `app` layer is unchanged. The canonical enum size stays at 13 (per ADR-0105); only the spelling changes. ### Why `usecase` - **Clean Architecture canonical name.** Uncle Bob's "Clean Architecture" book names the port-only orchestration ring "Use Cases." The new name borrows from a 30-year-old, well-known model. - **Visually + semantically distin

### ADR-172 residual

**ADR-0172-cqrs-read-replicas** — Oyatie adopts a per-bounded-context CQRS split for THREE specific high-read BCs at M02 graduation; all other BCs remain single-primary Postgres. The CQRS split per BC: ### Pattern: command-side primary + query-side read replicas ``` ┌─────────────────────────┐ writes ────▶│ Postgres primary │── async repl ───┐ (HTTP POST) │ (command-side) │ │ └─────────────────────────┘ ▼ ┌──────────────────────┐

### ADR-310 residual

**ADR-0310-investigation-case-management** — ### §B. Investigation case-management substrate Establish `microservices/detection/case-management/` (subdirectory of detection µservice — case-management is a closely-coupled companion primitive to detection substrate; same µservice flat layout per ADR-0131) exposing seven substrate primitives: 1. **Case lifecycle** — six-phase canonical workflow per §A.1 2. **Triage scorer** — per-signal priorit

### ADR-556 residual

**Build cache-warmth classification: deliberate cold/warm policy-as-data + the cold integrity-canary trust anchor** — ### D1 — The cache-warmth classification is POLICY-AS-DATA (R0 pack-shape) The classification ships as `/specs/cache-warmth-policy.json` — born pack-shaped per the ADR-0548 paved-road rule and the ADR-0544/ADR-0546 precedent: every build class maps to `{warmth: cold|warm, cache_read: bool, cache_write: bool, reason}`, all repo-specifics live in the DATA, and downstream consumers (interim CI quick-
