---
adr_id: ADR-SDK-0002
title: "Codegen pipeline is deterministic; two runs on identical input produce byte-identical SDK files"
status: Proposed
date: 2026-05-18
microservice: developer-sdk
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0211]
doc_status: published
---

# ADR-SDK-0002: Codegen pipeline is deterministic; two runs on identical input produce byte-identical SDK files

## Status

Proposed — 2026-05-18.

## Context

Scoped to developer-sdk µservice substrate. Codegen pipeline is deterministic; two runs on identical input produce byte-identical SDK files.

## Decision

Tera template engine with deterministic context; sorted dict iteration; pinned dependency versions; no timestamps in generated files (or pinned to spec version date). CI lane verifies determinism every PR.

## Alternatives considered

### Best-effort determinism (REJECTED)

Downstream consumer builds break on phantom diffs; supply-chain risk.

## Consequences

Deterministic codegen unblocks reproducible builds + supply-chain attestation per SLSA L3.

## References

- ADR-0213 (parent EaaS architecture)
- microservices/developer-sdk/PRD.md
