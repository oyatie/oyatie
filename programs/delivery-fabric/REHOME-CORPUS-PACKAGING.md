# Residual rehome → corpus packaging (fail-closed)

**Not merge authority.** Pattern proven on #1583 observability / #1584 audit.

## Defect class

Residual dual-home burn-down rehomes YAML from `oya/*` / `cloud/*` into a durable capability root. If that root has **no** buck package owning `**/*.yaml`, `corpus-index-coverage` spikes **unpackaged** and `affected-set` fails.

## Fix (each dual residual PR)

1. Add `{durable_home}/BUCK` mirroring `oya/BUCK`:
   - `corpus_yaml_facts_shards(srcs=glob(["**/*.yaml","**/*.yml"]), shard_size=256)`
   - optional live-service / proto filegroups
2. Run:
   - `cargo test -p ci-corpus-index-coverage --test corpus_index_coverage`
   - product-protocol suite if red
3. Re-anchor **measured shrink-only** in:
   - `ci/facade/corpus-index-coverage/corpus-index-coverage-policy.json`
   - product-protocol expected_total / live_count when manifests leave `oya/`+`cloud/`
4. Never invent ceilings; never raise floors above measured.

## Parallel fleet note

Many dual residual PRs edit the same policy JSON with **different** measured numbers. Land one → restack siblings and re-measure. Do not thrash tip for GHA queue.

## Split-brain exception

`ci` and `intelligence` crate multi-homes need **mixed/move** leaves — packaging alone does not close split_brain.
