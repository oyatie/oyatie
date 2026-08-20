# Ops next — unlock more runners (human)

**Why:** Live pool is still ~1 busy `oya-arm64`; four PRs + trunk post-merge serialize.
**Git PR already open:** #1564 dual-worker `maxRunners=2` (merge when CI green, then apply).

## Immediate (before #1564 if queue is critical)

1. Confirm ARC scale-set `oya-arm64` is healthy (listener + controller pods Ready).
2. Confirm worker-1 has mounted user volume `u-ci-workspace-general` (~48Gi).
3. Do **not** blindly set `maxRunners>1` without dual-worker storage admission (#1564) — risk DiskPressure.
4. Optional: cancel **superseded** Actions runs on old PR heads to free slots.

## After #1564 squash-merges

Follow committed runbook: `infra/arc/RUNBOOK-scale-runners.md`

1. Confirm worker-1 **and** worker-2 mount `ci-workspace-general`.
2. Apply/sync `infra/arc/ci-workspace-storage.yaml` (nodePathMap general on both workers).
3. Apply/sync `infra/arc/runner-scale-set-arm64-values.yaml` (`maxRunners: 2` + anti-affinity).
4. Smoke: two concurrent general jobs → different hostnames; no DiskPressure.
5. Keep `oya-live-postgres-arm64` at maxRunners=1.

## Success signal

- `gh api repos/.../actions/runners` shows **≥2** online `oya-arm64*` (or ARC shows 2 registered).
- Multi-PR jobs: more than one general gate `in_progress` at once.
