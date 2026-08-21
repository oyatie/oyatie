# RETIRED — ARC workspace / runner fleet (2026-08-11)

**Status: RETIRED.** Custom Actions Runner Controller scale sets `oya-arm64` and
`oya-live-postgres-arm64` are decommissioned. Merge CI is **GitHub-hosted
`ubuntu-latest` only** (`oya-ci-required`). Soft multi-arch stays on GitHub-hosted
`ubuntu-24.04-arm` / `windows-latest` / `macos-latest`. Lab CAS is the founder
**laptop NativeLink** (+ tunnel) — not ARC overflow.

Git declarations under this directory remain as **tombstones** (`maxRunners: 0`)
so Argo/Helm can sync scale-to-zero before Applications are removed. Do not
schedule new jobs on these labels. Keep Git history; do not resurrect capacity
without a new ADR.

Historical capacity / workspace prose that previously lived here is provenance
only (see Git history and `docs/adr-archive/ADR-0630-…`).

## Founder live-ops checklist (cluster unreachable from agent)

Agent tip commits set `maxRunners: 0` in:

- `infra/arc/runner-scale-set-arm64-values.yaml`
- `infra/arc/runner-scale-set-live-postgres-arm64-values.yaml`

When you have `kubectl` / Helm / Argo access to `oya-talos`:

1. **Sync** Argo Applications `oya-arm64` and `oya-live-postgres-arm64` (or
   `helm upgrade` the same values) so listeners observe `maxRunners: 0`.
2. **Wait** until no runner Pods remain:  
   `kubectl -n arc-runners get pods,ephemeralrunnersets,autoscalingrunnersets -o wide`
3. **Confirm** workspace PVCs drain:  
   `kubectl -n arc-runners get pvc`
4. **Remove** (or disable) Argo Application entries in
   `infra/gitops/values.yaml` for `arc`, `oya-arm64`,
   `oya-live-postgres-arm64`, `oya-live-postgres-network-policy` after a reviewed
   window — tip keeps them listed so first sync can apply scale-to-zero.
5. **Clear** any forced repo runner labels / webhook routing that still name
   `oya-arm64` or `oya-live-postgres-arm64` (GitHub Settings → Actions → Runners).
6. **Optional:** leave `arc-systems` controller until no scale sets remain; then
   remove the `arc` Application and controller namespace.
7. Do **not** flip warm CAS / NativeLink as part of this retirement.

Rollback of merge CI does **not** require ARC — hosted `ubuntu-latest` remains
the binding plane.
