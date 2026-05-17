---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-014
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community + council-architecture
related_adrs: [ADR-0105, ADR-0126, ADR-0131, ADR-0132, ADR-0133]
doc_status: published
---

# IP-014 — Hyperscaler maturity gate HG-COMMUNITY

## Intent

Wire the HG-COMMUNITY hyperscaler-maturity claim gate. Per `feedback_quality_performance_scalability_bar.md`, every µservice claims parity vs. industry leaders and that claim is CI-enforced.

## Scope

- Parity matrix: see `competitor-parity-matrix.md`.
- Per-claim test: per row in the matrix, a CI test verifies the claim.
- Per-performance claim: synthetic benchmark.
- Per-policy claim: policy-as-code coverage.
- Quarterly review.

## Deliverables

- Parity-matrix CI lane.
- Per-claim test files at `tests/parity/`.
- Synthetic benchmark suite at `tests/perf/`.

## Acceptance

- HG-COMMUNITY gate returns GREEN.
- All parity rows have evidence pointers.
- Performance numbers in matrix match measured.

## Owner

axis-community + council-architecture.
