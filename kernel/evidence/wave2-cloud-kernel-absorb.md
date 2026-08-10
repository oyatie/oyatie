# Wave-2 residual receipt — cloud/cloud-kernel → kernel/

| Field | Value |
|---|---|
| judgment | #1659 residual: retain the manifest-only landing slice; defer the full kernel absorb |
| lane | `integ/kernel` envelope `kernel/**` |
| base | `origin/dev` @ `9a56538c74b1fce4d474869956dd278f7fe1981e` |
| source residual | `#1659@45c1e5860` — `kernel/manifest.json` only |
| forever | `kernel/manifest.json` + existing `kernel/OWNERS` |

## Absorbed this tip

| Piece | Action |
|---|---|
| `kernel/manifest.json` | Harvest the machine-readable kernel landing-zone manifest from the prior #1659 tip. |
| `kernel/OWNERS` | Already present on the durable home at the base; no overwrite. |

## Deferred

The full `kernel/core/**` framekernel source tree, harnesses, generated artifacts, and nested
workspace files remain out of this residual. They require a dedicated absorb that resolves the
bare-metal no-Buck boundary, renamed `kernel-*` crate-catalog registrations, and tests-host
`Cargo.lock` supply-chain declaration before they can be admitted.

No none-platform Buck placeholders are introduced by this slice.

## Elevate

1. A future `integ/kernel` full-absorb lane owns the deferred source and harness correction.
2. Hubs may consume `kernel/manifest.json` independently; this receipt does not claim source,
   runtime, or production readiness.

## Verify

```text
test -f kernel/manifest.json
test -f kernel/OWNERS
test -f kernel/evidence/wave2-cloud-kernel-absorb.md
```
