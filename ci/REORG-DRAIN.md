# ci/ reorg drain notes (`integ/ci`)

## Completed (this rail)

- Slice: absorb `oya/ci-controller/**` → `ci/controller/**` (kernel, adapters, app, iac).
  Crate rename: `oya_ci_controller_*` → `ci_controller_*`; workspace members added under `ci/controller/*`.
- **Hyperscaler encode (process-kit forever home):** retired agent-local process residue → `ci/process-kit/**` (`//ci/process-kit:…`);
  daemon-hotset + perimeter → `ci/facade/harness/**`. BAN agent-dotdirs as forever homes.

## Next gaps (ordered)

1. **Profile A tip-entitlement encode (next commit):** in `domain-stabilize.yml` /
   `oya-ci-required` dual-emit path — when `tip_class` is `idle` **OR** preflight
   receipt is missing, **skip** Profile A expensive jobs (absence = skip, not CODE
   fail). Documented in workflow headers; wire job `if:` in a follow-on commit.
2. **Facade wiring** — register `ci/controller` in Buck graph + generated faces.
3. **Shrink-only burn** — delete `oya/ci-controller/**` on `integ/oya` after verify.
4. **Webhook gateway** — `oya/ci-webhook-gateway` rehome (separate slice).

## Out of envelope

- `oya/ci-controller/**` deletes — shrink-only on `integ/oya`.
- `ci/facade/**` gate policy changes unless envelope-bounded.
- Agent-runtime dot-directories are ignored and untracked; do not recreate them as project
  authority.
