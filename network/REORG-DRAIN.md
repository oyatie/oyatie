# network/ reorg drain notes (`integ/network`)

## Ownership

- **Forever home:** `network/**` (this rail).
- **Writes this tip:** capability-root `network/manifest.json` enrichment + this drain file only.
- **Absorbed product face (still nested):** `network/dns/` (registry absorb list).

## Completed (this rail)

- Seat A interior prep: rewrite `network/manifest.json` to capability-root shape (messaging/storage/tenancy pattern): `capability` key, registry stratum **S1**, verified `network/**` crate/contract/capability paths, eight-crate accounting + absorbed dns pointer.
- Path hygiene on the **root** manifest: remapped stale `crates/oya-cloud-network-*` cites → live `network/{core,ports,adapters}/**`; OpenAPI cites → `network/contracts/openapi/cloud/**`; retired ungrounded `specs/master-plan-sequencing.json#…` SLO evidence fragment.
- Catalog rows verified present under `registry/catalog/network-*.yaml` (cited, not edited — outside `network/**`).

## Stale refs found (next gaps, ordered)

1. **`network/dns/manifest.json`** — nested absorb face may still carry pre-move path cites; remap in a follow-up interior slice.
2. **OpenSLO scaffolds** — files under `network/observability/slos/` exist but are **not** claimed by capability-root `slos: []` / `slo_exemption` (no measured foundation SLI). Decide which YAML become live vs retire.
3. **Dual OpenAPI homes** — `network/contracts/openapi/cloud/cloud-network-*-v1.yaml` vs root `contracts/openapi/cloud/cloud-network-*-v1.yaml` (if present). Root copies are outside this envelope; converge on a contracts-aware rail.

## Out of envelope (do not touch from `integ/network`)

- `specs/**`, `Cargo.lock`, registry/catalog edits, merge/land.
- Shrink-only deletes of residual `microservices/*` or `crates/oya-cloud-network-*` mirrors — owning shrink rails only.
