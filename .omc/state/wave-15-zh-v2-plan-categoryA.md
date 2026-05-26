# Wave 15-ZH-v2 Category A delete plan

Scope guard: no `docs/decisions/ADR-*.md` deletion, no ADR renumbering, no ADR moves. `.omc/state/wave-15-zh-*` audit files are preserved.

## Delete files (16)

- `docs/decisions/RETIRED.md` — tombstone marker file
- `docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md` — tombstone runbook marker for retired external coordination demo
- `docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.sh` — tombstone script marker for retired external coordination demo
- `docs/runbooks/grit-session-bug-upstream.md` — tombstone runbook marker for retired external coordination workaround
- `specs/microservices/network.json` — spec status=Retired title='RetiredSpec:ConnectNetwork'
- `specs/microservices/shorts.json` — spec doc_class=RetiredMicroserviceMarker
- `registry/capability-tiers/bronze.json` — capability-tier registry file; tier doctrine retired per ADR-0329
- `registry/capability-tiers/checkpoint.json` — capability-tier registry file; tier doctrine retired per ADR-0329
- `registry/capability-tiers/gold.json` — capability-tier registry file; tier doctrine retired per ADR-0329
- `registry/capability-tiers/index.json` — capability-tier registry file; tier doctrine retired per ADR-0329
- `registry/capability-tiers/microservice-tier-mapping.yaml` — capability-tier registry file; tier doctrine retired per ADR-0329
- `registry/capability-tiers/platinum.json` — capability-tier registry file; tier doctrine retired per ADR-0329
- `registry/capability-tiers/silver.json` — capability-tier registry file; tier doctrine retired per ADR-0329
- `registry/capability-tiers/vendor-tier-mapping.yaml` — capability-tier registry file; tier doctrine retired per ADR-0329
- `docs/standards/capability-authoring.md` — docs/standards/capability-*.md standard doc in v2 delete scope
- `docs/standards/capability-tier-matrix.md` — docs/standards/capability-*.md standard doc in v2 delete scope

## Delete directories (3)

- `microservices/cell/` — retired microservice marker-only directory; sampled maxdepth files show only RETIRED.md
- `microservices/network/` — retired microservice marker-only directory; sampled maxdepth files show only RETIRED.md
- `microservices/shorts/` — retired microservice marker-only directory; sampled maxdepth files show only RETIRED.md

## Per-microservice capability-tier directories

- None found in the live tree during v2 re-audit: `find microservices -path '*/capability-tiers' -type d` returned zero rows.

## Explicit keeps / exclusions (2)

- `microservices/intelligence/RETIRED.md` — KEEP: RETIRED.md inside substantive microservice directory; sampled files: microservices/intelligence/ARCHITECTURE.md, microservices/intelligence/AUDIT-FINDINGS-2026-05-18.json, microservices/intelligence/IP-001-runtime-runtime-cluster-iac.md, microservices/intelligence/IP-002-runtime-redis-and-postgres-baseline.md
- `specs/microservices/intelligence.json` — KEEP: retired-shaped but outside v2 delete predicate (doc_class='Microservice-Retirement-Marker', status='Retired', title='Microservice:Foundry — RETIRED')
