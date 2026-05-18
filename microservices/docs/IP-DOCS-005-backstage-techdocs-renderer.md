# IP-DOCS-005 — Backstage TechDocs renderer adapter

> ADR anchor: ADR-0203, ADR-0170.
> Owner: `oya-docs`.
> Estimate: 3 days.

## Goal

Wrap Backstage TechDocs behind an adapter so a Phase-2
in-house `oya-developer-portal` (ADR-0203 trigger-conditional)
can swap in without rewriting service catalog integration.

## Why this IP

ADR-0203 §"In-house roadmap" names `oya-developer-portal` as
a Phase-2 conditional build. To preserve the option, the
substrate keeps TechDocs behind an adapter rather than baking
TechDocs assumptions into business logic.

## Tasks

### 1. Adapter trait

- `DocsCatalogAdapter` trait declares the operations
  Backstage TechDocs supplies (catalog query, doc render,
  search).

### 2. TechDocs implementation

- `BackstageTechDocsAdapter` implements the trait against
  the Backstage HTTP API.

### 3. Tests

- Integration test against a local Backstage instance.

## Acceptance criteria

- Every consumer of docs catalog data goes through the trait.
- Swapping the adapter (Phase 2) requires no consumer
  changes.

## References

- ADR-0203, ADR-0170.
