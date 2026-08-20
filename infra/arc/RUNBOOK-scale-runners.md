# RETIRED — Runbook: scale general ARC runners

**Status: RETIRED (2026-08-11).** Do not raise `maxRunners` on `oya-arm64` or
`oya-live-postgres-arm64`. Both scale sets are tip-declared at `maxRunners: 0`.
Merge CI is GitHub-hosted; soft multi-arch is hosted; lab CAS is laptop NativeLink.

This file is a **tombstone**. Historical apply steps (Talos UserVolumes, topology
spread, QEMU recreate) remain in Git history if archaeology is needed. For live
decommission steps see [`README.md`](./README.md) § Founder live-ops checklist.

**Agents do not apply** Talos patches, Helm upgrades, or Argo syncs.
