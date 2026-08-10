# network/ reorg drain notes (`integ/network`)

## Ownership

- **Forever home:** `network/**` (this rail).
- **Writes this tip:** capability-root `network/manifest.json` enrichment (+ this drain file).
- **Absorbed product face (still nested):** `network/dns/` (listed under `capability_root_accounting.absorbed_services`).

## Completed (this rail)

- Seat A interior prep: rewrite `network/manifest.json` to capability-root shape (observability@6dcdf9b08 pattern): `capability` key, registry stratum **S1**, verified `network/**` crate/contract/capability/OpenSLO cites, eight-crate accounting, and absorbed `dns` pointer.
- Path hygiene on the **root** manifest: remapped stale `crates/oya-cloud-network-*` cites → live `network/{core,ports,adapters}/**`; OpenAPI cites → `network/contracts/openapi/cloud/**` (+ dns OpenAPI under `network/dns/contracts/`); OpenSLO cites → `network/observability/slos/**` with live_exempted create-API SLI rationale.
- Catalog rows verified present under `registry/catalog/network-*.yaml` (cited, not edited — outside `network/**`).

## Stale refs found (next gaps, ordered)

1. **`network/dns/manifest.json`** — nested absorb face may still carry pre-move path cites; remap in a follow-up interior slice.
2. **Historical audit/parity docs** under `network/**` (e.g. `ARCH.md`, `coherence-audit-*`, `feature-parity-matrix-*`) still narrate `microservices/cloud-network*` / `crates/oya-cloud-network-*` dump paths. Retarget only when a verified in-tree destination exists; do not invent files.
3. **Dual OpenAPI homes** — `network/contracts/openapi/cloud/cloud-network-*-v1.yaml` vs root `contracts/openapi/cloud/cloud-network-*-v1.yaml` (if present). Root copies are outside this envelope; converge on a contracts-aware rail.

## Out of envelope (do not touch from `integ/network`)

- `specs/**`, `Cargo.lock`, registry/catalog edits, merge/land.
- Shrink-only deletes of residual `microservices/*` or `crates/oya-cloud-network-*` mirrors — owning shrink rails only.
