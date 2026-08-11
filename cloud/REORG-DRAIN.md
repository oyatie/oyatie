# `cloud/` REORG-DRAIN status

## Purged (migrated)

- **`cloud/cloud-os/**`** — deleted 2026-08-11 after dest-verify on `os/`. Receipt: `cloud/evidence/purge-cloud-os-20260811.md`.

## Keep until rehome (unique / not migrated)

- **`cloud/cloud-kernel/**`** — bare-metal kuberos workspace. Durable home `kernel/` (staged S4). `#1659` landed Asterinas ABI only — **not** a kuberos absorb. **BAN** delete until zero-crate residual (S5). See `evidence/reorg/rr-cloud-kernel-disposition-20260806.md`.

## Associated cite debt

Pointers to `cloud/cloud-os` in hubs/registry/CI/Cargo membership need sole-owner rewrite after this purge lands.
