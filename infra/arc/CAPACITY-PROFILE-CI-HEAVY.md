# RETIRED — CI-heavy Talos + ARC capacity profile

**Status: RETIRED (2026-08-11).** The custom ARC `oya-arm64` / live-postgres runner
fleets are decommissioned (`maxRunners: 0`). Do not size Talos workers for ARC
concurrency.

Merge plane: GitHub-hosted `ubuntu-latest`. Soft multi-arch: hosted
`ubuntu-24.04-arm` / windows / macos. Lab CAS: founder laptop NativeLink (+ tunnel).

Historical host/vCPU/RAM tables and apply checklists are provenance only (Git
history). Live decommission: [`README.md`](./README.md) § Founder live-ops checklist.
