# Purge receipt — `cloud/cloud-os/**` (2026-08-11)

| Field | Value |
|-------|-------|
| predicate | (a) migrated to forever `os/` (Wave-2 absorb `#1926` + `#1643`) — dual-home residual; forever crates advanced; no unique `.rs` basename only-in-source |
| source deleted | `cloud/cloud-os/**` (4 dual-home crate roots + `manifest.json`, 57 paths) |
| forever | `os/core/{cluster-mgmt,kubernetes,secrets,trustd}-domain/` + `os/manifest.json` + absorb receipt |
| **not** purged | `cloud/cloud-kernel/**` — unique kuberos bytes still only here; `#1659` was Asterinas ABI absorb only; disposition forbids delete until S5 rehome |
| associated debt | **Resolved in this PR (#1938):** root `Cargo.toml` `cloud/cloud-os/crates/oya-*` glob removed; four `registry/catalog/oya-cloud-os-*-domain.yaml` rows retired; hub rewrites landed (`capability-registry` / `masterplan` / `tier` / CI facade policies → `os`); `os/manifest.json` deleted crate IDs re-anchored to durable `os-*-domain`. **Genuinely remaining:** doc-axis legacy naming — the non-deleted `os/manifest.json` bounded-context rows still cite pre-rename `oya-cloud-os-*` IDs (pre-existing drift, predates the purge; tracked for a future rename pass), plus the staged `cloud/cloud-kernel` → `kernel/` rehome (`S4`/`S5`). |
| supersedes | Closed `#1839` which also burned `cloud-kernel` without forever home — that burn must not return |

