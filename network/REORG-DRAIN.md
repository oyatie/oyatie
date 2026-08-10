# network/ reorg drain notes (`integ/network`)

## Ownership

- **Forever home:** `network/**` (this rail).
- **Writes this tip:** capability-root `network/manifest.json` enrichment, interior verified path hygiene under `network/**`, absorbed-face `network/dns/manifest.json` path remaps, and this drain file.
- **Absorbed product face (still nested):** `network/dns/` (registry absorb list).

## Completed (this rail)

- Seat A interior prep: rewrite `network/manifest.json` to capability-root shape (messaging@`2d1c81693` / observability@`6dcdf9b08` / storage pattern): `capability` key, registry stratum **S1**, verified `network/**` crate/contract/capability/OpenSLO paths, eight-crate accounting + absorbed dns pointer.
- Path hygiene on root manifest + Rust crate comments: remapped stale `crates/oya-cloud-network-*` → live `network/{core,ports,adapters}/**`; OpenAPI/AsyncAPI/proto → `network/contracts/**` (+ dns face contracts under `network/dns/contracts/**`).
- Observability-style interior retarget: remapped `microservices/cloud-network{,-dns}/**` and `crates/oya-cloud-network-*` cites to in-tree destinations **only when the destination exists**; historical/missing assets (retired tenant_class artifact, absent ARCHITECTURE.md, etc.) keep legacy cites rather than inventing files.
- `network/dns/manifest.json`: capability/IP/contract path remaps to verified `network/**` homes; retired ungrounded `specs/master-plan-sequencing.json#…` SLO evidence fragment.
- Catalog rows verified present under `registry/catalog/network-*.yaml` (cited, not edited — outside `network/**`).

## Stale refs found (next gaps, ordered)

1. **Historical audit/tenant_class cites** — residual `microservices/cloud-network*/retired tenant_class adoption artifact` and absolute `/Users/jasonlee/oyatie/microservices/...` inventory lines in Wave-15 coherence/parity docs. Destinations do not exist in-tree; leave until a shrink rail deletes the dump narrative or a replacement artifact lands.
2. **`network/dns/manifest.json` shape** — still `microservice: cloud-network-dns` (absorbed residual). Full capability-root reshape of the nested face is a follow-up interior slice; root already owns the eight crates.
3. **OpenSLO scaffolds** — capability-root `slos[]` cites the two ADR-0348 autosharding-event YAML files under `network/observability/slos/`. Create-API / data-plane SLIs remain under `slo_exemption` until measured.
4. **Dual OpenAPI homes** — `network/contracts/openapi/cloud/cloud-network-*-v1.yaml` vs root `contracts/openapi/cloud/cloud-network-*-v1.yaml` (if present). Root copies are outside this envelope; converge on a contracts-aware rail.

## Out of envelope (do not touch from `integ/network`)

- `specs/**`, `Cargo.lock`, registry/catalog edits, merge/land.
- Shrink-only deletes of residual `microservices/*` or `crates/oya-cloud-network-*` mirrors — owning shrink rails only.
