# ci/ reorg drain notes (`integ/ci`)

## Completed (this rail)

- Slice: absorb `oya/ci-controller/**` → `ci/controller/**` (kernel, adapters, app, iac).
  Crate rename: `ci_controller_*` → `ci_controller_*`; workspace members added under `ci/controller/*`.
- **Hyperscaler encode (process-kit forever home):** `.grok/process-kit/**` → `ci/process-kit/**` (`//ci/process-kit:…`);
  daemon-hotset + perimeter → `ci/facade/harness/**`. BAN agent-dotdirs as forever homes.
  Ephemeral `.grok/mm-runs` / `.grok/memory` left in place (not forever policy).

## Next gaps (ordered)

1. **Profile A tip-entitlement encode (next commit):** in `domain-stabilize.yml` /
   `presubmit` dual-emit path — when `tip_class` is `idle` **OR** preflight
   receipt is missing, **skip** Profile A expensive jobs (absence = skip, not CODE
   fail). Documented in workflow headers; wire job `if:` in a follow-on commit.
2. **Facade wiring** — register `ci/controller` in Buck graph + generated faces.
3. **Shrink-only burn** — delete `oya/ci-controller/**` on `integ/oya` after verify.
4. **Webhook gateway leftover husk** — crates already live in `ci/{core,adapters,facade,ports}/webhook-gateway-*`. Product metadata (manifest, iac, policy, contracts, slos) drained to `ci/webhook-gateway/` (not a new `ci/facade` gate). `oya/ci-webhook-gateway/` is gone.
5. **Residual `.grok/` harness** (lenses, model-routing, drive, …) — judge forever
   homes separately; do not mass-delete `.grok/` in the process-kit wave.

## Out of envelope

- `oya/ci-controller/**` deletes — shrink-only on `integ/oya`.
- `ci/facade/**` gate policy changes unless envelope-bounded.
- Mass-delete of entire `.grok/` tree — banned this wave.
