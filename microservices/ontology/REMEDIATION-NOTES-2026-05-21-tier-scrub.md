# REMEDIATION-NOTES-2026-05-21-tier-scrub

## Files DELETED

- `performance-benchmark-numbers-2026-05-20.md` - stale Wave-4 benchmark artifact; canonical commitments belong under `slos/` and benchmark evidence.
- `coherence-audit-2026-05-20.md` - stale Wave-4 audit output with retired tier vocabulary.

## Files RETAINED with scrub

- `ARCHITECTURE.md`
- `onboarding/ontology-engineer-first-week.md`
- `decisions/ADR-ONT-001-rdf-shape-vs-property-graph-storage.md`
- `manifest.json`
- `policy/type-isolation.md`
- `policy/tenant-scope.cedar`
- `policy/auditor-scope.cedar`
- `threat-model.md`
- `capabilities/query-execute.yaml`
- `capabilities/cedar-evaluate.yaml`
- `IP-008-function-engine-oltp-and-olap.md`
- `feature-parity-matrix-2026-05-20.md`
- `contracts/openapi/ontology.yaml`

Scrub rationale: observability/test "golden" wording was normalized to reference vocabulary, and `max_tier` / `property_tier` were renamed to sensitivity-level fields because the policy intent is data sensitivity, not commercial capability tiering.

## Counterpart-fact preservations

- None.

## Wave 15-IP-substance scrub (2026-05-21)

- Assigned bucket: IP-BUCKET-J / Wave 15-IP-substance.
- Rewritten in place: 18 stamped or short-shell IPs.
- Preserved as already-substantive with counterpart evidence pointer where needed: 35 IPs.
- Deleted as duplicative: 0. No pair was merged because the apparent duplicate journey names carry different journey IDs or regulatory overlays.
- Source grounding used: `microservices/ontology/PRD.md`, `microservices/ontology/ARCHITECTURE.md`, `microservices/ontology/competitor-parity-matrix.md`, service `manifest.json`, contracts, policy, SLO, catalog, runbook, and IaC artifacts. No nonexistent `src/` paths were invented; these three assigned services have no `microservices/<ms>/src` tree in this checkout.
- Rewritten files:
  - `microservices/ontology/IP-001-ontology-iac-stack.md`
  - `microservices/ontology/IP-002-object-type-registry-kernel-domain.md`
  - `microservices/ontology/IP-003-link-action-function-type-registry.md`
  - `microservices/ontology/IP-004-entity-store-rls-citus.md`
  - `microservices/ontology/IP-005-link-store-traversal.md`
  - `microservices/ontology/IP-006-cedar-fragment-coverage-engine.md`
  - `microservices/ontology/IP-007-action-engine-cedar-gated.md`
  - `microservices/ontology/IP-008-function-engine-oltp-and-olap.md`
  - `microservices/ontology/IP-009-clickhouse-history-mirror.md`
  - `microservices/ontology/IP-010-audit-chain-merkle-ed25519.md`
  - `microservices/ontology/IP-011-query-engine-3layer-kg.md`
  - `microservices/ontology/IP-012-agent-gateway-llm-tool-call.md`
  - `microservices/ontology/IP-013-pillar-cross-pillar-grant.md`
  - `microservices/ontology/IP-014-rest-and-sdk-surfaces.md`
  - `microservices/ontology/IP-015-app-binaries-and-branch-protection.md`
  - `microservices/ontology/IP-016-read-path-library-rollout.md`
  - `microservices/ontology/IP-017-share-token-surface.md`
  - `microservices/ontology/IP-018-abuse-defence-edge-wiring.md`
- Follow-up: implementation PRs must create the declared crates/types before claiming cargo-test evidence; this scrub only converts IP documentation from stamp to service-grounded plan content.

