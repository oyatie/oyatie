# Wave-2 absorb receipt — cloud/cloud-os residual → os/

| Field | Value |
|-------|-------|
| founder_override | ACK 2026-08-10 — #1643 land accepted with findings still tracked (`oyatie-45t0` / #1925); cloud-os PARK LIFTED |
| judgment | REORG-DRAIN Wave-2: `cloud/cloud-os/` → forever `os/`; land after `#1643` |
| lane | `integ/os` envelope `os/**` |
| base | `origin/dev` @ `9a56538c74b1fce4d474869956dd278f7fe1981e` (#1643 squash) |
| source residual | `cloud/cloud-os/manifest.json` + dual-home crate OWNERS |
| forever | `os/manifest.json` + `os/core/{cluster-mgmt,kubernetes,secrets,trustd}-domain/` |

## Absorbed this tip

| Piece | Action |
|---|---|
| `os/manifest.json` | Harvest from `#1607@83a12f532` (`destination_path` / `owned_root` → `os/`) |
| `os/core/cluster-mgmt-domain/OWNERS` | Absorb missing dual-home OWNERS |
| `os/core/kubernetes-domain/OWNERS` | Absorb missing dual-home OWNERS |
| `os/core/secrets-domain/OWNERS` | Absorb missing dual-home OWNERS |
| `os/core/trustd-domain/OWNERS` | Already present on forever home — no overwrite |

## Chesterton (do not clobber #1643)

Forever crate bodies under `os/core/{cluster-mgmt,kubernetes,secrets,trustd}-domain/**` already advanced by `#1643` land. Dual-home bytes under `cloud/cloud-os/crates/oya-cloud-os-*` still drift (BUCK/Cargo.toml/src) — **not** copied onto forever homes. Delete of dual-home owned by `integ/cloud` after this receipt.

## Findings still open

`oyatie-45t0` / #1925 (F-PR5-06 / observation≠APPROVE on #1643) — **this absorb does not close them**.

## Elevate (out of envelope)

1. **integ/cloud** — delete drained `cloud/cloud-os/**` after this absorb receipt.
2. **Hubs** (capability-registry / masterplan / tier / automation excludes / crate-registration / slo-coverage) — tip-free `integ/specs` harvest from `#1607` only; **no hubs/NO_RAIL on this lane**.
3. No merge of `#1644` / `#1661` from this absorb.

## Verify

```
test -f os/manifest.json
test -f os/core/cluster-mgmt-domain/OWNERS
test -f os/core/kubernetes-domain/OWNERS
test -f os/core/secrets-domain/OWNERS
test -f os/core/trustd-domain/OWNERS
test -d os/core/cluster-mgmt-domain && test -d os/ports/kernel-abi
```
