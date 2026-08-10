# build/ reorg drain notes (`integ/build`)

## Completed (this rail)

- W0-B Slice 1: six-crate skeleton under `build/port-engine/` (landed on `dev` via #1642).
- W0-B Slice 2: neutral seam types extracted to `port-engine-api`; kernel depends on api.

## Next gaps (ordered)

1. **Slice 3+** — `port-engine-rust-ir`, adapter stubs, facade driver wiring (`port-engine-app`).
2. **Lock absorb** — `Cargo.lock` / root `Cargo.toml` workspace membership for new path deps
   blocked while #1643 is sole open lock tip; defer to dedicated hot-slot slice.
3. **Toolchains move** — `toolchains/` judgment → `build/toolchains/` (`ready_for_integ_build`);
   do not mass-move on port-engine slices unless trivial dual-home.

## Out of envelope (do not touch from `integ/build`)

- `specs/k8s-port/` — judgment pending; no rehome.
- `k8s/**` — separate integ rail.
