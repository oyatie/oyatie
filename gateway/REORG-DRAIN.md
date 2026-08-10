# gateway/ reorg drain notes (`integ/gateway`)

## Ownership

- **Forever home:** `gateway/**` (this rail).
- **Writes this tip:** capability-root `gateway/manifest.json` enrichment + this drain file only.
- **Absorbed product face (still nested):** `gateway/connector/` (registry absorb list).

## Completed (this rail)

- Interior path hygiene (Seat A follow-through): retargeted verified `microservices/api-gateway/**` and `microservices/connector/**` cites under `gateway/**` (capabilities, catalog, IPs, runbooks, connector absorb face, README tree label) only where destinations exist; missing historical assets remain legacy cites.
- Seat A interior prep: rewrite `gateway/manifest.json` to capability-root shape: `capability` key, registry stratum **S0** (dag_node=api-gateway), verified `gateway/**` capability/contract/OpenSLO paths, ten-crate adapter accounting + absorbed connector pointer.
- Path hygiene: dropped missing `microservices/api-gateway/contracts/metric-naming-convention.md` convention_docs cite; retained verified capability YAML + OpenSLO under `gateway/`.
- Catalog rows verified present under `registry/catalog/gateway-*-connector.yaml` (cited, not edited — outside `gateway/**`).
- Seat A follow-up: remapped verified `microservices/api-gateway/` and `microservices/connector/` interior cites across `gateway/**` (capabilities, catalog, IPs, connector face, runbooks, README layout) to existing `gateway/` destinations only.

## Stale refs found (next gaps, ordered)

1. **Missing historical IP markdown** (16 cites) — still referenced by pre-move microservices/api-gateway paths; do not invent. Sample:
- `IP-001` → `microservices/api-gateway/IP-001-api-gateway-design-readiness.md`
- `IP-002` → `microservices/api-gateway/IP-002-routing-domain-crate.md`
- `IP-003` → `microservices/api-gateway/IP-003-routing-kernel-crate.md`
- `IP-004` → `microservices/api-gateway/IP-004-routing-usecase-crate.md`
- `IP-005` → `microservices/api-gateway/IP-005-routing-adapter-crate.md`
- `IP-006` → `microservices/api-gateway/IP-006-routing-rest-crate.md`
- `IP-007` → `microservices/api-gateway/IP-007-routing-grpc-crate.md`
- `IP-008` → `microservices/api-gateway/IP-008-routing-worker-crate.md`
- `IP-009` → `microservices/api-gateway/IP-009-rate-limit-domain-crate.md`
- `IP-010` → `microservices/api-gateway/IP-010-rate-limit-adapter-valkey.md`
- `IP-011` → `microservices/api-gateway/IP-011-auth-handoff-usecase.md`
- `IP-012` → `microservices/api-gateway/IP-012-abuse-defence-domain.md`
- `IP-013` → `microservices/api-gateway/IP-013-abuse-defence-adapter-wasm.md`
- `IP-014` → `microservices/api-gateway/IP-014-tls-cert-rotation-worker.md`
- `IP-015` → `microservices/api-gateway/IP-015-canary-cohort-shifter.md`
- `IP-016` → `microservices/api-gateway/IP-016-app-supervisor.md`
2. **`gateway/connector/manifest.json`** — verified in-tree contract/capability/runbook cites remapped; missing connect-retirement / eval / PRD / threat-model cites remain deferred.
3. **North-south edge crates absent** — no `gateway/core|ports|facade` Cargo packages yet; edge behavior is capability YAML + OpenSLO only. Land kernels before claiming runtime readiness.
4. **OpenSLO extras** — additional files under `gateway/observability/slos/` (connector-*, dlq-*, oauth-*, webhook-*) exist beyond the seven claimed root `slos[]`; decide promotion vs product-face ownership.

## Out of envelope (do not touch from `integ/gateway`)

- `specs/**`, `Cargo.lock`, registry/catalog edits, merge/land.
- Shrink-only deletes of residual `microservices/api-gateway` mirrors — owning shrink rails only.
