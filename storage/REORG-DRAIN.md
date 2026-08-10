# storage/ reorg drain notes (`integ/storage`)

## Ownership

- **Forever home:** `storage/**` (this rail).
- **Writes this tip:** capability-root `storage/manifest.json` enrichment + this drain file only.
- **Absorbed product faces (still nested):** `storage/drive/`, `storage/imaging/`, `storage/recordings/` (registry absorb list; imaging is retained until app/healthcare move).

## Completed (this rail)

- Seat A interior prep: rewrite `storage/manifest.json` to capability-root shape (messaging@`2d1c81693` pattern): `capability` key, registry stratum **S2**, verified `storage/**` crate/contract/capability paths, eight-crate accounting + absorbed-service pointers.
- Path hygiene on the **root** manifest: dropped missing `evidence/multispectrum/cs-cloud-storage-foundation-20260523.json`, comma-joined dual adapter `file` cite, and ungrounded `specs/master-plan-sequencing.json#…` SLO evidence fragment.
- Catalog rows verified present under `registry/catalog/storage-*.yaml` (cited, not edited — outside `storage/**`).

## Stale refs found (next gaps, ordered)

1. **`storage/drive/manifest.json`** — 21 missing path cites. Remap `microservices/drive/**` → `storage/drive/**` where the tree already holds the file (capabilities, contracts). IP markdown cites (`IP-001`…`IP-015`) and bare `runbooks/dr-failover.md` remain missing or need `storage/drive/runbooks/…` rebasing.
2. **`storage/recordings/manifest.json`** — 21 missing path cites. Same class: `microservices/recordings/**` → `storage/recordings/**` for capabilities/contracts; IP markdown + bare runbook paths still drain.
3. **`storage/imaging/manifest.json`** — 5 missing path cites (`microservices/healthcare-integration/…`, bare `contracts/*.yaml`, bare `runbooks/imaging-vna-failover.md`). Prefer `storage/imaging/**` locals already in-tree.
4. **OpenSLO scaffolds** — 36 files under `storage/observability/slos/` exist but are **not** claimed by the capability-root `slos: []` / `slo_exemption` (no measured foundation SLI). Decide per product-face which YAML become live vs delete/retire.
5. **Dual OpenAPI homes** — `storage/contracts/openapi/cloud/cloud-storage-*-v1.yaml` vs root `contracts/openapi/cloud/cloud-storage-*-v1.yaml` (files differ). Root copies are outside this envelope; converge or delete the non-authority copy on a contracts-aware rail.
6. **Missing evidence blob** — `evidence/multispectrum/cs-cloud-storage-foundation-20260523.json` is absent from the tree; do not re-cite until a real evidence packet lands.

## Out of envelope (do not touch from `integ/storage`)

- `specs/**`, `Cargo.lock`, registry/catalog edits, merge/land.
- `app/healthcare` imaging relocate (Batch-5 move-plan) — separate rail.
- Shrink-only deletes of any residual `microservices/*` or `oya/*` mirrors — owning shrink rails only.
