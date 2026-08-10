# ci/ reorg drain notes (`integ/ci`)

## Completed (this rail)

- Slice: absorb `oya/ci-controller/**` → `ci/controller/**` (kernel, adapters, app, iac).
  Crate rename: `oya_ci_controller_*` → `ci_controller_*`; workspace members added under `ci/controller/*`.

## Next gaps (ordered)

1. **Facade wiring** — register `ci/controller` in Buck graph + generated faces.
2. **Shrink-only burn** — delete `oya/ci-controller/**` on `integ/oya` after verify.
3. **Webhook gateway** — `oya/ci-webhook-gateway` rehome (separate slice).

## Out of envelope

- `oya/ci-controller/**` deletes — shrink-only on `integ/oya`.
- `ci/facade/**` gate policy changes unless envelope-bounded.
