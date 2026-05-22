---
doc_class: Phase
shape: Reference
phase: PHASE-02-PACK-OVERLAY-BREACH-DPIA-BUILD
status: planned
related_adrs: [ADR-0209, ADR-0251, ADR-0263, ADR-0294]
companion_docs:
  - microservices/compliance/PHASE-01-EVIDENCE-PIPELINE-BOOTSTRAP.md
  - microservices/compliance/PRD.md
---

# PHASE-02 — Pack overlay registry + breach-notification + DPIA build

## A. Why

PHASE-01 landed the evidence pipeline + DSAR automation. PHASE-02 closes the substrate by
adding pack-overlay registry, breach-notification workflow, DPIA orchestration,
cell-certification attestation, and compliance-control mapping. Hyperscaler precedent:
OneTrust + AuditBoard + Vanta full-suite.

## B. Scope (in)

- IP-016 — pack-registry kernel.
- IP-017 — pack-registry domain.
- IP-018 — DPIA orchestration usecase.
- IP-019 — breach-notification workflow.
- IP-020 — regulator-audit-evidence REST.
- IP-021 — cell-certification-attestation worker.
- IP-022 — compliance-control-mapping domain.
- IP-023 — pack-registry gRPC surface.
- IP-024 — DPIA Postgres adapter.
- IP-025 — breach-notification AsyncAPI emitter.
- IP-026 — control-mapping REST + SDK.

## C. Scope (out)

- Tenant self-service auditor portal UI (PHASE-03).
- Cross-tenant attestation aggregator (handled in tenancy + ontology).

## D. Acceptance

- All IPs IP-016..IP-026 GA.
- `dashboards/pack-overlay-coverage.json`, `breach-notification-sla.json`,
  `regulator-engagement-activity.json` live.
- SLOs `pack-publish-soak-respected`, `breach-notify-authority-72h` green for 30d.
- Cedar fragments soaked ≥60s + ADR-0294 conformance.
- ECH + PQC offered on all internet-facing surfaces.

## E. Rollback

Per-IP rollback per `IP-xxx-*.md` "Failure modes" section.

## F. References

- ADR-0209 — compliance evidence pipeline
- ADR-0251 — compliance packs
- ADR-0294 — Cedar fragment soak
- PHASE-01 — evidence pipeline bootstrap
