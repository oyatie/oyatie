# Purge receipt — `cloud/cloud-os/**` (2026-08-11)

| Field | Value |
|-------|-------|
| predicate | (a) migrated to forever `os/` (Wave-2 absorb `#1926` + `#1643`) — dual-home residual; forever crates advanced; no unique `.rs` basename only-in-source |
| source deleted | `cloud/cloud-os/**` (4 dual-home crate roots + `manifest.json`, 57 paths) |
| forever | `os/core/{cluster-mgmt,kubernetes,secrets,trustd}-domain/` + `os/manifest.json` + absorb receipt |
| **not** purged | `cloud/cloud-kernel/**` — unique kuberos bytes still only here; `#1659` was Asterinas ABI absorb only; disposition forbids delete until S5 rehome |
| associated debt | Root `Cargo.toml` still globs `cloud/cloud-os/crates/oya-*`; `registry/catalog/oya-cloud-os-*.yaml`; hubs (`capability-registry` / `masterplan` / `tier` / CI facade policies). **Audit+rewrite on sole-owner tips** after this land — presumed stale |
| supersedes | Closed `#1839` which also burned `cloud-kernel` without forever home — that burn must not return |

